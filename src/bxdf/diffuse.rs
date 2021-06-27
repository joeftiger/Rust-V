use crate::bxdf::{bxdf_normal, BxDF, BxDFSample, BxDFSampleResult, Type};
use crate::mc::sample_unit_sphere;
use crate::*;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "f64"))]
use std::f32::consts::FRAC_1_PI;
#[cfg(feature = "f64")]
use std::f64::consts::FRAC_1_PI;

/// The diffuse reflection reflects along a virtual sphere at the intersection point.
/// It is therefore not equally distributed like the lambertian reflection.
#[derive(Serialize, Deserialize)]
pub struct DiffuseReflection {
    r: Spectrum,
}

#[typetag::serde]
impl BxDF for DiffuseReflection {
    fn get_type(&self) -> Type {
        Type::DIFFUSE | Type::GLOSSY
    }

    fn evaluate(&self, _: Vector3, _: Vector3) -> Spectrum {
        self.r * FRAC_1_PI
    }

    fn evaluate_wavelength(&self, _: Vector3, _: Vector3, index: usize) -> Float {
        self.r[index] * FRAC_1_PI
    }

    fn sample(&self, outgoing: Vector3, sample: Vector2) -> Option<BxDFSample<Spectrum>> {
        let target = sample_unit_sphere(sample) + bxdf_normal();
        let incident = target.normalized();

        let spectrum = self.evaluate(incident, outgoing);
        let pdf = self.pdf(incident, outgoing);

        Some(BxDFSample::new(spectrum, incident, pdf, self.get_type()))
    }

    fn sample_buf(
        &self,
        outgoing: Vector3,
        sample: Vector2,
        indices: &[usize],
    ) -> Option<BxDFSampleResult> {
        let target = sample_unit_sphere(sample) + bxdf_normal();
        let incident = target.normalized();

        let spectrum = self.evaluate_buf(incident, outgoing, indices);
        let pdf = self.pdf(incident, outgoing);

        let sample = BxDFSample::new(spectrum, incident, pdf, self.get_type());

        Some(BxDFSampleResult::Bundle(sample))
    }

    fn sample_wavelength(
        &self,
        outgoing: Vector3,
        sample: Vector2,
        index: usize,
    ) -> Option<BxDFSample<Float>> {
        let target = sample_unit_sphere(sample) + bxdf_normal();
        let incident = target.normalized();

        let spectrum = self.evaluate_wavelength(incident, outgoing, index);
        let pdf = self.pdf(incident, outgoing);

        Some(BxDFSample::new(spectrum, incident, pdf, self.get_type()))
    }
}
