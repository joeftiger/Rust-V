use crate::filters::Filter;
use ultraviolet::Vec2;

pub struct GaussianFilter {
    alpha: f32,
    exp: Vec2,
}

impl GaussianFilter {
    pub fn new(radius: Vec2, alpha: f32) -> Self {
        let exp = -alpha * radius * radius;

        Self { alpha, exp }
    }

    fn gaussian(&self, point: f32, exp: f32) -> f32 {
        f32::max(0.0, f32::exp(-self.alpha * point * point) - exp)
    }
}

impl Filter for GaussianFilter {
    fn evaluate(&self, point: &Vec2) -> f32 {
        self.gaussian(point.x, self.exp.x) * self.gaussian(point.y, self.exp.y)
    }
}
