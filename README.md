# sad-chrono

[![Build Status][gh-image]][gh-checks]
[![crates.io][cratesio-image]][cratesio]
[![Documentation][docsrs-image]][docsrs]
![Apache2/MIT licensed][license-image]
![MSRV][msrv-image]

[gh-image]: https://github.com/sadco-io/sad-chrono/actions/workflows/test.yml/badge.svg?branch=0.4.x
[gh-checks]: https://github.com/sadco-io/sad-chrono/actions/workflows/test.yml?query=branch%3A0.4.x
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

## Which version?

* **0.5.x (recommended):** MSRV 1.85, `rkyv` 0.8, and every fix as of `0.5.1` (see the
  [0.5 README](https://github.com/sadco-io/sad-chrono/blob/main/README.md)).
* **0.4.x (this line):** security fixes only — MSRV 1.62, `rkyv` 0.7, a strict drop-in for
  `chrono 0.4.x` builds that cannot move (also used by the transitive-dependency `[patch]` route
  below).

`sad-chrono 0.4.48` includes the non-blocking `TZ`/TZif and `tzdata` open (`O_NONBLOCK` on the
Unix targets listed under [Security and robustness fixes](#security-and-robustness-fixes)) and the
`TimeDelta::checked_mul` range fix from 0.5.1; the `tzdata` allocation bound from 0.5.0, with the
absolute 1 MiB caps and regular-file guard added in 0.4.47 (and ported to 0.5.1); value-invariant
`rkyv` 0.7 validation for `NaiveDate`, `NaiveTime`, `TimeDelta`, `FixedOffset`, and `IsoWeek`
(added in 0.4.47); and the saturating `From<DateTime<Tz>> for SystemTime` conversion and the
`format()` `# Panics` documentation from 0.5.0. Only the `rkyv` 0.8 upgrade
([RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235)) and validation of an
archived `DateTime<Tz>`'s local-time range (which would require trait bounds that change the `0.4`
API) are `0.5`-only.

`sad-chrono 0.5` is **chrono 0.4's API with current dependencies**. It is not upstream's unreleased
chrono 0.5 / `0.5.0-alpha` API.

## Usage

To switch your own code over without changing any `use chrono::...` paths, rename the dependency:

```toml
[dependencies]
chrono = { package = "sad-chrono", version = "0.4.48" }
```

> **Check your dependencies first.** `sad_chrono::DateTime` and `chrono::DateTime` are distinct
> types to the compiler. If a dependency takes or returns `chrono` types or implements its traits
> (for example `chrono-tz`, or the `chrono` features of `sqlx`, `diesel`, `postgres-types`, or
> `serde_with`), it will not accept your renamed `chrono::DateTime`, failing with errors such as
> `` expected `DateTime<Utc>`, found a different `DateTime<Utc>` `` (with the note
> `` `DateTime<Utc>` is defined in crate `sad_chrono` ``) or `` `Tz` implements similarly named
> trait `chrono::offset::TimeZone` ``. Run `cargo tree -i chrono` to find such dependencies; if you
> have any, replace `chrono` across the whole dependency graph instead (see
> [Transitive dependencies](#transitive-dependencies)).

Or depend on it under its own name and import from `sad_chrono`:

```toml
[dependencies]
sad-chrono = "0.4.48"
```

```rust
use sad_chrono::{DateTime, TimeZone, Utc};

let dt = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();
assert_eq!(dt.to_rfc3339(), "2026-11-17T12:00:00+00:00");
```

Feature flags are the same as chrono's (see [Crate features](#crate-features)).

## Compatibility with the `chrono` crate

`sad-chrono 0.4.46` was forked from chrono's `main` branch after `0.4.45` (commit
[`6adaa52`][fork-point]). The public API, feature flags, MSRV and serialization formats (serde,
rkyv) are unchanged from chrono `0.4.45`. The only behavior changes are fixes for inputs that
previously panicked, exhausted resources, or returned wrong or invalid results; they are listed in
the [changelog](https://github.com/sadco-io/sad-chrono/blob/0.4.x/CHANGELOG.md).

### Transitive dependencies

Crates in your dependency tree that depend on `chrono` keep pulling upstream `chrono`. Run
`cargo tree -i chrono` to see which ones. Cargo's `[patch]` matches on package name, so Cargo
ignores `[patch.crates-io] chrono = { package = "sad-chrono", ... }` and warns that the patch was
not used in the crate graph. Applications that want `sad-chrono` everywhere can patch with a
vendored copy whose package name is set to `chrono`:

```sh
git clone --depth 1 --branch v0.4.48 https://github.com/sadco-io/sad-chrono vendor/chrono
sed -i.bak 's/^name = "sad-chrono"/name = "chrono"/' vendor/chrono/Cargo.toml
rm -rf vendor/chrono/.git vendor/chrono/Cargo.toml.bak
```

```toml
[patch.crates-io]
chrono = { path = "vendor/chrono" }
```

Then run `cargo update -p chrono`, and verify that `cargo tree -i chrono` shows
`chrono v0.4.48 (vendor/chrono)`. Keep your own dependency as plain `chrono = "0.4"` without the
package rename so that all crates share one `DateTime` type.

Constraints:

* This works only when every `chrono` requirement in the dependency graph accepts `0.4.48` (a
  `^0.4` requirement); an exact pin such as `=0.4.40` blocks it.
* `[patch]` applies only in the top-level workspace, so a library cannot apply this for its users.

## For library authors

Libraries can switch with the same rename:

```toml
[dependencies]
chrono = { package = "sad-chrono", version = "0.4.48" }
```

Because the dependency key stays `chrono`, your `use chrono::...` paths,
`#[serde(with = "chrono::serde::ts_seconds")]` attributes, and feature names such as
`chrono = ["dep:chrono"]` or `serde = ["chrono?/serde"]` keep working. Enable every `sad-chrono`
feature you use explicitly: features of `chrono` and `sad-chrono` do not unify.

**Which line?** `sad-chrono = "0.5"` is recommended (see the
[0.5 guide for library authors](https://github.com/sadco-io/sad-chrono/blob/main/README.md#for-library-authors)),
but it requires Rust 1.85.0 on every target, and `rkyv` 0.8 for downstream users who enable
`rkyv*` features. Use `"0.4"` only when you must support older Rust versions or `rkyv` 0.7.
`sad-chrono 0.4` and `sad-chrono 0.5` types are distinct, so moving a public API from `0.4` to
`0.5` later is a breaking change.

**Is it a breaking change?** `sad_chrono::DateTime` and `chrono::DateTime` (as well as
`sad-chrono 0.4` and `sad-chrono 0.5` types) are distinct types. If
`chrono` types appear in your public API (function signatures, public fields, trait impls, or
`pub use chrono`), switching breaks callers that pass upstream `chrono` values. Release the switch
as a new semver-major version, or support both behind separate features (for example
`chrono = ["dep:chrono"]` and `sad-chrono = ["dep:sad-chrono"]`) with an impl for each. If you only
use `chrono` internally, switching is not a breaking change.

**Coexistence.** `chrono` and `sad-chrono` can both be in one dependency graph during a migration.
Convert values at the boundary through timestamps (for example
`sad_chrono::DateTime::from_timestamp(dt.timestamp(), dt.timestamp_subsec_nanos())`), `SystemTime`,
or RFC 3339 strings.

## Stability and versioning

`sad-chrono 0.4.x` keeps the public API of `chrono 0.4` and will not make semver-incompatible
changes on the `0.4` line. `0.4.x` is now security-fixes-only; new work lands in `0.5.x`. Version
numbers continue `chrono`'s `0.4` sequence independently of upstream: `sad-chrono 0.4.48` contains
`chrono 0.4.45` plus the fixes listed in
[CHANGELOG.md](https://github.com/sadco-io/sad-chrono/blob/0.4.x/CHANGELOG.md). Security fixes
from any new upstream `0.4.x` release are backported to the next `sad-chrono 0.4.x` release, and
each release's changelog states which `chrono` version and which fixes it includes (even if
`sad-chrono`'s patch number has already moved ahead of upstream's). Backports use
`git cherry-pick -x` and credit their authors.

MSRV is Rust 1.62.0. It is raised only in a release whose changelog says so, and never above the
MSRV of the latest upstream chrono 0.4 release.

## Security and robustness fixes

Fixes in `sad-chrono` that are not in any released version of chrono:

| Input | chrono 0.4.45 | sad-chrono |
|-------|---------------|------------|
| TZif header with huge record counts ([chrono#1802][audit]) | `usize` overflow panic on 32-bit targets | Rejected as an invalid TZif file |
| POSIX `TZ` with implicit DST offset reaching ±24h (e.g. `AAA-23BBB,J1,J365`) | `Local::now()` panics | Rejected as invalid `TZ`; `Local` falls back |
| TZif local time type with UTC offset outside ±23:59:59 | `Local::now()` panics | Rejected as invalid TZif; `Local` falls back |
| `TZ` pointing to a non-regular file or > 1 MiB file (e.g. `/dev/zero`, FIFO) (pre-check & 1 MiB cap 0.4.46; non-blocking open 0.4.48) | Unbounded read / memory exhaustion or hang | Checked with `is_file()` before and after opening, and capped at 1 MiB; since 0.4.48, opened non-blocking (`O_NONBLOCK`) on Linux/Android/OHOS (`x86`/`x86_64`/`arm`/`aarch64`/`riscv*`/`s390x`/`powerpc*`/`loongarch64`), Apple, FreeBSD/DragonFly/NetBSD/OpenBSD, and Solaris/illumos, so a FIFO swapped in after the pre-check cannot block the open there (it still can before 0.4.48 and on other Unix targets); `Local` falls back |
| TZif transition times near `i64::MIN` / `i64::MAX` | Overflow panic when mapping a local time (wraps in release builds) | Saturating arithmetic, no panic |
| Android / OpenHarmony tzdata index with a partial trailing entry | Out-of-bounds slice panic | Partial entry skipped |
| Android / OpenHarmony tzdata with oversized header or entry lengths (0.4.47) | Up to 4 GiB allocation before failing (panic or abort on 32-bit targets) | Rejected against file length and 1 MiB caps before allocating |
| Android / OpenHarmony tzdata path that is a FIFO or non-regular file (pre-check 0.4.47; non-blocking open 0.4.48) | Blocks in `File::open` | Checked with `is_file()` before and after opening; since 0.4.48, opened non-blocking (`O_NONBLOCK`) on every current Android and OpenHarmony target, so a FIFO swapped in after the pre-check cannot block the open (before 0.4.48 it still could) |
| `DateTime<Utc>::from(SystemTime)` outside `DateTime`'s range ([chrono#1802][audit]) | Panic | Saturates to `DateTime::MIN_UTC` / `MAX_UTC` |
| `From<DateTime<Tz>> for SystemTime` outside platform range (0.4.47) | Panic | Saturates to the nearest representable `SystemTime` |
| `NaiveDate::from_isoywd_opt` with `i32::MIN` / `i32::MAX` years | Overflow panic in builds with overflow checks (wraps in release builds) | Returns `None`, as documented |
| `TimeDelta::checked_mul` / `TimeDelta * i32` exceeding `[TimeDelta::MIN, TimeDelta::MAX]` (0.4.48) | Returns an out-of-range `TimeDelta` (later `num_milliseconds()` panics or wraps) | `checked_mul` returns `None`; `*` panics on overflow |
| Formatting offsets of `±23:59:30` or more to minute precision | Emits `+24:00`, which fails to parse back | Clamped to `+23:59` |
| Validated `rkyv` archives with out-of-range `NaiveDate`, `NaiveTime`, `TimeDelta`, `FixedOffset`, or `IsoWeek` values (0.4.47) | Accepted; later use could panic or return wrong values | Rejected by `CheckBytes` invariant checks (active with `rkyv-validation`; `DateTime<Tz>` local-time range check is `0.5`-only) |

See the [changelog](https://github.com/sadco-io/sad-chrono/blob/0.4.x/CHANGELOG.md) for the full list, and [SECURITY.md](https://github.com/sadco-io/sad-chrono/blob/0.4.x/SECURITY.md) to report a
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
  [`MappedLocalTime`](https://docs.rs/sad-chrono/latest/sad_chrono/offset/type.MappedLocalTime.html).
* Configurable parsing and formatting with an `strftime` inspired date and time formatting syntax.
* The [`Local`](https://docs.rs/sad-chrono/latest/sad_chrono/offset/struct.Local.html) timezone works with
  the current timezone of the OS.
* Types and operations are implemented to be reasonably efficient.

Timezone data is not shipped with sad-chrono by default to limit binary sizes. The companion crates
[Chrono-TZ](https://crates.io/crates/chrono-tz) and [`tzfile`](https://crates.io/crates/tzfile)
depend on upstream `chrono` and implement *its* `TimeZone` trait, so you cannot use their types with
`sad_chrono` types directly. To use them with `sad-chrono`, replace `chrono` across the entire
dependency graph (see [Transitive dependencies](#transitive-dependencies)).

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
* `wasmbind`: Interface with the JS Date API on wasm targets (`target_family = "wasm"`, excluding
  Emscripten and WASI).

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
* `defmt`: Implement `defmt::Format` for sad-chrono types, for embedded logging.
* `core-error`: Implement `core::error::Error` for the error types without `std` (requires Rust 1.81).
* `oldtime`: This feature no longer has any effect; it used to offer compatibility with the `time` 0.1 crate.

Note: The `rkyv{,-16,-32,-64}` features are mutually exclusive.

The `rkyv*` features use rkyv 0.7, which is no longer maintained upstream and is affected by
[RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235) (out-of-bounds reads when
validating untrusted archives that contain `Rc`, `Arc` or weak pointers to unsized data).
`sad-chrono`'s archived types contain no shared pointers, but if you validate untrusted rkyv
archives of your own types, review the advisory. The `0.4.x` line stays on rkyv 0.7 for
compatibility, while **0.5.x** uses rkyv 0.8.18, which fixes the advisory. As of `0.4.47`,
`rkyv-validation` checks value invariants of `sad-chrono`'s archived types (`NaiveDate`,
`NaiveTime`, `TimeDelta`, `FixedOffset`, and `IsoWeek`), except the local-time range of
`DateTime<Tz>`: before calling local-time methods such as `naive_local()` on a deserialized
`DateTime<Tz>`, check `dt.naive_utc().checked_add_offset(dt.offset().fix()).is_some()` (with
`sad_chrono::Offset` in scope).

[serde]: https://github.com/serde-rs/serde
[rkyv]: https://github.com/rkyv/rkyv

## Rust version requirements

MSRV is Rust 1.62.0. It is raised only in a release whose changelog says so, and never above the
MSRV of the latest upstream chrono 0.4 release.

## License

This project is licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
* [MIT License](https://opensource.org/licenses/MIT)

at your option.

`sad-chrono` is derived from [chrono][upstream], Copyright (c) 2014--2026, Kang Seonghoon and
contributors. See [NOTICE](https://github.com/sadco-io/sad-chrono/blob/0.4.x/NOTICE) for attribution and a summary of modifications.
