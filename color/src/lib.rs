use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use serde_big_array::big_array;
big_array! {
    ColorArray;
    +3, LAMBDA_NUM
}

pub mod cie;
mod int_spectrum;
pub mod spectral_data;
mod spectrum;
mod srgb;
mod xyz;

pub use int_spectrum::*;
pub use spectrum::*;
pub use srgb::*;
pub use xyz::*;

use crate::spectral_data::LAMBDA_NUM;
use definitions::{Float, Matrix3, Vector3};
use spectral_data::{LAMBDA_END, LAMBDA_START};
use utility::floats::FloatExt;

#[macro_export]
macro_rules! color {
    ($($name:ident => $storage:ident, $size:expr), +) => {
        $(
            #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
            pub struct $name {
                #[serde(with = "ColorArray")]
                data: [$storage; $size],
            }

            impl $name {
                pub fn new(data: [$storage; $size]) -> Self {
                    debug_assert!(data.iter().all(|f| !f.is_nan()));
                    Self { data }
                }

                pub const fn size() -> usize {
                    $size
                }

                pub fn as_light_waves(&self) -> [LightWave; $size] {
                    let mut light_waves = [LightWave::default(); $size];

                    for i in 0..$size {
                        light_waves[i].lambda = LAMBDA_START.lerp(LAMBDA_END, i as Float / $size as Float);
                        light_waves[i].intensity = self[i];
                    }

                    light_waves
                }
            }

            impl Color<$storage> for $name {
                fn mul_add(&self, a: Self, b: Self) -> Self {
                    let mut data = self.data.clone();
                    data.iter_mut()
                        .zip(a.data.iter())
                        .zip(b.data.iter())
                        .for_each(|((a, b), c)| *a = a.mul_add(*b, *c));

                    Self::new(data)
                }

                fn mul_add_assign(&mut self, a: Self, b: Self) {
                    self.data.iter_mut()
                        .zip(a.data.iter())
                        .zip(b.data.iter())
                        .for_each(|((a, b), c)| *a = a.mul_add(*b, *c));
                }

                fn add_mul(&self, a: Self, b: Self) -> Self {
                    let mut data = self.data.clone();
                    data.iter_mut()
                        .zip(a.data.iter())
                        .zip(b.data.iter())
                        .for_each(|((a, b), c)| *a = b.mul_add(*c, *a));

                    Self::new(data)
                }

                fn add_mul_assign(&mut self, a: Self, b: Self) {
                    self.data.iter_mut()
                        .zip(a.data.iter())
                        .zip(b.data.iter())
                        .for_each(|((a, b), c)| *a = b.mul_add(*c, *a));
                }

                fn broadcast(value: $storage) -> Self {
                    Self::new([value; $size])
                }

                fn len(&self) -> usize {
                    Self::size()
                }

                fn is_black(&self) -> bool {
                    self.data.iter().all(|value| *value == 0.0)
                }

                fn clamp(&self, min: $storage, max: $storage) -> Self {
                    debug_assert!(min < max);

                    let mut data = self.data;
                    data.iter_mut().for_each(|v| if *v < min {
                        *v = min
                    } else if *v > max {
                        *v = max
                    });

                    Self::new(data)
                }

                fn sqrt(&self) -> Self {
                    let mut data = self.data;
                    data.iter_mut().for_each(|f| *f = f.sqrt());

                    Self::new(data)
                }

                fn lerp(&self, other: &Self, t: $storage) -> Self {
                    let mut data = [Default::default(); $size];

                    for i in 0..$size {
                        data[i] = self.data[i] * (1.0 - t) + other.data[i] * t;
                    }

                    Self::new(data)
                }

                fn component_min(&self) -> $storage {
                    let mut max = $storage::MAX;
                    self.data.iter().for_each(|c| if *c < max { max = *c; });

                    max
                }

                fn component_max(&self) -> $storage {
                    let mut max = $storage::MIN;
                    self.data.iter().for_each(|c| if *c > max { max = *c; });

                    max
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    let data = [$storage::default(); $size];
                    Self::new(data)
                }
            }

            impl Add for $name {
                type Output = Self;

                fn add(self, rhs: Self) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] += rhs.data[i];
                    }

                    Self::new(data)
                }
            }

            impl AddAssign for $name {
                fn add_assign(&mut self, rhs: Self) {
                    for i in 0..self.data.len() {
                        self.data[i] += rhs.data[i];
                    }
                }
            }

            impl Sub for $name {
                type Output = Self;

                fn sub(self, rhs: Self) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] -= rhs.data[i];
                    }

                    Self::new(data)
                }
            }

            impl SubAssign for $name {
                fn sub_assign(&mut self, rhs: Self) {
                    for i in 0..self.data.len() {
                        self.data[i] -= rhs.data[i];
                    }
                }
            }

            impl Mul for $name {
                type Output = Self;

                fn mul(self, rhs: Self) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] *= rhs.data[i];
                    }

                    Self::new(data)
                }
            }

            impl MulAssign for $name {
                fn mul_assign(&mut self, rhs: Self) {
                    for i in 0..self.data.len() {
                        self.data[i] *= rhs.data[i];
                    }
                }
            }

            impl Mul<$storage> for $name {
                type Output = Self;

                fn mul(self, rhs: $storage) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] *= rhs;
                    }

                    Self::new(data)
                }
            }

            impl MulAssign<$storage> for $name {
                fn mul_assign(&mut self, rhs: $storage) {
                    for i in 0..self.data.len() {
                        self.data[i] *=  rhs;
                    }
                }
            }

            impl Div for $name {
                type Output = Self;

                fn div(self, rhs: Self) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] /= rhs.data[i];
                    }

                    Self::new(data)
                }
            }

            impl DivAssign for $name {
                fn div_assign(&mut self, rhs: Self) {
                    for i in 0..self.data.len() {
                        self.data[i] /= rhs.data[i];
                    }
                }
            }

            impl Div<$storage> for $name {
                type Output = Self;

                fn div(self, rhs: $storage) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] /= rhs;
                    }

                    Self::new(data)
                }
            }

            impl DivAssign<$storage> for $name {
                fn div_assign(&mut self, rhs: $storage) {
                    for i in 0..self.data.len() {
                        self.data[i] /= rhs;
                    }
                }
            }

            impl Index<usize> for $name {
                type Output = $storage;

                fn index(&self, index: usize) -> &Self::Output {
                    &self.data[index]
                }
            }

            impl IndexMut<usize> for $name {
                fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                    &mut self.data[index]
                }
            }

            impl PartialEq for $name {
                fn eq(&self, other: &Self) -> bool {
                    self.data.iter().zip(other.data.iter()).all(|(d0, d1)| d0 == d1)
                }
            }

            impl Eq for $name {}

            impl Sum for $name {
                fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                    iter.fold($name::default(), |a, b| a + b)
                }
            }
        )+
    }
}

/// A light wave is described by a wavelength (lambda) and an intensity (associated with amplitude).
#[derive(Copy, Clone, Default)]
pub struct LightWave {
    /// The wavelength in μm.
    pub lambda: Float,
    /// The intensity of the light wave.
    pub intensity: Float,
}

impl LightWave {
    pub fn new(lambda: Float, intensity: Float) -> Self {
        Self { lambda, intensity }
    }

    pub fn with_intensity(&self, intensity: Float) -> Self {
        Self::new(self.lambda, intensity)
    }
}

/// A trait for colors. Allows arithmetic operations to be performed and gives utility functions
/// like `is_black()`.
#[allow(clippy::len_without_is_empty)]
pub trait Color<T = Float>:
    Add
    + AddAssign
    + Sub
    + SubAssign
    + Mul
    + MulAssign
    + Mul<T>
    + MulAssign<T>
    + Div
    + DivAssign
    + Div<T>
    + DivAssign<T>
    + PartialEq
    + Eq
    + Index<usize>
    + IndexMut<usize>
    + Default
    + Debug
    + Sum
{
    /// self * a + b
    fn mul_add(&self, a: Self, b: Self) -> Self;
    /// self * a + b
    fn mul_add_assign(&mut self, a: Self, b: Self);
    /// self + a * b
    fn add_mul(&self, a: Self, b: Self) -> Self;
    /// self + a * b
    fn add_mul_assign(&mut self, a: Self, b: Self);

    /// Creates a new color with the given value assigned on the whole color spectrum.
    ///
    /// # Argumetns
    /// * `value` - The value to assign
    ///
    /// # Returns
    /// * Self
    fn broadcast(value: T) -> Self;

    /// Returns the length (number of entries) of this color.
    ///
    /// # Returns
    /// * The length
    fn len(&self) -> usize;

    /// Returns whether this color is black.
    ///
    /// # Returns
    /// * Whether this color is black
    fn is_black(&self) -> bool;

    /// Clamps the color values between min and max.

    /// Clamps the color values between min and max.
    ///
    /// # Constraints
    /// * `min` - Should be finite (neither infinite nor `NaN`).
    ///           Should be less than `max`.
    /// * `max` - Should be finite.
    ///
    /// # Arguments
    /// * `min` - The minimum for each color value
    /// * `max` - The maximum for each color value
    ///
    /// # Returns
    /// * Clamped self
    fn clamp(&self, min: T, max: T) -> Self;

    /// Returns the square-root of this color.
    ///
    /// # Returns
    /// * Square-rooted self
    fn sqrt(&self) -> Self;

    /// Linearly interpolates this color with the other one by parameter `t`.
    ///
    /// # Constraints
    /// * `t` - Should be in range `[0, 1]`.
    ///
    /// # Arguments
    /// * `other` - The other other to lerp to
    /// * `t` - The interpolation parameter
    ///
    /// # Returns
    /// * Lerped Self
    fn lerp(&self, other: &Self, t: T) -> Self;

    /// Returns the minimum component value.
    ///
    /// # Returns
    /// * Component minimum
    fn component_min(&self) -> T;

    /// Returns the maximum component value.
    ///
    /// # Returns
    /// * Component maximum
    fn component_max(&self) -> T;
}

/// A trait allowing colors to return known colors:
///  * black
///  * grey
///  * white
///  * red
///  * yellow
///  * green
///  * cyan
///  * blue
///  * pink
pub trait Colors<T = Float>: Color<T> {
    fn black() -> Self;

    fn grey() -> Self;

    fn white() -> Self;

    fn red() -> Self;

    fn yellow() -> Self;

    fn green() -> Self;

    fn cyan() -> Self;

    fn blue() -> Self;

    fn pink() -> Self;
}

/// Returns the matrix to convert `Xyz` to `Srgb`.
///
/// # Returns
/// * Conversion matrix
#[allow(clippy::excessive_precision)]
#[inline]
pub fn xyz_to_srgb_mat() -> Matrix3 {
    // https://en.wikipedia.org/wiki/SRGB#The_forward_transformation_(CIE_XYZ_to_sRGB)
    Matrix3::new(
        Vector3::new(3.24096994, -0.96924364, 0.05563008),
        Vector3::new(-1.53738318, 1.8759675, -0.20397696),
        Vector3::new(-0.49861076, 0.04155506, 1.05697151),
    )
}

/// Returns the matrix to convert `Srgb` to `Xyz`.
///
/// # Returns
/// * Conversion matrix
#[allow(clippy::excessive_precision)]
#[inline]
pub fn srgb_to_xyz_mat() -> Matrix3 {
    // https://en.wikipedia.org/wiki/SRGB#The_reverse_transformation
    Matrix3::new(
        Vector3::new(0.41239080, 0.21263901, 0.01933082),
        Vector3::new(0.35758434, 0.71516868, 0.07219232),
        Vector3::new(0.18048079, 0.07219232, 0.95053215),
    )
}

/// Converts an `Srgb` value to linear `Srgb`.
///
/// # Constraints
/// * `val` - Should be within `[0, 1]`.
///
/// # Arguments
/// * `val` - The `Srgb` value
///
/// # Returns
/// * Linear `Srgb` value
#[allow(clippy::excessive_precision)]
#[inline]
pub fn srgb_to_linear(val: Float) -> Float {
    // assert!(val >= 0.0);
    // assert!(val <= 1.0);
    // https://entropymine.com/imageworsener/srgbformula/
    if val <= 0.0404482362771082 {
        val / 12.92
    } else {
        ((val + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts an `Srgb` vector to linear `Srgb`.
///
/// # Constraints
/// * `val` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `val` - The `Srgb` vector
///
/// # Returns
/// * Linear `Srgb` vector
#[inline]
pub fn srgbs_to_linear(val: Vector3) -> Vector3 {
    val.map(srgb_to_linear)
}

/// Converts a linear `Srgb` value to `Srgb`.
///
/// # Constraints
/// * `val` - Should be within `[0, 1]`.
///
/// # Arguments
/// * `val` - The linear `Srgb` value
///
/// # Returns
/// * `Srgb` value
#[allow(clippy::excessive_precision)]
#[inline]
pub fn linear_to_srgb(val: Float) -> Float {
    // assert!(val >= 0.0);
    // assert!(val <= 1.0);
    // https://entropymine.com/imageworsener/srgbformula/
    if val <= 0.00313066844250063 {
        val * 12.92
    } else {
        1.055 * val.powf(1.0 / 2.4) - 0.055
    }
}

/// Converts a linear `Srgb` vector to `Srgb`.
///
/// # Constraints
/// * `val` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `val` - The linear `Srgb` vector
///
/// # Returns
/// * `Srgb` vector
#[inline]
pub fn linears_to_srgb(val: Vector3) -> Vector3 {
    val.map(linear_to_srgb)
}
