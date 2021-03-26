use crate::objects::SurfaceSample;
use geometry::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};
use ultraviolet::{Vec2, Vec3};

mod point;
mod sphere;

impl Boundable for Box<dyn Sampleable> {
    fn bounds(&self) -> Aabb {
        self.as_ref().bounds()
    }
}

impl Intersectable for Box<dyn Sampleable> {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        self.as_ref().intersect(ray)
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.as_ref().intersects(ray)
    }
}

#[typetag::serde]
impl Geometry for Box<dyn Sampleable> {}

/// Allows geometries to be sampled for a surface point.
#[typetag::serde]
pub trait Sampleable: Geometry + Send + Sync {
    /// Returns the surface area of this object.
    ///
    /// # Returns
    /// * The surface area
    fn surface_area(&self) -> f32;

    /// Samples the surface from the given point in the "solid angle" form.
    ///
    /// # Constraints
    /// * `point` - All values should be finite (neither infinite nor `NaN`).
    /// * `sample` - ALl values should be inside `[0, 1)`.
    ///
    /// # Arguments
    /// * `point` - The point at which we look at the object
    /// * `sample` - A random sample
    ///
    /// # Returns
    /// * A surface sample
    fn sample_surface(&self, origin: &Vec3, sample: &Vec2) -> SurfaceSample;
}
