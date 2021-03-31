use crate::demo_scenes::{CornellScene, DemoScene, PrismScene};
use ron::ser::{to_string_pretty, PrettyConfig};
use ultraviolet::UVec2;

mod demo_scenes;

fn main() {
    let scene = CornellScene::create(UVec2::broadcast(2000));

    let serialization =
        to_string_pretty(&scene, PrettyConfig::default()).expect("Could not serialize to RON");

    std::fs::write("./cornell.ron", serialization).expect("Could not write to RON file");
}
