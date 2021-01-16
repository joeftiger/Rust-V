use crate::debug_util::is_finite;
use crate::ray::Ray;
#[cfg(test)]
use crate::UNIT_VECTORS;
use crate::{Aabb, Boundable, Container, Intersectable, Intersection};
use ultraviolet::Vec3;
use utility::math::solve_quadratic;

/// # Summary
/// A sphere consists of a center and a radius.
#[derive(Copy, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    /// # Summary
    /// Creates a new sphere.
    ///
    /// # Constraints
    /// * The `radius` should be greater than `0.0`.
    ///
    /// # Arguments
    /// * `center` - The center
    /// * `radius` - The radius
    ///
    /// # Returns
    /// Self
    pub fn new(center: Vec3, radius: f32) -> Self {
        debug_assert!(is_finite(&center));
        debug_assert!(radius > 0.0);

        Self { center, radius }
    }
}

impl Container for Sphere {
    fn contains(&self, point: &Vec3) -> bool {
        (*point - self.center).mag_sq() <= self.radius * self.radius
    }
}

impl Boundable for Sphere {
    fn bounds(&self) -> Aabb {
        let diff = Vec3::one() * self.radius;
        let min = self.center - diff;
        let max = self.center + diff;

        Aabb::new(min, max)
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let dir = ray.direction;
        let oc = ray.origin - self.center;

        let a = dir.dot(dir);
        let b = 2.0 * dir.dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;

        let (t_min, t_max) = solve_quadratic(a, b, c)?;

        let t = if ray.contains(t_min) {
            t_min
        } else if ray.contains(t_max) {
            t_max
        } else {
            return None;
        };

        let point = ray.at(t);
        let normal = (point - self.center).normalized();

        Some(Intersection::new(point, normal, t, *ray))
    }

    fn intersects(&self, ray: &Ray) -> bool {
        let dir = ray.direction;
        let oc = ray.origin - self.center;

        if !ray.contains(oc.mag() - self.radius) {
            return false;
        }

        let a = dir.dot(dir);
        let b = 2.0 * dir.dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;

        if let Some((t_min, t_max)) = solve_quadratic(a, b, c) {
            ray.contains(t_min) || ray.contains(t_max)
        } else {
            false
        }
    }
}

impl Default for Sphere {
    /// # Summary
    /// Constructs the default sphere of radius `1.0` around the zero point.
    ///
    /// # Returns
    /// * Self
    fn default() -> Self {
        Self::new(Vec3::zero(), 1.0)
    }
}

#[test]
fn intersect_outside() {
    let mut origins = UNIT_VECTORS;
    origins.iter_mut().for_each(|v| *v *= 2.0);

    let mut directions = UNIT_VECTORS;
    directions.iter_mut().for_each(|v| *v *= -1.0);

    let sphere = Sphere::default();

    for i in 0..UNIT_VECTORS.len() {
        let ray = Ray::new_fast(origins[i], directions[i]);

        let intersection = sphere.intersect(&ray).unwrap();

        assert_eq!(UNIT_VECTORS[i], intersection.normal);
    }
}

#[test]
fn intersect_inside() {
    let origins = [Vec3::zero(); UNIT_VECTORS.len()];
    let directions = UNIT_VECTORS;

    let sphere = Sphere::default();

    for i in 0..UNIT_VECTORS.len() {
        let ray = Ray::new_fast(origins[i], directions[i]);

        let intersection = sphere.intersect(&ray).unwrap();

        // test normals
        assert_eq!(UNIT_VECTORS[i], intersection.normal);

        // test normal and ray direction show into the same general direction
        assert!(intersection.normal.dot(ray.direction) > 0.0);
    }
}
