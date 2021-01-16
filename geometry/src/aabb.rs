use crate::{Boundable, Container, Intersectable, Intersection, Ray};
use ultraviolet::Vec3;
use utility::floats::BIG_EPSILON;

#[derive(Copy, Clone)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// # Summary
    /// Creates a new aabb.
    ///
    /// # Constraints
    /// * The `min` components should be less-or-equal to the `max` components.
    ///
    /// # Arguments
    /// * `min` - The min corner
    /// * `max` - The max corner
    ///
    /// # Returns
    /// * Self
    pub fn new(min: Vec3, max: Vec3) -> Self {
        debug_assert_eq!(min, min.min_by_component(max));

        Self { min, max }
    }

    /// # Summary
    /// Returns the "empty aabb", spanning from `min: INFINITY` to `max: NEG_INFINITY`.
    ///
    /// This aabb is effectively **invalid**, but might be useful to compute bounding boxes
    /// of many objects, taking this empty aabb as starting point.
    pub fn empty() -> Self {
        let min = Vec3::one() * f32::INFINITY;
        let max = Vec3::one() * f32::NEG_INFINITY;

        Self { min, max }
    }

    /// # Summary
    /// Returns the size of this aabb in all 3 dimensions.
    ///
    /// # Returns
    /// * The size
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// # Summary
    /// Returns the center of this aabb.
    ///
    /// # Returns
    /// * The center
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) / 2.0
    }

    /// # Summary
    /// Joins this aabb with another one, effectively creating a aabb spanning both aabbs.
    ///
    /// # Arguments
    /// * `other` - Another aabb
    ///
    /// # Returns
    /// * The outer join
    pub fn join(&self, other: &Self) -> Self {
        let min = self.min.min_by_component(other.min);
        let max = self.max.max_by_component(other.max);
        Self::new(min, max)
    }
}

impl Container for Aabb {
    fn contains(&self, point: &Vec3) -> bool {
        *point == point.clamped(self.min, self.max)
    }
}

impl Boundable for Aabb {
    fn bounds(&self) -> Aabb {
        *self
    }
}

impl Intersectable for Aabb {
    /// # Summary
    /// Intersects the given ray **as infinite ray** with this object. Upon intersection, it will return some
    /// intersection info containing the reference to the infinite ray.
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect with (interpreted as infinite ray)
    ///
    /// # Returns
    /// * Intersection or `None`
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let t1 = (self.min - ray.origin) / ray.direction;
        let t2 = (self.max - ray.origin) / ray.direction;

        let vec_min = t1.min_by_component(t2);
        let vec_max = t1.max_by_component(t2);

        let t_min = vec_min.component_max();
        let t_max = vec_max.component_min();

        if t_min > t_max {
            return None;
        }

        let t = if t_min >= 0.0 {
            t_min
        } else if t_max >= 0.0 {
            t_max
        } else {
            return None;
        };

        let point = ray.at(t);
        let half_size = self.size() / 2.0;
        let center = self.min + half_size;
        let direction = point - center;
        let bias = 1.0 + BIG_EPSILON;

        let mut normal = direction * bias / half_size;
        normal.x = (normal.x as i32) as f32;
        normal.y = (normal.y as i32) as f32;
        normal.z = (normal.z as i32) as f32;
        normal.normalize();

        Some(Intersection::new(
            point,
            normal,
            t,
            Ray::new_fast(ray.origin, ray.direction),
        ))
    }

    /// # Summary
    /// Checks whether the given ray **as infinite ray** intersects with this object.
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect with (interpreted as infinite ray)
    ///
    /// # Returns
    /// * Whether an intersection occurs
    fn intersects(&self, ray: &Ray) -> bool {
        if self.contains(&ray.origin) {
            return true;
        }

        let t1 = (self.min - ray.origin) / ray.direction;
        let t2 = (self.max - ray.origin) / ray.direction;

        let vec_min = t1.min_by_component(t2);
        let vec_max = t1.max_by_component(t2);

        let t_min = vec_min.component_max();
        let t_max = vec_max.component_min();

        t_min <= t_max && (t_min >= 0.0 || t_max >= 0.0)
    }
}
