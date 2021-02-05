use crate::sampler::Sampler;

use crate::debug_utils::in_range_incl_left;

/// A simple random sampler using `fastrand` to generate random valuates.
///
/// Using `RandomSampler::default()` initializes the seed with value `0`.
#[derive(Clone)]
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
