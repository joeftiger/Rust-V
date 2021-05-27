use crate::filters::Filter;
use crate::*;

use serde::{Deserialize, Serialize};
use utility::math::sinc;

/// Sample weights considered with a combination of `sinc()` calls.
///
/// Suffers a bit from ringing, but less blurring.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LanczosSincFilter {
    pub radius: Vector2,
    pub inv_radius: Vector2,
    tau: Float,
}

impl LanczosSincFilter {
    pub fn new(radius: Vector2, tau: Float) -> Self {
        Self {
            radius,
            inv_radius: Vector2::one() / radius,
            tau,
        }
    }

    #[inline]
    fn windowed_sinc(&self, mut x: Float, radius: Float) -> Float {
        x = x.abs();

        if x > radius {
            0.0
        } else {
            let lanczos = sinc(x / self.tau);

            sinc(x) * lanczos
        }
    }
}

#[typetag::serde]
impl Filter for LanczosSincFilter {
    fn radius(&self) -> Vector2 {
        self.radius
    }

    fn inv_radius(&self) -> Vector2 {
        self.inv_radius
    }

    fn evaluate(&self, point: Vector2) -> Float {
        self.windowed_sinc(point.x, self.radius.x) * self.windowed_sinc(point.y, self.radius.y)
    }
}
