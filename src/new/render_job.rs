use crate::new::renderer::Renderer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

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

        self.renderer
            .progress_bar
            .lock()
            .expect("Progress bar poisoned")
            .finish();

        Ok(())
    }
}
