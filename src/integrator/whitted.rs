use crate::integrator::{direct_illumination, Integrator};
use crate::objects::{ReceiverExt, SceneObject};
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Colors;

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
        let object = &intersection.object;
        let bsdf = object.bsdf();

        let mut illumination = Spectrum::black();

        if let SceneObject::Emitter(e) = object {
            illumination += e.emission(); //e.radiance(&outgoing, &normal);
        }

        illumination += direct_illumination(scene, sampler, intersection, bsdf);

        let new_depth = depth + 1;
        if new_depth < self.max_depth {
            illumination += self.specular_reflection(scene, intersection, sampler, new_depth);
            illumination += self.specular_transmission(scene, intersection, sampler, new_depth);
        }

        illumination
    }
}
