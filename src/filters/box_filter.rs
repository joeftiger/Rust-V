use crate::filters::Filter;
use ultraviolet::Vec2;

pub struct BoxFilter;

impl Filter for BoxFilter {
    fn evaluate(&self, _: &Vec2) -> f32 {
        1.0
    }
}
