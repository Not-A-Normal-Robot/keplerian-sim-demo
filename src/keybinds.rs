use std::collections::HashMap;

use keplerian_sim::Orbit;
use three_d::{Event, Key, Modifiers, Srgba, GUI};

use super::{
    SimState,
    gui::PreviewBody,
    sim::{
        body::Body,
        universe::{BodyWrapper, Id, Universe},
    },
};

pub(super) fn handle_keybinds(sim_state: &mut SimState, events: &mut [Event], gui: &GUI) {
    for event in events {
        match event {
            Event::KeyPress {
                kind: key,
                modifiers,
                handled,
            } => handle_keypress(sim_state, key, modifiers, handled),
            Event::Text(text) => {
                if gui.context().wants_keyboard_input() {
                    continue;
                }
                handle_text_input(sim_state, &text)
            },
            _ => (),
        }
    }
}

fn handle_keypress(
    sim_state: &mut SimState,
    key: &mut Key,
    modifiers: &mut Modifiers,
    handled: &mut bool,
) {
    if *handled {
        return;
    }

    match key {
        Key::Space => {
            sim_state.running ^= true;
            *handled = true;
        }
        // TODO: Time control, focus switching keybinds, delete, edit
        _ => (),
    }
}

fn handle_text_input(sim_state: &mut SimState, text: &str) {
    text.chars()
        .for_each(|char| handle_char_input(sim_state, char));
}

fn handle_char_input(sim_state: &mut SimState, char: char) {
    match char {
        '[' => switch_to_prev_body(sim_state),
        ']' => switch_to_next_body(sim_state),
        'n' | 'N' => add_new_body(sim_state),
        _ => (),
    }
}

fn add_new_body(sim_state: &mut SimState) {
    if sim_state.preview_body.is_some() {
        return;
    }

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

fn switch_to_prev_body(sim_state: &mut SimState) {
    let id = get_prev_body_id(&sim_state.universe, sim_state.focused_body());
    sim_state.switch_focus(id, &sim_state.universe.get_all_body_positions());
}

fn get_prev_body_id(universe: &Universe, current_id: Id) -> Id {
    fn last_descendant(map: &HashMap<Id, BodyWrapper>, mut id: Id) -> Id {
        while let Some(w) = map.get(&id)
            && let Some(&child_id) = w.relations.satellites.last()
        {
            id = child_id;
        }
        id
    }

    let map = universe.get_bodies();

    let Some(current) = map.get(&current_id) else {
        return current_id;
    };

    let Some(parent_id) = current.relations.parent else {
        return last_descendant(map, current_id);
    };

    let Some(parent) = map.get(&parent_id) else {
        if cfg!(debug_assertions) {
            panic!("invalid state: parent not in universe\n{universe:?}");
        } else {
            return current_id;
        }
    };

    let Some(sibling_pos) = parent
        .relations
        .satellites
        .iter()
        .position(|&id| id == current_id)
    else {
        if cfg!(debug_assertions) {
            panic!("invalid state: parent doesn't have self in relations\n{universe:?}")
        } else {
            return current_id;
        }
    };

    let Some(prev_sibling_pos) = sibling_pos.checked_sub(1) else {
        return parent_id;
    };

    let prev_sibling_id = parent.relations.satellites[prev_sibling_pos];

    last_descendant(map, prev_sibling_id)
}

fn switch_to_next_body(sim_state: &mut SimState) {
    let id = get_next_body_id(&sim_state.universe, sim_state.focused_body());
    sim_state.switch_focus(id, &sim_state.universe.get_all_body_positions());
}

fn get_next_body_id(universe: &Universe, current_id: Id) -> Id {
    fn root(map: &HashMap<u64, BodyWrapper>, mut id: Id) -> Id {
        while let Some(w) = map.get(&id)
            && let Some(parent_id) = w.relations.parent
        {
            id = parent_id;
        }
        id
    }

    let map = universe.get_bodies();

    let Some(current) = map.get(&current_id) else {
        return current_id;
    };

    if let Some(&child_id) = current.relations.satellites.first() {
        return child_id;
    }

    let Some(parent_id) = current.relations.parent else {
        return current_id;
    };

    let Some(parent) = map.get(&parent_id) else {
        if cfg!(debug_assertions) {
            panic!("invalid state: parent listed but not found in map\n{universe:?}");
        } else {
            return current_id;
        }
    };

    let Some(sibling_pos) = parent
        .relations
        .satellites
        .iter()
        .position(|&id| id == current_id)
    else {
        if cfg!(debug_assertions) {
            panic!("invalid state: parent doesn't have self in satellites\n{universe:?}");
        } else {
            return current_id;
        }
    };

    let Some(next_sibling_pos) = sibling_pos.checked_add(1) else {
        return root(map, parent_id);
    };

    parent
        .relations
        .satellites
        .get(next_sibling_pos)
        .copied()
        .unwrap_or_else(|| root(map, parent_id))
}
