#![cfg(all(unix, feature = "clock", feature = "std"))]

use sad_chrono::Local;

#[test]
fn test_local_does_not_panic_on_invalid_tz() {
    std::env::set_var("TZ", "AAA-23BBB,J1,J365");
    let _ = Local::now();

    // `TZ_INFO` is cached per thread for 1 second, so test a second `TZ` value on a fresh thread.
    std::env::set_var("TZ", "/dev/zero");
    std::thread::spawn(|| {
        let _ = Local::now();
    })
    .join()
    .unwrap();
}
