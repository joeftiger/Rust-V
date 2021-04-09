use crate::sampler::pixel_samplers::PixelSampler;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ultraviolet::{UVec2, Vec2};

#[derive(Serialize, Deserialize)]
pub struct UniformSuperSampling {
    step: Vec2,
    sample_space: UVec2,
    #[serde(skip)]
    progress: HashMap<UVec2, u8>,
}

impl UniformSuperSampling {
    /// Creates a new super sampling pixel sampler.
    ///
    /// # Constraints
    /// * `sample_space` - Should not cover more than `u8::MAX`.
    ///
    /// # Arguments
    /// * `sample_space` - The number of samples for each pixel in both `x` and `y` axis.
    ///
    /// # Returns
    /// Self
    pub fn new(sample_space: UVec2) -> Self {
        assert!((sample_space.x * sample_space.y) <= u8::MAX as u32);

        let step = Vec2::one() / Vec2::new(sample_space.x as Float, sample_space.y as Float);

        Self {
            step,
            sample_space,
            progress: HashMap::new(),
        }
    }
}

impl PixelSampler for UniformSuperSampling {
    fn sample(&mut self, pixel: UVec2) -> Vec2 {
        // get index or insert it
        let i = if let Some(px) = self.progress.get_mut(&pixel) {
            *px %= (self.sample_space.x * self.sample_space.y) as u8;
            *px
        } else {
            self.progress.insert(pixel, 1);
            1
        };

        let coordinate = Vec2::new(
            (i % self.sample_space.x as u8) as Float,
            i as Float / self.sample_space.x as Float,
        );

        let out = self.step * 0.5 + self.step * coordinate;
        debug_assert_eq!(out.clamped(Vec2::zero(), Vec2::one()), out);

        out
    }
}
