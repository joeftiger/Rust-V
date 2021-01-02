use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use ultraviolet::Vec3;

/// # Summary
/// Represents a point in space.
pub struct Point(pub Vec3);

impl Boundable for Point {
    fn bounds(&self) -> Aabb {
        Aabb::new(self.0, self.0)
    }
}

impl Intersectable for Point {
    /// # Summary
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

    /// # Summary
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
