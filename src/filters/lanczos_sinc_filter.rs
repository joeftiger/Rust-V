use crate::filters::Filter;
use ultraviolet::Vec2;
use utility::math::sinc;

pub struct LanczosSincFilter {
    radius: Vec2,
    tau: f32,
}

impl LanczosSincFilter {
    pub fn new(radius: Vec2, tau: f32) -> Self {
        Self { radius, tau }
    }

    fn windowed_sinc(&self, mut x: f32, radius: f32) -> f32 {
        x = x.abs();

        if x > radius {
            0.0
        } else {
            let lanczos = sinc(x / self.tau);

            sinc(x) * lanczos
        }
    }
}

impl Filter for LanczosSincFilter {
    fn evaluate(&self, point: &Vec2) -> f32 {
        self.windowed_sinc(point.x, self.radius.x) * self.windowed_sinc(point.y, self.radius.y)
    }
}
