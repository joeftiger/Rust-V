#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::Spectrum;
use color::Color;
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
        Spectrum::new_const(0.0)
        // let color = (intersection.normal + Vec3::one()) / 2.0;
        // color.into()
    }
}
