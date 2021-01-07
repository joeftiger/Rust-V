#![allow(dead_code)]
#![allow(unused_variables)]

use crate::camera::Camera;
use crate::demo_scenes::DemoScene;
use crate::scene::Scene;
use std::sync::Arc;
use ultraviolet::UVec2;

pub struct SphereScene;

impl DemoScene for SphereScene {
    fn create(resolution: UVec2) -> (Scene, Arc<dyn Camera>) {
        unimplemented!()
    }
}
