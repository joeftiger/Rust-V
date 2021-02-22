use crate::filters::Filter;
use ultraviolet::Vec2;

pub struct TriangleFilter {
    radius: Vec2,
}

impl TriangleFilter {
    pub fn new(radius: Vec2) -> Self {
        Self { radius }
    }
}

impl Filter for TriangleFilter {
    fn evaluate(&self, point: &Vec2) -> f32 {
        f32::max(0.0, self.radius.x - point.x.abs()) * f32::max(0.0, self.radius.y - point.y.abs())
    }
}
