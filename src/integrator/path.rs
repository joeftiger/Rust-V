use crate::bxdf::Type;
use crate::integrator::{direct_illumination, Integrator};
use crate::objects::SceneObject;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::sensor::pixel::Pixel;
use crate::Spectrum;
use color::Color;
use geometry::{offset_ray_towards, Ray};

pub struct Path {
    max_depth: u32,
}

impl Path {
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }
}

impl Integrator for Path {
    fn integrate(
        &self,
        pixel: &mut Pixel,
        scene: &Scene,
        primary_ray: &Ray,
        sampler: &dyn Sampler,
    ) {
        if let Some(intersection) = scene.intersect(primary_ray) {
            let mut illumination = Spectrum::broadcast(0.0);
            let mut throughput = Spectrum::broadcast(1.0);

            let mut hit = intersection;
            let mut specular = false;

            for bounce in 0..self.max_depth {
                let outgoing = -hit.ray.direction;
                let normal = hit.normal;
                let bsdf = hit.object.bsdf();

                let mut bounce_illum = Spectrum::broadcast(0.0);

                if bounce == 0 || specular {
                    if let SceneObject::Emitter(e) = &hit.object {
                        bounce_illum += e.emission; //e.radiance(&outgoing, &normal);
                    }
                }

                bounce_illum += direct_illumination(scene, sampler, &hit, bsdf);

                illumination += throughput * bounce_illum;

                let sample = sampler.get_sample();
                if let Some(bxdf_sample) = bsdf.sample(normal, outgoing, Type::ALL, sample) {
                    if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum.is_black() {
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

            pixel.add(illumination);
        } else {
            pixel.add_black()
        }
    }
}
