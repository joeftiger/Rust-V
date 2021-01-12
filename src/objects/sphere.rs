use crate::debug_utils::{is_finite, within_01};
use crate::mc::{uniform_cone_pdf, uniform_sample_cone_frame, uniform_sample_sphere};
use crate::objects::emitter::{Sampleable, SurfaceSample};
use geometry::{CoordinateSystem, Sphere};
use std::f32::consts::PI;
use ultraviolet::{Vec2, Vec3};

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

        if dist_sq <= r2 {
            // inside the sphere (may happen)
            sample_surface_inside(&self, point, sample)
        } else {
            let frame = CoordinateSystem::from_y(point_to_center / -dist_sq.sqrt());

            let cos_theta_max = f32::max(0.0, 1.0 - r2 / dist_sq).sqrt();

            let normal = uniform_sample_cone_frame(sample, cos_theta_max, &frame);
            let point = self.center + self.radius * normal;
            let pdf = uniform_cone_pdf(cos_theta_max);

            SurfaceSample::new(point, normal, pdf)
        }
    }
}
