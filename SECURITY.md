# Security Policy

## Supported Versions

| Version | Supported | Notes |
|---------|-----------|-------|
| `0.5.x` | All fixes | Current release line |
| `0.4.x` | Security fixes only | Maintained on the `0.4.x` branch |
| `< 0.4.46` | No | Upgrade to `0.5.x` or `0.4.x` |

## Scope

`sad-chrono` parses untrusted input: date and time strings, the `TZ` environment variable,
TZif time zone files, Android/OpenHarmony `tzdata` bundles, and `rkyv`/`serde` deserialization of
untrusted bytes. Panics, unbounded resource use, or incorrect results reachable from such input are
in scope, as are panics in APIs documented as fallible (`*_opt`, `checked_*`, or functions returning
`Option`/`Result`) and in infallible conversions such as `From` impls.

## Advisories

For each security fix we publish a GitHub Security Advisory for `sad-chrono` and request a RustSec
advisory. We watch upstream chrono's issue tracker and RustSec advisories for `chrono`, and mirror
any that affect shared code. Because `cargo audit` and `cargo deny` match advisories by package
name, an upstream `chrono` advisory will not flag `sad-chrono` on its own.

## Reporting a Vulnerability

If you have discovered a security vulnerability in this project, please report
it privately. **Do not disclose it as a public issue.** This gives us time to
work with you to fix the issue before public exposure, reducing the chance that
the exploit will be used before a patch is released.

Please disclose it at [security advisory](https://github.com/sadco-io/sad-chrono/security/advisories/new).

If the issue also affects upstream [chrono](https://github.com/chronotope/chrono), we will
coordinate with its maintainers where possible.

This project is maintained by a team of volunteers on a reasonable-effort basis.
As such, please give us at least 90 days to work on a fix before public exposure.
