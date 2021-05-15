use crate::*;
use image::Rgb;

color!(
    Srgb => 3, color_data::srgb
);

impl TryFrom<SerdeColors> for Srgb {
    type Error = ();

    fn try_from(value: SerdeColors) -> Result<Self, Self::Error> {
        let srgb = match value {
            SerdeColors::Srgb(data) => Self::new(data),
            SerdeColors::Xyz(data) => Xyz::new(data).into(),
            SerdeColors::Spectrum(data) => Spectrum::new(data).into(),
            SerdeColors::Color(c) => Self::from(c),
            SerdeColors::Constant(c) => Self::broadcast(c),
        };

        Ok(srgb)
    }
}

impl From<Srgb> for Rgb<u8> {
    fn from(srgb: Srgb) -> Self {
        let conv = srgb * Srgb::broadcast(2u32.pow(16) as Float);
        let data = [conv[0] as u8, conv[1] as u8, conv[2] as u8];

        Self::from(data)
    }
}

impl From<Srgb> for Rgb<u16> {
    fn from(srgb: Srgb) -> Self {
        let conv = srgb * Srgb::broadcast(2u32.pow(16) as Float);
        let data = [conv[0] as u16, conv[1] as u16, conv[2] as u16];

        Self::from(data)
    }
}

impl From<Srgb> for Rgb<Float> {
    fn from(srgb: Srgb) -> Self {
        Self::from(srgb.data)
    }
}

impl From<Srgb> for Xyz {
    #[allow(clippy::excessive_precision)]
    #[allow(clippy::many_single_char_names)]
    fn from(srgb: Srgb) -> Self {
        let r = uncompand(srgb[0]);
        let g = uncompand(srgb[1]);
        let b = uncompand(srgb[2]);

        let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
        let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
        let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;

        Self::new([x, y, z])
    }
}

#[allow(clippy::excessive_precision)]
#[inline]
fn uncompand(val: Float) -> Float {
    // https://entropymine.com/imageworsener/srgbformula/
    if val <= 0.0404482362771082 {
        val / 12.92
    } else {
        ((val + 0.055) / 1.055).powf(2.4)
    }
}
