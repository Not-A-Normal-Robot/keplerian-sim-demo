use std::fmt::Display;

use float_pretty_print::PrettyPrintFloat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DateUnit {
    Nanos,
    Micros,
    Millis,
    Seconds,
    Minutes,
    Hours,
    Days,
    Years,
}

impl DateUnit {
    const fn get_next_smaller(self) -> Option<Self> {
        match self {
            Self::Nanos => None,
            Self::Micros => Some(Self::Nanos),
            Self::Millis => Some(Self::Micros),
            Self::Seconds => Some(Self::Millis),
            Self::Minutes => Some(Self::Seconds),
            Self::Hours => Some(Self::Minutes),
            Self::Days => Some(Self::Hours),
            Self::Years => Some(Self::Days),
        }
    }
    const fn get_next_larger(self) -> Option<Self> {
        match self {
            Self::Nanos => Some(Self::Micros),
            Self::Micros => Some(Self::Millis),
            Self::Millis => Some(Self::Seconds),
            Self::Seconds => Some(Self::Minutes),
            Self::Minutes => Some(Self::Hours),
            Self::Hours => Some(Self::Days),
            Self::Days => Some(Self::Years),
            Self::Years => None,
        }
    }
    const fn get_value(self) -> f64 {
        match self {
            DateUnit::Nanos => 1e-9,
            DateUnit::Micros => 1e-6,
            DateUnit::Millis => 1e-3,
            DateUnit::Seconds => 1.0,
            DateUnit::Minutes => 60.0,
            DateUnit::Hours => 3600.0,
            DateUnit::Days => 86400.0,
            DateUnit::Years => const { 86400.0 * 365.25 },
        }
    }
}

impl Display for DateUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateUnit::Nanos => write!(f, "ns"),
            DateUnit::Micros => write!(f, "us"),
            DateUnit::Millis => write!(f, "ms"),
            DateUnit::Seconds => write!(f, "s"),
            DateUnit::Minutes => write!(f, "m"),
            DateUnit::Hours => write!(f, "h"),
            DateUnit::Days => write!(f, "d"),
            DateUnit::Years => write!(f, "y"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimeDisplay {
    /// e.g. `1755069111.3 s`,
    SecondsOnly,
    /// e.g. `14 y, 211 d, 16 h, 49 m`
    MultiUnit,
    /// e.g. `84.602259283 d`
    SingleUnit,
}

impl TimeDisplay {
    pub(crate) fn format_time(self, seconds: f64) -> String {
        match self {
            TimeDisplay::SecondsOnly => Self::format_secs_only(seconds),
            TimeDisplay::MultiUnit => Self::format_secs_to_years(seconds),
            TimeDisplay::SingleUnit => Self::format_one_unit(seconds),
        }
    }

    fn format_secs_only(seconds: f64) -> String {
        format!("{:12.12} s", PrettyPrintFloat(seconds))
    }

    fn format_secs_to_years(seconds: f64) -> String {
        todo!();
    }

    fn format_one_unit(seconds: f64) -> String {
        todo!();
    }
}
