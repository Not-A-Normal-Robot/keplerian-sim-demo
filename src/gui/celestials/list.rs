use std::collections::HashMap;

use super::{
    Body, PreviewBody, RENAME_TEXTEDIT_ID, SimState, UniverseId, declare_id, selectable_body_button,
};
use glam::DVec3;
use keplerian_sim::Orbit;
use three_d::{
    Srgba,
    egui::{
        Button, Color32, Context, Id as EguiId, IntoAtoms, Key, Popup, Response, TextWrapMode, Ui,
        Window,
        collapsing_header::CollapsingState,
        text::{CCursor, CCursorRange},
        text_edit::TextEditState,
    },
};

declare_id!(BODY_PREFIX, b"Planets!");

pub(super) struct RenameState {
    pub universe_id: UniverseId,
    pub name_buffer: String,
    pub requesting_focus: bool,
}

pub(in super::super) struct BodyListWindowState {
    listed_body_with_popup: Option<UniverseId>,
    listed_body_with_rename: Option<RenameState>,
    pub(in super::super) window_open: bool,
}

impl Default for BodyListWindowState {
    fn default() -> Self {
        Self {
            listed_body_with_popup: None,
            listed_body_with_rename: None,
            window_open: true,
        }
    }
}

fn get_body_egui_id(universe_id: UniverseId) -> EguiId {
    BODY_PREFIX_ID.with(universe_id)
}

const BODY_TREE_ICON_SIZE: f32 = 16.0;
pub(super) fn body_tree_window(
    ctx: &Context,
    sim_state: &mut SimState,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let mut open = sim_state.ui.body_list_window_state.window_open;

    let window = Window::new("Celestial Bodies").scroll(true).open(&mut open);

    window.show(ctx, |ui| {
        ui.scope(|ui| {
            body_tree_window_contents(ui, sim_state, position_map);
        })
    });

    sim_state.ui.body_list_window_state.window_open = open;
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

    let response = selectable_body_button(
        ui,
        body,
        BODY_TREE_ICON_SIZE,
        selected,
        true,
        sim_state
            .ui
            .body_list_window_state
            .listed_body_with_rename
            .as_mut()
            .filter(|state| state.universe_id == universe_id),
    );

    if response.button_response.double_clicked() {
        set_rename_state(ui.ctx(), sim_state, universe_id);
    } else if response.button_response.clicked() {
        sim_state.switch_focus(universe_id, &position_map);
    }

    if let Some(edit_text) = response.rename_response
        && edit_text.lost_focus()
    {
        let string = sim_state
            .ui
            .body_list_window_state
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

    if let Some(button) = response.ellipsis_button {
        ellipsis_popup(
            sim_state,
            &button,
            &response.button_response,
            universe_id,
            position_map,
        );
    }
}

fn set_rename_state(ctx: &Context, sim_state: &mut SimState, universe_id: UniverseId) {
    let string = match sim_state.universe.get_body(universe_id) {
        Some(w) => w.body.name.clone(),
        None => return,
    };
    let strlen = string.chars().count();
    if let Some(state) = sim_state
        .ui
        .body_list_window_state
        .listed_body_with_rename
        .take()
    {
        sim_state
            .universe
            .get_body_mut(state.universe_id)
            .map(|w| w.body.name = state.name_buffer);
    }

    sim_state.ui.body_list_window_state.listed_body_with_rename = Some(RenameState {
        universe_id,
        name_buffer: string,
        requesting_focus: true,
    });

    let mut state = TextEditState::default();
    state.cursor.set_char_range(Some(CCursorRange::two(
        CCursor::new(0),
        CCursor::new(strlen),
    )));
    state.store(ctx, *RENAME_TEXTEDIT_ID);
}

fn ellipsis_popup(
    sim_state: &mut SimState,
    inner_response: &Response,
    outer_response: &Response,
    universe_id: UniverseId,
    position_map: &HashMap<UniverseId, DVec3>,
) {
    let open = sim_state.ui.body_list_window_state.listed_body_with_popup == Some(universe_id);

    if inner_response.clicked() || outer_response.secondary_clicked() {
        if open {
            sim_state.ui.body_list_window_state.listed_body_with_popup = None;
        } else {
            sim_state.ui.body_list_window_state.listed_body_with_popup = Some(universe_id);
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
            let this_radius = body_wrapper.map(|x| x.body.radius).unwrap_or(1.0);
            let child_name = body_wrapper
                .map(|w| format!("Child of {}", w.body.name))
                .unwrap_or_else(|| "Child body".to_owned());
            let mu = body_wrapper
                .map(|w| w.body.mass * sim_state.universe.get_gravitational_constant())
                .unwrap_or(1.0);

            sim_state.preview_body = Some(PreviewBody {
                body: Body {
                    name: child_name,
                    mass: 1.0,
                    radius: this_radius * 0.1,
                    color: Srgba::WHITE,
                    orbit: Some(Orbit::new(0.0, this_radius * 2.0, 0.0, 0.0, 0.0, 0.0, mu)),
                },
                parent_id: Some(universe_id),
            });
            sim_state.ui.body_list_window_state.listed_body_with_popup = None;
            if let Some(state) = &mut sim_state.ui.new_body_window_state {
                state.request_focus = true;
            }
        }
        if new_sibling_button.clicked() {
            let parent = parent_id
                .map(|id| sim_state.universe.get_body(id))
                .flatten();
            let parent_radius = parent.map(|w| w.body.radius).unwrap_or(1.0);
            let mu = parent
                .map(|w| w.body.mass * sim_state.universe.get_gravitational_constant())
                .unwrap_or(1.0);
            let sibling_name = parent
                .map(|w| format!("Child of {}", w.body.name))
                .unwrap_or_else(|| "Sibling body".to_owned());

            sim_state.preview_body = Some(PreviewBody {
                body: Body {
                    name: sibling_name,
                    mass: 1.0,
                    radius: parent_radius * 0.1,
                    color: Srgba::WHITE,
                    orbit: Some(Orbit::new(0.0, parent_radius * 2.0, 0.0, 0.0, 0.0, 0.0, mu)),
                },
                parent_id: parent_id,
            });
            sim_state.ui.body_list_window_state.listed_body_with_popup = None;
            if let Some(state) = &mut sim_state.ui.new_body_window_state {
                state.request_focus = true;
            }
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
            let result = sim_state.universe.duplicate_body(universe_id);
            sim_state.ui.body_list_window_state.listed_body_with_popup = None;

            if let Ok(universe_id) = result {
                sim_state.ui.body_list_window_state.listed_body_with_rename = Some(RenameState {
                    universe_id,
                    name_buffer: sim_state
                        .universe
                        .get_body(universe_id)
                        .map(|w| w.body.name.clone())
                        .unwrap_or_default(),
                    requesting_focus: true,
                });
            }
        }
        if delete_button.clicked() {
            let bodies_removed = sim_state.universe.remove_body(universe_id);
            if let Some(preview) = &sim_state.preview_body
                && let Some(parent_id) = preview.parent_id
                && bodies_removed.iter().any(|(id, _)| *id == parent_id)
            {
                sim_state.preview_body = None;
            }
            sim_state.switch_focus(parent_id.unwrap_or(0), position_map);
            sim_state.ui.body_list_window_state.listed_body_with_popup = None;
        }
        if rename_button.clicked() {
            set_rename_state(ui.ctx(), sim_state, universe_id);
            sim_state.ui.body_list_window_state.listed_body_with_popup = None;
        }
    });
    if outer_response.clicked_elsewhere()
        && inner_response.clicked_elsewhere()
        && popup
            .map(|p| p.response.clicked_elsewhere())
            .unwrap_or(false)
    {
        sim_state.ui.body_list_window_state.listed_body_with_popup = None;
    }
}
