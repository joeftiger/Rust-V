#![allow(unused_imports)]

use crate::demo_scenes::{CornellScene, Demo, PrismScene};
use color::Colors;
use ron::ser::{to_string_pretty, PrettyConfig};
use rust_v::Spectrum;
use ultraviolet::UVec2;

mod demo_scenes;

fn main() {
    let serialization = CornellScene::create();

    let serialization = to_string_pretty(&serialization, PrettyConfig::default())
        .expect("Could not serialize to RON");

    std::fs::write("./scenes/dragon_4.ron", serialization).expect("Could not write to RON file");
}
