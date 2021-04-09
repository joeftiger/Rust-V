#![allow(unused_variables)]
#![allow(dead_code)]

// mod super_sampling;
// pub use super_sampling::*;

use definitions::{Float, Vector2};
use serde::{Deserialize, Serialize};
use ultraviolet::UVec2;

#[derive(Serialize, Deserialize)]
pub enum PixelSamplerType {
    /// No-operation sampler, always returns `Vector2 { x: 0.5, y: 0.5 }`
    NoOp,
    /// Generates random samples
    Random,
    // UniformSuperSampling(UniformSuperSampling),
}

impl PixelSampler for PixelSamplerType {
    fn sample(&self, pixel: UVec2) -> Vector2 {
        match self {
            PixelSamplerType::NoOp => Vector2::broadcast(0.5),
            PixelSamplerType::Random => {
                Vector2::new(fastrand::f64() as Float, fastrand::f64() as Float)
            } // PixelSamplerType::UniformSuperSampling(s) => s.sample(pixel),
        }
    }
}

pub trait PixelSampler {
    /// Generates a new camera pixel anchor for a given pixel.
    ///
    /// # Arguments
    /// * `pixel` - The pixel we are sampling/generating for.
    ///
    /// # Returns
    /// * A camera pixel anchor in range `[0, 1)`
    fn sample(&self, pixel: UVec2) -> Vector2;
}
