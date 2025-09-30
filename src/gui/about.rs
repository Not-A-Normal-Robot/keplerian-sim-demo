use three_d::egui::{Align, Context as EguiContext, Layout, RichText, Ui, Window};

use crate::{assets::BANNER, gui::UiState};

/// Get the keplerian_sim version from build.rs
const KEPLERIAN_SIM_VERSION: &str = match option_env!("KEPLERIAN_SIM_VERSION") {
    Some(v) => v,
    None => "unknown",
};

pub(super) fn draw(ctx: &EguiContext, ui_state: &mut UiState) {
    let window = Window::new("About keplerian_sim")
        .open(&mut ui_state.is_about_window_open)
        .resizable(false)
        .collapsible(false);

    window.show(ctx, window_contents);
}

fn window_contents(ui: &mut Ui) {
    ui.allocate_ui_with_layout(
        ui.spacing().interact_size,
        Layout::left_to_right(Align::Max),
        header,
    );
    let image = BANNER.clone();
    ui.add(image);
    ui.allocate_ui_with_layout(
        ui.spacing().interact_size,
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.hyperlink_to(
                "Demo repo",
                "https://github.com/Not-A-Normal-Robot/keplerian-sim-demo",
            );
            ui.separator();
            ui.hyperlink_to(
                "Licensed GPL-3.0-or-later",
                "https://www.gnu.org/licenses/gpl-3.0.en.html",
            );
        },
    );
    ui.separator();

    ui.allocate_ui_with_layout(
        ui.spacing().interact_size,
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.hyperlink_to(
                "Library repo",
                "https://github.com/Not-A-Normal-Robot/keplerian-sim",
            );
            ui.separator();
            ui.hyperlink_to("Library crate", "https://crates.io/keplerian_sim");
            ui.separator();
            ui.hyperlink_to("Library documentation", "https://docs.rs/keplerian_sim");
        },
    );

    ui.separator();

    ui.hyperlink_to(
        "Rendering library (three-d) repo",
        "https://github.com/asny/three-d",
    );

    ui.hyperlink_to("UI library (egui) repo", "https://github.com/emilk/egui");

    ui.separator();

    ui.label("Some math symbols were taken from DejaVu Sans.");
    ui.hyperlink_to("DejaVu Sans' page", "https://dejavu-fonts.github.io/");
}

fn header(ui: &mut Ui) {
    let text = RichText::new("Demo for keplerian_sim").size(26.0);
    ui.label(text);

    ui.label(format!("{}", KEPLERIAN_SIM_VERSION));
}
