//! The drinking water and medicinal spring protection areas of the BfG (ADR 0142), for the
//! states whose own services give none: the areas the states report to WasserBLIcK, from the
//! ArcGIS REST service `INSPIRE/AM` of the Bundesanstalt für Gewässerkunde (CC BY 4.0). They
//! come without their zones, often as one feature per zone; the fetch joins the features of
//! one name into one area, so a share counts each piece of ground once.

use std::collections::BTreeMap;
use std::path::Path;

use freemapper_adapter_api::permits::{
    FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use geo::BooleanOps as _;
use geo_types::{Geometry, MultiPolygon, Rect};
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::wfs_permits::text;

const BASE: &str = "https://geoportal.bafg.de/arcgis1/rest/services/INSPIRE/AM/MapServer";

/// The credit the catalogue entry asks for, with the year of the fetch.
const CREDIT: &str = "© WasserBLIcK/BfG und Zuständige Behörden der Länder, {year}; Lizenz: Creative Commons Namensnennung – 4.0 International (CC BY 4.0)";

/// The catalogue entry's note on the coverage, verbatim.
const NOTE: &str =
    "Die bundesweite Flächendeckung des Datenbestandes ist nicht in jedem Fall gewährleistet.";

/// Features per page; the service gives at most 2000.
const PAGE: usize = 1000;
const MAX_PAGES: usize = 20;
const MAX_BYTES: u64 = 64 << 20;

/// The layers: drinking water protection areas (1) and medicinal spring protection areas (2).
const LAYERS: &[(&str, u32)] = &[("trinkwasser", 1), ("heilquellen", 2)];

/// The source of the BfG.
pub fn source() -> Box<dyn PermitSource> {
    Box::new(BfgSource {
        info: PermitSourceInfo {
            key: "bfg",
            label: Wording {
                en: "BfG (WasserBLIcK)",
                de: "BfG (WasserBLIcK)",
            },
            what: Wording {
                en: "drinking water and medicinal spring protection areas, without their zones",
                de: "Trinkwasser- und Heilquellenschutzgebiete, ohne Schutzzonen",
            },
            credit: CREDIT,
            note: Some(NOTE),
            sampled: false,
            layers: &["trinkwasser", "heilquellen"],
            sections: &[PermitSection::WaterProtection],
            offices: None,
        },
    })
}

struct BfgSource {
    info: PermitSourceInfo,
}

impl PermitSource for BfgSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        LAYERS
            .iter()
            .map(|(_, layer)| {
                let mut features = Vec::new();
                for page in 0..MAX_PAGES {
                    let url = query_url(*layer, bbox, page * PAGE);
                    let (found, more) =
                        get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| {
                            read_page(&url, bytes)
                        })?;
                    features.extend(found);
                    if !more {
                        break;
                    }
                }
                Ok(joined(&features))
            })
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let spring = layer == "heilquellen";
        features
            .iter()
            .filter_map(|feature| {
                let name = text(feature, "Name").map(|n| {
                    if spring {
                        format!("{n} (Heilquellenschutzgebiet)")
                    } else {
                        n
                    }
                });
                Some(PermitFinding {
                    kind: FindingKind::WaterProtection {
                        name,
                        zone: None,
                        citation: None,
                    },
                    geometry: geometry(feature)?,
                })
            })
            .collect()
    }
}

/// The query of `layer` for the areas that meet `bbox`, from the feature `offset` on, as
/// GeoJSON in WGS 84.
fn query_url(layer: u32, bbox: &Rect<f64>, offset: usize) -> String {
    format!(
        "{BASE}/{layer}/query?where=1%3D1&geometry={:.6},{:.6},{:.6},{:.6}\
         &geometryType=esriGeometryEnvelope&inSR=4326&spatialRel=esriSpatialRelIntersects\
         &outFields=Name&returnGeometry=true&outSR=4326&geometryPrecision=6\
         &orderByFields=OBJECTID&resultOffset={offset}&resultRecordCount={PAGE}&f=geojson",
        bbox.min().x,
        bbox.min().y,
        bbox.max().x,
        bbox.max().y,
    )
}

/// The features of one page and whether more follow (`exceededTransferLimit`, which the
/// service writes at the top and in `properties`).
fn read_page(url: &str, bytes: &[u8]) -> Result<(Vec<Value>, bool), FetchError> {
    let wrong = |message: &str| FetchError::Http {
        url: url.to_string(),
        message: message.to_string(),
    };
    let mut value: Value =
        serde_json::from_slice(bytes).map_err(|_| wrong("the answer is not GeoJSON"))?;
    if let Some(error) = value.get("error") {
        return Err(wrong(&format!("the service answers an error: {error}")));
    }
    let more = [
        &value["exceededTransferLimit"],
        &value["properties"]["exceededTransferLimit"],
    ]
    .iter()
    .any(|v| v.as_bool() == Some(true));
    match value.get_mut("features").map(Value::take) {
        Some(Value::Array(features)) => Ok((features, more)),
        _ => Err(wrong("the answer is not a FeatureCollection")),
    }
}

/// The features of one name joined into one area, by name; the name repaired where the data
/// carry UTF-8 read as Windows-1252 ("BÃ¶rÃŸum" for "Börßum").
fn joined(features: &[Value]) -> Vec<Value> {
    let mut by_name: BTreeMap<Option<String>, MultiPolygon<f64>> = BTreeMap::new();
    for feature in features {
        let Some(Geometry::MultiPolygon(area)) = geometry(feature) else {
            continue;
        };
        let name = text(feature, "Name").map(|n| repaired(&n));
        by_name
            .entry(name)
            .and_modify(|joined| *joined = joined.union(&area))
            .or_insert(area);
    }
    by_name
        .into_iter()
        .map(|(name, area)| {
            serde_json::json!({
                "type": "Feature",
                "geometry": geojson::Geometry::new(geojson::GeometryValue::from(&area)),
                "properties": { "Name": name },
            })
        })
        .collect()
}

/// `text` with UTF-8 that was read as Windows-1252 turned back ("ÃŸ" for "ß"); any other
/// text stays.
fn repaired(text: &str) -> String {
    if !text.contains(['Ã', 'Â']) {
        return text.to_string();
    }
    let bytes: Option<Vec<u8>> = text.chars().map(windows_1252).collect();
    bytes
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_else(|| text.to_string())
}

/// The byte of `c` in Windows-1252, which is Latin-1 except for 0x80 to 0x9F.
fn windows_1252(c: char) -> Option<u8> {
    const HIGH: [(char, u8); 27] = [
        ('€', 0x80),
        ('‚', 0x82),
        ('ƒ', 0x83),
        ('„', 0x84),
        ('…', 0x85),
        ('†', 0x86),
        ('‡', 0x87),
        ('ˆ', 0x88),
        ('‰', 0x89),
        ('Š', 0x8A),
        ('‹', 0x8B),
        ('Œ', 0x8C),
        ('Ž', 0x8E),
        ('‘', 0x91),
        ('’', 0x92),
        ('“', 0x93),
        ('”', 0x94),
        ('•', 0x95),
        ('–', 0x96),
        ('—', 0x97),
        ('˜', 0x98),
        ('™', 0x99),
        ('š', 0x9A),
        ('›', 0x9B),
        ('œ', 0x9C),
        ('ž', 0x9E),
        ('Ÿ', 0x9F),
    ];
    match u8::try_from(u32::from(c)) {
        Ok(b) if !(0x80..=0x9F).contains(&b) => Some(b),
        _ => HIGH.iter().find(|(h, _)| *h == c).map(|(_, b)| *b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use geo::Area as _;

    fn square(w: f64, s: f64, side: f64, name: &str) -> Value {
        serde_json::json!({"type": "Feature", "properties": {"Name": name},
            "geometry": {"type": "Polygon", "coordinates": [[
                [w, s], [w + side, s], [w + side, s + side], [w, s + side], [w, s]]]}})
    }

    #[test]
    fn names_read_as_windows_1252_are_repaired() {
        assert_eq!(repaired("BÃ¶rÃŸum"), "Börßum");
        assert_eq!(repaired("Börßum"), "Börßum");
        assert_eq!(repaired("Bad Harzburg"), "Bad Harzburg");
    }

    #[test]
    fn the_zones_of_one_name_become_one_area() {
        // Zone III with zone II inside it, and another area.
        let features = [
            square(10.0, 51.0, 0.02, "Bad Harzburg"),
            square(10.005, 51.005, 0.005, "Bad Harzburg"),
            square(10.5, 51.0, 0.01, "BÃ¶rÃŸum"),
        ];
        let joined = joined(&features);
        assert_eq!(joined.len(), 2);
        assert_eq!(joined[0]["properties"]["Name"], "Bad Harzburg");
        assert_eq!(joined[1]["properties"]["Name"], "Börßum");
        let Some(Geometry::MultiPolygon(area)) = geometry(&joined[0]) else {
            panic!()
        };
        assert!((area.unsigned_area() - 0.0004).abs() < 1e-9);
    }

    #[test]
    fn the_fetch_follows_the_pages_and_names_the_springs() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(
            geo_types::Coord { x: 10.0, y: 51.0 },
            geo_types::Coord { x: 10.6, y: 51.1 },
        );
        let page = |features: Vec<Value>, more: bool| {
            serde_json::to_vec(&serde_json::json!({"type": "FeatureCollection",
                "features": features, "properties": {"exceededTransferLimit": more}}))
            .unwrap()
        };
        fetcher.insert(
            &query_url(1, &bbox, 0),
            page(vec![square(10.0, 51.0, 0.02, "Bad Harzburg")], true),
        );
        fetcher.insert(
            &query_url(1, &bbox, PAGE),
            page(vec![square(10.5, 51.0, 0.01, "Börßum")], false),
        );
        fetcher.insert(
            &query_url(2, &bbox, 0),
            page(vec![square(10.0, 51.0, 0.01, "Bad Harzburg")], false),
        );
        let part = tempfile::tempdir().unwrap();
        let source = source();
        let answers = source
            .fetch(&fetcher, &bbox, &part.path().join("p.part"))
            .unwrap();
        assert_eq!(answers.iter().map(Vec::len).collect::<Vec<_>>(), [2, 1]);
        let names = |layer: &str, features: &[Value]| -> Vec<Option<String>> {
            source
                .findings(layer, features)
                .into_iter()
                .map(|f| match f.kind {
                    FindingKind::WaterProtection { name, zone, .. } => {
                        assert_eq!(zone, None);
                        name
                    }
                    other => panic!("{other:?}"),
                })
                .collect()
        };
        assert_eq!(
            names("trinkwasser", &answers[0]),
            [Some("Bad Harzburg".into()), Some("Börßum".into())]
        );
        assert_eq!(
            names("heilquellen", &answers[1]),
            [Some("Bad Harzburg (Heilquellenschutzgebiet)".into())]
        );
    }

    #[test]
    fn an_error_answer_is_an_error() {
        assert!(read_page("u", br#"{"error":{"code":400,"message":"x"}}"#).is_err());
        assert!(read_page("u", b"<html>").is_err());
    }
}
