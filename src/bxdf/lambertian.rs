use crate::bxdf::{BxDF, Type};
use crate::Spectrum;
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_1_PI;
use ultraviolet::Vec3;

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

impl BxDF for LambertianReflection {
    fn get_type(&self) -> Type {
        Type::DIFFUSE | Type::REFLECTION
    }

    fn evaluate(&self, _: &Vec3, _: &Vec3) -> Spectrum {
        self.r * FRAC_1_PI
    }

    fn evaluate_light_wave(&self, _: &Vec3, _: &Vec3, light_wave_index: usize) -> f32 {
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

impl BxDF for LambertianTransmission {
    fn get_type(&self) -> Type {
        Type::DIFFUSE | Type::TRANSMISSION
    }

    fn evaluate(&self, _: &Vec3, _: &Vec3) -> Spectrum {
        self.t * FRAC_1_PI
    }

    fn evaluate_light_wave(&self, _: &Vec3, _: &Vec3, light_wave_index: usize) -> f32 {
        self.t[light_wave_index] * FRAC_1_PI
    }
}
