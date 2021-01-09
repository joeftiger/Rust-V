#![allow(dead_code)]

use crate::bxdf::{LambertianReflection, BSDF};
use crate::camera::{Camera, PerspectiveCamera};
use crate::demo_scenes::{DemoScene, FOVY};
use crate::objects::Receiver;
use crate::objects::{Emitter, SceneObject};
use crate::scene::Scene;
use crate::Spectrum;
use color::Color;
use geometry::{Aabb, Sphere};
use std::sync::Arc;
use ultraviolet::{UVec2, Vec3};

const DIMENSION: f32 = 4.0;

const LEFT_WALL: f32 = -DIMENSION / 2.0;
const RIGHT_WALL: f32 = DIMENSION / 2.0;
const BACK_WALL: f32 = -DIMENSION / 2.0;
const FLOOR: f32 = -DIMENSION / 2.0;
const CEILING: f32 = DIMENSION / 2.0;
const FRONT: f32 = DIMENSION / 2.0;
const THICKNESS: f32 = DIMENSION / 50.0;
const RADIUS: f32 = DIMENSION / 8.0;

const X_CENTER: f32 = (RIGHT_WALL + LEFT_WALL) / 2.0;
const Y_CENTER: f32 = (CEILING + FLOOR) / 2.0;
const Z_CENTER: f32 = (BACK_WALL + FRONT) / 2.0;

pub struct CornellScene;

enum Wall {
    Top,
    Left,
    Back,
    Right,
    Bottom,
}

impl Wall {
    pub fn list() -> [Wall; 5] {
        [Self::Top, Self::Left, Self::Back, Self::Right, Self::Bottom]
    }
}

fn create_camera(resolution: UVec2) -> Arc<dyn Camera> {
    let position = Vec3::new(X_CENTER, Y_CENTER, FRONT + DIMENSION / 2.0);
    let target = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);

    let camera = PerspectiveCamera::new(position, target, Vec3::unit_y(), FOVY, resolution);

    Arc::new(camera)
}

fn create_wall(wall: &Wall) -> SceneObject {
    let min = match wall {
        Wall::Top => Vec3::new(LEFT_WALL - THICKNESS, CEILING, BACK_WALL - THICKNESS),
        Wall::Left | Wall::Back | Wall::Bottom => Vec3::new(
            LEFT_WALL - THICKNESS,
            FLOOR - THICKNESS,
            BACK_WALL - THICKNESS,
        ),
        Wall::Right => Vec3::new(RIGHT_WALL, FLOOR - THICKNESS, BACK_WALL - THICKNESS),
    };
    let max = match wall {
        Wall::Top => Vec3::new(RIGHT_WALL + THICKNESS, CEILING + THICKNESS, FRONT),
        Wall::Left => Vec3::new(LEFT_WALL, CEILING + THICKNESS, FRONT),
        Wall::Back => Vec3::new(RIGHT_WALL + THICKNESS, CEILING + THICKNESS, BACK_WALL),
        Wall::Right => Vec3::new(RIGHT_WALL + THICKNESS, CEILING + THICKNESS, FRONT),
        Wall::Bottom => Vec3::new(RIGHT_WALL + THICKNESS, FLOOR, FRONT),
    };
    let aabb = Aabb::new(min, max);

    let spectrum = match wall {
        Wall::Top | Wall::Back | Wall::Bottom => Spectrum::white(),
        Wall::Left => Spectrum::red(),
        Wall::Right => Spectrum::green(),
    };

    let lambertian = Box::new(LambertianReflection::new(spectrum));
    let bsdf = BSDF::new(vec![lambertian]);

    let receiver = Receiver::new(aabb, bsdf);
    SceneObject::new_receiver(receiver)
}

fn create_emitter() -> SceneObject {
    let center = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);
    let sphere = Sphere::new(center, RADIUS);

    let bsdf = BSDF::empty();

    let emission = Spectrum::white() * 15.0;
    let emitter = Emitter::new(sphere, bsdf, emission);
    SceneObject::new_emitter(emitter)
}

impl DemoScene for CornellScene {
    fn create(resolution: UVec2) -> (Scene, Arc<dyn Camera>) {
        let camera = create_camera(resolution);

        let mut scene = Scene::default();

        Wall::list().iter().for_each(|wall| {
            scene.add(create_wall(wall));
        });

        scene.add(create_emitter());

        (scene, camera)
    }
}
