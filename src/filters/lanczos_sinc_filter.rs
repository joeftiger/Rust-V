use crate::filters::Filter;
use serde::{Deserialize, Serialize};
use ultraviolet::Vec2;
use utility::math::sinc;

/// Sample weights considered with a combination of `sinc()` calls.
///
/// Suffers a bit from ringing, but less blurring.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LanczosSincFilter {
    pub radius: Vec2,
    pub inv_radius: Vec2,
    tau: f32,
}

impl LanczosSincFilter {
    pub fn new(radius: Vec2, tau: f32) -> Self {
        Self {
            radius,
            inv_radius: Vec2::one() / radius,
            tau,
        }
    }

    #[inline]
    fn windowed_sinc(&self, mut x: f32, radius: f32) -> f32 {
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
    fn radius(&self) -> Vec2 {
        self.radius
    }

    fn inv_radius(&self) -> Vec2 {
        self.inv_radius
    }

    fn evaluate(&self, point: &Vec2) -> f32 {
        self.windowed_sinc(point.x, self.radius.x) * self.windowed_sinc(point.y, self.radius.y)
    }
}
