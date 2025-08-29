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
    /// The desired distance to the target point.
    pub desired_distance: f32,
}

impl CameraControl {
    /// Creates a new orbit control with the given target and minimum and maximum distance to the target.
    pub fn new(target: Vec3, min_distance: f32, max_distance: f32, desired_distance: f32) -> Self {
        Self {
            target,
            min_distance,
            max_distance,
            desired_distance,
        }
    }

    /// Handles the events. Must be called each frame.
    pub fn handle_events(&mut self, camera: &mut Camera, events: &mut [Event], elapsed_time: f32) {
        for event in events.iter_mut() {
            self.handle_event(camera, event);
        }
        self.reclamp();
        self.update_zoom(camera, elapsed_time);
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
                        self.target,
                        speed * delta.0,
                        speed * delta.1,
                    );
                    let pos = camera.position();
                    let up = camera.up();
                    camera.set_view(pos, self.target, up);
                    *handled = true;
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
            }
            Event::PinchGesture { delta, handled, .. } => {
                // This doesn't get run on mobile for some reason
                if *handled {
                    return;
                }
                self.zoom(camera, *delta);
                *handled = true;
            }
            _ => {}
        }
    }
    fn zoom(&mut self, camera: &Camera, delta: f32) {
        let distance = self.target.distance(camera.position());
        self.desired_distance =
            (distance * delta.exp()).clamp(self.min_distance, self.max_distance);
    }
    fn reclamp(&mut self) {
        // let view = camera.view_direction();
        // let distance = self.target.distance(camera.position());
        // let up = camera.up();
        // if distance < self.min_distance {
        //     camera.set_view(view * -self.min_distance, self.target, up);
        // } else if distance > self.max_distance {
        //     camera.set_view(view * -self.max_distance, self.target, up);
        // }
        self.desired_distance = self
            .desired_distance
            .clamp(self.min_distance, self.max_distance);
    }
    fn update_zoom(&self, camera: &mut Camera, elapsed_time: f32) {
        let view = camera.view_direction();
        let up = camera.up();
        let old_distance = self.target.distance(camera.position());
        let factor = (-0.03 * elapsed_time).exp().min(1.0);
        let old_diff = self.desired_distance - old_distance;
        let new_diff = old_diff * factor.min(1.0);
        let new_distance = self.desired_distance - new_diff;
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
