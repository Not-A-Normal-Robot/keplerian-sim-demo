// Modified from three-d's OrbitControl struct.

#[cfg(target_family = "wasm")]
use std::sync::LazyLock;

use three_d::renderer::*;

///
/// A control that makes the camera orbit around a target.
///
#[derive(Clone, Copy, Debug)]
pub struct CameraControl {
    /// The target point to orbit around.
    pub target: Vec3,
    /// The minimum distance to the target point.
    pub min_distance: f32,
    /// The maximum distance to the target point.
    pub max_distance: f32,
}

impl CameraControl {
    /// Creates a new orbit control with the given target and minimum and maximum distance to the target.
    pub fn new(target: Vec3, min_distance: f32, max_distance: f32) -> Self {
        Self {
            target,
            min_distance,
            max_distance,
        }
    }

    /// Handles the events. Must be called each frame.
    pub fn handle_events(&self, camera: &mut Camera, events: &mut [Event]) -> bool {
        let mut change = false;
        for event in events.iter_mut() {
            self.handle_event(camera, event, &mut change);
        }
        change
    }

    fn handle_event(&self, camera: &mut Camera, event: &mut Event, change: &mut bool) {
        match event {
            Event::MouseMotion {
                delta,
                button,
                handled,
                ..
            } => {
                #[cfg(target_family = "wasm")]
                web_sys::console::log_1(&web_sys::js_sys::JsString::from(format!(
                    "camera info: {:?}",
                    &camera,
                )));
                if *handled {
                    return;
                }
                if Some(MouseButton::Left) == *button {
                    let speed = 0.01;
                    camera.rotate_around_with_fixed_up(
                        self.target,
                        speed * delta.0,
                        speed * delta.1,
                    );
                    let pos = camera.position();
                    let up = camera.up();
                    camera.set_view(pos, self.target, up);
                    *handled = true;
                    *change = true;
                }
            }
            Event::MouseWheel { delta, handled, .. } => {
                if *handled {
                    return;
                }

                // let delta = if cfg!(target_family = "wasm") {
                //     delta.1 * -0.002
                // } else {
                //     delta.1 * -0.02
                // };
                let delta = delta.1 * -0.02;

                #[cfg(target_family = "wasm")]
                let delta = if *IS_WEB_MOBILE {
                    delta * 1.2
                } else {
                    delta * 0.1
                };

                self.zoom(camera, delta);
                *handled = true;
                *change = true;
            }
            Event::PinchGesture { delta, handled, .. } => {
                // This doesn't get run on mobile for some reason
                if *handled {
                    return;
                }
                self.zoom(camera, *delta);
                *handled = true;
                *change = true;
            }
            _ => {}
        }
    }
    fn zoom(&self, camera: &mut Camera, delta: f32) {
        let view = camera.view_direction();
        let distance = self.target.distance(camera.position());
        let new_distance = (distance * delta.exp()).clamp(self.min_distance, self.max_distance);
        let up = camera.up();
        camera.set_view(view * -new_distance, self.target, up);
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
