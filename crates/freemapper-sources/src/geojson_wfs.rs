//! Features from a WFS 2.0 that answers GeoJSON (the BfN protected areas, the BKG state borders
//! VG250), asked by a box in WGS 84 and read page by page; and the same for a WFS that answers
//! only GML 3.2 (the forest offices of Brandenburg), read into GeoJSON features.

use std::path::Path;

use serde_json::Value;

use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::wfs::parse_gml;

/// Features asked for per request.
const PAGE_SIZE: usize = 500;

/// The largest answer read; a larger one is a misbehaving server.
const MAX_BYTES: u64 = 50 * 1024 * 1024;

/// The most pages read for one type (10 000 features); more is a service that ignores
/// `startIndex`, or a box far larger than a map area.
const MAX_PAGES: usize = 20;

/// A box in WGS 84 degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatLonBox {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

/// A WFS 2.0 that answers GeoJSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoJsonWfs {
    /// The service address without a query.
    pub base: &'static str,
    /// The `outputFormat` the service names its GeoJSON by.
    pub output_format: &'static str,
    /// Whether the box ends with the CRS (`,urn:ogc:def:crs:EPSG::4326`); a service without it
    /// reads the box in its own CRS.
    pub bbox_crs_suffix: bool,
}

/// The URL of the page of `type_name` that starts at feature `start_index`. WFS 2.0 with an
/// EPSG URN asks for latitude first, so the box is written as latitude, longitude.
pub(crate) fn query_url(
    service: &GeoJsonWfs,
    type_name: &str,
    bbox: &LatLonBox,
    start_index: usize,
) -> String {
    let suffix = if service.bbox_crs_suffix {
        ",urn:ogc:def:crs:EPSG::4326"
    } else {
        ""
    };
    format!(
        "{}?service=WFS&version=2.0.0&request=GetFeature&outputFormat={}\
         &srsName=urn:ogc:def:crs:EPSG::4326&typeNames={}&count={PAGE_SIZE}\
         &startIndex={start_index}&bbox={},{},{},{}{suffix}",
        service.base,
        service.output_format,
        type_name,
        bbox.min_lat,
        bbox.min_lon,
        bbox.max_lat,
        bbox.max_lon,
    )
}

/// All features of `type_name` inside `bbox`, in the order the service gives them. Pages of
/// 500 until one has fewer, at most 20: a 20th full page is an error with its URL. Each answer
/// streams into the file `part` (removed afterwards), so one over 50 MB is stopped while it is
/// read and refused with the URL.
pub fn get_features(
    fetcher: &dyn Fetcher,
    service: &GeoJsonWfs,
    type_name: &str,
    bbox: &LatLonBox,
    part: &Path,
) -> Result<Vec<Value>, FetchError> {
    pages(fetcher, service, type_name, bbox, part, read_page)
}

/// [`get_features`] for a service that answers GML 3.2 (`output_format` names it, `gml32` on a
/// GeoServer). The features come as GeoJSON features with the polygons of [`parse_gml`] and
/// their attributes as properties. Positions stay as the service writes them: with the
/// `srsName` of the query (WGS 84 as an EPSG URN) that is latitude first.
pub fn get_gml_features(
    fetcher: &dyn Fetcher,
    service: &GeoJsonWfs,
    type_name: &str,
    bbox: &LatLonBox,
    part: &Path,
) -> Result<Vec<Value>, FetchError> {
    pages(fetcher, service, type_name, bbox, part, read_gml_page)
}

/// The paging of [`get_features`], with `read` turning one answer into its features.
fn pages(
    fetcher: &dyn Fetcher,
    service: &GeoJsonWfs,
    type_name: &str,
    bbox: &LatLonBox,
    part: &Path,
    read: fn(&str, &[u8]) -> Result<Vec<Value>, FetchError>,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    for page_number in 1..=MAX_PAGES {
        let url = query_url(service, type_name, bbox, features.len());
        let page =
            get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| read(&url, bytes))?;
        let short = page.len() < PAGE_SIZE;
        features.extend(page);
        if short {
            return Ok(features);
        }
        if page_number == MAX_PAGES {
            return Err(FetchError::Http {
                url,
                message: format!(
                    "the service sent more than {MAX_PAGES} full pages ({} features); stopped",
                    MAX_PAGES * PAGE_SIZE
                ),
            });
        }
    }
    unreachable!("the last page returns")
}

/// The features of a GML 3.2 FeatureCollection.
fn read_gml_page(url: &str, bytes: &[u8]) -> Result<Vec<Value>, FetchError> {
    let not_gml = |message: String| FetchError::Http {
        url: url.to_string(),
        message,
    };
    let text =
        std::str::from_utf8(bytes).map_err(|e| not_gml(format!("the answer is not UTF-8: {e}")))?;
    parse_gml(text).map_err(|e| not_gml(e.to_string()))
}

/// The `features` of a GeoJSON FeatureCollection.
fn read_page(url: &str, bytes: &[u8]) -> Result<Vec<Value>, FetchError> {
    let not_geojson = |message: String| FetchError::Http {
        url: url.to_string(),
        message,
    };
    let mut value: Value = serde_json::from_slice(bytes)
        .map_err(|e| not_geojson(format!("the answer is not GeoJSON: {e}")))?;
    match value.get_mut("features").map(Value::take) {
        Some(Value::Array(features)) => Ok(features),
        // Hamburg's services answer an empty box with a collection that has no `features`.
        None if value.get("type").and_then(Value::as_str) == Some("FeatureCollection") => {
            Ok(Vec::new())
        }
        _ => Err(not_geojson(
            "the answer is not a FeatureCollection".to_string(),
        )),
    }
}

/// The area of `feature` as x = longitude, y = latitude, the order both services answer in;
/// `None` when it has no Polygon or MultiPolygon with at least one ring.
pub(crate) fn area_geometry(feature: &Value) -> Option<geo_types::MultiPolygon<f64>> {
    let geometry =
        serde_json::from_value::<geojson::Geometry>(feature.get("geometry")?.clone()).ok()?;
    let area = match geo_types::Geometry::<f64>::try_from(geometry).ok()? {
        geo_types::Geometry::Polygon(p) => geo_types::MultiPolygon(vec![p]),
        geo_types::Geometry::MultiPolygon(mp) => mp,
        _ => return None,
    };
    let usable = area.0.iter().any(|p| p.exterior().0.len() >= 4);
    usable.then_some(area)
}

/// The text of property `key`, trimmed; `None` when it is missing, not text or blank (the BfN
/// writes a single space for an empty value). A quote written as `\"` becomes a plain quote.
pub(crate) fn property_text(feature: &Value, key: &str) -> Option<String> {
    let text = feature.get("properties")?.get(key)?.as_str()?.trim();
    (!text.is_empty()).then(|| text.replace("\\\"", "\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    /// A part file in a fresh temporary folder (kept alive for the test run).
    fn part() -> std::path::PathBuf {
        let dir = tempfile::tempdir().unwrap().keep();
        dir.join("page.part")
    }

    const BFN: GeoJsonWfs = GeoJsonWfs {
        base: "https://geodienste.bfn.de/ogc/wfs/schutzgebiet",
        output_format: "application/json",
        bbox_crs_suffix: false,
    };
    const VG250: GeoJsonWfs = GeoJsonWfs {
        base: "https://sgx.geodatenzentrum.de/wfs_vg250",
        output_format: "application/json",
        bbox_crs_suffix: true,
    };
    const BOX: LatLonBox = LatLonBox {
        min_lat: 52.53,
        min_lon: 13.83,
        max_lat: 52.545,
        max_lon: 13.86,
    };

    fn collection(n: usize) -> Vec<u8> {
        let features: Vec<Value> = (0..n)
            .map(|i| serde_json::json!({"type": "Feature", "properties": {"i": i}}))
            .collect();
        serde_json::to_vec(&serde_json::json!({"type": "FeatureCollection", "features": features}))
            .unwrap()
    }

    #[test]
    fn query_puts_latitude_first() {
        let url = query_url(&BFN, "bfn_sch_Schutzgebiet:Naturschutzgebiet", &BOX, 0);
        assert!(url.contains("bbox=52.53,13.83,52.545,13.86"));
        assert!(url.ends_with("bbox=52.53,13.83,52.545,13.86"));
        assert!(url.contains("count=500&startIndex=0"));
    }

    #[test]
    fn vg250_query_ends_the_bbox_with_its_crs() {
        let url = query_url(&VG250, "vg250_lan", &BOX, 0);
        assert!(url.ends_with("bbox=52.53,13.83,52.545,13.86,urn:ogc:def:crs:EPSG::4326"));
    }

    #[test]
    fn pages_until_a_short_page() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&query_url(&BFN, "t", &BOX, 0), collection(500));
        fetcher.insert(&query_url(&BFN, "t", &BOX, 500), collection(3));
        let features = get_features(&fetcher, &BFN, "t", &BOX, &part()).unwrap();
        assert_eq!(features.len(), 503);
        assert_eq!(fetcher.calls.get(), 2);
    }

    /// A service that ignores `startIndex` sends the same full page again and again.
    #[test]
    fn paging_stops_after_twenty_pages() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", collection(500));
        let err = get_features(&fetcher, &BFN, "t", &BOX, &part()).unwrap_err();
        assert_eq!(fetcher.calls.get(), 20);
        let last = query_url(&BFN, "t", &BOX, 19 * 500);
        assert!(
            matches!(&err, FetchError::Http { url, .. } if *url == last),
            "{err:?}"
        );
        assert!(err.to_string().contains("10000 features"), "{err}");
    }

    const FOREST: GeoJsonWfs = GeoJsonWfs {
        base: "https://www.brandenburg-forst.de/geoserver/IFUEK/ows",
        output_format: "gml32",
        bbox_crs_suffix: true,
    };

    /// A GML page of `n` features with one attribute and a square each, latitude first.
    fn gml_collection(n: usize) -> Vec<u8> {
        let members: String = (0..n)
            .map(|i| {
                format!(
                    "<wfs:member><IFUEK:AM.ForestManagementArea gml:id=\"a.{i}\">\
                     <IFUEK:rev_name>R{i}</IFUEK:rev_name><IFUEK:shape>\
                     <gml:Polygon srsName=\"EPSG:4326\"><gml:exterior><gml:LinearRing>\
                     <gml:posList>52.5 13.8 52.5 13.9 52.6 13.9 52.5 13.8</gml:posList>\
                     </gml:LinearRing></gml:exterior></gml:Polygon></IFUEK:shape>\
                     </IFUEK:AM.ForestManagementArea></wfs:member>"
                )
            })
            .collect();
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><wfs:FeatureCollection \
             xmlns:wfs=\"http://www.opengis.net/wfs/2.0\" xmlns:gml=\"http://www.opengis.net/gml/3.2\" \
             xmlns:IFUEK=\"x\" numberReturned=\"{n}\">{members}</wfs:FeatureCollection>"
        )
        .into_bytes()
    }

    /// A GML service is paged like a GeoJSON one, and its features carry their attributes
    /// and polygons as the service wrote them.
    #[test]
    fn gml_pages_become_geojson_features() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&query_url(&FOREST, "t", &BOX, 0), gml_collection(500));
        fetcher.insert(&query_url(&FOREST, "t", &BOX, 500), gml_collection(2));
        let features = get_gml_features(&fetcher, &FOREST, "t", &BOX, &part()).unwrap();
        assert_eq!(features.len(), 502);
        assert_eq!(fetcher.calls.get(), 2);
        assert_eq!(features[1]["properties"]["rev_name"], "R1");
        assert_eq!(features[1]["geometry"]["type"], "MultiPolygon");
        assert_eq!(
            features[1]["geometry"]["coordinates"][0][0][0],
            serde_json::json!([52.5, 13.8])
        );
        assert!(query_url(&FOREST, "t", &BOX, 0).contains("outputFormat=gml32"));
    }

    /// An exception report in place of the features is an error with the URL.
    #[test]
    fn a_gml_service_error_is_refused() {
        let fetcher = MapFetcher::default();
        let url = query_url(&FOREST, "t", &BOX, 0);
        fetcher.insert(
            &url,
            b"<ows:ExceptionReport><ows:Exception/></ows:ExceptionReport>".to_vec(),
        );
        let err = get_gml_features(&fetcher, &FOREST, "t", &BOX, &part()).unwrap_err();
        assert!(
            matches!(&err, FetchError::Http { url: u, .. } if *u == url),
            "{err:?}"
        );
    }

    /// A collection without `features` is empty, as Hamburg answers an empty box; anything
    /// else without them is still an error.
    #[test]
    fn a_collection_without_features_is_empty() {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &query_url(&BFN, "t", &BOX, 0),
            br#"{"type": "FeatureCollection"}"#.to_vec(),
        );
        assert!(
            get_features(&fetcher, &BFN, "t", &BOX, &part())
                .unwrap()
                .is_empty()
        );
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &query_url(&BFN, "t", &BOX, 0),
            br#"{"type": "Feature"}"#.to_vec(),
        );
        assert!(get_features(&fetcher, &BFN, "t", &BOX, &part()).is_err());
    }

    #[test]
    fn a_huge_response_is_refused() {
        let fetcher = MapFetcher::default();
        let url = query_url(&BFN, "t", &BOX, 0);
        fetcher.insert(&url, vec![b' '; 50 * 1024 * 1024 + 1]);
        let err = get_features(&fetcher, &BFN, "t", &BOX, &part()).unwrap_err();
        assert!(matches!(err, FetchError::TooLarge { .. }));
        assert!(err.to_string().contains(&url));
    }
}
