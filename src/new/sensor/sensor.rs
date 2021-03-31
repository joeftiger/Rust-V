use crate::filters::Filter;
use crate::Spectrum;
use color::{Color, IntSpectrum};
use ultraviolet::UVec2;
use crate::new::sensor::pixel::Pixel;
use crate::new::sensor::bounds::UBounds2;

pub struct Sensor<T> {
    pub resolution: UVec2,
    pub diagonal: f32,
    pub filter: Box<dyn Filter>,
    pub filename: String,
    pub pixel_bounds: UBounds2,
    pixels: Vec<Pixel<T>>,
}

impl<T> Sensor<T> {
    // pub fn new(resolution: UVec2, filter: Box<dyn Filter>, pixel_bounds: Bounds) ->
}
