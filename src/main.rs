use keplerian_sim::Orbit;
use three_d::{
    AmbientLight, Camera, ClearState, Context, CpuTexture, Degrees, DirectionalLight, FrameInput,
    FrameOutput, GUI, Srgba, Texture2DRef, TextureData, Vec3, Viewport,
    window::{Window, WindowSettings},
};

use gui::SimState;

use self::body::Body;
use self::control::CameraControl;
use self::universe::Universe;
#[path = "assets/mod.rs"]
mod assets;
#[path = "autoscaling_sprites.rs"]
mod autoscaling_sprites;
#[path = "body.rs"]
mod body;
#[path = "control.rs"]
mod control;
#[path = "gui/mod.rs"]
mod gui;
#[path = "scene.rs"]
mod scene;
#[path = "units/mod.rs"]
mod units;
#[path = "universe.rs"]
mod universe;

#[cfg(not(target_family = "wasm"))]
::smol_macros::main! {
    async fn main() {
        run().await;
    }
}

#[cfg(target_family = "wasm")]
#[allow(dead_code)]
fn main() {
    unreachable!();
}

const CIRCLE_TEX_SIZE: usize = 64;

pub(crate) struct Program {
    window: Option<Window>,
    context: Context,
    camera: Camera,
    control: CameraControl,
    gui: GUI,

    top_light: DirectionalLight,
    ambient_light: AmbientLight,

    sim_state: SimState,

    circle_tex: Texture2DRef,
}

impl Program {
    fn new_window() -> Window {
        let res = Window::new(WindowSettings {
            title: "Keplerian Orbital Simulator Demo".into(),
            min_size: (64, 64),
            ..Default::default()
        });
        match res {
            Ok(w) => w,
            Err(e) => {
                if cfg!(target_family = "wasm") {
                    panic!("Error when creating window: {e}");
                } else {
                    println!("Error when creating window: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
    fn new_camera(viewport: Viewport) -> Camera {
        Camera::new_perspective(
            viewport,
            Vec3::new(3000.0, 2500.0, 6000.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Degrees { 0: 45.0 },
            1.0,
            5e16,
        )
    }
    fn new_control() -> CameraControl {
        CameraControl::new(Vec3::new(0.0, 0.0, 0.0), 100.0, 1e10, 7000.0)
    }
    fn new_dir_light(context: &Context) -> DirectionalLight {
        DirectionalLight::new(&context, 1.0, Srgba::WHITE, Vec3::new(0.0, -0.5, -0.5))
    }
    fn new_ambient_light(context: &Context) -> AmbientLight {
        AmbientLight::new(&context, 0.02, Srgba::WHITE)
    }
    fn generate_circle_tex(context: &Context) -> Texture2DRef {
        const CENTER: f32 = CIRCLE_TEX_SIZE as f32 - 1.0 / 4.0;
        const RADIUS: f32 = CENTER;

        let mut vec = Vec::with_capacity(CIRCLE_TEX_SIZE * CIRCLE_TEX_SIZE);
        for y in 0..CIRCLE_TEX_SIZE {
            for x in 0..CIRCLE_TEX_SIZE {
                let dx = 2.0 * x as f32 - CENTER;
                let dy = 2.0 * y as f32 - CENTER;
                let dist = dx.hypot(dy);

                if dist >= RADIUS {
                    vec.push([255, 255, 255, 0]);
                } else {
                    vec.push([255, 255, 255, 255]);
                }
            }
        }

        let cpu_texture = CpuTexture {
            width: CIRCLE_TEX_SIZE as u32,
            height: CIRCLE_TEX_SIZE as u32,
            data: TextureData::RgbaU8(vec),
            ..Default::default()
        };

        Texture2DRef::from_cpu_texture(context, &cpu_texture)
    }

    fn generate_sim_state() -> SimState {
        SimState::new(Self::generate_universe())
    }

    fn generate_universe() -> Universe {
        let mut universe = Universe::default();
        let root_id = universe
            .add_body(
                Body {
                    name: "Root".into(),
                    mass: 1e15,
                    radius: 100.0,
                    color: Srgba::new_opaque(255, 255, 255),
                    orbit: None,
                },
                None,
            )
            .unwrap();
        universe
            .add_body(
                Body {
                    name: "Alpha".into(),
                    mass: 1e12,
                    radius: 30.0,
                    color: Srgba::new_opaque(255, 196, 196),
                    orbit: Some(Orbit::new(0.0, 400.0, 0.0, 0.0, 0.0, 0.0, 1.0)),
                },
                Some(root_id),
            )
            .unwrap();
        let beta_id = universe
            .add_body(
                Body {
                    name: "Beta".into(),
                    mass: 1e12,
                    radius: 30.0,
                    color: Srgba::new_opaque(196, 196, 255),
                    orbit: Some(Orbit::new(0.7, 600.0, 0.0, 0.0, 0.0, 0.0, 1.0)),
                },
                Some(root_id),
            )
            .unwrap();
        universe
            .add_body(
                Body {
                    name: "Beta A".into(),
                    mass: 1e5,
                    radius: 3.0,
                    color: Srgba::new_opaque(196, 255, 196),
                    orbit: Some(Orbit::new(0.1, 40.0, 1.0, 1.0, 1.0, 1.0, 1.0)),
                },
                Some(beta_id),
            )
            .unwrap();
        universe
            .add_body(
                Body {
                    name: "Rogue".into(),
                    mass: 1e8,
                    radius: 8.0,
                    color: Srgba::new_opaque(255, 196, 255),
                    orbit: Some(Orbit::new(1.1, 120.0, 2.0, 2.0, 2.0, -4.0, 1.0)),
                },
                Some(root_id),
            )
            .unwrap();

        universe
    }

    pub(crate) fn new() -> Self {
        let window = Self::new_window();
        let context = window.gl();
        let camera = Self::new_camera(window.viewport());
        let control = Self::new_control();
        let gui = gui::create(&context);

        let top_light = Self::new_dir_light(&context);
        let ambient_light = Self::new_ambient_light(&context);

        let sim_state = Self::generate_sim_state();

        let circle_tex = Self::generate_circle_tex(&context);

        Self {
            window: Some(window),
            context,
            camera,
            control,
            gui,
            top_light,
            ambient_light,
            sim_state,
            circle_tex,
        }
    }

    pub(crate) fn run(mut self) {
        if let Some(window) = self.window.take() {
            window.render_loop(move |frame_input| self.tick(frame_input));
        }
    }

    fn tick(&mut self, mut frame_input: FrameInput) -> FrameOutput {
        if self.sim_state.running {
            self.sim_state
                .universe
                .tick(self.sim_state.sim_speed * frame_input.elapsed_time / 1000.0);
        }
        self.sim_state.focus_offset *= (-0.025 * frame_input.elapsed_time).exp();
        let position_map = self.sim_state.universe.get_all_body_positions();

        gui::update(
            &mut self.gui,
            &mut self.sim_state,
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            frame_input.elapsed_time,
            &position_map,
        );

        self.camera.set_viewport(frame_input.viewport);
        self.control.min_distance = self
            .sim_state
            .universe
            .get_body(self.sim_state.focused_body())
            .map(|wrapper| 1.5 * wrapper.body.radius)
            .unwrap_or(1e-3) as f32;
        self.control.handle_events(
            &mut self.camera,
            &mut frame_input.events,
            frame_input.elapsed_time as f32,
        );

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 100000.0))
            .render(
                &self.camera,
                &self.generate_scene(&position_map),
                &[&self.top_light, &self.ambient_light],
            )
            .write(|| self.gui.render())
            .unwrap();

        FrameOutput::default()
    }
}

pub async fn run() {
    Program::new().run();
}
