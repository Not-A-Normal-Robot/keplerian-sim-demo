use glam::{DVec3, Vec3A};
use keplerian_sim::Orbit;
use three_d::{
    AmbientLight, Axes, Camera, ClearState, Context, Degrees, DirectionalLight, FrameInput,
    FrameOutput, GUI, OrbitControl, Srgba, Vec3, Viewport,
    window::{Window, WindowSettings},
};

use self::body::Body;

use self::universe::Universe;

#[path = "body.rs"]
mod body;
#[path = "gui.rs"]
mod gui;
#[path = "scene.rs"]
mod scene;
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

pub(crate) struct Program {
    window: Option<Window>,
    context: Context,
    camera: Camera,
    camera_focus_pos: DVec3,
    control: OrbitControl,
    gui: GUI,
    last_frame_input: FrameInput,

    top_light: DirectionalLight,
    ambient_light: AmbientLight,

    universe: Universe,
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
            Vec3::new(3.0, 2.5, 6.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Degrees { 0: 45.0 },
            0.1,
            1e30,
        )
    }
    fn new_control() -> OrbitControl {
        OrbitControl::new(Vec3::new(0.0, 0.0, 0.0), 100.0, 1000.0)
    }
    fn new_dir_light(context: &Context) -> DirectionalLight {
        DirectionalLight::new(&context, 1.0, Srgba::WHITE, Vec3::new(0.0, -0.5, -0.5))
    }
    fn new_ambient_light(context: &Context) -> AmbientLight {
        AmbientLight::new(&context, 0.02, Srgba::WHITE)
    }

    pub(crate) fn get_camera_pos_64(&self) -> DVec3 {
        let offset = self.camera.position();
        self.camera_focus_pos
            + DVec3 {
                x: offset.x as f64,
                y: offset.y as f64,
                z: offset.z as f64,
            }
    }

    pub(crate) fn get_camera_pos_32(&self) -> Vec3 {
        let foc = self.camera_focus_pos;
        self.camera.position() + Vec3::new(foc.x as f32, foc.y as f32, foc.z as f32)
    }

    pub(crate) fn new() -> Self {
        let window = Self::new_window();
        let context = window.gl();
        let camera = Self::new_camera(window.viewport());
        let control = Self::new_control();
        let gui = gui::create(&context);

        let top_light = Self::new_dir_light(&context);
        let ambient_light = Self::new_ambient_light(&context);

        let mut universe = Universe::default();
        let root_id = universe
            .add_body(
                Body {
                    name: "Root".into(),
                    mass: 1e18,
                    radius: 100.0,
                    color: Srgba::BLUE,
                    orbit: None,
                },
                None,
            )
            .unwrap();
        universe
            .add_body(
                Body {
                    name: "Child".into(),
                    mass: 1.0,
                    radius: 30.0,
                    color: Srgba::new_opaque(196, 196, 196),
                    orbit: Some(Orbit::new(0.0, 200.0, 0.0, 0.0, 0.0, 0.0, 1.0)),
                },
                Some(root_id),
            )
            .unwrap();

        Self {
            window: Some(window),
            context: context.clone(),
            camera,
            camera_focus_pos: DVec3::default(),
            control,
            gui,
            last_frame_input: FrameInput {
                events: vec![],
                elapsed_time: 1.0,
                accumulated_time: 0.0,
                viewport: Viewport {
                    x: 1,
                    y: 1,
                    width: 1,
                    height: 1,
                },
                window_width: 1,
                window_height: 1,
                device_pixel_ratio: 1.0,
                first_frame: true,
                context,
            },
            top_light,
            ambient_light,
            universe,
        }
    }

    pub(crate) fn run(mut self) {
        if let Some(window) = self.window.take() {
            window.render_loop(move |frame_input| self.tick(frame_input));
        }
    }

    fn tick(&mut self, mut frame_input: FrameInput) -> FrameOutput {
        self.last_frame_input = frame_input.clone();

        self.universe.tick(frame_input.elapsed_time / 1000.0);

        gui::update(
            &mut self.gui,
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            frame_input.elapsed_time,
        );

        self.camera.set_viewport(frame_input.viewport);
        self.control
            .handle_events(&mut self.camera, &mut frame_input.events);

        let axes = Axes::new(&self.context, 4.0, 200.0);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 100000.0))
            .render(
                &self.camera,
                (&self.generate_scene(self.get_camera_pos_64())).into_iter(),
                // .chain(axes.into_iter()),
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
