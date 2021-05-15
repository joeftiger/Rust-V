use crate::*;
use image::Rgb;

color!(
    Xyz => 3, color_data::xyz
);

impl TryFrom<SerdeColors> for Xyz {
    type Error = ();

    fn try_from(value: SerdeColors) -> Result<Self, Self::Error> {
        let xyz = match value {
            SerdeColors::Srgb(data) => Srgb::new(data).into(),
            SerdeColors::Xyz(data) => Self::new(data),
            SerdeColors::Spectrum(data) => Spectrum::new(data).into(),
            SerdeColors::Color(c) => Self::from(c),
            SerdeColors::Constant(c) => Self::broadcast(c),
        };

        Ok(xyz)
    }
}

impl From<Xyz> for Rgb<u8> {
    fn from(xyz: Xyz) -> Self {
        Self::from(Srgb::from(xyz))
    }
}

impl From<Xyz> for Rgb<u16> {
    fn from(xyz: Xyz) -> Self {
        Self::from(Srgb::from(xyz))
    }
}

impl From<Xyz> for Rgb<Float> {
    fn from(xyz: Xyz) -> Self {
        Self::from(Srgb::from(xyz))
    }
}

impl From<Xyz> for Srgb {
    #[rustfmt::skip]
    #[allow(clippy::excessive_precision)]
    #[allow(clippy::many_single_char_names)]
    fn from(xyz: Xyz) -> Self {
        let x = xyz[0];
        let y = xyz[1];
        let z = xyz[2];

        let r =  3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
        let g = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
        let b =  0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

        Self::new([compand(r), compand(g), compand(b)])
    }
}

#[allow(clippy::excessive_precision)]
#[inline]
fn compand(val: Float) -> Float {
    // https://entropymine.com/imageworsener/srgbformula/
    if val <= 0.00313066844250063 {
        val * 12.92
    } else {
        1.055 * val.powf(1.0 / 2.4) - 0.055
    }
}
