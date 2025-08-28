use std::collections::HashMap;

use glam::DVec3;
use three_d::egui::{
    Atom, AtomLayout, Button, Color32, Context, ImageButton, Pos2, Rect, Response, RichText,
    Stroke, TextEdit, Ui, Vec2,
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
};

declare_id!(RENAME_TEXTEDIT, b"OmgRen??");

pub(super) mod edit;
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

/// A selectable button used in celestial lists.
///
/// `ren_state` should only be Some when this
/// specific button is getting renamed.
///
/// The caller is responsible for checking whether
/// the current `ren_state` matches this button's
/// `UniverseId`.
fn body_selectable_button(
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
    let fill_color = body.color;
    let fill_color =
        Color32::from_rgba_premultiplied(fill_color.r, fill_color.g, fill_color.b, fill_color.a);

    let circle_atom = Atom::custom(*CIRCLE_ICON_ID, Vec2::splat(height));

    let ellipsis_atom = Atom::custom(
        *ELLIPSIS_BUTTON_ID,
        if ellipsis {
            Vec2::splat(height)
        } else {
            Vec2::ZERO
        },
    );

    let mut layout = AtomLayout::new(circle_atom);

    let text = RichText::new(&body.name).color(Color32::WHITE);
    let text = if selected { text.underline() } else { text };
    layout.push_right(text);

    layout.push_right(Atom::grow());
    layout.push_right(ellipsis_atom);

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
        .map(|rect| ellipsis_button(ui, rect));

    let button_response = button_response.response;

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
