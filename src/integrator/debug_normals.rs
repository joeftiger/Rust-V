use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use ultraviolet::Vec3;

#[derive(Clone)]
pub struct DebugNormals;

impl Integrator for DebugNormals {
    fn illumination(
        &self,
        _: &Scene,
        intersection: &SceneIntersection,
        _: &dyn Sampler,
        _: u32,
    ) -> Spectrum {
        let color = (intersection.normal + Vec3::one()) / 2.0;
        color.into()
    }
}
