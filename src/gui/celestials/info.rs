use std::sync::Arc;

use float_pretty_print::PrettyPrintFloat;
use keplerian_sim::OrbitTrait;
use three_d::egui::{Color32, Label, RichText, Ui, WidgetText};

use super::{Body, Universe};

pub(super) fn body_window_info(ui: &mut Ui, body: &Body, universe: &Universe) {
    // TODO: Finish
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
    let mu = body.mass * universe.get_gravitational_constant();

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
        2.0 * PI * body.radius,
        "m",
        include_str!("row-descs/circumference.txt"),
    );

    add_row(
        ui,
        "Surface area",
        4.0 * PI * body.radius.powi(2),
        "m^2",
        include_str!("row-descs/surface_area.txt"),
    );

    add_row(
        ui,
        "Volume",
        4.0 / 3.0 * PI * body.radius.powi(3),
        "m^3",
        include_str!("row-descs/volume.txt"),
    );

    add_row(
        ui,
        "Density",
        body.mass / (4.0 / 3.0 * PI * body.radius.powi(3)),
        "kg/m^3",
        include_str!("row-descs/density.txt"),
    );

    add_row(
        ui,
        "Ideal surface gravity",
        mu / body.radius.powi(2),
        "m/s^2",
        include_str!("row-descs/ideal_surface_gravity.txt"),
    );

    add_row(
        ui,
        "Gravitational parameter",
        mu,
        "m^3 s^-2",
        include_str!("row-descs/gravitational_parameter.txt"),
    );

    add_row(
        ui,
        "Escape velocity",
        (2.0 * mu / body.mass).sqrt(),
        "m/s",
        include_str!("row-descs/escape_velocity.txt"),
    );

    let orbit = match &body.orbit {
        Some(o) => o,
        None => return,
    };

    add_row(
        ui,
        "Apoapsis",
        orbit.get_apoapsis(),
        "m",
        include_str!("row-descs/apoapsis.txt"),
    );

    add_row(
        ui,
        "Semi-major axis",
        orbit.get_semi_major_axis(),
        "m",
        include_str!("row-descs/semi_major_axis.txt"),
    );

    add_row(
        ui,
        "Semi-minor axis",
        orbit.get_semi_minor_axis(),
        "m",
        include_str!("row-descs/semi_minor_axis.txt"),
    );

    add_row(
        ui,
        "Linear eccentricity",
        orbit.get_linear_eccentricity(),
        "m",
        include_str!("row-descs/linear_eccentricity.txt"),
    );

    add_row(
        ui,
        "Semi-latus rectum",
        orbit.get_semi_latus_rectum(),
        "m",
        include_str!("row-descs/semi_latus_rectum.txt"),
    );

    if orbit.get_eccentricity() <= 1.0 {
        add_row(
            ui,
            "Orbital period",
            orbit.get_orbital_period(),
            "s",
            include_str!("row-descs/orbital_period.txt"),
        );
    }

    let measurement = if orbit.get_eccentricity() < 1.0 {
        "Curr. mean anomaly"
    } else {
        "Curr. hyp. m. anomaly"
    };

    let hover = if orbit.get_eccentricity() < 1.0 {
        include_str!("row-descs/mean_anomaly.elliptic.txt")
    } else {
        include_str!("row-descs/mean_anomaly.hyperbolic.txt")
    };

    let mean_anomaly = orbit.get_mean_anomaly_at_time(universe.time);

    let mean_anomaly = if orbit.get_eccentricity() < 1.0 {
        mean_anomaly.rem_euclid(TAU)
    } else {
        mean_anomaly
    };

    add_row(ui, measurement, mean_anomaly, "rad", &hover);

    let measurement = if orbit.get_eccentricity() < 1.0 {
        "Curr. ecc. anomaly"
    } else {
        "Curr. hyp. e. anomaly"
    };

    let hover = if orbit.get_eccentricity() < 1.0 {
        include_str!("row-descs/eccentric_anomaly.elliptic.txt")
    } else {
        include_str!("row-descs/eccentric_anomaly.hyperbolic.txt")
    };

    let eccentric_anomaly = orbit.get_eccentric_anomaly_at_mean_anomaly(mean_anomaly);

    add_row(ui, measurement, eccentric_anomaly, "rad", hover);

    let true_anomaly = orbit.get_true_anomaly_at_eccentric_anomaly(eccentric_anomaly);

    add_row(
        ui,
        "Curr. true anomaly",
        true_anomaly,
        "rad",
        include_str!("row-descs/true_anomaly.txt"),
    );

    let altitude = orbit.get_altitude_at_true_anomaly(true_anomaly);

    add_row(
        ui,
        "Curr. altitude",
        altitude,
        "m",
        include_str!("row-descs/altitude.txt"),
    );

    let speed = orbit.get_speed_at_altitude(altitude);

    add_row(
        ui,
        "Curr. speed",
        speed,
        "m/s",
        include_str!("row-descs/speed.txt"),
    );

    let true_sincos = true_anomaly.sin_cos();

    let pqw_position = orbit.get_pqw_position_at_true_anomaly_unchecked(altitude, true_sincos);

    add_row(
        ui,
        "Cur. PQW pos P",
        pqw_position.x,
        "m",
        include_str!("row-descs/pqw_pos_p.txt"),
    );

    add_row(
        ui,
        "Cur. PQW pos Q",
        pqw_position.y,
        "m",
        include_str!("row-descs/pqw_pos_q.txt"),
    );

    let pqw_velocity = orbit.get_pqw_velocity_at_eccentric_anomaly(eccentric_anomaly);

    add_row(
        ui,
        "Cur. PQW vel P",
        pqw_velocity.x,
        "m/s",
        include_str!("row-descs/pqw_vel_p.txt"),
    );

    add_row(
        ui,
        "Cur. PQW vel Q",
        pqw_velocity.y,
        "m/s",
        include_str!("row-descs/pqw_vel_q.txt"),
    );

    let position = orbit.transform_pqw_vector(pqw_position);
    let velocity = orbit.transform_pqw_vector(pqw_velocity);

    add_row(
        ui,
        "Cur. pos X",
        position.x,
        "m",
        include_str!("row-descs/cur_pos_x.txt"),
    );
    add_row(
        ui,
        "Cur. pos Y",
        position.y,
        "m",
        include_str!("row-descs/cur_pos_y.txt"),
    );
    add_row(
        ui,
        "Cur. pos Z",
        position.z,
        "m",
        include_str!("row-descs/cur_pos_z.txt"),
    );

    add_row(
        ui,
        "Cur. vel X",
        velocity.x,
        "m/s",
        include_str!("row-descs/cur_vel_x.txt"),
    );
    add_row(
        ui,
        "Cur. vel Y",
        velocity.y,
        "m/s",
        include_str!("row-descs/cur_vel_y.txt"),
    );
    add_row(
        ui,
        "Cur. vel Z",
        velocity.z,
        "m/s",
        include_str!("row-descs/cur_vel_z.txt"),
    );

    // TODO:
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
