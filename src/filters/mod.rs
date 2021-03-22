mod box_filter;
mod gaussian_filter;
mod lanczos_sinc_filter;
mod mitchell_filter;
mod triangle_filter;

pub use box_filter::*;
pub use gaussian_filter::*;
pub use lanczos_sinc_filter::*;
pub use mitchell_filter::*;
pub use triangle_filter::*;

use serde::{Deserialize, Serialize};
use ultraviolet::Vec2;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum FilterType {
    Box(BoxFilter),
    Triangle(TriangleFilter),
    Gaussian(GaussianFilter),
    LanczosSinc(LanczosSincFilter),
    Mitchell(MitchellFilter),
}

impl FilterType {
    pub fn radius(&self) -> Vec2 {
        match self {
            FilterType::Box(f) => f.radius,
            FilterType::Triangle(f) => f.radius,
            FilterType::Gaussian(f) => f.radius,
            FilterType::LanczosSinc(f) => f.radius,
            FilterType::Mitchell(f) => f.radius,
        }
    }

    pub fn inv_radius(&self) -> Vec2 {
        match self {
            FilterType::Box(f) => f.inv_radius,
            FilterType::Triangle(f) => f.inv_radius,
            FilterType::Gaussian(f) => f.inv_radius,
            FilterType::LanczosSinc(f) => f.inv_radius,
            FilterType::Mitchell(f) => f.inv_radius,
        }
    }
}

impl Filter for FilterType {
    fn evaluate(&self, point: &Vec2) -> f32 {
        match self {
            FilterType::Box(f) => f.evaluate(point),
            FilterType::Triangle(f) => f.evaluate(point),
            FilterType::Gaussian(f) => f.evaluate(point),
            FilterType::LanczosSinc(f) => f.evaluate(point),
            FilterType::Mitchell(f) => f.evaluate(point),
        }
    }
}

pub trait Filter {
    /// Evaluates a relative sample point to filter center position.
    ///
    /// # Constraints
    /// * `point` - All values should be finite (neither infinite nor `NaN`).
    ///             Should not be outside the filter's extent.
    ///
    /// # Arguments
    /// * `point` - The relative point to evaluate on the filter function
    ///
    /// # Returns
    /// * The evaluated value
    fn evaluate(&self, point: &Vec2) -> f32;
}
