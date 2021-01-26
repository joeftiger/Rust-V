use crate::bxdf::BxDFType;
use crate::integrator::{direct_illumination, Integrator};
use crate::objects::{ReceiverExt, SceneObject};
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Color;
use geometry::offset_ray_towards;

pub struct Path {
    max_depth: u32,
}

impl Path {
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }
}

impl Integrator for Path {
    fn max_depth(&self) -> u32 {
        self.max_depth
    }

    fn illumination(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        _: u32,
    ) -> Spectrum {
        let mut illumination = Spectrum::black();
        let mut throughput = Spectrum::new_const(1.0);

        let mut hit = intersection.clone();
        let mut specular = false;

        for bounce in 0..self.max_depth {
            let outgoing = -hit.ray.direction;
            let normal = hit.normal;
            let mut bounce_illum = Spectrum::black();

            let bsdf = hit.object.bsdf();

            if bounce == 0 || specular {
                if let SceneObject::Emitter(e) = &hit.object {
                    bounce_illum += e.emission(); //e.radiance(&outgoing, &normal);
                }
            }

            bounce_illum += direct_illumination(scene, sampler, intersection, bsdf);

            illumination += throughput * bounce_illum;

            let sample = sampler.get_sample();
            if let Some(bxdf_sample) = bsdf.sample(&normal, &outgoing, BxDFType::ALL, &sample) {
                if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum.is_black() {
                    break;
                }

                specular = bxdf_sample.typ.is_specular();
                let cos = bxdf_sample.incident.dot(normal);

                if !specular && !bxdf_sample.typ.is_reflection() {
                    throughput *= bxdf_sample.spectrum * (cos.abs() / bxdf_sample.pdf);
                }

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
