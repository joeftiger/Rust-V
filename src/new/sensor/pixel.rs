use crate::Spectrum;
use color::{IntSpectrum, Color};

pub struct Pixel<T> {
    pub average: Spectrum,
    samples: T,
}

impl Pixel<usize> {
    pub fn add(&mut self, spectrum: Spectrum) {
        let mut avg = self.average * self.samples as f32;
        avg += spectrum;
        self.samples += 1;

        self.average = avg / self.samples as f32;
    }
}

impl Pixel<IntSpectrum> {
    pub fn add(&mut self, spectrum: Spectrum) {
        let mut avg = self.average * self.samples;
        avg += spectrum;
        self.samples.increment();

        self.average = avg / self.samples;
    }

    pub fn add_light_wave(&mut self, lambda: f32, light_wave_index: usize) {
        let mut avg = self.average * self.samples;
        avg[light_wave_index] += lambda;
        self.samples[light_wave_index] += 1;

        self.average = avg / self.samples;
    }
}

impl<T> Default for Pixel<T>
    where
        T: Default,
{
    fn default() -> Self {
        Self {
            average: Spectrum::default(),
            samples: T::default(),
        }
    }
}