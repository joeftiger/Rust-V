use ultraviolet::UVec2;

pub mod bounds;
pub mod pixel;
pub mod sensor_tile;

pub fn vec2_to_index(v: UVec2, height: usize) -> usize {
    v.x as usize + v.y as usize * height
}

pub fn index_to_vec2(index: usize, height: usize) -> UVec2 {
    let y = index / height;
    let x = index % height;

    UVec2::new(x as u32, y as u32)
}

use crate::new::sensor::bounds::UBounds2;
use crate::new::sensor::sensor_tile::SensorTile;
use definitions::Float;
use std::sync::Mutex;

pub struct Sensor {
    pub resolution: UVec2,
    // pub filter: Box<dyn Filter>,
    pub filename: Option<String>,
    pub bounds: UBounds2,
    pub block_size: UVec2,
    pub tiles: Vec<Mutex<SensorTile>>,
}

impl Sensor {
    pub fn new(
        resolution: UVec2,
        // filter: Box<dyn Filter>,
        filename: Option<String>,
        bounds: UBounds2,
        block_size: UVec2,
    ) -> Self {
        Self {
            resolution,
            // filter,
            filename,
            bounds,
            block_size,
            tiles: Self::create_tiles(bounds, block_size),
        }
    }

    fn create_tiles(bounds: UBounds2, block_size: UVec2) -> Vec<Mutex<SensorTile>> {
        let range = bounds.to_range();
        let width = range.x;
        let height = range.y;

        let horizontal = (width as Float / block_size.x as Float).ceil() as u32;
        let vertical = (height as Float / block_size.y as Float).ceil() as u32;

        let mut sensor_tiles = Vec::with_capacity((horizontal * vertical) as usize);

        for y in 0..vertical {
            for x in 0..horizontal {
                let min = UVec2::new(x * block_size.x, y * block_size.y);
                let max = UVec2::new(
                    width.min(min.x + block_size.x),
                    height.min(min.y + block_size.y),
                );
                let bounds = UBounds2::new(min, max);
                let tile = SensorTile::new(bounds);

                sensor_tiles.push(Mutex::new(tile));
            }
        }

        sensor_tiles
    }

    pub fn num_tiles(&self) -> usize {
        self.tiles.len()
    }
}
