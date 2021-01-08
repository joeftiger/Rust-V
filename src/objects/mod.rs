use geometry::{Aabb, Boundable, Geometry, Intersectable, Intersection, Ray};

mod emitter;
mod point;
mod receiver;
mod sphere;

use crate::bxdf::BSDF;
pub use emitter::{Emitter, EmitterExt, Sampleable};
pub use receiver::{Receiver, ReceiverExt};
use std::sync::Arc;

#[derive(Clone)]
pub enum SceneObject {
    Emitter(Arc<dyn EmitterExt>),
    Receiver(Arc<dyn ReceiverExt>),
}

impl SceneObject {
    pub fn new_emitter<T: 'static + EmitterExt>(emitter: T) -> Self {
        Self::Emitter(Arc::new(emitter))
    }

    pub fn new_receiver<T: 'static + ReceiverExt>(receiver: T) -> Self {
        Self::Receiver(Arc::new(receiver))
    }
}

impl ReceiverExt for SceneObject {
    fn geometry(&self) -> &dyn Geometry {
        match self {
            SceneObject::Emitter(e) => e.geometry(),
            SceneObject::Receiver(r) => r.geometry(),
        }
    }

    fn bsdf(&self) -> &BSDF {
        match self {
            SceneObject::Emitter(e) => e.bsdf(),
            SceneObject::Receiver(r) => r.bsdf(),
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
