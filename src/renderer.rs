use crate::camera::Camera;
use crate::configuration::Configuration;
use crate::grid::GridBlock;
use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::Spectrum;
use bitflags::_core::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use ultraviolet::UVec2;

/// # Summary
/// A render job consists of thread handles.
/// It can be stopped or joined at the end of execution.
pub struct RenderJob<T> {
    should_stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<T>>,
}

impl<T> RenderJob<T> {
    /// # Summary
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

    /// # Summary
    /// Sets a flag to stop thread executions and joins the threads afterwards.
    ///
    /// # Returns
    /// * The result of the stopping operation
    pub fn stop(self) -> thread::Result<()> {
        self.should_stop.store(true, Ordering::Relaxed);
        self.join()
    }

    /// # Summary
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

/// # Summary
/// This struct is responsible to keep track of a pixel's color.
struct RenderPixel {
    pixel: UVec2,
    spectrum: Spectrum,
    samples: usize,
}

impl RenderPixel {
    /// # Summary
    /// Computes the average of this pixel.
    ///
    /// # Returns
    /// * Spectrum average
    fn average(&self) -> Spectrum {
        match self.samples {
            0 => self.spectrum,
            _ => self.spectrum / self.samples as f32,
        }
    }
}

impl From<UVec2> for RenderPixel {
    /// # Summary
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
            spectrum: Spectrum::new_const(0.0),
            samples: 0,
        }
    }
}

/// # Summary
/// A render block contains pixels. It is used to split a rendering into multiple blocks for a
/// thread to do computations upon.
struct RenderBlock {
    pixels: Vec<RenderPixel>,
}

impl From<GridBlock> for RenderBlock {
    /// #Summary
    /// Creates a new render block from the given grid block.
    ///
    /// # Arguments
    /// * `block` - The grid block to convert
    ///
    /// # Returns
    /// * Self
    fn from(block: GridBlock) -> Self {
        let pixels = block.prod().iter().map(|v| RenderPixel::from(*v)).collect();

        Self { pixels }
    }
}

/// WIP
pub struct Renderer<'a> {
    scene: &'a Scene<'a>,
    camera: &'a dyn Camera,
    sampler: &'a dyn Sampler,
    integrator: &'a dyn Integrator,
    render_blocks: Arc<Vec<RwLock<RenderBlock>>>,
    progress: Arc<AtomicUsize>,
    configuration: Arc<Configuration>,
}
