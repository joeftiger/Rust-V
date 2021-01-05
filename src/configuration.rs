use crate::renderer::Renderer;
use std::convert::TryInto;
use ultraviolet::UVec2;

/// # Summary
/// Teh render configuration stores all parameters relevant for the renderer.
#[derive(Debug, Copy, Clone)]
pub struct RenderConfig {
    pub resolution: UVec2,
    pub depth: u32,
    pub passes: u32,
    pub block_size: u32,
    pub threads: u32,
}
