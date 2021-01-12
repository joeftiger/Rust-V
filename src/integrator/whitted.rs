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
    max_depth: u32,
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
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }
}

impl Integrator for Whitted {
    fn max_depth(&self) -> u32 {
        self.max_depth
    }

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

        if let SceneObject::Emitter(e) = obj {
            illumination += e.radiance(&outgoing, &normal);
        }

        for light in &scene.lights {
            let emitter_sample = light.sample(&point, &sampler.get_2d());

            if emitter_sample.pdf > 0.0
                && !emitter_sample.radiance.is_black()
                && !emitter_sample.occlusion_tester.test(scene)
            {
                let bsdf_spectrum =
                    bsdf.evaluate(&normal, &emitter_sample.incident, &outgoing, BxDFType::ALL);

                if !bsdf_spectrum.is_black() {
                    let cos = emitter_sample.incident.dot(normal);

                    if cos != 0.0 {
                        illumination += bsdf_spectrum
                            * emitter_sample.radiance
                            * (cos.abs() / emitter_sample.pdf)
                    }
                }
            }
        }

        let new_depth = depth + 1;
        if new_depth < self.max_depth {
            illumination += self.specular_reflection(scene, intersection, sampler, new_depth);
            illumination += self.specular_transmission(scene, intersection, sampler, new_depth);
        }

        illumination
    }
}
