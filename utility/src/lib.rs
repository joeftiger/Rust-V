pub mod floats;
pub mod math;

#[cfg(not(feature = "f64"))]
type Float = f32;
#[cfg(feature = "f64")]
type Float = f64;
