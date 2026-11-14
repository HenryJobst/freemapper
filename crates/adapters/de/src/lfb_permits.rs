//! Brandenburg's source of the permit notes (ADR 0124): the Reviere of the Landesbetrieb Forst
//! Brandenburg (LFB), each with its Oberförsterei, from the INSPIRE WFS of the LFB
//! (dl-de/by-2-0, "Landesbetrieb Forst Brandenburg").
//!
//! The service is a GeoServer WFS 2.0.0 that answers GML 3.2 only (`gml32`; it refuses
//! `application/json`, and also its own default format). Asked with a WGS 84 box and `srsName`,
//! it writes the positions latitude first; the parser of the Berlin sources still takes
//! EPSG:25833, the default CRS of the service. The fetch keeps of every feature only the numbers
//! and names of the Oberförsterei and the Revier, so nothing else of the answer is stored.

use std::path::Path;

use freemapper_adapter_api::permits::{
    FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use freemapper_sources::geojson_wfs::{GeoJsonWfs, LatLonBox, get_gml_features};
use geo_types::Rect;
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::fetch::{FetchError, Fetcher};

/// The credit the licence asks for: the Quellenvermerk the LFB gives in its metadata, and the
/// licence of GovData's entry "Forstreviere im Land Brandenburg".
pub const CREDIT: &str = "Landesbetrieb Forst Brandenburg, Datenlizenz Deutschland – \
    Namensnennung – Version 2.0 (https://www.govdata.de/dl-de/by-2-0)";

/// The administration the notes name.
const LFB: &str = "Landesbetrieb Forst Brandenburg";

const SERVICE: GeoJsonWfs = GeoJsonWfs {
    base: "https://www.brandenburg-forst.de/geoserver/IFUEK/ows",
    output_format: "gml32",
    bbox_crs_suffix: true,
};

const TYPE_NAME: &str = "IFUEK:AM.ForestManagementArea";

/// The properties a Revier keeps. The others (`adresse`, and whatever the service adds) are
/// dropped before the features are stored.
const KEPT: [&str; 4] = ["foa_nr", "foa_name", "rev_nr", "rev_name"];

/// The Reviere of the LFB.
pub struct LfbReviere {
    info: PermitSourceInfo,
}

/// The one source of Brandenburg.
pub fn sources() -> Vec<Box<dyn PermitSource>> {
    vec![Box::new(LfbReviere {
        info: PermitSourceInfo {
            key: "reviere",
            label: Wording { en: LFB, de: LFB },
            what: Wording {
                en: "forest districts (Reviere) with their Oberförsterei",
                de: "Forstreviere mit ihrer Oberförsterei",
            },
            credit: CREDIT,
            note: None,
            layers: &["reviere"],
            sections: &[PermitSection::ForestUnits],
        },
    })]
}

impl PermitSource for LfbReviere {
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
        let features = get_gml_features(fetcher, &SERVICE, TYPE_NAME, &bbox, part)?;
        Ok(vec![features.iter().filter_map(stored).collect()])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != "reviere" {
            return Vec::new();
        }
        features.iter().filter_map(finding).collect()
    }
}

/// A feature as it is stored: the [`KEPT`] properties and the area as longitude, latitude.
/// `None` without a usable area.
fn stored(feature: &Value) -> Option<Value> {
    let area = geometry(feature)?;
    let properties: serde_json::Map<String, Value> = KEPT
        .iter()
        .filter_map(|key| {
            Some((
                key.to_string(),
                feature.get("properties")?.get(*key)?.clone(),
            ))
        })
        .collect();
    Some(serde_json::json!({
        "type": "Feature",
        "geometry": geojson::Geometry::new(geojson::GeometryValue::from(&area)),
        "properties": properties,
    }))
}

/// A Revier and its Oberförsterei; `None` without the name of the Oberförsterei or without an
/// area.
fn finding(feature: &Value) -> Option<PermitFinding> {
    let geometry =
        geometry(feature).filter(|g| matches!(g, geo_types::Geometry::MultiPolygon(_)))?;
    let text = |key: &str| {
        let text = feature.get("properties")?.get(key)?.as_str()?.trim();
        (!text.is_empty()).then(|| text.to_string())
    };
    Some(PermitFinding {
        kind: FindingKind::ForestUnit {
            administration: LFB.to_string(),
            office: format!("Oberförsterei {}", text("foa_name")?),
            unit: text("rev_name").map(|name| format!("Revier {name}")),
        },
        geometry,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use geo_types::Coord;

    const REVIERE: &str =
        include_str!("../../../../tests/fixtures/permits-brandenburg/reviere.gml");

    /// The box of the map area around Strausberg the fixture was recorded for.
    fn strausberg() -> Rect<f64> {
        Rect::new(Coord { x: 13.82, y: 52.52 }, Coord { x: 13.90, y: 52.58 })
    }

    fn fetched() -> Vec<Value> {
        let fetcher = MapFetcher::default();
        // Only a query in GML for the type of the Reviere is answered.
        fetcher.insert_prefix(
            "https://www.brandenburg-forst.de/geoserver/IFUEK/ows?service=WFS&version=2.0.0\
             &request=GetFeature&outputFormat=gml32&srsName=urn:ogc:def:crs:EPSG::4326\
             &typeNames=IFUEK:AM.ForestManagementArea&",
            REVIERE.as_bytes().to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let mut layers = sources()[0]
            .fetch(&fetcher, &strausberg(), &part.path().join("x.part"))
            .unwrap();
        assert_eq!(layers.len(), 1);
        assert_eq!(fetcher.calls.get(), 1, "one page");
        layers.remove(0)
    }

    /// The answer of the LFB, latitude first, becomes Reviere as longitude, latitude that keep
    /// only the numbers and names of the Revier and its Oberförsterei.
    #[test]
    fn reviere_are_stored_with_their_names_only() {
        let features = fetched();
        assert_eq!(features.len(), 3);
        for feature in &features {
            let keys: Vec<&String> = feature["properties"].as_object().unwrap().keys().collect();
            assert_eq!(
                keys,
                ["foa_name", "foa_nr", "rev_name", "rev_nr"],
                "{feature}"
            );
            let first = &feature["geometry"]["coordinates"][0][0][0];
            let (lon, lat) = (first[0].as_f64().unwrap(), first[1].as_f64().unwrap());
            assert!(
                (13.3..14.2).contains(&lon) && (52.3..52.9).contains(&lat),
                "{first}"
            );
        }
    }

    #[test]
    fn a_revier_names_its_oberfoersterei() {
        let found = sources()[0].findings("reviere", &fetched());
        let kinds: Vec<&FindingKind> = found.iter().map(|f| &f.kind).collect();
        let expected: Vec<FindingKind> = ["Rehfelde", "Neuenhagen", "Strausberg"]
            .iter()
            .map(|revier| FindingKind::ForestUnit {
                administration: "Landesbetrieb Forst Brandenburg".into(),
                office: "Oberförsterei Märkisch-Oderland".into(),
                unit: Some(format!("Revier {revier}")),
            })
            .collect();
        assert_eq!(kinds, expected.iter().collect::<Vec<_>>());
        assert!(sources()[0].findings("other", &fetched()).is_empty());
    }

    /// Stored before the swap to longitude first, or in the default CRS of the service, a
    /// Revier is still read in place.
    #[test]
    fn a_revier_in_the_default_crs_is_read_in_place() {
        let feature = serde_json::json!({
            "type": "Feature",
            "properties": {"foa_name": "Barnim", "rev_name": "Werneuchen"},
            "geometry": {"type": "Polygon", "coordinates": [[
                [421000.0, 5821000.0], [422000.0, 5821000.0],
                [422000.0, 5822000.0], [421000.0, 5821000.0]
            ]]},
        });
        let found = sources()[0].findings("reviere", &[feature]);
        assert_eq!(found.len(), 1);
        for c in geo::CoordsIter::coords_iter(&found[0].geometry) {
            assert!(
                (13.8..13.9).contains(&c.x) && (52.5..52.6).contains(&c.y),
                "{c:?}"
            );
        }
    }

    #[test]
    fn a_revier_without_its_oberfoersterei_is_left_out() {
        let mut features = fetched();
        features[0]["properties"]["foa_name"] = Value::String(" ".into());
        assert_eq!(sources()[0].findings("reviere", &features).len(), 2);
    }

    /// The Reviere around Strausberg, asked live through the hosts of Brandenburg's manifest
    /// with the fixed User-Agent of the program:
    /// `cargo test -p freemapper-adapters-de live_strausberg_reviere -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn live_strausberg_reviere() {
        use crate::fetch::HttpFetcher;
        use crate::states::{State, StateAdapter, register_backends};
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        use freemapper_adapter_api::policy::PolicyFetcher;
        use freemapper_adapter_api::registry::Registry;
        assert!(!freemapper::fetch::USER_AGENT.contains('@'));
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        let brandenburg = StateAdapter::new(State::Brandenburg, &backends);
        let http = HttpFetcher::new();
        let fetcher = PolicyFetcher::new(&http, brandenburg.policy().unwrap());
        let part = tempfile::tempdir().unwrap();
        let source = &brandenburg.permit_sources()[0];
        let layers = source
            .fetch(&fetcher, &strausberg(), &part.path().join("live.part"))
            .unwrap();
        let found = source.findings("reviere", &layers[0]);
        let units: std::collections::BTreeSet<String> = found
            .iter()
            .filter_map(|f| match &f.kind {
                FindingKind::ForestUnit { office, unit, .. } => {
                    Some(format!("{office} / {}", unit.as_deref().unwrap_or("-")))
                }
                _ => None,
            })
            .collect();
        // The probe of 2026-11-14 found these four.
        assert!(
            units.contains("Oberförsterei Märkisch-Oderland / Revier Strausberg"),
            "{units:?}"
        );
        assert!(units.len() >= 3, "{units:?}");
        for f in &found {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!(
                    (13.0..14.8).contains(&c.x) && (52.0..53.2).contains(&c.y),
                    "{c:?}"
                );
            }
        }
    }
}
