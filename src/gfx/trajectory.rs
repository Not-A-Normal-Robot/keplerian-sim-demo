use keplerian_sim::OrbitTrait;
use three_d::{
    AxisAlignedBoundingBox, ColorMapping, Context, ElementBuffer, Geometry, GeometryId, Mat4,
    Matrix4, Object, Program, RenderStates, Srgba, Vec2, Vec3, Vec4, Viewer, render_with_material,
};
use three_d::{Blend, EffectMaterialId, HasContext, Material, MaterialType};

pub struct Trajectory {
    context: Context,
    eccentricity: f32,
    a_norm: f32,
    b_norm: f32,
    matrix: Mat4,
    pub curr_ecc_anom: f32,
    point_count: u32,
    pub thickness: f32,
    element_buffer: ElementBuffer<u32>,
    pub color: Srgba,
}

impl Trajectory {
    /// Creates a new renderable conic section.
    ///
    /// Note: parent_pos is the parent position relative to the render origin
    /// (the camera focus). It is not relative to the "real"/simulation origin.
    pub fn new(
        context: &Context,
        orbit: &impl OrbitTrait,
        parent_pos: Vec3,
        eccentric_anomaly: f32,
        point_count: u32,
        thickness: f32,
    ) -> Self {
        let matrix = orbit.get_transformation_matrix();
        let rp = orbit.get_periapsis();
        let matrix = Matrix4 {
            x: Vec4::new(
                (matrix.e11 * rp) as f32,
                (matrix.e21 * rp) as f32,
                (matrix.e31 * rp) as f32,
                0.0,
            ),
            y: Vec4::new(
                (matrix.e12 * rp) as f32,
                (matrix.e22 * rp) as f32,
                (matrix.e32 * rp) as f32,
                0.0,
            ),
            z: Vec4::new(0.0, 0.0, 0.0, 0.0),
            w: Vec4::new(parent_pos.x, parent_pos.y, parent_pos.z, 1.0),
        };
        let eccentricity = orbit.get_eccentricity();
        let a_norm = (1.0 - eccentricity).recip();
        let b_norm = a_norm * (1.0 - eccentricity.powi(2)).abs().sqrt();

        let point_count = point_count.max(3);

        let indices = Self::get_indices(point_count, eccentricity as f32);
        let element_buffer = ElementBuffer::new_with_data(context, &indices);

        Self {
            context: context.clone(),
            eccentricity: eccentricity as f32,
            a_norm: a_norm as f32,
            b_norm: b_norm as f32,
            matrix,
            curr_ecc_anom: eccentric_anomaly,
            point_count,
            thickness,
            element_buffer,
            color: Srgba::new_opaque(255, 255, 255),
        }
    }

    pub fn set_eccentric_anomaly(&mut self, eccentric_anomaly: f64) {
        if self.eccentricity < 1.0 {
            self.curr_ecc_anom = eccentric_anomaly.rem_euclid(core::f64::consts::TAU) as f32;
        } else {
            self.curr_ecc_anom = eccentric_anomaly as f32;
        }
    }

    pub fn update_from_orbit(&mut self, orbit: &impl OrbitTrait, parent_pos: Vec3) {
        let matrix = orbit.get_transformation_matrix();
        let rp = orbit.get_periapsis();
        let matrix = Matrix4 {
            x: Vec4::new(
                (matrix.e11 * rp) as f32,
                (matrix.e21 * rp) as f32,
                (matrix.e31 * rp) as f32,
                0.0,
            ),
            y: Vec4::new(
                (matrix.e12 * rp) as f32,
                (matrix.e22 * rp) as f32,
                (matrix.e32 * rp) as f32,
                0.0,
            ),
            z: Vec4::new(0.0, 0.0, 0.0, 0.0),
            w: Vec4::new(parent_pos.x, parent_pos.y, parent_pos.z, 1.0),
        };
        let eccentricity = orbit.get_eccentricity();
        let a_norm = (1.0 - eccentricity).recip();
        let b_norm = a_norm * (1.0 - eccentricity.powi(2)).abs().sqrt();

        let old_eccentricity = self.eccentricity;

        self.matrix = matrix;
        self.eccentricity = eccentricity as f32;
        self.a_norm = a_norm as f32;
        self.b_norm = b_norm as f32;

        // Check if orbit kind changed (elliptic → hyperbolic, vice-versa)
        if (old_eccentricity < 1.0) != (eccentricity < 1.0) {
            self.set_point_count(self.point_count);
        }
    }

    pub fn set_point_count(&mut self, point_count: u32) {
        self.point_count = point_count.max(3);

        let data = Self::get_indices(point_count, self.eccentricity);

        self.element_buffer = ElementBuffer::new_with_data(&self.context, &data);
    }

    fn get_indices(point_count: u32, eccentricity: f32) -> Vec<u32> {
        let point_count = point_count.max(3);

        let segment_count = if eccentricity < 1.0 {
            point_count
        } else {
            point_count - 1
        };
        let mut indices: Vec<u32> = Vec::with_capacity((segment_count * 6) as usize);
        for i in 0..segment_count {
            let base = i * 2;
            // triangle A: curr_top (base), curr_bot (base+1), next_top (base+2)
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            // triangle B: next_top (base+2), curr_bot (base+1), next_bot (base+3)
            indices.push(base + 2);
            indices.push(base + 1);
            indices.push(base + 3);
        }
        indices
    }

    fn eccentric_anomaly_range(&self) -> f32 {
        if self.eccentricity < 1.0 {
            core::f32::consts::TAU
        } else {
            if self.curr_ecc_anom > 10.0 {
                // We don't need a path anymore.
                // It's way too far.
                0.0
            } else {
                // Bell curve to prevent physical range from becoming
                // too long too quickly.
                (30.0 * 2.0_f64.powf(-0.15 * (self.curr_ecc_anom as f64).powi(2))) as f32
            }
        }
    }
}

const VERTEX_SHADER_SOURCE: &'static str = {
    const SHADER_VERSION_HEADER: &'static str = "#version 330 core";
    const SHADER_FULL_SOURCE: &'static str = include_str!("shaders/trajectory.vert");
    const fn trim_bytes_start_len<'a>(mut bytes: &[u8], mut len: usize) -> &[u8] {
        while let [_first, rest @ ..] = bytes {
            if len == 0 {
                break;
            }
            bytes = rest;
            len -= 1;
        }
        bytes
    }
    match core::str::from_utf8(trim_bytes_start_len(
        SHADER_FULL_SOURCE.as_bytes(),
        SHADER_VERSION_HEADER.len(),
    )) {
        Ok(v) => v,
        Err(_) => panic!("resulting string not UTF-8"),
    }
};

impl Geometry for Trajectory {
    fn draw(&self, viewer: &dyn Viewer, program: &Program, render_states: RenderStates) {
        let eccentric_anomaly_range = self.eccentric_anomaly_range();

        let start_eccentric_anomaly = self.curr_ecc_anom - 0.5 * eccentric_anomaly_range;

        program.use_uniform("u_proj_view", viewer.projection() * viewer.view());
        program.use_uniform("u_tf", self.matrix);
        program.use_uniform("u_eccentricity", self.eccentricity);
        program.use_uniform("u_a_norm", self.a_norm);
        program.use_uniform("u_b_norm", self.b_norm);
        program.use_uniform("u_start_ecc_anom", start_eccentric_anomaly);
        program.use_uniform("u_vertex_count", self.point_count);
        program.use_uniform("u_thickness_px", self.thickness);
        program.use_uniform(
            "u_viewport",
            Vec2::new(
                viewer.viewport().width as f32,
                viewer.viewport().height as f32,
            ),
        );
        program.use_uniform("u_ecc_anom_range", eccentric_anomaly_range);

        // Ensure a VAO is bound even when we don't use any vertex attributes.
        // On core GL profiles a draw call without a bound VAO generates
        // GL_INVALID_OPERATION. Create a temporary VAO, bind it, draw and delete it.
        let vao = unsafe {
            self.context
                .create_vertex_array()
                .expect("Failed to create temporary VAO for conic draw")
        };
        unsafe {
            self.context.bind_vertex_array(Some(vao));
        }

        program.draw_elements(render_states, viewer.viewport(), &self.element_buffer);

        unsafe {
            self.context.bind_vertex_array(None);
            self.context.delete_vertex_array(vao);
        }
        // element_buffer is dropped here and its GPU buffer freed
    }

    fn vertex_shader_source(&self) -> String {
        VERTEX_SHADER_SOURCE.to_owned()
    }

    fn id(&self) -> GeometryId {
        GeometryId(0x5FA5)
    }

    fn render_with_material(
        &self,
        material: &dyn three_d::Material,
        viewer: &dyn three_d::Viewer,
        lights: &[&dyn three_d::Light],
    ) {
        render_with_material(&self.context, viewer, &self, material, lights);
    }

    fn render_with_effect(
        &self,
        _material: &dyn three_d::Effect,
        _viewer: &dyn three_d::Viewer,
        _lights: &[&dyn three_d::Light],
        _color_texture: Option<three_d::ColorTexture>,
        _depth_texture: Option<three_d::DepthTexture>,
    ) {
        panic!("Rendering conic sections with effects aren't supported");
    }

    fn aabb(&self) -> three_d::AxisAlignedBoundingBox {
        AxisAlignedBoundingBox::INFINITE
    }
}

const FRAGMENT_SHADER_SOURCE: &'static str = {
    const VERSION_HEADER: &'static str = "#version 330 core";
    const SHADER_FULL_SOURCE: &'static str = include_str!("shaders/trajectory.frag");
    const fn trim_bytes_start_len<'a>(mut bytes: &[u8], mut len: usize) -> &[u8] {
        while let [_first, rest @ ..] = bytes {
            if len == 0 {
                break;
            }
            bytes = rest;
            len -= 1;
        }
        bytes
    }
    match core::str::from_utf8(trim_bytes_start_len(
        SHADER_FULL_SOURCE.as_bytes(),
        VERSION_HEADER.len(),
    )) {
        Ok(v) => v,
        Err(_) => panic!("resulting string not UTF-8"),
    }
};

impl Material for Trajectory {
    fn fragment_shader_source(&self, _lights: &[&dyn three_d::Light]) -> String {
        let a = ColorMapping::fragment_shader_source();
        let b = FRAGMENT_SHADER_SOURCE;
        let mut string = String::with_capacity(a.len() + b.len());
        string.push_str(a);
        string.push_str(b);
        string
    }

    fn id(&self) -> EffectMaterialId {
        EffectMaterialId(0x5FA5)
    }

    fn use_uniforms(
        &self,
        program: &Program,
        viewer: &dyn Viewer,
        _lights: &[&dyn three_d::Light],
    ) {
        viewer.color_mapping().use_uniforms(program);
        program.use_uniform("surface_color", self.color.to_linear_srgb());
        program.use_uniform("curr_ecc_anom", self.curr_ecc_anom);
        program.use_uniform("anomaly_range", self.eccentric_anomaly_range());
        program.use_uniform("eccentricity", self.eccentricity);
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            blend: Blend::TRANSPARENCY,
            cull: three_d::Cull::Front,
            ..Default::default()
        }
    }

    fn material_type(&self) -> MaterialType {
        MaterialType::Transparent
    }
}

impl Object for Trajectory {
    fn material_type(&self) -> MaterialType {
        <Self as Material>::material_type(self)
    }
    fn render(&self, viewer: &dyn Viewer, _lights: &[&dyn three_d::Light]) {
        render_with_material(&self.context, viewer, self, self, &[]);
    }
}

impl<'a> IntoIterator for &'a Trajectory {
    type Item = &'a dyn Object;
    type IntoIter = core::iter::Once<&'a dyn Object>;

    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(self)
    }
}
