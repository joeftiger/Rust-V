#[cfg(debug_assertions)]
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
pub trait Sampler {
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

/// # Summary
/// A simple random sampler using `fastrand` to generate random valuates.
///
/// Using `RandomSampler::default()` initializes the seed with value `0`.
pub struct RandomSampler;

impl Default for RandomSampler {
    fn default() -> Self {
        fastrand::seed(0);
        Self
    }
}

impl Sampler for RandomSampler {
    #[inline]
    fn get_1d(&self) -> f32 {
        let rand = fastrand::f32();
        debug_assert!(in_range_incl_left(rand, 0.0, 1.0));
        rand
    }
}

/// # Summary
/// A no-op Sampler only returning `0.5`.
pub struct NoOpSampler;

impl Sampler for NoOpSampler {
    /// # Summary
    /// Always returns `0.5`
    ///
    /// # Returns
    /// * `0.5`
    #[inline(always)]
    fn get_1d(&self) -> f32 {
        0.5
    }
}
