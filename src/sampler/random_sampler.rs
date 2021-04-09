use crate::sampler::Sampler;
use definitions::Float;

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
    fn get_1d(&self) -> Float {
        #[cfg(feature = "f64")]
        return fastrand::f64() as Float;
        #[cfg(not(feature = "f64"))]
        return fastrand::f32() as Float;
    }
}
