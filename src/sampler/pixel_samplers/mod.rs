// mod super_sampling;
// pub use super_sampling::*;

use crate::mc::{sample_unit_disk, sample_unit_disk_concentric};
use definitions::Vector2;
#[cfg(not(feature = "f64"))]
use fastrand::f32 as rand;
#[cfg(feature = "f64")]
use fastrand::f64 as rand;
use serde::{Deserialize, Serialize};
use ultraviolet::UVec2;

#[derive(Serialize, Deserialize)]
pub enum PixelSamplerType {
    /// No-operation sampler, always returns `Vector2 { x: 0.5, y: 0.5 }`
    NoOp,
    /// Generates random samples
    Random,
    Concentric,
    NonConcentric,
    // UniformSuperSampling(UniformSuperSampling),
}

#[inline]
fn rand_vec() -> Vector2 {
    Vector2::new(rand(), rand())
}

impl PixelSampler for PixelSamplerType {
    fn sample(&self, _pixel: UVec2) -> Vector2 {
        match self {
            PixelSamplerType::NoOp => Vector2::broadcast(0.5),
            PixelSamplerType::Random => rand_vec(),
            PixelSamplerType::Concentric => {
                sample_unit_disk_concentric(rand_vec()) + Vector2::broadcast(0.5)
            }
            PixelSamplerType::NonConcentric => {
                sample_unit_disk(rand_vec()) + Vector2::broadcast(0.5)
            }
            // PixelSamplerType::UniformSuperSampling(s) => s.sample(pixel),
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
