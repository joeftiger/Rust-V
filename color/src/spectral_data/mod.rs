use std::ops::RangeInclusive;

pub mod blue;
pub mod blue_sky;
pub mod dark_skin;
pub mod foliage;
pub mod green;
pub mod light_skin;
pub mod red;
pub mod white;
pub mod yellow;

pub const LAMBDA_START: f32 = 0.38;
pub const LAMBDA_END: f32 = 0.73;
pub const LAMBDA_RANGE: f32 = LAMBDA_END - LAMBDA_START;
pub const LAMBDA_STEP: usize = 10;
pub const LAMBDA_NUM: usize = 36; //((LAMBDA_END - LAMBDA_START) / LAMBDA_STEP as f32 + 0.1) as u16;

pub const fn lambda_range() -> RangeInclusive<f32> {
    LAMBDA_START..=LAMBDA_END
}
