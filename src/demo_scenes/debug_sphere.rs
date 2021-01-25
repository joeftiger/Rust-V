use crate::bxdf::{LambertianReflection, BSDF};
use crate::camera::{Camera, PerspectiveCamera};
use crate::demo_scenes::{DemoScene, FOVY};
use crate::objects::{Emitter, Receiver, SceneObject};
use crate::scene::Scene;
use crate::Spectrum;
use geometry::{Cube, Sphere};
use std::sync::Arc;
use ultraviolet::{UVec2, Vec3};

pub struct DebugSphereScene;

impl DemoScene for DebugSphereScene {
    fn create(resolution: UVec2) -> (Scene, Arc<dyn Camera>) {
        let mut scene = Scene::default();

        scene.add(ground()).add(sphere_emitter());

        let camera = create_camera(resolution);

        (scene, camera)
    }
}

//noinspection DuplicatedCode
fn ground() -> SceneObject {
    let min = Vec3::new(-100.0, -5.0, -100.0);
    let max = Vec3::new(100.0, 0.0, 100.0);
    let cube = Cube::new(min, max);

    let lambertian = LambertianReflection::new(Spectrum::new_const(1.0));
    let bsdf = BSDF::new(vec![Box::new(lambertian)]);

    let receiver = Receiver::new(cube, bsdf);

    SceneObject::new_receiver(receiver)
}

fn sphere_emitter() -> SceneObject {
    let center = Vec3::new(0.0, 20.0, 0.0);
    let sphere = Sphere::new(center, 10.0);

    let bsdf = BSDF::empty();

    let emitter = Emitter::new(sphere, bsdf, Spectrum::new_const(5.0));
    SceneObject::new_emitter(emitter)
}

//noinspection DuplicatedCode
fn create_camera(resolution: UVec2) -> Arc<dyn Camera> {
    let position = Vec3::new(0.0, 3.0, 10.0);
    let target = Vec3::new(0.0, 0.0, 0.0);

    let camera = PerspectiveCamera::new(position, target, Vec3::unit_y(), FOVY / 2.0, resolution);
    Arc::new(camera)
}
