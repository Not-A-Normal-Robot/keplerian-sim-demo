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
        "Circumference (C) of the spherical planet.\n\
        A perfect sphere is assumed.\n\n    \
        C = 2 ∙ pi ∙ r.\n\n\
        ...where:\n\
        r = this body's radius",
    );

    add_row(
        ui,
        "Surface area",
        4.0 * PI * body.radius.powi(2),
        "m^2",
        "Surface area (A) of the spherical planet.\n\n    \
        A = 4 ∙ pi ∙ r.\n\n\
        ...where:\n\
        r = this body's radius",
    );

    add_row(
        ui,
        "Volume",
        4.0 / 3.0 * PI * body.radius.powi(3),
        "m^3",
        "Volume (V) of the spherical planet.\n\n    \
        V = 4/3 ∙ pi ∙ r^2.\n\n\
        ...where:\n\
        r = this body's radius",
    );

    add_row(
        ui,
        "Density",
        body.mass / (4.0 / 3.0 * PI * body.radius.powi(3)),
        "kg/m^3",
        "Density (ρ) of the spherical planet.\n\n    \
        ρ = m / V.\n\n\
        ...where:\n\
        m = this body's mass\n\
        V = this body's volume",
    );

    add_row(
        ui,
        "Ideal surface gravity",
        mu / body.radius.powi(2),
        "m/s^2",
        "Surface gravity of the celestial body.\n\
        This assumes an ideal sphere of constant density.\n\
        This is inaccurate to real-life as real planets have\
        an uneven mass distribution.\n\n    \
        g = G ∙ M / r^2.\n\n\
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
        (2.0 * mu / body.mass).sqrt(),
        "m/s",
        "Escape velocity (v_e) at the surface.\n\n    \
        v_e = √(2µ/d).\n\n\
        ...where:\n\
        μ = standard gravitational parameter of this body\n\
        d = distance (in this case, set to the body's radius)",
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
        "Apoapsis distance (r_a).\n\
        For elliptic orbits, this is the maximum distance between the parent \
        body and this body.
        For parabolic orbits, this value is not finite, and for hyperbolic \
        orbits this is negative.\n\n    \
        r_a = a ∙ (1 - e).\n\n\
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
        a = r_p / (1 - e).\n\n\
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
        b = a √|1 - e^2|.\n\n\
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
        c = a - r_p.\n\n\
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
        ℓ = a ∙ (1 - e^2).\n\n\
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
            T = 2 × pi × sqrt(a^3 / μ).\n\n\
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
        M = t × sqrt(μ / |a^3|) + M_0.\n\n\
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
        M = t × sqrt(μ / |a^3|) + M_0.\n\n\
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

    let measurement = if orbit.get_eccentricity() < 1.0 {
        "Curr. ecc. anomaly"
    } else {
        "Curr. hyp. e. anomaly"
    };

    let hover = if orbit.get_eccentricity() < 1.0 {
        "The current eccentric anomaly (E) of the orbit.\n\
        The eccentric anomaly is the angle measured at the center of the \
        ellipse between the orbit's periapsis and the current position when projected \
        into a circle fully containing the orbit ellipse.\n\
        This value is derived from the mean anomaly using \
        numerical approach methods and is \
        used to derive the true anomaly.\n\n    \
        M = E − e sin E.\n\n\
        ...where:\n\
        M = current mean anomaly\n\
        e = eccentricity\n"
    } else {
        "The current hyperbolic eccentric anomaly (H) of the orbit.\n\
        The hyperbolic eccentric anomaly is a generalization of the elliptic \
        eccentric anomaly (M) for hyperbolic orbits, where the elliptic eccentric \
        anomaly is defined as the angle measured at the center of the ellipse \
        between the orbit's periapsis and the current position when projected \
        into a circle fully containing the orbit ellipse.\n\
        The value is derived from the mean anomaly using \
        numerical approach methods and is used to derive the true anomaly.\n\n    \
        M_h = H - e sinh (H) - H.\n\n\
        ...where:\n\
        M_h = current hyperbolic mean anomaly\n\
        e = eccentricity"
    };

    let eccentric_anomaly = orbit.get_eccentric_anomaly_at_mean_anomaly(mean_anomaly);

    add_row(ui, measurement, eccentric_anomaly, "rad", hover);

    let true_anomaly = orbit.get_true_anomaly_at_eccentric_anomaly(eccentric_anomaly);

    add_row(
        ui,
        "Curr. true anomaly",
        true_anomaly,
        "rad",
        "The current true anomaly (ν) of the orbit.\n\
        The true anomaly of the orbit is defined as the angle measured at the parent body \
        between the periapsis and the current position of this body.\n\n  \
        Elliptic case (e < 1):\n    \
        ν = 2 arctan((β sin E) / (1 - β cos E)).\n    \
        β = e / (1 + √(1 - e^2)).\n  \
        Hyperbolic case (e > 1):\n    \
        ν = 2 arctan(tanh(H / 2) ∙ √((e + 1) / (e - 1))).\n\n\
        ...where:\n\
        E = current eccentric anomaly\n\
        H = current hyperbolic eccentric anomaly\n\
        e = eccentricity",
    );

    let altitude = orbit.get_altitude_at_true_anomaly(true_anomaly);

    add_row(
        ui,
        "Curr. altitude",
        altitude,
        "m",
        "The current altitude/radius (r) of the orbit.\n\
        The altitude is the distance between this body and the body that it orbits,\n\
        measured from their centers (not the surface).\n\n    \
        r = |ℓ / (1 + e cos ν)|.\n\n\
        ...where:\n\
        ℓ = semi-latus rectum\n\
        e = eccentricity\n\
        ν = current true anomaly",
    );

    let speed = orbit.get_speed_at_altitude(altitude);

    add_row(
        ui,
        "Curr. speed",
        speed,
        "m/s",
        "The current orbital speed, in meters per second.\n\
        This is relative to the parent body and not an absolute speed.\n\n    \
        v = sqrt(μ * (2/r - 1/a)).\n\
        ...where:\n\
        μ = standard gravitational parameter of the parent body\n\
        r = current altitude\n\
        a = semi-major axis",
    );

    let true_sincos = true_anomaly.sin_cos();

    let pqw_position = orbit.get_pqw_position_at_true_anomaly_unchecked(altitude, true_sincos);

    add_row(
        ui,
        "Cur. PQW pos P",
        pqw_position.x,
        "m",
        "The current P-position relative to the parent body.\n\
        P-axis, in the perifocal coordinate system, \
        points towards the periapsis point in the orbit.\n\n    \
        o.p = r cos ν.\n\n\
        ...where:\n
        r = distance to parent body center\n\
        ν = true anomaly",
    );

    add_row(
        ui,
        "Cur. PQW pos Q",
        pqw_position.y,
        "m",
        "The current Q-position relative to the parent body.\n\
        Q-axis, in the perifocal coordinate system, \
        points perpendicular to the periapsis point in the orbit, on the orbital plane.\n\n    \
        o.q = r sin ν.\n\n\
        ...where:\n
        r = distance to parent body center\n\
        ν = true anomaly",
    );

    let pqw_velocity = orbit.get_pqw_velocity_at_eccentric_anomaly(eccentric_anomaly);

    add_row(
        ui,
        "Cur. PQW vel P",
        pqw_velocity.x,
        "m/s",
        "The current P-velocity relative to the parent body.\n\
        P-axis, in the perifocal coordinate system, \
        points towards the periapsis point in the orbit.\n\n    \
        o'.p = -ms.\n    \
        m = √|μa| / r.\n    \
        s = e < 1: sin E; else: sinh H.\n\n\
        ...where:\n\
        μ = standard gravitational parameter of parent body\n\
        a = semi-major axis\n\
        r = distance to parent body center\n\
        e = eccentricity\n\
        E = current (elliptic) eccentric anomaly\n\
        H = current hyperbolic eccentric anomaly",
    );

    add_row(
        ui,
        "Cur. PQW vel Q",
        pqw_velocity.y,
        "m/s",
        "The current Q-velocity relative to the parent body.\n\
        Q-axis, in the perifocal coordinate system, \
        points perpendicular to the periapsis point in the orbit, on the orbital plane.\n\n    \
        o'.q = mqc.\n    \
        m = √(|μa|) / r.\n    \
        q = √(|1 - e^2|)
        c = e < 1: cos E; else: cosh H.\n\n\
        ...where:\n\
        μ = standard gravitational parameter of parent body\n\
        a = semi-major axis\n\
        r = distance to parent body center\n\
        e = eccentricity\n\
        E = current (elliptic) eccentric anomaly\n\
        H = current hyperbolic eccentric anomaly",
    );

    // TODO:
    // Display:
    // - Relative position (x, y, z)
    // - Relative velocity (x, y, z)
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
