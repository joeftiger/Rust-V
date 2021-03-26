use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use emitter::*;
use geometry::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};
pub use receiver::*;

use crate::bxdf::BSDF;

mod emitter;
mod receiver;
mod sampleables;
pub use sampleables::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum SceneObject {
    Emitter(Arc<Emitter>),
    Receiver(Arc<Receiver>),
}

impl SceneObject {
    pub fn bsdf(&self) -> &BSDF {
        match self {
            SceneObject::Emitter(e) => &e.bsdf,
            SceneObject::Receiver(r) => &r.bsdf,
        }
    }
}

impl Boundable for SceneObject {
    fn bounds(&self) -> Aabb {
        match self {
            SceneObject::Emitter(e) => e.bounds(),
            SceneObject::Receiver(r) => r.bounds(),
        }
    }
}

impl Intersectable for SceneObject {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            SceneObject::Emitter(e) => e.intersect(ray),
            SceneObject::Receiver(r) => r.intersect(ray),
        }
    }

    fn intersects(&self, ray: &Ray) -> bool {
        match self {
            SceneObject::Emitter(e) => e.intersects(ray),
            SceneObject::Receiver(r) => r.intersects(ray),
        }
    }
}
