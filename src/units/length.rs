use std::{fmt::Display, str::FromStr};

use strum_macros::{EnumCount, EnumIter};

use super::UnitEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumCount, EnumIter)]
pub(crate) enum LengthUnit {
    Millimeters,
    Meters,
    Kilometers,
    EarthRadii,
    JupiterRadii,
    SolarRadii,
    AstronomicalUnits,
    LightYears,
}

const METER: f64 = 1.0;
const MILLIMETER: f64 = 1e-3 * METER;
const KILOMETER: f64 = 1000.0 * METER;
const EARTH_RADIUS: f64 = 6378137.0 * METER;
const JUPITER_RADIUS: f64 = 69911.0 * KILOMETER;
const SOLAR_RADIUS: f64 = 695700.0 * KILOMETER;
const ASTRONOMICAL_UNIT: f64 = 149597870700.0 * METER;
const LIGHT_YEAR: f64 = 9460730472580.8 * KILOMETER;

const TEXT_MILLIMETER: &str = "mm";
const TEXT_METER: &str = "meter";
const TEXT_KILOMETER: &str = "km";
const TEXT_EARTH_RADIUS: &str = "Earth";
const TEXT_JUPITER_RADIUS: &str = "Jupiter";
const TEXT_SOLAR_RADIUS: &str = "Sun";
const TEXT_ASTRONOMICAL_UNIT: &str = "AU";
const TEXT_LIGHT_YEAR: &str = "ly";

impl LengthUnit {
    pub(crate) const fn get_value(self) -> f64 {
        match self {
            LengthUnit::Millimeters => MILLIMETER,
            LengthUnit::Meters => METER,
            LengthUnit::Kilometers => KILOMETER,
            LengthUnit::EarthRadii => EARTH_RADIUS,
            LengthUnit::JupiterRadii => JUPITER_RADIUS,
            LengthUnit::SolarRadii => SOLAR_RADIUS,
            LengthUnit::AstronomicalUnits => ASTRONOMICAL_UNIT,
            LengthUnit::LightYears => LIGHT_YEAR,
        }
    }
    pub(crate) const fn largest_unit_from_base(base: f64) -> Self {
        match base {
            x if x.abs() >= LIGHT_YEAR => LengthUnit::LightYears,
            x if x.abs() >= ASTRONOMICAL_UNIT => LengthUnit::AstronomicalUnits,
            x if x.abs() >= SOLAR_RADIUS => LengthUnit::SolarRadii,
            x if x.abs() >= JUPITER_RADIUS => LengthUnit::JupiterRadii,
            x if x.abs() >= EARTH_RADIUS => LengthUnit::EarthRadii,
            x if x.abs() >= KILOMETER => LengthUnit::Kilometers,
            x if x.abs() >= METER => LengthUnit::Meters,
            _ => LengthUnit::Millimeters,
        }
    }
}

impl Display for LengthUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LengthUnit::Millimeters => write!(f, "{TEXT_MILLIMETER}"),
            LengthUnit::Meters => write!(f, "{TEXT_METER}"),
            LengthUnit::Kilometers => write!(f, "{TEXT_KILOMETER}"),
            LengthUnit::EarthRadii => write!(f, "{TEXT_EARTH_RADIUS}"),
            LengthUnit::JupiterRadii => write!(f, "{TEXT_JUPITER_RADIUS}"),
            LengthUnit::SolarRadii => write!(f, "{TEXT_SOLAR_RADIUS}"),
            LengthUnit::AstronomicalUnits => write!(f, "{TEXT_ASTRONOMICAL_UNIT}"),
            LengthUnit::LightYears => write!(f, "{TEXT_LIGHT_YEAR}"),
        }
    }
}

impl FromStr for LengthUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            TEXT_MILLIMETER => Ok(LengthUnit::Millimeters),
            TEXT_METER => Ok(LengthUnit::Meters),
            TEXT_KILOMETER => Ok(LengthUnit::Kilometers),
            TEXT_EARTH_RADIUS => Ok(LengthUnit::EarthRadii),
            TEXT_JUPITER_RADIUS => Ok(LengthUnit::JupiterRadii),
            TEXT_SOLAR_RADIUS => Ok(LengthUnit::SolarRadii),
            TEXT_ASTRONOMICAL_UNIT => Ok(LengthUnit::AstronomicalUnits),
            TEXT_LIGHT_YEAR => Ok(LengthUnit::LightYears),
            _ => Err(()),
        }
    }
}

impl UnitEnum for LengthUnit {
    fn get_value(self) -> f64 {
        self.get_value()
    }
    fn largest_unit_from_base(base: f64) -> Self {
        Self::largest_unit_from_base(base)
    }
}
