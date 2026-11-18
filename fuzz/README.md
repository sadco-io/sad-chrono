# Fuzzing Chrono
To fuzz Chrono we rely on the [Cargo-fuzz](https://rust-fuzz.github.io/) project.

To install cargo-fuzz:
```sh
cargo install cargo-fuzz
```

Available fuzz targets:
- `fuzz_reader`: `DateTime::parse_from_rfc2822` and `DateTime::parse_from_rfc3339`
- `fuzz_format`: `DateTime::parse_from_str`
- `fuzz_tzif`: TZif parser (Unix-only)
- `fuzz_tz_string`: POSIX `TZ` string parser (Unix-only)

To run a fuzzer, navigate to the repository root and run `cargo fuzz run <target>`, for example:
```sh
cargo fuzz run fuzz_reader
```
