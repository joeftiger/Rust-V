use crate::Float;
use std::cmp::Ordering;

// WolframAlpha
#[allow(clippy::excessive_precision)]
pub const SQRT_PI: Float = 1.7724538509055160272981674833411451827975494561223871282138077898;
#[allow(clippy::excessive_precision)]
pub const FRAC_1_SQRT_PI: Float =
    0.5641895835477562869480794515607725858440506293289988568440857217;
#[allow(clippy::excessive_precision)]
pub const PI_2: Float = 6.2831853071795864769252867665590057683943387987502116419498891846;

pub trait FloatExt {
    fn epsilon() -> Self;
    fn big_epsilon() -> Self;
    fn in_range(&self, min: Self, max: Self) -> bool;
    fn in_range_incl(&self, min: Self, max: Self) -> bool;
    fn in_range_incl_left(&self, min: Self, max: Self) -> bool;
    fn in_range_incl_right(&self, min: Self, max: Self) -> bool;
    fn fast_min(self, other: Self) -> Self;
    fn fast_max(self, other: Self) -> Self;
    fn fast_clamp(self, min: Self, max: Self) -> Self;
    fn is_approx_zero(&self) -> bool;
    fn is_approx_eq(&self, other: Self) -> bool;
    fn is_approx_eq_with(&self, other: Self, tolerance: Self) -> bool;
    fn fast_cmp(self, other: Self) -> Ordering;
    fn lerp(self, to: Self, t: Self) -> Self;
    fn inv_lerp(self, to: Self, res: Self) -> Self;
    fn lerp_map(a_from: Self, a_to: Self, b_from: Self, b_to: Self, res: Self) -> Self;
}

macro_rules! impl_float_ext {
    ($t:ident, $eps:expr, $big_eps:expr) => {
        impl FloatExt for $t {
            #[inline(always)]
            fn epsilon() -> Self {
                $eps
            }

            #[inline(always)]
            fn big_epsilon() -> Self {
                $big_eps
            }

            #[inline]
            fn in_range(&self, min: Self, max: Self) -> bool {
                min < *self && *self < max
            }

            #[inline]
            fn in_range_incl(&self, min: Self, max: Self) -> bool {
                min <= *self && *self <= max
            }

            #[inline]
            fn in_range_incl_left(&self, min: Self, max: Self) -> bool {
                min <= *self && *self < max
            }

            #[inline]
            fn in_range_incl_right(&self, min: Self, max: Self) -> bool {
                min < *self && *self <= max
            }

            #[inline]
            fn fast_min(self, other: Self) -> Self {
                if self <= other {
                    self
                } else {
                    other
                }
            }

            #[inline]
            fn fast_max(self, other: Self) -> Self {
                if self >= other {
                    self
                } else {
                    other
                }
            }

            #[inline]
            fn fast_clamp(self, min: Self, max: Self) -> Self {
                debug_assert!(min <= max);

                self.fast_min(max).fast_max(min)
            }

            #[inline]
            fn is_approx_zero(&self) -> bool {
                self.abs() <= $eps
            }

            #[inline]
            fn is_approx_eq(&self, other: Self) -> bool {
                self.is_approx_eq_with(other, $eps)
            }

            #[inline]
            fn is_approx_eq_with(&self, other: Self, tolerance: Self) -> bool {
                let distance = (other - *self).abs();
                if distance <= tolerance {
                    true
                } else {
                    let largest = self.fast_max(other).fast_max(1.0);
                    distance <= largest * $eps
                }
            }

            #[inline]
            fn fast_cmp(self, other: Self) -> Ordering {
                if self < other {
                    Ordering::Less
                } else if self > other {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }

            #[inline]
            fn lerp(self, to: Self, t: Self) -> Self {
                (1.0 - t) * self + to * t
            }

            #[inline]
            fn inv_lerp(self, to: Self, res: Self) -> Self {
                (res - self) / (to - self)
            }

            #[inline]
            fn lerp_map(a_from: Self, a_to: Self, b_from: Self, b_to: Self, res: Self) -> Self {
                let t = a_from.inv_lerp(a_to, res);
                b_from.lerp(b_to, t)
            }
        }
    };
}

impl_float_ext!(f32, 1e-6, 1e-4);
impl_float_ext!(f64, 1e-8, 1e-6);
