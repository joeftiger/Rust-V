use ultraviolet::UVec2;

/// Teh render configuration stores all parameters relevant for the renderer.
#[derive(Debug, Copy, Clone)]
pub struct RenderConfig {
    pub resolution: UVec2,
    pub depth: u32,
    pub passes: u32,
    pub block_size: u32,
    pub threads: u32,
}

impl RenderConfig {
    /// Creates a new render configuration.
    ///
    /// # Arguments
    /// * `width` - The resolution in width
    /// * `height` - The resolution in height
    /// * `depth` - The render depth (for integrator)
    /// * `passes` - The number of passes/pixel
    /// * `block_size` - The block size for each thread
    /// * `threads` - The number of threads
    ///
    /// # Returns
    /// * Self
    pub fn new(
        width: u32,
        height: u32,
        depth: u32,
        passes: u32,
        block_size: u32,
        threads: u32,
    ) -> Self {
        Self {
            resolution: UVec2::new(width, height),
            depth,
            passes,
            block_size,
            threads,
        }
    }
}
