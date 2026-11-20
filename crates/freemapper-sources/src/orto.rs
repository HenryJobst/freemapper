// SPDX-License-Identifier: AGPL-3.0-or-later

//! Orthophoto sheets of GUGiK: a tiled TIFF with JPEG tiles (YCbCr) in PL-1992 (EPSG:2180).
//!
//! The `tiff` crate cannot read them: the tag `JPEGTables` holds the quantisation and Huffman
//! tables three times over, each with its own start and end marker, and its decoder stops with
//! "Premature End of image". The reader here takes the tiles out of the file itself, puts one set
//! of tables in front of each and lets the JPEG decoder of `image` do the rest. Only the tiles a
//! window touches are decoded, so a schoolyard out of a 46 MB sheet costs a few tiles.

use std::io::Cursor;

use tiff::decoder::Decoder;
use tiff::tags::Tag;

use crate::crs::BoundingBox;
use crate::raster::{GridSpec, Raster};

/// The coarsest pixel the reader refuses to go below: the 0.05 m flights are 1.4 GB a sheet, so
/// nothing finer than 0.2 m is read (the plan takes the 0.25 m sheets).
const FINEST_PIXEL_M: f64 = 0.2;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OrtoError {
    #[error("not a readable TIFF: {0}")]
    Tiff(String),
    #[error("the orthophoto lacks the tag {0}")]
    MissingTag(&'static str),
    #[error("the orthophoto is not in EPSG:2180 (EPSG:{0})")]
    Crs(u32),
    #[error("the pixel of {0} m is finer than the 0.25 m orthophoto freemapper reads")]
    TooFine(f64),
    #[error("the orthophoto is not made of JPEG tiles")]
    NotJpegTiles,
    #[error("tile {0} lies outside the file")]
    Truncated(usize),
    #[error("tile {tile} cannot be decoded: {reason}")]
    Tile { tile: usize, reason: String },
}

/// An orthophoto sheet held in memory.
pub struct Sheet<'a> {
    bytes: &'a [u8],
    /// The whole sheet: north-west corner and pixel size in PL-1992.
    pub grid: GridSpec,
    tile: (u32, u32),
    offsets: Vec<u64>,
    counts: Vec<u64>,
    tables: Vec<u8>,
}

impl<'a> Sheet<'a> {
    /// Reads the tags of the sheet; no pixel is decoded yet.
    pub fn open(bytes: &'a [u8]) -> Result<Self, OrtoError> {
        let tiff = |e: tiff::TiffError| OrtoError::Tiff(e.to_string());
        let mut d = Decoder::new(Cursor::new(bytes)).map_err(tiff)?;
        let (width, height) = d.dimensions().map_err(tiff)?;
        let tag = |d: &mut Decoder<Cursor<&[u8]>>, tag: Tag, name: &'static str| {
            d.find_tag(tag)
                .map_err(tiff)?
                .ok_or(OrtoError::MissingTag(name))
        };
        if tag(&mut d, Tag::Compression, "Compression")?
            .into_u32()
            .map_err(tiff)?
            != 7
        {
            return Err(OrtoError::NotJpegTiles);
        }
        let scale = tag(&mut d, Tag::ModelPixelScaleTag, "ModelPixelScale")?
            .into_f64_vec()
            .map_err(tiff)?;
        let tie = tag(&mut d, Tag::ModelTiepointTag, "ModelTiepoint")?
            .into_f64_vec()
            .map_err(tiff)?;
        let keys = tag(&mut d, Tag::GeoKeyDirectoryTag, "GeoKeyDirectory")?
            .into_u16_vec()
            .map_err(tiff)?;
        // Entries of four values after the header of four: the key, the location, the count and
        // the value. ProjectedCSTypeGeoKey is 3072.
        let epsg = keys
            .chunks(4)
            .skip(1)
            .find(|k| k[0] == 3072)
            .map_or(0, |k| u32::from(k[3]));
        if epsg != 2180 {
            return Err(OrtoError::Crs(epsg));
        }
        let pixel = scale.first().copied().unwrap_or(0.0);
        if pixel < FINEST_PIXEL_M {
            return Err(OrtoError::TooFine(pixel));
        }
        let [i, j, _, e, n, _] = tie[..] else {
            return Err(OrtoError::MissingTag("ModelTiepoint"));
        };
        // The tie point names a raster position; (0, 0) is the north-west corner of the sheet.
        let grid = GridSpec {
            west: e - i * pixel,
            north: n + j * pixel,
            pixel_size: pixel,
            width,
            height,
        };
        let tile = (
            tag(&mut d, Tag::TileWidth, "TileWidth")?
                .into_u32()
                .map_err(tiff)?,
            tag(&mut d, Tag::TileLength, "TileLength")?
                .into_u32()
                .map_err(tiff)?,
        );
        let offsets = tag(&mut d, Tag::TileOffsets, "TileOffsets")?
            .into_u64_vec()
            .map_err(tiff)?;
        let counts = tag(&mut d, Tag::TileByteCounts, "TileByteCounts")?
            .into_u64_vec()
            .map_err(tiff)?;
        let tables = d
            .find_tag(Tag::JPEGTables)
            .map_err(tiff)?
            .map(|v| v.into_u8_vec().map_err(tiff))
            .transpose()?
            .map(|t| table_segments(&t))
            .unwrap_or_default();
        Ok(Self {
            bytes,
            grid,
            tile,
            offsets,
            counts,
            tables,
        })
    }

    /// The pixels of `window` (PL-1992), cut to whole pixels of the sheet; `None` where the
    /// window does not touch the sheet.
    pub fn read_window(&self, window: &BoundingBox) -> Result<Option<Raster<[u8; 3]>>, OrtoError> {
        let g = &self.grid;
        let px = |metres: f64| metres / g.pixel_size;
        let clamp = |v: f64, max: u32| v.clamp(0.0, f64::from(max));
        let x0 = clamp(px(window.min_e - g.west).floor(), g.width) as u32;
        let x1 = clamp(px(window.max_e - g.west).ceil(), g.width) as u32;
        let y0 = clamp(px(g.north - window.max_n).floor(), g.height) as u32;
        let y1 = clamp(px(g.north - window.min_n).ceil(), g.height) as u32;
        if x0 >= x1 || y0 >= y1 {
            return Ok(None);
        }
        let out_grid = GridSpec {
            west: g.west + f64::from(x0) * g.pixel_size,
            north: g.north - f64::from(y0) * g.pixel_size,
            pixel_size: g.pixel_size,
            width: x1 - x0,
            height: y1 - y0,
        };
        let mut out = Raster::filled(out_grid, [0u8; 3]);
        let across = g.width.div_ceil(self.tile.0);
        for ty in y0 / self.tile.1..=(y1 - 1) / self.tile.1 {
            for tx in x0 / self.tile.0..=(x1 - 1) / self.tile.0 {
                let index = (ty * across + tx) as usize;
                let image = self.decode_tile(index)?;
                // The part of the tile inside the window, in sheet pixels.
                let (tile_x, tile_y) = (tx * self.tile.0, ty * self.tile.1);
                for y in tile_y.max(y0)..(tile_y + self.tile.1).min(y1) {
                    for x in tile_x.max(x0)..(tile_x + self.tile.0).min(x1) {
                        let p = image.get_pixel(x - tile_x, y - tile_y).0;
                        out.set(x - x0, y - y0, p);
                    }
                }
            }
        }
        Ok(Some(out))
    }

    fn decode_tile(&self, index: usize) -> Result<image::RgbImage, OrtoError> {
        let (Some(&offset), Some(&count)) = (self.offsets.get(index), self.counts.get(index))
        else {
            return Err(OrtoError::Truncated(index));
        };
        let tile = usize::try_from(offset)
            .ok()
            .zip(usize::try_from(count).ok())
            .and_then(|(o, c)| self.bytes.get(o..o.checked_add(c)?))
            .filter(|t| t.starts_with(&[0xFF, 0xD8]))
            .ok_or(OrtoError::Truncated(index))?;
        let mut jpeg = vec![0xFF, 0xD8];
        jpeg.extend_from_slice(&self.tables);
        jpeg.extend_from_slice(&tile[2..]);
        image::load_from_memory_with_format(&jpeg, image::ImageFormat::Jpeg)
            .map(|i| i.to_rgb8())
            .map_err(|e| OrtoError::Tile {
                tile: index,
                reason: e.to_string(),
            })
    }
}

/// The table segments of a `JPEGTables` tag without any start or end marker, so that they can be
/// put behind one start marker in front of the scan of a tile. A tag that repeats its tables
/// repeats the segments, which the decoder takes as the same tables once more.
fn table_segments(tables: &[u8]) -> Vec<u8> {
    let (mut out, mut i) = (Vec::new(), 0);
    while i + 3 < tables.len() {
        if tables[i] != 0xFF {
            break;
        }
        if tables[i + 1] == 0xD8 || tables[i + 1] == 0xD9 {
            i += 2;
            continue;
        }
        let end = i + 2 + usize::from(u16::from_be_bytes([tables[i + 2], tables[i + 3]]));
        let Some(segment) = tables.get(i..end) else {
            break;
        };
        out.extend_from_slice(segment);
        i = end;
    }
    out
}

/// The pixels of `window` (PL-1992) of an orthophoto sheet, as RGB.
pub fn read_orto(bytes: &[u8], window: &BoundingBox) -> Result<Option<Raster<[u8; 3]>>, OrtoError> {
    Sheet::open(bytes)?.read_window(window)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TILE: u32 = 16;

    /// A sheet of 2 by 2 tiles of 16 pixels with the pixel size `pixel` and its north-west
    /// corner at (1000, 2000); every tile has its own flat colour: red, green, blue, white.
    fn fixture(pixel: f64, epsg: u16, repeat_tables: bool) -> Vec<u8> {
        let colours = [[250, 20, 20], [20, 250, 20], [20, 20, 250], [250, 250, 250]];
        let mut tiles = Vec::new();
        for c in colours {
            let img = image::RgbImage::from_pixel(TILE, TILE, image::Rgb(c));
            let mut jpeg = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 95)
                .encode_image(&img)
                .unwrap();
            tiles.push(jpeg);
        }
        // The tag the real files carry: a tables stream, here the first segments of a JPEG, in
        // front of the table segments again with an end and a start marker between.
        let tables = {
            let first = &tiles[0];
            // Only the quantisation and Huffman tables, as the tag holds them.
            let (mut segments, mut at) = (Vec::new(), 2);
            while first[at + 1] != 0xDA {
                let end = at + 2 + usize::from(u16::from_be_bytes([first[at + 2], first[at + 3]]));
                if matches!(first[at + 1], 0xDB | 0xC4) {
                    segments.extend_from_slice(&first[at..end]);
                }
                at = end;
            }
            let segments = &segments[..];
            let mut t = vec![0xFF, 0xD8];
            t.extend_from_slice(segments);
            t.extend_from_slice(&[0xFF, 0xD9]);
            if repeat_tables {
                t.extend_from_slice(&[0xFF, 0xD8]);
                t.extend_from_slice(segments);
                t.extend_from_slice(&[0xFF, 0xD9]);
            }
            t
        };
        build_tiff(&tiles, &tables, pixel, epsg)
    }

    fn build_tiff(tiles: &[Vec<u8>], tables: &[u8], pixel: f64, epsg: u16) -> Vec<u8> {
        let mut f = vec![b'I', b'I', 42, 0, 0, 0, 0, 0];
        let put = |f: &mut Vec<u8>, bytes: &[u8]| {
            let at = f.len() as u32;
            f.extend_from_slice(bytes);
            if f.len() % 2 == 1 {
                f.push(0);
            }
            at
        };
        let (mut offsets, mut counts) = (Vec::new(), Vec::new());
        for t in tiles {
            offsets.push(put(&mut f, t));
            counts.push(t.len() as u32);
        }
        let longs = |v: &[u32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let doubles = |v: &[f64]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let shorts = |v: &[u16]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let offsets_at = put(&mut f, &longs(&offsets));
        let counts_at = put(&mut f, &longs(&counts));
        let tables_at = put(&mut f, tables);
        let scale_at = put(&mut f, &doubles(&[pixel, pixel, 0.0]));
        let tie_at = put(&mut f, &doubles(&[0.0, 0.0, 0.0, 1000.0, 2000.0, 0.0]));
        let keys = [1, 1, 0, 2, 1025, 0, 1, 1, 3072, 0, 1, epsg];
        let keys_at = put(&mut f, &shorts(&keys));
        let bits_at = put(&mut f, &shorts(&[8, 8, 8]));
        // (tag, type, count, value or offset); types: 3 SHORT, 4 LONG, 12 DOUBLE, 7 UNDEFINED.
        let side = 2 * TILE;
        let entries: [(u16, u16, u32, u32); 15] = [
            (256, 4, 1, side),
            (257, 4, 1, side),
            (258, 3, 3, bits_at),
            (259, 3, 1, 7),
            (262, 3, 1, 6),
            (277, 3, 1, 3),
            (322, 4, 1, TILE),
            (323, 4, 1, TILE),
            (324, 4, 4, offsets_at),
            (325, 4, 4, counts_at),
            (347, 7, tables.len() as u32, tables_at),
            (33550, 12, 3, scale_at),
            (33922, 12, 6, tie_at),
            (34735, 3, keys.len() as u32, keys_at),
            (339, 3, 1, 1),
        ];
        let mut entries = entries.to_vec();
        entries.sort_by_key(|e| e.0);
        let ifd_at = f.len() as u32;
        f.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        for (tag, kind, count, value) in entries {
            f.extend_from_slice(&tag.to_le_bytes());
            f.extend_from_slice(&kind.to_le_bytes());
            f.extend_from_slice(&count.to_le_bytes());
            // A SHORT or LONG that fits is stored in the entry itself.
            f.extend_from_slice(&value.to_le_bytes());
        }
        f.extend_from_slice(&[0, 0, 0, 0]);
        f[4..8].copy_from_slice(&ifd_at.to_le_bytes());
        f
    }

    /// A window in PL-1992 around the pixel (x, y) of the fixture, `size` pixels a side.
    fn window(pixel: f64, x: u32, y: u32, size: u32) -> BoundingBox {
        let (w, n) = (1000.0 + f64::from(x) * pixel, 2000.0 - f64::from(y) * pixel);
        BoundingBox {
            min_e: w,
            max_e: w + f64::from(size) * pixel,
            max_n: n,
            min_n: n - f64::from(size) * pixel,
        }
    }

    fn near(a: [u8; 3], b: [u8; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| x.abs_diff(y) < 12)
    }

    #[test]
    fn tiles_with_tables_repeated_three_times_over_still_decode() {
        for repeat in [false, true] {
            let bytes = fixture(0.25, 2180, repeat);
            let r = read_orto(&bytes, &window(0.25, 0, 0, 32)).unwrap().unwrap();
            assert_eq!((r.grid.width, r.grid.height), (32, 32), "repeat {repeat}");
            assert!(near(r.get(3, 3), [250, 20, 20]), "{:?}", r.get(3, 3));
            assert!(near(r.get(20, 3), [20, 250, 20]));
            assert!(near(r.get(3, 20), [20, 20, 250]));
            assert!(near(r.get(20, 20), [250, 250, 250]));
        }
    }

    #[test]
    fn a_window_across_a_tile_seam_reads_both_tiles_in_place() {
        let bytes = fixture(0.25, 2180, true);
        let r = read_orto(&bytes, &window(0.25, 12, 3, 8)).unwrap().unwrap();
        assert_eq!(r.grid.west, 1003.0);
        assert_eq!(r.grid.north, 2000.0 - 0.75);
        assert!(near(r.get(0, 0), [250, 20, 20]));
        assert!(near(r.get(7, 0), [20, 250, 20]));
    }

    #[test]
    fn a_window_outside_the_sheet_reads_nothing_and_one_over_the_edge_is_cut() {
        let bytes = fixture(0.25, 2180, true);
        let far = BoundingBox {
            min_e: 5000.0,
            min_n: 5000.0,
            max_e: 5010.0,
            max_n: 5010.0,
        };
        assert_eq!(read_orto(&bytes, &far).unwrap(), None);
        let over = BoundingBox {
            min_e: 990.0,
            min_n: 1990.0,
            max_e: 1002.0,
            max_n: 2010.0,
        };
        let r = read_orto(&bytes, &over).unwrap().unwrap();
        assert_eq!((r.grid.width, r.grid.height), (8, 32));
    }

    #[test]
    fn the_005_metre_orthophoto_is_refused() {
        let bytes = fixture(0.05, 2180, true);
        assert!(matches!(
            Sheet::open(&bytes),
            Err(OrtoError::TooFine(p)) if (p - 0.05).abs() < 1e-9
        ));
    }

    #[test]
    fn another_grid_is_refused_with_its_code() {
        let bytes = fixture(0.25, 25833, true);
        assert!(matches!(Sheet::open(&bytes), Err(OrtoError::Crs(25833))));
    }

    #[test]
    fn the_sheet_knows_its_corner_and_pixel() {
        let bytes = fixture(0.25, 2180, true);
        let s = Sheet::open(&bytes).unwrap();
        assert_eq!(
            (s.grid.west, s.grid.north, s.grid.pixel_size),
            (1000.0, 2000.0, 0.25)
        );
        assert_eq!((s.grid.width, s.grid.height), (32, 32));
    }

    #[test]
    fn table_segments_drop_every_start_and_end_marker() {
        let one = [0xFF, 0xDB, 0, 4, 1, 2];
        let mut t = vec![0xFF, 0xD8];
        t.extend_from_slice(&one);
        t.extend_from_slice(&[0xFF, 0xD9, 0xFF, 0xD8]);
        t.extend_from_slice(&one);
        t.extend_from_slice(&[0xFF, 0xD9]);
        assert_eq!(table_segments(&t), [one, one].concat());
    }

    #[test]
    fn a_tile_outside_the_file_is_named() {
        let mut bytes = fixture(0.25, 2180, true);
        bytes.truncate(bytes.len() - 600);
        let r = Sheet::open(&bytes);
        // The tags sit at the end of the file: cutting them away fails at open, a sheet with a
        // bad tile fails at the tile. Either way no panic and no silent black.
        assert!(r.is_err() || r.unwrap().read_window(&window(0.25, 0, 0, 32)).is_err());
    }
}
