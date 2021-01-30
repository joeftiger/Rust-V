use crate::debug_util::is_finite;
use crate::{Boundable, Cube, Intersectable, Intersection, Ray};
use ultraviolet::Vec3;
use utility::math::solve_quadratic;

pub struct Cylinder {
    caps: (Vec3, Vec3),
    radius: f32,
}

impl Cylinder {
    /// # Summary
    /// Creates a new cylinder.
    ///
    /// # Constraints
    /// * `caps` - All values should be finite (neither infinite nor `NaN`).
    /// * `radius` - Should be finite.
    ///              Should be in range `(0, inf)`.
    ///
    /// # Arguments
    /// * `caps` - The caps of the cylinder
    /// * `radius` - The radius
    ///
    /// # Returns
    /// * Self
    pub fn new(caps: (Vec3, Vec3), radius: f32) -> Self {
        debug_assert!(is_finite(&caps.0));
        debug_assert!(is_finite(&caps.1));
        debug_assert!(radius.is_finite());
        debug_assert!(radius > 0.0);

        Self { caps, radius }
    }

    /// # Summary
    /// Returns the center of this cylinder.
    ///
    /// # Returns
    /// * The center
    pub fn center(&self) -> Vec3 {
        (self.caps.0 + self.caps.1) / 2.0
    }

    /// # Summary
    /// Returns the axis of `caps.0` to `caps.1`.
    ///
    /// # Returns
    /// * The axis
    pub fn axis(&self) -> Vec3 {
        (self.caps.1 - self.caps.0).normalized()
    }

    /// # Summary
    /// Returns the height of this cylinder.
    ///
    /// # Returns
    /// * The height
    pub fn height(&self) -> f32 {
        (self.caps.1 - self.caps.0).mag()
    }
}

impl Boundable for Cylinder {
    // TODO: Not a close-fit bounding box
    fn bounds(&self) -> Cube {
        let offset = Vec3::one() * self.radius;
        let min = self.caps.0.min_by_component(self.caps.1) - offset;
        let max = self.caps.0.max_by_component(self.caps.1) + offset;

        Cube::new(min, max)
    }
}

impl Intersectable for Cylinder {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let dir = ray.direction;
        let oc = ray.origin - self.center();
        let axis = self.axis();

        let dir_parallel = axis.dot(dir);
        let oc_parallel = axis.dot(oc);

        let a = dir.dot(dir) - dir_parallel * dir_parallel;
        let b = 2.0 * (dir.dot(oc) - dir_parallel * oc_parallel);
        let c = oc.dot(oc) - oc_parallel * oc_parallel - self.radius * self.radius;

        let (t_min, t_max) = solve_quadratic(a, b, c)?;

        check_cylinder(self, ray, t_min).or(check_cylinder(self, ray, t_max))
    }

    fn intersects(&self, ray: &Ray) -> bool {
        let dir = ray.direction;
        let oc = ray.origin - self.center();
        let axis = self.axis();

        let dir_parallel = axis.dot(dir);
        let oc_parallel = axis.dot(oc);

        let a = dir.dot(dir) - dir_parallel * dir_parallel;
        let b = 2.0 * (dir.dot(oc) - dir_parallel * oc_parallel);
        let c = oc.dot(oc) - oc_parallel * oc_parallel - self.radius * self.radius;

        if let Some((t_min, t_max)) = solve_quadratic(a, b, c) {
            ray.contains(t_min) || ray.contains(t_max)
        } else {
            false
        }
    }
}

fn check_cylinder(c: &Cylinder, ray: &Ray, t: f32) -> Option<Intersection> {
    if !ray.contains(t) {
        return None;
    }

    let point = ray.at(t);

    let (axis, height) = {
        let ax = c.caps.1 - c.caps.0;
        (ax.normalized(), ax.mag())
    };
    let center = c.caps.0 + height / 2.0 * axis;

    let z = 2.0 * axis.dot(point - center);
    if z.abs() <= height {
        let mut normal = (point - center) / c.radius;
        normal -= normal.dot(axis) * axis;

        Some(Intersection::new(point, normal, t, *ray))
    } else {
        None
    }
}
