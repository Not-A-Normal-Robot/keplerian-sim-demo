use super::universe::Universe;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{self, Area, Color32, Context as EguiContext, FontId, Id, Label, RichText, Ui},
};

const FPS_AREA_ID: std::num::NonZeroU64 = std::num::NonZeroU64::new(0xFEED_A_DEFEA7ED_FAE).unwrap();
// const BOTTOM_PANEL_ID: std::num::NonZeroU64 = std::num::NonZeroU64::new(0xA_BAD_FAC7).unwrap();

pub(crate) struct SimState {
    pub universe: Universe,
    pub sim_speed: f64,
}

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
        |ctx| create_ui(ctx, elapsed_time),
    )
}

fn create_ui(ctx: &EguiContext, elapsed_time: f64) {
    Area::new(Id::new(FPS_AREA_ID))
        .constrain_to(ctx.screen_rect())
        .fixed_pos((12.0, 12.0))
        .default_width(1000.0)
        .show(&ctx, |ui| create_fps_overlay(ui, elapsed_time));

    egui::Window::new("Debug Window")
        .movable(true)
        .collapsible(true)
        .resizable(true)
        .max_size((10000.0, 10000.0))
        .show(&ctx, |ui| {
            ui.label("Hello World!");
        });
}

fn create_fps_overlay(ui: &mut Ui, elapsed_time: f64) {
    let string = format!("{:.0}", 1000.0 / elapsed_time);
    const BACKGROUND_COLOR: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 128);
    let text = RichText::new(string)
        .background_color(BACKGROUND_COLOR)
        .color(Color32::WHITE)
        .font(FontId::monospace(11.0));
    let label = Label::new(text)
        .wrap_mode(egui::TextWrapMode::Extend)
        .selectable(false);
    ui.add(label);
}
