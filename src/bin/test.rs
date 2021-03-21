use ron::from_str;
use ron::ser::{to_string_pretty, PrettyConfig};
use rust_v::demo_scenes::{CornellScene, DebugSphereScene, DemoScene};
use rust_v::scene::Scene;
use ultraviolet::UVec2;

fn main() {
    let scene = DebugSphereScene::create(UVec2::broadcast(256));

    let config = PrettyConfig::default();

    let serialization = to_string_pretty(&scene, config).expect("Could not serialize to RON");

    std::fs::write("./prism.ron", serialization.clone()).expect("Could not write to RON file");

    let mut deserialized_scene =
        from_str::<Scene>(serialization.as_str()).expect("Could not deserialize from RON");
    assert_eq!(0, deserialized_scene.lights.len());

    deserialized_scene.collect_emitters();
    assert_eq!(1, deserialized_scene.lights.len());
}
