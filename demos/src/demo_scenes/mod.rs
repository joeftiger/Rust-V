mod cornell;
mod debug;
mod prism;
mod spheres;

use ultraviolet::UVec2;

pub use cornell::CornellScene;
pub use debug::DebugScene;
use definitions::Float;
pub use prism::PrismScene;
use rust_v::scene::Scene;
pub use spheres::SphereScene;

pub const SIGMA: Float = 20.0;
pub const FOVY: Float = 70.0;

pub trait DemoScene {
    // TODO: WIP
    fn create(resolution: UVec2) -> Scene;
}
