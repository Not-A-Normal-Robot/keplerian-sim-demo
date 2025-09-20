use core::time::Duration;

use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::wasm_bindgen;

const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(250);
const WATCHDOG_TIMEOUT: Duration = Duration::from_secs(2);

static mut HAS_NEW_FRAME: bool = false;

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

    wasm_bindgen_futures::spawn_local(heartbeat_thread());
}

async fn heartbeat_thread() {
    loop {
        heartbeat_loop().await;
    }
}

async fn heartbeat_loop() {
    if unsafe { HAS_NEW_FRAME } {
        heartbeat();
    };
    unsafe {
        HAS_NEW_FRAME = false;
    }

    TimeoutFuture::new(HEARTBEAT_INTERVAL.as_millis() as u32).await;
}

pub(crate) fn update_frame_time() {
    unsafe {
        HAS_NEW_FRAME = true;
    }
}
