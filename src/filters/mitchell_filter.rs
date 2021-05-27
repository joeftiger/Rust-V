use crate::filters::Filter;
use crate::*;

use serde::{Deserialize, Serialize};

/// Sample weights considered with a parameterized filter function.
///
/// Has a good trade off between ringing and blurring.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct MitchellFilter {
    pub radius: Vector2,
    pub inv_radius: Vector2,
    b: Float,
    c: Float,
}

impl MitchellFilter {
    pub fn new(radius: Vector2, b: Float, c: Float) -> Self {
        Self {
            radius,
            inv_radius: Vector2::one() / radius,
            b,
            c,
        }
    }

    fn mitchell_1d(&self, mut x: Float) -> Float {
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

#[typetag::serde]
impl Filter for MitchellFilter {
    fn radius(&self) -> Vector2 {
        self.radius
    }

    fn inv_radius(&self) -> Vector2 {
        self.inv_radius
    }

    fn evaluate(&self, point: Vector2) -> Float {
        let var = point * self.inv_radius;
        self.mitchell_1d(var.x) * self.mitchell_1d(var.y)
    }
}
