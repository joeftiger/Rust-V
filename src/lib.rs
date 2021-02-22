#[macro_use]
extern crate bitflags;

use color::Srgb;

pub mod bxdf;
mod configuration;

pub mod camera;
mod debug_utils;
pub mod demo_scenes;
pub mod grid;
pub mod integrator;
pub mod mc;
pub mod objects;
pub mod renderer;
pub mod sampler;
pub mod scene;

pub use configuration::RenderConfig;

pub mod filters;
pub mod light;
pub mod refractive_index;
pub mod sensor;
#[cfg(feature = "live-window")]
mod window;

#[cfg(feature = "live-window")]
pub use window::RenderWindow;

pub type Spectrum = Srgb;
