use std::ops::Range;
use ultraviolet::UVec2;

/// # Summary
/// A block structure in 2d describing ranges of `T`.
pub struct Block {
    x: Range<u32>,
    y: Range<u32>,
}

impl Block {
    pub fn new(x: Range<u32>, y: Range<u32>) -> Self {
        Self { x, y }
    }

    pub fn prod(&self) -> Vec<UVec2> {
        self.x
            .clone()
            .flat_map(|x| self.y.clone().map(move |y| UVec2::new(x, y)))
            .collect()
    }
}

pub struct Grid {
    blocks: Vec<Grid>,
}
