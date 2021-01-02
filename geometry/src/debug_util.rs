#![cfg(debug_assertions)]
#![allow(dead_code)]

use ultraviolet::Vec3;
use utility::floats::BIG_EPSILON;

#[inline(always)]
pub fn in_range(f: f32, min: f32, max: f32) -> bool {
    min < f && f < max
}

#[inline(always)]
pub fn in_range_incl(f: f32, min: f32, max: f32) -> bool {
    min <= f && f <= max
}

#[inline(always)]
pub fn is_finite(v: &Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[inline(always)]
pub fn is_not_nan(v: &Vec3) -> bool {
    !(v.x.is_nan() && v.y.is_nan() && v.z.is_nan())
}

#[inline(always)]
pub fn is_normalized(v: &Vec3) -> bool {
    in_range(v.mag(), 1.0 - BIG_EPSILON, 1.0 + BIG_EPSILON)
}
