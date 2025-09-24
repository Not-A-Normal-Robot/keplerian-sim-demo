use gloo_storage::{LocalStorage, Storage, errors::StorageError};
use serde::{Deserialize, Serialize};

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

pub(super) fn reset() -> Result<(), ResetError> {
    LocalStorage::get_all::<Box<[(String, ())]>>()
        .map_err(|e| ResetError::GetAll(e))?
        .iter()
        .filter(|(s, _)| s.starts_with(PREFIX))
        .for_each(|(k, _)| LocalStorage::delete(k));
    Ok(())
}

pub(crate) type SaveError = StorageError;
pub(crate) type LoadError = StorageError;
pub(crate) enum ResetError {
    GetAll(StorageError),
    Delete(StorageError),
}
