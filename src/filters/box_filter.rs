use crate::filters::Filter;
use definitions::{Float, Vector2};
use serde::{Deserialize, Serialize};

/// Sample weights considered equally.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BoxFilter {
    pub radius: Vector2,
    pub inv_radius: Vector2,
}

impl BoxFilter {
    pub fn new(radius: Vector2) -> Self {
        Self {
            radius,
            inv_radius: Vector2::one() / radius,
        }
    }
}

#[typetag::serde]
impl Filter for BoxFilter {
    fn radius(&self) -> Vector2 {
        self.radius
    }

    fn inv_radius(&self) -> Vector2 {
        self.inv_radius
    }

    #[inline(always)]
    fn evaluate(&self, _: &Vector2) -> Float {
        1.0
    }
}
