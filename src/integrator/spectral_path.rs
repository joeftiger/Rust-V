#![allow(unused_variables)]
#![allow(dead_code)]

use crate::bxdf::Type;
use crate::integrator::{direct_illumination_light_wave, Integrator};
use crate::objects::SceneObject;
use crate::samplers::spectral_samplers::SpectralSampler;
use crate::samplers::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use definitions::Float;
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

    fn illumination(
        &self,
        scene: &Scene,
        original_intersection: &SceneIntersection,
        sampler: Sampler,
        light_wave_index: usize,
    ) -> Float {
        let mut illumination = 0.0;

        let mut hit = original_intersection.clone();

        let mut specular = false;
        let mut throughput = 1.0;

        for bounce in 0..self.max_depth {
            let outgoing = -hit.ray.direction;
            let normal = hit.normal;
            let bsdf = hit.object.bsdf();

            let mut bounce_illum = 0.0;

            if bounce == 0 {
                if let SceneObject::Emitter(e) = &hit.object {
                    illumination = e.emission_light_wave(light_wave_index); //e.radiance(&outgoing, &normal);
                    break;
                }
            } else if specular {
                if let SceneObject::Emitter(e) = &hit.object {
                    illumination += throughput * e.emission_light_wave(light_wave_index); //e.radiance(&outgoing, &normal);
                    break;
                }
            }

            bounce_illum +=
                direct_illumination_light_wave(scene, sampler, &hit, &bsdf, light_wave_index);

            illumination += throughput * bounce_illum;

            let sample = sampler.get_sample();
            if let Some(bxdf_sample) =
                bsdf.sample_light_wave(normal, outgoing, Type::ALL, sample, light_wave_index)
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

                throughput *= bxdf_sample.spectrum * (cos_abs / bxdf_sample.pdf);

                let ray = offset_ray_towards(hit.point, hit.normal, bxdf_sample.incident);
                match scene.intersect(&ray) {
                    Some(i) => hit = i,
                    None => break,
                }
            } else {
                break;
            }
        }

        illumination
    }
}

#[typetag::serde]
impl Integrator for SpectralPath {
    fn integrate(&self, pixel: &mut Pixel, scene: &Scene, primary_ray: &Ray, sampler: Sampler) {
        if let Some(original_intersection) = scene.intersect(primary_ray) {
            let mut buf = vec![0; self.light_wave_samples as usize];
            self.spectral_sampler.fill_samples(&mut buf);

            for index in buf {
                let lambda = self.illumination(scene, &original_intersection, sampler, index);
                pixel.add_light_wave(lambda, index);
            }
        } else {
            pixel.add_black();
        }
    }
}
