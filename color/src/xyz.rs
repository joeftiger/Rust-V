use crate::*;
use image::Rgb;

color!(
    Xyz => Float, 3
);

impl Colors for Xyz {
    fn black() -> Self {
        Self::from(Srgb::black())
    }

    fn grey() -> Self {
        Self::from(Srgb::grey())
    }

    fn white() -> Self {
        Self::from(Srgb::white())
    }

    fn red() -> Self {
        Self::from(Srgb::red())
    }

    fn yellow() -> Self {
        Self::from(Srgb::yellow())
    }

    fn green() -> Self {
        Self::from(Srgb::grey())
    }

    fn cyan() -> Self {
        Self::from(Srgb::cyan())
    }

    fn blue() -> Self {
        Self::from(Srgb::black())
    }

    fn pink() -> Self {
        Self::from(Srgb::pink())
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
    fn from(xyz: Xyz) -> Self {
        Srgb::from(linears_to_srgb(xyz_to_srgb_mat() * Vector3::from(xyz)))
    }
}

impl From<Xyz> for Vector3 {
    fn from(xyz: Xyz) -> Self {
        Self::from(xyz.data)
    }
}

impl From<Vector3> for Xyz {
    fn from(vec: Vector3) -> Self {
        Self::new([vec.x, vec.y, vec.z])
    }
}
