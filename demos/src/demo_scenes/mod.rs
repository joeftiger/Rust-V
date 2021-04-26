mod cornell;
mod debug;
mod prism;
mod spheres;

use ultraviolet::UVec2;

pub use cornell::CornellScene;
pub use debug::DebugScene;
use definitions::Float;
pub use prism::PrismScene;
use rust_v::camera::PerspectiveCamera;
use rust_v::config::Config;
use rust_v::integrator::{Integrator, Whitted};
use rust_v::samplers::Sampler;
use rust_v::scene::Scene;
use rust_v::sensor::bounds::UBounds2;
use rust_v::serialization::Serialization;
pub use spheres::SphereScene;
use std::sync::Arc;

pub const SIGMA: Float = 20.0;
pub const FOVY: Float = 70.0;

pub trait Demo {
    // TODO: WIP
    fn create() -> Serialization;

    fn empty() -> (UVec2, Config, Arc<dyn Integrator>, Sampler, Scene) {
        let resolution = UVec2::new(512, 512);
        let config = Config {
            filename: None,
            bounds: None,
            block_size: UVec2::broadcast(8),
            passes: 100,
            threads: None,
        };

        let integrator = Arc::new(Whitted::new(8));
        let sampler = Sampler::Random;
        let scene = Scene::default();

        (resolution, config, integrator, sampler, scene)
    }
}
