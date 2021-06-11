use crate::floats::FloatExt;
use crate::*;
use std::f64::consts::PI;

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
#[inline]
#[must_use]
pub fn solve_quadratic(a: Float, b: Float, c: Float) -> Option<(Float, Float)> {
    debug_assert!(a.is_finite());
    debug_assert!(b.is_finite());
    debug_assert!(c.is_finite());

    if a < Float::epsilon() {
        if b < Float::epsilon() {
            return None;
        }

        let sol = -c / b;

        return Some((sol, sol));
    }

    let discriminant = b.mul_add(b, -4.0 * a * c);
    if discriminant < 0.0 {
        return None;
    }

    let a_x1 = -0.5 * (b + Float::copysign(discriminant.sqrt(), b));

    let x0 = a_x1 / a;
    let x1 = c / a_x1;

    if x0 < x1 {
        Some((x0, x1))
    } else {
        Some((x1, x0))
    }
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
pub fn sinc(mut x: Float) -> Float {
    debug_assert!(x.is_finite());

    x = x.abs();

    if x < Float::epsilon() {
        1.0
    } else {
        let pix = PI as Float * x;
        Float::sin(pix) / pix
    }
}

#[allow(clippy::excessive_precision)]
#[inline]
#[must_use]
pub fn erf(x: Float) -> Float {
    // constants
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    // Save the sign of x
    let sign = x.signum();
    let x = x.abs();

    // A&S formula 7.1.26
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * Float::exp(-x * x);

    sign * y
}

#[allow(clippy::excessive_precision)]
#[inline]
#[must_use]
pub fn erf_inv(x: Float) -> Float {
    let x = Float::fast_clamp(x, -1.0 + Float::big_epsilon(), 1.0 - Float::big_epsilon());

    let mut w = -Float::ln((1.0 - x) * (1.0 + x));
    let mut p: Float;
    if w < 5.0 {
        w -= 2.5;
        p = 2.81022636e-08;
        p = 3.43273939e-07 + p * w;
        p = -3.5233877e-06 + p * w;
        p = -4.39150654e-06 + p * w;
        p = 0.00021858087 + p * w;
        p = -0.00125372503 + p * w;
        p = -0.00417768164 + p * w;
        p = 0.246640727 + p * w;
        p = 1.50140941 + p * w;
    } else {
        w = w.sqrt() - 3.0;
        p = -0.000200214257;
        p = 0.000100950558 + p * w;
        p = 0.00134934322 + p * w;
        p = -0.00367342844 + p * w;
        p = 0.00573950773 + p * w;
        p = -0.0076224613 + p * w;
        p = 0.00943887047 + p * w;
        p = 1.00167406 + p * w;
        p = 2.83297682 + p * w;
    }

    p * x
}
