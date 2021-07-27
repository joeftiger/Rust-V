#![allow(dead_code)]

use crate::demo_scenes::{Demo, Float, Vector3, FOVY, SIGMA};
use color::{Color, Colors};
use geometry::{Aabb, BiconvexLens, Boundable, Bubble, Mesh, ShadingMode, Sphere};
use rust_v::bxdf::{
    BeckmannDistribution, FresnelDielectric, FresnelSpecular, FresnelType, MicrofacetReflection,
    OrenNayar, BSDF,
};
use rust_v::camera::{Camera, PerspectiveCamera};
use rust_v::objects::{Emitter, Receiver, SceneObject};
use rust_v::refractive_index::RefractiveType;
use rust_v::samplers::camera::CameraSampler;

use crate::Rotation3;
use rust_v::serialization::Serialization;
use rust_v::Spectrum;
#[cfg(not(feature = "f64"))]
use std::f32::consts::FRAC_PI_2;
#[cfg(feature = "f64")]
use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;
use ultraviolet::UVec2;

const DIMENSION: Float = 4.0;

const LEFT_WALL: Float = -DIMENSION / 2.0;
const RIGHT_WALL: Float = DIMENSION / 2.0;
const BACK_WALL: Float = -DIMENSION / 2.0;
const FLOOR: Float = -DIMENSION / 2.0;
const CEILING: Float = DIMENSION / 2.0;
const FRONT: Float = DIMENSION / 2.0;
const THICKNESS: Float = DIMENSION / 50.0;
const RADIUS: Float = DIMENSION / 16.0;

const X_CENTER: Float = (RIGHT_WALL + LEFT_WALL) / 2.0;
const Y_CENTER: Float = (CEILING + FLOOR) / 2.0;
const Z_CENTER: Float = (BACK_WALL + FRONT) / 2.0;

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

impl Demo for CornellScene {
    fn create() -> Serialization {
        let (resolution, config, integrator, sampler, mut scene) = Self::empty();

        Wall::list().iter().for_each(|wall| {
            scene.add(create_wall(wall));
        });

        // scene.add(create_sphere());
        scene.add(create_dragon());
        // scene.add(create_bunny());
        // scene.add(create_biconvex_lens());
        // scene.add(create_bubble());
        scene.add(create_emitter());
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

fn create_camera(resolution: UVec2) -> Box<dyn Camera> {
    let position = Vector3::new(X_CENTER, Y_CENTER, FRONT + DIMENSION / 2.0);
    let target = Vector3::new(X_CENTER, Y_CENTER, Z_CENTER);

    let camera = PerspectiveCamera::new(
        CameraSampler::Random,
        position,
        target,
        Vector3::unit_y(),
        FOVY,
        resolution,
    );

    Box::new(camera)
}

fn create_dragon() -> SceneObject {
    let mut dragon = Mesh::load("./meshes/dragon_4.obj", ShadingMode::Flat);

    // scaling the dragon to (DIMENSION / 1.5)
    let scale = Vector3::broadcast(DIMENSION / 1.5) / dragon.bounds().size();
    let scale = Vector3::broadcast(scale.component_min());
    dragon.scale(scale);
    println!("Scale: {:?}", scale);

    // rotate dragon
    let rotation = Rotation3::from_rotation_xz(FRAC_PI_2 as Float);
    dragon.rotate(Rotation3::from_rotation_xz(FRAC_PI_2 as Float));
    println!("Rotation: {:?}", rotation);

    // translate dragon to center
    let bounds = dragon.bounds();
    let center = bounds.center();
    let center_floor = Vector3::new(center.x, bounds.min.y, center.z);
    let translation = Vector3::new(X_CENTER, FLOOR + 0.01, Z_CENTER) - center_floor;
    dragon.translate(translation);
    println!("Translation: {:?}", translation);

    dragon.build_bvh();

    let specular = FresnelSpecular::new(
        Spectrum::broadcast(1.0),
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Sapphire,
    );

    let microfacet = MicrofacetReflection::new(
        Spectrum::from(Colors::Cyan),
        Box::new(BeckmannDistribution::new(0.25, 0.25, false)),
        FresnelType::Dielectric(FresnelDielectric::new(
            RefractiveType::Air,
            RefractiveType::Sapphire,
        )),
    );

    let bsdf = BSDF::new(vec![Box::new(specular), Box::new(microfacet)]);
    let geometry = Box::new(dragon);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));

    SceneObject::Receiver(receiver)
}

fn create_bunny() -> SceneObject {
    let mut bunny = Mesh::load("./meshes/bunny.obj", ShadingMode::Flat);

    // scaling the bunny to (DIMENSION / 1.5)
    let scale = Vector3::broadcast(DIMENSION / 1.5) / bunny.bounds().size();
    let scale = Vector3::broadcast(scale.component_min());
    bunny.scale(scale);
    println!("Scale: {:?}", scale);

    // rotate bunny
    let rotation = Rotation3::from_rotation_xz(FRAC_PI_2 as Float);
    bunny.rotate(Rotation3::from_rotation_xz(FRAC_PI_2 as Float));
    println!("Rotation: {:?}", rotation);

    // translate bunny to center
    let bounds = bunny.bounds();
    let center = bounds.center();
    let center_floor = Vector3::new(center.x, bounds.min.y, center.z);
    let translation = Vector3::new(X_CENTER, FLOOR + 0.001, Z_CENTER) - center_floor;
    bunny.translate(translation);
    println!("Translation: {:?}", translation);

    bunny.build_bvh();

    let specular = FresnelSpecular::new(
        Spectrum::broadcast(1.0),
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Sapphire,
    );

    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);
    let geometry = Box::new(bunny);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));

    SceneObject::Receiver(receiver)
}

fn create_sphere() -> SceneObject {
    let center = Vector3::new(X_CENTER, Y_CENTER, Z_CENTER);
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
    let center = Vector3::new(X_CENTER, Y_CENTER, Z_CENTER);
    let offset = Vector3::unit_y() * (THICKNESS / 3.0);
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
    let center0 = Vector3::new(X_CENTER, Y_CENTER + radius - dist2, Z_CENTER);
    let center1 = Vector3::new(X_CENTER, Y_CENTER - radius + dist2, Z_CENTER);
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
        Wall::Top => Vector3::new(LEFT_WALL - THICKNESS, CEILING, BACK_WALL - THICKNESS),
        Wall::Left | Wall::Back | Wall::Bottom => Vector3::new(
            LEFT_WALL - THICKNESS,
            FLOOR - THICKNESS,
            BACK_WALL - THICKNESS,
        ),
        Wall::Right => Vector3::new(RIGHT_WALL, FLOOR - THICKNESS, BACK_WALL - THICKNESS),
    };
    let max = match wall {
        Wall::Top => Vector3::new(RIGHT_WALL + THICKNESS, CEILING + THICKNESS, FRONT),
        Wall::Left => Vector3::new(LEFT_WALL, CEILING + THICKNESS, FRONT),
        Wall::Back => Vector3::new(RIGHT_WALL + THICKNESS, CEILING + THICKNESS, BACK_WALL),
        Wall::Right => Vector3::new(RIGHT_WALL + THICKNESS, CEILING + THICKNESS, FRONT),
        Wall::Bottom => Vector3::new(RIGHT_WALL + THICKNESS, FLOOR, FRONT),
    };
    let cube = Aabb::new(min, max);

    let spectrum = match wall {
        Wall::Top | Wall::Back | Wall::Bottom => Spectrum::from(Colors::White),
        Wall::Left => Spectrum::from(Colors::Red),
        Wall::Right => Spectrum::from(Colors::Green),
    };

    let oren_nayar = OrenNayar::new(spectrum, SIGMA);
    let bxdf = Box::new(oren_nayar);

    let bsdf = BSDF::new(vec![bxdf]);
    let geometry = Box::new(cube);

    let receiver = Arc::new(Receiver::new(geometry, bsdf));
    SceneObject::Receiver(receiver)
}

fn create_emitter() -> SceneObject {
    let center = Vector3::new(X_CENTER, CEILING, Z_CENTER);
    let sphere = Sphere::new(center, RADIUS);

    let bsdf = BSDF::empty();
    let emission = Spectrum::from(Colors::White);
    let geometry = Box::new(sphere);

    let emitter = Arc::new(Emitter::new(geometry, bsdf, emission));
    SceneObject::Emitter(emitter)
}
