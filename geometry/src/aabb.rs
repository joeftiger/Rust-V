use crate::ray::Ray;
#[cfg(test)]
use crate::UNIT_VECTORS;
use crate::{Boundable, Container, Geometry, Intersectable, Intersection};
use definitions::{Float, Vector3};
use serde::{Deserialize, Serialize};
use utility::floats::FloatExt;

/// A cube represents an axis-aligned bounding box in 3 dimension. It is very efficient using only
/// 2 coordinates to represent such a box.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: Vector3,
    pub max: Vector3,
}

impl Aabb {
    /// Creates a new cube.
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
    pub fn new(min: Vector3, max: Vector3) -> Self {
        debug_assert_eq!(min, min.min_by_component(max));

        Self { min, max }
    }

    /// Returns the "empty cube", spanning from `min: INFINITY` to `max: NEG_INFINITY`.
    ///
    /// This cube is effectively **invalid**, but might be useful to compute bounding boxes
    /// of many objects, taking this empty cube as starting point.
    pub fn empty() -> Self {
        let min = Vector3::one() * Float::INFINITY;
        let max = Vector3::one() * Float::NEG_INFINITY;

        Self { min, max }
    }

    /// Returns the size of this cube in all 3 dimensions.
    ///
    /// # Returns
    /// * The size
    pub fn size(&self) -> Vector3 {
        self.max - self.min
    }

    /// Returns the volume of this cube.
    ///
    /// # Returns
    /// * The volume
    pub fn volume(&self) -> Float {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Returns the center of this cube.
    ///
    /// # Returns
    /// * The center
    pub fn center(&self) -> Vector3 {
        (self.min + self.max) / 2.0
    }

    /// Joins this cube with another one, effectively creating a cube spanning both cubes.
    ///
    /// # Arguments
    /// * `other` - Another cube
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
    fn contains(&self, point: &Vector3) -> bool {
        *point == point.clamped(self.min, self.max)
    }
}

impl Boundable for Aabb {
    fn bounds(&self) -> Aabb {
        *self
    }
}

impl Intersectable for Aabb {
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

        let t = if ray.contains(t_min) {
            t_min
        } else if ray.contains(t_max) {
            t_max
        } else {
            return None;
        };

        let point = ray.at(t);
        let half_size = self.size() / 2.0;
        let center = self.min + half_size;
        let direction = point - center;
        let bias = 1.0 + Float::big_epsilon();

        let mut normal = direction * bias / half_size;
        normal.apply(|f| f as i32 as Float);
        normal.normalize();

        Some(Intersection::new(point, normal, t, *ray))
    }

    fn intersects(&self, ray: &Ray) -> bool {
        let t1 = (self.min - ray.origin) / ray.direction;
        let t2 = (self.max - ray.origin) / ray.direction;

        let vec_min = t1.min_by_component(t2);
        let vec_max = t1.max_by_component(t2);

        let t_min = vec_min.component_max();
        let t_max = vec_max.component_min();

        t_min <= t_max && (ray.contains(t_min) || ray.contains(t_max))
    }
}

#[typetag::serde]
impl Geometry for Aabb {}

impl Default for Aabb {
    /// Constructs the default cube spanning the 3 dimensional space of `[-1, 1]`.
    ///
    /// # Returns
    /// * `[-1, 1]` Self
    fn default() -> Self {
        Self::new(-Vector3::one(), Vector3::one())
    }
}

#[test]
fn intersect_outside() {
    let mut origins = UNIT_VECTORS;
    origins.iter_mut().for_each(|v| *v *= 2.0);

    let mut directions = UNIT_VECTORS;
    directions.iter_mut().for_each(|v| *v *= -1.0);

    let cube = Aabb::default();

    for i in 0..UNIT_VECTORS.len() {
        let ray = Ray::new_fast(origins[i], directions[i]);

        let intersection = cube.intersect(&ray).unwrap();

        assert_eq!(UNIT_VECTORS[i], intersection.normal);
    }
}

#[test]
fn intersect_inside() {
    let origins = [Vector3::zero(); UNIT_VECTORS.len()];
    let directions = UNIT_VECTORS;

    let cube = Aabb::default();

    for i in 0..UNIT_VECTORS.len() {
        let ray = Ray::new_fast(origins[i], directions[i]);

        let intersection = cube.intersect(&ray).unwrap();

        // test normals
        assert_eq!(UNIT_VECTORS[i], intersection.normal);

        // test normal and ray direction show into the same general direction
        assert!(intersection.normal.dot(ray.direction) > 0.0);
    }
}
