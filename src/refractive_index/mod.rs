#![allow(clippy::excessive_precision)]

///! In optics, the **refractive index** of a material is a dimensionless number that describes
///! how fast light travels through the material.
///!
///! This trait helps describe the different spectra of refractive indices, as different wavelengths
///! refract differently.
///!
///! To complement the refractive index, this trait also specifies to return an **optional extinction
///! coefficient**. The extinction coefficient describes how strongly a material absorbs light at given
///! wavelength.
use utility::floats::fast_cmp;
use utility::math::lerp_map;

mod air;
mod glass;
mod plastic;
mod water;

pub enum RefractiveType {
    AIR,
    VACUUM,
    WATER,
    GLASS,
}

impl RefractiveType {
    pub fn n_uniform(&self) -> f32 {
        match self {
            RefractiveType::AIR => 1.00029,
            RefractiveType::VACUUM => 1.0,
            RefractiveType::WATER => 1.3325,
            RefractiveType::GLASS => 1.5168,
        }
    }

    pub fn k_uniform(&self) -> Option<f32> {
        match self {
            RefractiveType::AIR => None,
            RefractiveType::VACUUM => None,
            RefractiveType::WATER => Some(7.2792e-9),
            RefractiveType::GLASS => Some(9.7525e-9),
        }
    }

    /// Returns the refractive index at a given wavelength.
    ///
    /// # Arguments
    /// * `wavelength_nm` - The wavelength in nm
    ///
    /// # Returns
    /// * The corresponding refractive index
    pub fn n(&self, wavelength_nm: f32) -> f32 {
        match self {
            RefractiveType::AIR => search_and_get(&air::INDEX, &air::N, wavelength_nm),
            RefractiveType::VACUUM => 1.0,
            RefractiveType::WATER => search_and_get(&water::INDEX, &water::N, wavelength_nm),
            RefractiveType::GLASS => search_and_get(&glass::INDEX_N, &glass::N, wavelength_nm),
        }
    }

    /// Returns the extinction coefficient at a given wavelength (if it exists).
    ///
    /// # Arguments
    /// * `wavelength_nm` - The wavelength in nm
    ///
    /// # Returns
    /// * `Some` corresponding extinction coefficient
    /// * `None`
    pub fn k(&self, wavelength_nm: f32) -> Option<f32> {
        match self {
            RefractiveType::AIR => None,
            RefractiveType::VACUUM => None,
            RefractiveType::WATER => Some(search_and_get(&water::INDEX, &water::K, wavelength_nm)),
            RefractiveType::GLASS => {
                Some(search_and_get(&glass::INDEX_K, &glass::K, wavelength_nm))
            }
        }
    }
}

/// Searches for the index of a given value inside a given slice.
/// If no such value is found, it will return the the indexes below/above the value, allowing to
/// lerp further usages.
pub fn search_index(slice: &[f32], value: f32) -> Result<usize, (usize, usize)> {
    match slice.binary_search_by(|a| fast_cmp(*a, value)) {
        Ok(index) => Ok(index),
        Err(index) => Err((index - 1, index)),
    }
}

pub fn search_and_get(index_slice: &[f32], value_slice: &[f32], wavelength_nm: f32) -> f32 {
    match search_index(index_slice, wavelength_nm) {
        Ok(i) => value_slice[i],
        Err((min, max)) => {
            if max > value_slice.len() {
                return value_slice[min];
            }

            let to_lerp = (value_slice[min], value_slice[max]);

            lerp_map((min as f32, max as f32), to_lerp, wavelength_nm)
        }
    }
}
