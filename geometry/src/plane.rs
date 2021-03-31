use crate::debug_util::is_normalized;
use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;
use utility::floats::{approx_zero_tolerance, BIG_EPSILON};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    normal: Vec3,
    /// The distance of the plane into the normal direction
    d: f32,
}

impl Plane {
    pub fn new(normal: Vec3, d: f32) -> Self {
        debug_assert!(is_normalized(&normal));

        Self { normal, d }
    }
}

impl Boundable for Plane {
    fn bounds(&self) -> Aabb {
        let max = Vec3::broadcast(f32::INFINITY);

        Aabb::new(-max, max)
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let denom = self.normal.dot(ray.direction);

        if approx_zero_tolerance(denom, BIG_EPSILON) {
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
        approx_zero_tolerance(self.normal.dot(ray.direction), BIG_EPSILON)
    }
}
