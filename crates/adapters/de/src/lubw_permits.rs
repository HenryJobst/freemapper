//! The water protection zones of Baden-Württemberg (ADR 0143): the WFS of the LUBW
//! Landesanstalt für Umwelt Baden-Württemberg, under its terms of use for data of the
//! Umweltinformationssystem (UIS), which ask for the credit line below. Each zone names the
//! authority in charge (`DST`, a Landratsamt or a Stadt), its status and its zone; zones I and II
//! come joined, as the LUBW publishes no zone I on the internet.
//!
//! The names come as the register writes them, in capitals, in quotes and with a number
//! (`\"MAHDENTAL\" 111032`); the notes write the name ("Mahdental").

use std::path::Path;

use freemapper_adapter_api::permits::{
    FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use freemapper_sources::geojson_wfs::{GeoJsonWfs, LatLonBox, get_features};
use geo_types::{Geometry, Rect};
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::fetch::{FetchError, Fetcher};
use crate::wfs_permits::{name_case, text, zone_name};

/// The service, which Baden-Württemberg's manifest names as a host.
pub const BASE: &str = "https://rips-gdi.lubw.baden-wuerttemberg.de/arcgis/services/wms/UIS_0100007500200036_AGGR/MapServer/WFSServer";

const TYPE_NAME: &str = "Wasserschutzgebietszone_M1";

/// The credit the terms of use ask for, verbatim.
pub(crate) const CREDIT: &str = "Grundlage: Daten aus dem Umweltinformationssystem (UIS) der LUBW Landesanstalt für Umwelt Baden-Württemberg";

/// The catalogue entry's note on the zones, verbatim.
const NOTE: &str = "Die Geometrien der Zonen I werden aus Sicherheitserwägungen nicht im Internet veröffentlicht. WSG-Zonen werden nur mit zusammengefassten Zonen I und II zum Herunterladen bereitgestellt.";

/// The attributes a zone keeps.
const KEPT: [&str; 4] = ["WSG", "ZONE", "STATUS", "DST"];

/// The source of Baden-Württemberg.
pub fn source() -> Box<dyn PermitSource> {
    Box::new(LubwSource {
        info: PermitSourceInfo {
            key: "lubw",
            label: Wording {
                en: "LUBW",
                de: "LUBW",
            },
            what: Wording {
                en: "water protection zones with the authority in charge",
                de: "Wasserschutzgebietszonen mit der zuständigen Behörde",
            },
            credit: CREDIT,
            note: Some(NOTE),
            sampled: false,
            gaps: None,
            layers: &["wasserschutzgebiete"],
            sections: &[PermitSection::WaterProtection],
            offices: None,
        },
        service: GeoJsonWfs {
            base: BASE,
            output_format: "GEOJSON",
            bbox_crs_suffix: true,
        },
    })
}

struct LubwSource {
    info: PermitSourceInfo,
    service: GeoJsonWfs,
}

impl PermitSource for LubwSource {
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
        let features = get_features(fetcher, &self.service, TYPE_NAME, &bbox, part)?;
        Ok(vec![features.iter().filter_map(stored).collect()])
    }

    fn findings(&self, _layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        features.iter().filter_map(finding).collect()
    }
}

/// A zone as it is stored: its area, longitude first, and the attributes of [`KEPT`].
fn stored(feature: &Value) -> Option<Value> {
    let area = geometry(feature).filter(|g| matches!(g, Geometry::MultiPolygon(_)))?;
    let properties: serde_json::Map<String, Value> = KEPT
        .iter()
        .filter_map(|key| Some((key.to_string(), feature["properties"].get(*key)?.clone())))
        .collect();
    Some(serde_json::json!({
        "type": "Feature",
        "geometry": geojson::Geometry::new(geojson::GeometryValue::from(&area)),
        "properties": properties,
    }))
}

fn finding(feature: &Value) -> Option<PermitFinding> {
    let geometry = geometry(feature).filter(|g| matches!(g, Geometry::MultiPolygon(_)))?;
    let provisional = text(feature, "STATUS").is_some_and(|s| s.starts_with("vorläufig"));
    Some(PermitFinding {
        kind: FindingKind::WaterProtection {
            name: text(feature, "WSG").map(|n| register_name(&n)),
            zone: text(feature, "ZONE").map(|z| {
                let z = zone_name(&z);
                if provisional {
                    format!("{z} (vorläufig angeordnet)")
                } else {
                    z
                }
            }),
            citation: None,
            authority: text(feature, "DST"),
        },
        geometry,
    })
}

/// A name of the register as a name: without the quotes, the backslashes and the number at
/// its end, in the case of a name.
fn register_name(raw: &str) -> String {
    let cleaned = raw.replace(['\\', '"'], " ");
    let mut words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.len() > 1
        && words
            .last()
            .is_some_and(|w| w.chars().all(|c| c.is_ascii_digit()))
    {
        words.pop();
    }
    name_case(&words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    #[test]
    fn register_names_become_names() {
        assert_eq!(register_name(r#"\"MAHDENTAL\" 111032"#), "Mahdental");
        assert_eq!(register_name(r#"\"DITZINGEN\""#), "Ditzingen");
        assert_eq!(register_name("Brunnen Nord"), "Brunnen Nord");
    }

    #[test]
    fn a_zone_names_its_authority() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            BASE,
            include_bytes!("../../../../tests/fixtures/permits-states/bw-wsg.geojson").to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let source = source();
        let bbox = Rect::new(
            geo_types::Coord { x: 9.0, y: 48.7 },
            geo_types::Coord { x: 9.3, y: 48.85 },
        );
        let answers = source
            .fetch(&fetcher, &bbox, &part.path().join("p.part"))
            .unwrap();
        // Only the kept attributes.
        assert!(answers[0][0]["properties"].get("OBJECTID").is_none());
        let kinds: Vec<FindingKind> = source
            .findings("wasserschutzgebiete", &answers[0])
            .into_iter()
            .map(|f| f.kind)
            .collect();
        let zone = |name: &str, zone: &str, authority: &str| FindingKind::WaterProtection {
            name: Some(name.into()),
            zone: Some(zone.into()),
            citation: None,
            authority: Some(authority.into()),
        };
        assert_eq!(
            kinds,
            [
                zone("Mahdental", "Zone IIIB", "Stadt Stuttgart"),
                zone("Ditzingen", "Zone III und IIIA", "Landratsamt Ludwigsburg"),
                zone(
                    "Ditzingen",
                    "Zone I und II bzw. IIA",
                    "Landratsamt Ludwigsburg"
                ),
            ]
        );
    }

    #[test]
    fn a_provisional_zone_says_so() {
        let feature = serde_json::json!({"type": "Feature",
            "properties": {"WSG": "BEISPIEL", "ZONE": "Zone IIIB", "STATUS": "vorläufig angeordnet",
                "DST": "Landratsamt Tübingen"},
            "geometry": {"type": "Polygon", "coordinates": [[
                [9.0, 48.5], [9.01, 48.5], [9.01, 48.51], [9.0, 48.5]]]}});
        let Some(PermitFinding {
            kind: FindingKind::WaterProtection { name, zone, .. },
            ..
        }) = finding(&feature)
        else {
            panic!()
        };
        assert_eq!(name.as_deref(), Some("Beispiel"));
        assert_eq!(zone.as_deref(), Some("Zone IIIB (vorläufig angeordnet)"));
    }
}
