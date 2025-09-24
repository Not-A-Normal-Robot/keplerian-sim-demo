use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

pub(super) fn save<T: Serialize>(key: &str, value: T) -> Result<(), SaveError> {
    LocalStorage::set(key, value)
}

pub(super) fn load<T: for<'a> Deserialize<'a>>(key: &str) -> Result<T, LoadError> {
    LocalStorage::get(key)
}

pub(crate) type SaveError = gloo_storage::errors::StorageError;
pub(crate) type LoadError = gloo_storage::errors::StorageError;
