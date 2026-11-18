# sad-chrono

[![Build Status][gh-image]][gh-checks]
[![crates.io][cratesio-image]][cratesio]
[![Documentation][docsrs-image]][docsrs]
![Apache2/MIT licensed][license-image]
![MSRV][msrv-image]

[gh-image]: https://github.com/sadco-io/sad-chrono/actions/workflows/test.yml/badge.svg?branch=main
[gh-checks]: https://github.com/sadco-io/sad-chrono/actions/workflows/test.yml?query=branch%3Amain
[cratesio-image]: https://img.shields.io/crates/v/sad-chrono.svg
[cratesio]: https://crates.io/crates/sad-chrono
[docsrs-image]: https://docs.rs/sad-chrono/badge.svg
[docsrs]: https://docs.rs/sad-chrono
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[msrv-image]: https://img.shields.io/badge/rustc-1.62+-blue.svg

Timezone-aware date and time handling for Rust: a **maintained, API-compatible fork of
[chrono][upstream]** with security and robustness fixes.

Upstream chrono has been soft-deprecated by its maintainers ([chrono#1768][soft-deprecation]), and
fixes that landed after `0.4.45` were never released. `sad-chrono` keeps the chrono 0.4 API stable,
ships those fixes, and takes on new ones. It is maintained by the Sadco Security Team alongside
[sad-rsa](https://github.com/sadco-io/sad-rsa).

## Usage

To switch an existing project over **without touching any code**, rename the dependency so that
`use chrono::...` keeps working:

```toml
[dependencies]
chrono = { package = "sad-chrono", version = "0.4.46" }
```

Or depend on it under its own name and import from `sad_chrono`:

```toml
[dependencies]
sad-chrono = "0.4.46"
```

```rust
use sad_chrono::{DateTime, TimeZone, Utc};

let dt = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();
assert_eq!(dt.to_rfc3339(), "2026-11-17T12:00:00+00:00");
```

Feature flags are the same as chrono's (see [Crate features](#crate-features)).

## Compatibility with the `chrono` crate

**Version-line contract: `sad-chrono 0.4.x` is API-compatible with `chrono 0.4.x`.** The version
numbers are aligned deliberately, so the compatibility target is clear from the dependency line
alone. `sad-chrono 0.4.46` was forked from chrono's `main` branch after `0.4.45` (commit
[`6adaa52`][fork-point]).

The public API, feature flags, MSRV and serialization formats (serde, rkyv) are unchanged. The only
behavior changes are fixes for inputs that previously panicked or returned wrong results; they are
listed in the [changelog](CHANGELOG.md).

**Transitive dependencies:** crates in your dependency tree that depend on `chrono` will keep
pulling upstream `chrono`. Cargo's `[patch]` matches on package name, so it cannot substitute
`sad-chrono` for `chrono` deeper in the graph. The rename above covers your own code.

## Security and robustness fixes

Fixes in `sad-chrono` that are not in any released version of chrono:

| Input | chrono 0.4.45 | sad-chrono |
|-------|---------------|------------|
| TZif header with huge record counts ([chrono#1802][audit]) | `usize` overflow panic on 32-bit targets | Rejected as an invalid TZif file |
| TZif transition times near `i64::MIN` / `i64::MAX` | Overflow panic when mapping a local time (wraps in release builds) | Saturating arithmetic, no panic |
| Android / OpenHarmony tzdata index with a partial trailing entry | Out-of-bounds slice panic | Partial entry skipped |
| `DateTime<Utc>::from(SystemTime)` outside `DateTime`'s range ([chrono#1802][audit]) | Panic | Saturates to `DateTime::MIN_UTC` / `MAX_UTC` |
| `NaiveDate::from_isoywd_opt` with `i32::MIN` / `i32::MAX` years | Overflow panic | Returns `None`, as documented |
| Formatting offsets of `±23:59:30` or more to minute precision | Emits `+24:00`, which fails to parse back | Clamped to `+23:59` |

See the [changelog](CHANGELOG.md) for the full list, and [SECURITY.md](SECURITY.md) to report a
vulnerability.

[upstream]: https://github.com/chronotope/chrono
[soft-deprecation]: https://github.com/chronotope/chrono/issues/1768
[fork-point]: https://github.com/chronotope/chrono/commit/6adaa52
[audit]: https://github.com/chronotope/chrono/issues/1802

## About

Chrono aims to provide all functionality needed to do correct operations on dates and times in the
[proleptic Gregorian calendar](https://en.wikipedia.org/wiki/Proleptic_Gregorian_calendar):

* The [`DateTime`](https://docs.rs/sad-chrono/latest/sad_chrono/struct.DateTime.html) type is timezone-aware
  by default, with separate timezone-naive types.
* Operations that may produce an invalid or ambiguous date and time return `Option` or
  [`MappedLocalTime`](https://docs.rs/sad-chrono/latest/sad_chrono/offset/enum.MappedLocalTime.html).
* Configurable parsing and formatting with an `strftime` inspired date and time formatting syntax.
* The [`Local`](https://docs.rs/sad-chrono/latest/sad_chrono/offset/struct.Local.html) timezone works with
  the current timezone of the OS.
* Types and operations are implemented to be reasonably efficient.

Timezone data is not shipped with sad-chrono by default to limit binary sizes. Use the companion crate
[Chrono-TZ](https://crates.io/crates/chrono-tz) or [`tzfile`](https://crates.io/crates/tzfile) for
full timezone support.

## Documentation

See [docs.rs](https://docs.rs/sad-chrono/latest/sad_chrono/) for the API reference.

## Limitations

* Only the proleptic Gregorian calendar (i.e. extended to support older dates) is supported.
* Date types are limited to about +/- 262,000 years from the common epoch.
* Time types are limited to nanosecond accuracy.
* Leap seconds can be represented, but Chrono does not fully support them.
  See [Leap Second Handling](https://docs.rs/sad-chrono/latest/sad_chrono/naive/struct.NaiveTime.html#leap-second-handling).

## Crate features

Default features:

* `alloc`: Enable features that depend on allocation (primarily string formatting).
* `std`: Enables functionality that depends on the standard library. This is a superset of `alloc`
  and adds interoperation with standard library types and traits.
* `clock`: Enables reading the local timezone (`Local`). This is a superset of `now`.
* `now`: Enables reading the system time (`now`).
* `wasmbind`: Interface with the JS Date API for the `wasm32` target.

Optional features:

* `serde`: Enable serialization/deserialization via [serde].
* `rkyv`: Deprecated, use the `rkyv-*` features.
* `rkyv-16`: Enable serialization/deserialization via [rkyv], using 16-bit integers for integral `*size` types.
* `rkyv-32`: Enable serialization/deserialization via [rkyv], using 32-bit integers for integral `*size` types.
* `rkyv-64`: Enable serialization/deserialization via [rkyv], using 64-bit integers for integral `*size` types.
* `rkyv-validation`: Enable rkyv validation support using `bytecheck`.
* `arbitrary`: Construct arbitrary instances of a type with the Arbitrary crate.
* `unstable-locales`: Enable localization. This adds various methods with a `_localized` suffix.
  The implementation and API may change or even be removed in a patch release. Feedback welcome.
* `oldtime`: This feature no longer has any effect; it used to offer compatibility with the `time` 0.1 crate.

Note: The `rkyv{,-16,-32,-64}` features are mutually exclusive.

[serde]: https://github.com/serde-rs/serde
[rkyv]: https://github.com/rkyv/rkyv

## Rust version requirements

The Minimum Supported Rust Version (MSRV) is currently **Rust 1.62.0**.

The MSRV is explicitly tested in CI. It may be bumped in minor releases, but this is not done
lightly.

## License

This project is licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
* [MIT License](https://opensource.org/licenses/MIT)

at your option.

`sad-chrono` is derived from [chrono][upstream], Copyright (c) 2014--2026, Kang Seonghoon and
contributors. See [NOTICE](NOTICE) for attribution and a summary of modifications.
