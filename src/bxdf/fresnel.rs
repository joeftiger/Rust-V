#![allow(dead_code)]

use crate::Spectrum;
use color::Color;
use std::mem::swap;
use utility::floats::fast_clamp;

/// # Summary
/// Computes the fraction of reflected light for parallel polarized light.
///
/// # Arguments
/// * `cos_i` - The cosine of the angle between normal and incident
/// * `cos_t` - The cosine of the angle between normal and transmission
/// * `eta_i` - The index of refraction for the incident medium
/// * `eta_t` - The index of refraction for the transmission medium
///
/// # Returns
/// * The amount of light reflected
#[inline(always)]
pub fn dielectric_parallel(cos_i: f32, cos_t: f32, eta_i: f32, eta_t: f32) -> f32 {
    let it = eta_i * cos_t;
    let ti = eta_t * cos_i;

    (ti - it) / (ti + it)
}

/// # Summary
/// Computes the fraction of reflected light for perpendicular polarized light.
///
/// # Arguments
/// * `cos_i` - The cosine of the angle between normal and incident
/// * `cos_t` - The cosine of the angle between normal and transmission
/// * `eta_i` - The index of refraction for the incident medium
/// * `eta_t` - The index of refraction for the transmission medium
///
/// # Returns
/// * The amount of light reflected
#[inline(always)]
pub fn dielectric_perpendicular(cos_i: f32, cos_t: f32, eta_i: f32, eta_t: f32) -> f32 {
    let tt = eta_t * cos_t;
    let ii = eta_i * cos_i;

    (ii - tt) / (ii + tt)
}

/// # Summary
/// Computes the Fresnel reflection for dielectric materials and unpolarized light.
///
/// # Arguments
/// * `cos_i` - The cosine of the angle between normal and incident
/// * `eta_i` - The index of refraction for the incident medium
/// * `eta_t` - The index of refraction for the transmission medium
///
/// # Returns
/// * The Fresnel reflectance
pub fn fresnel_dielectric(mut cos_i: f32, mut eta_i: f32, mut eta_t: f32) -> f32 {
    cos_i = fast_clamp(cos_i, -1.0, 1.0);
    // potentially swap indices of refraction
    let entering = cos_i > 0.0;
    if !entering {
        swap(&mut eta_i, &mut eta_t);
        cos_i = cos_i.abs();
    }

    // compute cos_t using Snell's law
    let sin_i = f32::max(0.0, 1.0 - cos_i * cos_i).sqrt();
    let sin_t = eta_i * sin_i / eta_t;

    // handle total internal reflection
    if sin_t >= 1.0 {
        return 1.0;
    }

    let cos_t = f32::max(0.0, 1.0 - sin_t * sin_t).sqrt();
    let r_par = dielectric_parallel(cos_i, cos_t, eta_i, eta_t);
    let r_perp = dielectric_perpendicular(cos_i, cos_t, eta_i, eta_t);

    (r_par * r_par + r_perp * r_perp) / 2.0
}

/// # Summary
/// Provides an interface for computing Fresnel reflection coefficients.
pub trait Fresnel: Send + Sync {
    /// # Summary
    /// Evaluates the amount of light reflected by the surface.
    ///
    /// # Arguments
    /// * `cos_i` -The cosine of the angle between the normal and the incident
    ///
    /// # Returns
    /// * The reflectance
    fn evaluate(&self, cos_i: f32) -> Spectrum;
}

/// # Summary
/// An implementation of `Fresnel` for dielectric materials.
pub struct FresnelDielectric {
    eta_i: f32,
    eta_t: f32,
}

impl FresnelDielectric {
    /// # Summary
    /// Creates a new dielectric.
    ///
    /// # Arguments
    /// * `eta_i` - The index of refraction for the incident medium
    /// * `eta_t` - The index of refraction for the transmission medium
    ///
    /// # Returns
    /// * Self
    pub fn new(eta_i: f32, eta_t: f32) -> Self {
        Self { eta_i, eta_t }
    }
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cos_i: f32) -> Spectrum {
        Spectrum::new_const(fresnel_dielectric(cos_i, self.eta_i, self.eta_t))
    }
}

/// # Summary
/// A `Fresnel` implementation that returns 100% reflection for all incoming directions.
/// Although this is physically implausible, it is a convenient capability to have available.
pub struct FresnelNoOp;

impl Fresnel for FresnelNoOp {
    /// # Summary
    /// Returns full 100% reflection.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `1.0` spectrum
    fn evaluate(&self, _: f32) -> Spectrum {
        Spectrum::new_const(1.0)
    }
}
