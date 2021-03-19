use crate::bxdf::BSDF;
use geometry::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};
use serde::{Deserialize, Serialize};

/// A receiver consists of a geometry and a BSDF.
#[derive(Serialize, Deserialize)]
pub struct Receiver {
    geometry: Box<dyn Geometry>,
    pub bsdf: BSDF,
}

impl Receiver {
    /// Creates a new receiver.
    ///
    /// # Arguments
    /// * `geometry` - The geometry of the receiver
    /// * `bsdf` - The BSDF of the receiver
    ///
    /// # Returns
    /// * Self
    pub fn new(geometry: Box<dyn Geometry>, bsdf: BSDF) -> Self {
        Self { geometry, bsdf }
    }
}

impl Boundable for Receiver {
    fn bounds(&self) -> Aabb {
        self.geometry.bounds()
    }
}

impl Intersectable for Receiver {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        self.geometry.intersect(ray)
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.geometry.intersects(ray)
    }
}
