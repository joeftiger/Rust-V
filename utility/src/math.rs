use crate::floats::{BIG_EPSILON, EPSILON};
use std::f32::consts::PI;

/// Solves a quadratic equation, handling generics.
///
/// `a`x^2 + `b`x + `c`
///
/// # Constraints
/// * `a` - Should be finite (neither infinite nor `NaN`).
/// * `b` - Should be finite.
/// * `c` - Should be finite.
///
/// # Arguments
/// * `a` - The parameter for `x^2`
/// * `b` - The parameter for `x`
/// * `c` - The constant parameter
///
/// # Returns
/// * `Option<(f32, f32)>` - The solutions in ascending order (if any)
#[inline(always)]
#[must_use]
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    debug_assert!(a.is_finite());
    debug_assert!(b.is_finite());
    debug_assert!(c.is_finite());

    if a < BIG_EPSILON {
        if b < BIG_EPSILON {
            return None;
        }

        let sol = -c / b;

        return Some((sol, sol));
    }

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let a_x1 = -0.5 * (b + f32::copysign(discriminant.sqrt(), b));

    let x0 = a_x1 / a;
    let x1 = c / a_x1;

    if x0 < x1 {
        Some((x0, x1))
    } else {
        Some((x1, x0))
    }
}

#[inline(always)]
pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    (1.0 - t) * from + to * t
}

#[inline(always)]
pub fn inv_lerp(from: f32, to: f32, res: f32) -> f32 {
    (res - from) / (to - from)
}

#[inline(always)]
pub fn lerp_map(from_lerp: (f32, f32), to_lerp: (f32, f32), res: f32) -> f32 {
    let t = inv_lerp(from_lerp.0, from_lerp.1, res);
    lerp(to_lerp.0, to_lerp.1, t)
}

/// Computes the `sinc()` function.
///
/// # Constraints
/// * `x` - Should be finite (neither infinite nor `NaN`).
///
/// # Arguments
/// * `x` - The value to apply sinc() to
///
/// # Returns
/// * The sinc of `x`
#[inline(always)]
pub fn sinc(mut x: f32) -> f32 {
    debug_assert!(x.is_finite());

    x = x.abs();

    if x < EPSILON {
        1.0
    } else {
        let pix = PI * x;
        f32::sin(pix) / pix
    }
}
