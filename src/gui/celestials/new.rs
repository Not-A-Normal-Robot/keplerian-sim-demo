use std::sync::Arc;

use super::{
    PreviewBody, SimState, Universe, UniverseId, declare_id, selectable_body_tree,
    units::{AutoUnit, UnitEnum, length::LengthUnit, mass::MassUnit},
};
use float_pretty_print::PrettyPrintFloat;
use keplerian_sim::{Orbit, OrbitTrait};
use strum::IntoEnumIterator;
use three_d::egui::{
    Align, Color32, ComboBox, Context, DragValue, Grid, Label, Layout, PopupCloseBehavior,
    RichText, Slider, TextEdit, TextWrapMode, Ui, WidgetText, Window,
    color_picker::color_edit_button_srgb,
};

declare_id!(salt_only, NEW_BODY_PHYS, b"Creation");
declare_id!(salt_only, NEW_BODY_ORBIT, b"3111ptic");
declare_id!(salt_only, DRAG_VALUE_WITH_UNIT_PREFIX, b"2ParSecs");
declare_id!(salt_only, NEW_BODY_MASS, b"nMa551ve");
declare_id!(salt_only, NEW_BODY_RADIUS, b"extraRad");
declare_id!(salt_only, NEW_BODY_PARENT_COMBO_BOX, b"dr0pChld");
declare_id!(NEW_BODY_PARENT_TREE, b"treeL1K3");
declare_id!(salt_only, NEW_BODY_INFO_GRID, b"NEEEERD!");

pub(in super::super) struct NewBodyWindowState {
    mass_unit: AutoUnit<MassUnit>,
    radius_unit: AutoUnit<LengthUnit>,
    pub(super) request_focus: bool,
}

impl Default for NewBodyWindowState {
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
            request_focus: true,
        }
    }
}

pub(super) fn new_body_window(ctx: &Context, sim_state: &mut SimState) {
    let mut wrapper = match sim_state.preview_body.take() {
        Some(w) => Some(w),
        None => {
            sim_state.ui.new_body_window_state = None;
            return;
        }
    };

    let window_state = sim_state.ui.new_body_window_state.get_or_insert_default();

    let mut open = true;

    let window = Window::new("New Body")
        .scroll([false, true])
        .resizable([false, true])
        .default_width(300.0)
        .min_width(300.0)
        .max_width(300.0)
        .min_height(200.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let wrapper = match wrapper.take() {
                Some(w) => w,
                None => return,
            };
            ui.scope(|ui| {
                sim_state.preview_body =
                    new_body_window_content(ui, &mut sim_state.universe, wrapper, window_state);
            });
        });

    if let Some(w) = wrapper {
        sim_state.preview_body = Some(w);
    }

    if !open {
        sim_state.preview_body = None;
    }

    if let Some(w) = window
        && window_state.request_focus
    {
        w.response.request_focus();

        if w.response.has_focus() {
            window_state.request_focus = false;
        }
    }
}

fn new_body_window_content(
    ui: &mut Ui,
    universe: &mut Universe,
    mut wrapper: PreviewBody,
    window_state: &mut NewBodyWindowState,
) -> Option<PreviewBody> {
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);

    let text = RichText::new("Physical Characteristics")
        .underline()
        .size(16.0);
    let label = Label::new(text);
    ui.add(label);
    ui.add_space(8.0);
    Grid::new(NEW_BODY_PHYS_SALT)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            new_body_window_phys(ui, &mut wrapper, window_state)
        });

    let text = RichText::new("Orbital Parameters").underline().size(16.0);
    let label = Label::new(text);
    ui.add_space(12.0);
    ui.add(label);
    ui.add_space(8.0);
    Grid::new(NEW_BODY_ORBIT_SALT)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            new_body_window_orbit(
                ui,
                &mut wrapper.body.orbit,
                &mut wrapper.parent_id,
                universe,
            )
        });

    ui.add_space(12.0);

    let derived_info = RichText::new("Derived Information")
        .color(Color32::WHITE)
        .size(16.0)
        .underline();

    ui.collapsing(derived_info, |ui| {
        ui.set_min_width(ui.available_width());
        Grid::new(NEW_BODY_INFO_GRID_SALT)
            .num_columns(2)
            // .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                new_body_window_info(ui, &wrapper, universe);
            });
    });

    ui.add_space(16.0);
    if ui.button("Confirm").clicked() {
        let _ = universe.add_body(wrapper.body, wrapper.parent_id);
        return None;
    }

    return Some(wrapper);
}

fn new_body_window_phys(
    ui: &mut Ui,
    wrapper: &mut PreviewBody,
    window_state: &mut NewBodyWindowState,
) {
    // TODO: Hover popups
    ui.label("Body name");
    ui.add(
        TextEdit::singleline(&mut wrapper.body.name)
            .char_limit(255)
            .hint_text("Enter new body name")
            .desired_width(f32::INFINITY),
    );
    ui.end_row();

    ui.label("Body color");
    let original_srgb: [u8; 3] = wrapper.body.color.into();
    let mut srgb = original_srgb.clone();
    color_edit_button_srgb(ui, &mut srgb);
    if srgb != original_srgb {
        wrapper.body.color = srgb.into();
    }
    ui.end_row();

    ui.label("Mass");
    drag_value_with_unit(
        NEW_BODY_MASS_SALT,
        ui,
        &mut wrapper.body.mass,
        &mut window_state.mass_unit,
    );
    ui.end_row();

    ui.label("Radius");
    drag_value_with_unit(
        NEW_BODY_RADIUS_SALT,
        ui,
        &mut wrapper.body.radius,
        &mut window_state.radius_unit,
    );
    ui.end_row();
}

fn new_body_window_orbit(
    ui: &mut Ui,
    orbit: &mut Option<Orbit>,
    parent_id: &mut Option<UniverseId>,
    universe: &Universe,
) {
    // TODO: Hover popups
    ui.label("Parent body");
    ComboBox::from_id_salt(NEW_BODY_PARENT_COMBO_BOX_SALT)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .wrap_mode(TextWrapMode::Extend)
        .selected_text(
            parent_id
                .map(|parent_id| universe.get_body(parent_id))
                .flatten()
                .map(|w| &*w.body.name)
                .unwrap_or("—"),
        )
        .show_ui(ui, |ui| {
            selectable_body_tree(ui, *NEW_BODY_PARENT_TREE_ID, universe, parent_id);
        });
    ui.end_row();

    let parent_id = match parent_id {
        Some(id) => *id,
        None => return,
    };
    let orbit = orbit.get_or_insert_with(|| {
        let (periapsis, mu) = universe
            .get_body(parent_id)
            .map(|w| {
                (
                    w.body.radius * 2.0,
                    w.body.mass * universe.get_gravitational_constant(),
                )
            })
            .unwrap_or((2.0, 1.0));
        Orbit::new(0.0, periapsis, 0.0, 0.0, 0.0, 0.0, mu)
    });

    ui.label("Eccentricity");
    let mut eccentricity = orbit.get_eccentricity();
    let dv = DragValue::new(&mut eccentricity)
        .range(0.0..=f64::MAX)
        .speed(0.01);
    let dv = ui.add_sized((ui.available_width(), 18.0), dv);
    if dv.changed() {
        orbit.set_eccentricity(eccentricity);
    }
    ui.end_row();

    ui.label("Periapsis");
    let mut periapsis = orbit.get_periapsis();
    let dv = DragValue::new(&mut periapsis)
        .range(0.0..=f64::MAX)
        .suffix(" m");
    let dv = ui.add_sized((ui.available_width(), 18.0), dv);
    if dv.changed() {
        orbit.set_periapsis(periapsis);
    }
    ui.end_row();

    ui.label("Inclination");
    let mut inclination = orbit.get_inclination().to_degrees();
    let slider = Slider::new(&mut inclination, 0.0..=180.0).suffix('°');
    let slider = ui.add_sized((ui.available_width(), 18.0), slider);
    if slider.changed() {
        orbit.set_inclination(inclination.to_radians());
    }
    ui.end_row();

    ui.label("Arg. of Pe.");
    let mut arg_pe = orbit.get_arg_pe().to_degrees();
    let slider = Slider::new(&mut arg_pe, 0.0..=360.0).suffix('°');
    let slider = ui.add(slider);
    if slider.changed() {
        orbit.set_arg_pe(arg_pe.to_radians());
    }
    ui.end_row();

    ui.label("RAAN");
    let mut lan = orbit.get_long_asc_node().to_degrees();
    let slider = Slider::new(&mut lan, 0.0..=360.0).suffix('°');
    let slider = ui.add(slider);
    if slider.changed() {
        orbit.set_long_asc_node(lan.to_radians());
    }
    ui.end_row();

    let mut mean_anomaly = orbit.get_mean_anomaly_at_epoch().to_degrees();
    if orbit.get_eccentricity() < 1.0 {
        ui.label("Mean anom.");
        let slider = Slider::new(&mut mean_anomaly, 0.0..=360.0).suffix('°');
        let slider = ui.add(slider);
        if mean_anomaly < 0.0 || mean_anomaly > 360.0 {
            mean_anomaly = mean_anomaly.rem_euclid(360.0);
        }
        if slider.changed() {
            orbit.set_mean_anomaly_at_epoch(mean_anomaly.to_radians());
        }
    } else {
        ui.label("Hyp. m. anom.");
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

// TODO: Generalize to own file
fn new_body_window_info(ui: &mut Ui, preview_body: &PreviewBody, universe: &Universe) {
    // TODO: Finish
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
    let mu = preview_body.body.mass * universe.get_gravitational_constant();

    fn add_value(ui: &mut Ui, text: impl Into<WidgetText>, hover: impl Into<WidgetText>) {
        let label = Label::new(text);
        ui.add_sized(ui.available_size(), label)
            .on_hover_text(hover);
    }

    fn format_number(number: f64, suffix: &str) -> String {
        let number = PrettyPrintFloat(number);
        if suffix.is_empty() {
            number.to_string()
        } else {
            format!("{number} {suffix}")
        }
    }

    fn add_row(ui: &mut Ui, measurement: &str, value: f64, unit: &str, hover: &str) {
        let hover = Arc::new(RichText::new(hover).color(Color32::WHITE).size(16.0));
        ui.label(measurement).on_hover_text(Arc::clone(&hover));
        let value_text = format_number(value, unit);
        add_value(ui, value_text, Arc::clone(&hover));
        ui.end_row();
    }

    use core::f64::consts::{PI, TAU};

    add_row(
        ui,
        "Circumference",
        2.0 * PI * preview_body.body.radius,
        "m",
        "Circumference (C) of the spherical planet.\n\
        A perfect sphere is assumed.\n\n    \
        C = 2 × pi × r.\n\n\
        ...where:\n\
        r = this body's radius",
    );

    add_row(
        ui,
        "Surface area",
        4.0 * PI * preview_body.body.radius.powi(2),
        "m^2",
        "Surface area (A) of the spherical planet.\n\n    \
        A = 4 × pi × r.\n\n\
        ...where:\n\
        r = this body's radius",
    );

    add_row(
        ui,
        "Volume",
        4.0 / 3.0 * PI * preview_body.body.radius.powi(3),
        "m^3",
        "Volume (V) of the spherical planet.\n\n    \
        V = 4/3 × pi × r^2.\n\n\
        ...where:\n\
        r = this body's radius",
    );

    add_row(
        ui,
        "Density",
        preview_body.body.mass / (4.0 / 3.0 * PI * preview_body.body.radius.powi(3)),
        "kg/m^3",
        "Density (ρ) of the spherical planet.\n\n    \
        V = m ÷ V.\n\n\
        ...where:\n\
        m = this body's mass\n\
        V = this body's volume",
    );

    add_row(
        ui,
        "Ideal surface gravity",
        mu / preview_body.body.radius.powi(2),
        "m/s^2",
        "Surface gravity of the celestial body.\n\
        This assumes an ideal sphere of constant density.\n\
        This is inaccurate to real-life as real planets have\
        an uneven mass distribution.\n\n    \
        g = G × M ÷ r^2.\n\n\
        ...where:\n\
        G = gravitational constant/multiplier\n\
        M = this body's mass\n\
        r = this body's radius",
    );

    add_row(
        ui,
        "Gravitational parameter",
        mu,
        "m^3 s^-2",
        "Standard gravitational parameter (µ).\n\
        This partially describes the magnitude of gravity experienced by bodies that \
        orbit this one, prior to accounting for distance.\n\n    \
        µ = GM.\n\n\
        ...where:\n\
        G = gravitational constant/multiplier\n\
        M = this body's mass",
    );

    add_row(
        ui,
        "Escape velocity",
        (2.0 * mu / preview_body.body.mass).sqrt(),
        "m/s",
        "Escape velocity (v_e) at the surface.\n\n    \
        v_e = √(2µ/d)\n\n\
        ...where:\n\
        μ = standard gravitational parameter of this body\n\
        d = distance (in this case, set to the body's radius)",
    );

    let orbit = match &preview_body.body.orbit {
        Some(o) => o,
        None => return,
    };

    add_row(
        ui,
        "Apoapsis",
        orbit.get_apoapsis(),
        "m",
        "Apoapsis distance (r_a).\n\
        For elliptic orbits, this is the maximum distance between the parent \
        body and this body.
        For parabolic orbits, this value is not finite, and for hyperbolic \
        orbits this is negative.\n\n    \
        r_a = a × (1 - e)\n\n\
        ...where:
        a = semi-major axis\n\
        e = eccentricity",
    );

    add_row(
        ui,
        "Semi-major axis",
        orbit.get_semi_major_axis(),
        "m",
        "Semi-major axis (a) of the orbit.\n\
        For elliptic orbits (e < 1), this is half of the length \
        of the orbital ellipse.\n\n    \
        a = r_p ÷ (1 - e)\n\n\
        ...where:\n\
        r_p = periapsis radius/distance\n\
        e = eccentricity",
    );

    add_row(
        ui,
        "Semi-minor axis",
        orbit.get_semi_minor_axis(),
        "m",
        "Semi-minor axis (b) of the orbit.\n\
        For elliptic orbits (e < 1), this is half of the width \
        of the orbital ellipse.\n\n    \
        b = a √|1 - e^2|\n\n\
        ...where:\n\
        a = semi-major axis\n\
        e = eccentricity",
    );

    add_row(
        ui,
        "Linear eccentricity",
        orbit.get_linear_eccentricity(),
        "m",
        "Linear eccentricity (c) of the orbit.\n\
        In an elliptic orbit, the linear eccentricity is the distance \
        between its center and either of its two foci (focuses).\n\n    \
        c = a - r_p\n\n\
        ...where:\n\
        a = semi-major axis\n\
        r_p = periapsis",
    );

    add_row(
        ui,
        "Semi-latus rectum",
        orbit.get_semi_latus_rectum(),
        "m",
        "Semi-latus rectum (ℓ) of the orbit.\n\
        The semi-latus rectum is half of the length of the \
        chord parallel to the directrix and passing through a focus.\n\n    \
        ℓ = a * (1 - e^2)\n\n\
        ..where:\n\
        a = semi-major axis\n\
        e = eccentricity",
    );

    if orbit.get_eccentricity() <= 1.0 {
        add_row(
            ui,
            "Orbital period",
            orbit.get_orbital_period(),
            "s",
            "Period (T) of the orbit.\n\
            The time it takes to complete one revolution of the orbit.\n\
            Infinite for parabolic trajectories and NaN for hyperbolic trajectories.\n\n    \
            T = 2 × pi × sqrt(a^3 ÷ μ)\n\n\
            ...where:\n\
            a = semi-major axis\n\
            μ = standard gravitational parameter of parent body",
        );
    }

    let measurement = if orbit.get_eccentricity() < 1.0 {
        "Curr. mean anomaly"
    } else {
        "Curr. hyp. m. anomaly"
    };

    let hover = if orbit.get_eccentricity() < 1.0 {
        "The current mean anomaly (M) of the orbit.\n\
        The mean anomaly is the fraction of an elliptical orbit's period \
        that has elapsed since the orbiting body passed periapsis.\n\n    \
        M = t × sqrt(μ ÷ |a^3|) + M_0\n\n\
        ...where:
        t = the current time since epoch\n\
        μ = standard gravitational parameter of parent body\n\
        a = semi-major axis\n\
        M_0 = mean anomaly at epoch\n\
        (This equation is a generalization for all non-parabolic orbits)"
    } else {
        "The current hyperbolic mean anomaly (M_h) of the orbit.\n\
        The mean anomaly is the fraction of an elliptical orbit's period \
        that has elapsed since the orbiting body passed periapsis.\n\
        The hyperbolic mean anomaly is a generalization of this idea to \
        hyperbolic trajectories\n\n    \
        M = t × sqrt(μ ÷ |a^3|) + M_0\n\n\
        ...where:
        t = the current time since epoch\n\
        μ = standard gravitational parameter of parent body\n\
        a = semi-major axis\n\
        M_0 = mean anomaly at epoch\n\
        (This equation is a generalization for all non-parabolic orbits)"
    };

    let mean_anomaly = orbit.get_mean_anomaly_at_time(universe.time);

    let mean_anomaly = if orbit.get_eccentricity() < 1.0 {
        mean_anomaly.rem_euclid(TAU)
    } else {
        mean_anomaly
    };

    add_row(ui, measurement, mean_anomaly, "rad", &hover);

    // TODO:
    // Display:
    // - Eccentric anomaly `E` or `H`
    // - True anomaly `f`
    // - Relative position (x, y, z)
    // - Relative velocity (x, y, z)
    // - Relative PQW position (p, q)
    // - Relative PQW velocity (p, q)
    // - Altitude
    // - Speed
    // Research:
    // - Time until SOI exit (if any)
    // - Time until periapsis (signed if open)
    // - Time until apoapsis (if any)
    // - Time until AN, DN (signed if open)
    // - Focal parameter
    // - True anomaly range, if hyperbolic
    // - Velocity at periapsis `v_p`
    // - Velocity at apoapsis `v_a` or infinity `v_inf`
    // - Specific orbital energy `ε`
    // - Specific angular momentum `h`
    // - Mean motion `n`
    // - Area swept per unit time
    // - Longitude of periapsis
    // - True longitude
    // - This SOI radius
}

fn drag_value_with_unit<'a, U>(
    id_salt: impl std::hash::Hash,
    ui: &mut Ui,
    base_val: &'a mut f64,
    unit: &'a mut AutoUnit<U>,
) where
    U: UnitEnum,
{
    ui.scope(|ui| {
        ui.set_width(ui.available_width());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            drag_value_with_unit_inner(id_salt, ui, base_val, unit)
        });
    });
}

fn drag_value_with_unit_inner<'a, U>(
    id_salt: impl std::hash::Hash,
    ui: &mut Ui,
    base_val: &'a mut f64,
    unit: &'a mut AutoUnit<U>,
) where
    U: UnitEnum,
{
    let unit_scale = unit.get_value();
    let mut scaled_val = *base_val / unit_scale;
    let dv = DragValue::new(&mut scaled_val)
        .custom_formatter(|num, _| format!("{:3.8}", PrettyPrintFloat(num)))
        .range(f64::MIN_POSITIVE..=f64::MAX);
    let cb = ComboBox::from_id_salt((DRAG_VALUE_WITH_UNIT_PREFIX_SALT, id_salt))
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .selected_text(unit.unit.to_string());

    cb.show_ui(ui, |ui: &mut Ui| {
        for unit_variant in <U as IntoEnumIterator>::iter() {
            let button = ui.selectable_label(unit_variant == **unit, unit_variant.to_string());

            if button.clicked() {
                unit.unit = unit_variant;
                unit.auto = false;
            }
        }
        ui.separator();
        let button = ui.selectable_label(unit.auto, "Auto-pick");
        if button.clicked() {
            unit.auto ^= true;
        }
    });

    let dv = ui.add_sized([ui.available_width(), 18.0], dv);

    if dv.changed() {
        *base_val = scaled_val * unit_scale;
    }

    if !dv.dragged() && !dv.has_focus() {
        unit.update(*base_val);
    }
}
