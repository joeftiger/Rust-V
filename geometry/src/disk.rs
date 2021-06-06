use crate::*;
use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use utility::floats::FloatExt;

pub struct Disk {
    pub center: Vector3,
    pub normal: Vector3,
    pub radius: Float,
}

impl Disk {
    pub fn new(center: Vector3, normal: Vector3, radius: Float) -> Self {
        Self {
            center,
            normal,
            radius,
        }
    }
}

impl Boundable for Disk {
    fn bounds(&self) -> Aabb {
        let offset = Vector3::one() * self.radius;
        let min = self.center - offset;
        let max = self.center + offset;

        Aabb::new(min, max)
    }
}

impl Intersectable for Disk {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let denom = self.normal.dot(ray.direction);

        if denom.is_approx_zero() {
            return None;
        }

        let p = self.center - ray.origin;
        let t = p.dot(self.normal) / denom;
        if !ray.contains(t) {
            return None;
        }

        let point = ray.at(t);

        if (point - self.center).mag_sq() > self.radius * self.radius {
            return None;
        }

        Some(Intersection::new(point, self.normal, t, *ray))
    }

    fn intersects(&self, ray: &Ray) -> bool {
        let denom = self.normal.dot(ray.direction);

        if denom.is_approx_zero() {
            return false;
        }

        let p = self.center - ray.origin;
        let t = p.dot(self.normal) / denom;
        if !ray.contains(t) {
            return false;
        }

        let point = ray.at(t);

        (point - self.center).mag_sq() <= self.radius * self.radius
    }
}

#[typetag::serde]
impl Geometry for Aabb {}
