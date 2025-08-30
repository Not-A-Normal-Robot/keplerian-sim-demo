use super::SimState;
use three_d::egui::{Context, Window};

pub(in super::super) struct EditBodyWindowState {
    pub(in super::super) window_open: bool,
}

impl Default for EditBodyWindowState {
    fn default() -> Self {
        Self { window_open: false }
    }
}

pub(super) fn body_edit_window(ctx: &Context, sim_state: &mut SimState) {
    Window::new("Celestial Editor")
        .open(&mut sim_state.ui.edit_body_window_state.window_open)
        .show(ctx, |ui| {
            ui.label("This window is not implemented yet.");
        });
}
