// Modified from three-d's OrbitControl struct.

#[cfg(target_family = "wasm")]
use std::sync::LazyLock;

use three_d::renderer::*;

///
/// A control that makes the camera orbit around a target.
///
#[derive(Clone, Copy, Debug)]
pub struct CameraControl {
    /// The minimum distance to the target point.
    pub min_distance: f64,
    /// The maximum distance to the target point.
    pub max_distance: f64,
    /// The desired distance to the target point.
    pub desired_distance: f64,
    /// The current distance to the target point.
    pub current_distance: f64,
}

const ZOOM_APPROACH_SPEED: f64 = 0.03;

impl CameraControl {
    /// Creates a new orbit control with the given target and minimum and maximum distance to the target.
    pub fn new(min_distance: f64, max_distance: f64, desired_distance: f64) -> Self {
        Self {
            min_distance,
            max_distance,
            desired_distance,
            current_distance: desired_distance,
        }
    }

    /// Handles the events. Must be called each frame.
    pub fn handle_events(&mut self, camera: &mut Camera, events: &mut [Event], elapsed_time: f64) {
        for event in events.iter_mut() {
            self.handle_event(camera, event);
        }
        self.reclamp();
        self.update_zoom(elapsed_time);
    }

    fn handle_event(&mut self, camera: &mut Camera, event: &mut Event) {
        match event {
            Event::MouseMotion {
                delta,
                button,
                handled,
                ..
            } => {
                if *handled {
                    return;
                }
                if Some(MouseButton::Left) == *button {
                    let speed = 0.01;
                    camera.rotate_around_with_fixed_up(
                        Vec3::zero(),
                        speed * delta.0,
                        speed * delta.1,
                    );
                    let pos = camera.position().normalize();
                    let pos = if is_nan(pos) { Vec3::unit_x() } else { pos };
                    let up = camera.up();
                    camera.set_view(pos, Vec3::zero(), up);
                    *handled = true;
                }
            }
            Event::MouseWheel { delta, handled, .. } => {
                if *handled {
                    return;
                }

                let delta = delta.1 as f64 * -0.02;

                #[cfg(target_family = "wasm")]
                let delta = if *IS_WEB_MOBILE {
                    delta * 1.2
                } else {
                    delta * 0.1
                };

                self.zoom(delta);
                *handled = true;
            }
            Event::PinchGesture { delta, handled, .. } => {
                // This doesn't get run on mobile for some reason
                if *handled {
                    return;
                }
                self.zoom(*delta as f64);
                *handled = true;
            }
            _ => {}
        }
    }
    fn zoom(&mut self, delta: f64) {
        self.desired_distance =
            (self.current_distance * delta.exp()).clamp(self.min_distance, self.max_distance);
    }
    fn reclamp(&mut self) {
        self.desired_distance = self
            .desired_distance
            .clamp(self.min_distance, self.max_distance);
    }
    fn update_zoom(&mut self, elapsed_time: f64) {
        let old_distance = self.current_distance;
        let factor = (-ZOOM_APPROACH_SPEED * elapsed_time).exp().min(1.0);
        let old_diff = self.desired_distance - old_distance;
        let new_diff = old_diff * factor.min(1.0);
        let new_distance = self.desired_distance - new_diff;
        self.current_distance = new_distance;
    }
}

#[cfg(target_family = "wasm")]
static IS_WEB_MOBILE: LazyLock<bool> = LazyLock::new(|| {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };
    let ua = match window.navigator().user_agent().ok() {
        Some(ua) => ua.to_lowercase(),
        None => return false,
    };
    ua.contains("mobi") || ua.contains("android") || ua.contains("iphone") || ua.contains("ios")
});

fn is_nan(vec: Vec3) -> bool {
    vec.x.is_nan() || vec.y.is_nan() || vec.z.is_nan()
}
