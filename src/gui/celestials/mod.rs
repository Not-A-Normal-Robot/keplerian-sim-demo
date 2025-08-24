use std::collections::HashMap;

use glam::DVec3;
use three_d::egui::Context;

use super::{
    super::{
        assets,
        body::Body,
        units,
        universe::{Id as UniverseId, Universe},
    },
    SimState, declare_id,
};

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
