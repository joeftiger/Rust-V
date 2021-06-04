#![allow(unused_variables)]
#![allow(dead_code)]

use crate::bxdf::Type;
use crate::integrator::{direct_illumination_light_wave, Integrator};
use crate::objects::SceneObject;
use crate::samplers::spectral_samplers::SpectralSampler;
use crate::samplers::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use crate::Float;
use geometry::{offset_ray_towards, Ray};
use serde::{Deserialize, Serialize};

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

    // TODO: simplify
    fn illumination(
        &self,
        scene: &Scene,
        mut hit: SceneIntersection,
        sampler: Sampler,
        indices: &[usize],
        illumination: &mut [Float],
    ) {
        assert_eq!(indices.len(), illumination.len());

        let mut specular = false;
        let throughput: &mut [Float] = &mut vec![0.0; illumination.len()];

        let mut bounce = 0;

        // trace bounces as one bundle while not specular
        while bounce < self.max_depth && !specular {
            let outgoing = -hit.ray.direction;
            let normal = hit.normal;
            let bsdf = hit.object.bsdf();

            if bounce == 0 {
                if let SceneObject::Emitter(e) = &hit.object {
                    indices
                        .iter()
                        .zip(illumination.iter_mut())
                        .for_each(|(&i, v)| *v = e.emission_light_wave(i));
                    return;
                }
            }

            // add direction illumination
            for i in 0..illumination.len() {
                illumination[i] +=
                    throughput[i] * direct_illumination_light_wave(scene, sampler, &hit, bsdf, i);
            }

            // is there a matching bxdf
            if let Some(bxdf) = bsdf.random_matching_bxdf(Type::ALL, sampler.get_1d()) {
                // can we sample the bxdf
                if let Some((incident, pdf, typ)) =
                    bxdf.sample_for_light_waves(outgoing, sampler.get_2d())
                {
                    if pdf == 0.0 {
                        return;
                    }

                    specular = typ.is_specular();

                    // decrease throughput according to wavelength
                    for (index, through) in throughput.iter_mut().enumerate() {
                        let lambda = bxdf.evaluate_light_wave(incident, outgoing, index);
                        if lambda == 0.0 {
                            return;
                        }

                        let cos_abs = if specular {
                            // division of cosine omitted in specular bxdfs
                            1.0
                        } else {
                            incident.dot(normal).abs()
                        };

                        *through *= lambda * cos_abs / pdf;
                    }

                    // trace next intersection
                    let ray = offset_ray_towards(hit.point, hit.normal, incident);
                    match scene.intersect(&ray) {
                        Some(i) => hit = i,
                        None => return,
                    }

                    bounce += 1;
                } else {
                    return; // could not sample bxdf
                }
            } else {
                return; // no matching bxdf found
            }
        }

        if bounce == self.max_depth || !specular {
            return;
        }

        let last_bounce = bounce;
        let last_hit = hit;

        // continue tracing light_waves independently
        for i in 0..illumination.len() {
            let mut hit = last_hit.clone();
            let mut specular = false;

            for bounce in last_bounce..self.max_depth {
                let outgoing = -hit.ray.direction;
                let normal = hit.normal;
                let bsdf = hit.object.bsdf();

                if bounce == 0 {
                    if let SceneObject::Emitter(e) = &hit.object {
                        illumination[i] = e.emission_light_wave(indices[i]); //e.radiance(&outgoing, &normal);
                        break;
                    }
                } else if specular {
                    if let SceneObject::Emitter(e) = &hit.object {
                        illumination[i] += throughput[i] * e.emission_light_wave(indices[i]); //e.radiance(&outgoing, &normal);
                        break;
                    }
                }

                illumination[i] += throughput[i]
                    * direct_illumination_light_wave(scene, sampler, &hit, &bsdf, indices[i]);

                let sample = sampler.get_sample();
                if let Some(bxdf_sample) =
                    bsdf.sample_light_wave(normal, outgoing, Type::ALL, sample, indices[i])
                {
                    if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum == 0.0 {
                        break;
                    }

                    specular = bxdf_sample.typ.is_specular();
                    let cos_abs = if specular {
                        // division of cosine omitted in specular bxdfs
                        1.0
                    } else {
                        bxdf_sample.incident.dot(normal).abs()
                    };

                    throughput[i] *= bxdf_sample.spectrum * (cos_abs / bxdf_sample.pdf);

                    let ray = offset_ray_towards(hit.point, hit.normal, bxdf_sample.incident);
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

            self.spectral_sampler.fill_samples(&mut indices);

            self.illumination(
                scene,
                original_intersection,
                sampler,
                &indices,
                &mut illumination,
            );

            for (&index, &lambda) in indices.iter().zip(illumination.iter()) {
                pixel.add_light_wave(lambda, index);
            }
        } else {
            pixel.add_black();
        }
    }
}
