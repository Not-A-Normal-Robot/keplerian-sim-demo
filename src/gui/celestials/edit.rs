use super::{
    super::super::units::{AutoUnit, length::LengthUnit, mass::MassUnit},
    SimState,
};
use three_d::egui::{Context, Window};

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
    Window::new("Celestial Editor")
        .open(&mut sim_state.ui.edit_body_window_state.window_open)
        .show(ctx, |ui| {
            ui.label("This window is not implemented yet.");
        });
}
