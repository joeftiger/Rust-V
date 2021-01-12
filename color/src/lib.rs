use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use image::Rgb;
use std::fmt::Debug;
use ultraviolet::{Mat3, Vec3};
use utility::floats;

pub mod cie;
mod spectrum;
mod srgb;
mod xyz;

pub use spectrum::Spectrum;
pub use srgb::Srgb;
pub use xyz::Xyz;

#[macro_export]
macro_rules! color {
    ($($name:ident => $storage:ident, $mul:ident, $size:expr), +) => {
        $(
            #[derive(Clone, Copy, Debug)]
            pub struct $name {
                data: [$storage; $size],
            }

            impl $name {
                pub fn new(data: [$storage; $size]) -> Self {
                    debug_assert!(data.iter().all(|f| !f.is_nan()));
                    Self { data }
                }

                pub fn new_const(data: $storage) -> Self {
                    Self::new([data; $size])
                }

                pub fn len(&self) -> usize {
                    self.data.len()
                }

                pub fn sqrt(&self) -> Self {
                    let mut data = self.data;
                    data.iter_mut().for_each(|f| *f = f.sqrt());

                    Self::new(data)
                }

                pub fn lerp(&self, other: &Self, t: f32) -> Self {
                    *self * (1.0 - t) + *other * t
                }

                /// Clamps the color values between min and max.
                pub fn clamp(&self, min: f32, max: f32) -> Self {
                    let mut data = self.data;
                    floats::fast_clamp_ar(&mut data, min, max);

                    Self::new(data)
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

            impl Mul<$mul> for $name {
                type Output = Self;

                fn mul(self, rhs: $mul) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] *= rhs;
                    }

                    Self::new(data)
                }
            }

            impl MulAssign<$mul> for $name {
                fn mul_assign(&mut self, rhs: $mul) {
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

            impl Div<$mul> for $name {
                type Output = Self;

                fn div(self, rhs: $mul) -> Self::Output {
                    let mut data = self.data;
                    for i in 0..data.len() {
                        data[i] /= rhs;
                    }

                    Self::new(data)
                }
            }

            impl DivAssign<$mul> for $name {
                fn div_assign(&mut self, rhs: $mul) {
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

            impl Into<$name> for f32 {
                fn into(self) -> $name {
                    $name::new_const(self)
                }
            }
        )+
    }
}

/// # Summary
/// A trait for colors. Allows arithmetic operations to be performed and gives utility functions
/// like `is_black()` or `white()`.
pub trait Color:
    Add
    + AddAssign
    + Sub
    + SubAssign
    + Mul
    + MulAssign
    + Mul<f32>
    + MulAssign<f32>
    + Div
    + DivAssign
    + Div<f32>
    + DivAssign<f32>
    + PartialEq
    + Index<usize>
    + IndexMut<usize>
    + Debug
    + Into<Rgb<u8>>
    + Into<Rgb<u16>>
    + Sum
{
    /// # Summary
    /// Returns whether this color is black.
    ///
    /// # Returns
    /// * Whether this color is black
    fn is_black(&self) -> bool;

    /// Clamps the color values between min and max.
    /// # Summary
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
    fn clamp(&self, min: f32, max: f32) -> Self;

    /// # Summary
    /// Returns whether any value in this color is `NaN`.
    ///
    /// # Returns
    /// * Whether this color has `NaN`s
    fn has_nans(&self) -> bool;

    /// # Summary
    /// Returns the square-root of this color.
    ///
    /// # Returns
    /// * Square-rooted self
    fn sqrt(&self) -> Self;

    /// # Summary
    /// Converts this color to `Srgb` space.
    ///
    /// # Returns
    /// * Self as `Srgb`
    fn to_rgb(&self) -> Srgb;

    /// # Summary
    /// Converts this color to `Srgb` space.
    ///
    /// # Returns
    /// * Self as `Srgb`
    fn to_xyz(&self) -> Xyz {
        self.to_rgb().to_xyz()
    }

    /// # Summary
    /// Returns black of this color space.
    ///
    /// # Returns
    /// * Black
    fn black() -> Self;

    /// # Summary
    /// Returns white of this color space.
    ///
    /// # Returns
    /// * White
    fn white() -> Self;

    /// # Summary
    /// Returns red of this color space.
    ///
    /// # Returns
    /// * Red
    fn red() -> Self;

    /// # Summary
    /// Returns greeen of this color space.
    ///
    /// # Returns
    /// * Green
    fn green() -> Self;

    /// # Summary
    /// Returns blue of this color space.
    ///
    /// # Returns
    /// * Blue
    fn blue() -> Self;
}

/// # Summary
/// Returns the matrix to convert `Xyz` to `Srgb`.
///
/// # Returns
/// * Conversion matrix
#[allow(clippy::excessive_precision)]
pub fn xyz_to_srgb_mat() -> Mat3 {
    // https://en.wikipedia.org/wiki/SRGB#The_forward_transformation_(CIE_XYZ_to_sRGB)
    Mat3::new(
        Vec3::new(3.240_97, -0.96924364, 0.05563008),
        Vec3::new(-1.53738318, 1.8759675, -0.20397696),
        Vec3::new(-0.49861076, 0.04155506, 1.05697151),
    )
}


/// # Summary
/// Returns the matrix to convert `Srgb` to `Xyz`.
///
/// # Returns
/// * Conversion matrix
#[allow(clippy::excessive_precision)]
pub fn srgb_to_xyz_mat() -> Mat3 {
    // https://en.wikipedia.org/wiki/SRGB#The_reverse_transformation
    Mat3::new(
        Vec3::new(0.41239080, 0.21263901, 0.01933082),
        Vec3::new(0.35758434, 0.71516868, 0.07219232),
        Vec3::new(0.18048079, 0.07219232, 0.95053215),
    )
}

/// # Summary
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
pub fn srgb_to_linear(val: f32) -> f32 {
    assert!(val >= 0.0);
    assert!(val <= 1.0);
    // https://entropymine.com/imageworsener/srgbformula/
    if val <= 0.0404482362771082 {
        val / 12.92
    } else {
        ((val + 0.055) / 1.055).powf(2.4)
    }
}

/// # Summary
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
pub fn srgbs_to_linear(val: Vec3) -> Vec3 {
    val.map(srgb_to_linear)
}

/// # Summary
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
pub fn linear_to_srgb(val: f32) -> f32 {
    assert!(val >= 0.0);
    assert!(val <= 1.0);
    // https://entropymine.com/imageworsener/srgbformula/
    if val <= 0.00313066844250063 {
        val * 12.92
    } else {
        1.055 * val.powf(1.0 / 2.4) - 0.055
    }
}

/// # Summary
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
pub fn linears_to_srgb(val: Vec3) -> Vec3 {
    val.map(linear_to_srgb)
}
