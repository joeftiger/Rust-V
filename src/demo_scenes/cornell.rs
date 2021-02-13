#![allow(dead_code)]

use crate::bxdf::refraction_index::{AIR, GLASS, WATER};
use crate::bxdf::{FresnelSpecular, OrenNayar, BSDF};
use crate::camera::{Camera, PerspectiveCamera};
use crate::demo_scenes::{DemoScene, FOVY, SIGMA};
use crate::objects::Receiver;
use crate::objects::{Emitter, SceneObject};
use crate::scene::Scene;
use crate::Spectrum;
use color::{Color, Colors};
use geometry::lenses::BiconvexLens;
use geometry::{Aabb, Boundable, Bubble, Mesh, ShadingMode, Sphere};
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

fn create_camera(resolution: UVec2) -> Arc<dyn Camera> {
    let position = Vec3::new(X_CENTER, Y_CENTER, FRONT + DIMENSION / 2.0);
    let target = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);

    let camera = PerspectiveCamera::new(position, target, Vec3::unit_y(), FOVY, resolution);

    Arc::new(camera)
}

fn create_bunny() -> SceneObject {
    let file_name = "./meshes/buddha.obj";
    let (model, _) = tobj::load_obj(file_name, true).expect("Could not load bunny file");

    let mut bunny = Mesh::load(&model[0].mesh, ShadingMode::Phong);

    // bunny.translate(-bunny.bounds().center());
    bunny.scale(Vec3::broadcast(3.0));

    // translation + scale + rotation
    let bounds = bunny.bounds();
    let center = bounds.center();
    let center_floor = Vec3::new(center.x, bounds.min.y, center.z);

    let translation = Vec3::new(X_CENTER, FLOOR + 0.01, Z_CENTER) - center_floor;
    bunny.rotate(Rotor3::from_rotation_xz(PI));
    bunny.translate(translation);
    bunny.build_bvh();

    let specular = FresnelSpecular::new(
        Spectrum::new_const(1.0),
        Spectrum::new_const(1.0),
        AIR,
        GLASS,
    );

    let bsdf = BSDF::new(vec![Box::new(specular)]);

    let receiver = Receiver::new(bunny, bsdf);

    SceneObject::new_receiver(receiver)
}

fn create_sphere() -> SceneObject {
    let center = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);
    let sphere = Sphere::new(center, RADIUS * 2.0);

    let specular = FresnelSpecular::new(
        Spectrum::new_const(1.0),
        Spectrum::new_const(1.0),
        AIR,
        GLASS,
    );
    // let reflection = SpecularReflection::new(Spectrum::new_const(1.0), Box::new(FresnelNoOp));
    let bsdf = BSDF::new(vec![Box::new(specular)]);

    let receiver = Receiver::new(sphere, bsdf);
    SceneObject::new_receiver(receiver)
}

fn create_bubble() -> SceneObject {
    let center = Vec3::new(X_CENTER, Y_CENTER, Z_CENTER);
    let offset = Vec3::unit_y() * (THICKNESS / 3.0);
    let bubble = Bubble::new(center, RADIUS * 2.0, THICKNESS, offset);

    let specular = FresnelSpecular::new(
        Spectrum::new_const(1.0),
        Spectrum::new_const(1.0),
        AIR,
        WATER,
    );
    let bsdf = BSDF::new(vec![Box::new(specular)]);

    let receiver = Receiver::new(bubble, bsdf);
    SceneObject::new_receiver(receiver)
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
        Spectrum::new_const(1.0),
        Spectrum::new_const(1.0),
        AIR,
        GLASS,
    );
    let bsdf = BSDF::new(vec![Box::new(specular)]);

    let receiver = Receiver::new(lens, bsdf);
    SceneObject::new_receiver(receiver)
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

    let lambertian = Box::new(OrenNayar::new(spectrum, SIGMA));
    let bsdf = BSDF::new(vec![lambertian]);

    let receiver = Receiver::new(cube, bsdf);
    SceneObject::new_receiver(receiver)
}

fn create_emitter() -> SceneObject {
    let center = Vec3::new(X_CENTER, CEILING, Z_CENTER);
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

        // scene.add(create_sphere());
        // scene.add(create_bunny());
        scene.add(create_biconvex_lens());
        // scene.add(create_bubble());
        scene.add(create_emitter());

        (scene, camera)
    }
}
