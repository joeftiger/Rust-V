use crate::integrator::Integrator;
use crate::new::config::Config;
use crate::new::render_job::RenderJob;
use crate::new::sensor::sensor_tile::SensorTile;
use crate::new::sensor::Sensor;
use crate::sampler::Sampler;
use crate::scene::Scene;
use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct Renderer {
    scene: Arc<Scene>,
    sampler: Arc<dyn Sampler>,
    integrator: Arc<dyn Integrator>,
    sensor: Arc<Sensor>,
    config: Config,
    progress: Arc<AtomicUsize>,
    pub progress_bar: Arc<Mutex<ProgressBar>>,
}

impl Renderer {
    pub fn new(
        scene: Arc<Scene>,
        sampler: Arc<dyn Sampler>,
        integrator: Arc<dyn Integrator>,
        config: Config,
    ) -> Self {
        let clone = config.clone();
        let sensor = Sensor::new(
            clone.resolution,
            clone.filename,
            clone.bounds,
            clone.block_size,
        );

        let progress = Arc::new(AtomicUsize::new(0));

        let progress_bar = {
            let bar = ProgressBar::new(0);
            bar.set_style(ProgressStyle::default_bar().template(
                "{msg}\n[{elapsed_precise} elapsed] {wide_bar:.cyan/white} {percent}% [{eta_precise} remaining]\nrender-blocks: {per_sec}",
            ));
            Arc::new(Mutex::new(bar))
        };

        Self {
            scene,
            sampler,
            integrator,
            config,
            sensor: Arc::new(sensor),
            progress,
            progress_bar,
        }
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

    pub fn render(&mut self) -> RenderJob<()> {
        // reset progress bar
        {
            let bar = self.progress_bar.lock().expect("Progress bar poisoned");
            bar.set_length((self.sensor.num_tiles() * self.config.passes as usize) as u64);
            bar.reset();
        }

        let mut handles = Vec::with_capacity(self.config.threads as usize);
        let should_stop = Arc::new(AtomicBool::new(false));
        let frames = Arc::new(AtomicIsize::new(0));

        let tiles = self.sensor.num_tiles();

        for i in 0..self.config.threads {
            let this = self.clone();
            let this_should_stop = should_stop.clone();
            let this_frames = frames.clone();

            let handle = thread::Builder::new()
                .name(format!("Render thread {}", i))
                .stack_size(16 * 1024 * 1024)
                .spawn(move || loop {
                    if this_should_stop.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Some((progress, lock)) = this.clone().get_progress_and_next_tile() {
                        if progress % tiles == 0 {
                            let frame = this_frames.fetch_add(1, Ordering::Relaxed);
                            let bar = this.progress_bar.lock().expect("Progress bar poisoned");
                            bar.set_message(format!("Frames rendered: {}", frame).as_str());
                        }

                        let mut tile = lock.lock().expect("SensorTile is poisoned");

                        for px in &mut tile.pixels {
                            let primary_ray = this.scene.camera.primary_ray(px.position);
                            this.integrator.integrate(
                                px,
                                &this.scene,
                                &primary_ray,
                                &*this.sampler,
                            );
                        }

                        this.progress_bar
                            .lock()
                            .expect("Progress bar is poisoned")
                            .inc(1);
                    } else {
                        let bar = this.progress_bar.lock().expect("Progress bar poisoned");
                        bar.println(format!(
                            "Render thread {} has no more blocks to do. Stopping...",
                            i
                        ));

                        break;
                    }
                })
                .unwrap_or_else(|_| panic!("Could not spawn render thread {}", i));

            handles.push(handle);
        }

        RenderJob::new(should_stop, handles)
    }

    //noinspection DuplicatedCode
    pub fn get_image_u8(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let res = self.sensor.bounds.to_range();
        let mut buffer = ImageBuffer::new(res.x, res.y);

        for lock in &self.sensor.tiles {
            let tile = lock.lock().expect("SensorTile is poisoned");

            for px in &tile.pixels {
                let (x, y) = (px.position.x, px.position.y);

                buffer.put_pixel(x, y, Rgb::from(px.average));
            }
        }

        buffer
    }

    //noinspection DuplicatedCode
    pub fn get_image_u16(&self) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let res = self.sensor.bounds.to_range();
        let mut buffer = ImageBuffer::new(res.x, res.y);

        for lock in &self.sensor.tiles {
            let tile = lock.lock().expect("SensorTile is poisoned");

            for px in &tile.pixels {
                let (x, y) = (px.position.x, px.position.y);

                buffer.put_pixel(x, y, Rgb::from(px.average));
            }
        }

        buffer
    }
}
