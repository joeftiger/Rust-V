mod perspective;
//pub mod perspective_simone;

pub use perspective::PerspectiveCamera;

use crate::filters::Filter;
use dyn_clone::DynClone;
use geometry::Ray;
use ultraviolet::UVec2;

#[typetag::serde]
pub trait Camera: DynClone + Send + Sync {
    fn resolution(&self) -> UVec2;

    fn get_filter(&self) -> &dyn Filter;

    /// Creates a new primary ray of the given pixel.
    ///
    /// # Constraints
    /// * `pixel` - Should be within the camera's resolution.
    ///
    /// # Arguments
    /// * `pixel` - The pixel to generate the ray from
    ///
    /// # Returns
    /// * A ray
    fn primary_ray(&self, pixel: UVec2) -> Ray;
}
