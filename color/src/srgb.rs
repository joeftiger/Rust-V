use crate::*;
use image::Rgb;
use ultraviolet::Vec3;

color!(
    Srgb => f32, 3
);

impl Srgb {
    /// Converts itself to a vector.
    ///
    /// # Returns
    /// * Self as vector
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::from(self.data)
    }

    pub fn to_xyz(&self) -> Xyz {
        Xyz::from(srgb_to_xyz_mat() * srgbs_to_linear(self.to_vec3()))
    }
}

impl IndexSpectral for Srgb {
    fn index_spectral(&self, index: usize) -> f32 {
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
        Self::new_const(0.0)
    }

    fn grey() -> Self {
        Self::new_const(0.5)
    }

    fn white() -> Self {
        Self::new_const(1.0)
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

impl Into<Rgb<u8>> for Srgb {
    fn into(self) -> Rgb<u8> {
        let mut data = [0; 3];
        data.iter_mut()
            .zip(self.data.iter())
            .for_each(|(d0, d1)| *d0 = (d1 * 2u32.pow(8) as f32) as u8);

        Rgb::from(data)
    }
}

impl Into<Rgb<u16>> for Srgb {
    fn into(self) -> Rgb<u16> {
        let mut data = [0; 3];
        data.iter_mut()
            .zip(self.data.iter())
            .for_each(|(d0, d1)| *d0 = (d1 * 2u32.pow(16) as f32) as u16);

        Rgb::from(data)
    }
}

impl Into<Rgb<f32>> for Srgb {
    fn into(self) -> Rgb<f32> {
        Rgb::from(self.data)
    }
}

impl From<Vec3> for Srgb {
    fn from(vec: Vec3) -> Self {
        Self::new([vec.x, vec.y, vec.z])
    }
}
