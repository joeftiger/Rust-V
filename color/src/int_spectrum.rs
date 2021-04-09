use crate::spectral_data::LAMBDA_NUM;
use crate::*;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct IntSpectrum {
    #[serde(with = "ColorArray")]
    pub(crate) data: [u32; LAMBDA_NUM],
}

impl IntSpectrum {
    pub fn new(data: [u32; LAMBDA_NUM]) -> Self {
        Self { data }
    }

    pub fn size() -> usize {
        LAMBDA_NUM
    }

    pub fn broadcast(val: u32) -> Self {
        Self::new([val; LAMBDA_NUM])
    }

    pub fn increment(&mut self) {
        self.data.iter_mut().for_each(|v| *v += 1);
    }
}

impl Default for IntSpectrum {
    fn default() -> Self {
        Self {
            data: [0; LAMBDA_NUM],
        }
    }
}

impl<I> Index<I> for IntSpectrum
where
    I: SliceIndex<[u32]>,
{
    type Output = <I as SliceIndex<[u32]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl<I> IndexMut<I> for IntSpectrum
where
    I: SliceIndex<[u32]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Mul<IntSpectrum> for Spectrum {
    type Output = Self;

    fn mul(mut self, rhs: IntSpectrum) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<IntSpectrum> for Spectrum {
    fn mul_assign(&mut self, rhs: IntSpectrum) {
        for i in 0..LAMBDA_NUM {
            self[i] *= rhs[i] as Float
        }
    }
}

impl Div<IntSpectrum> for Spectrum {
    type Output = Self;

    fn div(mut self, rhs: IntSpectrum) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign<IntSpectrum> for Spectrum {
    fn div_assign(&mut self, rhs: IntSpectrum) {
        for i in 0..LAMBDA_NUM {
            self[i] /= rhs[i] as Float
        }
    }
}
