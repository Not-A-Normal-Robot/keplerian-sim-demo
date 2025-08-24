use core::time::Duration;

use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::wasm_bindgen;

const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(250);
const WATCHDOG_TIMEOUT: Duration = Duration::from_secs(2);

#[wasm_bindgen]
extern "C" {
    fn init_watchdog(heartbeat_interval_ms: f64, watchdog_timeout_ms: f64);
    fn heartbeat();
}

pub(crate) fn start_beating() {
    init_watchdog(
        HEARTBEAT_INTERVAL.as_secs_f64() * 1000.0,
        WATCHDOG_TIMEOUT.as_secs_f64() * 1000.0,
    );

    wasm_bindgen_futures::spawn_local(async {
        loop {
            heartbeat();
            TimeoutFuture::new(HEARTBEAT_INTERVAL.as_millis() as u32).await;
        }
    });
}
