use crate::*;
use image::Rgb;
use ultraviolet::Vec3;

color!(
    Xyz => f32, f32, 3
);

impl Xyz {
    /// Converts itself to a vector.
    ///
    /// # Returns
    /// * Self as vector
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::from(self.data)
    }

    pub fn to_srgb(&self) -> Srgb {
        Srgb::from(linears_to_srgb(xyz_to_srgb_mat() * self.to_vec3()))
    }
}

impl Colors for Xyz {
    fn black() -> Self {
        Srgb::black().to_xyz()
    }

    fn grey() -> Self {
        Srgb::grey().to_xyz()
    }

    fn white() -> Self {
        Srgb::white().to_xyz()
    }

    fn red() -> Self {
        Srgb::red().to_xyz()
    }

    fn yellow() -> Self {
        Srgb::yellow().to_xyz()
    }

    fn green() -> Self {
        Srgb::green().to_xyz()
    }

    fn cyan() -> Self {
        Srgb::cyan().to_xyz()
    }

    fn blue() -> Self {
        Srgb::blue().to_xyz()
    }

    fn pink() -> Self {
        Srgb::pink().to_xyz()
    }
}

impl Into<Rgb<u8>> for Xyz {
    fn into(self) -> Rgb<u8> {
        self.to_srgb().into()
    }
}

impl Into<Rgb<u16>> for Xyz {
    fn into(self) -> Rgb<u16> {
        self.to_srgb().into()
    }
}

impl Into<Rgb<f32>> for Xyz {
    fn into(self) -> Rgb<f32> {
        Rgb::from(self.data)
    }
}

impl From<Vec3> for Xyz {
    fn from(vec: Vec3) -> Self {
        Self::new([vec.x, vec.y, vec.z])
    }
}
