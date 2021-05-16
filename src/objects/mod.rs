use geometry::{Aabb, Boundable, Intersectable, Intersection, Ray};

mod emitter;
mod receiver;
mod sampleables;

use crate::bxdf::BSDF;
pub use emitter::*;
pub use receiver::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub enum SceneObject {
    Emitter(Arc<Emitter>),
    Receiver(Arc<Receiver>),
}

impl SceneObject {
    #[inline]
    pub fn bsdf(&self) -> &BSDF {
        match self {
            SceneObject::Emitter(e) => &e.bsdf,
            SceneObject::Receiver(r) => &r.bsdf,
        }
    }
}

impl Boundable for SceneObject {
    #[inline]
    fn bounds(&self) -> Aabb {
        match self {
            SceneObject::Emitter(e) => e.bounds(),
            SceneObject::Receiver(r) => r.bounds(),
        }
    }
}

impl Intersectable for SceneObject {
    #[inline]
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            SceneObject::Emitter(e) => e.intersect(ray),
            SceneObject::Receiver(r) => r.intersect(ray),
        }
    }

    #[inline]
    fn intersects(&self, ray: &Ray) -> bool {
        match self {
            SceneObject::Emitter(e) => e.intersects(ray),
            SceneObject::Receiver(r) => r.intersects(ray),
        }
    }
}
