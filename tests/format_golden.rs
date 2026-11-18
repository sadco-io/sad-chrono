#[cfg(feature = "serde")]
mod serde_golden {
    use sad_chrono::{
        DateTime, FixedOffset, Month, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone,
        Timelike, Utc, Weekday,
    };
    use serde_derive::{Deserialize, Serialize};

    macro_rules! with_mod {
        ($sname:ident, $ty:ty, $path:literal) => {
            #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
            struct $sname {
                #[serde(with = $path)]
                v: $ty,
            }
        };
    }

    with_mod!(TsS, DateTime<Utc>, "sad_chrono::serde::ts_seconds");
    with_mod!(TsMs, DateTime<Utc>, "sad_chrono::serde::ts_milliseconds");
    with_mod!(TsUs, DateTime<Utc>, "sad_chrono::serde::ts_microseconds");
    with_mod!(TsNs, DateTime<Utc>, "sad_chrono::serde::ts_nanoseconds");
    with_mod!(TsSOpt, Option<DateTime<Utc>>, "sad_chrono::serde::ts_seconds_option");
    with_mod!(TsMsOpt, Option<DateTime<Utc>>, "sad_chrono::serde::ts_milliseconds_option");
    with_mod!(TsUsOpt, Option<DateTime<Utc>>, "sad_chrono::serde::ts_microseconds_option");
    with_mod!(TsNsOpt, Option<DateTime<Utc>>, "sad_chrono::serde::ts_nanoseconds_option");
    with_mod!(NTsSOpt, Option<NaiveDateTime>, "sad_chrono::naive::serde::ts_seconds_option");
    with_mod!(NTsMsOpt, Option<NaiveDateTime>, "sad_chrono::naive::serde::ts_milliseconds_option");
    with_mod!(NTsUsOpt, Option<NaiveDateTime>, "sad_chrono::naive::serde::ts_microseconds_option");
    with_mod!(NTsNsOpt, Option<NaiveDateTime>, "sad_chrono::naive::serde::ts_nanoseconds_option");
    with_mod!(NTsS, NaiveDateTime, "sad_chrono::naive::serde::ts_seconds");
    with_mod!(NTsMs, NaiveDateTime, "sad_chrono::naive::serde::ts_milliseconds");
    with_mod!(NTsUs, NaiveDateTime, "sad_chrono::naive::serde::ts_microseconds");
    with_mod!(NTsNs, NaiveDateTime, "sad_chrono::naive::serde::ts_nanoseconds");

    const fn hex_val(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => panic!("invalid hex digit"),
        }
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        assert_eq!(hex.len() % 2, 0);
        hex.as_bytes().chunks_exact(2).map(|c| (hex_val(c[0]) << 4) | hex_val(c[1])).collect()
    }

    fn check_golden<T>(
        name: &str,
        value: &T,
        expected_json: &str,
        expected_postcard_hex: &str,
        expected_decoded: &T,
    ) where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        assert_eq!(json, expected_json, "{name}: serde_json mismatch");

        let postcard_bytes = postcard::to_allocvec(value).unwrap();
        let expected_bytes = decode_hex(expected_postcard_hex);
        assert_eq!(postcard_bytes, expected_bytes, "{name}: postcard mismatch");

        let from_json: T = serde_json::from_str(expected_json).unwrap();
        assert_eq!(&from_json, expected_decoded, "{name}: serde_json deserialize mismatch");

        let from_postcard: T = postcard::from_bytes(&expected_bytes).unwrap();
        assert_eq!(&from_postcard, expected_decoded, "{name}: postcard deserialize mismatch");
    }

    // Golden vectors captured from sad-chrono 0.4.46 (byte-identical to chrono 0.4.45 serde).
    const VEC_NAIVE_DATE: (&str, &str) = (r#""2016-07-08""#, "0a323031362d30372d3038");
    const VEC_NAIVE_DATE_MIN: (&str, &str) = (r#""-262143-01-01""#, "0d2d3236323134332d30312d3031");
    const VEC_NAIVE_DATE_MAX: (&str, &str) = (r#""+262142-12-31""#, "0d2b3236323134322d31322d3331");
    const VEC_NAIVE_DATE_BCE: (&str, &str) = (r#""-0001-12-31""#, "0b2d303030312d31322d3331");
    const VEC_NAIVE_TIME: (&str, &str) =
        (r#""03:05:07.098765432""#, "1230333a30353a30372e303938373635343332");
    const VEC_NAIVE_TIME_LEAP: (&str, &str) = (r#""23:59:60.500""#, "0c32333a35393a36302e353030");
    const VEC_NAIVE_DATETIME: (&str, &str) = (
        r#""2016-07-08T03:05:07.098765432""#,
        "1d323031362d30372d30385430333a30353a30372e303938373635343332",
    );
    const VEC_NAIVE_DATETIME_MIN: (&str, &str) =
        (r#""-262143-01-01T00:00:00""#, "162d3236323134332d30312d30315430303a30303a3030");
    const VEC_NAIVE_DATETIME_MAX: (&str, &str) = (
        r#""+262142-12-31T23:59:59.999999999""#,
        "202b3236323134322d31322d33315432333a35393a35392e393939393939393939",
    );
    const VEC_DATETIME_UTC: (&str, &str) = (
        r#""2016-07-08T03:05:07.098765432Z""#,
        "1e323031362d30372d30385430333a30353a30372e3039383736353433325a",
    );
    const VEC_DATETIME_FIXED_EAST: (&str, &str) = (
        r#""2016-07-08T08:35:07.098765432+05:30""#,
        "23323031362d30372d30385430383a33353a30372e3039383736353433322b30353a3330",
    );
    const VEC_DATETIME_FIXED_WEST: (&str, &str) = (
        r#""2016-07-07T23:20:00.098765432-03:45""#,
        "23323031362d30372d30375432333a32303a30302e3039383736353433322d30333a3435",
    );
    const VEC_DATETIME_UTC_MIN: (&str, &str) =
        (r#""-262143-01-01T00:00:00Z""#, "172d3236323134332d30312d30315430303a30303a30305a");
    const VEC_DATETIME_UTC_MAX: (&str, &str) = (
        r#""+262142-12-31T23:59:59.999999999Z""#,
        "212b3236323134322d31322d33315432333a35393a35392e3939393939393939395a",
    );
    const VEC_MONTH: (&str, &str) = (r#""September""#, "0953657074656d626572");
    const VEC_WEEKDAY: (&str, &str) = (r#""Thu""#, "03546875");
    const VEC_TS_SECONDS: (&str, &str) = (r#"{"v":1467947107}"#, "c6e1f8f70a");
    const VEC_TS_MILLISECONDS: (&str, &str) = (r#"{"v":1467947107098}"#, "b48cba87b955");
    const VEC_TS_MICROSECONDS: (&str, &str) = (r#"{"v":1467947107098765}"#, "9a82f19de2c59b05");
    const VEC_TS_NANOSECONDS: (&str, &str) = (r#"{"v":1467947107098765432}"#, "f0a1f9bab98799df28");
    const VEC_TS_SECONDS_OPTION_SOME: (&str, &str) = (r#"{"v":1467947107}"#, "01c6e1f8f70a");
    const VEC_TS_SECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_TIME_DELTA_POS: (&str, &str) = (r#"[123456,789012345]"#, "80890ff2fdbaf005");
    const VEC_TIME_DELTA_NEG: (&str, &str) = (r#"[-42,500000000]"#, "538094ebdc03");
    const VEC_TIME_DELTA_MIN: (&str, &str) =
        (r#"[-9223372036854776,193000000]"#, "efcf9adef4a6e22080c987b801");
    const VEC_TIME_DELTA_MAX: (&str, &str) =
        (r#"[9223372036854775,807000000]"#, "eecf9adef4a6e22080dfce8106");
    const VEC_TS_MILLISECONDS_PRE_EPOCH: (&str, &str) = (r#"{"v":-500}"#, "e707");
    const VEC_TS_SECONDS_PRE_EPOCH_INEXACT: (&str, &str) = (r#"{"v":-1}"#, "01");
    const VEC_TS_MILLISECONDS_PRE_EPOCH_INEXACT: (&str, &str) = (r#"{"v":-500}"#, "e707");
    const VEC_TS_MICROSECONDS_PRE_EPOCH_INEXACT: (&str, &str) = (r#"{"v":-499500}"#, "d7fc3c");
    const VEC_TS_NANOSECONDS_PRE_EPOCH_INEXACT: (&str, &str) =
        (r#"{"v":-499499500}"#, "d787aedc03");
    const VEC_TS_MILLISECONDS_OPTION_SOME: (&str, &str) =
        (r#"{"v":1467947107098}"#, "01b48cba87b955");
    const VEC_TS_MILLISECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_TS_MICROSECONDS_OPTION_SOME: (&str, &str) =
        (r#"{"v":1467947107098765}"#, "019a82f19de2c59b05");
    const VEC_TS_MICROSECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_TS_NANOSECONDS_OPTION_SOME: (&str, &str) =
        (r#"{"v":1467947107098765432}"#, "01f0a1f9bab98799df28");
    const VEC_TS_NANOSECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_NAIVE_TS_SECONDS_OPTION_SOME: (&str, &str) = (r#"{"v":1467947107}"#, "01c6e1f8f70a");
    const VEC_NAIVE_TS_SECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_NAIVE_TS_MILLISECONDS_OPTION_SOME: (&str, &str) =
        (r#"{"v":1467947107098}"#, "01b48cba87b955");
    const VEC_NAIVE_TS_MILLISECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_NAIVE_TS_MICROSECONDS_OPTION_SOME: (&str, &str) =
        (r#"{"v":1467947107098765}"#, "019a82f19de2c59b05");
    const VEC_NAIVE_TS_MICROSECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_NAIVE_TS_NANOSECONDS_OPTION_SOME: (&str, &str) =
        (r#"{"v":1467947107098765432}"#, "01f0a1f9bab98799df28");
    const VEC_NAIVE_TS_NANOSECONDS_OPTION_NONE: (&str, &str) = (r#"{"v":null}"#, "00");
    const VEC_NAIVE_TS_SECONDS: (&str, &str) = (r#"{"v":1467947107}"#, "c6e1f8f70a");
    const VEC_NAIVE_TS_MILLISECONDS: (&str, &str) = (r#"{"v":1467947107098}"#, "b48cba87b955");
    const VEC_NAIVE_TS_MICROSECONDS: (&str, &str) =
        (r#"{"v":1467947107098765}"#, "9a82f19de2c59b05");
    const VEC_NAIVE_TS_NANOSECONDS: (&str, &str) =
        (r#"{"v":1467947107098765432}"#, "f0a1f9bab98799df28");

    #[test]
    fn test_serde_golden_0_4_46() {
        let d = NaiveDate::from_ymd_opt(2016, 7, 8).unwrap();
        let d_bce = NaiveDate::from_ymd_opt(-1, 12, 31).unwrap();
        let t = NaiveTime::from_hms_nano_opt(3, 5, 7, 98_765_432).unwrap();
        let leap = NaiveTime::from_hms_milli_opt(23, 59, 59, 1_500).unwrap();
        let ndt = d.and_time(t);
        let utc = Utc.from_utc_datetime(&ndt);
        let dt_east = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap().from_utc_datetime(&ndt);
        let dt_west =
            FixedOffset::west_opt(3 * 3600 + 45 * 60 + 7).unwrap().from_utc_datetime(&ndt);
        // RFC 3339 serializes FixedOffset with minute precision, truncating the 7s sub-minute part.
        let dt_west_decoded = FixedOffset::west_opt(3 * 3600 + 45 * 60)
            .unwrap()
            .with_ymd_and_hms(2016, 7, 7, 23, 20, 0)
            .unwrap()
            .with_nanosecond(98_765_432)
            .unwrap();

        check_golden("naive_date", &d, VEC_NAIVE_DATE.0, VEC_NAIVE_DATE.1, &d);
        check_golden(
            "naive_date_min",
            &NaiveDate::MIN,
            VEC_NAIVE_DATE_MIN.0,
            VEC_NAIVE_DATE_MIN.1,
            &NaiveDate::MIN,
        );
        check_golden(
            "naive_date_max",
            &NaiveDate::MAX,
            VEC_NAIVE_DATE_MAX.0,
            VEC_NAIVE_DATE_MAX.1,
            &NaiveDate::MAX,
        );
        check_golden("naive_date_bce", &d_bce, VEC_NAIVE_DATE_BCE.0, VEC_NAIVE_DATE_BCE.1, &d_bce);
        check_golden("naive_time", &t, VEC_NAIVE_TIME.0, VEC_NAIVE_TIME.1, &t);
        check_golden("naive_time_leap", &leap, VEC_NAIVE_TIME_LEAP.0, VEC_NAIVE_TIME_LEAP.1, &leap);
        check_golden("naive_datetime", &ndt, VEC_NAIVE_DATETIME.0, VEC_NAIVE_DATETIME.1, &ndt);
        check_golden(
            "naive_datetime_min",
            &NaiveDateTime::MIN,
            VEC_NAIVE_DATETIME_MIN.0,
            VEC_NAIVE_DATETIME_MIN.1,
            &NaiveDateTime::MIN,
        );
        check_golden(
            "naive_datetime_max",
            &NaiveDateTime::MAX,
            VEC_NAIVE_DATETIME_MAX.0,
            VEC_NAIVE_DATETIME_MAX.1,
            &NaiveDateTime::MAX,
        );
        check_golden("datetime_utc", &utc, VEC_DATETIME_UTC.0, VEC_DATETIME_UTC.1, &utc);
        check_golden(
            "datetime_fixed_east",
            &dt_east,
            VEC_DATETIME_FIXED_EAST.0,
            VEC_DATETIME_FIXED_EAST.1,
            &dt_east,
        );
        let decoded_east: DateTime<FixedOffset> =
            serde_json::from_str(VEC_DATETIME_FIXED_EAST.0).unwrap();
        assert_eq!(decoded_east.offset(), dt_east.offset());
        check_golden(
            "datetime_fixed_west",
            &dt_west,
            VEC_DATETIME_FIXED_WEST.0,
            VEC_DATETIME_FIXED_WEST.1,
            &dt_west_decoded,
        );
        let decoded_west: DateTime<FixedOffset> =
            serde_json::from_str(VEC_DATETIME_FIXED_WEST.0).unwrap();
        assert_eq!(decoded_west.offset(), dt_west_decoded.offset());
        check_golden(
            "datetime_utc_min",
            &DateTime::<Utc>::MIN_UTC,
            VEC_DATETIME_UTC_MIN.0,
            VEC_DATETIME_UTC_MIN.1,
            &DateTime::<Utc>::MIN_UTC,
        );
        check_golden(
            "datetime_utc_max",
            &DateTime::<Utc>::MAX_UTC,
            VEC_DATETIME_UTC_MAX.0,
            VEC_DATETIME_UTC_MAX.1,
            &DateTime::<Utc>::MAX_UTC,
        );
        check_golden("month", &Month::September, VEC_MONTH.0, VEC_MONTH.1, &Month::September);
        check_golden("weekday", &Weekday::Thu, VEC_WEEKDAY.0, VEC_WEEKDAY.1, &Weekday::Thu);

        let utc_s = Utc.timestamp_opt(1_467_947_107, 0).unwrap();
        let utc_ms = Utc.timestamp_millis_opt(1_467_947_107_098).unwrap();
        let utc_us = Utc.timestamp_micros(1_467_947_107_098_765).unwrap();

        check_golden(
            "ts_seconds",
            &TsS { v: utc },
            VEC_TS_SECONDS.0,
            VEC_TS_SECONDS.1,
            &TsS { v: utc_s },
        );
        check_golden(
            "ts_milliseconds",
            &TsMs { v: utc },
            VEC_TS_MILLISECONDS.0,
            VEC_TS_MILLISECONDS.1,
            &TsMs { v: utc_ms },
        );
        check_golden(
            "ts_microseconds",
            &TsUs { v: utc },
            VEC_TS_MICROSECONDS.0,
            VEC_TS_MICROSECONDS.1,
            &TsUs { v: utc_us },
        );
        check_golden(
            "ts_nanoseconds",
            &TsNs { v: utc },
            VEC_TS_NANOSECONDS.0,
            VEC_TS_NANOSECONDS.1,
            &TsNs { v: utc },
        );
        check_golden(
            "ts_seconds_option_some",
            &TsSOpt { v: Some(utc) },
            VEC_TS_SECONDS_OPTION_SOME.0,
            VEC_TS_SECONDS_OPTION_SOME.1,
            &TsSOpt { v: Some(utc_s) },
        );
        check_golden(
            "ts_seconds_option_none",
            &TsSOpt { v: None },
            VEC_TS_SECONDS_OPTION_NONE.0,
            VEC_TS_SECONDS_OPTION_NONE.1,
            &TsSOpt { v: None },
        );
        check_golden(
            "naive_ts_seconds",
            &NTsS { v: ndt },
            VEC_NAIVE_TS_SECONDS.0,
            VEC_NAIVE_TS_SECONDS.1,
            &NTsS { v: utc_s.naive_utc() },
        );
        check_golden(
            "naive_ts_milliseconds",
            &NTsMs { v: ndt },
            VEC_NAIVE_TS_MILLISECONDS.0,
            VEC_NAIVE_TS_MILLISECONDS.1,
            &NTsMs { v: utc_ms.naive_utc() },
        );
        check_golden(
            "naive_ts_microseconds",
            &NTsUs { v: ndt },
            VEC_NAIVE_TS_MICROSECONDS.0,
            VEC_NAIVE_TS_MICROSECONDS.1,
            &NTsUs { v: utc_us.naive_utc() },
        );
        check_golden(
            "naive_ts_nanoseconds",
            &NTsNs { v: ndt },
            VEC_NAIVE_TS_NANOSECONDS.0,
            VEC_NAIVE_TS_NANOSECONDS.1,
            &NTsNs { v: ndt },
        );

        let delta_pos = TimeDelta::new(123_456, 789_012_345).unwrap();
        let delta_neg = TimeDelta::new(-42, 500_000_000).unwrap();
        check_golden(
            "time_delta_pos",
            &delta_pos,
            VEC_TIME_DELTA_POS.0,
            VEC_TIME_DELTA_POS.1,
            &delta_pos,
        );
        check_golden(
            "time_delta_neg",
            &delta_neg,
            VEC_TIME_DELTA_NEG.0,
            VEC_TIME_DELTA_NEG.1,
            &delta_neg,
        );
        check_golden(
            "time_delta_min",
            &TimeDelta::MIN,
            VEC_TIME_DELTA_MIN.0,
            VEC_TIME_DELTA_MIN.1,
            &TimeDelta::MIN,
        );
        check_golden(
            "time_delta_max",
            &TimeDelta::MAX,
            VEC_TIME_DELTA_MAX.0,
            VEC_TIME_DELTA_MAX.1,
            &TimeDelta::MAX,
        );

        let pre_epoch = DateTime::<Utc>::from_timestamp(-1, 500_000_000).unwrap();
        check_golden(
            "ts_milliseconds_pre_epoch",
            &TsMs { v: pre_epoch },
            VEC_TS_MILLISECONDS_PRE_EPOCH.0,
            VEC_TS_MILLISECONDS_PRE_EPOCH.1,
            &TsMs { v: pre_epoch },
        );

        let pre_epoch_inexact = DateTime::<Utc>::from_timestamp(-1, 500_500_500).unwrap();
        check_golden(
            "ts_seconds_pre_epoch_inexact",
            &TsS { v: pre_epoch_inexact },
            VEC_TS_SECONDS_PRE_EPOCH_INEXACT.0,
            VEC_TS_SECONDS_PRE_EPOCH_INEXACT.1,
            &TsS { v: DateTime::<Utc>::from_timestamp(-1, 0).unwrap() },
        );
        check_golden(
            "ts_milliseconds_pre_epoch_inexact",
            &TsMs { v: pre_epoch_inexact },
            VEC_TS_MILLISECONDS_PRE_EPOCH_INEXACT.0,
            VEC_TS_MILLISECONDS_PRE_EPOCH_INEXACT.1,
            &TsMs { v: DateTime::<Utc>::from_timestamp(-1, 500_000_000).unwrap() },
        );
        check_golden(
            "ts_microseconds_pre_epoch_inexact",
            &TsUs { v: pre_epoch_inexact },
            VEC_TS_MICROSECONDS_PRE_EPOCH_INEXACT.0,
            VEC_TS_MICROSECONDS_PRE_EPOCH_INEXACT.1,
            &TsUs { v: DateTime::<Utc>::from_timestamp(-1, 500_500_000).unwrap() },
        );
        check_golden(
            "ts_nanoseconds_pre_epoch_inexact",
            &TsNs { v: pre_epoch_inexact },
            VEC_TS_NANOSECONDS_PRE_EPOCH_INEXACT.0,
            VEC_TS_NANOSECONDS_PRE_EPOCH_INEXACT.1,
            &TsNs { v: pre_epoch_inexact },
        );

        check_golden(
            "ts_milliseconds_option_some",
            &TsMsOpt { v: Some(utc) },
            VEC_TS_MILLISECONDS_OPTION_SOME.0,
            VEC_TS_MILLISECONDS_OPTION_SOME.1,
            &TsMsOpt { v: Some(utc_ms) },
        );
        check_golden(
            "ts_milliseconds_option_none",
            &TsMsOpt { v: None },
            VEC_TS_MILLISECONDS_OPTION_NONE.0,
            VEC_TS_MILLISECONDS_OPTION_NONE.1,
            &TsMsOpt { v: None },
        );
        check_golden(
            "ts_microseconds_option_some",
            &TsUsOpt { v: Some(utc) },
            VEC_TS_MICROSECONDS_OPTION_SOME.0,
            VEC_TS_MICROSECONDS_OPTION_SOME.1,
            &TsUsOpt { v: Some(utc_us) },
        );
        check_golden(
            "ts_microseconds_option_none",
            &TsUsOpt { v: None },
            VEC_TS_MICROSECONDS_OPTION_NONE.0,
            VEC_TS_MICROSECONDS_OPTION_NONE.1,
            &TsUsOpt { v: None },
        );
        check_golden(
            "ts_nanoseconds_option_some",
            &TsNsOpt { v: Some(utc) },
            VEC_TS_NANOSECONDS_OPTION_SOME.0,
            VEC_TS_NANOSECONDS_OPTION_SOME.1,
            &TsNsOpt { v: Some(utc) },
        );
        check_golden(
            "ts_nanoseconds_option_none",
            &TsNsOpt { v: None },
            VEC_TS_NANOSECONDS_OPTION_NONE.0,
            VEC_TS_NANOSECONDS_OPTION_NONE.1,
            &TsNsOpt { v: None },
        );
        check_golden(
            "naive_ts_seconds_option_some",
            &NTsSOpt { v: Some(ndt) },
            VEC_NAIVE_TS_SECONDS_OPTION_SOME.0,
            VEC_NAIVE_TS_SECONDS_OPTION_SOME.1,
            &NTsSOpt { v: Some(utc_s.naive_utc()) },
        );
        check_golden(
            "naive_ts_seconds_option_none",
            &NTsSOpt { v: None },
            VEC_NAIVE_TS_SECONDS_OPTION_NONE.0,
            VEC_NAIVE_TS_SECONDS_OPTION_NONE.1,
            &NTsSOpt { v: None },
        );
        check_golden(
            "naive_ts_milliseconds_option_some",
            &NTsMsOpt { v: Some(ndt) },
            VEC_NAIVE_TS_MILLISECONDS_OPTION_SOME.0,
            VEC_NAIVE_TS_MILLISECONDS_OPTION_SOME.1,
            &NTsMsOpt { v: Some(utc_ms.naive_utc()) },
        );
        check_golden(
            "naive_ts_milliseconds_option_none",
            &NTsMsOpt { v: None },
            VEC_NAIVE_TS_MILLISECONDS_OPTION_NONE.0,
            VEC_NAIVE_TS_MILLISECONDS_OPTION_NONE.1,
            &NTsMsOpt { v: None },
        );
        check_golden(
            "naive_ts_microseconds_option_some",
            &NTsUsOpt { v: Some(ndt) },
            VEC_NAIVE_TS_MICROSECONDS_OPTION_SOME.0,
            VEC_NAIVE_TS_MICROSECONDS_OPTION_SOME.1,
            &NTsUsOpt { v: Some(utc_us.naive_utc()) },
        );
        check_golden(
            "naive_ts_microseconds_option_none",
            &NTsUsOpt { v: None },
            VEC_NAIVE_TS_MICROSECONDS_OPTION_NONE.0,
            VEC_NAIVE_TS_MICROSECONDS_OPTION_NONE.1,
            &NTsUsOpt { v: None },
        );
        check_golden(
            "naive_ts_nanoseconds_option_some",
            &NTsNsOpt { v: Some(ndt) },
            VEC_NAIVE_TS_NANOSECONDS_OPTION_SOME.0,
            VEC_NAIVE_TS_NANOSECONDS_OPTION_SOME.1,
            &NTsNsOpt { v: Some(ndt) },
        );
        check_golden(
            "naive_ts_nanoseconds_option_none",
            &NTsNsOpt { v: None },
            VEC_NAIVE_TS_NANOSECONDS_OPTION_NONE.0,
            VEC_NAIVE_TS_NANOSECONDS_OPTION_NONE.1,
            &NTsNsOpt { v: None },
        );
    }
}

#[cfg(all(feature = "rkyv-32", feature = "rkyv-validation"))]
mod rkyv_golden {
    use sad_chrono::{
        DateTime, Datelike, FixedOffset, Month, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc,
        Weekday,
    };

    // These vectors pin the sad-chrono 0.5.0 rkyv 0.8 (little-endian, 32-bit pointer width) archive format.
    const RKYV_NAIVE_DATE: &[u8] = &[0xe3, 0x0b, 0xfc, 0x00];
    const RKYV_NAIVE_TIME: &[u8] = &[0x63, 0x2b, 0x00, 0x00, 0x78, 0x0a, 0xe3, 0x05];
    const RKYV_NAIVE_TIME_LEAP: &[u8] = &[0x7f, 0x51, 0x01, 0x00, 0x00, 0x2f, 0x68, 0x59];
    const RKYV_NAIVE_DATETIME: &[u8] =
        &[0xe3, 0x0b, 0xfc, 0x00, 0x63, 0x2b, 0x00, 0x00, 0x78, 0x0a, 0xe3, 0x05];
    const RKYV_DATETIME_UTC: &[u8] =
        &[0xe3, 0x0b, 0xfc, 0x00, 0x63, 0x2b, 0x00, 0x00, 0x78, 0x0a, 0xe3, 0x05];
    const RKYV_DATETIME_FIXED_EAST: &[u8] = &[
        0xe3, 0x0b, 0xfc, 0x00, 0x63, 0x2b, 0x00, 0x00, 0x78, 0x0a, 0xe3, 0x05, 0x58, 0x4d, 0x00,
        0x00,
    ];
    const RKYV_FIXED_OFFSET_EAST: &[u8] = &[0x58, 0x4d, 0x00, 0x00];
    const RKYV_TIME_DELTA: &[u8] = &[
        0x40, 0xe2, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79, 0x5f, 0x07, 0x2f, 0x00, 0x00, 0x00,
        0x00,
    ];
    const RKYV_TIME_DELTA_NEG: &[u8] = &[
        0xd6, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x65, 0xcd, 0x1d, 0x00, 0x00, 0x00,
        0x00,
    ];
    const RKYV_MONTH: &[u8] = &[0x08];
    const RKYV_WEEKDAY: &[u8] = &[0x03];
    const RKYV_ISO_WEEK_2020_W53: &[u8] = &[0x51, 0x93, 0x1f, 0x00];

    fn check_rkyv_golden<T>(name: &str, value: &T, expected: &[u8])
    where
        T: PartialEq + core::fmt::Debug + rkyv::Archive,
        T: for<'a> rkyv::Serialize<
                rkyv::api::high::HighSerializer<
                    rkyv::util::AlignedVec,
                    rkyv::ser::allocator::ArenaHandle<'a>,
                    rkyv::rancor::Error,
                >,
            >,
        T::Archived: rkyv::Deserialize<T, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>
            + for<'a> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>,
            >,
    {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value).unwrap();
        assert_eq!(bytes.as_slice(), expected, "{name}: rkyv bytes mismatch");
        let mut aligned = rkyv::util::AlignedVec::<16>::new();
        aligned.extend_from_slice(expected);
        let decoded = rkyv::from_bytes::<T, rkyv::rancor::Error>(&aligned).unwrap();
        assert_eq!(&decoded, value, "{name}: rkyv deserialize mismatch");
    }

    #[test]
    fn test_rkyv_golden_0_5_0() {
        let d = NaiveDate::from_ymd_opt(2016, 7, 8).unwrap();
        let t = NaiveTime::from_hms_nano_opt(3, 5, 7, 98_765_432).unwrap();
        let leap = NaiveTime::from_hms_milli_opt(23, 59, 59, 1_500).unwrap();
        let ndt = d.and_time(t);
        let utc = Utc.from_utc_datetime(&ndt);
        let fixed_east = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
        let dt_fixed = fixed_east.from_utc_datetime(&ndt);
        let delta = TimeDelta::new(123_456, 789_012_345).unwrap();
        let delta_neg = TimeDelta::new(-42, 500_000_000).unwrap();

        check_rkyv_golden("naive_date", &d, RKYV_NAIVE_DATE);
        check_rkyv_golden("naive_time", &t, RKYV_NAIVE_TIME);
        check_rkyv_golden("naive_time_leap", &leap, RKYV_NAIVE_TIME_LEAP);
        check_rkyv_golden("naive_datetime", &ndt, RKYV_NAIVE_DATETIME);
        check_rkyv_golden("datetime_utc", &utc, RKYV_DATETIME_UTC);
        check_rkyv_golden("datetime_fixed_east", &dt_fixed, RKYV_DATETIME_FIXED_EAST);
        let mut aligned_fixed = rkyv::util::AlignedVec::<16>::new();
        aligned_fixed.extend_from_slice(RKYV_DATETIME_FIXED_EAST);
        let decoded_fixed =
            rkyv::from_bytes::<DateTime<FixedOffset>, rkyv::rancor::Error>(&aligned_fixed).unwrap();
        assert_eq!(decoded_fixed.offset(), dt_fixed.offset());
        check_rkyv_golden("fixed_offset_east", &fixed_east, RKYV_FIXED_OFFSET_EAST);
        check_rkyv_golden("time_delta", &delta, RKYV_TIME_DELTA);
        check_rkyv_golden("time_delta_neg", &delta_neg, RKYV_TIME_DELTA_NEG);
        check_rkyv_golden("month", &Month::September, RKYV_MONTH);
        check_rkyv_golden("weekday", &Weekday::Thu, RKYV_WEEKDAY);
        let iso_week = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap().iso_week();
        assert_eq!(iso_week.week(), 53);
        check_rkyv_golden("iso_week_2020_w53", &iso_week, RKYV_ISO_WEEK_2020_W53);
    }
}
