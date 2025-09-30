use std::{fmt::Display, str::FromStr};

use strum_macros::{EnumCount, EnumIter};

use crate::units::UnitEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumCount, EnumIter)]
pub(crate) enum MassUnit {
    Micrograms,
    Milligrams,
    Grams,
    Kilograms,
    Tons,
    EarthMasses,
    JupiterMasses,
    SolarMasses,
}

const KILOGRAM: f64 = 1.0;
const GRAM: f64 = 1e-3 * KILOGRAM;
const MILLIGRAM: f64 = 1e-3 * GRAM;
const MICROGRAM: f64 = 1e-3 * MILLIGRAM;
const TON: f64 = 1e3 * KILOGRAM;
const EARTH_MASS: f64 = 5.9722e24 * KILOGRAM;
const JUPITER_MASS: f64 = 1.8982e27 * KILOGRAM;
const SOLAR_MASS: f64 = 1.988416e30 * KILOGRAM;

const TEXT_MICROGRAM: &str = "µg";
const TEXT_MILLIGRAM: &str = "mg";
const TEXT_GRAM: &str = "gram";
const TEXT_KILOGRAM: &str = "kg";
const TEXT_TON: &str = "ton";
const TEXT_EARTH_MASS: &str = "Earth";
const TEXT_JUPITER_MASS: &str = "Jupiter";
const TEXT_SOLAR_MASS: &str = "Sun";

impl MassUnit {
    pub(crate) const fn get_value(self) -> f64 {
        match self {
            MassUnit::Micrograms => MICROGRAM,
            MassUnit::Milligrams => MILLIGRAM,
            MassUnit::Grams => GRAM,
            MassUnit::Kilograms => KILOGRAM,
            MassUnit::Tons => TON,
            MassUnit::EarthMasses => EARTH_MASS,
            MassUnit::JupiterMasses => JUPITER_MASS,
            MassUnit::SolarMasses => SOLAR_MASS,
        }
    }
    pub(crate) const fn largest_unit_from_base(base: f64) -> Self {
        match base {
            x if x.abs() >= SOLAR_MASS => MassUnit::SolarMasses,
            x if x.abs() >= JUPITER_MASS => MassUnit::JupiterMasses,
            x if x.abs() >= EARTH_MASS => MassUnit::EarthMasses,
            x if x.abs() >= TON => MassUnit::Tons,
            x if x.abs() >= KILOGRAM => MassUnit::Kilograms,
            x if x.abs() >= GRAM => MassUnit::Grams,
            x if x.abs() >= MILLIGRAM => MassUnit::Milligrams,
            _ => MassUnit::Micrograms,
        }
    }
}

impl Display for MassUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MassUnit::Micrograms => write!(f, "{TEXT_MICROGRAM}"),
            MassUnit::Milligrams => write!(f, "{TEXT_MILLIGRAM}"),
            MassUnit::Grams => write!(f, "{TEXT_GRAM}"),
            MassUnit::Kilograms => write!(f, "{TEXT_KILOGRAM}"),
            MassUnit::Tons => write!(f, "{TEXT_TON}"),
            MassUnit::EarthMasses => write!(f, "{TEXT_EARTH_MASS}"),
            MassUnit::JupiterMasses => write!(f, "{TEXT_JUPITER_MASS}"),
            MassUnit::SolarMasses => write!(f, "{TEXT_SOLAR_MASS}"),
        }
    }
}

impl FromStr for MassUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            TEXT_MICROGRAM => Ok(MassUnit::Micrograms),
            TEXT_MILLIGRAM => Ok(MassUnit::Milligrams),
            TEXT_GRAM => Ok(MassUnit::Grams),
            TEXT_KILOGRAM => Ok(MassUnit::Kilograms),
            TEXT_TON => Ok(MassUnit::Tons),
            TEXT_EARTH_MASS => Ok(MassUnit::EarthMasses),
            TEXT_JUPITER_MASS => Ok(MassUnit::JupiterMasses),
            TEXT_SOLAR_MASS => Ok(MassUnit::SolarMasses),
            _ => Err(()),
        }
    }
}

impl UnitEnum for MassUnit {
    fn get_value(self) -> f64 {
        self.get_value()
    }
    fn largest_unit_from_base(base: f64) -> Self {
        Self::largest_unit_from_base(base)
    }
}
