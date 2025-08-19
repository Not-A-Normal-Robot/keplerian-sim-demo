use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    sync::{Arc, LazyLock},
};

use super::{
    assets,
    universe::{Id as UniverseId, Universe},
};
use float_pretty_print::PrettyPrintFloat;
use glam::DVec3;
use ordered_float::NotNan;
use strum::IntoEnumIterator;
use three_d::{
    Context as ThreeDContext, Event as ThreeDEvent, GUI, Viewport,
    egui::{
        self, Area, Atom, AtomLayout, Button, Color32, ComboBox, Context as EguiContext,
        CornerRadius, DragValue, FontId, Frame, Id as EguiId, Image, ImageButton, IntoAtoms, Key,
        Label, Margin, Popup, PopupCloseBehavior, Pos2, Rect, Response, RichText, ScrollArea,
        Slider, Stroke, TextEdit, TextWrapMode, TopBottomPanel, Ui, Vec2, Window,
        collapsing_header::CollapsingState,
        text::{CCursor, CCursorRange},
        text_edit::TextEditState,
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
const BODY_PREFIX_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"Planets!")).unwrap();
const CIRCLE_ICON_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"Circles!")).unwrap();
const ELLIPSIS_BUTTON_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"see_more")).unwrap();
const RENAME_TEXTEDIT_SALT: std::num::NonZeroU64 =
    std::num::NonZeroU64::new(u64::from_be_bytes(*b"OmgRen??")).unwrap();

const FPS_AREA_ID: LazyLock<EguiId> = LazyLock::new(|| EguiId::new(FPS_AREA_SALT));
const BOTTOM_PANEL_ID: LazyLock<EguiId> = LazyLock::new(|| EguiId::new(BOTTOM_PANEL_SALT));
const BODY_PREFIX_ID: LazyLock<EguiId> = LazyLock::new(|| EguiId::new(BODY_PREFIX_SALT));
const CIRCLE_ICON_ID: LazyLock<EguiId> = LazyLock::new(|| EguiId::new(CIRCLE_ICON_SALT));
const ELLIPSIS_BUTTON_ID: LazyLock<EguiId> = LazyLock::new(|| EguiId::new(ELLIPSIS_BUTTON_SALT));
const RENAME_TEXTEDIT_ID: LazyLock<EguiId> = LazyLock::new(|| EguiId::new(RENAME_TEXTEDIT_SALT));
const TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME: &'static str = "TSDVF";

const MIN_TOUCH_TARGET_LEN: f32 = 48.0;
const MIN_TOUCH_TARGET_VEC: Vec2 = Vec2::splat(MIN_TOUCH_TARGET_LEN);

mod fmt {
    use std::ops::RangeInclusive;

    use float_pretty_print::PrettyPrintFloat;

    pub(super) fn format_dv_number(number: f64, _: RangeInclusive<usize>) -> String {
        let number = PrettyPrintFloat(number);
        format!("{number:5.1}")
    }
}

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

struct RenameState {
    universe_id: UniverseId,
    name_buffer: String,
    requesting_focus: bool,
}

struct UiState {
    time_disp: TimeDisplay,
    time_slider_pos: f64,
    time_speed_amount: f64,
    time_speed_unit: TimeUnit,
    time_speed_unit_auto: bool,
    frame_data: FrameData,
    listed_body_with_popup: Option<UniverseId>,
    listed_body_with_rename: Option<RenameState>,
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
            listed_body_with_popup: None,
            listed_body_with_rename: None,
        }
    }
}

pub(crate) struct SimState {
    pub universe: Universe,
    pub sim_speed: f64,
    pub running: bool,
    focused_body: UniverseId,
    pub focus_offset: DVec3,
    ui: UiState,
}

impl SimState {
    pub(crate) fn new(universe: Universe) -> Self {
        Self {
            universe,
            ..Default::default()
        }
    }
    pub(crate) fn switch_focus(
        &mut self,
        focus_body_id: UniverseId,
        position_map: &HashMap<UniverseId, DVec3>,
    ) {
        // old_position → old_focus
        //            \      ↓
        //             \new_focus

        let old_focus = *position_map.get(&self.focused_body).unwrap_or(&DVec3::ZERO);
        let old_position = old_focus + self.focus_offset;
        let new_focus = *position_map.get(&focus_body_id).unwrap_or(&DVec3::ZERO);
        let new_offset = old_position - new_focus;
        self.focused_body = focus_body_id;
        self.focus_offset = if new_offset.is_nan() {
            DVec3::ZERO
        } else {
            new_offset
        };
    }
    #[inline]
    pub(crate) fn focused_body(&self) -> UniverseId {
        self.focused_body
    }
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            universe: Universe::default(),
            sim_speed: 1.0,
            running: true,
            focused_body: 0,
            focus_offset: DVec3::ZERO,
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
    position_map: &HashMap<UniverseId, DVec3>,
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
        |ctx| handle_ui(ctx, elapsed_time, sim_state, position_map),
    )
}

fn handle_ui(
    ctx: &EguiContext,
    elapsed_time: f64,
    sim_state: &mut SimState,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    fps_area(ctx, &sim_state.ui.frame_data);
    bottom_panel(ctx, sim_state, elapsed_time);
    body_tree_window(ctx, sim_state, position_map);
    body_edit_window(ctx, sim_state);
}

fn fps_area(ctx: &EguiContext, frame_data: &FrameData) {
    let pos = 12.0;
    Area::new(*FPS_AREA_ID)
        .constrain_to(ctx.screen_rect())
        .fixed_pos((pos, pos))
        .default_width(1000.0)
        .show(&ctx, |ui| fps_inner(ui, frame_data));
}

fn fps_inner(ui: &mut Ui, frame_data: &FrameData) {
    let fps = frame_data.get_average_fps();
    let low = frame_data.get_low_average();

    let string = if low.is_nan() {
        format!("FPS: {fps:.0}")
    } else {
        format!("FPS: {fps:.0}\n1%L: {low:.0}")
    };
    const BACKGROUND_COLOR: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 128);
    let font = FontId::monospace(11.0);
    let text = RichText::new(string)
        .background_color(BACKGROUND_COLOR)
        .color(Color32::WHITE)
        .font(font);
    let label = Label::new(text)
        .wrap_mode(egui::TextWrapMode::Extend)
        .selectable(false);
    ui.add(label);
}

fn bottom_panel(ctx: &EguiContext, sim_state: &mut SimState, elapsed_time: f64) {
    let height = 64.0;
    // TODO: Bottom panel expansion using show_animated
    TopBottomPanel::bottom(*BOTTOM_PANEL_ID)
        .show_separator_line(false)
        .exact_height(height)
        .frame(Frame {
            inner_margin: Margin {
                top: (8.0) as i8,
                ..Default::default()
            },
            fill: Color32::from_black_alpha(192),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| bottom_panel_contents(ui, sim_state, elapsed_time))
                })
        });
}

fn bottom_panel_contents(ui: &mut Ui, sim_state: &mut SimState, elapsed_time: f64) {
    ui.set_height(MIN_TOUCH_TARGET_LEN);
    ui.add_space(16.0);
    pause_button(ui, sim_state);

    if ui.available_width() > 900.0 {
        time_display(ui, sim_state);
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        time_control(ui, sim_state, elapsed_time, false);
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
    } else {
        time_manager(ui, sim_state, elapsed_time);
        ui.separator();
    }
}

fn time_manager(ui: &mut Ui, sim_state: &mut SimState, elapsed_time: f64) {
    let image: &Image<'static> = &*assets::TIME_IMAGE;

    let hover_text = RichText::new("Manage time")
        .color(Color32::WHITE)
        .size(16.0);

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::ZERO;
        let widget_styles = &mut ui.visuals_mut().widgets;
        widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widget_styles.inactive.bg_stroke = Stroke::NONE;
        widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(16);
        widget_styles.hovered.bg_stroke = Stroke::NONE;
        widget_styles.active.weak_bg_fill = Color32::from_white_alpha(64);

        let button = ImageButton::new(image.clone().fit_to_exact_size(MIN_TOUCH_TARGET_VEC))
            .corner_radius(MIN_TOUCH_TARGET_LEN);
        let button = ui.add(button).on_hover_text(hover_text);

        let popup = Popup::menu(&button).close_behavior(PopupCloseBehavior::CloseOnClickOutside);
        popup.show(|ui| {
            ui.set_max_width(200.0);
            time_display(ui, sim_state);
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            time_control(ui, sim_state, elapsed_time, true);
            ui.add_space(16.0);
        });
    });

    let string = format!(
        "{time:5.5}{unit}\n{rate:6.6}/s",
        time = PrettyPrintFloat(sim_state.universe.time / sim_state.ui.time_speed_unit.get_value()),
        unit = sim_state.ui.time_speed_unit,
        rate = PrettyPrintFloat(sim_state.sim_speed / sim_state.ui.time_speed_unit.get_value()),
    );
    let text = RichText::new(string).monospace().color(Color32::WHITE);
    ui.label(text);
}

fn pause_button(ui: &mut Ui, sim_state: &mut SimState) {
    let image: &Image<'static> = if sim_state.running {
        &*assets::PAUSED_IMAGE
    } else {
        &*assets::PLAY_IMAGE
    };

    let hover_string = match sim_state.running {
        true => "Currently running\nClick/tap to pause",
        false => "Currently paused\nClick/tap to resume",
    };
    let hover_text = RichText::new(hover_string).color(Color32::WHITE).size(16.0);

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::ZERO;
        let widget_styles = &mut ui.visuals_mut().widgets;
        widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widget_styles.inactive.bg_stroke = Stroke::NONE;
        widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(16);
        widget_styles.hovered.bg_stroke = Stroke::NONE;
        widget_styles.active.weak_bg_fill = Color32::from_white_alpha(64);

        let button = ImageButton::new(image.clone().max_size(MIN_TOUCH_TARGET_VEC))
            .corner_radius(CornerRadius::same(MIN_TOUCH_TARGET_LEN as u8));

        let button_instance = ui.add(button).on_hover_text(hover_text);
        if button_instance.clicked() {
            sim_state.running = !sim_state.running;
        }
    });
}

fn time_display(ui: &mut Ui, sim_state: &mut SimState) {
    let display_size = Vec2::new(220.0, MIN_TOUCH_TARGET_LEN);

    let string = sim_state.ui.time_disp.format_time(sim_state.universe.time);

    let text = RichText::new(string)
        .monospace()
        .color(Color32::WHITE)
        .size(16.0);

    let hover_string = format!(
        "Currently in {} mode\nLeft click to cycle, right click to cycle backwards",
        sim_state.ui.time_disp
    );

    let hover_text = RichText::new(hover_string).color(Color32::WHITE).size(16.0);

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::ZERO;
        let widget_styles = &mut ui.visuals_mut().widgets;
        widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widget_styles.inactive.bg_stroke = Stroke::NONE;
        widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(16);
        widget_styles.hovered.bg_stroke = Stroke::NONE;
        widget_styles.active.weak_bg_fill = Color32::from_white_alpha(64);

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

fn time_control(ui: &mut Ui, sim_state: &mut SimState, elapsed_time: f64, column_mode: bool) {
    ui.scope(|ui| {
        time_slider(ui, sim_state, elapsed_time, column_mode);
        time_drag_value(ui, sim_state);
        if column_mode {
            time_unit_box_popup(ui, sim_state);
        } else {
            time_unit_box(ui, sim_state);
        }
    });
}

fn time_slider(ui: &mut Ui, sim_state: &mut SimState, elapsed_time: f64, column_mode: bool) {
    let hover_text = RichText::new(
        "Move the slider left to decelerate time.\n\
        Move the slider right to accelerate time.\n\
        Let go to stop changing time.",
    )
    .color(Color32::WHITE)
    .size(16.0);
    ui.spacing_mut().interact_size.y = MIN_TOUCH_TARGET_LEN;
    let slider = Slider::new(&mut sim_state.ui.time_slider_pos, -1.0..=1.0)
        .show_value(false)
        .handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 0.3 });

    if column_mode {
        ui.spacing_mut().slider_width = ui.available_width();
    }

    let slider_instance = ui.add(slider).on_hover_text(hover_text);

    if slider_instance.is_pointer_button_down_on() {
        let base = 10.0f64.powf(sim_state.ui.time_slider_pos);
        sim_state.sim_speed *= base.powf(elapsed_time / 1000.0);
    } else {
        sim_state.ui.time_slider_pos *= (-5.0 * elapsed_time / 1000.0).exp();
    }
}
fn time_drag_value(ui: &mut Ui, sim_state: &mut SimState) {
    sim_state.ui.time_speed_amount = sim_state.sim_speed / sim_state.ui.time_speed_unit.get_value();
    let prev_speed_amt = sim_state.ui.time_speed_amount;

    let dv_instance = ui.scope(|ui| time_drag_value_inner(ui, sim_state)).inner;

    let hover_text = RichText::new(
        "Drag left to slow down time.\n\
        Drag right to speed up time.\n\
        Click/tap to enter in an amount manually.",
    )
    .color(Color32::WHITE)
    .size(16.0);
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
fn time_drag_value_inner(ui: &mut Ui, sim_state: &mut SimState) -> Response {
    let dv_size = Vec2::new(MIN_TOUCH_TARGET_LEN * 2.0, MIN_TOUCH_TARGET_LEN);
    let style_name: Arc<str> = TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME.into();
    ui.style_mut().text_styles.insert(
        egui::TextStyle::Name(style_name.clone()),
        FontId::monospace(16.0),
    );
    ui.style_mut().drag_value_text_style = egui::TextStyle::Name(style_name);
    ui.spacing_mut().button_padding = Vec2::new(16.0, 8.0);
    let widget_styles = &mut ui.visuals_mut().widgets;
    widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
    widget_styles.inactive.bg_stroke = Stroke::NONE;
    widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(16);
    widget_styles.hovered.bg_stroke = Stroke::NONE;
    widget_styles.active.weak_bg_fill = Color32::from_white_alpha(64);

    let drag_value = DragValue::new(&mut sim_state.ui.time_speed_amount)
        .update_while_editing(false)
        .custom_formatter(fmt::format_dv_number);
    ui.add_sized(dv_size, drag_value)
}
fn time_unit_box(ui: &mut Ui, sim_state: &mut SimState) {
    let unit_string = format!("{}/s", sim_state.ui.time_speed_unit);

    let unit_text = RichText::new(unit_string).color(Color32::WHITE).size(16.0);

    let hover_text =
        RichText::new("Pick a different time speed unit or disable automatic unit selection")
            .color(Color32::WHITE)
            .size(16.0);

    ui.spacing_mut().interact_size.y = MIN_TOUCH_TARGET_LEN;
    ui.spacing_mut().button_padding.x = 16.0;

    ComboBox::from_id_salt(TIME_CONTROL_COMBO_BOX_SALT)
        .selected_text(unit_text)
        .height(f32::INFINITY)
        .show_ui(ui, |ui| time_unit_box_inner(ui, sim_state, true))
        .response
        .on_hover_text(hover_text);
}
fn time_unit_box_inner(ui: &mut Ui, sim_state: &mut SimState, per_second: bool) {
    let font = FontId::proportional(16.0);

    for unit in TimeUnit::iter() {
        let string = if per_second {
            format!("{unit}/s")
        } else {
            unit.to_string()
        };
        let text = RichText::new(string).font(font.clone());

        let label = Button::selectable(sim_state.ui.time_speed_unit == unit, text);
        let label = ui.add_sized(MIN_TOUCH_TARGET_VEC, label);

        if label.clicked() {
            sim_state.ui.time_speed_unit_auto = false;
            sim_state.ui.time_speed_unit = unit;
        }
    }

    ui.separator();

    let text = RichText::new("Auto-pick").font(font);
    let label = Button::selectable(sim_state.ui.time_speed_unit_auto, text);
    let auto = ui.add_sized(MIN_TOUCH_TARGET_VEC, label);
    if auto.clicked() {
        sim_state.ui.time_speed_unit_auto = !sim_state.ui.time_speed_unit_auto;
    }
}
fn time_unit_box_popup(ui: &mut Ui, sim_state: &mut SimState) {
    let font = FontId::proportional(16.0);

    let unit = sim_state.ui.time_speed_unit;
    let title_string = if sim_state.ui.time_speed_unit_auto {
        format!("Select unit ({unit}; auto)")
    } else {
        format!("Select unit ({unit})")
    };

    let title_text = RichText::new(title_string)
        .color(Color32::WHITE)
        .font(font.clone());

    ui.menu_button(title_text, |ui| {
        time_unit_box_inner(ui, sim_state, false);
    });
}

fn get_body_egui_id(universe_id: UniverseId) -> EguiId {
    BODY_PREFIX_ID.with(universe_id)
}

const BODY_TREE_ICON_SIZE: f32 = 16.0;
fn body_tree_window(
    ctx: &EguiContext,
    sim_state: &mut SimState,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let window = Window::new("Celestial Bodies").scroll(true);
    window.show(ctx, |ui| {
        ui.scope(|ui| {
            body_tree_window_contents(ui, sim_state, position_map);
        })
    });
}

fn body_tree_window_contents(
    ui: &mut Ui,
    sim_state: &mut SimState,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let roots: Box<[UniverseId]> = sim_state
        .universe
        .get_bodies()
        .iter()
        .filter_map(|(&id, wrapper)| match wrapper.relations.parent {
            Some(_) => None,
            None => Some(id),
        })
        .collect();

    for universe_id in roots {
        body_tree_node(ui, sim_state, universe_id, position_map);
    }
}

fn body_tree_node(
    ui: &mut Ui,
    sim_state: &mut SimState,
    universe_id: UniverseId,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let satellites = match sim_state.universe.get_body(universe_id) {
        Some(wrapper) => &wrapper.relations.satellites,
        None => return,
    };

    if satellites.is_empty() {
        ui.indent((*BODY_PREFIX_ID, universe_id), |ui| {
            body_tree_base_node(ui, sim_state, universe_id, position_map);
        });
    } else {
        body_tree_parent_node(ui, sim_state, universe_id, position_map);
    }
}

fn body_tree_parent_node(
    ui: &mut Ui,
    sim_state: &mut SimState,
    universe_id: UniverseId,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let wrapper = match sim_state.universe.get_body(universe_id) {
        Some(wrapper) => wrapper,
        None => return,
    };
    let satellites = wrapper.relations.satellites.clone();

    let egui_id = get_body_egui_id(universe_id);
    CollapsingState::load_with_default_open(ui.ctx(), egui_id, true)
        .show_header(ui, |ui| {
            body_tree_base_node(ui, sim_state, universe_id, position_map);
        })
        .body(|ui| {
            for id in satellites {
                body_tree_node(ui, sim_state, id, position_map)
            }
        });
}

fn body_tree_base_node(
    ui: &mut Ui,
    sim_state: &mut SimState,
    universe_id: UniverseId,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let body = match sim_state.universe.get_body(universe_id) {
        Some(wrapper) => &wrapper.body,
        None => return,
    };

    let selected = sim_state.focused_body == universe_id;

    const RADIUS: f32 = BODY_TREE_ICON_SIZE / 2.0;
    let center = Pos2::from((RADIUS, RADIUS));
    let fill_color = body.color;
    let fill_color =
        Color32::from_rgba_unmultiplied(fill_color.r, fill_color.g, fill_color.b, fill_color.a);

    let circle_atom = Atom::custom(*CIRCLE_ICON_ID, Vec2::splat(BODY_TREE_ICON_SIZE));

    let inner_button_atom = Atom::custom(*ELLIPSIS_BUTTON_ID, Vec2::splat(BODY_TREE_ICON_SIZE));

    let mut layout = AtomLayout::new(circle_atom);

    let text = RichText::new(&body.name).color(Color32::WHITE);
    let text = if selected { text.underline() } else { text };
    layout.push_right(text);

    layout.push_right(Atom::grow());
    layout.push_right(inner_button_atom);

    let res = Button::selectable(selected, layout.atoms)
        .min_size(Vec2::new(ui.available_width(), BODY_TREE_ICON_SIZE))
        .atom_ui(ui);

    if let Some(rect) = res.rect(*CIRCLE_ICON_ID) {
        ui.painter().with_clip_rect(rect).circle_filled(
            center + rect.min.to_vec2(),
            RADIUS,
            fill_color,
        );
    }

    let mut rect = res.response.rect;
    let padding = BODY_TREE_ICON_SIZE * 1.5;
    *rect.right_mut() -= padding;
    *rect.left_mut() += padding;

    if let Some(ren_state) = &mut sim_state.ui.listed_body_with_rename
        && ren_state.universe_id == universe_id
    {
        let text_edit = TextEdit::singleline(&mut ren_state.name_buffer).id(*RENAME_TEXTEDIT_ID);

        let response = ui.put(rect, text_edit);

        if ren_state.requesting_focus {
            response.request_focus();

            if response.has_focus() {
                ren_state.requesting_focus = false;
            }
        }

        if response.lost_focus() {
            let string = sim_state
                .ui
                .listed_body_with_rename
                .take()
                .map(|s| s.name_buffer);
            if let Some(string) = string
                && !ui.input(|i| i.key_down(Key::Escape))
            {
                sim_state
                    .universe
                    .get_body_mut(universe_id)
                    .map(|w| w.body.name = string);
            }
        }
    }

    let response = &res.response;

    if response.double_clicked() {
        set_rename_state(ui, sim_state, universe_id);
    } else if response.clicked() {
        sim_state.switch_focus(universe_id, &position_map);
    }

    if let Some(rect) = res.rect(*ELLIPSIS_BUTTON_ID) {
        let inner_button = ellipsis_button(ui, rect);
        ellipsis_popup(
            sim_state,
            &inner_button,
            response,
            universe_id,
            position_map,
        );
    }
}

fn set_rename_state(ui: &mut Ui, sim_state: &mut SimState, universe_id: UniverseId) {
    let string = match sim_state.universe.get_body(universe_id) {
        Some(w) => w.body.name.clone(),
        None => return,
    };
    let strlen = string.chars().count();
    if let Some(state) = sim_state.ui.listed_body_with_rename.take() {
        sim_state
            .universe
            .get_body_mut(state.universe_id)
            .map(|w| w.body.name = state.name_buffer);
    }

    sim_state.ui.listed_body_with_rename = Some(RenameState {
        universe_id,
        name_buffer: string,
        requesting_focus: true,
    });

    let mut state = TextEditState::default();
    state.cursor.set_char_range(Some(CCursorRange::two(
        CCursor::new(0),
        CCursor::new(strlen),
    )));
    state.store(ui.ctx(), *RENAME_TEXTEDIT_ID);
}

fn ellipsis_button(ui: &mut Ui, rect: Rect) -> Response {
    let ellipsis_button = ImageButton::new(assets::ELLIPSIS_IMAGE.clone());
    ui.spacing_mut().button_padding = Vec2::ZERO;
    let widget_styles = &mut ui.visuals_mut().widgets;
    widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
    widget_styles.inactive.bg_stroke = Stroke::NONE;
    widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(64);
    widget_styles.active.weak_bg_fill = Color32::from_white_alpha(128);

    ui.put(rect, ellipsis_button)
}

fn ellipsis_popup(
    sim_state: &mut SimState,
    inner_response: &Response,
    outer_response: &Response,
    universe_id: UniverseId,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let open = sim_state.ui.listed_body_with_popup == Some(universe_id);

    if inner_response.clicked() || outer_response.secondary_clicked() {
        if open {
            sim_state.ui.listed_body_with_popup = None;
        } else {
            sim_state.ui.listed_body_with_popup = Some(universe_id);
        }
    }

    let popup = Popup::from_response(inner_response).open(open);

    #[must_use = "Show the button using ui.show()"]
    fn button<'a>(atoms: impl IntoAtoms<'a>) -> Button<'a> {
        Button::new(atoms)
            .wrap_mode(TextWrapMode::Extend)
            .right_text("")
            .frame_when_inactive(false)
    }

    #[must_use = "Check for button interaction using .clicked()"]
    fn ui_button<'a>(ui: &'a mut Ui, atoms: impl IntoAtoms<'a>) -> Response {
        let button = button(atoms);
        ui.add_sized((ui.available_width(), 16.0), button)
    }

    let popup = popup.show(|ui| {
        let body_wrapper = sim_state.universe.get_body(universe_id);
        let parent_id = body_wrapper.map(|w| w.relations.parent).flatten();
        let siblings = parent_id
            .map(|id| sim_state.universe.get_body(id))
            .flatten()
            .map(|w| &w.relations.satellites);
        let cur_sibling_idx = siblings
            .map(|siblings| siblings.iter().position(|s| *s == universe_id))
            .flatten();

        ui.visuals_mut().override_text_color = Some(Color32::WHITE);

        let new_child_button = ui_button(ui, "New child...");

        let new_sibling_enabled = parent_id.is_some();
        let new_sibling_button = ui.scope(|ui| {
            if !new_sibling_enabled {
                ui.disable();
            }

            ui_button(ui, "New sibling...")
        });
        let new_sibling_button = new_sibling_button.inner;
        ui.separator();

        let focus_button =
            Button::selectable(sim_state.focused_body == universe_id, "Focus").right_text("");
        let focus_button = ui.add_sized((ui.available_width(), 16.0), focus_button);

        ui.separator();

        let up_enabled = cur_sibling_idx.map(|i| i > 0).unwrap_or(false);
        let up_button = ui.scope(|ui| {
            if !up_enabled {
                ui.disable();
            }
            ui.add_sized((ui.available_width(), 16.0), button("Move up"))
        });
        let up_button = up_button.inner;

        let down_enabled = cur_sibling_idx
            .map(|i| siblings.map(|s| s.len() > i + 1))
            .flatten()
            .unwrap_or(false);
        let down_button = ui.scope(|ui| {
            if !down_enabled {
                ui.disable();
            }
            ui.add_sized((ui.available_width(), 16.0), button("Move down"))
        });
        let down_button = down_button.inner;

        ui.separator();
        let duplicate_enabled = parent_id.is_some();
        let duplicate_button = ui.scope(|ui| {
            if !duplicate_enabled {
                ui.disable();
            }

            ui.add_sized((ui.available_width(), 16.0), button("Duplicate"))
        });
        let duplicate_button = duplicate_button.inner;
        let delete_enabled = parent_id.is_some();
        let delete_button = ui.scope(|ui| {
            if !delete_enabled {
                ui.disable();
            }

            ui.add_sized((ui.available_width(), 16.0), button("Delete"))
        });
        let delete_button = delete_button.inner;
        let rename_button = ui_button(ui, "Rename");

        if new_child_button.clicked() {
            // TODO
            sim_state.ui.listed_body_with_popup = None;
        }
        if new_sibling_button.clicked() {
            // TODO
            sim_state.ui.listed_body_with_popup = None;
        }
        if let Some(parent_id) = parent_id
            && let Some(cur_idx) = cur_sibling_idx
        {
            if up_button.clicked() && cur_idx > 0 {
                let prev_idx = cur_idx - 1;
                let parent = sim_state.universe.get_body_mut(parent_id).unwrap();
                parent.relations.satellites.swap(cur_idx, prev_idx);
            }
            if down_button.clicked() {
                let next_idx = cur_idx + 1;
                let parent = sim_state.universe.get_body_mut(parent_id).unwrap();
                parent.relations.satellites.swap(cur_idx, next_idx);
            }
        }
        if focus_button.clicked() {
            sim_state.switch_focus(universe_id, position_map);
        }
        if duplicate_button.clicked() {
            let _ = sim_state.universe.duplicate_body(universe_id);
            sim_state.ui.listed_body_with_popup = None;
        }
        if delete_button.clicked() {
            sim_state.universe.remove_body(universe_id);
            sim_state.switch_focus(parent_id.unwrap_or(0), position_map);
            sim_state.ui.listed_body_with_popup = None;
        }
        if rename_button.clicked() {
            set_rename_state(ui, sim_state, universe_id);
            sim_state.ui.listed_body_with_popup = None;
        }
    });
    if outer_response.clicked_elsewhere()
        && inner_response.clicked_elsewhere()
        && popup
            .map(|p| p.response.clicked_elsewhere())
            .unwrap_or(false)
    {
        sim_state.ui.listed_body_with_popup = None;
    }
}

fn body_edit_window(ctx: &EguiContext, _sim_state: &mut SimState) {
    Window::new("Celestial Editor").show(ctx, |ui| {
        ui.label("This window is not implemented yet.");
    });
}
