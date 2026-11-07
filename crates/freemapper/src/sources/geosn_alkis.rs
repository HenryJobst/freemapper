//! ALKIS buildings and land use from the GeoSN WFS ("ALKIS vereinfacht", GML 3.2 only), read
//! into GeoJSON features with the LGB property schema (the attribute names are the same).

use serde_json::Value;

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher, get_with_retries};
use crate::sources::alkis::{AlkisError, invalid};

pub const WFS_URL: &str = "https://geodienste.sachsen.de/aaa/public_alkis/vereinf/wfs";
pub const BUILDINGS_TYPE: &str = "ave:GebaeudeBauwerk";
pub const LAND_USE_TYPE: &str = "ave:NutzungFlurstueck";
/// Features per page; the server returns at most 20 000 per request.
#[cfg(not(test))]
const PAGE_COUNT: usize = 10_000;
#[cfg(test)]
const PAGE_COUNT: usize = 2;
/// `bez` of a Saxon square: GeoSN never records what a square is used for, so it stays a
/// candidate instead of becoming a paved map object.
pub const SQUARE_WITHOUT_FUNCTION: &str = "ohne Unterart (Sachsen)";

/// URL of the page of `type_name` features in `bbox` starting at feature `start`.
pub fn wfs_url(type_name: &str, bbox: &BoundingBox, start: usize) -> String {
    format!(
        "{WFS_URL}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}\
         &SRSNAME=urn:ogc:def:crs:EPSG::25833&BBOX={},{},{},{},urn:ogc:def:crs:EPSG::25833\
         &COUNT={PAGE_COUNT}&STARTINDEX={start}",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil()
    )
}

/// The features of a WFS answer as GeoJSON features; an answer that is not a
/// `wfs:FeatureCollection` (an HTML error page, an exception report) is an error.
pub fn parse_gml(text: &str) -> Result<Vec<Value>, AlkisError> {
    use quick_xml::events::Event;
    let refused = || {
        let start: String = text.trim_start().chars().take(80).collect();
        invalid(format!("not a WFS FeatureCollection: {start}"))
    };
    let mut reader = quick_xml::Reader::from_str(text);
    let mut features = Vec::new();
    // Local names of the open elements below the collection's root.
    let mut path: Vec<String> = Vec::new();
    let mut root_seen = false;
    let mut feature: Option<GmlFeature> = None;
    let mut text_buf = String::new();
    loop {
        let event = reader
            .read_event()
            .map_err(|e| invalid(format!("GML: {e}")))?;
        match event {
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                if !root_seen {
                    if name != "FeatureCollection" {
                        return Err(refused());
                    }
                    root_seen = true;
                    continue;
                }
                // wfs:member / <FeatureType> / property …
                if path.len() == 1 && feature.is_none() {
                    feature = Some(GmlFeature::default());
                } else if let Some(f) = feature.as_mut() {
                    match name.as_str() {
                        "Polygon" => f.polygons.push(Vec::new()),
                        "LinearRing" => {
                            if let Some(polygon) = f.polygons.last_mut() {
                                polygon.push(Vec::new());
                            }
                        }
                        _ => {}
                    }
                }
                text_buf.clear();
                path.push(name);
            }
            Event::Empty(e) => {
                if !root_seen {
                    return Err(refused());
                }
                let _ = e;
            }
            Event::Text(t) => text_buf.push_str(t.as_ref()),
            Event::GeneralRef(r) => {
                let name: &str = r.as_ref();
                let resolved = match name {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    _ => name
                        .strip_prefix("#x")
                        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                        .or_else(|| name.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                        .and_then(char::from_u32),
                };
                text_buf.extend(resolved);
            }
            Event::End(_) => {
                let Some(name) = path.pop() else { continue };
                if let Some(f) = feature.as_mut() {
                    match path.len() {
                        // The end of the feature element itself.
                        1 => {
                            if let Some(done) = feature.take() {
                                features.push(done.into_geojson());
                            }
                        }
                        // A direct child of the feature element: an attribute.
                        2 if name != "geometrie" && name != "identifier" => {
                            let value = text_buf.trim();
                            if !value.is_empty() {
                                f.properties.insert(name, Value::from(value));
                            }
                        }
                        _ if name == "posList" => {
                            let numbers: Vec<f64> = text_buf
                                .split_ascii_whitespace()
                                .filter_map(|v| v.parse().ok())
                                .collect();
                            if let Some(ring) = f.polygons.last_mut().and_then(|p| p.last_mut()) {
                                ring.extend(numbers.chunks_exact(2).map(|c| vec![c[0], c[1]]));
                            }
                        }
                        _ => {}
                    }
                }
                text_buf.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !root_seen {
        return Err(refused());
    }
    Ok(features)
}

/// One feature while it is read: its attributes and its polygons (rings of E, N pairs).
#[derive(Default)]
struct GmlFeature {
    properties: serde_json::Map<String, Value>,
    polygons: Vec<Vec<Vec<Vec<f64>>>>,
}

impl GmlFeature {
    fn into_geojson(self) -> Value {
        serde_json::json!({
            "type": "Feature",
            "id": self.properties.get("oid").cloned().unwrap_or(Value::Null),
            "geometry": { "type": "MultiPolygon", "coordinates": self.polygons },
            "properties": self.properties,
        })
    }
}

/// The element name without its namespace prefix.
fn local(name: &str) -> String {
    name.rsplit(':').next().unwrap_or_default().to_string()
}

/// All features of `type_name` in `bbox`, page by page; a feature repeated on a later page
/// (same `oid`) is kept once, and a page without new features ends the paging.
pub fn fetch_wfs(
    fetcher: &dyn Fetcher,
    type_name: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut start = 0;
    loop {
        let url = wfs_url(type_name, bbox, start);
        let page = get_with_retries(fetcher, &url, |bytes| {
            parse_gml(&String::from_utf8_lossy(bytes))
                .map_err(|e| FetchError::Alkis(invalid(format!("{url}: {}", e.message))))
        })?;
        let full = page.len() >= PAGE_COUNT;
        let mut new = false;
        for feature in page {
            let fresh = match feature["properties"]["oid"].as_str() {
                Some(oid) => seen.insert(oid.to_string()),
                None => true,
            };
            if fresh {
                new = true;
                features.push(feature);
            }
        }
        if !full || !new {
            return Ok(features);
        }
        start += PAGE_COUNT;
    }
}

/// The Saxon buildings in `bbox`.
pub fn fetch_buildings(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    fetch_wfs(fetcher, BUILDINGS_TYPE, bbox)
}

/// The Saxon land use in `bbox`; squares without `bez` get [`SQUARE_WITHOUT_FUNCTION`].
pub fn fetch_land_use(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<Vec<Value>, FetchError> {
    Ok(fetch_wfs(fetcher, LAND_USE_TYPE, bbox)?
        .into_iter()
        .map(normalise_land_use)
        .collect())
}

fn normalise_land_use(mut feature: Value) -> Value {
    let properties = &mut feature["properties"];
    if properties["nutzart"] == "Platz" && properties.get("bez").is_none() {
        properties["bez"] = Value::from(SQUARE_WITHOUT_FUNCTION);
    }
    feature
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const BUILDINGS: &str = include_str!("../../../../tests/fixtures/saxony-buildings.gml");
    pub const LAND_USE: &str = include_str!("../../../../tests/fixtures/saxony-land-use.gml");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::sources::alkis::{parse_buildings, parse_land_use};

    /// Evangelisches Kreuzgymnasium Dresden.
    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 415_800.0,
        min_n: 5_655_600.0,
        max_e: 416_150.0,
        max_n: 5_655_900.0,
    };

    fn collection(features: &[Value]) -> String {
        serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
    }

    #[test]
    fn url_matches_the_wfs() {
        assert_eq!(
            wfs_url(BUILDINGS_TYPE, &SCHOOL, 20),
            "https://geodienste.sachsen.de/aaa/public_alkis/vereinf/wfs?SERVICE=WFS&VERSION=2.0.0\
             &REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25833\
             &BBOX=415800,5655600,416150,5655900,urn:ogc:def:crs:EPSG::25833&COUNT=2&STARTINDEX=20"
        );
    }

    #[test]
    fn gml_buildings_parse_to_lgb_schema() {
        let features = parse_gml(BUILDINGS).unwrap();
        assert_eq!(features.len(), 3);
        let parsed = parse_buildings(&collection(&features)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        let school = &parsed.buildings[0];
        assert_eq!(school.oid, "DESNALK0CrM0003UBL");
        assert_eq!(school.kind, "Gebäude");
        assert_eq!(school.function, "Allgemein bildende Schule");
        assert_eq!(school.updated, "2024-01-04".parse().ok());
        assert!(!school.underground);
        let ring = &school.footprint.0[0].exterior().0;
        assert!((ring[0].x - 416_159.372).abs() < 1e-6 && (ring[0].y - 5_655_892.09).abs() < 1e-6);
        let courtyard = parsed
            .buildings
            .iter()
            .find(|b| b.footprint.0.iter().any(|p| !p.interiors().is_empty()))
            .expect("one building has a courtyard");
        assert_eq!(courtyard.footprint.0[0].interiors().len(), 1);
        assert_eq!(features[0]["id"], "DESNALK0CrM0003UBL");
    }

    #[test]
    fn gml_land_use_parse() {
        let features: Vec<Value> = parse_gml(LAND_USE)
            .unwrap()
            .into_iter()
            .map(normalise_land_use)
            .collect();
        let land = parse_land_use(&collection(&features)).unwrap();
        let classes: Vec<(&str, Option<&str>)> = land
            .iter()
            .map(|l| (l.nutzart.as_str(), l.bez.as_deref()))
            .collect();
        assert_eq!(
            classes,
            [
                ("Fläche besonderer funktionaler Prägung", None),
                ("Platz", Some(SQUARE_WITHOUT_FUNCTION)),
                ("Straßenverkehr", None),
                (
                    "Industrie- und Gewerbefläche",
                    Some("Handel und Dienstleistung")
                ),
            ]
        );
        assert!(land.iter().all(|l| l.updated.is_some()));
    }

    #[test]
    fn saxon_squares_stay_candidates() {
        let square = parse_gml(LAND_USE)
            .unwrap()
            .into_iter()
            .find(|f| f["properties"]["nutzart"] == "Platz")
            .unwrap();
        assert_eq!(
            normalise_land_use(square)["properties"]["bez"],
            SQUARE_WITHOUT_FUNCTION
        );
    }

    #[test]
    fn html_error_page_is_an_error() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            WFS_URL,
            b"<!DOCTYPE html><html><head><title>Seite kann nicht angezeigt werden</title>".to_vec(),
        );
        let err = fetch_wfs(&fetcher, BUILDINGS_TYPE, &SCHOOL).unwrap_err();
        assert!(
            err.to_string().contains("not a WFS FeatureCollection"),
            "{err}"
        );
        assert_eq!(fetcher.calls.get(), 3);
        let report = br#"<?xml version="1.0"?><ows:ExceptionReport xmlns:ows="http://www.opengis.net/ows/1.1"><ows:Exception exceptionCode="InvalidParameterValue"/></ows:ExceptionReport>"#;
        assert!(parse_gml(&String::from_utf8_lossy(report)).is_err());
    }

    #[test]
    fn entities_in_attributes_are_resolved() {
        let gml = r#"<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0"><wfs:member><GebaeudeBauwerk><oid>DESN1</oid><lagebeztxt>Hof &amp; Garten &#228;</lagebeztxt></GebaeudeBauwerk></wfs:member></wfs:FeatureCollection>"#;
        let features = parse_gml(gml).unwrap();
        assert_eq!(features[0]["properties"]["lagebeztxt"], "Hof & Garten ä");
    }

    #[test]
    fn empty_collection_has_no_features() {
        let empty = r#"<?xml version="1.0" encoding="utf-8"?><wfs:FeatureCollection numberReturned="0" xmlns:wfs="http://www.opengis.net/wfs/2.0"></wfs:FeatureCollection>"#;
        assert_eq!(parse_gml(empty).unwrap(), Vec::<Value>::new());
    }

    /// Serves the buildings fixture page by page (`COUNT` per page) by `STARTINDEX`.
    struct Paged {
        calls: std::cell::RefCell<Vec<usize>>,
    }

    impl Fetcher for Paged {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            let start: usize = url.rsplit("STARTINDEX=").next().unwrap().parse().unwrap();
            self.calls.borrow_mut().push(start);
            let head = &BUILDINGS[..BUILDINGS.find("<wfs:member>").unwrap()];
            let members: Vec<&str> = BUILDINGS
                .split("<wfs:member>")
                .skip(1)
                .map(|m| m.split("</wfs:member>").next().unwrap())
                .collect();
            let page: String = members
                .iter()
                .skip(start)
                .take(PAGE_COUNT)
                .map(|m| format!("<wfs:member>{m}</wfs:member>"))
                .collect();
            Ok(format!("{head}{page}</wfs:FeatureCollection>").into_bytes())
        }
    }

    #[test]
    fn full_pages_are_followed() {
        let fetcher = Paged {
            calls: std::cell::RefCell::new(Vec::new()),
        };
        let features = fetch_wfs(&fetcher, BUILDINGS_TYPE, &SCHOOL).unwrap();
        assert_eq!(features.len(), 3);
        assert_eq!(*fetcher.calls.borrow(), [0, 2]);
    }

    #[test]
    fn repeated_pages_end_the_paging() {
        // A server that ignores STARTINDEX answers every page with the same features.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(WFS_URL, BUILDINGS.as_bytes().to_vec());
        let features = fetch_wfs(&fetcher, BUILDINGS_TYPE, &SCHOOL).unwrap();
        assert_eq!(features.len(), 3);
        assert_eq!(fetcher.calls.get(), 2);
    }
}
