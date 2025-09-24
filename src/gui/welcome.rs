use three_d::egui::{Color32, Hyperlink, OpenUrl, Response, Ui, WidgetText, Window};

use super::{super::cfg::CONFIG, EguiContext, MIN_TOUCH_TARGET_VEC};

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
    ui.spacing_mut().interact_size = MIN_TOUCH_TARGET_VEC;
    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
    ui.heading("Welcome to the keplerian_sim demo");
    ui.label(
        "Here you can experiment with things you can do with the \
            keplerian_sim Rust library.\n\
            A default celestial body system has been created, feel free to explore!\n\
            You can also add new bodies and edit existing bodies.\n\
            If there's a term or UI element that seems unfamiliar, try hovering \
            on it; it might show a description or hint on what it does.\n\n\
            Do note that eccentricities very close to 1 is not supported yet and may \
            result in numerical instabilities.\n\
            Also, rendering breaks at huge scales for now.",
    );
    ui.separator();
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
    ui.separator();
    let cb = ui.checkbox(&mut state.dont_show_again, "Don't show this window again");
    if cb.changed()
        && let Ok(cfg) = CONFIG.try_lock()
    {
        cfg.show_welcome_window.set(!state.dont_show_again);
    }
}
