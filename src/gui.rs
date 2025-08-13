use std::sync::LazyLock;

use super::universe::Universe;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{
        self, Area, Color32, Context as EguiContext, FontId, Id, Label, RichText, TopBottomPanel,
        Ui,
    },
};

const FPS_AREA_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(0xFEED_A_DEFEA7ED_FAE).unwrap();
const BOTTOM_PANEL_SALT: std::num::NonZeroU64 = std::num::NonZeroU64::new(0xA_BAD_FAC7).unwrap();

const FPS_AREA_ID: LazyLock<Id> = LazyLock::new(|| Id::new(FPS_AREA_SALT));
const BOTTOM_PANEL_ID: LazyLock<Id> = LazyLock::new(|| Id::new(BOTTOM_PANEL_SALT));

pub(crate) struct SimState {
    pub universe: Universe,
    pub sim_speed: f64,
}

pub(super) fn create(context: &ThreeDContext) -> GUI {
    GUI::new(context)
}

pub(super) fn update(
    gui: &mut GUI,
    sim_state: &mut SimState,
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
        |ctx| handle_ui(ctx, elapsed_time, sim_state),
    )
}

fn handle_ui(ctx: &EguiContext, elapsed_time: f64, sim_state: &mut SimState) {
    fps_area(ctx, elapsed_time);
    bottom_panel(ctx, sim_state);
}

fn fps_area(ctx: &EguiContext, elapsed_time: f64) {
    Area::new(*FPS_AREA_ID)
        .constrain_to(ctx.screen_rect())
        .fixed_pos((12.0, 12.0))
        .default_width(1000.0)
        .show(&ctx, |ui| fps_inner(ui, elapsed_time));
}

fn fps_inner(ui: &mut Ui, elapsed_time: f64) {
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

fn bottom_panel(ctx: &EguiContext, _sim_state: &mut SimState) {
    TopBottomPanel::bottom(*BOTTOM_PANEL_ID)
        .exact_height(48.0)
        .show(ctx, |_ui| {});
}
