use crate::debug_util::{is_finite, is_normalized};
use definitions::{Float, Vector3};
use utility::floats::FloatExt;

/// A ray consists of of an origin and a direction.
/// Additionally, a ray contains information about the `start` and `end` to contain a range along
/// the ray's direction.
#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
    pub t_start: Float,
    pub t_end: Float,
}

impl Ray {
    /// Creates a new ray.
    ///
    /// # Constraints
    /// * `origin` - All values should be finite (neither infinite nor `NaN`).
    /// * `direction` - All values should be finite.
    ///                 Should be normalized.
    /// * `t_start` - Should be less than `t_end`.
    ///
    /// # Arguments
    /// * `origin` - The origin
    /// * `direction` - The direction
    /// * `t_start` - The start-inclusive parameter for the `direction`
    /// * `t_end` - The end-inclusive parameter for the `direction`
    ///
    /// # Returns
    /// * Self
    pub fn new(origin: Vector3, direction: Vector3, t_start: Float, t_end: Float) -> Self {
        debug_assert!(is_finite(&origin));
        debug_assert!(is_finite(&direction));
        debug_assert!(is_normalized(&direction));
        debug_assert!(t_start < t_end);

        Self {
            origin,
            direction,
            t_start,
            t_end,
        }
    }

    /// Creates a new ray with the `direction` constraints being from `0` to `infinity`.
    ///
    /// # Constraints
    /// * `origin` - All values should be finite (neither infinite nor `NaN`).
    /// * `direction` - All values should be finite (neither infinite nor `NaN`).
    ///                 Should be normalized.
    ///
    /// # Arguments
    /// * `origin` - The origin
    /// * `direction` - The direction
    ///
    /// # Returns
    /// * Self
    pub fn new_fast(origin: Vector3, direction: Vector3) -> Self {
        debug_assert!(is_finite(&origin));
        debug_assert!(is_finite(&direction));
        debug_assert!(is_normalized(&direction));

        Self::new(origin, direction, 0.0, Float::INFINITY)
    }

    /// Constructs a ray from the given `origin` to the given `target`, effectively limiting
    /// the `direction` parameters being inside-inclusive both coordinates.
    ///
    /// # Constraints
    /// * `origin` - All values should be finite (neither infinite nor `NaN`).
    /// * `target` - All values should be finite.
    ///
    /// # Arguments
    /// * `origin` - The origin
    /// * `target` - The target
    ///
    /// # Returns
    /// * Self
    pub fn between(origin: Vector3, target: Vector3) -> Self {
        debug_assert!(is_finite(&origin));
        debug_assert!(is_finite(&target));

        let dir = target - origin;

        Self::new(origin, dir.normalized(), 0.0, dir.mag())
    }

    /// Returns whether the given `t` parameter is inside-inclusive the ray constraints.
    ///
    /// # Constraints
    /// * `t` - Should **not** be `NaN`.
    ///
    /// # Arguments
    /// * `t` - The ray parameter
    ///
    /// # Returns
    /// * Whether the ray constraints contain the given `t`.
    #[inline(always)]
    pub fn contains(&self, t: Float) -> bool {
        debug_assert!(!t.is_nan());

        t.in_range_incl(self.t_start, self.t_end)
    }

    /// Calculates the position in space after applying the given ray parameter.
    ///
    /// # Constraints
    /// * `t` - Should **not** be `NaN`.
    ///
    /// # Arguments
    /// * `t` - The ray parameter
    ///
    /// # Returns
    /// * The resulting position in space
    pub fn at(&self, t: Float) -> Vector3 {
        debug_assert!(!t.is_nan());

        self.origin + self.direction * t
    }
}
