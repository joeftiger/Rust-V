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
mod path;
mod whitted;

pub use debug_normals::DebugNormals;
pub use path::Path;
pub use whitted::Whitted;

use crate::bxdf::{BxDFType, BSDF};
use crate::objects::ReceiverExt;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::{Color, Colors};
use geometry::{offset_ray_towards, Ray};

/// # Summary
/// An integrator to calculate the color of a pixel / ray.
///
/// This trait provides generic methods like `specular_reflection` and `specular_transmission`.
pub trait Integrator: Send + Sync {
    fn max_depth(&self) -> u32;

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
    fn integrate(&self, scene: &Scene, primary_ray: &Ray, sampler: &dyn Sampler) -> Spectrum {
        if let Some(si) = scene.intersect(primary_ray) {
            self.illumination(scene, &si, sampler, 0)
        } else {
            Spectrum::black()
        }
    }

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
        debug_assert!(depth < self.max_depth());

        let outgoing = -intersection.ray.direction;

        let bsdf = intersection.object.bsdf();
        let normal = intersection.normal;
        let sample = sampler.get_sample();

        let bxdf_sample_option = bsdf.sample(
            &normal,
            &outgoing,
            BxDFType::REFLECTION | BxDFType::SPECULAR,
            &sample,
        );

        let mut reflection = Spectrum::black();

        if let Some(bxdf_sample) = bxdf_sample_option {
            if bxdf_sample.pdf > 0.0 && !bxdf_sample.spectrum.is_black() {
                let cos = bxdf_sample.incident.dot(normal);

                if cos != 0.0 {
                    let refl_ray = offset_ray_towards(
                        intersection.point,
                        intersection.normal,
                        bxdf_sample.incident,
                    );

                    if let Some(si) = scene.intersect(&refl_ray) {
                        let illumination = self.illumination(scene, &si, sampler, depth);
                        reflection +=
                            illumination * bxdf_sample.spectrum * (cos.abs() / bxdf_sample.pdf);
                    }
                }
            }
        }

        reflection
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
        debug_assert!(depth < self.max_depth());

        let outgoing = -intersection.ray.direction;

        let bsdf = intersection.object.bsdf();
        let normal = intersection.normal;
        let sample = sampler.get_sample();

        let bxdf_sample_option = bsdf.sample(
            &normal,
            &outgoing,
            BxDFType::SPECULAR | BxDFType::TRANSMISSION,
            &sample,
        );

        let mut transmission = Spectrum::black();

        if let Some(bxdf_sample) = bxdf_sample_option {
            if bxdf_sample.pdf > 0.0 && !bxdf_sample.spectrum.is_black() {
                let cos = bxdf_sample.incident.dot(normal);

                if cos != 0.0 {
                    let refl_ray = offset_ray_towards(
                        intersection.point,
                        intersection.normal,
                        bxdf_sample.incident,
                    );

                    if let Some(si) = scene.intersect(&refl_ray) {
                        let illumination = self.illumination(scene, &si, sampler, depth + 1);
                        transmission +=
                            illumination * bxdf_sample.spectrum * (cos.abs() / bxdf_sample.pdf);
                    }
                }
            }
        }

        transmission
    }
}

fn direct_illumination(
    scene: &Scene,
    sampler: &dyn Sampler,
    intersection: &SceneIntersection,
    bsdf: &BSDF,
) -> Spectrum {
    let mut illumination = Spectrum::black();

    if bsdf.is_empty() {
        return illumination;
    }

    let outgoing_world = -intersection.ray.direction;

    for light in &scene.lights {
        let emitter_sample = light.sample(&intersection.point, &sampler.get_2d());

        if emitter_sample.pdf > 0.0
            && !emitter_sample.radiance.is_black()
            && emitter_sample.occlusion_tester.unoccluded(scene)
        {
            let bsdf_spectrum = bsdf.evaluate(
                &intersection.normal,
                &emitter_sample.incident,
                &outgoing_world,
                BxDFType::ALL,
            );

            if !bsdf_spectrum.is_black() {
                let cos = emitter_sample.incident.dot(intersection.normal);

                if cos != 0.0 {
                    illumination +=
                        bsdf_spectrum * emitter_sample.radiance * (cos.abs() / emitter_sample.pdf)
                }
            }
        }
    }

    illumination
}
