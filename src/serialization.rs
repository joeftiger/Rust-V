use crate::camera::Camera;
use crate::config::Config;
use crate::integrator::Integrator;
use crate::samplers::Sampler;
use crate::scene::Scene;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Serialization {
    pub config: Config,
    pub camera: Box<dyn Camera>,
    pub integrator: Box<dyn Integrator>,
    pub sampler: Sampler,
    pub scene: Scene,
}
