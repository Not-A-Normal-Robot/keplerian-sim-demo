use std::collections::HashMap;
use std::f64::consts::TAU;
use std::sync::LazyLock;

use glam::DVec3;
use three_d::{
    ColorMaterial, CpuMaterial, CpuMesh, Gm, InstancedMesh, Instances, Mat4, Object,
    PhysicalMaterial, RenderStates, Srgba, Vec3, Vec4,
};

use super::Program;
use super::autoscaling_sprites::AutoscalingSprites;
use super::universe::{BodyWrapper, Id};

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
    [1.0, 0.25, 0.125, 0.062, 0.031, 0.015, 0.007, 0.002];

const fn get_lod_type(radial_size: f64) -> Option<usize> {
    let mut i = 0;
    while i < LOD_LEVEL_COUNT {
        if radial_size >= LOD_CUTOFFS[i] {
            return Some(i);
        }
        i += 1;
    }
    None
}

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
    bodies: [Gm<InstancedMesh, PhysicalMaterial>; LOD_LEVEL_COUNT],
    lines: Box<[Gm<AutoscalingSprites, ColorMaterial>]>,
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
            core::slice::Iter<'a, Gm<AutoscalingSprites, ColorMaterial>>,
            fn(&'a Gm<AutoscalingSprites, ColorMaterial>) -> &'a dyn Object,
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
                gm_to_object::<AutoscalingSprites, ColorMaterial>
                    as fn(&Gm<AutoscalingSprites, ColorMaterial>) -> &dyn Object,
            ))
    }
}

fn get_radial_size(radius: f64, distance: f64) -> f64 {
    2.0 * radius / distance
}

fn get_matrix(position: DVec3, radius: f64) -> Mat4 {
    // let DVec3 { x, y, z } = position;
    let (x, y, z) = (position.x as f32, position.y as f32, position.z as f32);
    let r = radius as f32;
    Mat4 {
        x: Vec4::new(r, 0.0, 0.0, 0.0),
        y: Vec4::new(0.0, r, 0.0, 0.0),
        z: Vec4::new(0.0, 0.0, r, 0.0),
        w: Vec4::new(x, y, z, 1.0),
    }
}
fn add_body_instance(
    id: &super::universe::Id,
    body_wrapper: &super::universe::BodyWrapper,
    camera_offset: DVec3,
    position_map: &HashMap<u64, DVec3>,
    instances_arr: &mut [Instances; LOD_LEVEL_COUNT],
) {
    let body = &body_wrapper.body;
    let position = match position_map.get(id) {
        Some(p) => p - camera_offset,
        None => return,
    };
    let distance = position.length();
    let size = get_radial_size(body.radius, distance);
    let lod_group = match get_lod_type(size) {
        Some(l) => l,
        None => return,
    };
    let matrix = get_matrix(position, body.radius);
    let instances = &mut instances_arr[lod_group];
    instances.transformations.push(matrix);

    if let Some(colors) = &mut instances.colors {
        colors.push(body.color);
    }
}

fn add_body_instances(
    body_map: &HashMap<Id, BodyWrapper>,
    camera_offset: DVec3,
    position_map: &HashMap<u64, DVec3>,
    instances_arr: &mut [Instances; LOD_LEVEL_COUNT],
) {
    for (id, body_wrapper) in body_map {
        add_body_instance(id, body_wrapper, camera_offset, position_map, instances_arr);
    }
}

impl Program {
    pub(crate) fn generate_scene(&self, camera_offset: DVec3) -> Scene {
        let position_map = self.universe.get_all_body_positions();
        Scene {
            bodies: self.generate_body_tris(camera_offset, &position_map),
            lines: self.generate_orbit_lines(camera_offset, &position_map),
        }
    }

    fn generate_body_tris(
        &self,
        camera_offset: DVec3,
        position_map: &HashMap<u64, DVec3>,
    ) -> [Gm<InstancedMesh, PhysicalMaterial>; LOD_LEVEL_COUNT] {
        let mut instances_arr: [Instances; LOD_LEVEL_COUNT] = core::array::from_fn(|_| Instances {
            transformations: Vec::new(),
            colors: Some(Vec::new()),
            texture_transformations: None,
        });

        let body_map = self.universe.get_bodies();

        add_body_instances(body_map, camera_offset, position_map, &mut instances_arr);

        let material = PhysicalMaterial::new_opaque(&self.context, &CpuMaterial::default());

        core::array::from_fn(|index| {
            Gm::new(
                InstancedMesh::new(&self.context, &instances_arr[index], &SPHERE_MESHES[index]),
                material.clone(),
            )
        })
    }

    const POINTS_PER_ORBIT: usize = 128;
    const RAD_PER_POINT: f64 = TAU / Self::POINTS_PER_ORBIT as f64;
    const POINT_SCALE: f32 = 0.003;

    fn generate_orbit_lines(
        &self,
        camera_offset: DVec3,
        _position_map: &HashMap<u64, DVec3>,
    ) -> Box<[Gm<AutoscalingSprites, ColorMaterial>]> {
        // TODO: Hook this onto the actual orbits
        let orbit_points: Box<[[Vec3; Self::POINTS_PER_ORBIT]]> = vec![core::array::from_fn(|i| {
            let (sin, cos) = (i as f64 * Self::RAD_PER_POINT).sin_cos();
            let v = DVec3::new(sin * 200.0, cos * 200.0, 0.0) - camera_offset;
            Vec3::new(v.x as f32, v.y as f32, v.z as f32)
        })]
        .into_boxed_slice();
        let mat = ColorMaterial {
            color: Srgba::new_opaque(255, 255, 255),
            texture: None,
            render_states: RenderStates::default(),
            is_transparent: false,
        };
        orbit_points
            .into_iter()
            .map(|arr| {
                Gm::new(
                    AutoscalingSprites::new(&self.context, &arr, None, POINT_SCALE),
                    mat.clone(),
                )
            })
            .collect()
    }
}
