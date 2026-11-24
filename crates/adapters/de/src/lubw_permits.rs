// SPDX-License-Identifier: AGPL-3.0-or-later

//! The fixtures of the water protection zones of Baden-Württemberg (ADR 0143), read through the
//! table its manifest declares (issue #273). The LUBW's WFS had its table in this file until it
//! moved into the manifest; these tests, which ran against the code, now run against the
//! declared table. The names come as the register writes them, in capitals, in quotes and with
//! a number (`\"MAHDENTAL\" 111032`); the notes write the name ("Mahdental").

mod tests {
    use std::path::Path;

    use crate::fetch::test_support::MapFetcher;
    use crate::wfs_permits::{declared_source, register_name};
    use freemapper_adapter_api::permits::{FindingKind, PermitSource};
    use geo_types::Rect;

    /// The service, which Baden-Württemberg's manifest names as a host.
    const BASE: &str = "https://rips-gdi.lubw.baden-wuerttemberg.de/arcgis/services/wms/UIS_0100007500200036_AGGR/MapServer/WFSServer";

    /// The source of the LUBW as the manifest of Baden-Württemberg declares it.
    fn source() -> Box<dyn PermitSource> {
        let text = include_str!("../manifests/badenwuerttemberg/manifest.toml");
        freemapper_manifest::parse(text, Path::new("badenwuerttemberg"))
            .unwrap()
            .permits
            .iter()
            .find(|w| w.key == "lubw")
            .map(declared_source)
            .unwrap()
    }

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
        assert!(source.info().note.unwrap().contains("Zonen I"));
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
        let found = source().findings("wasserschutzgebiete", &[feature]);
        let [
            freemapper_adapter_api::permits::PermitFinding {
                kind: FindingKind::WaterProtection { name, zone, .. },
                ..
            },
        ] = found.as_slice()
        else {
            panic!("{found:?}")
        };
        assert_eq!(name.as_deref(), Some("Beispiel"));
        assert_eq!(zone.as_deref(), Some("Zone IIIB (vorläufig angeordnet)"));
    }
}
