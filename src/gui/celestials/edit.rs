use super::{
    super::super::{
        sim::universe::{BodyWrapper, Id as UniverseId, Universe},
        units::{AutoUnit, length::LengthUnit, mass::MassUnit},
    },
    SimState,
};
use three_d::egui::{Context, Ui, Window};

pub(in super::super) struct EditBodyWindowState {
    _mass_unit: AutoUnit<MassUnit>,
    _radius_unit: AutoUnit<LengthUnit>,
    pub(in super::super) window_open: bool,
}

impl Default for EditBodyWindowState {
    fn default() -> Self {
        Self {
            _mass_unit: AutoUnit {
                auto: true,
                unit: MassUnit::Kilograms,
            },
            _radius_unit: AutoUnit {
                auto: true,
                unit: LengthUnit::Meters,
            },
            window_open: true,
        }
    }
}

pub(super) fn body_edit_window(ctx: &Context, sim_state: &mut SimState) {
    let mut open = sim_state.ui.edit_body_window_state.window_open;

    let body_id = sim_state.focused_body();

    Window::new("Edit Body")
        .scroll([false, true])
        .resizable([false, true])
        .default_width(300.0)
        .min_width(300.0)
        .max_width(300.0)
        .min_height(200.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.scope(|ui| {
                body_edit_window_inner(
                    ui,
                    &mut sim_state.universe,
                    body_id,
                    &mut sim_state.ui.edit_body_window_state,
                );
            });
        });

    sim_state.ui.edit_body_window_state.window_open = open;
}

fn body_edit_window_inner(
    ui: &mut Ui,
    universe: &mut Universe,
    body_id: UniverseId,
    window_state: &mut EditBodyWindowState,
) {
    let wrapper = match universe.get_body_mut(body_id) {
        Some(w) => w,
        None => {
            ui.label("No body currently selected.");
            return;
        }
    };
}
