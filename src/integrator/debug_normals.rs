use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Color;
use geometry::Ray;
use ultraviolet::Vec3;

pub struct DebugNormals;

impl Integrator for DebugNormals {
    fn integrate<'a>(&self, scene: &Scene<'a>, primary_ray: &Ray, _: &dyn Sampler) -> Spectrum {
        if let Some(si) = scene.intersect(primary_ray) {
            let color = (si.info.normal + Vec3::one()) / 2.0;

            color.into()
        } else {
            Spectrum::black()
        }
    }

    fn illumination<'a>(
        &self,
        _: &Scene<'a>,
        _: &SceneIntersection<'a>,
        _: &dyn Sampler,
        _: usize,
    ) -> Spectrum {
        unreachable!()
    }
}
