use three_d::{
    AmbientLight, Axes, Camera, ClearState, CpuMaterial, CpuMesh, Degrees, DirectionalLight,
    FrameOutput, Gm, Mesh, OrbitControl, PhysicalMaterial, Srgba, Vec3,
    egui::{self, Color32, FontId, Label, RichText},
    window::{Window, WindowSettings},
};

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

const FPS_AREA_ID: std::num::NonZeroU64 = std::num::NonZeroU64::new(19823659234).unwrap();

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

    let mut gui = three_d::GUI::new(&context);

    window.render_loop(move |mut frame_input| {
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |ctx| {
                use egui::{Area, Id};
                Area::new(Id::new(FPS_AREA_ID))
                    .constrain_to(ctx.screen_rect())
                    .fixed_pos((12.0, 12.0))
                    .default_width(1000.0)
                    .show(&ctx, |ui| {
                        ui.add(
                            Label::new(
                                RichText::new(format!("{:.0}", 1000.0 / frame_input.elapsed_time))
                                    .background_color(Color32::from_rgba_premultiplied(
                                        0, 0, 0, 128,
                                    ))
                                    .color(Color32::WHITE)
                                    .font(FontId::monospace(11.0)),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend)
                            .selectable(false),
                        );
                    });

                egui::Window::new("Debug Window")
                    .movable(true)
                    .collapsible(true)
                    .resizable(true)
                    .max_size((10000.0, 10000.0))
                    .show(&ctx, |ui| {
                        ui.label("Hello World!");
                    });
            },
        );

        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 100000.0))
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
