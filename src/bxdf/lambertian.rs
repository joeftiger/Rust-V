use crate::bxdf::{BxDF, Type};
use crate::*;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "f64"))]
use std::f32::consts::FRAC_1_PI;
#[cfg(feature = "f64")]
use std::f64::consts::FRAC_1_PI;

/// The lambertian reflection reflects equally into all directions of the hemisphere.
#[derive(Serialize, Deserialize)]
pub struct LambertianReflection {
    r: Spectrum,
}

impl LambertianReflection {
    /// # Summmary
    /// Creates a new lambertian reflection.
    ///
    /// # Arguments
    /// * `r` - The reflection spectrum
    ///
    /// # Returns
    /// * Self
    pub fn new(r: Spectrum) -> Self {
        Self { r }
    }
}

#[typetag::serde]
impl BxDF for LambertianReflection {
    fn get_type(&self) -> Type {
        Type::DIFFUSE | Type::REFLECTION
    }

    fn evaluate(&self, _: Vector3, _: Vector3) -> Spectrum {
        self.r * FRAC_1_PI
    }

    #[inline]
    fn evaluate_wavelength(&self, _: Vector3, _: Vector3, light_wave_index: usize) -> Float {
        self.r[light_wave_index] * FRAC_1_PI
    }
}

/// The lambertian transmission transmits equally into all directions of the hemisphere.
#[derive(Serialize, Deserialize)]
pub struct LambertianTransmission {
    t: Spectrum,
}

impl LambertianTransmission {
    /// Creates a new lambertian transmission.
    ///
    /// # Arguments
    /// * `t` - The transmission spectrum
    ///
    /// # Returns
    /// * Self
    pub fn new(t: Spectrum) -> Self {
        Self { t }
    }
}

#[typetag::serde]
impl BxDF for LambertianTransmission {
    fn get_type(&self) -> Type {
        Type::DIFFUSE | Type::TRANSMISSION
    }

    fn evaluate(&self, _: Vector3, _: Vector3) -> Spectrum {
        self.t * FRAC_1_PI
    }

    #[inline]
    fn evaluate_wavelength(&self, _: Vector3, _: Vector3, light_wave_index: usize) -> Float {
        self.t[light_wave_index] * FRAC_1_PI
    }
}
