use crate::sensor::bounds::UBounds2;
use crate::sensor::pixel::Pixel;
use ultraviolet::UVec2;

pub struct SensorTile {
    pub bounds: UBounds2,
    pub pixels: Vec<Pixel>,
}

impl SensorTile {
    pub fn new(bounds: UBounds2) -> Self {
        let mut pixels = Vec::with_capacity(bounds.area());
        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
                let position = UVec2::new(x, y);
                pixels.push(Pixel::new(position));
            }
        }

        Self { bounds, pixels }
    }
}
