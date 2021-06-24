use crate::filters::Filter;
use crate::*;

use serde::{Deserialize, Serialize};

/// Sample weights considered equally.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BoxFilter {
    pub radius: Vector2,
}

impl BoxFilter {
    pub fn new(radius: Vector2) -> Self {
        Self { radius }
    }
}

#[typetag::serde]
impl Filter for BoxFilter {
    #[inline]
    fn evaluate(&self, point: Vector2) -> Float {
        let diff = self.radius - point.abs();
        if diff.x < 0.0 || diff.y < 0.0 {
            0.0
        } else {
            1.0
        }
    }
}
