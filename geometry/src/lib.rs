pub mod aabb;
pub mod bvh;
mod debug_util;
mod point;
pub mod ray;
pub mod sphere;

use ultraviolet::Vec3;

pub use aabb::Aabb;
pub use point::Point;
pub use ray::Ray;
pub use sphere::Sphere;
use utility::floats::BIG_EPSILON;

/// # Summary
/// The unit vectors in all directions.
#[rustfmt::skip]
pub const UNIT_VECTORS: [Vec3; 6] = [
    Vec3 { x:  1.0, y:  0.0, z:  0.0 },
    Vec3 { x: -1.0, y:  0.0, z:  0.0 },
    Vec3 { x:  0.0, y:  1.0, z:  0.0 },
    Vec3 { x:  0.0, y: -1.0, z:  0.0 },
    Vec3 { x:  0.0, y:  0.0, z:  1.0 },
    Vec3 { x:  0.0, y:  0.0, z: -1.0 },
];

/// # Summary
/// An intersection consists of the following 4 properties:
/// * `point` - The intersection point
/// * `normal` - The surface normal (showing outside, even if intersection hits inside!)
/// * `t` - The ray parameter at which it intersects
/// * `ray` - The reference to the intersecting ray
#[derive(Clone)]
pub struct Intersection {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub ray: Ray,
}

impl Intersection {
    /// # Summary
    /// Creates a new intersection.
    ///
    /// # Constraints
    /// * `normal` - Should be normalized.
    /// * `t` - Should be in range of the ray.
    ///
    /// # Arguments
    /// * `point` - The intersection point
    /// * `normal` - The surface normal
    /// * `t` - The ray parameter
    /// * `ray` - The reference to the intersecting ray
    ///
    /// # Returns
    /// * Self
    pub fn new(point: Vec3, normal: Vec3, t: f32, ray: Ray) -> Self {
        debug_assert!(ray.contains(t));
        debug_assert!({
            let mag = normal.mag();
            1.0 - BIG_EPSILON < mag && mag < 1.0 + BIG_EPSILON
        });

        Self {
            point,
            normal,
            t,
            ray,
        }
    }

    /// # Summary
    /// Creates a ray from this intersection into the given direction.
    ///
    /// # Constraints
    /// * `direction` - Should be normalized.
    ///
    /// # Arguments
    /// * `direction` - The direction of the ray
    ///
    /// # Returns
    /// * Ray from this intersection
    pub fn ray_towards(&self, direction: Vec3) -> Ray {
        debug_assert!({
            let mag = direction.mag();
            1.0 - BIG_EPSILON < mag && mag < 1.0 + BIG_EPSILON
        });

        Ray::new_fast(self.point, direction)
    }

    /// # Summary
    /// Creates a ray from this intersection to the given target.
    ///
    /// # Arguments
    /// * `target` - The target position
    ///
    /// # Returns
    /// * Ray from this intersection to the target
    pub fn ray_to(&self, target: Vec3) -> Ray {
        Ray::between(self.point, target)
    }

    /// # Summary
    /// Creates a ray from this intersection into the given direction.
    ///
    /// If the parameter `direction` shows into the same general direction of this intersection
    /// normal, the ray origin will be offset by an epsilon into the intersection normal.
    /// Otherwise, the opposite normal will be used.
    ///
    /// # Constraints
    /// * `direction` - Should be normalized.
    ///
    /// # Arguments
    /// * `direction` - The direction of the ray
    ///
    /// # Returns
    /// * Ray from this intersection, offset by an epsilon
    pub fn offset_ray_towards(&self, direction: Vec3) -> Ray {
        debug_assert!({
            let mag = direction.mag();
            1.0 - BIG_EPSILON < mag && mag < 1.0 + BIG_EPSILON
        });

        let offset = if self.normal.dot(direction) > 0.0 {
            self.normal * BIG_EPSILON
        } else {
            self.normal * -BIG_EPSILON
        };

        let origin = self.point + offset;

        Ray::new_fast(origin, direction)
    }

    /// # Summary
    /// Creates a ray from this intersection to the given target.
    ///
    /// If the direction to the paremeter `target` shows into the same general direction of this
    /// intersection normal, the ray origin will be offset by an epsilon into the intersection
    /// normal.
    /// Otherwise, the opposite normal will be used.
    ///
    /// # Arguments
    /// * `target` - The target position
    ///
    /// # Returns
    /// * Ray from this intersection to the target, offset by an epsilon
    pub fn offset_ray_to(&self, target: Vec3) -> Ray {
        let offset = if self.normal.dot(target - self.point) >= 0.0 {
            self.normal * BIG_EPSILON
        } else {
            self.normal * -BIG_EPSILON
        };

        let origin = self.point + offset;

        Ray::between(origin, target)
    }
}

/// # Summary
/// A trait for objects that surround space and may contain points.
pub trait Container {
    /// # Summary
    /// Returns whether this object contains the given `piint` bounds-inclusive.
    ///
    /// # Arguments
    /// * `point` - A point in space
    ///
    /// # Returns
    /// * Whether the given point is inside-inclusive this object
    fn contains(&self, point: &Vec3) -> bool;
}

/// # Summary
/// A trait for objects that can report an aabb as their bounds.
pub trait Boundable {
    /// # Summary
    /// Returns the bounds of this object
    ///
    /// # Returns
    /// * The bounds
    fn bounds(&self) -> Aabb;
}

/// # Summary
/// A trait for objects that can be intersected by rays.
pub trait Intersectable {
    /// # Summary
    /// Intersects the given ray with this object. Upon intersection, it will return some
    /// intersection info containing the reference to the given ray.
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect with
    ///
    /// # Return Constraints
    /// The **intersection normal** always points to the **outside**.
    /// To obtain the normal from inside the object, one can use following
    /// ```rust
    /// use geometry::{Aabb, Intersectable, Ray};
    /// use ultraviolet::Vec3;
    ///
    /// let aabb = Aabb::default();
    /// let mut ray = Ray::new_fast(Vec3::zero(), Vec3::unit_x());
    ///
    /// let intersection = aabb.intersect(&mut ray).unwrap(); // intersects surely
    ///
    /// let mut normal = intersection.normal;
    /// assert_eq!(Vec3::unit_x(), normal); // normal points to the outside
    ///
    /// // true geometric normal is inverted if both vectors point into the same general direction
    /// if normal.dot(ray.direction) > 0.0 {
    ///     normal = -normal;
    /// }
    ///
    /// assert_eq!(-Vec3::unit_x(), normal); // normal points to the outside
    /// ```
    ///
    /// # Returns
    /// * Intersection or `None`
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;

    /// # Summary
    /// Checks whether the given ray intersects with this object.
    /// Unless overridden, it naively checks if `intersect(ray)` is `Some`.
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect with
    ///
    /// # Returns
    /// * Whether an intersection occurs
    fn intersects(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }
}

/// # Summary
/// A super-trait to combine `Boundable` and `Intersectable`, therefore giving a valid geometry.
pub trait Geometry: Boundable + Intersectable {}
impl<T> Geometry for T where T: Boundable + Intersectable {}
