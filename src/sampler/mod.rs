mod noop_sampler;
mod random_sampler;

pub use noop_sampler::NoOpSampler;
pub use random_sampler::RandomSampler;

use crate::debug_utils::{in_range_incl_left, within_01};
use ultraviolet::{Vec2, Vec3};

/// # Summary
/// A sample consists of 3 random values packed together into a `f32` and a `Vec2`.
#[derive(Debug)]
pub struct Sample {
    pub one_d: f32,
    pub two_d: Vec2,
}

impl Sample {
    pub fn new(one_d: f32, two_d: Vec2) -> Self {
        debug_assert!(in_range_incl_left(one_d, 0.0, 1.0));
        debug_assert!(within_01(&two_d));

        Self { one_d, two_d }
    }
}

/// # Summary
/// A sampler is responsible for generating random values inside `[0, 1)` in various formats.
pub trait Sampler: Send + Sync {
    /// # Summary
    /// Generates a new random value inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random value
    fn get_1d(&self) -> f32;

    /// # Summary
    /// Generates a new random `Vec2` inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random `Vec2`
    #[inline]
    fn get_2d(&self) -> Vec2 {
        Vec2::new(self.get_1d(), self.get_1d())
    }

    /// # Summary
    /// Generates a new random `Vec3` inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random `Vec3`
    #[inline]
    fn get_3d(&self) -> Vec3 {
        Vec3::new(self.get_1d(), self.get_1d(), self.get_1d())
    }

    /// # Summary
    /// Generates a new random sample inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random sample
    #[inline]
    fn get_sample(&self) -> Sample {
        Sample::new(self.get_1d(), self.get_2d())
    }
}
