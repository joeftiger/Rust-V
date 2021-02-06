use crate::debug_util::is_finite;
use crate::{Boundable, Cube, Intersectable, Intersection, Ray};
use ultraviolet::Vec3;
use utility::math::solve_quadratic;

pub struct Cylinder {
    caps: (Vec3, Vec3),
    radius: f32,
}

impl Cylinder {
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

    /// Returns the center of this cylinder.
    ///
    /// # Returns
    /// * The center
    pub fn center(&self) -> Vec3 {
        (self.caps.0 + self.caps.1) / 2.0
    }

    /// Returns the axis of `caps.0` to `caps.1`.
    ///
    /// # Returns
    /// * The axis
    pub fn axis(&self) -> Vec3 {
        (self.caps.1 - self.caps.0).normalized()
    }

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

        let height = self.height();
        let center = self.center();

        let filter = |t: f32| {
            if ray.contains(t_min) {
                let point = ray.at(t_min);
                let diff = point - center;
                let z = diff.dot(axis);

                if 2.0 * z.abs() < height {
                    return Some((t, point, diff));
                }
            }

            None
        };

        let (t, point, diff) = filter(t_min).or_else(|| filter(t_max))?;

        let mut normal = diff / self.radius;
        normal -= normal.dot(axis) * axis;

        if normal.dot(dir) > 0.0 {
            normal = -normal;
        }

        Some(Intersection::new(point, normal, t, *ray))
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