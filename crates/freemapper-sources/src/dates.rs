//! The date of a raster, asked of a standard service: a WFS that answers GeoJSON, a STAC API,
//! or the feature info of a WMS. Each asks at the centre of the map area. The states that have
//! such a service call these readers with their addresses; a manifest names them as the `date`
//! of a raster table.

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::{BoundingBox, Crs, LatLon, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher, get_with_retries, one_line};
use crate::wcs::exception_text;
use crate::wms::mm;

fn centre(bbox: &BoundingBox) -> (f64, f64) {
    (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    )
}

/// The GetFeature of the features of `layer` within 10 m of the centre of `bbox` (in `crs`),
/// answered as GeoJSON.
pub fn wfs_feature_url(url: &str, layer: &str, crs: Crs, bbox: &BoundingBox) -> String {
    let (e, n) = centre(bbox);
    format!(
        "{url}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
         &TYPENAMES={layer}&OUTPUTFORMAT=GEOJSON\
         &BBOX={},{},{},{},urn:ogc:def:crs:EPSG::{}",
        (e - 10.0).floor(),
        (n - 10.0).floor(),
        (e + 10.0).ceil(),
        (n + 10.0).ceil(),
        crs.epsg()
    )
}

/// The value of `field` of the first feature of `layer` at the centre of `bbox` (in `crs`), as
/// the service writes it; `None` when the service does not tell it. A failed query is asked
/// again like any other, since a date that is missing costs a second download of the raster.
pub fn wfs_feature_date(
    fetcher: &dyn Fetcher,
    (url, layer, field): (&str, &str, &str),
    crs: Crs,
    bbox: &BoundingBox,
) -> Option<String> {
    let url = wfs_feature_url(url, layer, crs, bbox);
    get_with_retries(fetcher, &url, |bytes| {
        let text = String::from_utf8_lossy(bytes);
        serde_json::from_str::<Value>(&text)
            .ok()
            .filter(|answer| answer["features"].is_array())
            .map(|_| parse_feature_field(&text, field))
            .ok_or_else(|| FetchError::Http {
                url: url.clone(),
                message: format!("not a tile index answer: {}", exception_text(bytes)),
            })
    })
    .ok()
    .flatten()
}

/// The text in `field` of the first feature of a GeoJSON answer, if it has one.
pub fn parse_feature_field(json: &str, field: &str) -> Option<String> {
    let answer: Value = serde_json::from_str(json).ok()?;
    let date = answer["features"].get(0)?["properties"][field].as_str()?;
    (!date.is_empty()).then(|| date.to_string())
}

/// A date as the records write it: `29.04.2026`.
pub fn day_month_year(date: NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}

/// A STAC item: its id and the date of its `datetime`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StacItem {
    pub id: String,
    pub date: NaiveDate,
}

/// Items per STAC page (servers cap `limit`, LGLN's at 1000).
const STAC_LIMIT: u32 = 100;
/// Pages of one STAC search; a search at one point finds a dozen items.
pub const STAC_MAX_PAGES: usize = 10;

/// The STAC search of `host` at the centre of `bbox` (in `crs`): a box of ±10 m around it in
/// lon/lat, which the search wants; `None` when a corner has no lon/lat.
pub fn stac_search_url(host: &str, bbox: &BoundingBox, crs: Crs) -> Option<String> {
    let (e, n) = centre(bbox);
    let corners: Vec<_> = [(-10.0, -10.0), (10.0, -10.0), (-10.0, 10.0), (10.0, 10.0)]
        .into_iter()
        .map(|(de, dn)| {
            crs.inverse(ProjectedPoint {
                easting: e + de,
                northing: n + dn,
            })
        })
        .collect::<Option<Vec<LatLon>>>()?;
    let min = |f: fn(&LatLon) -> f64| corners.iter().map(f).fold(f64::MAX, f64::min);
    let max = |f: fn(&LatLon) -> f64| corners.iter().map(f).fold(f64::MIN, f64::max);
    Some(format!(
        "{host}/search?bbox={:.6},{:.6},{:.6},{:.6}&limit={STAC_LIMIT}",
        min(|p| p.lon),
        min(|p| p.lat),
        max(|p| p.lon),
        max(|p| p.lat)
    ))
}

/// The newest item (by `datetime`) of the STAC search at the centre of `bbox` (in `crs`),
/// following the `next` links (on `host` only, up to 10 pages); `None` when the search finds
/// nothing. Items without a readable `datetime` are passed over; paging ends at an empty page or
/// a page read before.
pub fn newest_stac_item(
    fetcher: &dyn Fetcher,
    host: &str,
    bbox: &BoundingBox,
    crs: Crs,
) -> Result<Option<StacItem>, FetchError> {
    let mut newest: Option<StacItem> = None;
    let mut visited: Vec<String> = Vec::new();
    let first = stac_search_url(host, bbox, crs).ok_or_else(|| FetchError::Http {
        url: format!("{host}/search"),
        message: "the centre of the map area has no lon/lat".to_string(),
    })?;
    let mut next = Some(first);
    while let Some(url) = next.take() {
        if visited.len() == STAC_MAX_PAGES {
            return Err(FetchError::Http {
                url,
                message: format!("the STAC search has more than {STAC_MAX_PAGES} pages"),
            });
        }
        let page = get_with_retries(fetcher, &url, |bytes| {
            serde_json::from_slice::<Value>(bytes)
                .ok()
                .filter(|page| page["features"].is_array())
                .ok_or_else(|| FetchError::Http {
                    url: url.clone(),
                    message: format!(
                        "not a STAC search answer: {}",
                        one_line(&String::from_utf8_lossy(bytes), 80)
                    ),
                })
        })?;
        let features = page["features"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        for item in features.iter().filter_map(stac_item) {
            if newest.as_ref().is_none_or(|n| item.date > n.date) {
                newest = Some(item);
            }
        }
        visited.push(url);
        if features.is_empty() {
            break;
        }
        let Some(href) = page["links"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|link| link["rel"] == "next")
            .and_then(|link| link["href"].as_str())
        else {
            break;
        };
        if !href.starts_with(&format!("{host}/")) {
            return Err(FetchError::Http {
                url: visited.pop().unwrap_or_default(),
                message: format!("the STAC next link leaves {host}: {}", one_line(href, 120)),
            });
        }
        next = Some(href.to_string()).filter(|href| !visited.contains(href));
    }
    Ok(newest)
}

/// The id and `properties.datetime` (`YYYY-MM-DDT00:00:00Z`) of a STAC item.
fn stac_item(feature: &Value) -> Option<StacItem> {
    let datetime = feature["properties"]["datetime"].as_str()?;
    let date = chrono::DateTime::parse_from_rfc3339(datetime)
        .ok()?
        .date_naive();
    Some(StacItem {
        id: feature["id"].as_str().unwrap_or_default().to_string(),
        date,
    })
}

/// The GetFeatureInfo (as plain text) of `layer` of the WMS `url` at the centre of `bbox` (in
/// `crs`).
pub fn feature_info_url(url: &str, crs: Crs, layer: &str, bbox: &BoundingBox) -> String {
    let (e, n) = centre(bbox);
    format!(
        "{url}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={layer}\
         &QUERY_LAYERS={layer}&STYLES=&CRS=EPSG:{}&BBOX={},{},{},{}\
         &WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain",
        crs.epsg(),
        mm(e - 100.0),
        mm(n - 100.0),
        mm(e + 100.0),
        mm(n + 100.0),
    )
}

/// The date (`DD.MM.YYYY`) in `field` of the feature info of `layer` at the centre of `bbox`;
/// `None` outside the coverage or when the answer has none.
pub fn feature_info_date(
    fetcher: &dyn Fetcher,
    (url, layer, field): (&str, &str, &str),
    crs: Crs,
    bbox: &BoundingBox,
) -> Option<String> {
    let url = feature_info_url(url, crs, layer, bbox);
    let bytes = get_with_retries(fetcher, &url, |b| Ok(b.to_vec())).ok()?;
    parse_feature_info_date(&String::from_utf8_lossy(&bytes), field)
}

/// `<field> = 'DD.MM.YYYY'` of a GetFeatureInfo text answer.
pub fn parse_feature_info_date(text: &str, field: &str) -> Option<String> {
    let rest = text.split(field).nth(1)?;
    let value = rest.split('\'').nth(1)?;
    let parts: Vec<&str> = value.split('.').collect();
    let [d, m, y] = parts[..] else { return None };
    let digits = |s: &str, n| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    (digits(d, 2) && digits(m, 2) && digits(y, 4)).then(|| format!("{d}.{m}.{y}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX: BoundingBox = BoundingBox {
        min_e: 706_440.0,
        min_n: 5_706_900.0,
        max_e: 706_460.0,
        max_n: 5_706_916.0,
    };

    /// The three lookups ask at the centre of the area, in the CRS and for the layer given.
    #[test]
    fn the_lookups_ask_at_the_centre_of_the_area() {
        assert_eq!(
            wfs_feature_url("https://geo.example.org/wfs", "ns:tiles", Crs::Utm33, &BOX),
            "https://geo.example.org/wfs?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
             &TYPENAMES=ns:tiles&OUTPUTFORMAT=GEOJSON\
             &BBOX=706440,5706898,706460,5706918,urn:ogc:def:crs:EPSG::25833"
        );
        assert_eq!(
            feature_info_url("https://geo.example.org/wms", Crs::Utm32, "info", &BOX),
            "https://geo.example.org/wms?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS=info\
             &QUERY_LAYERS=info&STYLES=&CRS=EPSG:25832&BBOX=706350,5706808,706550,5707008\
             &WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain"
        );
        let zone_32 = stac_search_url("https://stac.example.org", &BOX, Crs::Utm32).unwrap();
        let zone_33 = stac_search_url("https://stac.example.org", &BOX, Crs::Utm33).unwrap();
        assert!(
            zone_32.starts_with("https://stac.example.org/search?bbox=11."),
            "{zone_32}"
        );
        assert!(zone_32.ends_with("&limit=100"), "{zone_32}");
        // The same numbers in another zone are another place.
        assert_ne!(zone_32, zone_33);
    }

    #[test]
    fn a_date_is_read_from_the_field_named() {
        let answer =
            r#"{"features":[{"properties":{"BFDATUM":"29.04.2026","stand":"2025-03-01"}}]}"#;
        assert_eq!(
            parse_feature_field(answer, "BFDATUM").as_deref(),
            Some("29.04.2026")
        );
        assert_eq!(
            parse_feature_field(answer, "stand").as_deref(),
            Some("2025-03-01")
        );
        assert_eq!(parse_feature_field(answer, "other"), None);
        assert_eq!(parse_feature_field(r#"{"features":[]}"#, "BFDATUM"), None);
        let info =
            "Layer 'info'\n  Feature 1: \n    Tile = '32354_5642'\n    Flown = '11.05.2025'\n";
        assert_eq!(
            parse_feature_info_date(info, "Flown").as_deref(),
            Some("11.05.2025")
        );
        assert_eq!(parse_feature_info_date(info, "Tile"), None);
        assert_eq!(parse_feature_info_date(info, "Missing"), None);
        assert_eq!(
            day_month_year(NaiveDate::from_ymd_opt(2026, 4, 9).unwrap()),
            "09.04.2026"
        );
    }
}
