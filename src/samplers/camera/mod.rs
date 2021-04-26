use crate::mc::{sample_unit_disk, sample_unit_disk_concentric};
use definitions::Vector2;
#[cfg(not(feature = "f64"))]
use fastrand::f32 as rand;
#[cfg(feature = "f64")]
use fastrand::f64 as rand;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum CameraSampler {
    /// No-operation sampler, always returns `Vector2 { x: 0.5, y: 0.5 }`
    NoOp,
    /// Generates random samples
    Random,
    Concentric,
    NonConcentric,
}

#[inline]
fn rand_vec() -> Vector2 {
    Vector2::new(rand(), rand())
}

impl CameraSampler {
    /// Generates a new camera pixel anchor for a given pixel.
    ///
    /// # Arguments
    /// * `pixel` - The pixel we are sampling/generating for.
    ///
    /// # Returns
    /// * A camera pixel anchor in range `[0, 1)`
    #[inline]
    pub fn sample(&self) -> Vector2 {
        match self {
            CameraSampler::NoOp => Vector2::broadcast(0.5),
            CameraSampler::Random => rand_vec(),
            CameraSampler::Concentric => {
                sample_unit_disk_concentric(rand_vec()) + Vector2::broadcast(0.5)
            }
            CameraSampler::NonConcentric => sample_unit_disk(rand_vec()) + Vector2::broadcast(0.5),
        }
    }
}
