use std::sync::LazyLock;

use glam::DVec3;
use three_d::{
    Axes, ColorMaterial, Context, CpuMesh, Geometry, Gm, InstancedMesh, Line, Material, Object,
    PhysicalMaterial,
};

use super::Program;
use super::universe::Universe;

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

// pub fn generate_scene(context: &Context, universe: &Universe) -> Vec<InstancedMesh> {
//     let meshes: InstancedMesh = InstancedMesh::new(context, universe.get_bodies().values().map(), SPHERE_MESHES[0])
// }

pub(crate) struct Scene {
    bodies: Vec<Gm<InstancedMesh, PhysicalMaterial>>,
    lines: Vec<Gm<InstancedMesh, ColorMaterial>>,
}

impl IntoIterator for Scene {
    type Item = impl Object;
    type IntoIter = core::iter::Chain<
        std::vec::IntoIter<Gm<InstancedMesh, PhysicalMaterial>>,
        std::vec::IntoIter<Gm<InstancedMesh, ColorMaterial>>,
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.bodies.into_iter().chain(self.lines.into_iter())
    }
}

enum MyObject {
    Physical(Gm<InstancedMesh, PhysicalMaterial>),
    Color(Gm<InstancedMesh, ColorMaterial>),
}

impl Object for MyObject {
//     fn material_type(&self) -> three_d::MaterialType {
//         match self {
//             MyObject::Physical(gm) => gm.material_type(),
//             MyObject::Color(gm) => gm.material_type(),
//         }
//     }
//     fn render(&self, viewer: &dyn three_d::Viewer, lights: &[&dyn three_d::Light]) {
//         match self {
//             MyObject::Physical(gm) => gm.render(viewer, lights),
//             MyObject::Color(gm) => gm.render(viewer, lights),
//         }
//     }
// }

fn get_radial_size(radius: f64, distance: f64) -> f64 {
    2.0 * radius / distance
}

impl Program {
    pub(crate) fn generate_scene(&self, camera_pos: DVec3) -> Scene {
        let position_map = self.universe.get_all_body_positions();
    }

    fn generate_body_tris(
        &self,
        camera_pos: DVec3,
        position_map: &HashMap<u64, DVec3>,
    ) -> Gm<InstancedMesh, PhysicalMaterial> {
        let body_map = self.universe.get_bodies();
    }

    fn generate_orbit_lines(
        &self,
        camera_pos: DVec3,
        position_map: &HashMap<u64, DVec3>,
    ) -> Gm<InstancedMesh, PhysicalMaterial> {
        Gm::new()
    }
}
