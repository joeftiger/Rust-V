use crate::Float;
use crate::Spectrum;
use color::{Color, IntSpectrum};
use ultraviolet::UVec2;

#[derive(Default, Clone)]
pub struct Pixel {
    pub position: UVec2,
    pub average: Spectrum,
    samples: IntSpectrum<{ Spectrum::size() }>,
}

impl Pixel {
    pub fn new(position: UVec2) -> Self {
        Self {
            position,
            average: Spectrum::broadcast(0.0),
            samples: IntSpectrum::broadcast(0),
        }
    }

    pub fn add(&mut self, spectrum: Spectrum) {
        let mut avg = self.average * self.samples;
        avg += spectrum;
        self.samples.increment();

        self.average = avg / self.samples;
    }

    pub fn add_black(&mut self) {
        let avg = self.average * self.samples;
        self.samples.increment();
        self.average = avg / self.samples;
    }

    pub fn add_light_wave(&mut self, lambda: Float, light_wave_index: usize) {
        let before = self.samples[light_wave_index];
        let after = self.samples[light_wave_index] + 1;

        self.average[light_wave_index] =
            (self.average[light_wave_index] * before as Float + lambda) / after as Float;
        self.samples[light_wave_index] = after;
    }

    pub fn add_black_light_wave(&mut self, light_wave_index: usize) {
        let before = self.samples[light_wave_index];
        let after = self.samples[light_wave_index] + 1;

        self.average[light_wave_index] *= before as Float / after as Float;
        self.samples[light_wave_index] = after;
    }
}
