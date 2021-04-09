#[macro_use]
extern crate bitflags;

pub mod bxdf;
mod configuration;

pub mod camera;
mod debug_utils;
pub mod grid;
pub mod integrator;
pub mod mc;
pub mod objects;
// pub mod renderer;
pub mod sampler;
pub mod scene;

pub use configuration::RenderConfig;

pub mod filters;
pub mod new;
pub mod refractive_index;

#[cfg(feature = "show-image")]
mod window;

#[cfg(feature = "show-image")]
pub use window::RenderWindow;

pub type Spectrum = color::Spectrum;
