use crate::filters::Filter;
use crate::*;

use serde::{Deserialize, Serialize};
use utility::floats::FloatExt;

/// Sample weights fall off linearly from the center.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TriangleFilter {
    pub radius: Vector2,
}

impl TriangleFilter {
    pub fn new(radius: Vector2) -> Self {
        Self { radius }
    }
}

#[typetag::serde]
impl Filter for TriangleFilter {
    #[inline]
    fn evaluate(&self, point: Vector2) -> Float {
        Float::fast_max(0.0, self.radius.x - point.x.abs())
            * Float::fast_max(0.0, self.radius.y - point.y.abs())
    }
}
