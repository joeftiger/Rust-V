#![allow(dead_code)]

use crate::bxdf::fresnel::fresnel_dielectric;
use crate::bxdf::{
    bxdf_incident_to, bxdf_normal, cos_theta, refract, BxDF, BxDFSample, BxDFType, Fresnel,
    FresnelDielectric,
};
use crate::debug_utils::{is_normalized, within_01};
use crate::Spectrum;
use ultraviolet::{Vec2, Vec3};

/// # Summary
/// Describes a specular reflection
pub struct SpecularReflection {
    r: Spectrum,
    fresnel: Box<dyn Fresnel>,
}

impl SpecularReflection {
    /// # Summary
    /// Creates a new specular reflection.
    ///
    /// # Arguments
    /// * `r` - The reflection
    /// * `fresnel` - The fresnel
    ///
    /// # Returns
    /// * Self
    pub fn new(r: Spectrum, fresnel: Box<dyn Fresnel>) -> Self {
        Self { r, fresnel }
    }
}

impl BxDF for SpecularReflection {
    fn get_type(&self) -> BxDFType {
        BxDFType::REFLECTION | BxDFType::SPECULAR
    }

    /// # Summary
    /// No scattering for specular reflection.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0` spectrum
    fn evaluate(&self, _: &Vec3, _: &Vec3) -> Spectrum {
        Spectrum::new_const(0.0)
    }

    fn sample(&self, outgoing: &Vec3, _: &Vec2) -> Option<BxDFSample> {
        debug_assert!(is_normalized(outgoing));

        let incident = bxdf_incident_to(outgoing);

        let cos_i = cos_theta(&incident);
        let spectrum = self.fresnel.evaluate(cos_i) * self.r;

        Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
    }

    /// # Summary
    /// No scattering for specular reflection leads to no pdf.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0`
    fn pdf(&self, _: &Vec3, _: &Vec3) -> f32 {
        0.0
    }
}

/// # Summary
/// Describes a specular transmission.
pub struct SpecularTransmission {
    t: Spectrum,
    eta_i: f32,
    eta_t: f32,
    fresnel: FresnelDielectric,
}

impl SpecularTransmission {
    /// # Summary
    /// Creates a new specular transmission.
    ///
    /// # Arguments
    /// * `t` - The transmission
    /// * `eta_a` - The index of refraction above the surface
    /// * `eta_b` - The index of refraction below the surface
    /// * `mode` - The transport mode parameter
    ///
    /// # Returns
    /// * Self
    pub fn new(t: Spectrum, eta_i: f32, eta_t: f32) -> Self {
        let fresnel = FresnelDielectric::new(eta_i, eta_t);
        Self {
            t,
            eta_i,
            eta_t,
            fresnel,
        }
    }
}

impl BxDF for SpecularTransmission {
    fn get_type(&self) -> BxDFType {
        BxDFType::SPECULAR | BxDFType::TRANSMISSION
    }

    /// # Summary
    /// No scattering for specular transmission.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0` spectrum
    fn evaluate(&self, _: &Vec3, _: &Vec3) -> Spectrum {
        Spectrum::new_const(0.0)
    }

    fn sample(&self, outgoing: &Vec3, _: &Vec2) -> Option<BxDFSample> {
        debug_assert!(is_normalized(outgoing));

        let entering = cos_theta(outgoing) > 0.0;
        let (eta_i, eta_t, normal) = if entering {
            (self.eta_i, self.eta_t, bxdf_normal())
        } else {
            (self.eta_t, self.eta_i, -bxdf_normal())
        };

        if let Some(mut incident) = refract(*outgoing, normal, eta_i / eta_t) {
            incident.normalize();

            let cos_i = cos_theta(&incident);
            let spectrum = self.t * (Spectrum::new_const(1.0) - self.fresnel.evaluate(cos_i));

            Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
        } else {
            None
        }
    }

    /// # Summary
    /// No scattering for specular transmission leads to no pdf.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0`
    fn pdf(&self, _: &Vec3, _: &Vec3) -> f32 {
        0.0
    }
}

/// # Summary
/// Combines specular reflection and transmission for better efficiency.
pub struct FresnelSpecular {
    r: Spectrum,
    t: Spectrum,
    eta_i: f32,
    eta_t: f32,
    fresnel: FresnelDielectric,
}

impl FresnelSpecular {
    /// # Summary
    /// Creates a new fresnel specular.
    ///
    /// # Arguments
    /// * `r` - The reflection
    /// * `t` - The transmission
    /// * `eta_a` - The index of refraction above the surface
    /// * `eta_b` - The index of refraction below the surface
    /// * `mode` - The transport mode parameter
    ///
    /// # Returns
    /// * Self
    pub fn new(r: Spectrum, t: Spectrum, eta_i: f32, eta_t: f32) -> Self {
        let fresnel = FresnelDielectric::new(eta_i, eta_t);
        Self {
            r,
            t,
            eta_i,
            eta_t,
            fresnel,
        }
    }
}

impl BxDF for FresnelSpecular {
    fn get_type(&self) -> BxDFType {
        BxDFType::REFLECTION | BxDFType::SPECULAR | BxDFType::TRANSMISSION
    }

    /// # Summary
    /// No scattering for specular reflection/transmission.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0` spectrum
    fn evaluate(&self, _: &Vec3, _: &Vec3) -> Spectrum {
        Spectrum::new_const(0.0)
    }

    fn sample(&self, outgoing: &Vec3, sample: &Vec2) -> Option<BxDFSample> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let cos_outgoing = cos_theta(outgoing);
        let f = fresnel_dielectric(cos_outgoing, self.eta_i, self.eta_t);

        let entering = cos_theta(outgoing) > 0.0;
        let (eta_i, eta_t, normal) = if entering {
            (self.eta_i, self.eta_t, bxdf_normal())
        } else {
            (self.eta_t, self.eta_i, -bxdf_normal())
        };

        if let Some(incident) = refract(*outgoing, normal, eta_i / eta_t) {
            if f < sample.x {
                // let incident = incident.normalized();

                let cos_i = cos_theta(&incident);
                let spectrum = self.t * (Spectrum::new_const(1.0) - self.fresnel.evaluate(cos_i));

                let typ = BxDFType::SPECULAR | BxDFType::TRANSMISSION;

                return Some(BxDFSample::new(spectrum, incident, 1.0 - f, typ));
            }
        }

        let incident = bxdf_incident_to(outgoing);
        let typ = BxDFType::REFLECTION | BxDFType::SPECULAR;
        let spectrum = self.r * f;

        Some(BxDFSample::new(spectrum, incident, f, typ))
    }

    /// # Summary
    /// No scattering for specular reflection/transmission leads to no pdf.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0`
    fn pdf(&self, _: &Vec3, _: &Vec3) -> f32 {
        0.0
    }
}
