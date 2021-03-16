use crate::camera::Camera;
use crate::configuration::RenderConfig;
use crate::grid::{Grid, GridBlock};
use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::Spectrum;
use color::Color;
use image::{ImageBuffer, Rgb};
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use ultraviolet::UVec2;

/// A render job consists of thread handles.
/// It can be stopped or joined at the end of execution.
pub struct RenderJob<T> {
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
    pub fn new(should_stop: Arc<AtomicBool>, handles: Vec<JoinHandle<T>>) -> Self {
        Self {
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

        Ok(())
    }
}

/// This struct is responsible to keep track of a pixel's color.
struct RenderPixel {
    pixel: UVec2,
    pub average: Spectrum,
    samples: usize,
}

impl RenderPixel {
    /// Adds the given spectrum and increases the sample size by 1.
    ///
    /// # Arguments
    /// * `spectrum` - The spectrum to add
    pub fn add(&mut self, spectrum: Spectrum) {
        let mut avg = self.average * self.samples as f32;
        avg += spectrum;
        self.samples += 1;
        avg /= self.samples as f32;

        self.average = avg;
    }
}

impl From<UVec2> for RenderPixel {
    /// Creates a new render pixel.
    ///
    /// # Arguments
    /// * `pixel` - The pixel coordinate
    ///
    /// # Returns
    /// * Self, with initial spectrum of `0.0` and initial samples of `0`.
    fn from(pixel: UVec2) -> Self {
        Self {
            pixel,
            average: Spectrum::new_const(0.0),
            samples: 0,
        }
    }
}

/// A render block contains pixels. It is used to split a rendering into multiple blocks for a
/// thread to do computations upon.
struct RenderBlock {
    pixels: Vec<RenderPixel>,
}

impl From<&GridBlock> for RenderBlock {
    /// #Summary
    /// Creates a new render block from the given grid block.
    ///
    /// # Arguments
    /// * `block` - The grid block to convert
    ///
    /// # Returns
    /// * Self
    fn from(block: &GridBlock) -> Self {
        let pixels = block.prod().iter().map(|v| RenderPixel::from(*v)).collect();

        Self { pixels }
    }
}

/// WIP
#[derive(Clone)]
pub struct Renderer {
    scene: Arc<Scene>,
    camera: Arc<dyn Camera>,
    sampler: Arc<dyn Sampler>,
    integrator: Arc<dyn Integrator>,
    config: RenderConfig,
    #[allow(clippy::rc_buffer)]
    render_blocks: Arc<Vec<RwLock<RenderBlock>>>,
    progress: Arc<AtomicU32>,
    pub progress_bar: Arc<Mutex<ProgressBar>>,
}

impl Renderer {
    /// Creates and initializes a new renderer with the given arguments.
    ///
    /// # Arguments
    /// * `scene` - The objects scene
    /// * `camera` - The camera to use
    /// * `sampler` - The random sampler
    /// * `integrator` - The scene integrator
    /// * `config` - The original config
    ///
    /// # Returns
    /// * An initialized renderer
    pub fn new(
        scene: Scene,
        camera: Arc<dyn Camera>,
        sampler: Arc<dyn Sampler>,
        integrator: Arc<dyn Integrator>,
        config: RenderConfig,
    ) -> Self {
        let render_blocks = {
            let grid = Grid::new(&config.resolution, config.block_size);
            let blocks = grid
                .blocks
                .iter()
                .map(|b| RwLock::new(RenderBlock::from(b)))
                .collect();

            Arc::new(blocks)
        };

        let progress = Arc::new(AtomicU32::default());

        let progress_bar = {
            let bar = ProgressBar::new(0);
            bar.set_style(ProgressStyle::default_bar().template(
                "[{elapsed} elapsed] {wide_bar:.cyan/white} {percent}% [{eta} remaining]",
            ));
            Arc::new(Mutex::new(bar))
        };

        Self {
            scene: Arc::new(scene),
            camera,
            sampler,
            integrator,
            config,
            render_blocks,
            progress,
            progress_bar,
        }
    }

    /// Returns the number of render blocks.
    ///
    /// # Returns
    /// * Number of render blocks
    pub fn get_num_blocks(&self) -> u32 {
        self.render_blocks.len() as u32
    }

    /// Returns the current progress. It will/should be in the range `[0, z]` for
    /// `z = render_blocks * depth`.
    ///
    /// # Returns
    /// * The current progress
    pub fn get_progress(&self) -> u32 {
        self.progress.load(Ordering::Relaxed)
    }

    /// Returns whether the current progress is at/over the limit of `[0, z]` for
    /// `z = render_blocks * depth`.
    ///
    /// # Returns
    /// * Whether the render is done
    pub fn is_done(&self) -> bool {
        self.progress_out_of_range(self.get_progress())
    }

    /// Returns whether the given progress is at/over the limit of `[0, z]` for
    /// `z = render_blocks * depth`.
    ///
    /// # Returns
    /// * Whether the progress is at/over the limit
    fn progress_out_of_range(&self, progress: u32) -> bool {
        progress >= self.get_num_blocks() * self.config.passes
    }

    /// Renders the given pixel with this renderer's `sampler` and `integrator`.
    ///
    /// # Constraints
    /// * `pixel` - Should be within the camera's resolution.
    ///
    /// # Returns
    /// * The computed pixel spectrum
    fn render_pixel(&self, pixel: UVec2) -> Spectrum {
        debug_assert!(pixel == pixel.min_by_component(self.config.resolution));

        let sample = self.sampler.get_2d();
        let ray = self.camera.primary_ray(&pixel, &sample);

        self.integrator.integrate(&self.scene, &ray, &*self.sampler)
    }

    /// Fetch-adds-1 the current progress and returns the associated render block.
    /// If the current progress was already at/over the limit of `[0, z]` for
    ///`z = render_blocks * depth`, `None` will be returned.
    ///
    /// # Returns
    /// * Next block if available
    fn try_get_next_block(&mut self) -> Option<&RwLock<RenderBlock>> {
        let index = self.progress.fetch_add(1, Ordering::Relaxed);

        if self.progress_out_of_range(index) {
            None
        } else {
            Some(&self.render_blocks[(index % self.get_num_blocks()) as usize])
        }
    }

    /// Starts the rendering progress.
    ///
    /// `config.threads` threads will be allocated for this.
    /// This function returns a render job that can be stopped or
    /// waited for to end.
    ///
    /// # Returns
    /// * The render job
    pub fn render(&mut self) -> RenderJob<()> {
        assert!(!self.is_done());

        // reset progress bar
        {
            let bar = self.progress_bar.lock().expect("Progress bar poisoned");
            bar.set_length((self.get_num_blocks() * self.config.passes) as u64);
            bar.reset();
        }

        let num_threads = self.config.threads;
        let mut handles = Vec::with_capacity(num_threads as usize);

        let should_stop = Arc::new(AtomicBool::default());
        for i in 0..num_threads {
            let this = self.clone();
            let this_should_stop = should_stop.clone();

            // each thread loops and gets the next block unless it should stop or has finished.
            let handle = thread::Builder::new()
                .name(format!("Render thread {}", i))
                .stack_size(16 * 1024 * 1024)
                .spawn(move || loop {
                    if this_should_stop.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Some(block) = this.clone().try_get_next_block() {
                        let mut lock = block.write().expect("Block is poisoned");

                        lock.pixels.iter_mut().for_each(|px| {
                            let spectrum = this.render_pixel(px.pixel);
                            px.add(spectrum);
                        });

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
                .expect(&*format!("Could not spawn thread {}", i));

            handles.push(handle);
        }

        RenderJob::new(should_stop, handles)
    }

    //noinspection DuplicatedCode
    pub fn get_image_u8(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let res = self.config.resolution;
        let mut buffer = ImageBuffer::new(res.x, res.y);

        self.render_blocks.iter().for_each(|block| {
            let lock = block.read().expect("Block is poisoned");

            lock.pixels
                .iter()
                .for_each(|px| buffer.put_pixel(px.pixel.x, px.pixel.y, px.average.into()));
        });

        buffer
    }

    //noinspection DuplicatedCode
    pub fn get_image_u16(&self) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let res = self.config.resolution;
        let mut buffer = ImageBuffer::new(res.x, res.y);

        self.render_blocks.iter().for_each(|block| {
            let lock = block.read().expect("Block is poisoned");

            lock.pixels
                .iter()
                .for_each(|px| buffer.put_pixel(px.pixel.x, px.pixel.y, px.average.into()));
        });

        buffer
    }
}
