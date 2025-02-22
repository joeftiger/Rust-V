mod aabb;
mod bubble;
pub mod bvh;
mod composite;
mod cylinder;
mod debug_util;
mod disk;
mod lenses;
mod mesh;
pub mod obj_file;
mod plane;
mod point;
mod ray;
mod sphere;

#[cfg(not(feature = "f64"))]
pub type Float = f32;
#[cfg(not(feature = "f64"))]
pub type Vector3 = ultraviolet::Vec3;
#[cfg(not(feature = "f64"))]
pub type Rotation3 = ultraviolet::Rotor3;
#[cfg(not(feature = "f64"))]
pub type Matrix3 = ultraviolet::Mat3;
#[cfg(not(feature = "f64"))]
pub type Matrix4 = ultraviolet::Mat4;
#[cfg(not(feature = "f64"))]
use std::f32::consts::{PI, TAU};
#[cfg(feature = "f64")]
pub type Float = f64;
#[cfg(feature = "f64")]
pub type Vector3 = ultraviolet::DVec3;
#[cfg(feature = "f64")]
pub type Rotation3 = ultraviolet::DRotor3;
#[cfg(feature = "f64")]
pub type Matrix3 = ultraviolet::DMat3;
#[cfg(feature = "f64")]
pub type Matrix4 = ultraviolet::DMat4;
#[cfg(feature = "f64")]
use std::f64::consts::{PI, TAU};

pub use plane::*;

use crate::debug_util::{is_finite, is_normalized};
pub use aabb::*;
pub use bubble::*;
pub use composite::*;
pub use cylinder::*;
pub use disk::*;
pub use lenses::*;
pub use mesh::*;
pub use point::*;
pub use ray::*;
pub use sphere::*;
use utility::floats::FloatExt;

/// The unit vectors in all directions.
#[rustfmt::skip]
pub const UNIT_VECTORS: [Vector3; 6] = [
    Vector3 { x: 1.0, y: 0.0, z: 0.0 },
    Vector3 { x: -1.0, y: 0.0, z: 0.0 },
    Vector3 { x: 0.0, y: 1.0, z: 0.0 },
    Vector3 { x: 0.0, y: -1.0, z: 0.0 },
    Vector3 { x: 0.0, y: 0.0, z: 1.0 },
    Vector3 { x: 0.0, y: 0.0, z: -1.0 },
];

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
pub fn offset_point(point: Vector3, normal: Vector3, direction: Vector3) -> Vector3 {
    debug_assert!(is_finite(&point));
    debug_assert!(is_finite(&normal));
    debug_assert!(is_normalized(&normal));
    debug_assert!(is_finite(&direction));

    let offset = if direction.dot(normal) >= 0.0 {
        normal * Float::big_epsilon()
    } else {
        normal * -Float::big_epsilon()
    };

    point + offset
}

/// Offsets a point by an epsilon into the normal direction, depending on the angle to the given
/// direction and creates a ray from it.
///
/// If the parameter `direction` shows into the same general direction of this intersection
/// normal, the ray origin will be offset by an epsilon into the intersection normal.
/// Otherwise, the opposite normal will be used.
///
/// # Constraints
/// * `point` - ALl values should be finite (neither infinite nor `NaN`).
/// * `normal` - All values should be finite.
///              Should be normalized.
/// * `direction` - Should be finite.
///                 Should be normalized.
///
/// # Arguments
/// * `point` - The starting point
/// * `normal` - The normal vector to offset towards
/// * `direction` - The direction of the ray
///
/// # Returns
/// * Ray from this intersection, offset by an epsilon
pub fn offset_ray_towards(point: Vector3, normal: Vector3, direction: Vector3) -> Ray {
    debug_assert!(is_finite(&point));
    debug_assert!(is_finite(&normal));
    debug_assert!(is_normalized(&normal));
    debug_assert!(is_finite(&direction));
    debug_assert!(is_normalized(&direction));

    let origin = offset_point(point, normal, direction);

    Ray::new_fast(origin, direction)
}

/// Offsets a point by an epsilon into the normal direction, depending on the angle to the given
/// direction and creates a ray to the target from it.
///
/// If the direction to the parameter `target` shows into the same general direction of this
/// intersection normal, the ray origin will be offset by an epsilon into the intersection
/// normal.
/// Otherwise, the opposite normal will be used.
///
/// # Constraints
/// * `point` - ALl values should be finite (neither infinite nor `NaN`).
/// * `normal` - All values should be finite.
///              Should be normalized.
/// * `target` - Should be finite.
///
/// # Arguments
/// * `point` - The starting point
/// * `normal` - The normal vector to offset towards
/// * `target` - The target position
///
/// # Returns
/// * Ray from this intersection to the target, offset by an epsilon
pub fn offset_ray_to(point: Vector3, normal: Vector3, target: Vector3) -> Ray {
    debug_assert!(is_finite(&point));
    debug_assert!(is_finite(&normal));
    debug_assert!(is_normalized(&normal));
    debug_assert!(is_finite(&target));

    let dir = target - point;
    let origin = offset_point(point, normal, dir);
    let direction = target - origin;

    Ray::new(origin, direction.normalized(), 0.0, direction.mag())
}

/// Converts spherical coordinates to cartesian coordinates in the given frame wth following description:
/// * `frame.x_axis`: to your right
/// * `frame.y_axis`: to your top
/// * `frame.z_axis`: towards you
///
/// # Constraints
/// * `theta` - Should be within `[0, 2π]`.
/// * `phi` - Should be within `[0, π]`.
///
/// # Arguments
/// * `theta` - The angle between the `z`-axis and the spherical direction in the `zx` plane
/// * `phi` - The angle between the  `y`-axis and the spherical direction
/// * `frame` - The coordinate system/frame to use
///
/// # Returns
/// * The corresponding cartesian vector
pub fn spherical_to_cartesian_frame(theta: Float, phi: Float, frame: &CoordinateSystem) -> Vector3 {
    debug_assert!(theta.in_range_incl(0.0, TAU));
    debug_assert!(phi.in_range_incl(0.0, PI));

    let (sin_theta, cos_theta) = theta.sin_cos();
    let (sin_phi, cos_phi) = phi.sin_cos();

    spherical_to_cartesian_frame_trig(sin_theta, cos_theta, sin_phi, cos_phi, frame)
}

/// Converts spherical coordinates to cartesian coordinates in the given frame wth following description:
/// * `frame.x_axis`: to your right
/// * `frame.y_axis`: to your top
/// * `frame.z_axis`: towards you
///
/// To make below descriptions easier, we define the following:
/// * `theta` - The angle between the `z`-axis and the spherical direction in the `zx` plane.
/// * `phi` - The angle between the  `y`-axis and the spherical direction.
///
/// # Constraints
/// * `sin_theta` - Should be within `[-1, 1]`
/// * `cos_theta` - Should be within `[-1, 1]`
/// * `sin_phi` - Should be within `[-1, 1]`
/// * `cos_phi` - Should be within `[-1, 1]`
///
/// # Arguments
/// * `sin_theta` - The sine of `theta`
/// * `cos_theta` - The cosine of `theta`
/// * `sin_phi` - The sine of `phi`
/// * `cos_phi` - The cosine of `phi`
/// * `frame` - The coordinate system/frame to use
///
/// # Returns
/// * The corresponding cartesian vector
pub fn spherical_to_cartesian_frame_trig(
    sin_theta: Float,
    cos_theta: Float,
    sin_phi: Float,
    cos_phi: Float,
    frame: &CoordinateSystem,
) -> Vector3 {
    debug_assert!(sin_theta.in_range_incl(-1.0, 1.0));
    debug_assert!(cos_theta.in_range_incl(-1.0, 1.0));
    debug_assert!(sin_phi.in_range_incl(-1.0, 1.0));
    debug_assert!(cos_phi.in_range_incl(-1.0, 1.0));

    let x = frame.x_axis * sin_phi * sin_theta;
    let y = frame.y_axis * cos_phi;
    let z = frame.z_axis * sin_phi * cos_theta;

    x + y + z
}

#[inline]
pub fn spherical_to_cartesian1(sin_theta: Float, cos_theta: Float, phi: Float) -> Vector3 {
    Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
}

/// Converts spherical coordinates to cartesian coordinates in the following describe frame:
/// * x-axis: to your right
/// * y-axis: to your top
/// * z-axis: towards you
///
/// # Constraints
/// * `theta` - Should be within `[0, 2π]`.
/// * `phi` - Should be within `[0, π]`.
///
/// # Arguments
/// * `theta` - The angle between the `z`-axis and the spherical direction in the `zx` plane.
/// * `phi` - The angle between the  `y`-axis and the spherical direction .
///
/// # Returns
/// * The corresponding cartesian vector
pub fn spherical_to_cartesian(theta: Float, phi: Float) -> Vector3 {
    debug_assert!(theta.in_range_incl(0.0, TAU));
    debug_assert!(phi.in_range_incl(0.0, PI));

    let (sin_theta, cos_theta) = theta.sin_cos();
    let (sin_phi, cos_phi) = phi.sin_cos();

    spherical_to_cartesian_trig(sin_theta, cos_theta, sin_phi, cos_phi)
}

/// Converts spherical coordinates to cartesian coordinates in the following described frame:
/// * x-axis: to your right
/// * y-axis: to your top
/// * z-axis: towards you
///
/// To make below descriptions easier, we define the following:
/// * `theta` - The angle between the `z`-axis and the spherical direction in the `zx` plane.
/// * `phi` - The angle between the  `y`-axis and the spherical direction .
///
/// # Constraints
/// * `sin_theta` - Should be within `[-1, 1]`
/// * `cos_theta` - Should be within `[-1, 1]`
/// * `sin_phi` - Should be within `[-1, 1]`
/// * `cos_phi` - Should be within `[-1, 1]`
///
/// # Arguments
/// * `sin_theta` - The sine of `theta`
/// * `cos_theta` - The cosine of `theta`
/// * `sin_phi` - The sine of `phi`
/// * `cos_phi` - The cosine of `phi`
///
/// # Returns
/// * The corresponding cartesian vector
pub fn spherical_to_cartesian_trig(
    sin_theta: Float,
    cos_theta: Float,
    sin_phi: Float,
    cos_phi: Float,
) -> Vector3 {
    debug_assert!(sin_theta.in_range_incl(-1.0, 1.0));
    debug_assert!(cos_theta.in_range_incl(-1.0, 1.0));
    debug_assert!(sin_phi.in_range_incl(-1.0, 1.0));
    debug_assert!(cos_phi.in_range_incl(-1.0, 1.0));

    let x = sin_phi * sin_theta;
    let y = cos_phi;
    let z = sin_phi * cos_theta;

    Vector3::new(x, y, z)
}

/// An intersection consists of the following 4 properties:
/// * `point` - The intersection point
/// * `normal` - The surface normal (showing outside, even if intersection hits inside!)
/// * `t` - The ray parameter at which it intersects
/// * `ray` - The reference to the intersecting ray
#[derive(Clone)]
pub struct Intersection {
    pub point: Vector3,
    pub normal: Vector3,
    pub t: Float,
    pub ray: Ray,
}

impl Intersection {
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
    pub fn new(point: Vector3, normal: Vector3, t: Float, ray: Ray) -> Self {
        debug_assert!(ray.contains(t));
        debug_assert!(is_normalized(&normal));

        Self {
            point,
            normal,
            t,
            ray,
        }
    }
}

/// A coordinate system represents 3 (orthogonal) vectors in 3D space.
#[derive(Copy, Clone, Debug)]
pub struct CoordinateSystem {
    pub x_axis: Vector3,
    pub y_axis: Vector3,
    pub z_axis: Vector3,
}

impl CoordinateSystem {
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
    pub fn new(x: Vector3, y: Vector3, z: Vector3) -> Self {
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
    pub fn from_x(x_axis: Vector3) -> Self {
        debug_assert!(is_finite(&x_axis));
        debug_assert!(is_normalized(&x_axis));

        if x_axis == Vector3::unit_x() {
            Self::new(Vector3::unit_x(), Vector3::unit_y(), Vector3::unit_z())
        } else {
            let z = x_axis.cross(Vector3::unit_x()).normalized();
            let y = z.cross(x_axis).normalized();

            Self::new(x_axis, y, z)
        }
    }

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
    pub fn from_y(y_axis: Vector3) -> Self {
        debug_assert!(is_finite(&y_axis));
        debug_assert!(is_normalized(&y_axis));

        let s = if y_axis.x.abs() > y_axis.y.abs() {
            let l = (y_axis.x * y_axis.x + y_axis.z * y_axis.z).sqrt();
            Vector3::new(-y_axis.z / l, 0.0, y_axis.x / l)
        } else {
            let l = (y_axis.y * y_axis.y + y_axis.z * y_axis.z).sqrt();
            Vector3::new(0.0, y_axis.z / l, -y_axis.y / l)
        };
        Self::new(s, y_axis, y_axis.cross(s))

        // if y_axis == Vector3::unit_y() {
        //     Self::new(Vector3::unit_x(), Vector3::unit_y(), Vector3::unit_z())
        // } else {
        //     let x = y_axis.cross(Vector3::unit_y()).normalized();
        //     let z = x.cross(y_axis).normalized();
        //
        //     Self::new(x, y_axis, z)
        // }
    }

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
    pub fn from_z(z_axis: Vector3) -> Self {
        debug_assert!(is_finite(&z_axis));
        debug_assert!(is_normalized(&z_axis));

        if z_axis == Vector3::unit_z() {
            Self::new(Vector3::unit_x(), Vector3::unit_y(), Vector3::unit_z())
        } else {
            let x = z_axis.cross(Vector3::unit_z()).normalized();
            let y = z_axis.cross(x).normalized();

            Self::new(x, y, z_axis)
        }
    }
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self::new(Vector3::unit_x(), Vector3::unit_y(), Vector3::unit_z())
    }
}

/// A trait for objects that surround space and may contain points.
pub trait Container {
    /// Returns whether this object contains the given `piint` bounds-inclusive.
    ///
    /// # Arguments
    /// * `point` - A point in space
    ///
    /// # Returns
    /// * Whether the given point is inside-inclusive this object
    fn contains(&self, point: &Vector3) -> bool;
}

/// A trait for objects that can report a cube as their bounds.
pub trait Boundable {
    /// Returns the bounds of this object
    ///
    /// # Returns
    /// * The bounds
    fn bounds(&self) -> Aabb;
}

/// A trait for objects that can be intersected by rays.
pub trait Intersectable {
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
    /// use geometry::{Aabb, Intersectable, Ray, Vector3};
    /// use ultraviolet::Vec3;
    ///
    /// let cube = Aabb::default();
    /// let mut ray = Ray::new_fast(Vector3::zero(), Vector3::unit_x());
    ///
    /// let intersection = cube.intersect(&mut ray).unwrap(); // intersects surely
    ///
    /// let mut normal = intersection.normal;
    /// assert_eq!(Vector3::unit_x(), normal); // normal points to the outside
    ///
    /// // true geometric normal is inverted if both vectors point into the same general direction
    /// if normal.dot(ray.direction) > 0.0 {
    ///     normal = -normal;
    /// }
    ///
    /// assert_eq!(-Vector3::unit_x(), normal); // normal points to the outside
    /// ```
    ///
    /// # Returns
    /// * Intersection or `None`
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;

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

/// A super-trait to combine `Boundable` and `Intersectable`, therefore giving a valid geometry.
#[typetag::serde]
pub trait Geometry: Boundable + Intersectable + Send + Sync {}

impl Boundable for Box<dyn Geometry> {
    fn bounds(&self) -> Aabb {
        self.as_ref().bounds()
    }
}

impl Intersectable for Box<dyn Geometry> {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        self.as_ref().intersect(ray)
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.as_ref().intersects(ray)
    }
}

#[typetag::serde]
impl Geometry for Box<dyn Geometry> {}

pub trait ContainerGeometry: Container + Intersectable {
    fn contains_or_intersects(&self, ray: &Ray) -> bool {
        self.contains(&ray.origin) || self.intersects(ray)
    }
}

impl<T> ContainerGeometry for T where T: Container + Intersectable {}
