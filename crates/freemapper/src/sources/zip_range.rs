//! Reading single members out of huge zip archives by HTTP byte ranges (the central directory
//! from the archive's end, then one member's local header and data), for the statewide
//! Hamburg archive and the Hessian municipal ones.

use crate::fetch::{ByteRange, FetchError, Fetcher, get_range_with_retries};

/// Largest central directory read (LGV's are about 100 KB).
const MAX_DIRECTORY_BYTES: u64 = 16 << 20;
/// Largest member read or inflated (a 1 km tile is 4 MB).
const MAX_MEMBER_BYTES: u64 = 64 << 20;

/// One member of a zip archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Member {
    pub(crate) name: String,
    method: u16,
    compressed: u64,
    pub(crate) crc: u32,
    local_header: u64,
}

/// Where the central directory lies (offset, size) and how many members it lists, from the end
/// of a zip archive.
pub(crate) fn directory_location(tail: &[u8]) -> Result<(u64, u64, u16), String> {
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

/// The `count` members listed in a central directory; fewer is a truncated directory.
pub(crate) fn parse_directory(directory: &[u8], count: u16) -> Result<Vec<Member>, String> {
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
        members.push(Member {
            name,
            method: u16_at(directory, at + 10),
            crc: u32_at(directory, at + 16),
            compressed: u64::from(u32_at(directory, at + 20)),
            local_header: u64::from(u32_at(directory, at + 42)),
        });
        at += 46 + name_len + extra_len + comment_len;
    }
    if members.len() != usize::from(count) {
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
pub(crate) fn read_directory(fetcher: &dyn Fetcher, url: &str) -> Result<Vec<Member>, FetchError> {
    let (directory, count) = read_directory_bytes(fetcher, url)?;
    parse_directory(&directory, count).map_err(|message| refused(url, message))
}

/// The central directory of the archive at `url` and its member count (the end record lies in
/// the archive's last 64 KiB).
pub(crate) fn read_directory_bytes(
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

pub(crate) fn refused(url: &str, message: impl Into<String>) -> FetchError {
    FetchError::Http {
        url: url.to_string(),
        message: message.into(),
    }
}

/// The uncompressed content of `member`, read by two byte ranges and checked against its CRC.
pub(crate) fn read_member(
    fetcher: &dyn Fetcher,
    url: &str,
    member: &Member,
) -> Result<Vec<u8>, FetchError> {
    let too_large = || refused(url, format!("{}: member too large", member.name));
    if member.compressed > MAX_MEMBER_BYTES {
        return Err(too_large());
    }
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
    let data_start =
        member.local_header + 30 + u64::from(u16_at(&header, 26)) + u64::from(u16_at(&header, 28));
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

/// The file name of `member` without its folders (split on `/` and `\`).
fn file_name(member: &Member) -> &str {
    member.name.rsplit(['/', '\\']).next().unwrap_or_default()
}

/// The first member whose file name, lower-cased, satisfies `matches` (so archives may spell
/// names in any case; `matches` is written in lower case).
pub(crate) fn find_member(members: &[Member], matches: impl Fn(&str) -> bool) -> Option<&Member> {
    members
        .iter()
        .find(|m| matches(&file_name(m).to_lowercase()))
}

#[cfg(test)]
pub(crate) mod test_support {
    /// A zip like the state archives': deflated GeoTIFF members in folders, and a stored CSV.
    pub(crate) fn archive(members: &[(&str, Vec<u8>, bool)]) -> Vec<u8> {
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
