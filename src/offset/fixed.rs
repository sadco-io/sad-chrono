// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

//! The time zone which has a fixed offset from UTC.

use core::fmt;
use core::str::FromStr;

#[cfg(any(feature = "rkyv", feature = "rkyv-16", feature = "rkyv-32", feature = "rkyv-64"))]
use rkyv::{Archive, Deserialize, Serialize};

use super::{MappedLocalTime, Offset, TimeZone};
use crate::format::{OUT_OF_RANGE, ParseError, scan};
use crate::naive::{NaiveDate, NaiveDateTime};

/// The time zone with fixed offset, from UTC-23:59:59 to UTC+23:59:59.
///
/// Using the [`TimeZone`](./trait.TimeZone.html) methods
/// on a `FixedOffset` struct is the preferred way to construct
/// `DateTime<FixedOffset>` instances. See the [`east_opt`](#method.east_opt) and
/// [`west_opt`](#method.west_opt) methods for examples.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
#[cfg_attr(
    any(feature = "rkyv", feature = "rkyv-16", feature = "rkyv-32", feature = "rkyv-64"),
    derive(Archive, Deserialize, Serialize),
    archive(compare(PartialEq)),
    archive_attr(derive(Clone, Copy, PartialEq, Eq, Hash, Debug))
)]
pub struct FixedOffset {
    local_minus_utc: i32,
}

impl FixedOffset {
    /// Makes a new `FixedOffset` for the Eastern Hemisphere with given timezone difference.
    /// The negative `secs` means the Western Hemisphere.
    ///
    /// Panics on the out-of-bound `secs`.
    #[deprecated(since = "0.4.23", note = "use `east_opt()` instead")]
    #[must_use]
    pub fn east(secs: i32) -> FixedOffset {
        FixedOffset::east_opt(secs).expect("FixedOffset::east out of bounds")
    }

    /// Makes a new `FixedOffset` for the Eastern Hemisphere with given timezone difference.
    /// The negative `secs` means the Western Hemisphere.
    ///
    /// Returns `None` on the out-of-bound `secs`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use sad_chrono::{FixedOffset, TimeZone};
    /// let hour = 3600;
    /// let datetime =
    ///     FixedOffset::east_opt(5 * hour).unwrap().with_ymd_and_hms(2016, 11, 08, 0, 0, 0).unwrap();
    /// assert_eq!(&datetime.to_rfc3339(), "2016-11-08T00:00:00+05:00")
    /// # }
    /// ```
    #[must_use]
    pub const fn east_opt(secs: i32) -> Option<FixedOffset> {
        if -86_400 < secs && secs < 86_400 {
            Some(FixedOffset { local_minus_utc: secs })
        } else {
            None
        }
    }

    /// Makes a new `FixedOffset` for the Western Hemisphere with given timezone difference.
    /// The negative `secs` means the Eastern Hemisphere.
    ///
    /// Panics on the out-of-bound `secs`.
    #[deprecated(since = "0.4.23", note = "use `west_opt()` instead")]
    #[must_use]
    pub fn west(secs: i32) -> FixedOffset {
        FixedOffset::west_opt(secs).expect("FixedOffset::west out of bounds")
    }

    /// Makes a new `FixedOffset` for the Western Hemisphere with given timezone difference.
    /// The negative `secs` means the Eastern Hemisphere.
    ///
    /// Returns `None` on the out-of-bound `secs`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")] {
    /// use sad_chrono::{FixedOffset, TimeZone};
    /// let hour = 3600;
    /// let datetime =
    ///     FixedOffset::west_opt(5 * hour).unwrap().with_ymd_and_hms(2016, 11, 08, 0, 0, 0).unwrap();
    /// assert_eq!(&datetime.to_rfc3339(), "2016-11-08T00:00:00-05:00")
    /// # }
    /// ```
    #[must_use]
    pub const fn west_opt(secs: i32) -> Option<FixedOffset> {
        if -86_400 < secs && secs < 86_400 {
            Some(FixedOffset { local_minus_utc: -secs })
        } else {
            None
        }
    }

    /// Returns the number of seconds to add to convert from UTC to the local time.
    #[inline]
    pub const fn local_minus_utc(&self) -> i32 {
        self.local_minus_utc
    }

    /// Returns the number of seconds to add to convert from the local time to UTC.
    #[inline]
    pub const fn utc_minus_local(&self) -> i32 {
        -self.local_minus_utc
    }
}

/// Parsing a `str` into a `FixedOffset` uses the format [`%z`](crate::format::strftime).
impl FromStr for FixedOffset {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, offset) = scan::timezone_offset(s, scan::colon_or_space, false, false, true)?;
        Self::east_opt(offset).ok_or(OUT_OF_RANGE)
    }
}

impl TimeZone for FixedOffset {
    type Offset = FixedOffset;

    fn from_offset(offset: &FixedOffset) -> FixedOffset {
        *offset
    }

    fn offset_from_local_date(&self, _local: &NaiveDate) -> MappedLocalTime<FixedOffset> {
        MappedLocalTime::Single(*self)
    }
    fn offset_from_local_datetime(&self, _local: &NaiveDateTime) -> MappedLocalTime<FixedOffset> {
        MappedLocalTime::Single(*self)
    }

    fn offset_from_utc_date(&self, _utc: &NaiveDate) -> FixedOffset {
        *self
    }
    fn offset_from_utc_datetime(&self, _utc: &NaiveDateTime) -> FixedOffset {
        *self
    }
}

impl Offset for FixedOffset {
    fn fix(&self) -> FixedOffset {
        *self
    }
}

impl fmt::Debug for FixedOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let offset = self.local_minus_utc;
        let (sign, offset) = if offset < 0 { ('-', -offset) } else { ('+', offset) };
        let sec = offset.rem_euclid(60);
        let mins = offset.div_euclid(60);
        let min = mins.rem_euclid(60);
        let hour = mins.div_euclid(60);
        if sec == 0 {
            write!(f, "{sign}{hour:02}:{min:02}")
        } else {
            write!(f, "{sign}{hour:02}:{min:02}:{sec:02}")
        }
    }
}

impl fmt::Display for FixedOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for FixedOffset {
    fn format(&self, f: defmt::Formatter) {
        let offset = self.local_minus_utc;
        let (sign, offset) = if offset < 0 { ('-', -offset) } else { ('+', offset) };
        let sec = offset.rem_euclid(60);
        let mins = offset.div_euclid(60);
        let min = mins.rem_euclid(60);
        let hour = mins.div_euclid(60);
        if sec == 0 {
            defmt::write!(f, "{}{:02}:{:02}", sign, hour, min)
        } else {
            defmt::write!(f, "{}{:02}:{:02}:{:02}", sign, hour, min, sec)
        }
    }
}

#[cfg(all(feature = "arbitrary", feature = "std"))]
impl arbitrary::Arbitrary<'_> for FixedOffset {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<FixedOffset> {
        let secs = u.int_in_range(-86_399..=86_399)?;
        let fixed_offset = FixedOffset::east_opt(secs)
            .expect("Could not generate a valid sad_chrono::FixedOffset. It looks like implementation of Arbitrary for FixedOffset is erroneous.");
        Ok(fixed_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedOffset;
    use crate::offset::TimeZone;
    use std::str::FromStr;

    #[test]
    fn test_date_extreme_offset() {
        // starting from 0.3 we don't have an offset exceeding one day.
        // this makes everything easier!
        let offset = FixedOffset::east_opt(86399).unwrap();
        assert_eq!(
            format!("{:?}", offset.with_ymd_and_hms(2012, 2, 29, 5, 6, 7).unwrap()),
            "2012-02-29T05:06:07+23:59:59"
        );
        let offset = FixedOffset::east_opt(-86399).unwrap();
        assert_eq!(
            format!("{:?}", offset.with_ymd_and_hms(2012, 2, 29, 5, 6, 7).unwrap()),
            "2012-02-29T05:06:07-23:59:59"
        );
        let offset = FixedOffset::west_opt(86399).unwrap();
        assert_eq!(
            format!("{:?}", offset.with_ymd_and_hms(2012, 3, 4, 5, 6, 7).unwrap()),
            "2012-03-04T05:06:07-23:59:59"
        );
        let offset = FixedOffset::west_opt(-86399).unwrap();
        assert_eq!(
            format!("{:?}", offset.with_ymd_and_hms(2012, 3, 4, 5, 6, 7).unwrap()),
            "2012-03-04T05:06:07+23:59:59"
        );
    }

    #[test]
    fn test_parse_offset() {
        let offset = FixedOffset::from_str("-0500").unwrap();
        assert_eq!(offset.local_minus_utc, -5 * 3600);
        let offset = FixedOffset::from_str("-08:00").unwrap();
        assert_eq!(offset.local_minus_utc, -8 * 3600);
        let offset = FixedOffset::from_str("+06:30").unwrap();
        assert_eq!(offset.local_minus_utc, (6 * 3600) + 1800);
    }

    #[test]
    #[cfg(all(
        feature = "rkyv-validation",
        any(feature = "rkyv", feature = "rkyv-16", feature = "rkyv-32", feature = "rkyv-64")
    ))]
    fn test_rkyv_validation() {
        use core::mem::{MaybeUninit, size_of};
        use core::ptr::addr_of;

        for valid in [
            FixedOffset::east_opt(-86_399).unwrap(),
            FixedOffset::east_opt(0).unwrap(),
            FixedOffset::east_opt(86_399).unwrap(),
            FixedOffset::from_str("-0500").unwrap(),
        ] {
            let bytes = rkyv::to_bytes::<_, 4>(&valid).unwrap();
            assert!(rkyv::check_archived_root::<FixedOffset>(&bytes).is_ok());
            assert_eq!(rkyv::from_bytes::<FixedOffset>(&bytes).unwrap(), valid);
        }

        let bytes = rkyv::to_bytes::<_, 4>(&FixedOffset::east_opt(0).unwrap()).unwrap();
        let uninit = MaybeUninit::<super::ArchivedFixedOffset>::uninit();
        let base = uninit.as_ptr() as usize;
        let offset_off = unsafe { addr_of!((*uninit.as_ptr()).local_minus_utc) as usize - base };
        let root_off = bytes.len() - size_of::<super::ArchivedFixedOffset>();

        for bad_secs in [86_400, -86_400, i32::MAX, i32::MIN] {
            let mut tampered = bytes.clone();
            let bad_bytes = rkyv::to_bytes::<_, 4>(&bad_secs).unwrap();
            tampered[root_off + offset_off..root_off + offset_off + 4].copy_from_slice(&bad_bytes);
            assert!(rkyv::check_archived_root::<FixedOffset>(&tampered).is_err());
            assert!(rkyv::from_bytes::<FixedOffset>(&tampered).is_err());
        }
    }
}

#[cfg(all(
    feature = "rkyv-validation",
    any(feature = "rkyv", feature = "rkyv-16", feature = "rkyv-32", feature = "rkyv-64")
))]
mod rkyv_check {
    use super::{ArchivedFixedOffset, FixedOffset};
    use core::ptr::addr_of;
    use rkyv::bytecheck::{CheckBytes, ErrorBox, StructCheckError};
    use rkyv::{Archived, Deserialize, Infallible};

    /// `core::num::TryFromIntError` implements `bytecheck::Error` both with and without
    /// `bytecheck/std` (another crate can unify `rkyv/std` while `sad-chrono`'s `std` is off).
    fn invalid(field_name: &'static str) -> StructCheckError {
        StructCheckError { field_name, inner: ErrorBox::new(u8::try_from(256u16).unwrap_err()) }
    }

    impl<C: ?Sized> CheckBytes<C> for ArchivedFixedOffset {
        type Error = StructCheckError;

        #[allow(unreachable_code)] // `CheckBytes` on primitives returns `Infallible`
        unsafe fn check_bytes<'a>(
            value: *const Self,
            context: &mut C,
        ) -> Result<&'a Self, StructCheckError> {
            let local_minus_utc = <Archived<i32> as CheckBytes<C>>::check_bytes(
                addr_of!((*value).local_minus_utc),
                context,
            )
            .map_err(|e| StructCheckError {
                field_name: "local_minus_utc",
                inner: ErrorBox::new(e),
            })?;
            let local_minus_utc: i32 =
                match Deserialize::<i32, _>::deserialize(local_minus_utc, &mut Infallible) {
                    Ok(v) => v,
                    Err(e) => match e {},
                };
            if FixedOffset::east_opt(local_minus_utc).is_none() {
                return Err(invalid("local_minus_utc"));
            }
            Ok(&*value)
        }
    }
}
