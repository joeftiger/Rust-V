use crate::bxdf::BxDFType;
use crate::integrator::Integrator;
use crate::objects::{ReceiverExt, SceneObject};
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Color;
use utility::floats::fast_clamp;

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
            let outgoing = -hit.info.ray.direction;
            let normal = hit.info.normal;
            let mut bounce_illum = Spectrum::black();

            let bsdf = intersection.object.bsdf();

            if bounce == 0 || specular {
                if let SceneObject::Emitter(e) = &intersection.object {
                    bounce_illum += throughput * e.radiance(&outgoing, &normal);
                }
            }

            for light in &scene.lights {
                let emitter_sample = light.sample(&hit.info.point, &sampler.get_2d());

                if emitter_sample.pdf > 0.0
                    && !emitter_sample.radiance.is_black()
                    && !emitter_sample.occlusion_tester.test(scene)
                {
                    let c =
                        bsdf.evaluate(&normal, &emitter_sample.incident, &outgoing, BxDFType::ALL);

                    if !c.is_black() {
                        let cos = emitter_sample.incident.dot(normal);

                        if cos != 0.0 {
                            bounce_illum += light.emission()
                                * c
                                * emitter_sample.radiance
                                * (cos.abs() / emitter_sample.pdf)
                        }
                    }
                }
            }

            illumination += throughput * bounce_illum;

            let sample = sampler.get_sample();
            if let Some(bxdf_sample) = bsdf.sample(&normal, &outgoing, BxDFType::ALL, &sample) {
                if bxdf_sample.pdf == 0.0 || bxdf_sample.spectrum.is_black() {
                    break;
                }

                let dot = if bxdf_sample.typ.is_specular() {
                    specular = true;
                    1.0
                } else {
                    specular = false;
                    fast_clamp(bxdf_sample.incident.dot(normal).abs(), 0.0, 1.0)
                };

                throughput *= bxdf_sample.spectrum * (dot / bxdf_sample.pdf);

                let ray = hit.info.offset_ray_towards(bxdf_sample.incident);
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
