use std::{
    ops::RangeInclusive,
    sync::{Arc, LazyLock},
};

use super::{
    super::{
        assets,
        units::time::{TimeDisplayMode, TimeUnit},
    },
    MIN_TOUCH_TARGET_LEN, MIN_TOUCH_TARGET_VEC, SimState, declare_id,
};
use float_pretty_print::PrettyPrintFloat;
use strum::IntoEnumIterator;
use three_d::egui::{
    Button, Color32, ComboBox, Context, CornerRadius, DragValue, FontId, Frame, Image, ImageButton,
    Margin, Popup, PopupCloseBehavior, Response, RichText, ScrollArea, Slider, Stroke, TextStyle,
    TopBottomPanel, Ui, Vec2, style::HandleShape,
};

declare_id!(BOTTOM_PANEL, b"BluRigel");
declare_id!(salt_only, TIME_CONTROL_COMBO_BOX, b"Solstice");

pub(super) struct BottomBarData {
    time_disp: TimeDisplayMode,
    time_slider_pos: f64,
    time_speed_amount: f64,
    time_speed_unit: TimeUnit,
    time_speed_unit_auto: bool,
}

impl Default for BottomBarData {
    fn default() -> Self {
        Self {
            time_disp: TimeDisplayMode::SingleUnit,
            time_slider_pos: 0.0,
            time_speed_amount: 1.0,
            time_speed_unit: TimeUnit::Seconds,
            time_speed_unit_auto: true,
        }
    }
}

pub(super) const TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME: LazyLock<Arc<str>> =
    LazyLock::new(|| Arc::from("TSDVF"));

fn format_dv_number(number: f64, _: RangeInclusive<usize>) -> String {
    let number = PrettyPrintFloat(number);
    format!("{number:5.1}")
}

pub(super) fn bottom_bar(ctx: &Context, sim_state: &mut SimState, elapsed_time: f64) {
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
        time = PrettyPrintFloat(
            sim_state.universe.time / sim_state.ui.bottom_bar_data.time_speed_unit.get_value()
        ),
        unit = sim_state.ui.bottom_bar_data.time_speed_unit,
        rate = PrettyPrintFloat(
            sim_state.sim_speed / sim_state.ui.bottom_bar_data.time_speed_unit.get_value()
        ),
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

    let string = sim_state
        .ui
        .bottom_bar_data
        .time_disp
        .format_time(sim_state.universe.time);

    let text = RichText::new(string)
        .monospace()
        .color(Color32::WHITE)
        .size(16.0);

    let hover_string = format!(
        "Currently in {} mode\nLeft click to cycle, right click to cycle backwards",
        sim_state.ui.bottom_bar_data.time_disp
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
            sim_state.ui.bottom_bar_data.time_disp =
                sim_state.ui.bottom_bar_data.time_disp.get_next();
        }
        if button_instance.secondary_clicked() {
            sim_state.ui.bottom_bar_data.time_disp =
                sim_state.ui.bottom_bar_data.time_disp.get_prev();
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
    let slider = Slider::new(
        &mut sim_state.ui.bottom_bar_data.time_slider_pos,
        -1.0..=1.0,
    )
    .show_value(false)
    .handle_shape(HandleShape::Rect { aspect_ratio: 0.3 });

    if column_mode {
        ui.spacing_mut().slider_width = ui.available_width();
    }

    let slider_instance = ui.add(slider).on_hover_text(hover_text);

    if slider_instance.is_pointer_button_down_on() {
        let base = 10.0f64.powf(sim_state.ui.bottom_bar_data.time_slider_pos);
        sim_state.sim_speed *= base.powf(elapsed_time / 1000.0);
    } else {
        sim_state.ui.bottom_bar_data.time_slider_pos *= (-5.0 * elapsed_time / 1000.0).exp();
    }
}
fn time_drag_value(ui: &mut Ui, sim_state: &mut SimState) {
    sim_state.ui.bottom_bar_data.time_speed_amount =
        sim_state.sim_speed / sim_state.ui.bottom_bar_data.time_speed_unit.get_value();
    let prev_speed_amt = sim_state.ui.bottom_bar_data.time_speed_amount;

    let dv_instance = ui.scope(|ui| time_drag_value_inner(ui, sim_state)).inner;

    let hover_text = RichText::new(
        "Drag left to slow down time.\n\
        Drag right to speed up time.\n\
        Click/tap to enter in an amount manually.",
    )
    .color(Color32::WHITE)
    .size(16.0);
    let dv_instance = dv_instance.on_hover_text(hover_text);

    if prev_speed_amt != sim_state.ui.bottom_bar_data.time_speed_amount {
        sim_state.sim_speed = sim_state.ui.bottom_bar_data.time_speed_amount
            * sim_state.ui.bottom_bar_data.time_speed_unit.get_value();
    }

    if sim_state.ui.bottom_bar_data.time_speed_unit_auto && !dv_instance.dragged() {
        sim_state.ui.bottom_bar_data.time_speed_unit =
            TimeUnit::largest_unit_from_base(sim_state.sim_speed);
        sim_state.ui.bottom_bar_data.time_speed_amount =
            sim_state.sim_speed / sim_state.ui.bottom_bar_data.time_speed_unit.get_value();
    }
}
fn time_drag_value_inner(ui: &mut Ui, sim_state: &mut SimState) -> Response {
    let dv_size = Vec2::new(MIN_TOUCH_TARGET_LEN * 2.0, MIN_TOUCH_TARGET_LEN);
    let style_name = Arc::clone(&*TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME);
    ui.style_mut()
        .text_styles
        .insert(TextStyle::Name(style_name.clone()), FontId::monospace(16.0));
    ui.style_mut().drag_value_text_style = TextStyle::Name(style_name);
    ui.spacing_mut().button_padding = Vec2::new(16.0, 8.0);
    let widget_styles = &mut ui.visuals_mut().widgets;
    widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
    widget_styles.inactive.bg_stroke = Stroke::NONE;
    widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(16);
    widget_styles.hovered.bg_stroke = Stroke::NONE;
    widget_styles.active.weak_bg_fill = Color32::from_white_alpha(64);

    let drag_value = DragValue::new(&mut sim_state.ui.bottom_bar_data.time_speed_amount)
        .update_while_editing(false)
        .custom_formatter(format_dv_number);
    ui.add_sized(dv_size, drag_value)
}
fn time_unit_box(ui: &mut Ui, sim_state: &mut SimState) {
    let unit_string = format!("{}/s", sim_state.ui.bottom_bar_data.time_speed_unit);

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

        let label = Button::selectable(sim_state.ui.bottom_bar_data.time_speed_unit == unit, text);
        let label = ui.add_sized(MIN_TOUCH_TARGET_VEC, label);

        if label.clicked() {
            sim_state.ui.bottom_bar_data.time_speed_unit_auto = false;
            sim_state.ui.bottom_bar_data.time_speed_unit = unit;
        }
    }

    ui.separator();

    let text = RichText::new("Auto-pick").font(font);
    let label = Button::selectable(sim_state.ui.bottom_bar_data.time_speed_unit_auto, text);
    let auto = ui.add_sized(MIN_TOUCH_TARGET_VEC, label);
    if auto.clicked() {
        sim_state.ui.bottom_bar_data.time_speed_unit_auto =
            !sim_state.ui.bottom_bar_data.time_speed_unit_auto;
    }
}
fn time_unit_box_popup(ui: &mut Ui, sim_state: &mut SimState) {
    let font = FontId::proportional(16.0);

    let unit = sim_state.ui.bottom_bar_data.time_speed_unit;
    let title_string = if sim_state.ui.bottom_bar_data.time_speed_unit_auto {
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
