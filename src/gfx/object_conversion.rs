use std::collections::HashMap;
use std::sync::LazyLock;

use glam::DVec3;
use keplerian_sim::OrbitTrait;
use three_d::{
    Blend, ColorMaterial, Context, CpuMaterial, CpuMesh, Cull, Gm, InstancedMesh, Instances, Mat4,
    Mesh, Object, PhysicalMaterial, RenderStates, Srgba, Vec3, Vec4,
};

use super::{Body, BodyWrapper, Id, PreviewBody, Program, trajectory::Trajectory};

pub const LOD_LEVEL_COUNT: usize = 8;

/// Level of detail subdivisions for the celestial object(s).
///
/// Smaller indices mean smaller distance which means higher detail.
pub const LOD_SUBDIVS: [u32; LOD_LEVEL_COUNT] = [32, 24, 16, 12, 9, 7, 5, 3];

/// Level of detail cutoffs, in radians.
///
/// If the radial size of a sphere `theta` >= a cutoff `c_i`,
/// then subdivision index `i` should be used. If theta is less
/// than all the cutoffs, then the sphere should not be rendered at all.
pub const LOD_CUTOFFS: [f64; LOD_LEVEL_COUNT] =
    [0.25, 0.125, 0.062, 0.031, 0.015, 0.007, 0.002, 0.0005];

/// The minimum camera radial size to consider rendering an orbit.
/// If an orbit is smaller than this, it is ignored.
pub const MIN_ORBIT_RADIAL_SIZE: f64 = 0.002;

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

pub(crate) struct PreviewScene {
    body: Option<Gm<Mesh, ColorMaterial>>,
    path: Option<Trajectory>,
}

impl<'a> IntoIterator for &'a PreviewScene {
    type Item = &'a dyn Object;
    type IntoIter = std::iter::Chain<
        std::iter::Map<
            core::option::Iter<'a, Gm<Mesh, ColorMaterial>>,
            fn(&'a Gm<Mesh, ColorMaterial>) -> &'a dyn Object,
        >,
        std::iter::Map<core::option::Iter<'a, Trajectory>, fn(&'a Trajectory) -> &'a dyn Object>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.body
            .iter()
            .map(gm_to_object::<Mesh, ColorMaterial> as fn(&Gm<Mesh, ColorMaterial>) -> &dyn Object)
            .chain(
                self.path
                    .iter()
                    .map((|x| x) as fn(&'a Trajectory) -> &'a dyn Object),
            )
    }
}

pub(crate) struct Scene {
    bodies: [Gm<InstancedMesh, PhysicalMaterial>; LOD_LEVEL_COUNT],
    lines: Box<[Trajectory]>,
    preview: Option<PreviewScene>,
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
        std::iter::Chain<
            std::iter::Map<
                core::slice::Iter<'a, Gm<InstancedMesh, PhysicalMaterial>>,
                fn(&'a Gm<InstancedMesh, PhysicalMaterial>) -> &'a dyn Object,
            >,
            std::iter::Map<core::slice::Iter<'a, Trajectory>, fn(&'a Trajectory) -> &'a dyn Object>,
        >,
        std::iter::Flatten<
            std::iter::Map<
                core::option::IntoIter<&'a PreviewScene>,
                fn(
                    &'a PreviewScene,
                ) -> std::iter::Chain<
                    std::iter::Map<
                        core::option::Iter<'a, Gm<Mesh, ColorMaterial>>,
                        fn(&'a Gm<Mesh, ColorMaterial>) -> &'a dyn Object,
                    >,
                    std::iter::Map<
                        core::option::Iter<'a, Trajectory>,
                        fn(&'a Trajectory) -> &'a dyn Object,
                    >,
                >,
            >,
        >,
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.bodies
            .iter()
            .map(
                gm_to_object::<InstancedMesh, PhysicalMaterial>
                    as fn(&Gm<InstancedMesh, PhysicalMaterial>) -> &dyn Object,
            )
            .chain(
                self.lines
                    .iter()
                    .map((|t| t) as fn(&'a Trajectory) -> &'a dyn Object),
            )
            .chain(
                self.preview
                    .as_ref()
                    .into_iter()
                    .map(
                        <&PreviewScene as IntoIterator>::into_iter
                            as fn(_) -> <&'a PreviewScene as IntoIterator>::IntoIter,
                    )
                    .flatten(),
            )
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
    id: &Id,
    body_wrapper: &BodyWrapper,
    camera_offset: DVec3,
    camera_pos: DVec3,
    position_map: &HashMap<Id, DVec3>,
    instances_arr: &mut [Instances; LOD_LEVEL_COUNT],
) {
    let body = &body_wrapper.body;
    let position = match position_map.get(id) {
        Some(p) => p - camera_offset,
        None => return,
    };
    let distance = (position - camera_pos).length();
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
    camera_pos: DVec3,
    position_map: &HashMap<Id, DVec3>,
    instances_arr: &mut [Instances; LOD_LEVEL_COUNT],
) {
    for (id, body_wrapper) in body_map {
        add_body_instance(
            id,
            body_wrapper,
            camera_offset,
            camera_pos,
            position_map,
            instances_arr,
        );
    }
}

impl Program {
    pub(crate) fn to_objects(&self, position_map: &HashMap<Id, DVec3>) -> Scene {
        let camera_offset = *position_map
            .get(&self.sim_state.focused_body())
            .unwrap_or(&DVec3::ZERO)
            + self.sim_state.focus_offset;

        let camera_pos = self.camera.position();
        let camera_pos = DVec3::new(
            camera_pos.x as f64,
            camera_pos.y as f64,
            camera_pos.z as f64,
        );

        Scene {
            bodies: self.generate_body_gms(camera_offset, camera_pos, position_map),
            lines: self.generate_orbit_lines(camera_offset, camera_pos, position_map),
            preview: self.generate_preview_scene(camera_offset, camera_pos, position_map),
        }
    }

    fn generate_body_gms(
        &self,
        camera_offset: DVec3,
        camera_pos: DVec3,
        position_map: &HashMap<Id, DVec3>,
    ) -> [Gm<InstancedMesh, PhysicalMaterial>; LOD_LEVEL_COUNT] {
        let mut instances_arr: [Instances; LOD_LEVEL_COUNT] = core::array::from_fn(|_| Instances {
            transformations: Vec::new(),
            colors: Some(Vec::new()),
            texture_transformations: None,
        });

        let body_map = self.sim_state.universe.get_bodies();

        add_body_instances(
            body_map,
            camera_offset,
            camera_pos,
            position_map,
            &mut instances_arr,
        );

        let mut material = PhysicalMaterial::new_opaque(&self.context, &CpuMaterial::default());

        material.render_states = RenderStates {
            cull: Cull::Back,
            ..Default::default()
        };

        core::array::from_fn(|index| {
            Gm::new(
                InstancedMesh::new(&self.context, &instances_arr[index], &SPHERE_MESHES[index]),
                material.clone(),
            )
        })
    }

    const LINE_THICKNESS: f32 = 2.0;
    const FOCUSED_THICKNESS: f32 = Self::LINE_THICKNESS * 1.5;

    fn generate_orbit_lines(
        &self,
        camera_offset: DVec3,
        camera_pos: DVec3,
        position_map: &HashMap<Id, DVec3>,
    ) -> Box<[Trajectory]> {
        self.sim_state
            .universe
            .get_bodies()
            .iter()
            .filter_map(|(&id, body_wrapper)| {
                Self::generate_orbit_line(
                    &self.context,
                    &body_wrapper.body,
                    body_wrapper.relations.parent,
                    camera_offset,
                    camera_pos,
                    position_map,
                    self.sim_state.universe.time,
                    if id == self.sim_state.focused_body() {
                        Self::FOCUSED_THICKNESS
                    } else {
                        Self::LINE_THICKNESS
                    },
                )
            })
            .collect()
    }

    fn generate_orbit_line(
        context: &Context,
        body: &Body,
        parent_id: Option<Id>,
        camera_offset: DVec3,
        camera_pos: DVec3,
        position_map: &HashMap<Id, DVec3>,
        time: f64,
        thickness: f32,
    ) -> Option<Trajectory> {
        let orbit = match &body.orbit {
            Some(o) => o,
            None => return None,
        };

        let parent_pos = parent_id
            .map(|id| *position_map.get(&id).unwrap_or(&DVec3::default()))
            .unwrap_or(DVec3::default());

        let offset_d = parent_pos - camera_offset;

        let offset_s = Vec3::new(offset_d.x as f32, offset_d.y as f32, offset_d.z as f32);

        let eccentric_anomaly = orbit.get_eccentric_anomaly_at_time(time);

        let point_count = if orbit.get_eccentricity() < 1.0 {
            let semi_major_axis = orbit.get_semi_major_axis();
            let distance_to_camera = (offset_d - camera_pos).length();
            let radial_size = get_radial_size(semi_major_axis, distance_to_camera);
            if radial_size < MIN_ORBIT_RADIAL_SIZE {
                // Too small to see, skip
                return None;
            }

            (radial_size * 512.0).abs().clamp(16.0, 1024.0) as u32
        } else {
            512
        };

        Some(Trajectory::new(
            context,
            orbit,
            offset_s,
            eccentric_anomaly as f32,
            point_count,
            thickness,
            body.color,
        ))
    }

    fn generate_preview_body(
        &self,
        camera_offset: DVec3,
        camera_pos: DVec3,
        position_map: &HashMap<Id, DVec3>,
        wrapper: &PreviewBody,
    ) -> Option<Gm<Mesh, ColorMaterial>> {
        let parent_pos = wrapper
            .parent_id
            .map(|id| position_map.get(&id).map(|x| *x))
            .flatten()
            .unwrap_or(DVec3::ZERO);
        let body_pos = wrapper
            .body
            .orbit
            .as_ref()
            .map(|o| o.get_position_at_time(self.sim_state.universe.time))
            .unwrap_or(DVec3::ZERO)
            + parent_pos;
        let position = body_pos - camera_offset;
        let distance = (position - camera_pos).length();
        let radial_size = get_radial_size(wrapper.body.radius, distance);

        let cpu_mesh = &SPHERE_MESHES[get_lod_type(radial_size)?];
        let mut mesh = Mesh::new(&self.context, cpu_mesh);
        let r = wrapper.body.radius as f32;
        mesh.set_transformation(Mat4 {
            x: Vec4::new(r, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, r, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, r, 0.0),
            w: Vec4::new(position.x as f32, position.y as f32, position.z as f32, 1.0),
        });

        let material = ColorMaterial {
            color: Srgba {
                a: (((wrapper.body.color.a as u16 * 127u16) + 127) / 255) as u8,
                ..wrapper.body.color
            },
            texture: None,
            render_states: RenderStates {
                cull: Cull::Back,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
            is_transparent: true,
        };

        Some(Gm::new(mesh, material))
    }

    const PREVIEW_POINT_SCALE: f32 = Self::LINE_THICKNESS * 2.0;

    fn generate_preview_scene(
        &self,
        camera_offset: DVec3,
        camera_pos: DVec3,
        position_map: &HashMap<Id, DVec3>,
    ) -> Option<PreviewScene> {
        let body_wrapper = self.sim_state.preview_body.as_ref()?;

        let body_gm =
            self.generate_preview_body(camera_offset, camera_pos, position_map, body_wrapper);
        let path = Self::generate_orbit_line(
            &self.context,
            &body_wrapper.body,
            body_wrapper.parent_id,
            camera_offset,
            camera_pos,
            position_map,
            self.sim_state.universe.time,
            Self::PREVIEW_POINT_SCALE,
        );

        if body_gm.is_none() && path.is_none() {
            return None;
        }

        Some(PreviewScene {
            body: body_gm,
            path,
        })
    }
}
