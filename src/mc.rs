use crate::debug_utils::within_01;
use geometry::{spherical_to_cartesian_frame_trig, spherical_to_cartesian_trig, CoordinateSystem};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};
use ultraviolet::{Lerp, Vec2, Vec3};

pub fn sample_vector_from_angle(direction: Vec3, sin_theta_max: f32, sample: &Vec2) -> Vec3 {
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
pub fn sample_unit_disk(sample: &Vec2) -> Vec2 {
    debug_assert!(within_01(sample));

    let theta = sample.x * TAU;
    let (sin, cos) = theta.sin_cos();

    sample.y * Vec2::new(cos, sin)
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
pub fn sample_unit_disk_concentric(sample: &Vec2) -> Vec2 {
    debug_assert!(within_01(sample));

    // Map uniform random numbers to [-1,1]^2
    let offset = 2.0 * *sample - Vec2::one();

    // Handle degeneracy at the origin
    if offset.x == 0.0 || offset.y == 0.0 {
        return Vec2::zero();
    }

    // Apply concentric mapping to point
    let (r, theta) = if (offset.x * offset.x) > (offset.y * offset.y) {
        (offset.x, FRAC_PI_4 * offset.y / offset.x)
    } else {
        (offset.y, FRAC_PI_2 - FRAC_PI_4 * offset.x / offset.y)
    };

    let (sin, cos) = theta.sin_cos();

    Vec2::new(r * cos, r * sin)
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
pub fn sample_unit_hemisphere(sample: &Vec2) -> Vec3 {
    debug_assert!(within_01(sample));

    let d = sample_unit_disk_concentric(sample);
    let y = f32::max(0.0, 1.0 - d.x * d.x - d.y * d.y).sqrt();

    Vec3::new(d.x, y, d.y)
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
pub fn sample_unit_sphere(sample: &Vec2) -> Vec3 {
    debug_assert!(within_01(sample));

    let z = 1.0 - 2.0 * sample.x;

    let r = f32::max(0.0, 1.0 - z * z).sqrt();
    let (sin_phi, cos_phi) = f32::sin_cos(sample.y * TAU);

    let x = r * cos_phi;
    let y = r * sin_phi;

    Vec3::new(x, y, z)
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
/// * `Vec3` - A direction in the cone around `(0, 1, 0)`
pub fn sample_cone(sample: &Vec2, cos_theta_max: f32) -> Vec3 {
    debug_assert!(within_01(sample));

    let cos_theta = cos_theta_max.lerp(1.0, sample.x);
    let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);
    let (sin_phi, cos_phi) = f32::sin_cos(sample.y * TAU);

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
/// * `Vec3` - A direction in the cone around `frame.e2`
pub fn uniform_sample_cone_frame(
    sample: &Vec2,
    cos_theta_max: f32,
    frame: &CoordinateSystem,
) -> Vec3 {
    debug_assert!(within_01(sample));

    let cos_theta = cos_theta_max.lerp(1.0, sample.x);
    let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);
    let (sin_phi, cos_phi) = f32::sin_cos(sample.y * TAU);

    spherical_to_cartesian_frame_trig(sin_theta, cos_theta, sin_phi, cos_phi, frame)
}

/// Computes the pdf for uniformly sampling a code.
///
/// # Arguments
/// * `cos_theta` - The cone angle
///
/// # Results
/// * `f32` - The pdf
pub fn uniform_cone_pdf(cos_theta: f32) -> f32 {
    1.0 / (TAU * (1.0 - cos_theta))
}
