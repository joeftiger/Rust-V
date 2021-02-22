use crate::sampler::pixel_samplers::PixelSampler;
use ultraviolet::Vec2;

pub struct NoopSampler;

impl PixelSampler for NoopSampler {
    fn get(_: &Vec2) -> Vec2 {
        Vec2::broadcast(0.5)
    }
}
