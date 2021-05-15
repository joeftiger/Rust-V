use crate::objects::SceneObject;
use crate::samplers::spectral_samplers::SpectralSampler;
use crate::scene::{Scene, SceneIntersection};
use definitions::Float;

pub struct HeroIntegrator {
    max_depth: u32,
    light_wave_samples: u32,
    spectral_sampler: SpectralSampler,
}

struct StorageHelper<'a> {
    light_waves_indices: &'a [usize],
    illumination: &'a [Float],
    bounce_illumination: &'a [Float],
    throughput: &'a [Float],
}

impl HeroIntegrator {
    pub fn new(max_depth: u32, light_wave_samples: u32, spectral_sampler: SpectralSampler) -> Self {
        Self {
            max_depth,
            light_wave_samples,
            spectral_sampler,
        }
    }

    fn i() {}
}
