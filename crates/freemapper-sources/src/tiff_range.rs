// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tiled Float32 TIFFs read by range (ADR 0141): the layout from the IFD, then single blocks, so
//! a large file is read only where an area needs it.

use crate::fetch::{ByteRange, FetchError, Fetcher, get_range_with_retries};
use crate::float_tiff::decode_block;
use crate::raster::GridSpec;

/// Most entries an IFD may have; a DOM20 tile has 20.
const MAX_ENTRIES: u64 = 512;
/// Largest span of the arrays of an IFD read in one range; farther apart, each is read alone.
const ARRAYS_IN_ONE_RANGE: u64 = 1 << 20;
/// Largest array of an IFD read: the offsets of a million blocks.
const MAX_ARRAY_BYTES: u64 = 8 << 20;
/// Largest block read (the `tiff` crate's limit for a chunk).
const MAX_BLOCK_BYTES: u64 = 128 * 1024 * 1024;
/// Largest range of blocks read at once.
const MAX_RUN_BYTES: u64 = 16 * 1024 * 1024;

/// What the IFD of a tiled Float32 TIFF says: its size, its blocks and where they lie, its grid
/// and its GDAL metadata.
#[derive(Debug, Clone)]
pub struct TiledTiff {
    pub width: u32,
    pub height: u32,
    pub block_width: u32,
    pub block_height: u32,
    west: f64,
    north: f64,
    pixel: f64,
    metadata: Option<String>,
    offsets: Vec<u64>,
    counts: Vec<u64>,
    compression: u16,
    predictor: u16,
    little: bool,
}

impl TiledTiff {
    /// Blocks in a row.
    pub fn blocks_across(&self) -> u32 {
        self.width.div_ceil(self.block_width)
    }

    /// All blocks, row by row.
    pub fn block_count(&self) -> usize {
        self.offsets.len()
    }

    /// The grid of the image, from ModelTiepoint and ModelPixelScale.
    pub fn grid(&self) -> GridSpec {
        GridSpec {
            west: self.west,
            north: self.north,
            pixel_size: self.pixel,
            width: self.width,
            height: self.height,
        }
    }

    /// The bytes of the blocks still to read for `blocks`.
    pub fn bytes_of(&self, blocks: &[usize]) -> u64 {
        blocks.iter().filter_map(|&i| self.counts.get(i)).sum()
    }

    /// An item of the GDAL metadata (tag 42112), such as `BILDFLUG_DATUM`.
    pub fn metadata_item(&self, name: &str) -> Option<String> {
        let xml = self.metadata.as_deref()?;
        let item = &xml[xml.find(&format!("<Item name=\"{name}\""))?..];
        let value = &item[item.find('>')? + 1..];
        Some(value[..value.find("</Item>")?].trim().to_string())
    }
}

/// One IFD entry: its tag, type, count, and the four bytes of its value or offset.
struct Entry {
    tag: u16,
    kind: u16,
    count: u64,
    value: [u8; 4],
}

/// The bytes of one value of a TIFF type, 0 for an unknown type.
fn type_size(kind: u16) -> u64 {
    match kind {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => 0,
    }
}

/// The layout of the tiled TIFF at `url`, read by range: its header, the count of its first IFD,
/// its entries, then the arrays it needs (block offsets and sizes, pixel scale, tiepoint, GDAL
/// metadata), in one range if they lie within 1 MB of each other. Anything but a classic tiled
/// TIFF of one Float32 sample with LZW or Deflate is an error that names what the file has.
///
/// The outer error is a failed request; the inner one says what the file has instead, so the
/// caller can read the file another way.
pub fn read_layout(
    fetcher: &dyn Fetcher,
    url: &str,
) -> Result<Result<TiledTiff, String>, FetchError> {
    match layout(fetcher, url) {
        Ok(layout) => Ok(Ok(layout)),
        Err(LayoutError::Format(message)) => Ok(Err(message)),
        Err(LayoutError::Fetch(e)) => Err(e),
    }
}

/// Why [`layout`] failed: a request, or what the file has.
enum LayoutError {
    Fetch(FetchError),
    Format(String),
}

impl From<FetchError> for LayoutError {
    fn from(e: FetchError) -> Self {
        Self::Fetch(e)
    }
}

/// [`read_layout`] with one error type.
fn layout(fetcher: &dyn Fetcher, url: &str) -> Result<TiledTiff, LayoutError> {
    let wrong = LayoutError::Format;
    let header = get_range_with_retries(fetcher, url, ByteRange::From { start: 0, len: 8 })?;
    let little = match &header[..4] {
        b"II*\0" => true,
        b"MM\0*" => false,
        b"II+\0" | b"MM\0+" => return Err(wrong("a BigTIFF, which is not read by range".into())),
        _ => return Err(wrong("not a TIFF".into())),
    };
    let u16_at = |b: &[u8], at: usize| {
        let pair = [b[at], b[at + 1]];
        if little {
            u16::from_le_bytes(pair)
        } else {
            u16::from_be_bytes(pair)
        }
    };
    let u32_at = |b: &[u8], at: usize| {
        let word = [b[at], b[at + 1], b[at + 2], b[at + 3]];
        if little {
            u32::from_le_bytes(word)
        } else {
            u32::from_be_bytes(word)
        }
    };
    let ifd = u64::from(u32_at(&header, 4));
    let count = get_range_with_retries(fetcher, url, ByteRange::From { start: ifd, len: 2 })?;
    let n = u64::from(u16_at(&count, 0));
    if n == 0 || n > MAX_ENTRIES {
        return Err(wrong(format!("an IFD of {n} entries")));
    }
    let raw = get_range_with_retries(
        fetcher,
        url,
        ByteRange::From {
            start: ifd + 2,
            len: 12 * n,
        },
    )?;
    let entries: Vec<Entry> = raw
        .as_chunks::<12>()
        .0
        .iter()
        .map(|e| Entry {
            tag: u16_at(e, 0),
            kind: u16_at(e, 2),
            count: u64::from(u32_at(e, 4)),
            value: [e[8], e[9], e[10], e[11]],
        })
        .collect();
    let find = |tag: u16| entries.iter().find(|e| e.tag == tag);
    // A single SHORT or LONG value of an entry, or `default` without the entry.
    let single = |tag: u16, default: Option<u64>| -> Result<u64, LayoutError> {
        match find(tag) {
            Some(e) if e.count == 1 && e.kind == 3 => Ok(u64::from(u16_at(&e.value, 0))),
            Some(e) if e.count == 1 && e.kind == 4 => Ok(u64::from(u32_at(&e.value, 0))),
            Some(e) => Err(wrong(format!(
                "tag {tag} of type {} with {} values",
                e.kind, e.count
            ))),
            None => default.ok_or_else(|| wrong(format!("no tag {tag}"))),
        }
    };
    let (width, height) = (single(256, None)?, single(257, None)?);
    if find(322).is_none() || find(324).is_none() {
        return Err(wrong(
            "not tiled (strips), which is not read by range".into(),
        ));
    }
    let (block_width, block_height) = (single(322, None)?, single(323, None)?);
    let (samples, bits, format) = (
        single(277, Some(1))?,
        single(258, Some(1))?,
        single(339, Some(1))?,
    );
    if (samples, bits, format) != (1, 32, 3) {
        return Err(wrong(format!(
            "{samples} samples of {bits} bit (format {format}), not one Float32"
        )));
    }
    let compression = single(259, Some(1))?;
    if !matches!(compression, 5 | 8 | 32946) {
        return Err(wrong(format!(
            "compression {compression}, not LZW or Deflate"
        )));
    }
    if single(284, Some(1))? != 1 {
        return Err(wrong("planes stored apart".into()));
    }
    let predictor = single(317, Some(1))?;
    if !(1..=3).contains(&predictor) {
        return Err(wrong(format!("predictor {predictor}")));
    }
    let too_large = |v: u64| v == 0 || v > u64::from(u32::MAX);
    if too_large(width) || too_large(height) || too_large(block_width) || too_large(block_height) {
        return Err(wrong(format!(
            "{width} x {height} px in blocks of {block_width} x {block_height}"
        )));
    }
    // The values that lie outside their entries, read in one range where they lie close.
    let wanted = [324, 325, 33550, 33922, 42112];
    let outside: Vec<(u16, u64, u64)> = entries
        .iter()
        .filter(|e| wanted.contains(&e.tag))
        .filter_map(|e| {
            let len = type_size(e.kind).checked_mul(e.count)?;
            (len > 4).then(|| (e.tag, u64::from(u32_at(&e.value, 0)), len))
        })
        .collect();
    if let Some((tag, _, len)) = outside.iter().find(|(_, _, len)| *len > MAX_ARRAY_BYTES) {
        return Err(wrong(format!("tag {tag} with {len} bytes")));
    }
    let mut arrays: Vec<(u16, Vec<u8>)> = Vec::new();
    if let (Some(start), Some(end)) = (
        outside.iter().map(|(_, at, _)| *at).min(),
        outside.iter().map(|(_, at, len)| at + len).max(),
    ) {
        if end - start <= ARRAYS_IN_ONE_RANGE {
            let span = get_range_with_retries(
                fetcher,
                url,
                ByteRange::From {
                    start,
                    len: end - start,
                },
            )?;
            for (tag, at, len) in &outside {
                let from = (at - start) as usize;
                arrays.push((*tag, span[from..from + *len as usize].to_vec()));
            }
        } else {
            for (tag, at, len) in &outside {
                let bytes = get_range_with_retries(
                    fetcher,
                    url,
                    ByteRange::From {
                        start: *at,
                        len: *len,
                    },
                )?;
                arrays.push((*tag, bytes));
            }
        }
    }
    // The bytes of the values of `tag`: inside its entry or read above.
    let bytes_of = |tag: u16| -> Option<Vec<u8>> {
        let e = find(tag)?;
        match arrays.iter().find(|(t, _)| *t == tag) {
            Some((_, bytes)) => Some(bytes.clone()),
            None => {
                let len = (type_size(e.kind) * e.count).min(4) as usize;
                Some(e.value[..len].to_vec())
            }
        }
    };
    let integers = |tag: u16| -> Result<Vec<u64>, LayoutError> {
        let e = find(tag).ok_or_else(|| wrong(format!("no tag {tag}")))?;
        let bytes = bytes_of(tag).unwrap_or_default();
        match e.kind {
            3 => Ok(bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u64::from(u16_at(b, 0)))
                .collect()),
            4 => Ok(bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| u64::from(u32_at(b, 0)))
                .collect()),
            kind => Err(wrong(format!("tag {tag} of type {kind}"))),
        }
    };
    let doubles = |tag: u16| -> Result<Vec<f64>, LayoutError> {
        let bytes = bytes_of(tag).ok_or_else(|| wrong(format!("no tag {tag}")))?;
        Ok(bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|b| {
                if little {
                    f64::from_le_bytes(*b)
                } else {
                    f64::from_be_bytes(*b)
                }
            })
            .collect())
    };
    let (offsets, counts) = (integers(324)?, integers(325)?);
    let blocks = width.div_ceil(block_width) * height.div_ceil(block_height);
    if offsets.len() as u64 != blocks || counts.len() as u64 != blocks {
        return Err(wrong(format!(
            "{} offsets and {} byte counts for {blocks} blocks",
            offsets.len(),
            counts.len()
        )));
    }
    let (scale, tiepoint) = (doubles(33550)?, doubles(33922)?);
    let (Some(&pixel), Some(&west), Some(&north)) =
        (scale.first(), tiepoint.get(3), tiepoint.get(4))
    else {
        return Err(wrong("a GeoTIFF without tiepoint or pixel scale".into()));
    };
    // As `decode_float_tiff` does: tiepoints may carry float noise.
    let mm = |v: f64| (v * 1000.0).round() / 1000.0;
    let metadata = bytes_of(42112).map(|bytes| {
        String::from_utf8_lossy(&bytes)
            .trim_end_matches('\0')
            .to_string()
    });
    Ok(TiledTiff {
        width: width as u32,
        height: height as u32,
        block_width: block_width as u32,
        block_height: block_height as u32,
        west: mm(west),
        north: mm(north),
        pixel,
        metadata,
        offsets,
        counts,
        compression: compression as u16,
        predictor: predictor as u16,
        little,
    })
}

/// The pixels of `blocks` of the TIFF at `url` (each `block_width` × `block_height`, padding
/// included, NoData as `NaN`), sorted by block. Blocks that lie one after the other in the file
/// are read in one range.
pub fn read_blocks(
    fetcher: &dyn Fetcher,
    url: &str,
    layout: &TiledTiff,
    blocks: &[usize],
) -> Result<Vec<(usize, Vec<f32>)>, FetchError> {
    let wrong = |message: String| FetchError::Http {
        url: url.to_string(),
        message,
    };
    let mut wanted: Vec<usize> = blocks.to_vec();
    wanted.sort_unstable();
    wanted.dedup();
    if let Some(&i) = wanted.iter().find(|&&i| i >= layout.block_count()) {
        return Err(wrong(format!("no block {i} of {}", layout.block_count())));
    }
    if let Some(&i) = wanted.iter().find(|&&i| layout.counts[i] > MAX_BLOCK_BYTES) {
        return Err(wrong(format!(
            "block {i}: {} bytes are too large",
            layout.counts[i]
        )));
    }
    // Runs of blocks that follow each other in the file.
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for &i in &wanted {
        match runs.last_mut() {
            Some(run)
                if {
                    let last = *run.last().unwrap_or(&i);
                    let start = layout.offsets[run[0]];
                    layout.offsets[last] + layout.counts[last] == layout.offsets[i]
                        && layout.offsets[i] + layout.counts[i] - start <= MAX_RUN_BYTES
                } =>
            {
                run.push(i)
            }
            _ => runs.push(vec![i]),
        }
    }
    let mut out = Vec::with_capacity(wanted.len());
    for run in runs {
        let start = layout.offsets[run[0]];
        let end = run
            .iter()
            .map(|&i| layout.offsets[i] + layout.counts[i])
            .max()
            .unwrap_or(start);
        let bytes = get_range_with_retries(
            fetcher,
            url,
            ByteRange::From {
                start,
                len: end - start,
            },
        )?;
        for i in run {
            let from = (layout.offsets[i] - start) as usize;
            let chunk = &bytes[from..from + layout.counts[i] as usize];
            let values = decode_block(
                chunk,
                layout.compression,
                layout.block_width,
                layout.block_height,
                layout.predictor,
                layout.little,
            )
            .map_err(|m| wrong(format!("block {i}: {m}")))?;
            out.push((i, values));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use std::cell::RefCell;

    const URL: &str = "https://files.example.org/dom/32655_5479_20_DOM.tif";

    /// Records every range asked for.
    struct Counting<'a> {
        inner: &'a MapFetcher,
        asked: RefCell<Vec<ByteRange>>,
    }

    impl Fetcher for Counting<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.inner.get(url)
        }

        fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
            self.asked.borrow_mut().push(range);
            self.inner.get_range(url, range)
        }
    }

    /// A server that answers every range with the whole file.
    struct IgnoresRanges<'a>(&'a MapFetcher);

    impl Fetcher for IgnoresRanges<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.0.get(url)
        }

        fn get_range(&self, url: &str, _: ByteRange) -> Result<Vec<u8>, FetchError> {
            self.0.get(url)
        }
    }

    fn grid() -> GridSpec {
        GridSpec {
            west: 655_000.0,
            north: 5_480_000.0,
            pixel_size: 0.2,
            width: 40,
            height: 24,
        }
    }

    fn values(grid: &GridSpec) -> Vec<f32> {
        (0..grid.width * grid.height)
            .map(|i| 300.0 + (i % 89) as f32 * 0.5)
            .collect()
    }

    fn served(metadata: Option<&str>) -> MapFetcher {
        let fetcher = MapFetcher::default();
        let grid = grid();
        let bytes = crate::testing::tiled_float_tiff(&grid, &values(&grid), 16, 3, metadata);
        fetcher.insert(URL, bytes);
        fetcher
    }

    #[test]
    fn the_layout_is_read_from_the_end_of_the_file() {
        let metadata =
            "<GDALMetadata>\n  <Item name=\"BILDFLUG_DATUM\">13.06.2025</Item>\n</GDALMetadata>";
        let server = served(Some(metadata));
        let fetcher = Counting {
            inner: &server,
            asked: RefCell::new(Vec::new()),
        };
        let layout = read_layout(&fetcher, URL).unwrap().unwrap();
        assert_eq!((layout.width, layout.height), (40, 24));
        assert_eq!((layout.block_width, layout.block_height), (16, 16));
        assert_eq!((layout.blocks_across(), layout.block_count()), (3, 6));
        assert_eq!(layout.grid(), grid());
        assert_eq!(
            layout.metadata_item("BILDFLUG_DATUM").as_deref(),
            Some("13.06.2025")
        );
        // The header, the entry count, the entries and one range over the arrays.
        let asked = fetcher.asked.borrow();
        assert_eq!(asked.len(), 4, "{asked:?}");
        assert_eq!(asked[0], ByteRange::From { start: 0, len: 8 });
    }

    #[test]
    fn blocks_in_a_row_are_one_range() {
        let server = served(None);
        let layout = read_layout(&server, URL).unwrap().unwrap();
        let fetcher = Counting {
            inner: &server,
            asked: RefCell::new(Vec::new()),
        };
        // Blocks 0 and 1 lie one after the other; block 5 is the last of the next row.
        let blocks = read_blocks(&fetcher, URL, &layout, &[5, 1, 0]).unwrap();
        assert_eq!(fetcher.asked.borrow().len(), 2);
        let grid = grid();
        let values = values(&grid);
        assert_eq!(
            blocks.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            [0, 1, 5]
        );
        for (i, block) in &blocks {
            let (bx, by) = (*i as u32 % 3, *i as u32 / 3);
            for y in 0..16 {
                for x in 0..16 {
                    let (px, py) = (bx * 16 + x, by * 16 + y);
                    if px < grid.width && py < grid.height {
                        assert_eq!(
                            block[(y * 16 + x) as usize],
                            values[(py * grid.width + px) as usize]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn deflate_blocks_with_the_float_predictor_are_read() {
        let (grid, values) = (grid(), values(&grid()));
        let server = MapFetcher::default();
        let bytes = crate::testing::tiled_float_tiff_with(&grid, &values, 16, 3, None, 8);
        server.insert(URL, bytes);
        let layout = read_layout(&server, URL).unwrap().unwrap();
        let blocks = read_blocks(&server, URL, &layout, &[0, 4]).unwrap();
        assert_eq!(blocks.len(), 2);
        for (i, block) in &blocks {
            let (bx, by) = (*i as u32 % 3, *i as u32 / 3);
            for y in 0..16 {
                for x in 0..16 {
                    let (px, py) = (bx * 16 + x, by * 16 + y);
                    if px < grid.width && py < grid.height {
                        assert_eq!(
                            block[(y * 16 + x) as usize],
                            values[(py * grid.width + px) as usize]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_strip_tiff_is_refused() {
        let server = MapFetcher::default();
        let grid = grid();
        server.insert(URL, crate::testing::geotiff(&grid, &values(&grid)));
        let error = read_layout(&server, URL).unwrap().unwrap_err();
        assert!(error.contains("not tiled"), "{error}");
    }

    #[test]
    fn an_ignored_range_is_an_error() {
        let server = served(None);
        let error = read_layout(&IgnoresRanges(&server), URL)
            .unwrap_err()
            .to_string();
        assert!(error.contains("asked for 8 bytes from byte 0"), "{error}");
    }

    /// The offset of the IFD entry of `tag` in a little-endian classic TIFF.
    fn entry_at(bytes: &[u8], tag: u16) -> usize {
        let ifd = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let n = u16::from_le_bytes(bytes[ifd..ifd + 2].try_into().unwrap()) as usize;
        (0..n)
            .map(|i| ifd + 2 + i * 12)
            .find(|at| u16::from_le_bytes(bytes[*at..*at + 2].try_into().unwrap()) == tag)
            .unwrap_or_else(|| panic!("no tag {tag}"))
    }

    /// The test file with the inline value of `tag` set to `value`.
    fn with_value(tag: u16, value: u32) -> MapFetcher {
        let grid = grid();
        let mut bytes = crate::testing::tiled_float_tiff(&grid, &values(&grid), 16, 3, None);
        let at = entry_at(&bytes, tag);
        let kind = u16::from_le_bytes(bytes[at + 2..at + 4].try_into().unwrap());
        match kind {
            3 => bytes[at + 8..at + 10].copy_from_slice(&(value as u16).to_le_bytes()),
            4 => bytes[at + 8..at + 12].copy_from_slice(&value.to_le_bytes()),
            other => panic!("type {other}"),
        }
        let server = MapFetcher::default();
        server.insert(URL, bytes);
        server
    }

    /// The test file without the entry of `tag`: its number is one no reader knows.
    fn without(tag: u16) -> MapFetcher {
        let grid = grid();
        let mut bytes = crate::testing::tiled_float_tiff(&grid, &values(&grid), 16, 3, None);
        let at = entry_at(&bytes, tag);
        bytes[at..at + 2].copy_from_slice(&60_000u16.to_le_bytes());
        let server = MapFetcher::default();
        server.insert(URL, bytes);
        server
    }

    /// A file the reader cannot read by range says what it has instead, so the caller can read
    /// it another way; one that is no TIFF at all says that.
    #[test]
    fn a_tiff_of_another_kind_is_refused_with_what_it_has() {
        let refused = |server: MapFetcher| read_layout(&server, URL).unwrap().unwrap_err();
        let cases: [(MapFetcher, &str); 12] = [
            (with_value(259, 1), "compression 1, not LZW"),
            (with_value(317, 4), "predictor 4"),
            (with_value(317, 0), "predictor 0"),
            (with_value(284, 2), "planes stored apart"),
            (with_value(258, 16), "1 samples of 16 bit (format 3)"),
            (with_value(339, 1), "1 samples of 32 bit (format 1)"),
            (with_value(277, 3), "3 samples of 32 bit"),
            (with_value(256, 0), "0 x 24 px in blocks of 16 x 16"),
            (with_value(323, 0), "40 x 24 px in blocks of 16 x 0"),
            // Taller than the blocks of the file say: more blocks than it has offsets for.
            (
                with_value(257, 48),
                "6 offsets and 6 byte counts for 9 blocks",
            ),
            (without(33922), "no tag 33922"),
            (without(256), "no tag 256"),
        ];
        for (server, expected) in cases {
            let error = refused(server);
            assert!(error.contains(expected), "{expected}: {error}");
        }
        // A tile table of the wrong type is named by its tag and type.
        assert!(refused(with_value_kind(324, 5)).contains("tag 324 of type 5"));
        // No tag of the block size at all, or an empty IFD.
        assert!(refused(without(322)).contains("not tiled"));
        let grid = grid();
        let mut bytes = crate::testing::tiled_float_tiff(&grid, &values(&grid), 16, 3, None);
        let ifd = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        bytes[ifd..ifd + 2].copy_from_slice(&0u16.to_le_bytes());
        let server = MapFetcher::default();
        server.insert(URL, bytes.clone());
        assert!(refused(server).contains("an IFD of 0 entries"));
        // The header says what the file is.
        for (magic, expected) in [
            (&b"II+\0"[..], "BigTIFF"),
            (&b"MM\0+"[..], "BigTIFF"),
            (&b"GIF8"[..], "not a TIFF"),
        ] {
            bytes[..4].copy_from_slice(magic);
            let server = MapFetcher::default();
            server.insert(URL, bytes.clone());
            let error = refused(server);
            assert!(error.contains(expected), "{expected}: {error}");
        }
    }

    /// The test file with the type of the entry of `tag` changed to `kind`.
    fn with_value_kind(tag: u16, kind: u16) -> MapFetcher {
        let grid = grid();
        let mut bytes = crate::testing::tiled_float_tiff(&grid, &values(&grid), 16, 3, None);
        let at = entry_at(&bytes, tag);
        bytes[at + 2..at + 4].copy_from_slice(&kind.to_le_bytes());
        let server = MapFetcher::default();
        server.insert(URL, bytes);
        server
    }

    /// Blocks that do not lie one after the other are read apart; a block the file does not
    /// have and a block that does not decode are errors that name the block; the predictor of
    /// the file decides how a row is undone.
    #[test]
    fn blocks_are_read_apart_checked_and_decoded_by_the_predictor() {
        let server = served(None);
        let layout = read_layout(&server, URL).unwrap().unwrap();
        let fetcher = Counting {
            inner: &server,
            asked: RefCell::new(Vec::new()),
        };
        // Blocks 0 and 2 have block 1 between them in the file.
        let blocks = read_blocks(&fetcher, URL, &layout, &[2, 0, 2]).unwrap();
        assert_eq!(fetcher.asked.borrow().len(), 2);
        assert_eq!(blocks.iter().map(|(i, _)| *i).collect::<Vec<_>>(), [0, 2]);
        let error = read_blocks(&server, URL, &layout, &[6]).unwrap_err();
        assert!(error.to_string().contains("no block 6 of 6"), "{error}");
        // A block of garbage is no LZW stream.
        let mut bytes = server.responses.borrow()[URL].clone();
        let start = layout.offsets[3] as usize;
        let end = start + layout.counts[3] as usize;
        bytes[start..end].fill(0xFF);
        let broken = MapFetcher::default();
        broken.insert(URL, bytes);
        let error = read_blocks(&broken, URL, &layout, &[3]).unwrap_err();
        assert!(error.to_string().contains("block 3:"), "{error}");
        assert!(read_blocks(&broken, URL, &layout, &[0, 1]).is_ok());
        // The file without a predictor gives the same pixels as the one with it.
        let grid = grid();
        let plain = MapFetcher::default();
        plain.insert(
            URL,
            crate::testing::tiled_float_tiff(&grid, &values(&grid), 16, 1, None),
        );
        let plain_layout = read_layout(&plain, URL).unwrap().unwrap();
        assert_eq!(plain_layout.predictor, 1);
        // NoData is NaN, which is never equal to itself: compare the bits.
        let bits = |blocks: Vec<(usize, Vec<f32>)>| -> Vec<(usize, Vec<u32>)> {
            blocks
                .into_iter()
                .map(|(i, b)| (i, b.into_iter().map(f32::to_bits).collect()))
                .collect()
        };
        let (plain_blocks, blocks) = (
            read_blocks(&plain, URL, &plain_layout, &[0, 4]).unwrap(),
            read_blocks(&server, URL, &layout, &[0, 4]).unwrap(),
        );
        assert!(
            plain_blocks[1].1.iter().any(|z| z.is_nan()),
            "block 4 has padding"
        );
        assert_eq!(bits(plain_blocks), bits(blocks));
    }
}
