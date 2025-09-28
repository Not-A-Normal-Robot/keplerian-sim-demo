use three_d::egui::{
    CollapsingResponse, Color32, Grid, OpenUrl, Response, RichText, Ui, WidgetText, Window,
};

use super::{super::cfg::CONFIG, EguiContext, declare_id};

declare_id!(salt_only, KEYBINDS_GRID, b"BINGINGS");

pub(super) struct WindowState {
    open: bool,
    dont_show_again: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            open: CONFIG
                .try_lock()
                .map(|c| c.show_welcome_window.get())
                .unwrap_or(true),
            dont_show_again: false,
        }
    }
}

pub(super) fn draw(ctx: &EguiContext, state: &mut WindowState) {
    let mut open = state.open;
    Window::new("Welcome")
        .open(&mut open)
        .vscroll(true)
        .default_height(480.0)
        .show(ctx, |ui| draw_window_contents(ui, state));
    state.open &= open;
}

fn hyperlink_button(ui: &mut Ui, label: impl Into<WidgetText>, url: impl ToString) -> Response {
    let button = ui.button(label);
    if button.clicked_with_open_in_background() {
        ui.ctx().open_url(OpenUrl::new_tab(url));
    } else if button.clicked() {
        ui.ctx().open_url(OpenUrl::same_tab(url));
    }
    button
}

fn draw_window_contents(ui: &mut Ui, state: &mut WindowState) {
    // ui.spacing_mut().interact_size = MIN_TOUCH_TARGET_VEC;
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
    ui.heading("Welcome to the keplerian_sim demo");
    ui.label(
        "Here you can experiment with things you can do with the \
        keplerian_sim Rust library.\n\
        Feel free to explore around the solar system!\n\
        You can also add new bodies and edit existing bodies.\n\
        If there's a term or UI element that seems unfamiliar, try hovering \
        on it; it might show a description or hint on what it does.",
    );
    section(ui, "Keplerian orbits", draw_intro);
    section(ui, "Keybinds", draw_keybinds);
    section(ui, "Links", draw_links);
    section(ui, "Issues", draw_issues);
    ui.separator();
    let cb = ui.checkbox(
        &mut state.dont_show_again,
        RichText::new("Don't show this window again").color(Color32::WHITE),
    );
    if cb.changed()
        && let Ok(cfg) = CONFIG.try_lock()
    {
        let _ = cfg.show_welcome_window.set(!state.dont_show_again);
    }
}

fn section<I>(ui: &mut Ui, title: &str, content: fn(&mut Ui) -> I) -> CollapsingResponse<I> {
    let collapsing = ui.collapsing(
        RichText::new(title)
            .heading()
            .color(Color32::WHITE)
            .underline(),
        content,
    );

    collapsing
}

fn draw_intro(ui: &mut Ui) {
    ui.label(
        "Keplerian orbits are special in that they are more stable \
        and predictable than Newtonian orbits. In fact, unlike Newtonian \
        orbits, Keplerian orbits don’t use time steps to calculate the next \
        position of an object. Keplerian orbits use elements to determine \
        the object’s full trajectory at any given time.\n\
        What's being simulated here is known as an osculating orbit, where \
        the orbit only considers the gravitational pull of its parent body \
        and does not take into account the gravitational pull of other bodies.\n\
        For example, in this simulation, Ceres is not affected by Jupiter's \
        gravitational pull.",
    );
}

fn draw_keybinds(ui: &mut Ui) {
    const KEYBINDS: [(&str, &str); 8] = [
        (",", "Multiply time by 0.5×"),
        (".", "Multiply time by 2×"),
        ("Shift + ,", "Multiply time by 0.1×"),
        ("Shift + .", "Multiply time by 10×"),
        ("N", "Create a new body"),
        ("E", "Edit the currently-focused body"),
        ("[", "Switch focus to the previous body in the list"),
        ("]", "Switch focus to the next body in the list"),
    ];

    Grid::new(KEYBINDS_GRID_SALT)
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for (binding, action) in KEYBINDS {
                ui.label(RichText::new(binding).code());
                ui.label(action);
                ui.end_row();
            }
        });
}

fn draw_links(ui: &mut Ui) {
    ui.columns_const(|[l, r]| {
        l.label("Demo links");
        hyperlink_button(
            l,
            "Repository",
            "https://github.com/Not-A-Normal-Robot/keplerian-sim-demo",
        );
        hyperlink_button(
            l,
            "Issue tracker",
            "https://github.com/Not-A-Normal-Robot/keplerian-sim-demo/issues",
        );
        hyperlink_button(
            l,
            "Report bug",
            "https://github.com/Not-A-Normal-Robot/keplerian-sim-demo/issues/new",
        );

        r.label("Library links");
        hyperlink_button(
            r,
            "Repository",
            "https://github.com/Not-A-Normal-Robot/keplerian-sim",
        );
        hyperlink_button(
            r,
            "Documentation",
            "https://docs.rs/keplerian_sim/latest/keplerian_sim/",
        );
        hyperlink_button(
            r,
            "crates.io page",
            "https://crates.io/crates/keplerian_sim",
        );
        hyperlink_button(r, "lib.rs page", "https://lib.rs/crates/keplerian_sim");
    });
}

fn draw_issues(ui: &mut Ui) {
    ui.label(
        "Eccentricities very close to 1 are not supported yet \
        and may result in glitches and numerical instabilities.\n\
        Far away objects get hidden the more you zoom in. This is a problem \
        due to the way the depth buffer works in the framework used here.\n\
        Collisions and light is not simulated realistically. This is an orbit \
        simulator, not a direct ripoff of Universe Sandbox.",
    );
}
