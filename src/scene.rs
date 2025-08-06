use std::collections::HashMap;
use std::iter::FusedIterator;
use std::sync::LazyLock;

use enum_dispatch::enum_dispatch;
use glam::DVec3;
use three_d::{
    Axes, ColorMaterial, Context, CpuMaterial, CpuMesh, Geometry, Gm, InstancedMesh, Instances,
    Line, Material, Object, PhysicalMaterial, RenderStates, Srgba,
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

pub(crate) struct Scene {
    bodies: Vec<Gm<InstancedMesh, PhysicalMaterial>>,
    lines: Vec<Gm<InstancedMesh, ColorMaterial>>,
}

/// Converts a Gm into an abstract Object.
///
/// This uses the fact that `Gm::into_iter()` returns
/// a `std::iter::Once<&dyn Object>`. We can then
/// call `.next()` on it to get an `Option<&dyn Object>`,
/// which in this case is always `Some`, which can be unwrapped.
fn gm_to_object<G, M>(gm: &Gm<G, M>) -> &dyn Object
where
    G: three_d::Geometry,
    M: three_d::Material,
{
    let mut iter: core::iter::Once<&dyn Object> = gm.into_iter();
    iter.next().unwrap()
}

impl<'a> IntoIterator for &'a Scene {
    type Item = &'a dyn Object;
    type IntoIter = std::iter::Chain<
        std::iter::Map<
            core::slice::Iter<'a, Gm<InstancedMesh, PhysicalMaterial>>,
            fn(&'a Gm<InstancedMesh, PhysicalMaterial>) -> &'a dyn Object,
        >,
        std::iter::Map<
            core::slice::Iter<'a, Gm<InstancedMesh, ColorMaterial>>,
            fn(&'a Gm<InstancedMesh, ColorMaterial>) -> &'a dyn Object,
        >,
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.bodies
            .iter()
            .map(
                gm_to_object::<InstancedMesh, PhysicalMaterial>
                    as fn(&Gm<InstancedMesh, PhysicalMaterial>) -> &dyn Object,
            )
            .chain(self.lines.iter().map(
                gm_to_object::<InstancedMesh, ColorMaterial>
                    as fn(&Gm<InstancedMesh, ColorMaterial>) -> &dyn Object,
            ))
    }
}

fn get_radial_size(radius: f64, distance: f64) -> f64 {
    2.0 * radius / distance
}

impl Program {
    pub(crate) fn generate_scene(&self, camera_pos: DVec3) -> Scene {
        let position_map = self.universe.get_all_body_positions();
        Scene {
            bodies: Vec::new(),
            lines: Vec::new(),
        }
    }

    fn generate_body_tris(
        &self,
        camera_pos: DVec3,
        position_map: &HashMap<u64, DVec3>,
    ) -> [Gm<InstancedMesh, PhysicalMaterial>; LOD_LEVEL_COUNT] {
        let mut instances: [Instances; LOD_LEVEL_COUNT] = core::array::from_fn(|_| Instances {
            transformations: Vec::new(),
            colors: Some(Vec::new()),
            texture_transformations: None,
        });

        // TODO

        let body_map = self.universe.get_bodies();
        let material = PhysicalMaterial::new_opaque(&self.context, &CpuMaterial::default());

        core::array::from_fn(|index| {
            Gm::new(
                InstancedMesh::new(&self.context, &instances[index], &SPHERE_MESHES[index]),
                material.clone(),
            )
        })
    }

    fn generate_orbit_lines(
        &self,
        camera_pos: DVec3,
        position_map: &HashMap<u64, DVec3>,
    ) -> Gm<InstancedMesh, ColorMaterial> {
        // TODO
        Gm::new(
            InstancedMesh::new(
                &self.context,
                &Instances {
                    ..Default::default()
                },
                &CpuMesh::default(),
            ),
            ColorMaterial {
                color: Srgba::new_opaque(0, 0, 0),
                texture: None,
                render_states: RenderStates::default(),
                is_transparent: false,
            },
        )
    }
}
