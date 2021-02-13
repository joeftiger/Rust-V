use ultraviolet::Vec2;

mod stratified_sampler;

pub trait PixelSampler {
    fn new(samples_per_pixel: u32) {}
    /// Generates a new camera pixel offset for a given pixel
    ///
    /// # Constraints
    /// * `pixel` - All values should be finite (neither infinite nor `NaN`).
    ///
    /// # Arguments
    /// * `pixel` - The pixel we are sampling/generating for
    ///
    /// # Returns
    /// * A camera pixel offset
    fn get(pixel: &Vec2) -> Vec2;
}
