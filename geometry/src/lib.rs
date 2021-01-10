pub mod aabb;
pub mod bvh;
mod debug_util;
mod point;
pub mod ray;
pub mod sphere;

use ultraviolet::Vec3;

use crate::debug_util::{is_finite, is_normalized};
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
/// Offsets a point by an epsilon into the normal direction, depending on the angle to the given
/// direction.
///
/// # Constraints
/// * `point` - ALl values should be finite (neither infinite nor `NaN`).
/// * `normal` - All values should be finite.
///              Should be normalized.
/// * `direction` - Should be finite.
///                 (Does not need to be normalized.)
///
/// # Arguments
/// * `point` - The starting point
/// * `normal` - The normal vector to offset towards
/// * `direction` - The direction helper to decide whether to invert the normal
///
/// # Returns
/// * The offset point
pub fn offset_point(point: Vec3, normal: Vec3, direction: Vec3) -> Vec3 {
    debug_assert!(is_finite(&point));
    debug_assert!(is_finite(&normal));
    debug_assert!(is_normalized(&normal));
    debug_assert!(is_finite(&direction));

    let offset =  if direction.dot(normal) >= 0.0 {
        normal * BIG_EPSILON
    } else {
        normal * -BIG_EPSILON
    };

    point + offset
}

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
        debug_assert!(is_normalized(&normal));

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
        debug_assert!(is_normalized(&direction));

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
        debug_assert!(is_normalized(&direction));

        let origin = offset_point(self.point, self.normal, direction);

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
        let direction = target - self.point;

        let origin = offset_point(self.point, self.normal, direction);

        Ray::new(origin, direction.normalized(), 0.0, direction.mag())
    }
}

/// # Summary
/// A coordinate system represents 3 orthogonal vectors in 3D space.
pub struct CoordinateSystem {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl CoordinateSystem {
    /// # Summary
    /// Creates a new coordinate system.
    ///
    /// # Constraints
    /// * `x` - All values must be finite (neither infinite nor `NaN`).
    ///          Should be normalized.
    /// * `y` - All values must be finite.
    ///          Should be normalized.
    /// * `z` - All values must be finite.
    ///          Should be normalized.
    ///
    /// # Arguments
    /// * `x` - The first vector
    /// * `y` - The second vector
    /// * `z` - The third vector
    ///
    /// # Returns
    /// * Self
    pub fn new(x: Vec3, y: Vec3, z: Vec3) -> Self {
        debug_assert!(is_finite(&x));
        debug_assert!(is_normalized(&x));
        debug_assert!(is_finite(&y));
        debug_assert!(is_normalized(&y));
        debug_assert!(is_finite(&z));
        debug_assert!(is_normalized(&z));

        Self { x, y, z }
    }

    /// # Summary
    /// Creates a new coordinate system around the given `y` direction vector.
    ///
    /// # Constraints
    /// * `y` - All values must be finite (neither infinite nor `NaN`).
    ///          Should be normalized.
    ///
    /// # Arguments
    /// * `y` - The y direction vector
    ///
    /// # Returns
    /// * Self
    pub fn from_y(y: Vec3) -> Self {
        debug_assert!(is_finite(&y));
        debug_assert!(is_normalized(&y));

        let x = if y.x.abs() > y.y.abs() {
            let inv_len = 1.0 / f32::sqrt(y.x * y.x + y.z * y.z);
            Vec3::new(-y.z * inv_len, 0.0, y.x * inv_len)
        } else {
            let inv_len = 1.0 / f32::sqrt(y.y * y.y + y.z * y.z);
            Vec3::new(0.0, y.z * inv_len, -y.y * inv_len)
        };

        let z = x.cross(y);

        Self::new(x, y, z)
    }

    /// # Summary
    /// Creates a new coordinate system around the given `z` direction vector.
    ///
    /// # Constraints
    /// * `z` - All values must be finite (neither infinite nor `NaN`).
    ///          Should be normalized.
    ///
    /// # Arguments
    /// * `z` - The y direction vector
    ///
    /// # Returns
    /// * Self
    pub fn from_z(z: Vec3) -> Self {
        debug_assert!(is_finite(&z));
        debug_assert!(is_normalized(&z));

        let x = if z.x.abs() > z.y.abs() {
            let inv_len = 1.0 / f32::sqrt(z.x * z.x + z.z * z.z);
            Vec3::new(-z.z * inv_len, 0.0, z.x * inv_len)
        } else {
            let inv_len = 1.0 / f32::sqrt(z.y * z.y + z.z * z.z);
            Vec3::new(0.0, z.z * inv_len, -z.y * inv_len)
        };
        let y = z.cross(x);

        Self::new(x, y, z)
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
