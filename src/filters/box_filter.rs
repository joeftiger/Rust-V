use crate::filters::Filter;
use serde::{Deserialize, Serialize};
use ultraviolet::Vec2;

/// Sample weights considered equally.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BoxFilter {
    pub radius: Vec2,
    pub inv_radius: Vec2,
}

impl BoxFilter {
    pub fn new(radius: Vec2) -> Self {
        Self {
            radius,
            inv_radius: Vec2::one() / radius,
        }
    }
}

impl Filter for BoxFilter {
    #[inline(always)]
    fn evaluate(&self, _: &Vec2) -> f32 {
        1.0
    }
}
