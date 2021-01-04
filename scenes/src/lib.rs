use rust_v::camera::Camera;
use rust_v::scene::Scene;
use ultraviolet::UVec2;

pub trait DemoScene {
    // TODO: WIP
    fn create<'a>(resolution: UVec2) -> (Scene<'a>, dyn Camera);
}
