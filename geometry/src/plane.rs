use crate::debug_util::is_normalized;
use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use definitions::{Float, Vector3};
use serde::{Deserialize, Serialize};
use utility::floats::FloatExt;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    normal: Vector3,
    /// The distance of the plane into the normal direction
    d: Float,
}

impl Plane {
    pub fn new(normal: Vector3, d: Float) -> Self {
        debug_assert!(is_normalized(&normal));

        Self { normal, d }
    }
}

impl Boundable for Plane {
    fn bounds(&self) -> Aabb {
        let max = Vector3::broadcast(Float::INFINITY);

        Aabb::new(-max, max)
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let denom = self.normal.dot(ray.direction);

        if denom.is_approx_zero() {
            return None;
        }

        let p = self.normal * self.d - ray.origin;
        let t = p.dot(self.normal) / denom;
        if !ray.contains(t) {
            return None;
        }

        let point = ray.at(t);

        Some(Intersection::new(point, self.normal, t, *ray))
    }

    #[inline]
    fn intersects(&self, ray: &Ray) -> bool {
        self.normal.dot(ray.direction).is_approx_zero()
    }
}
