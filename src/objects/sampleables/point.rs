use crate::objects::emitter::{Sampleable, SurfaceSample};
use crate::*;

use geometry::Point;

#[typetag::serde]
impl Sampleable for Point {
    /// A point has no surface area
    ///
    /// # Returns
    /// * `0.0`
    fn surface_area(&self) -> Float {
        0.0
    }

    /// A point cannot be sampled plausibly.
    ///
    /// # Constraints
    /// * `point` - ALl values should be finite (neither infinite nor `NaN`).
    ///
    /// # Arguments
    /// * `point` - The point from which we sample the emitter
    /// * `_` - Ignored
    ///
    /// # Returns
    /// * An emitter sample with normal towards the `point`
    fn sample_surface(&self, point: Vector3, _: Vector2) -> SurfaceSample {
        let normal = point - self.0;

        SurfaceSample::new(self.0, normal.normalized(), 1.0)
    }
}
