use crate::camera::{Camera, NoOpCamera};
use crate::objects::{Emitter, SceneObject};
use definitions::{Float, Vector3};
use geometry::bvh::Tree;
use geometry::{Aabb, Boundable, Intersectable, Intersection, Ray};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A scene intersection is a more detailed `Intersection`, also containing a reference to the
/// intersected object.
#[derive(Clone)]
pub struct SceneIntersection {
    pub point: Vector3,
    pub normal: Vector3,
    pub ray: Ray,
    pub t: Float,
    pub object: SceneObject,
}

impl SceneIntersection {
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
            point: intersection.point,
            normal: intersection.normal,
            ray: intersection.ray,
            t: intersection.t,
            object,
        }
    }
}

/// A scene consists of scene objects and lights.
#[derive(Serialize, Deserialize)]
pub struct Scene {
    bounding_box: Aabb,
    #[serde(skip)]
    pub emitters: Vec<Arc<Emitter>>,
    objects: Vec<SceneObject>,
    #[serde(skip)]
    bvh: Tree<SceneObject>,
    pub camera: Box<dyn Camera>,
}

impl Scene {
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
        self.objects.push(obj.clone());

        if let SceneObject::Emitter(ref e) = obj {
            self.emitters.push(e.clone())
        }

        self.bounding_box = self.bounding_box.join(&obj.bounds());

        self
    }

    /// Recollects all emitters into a cached list.
    pub fn init(&mut self) {
        // collect emitters
        self.emitters.clear();
        for o in &self.objects {
            match o {
                SceneObject::Emitter(e) => self.emitters.push(e.clone()),
                SceneObject::Receiver(_) => {}
            }
        }
        self.emitters.shrink_to_fit();

        self.bvh = Tree::new(self.objects.clone(), |s| s.bounds());
    }

    /// Intersects the scene with the given ray.
    ///
    /// # Performance
    /// It currently uses no search tree, only brute force intersection code.
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect against
    ///
    /// # Returns
    /// * A scene intersection (if any)
    pub fn intersect(&self, ray: &Ray) -> Option<SceneIntersection> {
        let mut new_ray = *ray;
        let mut intersection = None;

        let hits = self.bvh.intersect(ray);
        for hit in &hits {
            if let Some(i) = hit.intersect(&new_ray) {
                new_ray.t_end = i.t;
                intersection = Some(SceneIntersection::new(i, hit.clone().as_ref().clone()));
            }
        }

        if let Some(mut i) = intersection {
            i.ray = *ray;
            Some(i)
        } else {
            None
        }
    }

    pub fn is_occluded(&self, ray: &Ray) -> bool {
        self.intersects(ray)
    }

    /// Intersects the scene with the given ray.
    ///
    /// # Performance
    /// It currently uses no search tree, only brute force intersection code.
    ///
    /// # Arguments
    /// * `ray` - The ray to intersect against
    ///
    /// # Returns
    /// * Whether the ray intersects
    pub fn intersects(&self, ray: &Ray) -> bool {
        self.bvh.intersect(ray).iter().any(|s| s.intersects(ray))
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            bounding_box: Aabb::empty(),
            emitters: Vec::default(),
            objects: Vec::default(),
            bvh: Tree::default(),
            camera: Box::new(NoOpCamera),
        }
    }
}
