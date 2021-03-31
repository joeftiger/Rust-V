use crate::filters::Filter;
use serde::{Deserialize, Serialize};
use ultraviolet::Vec2;

/// Sample weights considered with a Gaussian bump.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GaussianFilter {
    pub radius: Vec2,
    pub inv_radius: Vec2,
    alpha: f32,
    exp: Vec2,
}

impl GaussianFilter {
    pub fn new(radius: Vec2, alpha: f32) -> Self {
        let exp = -alpha * radius * radius;

        Self {
            radius,
            inv_radius: Vec2::one() / radius,
            alpha,
            exp,
        }
    }

    #[inline]
    fn gaussian(&self, point: f32, exp: f32) -> f32 {
        f32::max(0.0, f32::exp(-self.alpha * point * point) - exp)
    }
}

#[typetag::serde]
impl Filter for GaussianFilter {
    fn radius(&self) -> Vec2 {
        self.radius
    }
    fn inv_radius(&self) -> Vec2 {
        self.inv_radius
    }

    fn evaluate(&self, point: &Vec2) -> f32 {
        self.gaussian(point.x, self.exp.x) * self.gaussian(point.y, self.exp.y)
    }
}
