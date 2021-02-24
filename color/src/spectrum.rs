#![allow(dead_code)]
use crate::*;

pub const LAMBDA_START: f32 = 400.0;
pub const LAMBDA_END: f32 = 700.0;
pub const SPECTRAL_LENGTH: f32 = LAMBDA_END - LAMBDA_START;
pub const SPECTRAL_STEP: f32 = 10.0; //LEN / SPECTRAL_SAMPLES as f32;
pub const SPECTRAL_SAMPLES: usize = (SPECTRAL_LENGTH / SPECTRAL_STEP) as usize;

/// A light wave is described by a wavelength (lambda) and an intensity (associated with amplitude).
#[derive(Copy, Clone, Default)]
pub struct LightWave {
    /// The wavelength in μm.
    pub lambda: f32,
    /// The intensity of the light wave.
    pub intensity: f32,
}

impl LightWave {
    pub fn new(lambda: f32, intensity: f32) -> Self {
        Self { lambda, intensity }
    }

    pub fn with_intensity(&self, intensity: f32) -> Self {
        Self::new(self.lambda, intensity)
    }
}

impl Spectrum {
    #[inline(always)]
    pub fn get(&self, index: usize) -> LightWave {
        let lambda = LAMBDA_START + SPECTRAL_STEP * index as f32;
        let intensity = self[index];

        LightWave::new(lambda, intensity)
    }
}

color!(
    Spectrum => f32, SPECTRAL_SAMPLES
);

// const DUPONT_RED_27: [f32; SPECTRAL_SAMPLES] = {
//     [
//         1.0,
//         (1.0 + 0.91) / 2.0,
//         0.91,
//         (0.91 + 0.89) / 2.0,
//         0.89,
//         0.87,
//         0.86,
//         0.86,
//         0.85,
//         0.85,
//         0.84,
//         0.85,
//         0.87,
//         0.91,
//         0.91,
//         0.9,
//         0.92,
//         1.07,
//         1.34,
//         1.79,
//         3.15,
//         7.09,
//         16.49,
//         30.31,
//         41.06,
//         46.57,
//         49.08,
//         50.45,
//     ]
// };
