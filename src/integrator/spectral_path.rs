#![allow(unused_variables)]
#![allow(dead_code)]

use crate::bxdf::{BxDF, BxDFSample, Type, BSDF};
use crate::integrator::{direct_illumination_light_wave, Integrator};
use crate::objects::SceneObject;
use crate::samplers::spectral_samplers::SpectralSampler;
use crate::samplers::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use crate::Float;
use geometry::{offset_ray_towards, Ray};
use serde::{Deserialize, Serialize};
use utility::floats::FloatExt;

enum TraceResult {
    Done,
    Specular {
        bounce: u32,
        hit: SceneIntersection,
        incidents: Vec<Option<BxDFSample<Float>>>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct SpectralPath {
    max_depth: u32,
    light_wave_samples: u32,
    spectral_sampler: SpectralSampler,
}

impl SpectralPath {
    pub fn new(max_depth: u32, light_wave_samples: u32, spectral_sampler: SpectralSampler) -> Self {
        Self {
            max_depth,
            light_wave_samples,
            spectral_sampler,
        }
    }

    //#[allow(clippy::too_many_arguments)]
    fn trace_single(
        &self,
        scene: &Scene,
        mut hit: SceneIntersection,
        sampler: Sampler,
        bxdf: &dyn BxDF,
        current_bounce: u32,
        index: usize,
        illumination: &mut Float,
        throughput: &mut Float,
    ) {
        if let SceneObject::Emitter(e) = &hit.object {
            *illumination += *throughput * e.emission_light_wave(index); //e.radiance(&outgoing, &normal);
            return;
        }

        let mut specular = false;

        if let Some(bxdf_sample) = BSDF::sample_bxdf_light_wave(
            bxdf,
            hit.normal,
            -hit.ray.direction,
            sampler.get_sample(),
            index,
        ) {
            if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum == 0.0 {
                return;
            }

            *throughput *= bxdf_sample.spectrum / bxdf_sample.pdf;

            let ray = offset_ray_towards(hit.point, hit.normal, bxdf_sample.incident);
            match scene.intersect(&ray) {
                Some(i) => hit = i,
                None => return,
            }
        }

        for bounce in current_bounce..self.max_depth {
            let outgoing = -hit.ray.direction;
            let normal = hit.normal;
            let bsdf = hit.object.bsdf();

            if specular {
                if let SceneObject::Emitter(e) = &hit.object {
                    *illumination += *throughput * e.emission_light_wave(index); //e.radiance(&outgoing, &normal);
                    return;
                }
            }

            *illumination +=
                *throughput * direct_illumination_light_wave(scene, sampler, &hit, bsdf, index);

            if let Some(bxdf_sample) =
                bsdf.sample_light_wave(normal, outgoing, Type::ALL, sampler.get_sample(), index)
            {
                if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum == 0.0 {
                    return;
                }

                specular = bxdf_sample.typ.is_specular();
                let cos_abs = if specular {
                    // division of cosine omitted in specular bxdfs
                    1.0
                } else {
                    bxdf_sample.incident.dot(normal).abs()
                };

                *throughput *= bxdf_sample.spectrum * (cos_abs / bxdf_sample.pdf);

                let ray = offset_ray_towards(hit.point, hit.normal, bxdf_sample.incident);
                match scene.intersect(&ray) {
                    Some(i) => hit = i,
                    None => return,
                }
            } else {
                return;
            }
        }
    }

    fn trace_bundle(
        &self,
        scene: &Scene,
        mut hit: SceneIntersection,
        sampler: Sampler,
        indices: &[usize],
        illumination: &mut [Float],
        throughput: &mut [Float],
    ) {
        assert_eq!(indices.len(), illumination.len());
        assert_eq!(indices.len(), throughput.len());

        for bounce in 0..self.max_depth {
            let outgoing = -hit.ray.direction;
            let normal = hit.normal;
            let bsdf = hit.object.bsdf();

            // immediately hitting emitter?
            if bounce == 0 {
                if let SceneObject::Emitter(e) = &hit.object {
                    for i in 0..indices.len() {
                        illumination[i] = e.emission_light_wave(indices[i]);
                    }
                    return;
                }
            }

            // add direction illumination
            for i in 0..indices.len() {
                let direct_illumination =
                    direct_illumination_light_wave(scene, sampler, &hit, bsdf, indices[i]);
                illumination[i] += throughput[i] * direct_illumination;
            }

            // sample bxdf
            if let Some(bxdf) = bsdf.random_matching_bxdf(Type::ALL, sampler.get_1d()) {
                // if specular, trace single
                if bxdf.get_type().is_specular() {
                    for i in 0..indices.len() {
                        self.trace_single(
                            scene,
                            hit.clone(),
                            sampler,
                            bxdf,
                            bounce,
                            i,
                            &mut illumination[i],
                            &mut throughput[i],
                        );
                    }

                    return;
                }

                if let Some((incident, pdf)) =
                    BSDF::sample_bxdf_incident_pdf(bxdf, normal, outgoing, sampler.get_2d())
                {
                    if pdf == 0.0 {
                        return;
                    }

                    // decrease throughput according to wavelength
                    for i in 0..illumination.len() {
                        let intensity = BSDF::evaluate_bxdf_light_wave(
                            bxdf, normal, incident, outgoing, indices[i],
                        );

                        throughput[i] *=
                            intensity * incident.dot(normal).fast_clamp(0.0, 1.0) / pdf;
                    }

                    // trace next intersection
                    let ray = offset_ray_towards(hit.point, hit.normal, incident);
                    match scene.intersect(&ray) {
                        Some(i) => hit = i,
                        None => break,
                    }
                } else {
                    break;
                }
            }
        }
    }
}

#[typetag::serde]
impl Integrator for SpectralPath {
    fn integrate(&self, pixel: &mut Pixel, scene: &Scene, primary_ray: &Ray, sampler: Sampler) {
        if let Some(original_intersection) = scene.intersect(primary_ray) {
            let mut indices = vec![0; self.light_wave_samples as usize];
            let mut illumination = vec![0.0; self.light_wave_samples as usize];
            let mut throughput = vec![1.0; self.light_wave_samples as usize];

            self.spectral_sampler.fill_samples(&mut indices);

            self.trace_bundle(
                scene,
                original_intersection,
                sampler,
                &indices,
                &mut illumination,
                &mut throughput,
            );

            for (&index, &lambda) in indices.iter().zip(illumination.iter()) {
                pixel.add_light_wave(lambda, index);
            }
        } else {
            pixel.add_black();
        }
    }
}
