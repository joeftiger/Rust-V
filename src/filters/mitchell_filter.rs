use crate::filters::Filter;
use ultraviolet::Vec2;

pub struct MitchellFilter {
    inv_radius: Vec2,
    b: f32,
    c: f32,
}

impl MitchellFilter {
    pub fn new(radius: Vec2, b: f32, c: f32) -> Self {
        let inv_radius = Vec2::broadcast(1.0) / radius;

        Self { inv_radius, b, c }
    }

    fn mitchell_1d(&self, mut x: f32) -> f32 {
        x = 2.0 * x.abs();
        let x2 = x * x;
        let x3 = x2 * x;

        if x > 1.0 {
            ((-self.b - 6.0 * self.c) * x3
                + (6.0 * self.b + 30.0 * self.c) * x2
                + (-12.0 * self.b - 48.0 * self.c) * x
                + (8.0 * self.b + 24.0 * self.c))
                / 6.0
        } else {
            ((12.0 - 9.0 * self.b - 6.0 * self.c) * x3
                + (-18.0 + 12.0 * self.b + 6.0 * self.c) * x2
                + (6.0 - 2.0 * self.b))
                / 6.0
        }
    }
}

impl Filter for MitchellFilter {
    fn evaluate(&self, point: &Vec2) -> f32 {
        let var = *point * self.inv_radius;
        self.mitchell_1d(var.x) * self.mitchell_1d(var.y)
    }
}