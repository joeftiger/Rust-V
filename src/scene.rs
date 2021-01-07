use crate::objects::{EmitterExt, SceneObject};
use geometry::{Aabb, Boundable, Intersectable, Intersection, Ray};
use std::sync::Arc;

/// # Summary
/// A scene intersection is a more detailed `Intersection`, also containing a reference to the
/// intersected object.
pub struct SceneIntersection {
    pub info: Intersection,
    pub object: SceneObject,
}

impl SceneIntersection {
    /// # Summary
    /// Creates a new scene intersection.
    ///
    /// # Arguments
    /// * `intersection` - The intersection
    /// * `object` - The intersected object
    ///
    /// # Returns
    /// * Self
    pub fn new(intersection: Intersection, object: SceneObject) -> Self {
        Self {
            info: intersection,
            object,
        }
    }
}

/// # Summary
/// A scene consists of scene objects and lights.
pub struct Scene {
    aabb: Aabb,
    pub lights: Vec<Arc<dyn EmitterExt>>,
    objects: Vec<SceneObject>,
}

impl Scene {
    /// # Summary
    /// Adds the given object to the scene.
    ///
    /// If the object is an emitter, it will also be added to the lights.
    ///
    /// # Arguments
    /// * `obj` - The scene object to add
    ///
    /// # Returns
    /// * This same scene (for chaining operations)
    pub fn add(&mut self, obj: SceneObject) -> &mut Self {
        match obj.clone() {
            SceneObject::Emitter(e) => {
                self.lights.push(e.clone());
                self.objects.push(obj.clone());
            }
            SceneObject::Receiver(_) => self.objects.push(obj.clone()),
        };

        self.aabb = self.aabb.join(&obj.bounds());

        self
    }

    /// # Summary
    /// Intersects the scene with the given ray.
    ///
    /// # Performance
    /// It currently uses no search tree, only brute force intersection code.
    /// TODO: implement Bvh
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect against
    ///
    /// # Returns
    /// * A scene intersection (if any)
    pub fn intersect(&self, ray: &Ray) -> Option<SceneIntersection> {
        if !self.aabb.intersects(ray) {
            return None;
        }

        let mut new_ray = *ray;
        let mut intersection = None;

        for o in &self.objects {
            if let Some(i) = o.intersect(&new_ray) {
                new_ray.t_end = i.t;
                let si = SceneIntersection::new(i, o.clone());

                intersection = Some(si);
            }
        }

        intersection
    }

    /// # Summary
    /// Intersects the scene with the given ray.
    ///
    /// # Performance
    /// It currently uses no search tree, only brute force intersection code.
    /// TODO: implement Bvh
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect against
    ///
    /// # Returns
    /// * Whether the ray intersects
    pub fn intersects(&self, ray: &Ray) -> bool {
        if !self.aabb.intersects(ray) {
            return false;
        }

        self.objects.iter().any(|o| o.intersects(ray))
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            aabb: Aabb::empty(),
            lights: Vec::default(),
            objects: Vec::default(),
        }
    }
}
