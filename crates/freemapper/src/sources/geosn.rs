//! GeoSN Saxony open data (dl-de/by-2-0, source note "GeoSN"): 2 km tiles of the terrain and
//! surface models, their currency and download links from the ArcGIS tile index.

use std::path::Path;

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::BoundingBox;
use crate::crs::Crs;
use crate::fetch::{FetchError, Fetcher};
use crate::raster::GridSpec;
use crate::sources::lgb::Product;
use crate::sources::wms::{self, WmsService};
use crate::tiles::TileId;

/// The orthophoto WMS (DOP20 RGB, released flights), and its layers.
pub const WMS_URL: &str = "https://geodienste.sachsen.de/wms_geosn_dop-rgb/guest";
const WMS_LAYER: &str = "sn_dop_020";
/// The orthophoto WMS of GeoSN (EPSG:25833).
pub const WMS: WmsService = WmsService {
    url: WMS_URL,
    layer: WMS_LAYER,
    crs: Crs::Utm33,
    extra: "",
    max_px: 4000,
};
const WMS_INFO_LAYER: &str = "sn_dop_020_info";
pub use crate::sources::wms::ORTHOPHOTO_PIXEL_M;

/// The ArcGIS tile index with one layer per product (`Kachel`, `Download`, `Stand`).
pub const INDEX_URL: &str = "https://geodienste.sachsen.de/ags-relay/ArcGISServer/guest/arcgis/rest/services/geosn/rest_geosn_downloadlinks/MapServer";

/// One row of the tile index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexRow {
    pub tile: TileId,
    /// The latest date of `Stand`.
    pub currency: NaiveDate,
    /// The `Download` link, if the row has one.
    pub url: Option<String>,
}

/// Query of the index layer of `product` for the rows of `tiles` (2 km tiles).
pub fn index_url(product: Product, tiles: &[TileId]) -> String {
    let layer = match product {
        Product::SaxonyDsm => 4,
        _ => 6,
    };
    let km = product.grid_km();
    let bounds = tiles.iter().map(|t| t.bounds_km(km));
    let fold = |f: fn(f64, f64) -> f64, pick: fn(&crate::crs::BoundingBox) -> f64, start: f64| {
        bounds.clone().map(|b| pick(&b)).fold(start, f)
    };
    // Shrunk by 1 m: the index also returns tiles that only touch the envelope's edge.
    let min_e = fold(f64::min, |b| b.min_e, f64::INFINITY) + 1.0;
    let min_n = fold(f64::min, |b| b.min_n, f64::INFINITY) + 1.0;
    let max_e = fold(f64::max, |b| b.max_e, f64::NEG_INFINITY) - 1.0;
    let max_n = fold(f64::max, |b| b.max_n, f64::NEG_INFINITY) - 1.0;
    format!(
        "{INDEX_URL}/{layer}/query?f=json&geometry={min_e},{min_n},{max_e},{max_n}\
         &geometryType=esriGeometryEnvelope&inSR=25833&spatialRel=esriSpatialRelIntersects\
         &outFields=Kachel,Download,Stand&returnGeometry=false"
    )
}

/// The latest `YYYY-MM-DD` date in a `Stand` value such as `2024-11-27, 2024-11-30 bis 2024-12-28`.
pub fn latest_date(stand: &str) -> Option<NaiveDate> {
    stand
        .char_indices()
        .filter_map(|(i, _)| stand.get(i..i + 10)?.parse::<NaiveDate>().ok())
        .max()
}

/// The rows of an index answer; rows without a readable tile or date are skipped.
pub fn parse_index(json: &str) -> Result<Vec<IndexRow>, String> {
    let answer: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if let Some(error) = answer.get("error") {
        return Err(format!("tile index error: {error}"));
    }
    let features = answer["features"]
        .as_array()
        .ok_or("tile index answer without features")?;
    Ok(features
        .iter()
        .filter_map(|feature| {
            let attributes = &feature["attributes"];
            let kachel = attributes["Kachel"].as_str()?;
            if kachel.len() != 7 {
                return None;
            }
            Some(IndexRow {
                tile: TileId {
                    east_km: kachel[..3].parse().ok()?,
                    north_km: kachel[3..].parse().ok()?,
                },
                currency: latest_date(attributes["Stand"].as_str()?)?,
                url: attributes["Download"].as_str().map(str::to_string),
            })
        })
        .collect())
}

/// The GetMap requests covering `grid`; see [`wms::pieces`].
pub fn wms_requests(grid: &GridSpec) -> Vec<(GridSpec, String)> {
    wms::pieces(&WMS, grid)
}

/// Downloads the orthophoto over `bbox` from the GeoSN WMS; see [`wms::fetch_orthophoto`].
pub fn fetch_orthophoto(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<(), FetchError> {
    wms::fetch_orthophoto(fetcher, &WMS, bbox, root, out)
}

/// `v` rounded to millimetres.
fn mm(v: f64) -> f64 {
    wms::mm(v)
}

/// The flight date at the centre of `bbox` from the info layer; `None` when the server does not
/// tell it.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        mm((bbox.min_e + bbox.max_e) / 2.0),
        mm((bbox.min_n + bbox.max_n) / 2.0),
    );
    let url = format!(
        "{WMS_URL}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={WMS_INFO_LAYER}\
         &QUERY_LAYERS={WMS_INFO_LAYER}&STYLES=&CRS=EPSG:25833&BBOX={},{},{},{}&WIDTH=101\
         &HEIGHT=101&I=50&J=50&INFO_FORMAT=text/plain",
        e - 10.1,
        n - 10.1,
        e + 10.1,
        n + 10.1
    );
    let bytes = fetcher.get(&url).ok()?;
    parse_flight_date(&String::from_utf8_lossy(&bytes))
}

/// The flight date of a `sn_dop_020_info` answer, e.g. `19.03. + 07.04.2024` from
/// `Gebiet: Dresden | Datum: 19.03. + 07.04.2024 | ROHDOP: DOP`.
pub fn parse_flight_date(text: &str) -> Option<String> {
    let after = text.split_once("Datum:")?.1;
    let date = after.split(['|', '\n']).next()?.trim();
    (!date.is_empty()).then(|| date.to_string())
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const DGM1_INDEX: &str = include_str!("../../../../tests/fixtures/geosn-index-dgm1.json");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    const KREUZGYMNASIUM: TileId = TileId {
        east_km: 414,
        north_km: 5654,
    };
    const EAST: TileId = TileId {
        east_km: 416,
        north_km: 5654,
    };

    #[test]
    fn geosn_tile_names_and_members() {
        assert_eq!(
            Product::SaxonyDtm.tile_url(KREUZGYMNASIUM),
            "https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/JCcXyifaNdLDnxZ/dgm1_33414_5654_2_sn_tiff.zip"
        );
        assert_eq!(
            Product::SaxonyDsm.tile_url(KREUZGYMNASIUM),
            "https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/S6wwnFwX7882sZm/dom1_33414_5654_2_sn_tiff.zip"
        );
        assert_eq!(
            Product::SaxonyDtm.raster_member(KREUZGYMNASIUM),
            "dgm1_33414_5654_2_sn.tif"
        );
        assert_eq!(
            Product::SaxonyDsm.world_file_member(KREUZGYMNASIUM),
            "dom1_33414_5654_2_sn.tfw"
        );
        assert_eq!(Product::SaxonyDtm.grid_km(), 2);
        assert_eq!(Product::Dtm.grid_km(), 1);
        assert_eq!(Product::SaxonyDtm.cache_dir_name(), "dgm1");
        assert_eq!(Product::SaxonyDtm.provider_dir_name(), "geosn");
        assert_eq!(Product::Dtm.provider_dir_name(), "lgb");
    }

    #[test]
    fn index_url_is_an_envelope_query() {
        assert_eq!(
            index_url(Product::SaxonyDtm, &[KREUZGYMNASIUM, EAST]),
            format!(
                "{INDEX_URL}/6/query?f=json&geometry=414001,5654001,417999,5655999\
                 &geometryType=esriGeometryEnvelope&inSR=25833&spatialRel=esriSpatialRelIntersects\
                 &outFields=Kachel,Download,Stand&returnGeometry=false"
            )
        );
        assert!(
            index_url(Product::SaxonyDsm, &[EAST]).starts_with(&format!("{INDEX_URL}/4/query?"))
        );
    }

    #[test]
    fn stand_takes_the_latest_date() {
        let date = |s: &str| s.parse::<NaiveDate>().ok();
        assert_eq!(latest_date("2024-11-30"), date("2024-11-30"));
        assert_eq!(
            latest_date("2020-11-24, 2020-11-25, 2020-11-30"),
            date("2020-11-30")
        );
        assert_eq!(
            latest_date("2024-11-27, 2024-11-30 bis 2024-12-28"),
            date("2024-12-28")
        );
        assert_eq!(latest_date(""), None);
        assert_eq!(latest_date("unbekannt"), None);
    }

    #[test]
    fn index_rows_give_currency_and_url() {
        let rows = parse_index(DGM1_INDEX).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].tile, KREUZGYMNASIUM);
        assert_eq!(rows[0].currency, "2024-11-30".parse::<NaiveDate>().unwrap());
        assert!(
            rows[0]
                .url
                .as_deref()
                .unwrap()
                .ends_with("/dgm1_33414_5654_2_sn_tiff.zip")
        );
        assert_eq!(rows[1].tile, EAST);
        assert_eq!(rows[2].currency, "2024-12-28".parse::<NaiveDate>().unwrap());
        assert!(parse_index(r#"{"error":{"code":400,"message":"Invalid query"}}"#).is_err());
    }

    fn grid(width: u32, height: u32) -> GridSpec {
        GridSpec {
            west: 415_000.0,
            north: 5_656_000.0,
            pixel_size: ORTHOPHOTO_PIXEL_M,
            width,
            height,
        }
    }

    #[test]
    fn wms_pieces_cover_the_grid() {
        let pieces = wms_requests(&grid(9000, 5000));
        let sizes: Vec<(u32, u32)> = pieces.iter().map(|(g, _)| (g.width, g.height)).collect();
        assert_eq!(
            sizes,
            [
                (4000, 4000),
                (4000, 1000),
                (4000, 4000),
                (4000, 1000),
                (1000, 4000),
                (1000, 1000)
            ]
        );
        let (first, url) = &pieces[0];
        assert_eq!((first.west, first.north), (415_000.0, 5_656_000.0));
        assert_eq!(
            url,
            &format!(
                "{WMS_URL}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=sn_dop_020&STYLES=\
                 &CRS=EPSG:25833&BBOX=415000,5655200,415800,5656000&WIDTH=4000&HEIGHT=4000\
                 &FORMAT=image/jpeg"
            )
        );
        let (last, _) = &pieces[5];
        assert!((last.west - 416_600.0).abs() < 1e-6 && (last.north - 5_655_200.0).abs() < 1e-6);
    }

    /// Answers every GetMap with a JPEG of the requested size, red west of E 415 800, blue east;
    /// GetFeatureInfo with the Dresden flight text.
    struct Wms {
        calls: std::cell::Cell<usize>,
    }

    impl Fetcher for Wms {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.set(self.calls.get() + 1);
            if url.contains("REQUEST=GetFeatureInfo") {
                return Ok(b"Gebiet: Dresden | Datum: 19.03. + 07.04.2024 | ROHDOP: DOP".to_vec());
            }
            let param = |name: &str| -> String {
                url.split('&')
                    .find_map(|p| p.strip_prefix(&format!("{name}=")))
                    .unwrap()
                    .to_string()
            };
            let (w, h): (u32, u32) = (
                param("WIDTH").parse().unwrap(),
                param("HEIGHT").parse().unwrap(),
            );
            let west: f64 = param("BBOX").split(',').next().unwrap().parse().unwrap();
            let rgb = if west < 415_800.0 {
                [255, 0, 0]
            } else {
                [0, 0, 255]
            };
            Ok(crate::fetch::test_support::solid_jpeg(w, h, rgb))
        }
    }

    #[test]
    fn stitched_image_is_georeferenced() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("orthophoto-wms.jpg");
        let bbox = BoundingBox {
            min_e: 415_000.0,
            min_n: 5_655_500.0,
            max_e: 416_000.0,
            max_n: 5_656_000.0,
        };
        let fetcher = Wms {
            calls: std::cell::Cell::new(0),
        };
        fetch_orthophoto(&fetcher, &bbox, out.parent().unwrap(), &out).unwrap();
        assert_eq!(
            flight_date(&fetcher, &bbox).as_deref(),
            Some("19.03. + 07.04.2024")
        );
        let image = image::open(&out).unwrap().to_rgb8();
        assert_eq!((image.width(), image.height()), (5000, 2500));
        assert!(image.get_pixel(100, 100).0[0] > 200, "west piece is red");
        assert!(image.get_pixel(4900, 100).0[2] > 200, "east piece is blue");
        let world = std::fs::read_to_string(out.with_extension("jgw")).unwrap();
        assert_eq!(
            world,
            GridSpec::covering(&bbox, ORTHOPHOTO_PIXEL_M).world_file()
        );
        // Two pieces and one feature info request.
        assert_eq!(fetcher.calls.get(), 3);
    }

    #[test]
    fn xml_instead_of_jpeg_is_an_error_asked_once() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = crate::fetch::test_support::MapFetcher::default();
        fetcher.insert_prefix(
            WMS_URL,
            br#"<?xml version="1.0"?><ServiceExceptionReport/>"#.to_vec(),
        );
        let bbox = BoundingBox {
            min_e: 415_000.0,
            min_n: 5_655_900.0,
            max_e: 415_100.0,
            max_n: 5_656_000.0,
        };
        let err =
            fetch_orthophoto(&fetcher, &bbox, dir.path(), &dir.path().join("o.jpg")).unwrap_err();
        assert!(
            err.to_string().contains("the WMS answered with an error"),
            "{err}"
        );
        assert_eq!(fetcher.calls.get(), 1);
        assert!(!dir.path().join("o.jpg").exists());
    }

    #[test]
    fn flight_date_from_feature_info() {
        assert_eq!(
            parse_flight_date("Gebiet: Dresden | Datum: 19.03. + 07.04.2024 | ROHDOP: DOP")
                .as_deref(),
            Some("19.03. + 07.04.2024")
        );
        assert_eq!(
            parse_flight_date("Datum: 2025-04-03\n").as_deref(),
            Some("2025-04-03")
        );
        assert_eq!(parse_flight_date("no information"), None);
    }
}
