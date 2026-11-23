#![deny(missing_docs)]
#![allow(dead_code)]
#![warn(unreachable_pub)]

use std::num::ParseIntError;
use std::str::Utf8Error;
use std::time::SystemTimeError;
use std::{error, fmt, io};

mod timezone;
pub(crate) use timezone::TimeZone;
#[cfg(feature = "__internal_fuzz")]
pub(crate) use timezone::{fuzz_tz_string, fuzz_tzif};

mod parser;
mod rule;

/// Unified error type for everything in the crate
#[derive(Debug)]
pub(crate) enum Error {
    /// Date time error
    DateTime(&'static str),
    /// Local time type search error
    FindLocalTimeType(&'static str),
    /// Local time type error
    LocalTimeType(&'static str),
    /// Invalid slice for integer conversion
    InvalidSlice(&'static str),
    /// Invalid Tzif file
    InvalidTzFile(&'static str),
    /// Invalid TZ string
    InvalidTzString(&'static str),
    /// I/O error
    Io(io::Error),
    /// Out of range error
    OutOfRange(&'static str),
    /// Integer parsing error
    ParseInt(ParseIntError),
    /// Date time projection error
    ProjectDateTime(&'static str),
    /// System time error
    SystemTime(SystemTimeError),
    /// Time zone error
    TimeZone(&'static str),
    /// Transition rule error
    TransitionRule(&'static str),
    /// Unsupported Tzif file
    UnsupportedTzFile(&'static str),
    /// Unsupported TZ string
    UnsupportedTzString(&'static str),
    /// UTF-8 error
    Utf8(Utf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            DateTime(error) => write!(f, "invalid date time: {error}"),
            FindLocalTimeType(error) => error.fmt(f),
            LocalTimeType(error) => write!(f, "invalid local time type: {error}"),
            InvalidSlice(error) => error.fmt(f),
            InvalidTzString(error) => write!(f, "invalid TZ string: {error}"),
            InvalidTzFile(error) => error.fmt(f),
            Io(error) => error.fmt(f),
            OutOfRange(error) => error.fmt(f),
            ParseInt(error) => error.fmt(f),
            ProjectDateTime(error) => error.fmt(f),
            SystemTime(error) => error.fmt(f),
            TransitionRule(error) => write!(f, "invalid transition rule: {error}"),
            TimeZone(error) => write!(f, "invalid time zone: {error}"),
            UnsupportedTzFile(error) => error.fmt(f),
            UnsupportedTzString(error) => write!(f, "unsupported TZ string: {error}"),
            Utf8(error) => error.fmt(f),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Error::ParseInt(error)
    }
}

impl From<SystemTimeError> for Error {
    fn from(error: SystemTimeError) -> Self {
        Error::SystemTime(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

/// Number of hours in one day
const HOURS_PER_DAY: i64 = 24;
/// Number of seconds in one hour
const SECONDS_PER_HOUR: i64 = 3600;
/// Number of seconds in one day
const SECONDS_PER_DAY: i64 = SECONDS_PER_HOUR * HOURS_PER_DAY;
/// Number of days in one week
const DAYS_PER_WEEK: i64 = 7;

/// Month days in a normal year
const DAY_IN_MONTHS_NORMAL_YEAR: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
/// Cumulated month days in a normal year
const CUMUL_DAY_IN_MONTHS_NORMAL_YEAR: [i64; 12] =
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

#[cfg(test)]
mod tests {
    use super::Error;
    use super::rule::TransitionRule;
    use super::timezone::{LocalTimeType, TimeZone, Transition};
    use crate::LocalResult::{Ambiguous, Single};
    use crate::{MappedLocalTime, NaiveDateTime, TimeDelta};

    fn datetime(s: &str) -> NaiveDateTime {
        s.parse().expect("test datetime should be valid")
    }

    /// The UTC instants that the local wall time `local` denotes in `tz`.
    fn utc_instants(tz: &TimeZone, local: &str) -> MappedLocalTime<NaiveDateTime> {
        let local = datetime(local);
        tz.find_local_time_type_from_local(local)
            .expect("local time lookup should succeed")
            .map(|ltt| local - TimeDelta::seconds(ltt.offset().into()))
    }

    fn rule_zone(tz_string: &[u8]) -> Result<TimeZone, Error> {
        let rule = TransitionRule::from_tz_string(tz_string, false)?;
        let local_time_types = match rule {
            TransitionRule::Fixed(ltt) => vec![ltt],
            TransitionRule::Alternate(alternate) => vec![alternate.std, alternate.dst],
        };
        TimeZone::new(vec![], local_time_types, vec![], Some(rule))
    }

    /// Europe/Berlin in 2026: CET (UTC+1) and CEST (UTC+2), switching at 01:00 UTC on
    /// 2026-03-29 and 2026-10-25. An ambiguous result lists the earliest instant first.
    #[test]
    fn test_berlin_2026_transition_boundaries() -> Result<(), Error> {
        let cet = LocalTimeType::new(3600, false, Some(b"CET"))?;
        let cest = LocalTimeType::new(7200, true, Some(b"CEST"))?;
        let tzif_transitions = vec![
            Transition::new(datetime("2026-03-29T01:00:00").and_utc().timestamp(), 1),
            Transition::new(datetime("2026-10-25T01:00:00").and_utc().timestamp(), 0),
        ];
        let tzif = TimeZone::new(tzif_transitions, vec![cet, cest], vec![], None)?;
        let rule = rule_zone(b"CET-1CEST,M3.5.0,M10.5.0/3")?;

        let cases = [
            ("2026-03-29T01:59:59", Single(datetime("2026-03-29T00:59:59"))),
            ("2026-03-29T02:00:00", MappedLocalTime::None),
            ("2026-03-29T02:59:59", MappedLocalTime::None),
            ("2026-03-29T03:00:00", Single(datetime("2026-03-29T01:00:00"))),
            ("2026-10-25T01:59:59", Single(datetime("2026-10-24T23:59:59"))),
            (
                "2026-10-25T02:00:00",
                Ambiguous(datetime("2026-10-25T00:00:00"), datetime("2026-10-25T01:00:00")),
            ),
            (
                "2026-10-25T02:30:00",
                Ambiguous(datetime("2026-10-25T00:30:00"), datetime("2026-10-25T01:30:00")),
            ),
            (
                "2026-10-25T02:59:59",
                Ambiguous(datetime("2026-10-25T00:59:59"), datetime("2026-10-25T01:59:59")),
            ),
            ("2026-10-25T03:00:00", Single(datetime("2026-10-25T02:00:00"))),
        ];
        for (local, expected) in cases {
            assert_eq!(utc_instants(&tzif, local), expected, "TZif transitions, local {local}");
            assert_eq!(utc_instants(&rule, local), expected, "TZ rule, local {local}");
        }
        Ok(())
    }

    /// Rules whose fold comes first in the year (Australia/Sydney: AEST UTC+10, AEDT UTC+11),
    /// whose daylight-saving type has the smaller offset (Europe/Dublin: IST UTC+1, GMT UTC+0), or
    /// whose daylight-saving period is empty because it starts and ends at 2026-03-31 23:30 UTC.
    #[test]
    fn test_rule_transition_boundaries_by_utc_instant() -> Result<(), Error> {
        let sydney = rule_zone(b"AEST-10AEDT,M10.1.0,M4.1.0/3")?;
        let dublin = rule_zone(b"IST-1GMT0,M10.5.0,M3.5.0/1")?;
        let empty_dst = rule_zone(b"STD0DST,J90/23:30,J91/0:30")?;

        let cases = [
            (
                &sydney,
                "2026-04-05T02:00:00",
                Ambiguous(datetime("2026-04-04T15:00:00"), datetime("2026-04-04T16:00:00")),
            ),
            (&sydney, "2026-04-05T03:00:00", Single(datetime("2026-04-04T17:00:00"))),
            (&sydney, "2026-10-04T02:00:00", MappedLocalTime::None),
            (&sydney, "2026-10-04T03:00:00", Single(datetime("2026-10-03T16:00:00"))),
            (&dublin, "2026-03-29T01:00:00", MappedLocalTime::None),
            (&dublin, "2026-03-29T02:00:00", Single(datetime("2026-03-29T01:00:00"))),
            (
                &dublin,
                "2026-10-25T01:30:00",
                Ambiguous(datetime("2026-10-25T00:30:00"), datetime("2026-10-25T01:30:00")),
            ),
            (&dublin, "2026-10-25T02:00:00", Single(datetime("2026-10-25T02:00:00"))),
            (&empty_dst, "2026-03-31T23:30:00", Single(datetime("2026-03-31T23:30:00"))),
        ];
        for (tz, local, expected) in cases {
            assert_eq!(utc_instants(tz, local), expected, "local {local}");
        }
        Ok(())
    }

    /// A TZif zone at UTC+0 that adopts the Berlin rule with a last transition to CET at
    /// 2026-10-25 01:00 UTC, the instant that rule ends CEST. Before that instant the offset is
    /// UTC+0, so the rule's CEST reading of the 02:00-03:00 fold denotes no instant.
    #[test]
    fn test_rule_candidates_start_at_last_transition() -> Result<(), Error> {
        let rule = TransitionRule::from_tz_string(b"CET-1CEST,M3.5.0,M10.5.0/3", false)?;
        let utc = LocalTimeType::new(0, false, Some(b"UTC"))?;
        let cet = LocalTimeType::new(3600, false, Some(b"CET"))?;
        let cest = LocalTimeType::new(7200, true, Some(b"CEST"))?;
        let last_transition = datetime("2026-10-25T01:00:00").and_utc().timestamp();
        let tz = TimeZone::new(
            vec![Transition::new(last_transition, 1)],
            vec![utc, cet, cest],
            vec![],
            Some(rule),
        )?;

        let cases = [
            ("2026-10-25T00:59:59", Single(datetime("2026-10-25T00:59:59"))),
            ("2026-10-25T01:30:00", MappedLocalTime::None),
            ("2026-10-25T02:00:00", Single(datetime("2026-10-25T01:00:00"))),
            ("2026-10-25T02:30:00", Single(datetime("2026-10-25T01:30:00"))),
        ];
        for (local, expected) in cases {
            assert_eq!(utc_instants(&tz, local), expected, "local {local}");
        }
        Ok(())
    }
}
