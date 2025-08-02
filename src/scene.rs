use std::sync::LazyLock;

use three_d::CpuMesh;

pub const LOD_LEVEL_COUNT: usize = 8;

/// Level of detail subdivisions for the celestial object(s).
///
/// Smaller indices mean smaller distance which means higher detail.
pub const LOD_SUBDIVS: [u32; LOD_LEVEL_COUNT] = [24, 16, 12, 9, 7, 5, 3, 2];

/// Level of detail cutoffs, in radians.
///
/// If the radial size of a sphere `theta` >= a cutoff `c_i`,
/// then subdivision index `i` should be used. If theta is less
/// than all the cutoffs, then the sphere should not be rendered at all.
pub const LOD_CUTOFFS: [f64; LOD_LEVEL_COUNT] =
    [1.0, 0.75, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125];

const _: () = {
    assert!(
        LOD_SUBDIVS.len() == LOD_CUTOFFS.len(),
        "LOD_SUBDIVS and LOD_CUTOFFS should have the same length"
    )
};

pub static SPHERE_MESHES: LazyLock<[CpuMesh; LOD_LEVEL_COUNT]> = LazyLock::new(|| {
    let mut array = core::array::from_fn(|_| CpuMesh::default());

    for (i, &subdivs) in LOD_SUBDIVS.iter().enumerate() {
        array[i] = CpuMesh::sphere(subdivs);
    }

    array
});
