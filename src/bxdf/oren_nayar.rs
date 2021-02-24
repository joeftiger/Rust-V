use crate::bxdf::{cos_phi, cos_theta, sin_phi, sin_theta, BxDF, BxDFType};
use crate::Spectrum;
use std::f32::consts::FRAC_1_PI;
use ultraviolet::Vec3;
use utility::floats::{in_range_incl_left, EPSILON};

/// The Oren-Nayar reflectance model describes rough opaque diffuse surfaces where each facet is lambertian (diffuse).
#[derive(Debug)]
pub struct OrenNayar {
    r: Spectrum,
    a: f32,
    b: f32,
}

impl OrenNayar {
    /// Creates a new oren-nayar reflection.
    ///
    /// # Constraints
    /// * `sigma` - Should be in range `[0, inf)`.
    ///
    /// # Arguments
    /// * `r` - The reflection
    /// * `sigma` - The roughness (gradient of the surface elevation)
    ///
    /// # Returns
    /// * Self
    pub fn new(r: Spectrum, sigma: f32) -> Self {
        debug_assert!(in_range_incl_left(sigma, 0.0, f32::INFINITY));

        let sigma = sigma.to_radians();
        let sigma2 = sigma * sigma;
        let a = 1.0 - (sigma2 / (2.0 * (sigma2 + 0.33)));
        let b = 0.45 * sigma2 / (sigma2 + 0.09);

        Self { r, a, b }
    }
}

impl BxDF for OrenNayar {
    fn get_type(&self) -> BxDFType {
        BxDFType::DIFFUSE | BxDFType::REFLECTION
    }

    fn evaluate(&self, incident: &Vec3, outgoing: &Vec3) -> Spectrum {
        let sin_theta_i = sin_theta(incident);
        let sin_theta_o = sin_theta(outgoing);

        let max_cos = if sin_theta_i > EPSILON && sin_theta_o > EPSILON {
            let sin_phi_i = sin_phi(incident);
            let sin_phi_o = sin_phi(outgoing);
            let cos_phi_i = cos_phi(incident);
            let cos_phi_o = cos_phi(outgoing);

            let d_cos = cos_phi_i * cos_phi_o + sin_phi_i * sin_phi_o;
            d_cos.max(0.0)
        } else {
            0.0
        };

        let cos_theta_i_abs = cos_theta(incident).abs();
        let cos_theta_o_abs = cos_theta(outgoing).abs();

        let (sin_alpha, tan_beta) = if cos_theta_i_abs > cos_theta_o_abs {
            (sin_theta_o, sin_theta_i / cos_theta_i_abs)
        } else {
            (sin_theta_i, sin_theta_o / cos_theta_o_abs)
        };

        self.r * (FRAC_1_PI * (self.a + self.b * max_cos * sin_alpha * tan_beta))
    }

    fn evaluate_light_wave(
        &self,
        incident: &Vec3,
        outgoing: &Vec3,
        light_wave_index: usize,
    ) -> f32 {
        let sin_theta_i = sin_theta(incident);
        let sin_theta_o = sin_theta(outgoing);

        let max_cos = if sin_theta_i > EPSILON && sin_theta_o > EPSILON {
            let sin_phi_i = sin_phi(incident);
            let sin_phi_o = sin_phi(outgoing);
            let cos_phi_i = cos_phi(incident);
            let cos_phi_o = cos_phi(outgoing);

            let d_cos = cos_phi_i * cos_phi_o + sin_phi_i * sin_phi_o;
            d_cos.max(0.0)
        } else {
            0.0
        };

        let cos_theta_i_abs = cos_theta(incident).abs();
        let cos_theta_o_abs = cos_theta(outgoing).abs();

        let (sin_alpha, tan_beta) = if cos_theta_i_abs > cos_theta_o_abs {
            (sin_theta_o, sin_theta_i / cos_theta_i_abs)
        } else {
            (sin_theta_i, sin_theta_o / cos_theta_o_abs)
        };

        self.r[light_wave_index] * (FRAC_1_PI * (self.a + self.b * max_cos * sin_alpha * tan_beta))
    }
}
