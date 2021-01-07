//!# Summary
//! Integrators are a major component in ray tracing as they compute the color of each pixel.
//! They come in various forms, taking different paths.
//!
//! # Notation
//! * `E` - The eye
//! * `L` - The light
//! * `D` - Diffuse reflection or transmission
//! * `G` - Glossy reflection or transmission
//! * `S` - Specular reflection or refraction
//!
//! # Examles
//! The following set of traced paths are specified using regular expression.
//! * **Appel** ray casting: `E(D|G)L`
//! * **Whitted** recursive ray tracing: `E[S*](D|G)L`
//! * **Kajiya** path tracing: `E[(D|G|S)+(D|G)]L`
//! * **Goral** radiosity: `ED*L`

mod debug_normals;
mod whitted;

pub use debug_normals::DebugNormals;
pub use whitted::Whitted;

use crate::bxdf::BxDFType;
use crate::objects::ReceiverExt;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Color;
use geometry::Ray;

/// # Summary
/// An integrator to calculate the color of a pixel / ray.
///
/// This trait provides generic methods like `specular_reflection` and `specular_transmission`.
pub trait Integrator: Send + Sync {
    /// # Summary
    /// Integrates the given scene with the primary ray and the sampler
    ///
    /// # Arguments
    /// * `scene` - The scene to integrate
    /// * `primary_ray` - The primary ray shot into the scene
    /// * `sampler` - A sampler to generate values
    ///
    /// # Returns
    /// * The color spectrum of the given ray
    fn integrate(&self, scene: &Scene, primary_ray: &Ray, sampler: &dyn Sampler) -> Spectrum;

    /// # Summary
    /// Calculates the illumination (recursively if needed) at the given intersection.
    ///
    /// # Arguments
    /// * `scene` - The scene being integratd
    /// * `intersection` - The scene intersection we illuminate
    /// * `sampler` - A sampler to generate values
    /// * `depth` - The current recursive depth (if needed)
    ///
    /// # Returns
    /// * The illumination at the intersection
    fn illumination(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
    ) -> Spectrum;

    /// # Summary
    /// Computes the specular reflection at the given scene intersection, calling `illumination()`
    /// with a newly generated reflected ray.
    ///
    /// # Arguments
    /// * `scene` - The scene being integratd
    /// * `intersection` - The scene intersection we illuminate
    /// * `sampler` - A sampler to generate values
    /// * `depth` - The current recursive depth (if needed)
    ///
    /// # Returns
    /// * The illumination at the intersection
    //noinspection DuplicatedCode
    fn specular_reflection(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
    ) -> Spectrum {
        let outgoing = -intersection.info.ray.direction;

        let bsdf = intersection.object.bsdf();
        let normal = intersection.info.normal;
        let sample = sampler.get_sample();

        let bxdf_sample = bsdf.sample(
            &normal,
            &outgoing,
            BxDFType::REFLECTION | BxDFType::SPECULAR,
            &sample,
        );

        if let Some(bxdf_sample) = bxdf_sample {
            if bxdf_sample.pdf > 0.0 && !bxdf_sample.spectrum.is_black() {
                let cos = bxdf_sample.incident.dot(normal).abs();

                if cos != 0.0 {
                    let refl_ray = intersection.info.offset_ray_towards(bxdf_sample.incident);

                    if let Some(i) = scene.intersect(&refl_ray) {
                        let new_depth = depth - 1;

                        return if new_depth > 0 {
                            let illumination = self.illumination(scene, &i, sampler, depth - 1);
                            bxdf_sample.spectrum * illumination * cos / bxdf_sample.pdf
                        } else {
                            bxdf_sample.spectrum
                        };
                    }
                }
            }
        }

        Spectrum::black()
    }

    /// # Summary
    /// Computes the specular transmission at the given scene intersection, calling `illumination()`
    /// with a newly generated transmitted ray.
    ///
    /// # Arguments
    /// * `scene` - The scene being integratd
    /// * `intersection` - The scene intersection we illuminate
    /// * `sampler` - A sampler to generate values
    /// * `depth` - The current recursive depth (if needed)
    ///
    /// # Returns
    /// * The illumination at the intersection
    //noinspection DuplicatedCode
    fn specular_transmission(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
    ) -> Spectrum {
        let outgoing = -intersection.info.ray.direction;

        let bsdf = intersection.object.bsdf();
        let normal = intersection.info.normal;
        let sample = sampler.get_sample();

        let bxdf_sample = bsdf.sample(
            &normal,
            &outgoing,
            BxDFType::SPECULAR | BxDFType::TRANSMISSION,
            &sample,
        );

        if let Some(bxdf_sample) = bxdf_sample {
            if bxdf_sample.pdf > 0.0 && !bxdf_sample.spectrum.is_black() {
                let cos = bxdf_sample.incident.dot(normal).abs();

                if cos != 0.0 {
                    let refl_ray = intersection.info.offset_ray_towards(bxdf_sample.incident);

                    if let Some(i) = scene.intersect(&refl_ray) {
                        let new_depth = depth - 1;

                        return if new_depth > 0 {
                            let illumination = self.illumination(scene, &i, sampler, depth - 1);
                            bxdf_sample.spectrum * illumination * cos / bxdf_sample.pdf
                        } else {
                            bxdf_sample.spectrum
                        };
                    }
                }
            }
        }

        Spectrum::black()
    }
}
