mod bsdf;
mod fresnel;
mod lambertian;
mod oren_nayar;
mod specular;

pub use bsdf::BSDF;

pub use fresnel::{Fresnel, FresnelDielectric, FresnelNoOp};
pub use lambertian::{LambertianReflection, LambertianTransmission};
pub use oren_nayar::OrenNayar;
pub use specular::{FresnelSpecular, SpecularReflection, SpecularTransmission};

use crate::debug_utils::{is_finite, is_normalized, within_01};
use crate::mc::sample_unit_hemisphere;
use crate::Spectrum;
use color::LightWave;
use std::f32::consts::{FRAC_1_PI, PI};
use ultraviolet::{Rotor3, Vec2, Vec3};

/// Allows indicating whether an intersection was found along a path starting from a camera or one
/// starting from a light source.
///
/// This has implications on the calculations of `BSDF`.
#[derive(PartialEq)]
pub enum TransportMode {
    Radiance,
    Importance,
}

/// The BxDF normal is defined in the y-axis of the world space.
///
/// # Returns
/// * The global BxDF normal
#[inline(always)]
pub fn bxdf_normal() -> Vec3 {
    Vec3::unit_y()
}

#[inline(always)]
pub fn bxdf_incident_to(v: &Vec3) -> Vec3 {
    debug_assert!(is_finite(v));

    Vec3::new(-v.x, v.y, -v.z)
}

#[inline(always)]
pub fn is_neg(v: &Vec3) -> bool {
    debug_assert!(is_finite(v));

    v.y < 0.0
}

#[inline(always)]
pub fn flip_if_neg(mut v: Vec3) -> Vec3 {
    debug_assert!(is_finite(&v));

    if is_neg(&v) {
        v.y = -v.y;
    }
    v
}

#[inline(always)]
pub fn bxdf_is_parallel(v: &Vec3) -> bool {
    debug_assert!(is_finite(v));

    v.y == 0.0
}

#[inline(always)]
pub fn cos_theta(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    v.y
}

#[inline(always)]
pub fn cos2_theta(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    cos_theta(v) * cos_theta(v)
}

#[inline(always)]
pub fn sin2_theta(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    f32::max(0.0, 1.0 - cos2_theta(v))
}

#[inline(always)]
pub fn sin_theta(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    sin2_theta(v).sqrt()
}

#[inline(always)]
pub fn tan_theta(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    sin_theta(v) / cos_theta(v)
}

#[inline(always)]
pub fn tan2_theta(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    sin2_theta(v) / cos2_theta(v)
}

#[inline(always)]
pub fn cos_phi(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    let sin_theta = sin_theta(v);
    if sin_theta == 0.0 {
        0.0
    } else {
        f32::max(v.x / sin_theta, -1.0).min(1.0)
    }
}

#[inline(always)]
pub fn sin_phi(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    let sin_theta = sin_theta(v);
    if sin_theta == 0.0 {
        0.0
    } else {
        f32::max(v.z / sin_theta, -1.0).min(1.0)
    }
}

#[inline(always)]
pub fn cos2_phi(v: &Vec3) -> f32 {
    debug_assert!(is_finite(v));

    let cos_phi = cos_phi(v);
    cos_phi * cos_phi
}

#[inline(always)]
pub fn sin2_phi(v: &Vec3) -> f32 {
    let sin_phi = sin_phi(v);
    sin_phi * sin_phi
}

#[inline(always)]
pub fn cos_d_phi(a: &Vec3, b: &Vec3) -> f32 {
    debug_assert!(is_finite(a));
    debug_assert!(is_finite(b));

    let abxz = a.x * b.x + a.z * b.z;
    let axz = a.x * a.x + a.z * a.z;
    let bxz = b.x * b.x + b.z * b.z;

    f32::max(abxz / f32::sqrt(axz * bxz), -1.0).max(1.0)
}

#[inline]
pub fn refract(v: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    let cos_i = n.dot(v);
    let sin_t2 = eta * eta * f32::max(0.0, 1.0 - cos_i * cos_i);

    if sin_t2 > 1.0 {
        None
    } else {
        let cos_t = f32::sqrt(1.0 - sin_t2);
        let r = eta * -v + (eta * cos_i - cos_t) * n;

        Some(r)
    }
}

#[inline(always)]
pub fn face_forward(v: Vec3, n: Vec3) -> Vec3 {
    if n.dot(v) > 0.0 {
        v
    } else {
        -v
    }
}

#[inline(always)]
pub fn same_hemisphere(a: &Vec3, b: &Vec3) -> bool {
    debug_assert!(is_finite(a));
    debug_assert!(is_finite(b));

    a.y * b.y > 0.0
}

#[inline(always)]
pub fn world_to_bxdf(v: &Vec3) -> Rotor3 {
    debug_assert!(is_finite(v));

    if *v == Vec3::unit_y() {
        Rotor3::default()
    } else if *v == -Vec3::unit_y() {
        Rotor3::from_rotation_xy(PI)
    } else {
        Rotor3::from_rotation_between(*v, bxdf_normal())
    }
}

pub fn bxdf_to_world(v: Vec3) -> Rotor3 {
    debug_assert!(is_finite(&v));

    if v == Vec3::unit_y() {
        Rotor3::default()
    } else if v == -Vec3::unit_y() {
        Rotor3::from_rotation_xy(-PI)
    } else {
        Rotor3::from_rotation_between(bxdf_normal(), v)
    }
}

bitflags! {
    /// All BxDF have types, for example
    /// * `BRDF` - Bidirectional Reflection Function
    /// * `BTDF` - Bidirectional Transmission Function
    /// All these have properties like being `reflective`, `transmissive`, `diffuse`, `glossy`,
    /// `specular` or be any mixture in between.
    ///
    /// This type allows bit-flagging these properties at will.
    pub struct BxDFType: u8 {
        const NONE = 1 << 0;
        const REFLECTION = 1 << 1;
        const TRANSMISSION = 1 << 2;
        const DIFFUSE = 1 << 3;
        const GLOSSY = 1 << 4;
        const SPECULAR = 1 << 5;
        const ALL = Self::REFLECTION.bits | Self::TRANSMISSION.bits | Self::DIFFUSE.bits | Self::GLOSSY.bits | Self::SPECULAR.bits;
    }
}

impl BxDFType {
    /// Returns whether this type is `reflective`.
    ///
    /// # Returns
    /// * Is `reflective`
    pub fn is_reflection(&self) -> bool {
        *self & Self::REFLECTION == Self::REFLECTION
    }

    /// Returns whether this type is `transmissive`.
    ///
    /// # Returns
    /// * Is `transmissive`
    pub fn is_transmission(&self) -> bool {
        *self & Self::TRANSMISSION == Self::TRANSMISSION
    }

    /// Returns whether this type is `diffuse`.
    ///
    /// # Returns
    /// * Is `diffuse`
    pub fn is_diffuse(&self) -> bool {
        *self & Self::DIFFUSE == Self::DIFFUSE
    }

    /// Returns whether this type is `glossy`.
    ///
    /// # Returns
    /// * Is `glossy`
    pub fn is_glossy(&self) -> bool {
        *self & Self::GLOSSY == Self::GLOSSY
    }

    /// Returns whether this type is `specular`.
    ///
    /// # Returns
    /// * Is `specular`
    pub fn is_specular(&self) -> bool {
        *self & Self::SPECULAR == Self::SPECULAR
    }
}

/// Contains of
/// * `spectrum` - An evaluated scaling spectrum
/// * `incident` - An evaluated incident direction
/// * `pdf` - An evaluated pdf
/// * `typ` - The sampled `BxDFType`
pub struct BxDFSample<T> {
    pub spectrum: T,
    pub incident: Vec3,
    pub pdf: f32,
    pub typ: BxDFType,
}

impl<T> BxDFSample<T>
where
    T: Default,
{
    /// Creates a new sample.
    ///
    /// # Constraints
    /// * `incident` - Should be normalized.
    ///
    /// # Arguments
    /// * `spectrum` - The spectrum type
    /// * `incident` The incident vector
    /// * `pdf` - The probability density function value
    /// * `typ` - The bxdf type
    ///
    /// # Returns
    /// * Self
    pub fn new(spectrum: T, incident: Vec3, pdf: f32, typ: BxDFType) -> Self {
        debug_assert!(is_normalized(&incident));

        Self {
            spectrum,
            incident,
            pdf,
            typ,
        }
    }

    /// Creates an "empty" sample, assigning zeroes to all struct members.
    ///
    /// Strictly speaking, the resulting sample is invalid, as the `incident` is a zero vector,
    ///
    /// # Returns
    /// * Self with values:
    ///     * `spectrum` - Zero / black
    ///     * `incident` - Zero
    ///     * `pdf` - Zero
    ///     * `typ` - None
    pub fn empty() -> Self {
        let spectrum = Default::default();
        let incident = Vec3::zero();
        let pdf = 0.0;
        let typ = BxDFType::NONE;

        Self {
            spectrum,
            incident,
            pdf,
            typ,
        }
    }
}

/// The common base shared between BRDFs and BTDFs.
/// Provides methods for evaluating and sampling the distribution function for pairs of directions
/// at an intersection.
pub trait BxDF: Send + Sync {
    /// Returns the type of this bxdf.
    ///
    /// # Results
    /// * The type of this BxDF
    fn get_type(&self) -> BxDFType;

    /// Allows matching the user-supplied type to this BxDF.
    ///
    /// # Results
    /// * Whether the type matches.
    fn is_type(&self, t: BxDFType) -> bool {
        let st = self.get_type();
        (st & t) == st
    }

    /// Evaluates the BxDF for the pair of incident and outgoing light directions and the
    /// intersection normal.
    ///
    /// # Constraints
    /// * `incident` - All values should be finite (neither infinite nor `NaN`).
    ///                Should be normalized.
    /// * `outgoing` - All values should be finite.
    ///                Should be normalized.
    ///
    /// # Arguments
    /// * `incident` - The incident direction onto the intersection we evaluate
    /// * `outgoing` - The outgoing light direction
    ///
    /// # Results
    /// * A scaling spectrum
    fn evaluate(&self, incident: &Vec3, outgoing: &Vec3) -> Spectrum;

    fn evaluate_light_wave(&self, incident: &Vec3, outgoing: &Vec3, light_wave_index: usize)
        -> f32;

    /// Samples an incident light direction for an outgoing light direction from the given sample
    /// space.
    ///
    /// # Constraints
    /// * `outgoing` - All values should be finite.
    ///                Should be normalized.
    /// * `sample` - All values should be within `[0, 1]`.
    ///
    /// # Arguments
    /// * `outgoing` - The outgoing light direction
    /// * `sample` - The sample space for randomization
    ///
    /// # Results
    /// * The sampled spectrum, incident and pdf
    fn sample(&self, outgoing: &Vec3, sample: &Vec2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let incident = sample_unit_hemisphere(sample);
        let incident = flip_if_neg(incident);

        let spectrum = self.evaluate(&incident, outgoing);
        let pdf = self.pdf(&incident, outgoing);

        Some(BxDFSample::new(spectrum, incident, pdf, self.get_type()))
    }

    fn sample_light_wave(
        &self,
        outgoing: &Vec3,
        sample: &Vec2,
        light_wave_index: usize,
    ) -> Option<BxDFSample<f32>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));
        debug_assert!(light_wave_index < Spectrum::size());

        let incident = sample_unit_hemisphere(sample);
        let incident = flip_if_neg(incident);

        let light_wave = self.evaluate_light_wave(&incident, outgoing, light_wave_index);
        let pdf = self.pdf(&incident, outgoing);

        Some(BxDFSample::new(light_wave, incident, pdf, self.get_type()))
    }

    /// Computes the probability density function (`pdf`) for the pair of directions.
    ///
    /// # Constraints
    /// * `incident` - All values should be finite (neither infinite nor `NaN`).
    ///                Should be normalized.
    /// * `outgoing` - All values should be finite.
    ///                Should be normalized.
    ///
    /// # Arguments
    /// * `incident` - The incident direction onto the intersection we evaluate
    /// * `outgoing` - The outgoing light direction
    ///
    /// # Results
    /// * The evaluated pdf
    #[inline]
    fn pdf(&self, incident: &Vec3, outgoing: &Vec3) -> f32 {
        if same_hemisphere(incident, outgoing) {
            cos_theta(incident).abs() * FRAC_1_PI
        } else {
            0.0
        }
    }
}

/// This special BxDF scales all spectrum outputs of another one, effectively wrapping around
/// some `BxDF` with a `scale`.
pub struct ScaledBxDF<'a> {
    bxdf: &'a dyn BxDF,
    scale: Spectrum,
}

impl<'a> ScaledBxDF<'a> {
    /// Creates a new scaled `BxDF`.
    ///
    /// # Arguments
    /// * `bxdf` - The `BxDF`
    /// * `scale` - The scale
    ///
    /// # Returns
    /// * Self
    pub fn new(bxdf: &'a dyn BxDF, scale: Spectrum) -> Self {
        Self { bxdf, scale }
    }
}

impl BxDF for ScaledBxDF<'_> {
    fn get_type(&self) -> BxDFType {
        self.bxdf.get_type()
    }

    fn evaluate(&self, view: &Vec3, from: &Vec3) -> Spectrum {
        self.scale * self.bxdf.evaluate(view, from)
    }

    fn evaluate_light_wave(
        &self,
        incident: &Vec3,
        outgoing: &Vec3,
        light_wave_index: usize,
    ) -> f32 {
        self.scale[light_wave_index]
            * self
                .bxdf
                .evaluate_light_wave(incident, outgoing, light_wave_index)
    }

    fn sample(&self, outgoing: &Vec3, sample: &Vec2) -> Option<BxDFSample<Spectrum>> {
        if let Some(mut sample) = self.bxdf.sample(outgoing, sample) {
            sample.spectrum *= self.scale;

            Some(sample)
        } else {
            None
        }
    }

    fn pdf(&self, incident: &Vec3, outgoing: &Vec3) -> f32 {
        self.bxdf.pdf(incident, outgoing)
    }
}
