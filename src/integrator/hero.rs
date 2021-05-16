// use crate::bxdf::{BxDF, Type, BSDF};
// use crate::objects::SceneObject;
// use crate::samplers::spectral_samplers::SpectralSampler;
// use crate::samplers::Sampler;
// use crate::scene::{Scene, SceneIntersection};
// use definitions::{Float, Vector3};
// use geometry::offset_ray_towards;
// use serde::{Deserialize, Serialize};
//
// #[derive(Deserialize, Serialize)]
// pub struct HeroIntegrator {
//     max_depth: u32,
//     light_wave_samples: u32,
//     spectral_sampler: SpectralSampler,
// }
//
// impl HeroIntegrator {
//     pub fn new(max_depth: u32, light_wave_samples: u32, spectral_sampler: SpectralSampler) -> Self {
//         Self {
//             max_depth,
//             light_wave_samples,
//             spectral_sampler,
//         }
//     }
// }
