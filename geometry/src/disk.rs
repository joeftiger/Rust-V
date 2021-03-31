use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use ultraviolet::Vec3;
use utility::floats::{approx_zero_tolerance, BIG_EPSILON};

pub struct Disk {
    pub center: Vec3,
    pub normal: Vec3,
    pub radius: f32,
}

impl Disk {
    pub fn new(center: Vec3, normal: Vec3, radius: f32) -> Self {
        Self {
            center,
            normal,
            radius,
        }
    }
}

impl Boundable for Disk {
    fn bounds(&self) -> Aabb {
        let offset = Vec3::one() * self.radius;
        let min = self.center - offset;
        let max = self.center + offset;

        Aabb::new(min, max)
    }
}

impl Intersectable for Disk {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let denom = self.normal.dot(ray.direction);

        if approx_zero_tolerance(denom, BIG_EPSILON) {
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

        if approx_zero_tolerance(denom, BIG_EPSILON) {
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
