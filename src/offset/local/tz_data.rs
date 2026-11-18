//! Rust parser of ZoneInfoDb(`tzdata`) on Android and OpenHarmony
//!
//! Ported from: https://android.googlesource.com/platform/prebuilts/fullsdk/sources/+/refs/heads/androidx-appcompat-release/android-34/com/android/i18n/timezone/ZoneInfoDb.java
use std::{
    ffi::CStr,
    fmt::Debug,
    fs::File,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom},
    mem::size_of,
};

/// Open a `tzdata` path only if it is a regular file.
#[cfg(any(test, target_os = "android", target_env = "ohos"))]
fn open_regular_file(path: &str) -> Result<File> {
    // Check metadata before opening so a FIFO does not block in `File::open`.
    if !std::fs::metadata(path)?.is_file() {
        return Err(Error::new(ErrorKind::InvalidInput, "tzdata path is not a regular file"));
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(Error::new(ErrorKind::InvalidInput, "tzdata path is not a regular file"));
    }
    Ok(file)
}

/// Get timezone data from the `tzdata` file of HarmonyOS NEXT.
#[cfg(target_env = "ohos")]
pub(crate) fn for_zone(tz_string: &str) -> Result<Option<Vec<u8>>> {
    let mut file = open_regular_file("/system/etc/zoneinfo/tzdata")?;
    find_tz_data::<OHOS_ENTRY_LEN, _>(&mut file, tz_string.as_bytes())
}

/// Get timezone data from the `tzdata` file of Android.
#[cfg(target_os = "android")]
pub(crate) fn for_zone(tz_string: &str) -> Result<Option<Vec<u8>>> {
    let mut file = open_android_tz_data_file()?;
    find_tz_data::<ANDROID_ENTRY_LEN, _>(&mut file, tz_string.as_bytes())
}

/// Open the `tzdata` file of Android from the environment variables.
#[cfg(target_os = "android")]
fn open_android_tz_data_file() -> Result<File> {
    for (env_var, path) in [
        // Default root path: /apex/com.android.tzdata
        ("ANDROID_TZDATA_ROOT", "/etc/tz"),
        // Default root path: /data
        ("ANDROID_DATA", "/misc/zoneinfo/current"),
        // Default root path: /system
        ("ANDROID_ROOT", "/usr/share/zoneinfo"),
    ] {
        if let Ok(env_value) = std::env::var(env_var) {
            if let Ok(file) = open_regular_file(&format!("{}{}/tzdata", env_value, path)) {
                return Ok(file);
            }
        }
    }
    Err(Error::from(ErrorKind::NotFound))
}

/// Get timezone data from the `tzdata` file reader
#[cfg(any(test, target_env = "ohos", target_os = "android"))]
fn find_tz_data<const ENTRY_LEN: usize, R: Read + Seek>(
    mut reader: R,
    tz_name: &[u8],
) -> Result<Option<Vec<u8>>> {
    let header = TzDataHeader::new(&mut reader)?;
    let index = TzDataIndexes::new::<ENTRY_LEN, _>(&mut reader, &header)?;
    Ok(if let Some(entry) = index.find_timezone(tz_name) {
        Some(index.find_tzdata(reader, &header, entry)?)
    } else {
        None
    })
}

/// Header of the `tzdata` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TzDataHeader {
    version: [u8; 5],
    index_offset: u32,
    data_offset: u32,
    zonetab_offset: u32,
}

impl TzDataHeader {
    /// Parse the header of the `tzdata` file.
    fn new(mut data: impl Read) -> Result<Self> {
        let version = {
            let mut magic = [0; TZDATA_VERSION_LEN];
            data.read_exact(&mut magic)?;
            if !magic.starts_with(b"tzdata") || magic[TZDATA_VERSION_LEN - 1] != 0 {
                return Err(Error::new(ErrorKind::Other, "invalid tzdata header magic"));
            }
            let mut version = [0; 5];
            version.copy_from_slice(&magic[6..11]);
            version
        };

        let mut offset = [0; 4];
        data.read_exact(&mut offset)?;
        let index_offset = u32::from_be_bytes(offset);
        data.read_exact(&mut offset)?;
        let data_offset = u32::from_be_bytes(offset);
        data.read_exact(&mut offset)?;
        let zonetab_offset = u32::from_be_bytes(offset);

        Ok(Self { version, index_offset, data_offset, zonetab_offset })
    }
}

/// Indexes of the `tzdata` file.
struct TzDataIndexes {
    indexes: Vec<TzDataIndex>,
}

impl TzDataIndexes {
    /// Create a new `TzDataIndexes` from the `tzdata` file reader.
    fn new<const ENTRY_LEN: usize, R: Read + Seek>(
        mut reader: R,
        header: &TzDataHeader,
    ) -> Result<Self> {
        let index_len = header.data_offset.checked_sub(header.index_offset).ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, "tzdata data_offset precedes index_offset")
        })?;
        let file_len = reader.seek(SeekFrom::End(0))?;
        if u64::from(header.data_offset) > file_len {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "tzdata index region exceeds file length",
            ));
        }
        if index_len > MAX_TZDATA_INDEX_LEN {
            return Err(Error::new(ErrorKind::InvalidData, "tzdata index region exceeds 1 MiB"));
        }
        reader.seek(SeekFrom::Start(u64::from(header.index_offset)))?;
        let mut buf = vec![0; index_len as usize];
        reader.read_exact(&mut buf)?;
        // replace chunks_exact with array_chunks when it's stable. A trailing partial chunk
        // (when the index region size is not a multiple of `ENTRY_LEN`) is not a valid entry
        // and must be skipped, otherwise the slicing below reads out of bounds.
        Ok(TzDataIndexes {
            indexes: buf
                .chunks_exact(ENTRY_LEN)
                .filter_map(|chunk| {
                    from_bytes_until_nul(&chunk[..TZ_NAME_LEN]).map(|name| {
                        let name = name.to_bytes().to_vec().into_boxed_slice();
                        let offset = u32::from_be_bytes(
                            chunk[TZ_NAME_LEN..TZ_NAME_LEN + 4].try_into().unwrap(),
                        );
                        let length = u32::from_be_bytes(
                            chunk[TZ_NAME_LEN + 4..TZ_NAME_LEN + 8].try_into().unwrap(),
                        );
                        TzDataIndex { name, offset, length }
                    })
                })
                .collect(),
        })
    }

    /// Find a timezone by name.
    fn find_timezone(&self, timezone: &[u8]) -> Option<&TzDataIndex> {
        // timezones in tzdata are sorted by name.
        self.indexes.binary_search_by_key(&timezone, |x| &x.name).map(|x| &self.indexes[x]).ok()
    }

    /// Retrieve a chunk of timezone data by the index.
    fn find_tzdata(
        &self,
        mut reader: impl Read + Seek,
        header: &TzDataHeader,
        index: &TzDataIndex,
    ) -> Result<Vec<u8>> {
        let start = u64::from(index.offset) + u64::from(header.data_offset);
        let end = start + u64::from(index.length);
        let file_len = reader.seek(SeekFrom::End(0))?;
        if end > file_len {
            return Err(Error::new(ErrorKind::UnexpectedEof, "tzdata entry exceeds file length"));
        }
        if index.length > MAX_TZDATA_ENTRY_LEN {
            return Err(Error::new(ErrorKind::InvalidData, "tzdata entry exceeds 1 MiB"));
        }
        reader.seek(SeekFrom::Start(start))?;
        let mut buffer = vec![0; index.length as usize];
        reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

/// Index entry of the `tzdata` file.
struct TzDataIndex {
    name: Box<[u8]>,
    offset: u32,
    length: u32,
}

/// TODO: Change this `CStr::from_bytes_until_nul` once MSRV was bumped above 1.72.0
fn from_bytes_until_nul(bytes: &[u8]) -> Option<&CStr> {
    let nul_pos = bytes.iter().position(|&b| b == 0)?;
    // SAFETY:
    // 1. nul_pos + 1 <= bytes.len()
    // 2. We know there is a nul byte at nul_pos, so this slice (ending at the nul byte) is a well-formed C string.
    Some(unsafe { CStr::from_bytes_with_nul_unchecked(&bytes[..=nul_pos]) })
}

/// Ohos tzdata index entry size: `name + offset + length`
#[cfg(any(test, target_env = "ohos"))]
const OHOS_ENTRY_LEN: usize = TZ_NAME_LEN + 2 * size_of::<u32>();
/// Android tzdata index entry size: `name + offset + length + raw_utc_offset(legacy)`:
/// [reference](https://android.googlesource.com/platform/prebuilts/fullsdk/sources/+/refs/heads/androidx-appcompat-release/android-34/com/android/i18n/timezone/ZoneInfoDb.java#271)
#[cfg(any(test, target_os = "android"))]
const ANDROID_ENTRY_LEN: usize = TZ_NAME_LEN + 3 * size_of::<u32>();
/// The database reserves 40 bytes for each id.
const TZ_NAME_LEN: usize = 40;
/// Size of the version string in the header of `tzdata` file.
/// e.g. `tzdata2024b\0`
const TZDATA_VERSION_LEN: usize = 12;
/// Maximum size of the `tzdata` index region (1 MiB); real bundles have indexes of about 30 KB.
const MAX_TZDATA_INDEX_LEN: u32 = 1 << 20;
/// Maximum size of a single `tzdata` entry (1 MiB), matching the TZif read cap (`MAX_TZIF_LEN`,
/// 1 MiB) in `tz_info/timezone.rs`; real bundles are about 0.5 MB total.
const MAX_TZDATA_ENTRY_LEN: u32 = 1 << 20;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_tzdata_index_partial_entry() {
        // An index region whose size is not a multiple of `ENTRY_LEN` leaves a trailing
        // partial chunk that is too short to slice as an entry.
        let mut data = Vec::new();
        data.extend_from_slice(b"tzdata2024a\0");
        data.extend_from_slice(&24u32.to_be_bytes()); // index_offset
        data.extend_from_slice(&44u32.to_be_bytes()); // data_offset, index region = 20 bytes
        data.extend_from_slice(&100u32.to_be_bytes()); // zonetab_offset
        data.extend_from_slice(&[0u8; 20]);
        let mut cursor = Cursor::new(&data[..]);
        let header = TzDataHeader::new(&mut cursor).unwrap();
        let indexes = TzDataIndexes::new::<OHOS_ENTRY_LEN, _>(&mut cursor, &header).unwrap();
        assert!(indexes.indexes.is_empty());
    }

    #[test]
    fn test_tzdata_header_data_offset_precedes_index_offset() {
        let mut data = Vec::new();
        data.extend_from_slice(b"tzdata2024a\0");
        data.extend_from_slice(&40u32.to_be_bytes()); // index_offset
        data.extend_from_slice(&24u32.to_be_bytes()); // data_offset < index_offset
        data.extend_from_slice(&40u32.to_be_bytes()); // zonetab_offset
        data.extend_from_slice(&[0u8; 16]);

        for err in [
            find_tz_data::<OHOS_ENTRY_LEN, _>(Cursor::new(&data), b"UTC").unwrap_err(),
            find_tz_data::<ANDROID_ENTRY_LEN, _>(Cursor::new(&data), b"UTC").unwrap_err(),
        ] {
            assert_eq!(err.kind(), ErrorKind::InvalidData);
            assert_eq!(err.to_string(), "tzdata data_offset precedes index_offset");
        }
    }

    #[test]
    fn test_tzdata_header_oob_data_offset() {
        let mut data = Vec::new();
        data.extend_from_slice(b"tzdata2024a\0");
        data.extend_from_slice(&24u32.to_be_bytes()); // index_offset
        data.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes()); // data_offset
        data.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes()); // zonetab_offset
        assert_eq!(data.len(), 24);

        for err in [
            find_tz_data::<OHOS_ENTRY_LEN, _>(Cursor::new(&data), b"UTC").unwrap_err(),
            find_tz_data::<ANDROID_ENTRY_LEN, _>(Cursor::new(&data), b"UTC").unwrap_err(),
        ] {
            assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
            assert_eq!(err.to_string(), "tzdata index region exceeds file length");
        }
    }

    #[test]
    fn test_tzdata_entry_oob_length() {
        fn check_entry_oob<const ENTRY_LEN: usize>(trailing: &[u8]) {
            let data_offset = (24 + ENTRY_LEN) as u32;
            let mut data = Vec::new();
            data.extend_from_slice(b"tzdata2024a\0");
            data.extend_from_slice(&24u32.to_be_bytes()); // index_offset
            data.extend_from_slice(&data_offset.to_be_bytes());
            data.extend_from_slice(&data_offset.to_be_bytes()); // zonetab_offset

            let mut name = [0u8; TZ_NAME_LEN];
            name[..3].copy_from_slice(b"UTC");
            data.extend_from_slice(&name);
            data.extend_from_slice(&0u32.to_be_bytes()); // offset
            data.extend_from_slice(&0xFFFF_FFFFu32.to_be_bytes()); // length
            data.extend_from_slice(trailing);

            let err = find_tz_data::<ENTRY_LEN, _>(Cursor::new(&data), b"UTC").unwrap_err();
            assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
            assert_eq!(err.to_string(), "tzdata entry exceeds file length");
        }

        check_entry_oob::<OHOS_ENTRY_LEN>(&[]);
        check_entry_oob::<ANDROID_ENTRY_LEN>(&0u32.to_be_bytes());
    }

    #[test]
    fn test_tzdata_index_region_cap() {
        fn build(index_len: u32) -> Vec<u8> {
            let index_offset = 24u32;
            let data_offset = index_offset + index_len;
            let mut data = Vec::with_capacity(data_offset as usize);
            data.extend_from_slice(b"tzdata2024a\0");
            data.extend_from_slice(&index_offset.to_be_bytes());
            data.extend_from_slice(&data_offset.to_be_bytes());
            data.extend_from_slice(&data_offset.to_be_bytes()); // zonetab_offset
            data.resize(data_offset as usize, 0);
            data
        }

        let over = build(MAX_TZDATA_INDEX_LEN + 1);
        for err in [
            find_tz_data::<OHOS_ENTRY_LEN, _>(Cursor::new(&over), b"UTC").unwrap_err(),
            find_tz_data::<ANDROID_ENTRY_LEN, _>(Cursor::new(&over), b"UTC").unwrap_err(),
        ] {
            assert_eq!(err.kind(), ErrorKind::InvalidData);
            assert_eq!(err.to_string(), "tzdata index region exceeds 1 MiB");
        }

        let exact = build(MAX_TZDATA_INDEX_LEN);
        assert!(find_tz_data::<OHOS_ENTRY_LEN, _>(Cursor::new(&exact), b"UTC").unwrap().is_none());
        assert!(
            find_tz_data::<ANDROID_ENTRY_LEN, _>(Cursor::new(&exact), b"UTC").unwrap().is_none()
        );
    }

    #[test]
    fn test_tzdata_entry_cap() {
        fn build<const ENTRY_LEN: usize>(length: u32) -> Vec<u8> {
            let index_offset = 24u32;
            let data_offset = index_offset + ENTRY_LEN as u32;
            let total_len = data_offset as usize + length as usize;
            let mut data = Vec::with_capacity(total_len);
            data.extend_from_slice(b"tzdata2024a\0");
            data.extend_from_slice(&index_offset.to_be_bytes());
            data.extend_from_slice(&data_offset.to_be_bytes());
            data.extend_from_slice(&(total_len as u32).to_be_bytes()); // zonetab_offset

            let mut entry = vec![0u8; ENTRY_LEN];
            entry[..3].copy_from_slice(b"UTC");
            entry[TZ_NAME_LEN..TZ_NAME_LEN + 4].copy_from_slice(&0u32.to_be_bytes());
            entry[TZ_NAME_LEN + 4..TZ_NAME_LEN + 8].copy_from_slice(&length.to_be_bytes());
            data.extend_from_slice(&entry);
            data.resize(total_len, 0xAB);
            data
        }

        fn check<const ENTRY_LEN: usize>() {
            let over = build::<ENTRY_LEN>(MAX_TZDATA_ENTRY_LEN + 1);
            let err = find_tz_data::<ENTRY_LEN, _>(Cursor::new(&over), b"UTC").unwrap_err();
            assert_eq!(err.kind(), ErrorKind::InvalidData);
            assert_eq!(err.to_string(), "tzdata entry exceeds 1 MiB");

            let exact = build::<ENTRY_LEN>(MAX_TZDATA_ENTRY_LEN);
            let loaded =
                find_tz_data::<ENTRY_LEN, _>(Cursor::new(&exact), b"UTC").unwrap().unwrap();
            assert_eq!(loaded.len(), MAX_TZDATA_ENTRY_LEN as usize);
        }

        check::<OHOS_ENTRY_LEN>();
        check::<ANDROID_ENTRY_LEN>();
    }

    #[test]
    fn test_open_regular_file() {
        let dir = std::env::temp_dir();
        let err = open_regular_file(dir.to_str().unwrap()).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert_eq!(err.to_string(), "tzdata path is not a regular file");

        assert!(open_regular_file("./tests/android/tzdata").is_ok());
    }

    #[test]
    fn test_tzdata_entry_last_ends_at_eof() {
        // WET is the last entry in both fixtures and ends exactly at EOF.
        let file = File::open("./tests/ohos/tzdata").unwrap();
        let tzdata = find_tz_data::<OHOS_ENTRY_LEN, _>(file, b"WET").unwrap().unwrap();
        assert!(tzdata.starts_with(b"TZif"));
        let file = File::open("./tests/android/tzdata").unwrap();
        let tzdata = find_tz_data::<ANDROID_ENTRY_LEN, _>(file, b"WET").unwrap().unwrap();
        assert!(tzdata.starts_with(b"TZif"));
    }

    #[test]
    fn test_tzdata_fixture_content() {
        for tzdata in [
            find_tz_data::<OHOS_ENTRY_LEN, _>(
                File::open("./tests/ohos/tzdata").unwrap(),
                b"Asia/Shanghai",
            )
            .unwrap()
            .unwrap(),
            find_tz_data::<ANDROID_ENTRY_LEN, _>(
                File::open("./tests/android/tzdata").unwrap(),
                b"Asia/Shanghai",
            )
            .unwrap()
            .unwrap(),
        ] {
            assert!(tzdata.starts_with(b"TZif"));
            assert!(tzdata.ends_with(b"\nCST-8\n"));
        }
    }

    #[test]
    fn test_tzdata_entry_nonzero_offset_bounds() {
        fn build<const ENTRY_LEN: usize>(length: u32) -> Vec<u8> {
            let index_offset = 28u32; // 4 bytes of padding after the 24-byte header
            let data_offset = index_offset + ENTRY_LEN as u32;
            let mut data = Vec::new();
            data.extend_from_slice(b"tzdata2024a\0");
            data.extend_from_slice(&index_offset.to_be_bytes());
            data.extend_from_slice(&data_offset.to_be_bytes());
            data.extend_from_slice(&(data_offset + 5).to_be_bytes()); // zonetab_offset
            data.extend_from_slice(&[0xFF; 4]); // padding the index must skip
            let mut entry = vec![0u8; ENTRY_LEN];
            entry[..3].copy_from_slice(b"UTC");
            entry[TZ_NAME_LEN..TZ_NAME_LEN + 4].copy_from_slice(&2u32.to_be_bytes()); // offset
            entry[TZ_NAME_LEN + 4..TZ_NAME_LEN + 8].copy_from_slice(&length.to_be_bytes());
            data.extend_from_slice(&entry);
            data.extend_from_slice(b"xxABC"); // data region
            data
        }
        fn check<const ENTRY_LEN: usize>() {
            // offset 2 + length 3 ends exactly at EOF: accepted, exact bytes.
            let data = build::<ENTRY_LEN>(3);
            assert_eq!(
                find_tz_data::<ENTRY_LEN, _>(Cursor::new(&data), b"UTC").unwrap().unwrap(),
                b"ABC"
            );
            // offset 2 + length 4 ends one byte past EOF, although length alone fits.
            let data = build::<ENTRY_LEN>(4);
            let err = find_tz_data::<ENTRY_LEN, _>(Cursor::new(&data), b"UTC").unwrap_err();
            assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
            assert_eq!(err.to_string(), "tzdata entry exceeds file length");
        }
        check::<OHOS_ENTRY_LEN>();
        check::<ANDROID_ENTRY_LEN>();
    }

    #[test]
    fn test_ohos_tzdata_header_and_index() {
        let file = File::open("./tests/ohos/tzdata").unwrap();
        let header = TzDataHeader::new(&file).unwrap();
        assert_eq!(header.version, *b"2024a");
        assert_eq!(header.index_offset, 24);
        assert_eq!(header.data_offset, 21240);
        assert_eq!(header.zonetab_offset, 272428);

        let iter = TzDataIndexes::new::<OHOS_ENTRY_LEN, _>(&file, &header).unwrap();
        assert_eq!(iter.indexes.len(), 442);
        assert!(iter.find_timezone(b"Asia/Shanghai").is_some());
        assert!(iter.find_timezone(b"Pacific/Noumea").is_some());
    }

    #[test]
    fn test_ohos_tzdata_loading() {
        let file = File::open("./tests/ohos/tzdata").unwrap();
        let header = TzDataHeader::new(&file).unwrap();
        let iter = TzDataIndexes::new::<OHOS_ENTRY_LEN, _>(&file, &header).unwrap();
        let timezone = iter.find_timezone(b"Asia/Shanghai").unwrap();
        let tzdata = iter.find_tzdata(&file, &header, timezone).unwrap();
        assert_eq!(tzdata.len(), 393);
    }

    #[test]
    fn test_invalid_tzdata_header() {
        TzDataHeader::new(&b"tzdaaa2024aaaaaaaaaaaaaaa\0"[..]).unwrap_err();
    }

    #[test]
    fn test_android_tzdata_header_and_index() {
        let file = File::open("./tests/android/tzdata").unwrap();
        let header = TzDataHeader::new(&file).unwrap();
        assert_eq!(header.version, *b"2021a");
        assert_eq!(header.index_offset, 24);
        assert_eq!(header.data_offset, 30860);
        assert_eq!(header.zonetab_offset, 491837);

        let iter = TzDataIndexes::new::<ANDROID_ENTRY_LEN, _>(&file, &header).unwrap();
        assert_eq!(iter.indexes.len(), 593);
        assert!(iter.find_timezone(b"Asia/Shanghai").is_some());
        assert!(iter.find_timezone(b"Pacific/Noumea").is_some());
    }

    #[test]
    fn test_android_tzdata_loading() {
        let file = File::open("./tests/android/tzdata").unwrap();
        let header = TzDataHeader::new(&file).unwrap();
        let iter = TzDataIndexes::new::<ANDROID_ENTRY_LEN, _>(&file, &header).unwrap();
        let timezone = iter.find_timezone(b"Asia/Shanghai").unwrap();
        let tzdata = iter.find_tzdata(&file, &header, timezone).unwrap();
        assert_eq!(tzdata.len(), 573);
    }

    #[test]
    fn test_ohos_tzdata_find() {
        let file = File::open("./tests/ohos/tzdata").unwrap();
        let tzdata = find_tz_data::<OHOS_ENTRY_LEN, _>(file, b"Asia/Shanghai").unwrap().unwrap();
        assert_eq!(tzdata.len(), 393);
    }

    #[test]
    fn test_ohos_tzdata_find_missing() {
        let file = File::open("./tests/ohos/tzdata").unwrap();
        assert!(find_tz_data::<OHOS_ENTRY_LEN, _>(file, b"Asia/Sjasdfai").unwrap().is_none());
    }

    #[test]
    fn test_android_tzdata_find() {
        let file = File::open("./tests/android/tzdata").unwrap();
        let tzdata = find_tz_data::<ANDROID_ENTRY_LEN, _>(file, b"Asia/Shanghai").unwrap().unwrap();
        assert_eq!(tzdata.len(), 573);
    }

    #[test]
    fn test_android_tzdata_find_missing() {
        let file = File::open("./tests/android/tzdata").unwrap();
        assert!(find_tz_data::<ANDROID_ENTRY_LEN, _>(file, b"Asia/S000000i").unwrap().is_none());
    }

    #[cfg(target_env = "ohos")]
    #[test]
    fn test_ohos_machine_tz_data_loading() {
        let tzdata = for_zone("Asia/Shanghai").unwrap().unwrap();
        assert!(!tzdata.is_empty());
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_android_machine_tz_data_loading() {
        let tzdata = for_zone("Asia/Shanghai").unwrap().unwrap();
        assert!(!tzdata.is_empty());
    }
}
