use std::sync::LazyLock;

use super::assets;
use super::universe::Universe;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{
        self, Area, Color32, Context as EguiContext, FontId, Id, Image, ImageButton, Label,
        RichText, TopBottomPanel, Ui, Vec2,
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
    pub running: bool,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            universe: Universe::default(),
            sim_speed: 1.0,
            running: true,
        }
    }
}

pub(super) fn create(context: &ThreeDContext) -> GUI {
    let gui = GUI::new(context);
    egui_extras::install_image_loaders(gui.context());
    gui
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
        |ctx| handle_ui(ctx, device_pixel_ratio, elapsed_time, sim_state),
    )
}

fn handle_ui(
    ctx: &EguiContext,
    device_pixel_ratio: f32,
    elapsed_time: f64,
    sim_state: &mut SimState,
) {
    fps_area(ctx, device_pixel_ratio, elapsed_time);
    bottom_panel(ctx, device_pixel_ratio, sim_state);
}

fn fps_area(ctx: &EguiContext, device_pixel_ratio: f32, elapsed_time: f64) {
    let pos = 12.0 * device_pixel_ratio;
    Area::new(*FPS_AREA_ID)
        .constrain_to(ctx.screen_rect())
        .fixed_pos((pos, pos))
        .default_width(1000.0)
        .show(&ctx, |ui| fps_inner(ui, device_pixel_ratio, elapsed_time));
}

fn fps_inner(ui: &mut Ui, device_pixel_ratio: f32, elapsed_time: f64) {
    let string = format!("{:.0}", 1000.0 / elapsed_time);
    const BACKGROUND_COLOR: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 128);
    let font = FontId::monospace(11.0 * device_pixel_ratio);
    let text = RichText::new(string)
        .background_color(BACKGROUND_COLOR)
        .color(Color32::WHITE)
        .font(font);
    let label = Label::new(text)
        .wrap_mode(egui::TextWrapMode::Extend)
        .selectable(false);
    ui.add(label);
}

fn bottom_panel(ctx: &EguiContext, device_pixel_ratio: f32, sim_state: &mut SimState) {
    // let height = 56.0 * device_pixel_ratio;
    TopBottomPanel::bottom(*BOTTOM_PANEL_ID)
        // .exact_height(height)
        .show(ctx, |ui| {
            bottom_panel_contents(ui, device_pixel_ratio, sim_state)
        });
}

fn bottom_panel_contents(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    let min_touch_length = 48.0 * device_pixel_ratio;
    let _min_touch_target: Vec2 = (min_touch_length, min_touch_length).into();

    let image: &Image<'static> = if sim_state.running {
        &*assets::PLAY_IMAGE
    } else {
        &*assets::PAUSED_IMAGE
    };

    let button = ImageButton::new(image.clone());
    ui.add(button);
}
