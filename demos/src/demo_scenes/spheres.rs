#![allow(dead_code)]
#![allow(unused_variables)]

use crate::demo_scenes::{Demo, FOVY};
use color::{AsColor, Color, Colors};
use definitions::{Float, Vector3};
use geometry::{Aabb, Point, Sphere};
use rust_v::bxdf::{
    FresnelType, LambertianReflection, SpecularReflection, SpecularTransmission, BSDF,
};
use rust_v::camera::{Camera, PerspectiveCamera};
use rust_v::objects::{Emitter, Receiver, SceneObject};
use rust_v::refractive_index::RefractiveType;
use rust_v::samplers::camera::CameraSampler;
use rust_v::scene::Scene;
use rust_v::serialization::Serialization;
use rust_v::Spectrum;
use std::sync::Arc;
use ultraviolet::UVec2;

const FLOOR: Float = 0.0;
const SKY_RADIUS: Float = 500.0;
const RADIUS: Float = 0.5;

const DISTRIBUTION: Float = 10.0;
const NUM_SPHERES_IN_DIMENSION: u32 = 5;

pub struct SphereScene;

impl Demo for SphereScene {
    fn create() -> Serialization {
        let (resolution, config, integrator, sampler, _) = Self::empty();

        let scene = create_scene();
        let camera = create_camera(resolution);

        Serialization {
            config,
            camera,
            integrator,
            sampler,
            scene,
        }
    }
}

fn ground() -> SceneObject {
    let min = Vector3::new(-1000000.0, FLOOR - 5.0, -1000000.0);
    let max = Vector3::new(1000000.0, FLOOR, 1000000.0);
    let cube = Aabb::new(min, max);

    let lambertian = LambertianReflection::new(Spectrum::from(Colors::White));
    let bxdf = Box::new(lambertian);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(cube), bsdf));

    SceneObject::Receiver(receiver)
}

fn sky() -> SceneObject {
    let center = Vector3::zero();
    let sphere = Sphere::new(center, SKY_RADIUS);

    let lambertian = LambertianReflection::new(Spectrum::from(Colors::BlueSky));
    let bxdf = Box::new(lambertian);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(sphere), bsdf));
    SceneObject::Receiver(receiver)
}

fn random_pos() -> Vector3 {
    let (x, z) = {
        #[cfg(feature = "f64")]
        {
            (
                DISTRIBUTION * (fastrand::f64() - 0.5),
                DISTRIBUTION * (fastrand::f64() - 0.5),
            )
        }
        #[cfg(not(feature = "f64"))]
        {
            (
                DISTRIBUTION * (fastrand::f32() - 0.5),
                DISTRIBUTION * (fastrand::f32() - 0.5),
            )
        }
    };

    Vector3::new(x, RADIUS, z)
}

fn random_color() -> Spectrum {
    let rand = fastrand::f32() * 1.5;

    if rand < 0.25 {
        Spectrum::from(Colors::Red)
    } else if rand < 0.5 {
        Spectrum::from(Colors::Green)
    } else if rand < 0.75 {
        Spectrum::from(Colors::Blue)
    } else {
        Spectrum::from(Colors::White)
    }
}

fn random_bsdf(color: Spectrum) -> (bool, BSDF) {
    let rand = fastrand::f32();

    let mut out = false;
    let bsdf = if color == Spectrum::from(Colors::White) {
        if rand < 0.6 {
            out = true;
            BSDF::empty()
        } else if rand < 0.8 {
            let specular = SpecularReflection::new(Spectrum::broadcast(1.0), FresnelType::NoOp);
            let bxdf = Box::new(specular);

            BSDF::new(vec![bxdf])
        } else {
            let specular = SpecularTransmission::new(
                Spectrum::broadcast(1.0),
                RefractiveType::Air,
                RefractiveType::Glass,
            );
            let bxdf = Box::new(specular);

            BSDF::new(vec![bxdf])
        }
    } else {
        let lambertian = LambertianReflection::new(color);
        let bxdf = Box::new(lambertian);

        BSDF::new(vec![bxdf])
    };

    (out, bsdf)
}

fn create_emitter() -> SceneObject {
    let position = Vector3::new(0.0, SKY_RADIUS / 2.0, 0.0);
    let point = Point(position);

    let bsdf = BSDF::empty();
    let emission = Spectrum::from(Colors::OrangeYellow);

    let emitter = Arc::new(Emitter::new(Box::new(point), bsdf, emission));
    SceneObject::Emitter(emitter)
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
                let emitter = Arc::new(Emitter::new(Box::new(sphere), bsdf, color * 2.0));
                SceneObject::Emitter(emitter)
            } else {
                let receiver = Arc::new(Receiver::new(Box::new(sphere), bsdf));
                SceneObject::Receiver(receiver)
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
    let position = Vector3::new(0.0, 5.0, 10.0);
    let target = Vector3::new(0.0, 1.0, 0.0);

    let camera = PerspectiveCamera::new(
        CameraSampler::Random,
        position,
        target,
        Vector3::unit_y(),
        FOVY,
        resolution,
    );
    // let camera = crate::camera::perspective_simone::PerspectiveCamera::new(position, target, Vector3::unit_y(), FOVY, resolution);

    Arc::new(camera)
}
