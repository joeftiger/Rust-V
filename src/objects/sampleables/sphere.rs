use crate::debug_utils::{is_finite, within_01};
use crate::mc::{sample_unit_sphere, uniform_cone_pdf};
use crate::objects::emitter::SurfaceSample;
use crate::objects::Sampleable;
use geometry::{spherical_to_cartesian_frame_trig, CoordinateSystem, Sphere};
use std::f32::consts::TAU;
use ultraviolet::{Vec2, Vec3};
use utility::floats::fast_max;

fn sample_surface_inside(sphere: &Sphere, sample: &Vec2) -> SurfaceSample {
    let normal = sample_unit_sphere(sample);
    let point = sphere.center + sphere.radius * normal;
    let pdf = 1.0 / sphere.surface_area();

    SurfaceSample::new(point, normal, pdf)
}

#[typetag::serde]
impl Sampleable for Sphere {
    fn surface_area(&self) -> f32 {
        2.0 * TAU * self.radius * self.radius
    }

    // Copyright: https://github.com/mmp/pbrt-v3/blob/master/src/shapes/sphere.cpp
    fn sample_surface(&self, origin: &Vec3, sample: &Vec2) -> SurfaceSample {
        debug_assert!(is_finite(origin));
        debug_assert!(within_01(sample));

        let origin_to_center = self.center - *origin;
        let dist_sq = origin_to_center.mag_sq();
        let r2 = self.radius * self.radius;

        if dist_sq <= r2 {
            // inside the sphere (may happen)
            sample_surface_inside(&self, sample)
        } else {
            let distance = dist_sq.sqrt();
            let axis = origin_to_center / -distance;
            let frame = CoordinateSystem::from_y(axis);

            /* PBR code */
            let sin_theta_max = f32::sqrt(r2 / dist_sq);
            let sin_theta_max2 = sin_theta_max * sin_theta_max;
            let inv_sin_theta_max = 1.0 / sin_theta_max;
            let cos_theta_max = fast_max(0.0, 1.0 - sin_theta_max2).sqrt();

            let mut cos_theta = (cos_theta_max - 1.0) * sample.x + 1.0;
            let mut sin_theta2 = 1.0 - cos_theta * cos_theta;

            if sin_theta2 < 0.00068523 {
                sin_theta2 = sin_theta_max2 * sample.x;
                cos_theta = (1.0 - sin_theta2).sqrt();
            }

            let cos_alpha = sin_theta2.mul_add(
                inv_sin_theta_max,
                cos_theta
                    * fast_max(
                        0.0,
                        sin_theta2.mul_add(-inv_sin_theta_max * inv_sin_theta_max, 1.0),
                    )
                    .sqrt(),
            );
            let sin_alpha = fast_max(0.0, cos_alpha.mul_add(-cos_alpha, 1.0)).sqrt();
            let (sin_phi, cos_phi) = f32::sin_cos(sample.y * TAU);

            let normal =
                spherical_to_cartesian_frame_trig(sin_phi, cos_phi, sin_alpha, cos_alpha, &frame);
            /* end PBR code */

            let point = self.center + self.radius * normal;
            let pdf = uniform_cone_pdf(cos_theta_max);

            SurfaceSample::new(point, normal, pdf)
        }
    }
}
