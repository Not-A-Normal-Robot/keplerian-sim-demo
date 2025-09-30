use crate::HALT_FLAG;
use gloo_storage::{LocalStorage, Storage, errors::StorageError};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const PREFIX: &str = "/keplerian-sim-demo | ";
fn to_storage_key(key: &str) -> String {
    let mut result = String::with_capacity(PREFIX.len().saturating_add(key.len()));
    result += PREFIX;
    result += key;
    result
}

pub(super) fn save<T: Serialize>(key: &str, value: T) -> Result<(), SaveError> {
    LocalStorage::set(&to_storage_key(key), value)
}

pub(super) fn load<T: for<'a> Deserialize<'a>>(key: &str) -> Result<T, LoadError> {
    LocalStorage::get(&to_storage_key(key))
}

pub(crate) fn reset() -> Result<(), ResetError> {
    LocalStorage::get_all::<serde_json::Map<String, serde_json::Value>>()
        .map_err(|e| ResetError::GetAll(e))?
        .keys()
        .filter(|k| k.starts_with(PREFIX))
        .for_each(|k| LocalStorage::delete(k));
    let window = web_sys::window().ok_or(ResetError::NoWindow)?;
    window
        .location()
        .reload()
        .map_err(|e| ResetError::Reload(e))?;
    unsafe {
        HALT_FLAG = true;
    }
    Ok(())
}

pub(crate) type SaveError = StorageError;
pub(crate) type LoadError = StorageError;

#[derive(Debug)]
pub(crate) enum ResetError {
    GetAll(StorageError),
    NoWindow,
    #[allow(dead_code)]
    Reload(wasm_bindgen::JsValue),
}
impl Display for ResetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResetError::GetAll(error) => write!(f, "GetAll: {error}"),
            ResetError::NoWindow => write!(f, "No window found"),
            ResetError::Reload(_) => write!(f, "Reload failed"),
        }
    }
}
