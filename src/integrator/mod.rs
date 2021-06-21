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

pub use debug_normals::*;
pub use hero::*;
pub use path::*;
pub use spectral_path::*;
pub use whitted::*;

use crate::bxdf::{Type, BSDF};
use crate::objects::Emitter;
use crate::samplers::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use crate::Float;
use crate::Spectrum;
use color::Color;
use core::slice::Iter;
use geometry::Ray;
use std::sync::Arc;

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

use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DirectLightStrategy {
    All,
    Random,
}

impl DirectLightStrategy {
    pub fn get_emitters<'a>(&self, scene: &'a Scene, sample: Float) -> Iter<'a, Arc<Emitter>> {
        match self {
            DirectLightStrategy::All => scene.emitters.iter(),
            DirectLightStrategy::Random => {
                let i = (scene.emitters.len() as Float * sample) as usize;
                core::slice::from_ref(scene.emitters.index(i)).iter()
            }
        }
    }
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

fn direct_illumination_buf(
    scene: &Scene,
    sampler: Sampler,
    strategy: DirectLightStrategy,
    hit: &SceneIntersection,
    bsdf: &BSDF,
    indices: &[usize],
    illumination: &mut [Float],
    throughput: &[Float],
) {
    if bsdf.is_empty() {
        return;
    }

    let outgoing_world = -hit.ray.direction;
    for light in strategy.get_emitters(scene, sampler.get_1d()) {
        let sample = light.sample_buf(hit.point, sampler.get_2d(), indices);

        if sample.pdf > 0.0 && sample.occlusion_tester.unoccluded(scene) {
            let intensities = bsdf.evaluate_buf(
                hit.normal,
                sample.incident,
                outgoing_world,
                Type::ALL,
                indices,
            );

            for i in 0..indices.len() {
                if intensities[i] != 0.0 && sample.radiance[i] != 0.0 {
                    let cos_abs = sample.incident.dot(hit.normal).abs();

                    illumination[i] +=
                        throughput[i] * intensities[i] * sample.radiance[i] * cos_abs / sample.pdf;
                }
            }
        }
    }
}
