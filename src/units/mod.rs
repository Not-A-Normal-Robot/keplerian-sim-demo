use std::{fmt::Display, ops::Deref, str::FromStr};

use strum::IntoEnumIterator;

pub(crate) mod length;
pub(crate) mod mass;
pub(crate) mod time;

pub(crate) trait UnitEnum: Copy + Display + Eq + Ord + IntoEnumIterator + FromStr {
    fn get_next_smaller(self) -> Option<Self>;
    fn get_value(self) -> f64;
    fn largest_unit_from_base(base: f64) -> Self;
}

pub(crate) struct AutoUnit<U: UnitEnum> {
    pub auto: bool,
    pub unit: U,
}

impl<U: UnitEnum> AutoUnit<U> {
    pub fn update(&mut self, base_value: f64) {
        if !self.auto {
            return;
        }
        self.unit = U::largest_unit_from_base(base_value);
    }
}

impl<U: UnitEnum> Deref for AutoUnit<U> {
    type Target = U;
    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}
