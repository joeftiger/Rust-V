use crate::{Aabb, Boundable, Container, Intersectable, Intersection, Ray};
use std::ops::Deref;
use ultraviolet::Vec3;
use utility::floats::fast_cmp;

pub struct Composite<T> {
    content: Vec<T>,
}

impl<T> Composite<T> {
    pub fn new(mut content: Vec<T>) -> Self {
        content.shrink_to_fit();
        Self { content }
    }
}

impl<T> Container for Composite<T>
where
    T: Deref<Target = dyn Container>,
{
    fn contains(&self, point: &Vec3) -> bool {
        self.content.iter().any(|c| c.contains(point))
    }
}

impl<T> Boundable for Composite<T>
where
    T: Deref<Target = dyn Boundable>,
{
    fn bounds(&self) -> Aabb {
        self.content
            .iter()
            .fold(Aabb::empty(), |acc, next| acc.join(&next.bounds()))
    }
}

impl<T> Intersectable for Composite<T>
where
    T: Deref<Target = dyn Intersectable>,
{
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let mut min_ray = *ray;

        self.content
            .iter()
            .filter_map(|c| {
                c.intersect(&min_ray).map(|i| {
                    min_ray.t_end = i.t;
                    i
                })
            })
            .min_by(|a, b| fast_cmp(a.t, b.t))
            .map(|mut i| {
                i.ray = *ray;
                i
            })
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.content.iter().any(|c| c.intersects(ray))
    }
}
