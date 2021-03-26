use crate::camera::CameraType;
use crate::objects::{Emitter, SceneObject};
use geometry::{Aabb, Boundable, ContainerGeometry, Intersectable, Intersection, Ray};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use ultraviolet::Vec3;

/// A scene intersection is a more detailed `Intersection`, also containing a reference to the
/// intersected object.
#[derive(Clone)]
pub struct SceneIntersection {
    pub point: Vec3,
    pub normal: Vec3,
    pub ray: Ray,
    pub t: f32,
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
    pub lights: Vec<Arc<Emitter>>,
    objects: Vec<SceneObject>,
    pub camera: Mutex<CameraType>,
}

impl Scene {
    pub fn init(&mut self) {
        self.lights.clear();

        for s in &self.objects {
            if let SceneObject::Emitter(e) = s {
                self.lights.push(e.clone());
            }
        }

        self.collect_emitters();
    }

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
            self.lights.push(e.clone())
        }

        self.bounding_box = self.bounding_box.join(&obj.bounds());

        self
    }

    /// Recollects all emitters into a cached list.
    fn collect_emitters(&mut self) {
        self.lights.clear();

        for o in &self.objects {
            match o {
                SceneObject::Emitter(e) => self.lights.push(e.clone()),
                SceneObject::Receiver(_) => {}
            }
        }

        self.lights.shrink_to_fit();
    }

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
        if !self.bounding_box.intersects(&ray) {
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

        if let Some(mut i) = intersection {
            i.ray = *ray;

            Some(i)
        } else {
            None
        }
    }

    pub fn is_occluded(&self, ray: &Ray) -> bool {
        self.objects.iter().any(|o| o.intersects(ray))
    }

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
        self.bounding_box.contains_or_intersects(ray)
            && self.objects.iter().any(|o| o.intersects(ray))
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            bounding_box: Aabb::empty(),
            lights: Vec::default(),
            objects: Vec::default(),
            camera: Mutex::new(CameraType::Dummy),
        }
    }
}

// impl Serialize for Scene {
//     fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
//         S: Serializer {
//         self.objects.serialize(serializer)
//     }
// }
