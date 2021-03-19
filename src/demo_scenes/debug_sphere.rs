#![allow(dead_code)]

use crate::bxdf::{BSDFType, FresnelSpecular, LambertianReflection, BSDF};
use crate::camera::{Camera, PerspectiveCamera};
use crate::demo_scenes::{DemoScene, FOVY};
use crate::objects::{Emitter, Receiver, SceneObject};
use crate::refractive_index::RefractiveType;
use crate::scene::Scene;
use crate::Spectrum;
use color::{Color, Colors};
use geometry::{Aabb, Boundable, Cylinder, Mesh, Point, ShadingMode, Sphere};
use std::sync::Arc;
use ultraviolet::{UVec2, Vec3};

pub struct DebugSphereScene;

impl DemoScene for DebugSphereScene {
    fn create(resolution: UVec2) -> (Scene, Arc<dyn Camera>) {
        let mut scene = Scene::default();

        scene.add(ground()).add(prism());

        // light
        scene.add(light_bulb());
        // scene.add(global_light());

        let camera = create_camera(resolution);

        (scene, camera)
    }
}

//noinspection DuplicatedCode
fn ground() -> SceneObject {
    let min = Vec3::new(-100.0, -5.0, -100.0);
    let max = Vec3::new(100.0, 0.0, 100.0);
    let cube = Aabb::new(min, max);

    let lambertian = LambertianReflection::new(Spectrum::new_const(1.0));
    let bsdf = BSDF::new(vec![BSDFType::LReflection(lambertian)]);

    let receiver = Arc::new(Receiver::new(cube, bsdf));

    SceneObject::Receiver(receiver)
}

fn prism() -> SceneObject {
    let file_name = "./meshes/prism.obj";
    let (model, _) = tobj::load_obj(file_name, true).expect("Could not load bunny file");

    let mut prism = Mesh::load(&model[0].mesh, ShadingMode::Flat);

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
        RefractiveType::Glass,
    );

    let bsdf = BSDF::new(vec![BSDFType::SFresnel(specular)]);

    let receiver = Arc::new(Receiver::new(prism, bsdf));
    SceneObject::Receiver(receiver)
}

fn light_bulb() -> SceneObject {
    let center = Vec3::new(-4.0, 1.5, 0.0);
    let light_bulb = Sphere::new(center, 0.5);

    let bsdf = BSDF::empty();

    let emitter = Arc::new(Emitter::new(light_bulb, bsdf, Spectrum::white() * 10.0));
    SceneObject::Emitter(emitter)
}

fn light_bulb_rectifier() -> SceneObject {
    let position = Vec3::new(-4.0, 1.5, 0.0);
    let left_end = position + Vec3::new(-2.0, 0.5, 0.0);
    let right_end = position + Vec3::new(2.0, -0.5, 0.0);

    let rectifier = Cylinder::new((left_end, right_end), 0.51);

    let bsdf = BSDF::new(vec![BSDFType::LReflection(LambertianReflection::new(
        Spectrum::grey(),
    ))]);

    let receiver = Arc::new(Receiver::new(rectifier, bsdf));
    SceneObject::Receiver(receiver)
}

fn global_light() -> SceneObject {
    let point = Point(Vec3::unit_y() * 100.0);
    let bsdf = BSDF::empty();

    let emitter = Arc::new(Emitter::new(point, bsdf, Spectrum::white() * 0.01));
    SceneObject::Emitter(emitter)
}

//noinspection DuplicatedCode
fn create_camera(resolution: UVec2) -> Arc<dyn Camera> {
    let position = Vec3::new(0.0, 3.0, 10.0);
    let target = Vec3::new(0.0, 0.0, 0.0);

    let camera = PerspectiveCamera::new(position, target, Vec3::unit_y(), FOVY / 2.0, resolution);
    Arc::new(camera)
}
