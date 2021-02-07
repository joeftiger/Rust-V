use crate::bxdf::BxDFType;
use crate::integrator::{direct_illumination, Integrator};
use crate::objects::{ReceiverExt, SceneObject};
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::{Color, Colors};
use geometry::offset_ray_towards;

/// The Whitted integrator is a common integrator following specular reflection/transmission recursively.
#[derive(Clone)]
pub struct Whitted {
    max_depth: u32,
}

impl Whitted {
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

    /// Computes a specific `BxDFType` at the given scene intersection, calling
    /// `illumination()` with a newly generated reflected ray.
    ///
    /// # Arguments
    /// * `scene` - The scene being integratd
    /// * `intersection` - The scene intersection we illuminate
    /// * `sampler` - A sampler to generate values
    /// * `depth` - The current recursive depth (if needed)
    /// * `typ` - The typ to sample for
    ///
    /// # Returns
    /// * The illumination at the intersection
    fn illumination_target(
        &self,
        scene: &Scene,
        intersection: &SceneIntersection,
        sampler: &dyn Sampler,
        depth: u32,
        typ: BxDFType,
    ) -> Spectrum {
        debug_assert!(depth < self.max_depth());

        let outgoing = -intersection.ray.direction;

        let bsdf = intersection.object.bsdf();
        let normal = intersection.normal;
        let sample = sampler.get_sample();

        let bxdf_sample_option = bsdf.sample(&normal, &outgoing, typ, &sample);

        let mut reflection = Spectrum::black();

        if let Some(bxdf_sample) = bxdf_sample_option {
            if bxdf_sample.pdf > 0.0 && !bxdf_sample.spectrum.is_black() {
                let cos = bxdf_sample.incident.dot(normal);

                if cos != 0.0 {
                    let refl_ray = offset_ray_towards(
                        intersection.point,
                        intersection.normal,
                        bxdf_sample.incident,
                    );

                    if let Some(si) = scene.intersect(&refl_ray) {
                        let illumination = self.illumination(scene, &si, sampler, depth);
                        reflection +=
                            illumination * bxdf_sample.spectrum * (cos.abs() / bxdf_sample.pdf);
                    }
                }
            }
        }

        reflection
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
        let object = &intersection.object;
        let bsdf = object.bsdf();

        let mut illumination = Spectrum::black();

        if let SceneObject::Emitter(e) = object {
            illumination += e.emission(); //e.radiance(&outgoing, &normal);
        }

        illumination += direct_illumination(scene, sampler, intersection, bsdf);

        let new_depth = depth + 1;
        if new_depth < self.max_depth {
            let reflection = BxDFType::SPECULAR | BxDFType::REFLECTION;
            let transmission = BxDFType::SPECULAR | BxDFType::TRANSMISSION;
            let both = reflection | transmission;
            illumination +=
                self.illumination_target(scene, intersection, sampler, new_depth, reflection);
            illumination +=
                self.illumination_target(scene, intersection, sampler, new_depth, transmission);
            illumination += self.illumination_target(scene, intersection, sampler, new_depth, both);
        }

        illumination
    }
}
