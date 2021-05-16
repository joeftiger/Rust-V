use crate::debug_utils::within_01;
use definitions::{Float, Vector2, Vector3};
use geometry::{spherical_to_cartesian_frame_trig, spherical_to_cartesian_trig, CoordinateSystem};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, TAU};
use utility::floats::FloatExt;

#[inline]
pub fn sample_vector_from_angle(
    direction: Vector3,
    sin_theta_max: Float,
    sample: Vector2,
) -> Vector3 {
    debug_assert!(within_01(sample));

    let frame = CoordinateSystem::from_y(direction);

    let sample = sample_unit_disk_concentric(sample) * sin_theta_max;

    let right = frame.x_axis * sample.x;
    let forward = frame.z_axis * sample.y;

    (direction + right + forward).normalized()
}

/// Samples a non-concentric mapped point from the given random sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample
///
/// # Results
/// * A non-concentric sample on the unit disk
#[inline]
pub fn sample_unit_disk(sample: Vector2) -> Vector2 {
    debug_assert!(within_01(sample));

    let theta = sample.x * TAU as Float;
    let (sin, cos) = theta.sin_cos();

    sample.y * Vector2::new(cos, sin)
}

/// Samples a concentric mapped point from the given random sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample
///
/// # Results
/// * A concentric sample on the unit disk
#[inline]
pub fn sample_unit_disk_concentric(sample: Vector2) -> Vector2 {
    debug_assert!(within_01(sample));

    // Map uniform random numbers to [-1,1]^2
    let offset = 2.0 * sample - Vector2::one();

    // Handle degeneracy at the origin
    if offset.x == 0.0 || offset.y == 0.0 {
        return Vector2::zero();
    }

    // Apply concentric mapping to point
    let (r, theta) = if (offset.x * offset.x) > (offset.y * offset.y) {
        (offset.x, FRAC_PI_4 as Float * offset.y / offset.x)
    } else {
        (
            offset.y,
            (FRAC_PI_4 as Float).mul_add(-offset.x / offset.y, FRAC_PI_2 as Float),
        )
    };

    let (sin, cos) = theta.sin_cos();

    Vector2::new(r * cos, r * sin)
}

/// Samples a hemisphere with a cosine distribution described by the sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample
///
/// # Results
/// * A point on the unit hemisphere around the `(0, 1, 0)` axis
#[inline]
pub fn sample_unit_hemisphere(sample: Vector2) -> Vector3 {
    debug_assert!(within_01(sample));

    let d = sample_unit_disk_concentric(sample);
    let b = d.y.mul_add(-d.y, 1.0);
    let right = d.x.mul_add(-d.x, b);

    // let y = fast_max(0.0, 1.0 - d.x * d.x - d.y * d.y).sqrt();
    let y = right.fast_max(0.0).sqrt();

    Vector3::new(d.x, y, d.y)
}

/// Samples a sphere with a uniform distribution described by the sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample
///
/// # Results
/// * A point on the unit sphere around `(0, 0, 0)`
#[inline]
pub fn sample_unit_sphere(sample: Vector2) -> Vector3 {
    debug_assert!(within_01(sample));

    let z = sample.x.mul_add(-2.0, 1.0);

    let r = z.mul_add(-z, 1.0).fast_max(0.0).sqrt();
    let (sin_phi, cos_phi) = Float::sin_cos(sample.y * TAU as Float);

    let x = r * cos_phi;
    let y = r * sin_phi;

    Vector3::new(x, y, z)
}

/// Samples a cone around the `(0, 1, 0)` axis with a uniform distribution described by the sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample in `[0, 1]`
/// * `cos_theta_max` - The max angle
///
/// # Results
/// * `Vector3` - A direction in the cone around `(0, 1, 0)`
#[inline]
pub fn sample_cone(sample: Vector2, cos_theta_max: Float) -> Vector3 {
    debug_assert!(within_01(sample));

    let cos_theta = cos_theta_max.lerp(1.0, sample.x);
    let sin_theta = cos_theta.mul_add(-cos_theta, 1.0).sqrt();
    let (sin_phi, cos_phi) = Float::sin_cos(sample.y * TAU as Float);

    spherical_to_cartesian_trig(sin_theta, cos_theta, sin_phi, cos_phi)
}

/// Samples a cone around the `frame.e2` axis with a uniform distribution described by the sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample in `[0, 1]`
/// * `cos_theta_max` - The max angle
/// * `frame` - The coordinate system frame. Y-axis is "up"-axis.
///
/// # Results
// * `Vector3` - A direction in the cone around `frame.e2`
#[inline]
pub fn uniform_sample_cone_frame(
    sample: Vector2,
    cos_theta_max: Float,
    frame: &CoordinateSystem,
) -> Vector3 {
    debug_assert!(within_01(sample));

    let cos_theta = cos_theta_max.lerp(1.0, sample.x);
    let sin_theta = cos_theta.mul_add(-cos_theta, 1.0).sqrt();
    let (sin_phi, cos_phi) = Float::sin_cos(sample.y * TAU as Float);

    spherical_to_cartesian_frame_trig(sin_theta, cos_theta, sin_phi, cos_phi, frame)
}

/// Computes the pdf for uniformly sampling a code.
///
/// # Arguments
/// * `cos_theta` - The cone angle
///
/// # Results
/// * `Float` - The pdf
#[inline]
pub fn uniform_cone_pdf(cos_theta: Float) -> Float {
    1.0 / (TAU as Float * (1.0 - cos_theta))
}
