//! Reading single members out of huge zip archives by HTTP byte ranges (the central directory
//! from the archive's end, then one member's local header and data), for the statewide
//! Hamburg archive and the Hessian municipal ones, and for the Berlin laser archives (zip64,
//! members of hundreds of MB streamed to a file).

use std::io::{BufRead, BufReader, BufWriter, Write as _};
use std::path::Path;

use crate::fetch::{
    ByteRange, FetchError, FetchMode, Fetcher, get_range_with_retries, write_cache_file,
};
use crate::note::{Note, note};

/// Largest central directory read (LGV's are about 100 KB).
const MAX_DIRECTORY_BYTES: u64 = 16 << 20;
/// Largest member read or inflated (a 1 km tile is 4 MB).
const MAX_MEMBER_BYTES: u64 = 64 << 20;
/// Largest member streamed to a file, packed or unpacked (a Berlin laser tile has up to 1.4 GB).
const MAX_STREAMED_BYTES: u64 = 4 << 30;
/// Bytes asked for in one range while streaming a member.
const STREAM_CHUNK: u64 = 32 << 20;
/// The value of a zip field whose real value is in the zip64 records.
const ZIP64: u32 = u32::MAX;

/// One member of a zip archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    method: u16,
    compressed: u64,
    uncompressed: u64,
    pub crc: u32,
    local_header: u64,
}

impl Member {
    /// The compression method: 0 stored, 8 deflate, 9 deflate64.
    pub fn method(&self) -> u16 {
        self.method
    }

    /// The size of the member in the archive.
    pub fn compressed(&self) -> u64 {
        self.compressed
    }

    /// The size of the member unpacked.
    pub fn uncompressed(&self) -> u64 {
        self.uncompressed
    }
}

/// Where the central directory lies (offset, size) and how many members it lists, from the end
/// of a zip archive.
pub fn directory_location(tail: &[u8]) -> Result<(u64, u64, u16), String> {
    const END: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    let at = tail
        .windows(4)
        .rposition(|w| w == END)
        .ok_or("no end of central directory: not a zip archive")?;
    let record = tail
        .get(at..at + 22)
        .ok_or("truncated end of central directory")?;
    let (count, size, offset) = (u16_at(record, 10), u32_at(record, 12), u32_at(record, 16));
    if count == u16::MAX || size == u32::MAX || offset == u32::MAX {
        return Err("zip64 archives are not supported".to_string());
    }
    Ok((u64::from(offset), u64::from(size), count))
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&b[at..at + 8]);
    u64::from_le_bytes(bytes)
}

/// The `count` members listed in a central directory; fewer is a truncated directory.
pub fn parse_directory(directory: &[u8], count: u16) -> Result<Vec<Member>, String> {
    parse_entries(directory, u64::from(count))
}

/// Where the zip64 end record lies, when the end record of `tail` says its values are in the
/// zip64 records; `None` for a plain archive.
fn zip64_end_offset(tail: &[u8]) -> Result<Option<u64>, String> {
    const END: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    const LOCATOR: u32 = 0x0706_4b50;
    let at = tail
        .windows(4)
        .rposition(|w| w == END)
        .ok_or("no end of central directory: not a zip archive")?;
    let record = tail
        .get(at..at + 22)
        .ok_or("truncated end of central directory")?;
    if u16_at(record, 10) != u16::MAX && u32_at(record, 12) != ZIP64 && u32_at(record, 16) != ZIP64
    {
        return Ok(None);
    }
    let locator = at
        .checked_sub(20)
        .and_then(|from| tail.get(from..at))
        .filter(|l| u32_at(l, 0) == LOCATOR)
        .ok_or("no zip64 end of central directory locator")?;
    Ok(Some(u64_at(locator, 8)))
}

/// The central directory of the archive at `url` and its member count, also of a zip64 archive:
/// the end records lie in the archive's last 64 KiB, the zip64 end record is asked for apart.
pub fn read_members(fetcher: &dyn Fetcher, url: &str) -> Result<Vec<Member>, FetchError> {
    const ZIP64_END: u32 = 0x0606_4b50;
    let tail = get_range_with_retries(fetcher, url, ByteRange::Last(65_558))?;
    let (start, len, count) = match zip64_end_offset(&tail).map_err(|m| refused(url, m))? {
        None => {
            let (start, len, count) =
                directory_location(&tail).map_err(|message| refused(url, message))?;
            (start, len, u64::from(count))
        }
        Some(at) => {
            let record =
                get_range_with_retries(fetcher, url, ByteRange::From { start: at, len: 56 })?;
            if u32_at(&record, 0) != ZIP64_END {
                return Err(refused(url, "no zip64 end of central directory"));
            }
            (
                u64_at(&record, 48),
                u64_at(&record, 40),
                u64_at(&record, 32),
            )
        }
    };
    if len > MAX_DIRECTORY_BYTES {
        return Err(refused(
            url,
            format!("central directory too large ({len} bytes)"),
        ));
    }
    let directory = get_range_with_retries(fetcher, url, ByteRange::From { start, len })?;
    parse_entries(&directory, count).map_err(|message| refused(url, message))
}

/// The values the zip64 extra field (0x0001) of an entry gives for the fields that say
/// [`ZIP64`]: unpacked size, packed size and local header offset, in that order.
fn zip64_values(extra: &[u8], wanted: [bool; 3]) -> Result<[Option<u64>; 3], String> {
    let mut at = 0;
    while at + 4 <= extra.len() {
        let (id, len) = (u16_at(extra, at), usize::from(u16_at(extra, at + 2)));
        let data = extra
            .get(at + 4..at + 4 + len)
            .ok_or("truncated extra field")?;
        if id == 1 {
            let mut values = [None; 3];
            let mut next = 0;
            for (value, wanted) in values.iter_mut().zip(wanted) {
                if wanted {
                    let bytes = data.get(next..next + 8).ok_or("truncated zip64 field")?;
                    *value = Some(u64_at(bytes, 0));
                    next += 8;
                }
            }
            return Ok(values);
        }
        at += 4 + len;
    }
    if wanted.contains(&true) {
        return Err("no zip64 field for a zip64 member".to_string());
    }
    Ok([None; 3])
}

/// The `count` entries of a central directory; fewer is a truncated directory.
fn parse_entries(directory: &[u8], count: u64) -> Result<Vec<Member>, String> {
    const ENTRY: u32 = 0x0201_4b50;
    let mut members = Vec::new();
    let mut at = 0;
    while at < directory.len() {
        if at + 46 > directory.len() {
            return Err("truncated central directory".to_string());
        }
        if u32_at(directory, at) != ENTRY {
            return Err(format!("broken central directory at byte {at}"));
        }
        let name_len = usize::from(u16_at(directory, at + 28));
        let extra_len = usize::from(u16_at(directory, at + 30));
        let comment_len = usize::from(u16_at(directory, at + 32));
        let name = directory
            .get(at + 46..at + 46 + name_len)
            .ok_or("truncated central directory")?;
        let name = String::from_utf8_lossy(name).into_owned();
        if u16_at(directory, at + 8) & 1 != 0 {
            return Err(format!("{name}: encrypted members are not supported"));
        }
        let (compressed, uncompressed, local_header) = (
            u32_at(directory, at + 20),
            u32_at(directory, at + 24),
            u32_at(directory, at + 42),
        );
        let extra = directory
            .get(at + 46 + name_len..at + 46 + name_len + extra_len)
            .ok_or("truncated central directory")?;
        let [big_uncompressed, big_compressed, big_local_header] = zip64_values(
            extra,
            [uncompressed, compressed, local_header].map(|v| v == ZIP64),
        )
        .map_err(|message| format!("{name}: {message}"))?;
        members.push(Member {
            name,
            method: u16_at(directory, at + 10),
            crc: u32_at(directory, at + 16),
            compressed: big_compressed.unwrap_or(u64::from(compressed)),
            uncompressed: big_uncompressed.unwrap_or(u64::from(uncompressed)),
            local_header: big_local_header.unwrap_or(u64::from(local_header)),
        });
        at += 46 + name_len + extra_len + comment_len;
    }
    if members.len() as u64 != count {
        return Err(format!(
            "the central directory lists {} of {count} members",
            members.len()
        ));
    }
    Ok(members)
}

/// The members of the archive at `url`, read by two byte ranges.
// lgv and hvbg read the raw directory to cache it; only the tests parse it right away.
#[cfg(test)]
pub fn read_directory(fetcher: &dyn Fetcher, url: &str) -> Result<Vec<Member>, FetchError> {
    let (directory, count) = read_directory_bytes(fetcher, url)?;
    parse_directory(&directory, count).map_err(|message| refused(url, message))
}

/// The central directory of the archive at `url` and its member count (the end record lies in
/// the archive's last 64 KiB).
pub fn read_directory_bytes(
    fetcher: &dyn Fetcher,
    url: &str,
) -> Result<(Vec<u8>, u16), FetchError> {
    let tail = get_range_with_retries(fetcher, url, ByteRange::Last(65_558))?;
    let (start, len, count) = directory_location(&tail).map_err(|message| refused(url, message))?;
    if len > MAX_DIRECTORY_BYTES {
        return Err(refused(
            url,
            format!("central directory too large ({len} bytes)"),
        ));
    }
    let directory = get_range_with_retries(fetcher, url, ByteRange::From { start, len })?;
    Ok((directory, count))
}

pub fn refused(url: &str, message: impl Into<String>) -> FetchError {
    FetchError::Http {
        url: url.to_string(),
        message: message.into(),
    }
}

/// The uncompressed content of `member`, read by two byte ranges and checked against its CRC.
pub fn read_member(
    fetcher: &dyn Fetcher,
    url: &str,
    member: &Member,
) -> Result<Vec<u8>, FetchError> {
    let too_large = || refused(url, format!("{}: member too large", member.name));
    if member.compressed > MAX_MEMBER_BYTES {
        return Err(too_large());
    }
    let data_start = data_start(fetcher, url, member)?;
    let data = get_range_with_retries(
        fetcher,
        url,
        ByteRange::From {
            start: data_start,
            len: member.compressed,
        },
    )?;
    let bytes = match member.method {
        0 => data,
        8 => miniz_oxide::inflate::decompress_to_vec_with_limit(&data, MAX_MEMBER_BYTES as usize)
            .map_err(|e| match e.status {
            miniz_oxide::inflate::TINFLStatus::HasMoreOutput => too_large(),
            status => refused(url, format!("{}: {status:?}", member.name)),
        })?,
        method => {
            return Err(refused(
                url,
                format!("{}: unsupported compression {method}", member.name),
            ));
        }
    };
    if crc32fast::hash(&bytes) != member.crc {
        return Err(refused(
            url,
            format!("{}: CRC mismatch, the download is damaged", member.name),
        ));
    }
    Ok(bytes)
}

/// Where the data of `member` begins: after its local header, read by one byte range.
fn data_start(fetcher: &dyn Fetcher, url: &str, member: &Member) -> Result<u64, FetchError> {
    let header = get_range_with_retries(
        fetcher,
        url,
        ByteRange::From {
            start: member.local_header,
            len: 30,
        },
    )?;
    if u32_at(&header, 0) != 0x0403_4b50 {
        return Err(refused(url, format!("{}: no local header", member.name)));
    }
    Ok(member.local_header + 30 + u64::from(u16_at(&header, 26)) + u64::from(u16_at(&header, 28)))
}

/// Streams `member` of the archive at `url` unpacked into the file `path`: its packed data in
/// byte ranges of 32 MB to a partial file next to `path`, then unpacked (stored, deflate or
/// deflate64) and checked against its size and CRC. Neither file is left behind on an error.
pub fn stream_member(
    fetcher: &dyn Fetcher,
    url: &str,
    member: &Member,
    path: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<(), FetchError> {
    if member.compressed > MAX_STREAMED_BYTES || member.uncompressed > MAX_STREAMED_BYTES {
        return Err(refused(url, format!("{}: member too large", member.name)));
    }
    let packed = path.with_extension("zipped.part");
    let result = download_packed(fetcher, url, member, &packed, progress)
        .and_then(|()| unpack(url, member, &packed, path));
    let _ = std::fs::remove_file(&packed);
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}

/// The packed data of `member` into the file `packed`, by byte ranges.
fn download_packed(
    fetcher: &dyn Fetcher,
    url: &str,
    member: &Member,
    packed: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<(), FetchError> {
    let io = |e| FetchError::Io(packed.to_path_buf(), e);
    let start = data_start(fetcher, url, member)?;
    let mut file = BufWriter::new(std::fs::File::create(packed).map_err(io)?);
    let mut done = 0;
    while done < member.compressed {
        let len = STREAM_CHUNK.min(member.compressed - done);
        let bytes = get_range_with_retries(
            fetcher,
            url,
            ByteRange::From {
                start: start + done,
                len,
            },
        )?;
        file.write_all(&bytes).map_err(io)?;
        done += len;
        progress(note!(
            "progress-downloaded-of",
            megabytes = (done >> 20).to_string(),
            total = (member.compressed >> 20).to_string()
        ));
    }
    file.flush().map_err(io)
}

/// Unpacks the file `packed` with the method of `member` into `path`, checking size and CRC.
fn unpack(url: &str, member: &Member, packed: &Path, path: &Path) -> Result<(), FetchError> {
    let damaged = |what: &str| {
        refused(
            url,
            format!("{}: {what}, the download is damaged", member.name),
        )
    };
    let input = BufReader::new(
        std::fs::File::open(packed).map_err(|e| FetchError::Io(packed.to_path_buf(), e))?,
    );
    let mut output =
        BufWriter::new(std::fs::File::create(path).map_err(|e| FetchError::Io(path.into(), e))?);
    let mut crc = crc32fast::Hasher::new();
    let mut written = 0;
    let mut sink = |bytes: &[u8]| -> Result<(), FetchError> {
        written += bytes.len() as u64;
        if written > member.uncompressed {
            return Err(damaged("longer than the archive says"));
        }
        crc.update(bytes);
        output
            .write_all(bytes)
            .map_err(|e| FetchError::Io(path.to_path_buf(), e))
    };
    match member.method {
        0 => read_all(input, packed, &mut sink)?,
        8 => inflate_raw(input, packed, &mut sink).map_err(|e| match e {
            Unpacked::Broken => damaged("broken deflate data"),
            Unpacked::Failed(e) => e,
        })?,
        9 => read_all(
            deflate64::Deflate64Decoder::with_buffer(input),
            packed,
            &mut sink,
        )
        .map_err(|e| match e {
            FetchError::Io(_, e) if e.kind() == std::io::ErrorKind::InvalidData => {
                damaged("broken deflate64 data")
            }
            e => e,
        })?,
        method => {
            return Err(refused(
                url,
                format!("{}: unsupported compression {method}", member.name),
            ));
        }
    }
    output
        .flush()
        .map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
    if written != member.uncompressed {
        return Err(damaged("shorter than the archive says"));
    }
    if crc.finalize() != member.crc {
        return Err(damaged("CRC mismatch"));
    }
    Ok(())
}

/// Everything `reader` gives, in pieces, to `sink`.
fn read_all(
    mut reader: impl std::io::Read,
    packed: &Path,
    sink: &mut dyn FnMut(&[u8]) -> Result<(), FetchError>,
) -> Result<(), FetchError> {
    let mut buffer = vec![0; 1 << 20];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|e| FetchError::Io(packed.to_path_buf(), e))?;
        if read == 0 {
            return Ok(());
        }
        sink(&buffer[..read])?;
    }
}

/// Why raw deflate data could not be unpacked.
enum Unpacked {
    Broken,
    Failed(FetchError),
}

/// Raw deflate data (no zlib header) from `input`, unpacked in pieces to `sink`.
fn inflate_raw(
    mut input: impl BufRead,
    packed: &Path,
    sink: &mut dyn FnMut(&[u8]) -> Result<(), FetchError>,
) -> Result<(), Unpacked> {
    use miniz_oxide::inflate::stream::{InflateState, inflate};
    use miniz_oxide::{DataFormat, MZFlush, MZStatus};
    let mut state = InflateState::new_boxed(DataFormat::Raw);
    let mut output = vec![0; 1 << 20];
    loop {
        let available = input
            .fill_buf()
            .map_err(|e| Unpacked::Failed(FetchError::Io(packed.to_path_buf(), e)))?;
        let at_end = available.is_empty();
        let flush = if at_end {
            MZFlush::Finish
        } else {
            MZFlush::None
        };
        let result = inflate(&mut state, available, &mut output, flush);
        input.consume(result.bytes_consumed);
        sink(&output[..result.bytes_written]).map_err(Unpacked::Failed)?;
        match result.status {
            Ok(MZStatus::StreamEnd) => return Ok(()),
            Ok(_) => {}
            // No progress: more output room was needed and is there again on the next call.
            Err(miniz_oxide::MZError::Buf) if result.bytes_written > 0 => {}
            Err(_) => return Err(Unpacked::Broken),
        }
        if result.bytes_written == 0 && (at_end || result.bytes_consumed == 0) {
            return Err(Unpacked::Broken);
        }
    }
}

/// The content of `member` from the cache file `file` when its CRC matches the member's; else,
/// online, read from the archive at `url` and kept in `file`; offline `offline` is the error.
pub fn cached_member(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    url: &str,
    member: &Member,
    file: &Path,
    offline: impl FnOnce() -> FetchError,
) -> Result<Vec<u8>, FetchError> {
    match std::fs::read(file) {
        Ok(bytes) if crc32fast::hash(&bytes) == member.crc => Ok(bytes),
        _ if mode == FetchMode::Online => {
            let bytes = read_member(fetcher, url, member)?;
            write_cache_file(file, &bytes)?;
            Ok(bytes)
        }
        _ => Err(offline()),
    }
}

/// The file name of `member` without its folders (split on `/` and `\`).
fn file_name(member: &Member) -> &str {
    member.name.rsplit(['/', '\\']).next().unwrap_or_default()
}

/// The first member whose file name, lower-cased, satisfies `matches` (so archives may spell
/// names in any case; `matches` is written in lower case).
pub fn find_member(members: &[Member], matches: impl Fn(&str) -> bool) -> Option<&Member> {
    members
        .iter()
        .find(|m| matches(&file_name(m).to_lowercase()))
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    /// A zip like the state archives': deflated GeoTIFF members in folders, and a stored CSV.
    pub fn archive(members: &[(&str, Vec<u8>, bool)]) -> Vec<u8> {
        use std::io::Write as _;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes, deflate) in members {
            let method = if *deflate {
                zip::CompressionMethod::Deflated
            } else {
                zip::CompressionMethod::Stored
            };
            zip.start_file(
                *name,
                zip::write::SimpleFileOptions::default().compression_method(method),
            )
            .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::archive;
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    /// Something that looks like a TIFF, long enough to deflate.
    fn tile(_e: u32, _n: u32, _v: f32) -> Vec<u8> {
        [b"II*\0".as_slice(), &[7; 400]].concat()
    }

    /// Answers byte ranges with only their first half, like a proxy clamping them.
    struct Clamping<'a>(&'a MapFetcher);

    impl Fetcher for Clamping<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.0.get(url)
        }

        fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
            let mut bytes = self.0.get_range(url, range)?;
            if let ByteRange::From { len, .. } = range {
                bytes.truncate((len / 2) as usize);
            }
            Ok(bytes)
        }
    }

    #[test]
    fn central_directory_lists_members() {
        let zip = archive(&[
            (
                "dgm1_hh_2022/s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            ),
            (
                "dgm1_hh_2023-02-09.csv",
                b"Kachelname;Aktualitaet".to_vec(),
                false,
            ),
        ]);
        let (offset, size, count) =
            directory_location(&zip[zip.len().saturating_sub(70_000)..]).unwrap();
        let members =
            parse_directory(&zip[offset as usize..(offset + size) as usize], count).unwrap();
        let names: Vec<&str> = members.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "dgm1_hh_2022/s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                "dgm1_hh_2023-02-09.csv"
            ]
        );
    }

    #[test]
    fn member_is_inflated() {
        let csv = b"Kachelname;Aktualitaet\n".repeat(50);
        let zip = archive(&[
            (
                "a/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            ),
            ("b.csv", csv.clone(), false),
        ]);
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/dgm.zip", zip);
        let members = read_directory(&fetcher, "https://example.org/dgm.zip").unwrap();
        let csv_member = members.iter().find(|m| m.name == "b.csv").unwrap();
        assert_eq!(
            read_member(&fetcher, "https://example.org/dgm.zip", csv_member).unwrap(),
            csv
        );
        let tif = members.iter().find(|m| m.name.ends_with(".tif")).unwrap();
        let bytes = read_member(&fetcher, "https://example.org/dgm.zip", tif).unwrap();
        assert!(bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*"));
    }

    #[test]
    fn clamped_ranges_are_an_error() {
        let csv = b"Kachelname;Aktualitaet\n".repeat(50);
        let zip = archive(&[("b.csv", csv, false)]);
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/dgm.zip", zip);
        let members = read_directory(&fetcher, "https://example.org/dgm.zip").unwrap();
        let err = read_member(
            &Clamping(&fetcher),
            "https://example.org/dgm.zip",
            &members[0],
        )
        .unwrap_err();
        assert!(err.to_string().contains("bytes"), "{err}");
    }

    #[test]
    fn truncated_directory_is_an_error() {
        let zip = archive(&[
            ("a.csv", b"a".to_vec(), false),
            ("b.csv", b"b".to_vec(), false),
        ]);
        let (offset, size, count) = directory_location(&zip).unwrap();
        assert_eq!(count, 2);
        let first = 46 + "a.csv".len();
        let directory = &zip[offset as usize..(offset + size) as usize];
        let err = parse_directory(&directory[..first], count).unwrap_err();
        assert!(err.contains("1 of 2"), "{err}");
    }

    #[test]
    fn damaged_member_is_refused() {
        let csv = b"Kachelname;Aktualitaet\n".repeat(50);
        let mut zip = archive(&[("b.csv", csv, false)]);
        let at = zip.windows(10).position(|w| w == b"Kachelname").unwrap();
        zip[at] = b'X';
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/dgm.zip", zip);
        let members = read_directory(&fetcher, "https://example.org/dgm.zip").unwrap();
        let err = read_member(&fetcher, "https://example.org/dgm.zip", &members[0]).unwrap_err();
        assert!(err.to_string().contains("CRC"), "{err}");
    }

    #[test]
    fn oversized_member_is_refused() {
        let fetcher = MapFetcher::default();
        let member = Member {
            name: "huge.tif".to_string(),
            method: 8,
            compressed: 1 << 30,
            uncompressed: 1 << 30,
            crc: 0,
            local_header: 0,
        };
        let err = read_member(&fetcher, "https://example.org/dgm.zip", &member).unwrap_err();
        assert!(err.to_string().contains("too large"), "{err}");
        assert_eq!(fetcher.calls.get(), 0);
    }

    #[test]
    fn member_inflating_too_far_is_refused() {
        let zip = archive(&[("zeros.tif", vec![0; (MAX_MEMBER_BYTES + 1) as usize], true)]);
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/dgm.zip", zip);
        let members = read_directory(&fetcher, "https://example.org/dgm.zip").unwrap();
        let err = read_member(&fetcher, "https://example.org/dgm.zip", &members[0]).unwrap_err();
        assert!(err.to_string().contains("too large"), "{err}");
    }

    #[test]
    fn oversized_directory_is_refused() {
        let mut end = vec![0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 1, 0, 1, 0];
        end.extend_from_slice(&(100u32 << 20).to_le_bytes());
        end.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/dgm.zip", end);
        let err = read_directory(&fetcher, "https://example.org/dgm.zip").unwrap_err();
        assert!(err.to_string().contains("too large"), "{err}");
    }

    #[test]
    fn zip64_archive_is_refused() {
        let mut end = vec![0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0];
        end.extend_from_slice(&u32::MAX.to_le_bytes());
        end.extend_from_slice(&[0, 0]);
        let err = directory_location(&end).unwrap_err();
        assert!(err.contains("zip64"), "{err}");
    }

    #[test]
    fn encrypted_member_is_refused() {
        let zip = archive(&[("a.csv", b"a".to_vec(), false)]);
        let (offset, size, count) = directory_location(&zip).unwrap();
        let mut directory = zip[offset as usize..(offset + size) as usize].to_vec();
        directory[8] |= 1;
        let err = parse_directory(&directory, count).unwrap_err();
        assert!(err.contains("encrypted"), "{err}");
    }

    /// Bytes without long repeats: deflate codes them as literals, which deflate64 reads alike.
    fn noise(len: usize) -> Vec<u8> {
        let mut x: u32 = 0x1234_5678;
        (0..len)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                x as u8
            })
            .collect()
    }

    /// A zip whose members are all written as zip64 (sizes and offsets in the extra field, and
    /// the zip64 end records), like the Berlin laser archives.
    fn zip64_archive(members: &[(&str, Vec<u8>)]) -> Vec<u8> {
        use std::io::Write as _;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes) in members {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .large_file(true);
            zip.start_file(*name, options).unwrap();
            zip.write_all(bytes).unwrap();
        }
        let mut zip = zip.finish().unwrap().into_inner();
        // The writer ends a small archive with a plain end record; give it the zip64 ones.
        let end = zip
            .windows(4)
            .rposition(|w| w == [0x50, 0x4b, 0x05, 0x06])
            .unwrap();
        let (count, size, offset) = (
            u64::from(u16_at(&zip, end + 10)),
            u64::from(u32_at(&zip, end + 12)),
            u64::from(u32_at(&zip, end + 16)),
        );
        zip.truncate(end);
        let mut record = 0x0606_4b50u32.to_le_bytes().to_vec();
        record.extend(44u64.to_le_bytes());
        record.extend([45, 0, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        for value in [count, count, size, offset] {
            record.extend(value.to_le_bytes());
        }
        let mut locator = 0x0706_4b50u32.to_le_bytes().to_vec();
        locator.extend([0, 0, 0, 0]);
        locator.extend((end as u64).to_le_bytes());
        locator.extend(1u32.to_le_bytes());
        let mut end_record = vec![0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff];
        end_record.extend([0xff; 8]);
        end_record.extend([0, 0]);
        zip.extend(record);
        zip.extend(locator);
        zip.extend(end_record);
        zip
    }

    const LAS_URL: &str = "https://example.org/Mitte.zip";

    #[test]
    fn a_zip64_member_is_streamed_unpacked_to_a_file() {
        let tile = noise(5000);
        let zip = zip64_archive(&[
            ("3dm_33_391_5817_1_be.las", noise(10)),
            ("3dm_33_391_5818_1_be.las", tile.clone()),
        ]);
        assert!(zip64_end_offset(&zip).unwrap().is_some(), "a zip64 archive");
        let fetcher = MapFetcher::default();
        fetcher.insert(LAS_URL, zip);
        let members = read_members(&fetcher, LAS_URL).unwrap();
        let member = find_member(&members, |f| f == "3dm_33_391_5818_1_be.las").unwrap();
        assert_eq!((member.method(), member.uncompressed()), (8, 5000));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tile.las");
        let mut notes = Vec::new();
        stream_member(&fetcher, LAS_URL, member, &path, &mut |n| {
            notes.push(n.to_string())
        })
        .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), tile);
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with("downloaded 0 MB of 0 MB")),
            "{notes:?}"
        );
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(names.len(), 1, "no partial file is left");
        // The plain archives read the same way.
        let plain = archive(&[("a.csv", b"a;b".to_vec(), true)]);
        fetcher.insert("https://example.org/plain.zip", plain);
        let members = read_members(&fetcher, "https://example.org/plain.zip").unwrap();
        assert_eq!(members[0].uncompressed(), 3);
    }

    #[test]
    fn the_zip64_field_fills_only_the_values_that_say_so() {
        let mut extra = vec![0x0a, 0x00, 2, 0, 9, 9];
        extra.extend([0x01, 0x00, 16, 0]);
        extra.extend((7u64 << 32).to_le_bytes());
        extra.extend(16_637_111_163u64.to_le_bytes());
        assert_eq!(
            zip64_values(&extra, [true, false, true]),
            Ok([Some(7 << 32), None, Some(16_637_111_163)])
        );
        assert_eq!(zip64_values(&extra[..6], [false; 3]), Ok([None; 3]));
        assert!(zip64_values(&extra[..6], [false, true, false]).is_err());
        assert!(zip64_values(&extra[..20], [true, true, true]).is_err());
    }

    #[test]
    fn a_deflate64_member_is_unpacked() {
        let tile = noise(3000);
        let fetcher = MapFetcher::default();
        fetcher.insert(LAS_URL, zip64_archive(&[("t.las", tile.clone())]));
        let mut member = read_members(&fetcher, LAS_URL).unwrap().remove(0);
        member.method = 9;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tile.las");
        stream_member(&fetcher, LAS_URL, &member, &path, &mut |_| {}).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), tile);
    }

    #[test]
    fn a_damaged_streamed_member_leaves_no_file() {
        let fetcher = MapFetcher::default();
        fetcher.insert(LAS_URL, zip64_archive(&[("t.las", noise(3000))]));
        let member = read_members(&fetcher, LAS_URL).unwrap().remove(0);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tile.las");
        for (broken, why) in [
            (
                Member {
                    crc: member.crc ^ 1,
                    ..member.clone()
                },
                "CRC",
            ),
            (
                Member {
                    uncompressed: 2999,
                    ..member.clone()
                },
                "longer",
            ),
            (
                Member {
                    uncompressed: 3001,
                    ..member.clone()
                },
                "shorter",
            ),
            (
                Member {
                    method: 12,
                    ..member.clone()
                },
                "unsupported compression 12",
            ),
        ] {
            let err = stream_member(&fetcher, LAS_URL, &broken, &path, &mut |_| {}).unwrap_err();
            assert!(err.to_string().contains(why), "{err}");
            assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0, "{why}");
        }
        let err =
            stream_member(&Clamping(&fetcher), LAS_URL, &member, &path, &mut |_| {}).unwrap_err();
        assert!(err.to_string().contains("bytes"), "{err}");
        let huge = Member {
            compressed: 5 << 30,
            ..member
        };
        let err = stream_member(&fetcher, LAS_URL, &huge, &path, &mut |_| {}).unwrap_err();
        assert!(err.to_string().contains("too large"), "{err}");
    }

    #[test]
    fn member_lookup_ignores_case() {
        let zip = archive(&[
            ("S32_476/DGM1_32_476_5552_1_HE.TIF", b"a".to_vec(), false),
            ("x\\dgm1_32_448_5538_1_he.tif", b"b".to_vec(), false),
            ("dgm1_32_448_5538_1_he.csv", b"c".to_vec(), false),
        ]);
        let (offset, size, count) = directory_location(&zip).unwrap();
        let members =
            parse_directory(&zip[offset as usize..(offset + size) as usize], count).unwrap();
        let lookup = |prefix: &'static str| {
            find_member(&members, move |f| {
                f.starts_with(prefix) && f.ends_with(".tif")
            })
        };
        assert_eq!(
            lookup("dgm1_32_476_5552_").unwrap().name,
            "S32_476/DGM1_32_476_5552_1_HE.TIF"
        );
        assert_eq!(
            lookup("dgm1_32_448_5538_").unwrap().name,
            "x\\dgm1_32_448_5538_1_he.tif"
        );
        assert!(lookup("dgm1_32_449_5538_").is_none());
    }
}
