use crate::filters::Filter;
use serde::{Deserialize, Serialize};
use ultraviolet::Vec2;

/// Sample weights fall off linearly from the center.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TriangleFilter {
    pub radius: Vec2,
    pub inv_radius: Vec2,
}

impl TriangleFilter {
    pub fn new(radius: Vec2) -> Self {
        Self {
            radius,
            inv_radius: Vec2::one() / radius,
        }
    }
}

impl Filter for TriangleFilter {
    #[inline(always)]
    fn evaluate(&self, point: &Vec2) -> f32 {
        f32::max(0.0, self.radius.x - point.x.abs()) * f32::max(0.0, self.radius.y - point.y.abs())
    }
}
