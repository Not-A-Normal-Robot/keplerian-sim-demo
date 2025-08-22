use std::{fmt::Display, str::FromStr};

use strum::IntoEnumIterator;

pub(crate) mod length;
pub(crate) mod mass;
pub(crate) mod time;

pub(crate) trait UnitEnum: Copy + Display + Eq + Ord + IntoEnumIterator + FromStr {
    fn get_next_smaller(self) -> Option<Self>;
    fn get_value(self) -> f64;
    fn largest_unit_from_base(base: f64) -> Self;
}
