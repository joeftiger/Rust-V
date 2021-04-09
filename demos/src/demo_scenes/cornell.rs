#![allow(dead_code)]

use crate::demo_scenes::{DemoScene, FOVY, SIGMA};
use color::{Color, Colors};
use geometry::{Aabb, BiconvexLens, Boundable, Bubble, Mesh, ShadingMode, Sphere};
use rust_v::bxdf::{FresnelSpecular, OrenNayar, BSDF};
use rust_v::camera::{Camera, PerspectiveCamera};
use rust_v::objects::{Emitter, Receiver, SceneObject};
use rust_v::refractive_index::RefractiveType;
use rust_v::sampler::pixel_samplers::PixelSamplerType;
use rust_v::scene::Scene;
use rust_v::Spectrum;
use std::f32::consts::PI;
use std::sync::Arc;
use ultraviolet::{Rotor3, UVec2, Vec3};

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

impl DemoScene for CornellScene {
    fn create(resolution: UVec2) -> Scene {
        let mut scene = Scene::default();

        Wall::list().iter().for_each(|wall| {
            scene.add(create_wall(wall));
        });

        // scene.add(create_sphere());
        // scene.add(create_buddha());
        // scene.add(create_biconvex_lens());
        scene.add(create_bubble());
        scene.add(create_emitter());
        scene.camera = create_camera(resolution);

        scene
    }
}

fn create_camera(resolution: UVec2) -> Box<dyn Camera> {
    let position = Vec3::new(X_CENTER, Y_CENTER, FRONT + DIMENSION / 2.0);
    let target = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);

    let camera = PerspectiveCamera::new(
        PixelSamplerType::Random,
        position,
        target,
        Vec3::unit_y(),
        FOVY,
        resolution,
    );

    Box::new(camera)
}

fn create_buddha() -> SceneObject {
    let mut buddha = Mesh::load("./meshes/buddha.obj".to_string(), ShadingMode::Phong);

    buddha.scale(Vec3::broadcast(3.0));

    // translation + scale + rotation
    let bounds = buddha.bounds();
    let center = bounds.center();
    let center_floor = Vec3::new(center.x, bounds.min.y, center.z);

    let translation = Vec3::new(X_CENTER, FLOOR + 0.01, Z_CENTER) - center_floor;
    buddha.rotate(Rotor3::from_rotation_xz(PI + 1.0 / -8.0));
    buddha.translate(translation);
    buddha.build_bvh();

    let specular = FresnelSpecular::new(
        Spectrum::broadcast(1.0),
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Glass,
    );

    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);
    let geometry = Box::new(buddha);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));

    SceneObject::Receiver(receiver)
}

fn create_sphere() -> SceneObject {
    let center = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);
    let sphere = Sphere::new(center, RADIUS * 2.0);

    let specular = FresnelSpecular::new(
        Spectrum::broadcast(1.0),
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Glass,
    );

    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);
    let geometry = Box::new(sphere);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));
    SceneObject::Receiver(receiver)
}

fn create_bubble() -> SceneObject {
    let center = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);
    let offset = Vec3::unit_y() * (THICKNESS / 3.0);
    let bubble = Bubble::new(center, RADIUS * 2.0, THICKNESS, offset);

    let specular = FresnelSpecular::new(
        Spectrum::broadcast(1.0),
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Glass,
    );

    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);
    let geometry = Box::new(bubble);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));
    SceneObject::Receiver(receiver)
}

fn create_biconvex_lens() -> SceneObject {
    let radius = RADIUS * 3.0;
    let dist2 = THICKNESS * 3.0;
    let center0 = Vec3::new(X_CENTER, Y_CENTER + radius - dist2, Z_CENTER);
    let center1 = Vec3::new(X_CENTER, Y_CENTER - radius + dist2, Z_CENTER);
    let sphere0 = Sphere::new(center0, radius);
    let sphere1 = Sphere::new(center1, radius);
    let lens = BiconvexLens::new(sphere0, sphere1);

    let specular = FresnelSpecular::new(
        Spectrum::broadcast(1.0),
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Glass,
    );

    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);
    let geometry = Box::new(lens);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));
    SceneObject::Receiver(receiver)
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
    let cube = Aabb::new(min, max);

    let spectrum = match wall {
        Wall::Top | Wall::Back | Wall::Bottom => Spectrum::white() * 0.75,
        Wall::Left => Spectrum::red() * 0.75,
        Wall::Right => Spectrum::green() * 0.75,
    };

    let oren_nayar = OrenNayar::new(spectrum, SIGMA);
    let bxdf = Box::new(oren_nayar);

    let bsdf = BSDF::new(vec![bxdf]);
    let geometry = Box::new(cube);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));
    SceneObject::Receiver(receiver)
}

fn create_emitter() -> SceneObject {
    let center = Vec3::new(X_CENTER, CEILING, Z_CENTER);
    let sphere = Sphere::new(center, RADIUS);

    let bsdf = BSDF::empty();
    let emission = Spectrum::white() * 2.0;
    let geometry = Box::new(sphere);

    let emitter = Arc::new(Emitter::new(geometry, bsdf, emission));
    SceneObject::Emitter(emitter)
}
