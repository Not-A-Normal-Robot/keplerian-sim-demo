use three_d::{
    AmbientLight, Camera, ClearState, Context, Degrees, DirectionalLight, FrameInput, FrameOutput,
    GUI, InnerSpace, Srgba, Vec3, Viewport,
    window::{Window, WindowSettings},
};

use gui::SimState;

use self::control::CameraControl;
#[path = "assets/mod.rs"]
mod assets;
#[path = "cfg/mod.rs"]
mod cfg;
#[path = "control.rs"]
mod control;
#[path = "gfx/mod.rs"]
mod gfx;
#[path = "gui/mod.rs"]
mod gui;
#[path = "sim/mod.rs"]
mod sim;
#[path = "units/mod.rs"]
mod units;

static mut HALT_FLAG: bool = false;

#[cfg(not(target_family = "wasm"))]
fn main() {
    run()
}

#[cfg(target_family = "wasm")]
#[allow(dead_code)]
fn main() {
    unsafe { std::hint::unreachable_unchecked() }
}

pub(crate) struct Program {
    window: Option<Window>,
    context: Context,
    camera: Camera,
    control: CameraControl,
    gui: GUI,

    top_light: DirectionalLight,
    ambient_light: AmbientLight,

    sim_state: SimState,
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
            Vec3::new(6.2, 2.6, 4.2).normalize(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Degrees { 0: 45.0 },
            0.001,
            5e12,
        )
    }
    fn new_control() -> CameraControl {
        CameraControl::new(100.0, 1e16, 5e11)
    }
    fn new_dir_light(context: &Context) -> DirectionalLight {
        DirectionalLight::new(&context, 1.0, Srgba::WHITE, Vec3::new(0.0, -0.5, -0.5))
    }
    fn new_ambient_light(context: &Context) -> AmbientLight {
        AmbientLight::new(&context, 0.02, Srgba::WHITE)
    }
    fn generate_sim_state() -> SimState {
        SimState::new(sim::create_universe())
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

        Self {
            window: Some(window),
            context,
            camera,
            control,
            gui,
            top_light,
            ambient_light,
            sim_state,
        }
    }

    pub(crate) fn run(mut self) {
        if let Some(window) = self.window.take() {
            window.render_loop(move |frame_input| self.tick(frame_input));
        }
    }

    fn tick(&mut self, mut frame_input: FrameInput) -> FrameOutput {
        #[cfg(all(target_family = "wasm", not(feature = "is-bin")))]
        crate::web::heartbeat::update_frame_time();

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
            .unwrap_or(1e-3);
        self.control.max_distance = self.control.min_distance * 1e16;
        self.control.handle_events(
            &mut self.camera,
            &mut frame_input.events,
            frame_input.elapsed_time,
        );

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 100000.0))
            .render(
                &self.camera,
                &self.to_objects(&position_map),
                &[&self.top_light, &self.ambient_light],
            )
            .write(|| self.gui.render())
            .unwrap();

        FrameOutput {
            exit: unsafe { HALT_FLAG },
            ..Default::default()
        }
    }
}

pub fn run() {
    Program::new().run();
}
