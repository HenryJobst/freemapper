//! ALKIS buildings and land use from the states' "ALKIS vereinfacht" WFS (GeoSN, LVermGeo, HVBG; GML 3.2
//! only), read into GeoJSON features with the LGB property schema (the attribute names are the
//! same).

use serde_json::Value;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, Fetcher, get_with_retries};
use crate::sources::alkis::{AlkisError, invalid};

/// An ALKIS-vereinfacht WFS of one state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WfsService {
    /// For messages, e.g. "GeoSN ALKIS".
    pub name: &'static str,
    pub url: &'static str,
    pub buildings_type: &'static str,
    pub land_use_type: &'static str,
    /// The CRS asked for and answered in.
    pub crs: Crs,
    /// `bez` given to squares without one: the state never records what a square is used for,
    /// so it stays a candidate instead of becoming a paved map object.
    pub square_bez: &'static str,
}

/// GeoSN, Saxony (EPSG:25833; land use per parcel part).
pub const GEOSN: WfsService = WfsService {
    name: "GeoSN ALKIS",
    url: "https://geodienste.sachsen.de/aaa/public_alkis/vereinf/wfs",
    buildings_type: "ave:GebaeudeBauwerk",
    land_use_type: "ave:NutzungFlurstueck",
    crs: Crs::Utm33,
    square_bez: "ohne Unterart (Sachsen)",
};

/// LVermGeo, Saxony-Anhalt (EPSG:25832; land use objects, not split by parcel).
pub const LVERMGEO: WfsService = WfsService {
    name: "LVermGeo ALKIS",
    url: "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ALKIS_WFS_OpenData/guest",
    buildings_type: "ave:GebaeudeBauwerk",
    land_use_type: "ave:Nutzung",
    crs: Crs::Utm32,
    square_bez: "ohne Unterart (Sachsen-Anhalt)",
};

/// HVBG, Hesse (EPSG:25832; land use per parcel part). `ave:Nutzung` carries no `nutzart` in
/// Hesse, so the parcel-part type is used.
pub const HVBG: WfsService = WfsService {
    name: "HVBG ALKIS",
    url: "https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs",
    buildings_type: "ave:GebaeudeBauwerk",
    land_use_type: "ave:NutzungFlurstueck",
    crs: Crs::Utm32,
    square_bez: "ohne Unterart (Hessen)",
};

/// Features per page; GeoSN returns at most 20 000 per request.
#[cfg(not(test))]
const PAGE_COUNT: usize = 10_000;
#[cfg(test)]
const PAGE_COUNT: usize = 2;

impl WfsService {
    /// URL of the page of `type_name` features in `bbox` (in [`WfsService::crs`]) starting at
    /// feature `start`.
    pub fn wfs_url(&self, type_name: &str, bbox: &BoundingBox, start: usize) -> String {
        format!(
            "{}&SORTBY=oid&COUNT={PAGE_COUNT}&STARTINDEX={start}",
            self.query_url(type_name, bbox)
        )
    }

    /// The query of `type_name` features in `bbox`, without paging.
    fn query_url(&self, type_name: &str, bbox: &BoundingBox) -> String {
        let epsg = self.crs.epsg();
        format!(
            "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}\
             &SRSNAME=urn:ogc:def:crs:EPSG::{epsg}&BBOX={},{},{},{},urn:ogc:def:crs:EPSG::{epsg}",
            self.url,
            bbox.min_e.floor(),
            bbox.min_n.floor(),
            bbox.max_e.ceil(),
            bbox.max_n.ceil()
        )
    }
}

/// `numberMatched` of a WFS answer; `None` when the server does not know it ("unknown").
pub fn number_matched(text: &str) -> Option<usize> {
    let start = text.find("numberMatched=\"")? + "numberMatched=\"".len();
    text[start..].split('"').next()?.parse().ok()
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
                // wfs:member / <FeatureType> / property …; wfs:boundedBy holds no feature.
                if path.len() == 1 && path[0] == "member" && feature.is_none() {
                    feature = Some(GmlFeature::default());
                } else if let Some(f) = feature.as_mut()
                    && f.in_geometry(&path)
                {
                    let dimension = e
                        .try_get_attribute("srsDimension")
                        .ok()
                        .flatten()
                        .map(|a| a.value.to_string());
                    if dimension.is_some_and(|d| d != "2")
                        || !GEOMETRY_ELEMENTS.contains(&name.as_str())
                    {
                        f.unsupported = true;
                    }
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
                let name = local(e.name().as_ref());
                if !root_seen {
                    // An empty answer is a self-closing collection.
                    return if name == "FeatureCollection" {
                        Ok(Vec::new())
                    } else {
                        Err(refused())
                    };
                }
                if let Some(f) = feature.as_mut()
                    && f.in_geometry(&path)
                {
                    f.unsupported = true;
                }
            }
            Event::CData(t) => text_buf.push_str(&t.into_inner()),
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
                                ring.extend(
                                    numbers.as_chunks::<2>().0.iter().map(|&[e, n]| vec![e, n]),
                                );
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

/// The GML elements of the 2D polygon geometries GeoSN sends; anything else (curves, arcs,
/// patches, `gml:pos`) is not read, and the feature is listed as skipped instead.
const GEOMETRY_ELEMENTS: [&str; 7] = [
    "MultiSurface",
    "surfaceMember",
    "Polygon",
    "exterior",
    "interior",
    "LinearRing",
    "posList",
];

/// One feature while it is read: its attributes and its polygons (rings of E, N pairs).
#[derive(Default)]
struct GmlFeature {
    properties: serde_json::Map<String, Value>,
    polygons: Vec<Vec<Vec<Vec<f64>>>>,
    /// The geometry holds something [`GEOMETRY_ELEMENTS`] do not cover.
    unsupported: bool,
}

impl GmlFeature {
    /// Whether the element opened at `path` lies inside the feature's `geometrie`.
    fn in_geometry(&self, path: &[String]) -> bool {
        path.len() >= 3 && path[2] == "geometrie"
    }

    fn into_geojson(self) -> Value {
        let geometry = if self.unsupported || self.polygons.is_empty() {
            Value::Null
        } else {
            serde_json::json!({ "type": "MultiPolygon", "coordinates": self.polygons })
        };
        serde_json::json!({
            "type": "Feature",
            "id": self.properties.get("oid").cloned().unwrap_or(Value::Null),
            "geometry": geometry,
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
    service: &WfsService,
    type_name: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    let name = service.name;
    let get = |url: &str| -> Result<String, FetchError> {
        get_with_retries(fetcher, url, |bytes| {
            let text = String::from_utf8_lossy(bytes).into_owned();
            parse_gml(&text)
                .map(|_| text)
                .map_err(|e| FetchError::Alkis(invalid(format!("{url}: {}", e.message))))
        })
        .map_err(|e| match e {
            FetchError::Refused { status, .. } => FetchError::Alkis(invalid(format!(
                "{name} service refused the request (HTTP {status}): {url}"
            ))),
            e => e,
        })
    };
    // The number of matches first, so that a page cap below PAGE_COUNT cannot cut the result.
    let matched = number_matched(&get(&format!(
        "{}&RESULTTYPE=hits",
        service.query_url(type_name, bbox)
    ))?);
    let mut features = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut start = 0;
    loop {
        let url = service.wfs_url(type_name, bbox, start);
        let page = parse_gml(&get(&url)?).map_err(FetchError::Alkis)?;
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
        let all = matched.is_some_and(|m| features.len() >= m);
        if all || !new || !full {
            break;
        }
        start += PAGE_COUNT;
    }
    if let Some(matched) = matched.filter(|&m| features.len() < m) {
        return Err(FetchError::Alkis(invalid(format!(
            "{name} returned {} of {matched} features in the area; run `freemapper fetch` again",
            features.len()
        ))));
    }
    Ok(features)
}

/// The buildings of `service` in `bbox` (in the service's CRS).
pub fn fetch_buildings(
    fetcher: &dyn Fetcher,
    service: &WfsService,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    fetch_wfs(fetcher, service, service.buildings_type, bbox)
}

/// The land use of `service` in `bbox`; squares without `bez` get the service's `square_bez`.
pub fn fetch_land_use(
    fetcher: &dyn Fetcher,
    service: &WfsService,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    Ok(fetch_wfs(fetcher, service, service.land_use_type, bbox)?
        .into_iter()
        .map(|f| normalise_land_use(f, service.square_bez))
        .collect())
}

fn normalise_land_use(mut feature: Value, square_bez: &str) -> Value {
    let properties = &mut feature["properties"];
    if properties["nutzart"] == "Platz" && properties.get("bez").is_none() {
        properties["bez"] = Value::from(square_bez);
    }
    feature
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const BUILDINGS: &str = include_str!("../../../../tests/fixtures/saxony-buildings.gml");
    pub const LAND_USE: &str = include_str!("../../../../tests/fixtures/saxony-land-use.gml");
    pub const ST_BUILDINGS: &str =
        include_str!("../../../../tests/fixtures/saxony-anhalt-buildings.gml");
    pub const ST_LAND_USE: &str =
        include_str!("../../../../tests/fixtures/saxony-anhalt-land-use.gml");
    pub const HE_BUILDINGS: &str = include_str!("../../../../tests/fixtures/hessen-buildings.gml");
    pub const HE_LAND_USE: &str = include_str!("../../../../tests/fixtures/hessen-land-use.gml");
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
            GEOSN.wfs_url(GEOSN.buildings_type, &SCHOOL, 20),
            "https://geodienste.sachsen.de/aaa/public_alkis/vereinf/wfs?SERVICE=WFS&VERSION=2.0.0\
             &REQUEST=GetFeature&TYPENAMES=ave:GebaeudeBauwerk&SRSNAME=urn:ogc:def:crs:EPSG::25833\
             &BBOX=415800,5655600,416150,5655900,urn:ogc:def:crs:EPSG::25833&SORTBY=oid&COUNT=2\
             &STARTINDEX=20"
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
            .map(|f| normalise_land_use(f, GEOSN.square_bez))
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
                ("Platz", Some(GEOSN.square_bez)),
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
            normalise_land_use(square, GEOSN.square_bez)["properties"]["bez"],
            GEOSN.square_bez
        );
    }

    #[test]
    fn html_error_page_is_an_error() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            GEOSN.url,
            b"<!DOCTYPE html><html><head><title>Seite kann nicht angezeigt werden</title>".to_vec(),
        );
        let err = fetch_wfs(&fetcher, &GEOSN, GEOSN.buildings_type, &SCHOOL).unwrap_err();
        assert!(
            err.to_string().contains("not a WFS FeatureCollection"),
            "{err}"
        );
        assert_eq!(fetcher.calls.get(), 3, "the hits query, retried");
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
    fn bounded_by_is_not_a_feature() {
        // Live answers start with the collection's envelope.
        let gml = BUILDINGS.replacen(
            "<wfs:member>",
            r#"<wfs:boundedBy><gml:Envelope srsName="urn:ogc:def:crs:EPSG::25833" srsDimension="2"><gml:lowerCorner>415777.883 5655572.608</gml:lowerCorner><gml:upperCorner>416192.975 5655938.59</gml:upperCorner></gml:Envelope></wfs:boundedBy>
<wfs:member>"#,
            1,
        );
        let features = parse_gml(&gml).unwrap();
        assert_eq!(features.len(), 3);
        assert!(features.iter().all(|f| f["properties"]["oid"].is_string()));
    }

    #[test]
    fn live_empty_answer_has_no_features() {
        // An empty answer is a self-closing root.
        let empty = include_str!("../../../../tests/fixtures/saxony-empty.gml");
        assert_eq!(parse_gml(empty).unwrap(), Vec::<Value>::new());
        let hits = include_str!("../../../../tests/fixtures/saxony-hits.gml");
        assert_eq!(number_matched(hits), Some(123));
        assert_eq!(number_matched(BUILDINGS), None, "numberMatched=unknown");
    }

    #[test]
    fn refused_request_names_the_service() {
        let fetcher = MapFetcher::default();
        fetcher.refusing.borrow_mut().push(GEOSN.url.to_string());
        let err = fetch_wfs(&fetcher, &GEOSN, GEOSN.buildings_type, &SCHOOL).unwrap_err();
        assert!(
            err.to_string()
                .contains("GeoSN ALKIS service refused the request (HTTP 400)"),
            "{err}"
        );
        assert_eq!(fetcher.calls.get(), 1);
    }

    fn with_geometry(geometry: &str) -> String {
        format!(
            r#"<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:gml="http://www.opengis.net/gml/3.2"><wfs:member><GebaeudeBauwerk><oid>DESN1</oid><gebnutzbez>Gebäude</gebnutzbez><geometrie>{geometry}</geometrie></GebaeudeBauwerk></wfs:member></wfs:FeatureCollection>"#
        )
    }

    #[test]
    fn unsupported_geometry_skips_the_feature() {
        let curve = with_geometry(
            r#"<gml:MultiSurface srsDimension="2"><gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>0 0 10 0 10 10 0 10 0 0</gml:posList></gml:LinearRing></gml:exterior><gml:interior><gml:Ring><gml:curveMember><gml:Curve><gml:segments><gml:Arc><gml:posList>2 2 4 4 2 6</gml:posList></gml:Arc></gml:segments></gml:Curve></gml:curveMember></gml:Ring></gml:interior></gml:Polygon></gml:surfaceMember></gml:MultiSurface>"#,
        );
        let three_d = with_geometry(
            r#"<gml:MultiSurface srsDimension="3"><gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing><gml:posList srsDimension="3">0 0 1 10 0 1 10 10 1 0 0 1</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember></gml:MultiSurface>"#,
        );
        for gml in [curve, three_d] {
            let features = parse_gml(&gml).unwrap();
            assert_eq!(features[0]["geometry"], Value::Null, "{gml}");
            let parsed = parse_buildings(&collection(&features)).unwrap();
            assert!(parsed.buildings.is_empty());
            assert_eq!(parsed.skipped.len(), 1);
        }
    }

    #[test]
    fn cdata_text_is_read() {
        let gml = r#"<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0"><wfs:member><GebaeudeBauwerk><oid>DESN1</oid><name><![CDATA[Haus <A>]]></name></GebaeudeBauwerk></wfs:member></wfs:FeatureCollection>"#;
        assert_eq!(parse_gml(gml).unwrap()[0]["properties"]["name"], "Haus <A>");
    }

    #[test]
    fn fewer_features_than_matched_is_an_error() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(GEOSN.url, BUILDINGS.as_bytes().to_vec());
        fetcher.insert_prefix(
            &format!("{}&RESULTTYPE=hits", GEOSN.wfs_url(GEOSN.buildings_type, &SCHOOL, 0).split("&SORTBY").next().unwrap()),
            br#"<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0" numberReturned="0" numberMatched="5"/>"#.to_vec(),
        );
        let err = fetch_wfs(&fetcher, &GEOSN, GEOSN.buildings_type, &SCHOOL).unwrap_err();
        assert!(err.to_string().contains("3 of 5"), "{err}");
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
            if url.contains("RESULTTYPE=hits") {
                return Ok(br#"<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0" numberReturned="0" numberMatched="3"/>"#.to_vec());
            }
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
        let features = fetch_wfs(&fetcher, &GEOSN, GEOSN.buildings_type, &SCHOOL).unwrap();
        assert_eq!(features.len(), 3);
        assert_eq!(*fetcher.calls.borrow(), [0, 2]);
    }

    #[test]
    fn repeated_pages_end_the_paging() {
        // A server that ignores STARTINDEX answers every page with the same features.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(GEOSN.url, BUILDINGS.as_bytes().to_vec());
        let features = fetch_wfs(&fetcher, &GEOSN, GEOSN.buildings_type, &SCHOOL).unwrap();
        assert_eq!(features.len(), 3);
        assert_eq!(fetcher.calls.get(), 3, "hits and two pages");
    }

    #[test]
    fn lvermgeo_buildings_parse() {
        let features = parse_gml(ST_BUILDINGS).unwrap();
        let parsed = parse_buildings(&collection(&features)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        let latina = &parsed.buildings[0];
        assert_eq!(latina.oid, "DESTLIKA0001MuQqBL");
        assert_eq!(latina.function, "Allgemein bildende Schule");
        // Native EPSG:25832 coordinates.
        let corner = latina.footprint.0[0].exterior().0[0];
        assert!((700_000.0..710_000.0).contains(&corner.x), "{corner:?}");
        let garage = parsed
            .buildings
            .iter()
            .find(|b| b.function == "Tiefgarage")
            .unwrap();
        assert_eq!(garage.kind, "Bauteil");
        assert!(garage.underground);
    }

    #[test]
    fn lvermgeo_land_use_squares_stay_candidates() {
        let features: Vec<Value> = parse_gml(ST_LAND_USE)
            .unwrap()
            .into_iter()
            .map(|f| normalise_land_use(f, LVERMGEO.square_bez))
            .collect();
        let land = parse_land_use(&collection(&features)).unwrap();
        let classes: Vec<(&str, Option<&str>)> = land
            .iter()
            .map(|l| (l.nutzart.as_str(), l.bez.as_deref()))
            .collect();
        assert_eq!(
            classes,
            [
                (
                    "Fläche besonderer funktionaler Prägung",
                    Some("Öffentliche Zwecke")
                ),
                ("Platz", Some("ohne Unterart (Sachsen-Anhalt)")),
                ("Straßenverkehr", None),
                ("Wohnbaufläche", None),
            ]
        );
    }

    #[test]
    fn lvermgeo_urls_are_in_zone_32() {
        let bbox = BoundingBox {
            min_e: 706_350.0,
            min_n: 5_706_800.0,
            max_e: 706_550.0,
            max_n: 5_707_000.0,
        };
        let url = LVERMGEO.wfs_url(LVERMGEO.land_use_type, &bbox, 0);
        assert!(url.starts_with(LVERMGEO.url), "{url}");
        assert!(url.contains("TYPENAMES=ave:Nutzung&"), "{url}");
        assert!(url.contains("SRSNAME=urn:ogc:def:crs:EPSG::25832"), "{url}");
        assert!(
            url.contains("BBOX=706350,5706800,706550,5707000,urn:ogc:def:crs:EPSG::25832"),
            "{url}"
        );
    }

    #[test]
    fn hvbg_buildings_parse() {
        let features = parse_gml(HE_BUILDINGS).unwrap();
        let parsed = parse_buildings(&collection(&features)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        let school = parsed
            .buildings
            .iter()
            .find(|b| b.oid == "DEHE06200002dyI4BL")
            .unwrap();
        assert_eq!(school.function, "Gebäude für öffentliche Zwecke");
        // Native EPSG:25832 coordinates.
        let corner = school.footprint.0[0].exterior().0[0];
        assert!((476_000.0..477_000.0).contains(&corner.x), "{corner:?}");
        let garage = parsed
            .buildings
            .iter()
            .find(|b| b.function == "Tiefgarage")
            .unwrap();
        assert_eq!(garage.kind, "Bauteil");
        assert!(garage.underground, "rellage marks it underground");
        assert!(!school.underground);
    }

    #[test]
    fn hvbg_land_use_from_nutzung_flurstueck() {
        assert_eq!(HVBG.land_use_type, "ave:NutzungFlurstueck");
        let features = parse_gml(HE_LAND_USE).unwrap();
        let land = parse_land_use(&collection(&features)).unwrap();
        assert_eq!(land.len(), features.len());
        let nutzarten: Vec<&str> = land.iter().map(|l| l.nutzart.as_str()).collect();
        for want in ["Straßenverkehr", "Wohnbaufläche", "Platz"] {
            assert!(nutzarten.contains(&want), "{want} in {nutzarten:?}");
        }
        assert!(features.iter().all(|f| !f["geometry"].is_null()));
    }

    #[test]
    fn hvbg_squares_without_bez_stay_candidates() {
        let features: Vec<Value> = parse_gml(HE_LAND_USE)
            .unwrap()
            .into_iter()
            .map(|f| normalise_land_use(f, HVBG.square_bez))
            .collect();
        let land = parse_land_use(&collection(&features)).unwrap();
        let squares: Vec<Option<&str>> = land
            .iter()
            .filter(|l| l.nutzart == "Platz")
            .map(|l| l.bez.as_deref())
            .collect();
        assert!(squares.contains(&Some("Parkplatz")), "{squares:?}");
        assert!(
            squares.contains(&Some("ohne Unterart (Hessen)")),
            "{squares:?}"
        );
        assert!(squares.iter().all(Option::is_some), "{squares:?}");
    }

    #[test]
    fn hvbg_urls_are_in_zone_32() {
        let bbox = BoundingBox {
            min_e: 476_320.0,
            min_n: 5_552_421.5,
            max_e: 476_817.2,
            max_n: 5_552_753.0,
        };
        let url = HVBG.wfs_url(HVBG.land_use_type, &bbox, 0);
        assert!(
            url.starts_with("https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs?"),
            "{url}"
        );
        assert!(url.contains("TYPENAMES=ave:NutzungFlurstueck&"), "{url}");
        assert!(
            url.contains("BBOX=476320,5552421,476818,5552753,urn:ogc:def:crs:EPSG::25832"),
            "{url}"
        );
        assert!(url.contains("SORTBY=oid"), "{url}");
    }

    #[test]
    fn exception_with_status_200_is_an_error() {
        let report = r#"<?xml version="1.0" encoding="UTF-8"?><ServiceExceptionReport version="1.2.0" xmlns="http://www.opengis.net/ogc"><ServiceException>Could not process client request</ServiceException></ServiceExceptionReport>"#;
        assert!(parse_gml(report).is_err());
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(LVERMGEO.url, report.as_bytes().to_vec());
        let bbox = BoundingBox {
            min_e: 706_350.0,
            min_n: 5_706_800.0,
            max_e: 706_550.0,
            max_n: 5_707_000.0,
        };
        let err = fetch_wfs(&fetcher, &LVERMGEO, LVERMGEO.buildings_type, &bbox).unwrap_err();
        assert!(
            err.to_string().contains("not a WFS FeatureCollection"),
            "{err}"
        );
    }
}
