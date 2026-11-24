// SPDX-License-Identifier: AGPL-3.0-or-later

//! The fixtures of Hamburg's permit sources (ADR 0129), read through the tables its manifest
//! declares (issue #273). The six services had their table in this file until they moved into
//! the manifest; these tests, which ran against the code table, now run against the declared
//! one and so show that the manifest reads the answers alike.

mod tests {
    use std::path::Path;

    use crate::fetch::test_support::MapFetcher;
    use crate::wfs_permits::declared_source;
    use freemapper_adapter_api::permits::{
        DistrictOffices, FindingKind, ForestFunctionKind, PermitSource,
    };
    use geo_types::Rect;
    use serde_json::Value;

    macro_rules! fixture {
        ($name:literal) => {
            include_str!(concat!(
                "../../../../tests/fixtures/permits-hamburg/",
                $name
            ))
        };
    }

    fn features(collection: &str) -> Vec<Value> {
        let value: Value = serde_json::from_str(collection).unwrap();
        value["features"].as_array().unwrap().clone()
    }

    /// The sources the manifest of Hamburg declares, in its order.
    fn sources() -> Vec<Box<dyn PermitSource>> {
        let text = include_str!("../manifests/hamburg/manifest.toml");
        freemapper_manifest::parse(text, Path::new("hamburg"))
            .unwrap()
            .permits
            .iter()
            .map(declared_source)
            .collect()
    }

    fn source(key: &str) -> Box<dyn PermitSource> {
        sources().into_iter().find(|s| s.info().key == key).unwrap()
    }

    fn read(key: &str, layer: &str, collection: &str) -> Vec<FindingKind> {
        source(key)
            .findings(layer, &features(collection))
            .into_iter()
            .map(|f| f.kind)
            .collect()
    }

    #[test]
    fn six_sources_with_plain_keys() {
        let keys: Vec<&str> = sources().iter().map(|s| s.info().key).collect();
        assert_eq!(
            keys,
            [
                "bezirke",
                "gruenanlagen",
                "denkmale",
                "wald",
                "waldfunktionen",
                "wasserschutzgebiete",
                "naturdenkmale",
                "biotope"
            ]
        );
        for s in sources() {
            let info = s.info();
            assert!(freemapper_adapter_api::permits::plain_key(info.key));
            assert!(
                info.layers
                    .iter()
                    .all(|l| freemapper_adapter_api::permits::plain_key(l))
            );
            assert!(
                info.credit
                    .starts_with("Datenlizenz Deutschland Namensnennung 2.0")
            );
            assert_eq!(info.label.de, "Geoportal Hamburg");
            assert_eq!(
                info.offices.is_some(),
                info.key == "bezirke",
                "only the Bezirke name offices"
            );
        }
    }

    #[test]
    fn bezirke_name_hamburgs_offices() {
        let found = read("bezirke", "bezirke", fixture!("bezirke.geojson"));
        let names: Vec<String> = found
            .iter()
            .map(|k| match k {
                FindingKind::District { name } => name.clone(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(names, ["Hamburg-Mitte", "Altona", "Eimsbüttel"]);
        assert_eq!(
            source("bezirke").info().offices,
            Some(DistrictOffices::Hamburg)
        );
    }

    /// The positions come latitude first and are read as longitude, latitude.
    #[test]
    fn positions_are_read_as_longitude_latitude() {
        let found = source("gruenanlagen")
            .findings("gruenanlagen", &features(fixture!("gruenanlagen.geojson")));
        for f in &found {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!(
                    (9.9..10.1).contains(&c.x) && (53.5..53.6).contains(&c.y),
                    "{c:?}"
                );
            }
        }
    }

    #[test]
    fn green_spaces_take_their_name_and_bezirk() {
        let found = read(
            "gruenanlagen",
            "gruenanlagen",
            fixture!("gruenanlagen.geojson"),
        );
        let lines: Vec<String> = found
            .iter()
            .map(|k| match k {
                FindingKind::GreenSpace { name, district, .. } => {
                    format!("{name} | {}", district.as_deref().unwrap_or("-"))
                }
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            lines,
            [
                "Anderweitige Flächen | Hamburg-Mitte",
                "Planten un Blomen | Hamburg-Mitte",
                "Planten un Blomen | Hamburg-Mitte",
                "Kleine Wallanlagen | Hamburg-Mitte",
            ]
        );
        // The number is an index from 1 into the seven names.
        let at = |number: &str| {
            let feature = serde_json::json!({"type": "Feature", "properties": {
                "benennung": "x", "bezirk": number}, "geometry": {"type": "MultiPolygon",
                "coordinates": [[[[53.5, 9.9], [53.5, 9.91], [53.51, 9.91], [53.5, 9.9]]]]}});
            match source("gruenanlagen")
                .findings("gruenanlagen", &[feature])
                .remove(0)
                .kind
            {
                FindingKind::GreenSpace { district, .. } => district,
                other => panic!("{other:?}"),
            }
        };
        assert_eq!(at("7").as_deref(), Some("Harburg"));
        assert_eq!(at("0"), None);
        assert_eq!(at("8"), None);
    }

    #[test]
    fn monuments_are_counted_by_their_layer() {
        for (layer, file, kind) in [
            (
                "gartendenkmale",
                fixture!("gartendenkmale.geojson"),
                "Gartendenkmal",
            ),
            ("ensembles", fixture!("ensembles.geojson"), "Ensemble"),
            ("gebaeude", fixture!("gebaeude.geojson"), "Baudenkmal"),
        ] {
            let found = read("denkmale", layer, file);
            assert_eq!(found.len(), 2, "{layer}");
            assert!(found.iter().all(|k| *k
                == FindingKind::Monument {
                    kind: kind.into(),
                    link: None
                }));
        }
    }

    #[test]
    fn the_stadtwald_names_its_revierfoersterei_and_bannwald_its_name() {
        assert_eq!(
            read("wald", "stadtwald", fixture!("stadtwald.geojson")),
            [FindingKind::ForestUnit {
                administration: "Bezirksamt Altona".into(),
                office: "Revierförsterei Klövensteen".into(),
                unit: None,
            }]
        );
        assert_eq!(
            read("wald", "bannwald", fixture!("bannwald.geojson")),
            ["Die Reit", "Duvenstedter Brook"].map(|name| FindingKind::ForestFunction {
                kind: ForestFunctionKind::Protected,
                name: Some(name.into()),
            })
        );
        assert_eq!(
            read(
                "wald",
                "bannwald-heuckenlock",
                fixture!("bannwald-heuckenlock.geojson")
            )
            .len(),
            1
        );
        assert_eq!(
            read(
                "waldfunktionen",
                "erholungswald",
                fixture!("erholungswald.geojson")
            ),
            ["von hoher Bedeutung", "von sehr hoher Bedeutung"].map(|name| {
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Recreation,
                    name: Some(name.into()),
                }
            })
        );
    }

    #[test]
    fn water_protection_areas_carry_their_zone() {
        let found = read(
            "wasserschutzgebiete",
            "wasserschutzgebiete",
            fixture!("wasserschutzgebiete.geojson"),
        );
        let [
            FindingKind::WaterProtection {
                name,
                zone,
                citation,
                ..
            },
        ] = found.as_slice()
        else {
            panic!("{found:?}");
        };
        assert_eq!(name.as_deref(), Some("Baursberg"));
        assert_eq!(zone.as_deref(), Some("Schutzzone III"));
        assert!(
            citation
                .as_deref()
                .unwrap()
                .starts_with("Verordnung über das Wasserschutzgebiet")
        );
    }

    /// A monument keeps its kind, name and number and nothing else: the list describes the
    /// buildings with their architects, made up here.
    #[test]
    fn a_monument_is_stored_without_its_description() {
        let raw = serde_json::json!({
            "type": "Feature",
            "properties": {
                "kdstatus": "KD", "bezeichnung": "Etagenhaus", "fisid": "1",
                "info": "Beispielweg 1 (Etagenhaus / 1900 / Entwurf: Mustermann, Max)",
                "baujahr": "1900",
            },
            "geometry": {"type": "Polygon", "coordinates": [[
                [53.56, 9.98], [53.56, 9.99], [53.57, 9.99], [53.56, 9.98]
            ]]},
        });
        let fetcher = MapFetcher::default();
        let page = serde_json::to_vec(&serde_json::json!({
            "type": "FeatureCollection", "features": [raw]
        }))
        .unwrap();
        fetcher.insert_prefix("https://geodienste.hamburg.de/HH_WFS_Denkmalschutz", page);
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            geo_types::Coord { x: 9.97, y: 53.55 },
            geo_types::Coord { x: 10.0, y: 53.57 },
        );
        let layers = source("denkmale")
            .fetch(&fetcher, &bbox, &part.path().join("x.part"))
            .unwrap();
        assert_eq!(layers.len(), 3);
        let text = serde_json::to_string(&layers).unwrap();
        assert!(
            !text.contains("Mustermann") && !text.contains("info"),
            "{text}"
        );
        // The kind comes from the layer, so no attribute is kept.
        assert!(layers[0][0]["properties"].as_object().unwrap().is_empty());
        // Stored as longitude, latitude.
        assert_eq!(layers[0][0]["geometry"]["coordinates"][0][0][0][0], 9.98);
    }

    #[test]
    fn the_services_are_asked_for_geojson() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://geodienste.hamburg.de/HH_WFS_Wasserschutzgebiete?service=WFS&version=2.0.0\
             &request=GetFeature&outputFormat=application/geo%2Bjson&",
            fixture!("wasserschutzgebiete.geojson").as_bytes().to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            geo_types::Coord { x: 9.70, y: 53.59 },
            geo_types::Coord { x: 9.76, y: 53.62 },
        );
        let layers = source("wasserschutzgebiete")
            .fetch(&fetcher, &bbox, &part.path().join("x.part"))
            .unwrap();
        assert_eq!(layers[0].len(), 1);
        assert_eq!(fetcher.calls.get(), 1);
    }

    /// The six services around Planten un Blomen and at the Klövensteen, asked live through the
    /// hosts of Hamburg's manifest with the fixed User-Agent of the program:
    /// `cargo test -p freemapper-adapters-de live_hamburg_permit_sources -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn live_hamburg_permit_sources() {
        use crate::fetch::HttpFetcher;
        use crate::states::{State, StateAdapter, register_backends};
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        use freemapper_adapter_api::policy::PolicyFetcher;
        use freemapper_adapter_api::registry::Registry;
        assert!(!freemapper::fetch::USER_AGENT.contains('@'));
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        let hamburg = StateAdapter::new(State::Hamburg, &backends);
        let http = HttpFetcher::new();
        let fetcher = PolicyFetcher::new(&http, hamburg.policy().unwrap());
        let part = tempfile::tempdir().unwrap();
        let part = part.path().join("live.part");
        let mut found = Vec::new();
        for bbox in [
            Rect::new(
                geo_types::Coord {
                    x: 9.975,
                    y: 53.555,
                },
                geo_types::Coord {
                    x: 9.995,
                    y: 53.565,
                },
            ),
            Rect::new(
                geo_types::Coord { x: 9.70, y: 53.59 },
                geo_types::Coord { x: 9.76, y: 53.62 },
            ),
        ] {
            for source in hamburg.permit_sources() {
                let answers = source.fetch(&fetcher, &bbox, &part).unwrap();
                for (layer, features) in source.info().layers.iter().zip(&answers) {
                    found.extend(source.findings(layer, features));
                }
            }
        }
        let has = |test: &dyn Fn(&FindingKind) -> bool| found.iter().any(|f| test(&f.kind));
        assert!(has(
            &|k| matches!(k, FindingKind::District { name } if name == "Hamburg-Mitte")
        ));
        assert!(has(
            &|k| matches!(k, FindingKind::GreenSpace { name, .. } if name == "Planten un Blomen")
        ));
        assert!(has(
            &|k| matches!(k, FindingKind::Monument { kind, .. } if kind == "Gartendenkmal")
        ));
        assert!(has(
            &|k| matches!(k, FindingKind::ForestUnit { office, .. } if office == "Revierförsterei Klövensteen")
        ));
        assert!(has(&|k| matches!(
            k,
            FindingKind::ForestFunction {
                kind: ForestFunctionKind::Recreation,
                ..
            }
        )));
        assert!(has(
            &|k| matches!(k, FindingKind::WaterProtection { name: Some(n), .. } if n == "Baursberg")
        ));
        for f in &found {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!(
                    (8.0..10.5).contains(&c.x) && (53.3..54.1).contains(&c.y),
                    "{c:?}"
                );
            }
        }
    }
}
