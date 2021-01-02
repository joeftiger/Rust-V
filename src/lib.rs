#[macro_use]
extern crate bitflags;

use color::Srgb;

pub mod bxdf;
pub mod camera;
pub mod configuration;
#[cfg(debug_assertions)]
mod debug_utils;
pub mod grid;
pub mod integrator;
pub mod mc;
pub mod objects;
pub mod renderer;
pub mod sampler;
pub mod scene;

pub type Spectrum = Srgb;
