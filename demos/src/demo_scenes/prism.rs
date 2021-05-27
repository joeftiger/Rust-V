#![allow(dead_code)]
use crate::demo_scenes::{Demo, Vector3, FOVY, SIGMA};
use crate::*;
use color::{AsColor, Color, Colors};
use geometry::{Aabb, Boundable, Cylinder, Mesh, Point, ShadingMode, Sphere};
use rust_v::bxdf::{FresnelSpecular, LambertianReflection, OrenNayar, BSDF};
use rust_v::camera::{Camera, PerspectiveCamera};
use rust_v::objects::{Emitter, Receiver, SceneObject};
use rust_v::refractive_index::RefractiveType;
use rust_v::samplers::camera::CameraSampler;
use rust_v::scene::Scene;
use rust_v::serialization::Serialization;
use rust_v::Spectrum;
#[cfg(not(feature = "f64"))]
use std::f32::consts::FRAC_PI_2;
#[cfg(feature = "f64")]
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use ultraviolet::UVec2;

pub struct PrismScene;

impl Demo for PrismScene {
    fn create() -> Serialization {
        let (resolution, config, integrator, sampler, mut scene) = Self::empty();

        scene.add(ground()).add(prism());

        // light
        scene.add(light_bulb()); //.add(light_bulb_rectifier());
                                 // scene.add(global_light());

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

//noinspection DuplicatedCode
fn ground() -> SceneObject {
    let min = Vector3::new(-100.0, -5.0, -100.0);
    let max = Vector3::new(100.0, 0.0, 100.0);
    let cube = Aabb::new(min, max);

    let oren_nayar = OrenNayar::new(Spectrum::from(Colors::White), SIGMA);
    let bxdf = Box::new(oren_nayar);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(cube), bsdf));

    SceneObject::Receiver(receiver)
}

fn prism() -> SceneObject {
    let mut prism = Mesh::load("./meshes/prism.obj".to_string(), ShadingMode::Flat);

    {
        let bounds = prism.bounds();
        let unit_scale = Vector3::broadcast(2.0) / bounds.size();
        prism.scale(unit_scale);
    }
    {
        prism.rotate(Rotation3::from_rotation_yz(-FRAC_PI_2));
    }
    {
        let center = prism.bounds().center();
        let translation = Vector3::unit_y() - center;
        prism.translate(translation);
    }

    prism.build_bvh();

    let specular = FresnelSpecular::new(
        Spectrum::broadcast(1.0),
        Spectrum::broadcast(1.0),
        RefractiveType::Air,
        RefractiveType::Sapphire,
    );
    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(prism), bsdf));
    SceneObject::Receiver(receiver)
}

fn light_bulb() -> SceneObject {
    let center = Vector3::new(-2.0, 1.5, 0.0);
    let light_bulb = Sphere::new(center, 0.5);

    let bsdf = BSDF::empty();

    let emitter = Arc::new(Emitter::new(
        Box::new(light_bulb),
        bsdf,
        Spectrum::from(Colors::White) * 2.0,
    ));
    SceneObject::Emitter(emitter)
}

fn light_bulb_rectifier() -> SceneObject {
    let position = Vector3::new(-4.0, 1.5, 0.0);
    let left_end = position + Vector3::new(-2.0, 0.5, 0.0);
    let right_end = position + Vector3::new(2.0, -0.5, 0.0);

    let rectifier = Cylinder::new((left_end, right_end), 0.51);
    let lambertian = LambertianReflection::new(Spectrum::from(Colors::Grey2));
    let bxdf = Box::new(lambertian);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(rectifier), bsdf));
    SceneObject::Receiver(receiver)
}

fn global_light() -> SceneObject {
    let point = Point(Vector3::unit_y() * 100.0);
    let bsdf = BSDF::empty();

    let emitter = Arc::new(Emitter::new(
        Box::new(point),
        bsdf,
        Spectrum::from(Colors::White) * 0.01,
    ));
    SceneObject::Emitter(emitter)
}

//noinspection DuplicatedCode
fn create_camera(resolution: UVec2) -> Arc<dyn Camera> {
    let position = Vector3::new(0.0, 3.0, 10.0);
    let target = Vector3::new(0.0, 0.0, 0.0);

    let camera = PerspectiveCamera::new(
        CameraSampler::Random,
        position,
        target,
        Vector3::unit_y(),
        FOVY / 2.0,
        resolution,
    );

    Arc::new(camera)
}
