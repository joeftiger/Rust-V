mod aabb;
pub mod bvh;
mod cube;
mod debug_util;
mod point;
pub mod ray;
pub mod sphere;

use ultraviolet::Vec3;

use crate::debug_util::{is_finite, is_normalized};
pub use aabb::Aabb;
pub use cube::Cube;
pub use point::Point;
pub use ray::Ray;
pub use sphere::Sphere;
use utility::floats::BIG_EPSILON;

/// # Summary
/// The unit vectors in all directions.
#[rustfmt::skip]
pub const UNIT_VECTORS: [Vec3; 6] = [
    Vec3 { x: 1.0, y: 0.0, z: 0.0 },
    Vec3 { x: -1.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    Vec3 { x: 0.0, y: -1.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 1.0 },
    Vec3 { x: 0.0, y: 0.0, z: -1.0 },
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

    let offset = if direction.dot(normal) >= 0.0 {
        normal * BIG_EPSILON
    } else {
        normal * -BIG_EPSILON
    };

    point + offset
}

/// # Summary
/// Converts the given angles in the given coordinate system from a spherical coordinate system
/// into a cartesian direction.
///
/// # Constraints
/// * `sin_theta` - Should be finite (neither infinite nor `NaN`).
/// * `cos_theta` - Should be finite.
/// * `phi` - Should be finite.
///
/// # Arguments
/// * `sin_theta` - The sine of the theta angle
/// * `cos_theta` - The cosine of the theta angle
/// * `phi` - The phi angle (in radians)
/// * `frame` - The coordinate system frame
///
/// * Returns
/// * Normalized direction in cartesian coordinate system
pub fn from_spherical_direction(
    sin_theta: f32,
    cos_theta: f32,
    phi: f32,
    frame: &CoordinateSystem,
) -> Vec3 {
    debug_assert!(sin_theta.is_finite());
    debug_assert!(cos_theta.is_finite());
    debug_assert!(phi.is_finite());

    let (sin_phi, cos_phi) = phi.sin_cos();

    let x = frame.x_axis * sin_phi * sin_theta;
    let y = frame.y_axis * cos_phi;
    let z = frame.z_axis * sin_phi * cos_phi;

    (x + y + z).normalized()
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
/// A coordinate system represents 3 (orthogonal) vectors in 3D space.
pub struct CoordinateSystem {
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
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

        Self {
            x_axis: x,
            y_axis: y,
            z_axis: z,
        }
    }

    /// # Summary
    /// Creates a new coordinate system around the given `x` direction vector.
    ///
    /// # Constraints
    /// * `x_axis` - All values must be finite (neither infinite nor `NaN`).
    ///              Should be normalized.
    ///
    /// # Arguments
    /// * `x_axis` - The x direction vector
    ///
    /// # Returns
    /// * Self
    // TODO: Make more efficient
    pub fn from_x(x_axis: Vec3) -> Self {
        debug_assert!(is_finite(&x_axis));
        debug_assert!(is_normalized(&x_axis));

        if x_axis == Vec3::unit_x() {
            Self::new(Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z())
        } else {
            let z = x_axis.cross(Vec3::unit_x()).normalized();
            let y = z.cross(x_axis).normalized();

            Self::new(x_axis, y, z)
        }
    }

    /// # Summary
    /// Creates a new coordinate system around the given `y` direction vector.
    ///
    /// # Constraints
    /// * `y_axis` - All values must be finite (neither infinite nor `NaN`).
    ///              Should be normalized.
    ///
    /// # Arguments
    /// * `y_axis` - The y direction vector
    ///
    /// # Returns
    /// * Self
    // TODO: Make more efficient
    pub fn from_y(y_axis: Vec3) -> Self {
        debug_assert!(is_finite(&y_axis));
        debug_assert!(is_normalized(&y_axis));

        if y_axis == Vec3::unit_y() {
            Self::new(Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z())
        } else {
            let x = y_axis.cross(Vec3::unit_y()).normalized();
            let z = x.cross(y_axis).normalized();

            Self::new(x, y_axis, z)
        }
    }

    /// # Summary
    /// Creates a new coordinate system around the given `z` direction vector.
    ///
    /// # Constraints
    /// * `z_axis` - All values must be finite (neither infinite nor `NaN`).
    ///              Should be normalized.
    ///
    /// # Arguments
    /// * `z_axis` - The z direction vector
    ///
    /// # Returns
    /// * Self
    // TODO: Make more efficient
    pub fn from_z(z_axis: Vec3) -> Self {
        debug_assert!(is_finite(&z_axis));
        debug_assert!(is_normalized(&z_axis));

        if z_axis == Vec3::unit_z() {
            Self::new(Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z())
        } else {
            let x = z_axis.cross(Vec3::unit_z()).normalized();
            let y = z_axis.cross(x).normalized();

            Self::new(x, y, z_axis)
        }
    }
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self::new(Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z())
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
/// A trait for objects that can report a cube as their bounds.
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
    /// use geometry::{Cube, Intersectable, Ray};
    /// use ultraviolet::Vec3;
    ///
    /// let cube = Cube::default();
    /// let mut ray = Ray::new_fast(Vec3::zero(), Vec3::unit_x());
    ///
    /// let intersection = cube.intersect(&mut ray).unwrap(); // intersects surely
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
