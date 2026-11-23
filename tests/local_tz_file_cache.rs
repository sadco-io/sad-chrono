#![cfg(all(unix, feature = "clock", feature = "std"))]

use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, fs, thread};

use sad_chrono::{Local, Offset};

const CHILD_ENV: &str = "SAD_CHRONO_LOCAL_TZ_FILE_CACHE_CHILD";
const OFFSET_PREFIX: &str = "local-offset:";
// Longer than the one-second window in which the Unix `Local` cache skips revalidation.
const REVALIDATION_WAIT: Duration = Duration::from_millis(1500);

const PLUS_THREE: i32 = 3 * 3600;
const MINUS_FIVE: i32 = -5 * 3600;

#[test]
fn tz_path_follows_replaced_file() {
    check_offset_follows_source(
        "tz_path_follows_replaced_file",
        |dir| {
            write_tzif(&dir.join("zone"), PLUS_THREE, "+03");
            dir.join("zone").display().to_string()
        },
        |dir| replace_file(&dir.join("zone"), MINUS_FIVE, "-05"),
    );
}

#[test]
fn tz_colon_path_follows_replaced_file() {
    check_offset_follows_source(
        "tz_colon_path_follows_replaced_file",
        |dir| {
            write_tzif(&dir.join("zone"), PLUS_THREE, "+03");
            format!(":{}", dir.join("zone").display())
        },
        |dir| replace_file(&dir.join("zone"), MINUS_FIVE, "-05"),
    );
}

#[test]
fn tz_symlink_follows_replaced_target() {
    check_offset_follows_source(
        "tz_symlink_follows_replaced_target",
        |dir| {
            write_tzif(&dir.join("zone"), PLUS_THREE, "+03");
            symlink(dir.join("zone"), dir.join("link")).unwrap();
            dir.join("link").display().to_string()
        },
        |dir| replace_file(&dir.join("zone"), MINUS_FIVE, "-05"),
    );
}

#[test]
fn tz_symlink_follows_new_target() {
    check_offset_follows_source(
        "tz_symlink_follows_new_target",
        |dir| {
            write_tzif(&dir.join("plus_three"), PLUS_THREE, "+03");
            write_tzif(&dir.join("minus_five"), MINUS_FIVE, "-05");
            symlink(dir.join("plus_three"), dir.join("link")).unwrap();
            dir.join("link").display().to_string()
        },
        |dir| {
            symlink(dir.join("minus_five"), dir.join("link.new")).unwrap();
            fs::rename(dir.join("link.new"), dir.join("link")).unwrap();
        },
    );
}

// Oracle: the offsets the test itself encodes into the TZif files, +03:00 before and -05:00
// after the replacement.
fn check_offset_follows_source(
    test_name: &str,
    create_source: impl FnOnce(&Path) -> String,
    replace_source: impl FnOnce(&Path),
) {
    if env::var_os(CHILD_ENV).is_some() {
        report_offsets_around_replacement();
        return;
    }

    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(test_name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let tz = create_source(&dir);

    let mut child = Command::new(env::current_exe().unwrap())
        .args([test_name, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, "1")
        .env("TZ", &tz)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut offsets = BufReader::new(child.stderr.take().unwrap())
        .lines()
        .map(|line| line.unwrap())
        .filter_map(|line| line.strip_prefix(OFFSET_PREFIX).map(|o| o.parse::<i32>().unwrap()));

    let before = offsets.next().expect("child should report the offset before replacement");
    replace_source(&dir);
    writeln!(child.stdin.take().unwrap()).unwrap();
    let after = offsets.next().expect("child should report the offset after replacement");
    assert!(child.wait().unwrap().success());

    assert_eq!(before, PLUS_THREE, "TZ={tz} before replacement");
    assert_eq!(after, MINUS_FIVE, "TZ={tz} after replacement");
}

fn report_offsets_around_replacement() {
    eprintln!("{OFFSET_PREFIX}{}", Local::now().offset().fix().local_minus_utc());
    io::stdin().lock().read_line(&mut String::new()).unwrap();
    thread::sleep(REVALIDATION_WAIT);
    eprintln!("{OFFSET_PREFIX}{}", Local::now().offset().fix().local_minus_utc());
}

fn replace_file(path: &Path, utoff: i32, designation: &str) {
    let staged = path.with_extension("new");
    write_tzif(&staged, utoff, designation);
    fs::rename(staged, path).unwrap();
}

// TZif version 1 layout from RFC 8536, with one local time type and no transitions.
fn write_tzif(path: &Path, utoff: i32, designation: &str) {
    let mut bytes = b"TZif".to_vec();
    bytes.push(0); // version 1
    bytes.extend([0; 15]);
    let char_count = designation.len() as u32 + 1;
    // isutcnt, isstdcnt, leapcnt, timecnt, typecnt, charcnt
    for count in [0, 0, 0, 0, 1, char_count] {
        bytes.extend(u32::to_be_bytes(count));
    }
    bytes.extend(utoff.to_be_bytes());
    bytes.extend([0, 0]); // isdst, desigidx
    bytes.extend(designation.as_bytes());
    bytes.push(0);
    fs::write(path, bytes).unwrap();
}
