use crate::*;
use image::Rgb;

color!(
    Srgb => Float, 3
);

impl IndexSpectral<Float> for Srgb {
    fn index_spectral(&self, index: usize) -> Float {
        match index {
            0 => 0.645,
            1 => 0.526,
            2 => 0.444,
            _ => panic!(),
        }
    }
}

impl Colors for Srgb {
    fn black() -> Self {
        Self::broadcast(0.0)
    }

    fn grey() -> Self {
        Self::broadcast(0.5)
    }

    fn white() -> Self {
        Self::broadcast(1.0)
    }

    fn red() -> Self {
        Self::new([1.0, 0.0, 0.0])
    }

    fn yellow() -> Self {
        Self::new([1.0, 1.0, 0.0])
    }

    fn green() -> Self {
        Self::new([0.0, 1.0, 0.0])
    }

    fn cyan() -> Self {
        Self::new([0.0, 1.0, 1.0])
    }

    fn blue() -> Self {
        Self::new([0.0, 0.0, 1.0])
    }

    fn pink() -> Self {
        Self::new([1.0, 0.0, 1.0])
    }
}

impl From<Srgb> for Rgb<u8> {
    fn from(srgb: Srgb) -> Self {
        let mut data = [0; 3];
        data.iter_mut()
            .zip(srgb.data.iter())
            .for_each(|(d0, d1)| *d0 = (d1 * 2u32.pow(8) as Float) as u8);

        Self::from(data)
    }
}

impl From<Srgb> for Rgb<u16> {
    fn from(srgb: Srgb) -> Self {
        let mut data = [0; 3];
        data.iter_mut()
            .zip(srgb.data.iter())
            .for_each(|(d0, d1)| *d0 = (d1 * 2u32.pow(16) as Float) as u16);

        Self::from(data)
    }
}

impl From<Srgb> for Rgb<Float> {
    fn from(srgb: Srgb) -> Self {
        Self::from(srgb.data)
    }
}

impl From<Srgb> for Xyz {
    fn from(srgb: Srgb) -> Self {
        Xyz::from(srgb_to_xyz_mat() * srgbs_to_linear(Vector3::from(srgb)))
    }
}

impl From<Srgb> for Vector3 {
    fn from(srgb: Srgb) -> Self {
        Self::from(srgb.data)
    }
}

impl From<Vector3> for Srgb {
    fn from(vec: Vector3) -> Self {
        Self::new([vec.x, vec.y, vec.z])
    }
}
