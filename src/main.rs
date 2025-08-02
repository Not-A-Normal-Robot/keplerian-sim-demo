use three_d::{
    AmbientLight, Axes, Camera, ClearState, CpuMaterial, CpuMesh, Degrees, DirectionalLight,
    FrameOutput, Gm, Mesh, OrbitControl, PhysicalMaterial, Srgba, Vec3,
    egui::{self, Color32, FontId, Label, RichText},
    window::{Window, WindowSettings},
};

#[path = "gui.rs"]
mod gui;
#[path = "scene.rs"]
mod scene;

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

pub async fn run() {
    let window = {
        let res = Window::new(WindowSettings {
            title: "Keplerian Orbital Simulator Demo".into(),
            min_size: (64, 64),
            ..Default::default()
        });
        match res {
            Ok(w) => w,
            Err(e) => {
                println!("Error when creating window: {e}");
                std::process::exit(1);
            }
        }
    };
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        Vec3::new(3.0, 2.5, 6.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Degrees { 0: 45.0 },
        0.1,
        1000.0,
    );

    let mut control = OrbitControl::new(camera.target(), 1.0, 1000.0);

    let sphere = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(16)),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 43,
                    g: 89,
                    b: 200,
                    a: 200,
                },
                ..Default::default()
            },
        ),
    );

    let axes = Axes::new(&context, 0.1, 2.0);

    let top_light = DirectionalLight::new(&context, 1.0, Srgba::WHITE, Vec3::new(0.0, -0.5, -0.5));
    let ambient_light = AmbientLight::new(&context, 0.02, Srgba::WHITE);

    let mut gui = gui::create(&context);

    window.render_loop(move |mut frame_input| {
        gui::update(
            &mut gui,
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            frame_input.elapsed_time,
        );

        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.7, 0.7, 0.7, 1.0, 100000.0))
            .render(
                &camera,
                sphere.into_iter().chain(&axes),
                &[&top_light, &ambient_light],
            )
            .write(|| gui.render())
            .unwrap();

        FrameOutput::default()
    });
}
