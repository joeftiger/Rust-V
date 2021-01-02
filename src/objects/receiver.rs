use crate::bxdf::BSDF;
use geometry::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};

/// # Summary
/// A receiver is a geometry that also has a bsdf.
pub trait ReceiverExt: Geometry {
    /// # Summary
    /// Returns the geometry of this receiver.
    ///
    /// # Returns
    /// * A geometry
    fn geometry(&self) -> &dyn Geometry;

    /// # Summary
    /// Returns the BSDF of this receiver.
    ///
    /// # Returns
    /// * A BSDF
    fn bsdf(&self) -> &BSDF<'_>;
}

/// # Summary
/// A receiver consists of a geometry and a BSDF.
pub struct Receiver<'a, T> {
    geometry: T,
    bsdf: BSDF<'a>,
}

impl<'a, T> Receiver<'a, T> {
    /// # Summary
    /// Creates a new receiver.
    ///
    /// # Arguments
    /// * `geometry` - The geometry of the receiver
    /// * `bsdf` - The BSDF of the receiver
    ///
    /// # Returns
    /// * Self
    pub fn new(geometry: T, bsdf: BSDF<'a>) -> Self {
        Self { geometry, bsdf }
    }
}

impl<T> ReceiverExt for Receiver<'_, T>
where
    T: Geometry,
{
    fn geometry(&self) -> &dyn Geometry {
        &self.geometry
    }

    fn bsdf(&self) -> &BSDF<'_> {
        &self.bsdf
    }
}

impl<T> Boundable for Receiver<'_, T>
where
    T: Boundable,
{
    fn bounds(&self) -> Aabb {
        self.geometry.bounds()
    }
}

impl<T> Intersectable for Receiver<'_, T>
where
    T: Intersectable,
{
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        self.geometry.intersect(ray)
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.geometry.intersects(ray)
    }
}
