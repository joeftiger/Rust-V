use crate::debug_util::is_finite;
use crate::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};
use definitions::{Float, Vector3};
use serde::{Deserialize, Serialize};
use utility::math::solve_quadratic;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cylinder {
    caps: (Vector3, Vector3),
    radius: Float,
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
    pub fn new(caps: (Vector3, Vector3), radius: Float) -> Self {
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
    pub fn center(&self) -> Vector3 {
        (self.caps.0 + self.caps.1) / 2.0
    }

    /// Returns the axis of `caps.0` to `caps.1`.
    ///
    /// # Returns
    /// * The axis
    pub fn axis(&self) -> Vector3 {
        (self.caps.1 - self.caps.0).normalized()
    }

    /// Returns the height of this cylinder.
    ///
    /// # Returns
    /// * The height
    pub fn height(&self) -> Float {
        (self.caps.1 - self.caps.0).mag()
    }
}

impl Boundable for Cylinder {
    // TODO: Not a close-fit bounding box
    fn bounds(&self) -> Aabb {
        let offset = Vector3::one() * self.radius;
        let min = self.caps.0.min_by_component(self.caps.1) - offset;
        let max = self.caps.0.max_by_component(self.caps.1) + offset;

        Aabb::new(min, max)
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

        let filter = |t: Float| {
            if ray.contains(t) {
                let point = ray.at(t);
                let center_to_point = point - center;
                let z = center_to_point.dot(axis);

                if 2.0 * z.abs() < height {
                    return Some((t, point, center_to_point));
                }
            }

            None
        };

        let (t, point, center_to_point) = filter(t_min).or_else(|| filter(t_max))?;

        let mut normal = center_to_point / self.radius;
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
            let center = self.center();
            let height = self.height();

            let filter = |t: Float| {
                if ray.contains(t) {
                    let point = ray.at(t);
                    let center_to_point = point - center;
                    let z = center_to_point.dot(axis);

                    if 2.0 * z.abs() < height {
                        return true;
                    }
                }

                false
            };

            filter(t_min) || filter(t_max)
        } else {
            false
        }
    }
}

#[typetag::serde]
impl Geometry for Cylinder {}
