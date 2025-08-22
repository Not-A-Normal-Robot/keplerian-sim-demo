use std::{fmt::Display, str::FromStr};

use float_pretty_print::PrettyPrintFloat;
use strum_macros::{EnumCount, EnumIter};

use super::UnitEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumCount, EnumIter)]
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

const TEXT_SECOND: &str = "s";
const TEXT_MILLI: &str = "ms";
const TEXT_MICRO: &str = "µs";
const TEXT_NANO: &str = "ns";
const TEXT_MINUTE: &str = "min";
const TEXT_HOUR: &str = "h";
const TEXT_DAY: &str = "d";
const TEXT_YEAR: &str = "y";

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
    pub(crate) const fn largest_unit_from_base(base: f64) -> Self {
        match base {
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
            TimeUnit::Nanos => write!(f, "{TEXT_NANO}"),
            TimeUnit::Micros => write!(f, "{TEXT_MICRO}"),
            TimeUnit::Millis => write!(f, "{TEXT_MILLI}"),
            TimeUnit::Seconds => write!(f, "{TEXT_SECOND}"),
            TimeUnit::Minutes => write!(f, "{TEXT_MINUTE}"),
            TimeUnit::Hours => write!(f, "{TEXT_HOUR}"),
            TimeUnit::Days => write!(f, "{TEXT_DAY}"),
            TimeUnit::Years => write!(f, "{TEXT_YEAR}"),
        }
    }
}

impl FromStr for TimeUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            TEXT_NANO => Ok(TimeUnit::Nanos),
            TEXT_MICRO => Ok(TimeUnit::Micros),
            TEXT_MILLI => Ok(TimeUnit::Millis),
            TEXT_SECOND => Ok(TimeUnit::Seconds),
            TEXT_MINUTE => Ok(TimeUnit::Minutes),
            TEXT_HOUR => Ok(TimeUnit::Hours),
            TEXT_DAY => Ok(TimeUnit::Days),
            TEXT_YEAR => Ok(TimeUnit::Years),
            _ => Err(()),
        }
    }
}

impl UnitEnum for TimeUnit {
    fn get_next_smaller(self) -> Option<Self> {
        self.get_next_smaller()
    }
    fn get_value(self) -> f64 {
        self.get_value()
    }
    fn largest_unit_from_base(base: f64) -> Self {
        Self::largest_unit_from_base(base)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumCount, EnumIter)]
pub(crate) enum TimeDisplayMode {
    /// e.g. `1755069111.3 s`,
    SecondsOnly,
    /// Top 3 units, e.g. `14 y, 211 d, 16 h`
    MultiUnit,
    /// e.g. `84.602259283 d`
    SingleUnit,
}

impl TimeDisplayMode {
    pub(crate) fn format_time(self, seconds: f64) -> String {
        match self {
            TimeDisplayMode::SecondsOnly => Self::format_secs_only(seconds),
            TimeDisplayMode::MultiUnit => Self::format_secs_to_years(seconds),
            TimeDisplayMode::SingleUnit => Self::format_one_unit(seconds),
        }
    }

    fn format_secs_only(seconds: f64) -> String {
        format!("{:15.15} {}", PrettyPrintFloat(seconds), TimeUnit::Seconds)
    }

    fn format_secs_to_years(mut seconds: f64) -> String {
        const MAX_UNIT_AMOUNT: usize = 3;
        let mut unit = TimeUnit::largest_unit_from_base(seconds);
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

            if quo < 1000.0 {
                string += &format!("{quo} {unit}");
            } else {
                let amount = PrettyPrintFloat(quo);
                string += &format!("{amount:5.3} {unit}");
            }

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
        let unit = TimeUnit::largest_unit_from_base(seconds);
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

impl Display for TimeDisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeDisplayMode::SecondsOnly => write!(f, "seconds-only"),
            TimeDisplayMode::MultiUnit => write!(f, "multi-unit"),
            TimeDisplayMode::SingleUnit => write!(f, "single-unit"),
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
        let mut cur = TimeDisplayMode::SecondsOnly;
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
