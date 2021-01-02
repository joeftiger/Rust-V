use crate::camera::Camera;
use crate::configuration::Configuration;
use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::Spectrum;
use bitflags::_core::sync::atomic::AtomicBool;
use color::Color;
use std::sync::atomic::Ordering;
use std::sync::Arc;
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
#[derive(Default)]
struct PixelStats {
    spectrum: Spectrum,
    samples: usize,
}

impl PixelStats {
    /// # Summary
    /// Computes the average of this pixel.
    ///
    /// # Returns
    /// * Spectrum average
    pub fn average(&self) -> Spectrum {
        match self.samples {
            0 => self.spectrum,
            _ => self.spectrum / self.samples as f32,
        }
    }
}

pub struct Renderer<'a> {
    scene: &'a Scene<'a>,
    camera: &'a dyn Camera,
    sampler: &'a dyn Sampler,
    integrator: &'a dyn Integrator,
    configuration: Configuration,
}
