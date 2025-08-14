use std::{fmt::Display, str::FromStr};

use float_pretty_print::PrettyPrintFloat;
use strum_macros::{EnumCount, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, EnumIter)]
pub(crate) enum TimeUnit {
    Nanos,
    Micros,
    Millis,
    Seconds,
    Minutes,
    Hours,
    Days,
    Years,
}

const SECOND: f64 = 1.0;
const MILLI: f64 = 1e-3 * SECOND;
const MICRO: f64 = 1e-3 * MILLI;
const NANO: f64 = 1e-3 * MICRO;
const MINUTE: f64 = 60.0 * SECOND;
const HOUR: f64 = 60.0 * MINUTE;
const DAY: f64 = 24.0 * HOUR;
const YEAR: f64 = 365.25 * DAY;

impl TimeUnit {
    pub(crate) const fn get_next_smaller(self) -> Option<Self> {
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
    pub(crate) const fn get_value(self) -> f64 {
        match self {
            TimeUnit::Nanos => NANO,
            TimeUnit::Micros => MICRO,
            TimeUnit::Millis => MILLI,
            TimeUnit::Seconds => SECOND,
            TimeUnit::Minutes => MINUTE,
            TimeUnit::Hours => HOUR,
            TimeUnit::Days => DAY,
            TimeUnit::Years => YEAR,
        }
    }
    pub(crate) const fn largest_unit_from_seconds(seconds: f64) -> Self {
        match seconds {
            x if x.abs() >= YEAR => TimeUnit::Years,
            x if x.abs() >= DAY => TimeUnit::Days,
            x if x.abs() >= HOUR => TimeUnit::Hours,
            x if x.abs() >= MINUTE => TimeUnit::Minutes,
            x if x.abs() >= SECOND => TimeUnit::Seconds,
            x if x.abs() >= MILLI => TimeUnit::Millis,
            x if x.abs() >= MICRO => TimeUnit::Micros,
            _ => TimeUnit::Nanos,
        }
    }
}

impl Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Nanos => write!(f, "ns"),
            TimeUnit::Micros => write!(f, "µs"),
            TimeUnit::Millis => write!(f, "ms"),
            TimeUnit::Seconds => write!(f, "s"),
            TimeUnit::Minutes => write!(f, "m"),
            TimeUnit::Hours => write!(f, "h"),
            TimeUnit::Days => write!(f, "d"),
            TimeUnit::Years => write!(f, "y"),
        }
    }
}

impl FromStr for TimeUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ns" => Ok(TimeUnit::Nanos),
            "us" => Ok(TimeUnit::Micros),
            "ms" => Ok(TimeUnit::Millis),
            "s" => Ok(TimeUnit::Seconds),
            "m" => Ok(TimeUnit::Minutes),
            "h" => Ok(TimeUnit::Hours),
            "d" => Ok(TimeUnit::Days),
            "y" => Ok(TimeUnit::Years),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumCount, EnumIter)]
pub(crate) enum TimeDisplay {
    /// e.g. `1755069111.3 s`,
    SecondsOnly,
    /// Top 4 units, e.g. `14 y, 211 d, 16 h, 49 m`
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
        format!("{:15.15} {}", PrettyPrintFloat(seconds), TimeUnit::Seconds)
    }

    fn format_secs_to_years(mut seconds: f64) -> String {
        const MAX_UNIT_AMOUNT: usize = 4;
        let mut unit = TimeUnit::largest_unit_from_seconds(seconds);
        let mut units = Vec::with_capacity(MAX_UNIT_AMOUNT);

        units.push(unit);

        while let Some(u) = unit.get_next_smaller() {
            if units.len() < MAX_UNIT_AMOUNT {
                units.push(u);
                unit = u;
            } else {
                break;
            }
        }

        let mut string = String::new();

        if seconds.is_sign_negative() {
            string.push('−');
            seconds = seconds.abs();
        }

        for (idx, &unit) in units.iter().enumerate() {
            let unit_value = unit.get_value();
            let (quo, rem) = ((seconds / unit_value).trunc(), seconds % unit_value);

            string += &format!("{quo} {unit}");

            if string.len() >= 30 {
                // Too long!
                break;
            }

            if idx + 1 < units.len() {
                string.push_str(", ");
            }

            seconds = rem;
        }

        string
    }

    fn format_one_unit(seconds: f64) -> String {
        let unit = TimeUnit::largest_unit_from_seconds(seconds);
        let amount = seconds / unit.get_value();

        format!("{:15.15} {unit}", PrettyPrintFloat(amount))
    }

    pub(crate) fn get_next(self) -> Self {
        match self {
            Self::SecondsOnly => Self::MultiUnit,
            Self::MultiUnit => Self::SingleUnit,
            Self::SingleUnit => Self::SecondsOnly,
        }
    }

    pub(crate) fn get_prev(self) -> Self {
        match self {
            Self::SecondsOnly => Self::SingleUnit,
            Self::MultiUnit => Self::SecondsOnly,
            Self::SingleUnit => Self::MultiUnit,
        }
    }
}

impl Display for TimeDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeDisplay::SecondsOnly => write!(f, "seconds-only"),
            TimeDisplay::MultiUnit => write!(f, "multi-unit"),
            TimeDisplay::SingleUnit => write!(f, "single-unit"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const TIME_DISPLAY_ENUM_VARIANTS: usize = 3;

    #[test]
    fn test_next() {
        let mut cur = TimeDisplay::SecondsOnly;
        let mut encountered = HashSet::new();

        while encountered.insert(cur) {
            cur = cur.get_next();
        }

        assert_eq!(encountered.len(), TIME_DISPLAY_ENUM_VARIANTS);

        for variant in encountered {
            let next = variant.get_next();
            let next_prev = next.get_prev();
            assert_eq!(variant, next_prev);
        }
    }
}
