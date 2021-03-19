#![allow(unused_variables)]
#![allow(dead_code)]

use crate::bxdf::Type;
use crate::integrator::{direct_illumination_light_wave, Integrator};
use crate::objects::SceneObject;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Color;
use geometry::offset_ray_towards;

pub struct SpectralPath {
    max_depth: u32,
    max_specular_depth: u32,
}

impl SpectralPath {
    pub fn new(max_depth: u32, max_specular_depth: u32) -> Self {
        Self {
            max_depth,
            max_specular_depth,
        }
    }
}

impl Integrator for SpectralPath {
    fn illumination(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
    ) -> Spectrum {
        let mut illumination = Spectrum::new_const(0.0);

        for light_wave_index in 0..Spectrum::size() {
            let mut hit = intersection.clone();

            let mut specular = false;
            let mut bounce = 0;
            let mut specular_bounce = 0;
            let mut throughput = 1.0;

            while bounce < self.max_depth && specular_bounce < self.max_specular_depth {
                let outgoing = -hit.ray.direction;
                let normal = hit.normal;
                let bsdf = hit.object.bsdf();
                let mut bounce_illum = 0.0;

                if bounce == 0 || specular {
                    if let SceneObject::Emitter(e) = &hit.object {
                        bounce_illum += e.emission_light_wave(light_wave_index);
                        //e.radiance(&outgoing, &normal);
                    }
                }

                bounce_illum +=
                    direct_illumination_light_wave(scene, sampler, &hit, &bsdf, light_wave_index);

                illumination[light_wave_index] += throughput * bounce_illum;

                let sample = sampler.get_sample();
                if let Some(bxdf_sample) =
                    bsdf.sample_light_wave(&normal, &outgoing, Type::ALL, &sample, light_wave_index)
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

                if specular {
                    specular_bounce += 1;
                } else {
                    bounce += 1;
                }
            }
        }

        illumination
    }
}
