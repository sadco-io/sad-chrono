ChangeLog for sad-chrono
========================

`sad-chrono` is a maintained fork of [chrono](https://github.com/chronotope/chrono).
Its releases are documented below. Upstream chrono releases from 0.4.20 to
0.4.45 are documented in the
[chrono release notes](https://github.com/chronotope/chrono/releases), and
earlier ones further down this file.

Entries reference the upstream chrono pull request or issue each change came
from, as `chrono#NNNN`.

## sad-chrono 0.5.2

Patch release on the 0.5 line. Public API, MSRV (Rust 1.85.0), and serde/rkyv
wire formats are unchanged from `0.5.1`. It fixes wrong `Local` results at
daylight-saving transitions on Unix and three parsing and caching bugs. Two of
the fixes change results that callers can observe; they are marked below.

### Fixes

* Unix: resolve local times at daylight-saving transitions from the periods in
  which each offset applies (chrono#1824). `Local.from_local_datetime` and the
  other local-to-UTC lookups previously treated the instant a transition ends
  as part of it, for both TZif transitions and POSIX `TZ` rules:
  * A local time at the start of a gap (Europe/Berlin `2026-03-29 02:00`) now
    maps to `None` instead of an instant that has a different wall time.
  * A local time at the end of a fold (Europe/Berlin `2026-10-25 03:00`) now
    maps to a single instant instead of `Ambiguous`.
  * **Behavior change:** inside a fold, `MappedLocalTime::Ambiguous` now lists
    the earlier instant first, as documented. Before, the candidates were
    ordered by offset, so `earliest()` returned the later instant and
    `latest()` the earlier one for a normal daylight-saving fold.
  * A POSIX rule whose daylight-saving period is empty no longer reports its
    local times as nonexistent.

  Windows uses a separate lookup and is unchanged. By Sebastian Sala (#18).
* `NaiveDateTime::from_str` accepts a space in place of the `T` between the
  date and the time, so the `Display` output (`2016-07-08 00:00:00`) parses
  back to the same value (chrono#1792). **Behavior change:** such strings
  previously failed with `ParseError(Invalid)`. Whitespace after that space is
  skipped, as it is before every numeric field. Inputs with a `T` parse as
  before, and serde deserialization still requires the `T`. By Sebastian Sala
  (#17).
* Unix: reload `Local` when the TZif file named by an absolute path in `TZ`
  (with or without a leading `:`) is replaced or modified, including through a
  symlink. The cache compared only the `TZ` string, so the first offset read
  stayed in use for the rest of the thread. The file's device, inode, and
  change time are now compared at the existing one-second revalidation; the
  file is not opened for this check. By Sebastian Sala (#16).
* Parse the offsets that `%::z` and `%:::z` format. `%::z` now reads an
  optional `:00` through `:59` after the minutes as offset seconds, so
  `+01:02:03` parses as that offset instead of failing with `TooLong`.
  `%:::z` now accepts an hours-only offset such as `-05` when no digit follows
  the hours and their separator. Other inputs that `%::z` and `%:::z` accepted
  before parse as before, except that a `:SS` after `%::z` now belongs to the
  offset even when a later item such as `:%S` used to read it. `%z`, `%:z`,
  `%#z`, and RFC 3339 parsing are unchanged. By Sebastian Sala (#15).

### CI

* Add `x86_64-unknown-netbsd`, `x86_64-pc-solaris`, and `s390x-unknown-linux-gnu`
  to `alternative_targets` and compile `--lib --tests` with `RUSTFLAGS="-D
  warnings"` on those targets (alongside `aarch64-linux-android`,
  `aarch64-unknown-linux-ohos`, `x86_64-unknown-illumos`, and
  `x86_64-unknown-freebsd`) to verify each per-target `O_NONBLOCK` constant's
  compile-time assertion.

## sad-chrono 0.4.48

Released from the `0.4.x` maintenance branch.

Security release on the 0.4 maintenance line. It is API-compatible with
0.4.47, and the MSRV (1.62.0) and rkyv 0.7 are unchanged. When a product falls
outside the `TimeDelta` range, `TimeDelta::checked_mul` now returns `None` and
`TimeDelta * i32` now panics. On the Unix targets listed below, opening TZif
and `tzdata` files also no longer waits for a FIFO writer.
[`sad-chrono 0.5.1`](https://github.com/sadco-io/sad-chrono/releases/tag/v0.5.1)
(MSRV 1.85, rkyv 0.8) is the recommended line; `0.4.x` receives security
fixes only.

### Security and robustness

* Open TZif (`TZ`, `/etc/localtime`, and system fallback paths) and
  Android/OpenHarmony `tzdata` files with `O_NONBLOCK` on the Unix targets
  listed below so that opening a FIFO for reading does not wait for a writer,
  and reject non-regular files through the existing `is_file()` check on the
  opened `File` handle (present since `0.4.46` for `TZ` and `0.4.47` for
  `tzdata`; the pre-open `is_file()` check is kept as a fast path). On those
  targets, this closes a race in which a FIFO swapped in after the `is_file()`
  pre-check could hang `Local::now()` or the `tzdata` loader indefinitely.
  `O_NONBLOCK` is enabled on Linux (including OpenHarmony) and Android on `x86`,
  `x86_64`, `arm`, `aarch64`, `riscv32`, `riscv64`, `s390x`, `powerpc`,
  `powerpc64`, and `loongarch64` (every current Android and OpenHarmony target);
  on Apple platforms, FreeBSD, DragonFly, NetBSD, and OpenBSD; and on Solaris
  and illumos. Other Unix targets open without `O_NONBLOCK` but still verify
  `is_file()` both before and after opening, so the race is not closed there.
  Backported from `0.5.1`.
* Enforce `[TimeDelta::MIN, TimeDelta::MAX]` bounds in `TimeDelta::checked_mul`
  via `TimeDelta::new`, returning `None` instead of an out-of-range `TimeDelta`
  (on which `num_milliseconds()` could panic with overflow checks enabled or
  wrap with them disabled). `TimeDelta * i32` now panics with its overflow
  message instead of returning an out-of-range value. Document `# Panics` on
  `TimeDelta` arithmetic operators (`Add`, `Sub`, `AddAssign`, `SubAssign`,
  `Mul<i32>`, `Div<i32>`, and `Sum`) (backported from `0.5.1`).

### CI

* Add `x86_64-unknown-illumos`, `x86_64-unknown-freebsd`,
  `x86_64-unknown-netbsd`, `x86_64-pc-solaris`, and `s390x-unknown-linux-gnu`
  to `alternative_targets` and compile `--lib --tests` with `RUSTFLAGS="-D
  warnings"` across those targets plus `aarch64-linux-android` and
  `aarch64-unknown-linux-ohos` to verify the hardcoded `O_NONBLOCK` constants.

## sad-chrono 0.5.1

Patch release on the 0.5 line. Public API, MSRV (Rust 1.85.0), and serde/rkyv
wire formats are unchanged from `0.5.0`.

### Security and robustness

* Android and OpenHarmony `tzdata`: cap index-region and per-entry lengths at
  1 MiB before allocating, and require the `tzdata` path to be a regular file
  (ported from `0.4.47`; strengthened below with a non-blocking open so that a
  FIFO swapped in concurrently cannot block the open on the platforms listed
  below). In `0.5.0`, lengths were bounded only against the file's reported
  length, so a sparse file could still force GiB-sized allocations before
  failing.
* Open TZif (`TZ`) and Android/OpenHarmony `tzdata` files with `O_NONBLOCK` on
  the Unix targets listed below, and verify `is_file()` on the opened `File`
  handle (keeping the pre-open `is_file()` check as a fast path). On those
  targets, this closes a race in which a FIFO swapped in after the `is_file()`
  pre-check (present for `TZ` since `0.4.46` / `0.5.0`, and for `tzdata` since
  `0.4.47`) could hang `Local::now()` or the `tzdata` loader indefinitely.
  `O_NONBLOCK` is enabled on Linux (including OpenHarmony) and Android on
  `x86`, `x86_64`, `arm`, `aarch64`, `riscv32`, `riscv64`, `s390x`, `powerpc`,
  `powerpc64`, and `loongarch64`; on Apple platforms, FreeBSD, DragonFly,
  NetBSD, and OpenBSD; and on Solaris and illumos. Other Unix targets open
  without `O_NONBLOCK` but still verify `is_file()` both before and after
  opening, so the race is not closed there. Not in `0.4.47`.
* Enforce `[TimeDelta::MIN, TimeDelta::MAX]` bounds in `TimeDelta::checked_mul`
  via `TimeDelta::new`, returning `None` instead of an out-of-range `TimeDelta`
  (on which `num_milliseconds()` could panic with overflow checks enabled or
  wrap with them disabled). `TimeDelta * i32` now panics with its overflow
  message instead of returning an out-of-range value. Document `# Panics` on
  `TimeDelta` arithmetic operators (`Add`, `Sub`, `AddAssign`, `SubAssign`,
  `Mul<i32>`, `Div<i32>`, and `Sum`). Not in `0.4.47`.

### Fixes

* Android and OpenHarmony `tzdata`: fix on-device test `for_zone` argument
  types so the on-device `tzdata` tests compile (ported from `0.4.47`).

### CI

* Pin `wasm-pack` to `v0.15.1` in `test_wasm` (`0.5.0` intended all CI tools to
  be version-pinned, but missed `wasm-pack-action` floating to `latest`), and
  update `taiki-e/install-action` SHA comments from `# v2` to `# v2.90.5`.
* Add `aarch64-unknown-linux-ohos`, `x86_64-unknown-illumos`, and
  `x86_64-unknown-freebsd` to `alternative_targets`, and compile `--lib --tests`
  with `-D warnings` on those three targets and `aarch64-linux-android`, so the
  on-device `tzdata` tests and the per-platform `O_NONBLOCK` constants are
  checked in CI.
* Add a `0.4.x` `github-actions` entry to `.github/dependabot.yml` so
  Dependabot updates action SHA pins on the `0.4.x` maintenance branch as well.

## sad-chrono 0.4.47

Released from the `0.4.x` maintenance branch.

Security release on the 0.4 maintenance line. It is API-compatible with
0.4.46, and the MSRV (1.62.0) and rkyv 0.7 are unchanged.
[`sad-chrono 0.5.0`](https://github.com/sadco-io/sad-chrono/blob/main/CHANGELOG.md#sad-chrono-050)
(MSRV 1.85, rkyv 0.8) is the recommended line; `0.4.x` receives security
fixes only.

### Security and robustness

* Android and OpenHarmony `tzdata`: header index-region and per-entry lengths
  are checked against the file length (`SeekFrom::End(0)`) and capped at 1 MiB
  before allocating, the reader seeks to `index_offset` before reading the
  index, and the `tzdata` path must be a regular file (`is_file()` is checked
  before opening and again on the opened handle; a concurrent path replacement
  can still block the open in `0.4.47`). Previously a malformed bundle could
  make the reader allocate up to 4 GiB before failing on 64-bit targets, and
  panic (`capacity overflow`) or abort on 32-bit. An index region or selected
  entry extending past the end of the file returns `UnexpectedEof`, a region
  or selected entry exceeding its 1 MiB cap returns `InvalidData` when it fits
  within the file, and non-regular paths return `InvalidInput` (file-length
  bounds backported from `0.5.0`; 1 MiB caps and regular-file guard new in
  `0.4.47`).
* With `rkyv-validation`, `CheckBytes` validation now enforces value
  invariants (not just structural layout) for `NaiveDate`, `NaiveTime`,
  `TimeDelta`, `FixedOffset`, and `IsoWeek`, so untrusted archives containing
  out-of-range or inconsistent values are rejected during validation instead
  of panicking or returning wrong values on subsequent use.
* `From<DateTime<Tz>> for SystemTime` now saturates at the limit of checked
  arithmetic from `UNIX_EPOCH` in that direction using `checked_add` /
  `checked_sub` and binary search, and normalizes leap seconds before
  inverting pre-epoch offsets. Previously the conversion could panic outside
  the Windows arithmetic range (on Rust 1.94 and later, January 1601 to
  September 30828), and before 1970 on `wasm32-unknown-unknown` (backported
  from `0.5.0`).
* Document `# Panics` sections on `DateTime::format`, `NaiveDateTime::format`,
  `NaiveDate::format`, and `NaiveTime::format` covering all failure classes
  (invalid specifiers, parse-only specifiers such as `%#z`, and specifiers for
  fields the type lacks), and recommend `write!` / `core::fmt::Write` with
  error handling for untrusted format strings (backported from `0.5.0`).

### Fixes

* Android and OpenHarmony `tzdata`: restore compilation on Rust 1.62–1.79
  (import `std::mem::size_of` and use named generic parameters on
  `find_tz_data` / `TzDataIndexes::new`) and fix on-device test argument types,
  so the documented 1.62.0 MSRV holds for those targets and the on-device tests
  compile. On Rust older than 1.63, Android builds also need `cargo update -p
  libc --precise 0.2.163`, because Cargo before 1.84 doesn't select
  MSRV-compatible dependency versions.

### Known issues (0.4.x line)

* The `rkyv*` features still use rkyv 0.7 and are affected by
  [RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235), which
  is unchanged from 0.4.46. Fixed in 0.5.0 by rkyv 0.8.18.
* With `rkyv-validation`, the archived `DateTime<Tz>` local-time range is not
  validated (enforcing it generically would require extra `Tz::Offset` bounds
  that change the `0.4` API). A validated archive can hold a UTC date-time
  plus offset whose local time is out of range, so methods such as
  `naive_local()` can panic. Mitigation: check
  `dt.naive_utc().checked_add_offset(dt.offset().fix()).is_some()` (with
  `Offset` in scope) before use. `0.5.x` validates this.

### CI

* Workflows now run on pushes to `0.4.x`. All actions are pinned to commit
  SHAs, tool versions (including `wasm-pack`) are pinned, checkouts set
  `persist-credentials: false`, baselines are fetched by commit, `rust_msrv`
  adds an Android MSRV check, `alternative_targets` adds OpenHarmony and
  checks Android tests, `semver` adds a frozen-API reverse check against
  `0.4.46` and a `Cargo.toml` MSRV guard, and the stale `RUSTSEC-2026-0173`
  ignore is removed from `deny.toml`.

## sad-chrono 0.5.0

`sad-chrono 0.5.0` preserves chrono 0.4's public API, feature names, and serde
wire formats while moving to current dependencies (`rkyv` 0.8.18,
`windows-link` 0.100, `defmt` 1.2) and raising the MSRV to Rust 1.85.0. Its
upstream chrono base is unchanged from `0.4.46` (`chrono`
[`6adaa52`](https://github.com/chronotope/chrono/commit/6adaa52)); the only new
upstream code is the `chrono#1779` `rkyv` 0.8 port. It also enforces semantic
`bytecheck` validation invariants whenever any `rkyv*` feature is enabled,
bounds Android/OpenHarmony `tzdata` allocations by file length, and makes
`From<DateTime<Tz>> for SystemTime` saturate instead of panicking.

### Breaking changes

* MSRV raised from Rust 1.62.0 to Rust 1.85.0 (required by `windows-link`
  `0.100`; `rkyv` `0.8.18` and `defmt` `1.2` require Rust `1.81.0`).
* `rkyv*` features upgraded from `rkyv` 0.7 to `rkyv` 0.8 (`0.8.18`), which
  changes trait implementations and the archive binary format; stored archives
  must be re-archived. Feature names are unchanged, with `rkyv-16`, `rkyv-32`,
  and `rkyv-64` mapping to `rkyv`'s `pointer_width_*` features (and now enabling
  `rkyv/bytecheck` directly; `rkyv-validation` remains as a compatibility
  feature adding `rkyv/alloc`).
* `From<DateTime<Tz>> for SystemTime` now saturates at the limit of checked
  arithmetic from `UNIX_EPOCH` in that direction instead of panicking on
  out-of-range dates (behavior change).

### Security and robustness

* Upgrade `rkyv` to `0.8.18`, resolving
  [RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235).
* Enforce fail-closed semantic invariant checks (`bytecheck::Verify`) whenever
  any `rkyv*` feature (`rkyv`, `rkyv-16`, `rkyv-32`, `rkyv-64`) enables
  `rkyv/bytecheck`, covering `DateTime<Tz>` (verifying local time is in range),
  `NaiveDate`, `NaiveTime`, `FixedOffset`, `TimeDelta`, and `IsoWeek`
  (alongside discriminant checks for `Month` and `Weekday`), so `rkyv::access`
  and `rkyv::from_bytes` reject out-of-range or inconsistent archived values
  instead of panicking or returning wrong values on subsequent use.
* Bound Android and OpenHarmony `tzdata` header and index-entry allocations
  against the underlying file length (`SeekFrom::End(0)`) before allocating,
  rejecting oversized lengths up to 4 GiB (`0xFFFF_FFFF`) with `UnexpectedEof`
  or `InvalidData`.
* Saturate `From<DateTime<Tz>> for SystemTime` using `checked_add` /
  `checked_sub` and binary search on platforms with narrower `SystemTime`
  bounds (such as Windows or `wasm32-unknown-unknown`) instead of panicking,
  and normalize leap seconds before inverting pre-epoch offsets.
* Document `# Panics` sections on `DateTime::format`, `NaiveDateTime::format`,
  `NaiveDate::format`, and `NaiveTime::format` covering invalid specifiers,
  parse-only specifiers such as `%#z`, and specifiers for fields the type lacks,
  and recommend `write!` / `core::fmt::Write` with error handling for untrusted
  format strings.

### Dependencies

* `rkyv` `0.7.43` -> `0.8.18`; the `rkyv` 0.8 port is from upstream PR
  chrono#1779, by Yuriy Yarosh, closed unmerged upstream.
* `windows-link` `0.2` -> `0.100` with regenerated
  `src/offset/local/win_bindings.rs`, and `windows-bindgen` -> `0.100`
  (dev-dependency).
* `defmt` `1.0.1` -> `1.2` (informational
  [RUSTSEC-2026-0173](https://rustsec.org/advisories/RUSTSEC-2026-0173) on
  transitive `proc-macro-error2`; `defmt` `1.0.1` also allows `1.2` via
  `cargo update`).
* Replace unmaintained `bincode` `1.3.0`
  ([RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141)) with
  `postcard` `1` (`alloc`) in `dev-dependencies`.
* `criterion` `0.5.0` -> `0.8` in `bench/Cargo.toml` (switching
  `criterion::black_box` calls to `std::hint::black_box`).

### Testing and CI

* Add serde `0.4.46` (JSON and `postcard`) and `rkyv` `0.5.0` (`rkyv-32` +
  `rkyv-validation`) golden format tests in `tests/format_golden.rs`.
* Add an internal `__internal_fuzz` harness exposing `fuzz_tzif` and
  `fuzz_tz_string` targets alongside `fuzz_reader` and `fuzz_format`, and run
  all four targets in a new `fuzz` job in `lint.yml`.
* Pin all GitHub Actions `uses:` references to full commit SHAs, clear
  `deny.toml` advisory ignores, run `semver` directly against the published
  `sad-chrono` crate, and add Codecov coverage reporting (`coverage.yml`).

### Migration

* See [Migrating from 0.4.46](README.md#migrating-from-0446) in the README.

## sad-chrono 0.4.46

First release of `sad-chrono`. It is API-compatible with chrono 0.4.45 and
was forked from chrono's `main` branch at
[`6adaa52`](https://github.com/chronotope/chrono/commit/6adaa52), so it
includes every fix merged upstream since 0.4.45 that was never released
(except `chrono#1809`, noted below). It also includes fixes from three
upstream pull requests that are not merged yet: chrono#1811, chrono#1812,
and chrono#1825.

### Security and robustness

* TZif parsing: use checked arithmetic for data block lengths, so a header
  with huge record counts is rejected as an invalid file instead of
  overflowing `usize`, which panicked on 32-bit targets (from open upstream
  PR chrono#1812, by n0liu; reported in chrono#1802).
* `From<SystemTime> for DateTime<Utc>` saturates to `DateTime::MIN_UTC` /
  `DateTime::MAX_UTC` for values outside the range of `DateTime` instead of
  panicking (from open upstream PR chrono#1811, by n0liu; reported in
  chrono#1802). `sad-chrono` additionally fixes an overflow panic, in builds
  with overflow checks, at the earliest `SystemTime` on platforms with an
  `i64` `tv_sec`. Note that `From<SystemTime> for DateTime<Local>` saturates
  in UTC, so in time zones offset from UTC a saturated `DateTime<Local>`
  can have a local date-time outside `NaiveDateTime`'s range (methods such
  as `naive_local()` panic, as for any `DateTime` at the UTC bounds).
* Time zones with a UTC offset outside ±23:59:59 are rejected as invalid
  instead of making `Local` panic. This covers POSIX `TZ` strings whose
  implicit DST offset reaches ±24h (such as `AAA-23BBB,J1,J365`) and TZif
  files with out-of-range local time type offsets. `Local` takes its
  existing fallback for an invalid `TZ` (by the Sadco Security Team).
* Require TZif files loaded via `TZ=<path>`, `TZ=:<path>`,
  `/etc/localtime`, or the `iana-time-zone` fallback to be regular files
  (`is_file()`) and cap reads at 1 MiB, bounding TZif reads and rejecting
  paths already identified as non-regular before opening (by the Sadco
  Security Team).
* TZif local time lookup: avoid `i64` overflow for transition times near
  `i64::MIN` / `i64::MAX` (chrono#1796, by nvxbug).
* Android and OpenHarmony tzdata: skip a partial trailing index entry instead
  of slicing out of bounds (chrono#1797, by nvxbug).
* `NaiveDate::from_isoywd_opt` returns `None` for `i32::MIN` / `i32::MAX`
  years instead of panicking in builds with overflow checks (chrono#1810,
  by n0liu).

### Fixes

* Offsets of ±23:59:30 or more formatted at minute precision (including by
  `to_rfc3339`) now produce `±23:59` instead of the invalid `±24:00`, which
  failed to parse back (chrono#1798, by nvxbug).
* TZif files with time zone designations longer than 7 characters or
  containing `_` are accepted. Previously, a valid `/etc/localtime` using one
  made `Local` silently fall back to UTC (chrono#1795, by Dave Sims).
* Use `target_family = "wasm"` instead of `target_arch = "wasm32"` for the
  wasm-bindgen integration (chrono#1803, by Tomáš Procházka).

### Not included from upstream `main`

* `chrono#1809` (changing `NaiveDate::iter_days().rev()` and
  `iter_weeks().rev()` to walk backwards from `NaiveDate::MAX` instead of
  the start date) was omitted because it changes behavior documented in
  chrono 0.4.45 and would break callers relying on the drop-in contract.

### Known issues

* The `rkyv*` features use rkyv 0.7, which is no longer maintained upstream
  and is affected by
  [RUSTSEC-2026-0235](https://rustsec.org/advisories/RUSTSEC-2026-0235)
  (out-of-bounds reads when validating untrusted archives containing `Rc`,
  `Arc`, or weak pointers to unsized data). `sad-chrono`'s archived types
  contain no shared pointers, and `sad-chrono` stays on rkyv 0.7 to avoid
  breaking compatibility for existing `rkyv*` users; review the advisory if
  you validate untrusted rkyv archives of your own types.

### Packaging and internal

* Published as `sad-chrono`, with library name `sad_chrono`. To keep
  existing `chrono::` paths, depend on it as
  `chrono = { package = "sad-chrono", version = "0.4.46" }`.
* Fix doctests that failed to compile on Rust 1.99 and later because
  `core::i32::MAX` is deprecated (from open upstream PR chrono#1825, by
  Benjamin A. Beasley).
* Clippy 1.97 cleanups and CI maintenance (chrono#1794, chrono#1805); drop
  the Codecov workflow.

ChangeLog for Chrono
====================

This documents notable changes to [Chrono](https://github.com/chronotope/chrono)
up to and including version 0.4.19. For later releases, please review the
release notes on [GitHub](https://github.com/chronotope/chrono/releases).

## 0.4.19

* Correct build on solaris/illumos

## 0.4.18

* Restore support for x86_64-fortanix-unknown-sgx

## 0.4.17

* Fix a name resolution error in wasm-bindgen code introduced by removing the dependency on time
  v0.1

## 0.4.16

### Features

* Add %Z specifier to the `FromStr`, similar to the glibc strptime
  (does not set the offset from the timezone name)

* Drop the dependency on time v0.1, which is deprecated, unless the `oldtime`
  feature is active. This feature is active by default in v0.4.16 for backwards
  compatibility, but will likely be removed in v0.5. Code that imports
  `time::Duration` should be switched to import `chrono::Duration` instead to
  avoid breakage.

## 0.4.15

### Fixes

* Correct usage of vec in specific feature combinations (@quodlibetor)

## 0.4.14 **YANKED**

### Features

* Add day and week iterators for `NaiveDate` (@gnzlbg & @robyoung)
* Add a `Month` enum (@hhamana)
* Add `locales`. All format functions can now use locales, see the documentation for the
  `unstable-locales` feature.
* Fix `Local.from_local_datetime` method for wasm

### Improvements

* Added MIN and MAX values for `NaiveTime`, `NaiveDateTime` and `DateTime<Utc>`.

## 0.4.13

### Features

* Add `DurationRound` trait that allows rounding and truncating by `Duration` (@robyoung)

### Internal Improvements

* Code improvements to impl `From` for `js_sys` in wasm to reuse code (@schrieveslaach)

## 0.4.12

### New Methods and impls

* `Duration::abs` to ensure that a duration is just a magnitude (#418 @abreis).

### Compatibility improvements

* impl `From` for `js_sys` in wasm (#424 @schrieveslaach)
* Bump required version of `time` for redox support.

### Bugfixes

* serde modules do a better job with `Option` types (#417 @mwkroening and #429
  @fx-kirin)
* Use js runtime when using wasmbind to get the local offset (#412
  @quodlibetor)

### Internal Improvements

* Migrate to github actions from travis-ci, make the overall CI experience more comprehensible,
  significantly faster and more correct (#439 @quodlibetor)

## 0.4.11

### Improvements

* Support a space or `T` in `FromStr` for `DateTime<Tz>`, meaning that e.g.
  `dt.to_string().parse::<DateTime<Utc>>()` now correctly works on round-trip.
  (@quodlibetor in #378)
* Support "negative UTC" in `parse_from_rfc2822` (@quodlibetor #368 reported in
  #102)
* Support comparisons of DateTimes with different timezones (@dlalic in #375)
* Many documentation improvements

### Bitrot and external integration fixes

* Don't use wasmbind on wasi (@coolreader18 #365)
* Avoid deprecation warnings for `Error::description` (@AnderEnder and
  @quodlibetor #376)

### Internal improvements

* Use Criterion for benchmarks (@quodlibetor)

## 0.4.10

### Compatibility notes

* Putting some functionality behind an `alloc` feature to improve no-std
  support (in #341) means that if you were relying on chrono with
  `no-default-features` *and* using any of the functions that require alloc
  support (i.e. any of the string-generating functions like `to_rfc3339`) you
  will need to add the `alloc` feature in your Cargo.toml.

### Improvements

* `DateTime::parse_from_str` is more than 2x faster in some cases. (@michalsrb
  #358)
* Significant improvements to no-std and alloc support (This should also make
  many format/serialization operations induce zero unnecessary allocations)
  (@CryZe #341)

### Features

* Functions that were accepting `Iterator` of `Item`s (for example
  `format_with_items`) now accept `Iterator` of `Borrow<Item>`, so one can
  use values or references. (@michalsrb #358)
* Add built-in support for structs with nested `Option<Datetime>` etc fields
  (@manifest #302)

### Internal/doc improvements

* Use markdown footnotes on the `strftime` docs page (@qudlibetor #359)
* Migrate from `try!` -> `?` (question mark) because it is now emitting
  deprecation warnings and has been stable since rustc 1.13.0
* Deny dead code

## 0.4.9

### Fixes

* Make Datetime arithmetic adjust their offsets after discovering their new
  timestamps (@quodlibetor #337)
* Put wasm-bindgen related code and dependencies behind a `wasmbind` feature
  gate. (@quodlibetor #335)

## 0.4.8

### Fixes

* Add '0' to single-digit days in rfc2822 date format (@wyhaya #323)
* Correctly pad DelayedFormat (@SamokhinIlya #320)

### Features

* Support `wasm-unknown-unknown` via wasm-bindgen (in addition to
  emscripten/`wasm-unknown-emscripten`). (finished by @evq in #331, initial
  work by @jjpe #287)

## 0.4.7

### Fixes

* Disable libc default features so that CI continues to work on rust 1.13
* Fix panic on negative inputs to timestamp_millis (@cmars #292)
* Make `LocalResult` `Copy/Eq/Hash`

### Features

* Add `std::convert::From` conversions between the different timezone formats
  (@mqudsi #271)
* Add `timestamp_nanos` methods (@jean-airoldie #308)
* Documentation improvements

## 0.4.6

### Maintenance

* Doc improvements -- improve README CI verification, external links
* winapi upgrade to 0.3

## Unreleased

### Features

* Added `NaiveDate::from_weekday_of_month{,_opt}` for getting eg. the 2nd Friday of March 2017.

## 0.4.5

### Features

* Added several more serde deserialization helpers (@novacrazy #258)
* Enabled all features on the playground (@davidtwco #267)
* Derive `Hash` on `FixedOffset` (@LuoZijun #254)
* Improved docs (@storyfeet #261, @quodlibetor #252)

## 0.4.4

### Features

* Added support for parsing nanoseconds without the leading dot (@emschwartz #251)

## 0.4.3

### Features

* Added methods to DateTime/NaiveDateTime to present the stored value as a number
  of nanoseconds since the UNIX epoch (@harkonenbade #247)
* Added a serde serialise/deserialise module for nanosecond timestamps. (@harkonenbade #247)
* Added "Permissive" timezone parsing which allows a numeric timezone to
  be specified without minutes. (@quodlibetor #242)

## 0.4.2

### Deprecations

* More strongly deprecate RustcSerialize: remove it from documentation unless
  the feature is enabled, issue a deprecation warning if the rustc-serialize
  feature is enabled (@quodlibetor #174)

### Features

* Move all uses of the system clock behind a `clock` feature, for use in
  environments where we don't have access to the current time. (@jethrogb #236)
* Implement subtraction of two `Date`s, `Time`s, or `DateTime`s, returning a
  `Duration` (@tobz1000 #237)

## 0.4.1

### Bug Fixes

* Allow parsing timestamps with subsecond precision (@jonasbb)
* RFC2822 allows times to not include the second (@upsuper)

### Features

* New `timestamp_millis` method on `DateTime` and `NaiveDateTime` that returns
  number of milliseconds since the epoch. (@quodlibetor)
* Support exact decimal width on subsecond display for RFC3339 via a new
  `to_rfc3339_opts` method on `DateTime` (@dekellum)
* Use no_std-compatible num dependencies (@cuviper)
* Add `SubsecRound` trait that allows rounding to the nearest second
  (@dekellum)

### Code Hygiene and Docs

* Docs! (@alatiera @kosta @quodlibetor @kennytm)
* Run clippy and various fixes (@quodlibetor)

## 0.4.0 (2017-06-22)

This was originally planned as a minor release but was pushed to a major
release due to the compatibility concern raised.

### Added

- `IsoWeek` has been added for the ISO week without time zone.

- The `+=` and `-=` operators against `time::Duration` are now supported for
  `NaiveDate`, `NaiveTime` and `NaiveDateTime`. (#99)

  (Note that this does not invalidate the eventual deprecation of `time::Duration`.)

- `SystemTime` and `DateTime<Tz>` types can be now converted to each other via `From`.
  Due to the obvious lack of time zone information in `SystemTime`,
  the forward direction is limited to `DateTime<Utc>` and `DateTime<Local>` only.

### Changed

- Intermediate implementation modules have been flattened (#161),
  and `UTC` has been renamed to `Utc` in accordance with the current convention (#148).

  The full list of changes is as follows:

  Before                                   | After
  ---------------------------------------- | ----------------------------
  `chrono::date::Date`                     | `chrono::Date`
  `chrono::date::MIN`                      | `chrono::MIN_DATE`
  `chrono::date::MAX`                      | `chrono::MAX_DATE`
  `chrono::datetime::DateTime`             | `chrono::DateTime`
  `chrono::naive::time::NaiveTime`         | `chrono::naive::NaiveTime`
  `chrono::naive::date::NaiveDate`         | `chrono::naive::NaiveDate`
  `chrono::naive::date::MIN`               | `chrono::naive::MIN_DATE`
  `chrono::naive::date::MAX`               | `chrono::naive::MAX_DATE`
  `chrono::naive::datetime::NaiveDateTime` | `chrono::naive::NaiveDateTime`
  `chrono::offset::utc::UTC`               | `chrono::offset::Utc`
  `chrono::offset::fixed::FixedOffset`     | `chrono::offset::FixedOffset`
  `chrono::offset::local::Local`           | `chrono::offset::Local`
  `chrono::format::parsed::Parsed`         | `chrono::format::Parsed`

  With an exception of `Utc`, this change does not affect any direct usage of
  `chrono::*` or `chrono::prelude::*` types.

- `Datelike::isoweekdate` is replaced by `Datelike::iso_week` which only returns the ISO week.

  The original method used to return a tuple of year number, week number and day of the week,
  but this duplicated the `Datelike::weekday` method and it had been hard to deal with
  the raw year and week number for the ISO week date.
  This change isolates any logic and API for the week date into a separate type.

- `NaiveDateTime` and `DateTime` can now be deserialized from an integral UNIX timestamp. (#125)

  This turns out to be very common input for web-related usages.
  The existing string representation is still supported as well.

- `chrono::serde` and `chrono::naive::serde` modules have been added
  for the serialization utilities. (#125)

  Currently they contain the `ts_seconds` modules that can be used to
  serialize `NaiveDateTime` and `DateTime` values into an integral UNIX timestamp.
  This can be combined with Serde's `[de]serialize_with` attributes
  to fully support the (de)serialization to/from the timestamp.

  For rustc-serialize, there are separate `chrono::TsSeconds` and `chrono::naive::TsSeconds` types
  that are newtype wrappers implementing different (de)serialization logics.
  This is a suboptimal API, however, and it is strongly recommended to migrate to Serde.

### Fixed

- The major version was made to fix the broken Serde dependency issues. (#146, #156, #158, #159)

  The original intention to technically break the dependency was
  to facilitate the use of Serde 1.0 at the expense of temporary breakage.
  Whether this was appropriate or not is quite debatable,
  but it became clear that there are several high-profile crates requiring Serde 0.9
  and it is not feasible to force them to use Serde 1.0 anyway.

  To the end, the new major release was made with some known lower-priority breaking changes.
  0.3.1 is now yanked and any remaining 0.3 users can safely roll back to 0.3.0.

- Various documentation fixes and goodies. (#92, #131, #136)

## 0.3.1 (2017-05-02)

### Added

- `Weekday` now implements `FromStr`, `Serialize` and `Deserialize`. (#113)

  The syntax is identical to `%A`, i.e. either the shortest or the longest form of English names.

### Changed

- Serde 1.0 is now supported. (#142)

  This is technically a breaking change because Serde 0.9 and 1.0 are not compatible,
  but this time we decided not to issue a minor version because
  we have already seen Serde 0.8 and 0.9 compatibility problems even after 0.3.0 and
  a new minor version turned out to be not very helpful for this kind of issues.

### Fixed

- Fixed a bug that the leap second can be mapped wrongly in the local time zone.
  Only occurs when the local time zone is behind UTC. (#130)

## 0.3.0 (2017-02-07)

The project has moved to the [Chronotope](https://github.com/chronotope/) organization.

### Added

- `chrono::prelude` module has been added. All other glob imports are now discouraged.

- `FixedOffset` can be added to or subtracted from any timelike types.

    - `FixedOffset::local_minus_utc` and `FixedOffset::utc_minus_local` methods have been added.
      Note that the old `Offset::local_minus_utc` method is gone; see below.

- Serde support for non-self-describing formats like Bincode is added. (#89)

- Added `Item::Owned{Literal,Space}` variants for owned formatting items. (#76)

- Formatting items and the `Parsed` type have been slightly adjusted so that
  they can be internally extended without breaking any compatibility.

- `Weekday` is now `Hash`able. (#109)

- `ParseError` now implements `Eq` as well as `PartialEq`. (#114)

- More documentation improvements. (#101, #108, #112)

### Changed

- Chrono now only supports Rust 1.13.0 or later (previously: Rust 1.8.0 or later).

- Serde 0.9 is now supported.
  Due to the API difference, support for 0.8 or older is discontinued. (#122)

- Rustc-serialize implementations are now on par with corresponding Serde implementations.
  They both standardize on the `std::fmt::Debug` textual output.

  **This is a silent breaking change (hopefully the last though).**
  You should be prepared for the format change if you depended on rustc-serialize.

- `Offset::local_minus_utc` is now `Offset::fix`, and returns `FixedOffset` instead of a duration.

  This makes every time zone operation operate within a bias less than one day,
  and vastly simplifies many logics.

- `chrono::format::format` now receives `FixedOffset` instead of `time::Duration`.

- The following methods and implementations have been renamed and older names have been *removed*.
  The older names will be reused for the same methods with `std::time::Duration` in the future.

    - `checked_*` → `checked_*_signed` in `Date`, `DateTime`, `NaiveDate` and `NaiveDateTime` types

    - `overflowing_*` → `overflowing_*_signed` in the `NaiveTime` type

    - All subtraction implementations between two time instants have been moved to
      `signed_duration_since`, following the naming in `std::time`.

### Fixed

- Fixed a panic when the `Local` offset receives a leap second. (#123)

### Removed

- Rustc-serialize support for `Date<Tz>` types and all offset types has been dropped.

  These implementations were automatically derived and never had been in a good shape.
  Moreover there are no corresponding Serde implementations, limiting their usefulness.
  In the future they may be revived with more complete implementations.

- The following method aliases deprecated in the 0.2 branch have been removed.

    - `DateTime::num_seconds_from_unix_epoch` (→ `DateTime::timestamp`)
    - `NaiveDateTime::from_num_seconds_from_unix_epoch` (→ `NaiveDateTime::from_timestamp`)
    - `NaiveDateTime::from_num_seconds_from_unix_epoch_opt` (→ `NaiveDateTime::from_timestamp_opt`)
    - `NaiveDateTime::num_seconds_unix_epoch` (→ `NaiveDateTime::timestamp`)

- Formatting items are no longer `Copy`, except for `chrono::format::Pad`.

- `chrono::offset::add_with_leapsecond` has been removed.
  Use a direct addition with `FixedOffset` instead.

## 0.2.25 (2016-08-04)

This is the last version officially supports Rust 1.12.0 or older.

(0.2.24 was accidentally uploaded without a proper check for warnings in the default state,
and replaced by 0.2.25 very shortly. Duh.)

### Added

- Serde 0.8 is now supported. 0.7 also remains supported. (#86)

### Fixed

- The deserialization implementation for rustc-serialize now properly verifies the input.
  All serialization codes are also now thoroughly tested. (#42)

## 0.2.23 (2016-08-03)

### Added

- The documentation was greatly improved for several types,
  and tons of cross-references have been added. (#77, #78, #80, #82)

- `DateTime::timestamp_subsec_{millis,micros,nanos}` methods have been added. (#81)

### Fixed

- When the system time records a leap second,
  the nanosecond component was mistakenly reset to zero. (#84)

- `Local` offset misbehaves in Windows for August and later,
  due to the long-standing libtime bug (dates back to mid-2015).
  Workaround has been implemented. (#85)

## 0.2.22 (2016-04-22)

### Fixed

- `%.6f` and `%.9f` used to print only three digits when the nanosecond part is zero. (#71)
- The documentation for `%+` has been updated to reflect the current status. (#71)

## 0.2.21 (2016-03-29)

### Fixed

- `Fixed::LongWeekdayName` was unable to recognize `"sunday"` (whoops). (#66)

## 0.2.20 (2016-03-06)

### Changed

- `serde` dependency has been updated to 0.7. (#63, #64)

## 0.2.19 (2016-02-05)

### Added

- The documentation for `Date` is made clear about its ambiguity and guarantees.

### Fixed

- `DateTime::date` had been wrong when the local date and the UTC date is in disagreement. (#61)

## 0.2.18 (2016-01-23)

### Fixed

- Chrono no longer pulls a superfluous `rand` dependency. (#57)

## 0.2.17 (2015-11-22)

### Added

- Naive date and time types and `DateTime` now have a `serde` support.
  They serialize as an ISO 8601 / RFC 3339 string just like `Debug`. (#51)

## 0.2.16 (2015-09-06)

### Added

- Added `%.3f`, `%.6f` and `%.9f` specifier for formatting fractional seconds
  up to 3, 6 or 9 decimal digits. This is a natural extension to the existing `%f`.
  Note that this is (not yet) generic, no other value of precision is supported. (#45)

### Changed

- Forbade unsized types from implementing `Datelike` and `Timelike`.
  This does not make a big harm as any type implementing them should be already sized
  to be practical, but this change still can break highly generic codes. (#46)

### Fixed

- Fixed a broken link in the `README.md`. (#41)

## 0.2.15 (2015-07-05)

### Added

- Padding modifiers `%_?`, `%-?` and `%0?` are implemented.
  They are glibc extensions which seem to be reasonably widespread (e.g. Ruby).

- Added `%:z` specifier and corresponding formatting items
  which is essentially the same as `%z` but with a colon.

- Added a new specifier `%.f` which precision adapts from the input.
  This was added as a response to the UX problems in the original nanosecond specifier `%f`.

### Fixed

- `Numeric::Timestamp` specifier (`%s`) was ignoring the time zone offset when provided.

- Improved the documentation and associated tests for `strftime`.

## 0.2.14 (2015-05-15)

### Fixed

- `NaiveDateTime +/- Duration` or `NaiveTime +/- Duration` could have gone wrong
  when the `Duration` to be added is negative and has a fractional second part.
  This was caused by an underflow in the conversion from `Duration` to the parts;
  the lack of tests for this case allowed a bug. (#37)

## 0.2.13 (2015-04-29)

### Added

- The optional dependency on `rustc_serialize` and
  relevant `Rustc{En,De}codable` implementations for supported types has been added.
  This is enabled by the `rustc-serialize` Cargo feature. (#34)

### Changed

- `chrono::Duration` reexport is changed to that of crates.io `time` crate.
  This enables Rust 1.0 beta compatibility.

## 0.2.4 (2015-03-03)

### Fixed

- Clarified the meaning of `Date<Tz>` and fixed unwanted conversion problem
  that only occurs with positive UTC offsets. (#27)

## 0.2.3 (2015-02-27)

### Added

- `DateTime<Tz>` and `Date<Tz>` is now `Copy`/`Send` when `Tz::Offset` is `Copy`/`Send`.
  The implementations for them were mistakenly omitted. (#25)

### Fixed

- `Local::from_utc_datetime` didn't set a correct offset. (#26)

## 0.2.1 (2015-02-21)

### Changed

- `DelayedFormat` no longer conveys a redundant lifetime.

## 0.2.0 (2015-02-19)

### Added

- `Offset` is split into `TimeZone` (constructor) and `Offset` (storage) types.
  You would normally see only the former, as the latter is mostly an implementation detail.
  Most importantly, `Local` now can be used to directly construct timezone-aware values.

  Some types (currently, `UTC` and `FixedOffset`) are both `TimeZone` and `Offset`,
  but others aren't (e.g. `Local` is not what is being stored to each `DateTime` values).

- `LocalResult::map` convenience method has been added.

- `TimeZone` now allows a construction of `DateTime` values from UNIX timestamp,
  via `timestamp` and `timestamp_opt` methods.

- `TimeZone` now also has a method for parsing `DateTime`, namely `datetime_from_str`.

- The following methods have been added to all date and time types:

    - `checked_add`
    - `checked_sub`
    - `format_with_items`

- The following methods have been added to all timezone-aware types:

    - `timezone`
    - `with_timezone`
    - `naive_utc`
    - `naive_local`

- `parse_from_str` method has been added to all naive types and `DateTime<FixedOffset>`.

- All naive types and instances of `DateTime` with time zones `UTC`, `Local` and `FixedOffset`
  implement the `FromStr` trait. They parse what `std::fmt::Debug` would print.

- `chrono::format` has been greatly rewritten.

    - The formatting syntax parser is modular now, available at `chrono::format::strftime`.

    - The parser and resolution algorithm is also modular, the former is available at
      `chrono::format::parse` while the latter is available at `chrono::format::parsed`.

    - Explicit support for RFC 2822 and 3339 syntaxes is landed.

    - There is a minor formatting difference with atypical values,
      e.g. for years not between 1 BCE and 9999 CE.

### Changed

- Most uses of `Offset` are converted to `TimeZone`.
  In fact, *all* user-facing code is expected to be `Offset`-free.

- `[Naive]DateTime::*num_seconds_from_unix_epoch*` methods have been renamed to
  simply `timestamp` or `from_timestamp*`. The original names have been deprecated.

### Removed

- `Time` has been removed. This also prompts a related set of methods in `TimeZone`.

  This is in principle possible, but in practice has seen a little use
  because it can only be meaningfully constructed via an existing `DateTime` value.
  This made many operations to `Time` unintuitive or ambiguous,
  so we simply let it go.

  In the case that `Time` is really required, one can use a simpler `NaiveTime`.
  `NaiveTime` and `NaiveDate` can be freely combined and split,
  and `TimeZone::from_{local,utc}_datetime` can be used to convert from/to the local time.

- `with_offset` method has been removed. Use `with_timezone` method instead.
  (This is not deprecated since it is an integral part of offset reform.)

## 0.1.14 (2015-01-10)

### Added

- Added a missing `std::fmt::String` impl for `Local`.

## 0.1.13 (2015-01-10)

### Changed

- Most types now implement both `std::fmt::Show` and `std::fmt::String`,
  with the former used for the stricter output and the latter used for more casual output.

### Removed

- `Offset::name` has been replaced by a `std::fmt::String` implementation to `Offset`.

## 0.1.12 (2015-01-08)

### Removed

- `Duration + T` no longer works due to the updated impl reachability rules.
  Use `T + Duration` as a workaround.

## 0.1.4 (2014-12-13)

### Fixed

- Fixed a bug that `Date::and_*` methods with an offset that can change the date are
  off by one day.

## 0.1.3 (2014-11-28)

### Added

- `{Date,Time,DateTime}::with_offset` methods have been added.

- `LocalResult` now implements a common set of traits.

- `LocalResult::and_*` methods have been added.
  They are useful for safely chaining `LocalResult<Date<Off>>` methods
  to make `LocalResult<DateTime<Off>>`.

### Changed

- `Offset::name` now returns `SendStr`.

- `{Date,Time} - Duration` overloadings are now allowed.

## 0.1.2 (2014-11-24)

### Added

- `Duration + Date` overloading is now allowed.

### Changed

- Chrono no longer needs `num` dependency.

## 0.1.0 (2014-11-20)

The initial version that was available to `crates.io`.
