mod cornell_box;
mod spheres;

use rust_v::camera::Camera;
use rust_v::scene::Scene;
use ultraviolet::UVec2;

pub use cornell_box::CornellBox;
pub use spheres::SphereScene;

pub const SIGMA: f32 = 20.0;
pub const FOVY: f32 = 70.0;

pub trait DemoScene {
    // TODO: WIP
    fn create<'a>(resolution: UVec2) -> (Scene<'a>, Box<dyn Camera>);
}
