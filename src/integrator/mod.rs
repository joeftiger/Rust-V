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
mod hero;
mod path;
mod spectral_path;
mod whitted;

pub use debug_normals::DebugNormals;
pub use path::Path;
pub use spectral_path::SpectralPath;
pub use whitted::Whitted;

use crate::bxdf::{Type, BSDF};
use crate::samplers::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use crate::Spectrum;
use color::Color;
use definitions::Float;
use geometry::Ray;

/// An integrator to calculate the color of a pixel / ray.
#[typetag::serde]
pub trait Integrator: Send + Sync {
    /// Integrates the given scene with the primary ray and the sampler for a given pixel.
    ///
    /// # Arguments
    /// * `pixel` - The pixel to integrate for
    /// * `scene` - The scene to integrate
    /// * `primary_ray` - The primary ray shot into the scene
    /// * `sampler` - A sampler to generate values
    ///
    /// # Returns
    /// * The color spectrum of the given ray
    fn integrate(&self, pixel: &mut Pixel, scene: &Scene, primary_ray: &Ray, sampler: Sampler);
}

#[inline]
fn direct_illumination(
    scene: &Scene,
    sampler: Sampler,
    intersection: &SceneIntersection,
    bsdf: &BSDF,
) -> Spectrum {
    let mut illumination = Spectrum::broadcast(0.0);

    if bsdf.is_empty() {
        return illumination;
    }

    let outgoing_world = -intersection.ray.direction;

    for light in &scene.emitters {
        let emitter_sample = light.sample(intersection.point, sampler.get_2d());

        if emitter_sample.pdf > 0.0
            && !emitter_sample.radiance.is_black()
            && emitter_sample.occlusion_tester.unoccluded(scene)
        {
            let bsdf_spectrum = bsdf.evaluate(
                intersection.normal,
                emitter_sample.incident,
                outgoing_world,
                Type::ALL,
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

#[inline]
fn direct_illumination_light_wave(
    scene: &Scene,
    sampler: Sampler,
    intersection: &SceneIntersection,
    bsdf: &BSDF,
    light_wave_index: usize,
) -> Float {
    let mut illumination = 0.0;

    if bsdf.is_empty() {
        return illumination;
    }

    let outgoing_world = -intersection.ray.direction;

    for light in &scene.emitters {
        let emitter_sample =
            light.sample_light_wave(intersection.point, sampler.get_2d(), light_wave_index);

        if emitter_sample.pdf > 0.0
            && emitter_sample.radiance != 0.0
            && emitter_sample.occlusion_tester.unoccluded(scene)
        {
            let bsdf_intensity = bsdf.evaluate_light_wave(
                intersection.normal,
                emitter_sample.incident,
                outgoing_world,
                Type::ALL,
                light_wave_index,
            );

            if bsdf_intensity != 0.0 {
                let cos = emitter_sample.incident.dot(intersection.normal);

                if cos != 0.0 {
                    illumination +=
                        bsdf_intensity * emitter_sample.radiance * cos.abs() / emitter_sample.pdf;
                }
            }
        }
    }

    illumination
}
