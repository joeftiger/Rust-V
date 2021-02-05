use crate::bxdf::BSDF;
use geometry::{Boundable, Cube, Geometry, Intersectable, Intersection, Ray};

/// A receiver is a geometry that also has a bsdf.
pub trait ReceiverExt: Geometry + Send + Sync {
    /// Returns the geometry of this receiver.
    ///
    /// # Returns
    /// * A geometry
    fn geometry(&self) -> &dyn Geometry;

    /// Returns the BSDF of this receiver.
    ///
    /// # Returns
    /// * A BSDF
    fn bsdf(&self) -> &BSDF;
}

/// A receiver consists of a geometry and a BSDF.
pub struct Receiver<T> {
    geometry: T,
    bsdf: BSDF,
}

impl<T> Receiver<T> {
    /// Creates a new receiver.
    ///
    /// # Arguments
    /// * `geometry` - The geometry of the receiver
    /// * `bsdf` - The BSDF of the receiver
    ///
    /// # Returns
    /// * Self
    pub fn new(geometry: T, bsdf: BSDF) -> Self {
        Self { geometry, bsdf }
    }
}

impl<T> ReceiverExt for Receiver<T>
where
    T: Geometry + Send + Sync,
{
    fn geometry(&self) -> &dyn Geometry {
        &self.geometry
    }

    fn bsdf(&self) -> &BSDF {
        &self.bsdf
    }
}

impl<T> Boundable for Receiver<T>
where
    T: Boundable,
{
    fn bounds(&self) -> Cube {
        self.geometry.bounds()
    }
}

impl<T> Intersectable for Receiver<T>
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
