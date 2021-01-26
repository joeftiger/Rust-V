#![allow(dead_code)]
#![allow(unused_variables)]

use crate::bxdf::refraction_index::{AIR, GLASS};
use crate::bxdf::TransportMode::Radiance;
use crate::bxdf::{
    FresnelNoOp, LambertianReflection, SpecularReflection, SpecularTransmission, BSDF,
};
use crate::camera::{Camera, PerspectiveCamera};
use crate::demo_scenes::{DemoScene, FOVY};
use crate::objects::{Emitter, Receiver, SceneObject};
use crate::scene::Scene;
use crate::Spectrum;
use geometry::{Cube, Point, Sphere};
use std::sync::Arc;
use ultraviolet::{UVec2, Vec3};

const FLOOR: f32 = 0.0;
const RADIUS: f32 = 0.5;

pub struct DebugScene;

impl DemoScene for DebugScene {
    fn create(resolution: UVec2) -> (Scene, Arc<dyn Camera>) {
        let mut scene = Scene::default();

        // scene.add(ground()).add(sphere()).add(sphere_emitter());
        scene.add(ground()).add(sphere_emitter());
        // .add(create_emitter());

        let camera = create_camera(resolution);

        (scene, camera)
    }
}

//noinspection DuplicatedCode
fn ground() -> SceneObject {
    let min = Vec3::new(-10000.0, FLOOR - 5.0, -10000.0);
    let max = Vec3::new(10000.0, FLOOR, 10000.0);
    let cube = Cube::new(min, max);

    let lambertian = LambertianReflection::new(Spectrum::new_const(1.0));
    let bsdf = BSDF::new(vec![Box::new(lambertian)]);

    let receiver = Receiver::new(cube, bsdf);

    SceneObject::new_receiver(receiver)
}

fn sphere() -> SceneObject {
    let center = Vec3::new(-RADIUS * 1.25, RADIUS, 0.0);
    let sphere = Sphere::new(center, RADIUS);

    let _transmission = Box::new(SpecularTransmission::new(
        Spectrum::new_const(1.0),
        AIR,
        GLASS,
        Radiance,
    ));
    let reflection = Box::new(SpecularReflection::new(
        Spectrum::new_const(1.0),
        Box::new(FresnelNoOp),
    ));

    let bsdf = BSDF::new(vec![
        // transmission,
        reflection,
    ]);

    let receiver = Receiver::new(sphere, bsdf);
    SceneObject::new_receiver(receiver)
}

fn sphere_emitter() -> SceneObject {
    // let center = Vec3::new(RADIUS * 1.25, RADIUS, 0.0);
    let center = Vec3::new(0.0, RADIUS, 0.0);
    let sphere = Sphere::new(center, RADIUS);

    let bsdf = BSDF::empty();

    let emitter = Emitter::new(sphere, bsdf, Spectrum::new_const(2.0));
    SceneObject::new_emitter(emitter)
}

fn create_emitter() -> SceneObject {
    let position = Vec3::new(0.0, 10.0, 0.0);
    let point = Point(position);

    let bsdf = BSDF::empty();
    let emitter = Emitter::new(point, bsdf, Spectrum::new_const(1.0));
    SceneObject::new_emitter(emitter)
}

//noinspection DuplicatedCode
fn create_camera(resolution: UVec2) -> Arc<dyn Camera> {
    let position = Vec3::new(0.0, 2.0, 5.0);
    let target = Vec3::new(0.0, RADIUS, 0.0);

    let camera = PerspectiveCamera::new(position, target, Vec3::unit_y(), FOVY / 2.0, resolution);
    Arc::new(camera)
}
