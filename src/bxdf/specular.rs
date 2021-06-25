#![allow(dead_code)]

use crate::bxdf::fresnel::fresnel_dielectric;
use crate::bxdf::{
    bxdf_incident_to, bxdf_normal, cos_theta, refract, BxDF, BxDFSample, BxDFSampleBuf,
    BxDFSampleBufResult, Fresnel, FresnelDielectric, FresnelType, Type,
};
use crate::debug_utils::{is_normalized, within_01};
use crate::refractive_index::RefractiveType;
use crate::*;
use color::Color;

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

    fn evaluate_buf(&self, _: Vector3, _: Vector3, indices: &[usize]) -> Vec<Float> {
        vec![0.0; indices.len()]
    }

    fn evaluate_wavelength(&self, _: Vector3, _: Vector3, _: usize) -> Float {
        0.0
    }

    fn sample(&self, outgoing: Vector3, _: Vector2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));

        let incident = bxdf_incident_to(outgoing);

        let cos_i = cos_theta(incident);
        let spectrum = self.fresnel.evaluate(cos_i) * self.r;

        Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
    }

    fn sample_buf(
        &self,
        outgoing: Vector3,
        sample: Vector2,
        indices: &[usize],
    ) -> Option<BxDFSampleBufResult> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let incident = bxdf_incident_to(outgoing);

        let cos_i = cos_theta(incident);
        let spectrum = indices
            .iter()
            .map(|&i| {
                self.r[i]
                    * self
                        .fresnel
                        .evaluate_lambda(Spectrum::lambda_of_index(i), cos_i)
            })
            .collect();

        Some(BxDFSampleBufResult::Single(BxDFSample::new(
            spectrum,
            incident,
            1.0,
            self.get_type(),
        )))
    }

    fn sample_wavelength(
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
        Self { t, fresnel }
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

    fn evaluate_buf(&self, _: Vector3, _: Vector3, indices: &[usize]) -> Vec<Float> {
        vec![0.0; indices.len()]
    }

    fn evaluate_wavelength(&self, _: Vector3, _: Vector3, _: usize) -> Float {
        0.0
    }

    fn sample(&self, outgoing: Vector3, _: Vector2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));

        let (eta_i, eta_t, normal) = etas(self.fresnel.eta_i, self.fresnel.eta_t, outgoing);
        let incident = refract(outgoing, normal, eta_i.n_uniform() / eta_t.n_uniform())?;

        let cos_i = cos_theta(incident);
        let spectrum = self.t * (Spectrum::broadcast(1.0) - self.fresnel.evaluate(cos_i));

        Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
    }

    fn sample_buf(
        &self,
        outgoing: Vector3,
        sample: Vector2,
        indices: &[usize],
    ) -> Option<BxDFSampleBufResult> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let (eta_i, eta_t, normal) = etas(self.fresnel.eta_i, self.fresnel.eta_t, outgoing);

        let mut spectrum = vec![0.0; indices.len()];
        let mut incidents = vec![Vector3::zero(); indices.len()];
        let types = vec![self.get_type(); indices.len()];

        let scattered_pdf = indices.len() as Float / Spectrum::size() as Float;
        let pdfs = vec![scattered_pdf; indices.len()];

        for i in 0..indices.len() {
            let lambda = Spectrum::lambda_of_index(indices[i]);

            // TODO: Handle refractions where only specific lambdas are invalid.
            //       Currently we just "get out" if there is any invalid lambda.
            incidents[i] = refract(outgoing, normal, eta_i.n(lambda) / eta_t.n(lambda))?;

            let cos_i = cos_theta(incidents[i]);
            spectrum[i] = self.t[i] * (1.0 - self.fresnel.evaluate_lambda(cos_i, lambda));
        }

        let sample = BxDFSampleBuf {
            spectrum,
            incidents,
            pdfs,
            types,
        };

        Some(BxDFSampleBufResult::Buffer(sample))
    }

    fn sample_wavelength(
        &self,
        outgoing: Vector3,
        _: Vector2,
        index: usize,
    ) -> Option<BxDFSample<Float>> {
        debug_assert!(is_normalized(outgoing));

        let (eta_i, eta_t, normal) = etas(self.fresnel.eta_i, self.fresnel.eta_t, outgoing);

        let incident = refract(outgoing, normal, eta_i.n_uniform() / eta_t.n_uniform())?;

        let cos_i = cos_theta(incident);
        let lambda = Spectrum::lambda_of_index(index);
        let spectrum = self.t[index] * (1.0 - self.fresnel.evaluate_lambda(cos_i, lambda));

        Some(BxDFSample::new(spectrum, incident, 1.0, self.get_type()))
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

    fn fresnel_incident(
        outgoing: Vector3,
        sample: Vector2,
        eta_i_orig: Float,
        eta_t_orig: Float,
    ) -> Option<Vector3> {
        let cos_outgoing = cos_theta(outgoing);
        let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

        let incident = if f < sample.x {
            // if entering
            let (eta_i, eta_t, normal) = if cos_outgoing > 0.0 {
                (eta_i_orig, eta_t_orig, bxdf_normal())
            } else {
                (eta_t_orig, eta_i_orig, -bxdf_normal())
            };

            refract(outgoing, normal, eta_i / eta_t)?
        } else {
            bxdf_incident_to(outgoing)
        };

        Some(incident)
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

    fn evaluate_buf(&self, _: Vector3, _: Vector3, indices: &[usize]) -> Vec<Float> {
        vec![0.0; indices.len()]
    }

    fn evaluate_wavelength(&self, _: Vector3, _: Vector3, _: usize) -> Float {
        0.0
    }

    fn sample(&self, outgoing: Vector3, sample: Vector2) -> Option<BxDFSample<Spectrum>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let cos_outgoing = cos_theta(outgoing);
        let eta_i_orig = self.fresnel.eta_i.n_uniform();
        let eta_t_orig = self.fresnel.eta_t.n_uniform();
        let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

        if sample.x < f {
            // specular reflection

            let incident = bxdf_incident_to(outgoing);
            let typ = Type::REFLECTION | Type::SPECULAR;
            let spectrum = self.r * f;
            let pdf = f;

            Some(BxDFSample::new(spectrum, incident, pdf, typ))
        } else {
            // specular transmission

            let entering = cos_outgoing > 0.0;
            let (eta_i, eta_t, normal) = if entering {
                (eta_i_orig, eta_t_orig, bxdf_normal())
            } else {
                (eta_t_orig, eta_i_orig, -bxdf_normal())
            };

            let incident = refract(outgoing, normal, eta_i / eta_t)?;

            let spectrum = self.t * (1.0 - f);
            let typ = Type::SPECULAR | Type::TRANSMISSION;
            let pdf = 1.0 - f;

            Some(BxDFSample::new(spectrum, incident, pdf, typ))
        }
    }

    fn sample_buf(
        &self,
        outgoing: Vector3,
        sample: Vector2,
        indices: &[usize],
    ) -> Option<BxDFSampleBufResult> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let cos_outgoing = cos_theta(outgoing);

        let mut spectrum = vec![0.0; indices.len()];
        let mut incidents = vec![Vector3::zero(); indices.len()];
        let mut types = vec![Type::empty(); indices.len()];
        let mut pdfs = vec![0.0; indices.len()];

        for i in 0..indices.len() {
            let lambda = Spectrum::lambda_of_index(i);

            let eta_i_orig = self.fresnel.eta_i.n(lambda);
            let eta_t_orig = self.fresnel.eta_t.n(lambda);
            let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

            if sample.x < f {
                // specular reflection

                incidents[i] = bxdf_incident_to(outgoing);
                types[i] = Type::REFLECTION | Type::SPECULAR;
                spectrum[i] = self.r[i] * f;
                pdfs[i] = f;
            } else {
                // specular transmission

                let entering = cos_outgoing > 0.0;
                let (eta_i, eta_t, normal) = if entering {
                    (eta_i_orig, eta_t_orig, bxdf_normal())
                } else {
                    (eta_t_orig, eta_i_orig, -bxdf_normal())
                };

                incidents[i] = refract(outgoing, normal, eta_i / eta_t)?;
                spectrum[i] = self.t[i] * (1.0 - f);
                types[i] = Type::SPECULAR | Type::TRANSMISSION;
                pdfs[i] = 1.0 - f;
            }
        }

        let sample = BxDFSampleBuf {
            spectrum,
            incidents,
            pdfs,
            types,
        };

        Some(BxDFSampleBufResult::Buffer(sample))
    }

    fn sample_wavelength(
        &self,
        outgoing: Vector3,
        sample: Vector2,
        index: usize,
    ) -> Option<BxDFSample<Float>> {
        debug_assert!(is_normalized(outgoing));
        debug_assert!(within_01(sample));

        let cos_outgoing = cos_theta(outgoing);

        let lambda = Spectrum::lambda_of_index(index);

        let eta_i_orig = self.fresnel.eta_i.n(lambda);
        let eta_t_orig = self.fresnel.eta_t.n(lambda);
        let f = fresnel_dielectric(cos_outgoing, eta_i_orig, eta_t_orig);

        if sample.x < f {
            // specular reflection

            let incident = bxdf_incident_to(outgoing);
            let typ = Type::REFLECTION | Type::SPECULAR;
            let spectrum = self.r[index] * f;
            let pdf = f;

            Some(BxDFSample::new(spectrum, incident, pdf, typ))
        } else {
            // specular transmission

            let entering = cos_outgoing > 0.0;
            let (eta_i, eta_t, normal) = if entering {
                (eta_i_orig, eta_t_orig, bxdf_normal())
            } else {
                (eta_t_orig, eta_i_orig, -bxdf_normal())
            };

            let incident = refract(outgoing, normal, eta_i / eta_t)?;
            let spectrum = self.t[index] * (1.0 - f);
            let typ = Type::SPECULAR | Type::TRANSMISSION;
            let pdf = 1.0 - f;

            Some(BxDFSample::new(spectrum, incident, pdf, typ))
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
