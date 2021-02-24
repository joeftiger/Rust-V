#![allow(dead_code)]

use crate::spectral_data::{
    blue, green, red, white, yellow, LAMBDA_NUM, LAMBDA_START, LAMBDA_STEP,
};
use crate::*;
use std::iter::{Enumerate, Map};
use std::slice::Iter;

impl Spectrum {
    #[inline(always)]
    pub fn get(&self, index: usize) -> LightWave {
        let lambda = LAMBDA_START + (LAMBDA_STEP * index) as f32;
        let intensity = self[index];

        LightWave::new(lambda, intensity)
    }

    pub fn get_iter(&self) -> Map<Enumerate<Iter<'_, f32>>, fn((usize, &f32)) -> LightWave> {
        self.data.iter().enumerate().map(|(index, intensity)| {
            let lambda = LAMBDA_START + (LAMBDA_STEP * index) as f32;

            LightWave::new(lambda, *intensity)
        })
    }
}

impl Colors for Spectrum {
    fn black() -> Self {
        Self::new_const(0.0)
    }

    fn grey() -> Self {
        Self::white() * 0.5
    }

    fn white() -> Self {
        Self::new(white::SPECTRUM)
    }

    fn red() -> Self {
        Self::new(red::SPECTRUM)
    }

    fn yellow() -> Self {
        Self::new(yellow::SPECTRUM)
    }

    fn green() -> Self {
        Self::new(green::SPECTRUM)
    }

    fn cyan() -> Self {
        (Self::green() + Self::blue()) * 0.5
    }

    fn blue() -> Self {
        Self::new(blue::SPECTRUM)
    }

    fn pink() -> Self {
        (Self::red() + Self::blue()) * 0.5
    }
}

color!(
    Spectrum => f32, LAMBDA_NUM
);
