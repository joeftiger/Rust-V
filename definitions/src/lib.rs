pub use definition::*;

#[cfg(not(feature = "f64"))]
mod definition {
    use ultraviolet::{Mat3, Mat4, Rotor3, Vec2, Vec3};

    pub type Float = f32;
    pub type Vector2 = Vec2;
    pub type Vector3 = Vec3;
    pub type Rotation3 = Rotor3;
    pub type Matrix3 = Mat3;
    pub type Matrix4 = Mat4;
}

#[cfg(feature = "f64")]
mod definition {
    use ultraviolet::{DMat3, DMat4, DRotor3, DVec2, DVec3};

    pub type Float = f64;
    pub type Vector2 = DVec2;
    pub type Vector3 = DVec3;
    pub type Rotation3 = DRotor3;
    pub type Matrix3 = DMat3;
    pub type Matrix4 = DMat4;
}
