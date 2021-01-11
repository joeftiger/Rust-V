use crate::debug_utils::{is_finite, within_01};
use crate::mc::{uniform_cone_pdf, uniform_sample_sphere};
use crate::objects::emitter::{Sampleable, SurfaceSample};
use geometry::{from_spherical_direction, CoordinateSystem, Sphere};
use std::f32::consts::{PI, TAU};
use ultraviolet::{Lerp, Vec2, Vec3};
use utility::floats::BIG_EPSILON;

fn sample_surface_inside(sphere: &Sphere, point: &Vec3, sample: &Vec2) -> SurfaceSample {
    let p = uniform_sample_sphere(sample) * sphere.radius;
    let normal = p.normalized();

    let incident = p - *point;
    let mag_sq = incident.mag_sq();

    let pdf = if mag_sq > 0.0 {
        0.0
    } else {
        // Convert to solid angle measure
        mag_sq / (sphere.surface_area() * normal.dot(-incident.normalized()))
    };

    debug_assert!(pdf.is_finite());

    SurfaceSample::new(sphere.center + p, normal, pdf)
}

impl Sampleable for Sphere {
    fn surface_area(&self) -> f32 {
        4.0 * PI * self.radius * self.radius
    }

    // Copyright: https://github.com/mmp/pbrt-v3/blob/master/src/shapes/sphere.cpp
    fn sample_surface(&self, point: &Vec3, sample: &Vec2) -> SurfaceSample {
        debug_assert!(is_finite(point));
        debug_assert!(within_01(sample));

        let point_to_center = self.center - *point;
        let dist_sq = point_to_center.mag_sq();
        let r2 = self.radius * self.radius;

        if dist_sq - r2 < BIG_EPSILON {
            // inside the sphere (may happen)
            sample_surface_inside(&self, point, sample)
        } else {
            let coordinate_system =
                CoordinateSystem::from_z(point_to_center / -dist_sq.sqrt());

            let sin_theta_max = r2 / dist_sq;
            let sin_theta_max2 = sin_theta_max * sin_theta_max;
            let inv_sin_theta_max = 1.0 / sin_theta_max;
            let cos_theta_max = f32::max(0.0, 1.0 - sin_theta_max2);

            let mut cos_theta = cos_theta_max.lerp(1.0, sample.x);
            let mut sin_theta2 = 1.0 - cos_theta * cos_theta;

            if sin_theta_max2 < 0.00068523 {
                // sin^2(1.5 deg)
                // Fall back to a Taylor series expansion for small angles, where the standard
                // approach suffers from sever cancellation errors.
                sin_theta2 = sin_theta_max2 * sample.x;
                cos_theta = f32::sqrt(1.0 - sin_theta2);
            }

            let cos_alpha = sin_theta2
                * inv_sin_theta_max
                * cos_theta
                * f32::max(
                    0.0,
                    1.0 - sin_theta2 * inv_sin_theta_max * inv_sin_theta_max,
                )
                .sqrt();
            let sin_alpha = f32::max(0.0, 1.0 - cos_alpha * cos_alpha).sqrt();
            let phi = sample.y * TAU;

            let normal = from_spherical_direction(sin_alpha, cos_alpha, phi, &coordinate_system);
            let point = self.center + self.radius * normal;
            let pdf = uniform_cone_pdf(cos_theta_max);

            SurfaceSample::new(point, normal, pdf)
        }
    }
}
