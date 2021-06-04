use crate::filters::Filter;
use crate::*;

use serde::{Deserialize, Serialize};
use utility::floats::FloatExt;

/// Sample weights fall off linearly from the center.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TriangleFilter {
    pub radius: Vector2,
    pub inv_radius: Vector2,
}

impl TriangleFilter {
    pub fn new(radius: Vector2) -> Self {
        Self {
            radius,
            inv_radius: Vector2::one() / radius,
        }
    }
}

#[typetag::serde]
impl Filter for TriangleFilter {
    fn radius(&self) -> Vector2 {
        self.radius
    }

    fn inv_radius(&self) -> Vector2 {
        self.inv_radius
    }

    #[inline(always)]
    fn evaluate(&self, point: Vector2) -> Float {
        Float::fast_max(0.0, self.radius.x - point.x.abs())
            * Float::fast_max(0.0, self.radius.y - point.y.abs())
    }
}
