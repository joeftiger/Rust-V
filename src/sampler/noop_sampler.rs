use crate::sampler::Sampler;

/// # Summary
/// A no-op Sampler only returning `0.5`.
#[derive(Clone)]
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
