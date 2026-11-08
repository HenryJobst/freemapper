//! LVermGeo Saxony-Anhalt open data (dl-de/by-2-0, "© GeoBasis-DE / LVermGeo ST"), all in
//! EPSG:25832: the orthophoto WMS, DGM1 and DOM1 from the WCS, and flight dates from the tile
//! index WFS.

use std::path::Path;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, Fetcher, get_with_retries, write_atomically};
use crate::raster::{GridSpec, Raster};
use crate::sources::wms::WmsService;

/// The orthophoto WMS (TrueDOP20, newest flight).
pub const WMS: WmsService = WmsService {
    url: "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest",
    layer: "lsa_lvermgeo_dop20_2",
    crs: Crs::Utm32,
    extra: "",
    max_px: 4000,
};
/// The tile index with flight dates (ArcGIS WFS, GeoJSON output).
pub const INDEX_URL: &str = "https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis";

/// A height model served by the WCS at 1 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain.
    Dgm1,
    /// DOM1, the (image-based) surface.
    Dom1,
}

impl Model {
    pub fn wcs_endpoint(self) -> &'static str {
        match self {
            Self::Dgm1 => "https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1",
            Self::Dom1 => "https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LVermGeo DGM1)",
            Self::Dom1 => "surface model (LVermGeo DOM1)",
        }
    }
}

/// GetCoverage of `model` over `bbox` (EPSG:25832), rounded outward to whole metres.
pub fn wcs_url(model: Model, bbox: &BoundingBox) -> String {
    format!(
        "{}?SERVICE=WCS&VERSION=2.0.1&REQUEST=GetCoverage&COVERAGEID=Coverage1&FORMAT=image/tiff\
         &SUBSET=x({},{})&SUBSET=y({},{})",
        model.wcs_endpoint(),
        bbox.min_e.floor(),
        bbox.max_e.ceil(),
        bbox.min_n.floor(),
        bbox.max_n.ceil()
    )
}

/// The TIFF part of a WCS `multipart/related` answer.
pub fn multipart_tiff(bytes: &[u8]) -> Result<&[u8], String> {
    let refused = || format!("no TIFF in the WCS answer: {}", exception_text(bytes));
    // The first line is the delimiter, e.g. `--wcs`.
    let first_line_end = bytes.iter().position(|&b| b == b'\n').ok_or_else(refused)?;
    let delimiter = bytes[..first_line_end]
        .strip_suffix(b"\r")
        .unwrap_or(&bytes[..first_line_end]);
    if !delimiter.starts_with(b"--") {
        return Err(refused());
    }
    let mut rest = &bytes[first_line_end + 1..];
    loop {
        // Headers end at an empty line.
        let (header_end, body_start) = find(rest, b"\r\n\r\n")
            .map(|i| (i, i + 4))
            .or_else(|| find(rest, b"\n\n").map(|i| (i, i + 2)))
            .ok_or_else(refused)?;
        let headers = String::from_utf8_lossy(&rest[..header_end]).to_ascii_lowercase();
        let body = &rest[body_start..];
        let mut marker = b"\n".to_vec();
        marker.extend_from_slice(delimiter);
        let body_end = find(body, &marker).ok_or_else(refused)?;
        let part = body[..body_end]
            .strip_suffix(b"\r")
            .unwrap_or(&body[..body_end]);
        if headers.contains("content-type: image/tiff") {
            return Ok(part);
        }
        rest = &body[body_end + marker.len()..];
        if rest.starts_with(b"--") {
            return Err(refused());
        }
        rest = rest
            .strip_prefix(b"\r\n")
            .or_else(|| rest.strip_prefix(b"\n"))
            .unwrap_or(rest);
    }
}

/// What an error answer says: an OWS exception's code and text, else its start.
fn exception_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let between = |open: &str, close: char| {
        let start = text.find(open)? + open.len();
        text[start..].split(close).next().map(str::to_string)
    };
    let code = between("exceptionCode=\"", '"');
    let message = between("ExceptionText>", '<');
    match (code, message) {
        (Some(code), Some(message)) => format!("{code}: {message}"),
        (Some(one), None) | (None, Some(one)) => one,
        (None, None) => text.chars().take(80).collect(),
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// A Float32 GeoTIFF from the WCS as a raster; its grid comes from ModelTiepoint and
/// ModelPixelScale (the 25832 answers carry no GeoKeys). NoData (−9999) becomes `NaN`.
/// Striped and tiled (LVermGeo SH: 256 px tiles) TIFFs are read.
pub fn decode_float_tiff(bytes: &[u8]) -> Result<Raster<f32>, String> {
    decode_float_tiff_from(std::io::Cursor::new(bytes))
}

/// [`decode_float_tiff`] from a reader, so a large file is never held as bytes besides its
/// pixels.
pub(crate) fn decode_float_tiff_from(
    reader: impl std::io::Read + std::io::Seek,
) -> Result<Raster<f32>, String> {
    use tiff::tags::Tag;
    let error = |e: tiff::TiffError| e.to_string();
    let mut decoder = tiff::decoder::Decoder::new(reader).map_err(error)?;
    let (width, height) = decoder.dimensions().map_err(error)?;
    let scale = decoder
        .get_tag_f64_vec(Tag::ModelPixelScaleTag)
        .map_err(error)?;
    let tiepoint = decoder
        .get_tag_f64_vec(Tag::ModelTiepointTag)
        .map_err(error)?;
    let (Some(&pixel), Some(&west), Some(&north)) =
        (scale.first(), tiepoint.get(3), tiepoint.get(4))
    else {
        return Err("GeoTIFF without tiepoint or pixel scale".to_string());
    };
    let tiff::decoder::DecodingResult::F32(values) = decoder.read_image().map_err(error)? else {
        return Err("height model is not 32-bit float".to_string());
    };
    // Tiepoints may carry float noise (LGV: 564999.9999999999); round to millimetres.
    let mm = |v: f64| (v * 1000.0).round() / 1000.0;
    let grid = GridSpec {
        west: mm(west),
        north: mm(north),
        pixel_size: pixel,
        width,
        height,
    };
    Ok(Raster {
        grid,
        data: values
            .into_iter()
            .map(|z| if z < NODATA_BELOW { f32::NAN } else { z })
            .collect(),
    })
}

/// LVermGeo marks cells without data with −9999.
const NODATA_BELOW: f32 = -9998.0;
/// Largest side of one GetCoverage; the WCS allows 4000 px (= m) per axis.
const WCS_MAX_M: f64 = 4000.0;
/// The WCS answers 404 for any subset reaching past its coverage (verified live 2026-11-07), so
/// requests are cut to it; the rest stays empty.
const COVERAGE: BoundingBox = BoundingBox {
    min_e: 606_000.0,
    min_n: 5_646_000.0,
    max_e: 790_000.0,
    max_n: 5_882_000.0,
};

/// Downloads `model` over `bbox` (EPSG:25832) in pieces of at most 4000 m and writes it as a
/// Float32 GeoTIFF `out` with its world file (`.tfw`).
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    model: Model,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<(), FetchError> {
    let grid = GridSpec::covering(bbox, 1.0);
    let mut raster = Raster::filled(grid, f32::NAN);
    let Some(b) = grid.bounds().intersection(&COVERAGE) else {
        // Nothing of the area lies in the coverage: an empty model.
        return write_float_raster(&raster, out, model.wcs_endpoint());
    };
    let starts = |min: f64, max: f64| {
        let n = ((max - min) / WCS_MAX_M).ceil().max(1.0) as u32;
        (0..n).map(move |i| {
            (
                min + f64::from(i) * WCS_MAX_M,
                (min + f64::from(i + 1) * WCS_MAX_M).min(max),
            )
        })
    };
    for (min_e, max_e) in starts(b.min_e, b.max_e) {
        for (min_n, max_n) in starts(b.min_n, b.max_n) {
            let piece = BoundingBox {
                min_e,
                min_n,
                max_e,
                max_n,
            };
            let url = wcs_url(model, &piece);
            let part = get_with_retries(fetcher, &url, |bytes| {
                multipart_tiff(bytes)
                    .and_then(decode_float_tiff)
                    .map_err(|message| FetchError::Http {
                        url: url.clone(),
                        message,
                    })
            })?;
            raster
                .paste_with(&part.grid, |x, y| part.get(x, y))
                .map_err(|e| FetchError::Http {
                    url: url.clone(),
                    message: e.to_string(),
                })?;
        }
    }
    write_float_raster(&raster, out, model.wcs_endpoint())
}

/// Writes `raster` as a Float32 GeoTIFF `out` with its world file (`.tfw`); `source` names the
/// service in an error.
pub(crate) fn write_float_raster(
    raster: &Raster<f32>,
    out: &Path,
    source: &str,
) -> Result<(), FetchError> {
    let grid = raster.grid;
    let mut tiff = std::io::Cursor::new(Vec::new());
    tiff::encoder::TiffEncoder::new(&mut tiff)
        .and_then(|encoder| {
            encoder
                .with_compression(tiff::encoder::Compression::Lzw)
                .write_image::<tiff::encoder::colortype::Gray32Float>(
                grid.width,
                grid.height,
                &raster.data,
            )
        })
        .map_err(|e| FetchError::Http {
            url: source.to_string(),
            message: e.to_string(),
        })?;
    write_atomically(out, tiff.get_ref())?;
    write_atomically(&out.with_extension("tfw"), grid.world_file().as_bytes())?;
    Ok(())
}

/// `raster` as an LZW Float32 GeoTIFF with ModelPixelScale and ModelTiepoint, which
/// [`decode_float_tiff`] reads back (for caching; no world file).
pub(crate) fn encode_float_geotiff(raster: &Raster<f32>) -> Result<Vec<u8>, String> {
    use tiff::tags::Tag;
    let grid = raster.grid;
    let mut tiff = std::io::Cursor::new(Vec::new());
    let mut encoder = tiff::encoder::TiffEncoder::new(&mut tiff)
        .map_err(|e| e.to_string())?
        .with_compression(tiff::encoder::Compression::Lzw);
    let mut image = encoder
        .new_image::<tiff::encoder::colortype::Gray32Float>(grid.width, grid.height)
        .map_err(|e| e.to_string())?;
    image
        .encoder()
        .write_tag(
            Tag::ModelPixelScaleTag,
            &[grid.pixel_size, grid.pixel_size, 0.0][..],
        )
        .map_err(|e| e.to_string())?;
    image
        .encoder()
        .write_tag(
            Tag::ModelTiepointTag,
            &[0.0, 0.0, 0.0, grid.west, grid.north, 0.0][..],
        )
        .map_err(|e| e.to_string())?;
    image.write_data(&raster.data).map_err(|e| e.to_string())?;
    Ok(tiff.into_inner())
}

/// The flight date of the orthophoto at the centre of `bbox` (EPSG:25832); `None` when the index
/// does not tell it.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    index_date(fetcher, "truedop_kacheln", bbox)
}

/// The date of the surface model (DOM1) at the centre of `bbox`.
pub fn surface_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    index_date(fetcher, "bdom_aktualitaet_zyklus", bbox)
}

fn index_date(fetcher: &dyn Fetcher, layer: &str, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    let url = format!(
        "{INDEX_URL}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
         &TYPENAMES=atkis_geobasis_bestand:{layer}&OUTPUTFORMAT=GEOJSON\
         &BBOX={},{},{},{},urn:ogc:def:crs:EPSG::25832",
        (e - 10.0).floor(),
        (n - 10.0).floor(),
        (e + 10.0).ceil(),
        (n + 10.0).ceil()
    );
    // A failed index costs a re-download of the orthophoto, so it gets the usual retries.
    get_with_retries(fetcher, &url, |bytes| {
        let text = String::from_utf8_lossy(bytes);
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .filter(|answer| answer["features"].is_array())
            .map(|_| parse_index_date(&text))
            .ok_or_else(|| FetchError::Http {
                url: url.clone(),
                message: format!("not a tile index answer: {}", exception_text(bytes)),
            })
    })
    .ok()
    .flatten()
}

/// The flight date (`BFDATUM`, e.g. `29.04.2026`) of the first feature in a tile index answer.
pub fn parse_index_date(json: &str) -> Option<String> {
    let answer: serde_json::Value = serde_json::from_str(json).ok()?;
    let date = answer["features"].get(0)?["properties"]["BFDATUM"].as_str()?;
    (!date.is_empty()).then(|| date.to_string())
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const WCS_DGM1: &[u8] =
        include_bytes!("../../../../tests/fixtures/lvermgeo-wcs-dgm1.multipart");
    pub const TRUEDOP_INDEX: &str =
        include_str!("../../../../tests/fixtures/lvermgeo-truedop-index.json");
    pub const BDOM_INDEX: &str =
        include_str!("../../../../tests/fixtures/lvermgeo-bdom-index.json");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    /// The live sample: 20 × 16 m at the Latina, Halle.
    const SAMPLE: BoundingBox = BoundingBox {
        min_e: 706_440.0,
        min_n: 5_706_900.0,
        max_e: 706_460.0,
        max_n: 5_706_916.0,
    };

    #[test]
    fn wcs_url_subsets_whole_metres() {
        let bbox = BoundingBox {
            min_e: 706_440.4,
            min_n: 5_706_899.6,
            max_e: 706_459.2,
            max_n: 5_706_915.1,
        };
        assert_eq!(
            wcs_url(Model::Dgm1, &bbox),
            "https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1?SERVICE=WCS&VERSION=2.0.1\
             &REQUEST=GetCoverage&COVERAGEID=Coverage1&FORMAT=image/tiff\
             &SUBSET=x(706440,706460)&SUBSET=y(5706899,5706916)"
        );
        assert!(wcs_url(Model::Dom1, &bbox).starts_with(Model::Dom1.wcs_endpoint()));
    }

    #[test]
    fn multipart_answer_yields_the_tiff() {
        let tiff = multipart_tiff(WCS_DGM1).unwrap();
        assert!(tiff.starts_with(b"II*\0"), "{:?}", &tiff[..8]);
        assert!(multipart_tiff(b"<?xml version=\"1.0\"?><ExceptionReport/>").is_err());
        assert!(multipart_tiff(b"--wcs\nContent-Type: text/xml\n\n<x/>\n--wcs--\n").is_err());
    }

    #[test]
    fn geotiff_tags_give_the_grid() {
        let raster = decode_float_tiff(multipart_tiff(WCS_DGM1).unwrap()).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (20, 16));
        assert_eq!(
            (raster.grid.west, raster.grid.north),
            (706_440.0, 5_706_916.0)
        );
        assert_eq!(raster.grid.pixel_size, 1.0);
        // The school yard lies at about 90.8 m (reference §5).
        let z = raster.get(10, 8);
        assert!((85.0..96.0).contains(&z), "{z}");
    }

    #[test]
    fn fetched_model_is_written_with_its_world_file() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(Model::Dgm1.wcs_endpoint(), WCS_DGM1.to_vec());
        let out = dir.path().join("dgm1.tif");
        fetch_model(&fetcher, Model::Dgm1, &SAMPLE, &out).unwrap();
        let world = std::fs::read_to_string(out.with_extension("tfw")).unwrap();
        assert_eq!(
            world,
            crate::raster::GridSpec::covering(&SAMPLE, 1.0).world_file()
        );
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (20, 16));
        assert!((85.0..96.0).contains(&raster.get(10, 8)));
    }

    #[test]
    fn wcs_error_page_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            Model::Dgm1.wcs_endpoint(),
            b"<html><body>ArcGIS Server Error</body></html>".to_vec(),
        );
        let err =
            fetch_model(&fetcher, Model::Dgm1, &SAMPLE, &dir.path().join("x.tif")).unwrap_err();
        assert!(err.to_string().contains("no TIFF"), "{err}");
        assert_eq!(fetcher.calls.get(), 3, "retried");
        assert!(!dir.path().join("x.tif").exists());
    }

    #[test]
    fn tile_index_gives_the_flight_date() {
        assert_eq!(
            parse_index_date(TRUEDOP_INDEX).as_deref(),
            Some("29.04.2026")
        );
        assert_eq!(parse_index_date(BDOM_INDEX).as_deref(), Some("13.05.2024"));
        assert_eq!(
            parse_index_date(r#"{"type":"FeatureCollection","features":[]}"#),
            None
        );
    }

    #[test]
    fn wms_is_asked_in_zone_32() {
        let grid = crate::raster::GridSpec::covering(&SAMPLE, 0.2);
        let (_, url) = &crate::sources::wms::pieces(&WMS, &grid)[0];
        assert!(
            url.starts_with(WMS.url) && url.contains("&CRS=EPSG:25832&"),
            "{url}"
        );
        assert!(url.contains("LAYERS=lsa_lvermgeo_dop20_2"), "{url}");
    }

    /// Answers every GetCoverage with a synthetic TIFF of exactly the requested subset (heights
    /// 50 m), and records the subsets asked for.
    struct Wcs {
        subsets: std::cell::RefCell<Vec<(f64, f64, f64, f64)>>,
    }

    impl Fetcher for Wcs {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            let range = |axis: &str| -> (f64, f64) {
                let start = url.find(&format!("SUBSET={axis}(")).unwrap() + axis.len() + 8;
                let inner = &url[start..url[start..].find(')').unwrap() + start];
                let (a, b) = inner.split_once(',').unwrap();
                (a.parse().unwrap(), b.parse().unwrap())
            };
            let ((x0, x1), (y0, y1)) = (range("x"), range("y"));
            self.subsets.borrow_mut().push((x0, x1, y0, y1));
            let grid = crate::raster::GridSpec {
                west: x0,
                north: y1,
                pixel_size: 1.0,
                width: (x1 - x0) as u32,
                height: (y1 - y0) as u32,
            };
            let values = vec![50.0; (grid.width * grid.height) as usize];
            Ok(crate::fetch::test_support::wcs_answer(&grid, &values))
        }
    }

    #[test]
    fn wcs_requests_stay_inside_the_coverage_and_split_at_4000_m() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = Wcs {
            subsets: std::cell::RefCell::new(Vec::new()),
        };
        // 4100 m wide, reaching 50 m past the east edge of the coverage (E 790 000).
        let bbox = BoundingBox {
            min_e: 785_950.0,
            min_n: 5_700_000.0,
            max_e: 790_050.0,
            max_n: 5_700_010.0,
        };
        let out = dir.path().join("dgm1.tif");
        fetch_model(&fetcher, Model::Dgm1, &bbox, &out).unwrap();
        let subsets = fetcher.subsets.borrow();
        assert_eq!(
            *subsets,
            [
                (785_950.0, 789_950.0, 5_700_000.0, 5_700_010.0),
                (789_950.0, 790_000.0, 5_700_000.0, 5_700_010.0)
            ]
        );
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!(raster.grid.width, 4100);
        assert_eq!(raster.get(0, 0), 50.0);
        assert!(raster.get(4099, 0).is_nan(), "beyond the coverage");
    }

    #[test]
    fn area_outside_the_coverage_asks_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = Wcs {
            subsets: std::cell::RefCell::new(Vec::new()),
        };
        let bbox = BoundingBox {
            min_e: 795_000.0,
            min_n: 5_700_000.0,
            max_e: 795_100.0,
            max_n: 5_700_100.0,
        };
        fetch_model(&fetcher, Model::Dgm1, &bbox, &dir.path().join("x.tif")).unwrap();
        assert!(fetcher.subsets.borrow().is_empty());
    }

    #[test]
    fn wcs_exception_text_is_shown() {
        let report = br#"<?xml version="1.0" encoding="UTF-8"?><ExceptionReport xmlns="http://www.opengis.net/ows/2.0" version="2.0.0"><Exception exceptionCode="InvalidSubsetting" locator="x"><ExceptionText>Subset is outside the coverage extent</ExceptionText></Exception></ExceptionReport>"#;
        let err = multipart_tiff(report).unwrap_err();
        assert!(err.contains("InvalidSubsetting"), "{err}");
        assert!(
            err.contains("Subset is outside the coverage extent"),
            "{err}"
        );
    }

    #[test]
    fn tile_index_is_retried() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(INDEX_URL, TRUEDOP_INDEX.as_bytes().to_vec());
        fetcher.flaky.borrow_mut().insert(INDEX_URL.to_string(), 1);
        assert_eq!(
            flight_date(&fetcher, &SAMPLE).as_deref(),
            Some("29.04.2026")
        );
    }
}
