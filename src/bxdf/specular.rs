#![allow(dead_code)]

use crate::bxdf::fresnel::fresnel_dielectric;
use crate::bxdf::{
    bxdf_incident_to, bxdf_normal, cos_theta, refract, BxDF, BxDFSample, Fresnel,
    FresnelDielectric, FresnelType, Type,
};
use crate::debug_utils::{is_normalized, within_01};
use crate::refractive_index::RefractiveType;
use crate::Spectrum;
use color::Color;
use definitions::{Float, Vector2, Vector3};
use serde::{Deserialize, Serialize};

#[inline]
fn etas(
    eta_i: RefractiveType,
    eta_t: RefractiveType,
    outgoing: Vector3,
) -> (RefractiveType, RefractiveType, Vector3) {
    debug_assert!(is_normalized(outgoing));

    let entering = cos_theta(outgoing) > 0.0;
    if entering {
        (eta_i, eta_t, bxdf_normal())
    } else {
        (eta_t, eta_i, -bxdf_normal())
    }
}

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

#[typetag::serde]
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
    fn evaluate(&self, _: Vector3, _: Vector3) -> Spectrum {
        Spectrum::broadcast(0.0)
    }

    fn evaluate_light_wave(&self, _: Vector3, _: Vector3, _: usize) -> Float {
        0.0
    }

    fn sample(&self, outgoing: Vector3, _: Vector2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));

        let incident = bxdf_incident_to(outgoing);

        let cos_i = cos_theta(incident);
        let spectrum = self.fresnel.evaluate(cos_i) * self.r;

        Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
    }

    fn sample_light_wave(
        &self,
        outgoing: Vector3,
        _: Vector2,
        light_wave_index: usize,
    ) -> Option<BxDFSample<Float>> {
        debug_assert!(is_normalized(outgoing));

        let incident = bxdf_incident_to(outgoing);

        let cos_i = cos_theta(incident);
        let light_wave = self.r.as_light_wave(light_wave_index);
        let spectrum =
            self.fresnel.evaluate_lambda(light_wave.lambda, cos_i) * light_wave.intensity;

        Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
    }

    fn sample_light_waves(
        &self,
        outgoing: Vector3,
        _: Vector2,
        light_wave_indices: &[usize],
        samples_buf: &mut [Option<BxDFSample<Float>>],
    ) {
        debug_assert!(is_normalized(outgoing));
        debug_assert_eq!(light_wave_indices.len(), samples_buf.len());

        let incident = bxdf_incident_to(outgoing);
        let cos_i = cos_theta(incident);

        let typ = self.get_type();

        for (&index, sample) in light_wave_indices.iter().zip(samples_buf.iter_mut()) {
            let light_wave = self.r.as_light_wave(index);
            let lambda =
                self.fresnel.evaluate_lambda(light_wave.lambda, cos_i) * light_wave.intensity;

            *sample = Some(BxDFSample::new(lambda, incident, 1.0, typ))
        }
    }

    /// No scattering for specular reflection leads to no pdf.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0`
    fn pdf(&self, _: Vector3, _: Vector3) -> Float {
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

#[typetag::serde]
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
    fn evaluate(&self, _: Vector3, _: Vector3) -> Spectrum {
        Spectrum::broadcast(0.0)
    }

    fn evaluate_light_wave(&self, _: Vector3, _: Vector3, _: usize) -> Float {
        0.0
    }

    fn sample(&self, outgoing: Vector3, _: Vector2) -> Option<BxDFSample<Spectrum>> {
        let (eta_i, eta_t, normal) = etas(self.fresnel.eta_i, self.fresnel.eta_t, outgoing);

        refract(outgoing, normal, eta_i.n_uniform() / eta_t.n_uniform()).map(|mut incident| {
            incident.normalize();

            let cos_i = cos_theta(incident);
            let spectrum = self.t * (Spectrum::broadcast(1.0) - self.fresnel.evaluate(cos_i));

            BxDFSample::new(spectrum, incident, 1.0, self.get_type())
        })
    }

    fn sample_light_wave(
        &self,
        outgoing: Vector3,
        _: Vector2,
        light_wave_index: usize,
    ) -> Option<BxDFSample<Float>> {
        let (eta_i, eta_t, normal) = etas(self.fresnel.eta_i, self.fresnel.eta_t, outgoing);

        let light_wave = self.t.as_light_wave(light_wave_index);

        refract(outgoing, normal, eta_i.n_uniform() / eta_t.n_uniform()).map(|mut incident| {
            incident.normalize();

            let cos_i = cos_theta(incident);
            let lambda = light_wave.intensity
                * (1.0 - self.fresnel.evaluate_lambda(light_wave.lambda, cos_i));

            BxDFSample::new(lambda, incident, 1.0, self.get_type())
        })
    }

    fn sample_light_waves(
        &self,
        outgoing: Vector3,
        _: Vector2,
        light_wave_indices: &[usize],
        samples_buf: &mut [Option<BxDFSample<Float>>],
    ) {
        let (eta_i, eta_t, normal) = etas(self.fresnel.eta_i, self.fresnel.eta_t, outgoing);

        let typ = self.get_type();

        for (&index, sample) in light_wave_indices.iter().zip(samples_buf.iter_mut()) {
            let light_wave = self.t.as_light_wave(index);

            if let Some(mut incident) = refract(
                outgoing,
                normal,
                eta_i.n(light_wave.lambda) / eta_t.n(light_wave.lambda),
            ) {
                incident.normalize();

                let cos_i = cos_theta(incident);
                let lambda = light_wave.intensity
                    * (1.0 - self.fresnel.evaluate_lambda(light_wave.lambda, cos_i));

                *sample = Some(BxDFSample::new(lambda, incident, 1.0, typ))
            }
        }
    }

    /// No scattering for specular transmission leads to no pdf.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0`
    fn pdf(&self, _: Vector3, _: Vector3) -> Float {
        0.0
    }
}

/// Combines specular reflection and transmission for better efficiency.
#[derive(Serialize, Deserialize)]
pub struct FresnelSpecular {
    r: Spectrum,
    t: Spectrum,
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
        Self { r, t, fresnel }
    }
}

#[typetag::serde]
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
    fn evaluate(&self, _: Vector3, _: Vector3) -> Spectrum {
        Spectrum::broadcast(0.0)
    }

    fn evaluate_light_wave(&self, _: Vector3, _: Vector3, _: usize) -> Float {
        0.0
    }

    fn sample(&self, outgoing: Vector3, sample: Vector2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let cos_outgoing = cos_theta(outgoing);
        let eta_i_orig = self.fresnel.eta_i.n_uniform();
        let eta_t_orig = self.fresnel.eta_t.n_uniform();
        let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

        // if entering
        let (eta_i, eta_t, normal) = if cos_outgoing > 0.0 {
            (eta_i_orig, eta_t_orig, bxdf_normal())
        } else {
            (eta_t_orig, eta_i_orig, -bxdf_normal())
        };

        if f < sample.x {
            if let Some(incident) = refract(outgoing, normal, eta_i / eta_t) {
                let cos_i = cos_theta(incident);

                let spectrum = self.t * (Spectrum::broadcast(1.0) - self.fresnel.evaluate(cos_i));
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
        outgoing: Vector3,
        sample: Vector2,
        light_wave_index: usize,
    ) -> Option<BxDFSample<Float>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let cos_outgoing = cos_theta(outgoing);

        let lambda = Spectrum::lambda_of_index(light_wave_index);

        let eta_i_orig = self.fresnel.eta_i.n(lambda);
        let eta_t_orig = self.fresnel.eta_t.n(lambda);
        let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

        // if entering
        let (eta_i, eta_t, normal) = if cos_outgoing > 0.0 {
            (eta_i_orig, eta_t_orig, bxdf_normal())
        } else {
            (eta_t_orig, eta_i_orig, -bxdf_normal())
        };

        if f < sample.x {
            if let Some(incident) = refract(outgoing, normal, eta_i / eta_t) {
                let cos_i = cos_theta(incident);

                let intensity =
                    self.t[light_wave_index] * (1.0 - self.fresnel.evaluate_lambda(lambda, cos_i));
                let typ = Type::SPECULAR | Type::TRANSMISSION;

                return Some(BxDFSample::new(intensity, incident, 1.0 - f, typ));
            }
        }

        let intensity = self.r[light_wave_index] * f;
        let incident = bxdf_incident_to(outgoing);
        let typ = Type::REFLECTION | Type::SPECULAR;

        Some(BxDFSample::new(intensity, incident, f, typ))
    }

    fn sample_light_waves(
        &self,
        outgoing: Vector3,
        sample: Vector2,
        light_wave_indices: &[usize],
        samples_buf: &mut [Option<BxDFSample<Float>>],
    ) {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));
        debug_assert_eq!(light_wave_indices.len(), samples_buf.len());

        let cos_outgoing = cos_theta(outgoing);

        for (&index, buf_sample) in light_wave_indices.iter().zip(samples_buf.iter_mut()) {
            let lambda = Spectrum::lambda_of_index(index);

            let eta_i_orig = self.fresnel.eta_i.n(lambda);
            let eta_t_orig = self.fresnel.eta_t.n(lambda);
            let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

            // if entering
            let (eta_i, eta_t, normal) = if cos_outgoing > 0.0 {
                (eta_i_orig, eta_t_orig, bxdf_normal())
            } else {
                (eta_t_orig, eta_i_orig, -bxdf_normal())
            };

            if f < sample.x {
                if let Some(incident) = refract(outgoing, normal, eta_i / eta_t) {
                    let cos_i = cos_theta(incident);

                    let intensity =
                        self.t[index] * (1.0 - self.fresnel.evaluate_lambda(lambda, cos_i));
                    let typ = Type::SPECULAR | Type::TRANSMISSION;

                    *buf_sample = Some(BxDFSample::new(intensity, incident, 1.0 - f, typ));
                    continue;
                }
            }

            let intensity = self.r[index] * f;
            let incident = bxdf_incident_to(outgoing);
            let typ = Type::REFLECTION | Type::SPECULAR;

            *buf_sample = Some(BxDFSample::new(intensity, incident, f, typ))
        }
    }

    /// No scattering for specular reflection/transmission leads to no pdf.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `0.0`
    fn pdf(&self, _: Vector3, _: Vector3) -> Float {
        0.0
    }
}
