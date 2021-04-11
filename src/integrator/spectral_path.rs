#![allow(unused_variables)]
#![allow(dead_code)]

use crate::bxdf::Type;
use crate::integrator::{direct_illumination_light_wave, Integrator};
use crate::objects::SceneObject;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use crate::Spectrum;
use definitions::Float;
use geometry::{offset_ray_towards, Ray};

pub struct SpectralPath {
    max_depth: u32,
    light_wave_samples: u32,
}

impl SpectralPath {
    pub fn new(max_depth: u32, light_wave_samples: u32) -> Self {
        Self {
            max_depth,
            light_wave_samples,
        }
    }

    fn illumination(
        &self,
        scene: &Scene,
        original_intersection: &SceneIntersection,
        sampler: &dyn Sampler,
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

            if bounce == 0 || specular {
                if let SceneObject::Emitter(e) = &hit.object {
                    bounce_illum += e.emission_light_wave(light_wave_index);
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

impl Integrator for SpectralPath {
    fn integrate(
        &self,
        pixel: &mut Pixel,
        scene: &Scene,
        primary_ray: &Ray,
        sampler: &dyn Sampler,
    ) {
        if let Some(original_intersecton) = scene.intersect(primary_ray) {
            for _ in 0..self.light_wave_samples {
                let light_wave_index = (sampler.get_1d() * Spectrum::size() as Float) as usize;

                let lambda =
                    self.illumination(scene, &original_intersecton, sampler, light_wave_index);

                pixel.add_light_wave(lambda, light_wave_index);
            }
        } else {
            pixel.add_black();
        }
    }
}
