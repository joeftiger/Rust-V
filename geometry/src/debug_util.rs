#![allow(dead_code)]

use definitions::{Float, Vector3};
use utility::floats::FloatExt;

#[inline]
pub fn is_finite(v: &Vector3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[inline]
pub fn is_not_nan(v: &Vector3) -> bool {
    !(v.x.is_nan() && v.y.is_nan() && v.z.is_nan())
}

#[inline]
pub fn is_normalized(v: &Vector3) -> bool {
    v.mag()
        .in_range(1.0 - Float::big_epsilon(), 1.0 + Float::big_epsilon())
}
