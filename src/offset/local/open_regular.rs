use std::fs::{self, File};
use std::io;
use std::path::Path;

#[cfg(all(
    unix,
    any(target_os = "linux", target_os = "android"),
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "loongarch64",
    )
))]
pub(super) const O_NONBLOCK: Option<i32> = Some(0o4000);

#[cfg(all(
    unix,
    any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd",
    )
))]
pub(super) const O_NONBLOCK: Option<i32> = Some(0x0004);

#[cfg(all(unix, any(target_os = "solaris", target_os = "illumos")))]
pub(super) const O_NONBLOCK: Option<i32> = Some(0x80);

#[cfg(all(
    unix,
    not(any(
        all(
            any(target_os = "linux", target_os = "android"),
            any(
                target_arch = "x86",
                target_arch = "x86_64",
                target_arch = "arm",
                target_arch = "aarch64",
                target_arch = "riscv32",
                target_arch = "riscv64",
                target_arch = "s390x",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "loongarch64",
            )
        ),
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
        target_os = "illumos",
    ))
))]
pub(super) const O_NONBLOCK: Option<i32> = None;

#[cfg(all(test, unix))]
const _: () = if let Some(val) = O_NONBLOCK {
    assert!(val == libc::O_NONBLOCK);
};

#[cfg(all(
    test,
    unix,
    any(
        all(
            any(target_os = "linux", target_os = "android"),
            any(
                target_arch = "x86",
                target_arch = "x86_64",
                target_arch = "arm",
                target_arch = "aarch64",
                target_arch = "riscv32",
                target_arch = "riscv64",
                target_arch = "s390x",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "loongarch64",
            )
        ),
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
        target_os = "illumos",
    )
))]
const _: () = assert!(O_NONBLOCK.is_some());

/// Open `path` read-only without blocking on FIFOs/devices (when `O_NONBLOCK` is known), then
/// verify on the opened handle that it is a regular file.
///
/// Returns `Ok(None)` if `path` exists and is not a regular file.
#[cfg(unix)]
pub(super) fn open_nonblocking_checked(path: &Path) -> io::Result<Option<File>> {
    let file = match O_NONBLOCK {
        Some(nonblock) => {
            use std::os::unix::fs::OpenOptionsExt;
            fs::OpenOptions::new().read(true).custom_flags(nonblock).open(path)?
        }
        None => File::open(path)?,
    };
    if !file.metadata()?.is_file() {
        return Ok(None);
    }
    Ok(Some(file))
}

/// Check `fs::metadata(path)?.is_file()` as a fast path, then open `path` read-only without
/// blocking on FIFOs/devices and verify `is_file()` on the opened handle.
///
/// Returns `Ok(None)` if `path` exists and is not a regular file.
pub(super) fn open_regular_nonblocking(path: &Path) -> io::Result<Option<File>> {
    if !fs::metadata(path)?.is_file() {
        return Ok(None);
    }
    #[cfg(unix)]
    {
        open_nonblocking_checked(path)
    }
    #[cfg(not(unix))]
    {
        let file = File::open(path)?;
        if !file.metadata()?.is_file() {
            return Ok(None);
        }
        Ok(Some(file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    struct RemoveDirOnDrop(std::path::PathBuf);

    #[cfg(unix)]
    impl Drop for RemoveDirOnDrop {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[cfg(unix)]
    fn create_temp_dir(prefix: &str) -> (std::path::PathBuf, RemoveDirOnDrop) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique =
            SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("{prefix}-{}-{unique:x}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let guard = RemoveDirOnDrop(dir.clone());
        (dir, guard)
    }

    #[test]
    fn test_open_regular_nonblocking_basic() {
        let missing = std::env::temp_dir()
            .join(format!("sad-chrono-open-regular-missing-{}", std::process::id()));
        let _ = fs::remove_file(&missing);
        assert_eq!(open_regular_nonblocking(&missing).unwrap_err().kind(), io::ErrorKind::NotFound);

        let dir = std::env::temp_dir();
        assert!(open_regular_nonblocking(&dir).unwrap().is_none());

        let reg = Path::new("./tests/android/tzdata");
        assert!(open_regular_nonblocking(reg).unwrap().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn test_open_regular_nonblocking_fifo_and_symlink() {
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::OpenOptionsExt;
        use std::sync::mpsc;
        use std::time::Duration;

        let (dir, _guard) = create_temp_dir("sad-chrono-open-regular");

        // Symlink to a regular file succeeds.
        let target = std::env::current_dir().unwrap().join("tests/android/tzdata");
        let link = dir.join("tzdata-link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(open_regular_nonblocking(&link).unwrap().is_some());
        assert!(open_nonblocking_checked(&link).unwrap().is_some());

        // FIFO with no writer is rejected promptly.
        let fifo = dir.join("fifo");
        let c_fifo = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(
            unsafe { libc::mkfifo(c_fifo.as_ptr(), 0o644) },
            0,
            "mkfifo failed: {}",
            io::Error::last_os_error()
        );

        let run_open = |bypass_precheck: bool| {
            let p = fifo.clone();
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let res = if bypass_precheck {
                    open_nonblocking_checked(&p)
                } else {
                    open_regular_nonblocking(&p)
                };
                let _ = tx.send(res.map(|opt| opt.is_some()));
            });
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(res) => res,
                Err(_) => {
                    let mut opts = fs::OpenOptions::new();
                    opts.write(true);
                    if let Some(nb) = O_NONBLOCK {
                        opts.custom_flags(nb);
                    }
                    let _ = opts.open(&fifo);
                    panic!("open blocked on FIFO (bypass_precheck={bypass_precheck})");
                }
            }
        };

        assert!(!run_open(false).unwrap());
        if O_NONBLOCK.is_some() {
            assert!(!run_open(true).unwrap());
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_open_regular_nonblocking_socket_rejected_before_open() {
        use std::os::unix::net::UnixListener;

        let (dir, _guard) = create_temp_dir("sc-or");
        let sock = dir.join("s");
        let _listener = UnixListener::bind(&sock).unwrap();

        // `open(2)` on a socket fails (ENXIO on Linux, EOPNOTSUPP on macOS), so `Ok(None)`
        // proves the `fs::metadata` pre-check ran before opening.
        assert!(open_regular_nonblocking(&sock).unwrap().is_none());
    }
}
