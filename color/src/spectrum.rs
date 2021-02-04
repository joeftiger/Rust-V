#![allow(dead_code)]
use crate::*;

pub const LAMBDA_START: f32 = 400.0;
pub const LAMBDA_END: f32 = 700.0;
pub const SPECTRAL_SAMPLES: usize = 60;

pub struct Spectrum {
    spectrum: SpectrumData,
}

color!(
    SpectrumData => f32, f32, SPECTRAL_SAMPLES
);
