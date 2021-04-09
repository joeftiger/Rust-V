use crate::new::sensor::bounds::UBounds2;
use ultraviolet::UVec2;

#[derive(Clone)]
pub struct Config {
    pub resolution: UVec2,
    pub filename: Option<String>,
    pub bounds: UBounds2,
    pub block_size: UVec2,
    pub passes: u32,
    pub threads: u32,
}
