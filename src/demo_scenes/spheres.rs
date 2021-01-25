#![allow(dead_code)]
#![allow(unused_variables)]

use crate::bxdf::refraction_index::{AIR, GLASS};
use crate::bxdf::{
    FresnelNoOp, LambertianReflection, SpecularReflection, SpecularTransmission, TransportMode,
    BSDF,
};
use crate::camera::{Camera, PerspectiveCamera};
use crate::demo_scenes::{DemoScene, FOVY};
use crate::objects::{Emitter, Receiver, SceneObject};
use crate::scene::Scene;
use crate::Spectrum;
use color::Color;
use geometry::{Cube, Point, Sphere};
use std::sync::Arc;
use ultraviolet::{UVec2, Vec3};

const FLOOR: f32 = 0.0;
const SKY_RADIUS: f32 = 500.0;
const RADIUS: f32 = 0.5;

const DISTRIBUTION: f32 = 10.0;
const NUM_SPHERES_IN_DIMENSION: u32 = 5;

pub struct SphereScene;

impl DemoScene for SphereScene {
    fn create(resolution: UVec2) -> (Scene, Arc<dyn Camera>) {
        fastrand::seed(0);
        let scene = create_scene();
        let camera = create_camera(resolution);

        (scene, camera)
    }
}

fn ground() -> SceneObject {
    let min = Vec3::new(-1000000.0, FLOOR - 5.0, -1000000.0);
    let max = Vec3::new(1000000.0, FLOOR, 1000000.0);
    let cube = Cube::new(min, max);

    let lambertian = LambertianReflection::new(Spectrum::white());
    let bsdf = BSDF::new(vec![Box::new(lambertian)]);

    let receiver = Receiver::new(cube, bsdf);

    SceneObject::new_receiver(receiver)
}

fn sky() -> SceneObject {
    let center = Vec3::zero();
    let sphere = Sphere::new(center, SKY_RADIUS);

    let lambertian = LambertianReflection::new(Spectrum::blue() + Spectrum::white() * 0.2);
    let bsdf = BSDF::new(vec![Box::new(lambertian)]);

    let receiver = Receiver::new(sphere, bsdf);
    SceneObject::new_receiver(receiver)
}

fn random_pos() -> Vec3 {
    let x = DISTRIBUTION * (fastrand::f32() - 0.5);
    let z = DISTRIBUTION * (fastrand::f32() - 0.5);

    Vec3::new(x, RADIUS, z)
}

fn random_color() -> Spectrum {
    let rand = fastrand::f32() * 1.5;

    if rand < 0.25 {
        Spectrum::red()
    } else if rand < 0.5 {
        Spectrum::green()
    } else if rand < 0.75 {
        Spectrum::blue()
    } else {
        Spectrum::white()
    }
}

fn random_bsdf(color: Spectrum) -> (bool, BSDF) {
    let rand = fastrand::f32();

    let mut out = false;
    let bsdf = if color == Spectrum::white() {
        if rand < 0.6 {
            out = true;
            BSDF::empty()
        } else if rand < 0.8 {
            let specular = SpecularReflection::new(Spectrum::new_const(1.0), Box::new(FresnelNoOp));
            BSDF::new(vec![Box::new(specular)])
        } else {
            let transmission = SpecularTransmission::new(
                Spectrum::new_const(1.0),
                AIR,
                GLASS,
                TransportMode::Importance,
            );
            let reflection =
                SpecularReflection::new(Spectrum::new_const(1.0), Box::new(FresnelNoOp));
            BSDF::new(vec![
                Box::new(transmission),
                // Box::new(reflection),
            ])
        }
    } else {
        let lambertian = LambertianReflection::new(color);
        BSDF::new(vec![Box::new(lambertian)])
    };

    (out, bsdf)
}

fn create_emitter() -> SceneObject {
    let position = Vec3::new(0.0, SKY_RADIUS / 2.0, 0.0);
    let point = Point(position);

    let bsdf = BSDF::empty();
    let emitter = Emitter::new(
        point,
        bsdf,
        Spectrum::white() + Spectrum::green() + Spectrum::red(),
    );
    SceneObject::new_emitter(emitter)
}

fn create_scene() -> Scene {
    let mut scene = Scene::default();

    for _ in 0..NUM_SPHERES_IN_DIMENSION {
        for _ in 0..NUM_SPHERES_IN_DIMENSION {
            let center = random_pos();
            let sphere = Sphere::new(center, RADIUS);

            let color = random_color();
            let (emitting, bsdf) = random_bsdf(color);

            let obj = if emitting {
                let emitter = Emitter::new(sphere, bsdf, color * 5.0);
                SceneObject::new_emitter(emitter)
            } else {
                let receiver = Receiver::new(sphere, bsdf);
                SceneObject::new_receiver(receiver)
            };

            scene.add(obj);
        }
    }

    scene.add(ground());
    scene.add(sky());
    scene.add(create_emitter());

    scene
}

//noinspection DuplicatedCode
fn create_camera(resolution: UVec2) -> Arc<dyn Camera> {
    let position = Vec3::new(0.0, 5.0, 10.0);
    let target = Vec3::new(0.0, 1.0, 0.0);

    let camera = PerspectiveCamera::new(position, target, Vec3::unit_y(), FOVY, resolution);
    // let camera = crate::camera::perspective_simone::PerspectiveCamera::new(position, target, Vec3::unit_y(), FOVY, resolution);

    Arc::new(camera)
}
