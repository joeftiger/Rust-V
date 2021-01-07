mod cornell;
mod debug;
mod spheres;

use crate::scene::Scene;
use ultraviolet::UVec2;

use crate::camera::Camera;
pub use cornell::CornellScene;
pub use debug::DebugScene;
pub use spheres::SphereScene;
use std::sync::Arc;

pub const SIGMA: f32 = 20.0;
pub const FOVY: f32 = 70.0;

pub trait DemoScene {
    // TODO: WIP
    fn create(resolution: UVec2) -> (Scene, Arc<dyn Camera>);
}
