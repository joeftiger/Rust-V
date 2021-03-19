#![allow(dead_code)]

use crate::bxdf::fresnel::fresnel_dielectric;
use crate::bxdf::{
    bxdf_incident_to, bxdf_normal, cos_theta, refract, BxDF, BxDFSample, Fresnel,
    FresnelDielectric, FresnelType, Type,
};
use crate::debug_utils::{is_normalized, within_01};
use crate::refractive_index::RefractiveType;
use crate::Spectrum;
use color::{Color, IndexSpectral};
use serde::{Deserialize, Serialize};
use ultraviolet::{Vec2, Vec3};

/// Describes a specular reflection
#[derive(Serialize, Deserialize)]
pub struct SpecularReflection {
    r: Spectrum,
    fresnel: FresnelType,
}

impl SpecularReflection {
    /// Creates a new specular reflection.
    ///
    /// # Arguments
    /// * `r` - The reflection
    /// * `fresnel` - The fresnel
    ///
    /// # Returns
    /// * Self
    pub fn new(r: Spectrum, fresnel: FresnelType) -> Self {
        Self { r, fresnel }
    }
}

impl BxDF for SpecularReflection {
    fn get_type(&self) -> Type {
        Type::REFLECTION | Type::SPECULAR
    }

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

    fn evaluate_light_wave(&self, _: &Vec3, _: &Vec3, _: usize) -> f32 {
        0.0
    }

    fn sample(&self, outgoing: &Vec3, _: &Vec2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));

        let incident = bxdf_incident_to(outgoing);

        let cos_i = cos_theta(&incident);
        let spectrum = self.fresnel.evaluate(cos_i) * self.r;

        Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
    }

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

/// Describes a specular transmission.
#[derive(Serialize, Deserialize)]
pub struct SpecularTransmission {
    t: Spectrum,
    eta_i: RefractiveType,
    eta_t: RefractiveType,
    fresnel: FresnelDielectric,
}

impl SpecularTransmission {
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
    pub fn new(t: Spectrum, eta_i: RefractiveType, eta_t: RefractiveType) -> Self {
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
    fn get_type(&self) -> Type {
        Type::SPECULAR | Type::TRANSMISSION
    }

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

    fn evaluate_light_wave(&self, _: &Vec3, _: &Vec3, _: usize) -> f32 {
        0.0
    }

    fn sample(&self, outgoing: &Vec3, _: &Vec2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));

        let entering = cos_theta(outgoing) > 0.0;
        let (eta_i, eta_t, normal) = if entering {
            (self.eta_i, self.eta_t, bxdf_normal())
        } else {
            (self.eta_t, self.eta_i, -bxdf_normal())
        };

        if let Some(mut incident) =
            refract(*outgoing, normal, eta_i.n_uniform() / eta_t.n_uniform())
        {
            incident.normalize();

            let cos_i = cos_theta(&incident);
            let spectrum = self.t * (Spectrum::new_const(1.0) - self.fresnel.evaluate(cos_i));

            Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
        } else {
            None
        }
    }

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

/// Combines specular reflection and transmission for better efficiency.
#[derive(Serialize, Deserialize)]
pub struct FresnelSpecular {
    r: Spectrum,
    t: Spectrum,
    eta_i: RefractiveType,
    eta_t: RefractiveType,
    fresnel: FresnelDielectric,
}

impl FresnelSpecular {
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
    pub fn new(r: Spectrum, t: Spectrum, eta_i: RefractiveType, eta_t: RefractiveType) -> Self {
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
    fn get_type(&self) -> Type {
        Type::REFLECTION | Type::SPECULAR | Type::TRANSMISSION
    }

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

    fn evaluate_light_wave(&self, _: &Vec3, _: &Vec3, _: usize) -> f32 {
        0.0
    }

    fn sample(&self, outgoing: &Vec3, sample: &Vec2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let cos_outgoing = cos_theta(outgoing);
        let eta_i_orig = self.eta_i.n_uniform();
        let eta_t_orig = self.eta_t.n_uniform();
        let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

        // if entering
        let (eta_i, eta_t, normal) = if cos_outgoing > 0.0 {
            (eta_i_orig, eta_t_orig, bxdf_normal())
        } else {
            (eta_t_orig, eta_i_orig, -bxdf_normal())
        };

        if f < sample.x {
            if let Some(incident) = refract(*outgoing, normal, eta_i / eta_t) {
                let cos_i = cos_theta(&incident);

                let spectrum = self.t * (Spectrum::new_const(1.0) - self.fresnel.evaluate(cos_i));
                let typ = Type::SPECULAR | Type::TRANSMISSION;

                return Some(BxDFSample::new(spectrum, incident, 1.0 - f, typ));
            }
        }

        let incident = bxdf_incident_to(outgoing);
        let typ = Type::REFLECTION | Type::SPECULAR;
        let spectrum = self.r * f;

        Some(BxDFSample::new(spectrum, incident, f, typ))
    }

    fn sample_light_wave(
        &self,
        outgoing: &Vec3,
        sample: &Vec2,
        light_wave_index: usize,
    ) -> Option<BxDFSample<f32>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let lambda = Spectrum::new_const(0.0).as_light_waves()[light_wave_index].lambda;

        let cos_outgoing = cos_theta(outgoing);

        let eta_i_orig = self.eta_i.n(lambda);
        let eta_t_orig = self.eta_t.n(lambda);
        let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

        // if entering
        let (eta_i, eta_t, normal) = if cos_outgoing > 0.0 {
            (eta_i_orig, eta_t_orig, bxdf_normal())
        } else {
            (eta_t_orig, eta_i_orig, -bxdf_normal())
        };

        if f < sample.x {
            if let Some(incident) = refract(*outgoing, normal, eta_i / eta_t) {
                let cos_i = cos_theta(&incident);

                let intensity = self.t.index_spectral(light_wave_index)
                    * (1.0 - self.fresnel.evaluate_lambda(lambda, cos_i));
                let typ = Type::SPECULAR | Type::TRANSMISSION;

                return Some(BxDFSample::new(intensity, incident, 1.0 - f, typ));
            }
        }

        let intensity = self.r.index_spectral(light_wave_index) * f;
        let incident = bxdf_incident_to(outgoing);
        let typ = Type::REFLECTION | Type::SPECULAR;

        Some(BxDFSample::new(intensity, incident, f, typ))
    }

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
