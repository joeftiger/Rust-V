use crate::bxdf::BxDFType;
use crate::integrator::Integrator;
use crate::objects::{ReceiverExt, SceneObject};
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Color;

/// # Summary
/// The Whitted integrator is a common integrator following specular reflection/transmission recursively.
#[derive(Clone)]
pub struct Whitted {
    depth: u32,
}

impl Whitted {
    /// # Summary
    /// Creates a new Whitted integrator.
    ///
    /// # Arguments
    /// * `depth` - The depth of recursive ray tracing
    ///
    /// # Returns
    /// * Self
    pub fn new(depth: u32) -> Self {
        Self { depth }
    }
}

impl Integrator for Whitted {
    fn illumination(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
    ) -> Spectrum {
        let outgoing = -intersection.info.ray.direction;

        let obj = &intersection.object;

        let bsdf = obj.bsdf();
        let point = intersection.info.point;
        let normal = intersection.info.normal;

        let mut illumination = Spectrum::black();

        if depth == 0 {
            if let SceneObject::Emitter(e) = obj {
                illumination += e.emission()
            }
        }

        for light in &scene.lights {
            let emitter_sample = light.sample(&point, &sampler.get_2d());

            if emitter_sample.pdf > 0.0
                && !emitter_sample.radiance.is_black()
                && !emitter_sample.occlusion_tester.test(scene)
            {
                let c = bsdf.evaluate(&normal, &emitter_sample.incident, &outgoing, BxDFType::ALL);

                if !c.is_black() {
                    let cos = emitter_sample.incident.dot(normal);

                    if cos != 0.0 {
                        illumination += light.emission()
                            * c
                            * emitter_sample.radiance
                            * (cos.abs() / emitter_sample.pdf)
                    }
                }
            }
        }

        let new_depth = depth + 1;

        if new_depth < self.depth {
            illumination += self.specular_reflection(scene, intersection, sampler, new_depth);
            illumination += self.specular_transmission(scene, intersection, sampler, new_depth);
        }

        illumination
    }
}