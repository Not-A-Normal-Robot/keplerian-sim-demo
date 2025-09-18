use super::{
    super::super::{
        sim::universe::{BodyWrapper, Id as UniverseId, Universe},
        units::{AutoUnit, length::LengthUnit, mass::MassUnit},
    },
    SimState, declare_id,
    info::body_window_info,
};
use three_d::egui::{Color32, Context, Grid, Label, RichText, Ui, Window};

declare_id!(salt_only, EDIT_BODY_PHYS, b"mutB0dyP");
declare_id!(salt_only, EDIT_BODY_ORBIT, b"mutB0dyO");
declare_id!(salt_only, EDIT_BODY_INFO_GRID, b"mutInF0!");

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
                body_edit_window_contents(
                    ui,
                    &mut sim_state.universe,
                    body_id,
                    &mut sim_state.ui.edit_body_window_state,
                );
            });
        });

    sim_state.ui.edit_body_window_state.window_open = open;
}

fn body_edit_window_contents(
    ui: &mut Ui,
    universe: &mut Universe,
    body_id: UniverseId,
    window_state: &mut EditBodyWindowState,
) {
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);

    let text = RichText::new("Physical Characteristics")
        .underline()
        .size(16.0);
    ui.label(text);
    ui.add_space(8.0);
    Grid::new(EDIT_BODY_PHYS_SALT)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            edit_body_window_phys(ui, universe, body_id, window_state)
        });

    let text = RichText::new("Orbital Parameters").underline().size(16.0);
    let label = Label::new(text);
    ui.add_space(12.0);
    ui.add(label);
    ui.add_space(8.0);
    Grid::new(EDIT_BODY_ORBIT_SALT)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            edit_body_window_orbit(ui, universe, body_id, window_state)
        });

    ui.add_space(12.0);

    let derived_info = RichText::new("Derived Information")
        .color(Color32::WHITE)
        .size(16.0)
        .underline();

    if let Some(wrapper) = universe.get_body(body_id) {
        ui.collapsing(derived_info, |ui| {
            ui.set_min_width(ui.available_width());
            Grid::new(EDIT_BODY_INFO_GRID_SALT)
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    body_window_info(ui, &wrapper.body, wrapper.relations.parent, universe);
                });
        });
    }
}

fn edit_body_window_phys(
    ui: &mut Ui,
    _universe: &mut Universe,
    _body_id: UniverseId,
    _window_state: &mut EditBodyWindowState,
) {
    ui.label("TODO");
    // TODO
}

fn edit_body_window_orbit(
    ui: &mut Ui,
    _universe: &mut Universe,
    _body_id: UniverseId,
    _window_state: &mut EditBodyWindowState,
) {
    ui.label("TODO");
    // TODO
}
