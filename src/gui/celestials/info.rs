use std::sync::Arc;

use crate::sim::{
    body::Body,
    universe::{Id as UniverseId, Universe},
};

use float_pretty_print::PrettyPrintFloat;
use keplerian_sim::OrbitTrait;
use three_d::egui::{Align, Color32, CursorIcon, Label, Layout, RichText, Sense, Ui, WidgetText};

pub(super) fn body_window_info(
    ui: &mut Ui,
    body: &Body,
    parent_id: Option<UniverseId>,
    universe: &Universe,
) {
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
    let mu = body.mass * universe.get_gravitational_constant();

    fn add_value(ui: &mut Ui, text: impl Into<WidgetText>, hover: Arc<RichText>) {
        ui.allocate_ui_with_layout(
            ui.available_size(),
            Layout::right_to_left(Align::Center),
            |ui| {
                ui.add_space(ui.spacing().menu_spacing);
                let label = Label::new(text);
                ui.add(label)
                    .on_hover_text(hover)
                    .on_hover_cursor(CursorIcon::Help);
            },
        );
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
        let hover = RichText::new(hover.trim()).color(Color32::WHITE).size(16.0);
        let hover = Arc::new(hover);

        let label = ui
            .label(measurement)
            .on_hover_text(Arc::clone(&hover))
            .on_hover_cursor(CursorIcon::Help);

        let mut hitbox_rect = label.rect;
        hitbox_rect.set_width(hitbox_rect.width() + ui.available_width());

        let value_text = format_number(value, unit);
        add_value(ui, value_text, Arc::clone(&hover));

        ui.allocate_rect(hitbox_rect, Sense::HOVER)
            .on_hover_text(hover)
            .on_hover_cursor(CursorIcon::Help);

        ui.end_row();
    }

    use core::f64::consts::{PI, TAU};

    add_row(
        ui,
        "Circumference",
        2.0 * PI * body.radius,
        "m",
        include_str!("row_descs/circumference.txt"),
    );

    add_row(
        ui,
        "Surface area",
        4.0 * PI * body.radius.powi(2),
        "m^2",
        include_str!("row_descs/surface_area.txt"),
    );

    add_row(
        ui,
        "Volume",
        4.0 / 3.0 * PI * body.radius.powi(3),
        "m^3",
        include_str!("row_descs/volume.txt"),
    );

    add_row(
        ui,
        "Density",
        body.mass / (4.0 / 3.0 * PI * body.radius.powi(3)),
        "kg/m^3",
        include_str!("row_descs/density.txt"),
    );

    add_row(
        ui,
        "Ideal surface gravity",
        mu / body.radius.powi(2),
        "m/s^2",
        include_str!("row_descs/ideal_surface_gravity.txt"),
    );

    add_row(
        ui,
        "Gravitational parameter",
        mu,
        "m^3 s^-2",
        include_str!("row_descs/gravitational_parameter.txt"),
    );

    add_row(
        ui,
        "Escape velocity",
        (2.0 * mu / body.radius).sqrt(),
        "m/s",
        include_str!("row_descs/escape_velocity.txt"),
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
        include_str!("row_descs/apoapsis.txt"),
    );

    add_row(
        ui,
        "Semi-major axis",
        orbit.get_semi_major_axis(),
        "m",
        include_str!("row_descs/semi_major_axis.txt"),
    );

    add_row(
        ui,
        "Semi-minor axis",
        orbit.get_semi_minor_axis(),
        "m",
        include_str!("row_descs/semi_minor_axis.txt"),
    );

    add_row(
        ui,
        "Linear eccentricity",
        orbit.get_linear_eccentricity(),
        "m",
        include_str!("row_descs/linear_eccentricity.txt"),
    );

    add_row(
        ui,
        "Semi-latus rectum",
        orbit.get_semi_latus_rectum(),
        "m",
        include_str!("row_descs/semi_latus_rectum.txt"),
    );

    let period = orbit.get_orbital_period();

    if orbit.get_eccentricity() <= 1.0 {
        add_row(
            ui,
            "Orbital period",
            period,
            "s",
            include_str!("row_descs/orbital_period.txt"),
        );
    }

    let measurement = if orbit.get_eccentricity() < 1.0 {
        "Curr. mean anomaly"
    } else {
        "Curr. hyp. m. anomaly"
    };

    let hover = if orbit.get_eccentricity() < 1.0 {
        include_str!("row_descs/mean_anomaly.elliptic.txt")
    } else {
        include_str!("row_descs/mean_anomaly.hyperbolic.txt")
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
        include_str!("row_descs/eccentric_anomaly.elliptic.txt")
    } else {
        include_str!("row_descs/eccentric_anomaly.hyperbolic.txt")
    };

    let eccentric_anomaly = orbit.get_eccentric_anomaly_at_mean_anomaly(mean_anomaly);

    add_row(ui, measurement, eccentric_anomaly, "rad", hover);

    let true_anomaly = orbit.get_true_anomaly_at_eccentric_anomaly(eccentric_anomaly);

    add_row(
        ui,
        "Curr. true anomaly",
        true_anomaly,
        "rad",
        include_str!("row_descs/true_anomaly.txt"),
    );

    let altitude = orbit.get_altitude_at_true_anomaly(true_anomaly);

    add_row(
        ui,
        "Curr. altitude",
        altitude,
        "m",
        include_str!("row_descs/altitude.txt"),
    );

    let speed = orbit.get_speed_at_altitude(altitude);

    add_row(
        ui,
        "Curr. speed",
        speed,
        "m/s",
        include_str!("row_descs/speed.txt"),
    );

    let true_sincos = true_anomaly.sin_cos();

    let pqw_position = orbit.get_pqw_position_at_true_anomaly_unchecked(altitude, true_sincos);

    add_row(
        ui,
        "Curr. PQW pos P",
        pqw_position.x,
        "m",
        include_str!("row_descs/pqw_pos_p.txt"),
    );

    add_row(
        ui,
        "Curr. PQW pos Q",
        pqw_position.y,
        "m",
        include_str!("row_descs/pqw_pos_q.txt"),
    );

    let pqw_velocity = orbit.get_pqw_velocity_at_eccentric_anomaly(eccentric_anomaly);

    add_row(
        ui,
        "Curr. PQW vel P",
        pqw_velocity.x,
        "m/s",
        include_str!("row_descs/pqw_vel_p.txt"),
    );

    add_row(
        ui,
        "Curr. PQW vel Q",
        pqw_velocity.y,
        "m/s",
        include_str!("row_descs/pqw_vel_q.txt"),
    );

    let position = orbit.transform_pqw_vector(pqw_position);
    let velocity = orbit.transform_pqw_vector(pqw_velocity);

    add_row(
        ui,
        "Curr. pos X",
        position.x,
        "m",
        include_str!("row_descs/cur_pos_x.txt"),
    );
    add_row(
        ui,
        "Curr. pos Y",
        position.y,
        "m",
        include_str!("row_descs/cur_pos_y.txt"),
    );
    add_row(
        ui,
        "Curr. pos Z",
        position.z,
        "m",
        include_str!("row_descs/cur_pos_z.txt"),
    );

    add_row(
        ui,
        "Curr. vel X",
        velocity.x,
        "m/s",
        include_str!("row_descs/cur_vel_x.txt"),
    );
    add_row(
        ui,
        "Curr. vel Y",
        velocity.y,
        "m/s",
        include_str!("row_descs/cur_vel_y.txt"),
    );
    add_row(
        ui,
        "Curr. vel Z",
        velocity.z,
        "m/s",
        include_str!("row_descs/cur_vel_z.txt"),
    );

    let f_asympt = orbit.get_true_anomaly_at_asymptote();
    if orbit.is_hyperbolic() {
        add_row(
            ui,
            "True anom. asymptote",
            f_asympt,
            "rad",
            include_str!("row_descs/true_anomaly_asymptote.txt"),
        );
    }

    let longitude_of_periapsis = orbit.get_longitude_of_periapsis();

    add_row(
        ui,
        "Longitude of periapsis",
        longitude_of_periapsis,
        "rad",
        include_str!("row_descs/longitude_of_periapsis.txt"),
    );

    add_row(
        ui,
        "Curr. true longitude",
        true_anomaly + longitude_of_periapsis,
        "rad",
        include_str!("row_descs/true_longitude.txt"),
    );

    let soi_radius = parent_id.map(|id| universe.get_soi_radius(id)).flatten();

    if let Some(soi_radius) = soi_radius
        && soi_radius.is_finite()
        && (orbit.is_open() || orbit.get_apoapsis() > soi_radius)
        && let soi_true_anom = orbit.get_true_anomaly_at_altitude(soi_radius)
        && soi_true_anom.is_finite()
    {
        let exit_time = orbit.get_time_at_true_anomaly(soi_true_anom);
        let entry_time = orbit.get_time_at_true_anomaly(-soi_true_anom);

        if orbit.is_open() {
            add_row(
                ui,
                "Time since SOI entry",
                universe.time - entry_time,
                "s",
                include_str!("row_descs/soi_entry_time.txt"),
            );
            add_row(
                ui,
                "Time to SOI exit",
                exit_time - universe.time,
                "s",
                include_str!("row_descs/soi_exit_time.txt"),
            );
        } else {
            add_row(
                ui,
                "Time since SOI entry",
                (universe.time - entry_time).rem_euclid(period),
                "s",
                include_str!("row_descs/soi_entry_time.txt"),
            );

            add_row(
                ui,
                "Time to SOI exit",
                (exit_time - universe.time).rem_euclid(period),
                "s",
                include_str!("row_descs/soi_exit_time.txt"),
            );
        }
    }

    if let Some(parent_wrapper) = parent_id.map(|id| universe.get_body(id)).flatten() {
        // Equation from https://en.wikipedia.org/wiki/Sphere_of_influence_(astrodynamics)
        // r_SOI \approx a (m/M)^(2/5)
        let soi_radius =
            orbit.get_semi_major_axis() * (body.mass / parent_wrapper.body.mass).powf(2.0 / 5.0);

        add_row(
            ui,
            "SOI radius",
            soi_radius,
            "m",
            include_str!("row_descs/soi_radius.txt"),
        );
    }

    let f_an = orbit.get_true_anomaly_at_asc_node();
    let f_dn = orbit.get_true_anomaly_at_desc_node();

    let t_an = orbit.get_time_at_true_anomaly(f_an);
    let t_dn = orbit.get_time_at_true_anomaly(f_dn);

    let (an_time_rel, dn_time_rel) = if orbit.is_open() {
        (t_an - universe.time, t_dn - universe.time)
    } else {
        (
            (t_an - universe.time).rem_euclid(period),
            (t_dn - universe.time).rem_euclid(period),
        )
    };

    if orbit.is_closed() || f_an.abs() < f_asympt {
        add_row(
            ui,
            "Time to AN",
            an_time_rel,
            "s",
            include_str!("row_descs/time_to_an.txt"),
        );
    }
    if orbit.is_closed() || f_dn.abs() < f_asympt {
        add_row(
            ui,
            "Time to DN",
            dn_time_rel,
            "s",
            include_str!("row_descs/time_to_dn.txt"),
        );
    }

    add_row(
        ui,
        "Mean motion",
        orbit.get_mean_motion(),
        "rad/s",
        include_str!("row_descs/mean_motion.txt"),
    );

    add_row(
        ui,
        "Periapsis speed",
        orbit.get_speed_at_periapsis(),
        "m/s",
        include_str!("row_descs/periapsis_speed.txt"),
    );

    if orbit.is_closed() {
        add_row(
            ui,
            "Apoapsis speed",
            orbit.get_speed_at_apoapsis(),
            "m/s",
            include_str!("row_descs/apoapsis_speed.txt"),
        );
    } else {
        add_row(
            ui,
            "Asymptote speed",
            orbit.get_speed_at_infinity(),
            "m/s",
            include_str!("row_descs/asymptote_speed.txt"),
        );
    }

    let periapsis_time = orbit.get_time_of_periapsis();
    let periapsis_time_rel = if orbit.is_open() {
        periapsis_time - universe.time
    } else {
        (periapsis_time - universe.time).rem_euclid(period)
    };

    add_row(
        ui,
        "Time to periapsis",
        periapsis_time_rel,
        "s",
        include_str!("row_descs/time_to_periapsis.txt"),
    );

    if orbit.is_closed() {
        let apoapsis_time = orbit.get_time_of_apoapsis();
        let apoapsis_time_rel = apoapsis_time - universe.time;

        add_row(
            ui,
            "Time to apoapsis",
            apoapsis_time_rel,
            "s",
            include_str!("row_descs/time_to_apoapsis.txt"),
        );
    }

    add_row(
        ui,
        "Focal parameter",
        orbit.get_focal_parameter(),
        "",
        include_str!("row_descs/focal_parameter.txt"),
    );

    add_row(
        ui,
        "Spec. energy",
        orbit.get_specific_orbital_energy(),
        "J/kg",
        include_str!("row_descs/specific_energy.txt"),
    );

    add_row(
        ui,
        "Ang. momentum",
        orbit.get_specific_angular_momentum(),
        "m^2/s",
        include_str!("row_descs/specific_angular_momentum.txt"),
    );

    add_row(
        ui,
        "Area sweep rate",
        orbit.get_area_sweep_rate(),
        "m^2/s",
        include_str!("row_descs/area_sweep_rate.txt"),
    );
}
