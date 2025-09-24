use std::sync::Mutex;

pub(crate) mod saved_cell;

#[cfg_attr(target_family = "wasm", path = "storage_web.rs")]
#[cfg_attr(not(target_family = "wasm"), path = "storage_native.rs")]
mod storage;

pub(crate) use storage::reset;

use saved_cell::SavedCell;

pub(crate) struct Config<'a> {
    pub show_body_list_help: SavedCell<'a, bool>,
    pub show_welcome_window: SavedCell<'a, bool>,
}

impl Config<'_> {
    pub const fn new() -> Self {
        Self {
            show_body_list_help: SavedCell::new("show_body_list_help", true),
            show_welcome_window: SavedCell::new("show_welcome_window", true),
        }
    }
}

pub(crate) static CONFIG: Mutex<Config> = Mutex::new(Config::new());
