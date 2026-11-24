// SPDX-License-Identifier: AGPL-3.0-or-later

//! The fixtures of Brandenburg's permit sources (ADR 0124), read through the tables its manifest
//! declares (issue #273): the Reviere and the forest functions of the Landesbetrieb Forst
//! Brandenburg (LFB) and the water protection areas of the LfU. The tests of the Reviere read
//! the answers of the LFB: the INSPIRE WFS is a GeoServer that answers GML 3.2 only (`gml32`; it
//! refuses `application/json`) and writes the positions latitude first.

mod tests {
    use std::path::Path;

    use crate::fetch::test_support::MapFetcher;
    use crate::wfs_permits::declared_source;
    use freemapper_adapter_api::permits::{FindingKind, PermitSource};
    use geo_types::{Coord, Rect};
    use serde_json::Value;

    /// The sources the manifest of Brandenburg declares; the first is the Reviere.
    fn sources() -> Vec<Box<dyn PermitSource>> {
        let text = include_str!("../manifests/brandenburg/manifest.toml");
        freemapper_manifest::parse(text, Path::new("brandenburg"))
            .unwrap()
            .permits
            .iter()
            .map(declared_source)
            .collect()
    }

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
            assert_eq!(keys, ["foa_name", "rev_name"], "{feature}");
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
        let sources = brandenburg.permit_sources();
        let source = sources.iter().find(|s| s.info().key == "reviere").unwrap();
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
