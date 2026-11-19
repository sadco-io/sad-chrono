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
[msrv-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg

Timezone-aware date and time handling for Rust: a **maintained, API-compatible fork of
[chrono][upstream]** with security and robustness fixes.

Upstream chrono has been soft-deprecated by its maintainers ([chrono#1768][soft-deprecation]), and
fixes that landed after `0.4.45` were never released. `sad-chrono` keeps the chrono 0.4 API stable,
ships those fixes, and takes on new ones. It is maintained by the Sadco Security Team alongside
[sad-rsa](https://github.com/sadco-io/sad-rsa).

## Which version?

* **0.5.x** (recommended): MSRV 1.85, `rkyv` 0.8, and every fix.
* **0.4.x**: maintained on the `0.4.x` branch for security fixes only — MSRV 1.62, `rkyv` 0.7, a
  strict drop-in for `chrono 0.4.x` builds that cannot move (also used by the transitive-dependency
  `[patch]` route below).

`sad-chrono 0.4.47` (the security-fixes-only maintenance release) includes the `tzdata` allocation
bound (with absolute 1 MiB caps and a regular-file guard), value-invariant `rkyv` 0.7 validation for
`NaiveDate`, `NaiveTime`, `TimeDelta`, `FixedOffset`, and `IsoWeek`, the saturating
`From<DateTime<Tz>> for SystemTime` conversion, and the `format()` `# Panics` documentation. The
`rkyv` 0.8 upgrade ([RUSTSEC-2026-0235][rustsec-rkyv]) and validation of an archived
`DateTime<Tz>`'s local-time range are `0.5`-only, and the `0.5.1` `TimeDelta::checked_mul` range
fix and non-blocking `TZ`/`tzdata` open are not in `0.4.47` yet.

`sad-chrono 0.5` is **chrono 0.4's API with current dependencies**. It is not upstream's
unreleased chrono 0.5 / `0.5.0-alpha` API.

## Usage

To switch your own code over without changing any `use chrono::...` paths, rename the dependency:

```toml
[dependencies]
chrono = { package = "sad-chrono", version = "0.5" }
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
sad-chrono = "0.5"
```

```rust
use sad_chrono::{DateTime, TimeZone, Utc};

let dt = Utc.with_ymd_and_hms(2026, 11, 17, 12, 0, 0).unwrap();
assert_eq!(dt.to_rfc3339(), "2026-11-17T12:00:00+00:00");
```

Feature flags are the same as chrono's (see [Crate features](#crate-features)).

## Compatibility with the `chrono` crate

`sad-chrono` was forked from chrono's `main` branch after `0.4.45` (commit
[`6adaa52`][fork-point]). `sad-chrono 0.5` keeps chrono `0.4.45`'s public API and feature names,
with these differences from chrono `0.4.45` / `sad-chrono 0.4.46`:

* **MSRV 1.85** (`1.85.0`).
* **`rkyv*` features use `rkyv` 0.8.** Trait impls and the archive binary format changed, so stored
  archived data must be re-archived. Feature names stay the same, and `rkyv-16` / `rkyv-32` /
  `rkyv-64` map to `rkyv`'s `pointer_width_*` features and enable `rkyv/bytecheck`. Semantic `Verify`
  checks (rejecting out-of-range dates, times, offsets, durations, ISO weeks, and `DateTime`
  local-time ranges) are always active whenever `rkyv` `CheckBytes` is available. `rkyv-validation`
  remains as a compatibility feature that also enables `rkyv/alloc` (needed for the high-level
  `rkyv::to_bytes`, `rkyv::from_bytes`, and `rkyv::access`; no-alloc users call
  `rkyv::api::low::{access, from_bytes}`, which run the same semantic checks).
* **`From<DateTime<Tz>> for SystemTime` saturates** to the platform's nearest representable
  `SystemTime` instead of panicking on out-of-range dates.
* **Malformed Android / OpenHarmony `tzdata` is rejected before allocating** when header or index
  lengths exceed the file size or 1 MiB caps, and TZif (`TZ`) / `tzdata` paths must be regular
  files (pre-checked with `is_file()`, and opened with `O_NONBLOCK` then checked on the open handle
  on Linux/Android/OpenHarmony on `x86`/`x86_64`/`arm`/`aarch64`/`riscv32`/`riscv64`/`s390x`/`powerpc`/`powerpc64`/`loongarch64`,
  Apple/FreeBSD/DragonFly/NetBSD/OpenBSD, and Solaris/illumos; other Unix targets keep the
  check-then-open behavior).
* **`TimeDelta::checked_mul` returns `None` instead of an out-of-range `TimeDelta`** when the
  product exceeds `[TimeDelta::MIN, TimeDelta::MAX]`.
* **Everything else, including serde formats, is unchanged.** Golden tests in
  `tests/format_golden.rs` pin the serde wire bytes to `sad-chrono 0.4.46` (byte-identical to
  chrono `0.4.45`).

### Migrating from 0.4.46

These steps also apply from `0.4.47`, except step 4, because `0.4.47` already saturates
`SystemTime`.

1. Bump your dependency to `version = "0.5"` (or `sad-chrono = "0.5"`). If your crate exposes
   `sad-chrono` types in its public API, moving from `sad-chrono 0.4` to `0.5` is a breaking change
   (the `0.4` and `0.5` types are distinct, and `0.5` raises MSRV to 1.85 and `rkyv` to 0.8).
2. Verify your toolchain meets the Rust `1.85.0` MSRV.
3. If you enable any `rkyv*` feature:
   * Bump your own `rkyv` dependency to `0.8`.
   * Keep or enable `rkyv-validation` if your code calls the high-level `rkyv::to_bytes`,
     `rkyv::from_bytes`, or `rkyv::access` (which require `rkyv/alloc`); `rkyv-16` / `rkyv-32` /
     `rkyv-64` already enable `rkyv/bytecheck` and all semantic validation checks for
     `rkyv::api::low::{access, from_bytes}` without `alloc`.
   * Update callers to the `rkyv` 0.8 API (`rkyv::to_bytes::<rkyv::rancor::Error>(&v)`,
     `rkyv::from_bytes::<T, rkyv::rancor::Error>(&b)`, and `rkyv::check_archived_root` ->
     `rkyv::access::<rkyv::Archived<T>, rkyv::rancor::Error>`), and re-archive stored data.
4. Review callers of `From<DateTime<Tz>> for SystemTime` if any relied on out-of-range dates
   panicking instead of saturating to the platform's nearest representable `SystemTime`.
5. Since `0.5.1`, `TimeDelta * i32` panics and `TimeDelta::checked_mul` returns `None` instead of
   an out-of-range `TimeDelta`.

### Transitive dependencies

This route uses the `0.4.x` line (MSRV 1.62, `rkyv` 0.7, without the `0.5`-only fixes or the
`0.5.1` fixes), because
other crates in your dependency graph require `chrono = "^0.4"`. Cargo's `[patch]` matches both
package name and compatible version requirement, so Cargo ignores both `[patch.crates-io] chrono =
{ package = "sad-chrono", ... }` and a renamed `0.5.x` copy (`patch chrono v0.5.1 was not used
in the crate graph`). Run `cargo tree -i chrono` to see which crates pull upstream `chrono`.
Applications that want `sad-chrono` everywhere can patch with a vendored `0.4.x` release tag
whose package name is set to `chrono` (use the newest `v0.4.*` release tag, `v0.4.47` at the time
of writing):

```sh
git clone --depth 1 --branch v0.4.47 https://github.com/sadco-io/sad-chrono vendor/chrono
sed -i.bak 's/^name = "sad-chrono"/name = "chrono"/' vendor/chrono/Cargo.toml
rm -rf vendor/chrono/.git vendor/chrono/Cargo.toml.bak
```

```toml
[patch.crates-io]
chrono = { path = "vendor/chrono" }
```

Then run `cargo update -p chrono`, and verify that `cargo tree -i chrono` shows
`chrono v0.4.47 (vendor/chrono)` (or the `v0.4.*` release tag you cloned). Keep your own dependency
as plain `chrono = "0.4"` without the package rename so that all crates share one `DateTime` type.

Constraints:

* This works only when every `chrono` requirement in the dependency graph is `^0.4`-compatible; an
  exact pin such as `=0.4.40` blocks it.
* `[patch]` applies only in the top-level workspace, so a library cannot apply this for its users.

## For library authors

Libraries can switch with the same rename:

```toml
[dependencies]
chrono = { package = "sad-chrono", version = "0.5" }
```

Because the dependency key stays `chrono`, your `use chrono::...` paths,
`#[serde(with = "chrono::serde::ts_seconds")]` attributes, and feature names such as
`chrono = ["dep:chrono"]` or `serde = ["chrono?/serde"]` keep working. Enable every `sad-chrono`
feature you use explicitly: features of `chrono` and `sad-chrono` do not unify.

**Which line?** Depending on `sad-chrono = "0.5"` requires Rust `1.85.0` on every target (Cargo's
`rust-version` is crate-wide, even though it was raised for `windows-link` `0.100`, a Windows-only
dependency, with no older compatible `0.5` release to fall back to). Only downstream users who
enable `rkyv*` features have to move to `rkyv` `0.8`. Libraries that must support older Rust
versions or `rkyv` `0.7` can target `sad-chrono = "0.4"`, noting that `sad-chrono 0.4` and
`sad-chrono 0.5` are distinct crates whose types do not interoperate — moving a library's public
API from `0.4` to `0.5` is a breaking change.

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

`sad-chrono 0.5.x` keeps the public API of `chrono 0.4` and will not make semver-incompatible
changes within `0.5.x`. `0.5.0` deliberately leaves `chrono`'s `0.4` version sequence because the
dependency upgrades (`rkyv` 0.8, `windows-link` 0.100, MSRV 1.85) are breaking changes.

The `0.4.x` branch remains maintained for security fixes only (MSRV 1.62, `rkyv` 0.7). Each
release's changelog states which `chrono` version and which fixes it includes. Upstream fixes are
backported with `git cherry-pick -x` and credit their authors.

MSRV is Rust 1.85.0 for `0.5.x`. Within `0.5.x`, it is raised only with a changelog entry and only if
a dependency's security fix requires it.

## Security and robustness fixes

Fixes in `sad-chrono` that are not in any released version of chrono:

| Input | chrono 0.4.45 | sad-chrono |
|-------|---------------|------------|
| TZif header with huge record counts ([chrono#1802][audit]) | `usize` overflow panic on 32-bit targets | Rejected as an invalid TZif file |
| POSIX `TZ` with implicit DST offset reaching ±24h (e.g. `AAA-23BBB,J1,J365`) | `Local::now()` panics | Rejected as invalid `TZ`; `Local` falls back |
| TZif local time type with UTC offset outside ±23:59:59 | `Local::now()` panics | Rejected as invalid TZif; `Local` falls back |
| `TZ` pointing to a non-regular file or > 1 MiB file (e.g. `/dev/zero`, FIFO) (pre-check & 1 MiB cap 0.4.46; non-blocking open 0.5.1) | Unbounded read / memory exhaustion or hang | Pre-checked with `is_file()` and capped at 1 MiB; on Linux/Android/OHOS (`x86`/`x86_64`/`arm`/`aarch64`/`riscv*`/`s390x`/`powerpc*`/`loongarch64`), Apple/BSD, and Solaris/illumos, opened non-blocking (`O_NONBLOCK`) and checked on the opened handle (before 0.5.1, a concurrent swap could still block); `Local` falls back |
| TZif transition times near `i64::MIN` / `i64::MAX` | Overflow panic when mapping a local time (wraps in release builds) | Saturating arithmetic, no panic |
| Android / OpenHarmony tzdata index with a partial trailing entry | Out-of-bounds slice panic | Partial entry skipped |
| Android / OpenHarmony tzdata with oversized header or entry lengths (file-length bound 0.5.0; 1 MiB caps 0.5.1, 0.4.47) | Up to 4 GiB allocation before failing (panic or abort on 32-bit targets) | Rejected against file length and 1 MiB caps before allocating |
| Android / OpenHarmony tzdata path that is a FIFO or non-regular file (pre-check 0.5.1, 0.4.47; non-blocking open 0.5.1) | Blocks in `File::open` | Pre-checked with `is_file()`; on listed platforms (including Linux/Android/OHOS), opened non-blocking (`O_NONBLOCK`) and checked on the opened handle (before 0.5.1, a concurrent swap could still block) |
| `DateTime<Utc>::from(SystemTime)` outside `DateTime`'s range ([chrono#1802][audit]) | Panic | Saturates to `DateTime::MIN_UTC` / `MAX_UTC` |
| `From<DateTime<Tz>> for SystemTime` outside platform range (0.5.0, 0.4.47) | Panic | Saturates to the nearest representable `SystemTime` |
| `NaiveDate::from_isoywd_opt` with `i32::MIN` / `i32::MAX` years | Overflow panic in builds with overflow checks (wraps in release builds) | Returns `None`, as documented |
| `TimeDelta::checked_mul` / `TimeDelta * i32` exceeding `[TimeDelta::MIN, TimeDelta::MAX]` (0.5.1) | Returns an out-of-range `TimeDelta` (later `num_milliseconds()` panics or wraps) | `checked_mul` returns `None`; `*` panics on overflow |
| Formatting offsets of `±23:59:30` or more to minute precision | Emits `+24:00`, which fails to parse back | Clamped to `+23:59` |
| `rkyv` 0.7 out-of-bounds read in checked validation of `Rc`/`Arc`/`Weak` pointers to unsized data ([RUSTSEC-2026-0235][rustsec-rkyv], 0.5.0) | Uses vulnerable unmaintained `rkyv` 0.7 (`sad-chrono`'s own types contain no shared pointers); `0.4.x` still carries `rkyv` 0.7 | Fixed by upgrading to `rkyv` 0.8.18 (patched in `>=0.8.17`) |
| Validated `rkyv` archives with out-of-range dates, times, offsets, durations, ISO weeks, or `DateTime` local-time ranges (0.5.0; non-`DateTime` types in 0.4.47) | Accepted; later use could panic or return wrong values | 0.5.x: `bytecheck` `Verify`, active with any `rkyv*` feature; 0.4.47: rkyv 0.7 `CheckBytes` invariant checks, active with `rkyv-validation` |

See the [changelog](CHANGELOG.md) for the full list, and [SECURITY.md](SECURITY.md) to report a
vulnerability.

[upstream]: https://github.com/chronotope/chrono
[soft-deprecation]: https://github.com/chronotope/chrono/issues/1768
[fork-point]: https://github.com/chronotope/chrono/commit/6adaa52
[audit]: https://github.com/chronotope/chrono/issues/1802
[rustsec-rkyv]: https://rustsec.org/advisories/RUSTSEC-2026-0235

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
* `rkyv`: Deprecated, use the `rkyv-*` features. Enables [rkyv] 0.8 (`pointer_width_32`) and `bytecheck` semantic validation.
* `rkyv-16`: Enable serialization/deserialization via [rkyv] 0.8 (`pointer_width_16`), using 16-bit integers for integral `*size` types, with `bytecheck` semantic validation.
* `rkyv-32`: Enable serialization/deserialization via [rkyv] 0.8 (`pointer_width_32`), using 32-bit integers for integral `*size` types, with `bytecheck` semantic validation.
* `rkyv-64`: Enable serialization/deserialization via [rkyv] 0.8 (`pointer_width_64`), using 64-bit integers for integral `*size` types, with `bytecheck` semantic validation.
* `rkyv-validation`: Compatibility feature that enables `rkyv/alloc` (needed for the high-level `rkyv::to_bytes`, `rkyv::from_bytes`, and `rkyv::access`; no-alloc users call `rkyv::api::low::{access, from_bytes}`, which run the same semantic checks). Semantic `bytecheck` invariant checks (rejecting out-of-range archived dates, times, offsets, durations, ISO weeks, and `DateTime` local-time ranges) are always active whenever `rkyv` `CheckBytes` is available via any `rkyv*` feature.
* `arbitrary`: Construct arbitrary instances of a type with the Arbitrary crate.
* `unstable-locales`: Enable localization. This adds various methods with a `_localized` suffix.
  The implementation and API may change or even be removed in a patch release. Feedback welcome.
* `defmt`: Implement `defmt::Format` for sad-chrono types, for embedded logging.
* `core-error`: Implement `core::error::Error` for the error types without `std`.
* `oldtime`: This feature no longer has any effect; it used to offer compatibility with the `time` 0.1 crate.

Note: The `rkyv{,-16,-32,-64}` features are mutually exclusive.

[serde]: https://github.com/serde-rs/serde
[rkyv]: https://github.com/rkyv/rkyv

## Rust version requirements

MSRV is Rust 1.85.0 for `0.5.x`. Within `0.5.x`, it is raised only with a changelog entry and only if
a dependency's security fix requires it.

## License

This project is licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
* [MIT License](https://opensource.org/licenses/MIT)

at your option.

`sad-chrono` is derived from [chrono][upstream], Copyright (c) 2014--2026, Kang Seonghoon and
contributors. See [NOTICE](NOTICE) for attribution and a summary of modifications.
