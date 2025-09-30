use std::cell::Cell;

use crate::cfg::storage;
use serde::{Deserialize, Serialize};
pub(crate) struct SavedCell<'a, T>
where
    T: Serialize + for<'d> Deserialize<'d> + ?Sized,
{
    key: &'a str,
    cell: Cell<T>,
    uninit: Cell<bool>,
}

impl<'a, T> SavedCell<'a, T>
where
    T: Serialize + for<'d> Deserialize<'d> + ?Sized + PartialEq,
{
    pub const fn new(key: &'a str, default: T) -> Self {
        Self {
            key,
            cell: Cell::new(default),
            uninit: Cell::new(true),
        }
    }
}

impl<'a, T> SavedCell<'a, T>
where
    T: Serialize + for<'d> Deserialize<'d> + Copy + PartialEq,
{
    pub fn get(&self) -> T {
        if self.uninit.get() {
            let _ = self.load();
        }

        self.cell.get()
    }

    pub fn set(&self, value: T) -> Result<(), storage::SaveError> {
        let old = self.get();
        if old == value {
            return Ok(());
        }
        self.cell.set(value);
        self.save()
    }

    // pub fn update(&self, f: impl FnOnce(T) -> T) {
    //     let old = self.get();
    //     let new = f(old);
    //     if old != new {
    //         self.cell.set(new);
    //         self.save();
    //     }
    // }

    pub fn save(&self) -> Result<(), storage::SaveError> {
        storage::save(self.key, self.cell.get())
    }

    pub fn load(&self) -> Result<T, storage::LoadError> {
        let res = storage::load(self.key)?;
        self.uninit.set(false);
        self.cell.set(res);
        Ok(res)
    }
}
