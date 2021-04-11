#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::integrator::Integrator;
use crate::sampler::Sampler;
use crate::scene::{Scene, SceneIntersection};
use crate::sensor::pixel::Pixel;
use crate::Spectrum;
use color::{Color, Srgb};
use definitions::Vector3;
use geometry::Ray;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct DebugNormals;

impl Integrator for DebugNormals {
    fn integrate(
        &self,
        pixel: &mut Pixel,
        scene: &Scene,
        primary_ray: &Ray,
        sampler: &dyn Sampler,
    ) {
        if let Some(i) = scene.intersect(primary_ray) {
            let color = (i.normal + Vector3::one()) / 2.0;
            // Spectrum::try_from(color)
            //     .expect("Cannot parse Spectrum from Vector3")

            todo!()
        } else {
            pixel.add_black()
        }
    }
}
