use super::SimState;
use three_d::egui::{Context, Window};

pub(super) fn body_edit_window(ctx: &Context, _sim_state: &mut SimState) {
    Window::new("Celestial Editor").show(ctx, |ui| {
        ui.label("This window is not implemented yet.");
    });
}
