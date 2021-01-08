mod perspective;
pub mod perspective_simone;

pub use perspective::PerspectiveCamera;

use geometry::Ray;
use ultraviolet::{UVec2, Vec2};

pub trait Camera: Send + Sync {
    /// # Summary
    /// Creates a new primary ray of the given pixel.
    ///
    /// # Constraints
    /// * `pixel` - Should be within the camera's resolution.
    /// * `sample` - Should be within `[0, 1)`.
    ///
    /// # Arguments
    /// * `pixel` - The pixel to generate the ray from
    /// * `sample` - A random sample to "fine-tune" where in the pixel
    ///
    /// # Returns
    /// * A ray
    fn primary_ray(&self, pixel: &UVec2, sample: &Vec2) -> Ray;
}
