// use crate::bxdf::{BxDF, Type, BSDF};
// use crate::objects::SceneObject;
// use crate::samplers::spectral_samplers::SpectralSampler;
// use crate::samplers::Sampler;
// use crate::scene::{Scene, SceneIntersection};
// use definitions::{Float, Vector3};
// use geometry::offset_ray_towards;
//
// pub struct HeroIntegrator {
//     max_depth: u32,
//     light_wave_samples: u32,
//     spectral_sampler: SpectralSampler,
// }
//
// struct Path<'a> {
//     hits: Vec<PathComponent<'a>>,
//     done: bool,
// }
//
// impl<'a> Path<'a> {
//     pub fn new(hits: Vec<PathComponent<'a>>, done: bool) -> Self {
//         Self { hits, done }
//     }
// }
//
// struct PathComponent<'a> {
//     hit: SceneIntersection,
//     bxdf: &'a dyn BxDF,
// }
//
// impl<'a> PathComponent<'a> {
//     pub fn new<T>(hit: SceneIntersection, bxdf: &'a T) -> Self
//     where
//         T: BxDF,
//     {
//         Self { hit, bxdf }
//     }
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
//
//     fn create_path<'a>(
//         &self,
//         scene: &Scene,
//         original_intersection: &SceneIntersection,
//         sampler: Sampler,
//     ) -> Path {
//         let mut hits = Vec::with_capacity(self.max_depth as usize - 1);
//
//         let hit = original_intersection;
//
//         for bounce in 0..self.max_depth {
//             let bsdf = hit.object.bsdf();
//             let sample = sampler.get_sample();
//
//             if let Some(bxdf) = bsdf.random_matching_bxdf(Type::ALL, sample.one_d) {
//                 hits.push(PathComponent::new(hit.clone(), bxdf));
//
//                 BSDF::sample
//
//
//                 let ray = offset_ray_towards(hit.point, hit.normal, bxdf.incident);
//                 match scene.intersect(&ray) {
//                     Some(i) => hit = i,
//                     None => break,
//                 }
//             } else {
//                 return Path::new(hits, true);
//             }
//
//             match bsdf.random_matching_bxdf(Type::ALL, sampler.get_1d()) {
//                 Some(bxdf) => hits.push(PathComponent::new(hit.clone(), bxdf)),
//                 None => return Path::new(hits, true),
//             }
//
//
//         }
//
//         path
//     }
// }
