#![allow(dead_code)]
#![allow(unused_variables)]

use crate::demo_scenes::{DemoScene, FOVY};
use color::Color;
use geometry::{Aabb, Cylinder, Point, Sphere};
use rust_v::bxdf::{
    FresnelType, LambertianReflection, SpecularReflection, SpecularTransmission, BSDF,
};
use rust_v::camera::{Camera, PerspectiveCamera};
use rust_v::objects::{Emitter, Receiver, SceneObject};
use rust_v::refractive_index::RefractiveType;
use rust_v::sampler::pixel_samplers::PixelSamplerType;
use rust_v::scene::Scene;
use rust_v::Spectrum;
use std::sync::Arc;
use ultraviolet::{UVec2, Vec3};

const FLOOR: f32 = 0.0;
const RADIUS: f32 = 0.5;

pub struct DebugScene;

impl DemoScene for DebugScene {
    fn create(resolution: UVec2) -> Scene {
        let mut scene = Scene::default();

        // scene.add(ground()).add(sphere()).add(sphere_emitter());
        scene.add(ground());
        // scene.add(sphere());
        scene.add(cylinder());
        scene.add(sphere_emitter());
        scene.add(create_emitter());

        scene.camera = create_camera(resolution);

        scene
    }
}

//noinspection DuplicatedCode
fn ground() -> SceneObject {
    let min = Vec3::new(-10000.0, FLOOR - 5.0, -10000.0);
    let max = Vec3::new(10000.0, FLOOR, 10000.0);
    let cube = Aabb::new(min, max);

    let lambertian = LambertianReflection::new(Spectrum::broadcast(1.0));
    let bxdf = Box::new(lambertian);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(cube), bsdf));

    SceneObject::Receiver(receiver)
}

fn cylinder() -> SceneObject {
    let bot = Vec3::new(-RADIUS * 1.25, 0.0, 0.0);
    let top = bot + Vec3::unit_y() * RADIUS;

    let cylinder = Cylinder::new((bot, top), RADIUS);

    let specular = SpecularReflection::new(Spectrum::broadcast(1.0), FresnelType::NoOp);
    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(cylinder), bsdf));
    SceneObject::Receiver(receiver)
}

fn sphere() -> SceneObject {
    let center = Vec3::new(-RADIUS * 1.25, RADIUS, 0.0);
    let sphere = Sphere::new(center, RADIUS);

    let specular = SpecularTransmission::new(
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Glass,
    );
    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(sphere), bsdf));
    SceneObject::Receiver(receiver)
}

fn sphere_emitter() -> SceneObject {
    let center = Vec3::new(RADIUS * 1.25, RADIUS, 0.0);
    // let center = Vec3::new(0.0, RADIUS, 0.0);
    let sphere = Sphere::new(center, RADIUS);

    let bsdf = BSDF::empty();

    let emitter = Arc::new(Emitter::new(
        Box::new(sphere),
        bsdf,
        Spectrum::broadcast(1.0),
    ));
    SceneObject::Emitter(emitter)
}

fn create_emitter() -> SceneObject {
    let position = Vec3::new(0.0, 200.0, 0.0);
    let point = Point(position);

    let bsdf = BSDF::empty();
    let emitter = Arc::new(Emitter::new(
        Box::new(point),
        bsdf,
        Spectrum::broadcast(0.1),
    ));
    SceneObject::Emitter(emitter)
}

//noinspection DuplicatedCode
fn create_camera(resolution: UVec2) -> Box<dyn Camera> {
    let position = Vec3::new(0.0, 2.0, 5.0);
    let target = Vec3::new(0.0, RADIUS, 0.0);

    let camera = PerspectiveCamera::new(
        PixelSamplerType::Random,
        position,
        target,
        Vec3::unit_y(),
        FOVY / 2.0,
        resolution,
    );

    Box::new(camera)
}
