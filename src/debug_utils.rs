#![allow(dead_code)]

use ultraviolet::{Vec2, Vec3};
use utility::floats::BIG_EPSILON;

pub use utility::floats::{in_range, in_range_incl, in_range_incl_left};

#[inline]
pub fn is_finite(v: &Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[inline]
pub fn is_not_nan(v: &Vec3) -> bool {
    !(v.x.is_nan() && v.y.is_nan() && v.z.is_nan())
}

#[inline]
pub fn is_normalized(v: &Vec3) -> bool {
    in_range(v.mag(), 1.0 - BIG_EPSILON, 1.0 + BIG_EPSILON)
}

#[inline]
pub fn within_01(v: &Vec2) -> bool {
    in_range_incl_left(v.x, 0.0, 1.0) && in_range_incl_left(v.y, 0.0, 1.0)
}
