use crate::Spectrum;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SpectralSampler {
    /// Fully randomized samples
    Random,
    /// Hero wavelength sampling with a given index spread between samples
    Hero,
}

impl SpectralSampler {
    #[inline]
    pub fn fill_samples(&self, buf: &mut [usize]) {
        if buf.len() == Spectrum::size() {
            buf.iter_mut().enumerate().for_each(|(i, idx)| *idx = i);
            return;
        }

        match self {
            SpectralSampler::Random => buf
                .iter_mut()
                .for_each(|idx| *idx = fastrand::usize(0..Spectrum::size())),
            SpectralSampler::Hero => {
                let c = buf.len();
                let hero_index = fastrand::usize(0..Spectrum::size());

                buf.iter_mut()
                    .enumerate()
                    .for_each(|(j, idx)| *idx = Self::hero_rotation_function(c, j, hero_index));
            }
        }
    }

    /// The rotation function according to
    /// - authors: A. Wilkie & S. Nawaz & M. Droske & A. Weidlich & J. Hanika
    /// - paper: Hero Wavelength Spectral Sampling
    /// - year: 2014
    /// - page: 3
    /// - equation: 5
    #[inline]
    fn hero_rotation_function(c: usize, j: usize, hero_index: usize) -> usize {
        let j_c = j as f32 / c as f32;

        let spread = j_c * Spectrum::size() as f32;

        (hero_index + spread.round() as usize) % Spectrum::size()
    }
}
