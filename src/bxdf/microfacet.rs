#[cfg(not(feature = "f64"))]
use std::f32::consts::{FRAC_PI_2, PI, TAU};
#[cfg(feature = "f64")]
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::{Float, Spectrum, Vector2, Vector3};

use color::Color;
use utility::{floats, math};

use crate::bxdf::fresnel::Fresnel;
use crate::bxdf::*;
use geometry::spherical_to_cartesian1;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub fn roughness_to_alpha(roughness: Float) -> Float {
    let roughness = Float::big_epsilon().fast_max(roughness);
    let x = roughness.ln();
    let x2 = x * x;

    1.62142 + 0.819955 * x + 0.1734 * x2 + 0.0171201 * x2 * x + 0.000640711 * x2 * x2
}

#[typetag::serde]
pub trait MicrofacetDistribution: Send + Sync {
    fn d(&self, wh: Vector3) -> Float;

    fn lambda(&self, w: Vector3) -> Float;

    fn g1(&self, w: Vector3) -> Float {
        1.0 / (1.0 + self.lambda(w))
    }

    fn g(&self, wi: Vector3, wo: Vector3) -> Float {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    fn sample_wh(&self, wo: Vector3, sample: Vector2) -> Vector3;

    fn pdf(&self, wo: Vector3, wh: Vector3) -> Float {
        if self.is_sample_visible_area() {
            self.d(wh) * self.g1(wo) * wo.dot(wh).abs() / cos_theta(wo).abs()
        } else {
            self.d(wh) * cos_theta(wh).abs()
        }
    }

    fn is_sample_visible_area(&self) -> bool;
}

#[derive(Deserialize, Serialize)]
pub struct BeckmannDistribution {
    alpha_x: Float,
    alpha_y: Float,
    sample_visible_area: bool,
}

impl BeckmannDistribution {
    pub fn new(alpha_x: Float, alpha_y: Float, sample_visible_area: bool) -> Self {
        Self {
            alpha_x,
            alpha_y,
            sample_visible_area,
        }
    }

    fn beckmann_sample11(cos_theta_i: Float, sample: Vector2) -> Vector2 {
        /* Special case (normal incidence) */
        if cos_theta_i > 1.0 - Float::big_epsilon() {
            let r = Float::sqrt(-Float::ln(1.0 - sample.x));
            let phi = 2.0 * PI * sample.y;
            let (sin, cos) = phi.sin_cos();

            return Vector2::new(r * sin, r * cos);
        }

        /* The original inversion routine from the paper contained
        discontinuities, which causes issues for QMC integration
        and techniques like Kelemen-style MLT. The following code
        performs a numerical inversion with better behavior */
        let sin_theta_i = Float::fast_max(0.0, 1.0 - cos_theta_i * cos_theta_i).sqrt();
        let tan_theta_i = sin_theta_i / cos_theta_i;
        let cot_theta_i = 1.0 / tan_theta_i;

        /* Search interval -- everything is parameterized
        in the Erf() domain */
        let mut a = -1.0;
        let mut c = math::erf(cot_theta_i);
        let sample_x = Float::big_epsilon().fast_max(sample.x);

        /* Start with a good initial guess */
        // Float b = (1-sample_x) * a + sample_x * c;

        /* We can do better (inverse of an approximation computed in
         * Mathematica) */
        let theta_i = cos_theta_i.acos();
        let fit = 1.0 + theta_i * (-0.876 + theta_i * (0.4265 - 0.0594 * theta_i));
        let mut b = c - (1.0 + c) * Float::powf(1.0 - sample_x, fit);

        /* Normalization factor for the CDF */

        let normalization = 1.0
            / (1.0
                + c
                + floats::FRAC_1_SQRT_PI * tan_theta_i * Float::exp(-cot_theta_i * cot_theta_i));

        for _ in 0..10 {
            /* Bisection criterion -- the oddly-looking
            Boolean expression are intentional to check
            for NaNs at little additional cost */
            // if !(b >= a && b <= c) {
            if b < a || b > c {
                b = 0.5 * (a + c);
            }

            /* Evaluate the CDF and its derivative
            (i.e. the density function) */
            let inv_erf = math::erf_inv(b);
            let value = normalization
                * (1.0 + b + floats::FRAC_1_SQRT_PI * tan_theta_i * Float::exp(-inv_erf * inv_erf))
                - sample_x;
            if value.abs() < Float::big_epsilon() {
                break;
            }

            let derivative = normalization * (1.0 - inv_erf * tan_theta_i);

            /* Update bisection intervals */
            if value > 0.0 {
                c = b;
            } else {
                a = b;
            }

            b -= value / derivative;
        }

        /* Now convert back into a slope value */
        let out = Vector2::new(
            math::erf_inv(b),
            math::erf_inv(2.0 * Float::big_epsilon().fast_max(sample.y) - 1.0),
        );

        debug_assert!(out.x.is_finite());
        debug_assert!(out.y.is_finite());

        out
    }

    fn beckmann_sample(wi: Vector3, alpha_x: Float, alpha_y: Float, sample: Vector2) -> Vector3 {
        // 1. stretch wi
        let wi_stretched = Vector3::new(alpha_x * wi.x, alpha_y * wi.y, wi.z).normalized();

        // 2. simulate P22_{wi}(x_slope, y_slope, 1, 1)
        let cos_theta = cos_theta(wi_stretched);
        let mut slope = Self::beckmann_sample11(cos_theta, sample);

        // 3. rotate
        let cos_phi = cos_phi(wi_stretched);
        let sin_phi = sin_phi(wi_stretched);
        let tmp = cos_phi * slope.x - sin_phi * slope.y;
        slope.y = sin_phi * slope.x + cos_phi * slope.y;
        slope.x = tmp;

        // 4. unstretch
        slope.x *= alpha_x;
        slope.y *= alpha_y;

        // 5. compute normal
        Vector3::new(-slope.x, -slope.y, 1.0).normalized()
    }
}

#[typetag::serde]
impl MicrofacetDistribution for BeckmannDistribution {
    fn d(&self, wh: Vector3) -> Float {
        let tan2_theta = tan2_theta(wh);
        if tan2_theta.is_infinite() {
            0.0
        } else {
            let cos2_theta = cos2_theta(wh);
            let cos4_theta = cos2_theta * cos2_theta;

            let alpha_x2 = self.alpha_x * self.alpha_x;

            let cos2 = cos2_phi(wh) / alpha_x2;
            let sin2 = sin2_phi(wh) / self.alpha_y * self.alpha_y;

            Float::exp(-tan2_theta * (cos2 + sin2)) / (PI * alpha_x2 * cos4_theta)
        }
    }

    fn lambda(&self, w: Vector3) -> Float {
        let tan_theta = tan_theta(w);
        if tan_theta.is_infinite() {
            return 0.0;
        }

        // Compute _alpha_ for direction _w_
        let cos2 = cos2_phi(w) * self.alpha_x * self.alpha_x;
        let sin2 = sin2_phi(w) * self.alpha_y * self.alpha_y;

        let alpha = Float::sqrt(cos2 + sin2);
        let a = 1.0 / (alpha * tan_theta.abs());
        if a >= Float::big_epsilon() {
            0.0
        } else {
            (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a)
        }
    }

    fn sample_wh(&self, wo: Vector3, sample: Vector2) -> Vector3 {
        if self.sample_visible_area {
            let is_neg = is_neg(wo);
            let wo_new = if is_neg { -wo } else { wo };

            let mut wh = Self::beckmann_sample(wo_new, self.alpha_x, self.alpha_y, sample);
            if is_neg {
                wh = -wh;
            }

            return wh;
        }

        // Sample full distribution of normals for Beckmann distribution

        // Compute $\tan^2 \theta$ and $\phi$ for Beckmann distribution sample
        let alpha_x2 = self.alpha_x * self.alpha_x;
        let log_sample = Float::ln(1.0 - sample.x);
        debug_assert!(log_sample.is_finite());

        let tan2_theta: Float;
        let phi: Float;
        if sample.x.is_approx_eq(sample.y) {
            tan2_theta = -alpha_x2 * log_sample;
            phi = TAU * sample.y;
        } else {
            // Compute _tan2Theta_ and _phi_ for anisotropic Beckmann
            // distribution
            let tan = Float::tan(TAU * sample.y * FRAC_PI_2);
            let mut phi_new = Float::atan(self.alpha_y * tan / self.alpha_x);
            if sample.y > 0.5 {
                phi_new += PI;
            }
            phi = phi_new;

            let (sin_phi, cos_phi) = phi.sin_cos();

            tan2_theta = -log_sample
                / (cos_phi * cos_phi / alpha_x2
                    + sin_phi * sin_phi / (self.alpha_y * self.alpha_y));
        }

        // Map sampled Beckmann angles to normal direction _wh_
        let cos_theta = 1.0 / Float::sqrt(1.0 - tan2_theta);
        let sin_theta = Float::fast_max(0.0, 1.0 - cos_theta * cos_theta).sqrt();

        let mut wh = spherical_to_cartesian1(sin_theta, cos_theta, phi);
        if !same_hemisphere(wo, wh) {
            wh = -wh;
        }

        wh
    }

    fn is_sample_visible_area(&self) -> bool {
        self.sample_visible_area
    }
}

/*pub struct TrowbridgeReitzDistribution {
    alpha_x: Float,
    alpha_y: Float,
    sample_visible_area: bool
}

impl TrowbridgeReitzDistribution {
    pub fn new(alpha_x: Float, alpha_y: Float, sample_visible_area: bool) -> Self {
        Self { alpha_x, alpha_y, sample_visible_area }
    }
}

impl MicrofacetDistribution for TrowbridgeReitzDistribution {
    fn d(&self, wh: &Vector3) -> Float {
        unimplemented!()
    }

    fn lambda(&self, w: &Vector3) -> Float {
        let tan_theta = tan_theta(w);
        if tan_theta.is_infinite() {
            return 0.0;
        }

        let tan2 = tan_theta * tan_theta;
        let cos2 = cos2_phi(w) * self.alpha_x * self.alpha_x;
        let sin2 = sin2_phi(w) * self.alpha_y * self.alpha_y;

        let alpha = Float::sqrt(cos2 + sin2);
        let alpha2_tan2 = tan2 * alpha * alpha;

        (-1.0 + Float::sqrt(1.0 + alpha2_tan2)) / 2.0
    }

    fn sample_wh(&self, wo: &Vector3, sample: &Vector2) -> Vector3 {
        unimplemented!()
    }

    fn pdf(&self, wo: &Vector3, wh: &Vector3) -> Float {
        unimplemented!()
    }
}*/

#[derive(Deserialize, Serialize)]
pub struct MicrofacetReflection {
    r: Spectrum,
    distribution: Box<dyn MicrofacetDistribution>,
    fresnel: FresnelType,
}

impl MicrofacetReflection {
    pub fn new(
        r: Spectrum,
        distribution: Box<dyn MicrofacetDistribution>,
        fresnel: FresnelType,
    ) -> Self {
        Self {
            r,
            distribution,
            fresnel,
        }
    }
}

#[typetag::serde]
impl BxDF for MicrofacetReflection {
    fn get_type(&self) -> Type {
        Type::REFLECTION | Type::GLOSSY
    }

    fn evaluate(&self, incident: Vector3, outgoing: Vector3) -> Spectrum {
        let cos_theta_i = cos_theta(incident).abs();
        let cos_theta_o = cos_theta(outgoing).abs();
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return Spectrum::broadcast(0.0);
        }

        let wh = incident + outgoing;
        if wh == Vector3::zero() {
            return Spectrum::broadcast(0.0);
        }

        let wh = wh.normalized();

        let f = self.fresnel.evaluate(incident.dot(wh));
        let mul = self.distribution.d(wh) * self.distribution.g(incident, outgoing);

        self.r * f * (mul / (4.0 * cos_theta_i * cos_theta_o))
    }

    fn evaluate_light_wave(
        &self,
        incident: Vector3,
        outgoing: Vector3,
        light_wave_index: usize,
    ) -> Float {
        let cos_theta_i = cos_theta(incident).abs();
        let cos_theta_o = cos_theta(outgoing).abs();
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return 0.0;
        }

        let wh = incident + outgoing;
        if wh == Vector3::zero() {
            return 0.0;
        }

        let wh = wh.normalized();

        let f = self.fresnel.evaluate(incident.dot(wh));
        let mul = self.distribution.d(wh) * self.distribution.g(incident, outgoing);

        self.r[light_wave_index] * f[light_wave_index] * (mul / (4.0 * cos_theta_i * cos_theta_o))
    }

    fn sample(&self, outgoing: Vector3, sample: Vector2) -> Option<BxDFSample<Spectrum>> {
        // Sample microfacet orientation $\wh$ and reflected direction $\wi$
        if bxdf_is_parallel(outgoing) {
            return None;
        }

        let wh = self.distribution.sample_wh(outgoing, sample);
        let cos_o = outgoing.dot(wh);
        // Should be rare
        if cos_o < 0.0 {
            return None;
        }

        let incident = outgoing.reflected(wh);
        if !same_hemisphere(incident, outgoing) {
            return None;
        }

        let spectrum = self.evaluate(incident, outgoing);
        let pdf = self.distribution.pdf(outgoing, wh) / (4.0 * cos_o);

        Some(BxDFSample::new(spectrum, incident, pdf, self.get_type()))
    }

    fn pdf(&self, incident: Vector3, outgoing: Vector3) -> Float {
        if !same_hemisphere(incident, outgoing) {
            0.0
        } else {
            let wh = (incident + outgoing).normalized();

            self.distribution.pdf(outgoing, wh) / (4.0 * outgoing.dot(wh))
        }
    }
}

// #[derive(Debug)]
// pub struct MicrofacetTransmission {
//     t: Spectrum,
//     distribution: Box<dyn MicrofacetDistribution>,
//     fresnel: Dielectric,
// }
//
// impl MicrofacetTransmission {
//     pub fn new(
//         t: Spectrum,
//         distribution: Box<dyn MicrofacetDistribution>,
//         fresnel: Dielectric,
//     ) -> Self {
//         Self {
//             t,
//             distribution,
//             fresnel,
//         }
//     }
// }
//
// impl BxDF for MicrofacetTransmission {
//     fn get_type(&self) -> BxDFType {
//         BxDFType::TRANSMISSION | BxDFType::GLOSSY
//     }
//
//     fn evaluate(&self, incident: &Vector3, outgoing: &Vector3) -> Spectrum {
//         if same_hemisphere(incident, outgoing) {
//             return Spectrum::black();
//         }
//
//         let cos_theta_i = cos_theta(incident);
//         let cos_theta_o = cos_theta(outgoing);
//         if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
//             return Spectrum::black();
//         }
//
//         // Compute $\wh$ from $\wo$ and $\wi$ for microfacet transmission
//         let (eta, wh) = {
//             let eta = if cos_theta_o > 0.0 {
//                 self.fresnel.eta_t / self.fresnel.eta_i
//             } else {
//                 self.fresnel.eta_i / self.fresnel.eta_t
//             };
//             let wh = flip_if_neg(*outgoing + *incident * eta);
//             (eta, wh)
//         };
//
//         // Same side?
//         let cos_i = outgoing.dot(wh);
//         let cos_t = incident.dot(wh);
//         if cos_i * cos_t > 0.0 {
//             return Spectrum::black();
//         }
//
//         let f = self.fresnel.evaluate(cos_i);
//
//         let sqrt_denom = cos_i + eta * cos_t;
//
//         let t = (Spectrum::new_const(1.0) - f) * self.t;
//         let dist = self.distribution.d(&wh) * self.distribution.g(incident, outgoing);
//         let factor =
//             cos_i.abs() * cos_t.abs() / (cos_theta_i * cos_theta_i * sqrt_denom * sqrt_denom);
//
//         t * (dist * factor).abs()
//     }
//
//     fn sample(&self, outgoing: &Vector3, sample: &Vector2) -> BxDFSample {
//         if bxdf_is_parallel(outgoing) {
//             return BxDFSample::black_nan_0();
//         }
//
//         let wh = self.distribution.sample_wh(outgoing, sample);
//         // Should be rare
//         if outgoing.dot(wh) < 0.0 {
//             return BxDFSample::black_nan_0();
//         }
//
//         let eta = if cos_theta(outgoing) > 0.0 {
//             self.fresnel.eta_i / self.fresnel.eta_t
//         } else {
//             self.fresnel.eta_t / self.fresnel.eta_i
//         };
//
//         let incident = outgoing.refracted(wh, eta);
//         let spectrum = self.evaluate(&incident, outgoing);
//         let pdf = self.pdf(&incident, outgoing);
//
//         BxDFSample::new(spectrum, incident, pdf, self.get_type())
//     }
//
//     fn pdf(&self, incident: &Vector3, outgoing: &Vector3) -> Float {
//         if same_hemisphere(incident, outgoing) {
//             return 0.0;
//         }
//
//         // Compute $\wh$ from $\wo$ and $\wi$ for microfacet transmission
//         let eta = if cos_theta(outgoing) > 0.0 {
//             self.fresnel.eta_i / self.fresnel.eta_t
//         } else {
//             self.fresnel.eta_t / self.fresnel.eta_i
//         };
//
//         let wh = (*outgoing + *incident * eta).normalized();
//
//         let cos_i = incident.dot(wh);
//         let cos_o = outgoing.dot(wh);
//         if cos_i * cos_o > 0.0 {
//             return 0.0;
//         }
//
//         let sqrt_denom = cos_o + eta * cos_i;
//         let dwh_dwi = eta * eta * cos_i.abs() / (sqrt_denom * sqrt_denom);
//
//         self.distribution.pdf(outgoing, &wh) * dwh_dwi
//     }
// }
