use std::{
    ops::RangeInclusive,
    sync::{Arc, LazyLock},
};

use super::{
    super::{
        assets,
        sim::{body::Body, universe::BulkMuSetterMode},
        units::time::{TimeDisplayMode, TimeUnit},
    },
    MIN_TOUCH_TARGET_LEN, MIN_TOUCH_TARGET_VEC, SimState,
    celestials::PreviewBody,
    declare_id,
};
use float_pretty_print::PrettyPrintFloat;
use keplerian_sim::Orbit;
use strum::IntoEnumIterator;
use three_d::{
    Srgba,
    egui::{
        Align2, Area, Atom, Button, Color32, ComboBox, Context, CornerRadius, DragValue, FontId,
        Frame, Image, ImageButton, Margin, Popup, PopupCloseBehavior, Rect, RectAlign, Response,
        RichText, ScrollArea, Shape, Slider, Stroke, TextStyle, TopBottomPanel, Ui, Vec2,
        style::HandleShape,
    },
};

declare_id!(BOTTOM_PANEL, b"BluRigel");
declare_id!(PANEL_SHOW_AREA, b"Huzzah!!");
declare_id!(salt_only, TIME_CONTROL_COMBO_BOX, b"Solstice");
declare_id!(BOTTOM_BAR_TOGGLE_BUTTON, b"$D0wn^Up");
declare_id!(OPTIONS_POPUP, b"M0D1F13D");
declare_id!(salt_only, MU_SETTER_COMBO_BOX, b"whichWAY");

pub(super) struct BottomBarState {
    time_disp: TimeDisplayMode,
    time_slider_pos: f64,
    time_speed_amount: f64,
    time_speed_unit: TimeUnit,
    time_speed_unit_auto: bool,
    expanded: bool,
    options_open: bool,
}

impl Default for BottomBarState {
    fn default() -> Self {
        Self {
            time_disp: TimeDisplayMode::SingleUnit,
            time_slider_pos: 0.0,
            time_speed_amount: 1.0,
            time_speed_unit: TimeUnit::Seconds,
            time_speed_unit_auto: true,
            expanded: true,
            options_open: false,
        }
    }
}

pub(super) const TIME_SPEED_DRAG_VALUE_TEXT_STYLE_NAME: LazyLock<Arc<str>> =
    LazyLock::new(|| Arc::from("TSDVF"));

fn format_dv_number(number: f64, _: RangeInclusive<usize>) -> String {
    let number = PrettyPrintFloat(number);
    format!("{number:5.1}")
}

pub(super) fn draw(ctx: &Context, sim_state: &mut SimState, elapsed_time: f64) {
    if sim_state.ui.bottom_bar_state.expanded {
        bottom_panel(ctx, sim_state, elapsed_time);
    } else {
        Area::new(*PANEL_SHOW_AREA_ID)
            .anchor(Align2::RIGHT_BOTTOM, [-16.0, -4.0])
            .default_size(COLLAPSE_TOGGLE_SIZE)
            .show(ctx, |ui| {
                collapse_toggle(ui, sim_state);
            });
    }
}

fn bottom_panel(ctx: &Context, sim_state: &mut SimState, elapsed_time: f64) {
    let height = 64.0;
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
            ui.style_mut().always_scroll_the_only_direction = true;
            ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| bottom_panel_contents(ui, sim_state, elapsed_time))
                });
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

    let remaining_width = ui.available_width()
        - END_ITEMS_SIZE.x
        - ui.spacing().item_spacing.x * 4.0
        - WINDOW_TOGGLES_TOTAL_SIZE.x
        - 16.0; // 16.0 from the space before the pause button
    let spacing = remaining_width.max(0.0) / 2.0;

    ui.add_space(spacing);
    window_toggles(ui, sim_state);
    ui.add_space(spacing);

    end_items(ui, sim_state);
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
            sim_state.universe.time / sim_state.ui.bottom_bar_state.time_speed_unit.get_value()
        ),
        unit = sim_state.ui.bottom_bar_state.time_speed_unit,
        rate = PrettyPrintFloat(
            sim_state.sim_speed / sim_state.ui.bottom_bar_state.time_speed_unit.get_value()
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
        .bottom_bar_state
        .time_disp
        .format_time(sim_state.universe.time);

    let text = RichText::new(string)
        .monospace()
        .color(Color32::WHITE)
        .size(16.0);

    let hover_string = format!(
        "Currently in {} mode\nLeft click to cycle, right click to cycle backwards",
        sim_state.ui.bottom_bar_state.time_disp
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
            sim_state.ui.bottom_bar_state.time_disp =
                sim_state.ui.bottom_bar_state.time_disp.get_next();
        }
        if button_instance.secondary_clicked() {
            sim_state.ui.bottom_bar_state.time_disp =
                sim_state.ui.bottom_bar_state.time_disp.get_prev();
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
        &mut sim_state.ui.bottom_bar_state.time_slider_pos,
        -1.0..=1.0,
    )
    .show_value(false)
    .handle_shape(HandleShape::Rect { aspect_ratio: 0.3 });

    if column_mode {
        ui.spacing_mut().slider_width = ui.available_width();
    }

    let slider_instance = ui.add(slider).on_hover_text(hover_text);

    if slider_instance.is_pointer_button_down_on() {
        let base = 10.0f64.powf(sim_state.ui.bottom_bar_state.time_slider_pos);
        sim_state.sim_speed *= base.powf(elapsed_time / 1000.0);
    } else {
        sim_state.ui.bottom_bar_state.time_slider_pos *= (-5.0 * elapsed_time / 1000.0).exp();
    }
}
fn time_drag_value(ui: &mut Ui, sim_state: &mut SimState) {
    sim_state.ui.bottom_bar_state.time_speed_amount =
        sim_state.sim_speed / sim_state.ui.bottom_bar_state.time_speed_unit.get_value();
    let prev_speed_amt = sim_state.ui.bottom_bar_state.time_speed_amount;

    let dv_instance = ui.scope(|ui| time_drag_value_inner(ui, sim_state)).inner;

    let hover_text = RichText::new(
        "Drag left to slow down time.\n\
        Drag right to speed up time.\n\
        Click/tap to enter in an amount manually.",
    )
    .color(Color32::WHITE)
    .size(16.0);
    let dv_instance = dv_instance.on_hover_text(hover_text);

    if prev_speed_amt != sim_state.ui.bottom_bar_state.time_speed_amount {
        sim_state.sim_speed = sim_state.ui.bottom_bar_state.time_speed_amount
            * sim_state.ui.bottom_bar_state.time_speed_unit.get_value();
    }

    if sim_state.ui.bottom_bar_state.time_speed_unit_auto && !dv_instance.dragged() {
        sim_state.ui.bottom_bar_state.time_speed_unit =
            TimeUnit::largest_unit_from_base(sim_state.sim_speed);
        sim_state.ui.bottom_bar_state.time_speed_amount =
            sim_state.sim_speed / sim_state.ui.bottom_bar_state.time_speed_unit.get_value();
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

    let drag_value = DragValue::new(&mut sim_state.ui.bottom_bar_state.time_speed_amount)
        .update_while_editing(false)
        .custom_formatter(format_dv_number);
    ui.add_sized(dv_size, drag_value)
}
fn time_unit_box(ui: &mut Ui, sim_state: &mut SimState) {
    let unit_string = format!("{}/s", sim_state.ui.bottom_bar_state.time_speed_unit);

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

        let label = Button::selectable(sim_state.ui.bottom_bar_state.time_speed_unit == unit, text);
        let label = ui.add_sized(MIN_TOUCH_TARGET_VEC, label);

        if label.clicked() {
            sim_state.ui.bottom_bar_state.time_speed_unit_auto = false;
            sim_state.ui.bottom_bar_state.time_speed_unit = unit;
        }
    }

    ui.separator();

    let text = RichText::new("Auto-pick").font(font);
    let label = Button::selectable(sim_state.ui.bottom_bar_state.time_speed_unit_auto, text);
    let auto = ui.add_sized(MIN_TOUCH_TARGET_VEC, label);
    if auto.clicked() {
        sim_state.ui.bottom_bar_state.time_speed_unit_auto =
            !sim_state.ui.bottom_bar_state.time_speed_unit_auto;
    }
}
fn time_unit_box_popup(ui: &mut Ui, sim_state: &mut SimState) {
    let font = FontId::proportional(16.0);

    let unit = sim_state.ui.bottom_bar_state.time_speed_unit;
    let title_string = if sim_state.ui.bottom_bar_state.time_speed_unit_auto {
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

const WINDOW_TOGGLES_TOTAL_SIZE: Vec2 = Vec2::new(
    WINDOW_TOGGLE_BUTTON_SIZE.x * 3.0,
    WINDOW_TOGGLE_BUTTON_SIZE.y,
);
const WINDOW_TOGGLE_BUTTON_SIZE: Vec2 = MIN_TOUCH_TARGET_VEC;
fn window_toggles(ui: &mut Ui, sim_state: &mut SimState) {
    ui.spacing_mut().button_padding = Vec2::ZERO;
    let widget_styles = &mut ui.visuals_mut().widgets;
    widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
    widget_styles.inactive.bg_stroke = Stroke::NONE;
    widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(16);
    widget_styles.hovered.bg_stroke = Stroke::NONE;
    widget_styles.active.weak_bg_fill = Color32::from_white_alpha(64);

    let list_open = &mut sim_state.ui.body_list_window_state.window_open;
    let list_button = ImageButton::new(assets::TREE_LIST_IMAGE.clone()).selected(*list_open);
    let list_button = ui.add_sized(WINDOW_TOGGLE_BUTTON_SIZE, list_button);

    if list_button.clicked() {
        *list_open ^= true;
    }

    let add_open = sim_state.preview_body.is_some();
    let add_button = ImageButton::new(assets::ADD_ORBIT_IMAGE.clone()).selected(add_open);
    let add_button = ui.add_sized(WINDOW_TOGGLE_BUTTON_SIZE, add_button);

    if add_button.clicked() {
        if sim_state.preview_body.is_some() {
            sim_state.preview_body = None;
        } else {
            // let root = sim_state.universe.get_bodies().iter().next();
            let root = sim_state
                .universe
                .get_bodies()
                .iter()
                .min_by_key(|(id, _)| **id);

            if let Some((&root_id, root_wrapper)) = root {
                let root_body = &root_wrapper.body;
                sim_state.preview_body = Some(PreviewBody {
                    body: Body {
                        mass: 1.0,
                        name: format!("Child of {}", &root_body.name),
                        radius: root_body.radius * 0.1,
                        color: Srgba::WHITE,
                        orbit: Some(Orbit::new(
                            0.0,
                            root_body.radius * 2.0,
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            root_body.mass * sim_state.universe.get_gravitational_constant(),
                        )),
                    },
                    parent_id: Some(root_id),
                })
            } else {
                sim_state.preview_body = Some(PreviewBody {
                    body: Body::default(),
                    parent_id: None,
                })
            }
        }
    }

    let edit_open = &mut sim_state.ui.edit_body_window_state.window_open;
    let edit_button = ImageButton::new(assets::EDIT_ORBIT_IMAGE.clone()).selected(*edit_open);
    let edit_button = ui.add_sized(WINDOW_TOGGLE_BUTTON_SIZE, edit_button);

    if edit_button.clicked() {
        *edit_open ^= true;
    }
}

const END_ITEMS_SIZE: Vec2 = Vec2::new(
    OPTIONS_BUTTON_SIZE.x + COLLAPSE_TOGGLE_SIZE.x,
    MIN_TOUCH_TARGET_VEC.y,
);

fn end_items(ui: &mut Ui, sim_state: &mut SimState) {
    options_button(ui, sim_state);
    collapse_toggle(ui, sim_state);
}

const OPTIONS_BUTTON_SIZE: Vec2 = MIN_TOUCH_TARGET_VEC;
fn options_button(ui: &mut Ui, sim_state: &mut SimState) {
    // TODO: Hover tooltip
    let button = ImageButton::new(assets::OPTIONS.clone())
        .selected(sim_state.ui.bottom_bar_state.options_open);

    let button = ui.add_sized(OPTIONS_BUTTON_SIZE, button);

    if button.clicked() {
        sim_state.ui.bottom_bar_state.options_open ^= true;
    }

    let popup = Popup::menu(&button)
        .align(RectAlign::TOP_END)
        .open(sim_state.ui.bottom_bar_state.options_open)
        .show(|ui| {
            ui.scope(|ui| {
                ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                ui.spacing_mut().interact_size = MIN_TOUCH_TARGET_VEC;
                options_menu(ui, sim_state)
            })
        });

    if let Some(popup) = popup {
        let force_open = popup.inner.inner || button.clicked();

        if popup.response.clicked_elsewhere() && !force_open {
            sim_state.ui.bottom_bar_state.options_open = false;
        }
    }
}

/// Whether or not this menu should be forced open.
fn options_menu(ui: &mut Ui, sim_state: &mut SimState) -> bool {
    ui.style_mut().drag_value_text_style = TextStyle::Heading;
    const G_TOOLTIP: &'static str = "Gravity multiplier.\n\
        Change how strong the \"force\" of gravity is.\n\
        Default: 6.67e-11";
    let tooltip = Arc::new(RichText::new(G_TOOLTIP).color(Color32::WHITE).size(16.0));

    let label_text = RichText::new("Gravity multi.")
        .color(Color32::WHITE)
        .size(16.0);
    ui.label(label_text).on_hover_text(Arc::clone(&tooltip));
    let initial_g = sim_state.universe.get_gravitational_constant();
    let mut g = initial_g.clone();
    let dv = DragValue::new(&mut g)
        .speed(initial_g * 1e-3)
        .range(1e-20..=f64::MAX)
        .custom_formatter(|g, _| format!("{:15.15}", PrettyPrintFloat(g)))
        .update_while_editing(false);

    ui.add(dv).on_hover_text(tooltip);

    if g != initial_g {
        sim_state
            .universe
            .set_gravitational_constant(g, sim_state.mu_setter_mode);
    }

    ui.separator();

    const MU_TOOLTIP: &str = "Gravitational parameter (µ) setter mode.\n\
        Change the behavior of celestial bodies when their \
        gravitational parameter (parent mass × gravitational multiplier) is modified.";

    let tooltip = Arc::new(RichText::new(MU_TOOLTIP).color(Color32::WHITE).size(16.0));

    let label_text = RichText::new("µ setter mode")
        .color(Color32::WHITE)
        .size(16.0);

    ui.label(label_text).on_hover_text(Arc::clone(&tooltip));

    let mode_text = RichText::new(sim_state.mu_setter_mode.name())
        .color(Color32::WHITE)
        .size(16.0);

    let cb = ComboBox::from_id_salt(MU_SETTER_COMBO_BOX_SALT)
        .selected_text(mode_text)
        .show_ui(ui, |ui| mu_mode_menu(ui, &mut sim_state.mu_setter_mode));

    cb.response.on_hover_text(Arc::clone(&tooltip));

    cb.inner.unwrap_or(false)
}

/// Returns whether or not any button was clicked
fn mu_mode_menu(ui: &mut Ui, mu_setter_mode: &mut BulkMuSetterMode) -> bool {
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
    ui.spacing_mut().interact_size = MIN_TOUCH_TARGET_VEC;

    let mut clicked = false;

    for mode in BulkMuSetterMode::iter() {
        let text = RichText::new(mode.name()).size(16.0);
        let button = Button::selectable(*mu_setter_mode == mode, text);
        let button = ui.add(button).on_hover_text(
            RichText::new(mode.description())
                .color(Color32::WHITE)
                .size(16.0),
        );

        if button.clicked() {
            *mu_setter_mode = mode;
            clicked = true;
        }
    }

    clicked
}

const COLLAPSE_TOGGLE_SIZE: Vec2 = MIN_TOUCH_TARGET_VEC;
fn collapse_toggle(ui: &mut Ui, sim_state: &mut SimState) {
    // TODO: Hover tooltip
    ui.spacing_mut().button_padding = Vec2::ZERO;
    let widget_styles = &mut ui.visuals_mut().widgets;
    widget_styles.inactive.weak_bg_fill = Color32::TRANSPARENT;
    widget_styles.inactive.bg_stroke = Stroke::NONE;
    widget_styles.hovered.weak_bg_fill = Color32::from_white_alpha(16);
    widget_styles.hovered.bg_stroke = Stroke::NONE;
    widget_styles.active.weak_bg_fill = Color32::from_white_alpha(64);

    let atom = Atom::custom(*BOTTOM_BAR_TOGGLE_BUTTON_ID, COLLAPSE_TOGGLE_SIZE);
    let button = Button::new(atom).min_size(COLLAPSE_TOGGLE_SIZE);
    let button = button.atom_ui(ui);

    let open = &mut sim_state.ui.bottom_bar_state.expanded;

    if let Some(rect) = button.rect(*BOTTOM_BAR_TOGGLE_BUTTON_ID) {
        let rect =
            Rect::from_center_size(rect.center(), Vec2::new(rect.width(), rect.height()) * 0.25);
        let points = if *open {
            vec![rect.left_top(), rect.right_top(), rect.center_bottom()]
        } else {
            vec![rect.left_bottom(), rect.right_bottom(), rect.center_top()]
        };

        ui.painter().add(Shape::convex_polygon(
            points,
            ui.style().interact(&button.response).fg_stroke.color,
            Stroke::NONE,
        ));
    }

    if button.response.clicked() {
        *open ^= true;
    }
}
