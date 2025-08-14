use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
    sync::{Arc, LazyLock},
};

use super::assets;
use super::universe::Universe;
use ordered_float::NotNan;
use strum::IntoEnumIterator;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{
        self, Area, Button, Color32, ComboBox, Context as EguiContext, DragValue, FontId, Frame,
        Id, Image, ImageButton, Label, Margin, Response, RichText, Rounding, ScrollArea,
        SelectableLabel, Slider, Stroke, TopBottomPanel, Ui, Vec2,
    },
};

#[path = "time.rs"]
mod time;

use time::{TimeDisplay, TimeUnit};

const FPS_AREA_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(0xFEED_A_DEFEA7ED_FAE).unwrap();
const BOTTOM_PANEL_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"BluRigel")).unwrap();
const TIME_CONTROL_COMBO_BOX_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"Solstice")).unwrap();

const FPS_AREA_ID: LazyLock<Id> = LazyLock::new(|| Id::new(FPS_AREA_SALT));
const BOTTOM_PANEL_ID: LazyLock<Id> = LazyLock::new(|| Id::new(BOTTOM_PANEL_SALT));
const TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME: &'static str = "TSDVF";

struct FrameData {
    frame_len_secs: VecDeque<NotNan<f64>>,
}

impl FrameData {
    const WINDOW_SIZE: usize = 1200;

    fn new() -> Self {
        Self {
            frame_len_secs: VecDeque::with_capacity(Self::WINDOW_SIZE),
        }
    }

    /// Returns NaN if no frames recorded yet
    fn get_average_fps(&self) -> f64 {
        self.frame_len_secs.len() as f64 / *self.frame_len_secs.iter().copied().sum::<NotNan<f64>>()
    }

    /// Gets the 1% lows of FPS in the sliding window
    /// Returns NaN if no frames recorded yet
    fn get_low_average(&self) -> f64 {
        let data_amount = self.frame_len_secs.len() / 100;
        if data_amount == 0 {
            return f64::NAN;
        }

        let mut heap = BinaryHeap::with_capacity(data_amount + 1);

        for &time in &self.frame_len_secs {
            heap.push(Reverse(time));

            if heap.len() > data_amount {
                heap.pop();
            }
        }

        heap.len() as f64 / *heap.iter().map(|&x| x.0).sum::<NotNan<f64>>()
    }

    fn insert_frame_data(&mut self, frame_duration: NotNan<f64>) {
        if self.frame_len_secs.capacity() == self.frame_len_secs.len() {
            self.frame_len_secs.pop_front();
        }

        self.frame_len_secs.push_back(frame_duration);
    }
}

struct UiState {
    time_disp: TimeDisplay,
    time_slider_pos: f64,
    time_speed_amount: f64,
    time_speed_unit: TimeUnit,
    time_speed_unit_auto: bool,
    frame_data: FrameData,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            time_disp: TimeDisplay::SingleUnit,
            time_slider_pos: 0.0,
            time_speed_amount: 1.0,
            time_speed_unit: TimeUnit::Seconds,
            time_speed_unit_auto: true,
            frame_data: FrameData::new(),
        }
    }
}

pub(crate) struct SimState {
    pub universe: Universe,
    pub sim_speed: f64,
    pub running: bool,
    ui: UiState,
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
            ui: UiState::default(),
        }
    }
}

pub(super) fn create(context: &ThreeDContext) -> GUI {
    let gui = GUI::new(context);
    egui_extras::install_image_loaders(gui.context());
    gui.context().style_mut(|styles| {
        styles.text_styles.insert(
            egui::TextStyle::Name(TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME.into()),
            FontId::monospace(16.0),
        );
    });
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
    if let Ok(frame_duration) = NotNan::new(elapsed_time / 1000.0)
        && frame_duration.is_finite()
    {
        sim_state.ui.frame_data.insert_frame_data(frame_duration);
    }
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
    fps_area(ctx, device_pixel_ratio, &sim_state.ui.frame_data);
    bottom_panel(ctx, device_pixel_ratio, sim_state, elapsed_time);
}

fn fps_area(ctx: &EguiContext, device_pixel_ratio: f32, frame_data: &FrameData) {
    let pos = 12.0 * device_pixel_ratio;
    Area::new(*FPS_AREA_ID)
        .constrain_to(ctx.screen_rect())
        .fixed_pos((pos, pos))
        .default_width(1000.0)
        .show(&ctx, |ui| fps_inner(ui, device_pixel_ratio, frame_data));
}

fn fps_inner(ui: &mut Ui, device_pixel_ratio: f32, frame_data: &FrameData) {
    let fps = frame_data.get_average_fps();
    let low = frame_data.get_low_average();

    let string = if low.is_nan() {
        format!("FPS: {fps:.0}")
    } else {
        format!("FPS: {fps:.0}\n1%L: {low:.0}")
    };
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

fn bottom_panel(
    ctx: &EguiContext,
    device_pixel_ratio: f32,
    sim_state: &mut SimState,
    elapsed_time: f64,
) {
    let height = 64.0 * device_pixel_ratio;
    // TODO: Bottom panel expansion using show_animated
    TopBottomPanel::bottom(*BOTTOM_PANEL_ID)
        .show_separator_line(false)
        .exact_height(height)
        .frame(Frame {
            inner_margin: Margin {
                top: 8.0 * device_pixel_ratio,
                ..Default::default()
            },
            fill: Color32::from_black_alpha(192),
            ..Default::default()
        })
        .show(ctx, |ui| {
            bottom_panel_contents(ui, device_pixel_ratio, sim_state, elapsed_time)
        });
}

fn bottom_panel_contents(
    ui: &mut Ui,
    device_pixel_ratio: f32,
    sim_state: &mut SimState,
    elapsed_time: f64,
) {
    let scroll_area = ScrollArea::horizontal().auto_shrink([false, false]);
    scroll_area.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.set_height(48.0 * device_pixel_ratio);
            ui.add_space(16.0 * device_pixel_ratio);
            pause_button(ui, device_pixel_ratio, sim_state);
            // ui.add_space(12.0 * device_pixel_ratio);
            time_display(ui, device_pixel_ratio, sim_state);
            ui.add_space(12.0 * device_pixel_ratio);
            ui.separator();
            ui.add_space(12.0 * device_pixel_ratio);
            time_control(ui, device_pixel_ratio, sim_state, elapsed_time);
            ui.add_space(16.0 * device_pixel_ratio);
        })
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

    let hover_string = match sim_state.running {
        true => "Currently running\nClick/tap to pause",
        false => "Currently paused\nClick/tap to resume",
    };
    let hover_text = RichText::new(hover_string)
        .color(Color32::WHITE)
        .size(16.0 * device_pixel_ratio);

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

        let button_instance = ui.add(button).on_hover_text(hover_text);
        if button_instance.clicked {
            sim_state.running = !sim_state.running;
        }
    });
}

fn time_display(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    let min_touch_size = 48.0 * device_pixel_ratio;
    let display_size = Vec2::new(200.0 * device_pixel_ratio, min_touch_size);

    let string = sim_state.ui.time_disp.format_time(sim_state.universe.time);

    let text = RichText::new(string)
        .monospace()
        .color(Color32::WHITE)
        .size(16.0 * device_pixel_ratio);

    let hover_string = format!(
        "Currently in {} mode\nLeft click to cycle, right click to cycle backwards",
        sim_state.ui.time_disp
    );

    let hover_text = RichText::new(hover_string)
        .color(Color32::WHITE)
        .size(16.0 * device_pixel_ratio);

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::ZERO;
        let widget_styles = &mut ui.visuals_mut().widgets;
        widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widget_styles.inactive.bg_stroke = Stroke::NONE;
        widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(8);
        widget_styles.hovered.bg_stroke = Stroke::NONE;
        widget_styles.active.weak_bg_fill = Color32::from_white_alpha(32);

        let button = Button::new(text).wrap().min_size(display_size);
        let button_instance = ui.add(button).on_hover_text(hover_text);

        if button_instance.clicked() {
            sim_state.ui.time_disp = sim_state.ui.time_disp.get_next();
        }
        if button_instance.secondary_clicked() {
            sim_state.ui.time_disp = sim_state.ui.time_disp.get_prev();
        }
    });
}

fn time_control(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState, elapsed_time: f64) {
    ui.scope(|ui| {
        time_slider(ui, device_pixel_ratio, sim_state, elapsed_time);
        time_drag_value(ui, device_pixel_ratio, sim_state);
        time_unit_box(ui, device_pixel_ratio, sim_state);
    });
}

fn time_slider(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState, elapsed_time: f64) {
    let hover_text = RichText::new(
        "Move the slider left to decelerate time.\n\
        Move the slider right to accelerate time.\n\
        Let go to stop changing time.",
    )
    .color(Color32::WHITE)
    .size(16.0 * device_pixel_ratio);
    ui.spacing_mut().interact_size.y = 48.0;
    let slider = Slider::new(&mut sim_state.ui.time_slider_pos, -1.0..=1.0)
        .show_value(false)
        .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.3 });
    let slider_instance = ui.add(slider).on_hover_text(hover_text);

    if slider_instance.is_pointer_button_down_on() {
        let base = 10.0f64.powf(sim_state.ui.time_slider_pos);
        sim_state.sim_speed *= base.powf(elapsed_time / 1000.0);
    } else {
        sim_state.ui.time_slider_pos *= (-5.0 * elapsed_time / 1000.0).exp();
    }
}
fn time_drag_value(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    sim_state.ui.time_speed_amount = sim_state.sim_speed / sim_state.ui.time_speed_unit.get_value();
    let prev_speed_amt = sim_state.ui.time_speed_amount;

    let dv_instance = ui
        .scope(|ui| time_drag_value_inner(ui, device_pixel_ratio, sim_state))
        .inner;

    let hover_text = RichText::new(
        "Drag left to slow down time.\n\
        Drag right to speed up time.\n\
        Click/tap to enter in an amount manually.",
    )
    .color(Color32::WHITE)
    .size(16.0 * device_pixel_ratio);
    let dv_instance = dv_instance.on_hover_text(hover_text);

    if prev_speed_amt != sim_state.ui.time_speed_amount {
        sim_state.sim_speed =
            sim_state.ui.time_speed_amount * sim_state.ui.time_speed_unit.get_value();
    }

    if sim_state.ui.time_speed_unit_auto && !dv_instance.dragged() {
        sim_state.ui.time_speed_unit = TimeUnit::largest_unit_from_seconds(sim_state.sim_speed);
        sim_state.ui.time_speed_amount =
            sim_state.sim_speed / sim_state.ui.time_speed_unit.get_value();
    }
}
fn time_drag_value_inner(
    ui: &mut Ui,
    device_pixel_ratio: f32,
    sim_state: &mut SimState,
) -> Response {
    let dv_size = Vec2::new(96.0 * device_pixel_ratio, 48.0 * device_pixel_ratio);
    let style_name: Arc<str> = TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME.into();
    ui.style_mut().text_styles.insert(
        egui::TextStyle::Name(style_name.clone()),
        FontId::monospace(16.0 * device_pixel_ratio),
    );
    ui.style_mut().drag_value_text_style = egui::TextStyle::Name(style_name);
    ui.spacing_mut().button_padding =
        Vec2::new(16.0 * device_pixel_ratio, 8.0 * device_pixel_ratio);
    let widget_styles = &mut ui.visuals_mut().widgets;
    widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
    widget_styles.inactive.bg_stroke = Stroke::NONE;
    widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(8);
    widget_styles.hovered.bg_stroke = Stroke::NONE;
    widget_styles.active.weak_bg_fill = Color32::from_white_alpha(32);

    let drag_value =
        DragValue::new(&mut sim_state.ui.time_speed_amount).update_while_editing(false);
    ui.add_sized(dv_size, drag_value)
}
fn time_unit_box(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    let min_touch_len = 48.0 * device_pixel_ratio;
    let unit_string = format!("{}/s", sim_state.ui.time_speed_unit);

    let unit_text = RichText::new(unit_string)
        .color(Color32::WHITE)
        .size(16.0 * device_pixel_ratio);

    let hover_text =
        RichText::new("Pick a different time speed unit or disable automatic unit selection")
            .color(Color32::WHITE)
            .size(16.0 * device_pixel_ratio);

    ui.spacing_mut().interact_size.y = min_touch_len;
    ui.spacing_mut().button_padding.x = 16.0 * device_pixel_ratio;
    ComboBox::from_id_salt(TIME_CONTROL_COMBO_BOX_SALT)
        .selected_text(unit_text)
        .truncate()
        .height(f32::INFINITY)
        .show_ui(ui, |ui| {
            time_unit_box_inner(ui, device_pixel_ratio, sim_state)
        })
        .response
        .on_hover_text(hover_text);
}
fn time_unit_box_inner(ui: &mut Ui, device_pixel_ratio: f32, sim_state: &mut SimState) {
    let min_touch_len = 48.0 * device_pixel_ratio;
    let min_touch_vec = Vec2::splat(min_touch_len);
    let font = FontId::proportional(16.0 * device_pixel_ratio);

    for unit in TimeUnit::iter() {
        let string = format!("{unit}/s");
        let text = RichText::new(string).font(font.clone());

        let label = SelectableLabel::new(sim_state.ui.time_speed_unit == unit, text);
        let label = ui.add_sized(min_touch_vec, label);

        if label.clicked() {
            sim_state.ui.time_speed_unit_auto = false;
            sim_state.ui.time_speed_unit = unit;
        }
    }

    ui.separator();

    let text = RichText::new("Auto-pick").font(font);
    let label = SelectableLabel::new(sim_state.ui.time_speed_unit_auto, text);
    let auto = ui.add_sized(min_touch_vec, label);
    if auto.clicked() {
        sim_state.ui.time_speed_unit_auto = !sim_state.ui.time_speed_unit_auto;
    }
}
