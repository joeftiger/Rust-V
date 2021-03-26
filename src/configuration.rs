/// The render configuration stores all parameters relevant for the renderer.
#[derive(Debug, Copy, Clone)]
pub struct RenderConfig {
    pub depth: u32,
    pub passes: u32,
    pub block_size: u32,
    pub threads: u32,
}

impl RenderConfig {
    /// Creates a new render configuration.
    ///
    /// # Arguments
    /// * `depth` - The render depth (for integrator)
    /// * `passes` - The number of passes/pixel
    /// * `block_size` - The block size for each thread
    /// * `threads` - The number of threads
    ///
    /// # Returns
    /// * Self
    pub fn new(depth: u32, passes: u32, block_size: u32, threads: u32) -> Self {
        Self {
            depth,
            passes,
            block_size,
            threads,
        }
    }
}
