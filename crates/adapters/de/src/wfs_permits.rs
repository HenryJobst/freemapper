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
    Wording,
};
use freemapper_sources::geojson_wfs::{GeoJsonWfs, LatLonBox, get_features};
use geo_types::{Geometry, Rect};
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::fetch::{FetchError, Fetcher};

/// How the features of a layer become findings, with the attributes they read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// A water protection area or zone: the attributes of its name, its zone and its
    /// ordinance.
    Water {
        name: &'static str,
        zone: &'static str,
        citation: Option<&'static str>,
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
    /// A listed monument of `kind`, with a link from the attribute `link` when it starts with
    /// `list` (the state's monument list).
    Monument {
        kind: &'static str,
        link: Option<(&'static str, &'static str)>,
    },
}

impl Reading {
    /// The attributes a feature keeps.
    fn kept(self) -> Vec<&'static str> {
        match self {
            Reading::Water {
                name,
                zone,
                citation,
            } => [Some(name), Some(zone), citation]
                .into_iter()
                .flatten()
                .collect(),
            Reading::Function { name, .. } => name.into_iter().collect(),
            Reading::Unit { office, .. } => vec![office],
            Reading::Monument { link, .. } => {
                link.map(|(attribute, _)| attribute).into_iter().collect()
            }
        }
    }

    /// Whether the finding may be a point; all others are areas.
    fn point_allowed(self) -> bool {
        matches!(self, Reading::Monument { .. })
    }
}

/// One layer: its key (a plain name), its WFS type and its reading.
pub type Layer = (&'static str, &'static str, Reading);

/// A WFS of a state read by a table.
pub struct TableSource {
    info: PermitSourceInfo,
    service: GeoJsonWfs,
    layers: &'static [Layer],
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
        let bbox = LatLonBox {
            min_lat: bbox.min().y,
            min_lon: bbox.min().x,
            max_lat: bbox.max().y,
            max_lon: bbox.max().x,
        };
        self.layers
            .iter()
            .map(|(_, type_name, reading)| {
                let features = get_features(fetcher, &self.service, type_name, &bbox, part)?;
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

/// A feature as it is stored: the attributes `reading` keeps and the geometry as longitude,
/// latitude.
fn stored(feature: &Value, reading: Reading) -> Option<Value> {
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
fn text(feature: &Value, key: &str) -> Option<String> {
    let value = feature.get("properties")?.get(key)?;
    let text = match value {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!text.is_empty() && text != "null").then_some(text)
}

/// A zone as the hints name it: a bare "I" or "IIIA" becomes "Schutzzone I"; a text that
/// already says zone stays.
fn zone_name(zone: &str) -> String {
    if zone.to_lowercase().contains("zone") {
        zone.to_string()
    } else {
        format!("Schutzzone {zone}")
    }
}

fn finding(reading: Reading, feature: &Value) -> Option<PermitFinding> {
    let geometry = geometry(feature)?;
    if !(matches!(geometry, Geometry::MultiPolygon(_)) || reading.point_allowed()) {
        return None;
    }
    let kind = match reading {
        Reading::Water {
            name,
            zone,
            citation,
        } => FindingKind::WaterProtection {
            name: text(feature, name),
            zone: text(feature, zone).map(|z| zone_name(&z)),
            citation: citation.and_then(|c| text(feature, c)),
        },
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
            office: format!("{prefix}{}", text(feature, office)?),
            unit: None,
        },
        Reading::Monument { kind, link } => FindingKind::Monument {
            kind: kind.to_string(),
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
            },
        ),
        (
            "monuments",
            "x:monuments",
            Reading::Monument {
                kind: "Baudenkmal",
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

    /// A water zone or a forest function must be an area; a point is left out.
    #[test]
    fn an_area_reading_leaves_points_out() {
        let point = serde_json::json!({"type": "Feature", "properties": {"NAME": "x"},
            "geometry": {"type": "Point", "coordinates": [49.3, 6.85]}});
        assert!(table().findings("zones", &[point]).is_empty());
    }
}
