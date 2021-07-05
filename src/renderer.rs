use crate::camera::Camera;
use crate::config::Config;
use crate::integrator::Integrator;
use crate::samplers::Sampler;
use crate::scene::Scene;
use crate::sensor::bounds::UBounds2;
use crate::sensor::sensor_tile::SensorTile;
use crate::sensor::Sensor;
use crate::serialization::Serialization;
use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use ultraviolet::UVec2;

/// A render job consists of thread handles.
/// It can be stopped or joined at the end of execution.
pub struct RenderJob<T> {
    renderer: Renderer,
    should_stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<T>>,
}

impl<T> RenderJob<T> {
    /// Creates a new render job.
    ///
    /// # Arguments
    /// * `should_stop` - An atomic boolean to indicate stopping behaviour.
    ///                   Should be watched by a renderer.
    /// * `handles` - The thread handles
    ///
    /// # Returns
    /// * Self
    pub fn new(
        renderer: Renderer,
        should_stop: Arc<AtomicBool>,
        handles: Vec<JoinHandle<T>>,
    ) -> Self {
        Self {
            renderer,
            should_stop,
            handles,
        }
    }

    /// Sets a flag to stop thread executions and joins the threads afterwards.
    ///
    /// # Returns
    /// * The result of the stopping operation
    pub fn stop(self) -> thread::Result<()> {
        self.should_stop.store(true, Ordering::Relaxed);
        self.join()
    }

    /// Waits for the thread handles to join.
    ///
    /// # Returns
    /// * The result of the joining operation
    pub fn join(self) -> thread::Result<()> {
        for handle in self.handles {
            handle.join()?;
        }

        self.renderer.progress_bar.lock().finish();

        Ok(())
    }
}

pub struct Renderer {
    scene: Arc<Scene>,
    camera: Box<dyn Camera>,
    sampler: Sampler,
    integrator: Box<dyn Integrator>,
    sensor: Arc<Sensor>,
    config: Config,
    progress: Arc<AtomicUsize>,
    pub progress_bar: Arc<Mutex<ProgressBar>>,
}

impl Clone for Renderer {
    fn clone(&self) -> Self {
        Self {
            scene: self.scene.clone(),
            camera: dyn_clone::clone_box(&*self.camera),
            sampler: self.sampler,
            integrator: dyn_clone::clone_box(&*self.integrator),
            sensor: self.sensor.clone(),
            config: self.config.clone(),
            progress: self.progress.clone(),
            progress_bar: self.progress_bar.clone(),
        }
    }
}

impl Renderer {
    pub fn new(serialization: Serialization) -> Self {
        let progress = Arc::new(AtomicUsize::new(0));
        let progress_bar = {
            let bar = ProgressBar::new(0);
            bar.set_style(ProgressStyle::default_bar().template(
                "{msg}\n[{elapsed_precise} elapsed] {wide_bar:.cyan/white} {percent}% [{eta_precise} remaining]\nrender-blocks: {per_sec}",
            ));
            Arc::new(Mutex::new(bar))
        };

        let mut scene = serialization.scene;
        scene.init();

        let camera = serialization.camera;

        let sampler = serialization.sampler;
        let integrator = serialization.integrator;

        let config = serialization.config.clone();
        let sensor = Sensor::new(
            camera.resolution(),
            config.filename,
            config
                .bounds
                .unwrap_or_else(|| UBounds2::from(camera.resolution())),
            config.block_size,
        );

        Self {
            scene: Arc::new(scene),
            camera,
            sampler,
            integrator,
            config: serialization.config,
            sensor: Arc::new(sensor),
            progress,
            progress_bar,
        }
    }

    pub fn filename(&self) -> &Option<String> {
        &self.sensor.filename
    }

    /// Returns the current progress. It will/should be in the range `[0, z]` for
    /// `z = render_blocks * passes`.
    ///
    /// # Returns
    /// * The current progress
    pub fn get_progress(&self) -> usize {
        self.progress.load(Ordering::Relaxed)
    }

    /// Returns whether the current progress is at/over the limit of `[0, z]` for
    /// `z = render_blocks * passes`.
    ///
    /// # Returns
    /// * Whether the render is done
    pub fn is_done(&self) -> bool {
        self.progress_out_of_range(self.get_progress())
    }

    /// Returns whether the given progress is at/over the limit of `[0, z]` for
    /// `z = render_blocks * passes`.
    ///
    /// # Returns
    /// * Whether the progress is at/over the limit
    fn progress_out_of_range(&self, progress: usize) -> bool {
        progress >= self.sensor.num_tiles() * self.config.passes as usize
    }

    fn get_progress_and_next_tile(&mut self) -> Option<(usize, &Mutex<SensorTile>)> {
        let index = self.progress.fetch_add(1, Ordering::Relaxed);

        if index < self.config.passes as usize * self.sensor.num_tiles() {
            Some((
                index,
                &self.sensor.tiles[index as usize % self.sensor.num_tiles()],
            ))
        } else {
            None
        }
    }

    #[inline]
    pub fn resolution(&self) -> UVec2 {
        self.camera.resolution()
    }

    pub fn render(&mut self) -> RenderJob<()> {
        // reset progress bar
        {
            let bar = self.progress_bar.lock();
            bar.set_length((self.sensor.num_tiles() * self.config.passes as usize) as u64);
            bar.reset();
        }

        let threads = self.config.threads.unwrap_or(num_cpus::get() as u32);

        let mut handles = Vec::with_capacity(threads as usize);
        let should_stop = Arc::new(AtomicBool::new(false));
        let frames = Arc::new(AtomicIsize::new(0));

        let tiles = self.sensor.num_tiles();

        for i in 0..threads {
            let this = self.clone();
            let this_should_stop = should_stop.clone();
            let this_frames = frames.clone();

            let handle = thread::Builder::new()
                .name(format!("Render thread {}", i))
                .stack_size(32 * 1024 * 1024)
                .spawn(move || loop {
                    if this_should_stop.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Some((progress, sensor)) = this.clone().get_progress_and_next_tile() {
                        if progress % tiles == 0 {
                            let frame = this_frames.fetch_add(1, Ordering::Relaxed);
                            this.progress_bar
                                .lock()
                                .set_message(format!("Frames rendered: {}", frame));
                        }

                        for px in &mut sensor.lock().pixels {
                            let primary_ray = this.camera.primary_ray(px.position);
                            this.integrator
                                .integrate(px, &this.scene, &primary_ray, this.sampler);
                        }

                        this.progress_bar.lock().inc(1);
                    } else {
                        break;
                    }
                })
                .unwrap_or_else(|_| panic!("Could not spawn render thread {}", i));

            handles.push(handle);
        }

        RenderJob::new(self.clone(), should_stop, handles)
    }

    //noinspection DuplicatedCode
    pub fn get_image_u8(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let bounds = self.sensor.bounds;
        let res = bounds.to_range();
        let mut buffer = ImageBuffer::new(res.x, res.y);

        for lock in &self.sensor.tiles {
            for px in &lock.lock().pixels {
                let (x, y) = (px.position.x - bounds.min.x, px.position.y - bounds.min.y);

                buffer.put_pixel(x, y, Rgb::from(px.average));
            }
        }

        buffer
    }

    //noinspection DuplicatedCode
    pub fn get_image_u16(&self) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let bounds = self.sensor.bounds;
        let res = bounds.to_range();
        let mut buffer = ImageBuffer::new(res.x, res.y);

        for lock in &self.sensor.tiles {
            for px in &lock.lock().pixels {
                let (x, y) = (px.position.x - bounds.min.x, px.position.y - bounds.min.y);

                buffer.put_pixel(x, y, Rgb::from(px.average));
            }
        }

        buffer
    }
}
