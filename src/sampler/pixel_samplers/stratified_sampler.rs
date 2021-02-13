use crate::sampler::pixel_samplers::PixelSampler;
use ultraviolet::{UVec2, Vec2};

/// A stratified sampler with uniform distribution
pub struct StratifiedSampler {
    sample_space: UVec2,
}
