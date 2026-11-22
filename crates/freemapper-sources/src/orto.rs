// SPDX-License-Identifier: AGPL-3.0-or-later

//! Orthophoto sheets of GUGiK: a tiled TIFF with JPEG tiles (YCbCr) in PL-1992 (EPSG:2180) or,
//! for the sheets of older flights, in PL-2000 zone 5 to 8 (EPSG:2176 to 2179), written either
//! with its EPSG code or as a user-defined transverse Mercator (code 32767 with its parameters).
//!
//! The `tiff` crate cannot read them: the tag `JPEGTables` holds the quantisation and Huffman
//! tables three times over, each with its own start and end marker, and its decoder stops with
//! "Premature End of image". The reader here takes the tiles out of the file itself, puts one set
//! of tables in front of each and lets the JPEG decoder of `image` do the rest. Only the tiles a
//! window touches are decoded, so a schoolyard out of a 46 MB sheet costs a few tiles.

use std::io::Cursor;

use freemapper_georef::crs::ProjectedPoint;
use freemapper_georef::tm::{GRS80, TransverseMercator};
use tiff::decoder::Decoder;
use tiff::tags::Tag;

use crate::crs::BoundingBox;
use crate::raster::{GridSpec, Raster};

/// The coarsest pixel the reader refuses to go below: the 0.05 m flights are 1.4 GB a sheet, so
/// nothing finer than 0.2 m is read (the plan takes the 0.25 m sheets).
pub const FINEST_PIXEL_M: f64 = 0.2;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OrtoError {
    #[error("not a readable TIFF: {0}")]
    Tiff(String),
    #[error("the orthophoto lacks the tag {0}")]
    MissingTag(&'static str),
    #[error(
        "the orthophoto is in EPSG:{0}, which freemapper does not read (it reads EPSG:2180 and \
         the PL-2000 zones 5 to 8, EPSG:2176 to 2179)"
    )]
    Crs(u32),
    #[error(
        "the orthophoto is in a user-defined coordinate system ({0}) that is none of EPSG:2180 \
         and EPSG:2176 to 2179"
    )]
    UserCrs(String),
    #[error("the pixel of {0} m is finer than the 0.25 m orthophoto freemapper reads")]
    TooFine(f64),
    #[error("the pixels are not square ({0} by {1} m)")]
    NonSquare(f64, f64),
    #[error("the orthophoto is not made of JPEG tiles")]
    NotJpegTiles,
    #[error("tile {0} lies outside the file")]
    Truncated(usize),
    #[error("tile {tile} cannot be decoded: {reason}")]
    Tile { tile: usize, reason: String },
}

/// The coordinate system a sheet is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetCrs {
    /// PL-1992, EPSG:2180.
    Pl1992,
    /// PL-2000, zone 5 to 8 (central meridian 15, 18, 21 or 24 degrees): EPSG:2176 to 2179.
    Pl2000(u8),
}

/// How far a user-defined parameter may lie from the one of a known system: a millionth of a
/// degree (0.1 m on the ground), a millimetre and a scale of 1e-9.
const ANGLE_TOLERANCE_DEG: f64 = 1e-6;
const METRE_TOLERANCE: f64 = 1e-3;
const SCALE_TOLERANCE: f64 = 1e-9;

impl SheetCrs {
    /// The EPSG code.
    pub fn epsg(self) -> u32 {
        match self {
            Self::Pl1992 => 2180,
            Self::Pl2000(zone) => 2171 + u32::from(zone),
        }
    }

    /// The system of an EPSG code, if it is one of those the reader knows.
    pub fn from_epsg(epsg: u32) -> Option<Self> {
        match epsg {
            2180 => Some(Self::Pl1992),
            2176..=2179 => Some(Self::Pl2000((epsg - 2171) as u8)),
            _ => None,
        }
    }

    /// The projection on GRS80 (the WGS84 of the user-defined sheets differs by well below a
    /// millimetre here).
    pub fn projection(self) -> TransverseMercator {
        match self {
            Self::Pl1992 => TransverseMercator {
                ellipsoid: GRS80,
                lat0: 0.0,
                lon0: 19.0,
                k0: 0.9993,
                false_easting: 500_000.0,
                false_northing: -5_300_000.0,
            },
            Self::Pl2000(zone) => TransverseMercator {
                ellipsoid: GRS80,
                lat0: 0.0,
                lon0: f64::from(zone) * 3.0,
                k0: 0.999_923,
                false_easting: f64::from(zone) * 1_000_000.0 + 500_000.0,
                false_northing: 0.0,
            },
        }
    }

    /// The known system a user-defined transverse Mercator describes: central meridian, latitude
    /// of origin, false easting and northing and scale, each within a small tolerance.
    fn from_parameters(lon0: f64, lat0: f64, easting: f64, northing: f64, k0: f64) -> Option<Self> {
        [
            Self::Pl1992,
            Self::Pl2000(5),
            Self::Pl2000(6),
            Self::Pl2000(7),
            Self::Pl2000(8),
        ]
        .into_iter()
        .find(|c| {
            let p = c.projection();
            within(p.lon0, lon0, ANGLE_TOLERANCE_DEG)
                && within(p.lat0, lat0, ANGLE_TOLERANCE_DEG)
                && within(p.false_easting, easting, METRE_TOLERANCE)
                && within(p.false_northing, northing, METRE_TOLERANCE)
                && within(p.k0, k0, SCALE_TOLERANCE)
        })
    }
}

/// Whether `a` and `b` differ by no more than `tolerance`.
fn within(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

/// The value of a GeoKey stored as a double in `GeoDoubleParamsTag` (location 34736).
fn geo_double(keys: &[u16], doubles: &[f64], key: u16) -> Option<f64> {
    keys.chunks(4)
        .skip(1)
        .find(|k| k.len() == 4 && k[0] == key && k[1] == 34736)
        .and_then(|k| doubles.get(usize::from(k[3])).copied())
}

/// The coordinate system the GeoKeys name: `ProjectedCSTypeGeoKey` (3072) as an EPSG code, or,
/// when that is 32767 (user-defined), the parameters of a transverse Mercator.
fn sheet_crs(keys: &[u16], doubles: &[f64]) -> Result<SheetCrs, OrtoError> {
    let key = |id: u16| {
        keys.chunks(4)
            .skip(1)
            .find(|k| k.len() == 4 && k[0] == id && k[1] == 0)
            .map(|k| k[3])
    };
    let epsg = u32::from(key(3072).unwrap_or(0));
    if epsg != 32767 {
        return SheetCrs::from_epsg(epsg).ok_or(OrtoError::Crs(epsg));
    }
    let double = |id| geo_double(keys, doubles, id);
    // ProjCoordTransGeoKey 3075: 1 is CT_TransverseMercator. The linear unit (3076), if given,
    // must be the metre (9001).
    let metres = key(3076).is_none_or(|u| u == 9001);
    let parameters = (|| {
        Some((
            double(3080)?,
            double(3081).unwrap_or(0.0),
            double(3082)?,
            double(3083)?,
            double(3092)?,
        ))
    })();
    match (key(3075), metres, parameters) {
        (Some(1), true, Some((lon0, lat0, e, n, k0))) => SheetCrs::from_parameters(
            lon0, lat0, e, n, k0,
        )
        .ok_or_else(|| {
            OrtoError::UserCrs(format!(
                "transverse Mercator, central meridian {lon0}, latitude {lat0}, false easting {e}, \
                     false northing {n}, scale {k0}"
            ))
        }),
        (Some(ct), _, _) => Err(OrtoError::UserCrs(format!("projection method {ct}"))),
        (None, ..) => Err(OrtoError::UserCrs("no projection method".into())),
    }
}

/// An orthophoto sheet held in memory.
pub struct Sheet<'a> {
    bytes: &'a [u8],
    /// The coordinate system of the sheet.
    pub crs: SheetCrs,
    /// The whole sheet: north-west corner and pixel size in its own coordinate system
    /// ([`Sheet::crs`], PL-1992 for most).
    pub grid: GridSpec,
    tile: (u32, u32),
    offsets: Vec<u64>,
    counts: Vec<u64>,
    tables: Vec<u8>,
}

/// How far the two pixel sizes of a sheet may differ, in metres (2^-30, exact in a float).
const SQUARE_TOLERANCE_M: f64 = 1.0 / 1_073_741_824.0;

/// Whether the pixel sizes `x` and `y` make a square pixel.
fn is_square(x: f64, y: f64) -> bool {
    (x - y).abs() <= SQUARE_TOLERANCE_M
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
        let doubles = d
            .find_tag(Tag::GeoDoubleParamsTag)
            .map_err(tiff)?
            .map(|v| v.into_f64_vec().map_err(tiff))
            .transpose()?
            .unwrap_or_default();
        // Entries of four values after the header of four: the key, the location, the count and
        // the value. ProjectedCSTypeGeoKey is 3072.
        let crs = sheet_crs(&keys, &doubles)?;
        let [pixel, pixel_y, ..] = scale[..] else {
            return Err(OrtoError::MissingTag("ModelPixelScale"));
        };
        if pixel.is_nan() || pixel < FINEST_PIXEL_M {
            return Err(OrtoError::TooFine(pixel));
        }
        if !is_square(pixel, pixel_y) {
            return Err(OrtoError::NonSquare(pixel, pixel_y));
        }
        let [i, j, _, e, n, _, ..] = tie[..] else {
            return Err(OrtoError::MissingTag("ModelTiepoint"));
        };
        // GTRasterTypeGeoKey (1025): 2 is PixelIsPoint, where the tie point names the centre of a
        // pixel and not its north-west corner.
        let half = if keys.chunks(4).skip(1).any(|k| k[0] == 1025 && k[3] == 2) {
            0.5
        } else {
            0.0
        };
        // The tie point names a raster position; (0, 0) is the north-west corner of the sheet.
        let grid = GridSpec {
            west: e - (i + half) * pixel,
            north: n + (j + half) * pixel,
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
            crs,
            grid,
            tile,
            offsets,
            counts,
            tables,
        })
    }

    /// `area` (PL-1992) as a window in the coordinate system of the sheet: the area itself for a
    /// PL-1992 sheet, otherwise the box around its projected edges.
    pub fn window_for(&self, area: &BoundingBox) -> BoundingBox {
        if self.crs == SheetCrs::Pl1992 {
            return *area;
        }
        let (from, to) = (SheetCrs::Pl1992.projection(), self.crs.projection());
        const STEPS: u32 = 16;
        let mut out: Option<BoundingBox> = None;
        for i in 0..=STEPS {
            for j in 0..=STEPS {
                let (fi, fj) = (
                    f64::from(i) / f64::from(STEPS),
                    f64::from(j) / f64::from(STEPS),
                );
                let p = ProjectedPoint {
                    easting: area.min_e + fi * (area.max_e - area.min_e),
                    northing: area.min_n + fj * (area.max_n - area.min_n),
                };
                let (lat, lon) = from.inverse(p.easting, p.northing);
                let (e, n) = to.forward(lat, lon);
                let b = out.get_or_insert(BoundingBox {
                    min_e: e,
                    min_n: n,
                    max_e: e,
                    max_n: n,
                });
                b.min_e = b.min_e.min(e);
                b.max_e = b.max_e.max(e);
                b.min_n = b.min_n.min(n);
                b.max_n = b.max_n.max(n);
            }
        }
        out.unwrap_or(*area)
    }

    /// The position of a point of the sheet grid in PL-1992 (for tests and callers that need it).
    pub fn to_pl1992(&self, p: ProjectedPoint) -> ProjectedPoint {
        if self.crs == SheetCrs::Pl1992 {
            return p;
        }
        let (lat, lon) = self.crs.projection().inverse(p.easting, p.northing);
        let (easting, northing) = SheetCrs::Pl1992.projection().forward(lat, lon);
        ProjectedPoint { easting, northing }
    }

    /// The pixels of `window` in the coordinate system of the sheet (see [`Sheet::window_for`]),
    /// cut to whole pixels of the sheet; `None` where the window does not touch the sheet. The
    /// raster is in the grid of the sheet, so a sheet that is not in PL-1992 has to be warped by
    /// its [`Sheet::crs`].
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
        let broken = |reason: String| OrtoError::Tile {
            tile: index,
            reason,
        };
        let decoder = image::codecs::jpeg::JpegDecoder::new(Cursor::new(&jpeg[..]))
            .map_err(|e| broken(e.to_string()))?;
        // The size is read before the pixels, so a damaged tile cannot ask for gigabytes.
        let (width, height) = image::ImageDecoder::dimensions(&decoder);
        if width > self.tile.0 || height > self.tile.1 {
            return Err(broken(format!("{width} by {height}, larger than declared")));
        }
        let image = image::DynamicImage::from_decoder(decoder)
            .map_err(|e| broken(e.to_string()))?
            .to_rgb8();
        // A damaged tile can decode to less than the file declares; reading it would panic.
        if image.width() < self.tile.0 || image.height() < self.tile.1 {
            return Err(OrtoError::Tile {
                tile: index,
                reason: format!("decoded to {} by {}", image.width(), image.height()),
            });
        }
        Ok(image)
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

/// A synthetic sheet like the ones of GUGiK (2 by 2 tiles of 16 pixels with a flat colour each:
/// red, green, blue, white), for the tests of this crate and of the adapters that read it.
#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    pub const TILE: u32 = 16;

    /// A sheet of 2 by 2 tiles of 16 pixels with the pixel size `pixel` and its north-west
    /// corner at (1000, 2000); every tile has its own flat colour: red, green, blue, white.
    pub fn fixture(pixel: f64, epsg: u16, repeat_tables: bool) -> Vec<u8> {
        fixture_at(pixel, epsg, repeat_tables, (0.0, 0.0), (1000.0, 2000.0))
    }

    /// Like [`fixture`], with the tie point at raster position `tie_ij` instead of (0, 0), where
    /// the sheet lies at `origin` (easting, northing) in PL-1992.
    pub fn fixture_at(
        pixel: f64,
        epsg: u16,
        repeat_tables: bool,
        tie_ij: (f64, f64),
        origin: (f64, f64),
    ) -> Vec<u8> {
        let (tiles, tables) = parts(repeat_tables);
        let keys = [1, 1, 0, 2, 1025, 0, 1, 1, 3072, 0, 1, epsg];
        build_tiff(&tiles, &tables, pixel, &keys, &[], tie_ij, origin)
    }

    /// Like [`fixture`], with the GeoKey directory `keys` and the GeoDoubleParams `doubles` as
    /// given and the north-west corner at `origin`, for the sheets that define their coordinate
    /// system themselves.
    pub fn fixture_with_keys(
        pixel: f64,
        keys: &[u16],
        doubles: &[f64],
        origin: (f64, f64),
    ) -> Vec<u8> {
        let (tiles, tables) = parts(true);
        build_tiff(&tiles, &tables, pixel, keys, doubles, (0.0, 0.0), origin)
    }

    /// The four JPEG tiles and the tables stream of the fixtures.
    fn parts(repeat_tables: bool) -> (Vec<Vec<u8>>, Vec<u8>) {
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
        (tiles, tables)
    }

    fn build_tiff(
        tiles: &[Vec<u8>],
        tables: &[u8],
        pixel: f64,
        keys: &[u16],
        doubles: &[f64],
        tie_ij: (f64, f64),
        origin: (f64, f64),
    ) -> Vec<u8> {
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
        let doubles_bytes = |v: &[f64]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let shorts = |v: &[u16]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let offsets_at = put(&mut f, &longs(&offsets));
        let counts_at = put(&mut f, &longs(&counts));
        let tables_at = put(&mut f, tables);
        let scale_at = put(&mut f, &doubles_bytes(&[pixel, pixel, 0.0]));
        let tie_at = put(
            &mut f,
            &doubles_bytes(&[tie_ij.0, tie_ij.1, 0.0, origin.0, origin.1, 0.0]),
        );
        let keys_at = put(&mut f, &shorts(keys));
        let doubles_at = put(&mut f, &doubles_bytes(doubles));
        let bits_at = put(&mut f, &shorts(&[8, 8, 8]));
        // (tag, type, count, value or offset); types: 3 SHORT, 4 LONG, 12 DOUBLE, 7 UNDEFINED.
        let side = 2 * TILE;
        let mut entries: Vec<(u16, u16, u32, u32)> = vec![
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
        if !doubles.is_empty() {
            entries.push((34736, 12, doubles.len() as u32, doubles_at));
        }
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
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

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
        assert_eq!((r.grid.width, r.grid.height), (8, 8));
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

    /// The GeoKeys of a sheet of GUGiK that defines PL-2000 zone 5 itself (the header of
    /// `5.176.17.3`, 2011): user-defined (32767), transverse Mercator, metres, and the five
    /// parameters in the GeoDoubleParams (central meridian, latitude of origin, false easting,
    /// false northing, scale).
    fn user_keys() -> Vec<u16> {
        let entries: [[u16; 4]; 12] = [
            [1024, 0, 1, 1],
            [1025, 0, 1, 1],
            [2048, 0, 1, 4326],
            [3072, 0, 1, 32767],
            [3074, 0, 1, 32767],
            [3075, 0, 1, 1],
            [3076, 0, 1, 9001],
            [3080, 34736, 1, 0],
            [3081, 34736, 1, 1],
            [3082, 34736, 1, 2],
            [3083, 34736, 1, 3],
            [3092, 34736, 1, 4],
        ];
        let mut keys = vec![1, 1, 0, 12];
        keys.extend(entries.iter().flatten());
        keys
    }

    const ZONE5: [f64; 5] = [15.0, 0.0, 5_500_000.0, 0.0, 0.999_923];

    #[test]
    fn the_pl2000_zones_are_read_by_their_epsg_codes() {
        for (epsg, zone) in [(2176, 5), (2177, 6), (2178, 7), (2179, 8)] {
            let bytes = fixture(0.25, epsg, true);
            let s = Sheet::open(&bytes).unwrap();
            assert_eq!(s.crs, SheetCrs::Pl2000(zone));
            assert_eq!(s.crs.epsg(), u32::from(epsg));
        }
        let bytes = fixture(0.25, 2180, true);
        assert_eq!(Sheet::open(&bytes).unwrap().crs, SheetCrs::Pl1992);
    }

    #[test]
    fn a_user_defined_transverse_mercator_of_pl2000_is_read() {
        let bytes = fixture_with_keys(0.5, &user_keys(), &ZONE5, (5_468_000.0, 5_802_500.0));
        let s = Sheet::open(&bytes).unwrap();
        assert_eq!(s.crs, SheetCrs::Pl2000(5));
        assert_eq!((s.grid.west, s.grid.north), (5_468_000.0, 5_802_500.0));
        // Within the tolerance of the parameters, outside it, and with an unknown method.
        let near = [15.0 + 5e-7, 0.0, 5_500_000.0 + 5e-4, 0.0, 0.999_923];
        let bytes = fixture_with_keys(0.5, &user_keys(), &near, (0.0, 0.0));
        assert_eq!(Sheet::open(&bytes).unwrap().crs, SheetCrs::Pl2000(5));
        let far = [15.0, 0.0, 5_500_001.0, 0.0, 0.999_923];
        let bytes = fixture_with_keys(0.5, &user_keys(), &far, (0.0, 0.0));
        assert!(matches!(Sheet::open(&bytes), Err(OrtoError::UserCrs(_))));
        let mut utm = user_keys();
        utm[27] = 32; // another projection method
        let bytes = fixture_with_keys(0.5, &utm, &ZONE5, (0.0, 0.0));
        let Err(OrtoError::UserCrs(why)) = Sheet::open(&bytes) else {
            panic!("a method other than transverse Mercator is refused");
        };
        assert!(why.contains("method 32"), "{why}");
        // Feet are not metres.
        let mut feet = user_keys();
        feet[31] = 9002;
        let bytes = fixture_with_keys(0.5, &feet, &ZONE5, (0.0, 0.0));
        assert!(matches!(Sheet::open(&bytes), Err(OrtoError::UserCrs(_))));
    }

    #[test]
    fn parameters_are_compared_within_the_tolerance_in_both_directions() {
        // [lon0, lat0, easting, northing, k0] of PL-2000 zone 7 and of PL-1992.
        let zone7 = [21.0, 0.0, 7_500_000.0, 0.0, 0.999_923];
        let pl1992 = [19.0, 0.0, 500_000.0, -5_300_000.0, 0.9993];
        let steps = [
            ANGLE_TOLERANCE_DEG,
            ANGLE_TOLERANCE_DEG,
            METRE_TOLERANCE,
            METRE_TOLERANCE,
            SCALE_TOLERANCE,
        ];
        let find = |v: [f64; 5]| SheetCrs::from_parameters(v[0], v[1], v[2], v[3], v[4]);
        assert_eq!(find(zone7), Some(SheetCrs::Pl2000(7)));
        assert_eq!(find(pl1992), Some(SheetCrs::Pl1992));
        for (known, crs) in [(zone7, SheetCrs::Pl2000(7)), (pl1992, SheetCrs::Pl1992)] {
            for i in 0..5 {
                for sign in [-1.0, 1.0] {
                    let mut inside = known;
                    inside[i] += sign * 0.5 * steps[i];
                    assert_eq!(find(inside), Some(crs), "inside {i} {sign}");
                    let mut outside = known;
                    outside[i] += sign * 2.0 * steps[i];
                    assert_eq!(find(outside), None, "outside {i} {sign}");
                }
            }
        }
        assert!(within(100.0, 100.4, 0.5) && within(100.4, 100.0, 0.5));
        assert!(!within(100.0, 100.6, 0.5) && !within(100.6, 100.0, 0.5));
    }

    #[test]
    fn the_projections_agree_with_proj() {
        // Reference values of cs2cs (PROJ 9, GRS80, no datum shift) for the point of PL-1992
        // E 197747.0 N 507596.0 (Slubice) and one east of it.
        let at = SheetCrs::Pl1992.projection();
        let (lat, lon) = at.inverse(197_747.0, 507_596.0);
        assert!((lat - 52.350_797_228).abs() < 1e-8 && (lon - 14.559_993_446).abs() < 1e-8);
        for (zone, e, n) in [
            (5, 5_470_020.383_4, 5_802_021.358_9),
            (6, 6_265_652.053_2, 5_807_503.265_0),
            (7, 7_061_447.877_2, 5_821_480.513_6),
            (8, 7_857_556.739_2, 5_844_000.518_7),
        ] {
            let (x, y) = SheetCrs::Pl2000(zone).projection().forward(lat, lon);
            assert!(
                (x - e).abs() < 0.001 && (y - n).abs() < 0.001,
                "zone {zone}: {x} {y}"
            );
        }
        // The PL-1992 here is the one of the core.
        let ll = freemapper_georef::crs::LatLon { lat, lon };
        let core = freemapper_georef::crs::Pl1992::forward(ll);
        assert!(
            (core.easting - 197_747.0).abs() < 0.001 && (core.northing - 507_596.0).abs() < 0.001
        );
    }

    #[test]
    fn a_window_of_a_pl2000_sheet_is_the_box_of_its_projected_area() {
        let bytes = fixture_with_keys(0.5, &user_keys(), &ZONE5, (5_470_000.0, 5_802_100.0));
        let s = Sheet::open(&bytes).unwrap();
        let area = BoundingBox {
            min_e: 197_700.0,
            min_n: 507_550.0,
            max_e: 197_800.0,
            max_n: 507_650.0,
        };
        let w = s.window_for(&area);
        // PL-1992 is turned by about 3 degrees against zone 5 here, so the box is wider than 100 m.
        assert!(w.width() > 100.0 && w.width() < 110.0, "{w:?}");
        assert!((w.min_e - 5_470_000.0).abs() < 100.0 && (w.min_n - 5_802_000.0).abs() < 100.0);
        // A PL-1992 sheet keeps the window as it is.
        let plain = fixture(0.25, 2180, true);
        assert_eq!(Sheet::open(&plain).unwrap().window_for(&area), area);
        // The corner of the window comes back to its PL-1992 position.
        let back = s.to_pl1992(ProjectedPoint {
            easting: 5_470_020.383_4,
            northing: 5_802_021.358_9,
        });
        assert!(
            (back.easting - 197_747.0).abs() < 0.001 && (back.northing - 507_596.0).abs() < 0.001
        );
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

    /// Sets the value of the one-entry LONG tag `tag` in the IFD of the fixture.
    fn patch_long(bytes: &mut [u8], tag: u16, value: u32) {
        let mut key = tag.to_le_bytes().to_vec();
        key.extend_from_slice(&[4, 0, 1, 0, 0, 0]);
        let at = bytes.windows(8).rposition(|w| w == key).unwrap() + 8;
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn a_tile_size_of_zero_is_refused_by_the_tiff_reader_before_any_division() {
        // `read_window` divides by the tile size; the `tiff` crate never lets a zero through.
        for tag in [322, 323] {
            let mut bytes = fixture(0.25, 2180, true);
            patch_long(&mut bytes, tag, 0);
            assert!(matches!(Sheet::open(&bytes), Err(OrtoError::Tiff(_))));
        }
    }

    #[test]
    fn a_tile_that_decodes_smaller_than_declared_is_an_error_and_not_a_panic() {
        // The file declares tiles of (32, 16) or (16, 32), the JPEGs hold 16 by 16.
        for (tw, th) in [(32, 16), (16, 32)] {
            let mut bytes = fixture(0.25, 2180, true);
            patch_long(&mut bytes, 322, tw);
            patch_long(&mut bytes, 323, th);
            for tag in [324u16, 325] {
                let mut key = tag.to_le_bytes().to_vec();
                key.extend_from_slice(&[4, 0, 4, 0, 0, 0]);
                let at = bytes.windows(8).rposition(|w| w == key).unwrap() + 4;
                bytes[at..at + 4].copy_from_slice(&2u32.to_le_bytes());
            }
            let r = Sheet::open(&bytes).and_then(|s| s.read_window(&window(0.25, 0, 0, 32)));
            assert!(
                matches!(r, Err(OrtoError::Tile { .. } | OrtoError::Truncated(_))),
                "{tw} by {th}: {r:?}"
            );
        }
    }

    #[test]
    fn the_tie_point_may_name_a_raster_position_other_than_the_corner() {
        // The point (1000, 2000) lies at pixel (4, 8): the corner is 1 m west and 2 m north.
        let bytes = fixture_at(0.25, 2180, true, (4.0, 8.0), (1000.0, 2000.0));
        let g = Sheet::open(&bytes).unwrap().grid;
        assert_eq!((g.west, g.north), (999.0, 2002.0));
    }

    #[test]
    fn a_pixel_of_exactly_the_finest_size_is_read() {
        let bytes = fixture(FINEST_PIXEL_M, 2180, true);
        assert!(Sheet::open(&bytes).is_ok());
    }

    #[test]
    fn a_window_beside_the_sheet_reads_nothing_even_when_it_overlaps_in_one_direction() {
        let bytes = fixture(0.25, 2180, true);
        // East of the sheet, but in the rows of the sheet.
        let east = BoundingBox {
            min_e: 1100.0,
            min_n: 1995.0,
            max_e: 1110.0,
            max_n: 2000.0,
        };
        assert_eq!(read_orto(&bytes, &east).unwrap(), None);
        // In the columns of the sheet, but south of it.
        let south = BoundingBox {
            min_e: 1000.0,
            min_n: 1900.0,
            max_e: 1004.0,
            max_n: 1910.0,
        };
        assert_eq!(read_orto(&bytes, &south).unwrap(), None);
    }

    #[test]
    fn a_table_stream_that_ends_inside_a_marker_gives_no_tables() {
        assert!(table_segments(&[0xFF, 0xFF, 0xFF]).is_empty());
    }

    #[test]
    fn a_tile_larger_than_declared_in_either_direction_is_refused_before_it_is_decoded() {
        // Two tiles of (tw, th) side by side; the JPEGs hold 16 by 16.
        for (tw, th) in [(8, 8), (8, 16), (16, 8)] {
            let mut bytes = fixture(0.25, 2180, true);
            for (tag, value) in [(256, 2 * tw), (257, th), (322, tw), (323, th)] {
                patch_long(&mut bytes, tag, value);
            }
            for tag in [324u16, 325] {
                let mut key = tag.to_le_bytes().to_vec();
                key.extend_from_slice(&[4, 0, 4, 0, 0, 0]);
                let at = bytes.windows(8).rposition(|w| w == key).unwrap() + 4;
                bytes[at..at + 4].copy_from_slice(&2u32.to_le_bytes());
            }
            let r = Sheet::open(&bytes).and_then(|s| s.read_window(&window(0.25, 0, 0, 8)));
            assert!(
                matches!(&r, Err(OrtoError::Tile { reason, .. }) if reason.contains("larger")),
                "{tw} by {th}: {r:?}"
            );
        }
    }

    #[test]
    fn pixel_sizes_that_differ_by_no_more_than_the_tolerance_make_a_square() {
        assert!(is_square(0.25, 0.25));
        assert!(is_square(1.0, 1.0 + SQUARE_TOLERANCE_M));
        assert!(!is_square(1.0, 1.0 + 2.0 * SQUARE_TOLERANCE_M));
    }

    /// Writes `value` over the 8 bytes at the first place `needle` occurs in `bytes`.
    fn patch_bytes(bytes: &mut [u8], needle: &[u8], value: &[u8]) {
        let at = bytes
            .windows(needle.len())
            .position(|w| w == needle)
            .unwrap();
        bytes[at..at + value.len()].copy_from_slice(value);
    }

    #[test]
    fn pixels_that_are_not_square_are_refused() {
        let mut bytes = fixture(0.25, 2180, true);
        let scale: Vec<u8> = [0.25f64, 0.25, 0.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let wrong: Vec<u8> = [0.25f64, 0.5, 0.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        patch_bytes(&mut bytes, &scale, &wrong);
        assert!(matches!(
            Sheet::open(&bytes),
            Err(OrtoError::NonSquare(x, y)) if x == 0.25 && y == 0.5
        ));
    }

    #[test]
    fn a_pixel_is_point_sheet_has_its_corner_half_a_pixel_further_out() {
        let mut bytes = fixture_at(0.25, 2180, true, (4.0, 8.0), (1000.0, 2000.0));
        // GTRasterTypeGeoKey 1025: the value 1 (PixelIsArea) becomes 2 (PixelIsPoint).
        let key: Vec<u8> = [1025u16, 0, 1, 1]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let point: Vec<u8> = [1025u16, 0, 1, 2]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        patch_bytes(&mut bytes, &key, &point);
        let g = Sheet::open(&bytes).unwrap().grid;
        // Pixel (4, 8) has its centre on (1000, 2000): its corner is 4.5 and 8.5 pixels away.
        assert_eq!(
            (g.west, g.north),
            (1000.0 - 4.5 * 0.25, 2000.0 + 8.5 * 0.25)
        );
    }
}
