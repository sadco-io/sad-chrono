#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    #[cfg(unix)]
    chrono::__fuzz::tz_string(data);
    #[cfg(not(unix))]
    let _ = data;
});
