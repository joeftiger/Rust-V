use crate::debug_utils::within_01;
use geometry::{from_spherical_direction, CoordinateSystem};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};
use ultraviolet::{Lerp, Vec2, Vec3};

/// # Summary
/// Samples a concentric mapped point from the given random sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample in `[0, 1]`
///
/// # Results
/// * `Vec2` - A concentric sample
pub fn concentric_sample_disk(sample: &Vec2) -> Vec2 {
    debug_assert!(within_01(sample));

    // Map uniform random numbers to [-1,1]^2
    let offset = 2.0 * *sample - Vec2::one();

    // Handle degeneracy at the origin
    if offset.x == 0.0 || offset.y == 0.0 {
        return Vec2::zero();
    }

    // Apply concentric mapping to point
    let r;
    let theta;
    if offset.x.abs() > offset.y.abs() {
        r = offset.x;
        theta = FRAC_PI_4 * offset.y / offset.x;
    } else {
        r = offset.y;
        theta = FRAC_PI_2 - FRAC_PI_4 * offset.x / offset.y;
    }

    r * Vec2::new(theta.cos(), theta.sin())
}

/// # Summary
/// Samples a hemisphere with a cosine distribution described by the sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample in `[0, 1]`
///
/// # Results
/// * `Vec3` - A point on the hemisphere around `(0, 0, 1)`
pub fn cos_sample_hemisphere(sample: &Vec2) -> Vec3 {
    debug_assert!(within_01(sample));

    let d = concentric_sample_disk(sample);
    let z = f32::max(0.0, 1.0 - d.x * d.x - d.y * d.y).sqrt();

    Vec3::new(d.x, d.y, z)
}

/// # Summary
/// Samples a sphere with a uniform distribution described by the sample.
///
/// # Constraints
/// * `sample` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `sample` - A random sample in `[0, 1]`
///
/// # Results
/// * `Vec3` - A point on the sphere around `(0, 0, 0)`
pub fn uniform_sample_sphere(sample: &Vec2) -> Vec3 {
    debug_assert!(within_01(sample));

    let z = 1.0 - 2.0 * sample.x;

    let r = f32::max(0.0, 1.0 - z * z).sqrt();
    let (sin_phi, cos_phi) = f32::sin_cos(sample.y * TAU);

    let x = r * cos_phi;
    let y = r * sin_phi;

    Vec3::new(x, y, z)
}

/// # Summary
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
// TODO: Make more efficient for this edge case!
pub fn uniform_sample_cone(sample: &Vec2, cos_theta_max: f32) -> Vec3 {
    debug_assert!(within_01(sample));

    let frame = CoordinateSystem::default();
    uniform_sample_cone_frame(sample, cos_theta_max, &frame)
}

/// # Summary
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
    let phi = sample.y * TAU;

    from_spherical_direction(sin_theta, cos_theta, phi, frame)
}

/// # Summary
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
