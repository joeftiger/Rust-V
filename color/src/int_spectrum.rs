use crate::color_data::LAMBDA_NUM;
use crate::*;
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

#[derive(Copy, Clone)]
pub struct IntSpectrum<const N: usize> {
    pub(crate) data: [u32; N],
}

impl<const N: usize> IntSpectrum<N> {
    pub fn new(data: [u32; N]) -> Self {
        Self { data }
    }

    pub const fn size() -> usize {
        N
    }

    pub fn broadcast(val: u32) -> Self {
        Self::new([val; N])
    }

    pub fn increment(&mut self) {
        self.data.iter_mut().for_each(|v| *v += 1);
    }
}

impl<const N: usize> Default for IntSpectrum<N> {
    fn default() -> Self {
        Self::broadcast(0)
    }
}

impl<I, const N: usize> Index<I> for IntSpectrum<N>
where
    I: SliceIndex<[u32]>,
{
    type Output = <I as SliceIndex<[u32]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl<I, const N: usize> IndexMut<I> for IntSpectrum<N>
where
    I: SliceIndex<[u32]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.data[index]
    }
}

macro_rules! impl_int_spectrum {
    ($name:ident, $size:expr) => {
        impl Mul<IntSpectrum<$size>> for $name {
            type Output = Self;

            #[inline]
            fn mul(mut self, rhs: IntSpectrum<$size>) -> Self::Output {
                self *= rhs;
                self
            }
        }

        impl MulAssign<IntSpectrum<$size>> for $name {
            #[inline]
            fn mul_assign(&mut self, rhs: IntSpectrum<$size>) {
                for i in 0..Self::size() {
                    self[i] *= rhs[i] as Float;
                }
            }
        }

        impl Div<IntSpectrum<$size>> for $name {
            type Output = Self;

            #[inline]
            fn div(mut self, rhs: IntSpectrum<$size>) -> Self::Output {
                self /= rhs;
                self
            }
        }

        impl DivAssign<IntSpectrum<$size>> for $name {
            #[inline]
            fn div_assign(&mut self, rhs: IntSpectrum<$size>) {
                for i in 0..Self::size() {
                    self[i] /= rhs[i] as Float
                }
            }
        }
    };
}

impl_int_spectrum!(Srgb, 3);
impl_int_spectrum!(Xyz, 3);
impl_int_spectrum!(Spectrum, LAMBDA_NUM);
