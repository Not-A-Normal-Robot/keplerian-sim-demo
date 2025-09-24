use super::{
    super::{
        super::{
            sim::universe::{BodyWrapper, BulkMuSetterMode, Id as UniverseId, Universe},
            units::{AutoUnit, length::LengthUnit, mass::MassUnit},
        },
        unit_dv::drag_value_with_unit,
    },
    SimState, declare_id,
    info::body_window_info,
    selectable_body_tree,
};
use keplerian_sim::OrbitTrait;
use three_d::egui::{
    Color32, ComboBox, Context, CursorIcon, DragValue, Grid, Label, PopupCloseBehavior, RichText,
    Slider, TextEdit, TextWrapMode, Ui, Window, color_picker::color_edit_button_srgb,
};

declare_id!(salt_only, EDIT_BODY_PHYS, b"mutB0dyP");
declare_id!(salt_only, EDIT_BODY_ORBIT, b"mutB0dyO");
declare_id!(salt_only, EDIT_BODY_INFO_GRID, b"mutInF0!");
declare_id!(salt_only, EDIT_BODY_MASS, b"mut|mass");
declare_id!(salt_only, EDIT_BODY_RADIUS, b"m|Radius");
declare_id!(salt_only, EDIT_BODY_PARENT_COMBO_BOX, b"mNoder3l");
declare_id!(EDIT_BODY_PARENT_TREE, b"m|->N0d3");
declare_id!(salt_only, EDIT_BODY_PERIAPSIS, b"m|PeDist");

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
            window_open: false,
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
                    sim_state.mu_setter_mode,
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
    mu_mode: BulkMuSetterMode,
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
            edit_body_window_phys(ui, universe, body_id, window_state, mu_mode)
        });

    if let Some(w) = universe.get_body(body_id)
        && w.body.orbit.is_some()
        && w.relations.parent.is_some()
    {
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
                edit_body_window_orbit(ui, universe, body_id, window_state, mu_mode)
            });
    }

    ui.add_space(12.0);

    let derived_info = RichText::new("Derived Information")
        .color(Color32::WHITE)
        .size(16.0)
        .underline();

    if let Some(wrapper) = universe.get_body(body_id) {
        let coll_res = ui.collapsing(derived_info, |ui| {
            ui.set_min_width(ui.available_width());
            Grid::new(EDIT_BODY_INFO_GRID_SALT)
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    body_window_info(ui, &wrapper.body, wrapper.relations.parent, universe);
                });
        });

        coll_res
            .header_response
            .on_hover_cursor(CursorIcon::PointingHand);
    }
}

fn edit_body_window_phys(
    ui: &mut Ui,
    universe: &mut Universe,
    body_id: UniverseId,
    window_state: &mut EditBodyWindowState,
    mu_mode: BulkMuSetterMode,
) {
    let wrapper = match universe.get_body_mut(body_id) {
        Some(w) => w,
        None => return,
    };

    ui.label("Body name")
        .on_hover_text(
            RichText::new("The name that will show up in the list of bodies.")
                .color(Color32::WHITE)
                .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    ui.add(
        TextEdit::singleline(&mut wrapper.body.name)
            .char_limit(255)
            .hint_text("Enter new body name")
            .desired_width(f32::INFINITY),
    );
    ui.end_row();

    ui.label("Body color")
        .on_hover_text(
            RichText::new("The color that this body will be rendered in.")
                .color(Color32::WHITE)
                .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let original_srgb: [u8; 3] = wrapper.body.color.into();
    let mut srgb = original_srgb.clone();
    color_edit_button_srgb(ui, &mut srgb);
    if srgb != original_srgb {
        wrapper.body.color = srgb.into();
    }
    ui.end_row();

    ui.label("Mass")
        .on_hover_text(
            RichText::new(
                "The mass of the body.\n\
            Determines the speed of orbiting objects.",
            )
            .color(Color32::WHITE)
            .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let mut mass = wrapper.body.mass;
    drag_value_with_unit(
        EDIT_BODY_MASS_SALT,
        ui,
        &mut mass,
        &mut window_state.mass_unit,
    );
    ui.end_row();

    ui.label("Radius")
        .on_hover_text(
            RichText::new("The radius that this body will be rendered in.")
                .color(Color32::WHITE)
                .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    drag_value_with_unit(
        EDIT_BODY_RADIUS_SALT,
        ui,
        &mut wrapper.body.radius,
        &mut window_state.radius_unit,
    );
    ui.end_row();

    if wrapper.body.mass != mass {
        wrapper.body.mass = mass;

        let _ = universe.update_children_gravitational_parameters(body_id, mu_mode);
    }
}

fn edit_body_window_orbit(
    ui: &mut Ui,
    universe: &mut Universe,
    body_id: UniverseId,
    window_state: &mut EditBodyWindowState,
    mu_mode: BulkMuSetterMode,
) {
    let wrapper = match universe.get_body(body_id) {
        Some(w) => w,
        None => return,
    };

    let parent_id = parent_selector(ui, universe, wrapper);

    if parent_id != wrapper.relations.parent {
        let res = universe.move_body(body_id, parent_id, mu_mode);
        if let Err(e) = res {
            eprintln!("{e}");
        }
    }

    let wrapper = match universe.get_body_mut(body_id) {
        Some(w) => w,
        None => return,
    };

    let orbit = match wrapper.body.orbit.as_mut() {
        Some(o) => o,
        None => return,
    };

    ui.label("Eccentricity")
        .on_hover_text(
            RichText::new(
                "How eccentric the orbit is.\n\
            An eccentricity of 1 (parabolic) is not supported.\n\
            An eccentricity less than one means the orbit is closed.\n\
            An eccentricity of more than one means the orbit never loops (is open; hyperbolic).",
            )
            .color(Color32::WHITE)
            .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let mut eccentricity = orbit.get_eccentricity();
    let dv = DragValue::new(&mut eccentricity)
        .range(0.0..=f64::MAX)
        .speed(0.01);
    let dv = ui.add_sized((ui.available_width(), 18.0), dv);
    if dv.changed() {
        orbit.set_eccentricity(eccentricity);
    }
    ui.end_row();

    ui.label("Periapsis")
        .on_hover_text(
            RichText::new(
                "The minimum distance of the orbit \
            to the center of the parent body.",
            )
            .color(Color32::WHITE)
            .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let mut periapsis = orbit.get_periapsis();
    drag_value_with_unit(
        EDIT_BODY_PERIAPSIS_SALT,
        ui,
        &mut periapsis,
        &mut window_state.periapsis_unit,
    );
    if periapsis != orbit.get_periapsis() {
        orbit.set_periapsis(periapsis);
    }
    ui.end_row();

    ui.label("Inclination")
        .on_hover_text(
            RichText::new("How inclined from the up axis the orbit is.")
                .color(Color32::WHITE)
                .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let mut inclination = orbit.get_inclination().to_degrees();
    let slider = Slider::new(&mut inclination, 0.0..=180.0).suffix('°');
    let slider = ui.add_sized((ui.available_width(), 18.0), slider);
    if slider.changed() {
        orbit.set_inclination(inclination.to_radians());
    }
    ui.end_row();

    ui.label("Arg. of Pe.")
        .on_hover_text(
            RichText::new(
                "The argument of periapsis of the orbit.\n\
            This is the angle offset of the periapsis along the orbital plane.",
            )
            .color(Color32::WHITE)
            .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let mut arg_pe = orbit.get_arg_pe().to_degrees();
    let slider = Slider::new(&mut arg_pe, 0.0..=360.0).suffix('°');
    let slider = ui.add(slider);
    if slider.changed() {
        orbit.set_arg_pe(arg_pe.to_radians());
    }
    ui.end_row();

    ui.label("RAAN")
        .on_hover_text(
            RichText::new(
                "The right ascension of the ascending node.\n\
            a.k.a.: the longitude of ascending node.\n\
            This is the angle offset of the ascending node along \
            the reference plane (horizontal plane).",
            )
            .color(Color32::WHITE)
            .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let mut lan = orbit.get_long_asc_node().to_degrees();
    let slider = Slider::new(&mut lan, 0.0..=360.0).suffix('°');
    let slider = ui.add(slider);
    if slider.changed() {
        orbit.set_long_asc_node(lan.to_radians());
    }
    ui.end_row();

    let mut mean_anomaly = orbit.get_mean_anomaly_at_epoch().to_degrees();
    if orbit.get_eccentricity() < 1.0 {
        ui.label("Mean anom.")
            .on_hover_text(
                RichText::new(
                    "Mean anomaly at epoch.\n\
                This is the offset to the mean anomaly.\n\
                At time = 0, the mean anomaly of this orbit will be equal to this.",
                )
                .color(Color32::WHITE)
                .size(16.0),
            )
            .on_hover_cursor(CursorIcon::Help);
        let slider = Slider::new(&mut mean_anomaly, 0.0..=360.0).suffix('°');
        let slider = ui.add(slider);
        if mean_anomaly < 0.0 || mean_anomaly > 360.0 {
            mean_anomaly = mean_anomaly.rem_euclid(360.0);
        }
        if slider.changed() {
            orbit.set_mean_anomaly_at_epoch(mean_anomaly.to_radians());
        }
    } else {
        ui.label("Hyp. m. anom.")
            .on_hover_text(
                RichText::new(
                    "Hyperbolic mean anomaly at epoch.\n\
                This is the offset to the hyperbolic mean anomaly.\n\
                At time = 0, the hyperbolic mean anomaly of this orbit will be equal to this.",
                )
                .color(Color32::WHITE)
                .size(16.0),
            )
            .on_hover_cursor(CursorIcon::Help);
        let dv = DragValue::new(&mut mean_anomaly)
            .range(f64::MIN..=f64::MAX)
            .suffix('°');
        let dv = ui.add(dv);
        if dv.changed() {
            orbit.set_mean_anomaly_at_epoch(mean_anomaly.to_radians());
        }
    }
    ui.end_row();
}

/// Returns the new parent ID
fn parent_selector(ui: &mut Ui, universe: &Universe, wrapper: &BodyWrapper) -> Option<UniverseId> {
    ui.label("Parent body")
        .on_hover_text(
            RichText::new("The body that this body is orbiting around.")
                .color(Color32::WHITE)
                .size(16.0),
        )
        .on_hover_cursor(CursorIcon::Help);
    let mut parent_id = wrapper.relations.parent;
    ComboBox::from_id_salt(EDIT_BODY_PARENT_COMBO_BOX_SALT)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .wrap_mode(TextWrapMode::Extend)
        .selected_text(
            wrapper
                .relations
                .parent
                .and_then(|parent_id| universe.get_body(parent_id))
                .map(|w| &*w.body.name)
                .unwrap_or("—"),
        )
        .show_ui(ui, |ui| {
            selectable_body_tree(ui, *EDIT_BODY_PARENT_TREE_ID, universe, &mut parent_id);
        });
    ui.end_row();

    parent_id
}
