//! Permit sources of the states that need no code of their own (ADR 0130): WFS 2.0 services
//! that answer GeoJSON, read by a table of their layers. Each layer says which attributes give
//! the name, the zone or the office of a finding; the fetch keeps those attributes and nothing
//! else, so a service that also carries people's names or long descriptions stores none of it.
//!
//! The services are asked by the generic reader ([`get_features`]) with a box in WGS 84 and
//! `srsName` WGS 84; the parser of the Berlin sources reads the positions in either axis order.

use std::path::Path;

use freemapper_adapter_api::permits::{
    FindingKind, ForestFunctionKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo,
    ProtectedObjectKind, Wording,
};
use freemapper_sources::geojson_wfs::{
    GeoJsonWfs, LatLonBox, ProjectedBox, get_features, get_gml_features, get_projected_features,
};
use geo_types::{Coord, Geometry, Rect};
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::crs::{Crs, LatLon, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher};

/// How the features of a layer become findings, with the attributes they read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// A water protection area or zone: the attributes of its name, its zone and its
    /// ordinance, and where the layer also has planned zones, the attribute and value that
    /// mark one (its zone then says "geplant").
    Water {
        name: &'static str,
        zone: &'static str,
        citation: Option<&'static str>,
        planned: Option<(&'static str, &'static str)>,
    },
    /// Forest with a function or protection, named by an attribute where the layer has one.
    Function {
        kind: ForestFunctionKind,
        name: Option<&'static str>,
    },
    /// A unit of a forest administration: `office` is `prefix` and the attribute.
    Unit {
        administration: &'static str,
        prefix: &'static str,
        office: &'static str,
    },
    /// Forest of the state: `owner` as the notes name it, and the forest office that looks
    /// after it, `prefix` and the attribute `office`.
    StateForest {
        owner: &'static str,
        prefix: &'static str,
        office: &'static str,
    },
    /// A protected object of nature conservation law of `kind` (ADR 0146): its name (or species,
    /// or type of biotope) and its legal act from the attributes the layer names. With `only`,
    /// a feature counts only when the attribute holds one of the values (a register that lists
    /// unprotected biotopes too).
    Object {
        kind: ProtectedObjectKind,
        name: Option<&'static str>,
        citation: Option<&'static str>,
        only: Option<(&'static str, &'static [&'static str])>,
    },
    /// A listed monument of `kind`, or of the kind the attribute `kind_from` names where the
    /// layer has one, with a link from the attribute `link` when it starts with `list` (the
    /// state's monument list).
    Monument {
        kind: &'static str,
        kind_from: Option<&'static str>,
        link: Option<(&'static str, &'static str)>,
    },
}

impl Reading {
    /// The attributes a feature keeps.
    pub(crate) fn kept(self) -> Vec<&'static str> {
        match self {
            Reading::Water {
                name,
                zone,
                citation,
                planned,
            } => [Some(name), Some(zone), citation, planned.map(|(a, _)| a)]
                .into_iter()
                .flatten()
                .collect(),
            Reading::Function { name, .. } => name.into_iter().collect(),
            Reading::Unit { office, .. } | Reading::StateForest { office, .. } => vec![office],
            Reading::Object {
                name,
                citation,
                only,
                ..
            } => [name, citation, only.map(|(attribute, _)| attribute)]
                .into_iter()
                .flatten()
                .collect(),
            Reading::Monument {
                kind_from, link, ..
            } => [kind_from, link.map(|(attribute, _)| attribute)]
                .into_iter()
                .flatten()
                .collect(),
        }
    }

    /// Whether the finding may be a point; all others are areas.
    fn point_allowed(self) -> bool {
        matches!(self, Reading::Monument { .. } | Reading::Object { .. })
    }
}

/// One layer: its key (a plain name), its WFS type and its reading.
pub type Layer = (&'static str, &'static str, Reading);

/// A WFS of a state read by a table.
pub struct TableSource {
    info: PermitSourceInfo,
    service: GeoJsonWfs,
    layers: &'static [Layer],
    /// The service answers GML 3.2 only (ADR 0137), read by [`get_gml_features`].
    gml: bool,
    /// The service is asked and answers in this CRS rather than in WGS 84 (ADR 0144).
    native: Option<Crs>,
}

/// A source of `layers` on the service `base`, which names its GeoJSON `output_format`.
#[allow(clippy::too_many_arguments)]
pub fn source(
    key: &'static str,
    label: Wording,
    what: Wording,
    credit: &'static str,
    keys: &'static [&'static str],
    sections: &'static [PermitSection],
    base: &'static str,
    output_format: &'static str,
    layers: &'static [Layer],
) -> Box<dyn PermitSource> {
    debug_assert!(
        keys.len() == layers.len() && keys.iter().zip(layers).all(|(a, (b, _, _))| a == b)
    );
    Box::new(TableSource {
        info: PermitSourceInfo {
            key,
            label,
            what,
            credit,
            note: None,
            sampled: false,
            gaps: None,
            layers: keys,
            sections,
            offices: None,
        },
        service: GeoJsonWfs {
            base,
            output_format,
            bbox_crs_suffix: true,
        },
        layers,
        gml: false,
        native: None,
    })
}

/// [`source`] for a service that answers GML 3.2 only (ADR 0137), with the `output_format` it
/// names GML 3.2 by, URL-encoded as the service needs it (`gml32` on a GeoServer,
/// `application%2Fgml%2Bxml%3B%20version%3D3.2` on a deegree).
#[allow(clippy::too_many_arguments)]
pub fn gml_source(
    key: &'static str,
    label: Wording,
    what: Wording,
    credit: &'static str,
    keys: &'static [&'static str],
    sections: &'static [PermitSection],
    base: &'static str,
    output_format: &'static str,
    layers: &'static [Layer],
) -> Box<dyn PermitSource> {
    debug_assert!(
        keys.len() == layers.len() && keys.iter().zip(layers).all(|(a, (b, _, _))| a == b)
    );
    Box::new(TableSource {
        info: PermitSourceInfo {
            key,
            label,
            what,
            credit,
            note: None,
            sampled: false,
            gaps: None,
            layers: keys,
            sections,
            offices: None,
        },
        service: GeoJsonWfs {
            base,
            output_format,
            bbox_crs_suffix: true,
        },
        layers,
        gml: true,
        native: None,
    })
}

/// [`source`] (or, with `gml`, [`gml_source`]) for a service that finds nothing by a box in
/// WGS 84 or answers only in its own CRS `native` (ADR 0144): it is asked by the box in that
/// CRS, and its positions are turned into WGS 84.
#[allow(clippy::too_many_arguments)]
pub fn native_source(
    native: Crs,
    gml: bool,
    key: &'static str,
    label: Wording,
    what: Wording,
    credit: &'static str,
    keys: &'static [&'static str],
    sections: &'static [PermitSection],
    base: &'static str,
    output_format: &'static str,
    layers: &'static [Layer],
) -> Box<dyn PermitSource> {
    debug_assert!(
        keys.len() == layers.len() && keys.iter().zip(layers).all(|(a, (b, _, _))| a == b)
    );
    Box::new(TableSource {
        info: PermitSourceInfo {
            key,
            label,
            what,
            credit,
            note: None,
            sampled: false,
            gaps: None,
            layers: keys,
            sections,
            offices: None,
        },
        service: GeoJsonWfs {
            base,
            output_format,
            bbox_crs_suffix: true,
        },
        layers,
        gml,
        native: Some(native),
    })
}

impl PermitSource for TableSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let projected = self.native.map(|crs| projected_box(crs, bbox));
        let bbox = LatLonBox {
            min_lat: bbox.min().y,
            min_lon: bbox.min().x,
            max_lat: bbox.max().y,
            max_lon: bbox.max().x,
        };
        self.layers
            .iter()
            .map(|(_, type_name, reading)| {
                let features = if let (Some(crs), Some(projected)) = (self.native, &projected) {
                    get_projected_features(
                        fetcher,
                        &self.service,
                        type_name,
                        projected,
                        self.gml,
                        part,
                    )?
                    .iter()
                    .filter_map(|f| in_wgs84(f, crs))
                    .collect()
                } else if self.gml {
                    get_gml_features(fetcher, &self.service, type_name, &bbox, part)?
                } else {
                    get_features(fetcher, &self.service, type_name, &bbox, part)?
                };
                Ok(features
                    .iter()
                    .filter_map(|f| stored(f, *reading))
                    .collect())
            })
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let Some((_, _, reading)) = self.layers.iter().find(|(key, _, _)| *key == layer) else {
            return Vec::new();
        };
        features
            .iter()
            .filter_map(|feature| finding(*reading, feature))
            .collect()
    }
}

/// The box in `crs` around `bbox` (WGS 84, x = longitude): the corners projected, their
/// smallest and largest values.
pub(crate) fn projected_box(crs: Crs, bbox: &Rect<f64>) -> ProjectedBox {
    let corners = [
        (bbox.min().y, bbox.min().x),
        (bbox.min().y, bbox.max().x),
        (bbox.max().y, bbox.min().x),
        (bbox.max().y, bbox.max().x),
    ]
    .map(|(lat, lon)| crs.forward(LatLon { lat, lon }));
    let fold = |f: fn(f64, f64) -> f64, start: f64, get: fn(&ProjectedPoint) -> f64| {
        corners.iter().map(get).fold(start, f)
    };
    ProjectedBox {
        epsg: crs.epsg(),
        min_x: fold(f64::min, f64::INFINITY, |p| p.easting),
        min_y: fold(f64::min, f64::INFINITY, |p| p.northing),
        max_x: fold(f64::max, f64::NEG_INFINITY, |p| p.easting),
        max_y: fold(f64::max, f64::NEG_INFINITY, |p| p.northing),
    }
}

/// `feature` with its positions in `crs` (metres, easting first) turned into WGS 84, longitude
/// first; `None` when its geometry cannot be read or a position cannot be turned.
pub(crate) fn in_wgs84(feature: &Value, crs: Crs) -> Option<Value> {
    let geometry =
        serde_json::from_value::<geojson::Geometry>(feature.get("geometry")?.clone()).ok()?;
    let mut geometry = Geometry::<f64>::try_from(geometry).ok()?;
    let turn = |c: Coord<f64>| {
        crs.inverse(ProjectedPoint {
            easting: c.x,
            northing: c.y,
        })
        .map(|p| Coord { x: p.lon, y: p.lat })
    };
    if geo::CoordsIter::coords_iter(&geometry).any(|c| turn(c).is_none()) {
        return None;
    }
    geo::MapCoordsInPlace::map_coords_in_place(&mut geometry, |c| turn(c).unwrap_or(c));
    let mut feature = feature.clone();
    feature["geometry"] = serde_json::to_value(geojson::Geometry::new(
        geojson::GeometryValue::from(&geometry),
    ))
    .ok()?;
    Some(feature)
}

/// A feature as it is stored: the attributes `reading` keeps and the geometry as longitude,
/// latitude.
pub(crate) fn stored(feature: &Value, reading: Reading) -> Option<Value> {
    let shape = geometry(feature)?;
    if !(matches!(shape, Geometry::MultiPolygon(_)) || reading.point_allowed()) {
        return None;
    }
    let properties: serde_json::Map<String, Value> = reading
        .kept()
        .into_iter()
        .filter_map(|key| {
            Some((
                key.to_string(),
                feature.get("properties")?.get(key)?.clone(),
            ))
        })
        .collect();
    Some(serde_json::json!({
        "type": "Feature",
        "geometry": geojson::Geometry::new(geojson::GeometryValue::from(&shape)),
        "properties": properties,
    }))
}

/// The text of attribute `key`, trimmed; `None` when it is missing, blank or the text "null"
/// (an ArcGIS server writes that for an empty value).
pub(crate) fn text(feature: &Value, key: &str) -> Option<String> {
    let value = feature.get("properties")?.get(key)?;
    let text = match value {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    // The LUBW writes quotes inside a name with a backslash (`\"Ulrichslinde\"`).
    let text = text.replace("\\\"", "\"");
    (!text.is_empty() && text != "null").then_some(text)
}

/// A name written all in capitals ("HÜTTEN", "BAD SEGEBERG") as a name is written ("Hütten",
/// "Bad Segeberg"); a name with small letters stays as it is.
pub(crate) fn name_case(name: &str) -> String {
    if name.chars().any(char::is_lowercase) {
        return name.to_string();
    }
    let mut out = String::with_capacity(name.len());
    let mut start = true;
    for c in name.chars() {
        if start {
            out.push(c);
        } else {
            out.extend(c.to_lowercase());
        }
        start = !c.is_alphabetic();
    }
    out
}

/// A zone as the hints name it: a bare "I" or "IIIA" becomes "Schutzzone I"; a text that
/// already says zone stays.
pub(crate) fn zone_name(zone: &str) -> String {
    if zone.to_lowercase().contains("zone") {
        zone.to_string()
    } else {
        format!("Schutzzone {zone}")
    }
}

pub(crate) fn finding(reading: Reading, feature: &Value) -> Option<PermitFinding> {
    let geometry = geometry(feature)?;
    if !(matches!(geometry, Geometry::MultiPolygon(_)) || reading.point_allowed()) {
        return None;
    }
    let kind = match reading {
        Reading::Water {
            name,
            zone,
            citation,
            planned,
        } => {
            let planned = planned.is_some_and(|(attribute, value)| {
                text(feature, attribute).as_deref() == Some(value)
            });
            FindingKind::WaterProtection {
                name: text(feature, name),
                zone: text(feature, zone).map(|z| {
                    let z = zone_name(&z);
                    if planned { format!("{z} (geplant)") } else { z }
                }),
                citation: citation.and_then(|c| text(feature, c)),
                authority: None,
            }
        }
        Reading::Function { kind, name } => FindingKind::ForestFunction {
            kind,
            name: name.and_then(|n| text(feature, n)),
        },
        Reading::Unit {
            administration,
            prefix,
            office,
        } => FindingKind::ForestUnit {
            administration: administration.to_string(),
            office: format!("{prefix}{}", name_case(&text(feature, office)?)),
            unit: None,
        },
        Reading::StateForest {
            owner,
            prefix,
            office,
        } => FindingKind::StateForest {
            owner: owner.to_string(),
            office: text(feature, office).map(|o| format!("{prefix}{}", name_case(&o))),
        },
        Reading::Object {
            kind,
            name,
            citation,
            only,
        } => {
            if let Some((attribute, values)) = only
                && !text(feature, attribute).is_some_and(|v| values.contains(&v.as_str()))
            {
                return None;
            }
            FindingKind::ProtectedObject {
                kind,
                name: name.and_then(|n| text(feature, n)),
                citation: citation.and_then(|c| text(feature, c)),
            }
        }
        Reading::Monument {
            kind,
            kind_from,
            link,
        } => FindingKind::Monument {
            kind: kind_from
                .and_then(|k| text(feature, k))
                .unwrap_or_else(|| kind.to_string()),
            link: link.and_then(|(attribute, list)| {
                text(feature, attribute)
                    .filter(|l| l.starts_with(list) && !l.contains(char::is_whitespace))
            }),
        },
    };
    Some(PermitFinding { kind, geometry })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const LAYERS: &[Layer] = &[
        (
            "zones",
            "x:zones",
            Reading::Water {
                name: "NAME",
                zone: "ZONE",
                citation: Some("VO"),
                planned: None,
            },
        ),
        (
            "monuments",
            "x:monuments",
            Reading::Monument {
                kind: "Baudenkmal",
                kind_from: None,
                link: Some(("LIST", "https://example.org/list")),
            },
        ),
    ];

    fn table() -> Box<dyn PermitSource> {
        source(
            "test",
            Wording { en: "T", de: "T" },
            Wording { en: "t", de: "t" },
            "CC BY 4.0",
            &["zones", "monuments"],
            &[PermitSection::WaterProtection, PermitSection::Monuments],
            "https://example.org/wfs",
            "GEOJSON",
            LAYERS,
        )
    }

    /// A service asked in its own CRS gets the box in metres, and its answer in metres comes
    /// out in WGS 84; a monument takes its kind from the attribute the layer names (ADR 0144).
    #[test]
    fn a_native_source_asks_and_answers_in_its_crs() {
        const NATIVE: &[Layer] = &[(
            "denkmale",
            "x:denkmal_punkte",
            Reading::Monument {
                kind: "Denkmal",
                kind_from: Some("kategorie"),
                link: None,
            },
        )];
        let source = native_source(
            Crs::Utm32,
            false,
            "native",
            Wording { en: "N", de: "N" },
            Wording { en: "n", de: "n" },
            "dl-de/zero-2-0",
            &["denkmale"],
            &[PermitSection::Monuments],
            "https://example.org/wfs",
            "GEOJSON",
            NATIVE,
        );
        // Hagen, about 7.55 E, 51.35 N.
        let point = Crs::Utm32.forward(LatLon {
            lat: 51.35,
            lon: 7.55,
        });
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://example.org/wfs?service=WFS",
            serde_json::to_vec(
                &serde_json::json!({"type": "FeatureCollection", "features": [
                {"type": "Feature", "properties": {"kategorie": "Bodendenkmal", "anschrift": "x"},
                 "geometry": {"type": "Point", "coordinates": [point.easting, point.northing]}},
                {"type": "Feature", "properties": {},
                 "geometry": {"type": "Point", "coordinates": [point.easting, point.northing]}}]}),
            )
            .unwrap(),
        );
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(Coord { x: 7.54, y: 51.34 }, Coord { x: 7.56, y: 51.36 });
        let answers = source
            .fetch(&fetcher, &bbox, &part.path().join("p.part"))
            .unwrap();
        let url = fetcher.log.borrow()[0].clone();
        assert!(
            url.contains("&srsName=urn:ogc:def:crs:EPSG::25832&"),
            "{url}"
        );
        assert!(url.contains(",urn:ogc:def:crs:EPSG::25832"), "{url}");
        // Only the attribute the reading keeps.
        assert!(answers[0][0]["properties"].get("anschrift").is_none());
        let found = source.findings("denkmale", &answers[0]);
        let kinds: Vec<&FindingKind> = found.iter().map(|f| &f.kind).collect();
        assert_eq!(
            kinds,
            [
                &FindingKind::Monument {
                    kind: "Bodendenkmal".into(),
                    link: None
                },
                &FindingKind::Monument {
                    kind: "Denkmal".into(),
                    link: None
                },
            ]
        );
        let c = geo::CoordsIter::coords_iter(&found[0].geometry)
            .next()
            .unwrap();
        assert!(
            (c.x - 7.55).abs() < 1e-6 && (c.y - 51.35).abs() < 1e-6,
            "{c:?}"
        );
    }

    /// A protected object reads its name and act, and a register of unprotected biotopes
    /// keeps only the protected ones (ADR 0146).
    #[test]
    fn an_object_reading_keeps_only_the_protected_values() {
        let reading = Reading::Object {
            kind: ProtectedObjectKind::Biotope,
            name: Some("bt_name"),
            citation: None,
            only: Some(("schutz_ges", &["1"])),
        };
        let feature = |schutz: &str| {
            serde_json::json!({"type": "Feature",
                "properties": {"bt_name": "Erlenbruch", "schutz_ges": schutz, "x": 1},
                "geometry": {"type": "Point", "coordinates": [13.2, 52.45]}})
        };
        let stored_one = stored(&feature("1"), reading).unwrap();
        assert!(stored_one["properties"].get("x").is_none());
        assert_eq!(
            finding(reading, &stored_one).map(|f| f.kind),
            Some(FindingKind::ProtectedObject {
                kind: ProtectedObjectKind::Biotope,
                name: Some("Erlenbruch".into()),
                citation: None,
            })
        );
        let suspected = stored(&feature("2"), reading).unwrap();
        assert!(finding(reading, &suspected).is_none());
    }

    /// A zone keeps the attributes it reads and nothing else; positions written latitude first
    /// come out as longitude, latitude; "null" and blanks are no value; a monument may be a
    /// point and keeps only a link into the list.
    #[test]
    fn a_table_keeps_what_it_reads() {
        let zones = serde_json::json!({"type": "FeatureCollection", "features": [{
            "type": "Feature",
            "properties": {"NAME": "WSG Elm", "ZONE": "III", "VO": "null",
                           "BEGUENSTIGT": "Wasserwerk", "CONTACT": "someone@example.org"},
            "geometry": {"type": "Polygon", "coordinates": [[
                [49.30, 6.85], [49.30, 6.86], [49.31, 6.86], [49.30, 6.85]
            ]]},
        }]});
        let monuments = serde_json::json!({"type": "FeatureCollection", "features": [
            {"type": "Feature", "properties": {"LIST": "https://example.org/list.pdf", "TEXT": "Haus"},
             "geometry": {"type": "Point", "coordinates": [49.23, 6.99]}},
            {"type": "Feature", "properties": {"LIST": "https://elsewhere.org/x"},
             "geometry": {"type": "Point", "coordinates": [49.23, 6.99]}},
        ]});
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://example.org/wfs?service=WFS&version=2.0.0&request=GetFeature&outputFormat=GEOJSON&srsName=urn:ogc:def:crs:EPSG::4326&typeNames=x:zones&",
            serde_json::to_vec(&zones).unwrap(),
        );
        fetcher.insert_prefix(
            "https://example.org/wfs?service=WFS&version=2.0.0&request=GetFeature&outputFormat=GEOJSON&srsName=urn:ogc:def:crs:EPSG::4326&typeNames=x:monuments&",
            serde_json::to_vec(&monuments).unwrap(),
        );
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            geo_types::Coord { x: 6.8, y: 49.2 },
            geo_types::Coord { x: 7.0, y: 49.4 },
        );
        let layers = table()
            .fetch(&fetcher, &bbox, &part.path().join("x.part"))
            .unwrap();
        let text = serde_json::to_string(&layers).unwrap();
        assert!(
            !text.contains("someone") && !text.contains("BEGUENSTIGT") && !text.contains("Haus"),
            "{text}"
        );
        assert_eq!(layers[0][0]["geometry"]["coordinates"][0][0][0][0], 6.85);
        let zone = table().findings("zones", &layers[0]);
        assert_eq!(
            zone[0].kind,
            FindingKind::WaterProtection {
                name: Some("WSG Elm".into()),
                zone: Some("Schutzzone III".into()),
                citation: None,
                authority: None,
            }
        );
        let monuments = table().findings("monuments", &layers[1]);
        let links: Vec<Option<&str>> = monuments
            .iter()
            .map(|m| match &m.kind {
                FindingKind::Monument { link, .. } => link.as_deref(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(links, [Some("https://example.org/list.pdf"), None]);
        assert!(table().findings("other", &layers[0]).is_empty());
    }

    #[test]
    fn capital_names_are_written_as_names() {
        assert_eq!(name_case("HÜTTEN"), "Hütten");
        assert_eq!(name_case("BAD SEGEBERG"), "Bad Segeberg");
        assert_eq!(name_case("NEUMÜNSTER-HAHNKAMP"), "Neumünster-Hahnkamp");
        assert_eq!(name_case("Rastpfuhl/Puettlingen"), "Rastpfuhl/Puettlingen");
    }

    /// A water zone or a forest function must be an area; a point is left out.
    #[test]
    fn an_area_reading_leaves_points_out() {
        let point = serde_json::json!({"type": "Feature", "properties": {"NAME": "x"},
            "geometry": {"type": "Point", "coordinates": [49.3, 6.85]}});
        assert!(table().findings("zones", &[point]).is_empty());
    }
}
