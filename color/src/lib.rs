use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};
use std::iter::Sum;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_big_array::big_array;

use color_data::{LAMBDA_END, LAMBDA_START};
pub use int_spectrum::*;
pub use spectrum::*;
pub use srgb::*;
use utility::floats::FloatExt;
pub use xyz::*;

use crate::color_data::LAMBDA_NUM;
use serde::de::Error;

pub mod cie;
pub mod color_data;
mod int_spectrum;
mod spectrum;
mod srgb;
mod xyz;

#[cfg(not(feature = "f64"))]
type Float = f32;
#[cfg(feature = "f64")]
type Float = f64;

big_array! {
    SerdeBigArray;
    3, LAMBDA_NUM
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerdeColors {
    Srgb([Float; 3]),
    Xyz([Float; 3]),
    #[serde(with = "SerdeBigArray")]
    Spectrum([Float; 36]),
    Color(Colors),
    Constant(Float),
}

#[macro_export]
macro_rules! color {
    ($name:ident => $size:expr, $path:ident $(::$path2:ident)*) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name {
            pub data: [Float; $size],
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
                S: Serializer {
                for c in &Colors::variants() {
                    if self.eq(&Self::from(*c)) {
                        return SerdeColors::Color(*c).serialize(serializer);
                    }
                }

                let first = self.data[0];
                #[allow(clippy::float_cmp)]
                if self.data.iter().skip(1).all(|v| *v == first) {
                    return SerdeColors::Constant(first).serialize(serializer);
                }

                SerdeColors::$name(self.data).serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
                D: Deserializer<'de> {
                let c = SerdeColors::deserialize(deserializer)?;
                Self::try_from(c.clone()).map_err(|_| D::Error::custom(format!("Unable to parse {} from {:?}", std::any::type_name::<$name>(), c)))
            }
        }

        impl $name {
            pub fn new(data: [Float; $size]) -> Self {
                debug_assert!(data.iter().all(|f| !f.is_nan()));
                Self { data }
            }

            #[inline(always)]
            pub const fn size() -> usize {
                $size
            }

            #[inline(always)]
            pub fn lambda_of_index(index: usize) -> Float {
                LAMBDA_START.lerp(LAMBDA_END, index as Float / $size as Float)
            }

            #[inline]
            pub fn as_light_wave(&self, light_wave_index: usize) -> LightWave {
                let lambda = Self::lambda_of_index(light_wave_index);
                let intensity = self[light_wave_index];

                LightWave {
                    lambda,
                    intensity,
                }
            }

            pub fn as_light_waves(&self) -> [LightWave; $size] {
                let mut light_waves = [LightWave::default(); $size];

                for i in 0..$size {
                    light_waves[i].lambda = Self::lambda_of_index(i);
                    light_waves[i].intensity = self[i]
                }

                light_waves
            }

            pub fn approx_eq(&self, other: &Self) -> bool {
                self.data
                    .iter()
                    .zip(other.data.iter())
                    .all(|(d0, d1)| d0.is_approx_eq_with(*d1, 0.1))
            }
        }

        impl From<Colors> for $name {
            #[rustfmt::skip]
            fn from(c: Colors) -> Self {
                // @formatter:off
                let data = match c {
                    Colors::DarkSkin     => $path $(::$path2)* ::DARK_SKIN,
                    Colors::LightSkin    => $path $(::$path2)* ::LIGHT_SKIN,
                    Colors::BlueSky      => $path $(::$path2)* ::BLUE_SKY,
                    Colors::Foliage      => $path $(::$path2)* ::FOLIAGE,
                    Colors::BlueFlower   => $path $(::$path2)* ::BLUE_FLOWER,
                    Colors::BluishGreen  => $path $(::$path2)* ::BLUISH_GREEN,
                    Colors::Orange       => $path $(::$path2)* ::ORANGE,
                    Colors::PurplishBlue => $path $(::$path2)* ::PURPLISH_BLUE,
                    Colors::ModerateRed  => $path $(::$path2)* ::MODERATE_RED,
                    Colors::Purple       => $path $(::$path2)* ::PURPLE,
                    Colors::YellowGreen  => $path $(::$path2)* ::YELLOW_GREEN,
                    Colors::OrangeYellow => $path $(::$path2)* ::ORANGE_YELLOW,
                    Colors::Blue         => $path $(::$path2)* ::BLUE,
                    Colors::Green        => $path $(::$path2)* ::GREEN,
                    Colors::Red          => $path $(::$path2)* ::RED,
                    Colors::Yellow       => $path $(::$path2)* ::YELLOW,
                    Colors::Magenta      => $path $(::$path2)* ::MAGENTA,
                    Colors::Cyan         => $path $(::$path2)* ::CYAN,
                    Colors::White        => $path $(::$path2)* ::WHITE,
                    Colors::Grey1        => $path $(::$path2)* ::GREY_1,
                    Colors::Grey2        => $path $(::$path2)* ::GREY_2,
                    Colors::Grey3        => $path $(::$path2)* ::GREY_3,
                    Colors::Grey4        => $path $(::$path2)* ::GREY_4,
                    Colors::Black        => $path $(::$path2)* ::BLACK,
                };
                // @formatter:on

                Self::new(data)
            }
        }

        impl Color for $name {
            fn mul_add(&self, a: Self, b: Self) -> Self {
                let mut data = self.data.clone();
                data.iter_mut()
                    .zip(a.data.iter())
                    .zip(b.data.iter())
                    .for_each(|((a, b), c)| *a = a.mul_add(*b, *c));

                Self::new(data)
            }

            fn mul_add_assign(&mut self, a: Self, b: Self) {
                self.data
                    .iter_mut()
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
                self.data
                    .iter_mut()
                    .zip(a.data.iter())
                    .zip(b.data.iter())
                    .for_each(|((a, b), c)| *a = b.mul_add(*c, *a));
            }

            fn broadcast(value: Float) -> Self {
                Self::new([value; $size])
            }

            fn len(&self) -> usize {
                Self::size()
            }

            fn is_black(&self) -> bool {
                self.data.iter().all(|value| *value == 0.0)
            }

            fn clamp(&self, min: Float, max: Float) -> Self {
                debug_assert!(min < max);

                let mut data = self.data;
                data.iter_mut().for_each(|v| {
                    if *v < min {
                        *v = min
                    } else if *v > max {
                        *v = max
                    }
                });

                Self::new(data)
            }

            fn sqrt(&self) -> Self {
                let mut data = self.data;
                data.iter_mut().for_each(|f| *f = f.sqrt());

                Self::new(data)
            }

            fn lerp(&self, other: &Self, t: Float) -> Self {
                let mut data = [Default::default(); $size];

                for i in 0..$size {
                    data[i] = self.data[i] * (1.0 - t) + other.data[i] * t;
                }

                Self::new(data)
            }

            fn component_min(&self) -> Float {
                let mut max = Float::MAX;
                self.data.iter().for_each(|c| {
                    if *c < max {
                        max = *c;
                    }
                });

                max
            }

            fn component_max(&self) -> Float {
                let mut max = Float::MIN;
                self.data.iter().for_each(|c| {
                    if *c > max {
                        max = *c;
                    }
                });

                max
            }
        }

        impl Default for $name {
            fn default() -> Self {
                let data = [Float::default(); $size];
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

        impl Mul<Float> for $name {
            type Output = Self;

            fn mul(self, rhs: Float) -> Self::Output {
                let mut data = self.data;
                for i in 0..data.len() {
                    data[i] *= rhs;
                }

                Self::new(data)
            }
        }

        impl MulAssign<Float> for $name {
            fn mul_assign(&mut self, rhs: Float) {
                for i in 0..self.data.len() {
                    self.data[i] *= rhs;
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

        impl Div<Float> for $name {
            type Output = Self;

            fn div(self, rhs: Float) -> Self::Output {
                let mut data = self.data;
                for i in 0..data.len() {
                    data[i] /= rhs;
                }

                Self::new(data)
            }
        }

        impl DivAssign<Float> for $name {
            fn div_assign(&mut self, rhs: Float) {
                for i in 0..self.data.len() {
                    self.data[i] /= rhs;
                }
            }
        }

        impl Index<usize> for $name {
            type Output = Float;

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
                self.data
                    .iter()
                    .zip(other.data.iter())
                    .all(|(d0, d1)| d0 == d1)
            }
        }

        impl Eq for $name {}

        impl Sum for $name {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold($name::default(), |a, b| a + b)
            }
        }
    };
}

/// A light wave is described by a wavelength (lambda) in `μm` and an intensity (associated with amplitude).
#[derive(Copy, Clone, Default)]
pub struct LightWave {
    /// The wavelength in `μm`.
    pub lambda: Float,
    /// The intensity of the light wave.
    pub intensity: Float,
}

/// A trait for colors. Allows arithmetic operations to be performed and gives utility functions
/// like `is_black()`.
#[allow(clippy::len_without_is_empty)]
pub trait Color:
    Add
    + AddAssign
    + Sub
    + SubAssign
    + Mul
    + MulAssign
    + Mul<Float>
    + MulAssign<Float>
    + Div
    + DivAssign
    + Div<Float>
    + DivAssign<Float>
    + PartialEq
    + Eq
    + Index<usize>
    + IndexMut<usize>
    + Default
    + Debug
    + Sum
    + From<Colors>
    + TryFrom<SerdeColors>
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
    fn broadcast(value: Float) -> Self;

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
    fn clamp(&self, min: Float, max: Float) -> Self;

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
    fn lerp(&self, other: &Self, t: Float) -> Self;

    /// Returns the minimum component value.
    ///
    /// # Returns
    /// * Component minimum
    fn component_min(&self) -> Float;

    /// Returns the maximum component value.
    ///
    /// # Returns
    /// * Component maximum
    fn component_max(&self) -> Float;
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
pub trait AsColor {
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

/// Describes colors.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Colors {
    DarkSkin,
    LightSkin,
    BlueSky,
    Foliage,
    BlueFlower,
    BluishGreen,
    Orange,
    PurplishBlue,
    ModerateRed,
    Purple,
    YellowGreen,
    OrangeYellow,
    Blue,
    Green,
    Red,
    Yellow,
    Magenta,
    Cyan,
    White,
    Grey1,
    Grey2,
    Grey3,
    Grey4,
    Black,
}

impl Colors {
    pub const fn variants() -> [Self; 24] {
        [
            Self::DarkSkin,
            Self::LightSkin,
            Self::BlueSky,
            Self::Foliage,
            Self::BlueFlower,
            Self::BluishGreen,
            Self::Orange,
            Self::PurplishBlue,
            Self::ModerateRed,
            Self::Purple,
            Self::YellowGreen,
            Self::OrangeYellow,
            Self::Blue,
            Self::Green,
            Self::Red,
            Self::Yellow,
            Self::Magenta,
            Self::Cyan,
            Self::White,
            Self::Grey1,
            Self::Grey2,
            Self::Grey3,
            Self::Grey4,
            Self::Black,
        ]
    }
}

impl TryFrom<&str> for Colors {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let c = match value.to_lowercase().as_str() {
            "darkskin" => Self::DarkSkin,
            "lightskin" => Self::LightSkin,
            "bluesky" => Self::BlueSky,
            "foliage" => Self::Foliage,
            "blueflower" => Self::BlueFlower,
            "bluishGreen" => Self::BluishGreen,
            "orange" => Self::Orange,
            "purplishblue" => Self::PurplishBlue,
            "moderatered" => Self::ModerateRed,
            "purple" => Self::Purple,
            "yellowgreen" => Self::YellowGreen,
            "orangeyellow" => Self::OrangeYellow,
            "blue" => Self::Blue,
            "green" => Self::Green,
            "red" => Self::Red,
            "yellow" => Self::Yellow,
            "magenta" => Self::Magenta,
            "cyan" => Self::Cyan,
            "white" => Self::White,
            "grey1" => Self::Grey1,
            "grey2" => Self::Grey2,
            "grey3" => Self::Grey3,
            "grey4" => Self::Grey4,
            "black" => Self::Black,
            _ => return Err(format!("Unable to parse Color: {}", value)),
        };

        Ok(c)
    }
}
