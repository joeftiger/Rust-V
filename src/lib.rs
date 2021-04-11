#[macro_use]
extern crate bitflags;

#[cfg(feature = "show-image")]
pub use window::RenderWindow;

pub mod bxdf;

pub mod camera;
mod debug_utils;
pub mod integrator;
pub mod mc;
pub mod objects;
pub mod sampler;
pub mod scene;

pub mod filters;
pub mod refractive_index;

pub mod config;
pub mod renderer;
pub mod sensor;
#[cfg(feature = "show-image")]
mod window;

pub type Spectrum = color::Spectrum;
