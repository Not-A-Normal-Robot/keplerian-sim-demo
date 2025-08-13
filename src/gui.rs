use std::sync::LazyLock;

use super::assets;
use super::universe::Universe;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{
        self, Area, Color32, Context as EguiContext, FontId, Frame, Grid, Id, Image, ImageButton,
        Label, Margin, RichText, Rounding, Sense, Stroke, TextWrapMode, TopBottomPanel, Ui, Vec2,
    },
};

#[path = "time.rs"]
mod time;

use time::TimeDisplay;

const FPS_AREA_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(0xFEED_A_DEFEA7ED_FAE).unwrap();
const BOTTOM_PANEL_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"BluRigel")).unwrap();
const BOTTOM_PANEL_GRID_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"Solstice")).unwrap();

const FPS_AREA_ID: LazyLock<Id> = LazyLock::new(|| Id::new(FPS_AREA_SALT));
const BOTTOM_PANEL_ID: LazyLock<Id> = LazyLock::new(|| Id::new(BOTTOM_PANEL_SALT));

struct UiState {
    time_disp: TimeDisplay,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            time_disp: TimeDisplay::SingleUnit,
        }
    }
}

pub(crate) struct SimState {
    pub universe: Universe,
    pub sim_speed: f64,
    pub running: bool,
    ui_state: UiState,
}

impl SimState {
    pub(crate) fn new(universe: Universe) -> Self {
        Self {
            universe,
            ..Default::default()
        }
    }
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            universe: Universe::default(),
            sim_speed: 1.0,
            running: true,
            ui_state: UiState::default(),
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
    let height = 64.0 * device_pixel_ratio;
    // TODO: Bottom panel expansion using show_animated
    TopBottomPanel::bottom(*BOTTOM_PANEL_ID)
        .show_separator_line(false)
        .exact_height(height)
        .frame(Frame {
            inner_margin: Margin::symmetric(16.0, 8.0),
            fill: Color32::from_black_alpha(128),
            ..Default::default()
        })
        .show(ctx, |ui| {
            bottom_panel_contents(ui, device_pixel_ratio, sim_state)
        });
}

fn bottom_panel_contents(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    ui.horizontal(|ui| {
        ui.set_height(48.0 * device_pixel_ratio);
        ui.spacing_mut().item_spacing = Vec2::new(24.0 * device_pixel_ratio, 0.0);
        pause_button(ui, device_pixel_ratio, sim_state);
        time_display(ui, device_pixel_ratio, sim_state);
    });
}

fn pause_button(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    let min_touch_size = 48.0 * device_pixel_ratio;
    let min_touch_target = Vec2::splat(min_touch_size);

    let image: &Image<'static> = if sim_state.running {
        &*assets::PAUSED_IMAGE
    } else {
        &*assets::PLAY_IMAGE
    };

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::ZERO;
        let widget_styles = &mut ui.visuals_mut().widgets;
        widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widget_styles.inactive.bg_stroke = Stroke::NONE;
        widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(8);
        widget_styles.hovered.bg_stroke = Stroke::NONE;
        widget_styles.active.weak_bg_fill = Color32::from_white_alpha(32);

        let button = ImageButton::new(image.clone().max_size(min_touch_target))
            .rounding(Rounding::same(min_touch_size));

        let button_instance = ui.add(button);
        if button_instance.clicked {
            sim_state.running = !sim_state.running;
        }
    });
}

fn time_display(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    let min_touch_size = 48.0 * device_pixel_ratio;
    let _min_touch_target = Vec2::splat(min_touch_size);

    let string = sim_state
        .ui_state
        .time_disp
        .format_time(sim_state.universe.time);

    let text = RichText::new(string)
        .color(Color32::WHITE)
        .size(16.0 * device_pixel_ratio);

    let label = Label::new(text)
        .wrap_mode(TextWrapMode::Extend)
        .selectable(false);
    ui.add(label);
}
