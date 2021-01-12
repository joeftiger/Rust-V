#![allow(dead_code)]

// TODO: WIP

//! Credit for Morton code calculation goes to
//! https://developer.nvidia.com/blog/thinking-parallel-part-iii-tree-construction-gpu/

mod bottom_up_bvh;

use crate::debug_util::in_range_incl;
use crate::Geometry;
pub use bottom_up_bvh::BottomUpBVH;
use ultraviolet::Vec3;

/// # Summary
/// Inverse linear interpolation.
///
/// # Arguments
/// * `min` - The minimum value
/// * `max` - The maximum value
/// * `val` - The lerped value
///
/// # Returns
/// * The parameter `t` of the lerp
fn inv_lerp(min: Vec3, max: Vec3, val: Vec3) -> Vec3 {
    (val - min) / (max - min)
}

/// # Summary
/// Expands a 10-bit integer into 30 bits by inserting 2 zeros after each bit.
///
/// # Constraints
/// * `bits` - Should be inside `[0, 1023]`.
///
/// # Arguments
/// * `bits` - 10-bit integer
///
/// # Returns
/// * Morton code of `bits`
fn expand_bits(mut bits: u32) -> u32 {
    debug_assert!(bits < 1024);

    bits = (bits * 0x00010001) & 0xFF0000FF;
    bits = (bits * 0x00000101) & 0x0F00F00F;
    bits = (bits * 0x00000011) & 0xC30C30C3;
    bits = (bits * 0x00000005) & 0x49249249;
    bits
}

/// # Summary
/// Calculates a 30-bit morton code for the given point located within the unit cube `0[0, 1]`.
///
/// # Constraints
/// * `v` - All values should be within `[0, 1]`.
///
/// # Arguments
/// * `v` - The point to transform
///
/// # Returns
/// * The morton code of `v`
fn morton_3d(v: Vec3) -> u32 {
    debug_assert!(in_range_incl(v.x, 0.0, 1.0));
    debug_assert!(in_range_incl(v.y, 0.0, 1.0));
    debug_assert!(in_range_incl(v.z, 0.0, 1.0));

    let exp = (v * 1024.0)
        .max_by_component(Vec3::zero())
        .min_by_component(Vec3::broadcast(1023.0));

    let x = expand_bits(exp.x as u32);
    let y = expand_bits(exp.y as u32);
    let z = expand_bits(exp.z as u32);

    (x << 2) + (y << 1) + z
}

/// # Summary
/// Computes the morton codes for the centers of the given boundables.
///
/// # Arguments
/// * `boundables` - The boundables to mortonize
///
/// # Returns
/// * The morton codes
fn compute_morton(boundables: &[Box<dyn Geometry>]) -> Vec<u32> {
    let len = boundables.len();

    // we cycle for better efficiency
    let mut centers = boundables.iter().map(|b| b.bounds().center()).cycle();

    let mut min = Vec3::one() * f32::INFINITY;
    let mut max = Vec3::one() * f32::NEG_INFINITY;

    for _ in 0..len {
        let center = centers.next().unwrap();
        min = min.min_by_component(center);
        max = max.max_by_component(center);
    }

    let mut morton_codes = Vec::with_capacity(len);
    for _ in 0..len {
        let center = centers.next().unwrap();

        let t = inv_lerp(min, max, center);

        let morton_code = morton_3d(t);
        morton_codes.push(morton_code);
    }

    morton_codes
}

// fn aac(primitives: &mut Vec<Arc<dyn Geometry>>) {
//     let mut morton_codes = compute_morton(primitives);
//     co_sort!(morton_codes, primitives);
//
//     let c = build_tree(primitives);
//
//     unimplemented!();
// }
//
// fn build_tree(subset: &Vec<Box<dyn Geometry>>) {
//
//     if subset.len() < 2 {}
//     unimplemented!();
// }
