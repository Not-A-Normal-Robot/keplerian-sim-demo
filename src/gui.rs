use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{self, Area, Color32, FontId, Id, Label, RichText},
};

const FPS_AREA_ID: std::num::NonZeroU64 = std::num::NonZeroU64::new(19823659234).unwrap();

pub(super) fn create(context: &ThreeDContext) -> GUI {
    GUI::new(context)
}

pub(super) fn update(
    gui: &mut GUI,
    events: &mut Vec<ThreeDEvent>,
    accumulated_time_ms: f64,
    viewport: Viewport,
    device_pixel_ratio: f32,
    elapsed_time: f64,
) -> bool {
    gui.update(
        events,
        accumulated_time_ms,
        viewport,
        device_pixel_ratio,
        |ctx| {
            Area::new(Id::new(FPS_AREA_ID))
                .constrain_to(ctx.screen_rect())
                .fixed_pos((12.0, 12.0))
                .default_width(1000.0)
                .show(&ctx, |ui| {
                    ui.add(
                        Label::new(
                            RichText::new(format!("{:.0}", 1000.0 / elapsed_time))
                                .background_color(Color32::from_rgba_premultiplied(0, 0, 0, 128))
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
    )
}
