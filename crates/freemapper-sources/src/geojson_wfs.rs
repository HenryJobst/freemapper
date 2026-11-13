//! Features from a WFS 2.0 that answers GeoJSON (the BfN protected areas, the BKG state borders
//! VG250), asked by a box in WGS 84 and read page by page.

use serde_json::Value;

use crate::fetch::{FetchError, Fetcher, get_with_retries, too_large};

/// Features asked for per request.
const PAGE_SIZE: usize = 500;

/// The largest answer read; a larger one is a misbehaving server.
const MAX_BYTES: u64 = 50 * 1024 * 1024;

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
/// 500 until one has fewer; an answer over 50 MB is refused with the URL.
pub fn get_features(
    fetcher: &dyn Fetcher,
    service: &GeoJsonWfs,
    type_name: &str,
    bbox: &LatLonBox,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    loop {
        let url = query_url(service, type_name, bbox, features.len());
        let page = get_with_retries(fetcher, &url, |bytes| {
            if bytes.len() as u64 > MAX_BYTES {
                return Err(too_large(&url, MAX_BYTES));
            }
            read_page(&url, bytes)
        })?;
        let short = page.len() < PAGE_SIZE;
        features.extend(page);
        if short {
            return Ok(features);
        }
    }
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
        _ => Err(not_geojson(
            "the answer is not a FeatureCollection".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

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
        let features = get_features(&fetcher, &BFN, "t", &BOX).unwrap();
        assert_eq!(features.len(), 503);
        assert_eq!(fetcher.calls.get(), 2);
    }

    #[test]
    fn a_huge_response_is_refused() {
        let fetcher = MapFetcher::default();
        let url = query_url(&BFN, "t", &BOX, 0);
        fetcher.insert(&url, vec![b' '; 50 * 1024 * 1024 + 1]);
        let err = get_features(&fetcher, &BFN, "t", &BOX).unwrap_err();
        assert!(matches!(err, FetchError::TooLarge { .. }));
        assert!(err.to_string().contains(&url));
    }
}
