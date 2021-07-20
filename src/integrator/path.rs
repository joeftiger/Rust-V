use crate::bxdf::Type;
use crate::integrator::{direct_illumination, Integrator};
use crate::objects::SceneObject;
use crate::samplers::Sampler;
use crate::scene::Scene;
use crate::sensor::pixel::Pixel;
use crate::Spectrum;
use color::Color;
use geometry::{offset_ray_towards, Ray};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    max_depth: u32,
}

impl Path {
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }
}

#[typetag::serde]
impl Integrator for Path {
    fn integrate(&self, pixel: &mut Pixel, scene: &Scene, primary_ray: &Ray, sampler: Sampler) {
        if let Some(intersection) = scene.intersect(primary_ray) {
            let mut illumination = Spectrum::broadcast(0.0);
            let mut throughput = Spectrum::broadcast(1.0);

            let mut hit = intersection;

            for _ in 0..self.max_depth {
                let outgoing = -hit.ray.direction;
                let normal = hit.normal;
                let bsdf = hit.object.bsdf();

                if let SceneObject::Emitter(e) = &hit.object {
                    illumination += throughput * e.emission;
                } else {
                    illumination += throughput * direct_illumination(scene, sampler, &hit, bsdf);
                }

                let sample = sampler.get_sample();
                if let Some(bxdf_sample) = bsdf.sample(normal, outgoing, Type::ALL, sample) {
                    if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum.is_black() {
                        break;
                    }

                    let cos_abs = if bxdf_sample.typ.is_specular() {
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

            pixel.add(illumination);
        } else {
            pixel.add_black()
        }
    }
}
