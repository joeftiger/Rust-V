use crate::filters::Filter;
use serde::{Deserialize, Serialize};
use ultraviolet::Vec2;
use utility::floats::fast_max;

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

#[typetag::serde]
impl Filter for TriangleFilter {
    fn radius(&self) -> Vec2 {
        self.radius
    }

    fn inv_radius(&self) -> Vec2 {
        self.inv_radius
    }

    #[inline(always)]
    fn evaluate(&self, point: &Vec2) -> f32 {
        fast_max(0.0, self.radius.x - point.x.abs()) * fast_max(0.0, self.radius.y - point.y.abs())
    }
}
