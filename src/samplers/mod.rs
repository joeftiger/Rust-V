use crate::debug_utils::within_01;

use crate::*;
use serde::{Deserialize, Serialize};
use utility::floats::FloatExt;

pub mod camera;
pub mod spectral_samplers;

#[cfg(not(feature = "f64"))]
use fastrand::f32 as rand;
#[cfg(feature = "f64")]
use fastrand::f64 as rand;

/// A sample consists of 3 random values packed together into a `Float` and a `Vector2`.
#[derive(Debug, Copy, Clone)]
pub struct Sample {
    pub one_d: Float,
    pub two_d: Vector2,
}

impl Sample {
    pub fn new(one_d: Float, two_d: Vector2) -> Self {
        debug_assert!(one_d.in_range_incl_left(0.0, 1.0));
        debug_assert!(within_01(two_d));

        Self { one_d, two_d }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Sampler {
    NoOp,
    Random,
}

impl Sampler {
    /// Generates a new random value inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random value
    #[inline]
    pub fn get_1d(&self) -> Float {
        match self {
            Sampler::NoOp => 0.5,
            Sampler::Random => rand(),
        }
    }

    /// Generates a new random `Vector2` inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random `Vector2`
    #[inline]
    pub fn get_2d(&self) -> Vector2 {
        Vector2::new(self.get_1d(), self.get_1d())
    }

    /// Generates a new random `Vector3` inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random `Vector3`
    #[inline]
    pub fn get_3d(&self) -> Vector3 {
        Vector3::new(self.get_1d(), self.get_1d(), self.get_1d())
    }

    /// Generates a new random sample inside `[0, 1)`.
    ///
    /// # Returns
    /// * A random sample
    #[inline]
    pub fn get_sample(&self) -> Sample {
        Sample::new(self.get_1d(), self.get_2d())
    }
}
