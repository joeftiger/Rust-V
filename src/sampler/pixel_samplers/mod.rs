#![allow(unused_variables)]
#![allow(dead_code)]

use ultraviolet::Vec2;

mod noop_sampler;
mod stratified_sampler;

pub trait PixelSampler {
    /// Generates a new camera pixel anchor for a given pixel.
    ///
    /// # Constraints
    /// * `pixel` - All values should be finite (neither infinite nor `NaN`).
    ///
    /// # Arguments
    /// * `pixel` - The pixel we are sampling/generating for
    ///
    /// # Returns
    /// * A camera pixel anchor in range `[0, 1)`
    fn get(pixel: &Vec2) -> Vec2;
}
