//! ALKIS buildings and land use from the states' "ALKIS vereinfacht" WFS (GeoSN, LVermGeo, HVBG,
//! LVermGeo SH; GML 3.2 only), read into GeoJSON features with the LGB property schema (the
//! attribute names are the same).

use serde_json::Value;
use std::borrow::Cow;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, Fetcher, get_with_retries};
use crate::ogc_api::{AlkisError, invalid};

/// An ALKIS-vereinfacht WFS of one state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfsService {
    /// For messages, e.g. "GeoSN ALKIS".
    pub name: Cow<'static, str>,
    pub url: Cow<'static, str>,
    pub buildings_type: Cow<'static, str>,
    pub land_use_type: Cow<'static, str>,
    /// The CRS asked for and answered in.
    pub crs: Crs,
    /// `bez` given to squares without one: the state never records what a square is used for,
    /// so it stays a candidate instead of becoming a paved map object. `None` where the
    /// state's OGC API keeps squares without `bez` as they are (the WFS then does the same).
    pub square_bez: Option<Cow<'static, str>>,
    /// Whether the pages are asked sorted by `oid`. `false` for a service that refuses
    /// `SORTBY` (Saarland's ArcGIS server): its pages are read in the order it gives them.
    pub sorted: bool,
}

/// GeoSN, Saxony (EPSG:25833; land use per parcel part).
pub const GEOSN: WfsService = WfsService {
    name: Cow::Borrowed("GeoSN ALKIS"),
    url: Cow::Borrowed("https://geodienste.sachsen.de/aaa/public_alkis/vereinf/wfs"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
    crs: Crs::Utm33,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Sachsen)")),
    sorted: true,
};

/// LVermGeo, Saxony-Anhalt (EPSG:25832; land use objects, not split by parcel).
pub const LVERMGEO: WfsService = WfsService {
    name: Cow::Borrowed("LVermGeo ALKIS"),
    url: Cow::Borrowed(
        "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ALKIS_WFS_OpenData/guest",
    ),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:Nutzung"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Sachsen-Anhalt)")),
    sorted: true,
};

/// HVBG, Hesse (EPSG:25832; land use per parcel part). `ave:Nutzung` carries no `nutzart` in
/// Hesse, so the parcel-part type is used.
pub const HVBG: WfsService = WfsService {
    name: Cow::Borrowed("HVBG ALKIS"),
    url: Cow::Borrowed("https://www.gds.hessen.de/wfs2/aaa-suite/cgi-bin/alkis/vereinf/wfs"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Hessen)")),
    sorted: true,
};

/// Geobasis NRW, Nordrhein-Westfalen (EPSG:25832; land use per parcel part). Same schema and
/// LGB spellings as Hesse, but with detailed `funktion` values. Errors come as a bare HTTP 400.
pub const GEOBASIS_NRW: WfsService = WfsService {
    name: Cow::Borrowed("Geobasis NRW ALKIS"),
    url: Cow::Borrowed("https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Nordrhein-Westfalen)")),
    sorted: true,
};

/// LGLN, Niedersachsen (EPSG:25832; land use per parcel part). Same schema and LGB spellings as
/// Hesse; underground garages are `Gebäude` with `rellage` "Unter der Erdoberfläche". The service
/// cuts an answer without `COUNT` silently at 10000 and refuses more than 10000.
pub const LGLN: WfsService = WfsService {
    name: Cow::Borrowed("LGLN ALKIS"),
    url: Cow::Borrowed("https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Niedersachsen)")),
    sorted: true,
};

/// LVermGeo RP, Rheinland-Pfalz (EPSG:25832; MapServer). Land use is `ave:Nutzung` (per object, with
/// `nutzart` and `bez`; there is no `NutzungFlurstueck`), and `aktualit` sits inside a
/// `gml:timePosition`. Names of several objects are joined with `;` and kept as they are.
pub const LVERMGEO_RP: WfsService = WfsService {
    name: Cow::Borrowed("LVermGeo RP ALKIS"),
    url: Cow::Borrowed("https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:Nutzung"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Rheinland-Pfalz)")),
    sorted: true,
};

/// LVermGeo SH, Schleswig-Holstein (EPSG:25832; land use per parcel part). The VwG service
/// (named for its administrative-boundary theme) serves every type with normal queries; the
/// advertised `WFS_SH_ALKIS_vereinf_OpenGBD` refuses them (stored queries only).
pub const LVERMGEO_SH: WfsService = WfsService {
    name: Cow::Borrowed("LVermGeo SH ALKIS"),
    url: Cow::Borrowed("https://dienste.gdi-sh.de/WFS_SH_ALKIS_VWG_OpenGBD"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
    crs: Crs::Utm32,
    square_bez: Some(Cow::Borrowed("ohne Unterart (Schleswig-Holstein)")),
    sorted: true,
};

/// LGV, Hamburg (EPSG:25832): the classic WFS beside the OGC API, with the same data. Used when
/// the OGC API fails; its land use keeps squares without `bez` as they are (the OGC API path
/// has no `square_bez` either), so `square_bez` is `None`.
pub const LGV: WfsService = WfsService {
    name: Cow::Borrowed("LGV ALKIS (WFS)"),
    url: Cow::Borrowed("https://geodienste.hamburg.de/WFS_HH_ALKIS_vereinfacht"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
    crs: Crs::Utm32,
    square_bez: None,
    sorted: true,
};

/// LAiV M-V, Mecklenburg-Vorpommern (EPSG:25833; land use per parcel part): the classic WFS beside
/// the OGC API, with the same data and the LGB spellings. Used when the OGC API fails. Only
/// 25833 is accepted (25832 gives HTTP 400).
pub const LAIV_MV: WfsService = WfsService {
    name: Cow::Borrowed("LAiV M-V ALKIS (WFS)"),
    url: Cow::Borrowed("https://www.geodaten-mv.de/dienste/alkis_wfs_einfach"),
    buildings_type: Cow::Borrowed("ave:GebaeudeBauwerk"),
    land_use_type: Cow::Borrowed("ave:NutzungFlurstueck"),
    crs: Crs::Utm33,
    square_bez: None,
    sorted: true,
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
        self.page_url(type_name, bbox, start, "oid")
    }

    /// [`Self::wfs_url`] with the pages sorted by the attribute `key`.
    fn page_url(&self, type_name: &str, bbox: &BoundingBox, start: usize, key: &str) -> String {
        let sort = if self.sorted {
            format!("&SORTBY={key}")
        } else {
            String::new()
        };
        format!(
            "{}{sort}&COUNT={PAGE_COUNT}&STARTINDEX={start}",
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
            Event::GeneralRef(r) => text_buf.extend(crate::xml::resolve_entity(r.as_ref())),
            Event::End(_) => {
                let Some(name) = path.pop() else { continue };
                let name = if path.len() == 2 {
                    attribute(&name)
                } else {
                    name
                };
                if let Some(f) = feature.as_mut() {
                    match path.len() {
                        // The end of the feature element itself.
                        1 => {
                            if let Some(done) = feature.take() {
                                features.push(done.into_geojson());
                            }
                        }
                        // LVermGeo RP wraps the date: `<aktualit><gml:timePosition>…` (a `TimeInstant` may sit between).
                        depth
                            if depth >= 3
                                && name == "timePosition"
                                && attribute(&path[2]) == "aktualit" =>
                        {
                            let value = text_buf.trim();
                            if !value.is_empty() {
                                f.properties.insert(attribute(&path[2]), Value::from(value));
                            }
                        }
                        // A direct child of the feature element: an attribute.
                        2 if !is_geometry(&name) && name != "identifier" => {
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
        path.len() >= 3 && is_geometry(&path[2])
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

/// The name of an attribute as the simplified schema writes it. A server that writes the
/// names all in upper case (`OID`, `AKTUALIT`; Saarland's ArcGIS server) means the same
/// attributes; a name with a lower-case letter is taken as it stands.
fn attribute(name: &str) -> String {
    if name.bytes().any(|byte| byte.is_ascii_lowercase()) {
        name.to_string()
    } else {
        name.to_lowercase()
    }
}

/// Whether the child `name` of a feature holds its geometry: `geometrie` in the schema,
/// `SHAPE` from an ArcGIS server, `geom` in the views of Baden-Württemberg.
fn is_geometry(name: &str) -> bool {
    matches!(attribute(name).as_str(), "geometrie" | "shape" | "geom")
}

/// The element name without its namespace prefix.
fn local(name: &str) -> String {
    name.rsplit(':').next().unwrap_or_default().to_string()
}

/// All features of `type_name` in `bbox`, page by page; a feature repeated on a later page
/// (same `oid`) is kept once, and a page without new features ends the paging. The features
/// come in the order of their `oid`: from the service, or sorted here when it does not sort.
pub fn fetch_wfs(
    fetcher: &dyn Fetcher,
    service: &WfsService,
    type_name: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    fetch_wfs_by(fetcher, service, type_name, bbox, "oid")
}

/// [`fetch_wfs`] for a service outside the simplified schema: `key` is the attribute that
/// names a feature there (`gml_id` in Baden-Württemberg), by which the pages are sorted and a
/// repeated feature is known. The features keep the attributes of the service; who asks turns
/// them into the schema.
pub fn fetch_wfs_by(
    fetcher: &dyn Fetcher,
    service: &WfsService,
    type_name: &str,
    bbox: &BoundingBox,
    key: &str,
) -> Result<Vec<Value>, FetchError> {
    let name = &service.name;
    let get = |url: &str| -> Result<String, FetchError> {
        get_with_retries(fetcher, url, |bytes| {
            let text = String::from_utf8_lossy(bytes).into_owned();
            parse_gml(&text)
                .map(|_| text)
                .map_err(|e| FetchError::Alkis(invalid(format!("{url}: {}", e.message))))
        })
        .map_err(|e| match e {
            FetchError::Refused { status, detail, .. } => FetchError::Alkis(invalid(format!(
                "{name} service refused the request (HTTP {status}){}: {url}",
                detail.map(|d| format!(": {d}")).unwrap_or_default()
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
        let url = service.page_url(type_name, bbox, start, key);
        let page = parse_gml(&get(&url)?).map_err(FetchError::Alkis)?;
        let full = page.len() >= PAGE_COUNT;
        let mut new = false;
        for feature in page {
            let fresh = match feature["properties"][key].as_str() {
                Some(id) => seen.insert(id.to_string()),
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
    if !service.sorted {
        // The order the service did not give: two runs over one area write the same.
        features.sort_by(|a, b| {
            let id = |feature: &Value| feature["properties"][key].as_str().map(str::to_string);
            id(a).cmp(&id(b))
        });
    }
    Ok(features)
}

/// The buildings of `service` in `bbox` (in the service's CRS).
pub fn fetch_buildings(
    fetcher: &dyn Fetcher,
    service: &WfsService,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    fetch_wfs(fetcher, service, &service.buildings_type, bbox)
}

/// The land use of `service` in `bbox`; squares without `bez` get the service's `square_bez`, if it has one.
pub fn fetch_land_use(
    fetcher: &dyn Fetcher,
    service: &WfsService,
    bbox: &BoundingBox,
) -> Result<Vec<Value>, FetchError> {
    Ok(fetch_wfs(fetcher, service, &service.land_use_type, bbox)?
        .into_iter()
        .map(|f| normalise_land_use(f, service.square_bez.as_deref()))
        .collect())
}

/// `feature` with its class in the LGB spelling (Saarland writes `Strassenverkehr`), and a
/// square without `bez` with `square_bez`.
fn normalise_land_use(feature: Value, square_bez: Option<&str>) -> Value {
    let mut feature = crate::alkis_source::normalise_land_use(feature);
    let properties = &mut feature["properties"];
    if let Some(square_bez) = square_bez
        && properties["nutzart"] == "Platz"
        && properties.get("bez").is_none()
    {
        properties["bez"] = Value::from(square_bez);
    }
    feature
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    pub const BUILDINGS: &str = include_str!("../../../tests/fixtures/saxony-buildings.gml");
    pub const LAND_USE: &str = include_str!("../../../tests/fixtures/saxony-land-use.gml");
    pub const ST_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/saxony-anhalt-buildings.gml");
    pub const ST_LAND_USE: &str =
        include_str!("../../../tests/fixtures/saxony-anhalt-land-use.gml");
    pub const HE_BUILDINGS: &str = include_str!("../../../tests/fixtures/hessen-buildings.gml");
    pub const HE_LAND_USE: &str = include_str!("../../../tests/fixtures/hessen-land-use.gml");
    pub const SH_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/schleswig-holstein-buildings.gml");
    pub const SH_LAND_USE: &str =
        include_str!("../../../tests/fixtures/schleswig-holstein-land-use.gml");
    pub const NRW_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/nordrhein-westfalen-buildings.gml");
    pub const NRW_LAND_USE: &str =
        include_str!("../../../tests/fixtures/nordrhein-westfalen-land-use.gml");
    pub const NI_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/niedersachsen-buildings.gml");
    pub const NI_LAND_USE: &str =
        include_str!("../../../tests/fixtures/niedersachsen-land-use.gml");
    pub const RP_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/rheinland-pfalz-buildings.gml");
    pub const RP_LAND_USE: &str =
        include_str!("../../../tests/fixtures/rheinland-pfalz-land-use.gml");
    /// Two buildings and two land use areas at Saarbrücken, from the WFS of the Saarland
    /// geoportal (an ArcGIS server): attribute names in upper case, the geometry in `SHAPE`.
    pub const SL_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/saarland-wfs-gebaeude.gml");
    pub const SL_LAND_USE: &str = include_str!("../../../tests/fixtures/saarland-wfs-nutzung.gml");
    /// Three buildings and three land use areas at a school in Stuttgart, from the ALKIS WFS of
    /// the LGL (GeoServer views, not the simplified schema): the id in `gml_id`, the geometry
    /// in `geom`, names of their own for the attributes.
    pub const BW_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/baden-wuerttemberg-buildings.gml");
    pub const BW_LAND_USE: &str =
        include_str!("../../../tests/fixtures/baden-wuerttemberg-land-use.gml");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use crate::ogc_api::{parse_buildings, parse_land_use};

    /// The answer of a server that writes the attributes of the simplified schema in upper
    /// case and calls the geometry `SHAPE`, with the names of the schema put in by hand.
    fn lowered(gml: &str) -> String {
        let mut text = gml.replace("ALKIS_ALKIS_WFS_ohne_Eig:SHAPE>", "ave:geometrie>");
        for name in [
            "OID",
            "AKTUALIT",
            "GEBNUTZBEZ",
            "FUNKTION",
            "GFKZSHH",
            "GMDSCHL",
            "LAGEBEZTXT",
            "NUTZART",
            "BEZ",
            "FID",
        ] {
            text = text.replace(
                &format!("ALKIS_ALKIS_WFS_ohne_Eig:{name}>"),
                &format!("ave:{}>", name.to_lowercase()),
            );
        }
        text
    }

    /// Saarland's server writes the attribute names of the simplified schema in upper case.
    /// The reader stores them under the names of the schema, with the geometry of `SHAPE`.
    #[test]
    fn saarland_buildings_parse_like_the_lower_case_schema() {
        let features = parse_gml(SL_BUILDINGS).unwrap();
        assert_eq!(features.len(), 2);
        let first = &features[0];
        assert_eq!(first["id"], "DESLALKI4T001GlcBL");
        let properties = &first["properties"];
        assert_eq!(properties["oid"], "DESLALKI4T001GlcBL");
        assert_eq!(
            properties["funktion"],
            "Gebäude für Wirtschaft oder Gewerbe"
        );
        assert_eq!(properties["gebnutzbez"], "Gebäude");
        assert_eq!(properties["aktualit"], "2026-04-16Z");
        assert!(properties.get("OID").is_none() && properties.get("shape").is_none());
        assert_eq!(first["geometry"]["type"], "MultiPolygon");
        let ring = first["geometry"]["coordinates"][0][0].as_array().unwrap();
        assert_eq!(ring.len(), 28);
        assert_eq!(
            ring[0],
            serde_json::json!([353_919.482_999_8, 5_455_359.592_999_94])
        );
        assert_eq!(features, parse_gml(&lowered(SL_BUILDINGS)).unwrap());
        // The buildings are what the build reads from every state.
        let parsed = parse_buildings(
            &serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string(),
        )
        .unwrap();
        assert_eq!(parsed.buildings.len(), 2);
        assert_eq!(parsed.buildings[0].oid, "DESLALKI4T001GlcBL");
    }

    /// Saarland's server writes `Strassenverkehr` and `Fliessgewässer`: the rules know the
    /// classes in the LGB spellings, so the streets and the rivers would be missing.
    #[test]
    fn land_use_classes_get_the_lgb_spellings() {
        let feature = |nutzart: &str| serde_json::json!({ "type": "Feature", "properties": { "nutzart": nutzart } });
        for (served, known) in [
            ("Strassenverkehr", "Straßenverkehr"),
            ("Fliessgewässer", "Fließgewässer"),
            ("Straßenverkehr", "Straßenverkehr"),
            ("Wohnbaufläche", "Wohnbaufläche"),
            ("Irgendwas Neues", "Irgendwas Neues"),
        ] {
            let normalised = normalise_land_use(feature(served), None);
            assert_eq!(normalised["properties"]["nutzart"], known, "{served}");
        }
    }

    /// A project fetched before the classes were normalised has `Strassenverkehr` stored: it is
    /// read in the LGB spelling too.
    #[test]
    fn stored_land_use_is_read_in_the_lgb_spellings() {
        let stored = serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": { "oid": "DESLALKI1", "nutzart": "Strassenverkehr" },
                "geometry": { "type": "Polygon", "coordinates":
                    [[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 0.0]]] }
            }]
        });
        let land = crate::ogc_api::parse_land_use(&stored.to_string()).unwrap();
        assert_eq!(land[0].nutzart, "Straßenverkehr");
    }

    #[test]
    fn saarland_land_use_parses_like_the_lower_case_schema() {
        let features = parse_gml(SL_LAND_USE).unwrap();
        assert_eq!(features.len(), 2);
        let properties = &features[0]["properties"];
        assert_eq!(properties["oid"], "DESLALKI1Bv000ABTN");
        assert_eq!(properties["nutzart"], "Industrie- und Gewerbefläche");
        assert_eq!(properties["bez"], "Handel und Dienstleistung");
        assert_eq!(
            features[0]["geometry"]["coordinates"][0][0]
                .as_array()
                .unwrap()
                .len(),
            30
        );
        assert_eq!(features, parse_gml(&lowered(SL_LAND_USE)).unwrap());
    }

    /// A name with a lower-case letter is an attribute as the service writes it: only names
    /// all in upper case are those of the schema.
    #[test]
    fn mixed_case_attribute_names_stay_as_they_are() {
        let features = parse_gml(SL_BUILDINGS).unwrap();
        assert_eq!(features[0]["properties"]["Shape_Area"], "1615.21775350");
        assert_eq!(features[0]["properties"]["fid"], "344581");
    }

    /// A service that refuses `SORTBY` (Saarland's ArcGIS server answers it with an
    /// exception) is paged in the order it gives, and its features are sorted by `oid` here;
    /// the others are sorted by the service as before.
    #[test]
    fn an_unsorted_service_is_paged_without_sortby() {
        let bbox = BoundingBox {
            min_e: 353_875.0,
            min_n: 5_455_271.0,
            max_e: 354_075.0,
            max_n: 5_455_471.0,
        };
        let unsorted = WfsService {
            url: "https://geoportal.saarland.de/registry/wfs/414".into(),
            sorted: false,
            ..LVERMGEO_RP
        };
        assert_eq!(
            unsorted.wfs_url("ALKIS_ALKIS_WFS_ohne_Eig:GebaeudeBauwerk", &bbox, 4),
            format!(
                "https://geoportal.saarland.de/registry/wfs/414?SERVICE=WFS&VERSION=2.0.0\
                 &REQUEST=GetFeature&TYPENAMES=ALKIS_ALKIS_WFS_ohne_Eig:GebaeudeBauwerk\
                 &SRSNAME=urn:ogc:def:crs:EPSG::25832\
                 &BBOX=353875,5455271,354075,5455471,urn:ogc:def:crs:EPSG::25832\
                 &COUNT={PAGE_COUNT}&STARTINDEX=4"
            )
        );
        let sorted = LVERMGEO_RP.wfs_url("ave:GebaeudeBauwerk", &bbox, 0);
        assert!(sorted.contains("&SORTBY=oid&COUNT="), "{sorted}");
        // Paging goes on without the sort: two pages, each feature once.
        let fetcher = MapFetcher::default();
        let members: Vec<&str> = SL_BUILDINGS
            .split("<wfs:member>")
            .skip(1)
            .map(|rest| rest.split("</wfs:member>").next().unwrap())
            .collect();
        let head = SL_BUILDINGS.split("<wfs:member>").next().unwrap();
        let page = |members: &[&str]| {
            let body: String = members
                .iter()
                .map(|member| format!("<wfs:member>{member}</wfs:member>"))
                .collect();
            format!("{head}{body}</wfs:FeatureCollection>").into_bytes()
        };
        let type_name = "ALKIS_ALKIS_WFS_ohne_Eig:GebaeudeBauwerk";
        fetcher.insert_prefix(
            &format!("{}&RESULTTYPE=hits", unsorted.query_url(type_name, &bbox)),
            format!(
                "{}</wfs:FeatureCollection>",
                head.replace("numberMatched=\"44\"", "numberMatched=\"3\"")
            )
            .into_bytes(),
        );
        fetcher.insert(&unsorted.wfs_url(type_name, &bbox, 0), page(&members));
        fetcher.insert(
            &unsorted.wfs_url(type_name, &bbox, PAGE_COUNT),
            page(&[&members[1].replace("DESLALKI4T0019CcBL", "DESLALKI4T00third")]),
        );
        let features = fetch_wfs(&fetcher, &unsorted, type_name, &bbox).unwrap();
        let oids: Vec<&str> = features
            .iter()
            .map(|feature| feature["properties"]["oid"].as_str().unwrap())
            .collect();
        // In the order of the `oid`, as a sorted service gives them: two runs write the same.
        assert_eq!(
            oids,
            [
                "DESLALKI4T0019CcBL",
                "DESLALKI4T001GlcBL",
                "DESLALKI4T00third"
            ]
        );
    }

    /// The views of Baden-Württemberg call the geometry `geom`. It is read like `geometrie`;
    /// the attributes keep the names the service gives them.
    #[test]
    fn a_geometry_named_geom_is_read() {
        let buildings = parse_gml(BW_BUILDINGS).unwrap();
        assert_eq!(buildings.len(), 3);
        let garage = &buildings[0];
        assert_eq!(garage["properties"]["gml_id"], "DEBWL52210003w2f");
        assert_eq!(garage["properties"]["gebaeudefunktion_name"], "Tiefgarage");
        assert!(garage["properties"].get("geom").is_none());
        assert_eq!(garage["geometry"]["type"], "MultiPolygon");
        assert_eq!(
            garage["geometry"]["coordinates"][0][0][0],
            serde_json::json!([511909.38, 5403830.2])
        );
        let land_use = parse_gml(BW_LAND_USE).unwrap();
        assert_eq!(land_use.len(), 3);
        assert_eq!(
            land_use[1]["properties"]["unterart_name"],
            "Öffentliche Zwecke"
        );
        assert_eq!(land_use[1]["geometry"]["type"], "MultiPolygon");
    }

    /// A service outside the simplified schema has no `oid`. Its pages are sorted by the
    /// attribute that names a feature there, and a feature repeated on a later page is kept
    /// once by that attribute.
    #[test]
    fn a_service_with_a_key_of_its_own_is_paged_by_it() {
        let bbox = BoundingBox {
            min_e: 511_780.0,
            min_n: 5_403_760.0,
            max_e: 511_920.0,
            max_n: 5_403_930.0,
        };
        let service = WfsService {
            url: "https://owsproxy.lgl-bw.de/owsproxy/wfs/WFS_LGL-BW_ALKIS".into(),
            ..LVERMGEO_RP
        };
        let type_name = "nora:v_al_gebaeude";
        let members: Vec<&str> = BW_BUILDINGS
            .split("<wfs:member>")
            .skip(1)
            .map(|rest| rest.split("</wfs:member>").next().unwrap())
            .collect();
        let head = BW_BUILDINGS.split("<wfs:member>").next().unwrap();
        let page = |members: &[&str]| {
            let body: String = members
                .iter()
                .map(|member| format!("<wfs:member>{member}</wfs:member>"))
                .collect();
            format!("{head}{body}</wfs:FeatureCollection>").into_bytes()
        };
        let query = service.query_url(type_name, &bbox);
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!("{query}&RESULTTYPE=hits"),
            format!("{head}</wfs:FeatureCollection>").into_bytes(),
        );
        fetcher.insert(
            &format!("{query}&SORTBY=gml_id&COUNT={PAGE_COUNT}&STARTINDEX=0"),
            page(&members[..2]),
        );
        // The second page repeats a feature of the first.
        fetcher.insert(
            &format!("{query}&SORTBY=gml_id&COUNT={PAGE_COUNT}&STARTINDEX={PAGE_COUNT}"),
            page(&members[1..]),
        );
        let features = fetch_wfs_by(&fetcher, &service, type_name, &bbox, "gml_id").unwrap();
        let ids: Vec<&str> = features
            .iter()
            .map(|feature| feature["properties"]["gml_id"].as_str().unwrap())
            .collect();
        assert_eq!(
            ids,
            ["DEBWL52210003w2f", "DEBWL52210003w4m", "DEBWL52210003xQ7"]
        );
    }

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
            GEOSN.wfs_url(&GEOSN.buildings_type, &SCHOOL, 20),
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
            .map(|f| normalise_land_use(f, GEOSN.square_bez.as_deref()))
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
                ("Platz", GEOSN.square_bez.as_deref()),
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
            normalise_land_use(square, GEOSN.square_bez.as_deref())["properties"]["bez"],
            GEOSN.square_bez.as_deref().unwrap()
        );
    }

    #[test]
    fn html_error_page_is_an_error() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &GEOSN.url,
            b"<!DOCTYPE html><html><head><title>Seite kann nicht angezeigt werden</title>".to_vec(),
        );
        let err = fetch_wfs(&fetcher, &GEOSN, &GEOSN.buildings_type, &SCHOOL).unwrap_err();
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
        let empty = include_str!("../../../tests/fixtures/saxony-empty.gml");
        assert_eq!(parse_gml(empty).unwrap(), Vec::<Value>::new());
        let hits = include_str!("../../../tests/fixtures/saxony-hits.gml");
        assert_eq!(number_matched(hits), Some(123));
        assert_eq!(number_matched(BUILDINGS), None, "numberMatched=unknown");
    }

    #[test]
    fn refused_request_names_the_service() {
        let fetcher = MapFetcher::default();
        fetcher.refusing.borrow_mut().push(GEOSN.url.to_string());
        let err = fetch_wfs(&fetcher, &GEOSN, &GEOSN.buildings_type, &SCHOOL).unwrap_err();
        assert!(
            err.to_string()
                .contains("GeoSN ALKIS service refused the request (HTTP 400)"),
            "{err}"
        );
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn refusal_keeps_the_exception_text() {
        // XtraServer answers errors with 400 and an OWS report (LAiV M-V).
        let fetcher = MapFetcher::default();
        fetcher.refusing.borrow_mut().push(GEOSN.url.to_string());
        fetcher.refusal_bodies.borrow_mut().insert(
            GEOSN.url.to_string(),
            br#"<ows:ExceptionReport><ows:Exception exceptionCode="InvalidParameterValue"><ows:ExceptionText>COUNT exceeds 30000</ows:ExceptionText></ows:Exception></ows:ExceptionReport>"#.to_vec(),
        );
        let err = fetch_wfs(&fetcher, &GEOSN, &GEOSN.buildings_type, &SCHOOL)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("GeoSN ALKIS service refused the request (HTTP 400)"),
            "{err}"
        );
        assert!(
            err.contains("InvalidParameterValue: COUNT exceeds 30000"),
            "{err}"
        );
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
        fetcher.insert_prefix(&GEOSN.url, BUILDINGS.as_bytes().to_vec());
        fetcher.insert_prefix(
            &format!("{}&RESULTTYPE=hits", GEOSN.wfs_url(&GEOSN.buildings_type, &SCHOOL, 0).split("&SORTBY").next().unwrap()),
            br#"<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0" numberReturned="0" numberMatched="5"/>"#.to_vec(),
        );
        let err = fetch_wfs(&fetcher, &GEOSN, &GEOSN.buildings_type, &SCHOOL).unwrap_err();
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
        let features = fetch_wfs(&fetcher, &GEOSN, &GEOSN.buildings_type, &SCHOOL).unwrap();
        assert_eq!(features.len(), 3);
        assert_eq!(*fetcher.calls.borrow(), [0, 2]);
    }

    #[test]
    fn repeated_pages_end_the_paging() {
        // A server that ignores STARTINDEX answers every page with the same features.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(&GEOSN.url, BUILDINGS.as_bytes().to_vec());
        let features = fetch_wfs(&fetcher, &GEOSN, &GEOSN.buildings_type, &SCHOOL).unwrap();
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
            .map(|f| normalise_land_use(f, LVERMGEO.square_bez.as_deref()))
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
        let url = LVERMGEO.wfs_url(&LVERMGEO.land_use_type, &bbox, 0);
        assert!(url.starts_with(&*LVERMGEO.url), "{url}");
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
            .map(|f| normalise_land_use(f, HVBG.square_bez.as_deref()))
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
    fn nrw_buildings_parse() {
        let features = parse_gml(NRW_BUILDINGS).unwrap();
        let parsed = parse_buildings(&collection(&features)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        assert_eq!(parsed.buildings.len(), features.len());
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DENW")));
        // Native EPSG:25832 coordinates (Cologne).
        let school = parsed
            .buildings
            .iter()
            .find(|b| b.function == "Allgemein bildende Schule")
            .unwrap();
        let corner = school.footprint.0[0].exterior().0[0];
        assert!((354_000.0..355_000.0).contains(&corner.x), "{corner:?}");
        assert!(!school.underground);
        let part = parsed.buildings.iter().find(|b| b.underground).unwrap();
        assert!(part.oid.starts_with("DENW"));
        assert!(parsed.buildings.iter().any(|b| b.updated.is_some()));
    }

    #[test]
    fn nrw_land_use_from_nutzung_flurstueck() {
        let features: Vec<Value> = parse_gml(NRW_LAND_USE)
            .unwrap()
            .into_iter()
            .map(|f| normalise_land_use(f, GEOBASIS_NRW.square_bez.as_deref()))
            .collect();
        let land = parse_land_use(&collection(&features)).unwrap();
        assert_eq!(land.len(), features.len());
        assert!(land.iter().all(|l| l.oid.starts_with("DENW")));
        let nutzarten: Vec<&str> = land.iter().map(|l| l.nutzart.as_str()).collect();
        for want in ["Straßenverkehr", "Wohnbaufläche", "Platz"] {
            assert!(nutzarten.contains(&want), "{want} in {nutzarten:?}");
        }
        assert!(land.iter().any(|l| l.nutzart == "Straßenverkehr"
            && l.bez.as_deref() == Some("Begleitfläche Straßenverkehr")));
        let squares: Vec<Option<&str>> = land
            .iter()
            .filter(|l| l.nutzart == "Platz")
            .map(|l| l.bez.as_deref())
            .collect();
        assert!(squares.contains(&Some("Parkplatz")), "{squares:?}");
        assert!(
            squares.contains(&Some("ohne Unterart (Nordrhein-Westfalen)")),
            "{squares:?}"
        );
        assert!(squares.iter().all(Option::is_some), "{squares:?}");
        assert!(features.iter().all(|f| !f["geometry"].is_null()));
    }

    #[test]
    fn lgln_buildings_parse() {
        let features = parse_gml(NI_BUILDINGS).unwrap();
        let parsed = parse_buildings(&collection(&features)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        assert_eq!(parsed.buildings.len(), features.len());
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DENI")));
        // Native EPSG:25832 coordinates (Hannover).
        let school = parsed
            .buildings
            .iter()
            .find(|b| b.function == "Allgemein bildende Schule")
            .unwrap();
        let corner = school.footprint.0[0].exterior().0[0];
        assert!((540_000.0..560_000.0).contains(&corner.x), "{corner:?}");
        assert!(!school.underground);
        assert!(parsed.buildings.iter().any(|b| b.updated.is_some()));
        // Names come with trailing blanks in the service; they are trimmed.
        let names: Vec<&str> = features
            .iter()
            .filter_map(|f| f["properties"]["name"].as_str())
            .collect();
        assert!(names.contains(&"Sporthalle"), "{names:?}");
        assert!(names.iter().all(|n| *n == n.trim()), "{names:?}");
    }

    #[test]
    fn lgln_land_use_from_nutzung_flurstueck() {
        assert_eq!(LGLN.land_use_type, "ave:NutzungFlurstueck");
        assert_eq!(LGLN.buildings_type, "ave:GebaeudeBauwerk");
        assert_eq!(
            LGLN.url,
            "https://opendata.lgln.niedersachsen.de/doorman/noauth/alkis_wfs_einfach"
        );
        assert_eq!(LGLN.crs, Crs::Utm32);
        let features: Vec<Value> = parse_gml(NI_LAND_USE)
            .unwrap()
            .into_iter()
            .map(|f| normalise_land_use(f, LGLN.square_bez.as_deref()))
            .collect();
        let land = parse_land_use(&collection(&features)).unwrap();
        assert_eq!(land.len(), features.len());
        assert!(land.iter().all(|l| l.oid.starts_with("DENI")));
        let nutzarten: Vec<&str> = land.iter().map(|l| l.nutzart.as_str()).collect();
        for want in ["Straßenverkehr", "Wohnbaufläche", "Platz", "Wald"] {
            assert!(nutzarten.contains(&want), "{want} in {nutzarten:?}");
        }
        assert!(land.iter().any(|l| l.nutzart == "Straßenverkehr"
            && l.bez.as_deref() == Some("Begleitfläche Straßenverkehr")));
        let squares: Vec<Option<&str>> = land
            .iter()
            .filter(|l| l.nutzart == "Platz")
            .map(|l| l.bez.as_deref())
            .collect();
        assert!(squares.contains(&Some("Parkplatz")), "{squares:?}");
        assert!(
            squares.contains(&Some("ohne Unterart (Niedersachsen)")),
            "{squares:?}"
        );
        assert!(squares.iter().all(Option::is_some), "{squares:?}");
        assert!(features.iter().all(|f| !f["geometry"].is_null()));
    }

    #[test]
    fn lgln_underground_buildings_by_rellage() {
        let features = parse_gml(NI_BUILDINGS).unwrap();
        let parsed = parse_buildings(&collection(&features)).unwrap();
        let underground: Vec<_> = parsed.buildings.iter().filter(|b| b.underground).collect();
        // The service marks underground garages as `Gebäude` + `rellage "Unter der Erdoberfläche"`.
        assert_eq!(underground.len(), 4, "{underground:?}");
        assert!(underground.iter().all(|b| b.kind == "Gebäude"));
        // Other `rellage` values (raised buildings) stay above ground.
        let raised = features
            .iter()
            .filter(|f| f["properties"]["rellage"] == "Aufgeständert")
            .count();
        assert!(raised > 0);
        assert_eq!(
            parsed.buildings.iter().filter(|b| !b.underground).count(),
            features.len() - 4
        );
    }

    #[test]
    fn rp_buildings_parse() {
        let features = parse_gml(RP_BUILDINGS).unwrap();
        let parsed = parse_buildings(&collection(&features)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        assert_eq!(parsed.buildings.len(), features.len());
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DERP")));
        // Native EPSG:25832 coordinates (Koblenz).
        let school = parsed
            .buildings
            .iter()
            .find(|b| b.function == "Allgemein bildende Schule")
            .unwrap();
        let corner = school.footprint.0[0].exterior().0[0];
        assert!((390_000.0..410_000.0).contains(&corner.x), "{corner:?}");
        // `aktualit` is read from inside `gml:timePosition`.
        assert!(parsed.buildings.iter().all(|b| b.updated.is_some()));
        assert_eq!(
            school.updated,
            chrono::NaiveDate::from_ymd_opt(2026, 7, 20),
            "{school:?}"
        );
        // Underground parts are marked by `rellage`.
        assert!(parsed.buildings.iter().any(|b| b.underground));
        assert!(
            parsed
                .buildings
                .iter()
                .any(|b| !b.underground && b.function == "Wohngebäude")
        );
    }

    #[test]
    fn rp_land_use_from_nutzung() {
        assert_eq!(LVERMGEO_RP.land_use_type, "ave:Nutzung");
        assert_eq!(LVERMGEO_RP.buildings_type, "ave:GebaeudeBauwerk");
        assert_eq!(
            LVERMGEO_RP.url,
            "https://geo5.service24.rlp.de/wfs/alkis_rp.fcgi"
        );
        assert_eq!(LVERMGEO_RP.crs, Crs::Utm32);
        let features: Vec<Value> = parse_gml(RP_LAND_USE)
            .unwrap()
            .into_iter()
            .map(|f| normalise_land_use(f, LVERMGEO_RP.square_bez.as_deref()))
            .collect();
        let land = parse_land_use(&collection(&features)).unwrap();
        assert_eq!(land.len(), features.len());
        assert!(land.iter().all(|l| l.oid.starts_with("DERP")));
        let nutzarten: Vec<&str> = land.iter().map(|l| l.nutzart.as_str()).collect();
        for want in [
            "Straßenverkehr",
            "Platz",
            "Wald",
            "Fließgewässer",
            "Stehendes Gewässer",
            "Hafenbecken",
            // RP writes "Unland, Vegetationslose Fläche"; it is read in the LGB spelling.
            "Unland/Vegetationslose Fläche",
        ] {
            assert!(nutzarten.contains(&want), "{want} in {nutzarten:?}");
        }
        assert!(land.iter().any(|l| l.nutzart == "Straßenverkehr"
            && l.bez.as_deref() == Some("Begleitfläche Straßenverkehr")));
        // `;`-terminated values are kept as they are.
        assert!(
            land.iter()
                .any(|l| l.bez.as_deref() == Some("Begleitfläche Bahnverkehr;"))
        );
        let squares: Vec<Option<&str>> = land
            .iter()
            .filter(|l| l.nutzart == "Platz")
            .map(|l| l.bez.as_deref())
            .collect();
        assert!(squares.contains(&Some("Parkplatz")), "{squares:?}");
        assert!(
            squares.contains(&Some("ohne Unterart (Rheinland-Pfalz)")),
            "{squares:?}"
        );
        assert!(squares.iter().all(Option::is_some), "{squares:?}");
        assert!(features.iter().all(|f| !f["geometry"].is_null()));
        // Names are `;`-joined as the service sends them and otherwise untouched.
        let names: Vec<String> = parse_gml(RP_BUILDINGS)
            .unwrap()
            .iter()
            .filter_map(|f| f["properties"]["name"].as_str().map(str::to_string))
            .collect();
        assert!(
            names.contains(&"Amtsgericht Koblenz;Landgericht Koblenz".to_string()),
            "{names:?}"
        );
        assert!(names.iter().all(|n| n == n.trim()), "{names:?}");
    }

    #[test]
    fn aktualit_in_time_position_is_read() {
        let one = |inner: &str| {
            format!(
                r#"<?xml version="1.0"?><wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:ave="x"><wfs:member><ave:GebaeudeBauwerk gml:id="a"><ave:oid>DERPLP0000000001BL</ave:oid>{inner}<ave:gebnutzbez>Gebäude</ave:gebnutzbez></ave:GebaeudeBauwerk></wfs:member></wfs:FeatureCollection>"#
            )
        };
        let date = |inner: &str| {
            parse_gml(&one(inner)).unwrap()[0]["properties"]["aktualit"]
                .as_str()
                .map(str::to_string)
        };
        assert_eq!(
            date(
                r#"<ave:aktualit gml:id="a.aktualit"><gml:timePosition>2024-05-03</gml:timePosition></ave:aktualit>"#
            ),
            Some("2024-05-03".to_string())
        );
        // With a TimeInstant around it, too.
        assert_eq!(
            date(
                r#"<ave:aktualit><gml:TimeInstant gml:id="t"><gml:timePosition>2024-05-03</gml:timePosition></gml:TimeInstant></ave:aktualit>"#
            ),
            Some("2024-05-03".to_string())
        );
        // A plain text value (the other states) is read as before, an empty one is absent.
        assert_eq!(
            date("<ave:aktualit>2013-02-18Z</ave:aktualit>"),
            Some("2013-02-18Z".to_string())
        );
        assert_eq!(date("<ave:aktualit></ave:aktualit>"), None);
        // A `timePosition` anywhere else is no attribute.
        let other = parse_gml(&one(
            "<ave:zeit><gml:timePosition>2020-01-01</gml:timePosition></ave:zeit>",
        ))
        .unwrap();
        assert!(other[0]["properties"].get("aktualit").is_none());
        // The other states' fixtures keep their dates.
        for gml in [HE_BUILDINGS, NI_BUILDINGS, NRW_BUILDINGS] {
            let parsed = parse_buildings(&collection(&parse_gml(gml).unwrap())).unwrap();
            assert!(parsed.buildings.iter().any(|b| b.updated.is_some()));
        }
    }

    #[test]
    fn lvermgeo_sh_buildings_parse() {
        let features = parse_gml(SH_BUILDINGS).unwrap();
        let parsed = parse_buildings(&collection(&features)).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        assert_eq!(parsed.buildings.len(), features.len());
        assert!(parsed.buildings.iter().all(|b| b.oid.starts_with("DESH")));
        // Native EPSG:25832 coordinates (Norderstedt).
        let corner = parsed.buildings[0].footprint.0[0].exterior().0[0];
        assert!((565_000.0..566_000.0).contains(&corner.x), "{corner:?}");
        let part = parsed
            .buildings
            .iter()
            .find(|b| b.kind == "Bauteil" && b.underground)
            .expect("an underground building part");
        assert!(!part.function.is_empty());
        assert!(parsed.buildings.iter().any(|b| !b.underground));
        assert!(parsed.buildings.iter().any(|b| b.kind == "Gebäude"));
    }

    #[test]
    fn lvermgeo_sh_land_use_from_nutzung_flurstueck() {
        assert_eq!(LVERMGEO_SH.land_use_type, "ave:NutzungFlurstueck");
        assert_eq!(
            LVERMGEO_SH.url,
            "https://dienste.gdi-sh.de/WFS_SH_ALKIS_VWG_OpenGBD"
        );
        assert_eq!(LVERMGEO_SH.crs, Crs::Utm32);
        let features: Vec<Value> = parse_gml(SH_LAND_USE)
            .unwrap()
            .into_iter()
            .map(|f| normalise_land_use(f, LVERMGEO_SH.square_bez.as_deref()))
            .collect();
        let land = parse_land_use(&collection(&features)).unwrap();
        assert_eq!(land.len(), features.len());
        let nutzarten: Vec<&str> = land.iter().map(|l| l.nutzart.as_str()).collect();
        for want in ["Straßenverkehr", "Wohnbaufläche", "Platz"] {
            assert!(nutzarten.contains(&want), "{want} in {nutzarten:?}");
        }
        assert!(features.iter().all(|f| !f["geometry"].is_null()));
        let squares: Vec<Option<&str>> = land
            .iter()
            .filter(|l| l.nutzart == "Platz")
            .map(|l| l.bez.as_deref())
            .collect();
        assert_eq!(squares.len(), 2, "{squares:?}");
        assert!(squares.iter().all(Option::is_some), "{squares:?}");
        assert!(
            squares.contains(&Some("ohne Unterart (Schleswig-Holstein)")),
            "{squares:?}"
        );
        let url = LVERMGEO_SH.wfs_url(
            &LVERMGEO_SH.land_use_type,
            &BoundingBox {
                min_e: 565_115.0,
                min_n: 5_949_136.0,
                max_e: 565_312.0,
                max_n: 5_949_365.0,
            },
            0,
        );
        assert!(
            url.contains("BBOX=565115,5949136,565312,5949365,urn:ogc:def:crs:EPSG::25832"),
            "{url}"
        );
    }

    #[test]
    fn hvbg_urls_are_in_zone_32() {
        let bbox = BoundingBox {
            min_e: 476_320.0,
            min_n: 5_552_421.5,
            max_e: 476_817.2,
            max_n: 5_552_753.0,
        };
        let url = HVBG.wfs_url(&HVBG.land_use_type, &bbox, 0);
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
        fetcher.insert_prefix(&LVERMGEO.url, report.as_bytes().to_vec());
        let bbox = BoundingBox {
            min_e: 706_350.0,
            min_n: 5_706_800.0,
            max_e: 706_550.0,
            max_n: 5_707_000.0,
        };
        let err = fetch_wfs(&fetcher, &LVERMGEO, &LVERMGEO.buildings_type, &bbox).unwrap_err();
        assert!(
            err.to_string().contains("not a WFS FeatureCollection"),
            "{err}"
        );
    }
}
