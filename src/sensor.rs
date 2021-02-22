use crate::filters::Filter;
use std::sync::Arc;
use ultraviolet::{IVec2, UVec2, Vec3};

struct Pixel {
    xyz: Vec3,
    filter_weight_sum: f32,
    splat_xyz: Vec3,
}

pub struct Sensor {
    pub resolution: UVec2,
    pub diagonal: f32,
    pub filter: Arc<dyn Filter>,
    pub filename: String,
    pub pixel_bounds: IVec2,
    pixels: Vec<Pixel>,
}

impl Sensor {}

struct SensorTile {}
