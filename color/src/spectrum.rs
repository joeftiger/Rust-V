#![allow(dead_code)]

use crate::cie::xyz_of;
use crate::spectral_data::*;
use crate::*;
use image::Rgb;

impl Spectrum {
    pub fn as_xyz(self) -> Xyz {
        self.into()
    }

    pub fn as_srgb(self) -> Srgb {
        self.into()
    }
}

impl Spectrum {
    pub fn blue_sky() -> Self {
        Self::new(blue_sky::SPECTRUM)
    }

    pub fn dark_skin() -> Self {
        Self::new(dark_skin::SPECTRUM)
    }

    pub fn foliage() -> Self {
        Self::new(foliage::SPECTRUM)
    }

    pub fn light_skin() -> Self {
        Self::new(light_skin::SPECTRUM)
    }
}

impl IndexSpectral for Spectrum {
    fn index_spectral(&self, index: usize) -> f32 {
        self.data[index]
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

impl Into<Xyz> for Spectrum {
    fn into(self) -> Xyz {
        self.as_light_waves()
            .iter()
            .fold(Xyz::new_const(0.0), |acc, next| {
                acc + xyz_of(next.lambda) * next.intensity
            })
    }
}

impl Into<Srgb> for Spectrum {
    fn into(self) -> Srgb {
        self.as_xyz().to_srgb()
    }
}

impl Into<Rgb<u8>> for Spectrum {
    fn into(self) -> Rgb<u8> {
        self.as_srgb().into()
    }
}

impl Into<Rgb<u16>> for Spectrum {
    fn into(self) -> Rgb<u16> {
        self.as_srgb().into()
    }
}

impl Into<Rgb<f32>> for Spectrum {
    fn into(self) -> Rgb<f32> {
        self.as_srgb().into()
    }
}

color!(
    Spectrum => f32, LAMBDA_NUM
);
