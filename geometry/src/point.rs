use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use ultraviolet::Vec3;

/// Represents a point in space.
#[derive(Copy, Clone, PartialEq)]
pub struct Point(pub Vec3);

impl Boundable for Point {
    fn bounds(&self) -> Aabb {
        Aabb::new(self.0, self.0)
    }
}

impl Intersectable for Point {
    /// A point never intersects.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `None`
    fn intersect(&self, _: &Ray) -> Option<Intersection> {
        None
    }

    /// A point never intersects.
    ///
    /// # Arguments
    /// Ignored
    ///
    /// # Returns
    /// * `false`
    fn intersects(&self, _: &Ray) -> bool {
        false
    }
}
