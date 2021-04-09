#![allow(dead_code)]

use utility::floats::FloatExt;

use definitions::{Float, Vector2, Vector3};

#[inline(always)]
pub fn is_finite(v: &Vector3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[inline(always)]
pub fn is_not_nan(v: &Vector3) -> bool {
    !(v.x.is_nan() && v.y.is_nan() && v.z.is_nan())
}

#[inline(always)]
pub fn is_normalized(v: &Vector3) -> bool {
    v.mag()
        .in_range(1.0 - Float::big_epsilon(), 1.0 + Float::big_epsilon())
}

#[inline(always)]
pub fn within_01(v: &Vector2) -> bool {
    v.x.in_range_incl_left(0.0, 1.0) && v.y.in_range_incl_left(0.0, 1.0)
}
