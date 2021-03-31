#![allow(dead_code)]
use rust_v::Spectrum;
use rust_v::bxdf::{BSDF, LambertianReflection, OrenNayar, FresnelSpecular};
use rust_v::refractive_index::RefractiveType;
use rust_v::objects::{SceneObject, Emitter, Receiver};
use rust_v::scene::Scene;
use rust_v::camera::{Camera, PerspectiveCamera};
use rust_v::sampler::pixel_samplers::PixelSamplerType;
use color::{Color, Colors};
use geometry::{Aabb, Boundable, Cylinder, Mesh, Point, ShadingMode, Sphere};
use std::sync::{Arc, Mutex};
use ultraviolet::{UVec2, Vec3};
use crate::demo_scenes::{DemoScene, SIGMA, FOVY};

pub struct PrismScene;

impl DemoScene for PrismScene {
    fn create(resolution: UVec2) -> Scene {
        let mut scene = Scene::default();

        scene.add(ground()).add(prism());

        // light
        scene.add(light_bulb()); //.add(light_bulb_rectifier());
                                 // scene.add(global_light());

        scene.camera = Mutex::new(create_camera(resolution));

        scene
    }
}

//noinspection DuplicatedCode
fn ground() -> SceneObject {
    let min = Vec3::new(-100.0, -5.0, -100.0);
    let max = Vec3::new(100.0, 0.0, 100.0);
    let cube = Aabb::new(min, max);

    let oren_nayar = OrenNayar::new(Spectrum::white(), SIGMA);
    let bxdf = Box::new(oren_nayar);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(cube), bsdf));

    SceneObject::Receiver(receiver)
}

fn prism() -> SceneObject {
    let mut prism = Mesh::load("./meshes/prism.obj".to_string(), ShadingMode::Flat);

    {
        let bounds = prism.bounds();
        let unit_scale = Vec3::broadcast(2.0) / bounds.size();
        prism.scale(unit_scale);
    }
    {
        let center = prism.bounds().center();
        let translation = Vec3::unit_y() - center;
        prism.translate(translation);
    }

    prism.build_bvh();

    let specular = FresnelSpecular::new(
        Spectrum::new_const(1.0),
        Spectrum::new_const(1.0),
        RefractiveType::Air,
        RefractiveType::Sapphire,
    );
    let bxdf = Box::new(specular);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(prism), bsdf));
    SceneObject::Receiver(receiver)
}

fn light_bulb() -> SceneObject {
    let center = Vec3::new(-2.0, 1.5, 0.0);
    let light_bulb = Sphere::new(center, 0.5);

    let bsdf = BSDF::empty();

    let emitter = Arc::new(Emitter::new(
        Box::new(light_bulb),
        bsdf,
        Spectrum::white() * 2.0,
    ));
    SceneObject::Emitter(emitter)
}

fn light_bulb_rectifier() -> SceneObject {
    let position = Vec3::new(-4.0, 1.5, 0.0);
    let left_end = position + Vec3::new(-2.0, 0.5, 0.0);
    let right_end = position + Vec3::new(2.0, -0.5, 0.0);

    let rectifier = Cylinder::new((left_end, right_end), 0.51);
    let lambertian = LambertianReflection::new(Spectrum::grey());
    let bxdf = Box::new(lambertian);

    let bsdf = BSDF::new(vec![bxdf]);

    let receiver = Arc::new(Receiver::new(Box::new(rectifier), bsdf));
    SceneObject::Receiver(receiver)
}

fn global_light() -> SceneObject {
    let point = Point(Vec3::unit_y() * 100.0);
    let bsdf = BSDF::empty();

    let emitter = Arc::new(Emitter::new(
        Box::new(point),
        bsdf,
        Spectrum::white() * 0.01,
    ));
    SceneObject::Emitter(emitter)
}

//noinspection DuplicatedCode
fn create_camera(resolution: UVec2) -> Box<dyn Camera> {
    let position = Vec3::new(0.0, 3.0, 10.0);
    let target = Vec3::new(0.0, 0.0, 0.0);

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
