use super::{
    super::{
        super::{
            sim::universe::{Id as UniverseId, Universe},
            units::{AutoUnit, length::LengthUnit, mass::MassUnit},
        },
        unit_dv::drag_value_with_unit,
    },
    SimState, declare_id,
    info::body_window_info,
};
use three_d::egui::{
    Color32, Context, Grid, Label, RichText, TextEdit, Ui, Window,
    color_picker::color_edit_button_srgb,
};

declare_id!(salt_only, EDIT_BODY_PHYS, b"mutB0dyP");
declare_id!(salt_only, EDIT_BODY_ORBIT, b"mutB0dyO");
declare_id!(salt_only, EDIT_BODY_INFO_GRID, b"mutInF0!");
declare_id!(salt_only, EDIT_BODY_MASS, b"mut|mass");
declare_id!(salt_only, EDIT_BODY_RADIUS, b"m|Radius");

pub(in super::super) struct EditBodyWindowState {
    mass_unit: AutoUnit<MassUnit>,
    radius_unit: AutoUnit<LengthUnit>,
    periapsis_unit: AutoUnit<LengthUnit>,
    pub(in super::super) window_open: bool,
}

impl Default for EditBodyWindowState {
    fn default() -> Self {
        Self {
            mass_unit: AutoUnit {
                auto: true,
                unit: MassUnit::Kilograms,
            },
            radius_unit: AutoUnit {
                auto: true,
                unit: LengthUnit::Meters,
            },
            periapsis_unit: AutoUnit {
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
    universe: &mut Universe,
    body_id: UniverseId,
    window_state: &mut EditBodyWindowState,
) {
    let wrapper = match universe.get_body_mut(body_id) {
        Some(w) => w,
        None => return,
    };

    ui.label("Body name").on_hover_text(
        RichText::new("The name that will show up in the list of bodies.")
            .color(Color32::WHITE)
            .size(16.0),
    );
    ui.add(
        TextEdit::singleline(&mut wrapper.body.name)
            .char_limit(255)
            .hint_text("Enter new body name")
            .desired_width(f32::INFINITY),
    );
    ui.end_row();

    ui.label("Body color").on_hover_text(
        RichText::new("The color that this body will be rendered in.")
            .color(Color32::WHITE)
            .size(16.0),
    );
    let original_srgb: [u8; 3] = wrapper.body.color.into();
    let mut srgb = original_srgb.clone();
    color_edit_button_srgb(ui, &mut srgb);
    if srgb != original_srgb {
        wrapper.body.color = srgb.into();
    }
    ui.end_row();

    ui.label("Mass").on_hover_text(
        RichText::new(
            "The mass of the body.\n\
            Determines the speed of orbiting objects.",
        )
        .color(Color32::WHITE)
        .size(16.0),
    );
    drag_value_with_unit(
        EDIT_BODY_MASS_SALT,
        ui,
        &mut wrapper.body.mass,
        &mut window_state.mass_unit,
    );
    ui.end_row();

    ui.label("Radius").on_hover_text(
        RichText::new("The radius that this body will be rendered in.")
            .color(Color32::WHITE)
            .size(16.0),
    );
    drag_value_with_unit(
        EDIT_BODY_RADIUS_SALT,
        ui,
        &mut wrapper.body.radius,
        &mut window_state.radius_unit,
    );
    ui.end_row();
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
