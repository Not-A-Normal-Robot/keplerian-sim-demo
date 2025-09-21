use std::collections::HashMap;

use glam::DVec3;
use three_d::egui::{
    Atom, AtomLayout, Button, Color32, Context, CursorIcon, Id as EguiId, ImageButton, Pos2, Rect,
    Response, RichText, Stroke, TextEdit, Ui, Vec2, collapsing_header::CollapsingState,
};

use super::{
    super::{
        assets,
        sim::{
            body::Body,
            universe::{Id as UniverseId, Universe},
        },
        units,
    },
    SimState, declare_id,
    unit_dv::drag_value_with_unit,
};

declare_id!(RENAME_TEXTEDIT, b"OmgRen??");

pub(super) mod edit;
mod info;
pub(super) mod list;
pub(super) mod new;

pub(crate) struct PreviewBody {
    pub body: Body,
    pub parent_id: Option<UniverseId>,
}

pub(super) fn celestial_windows(
    ctx: &Context,
    sim_state: &mut SimState,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    list::body_tree_window(ctx, sim_state, position_map);
    edit::body_edit_window(ctx, sim_state);
    new::new_body_window(ctx, sim_state);
}

struct BodySelectableButtonResponse {
    button_response: Response,
    rename_response: Option<Response>,
    ellipsis_button: Option<Response>,
}

/// Returns whether the already-selected body was clicked again
fn selectable_body_tree(
    ui: &mut Ui,
    egui_id: EguiId,
    universe: &Universe,
    selected: &mut Option<UniverseId>,
) -> bool {
    // TODO: Hover tooltips for focusing, context menu, etc.
    fn selectable_body_node(
        ui: &mut Ui,
        egui_id: EguiId,
        universe: &Universe,
        universe_id: UniverseId,
        selected: &mut Option<UniverseId>,
        clicked_selected: &mut bool,
    ) {
        let wrapper = match universe.get_body(universe_id) {
            Some(w) => w,
            None => return,
        };

        if wrapper.relations.satellites.is_empty() {
            ui.indent((egui_id, [universe_id]), |ui| {
                selectable_body_leaf(ui, &wrapper.body, universe_id, selected, clicked_selected);
            });
        } else {
            selectable_body_parent(
                ui,
                egui_id,
                universe,
                universe_id,
                selected,
                clicked_selected,
            );
        }
    }

    fn selectable_body_leaf(
        ui: &mut Ui,
        body: &Body,
        universe_id: UniverseId,
        selected: &mut Option<UniverseId>,
        clicked_selected: &mut bool,
    ) {
        let response =
            selectable_body_button(ui, body, 16.0, *selected == Some(universe_id), false, None);

        if response.button_response.clicked() {
            if *selected == Some(universe_id) {
                *clicked_selected = true;
            } else {
                *selected = Some(universe_id);
            }
        }
    }

    fn selectable_body_parent(
        ui: &mut Ui,
        egui_id: EguiId,
        universe: &Universe,
        universe_id: UniverseId,
        selected: &mut Option<UniverseId>,
        clicked_selected: &mut bool,
    ) {
        let wrapper = match universe.get_body(universe_id) {
            Some(wrapper) => wrapper,
            None => return,
        };
        let satellites = wrapper.relations.satellites.clone();

        let this_egui_id = egui_id.with(universe_id);

        CollapsingState::load_with_default_open(ui.ctx(), this_egui_id, true)
            .show_header(ui, |ui| {
                let response = selectable_body_button(
                    ui,
                    &wrapper.body,
                    16.0,
                    *selected == Some(universe_id),
                    false,
                    None,
                );

                if response.button_response.clicked() {
                    if *selected == Some(universe_id) {
                        *clicked_selected = true;
                    } else {
                        *selected = Some(universe_id);
                    }
                }
            })
            .body(|ui| {
                for universe_id in satellites {
                    selectable_body_node(
                        ui,
                        egui_id,
                        universe,
                        universe_id,
                        selected,
                        clicked_selected,
                    );
                }
            });
    }

    let mut clicked_selected = false;

    universe
        .get_bodies()
        .iter()
        .filter(|(_, w)| w.relations.parent.is_none())
        .map(|(id, _)| id)
        .copied()
        .for_each(|universe_id| {
            selectable_body_node(
                ui,
                egui_id,
                universe,
                universe_id,
                selected,
                &mut clicked_selected,
            )
        });

    clicked_selected
}

/// A selectable button used in celestial lists.
///
/// `ren_state` should only be Some when this
/// specific button is getting renamed.
///
/// The caller is responsible for checking whether
/// the current `ren_state` matches this button's
/// `UniverseId`.
fn selectable_body_button(
    ui: &mut Ui,
    body: &Body,
    height: f32,
    selected: bool,
    ellipsis: bool,
    ren_state: Option<&mut list::RenameState>,
) -> BodySelectableButtonResponse {
    declare_id!(CIRCLE_ICON, b"Circles!");
    declare_id!(ELLIPSIS_BUTTON, b"see_more");

    let radius = height / 2.0;
    let center = Pos2::from([radius, radius]);
    let fill_color = {
        let c = body.color;
        Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
    };

    let circle_atom = Atom::custom(*CIRCLE_ICON_ID, Vec2::splat(height));

    let ellipsis_atom = ellipsis.then(|| {
        Atom::custom(
            *ELLIPSIS_BUTTON_ID,
            if ellipsis {
                Vec2::splat(height)
            } else {
                Vec2::ZERO
            },
        )
    });

    let mut layout = AtomLayout::new(circle_atom);

    let text = RichText::new(&body.name).color(Color32::WHITE);
    let text = if selected { text.underline() } else { text };
    layout.push_right(text);

    layout.push_right(Atom::grow());
    if let Some(atom) = ellipsis_atom {
        layout.push_right(atom);
    }

    let button_response = Button::selectable(selected, layout.atoms)
        .min_size(Vec2::new(ui.available_width(), height))
        .atom_ui(ui);

    if let Some(rect) = button_response.rect(*CIRCLE_ICON_ID) {
        ui.painter().with_clip_rect(rect).circle_filled(
            center + rect.min.to_vec2(),
            radius,
            fill_color,
        );
    }

    let ellipsis_button = button_response
        .rect(*ELLIPSIS_BUTTON_ID)
        .map(|rect| ellipsis_button(ui, rect).on_hover_cursor(CursorIcon::PointingHand));

    let button_response = button_response.response.on_hover_cursor(if selected {
        CursorIcon::ContextMenu
    } else {
        CursorIcon::PointingHand
    });

    let mut rect = button_response.rect;
    let padding = height * 1.5;
    *rect.right_mut() -= padding;
    *rect.left_mut() += padding;

    let rename_response = ren_state.map(|ren_state| {
        let text_edit = TextEdit::singleline(&mut ren_state.name_buffer).id(*RENAME_TEXTEDIT_ID);

        let response = ui.put(rect, text_edit);

        if ren_state.requesting_focus {
            response.request_focus();

            if response.has_focus() {
                ren_state.requesting_focus = false;
            }
        }

        response
    });

    BodySelectableButtonResponse {
        button_response,
        rename_response,
        ellipsis_button,
    }
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
