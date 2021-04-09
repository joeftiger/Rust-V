pub mod pixel_samplers;

pub use noop_sampler::NoOpSampler;
pub use random_sampler::RandomSampler;

use crate::debug_utils::within_01;
use definitions::{Float, Vector2, Vector3};
use utility::floats::FloatExt;

mod noop_sampler;
mod random_sampler;

/// A sample consists of 3 random values packed together into a `Float` and a `Vector2`.
#[derive(Debug)]
pub struct Sample {
    pub one_d: Float,
    pub two_d: Vector2,
}

impl Sample {
    pub fn new(one_d: Float, two_d: Vector2) -> Self {
        debug_assert!(one_d.in_range_incl_left(0.0, 1.0));
        debug_assert!(within_01(&two_d));

        Self { one_d, two_d }
    }
}

/// A sampler is responsible for generating random values inside `[0, 1)` in various formats.
pub trait Sampler: Send + Sync {
    /// Generates a new random value inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random value
    fn get_1d(&self) -> Float;

    /// Generates a new random `Vector2` inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random `Vector2`
    #[inline]
    fn get_2d(&self) -> Vector2 {
        Vector2::new(self.get_1d(), self.get_1d())
    }

    /// Generates a new random `Vector3` inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random `Vector3`
    #[inline]
    fn get_3d(&self) -> Vector3 {
        Vector3::new(self.get_1d(), self.get_1d(), self.get_1d())
    }

    /// Generates a new random sample inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random sample
    #[inline]
    fn get_sample(&self) -> Sample {
        Sample::new(self.get_1d(), self.get_2d())
    }
}
