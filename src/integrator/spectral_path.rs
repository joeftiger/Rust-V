#![allow(unused_variables)]
#![allow(dead_code)]

use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;

pub struct SpectralPath {
    max_depth: u32,
    max_specular_depth: u32,
}

impl SpectralPath {
    pub fn new(max_depth: u32, max_specular_depth: u32) -> Self {
        Self {
            max_depth,
            max_specular_depth,
        }
    }
}

impl Integrator for SpectralPath {
    fn illumination(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
    ) -> Spectrum {
        unimplemented!()
    }
}
