use crate::cie::xyz_of;
use crate::spectral_data::*;
use crate::*;
use image::Rgb;

color!(
    Spectrum => Float, LAMBDA_NUM
);

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

impl IndexSpectral<Float> for Spectrum {
    fn index_spectral(&self, index: usize) -> Float {
        self.data[index]
    }
}

impl Colors for Spectrum {
    fn black() -> Self {
        Self::broadcast(0.0)
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

impl From<Spectrum> for Srgb {
    fn from(spectrum: Spectrum) -> Self {
        Srgb::from(Xyz::from(spectrum))
    }
}

impl From<Spectrum> for Xyz {
    fn from(spectrum: Spectrum) -> Self {
        spectrum
            .as_light_waves()
            .iter()
            .fold(Xyz::broadcast(0.0), |acc, next| {
                acc + xyz_of(next.lambda) * next.intensity
            })
    }
}

impl From<Spectrum> for Rgb<u8> {
    fn from(spectrum: Spectrum) -> Self {
        Self::from(Srgb::from(spectrum))
    }
}

impl From<Spectrum> for Rgb<u16> {
    fn from(spectrum: Spectrum) -> Self {
        Self::from(Srgb::from(spectrum))
    }
}

impl From<Spectrum> for Rgb<Float> {
    fn from(spectrum: Spectrum) -> Self {
        Self::from(Srgb::from(spectrum))
    }
}
