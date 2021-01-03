use std::ops::Range;
use ultraviolet::UVec2;

/// # Summary
/// A block structure in 2D describing ranges of `u32`.
pub struct GridBlock {
    pub x: Range<u32>,
    pub y: Range<u32>,
}

impl GridBlock {
    /// # Summary
    /// Creates a new block.
    ///
    /// # Arguments
    /// * `x` - The range in `x` direction
    /// * `y` - The range in `y` direction
    ///
    /// # Returns
    /// * Self
    pub fn new(x: Range<u32>, y: Range<u32>) -> Self {
        Self { x, y }
    }

    /// # Summary
    /// Applies the `prod` function to this block, merging `x` and `y` ranges into all possible
    /// `UVec2` vectors.
    ///
    /// # Returns
    /// * The range product
    pub fn prod(&self) -> Vec<UVec2> {
        self.x
            .clone()
            .flat_map(|x| self.y.clone().map(move |y| UVec2::new(x, y)))
            .collect()
    }
}

/// # Summary
/// A grid contains blocks with ranges in 2D.
/// The blocks are sorted the following way:
/// * left to right
/// * top to bottom
///
/// Edge blocks might have a smaller size if the original width / height were not a multiple of
/// the original block size.
pub struct Grid {
    pub blocks: Vec<GridBlock>,
}

impl Grid {
    /// # Summary
    /// Creates a new grid.
    ///
    /// # Arguments
    /// * `width` - The width of the 2D structure
    /// * `height` - The height of the 2D structure
    /// * `block_size` - The block size to partition the grid with
    ///
    /// # Returns
    /// * Self
    pub fn new(width: u32, height: u32, block_size: u32) -> Self {
        let x_blocks = (width as f32 / block_size as f32).ceil() as u32;
        let y_blocks = (height as f32 / block_size as f32).ceil() as u32;

        let mut blocks = Vec::with_capacity((x_blocks * y_blocks) as usize);

        for y in 0..y_blocks {
            let y_start = y * block_size;
            let y_end = y_start + block_border_size;
            let y_range = y_start..(y_max.min(y_end));

            for x in 0..x_blocks {
                let x_start = x * block_size;
                let y_start = y * block_size;

                let x_end = x_start + block_border_size;
                let y_end = y_start + block_border_size;

                let x_range = x_start..(x_max.min(x_end));
                let y_range = y_start..(y_max.min(y_end));

                blocks.push(GridBlock::new(x_range, y_range));
            }
        }

        Self { blocks }
    }
}
