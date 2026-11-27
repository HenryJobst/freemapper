// SPDX-License-Identifier: AGPL-3.0-or-later

//! The fixtures of the permit source of Saxony (ADR 0139): two WMS of the Staatsbetrieb
//! Sachsenforst, asked at points. The table of the two queries was written here until it moved
//! into the manifest of Saxony (`[[permits.getfeatureinfo]]`, issue #273); these tests, which
//! ran against the code, now run against the declared table. "Wald nach SächsWaldG" gives the
//! Revier and the Forstbezirk at a point, "Waldfunktionen in Sachsen" the functions of the
//! forest there: Erholungswald, Naturwaldzellen, Bodenschutzwald and the drinking water
//! protection zones (in the forest only).

#[cfg(test)]
mod tests {
    use std::path::Path;

    use freemapper_adapter_api::permits::{
        FindingKind, ForestFunctionKind, PermitSection, PermitSource,
    };
    use serde_json::Value;

    use crate::wfs_permits::declared_source;

    /// The source of Saxony as its manifest declares it.
    fn source() -> Box<dyn PermitSource> {
        let text = include_str!("../manifests/saxony/manifest.toml");
        freemapper_manifest::parse(text, Path::new("saxony"))
            .unwrap()
            .permits
            .iter()
            .find(|w| w.key == "sachsenforst")
            .map(declared_source)
            .unwrap()
    }

    fn kinds(layer: &str, collection: &str) -> Vec<FindingKind> {
        // A stored answer: the trimmed live features with a cell each, as the fetch writes them.
        let value: Value = serde_json::from_str(collection).unwrap();
        let cell = serde_json::json!({"type": "Polygon", "coordinates": [[
            [13.8, 51.05], [13.81, 51.05], [13.81, 51.06], [13.8, 51.06], [13.8, 51.05]]]});
        let features: Vec<Value> = value["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| serde_json::json!({"type": "Feature", "geometry": cell, "properties": f["properties"]}))
            .collect();
        source()
            .findings(layer, &features)
            .into_iter()
            .map(|f| f.kind)
            .collect()
    }

    #[test]
    fn a_revier_names_its_forstbezirk() {
        assert_eq!(
            kinds(
                "reviere",
                include_str!("../../../../tests/fixtures/permits-states/sn-reviere.geojson")
            ),
            [
                FindingKind::ForestUnit {
                    administration: "Staatsbetrieb Sachsenforst".into(),
                    office: "Forstbezirk Dresden".into(),
                    unit: Some("Revier Bühlau".into()),
                },
                // The Reviere cut the name of the national park's office at 40 characters.
                FindingKind::ForestUnit {
                    administration: "Staatsbetrieb Sachsenforst".into(),
                    office: "Nationalpark- und Forstverwaltung Sächsische Schweiz".into(),
                    unit: Some("Revier Hohnstein".into()),
                },
            ]
        );
    }

    #[test]
    fn the_functions_read_their_key_and_the_zone() {
        assert_eq!(
            kinds(
                "waldfunktionen",
                include_str!("../../../../tests/fixtures/permits-states/sn-waldfunktionen.geojson")
            ),
            [
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Protected,
                    name: Some("Bodenschutzwald".into()),
                },
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Natural,
                    name: Some("Wodrich".into()),
                },
                FindingKind::WaterProtection {
                    name: Some("Arnsdorf-Wasserwerk Karswald".into()),
                    zone: Some("Schutzzone III".into()),
                    citation: None,
                    authority: None,
                },
                FindingKind::WaterProtection {
                    name: Some("Talsperre Eibenstock".into()),
                    zone: Some("Schutzzone II A".into()),
                    citation: None,
                    authority: None,
                },
                FindingKind::WaterProtection {
                    name: Some("Speichersystem Radeburg (WW Rödern)".into()),
                    zone: Some("Schutzzone I".into()),
                    citation: None,
                    authority: None,
                },
            ]
        );
    }

    /// What the code table said, word for word: the source, its sections and its two queries.
    #[test]
    fn the_declared_table_is_the_one_the_code_had() {
        let text = include_str!("../manifests/saxony/manifest.toml");
        let manifest = freemapper_manifest::parse(text, Path::new("saxony")).unwrap();
        let declared = manifest
            .permits
            .iter()
            .find(|w| w.key == "sachsenforst")
            .unwrap();
        assert!(declared.getfeatureinfo && !declared.shapefile && declared.url.is_empty());
        let source = declared_source(declared);
        let info = source.info();
        assert_eq!(info.label.de, "Staatsbetrieb Sachsenforst");
        assert_eq!(info.label.en, "Staatsbetrieb Sachsenforst");
        assert_eq!(
            info.credit,
            "Datenlizenz Deutschland 2.0 – Namensnennung (dl-de/by-2-0) unter Angabe der Quelle: \"Geodaten Sachsenforst\""
        );
        assert!(info.sampled && info.note.is_none() && info.gaps.is_none());
        assert_eq!(info.layers, ["reviere", "waldfunktionen"]);
        assert_eq!(
            info.sections,
            [
                PermitSection::ForestUnits,
                PermitSection::ForestFunctions,
                PermitSection::WaterProtection,
            ]
        );
        let queries: Vec<_> = declared
            .layers
            .iter()
            .map(|l| l.query.clone().unwrap())
            .collect();
        assert_eq!(
            queries
                .iter()
                .map(|q| (q.url.as_str(), q.layers.as_str(), q.one))
                .collect::<Vec<_>>(),
            [
                (
                    "https://www.forsten.sachsen.de/kartendienste/wald/MapServer/WMSServer",
                    "Reviere",
                    true
                ),
                (
                    "https://www.forsten.sachsen.de/kartendienste/waldfunktionen/MapServer/WMSServer",
                    "7,46,101,80,81,82,83,84,86,87,88,90,91,92,93,94",
                    false
                ),
            ]
        );
    }

    /// The functions are told by their code: the three of the forest, and the codes of five
    /// characters that start with 211 for a zone; everything else is left out.
    #[test]
    fn a_code_that_is_none_of_them_is_left_out() {
        let feature = |code: &str, name: Option<&str>, short: &str| {
            let mut properties = serde_json::json!({"WFZ": code, "KNM": short});
            if let Some(name) = name {
                properties["GNM"] = name.into();
            }
            serde_json::json!({"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [[
                [13.8, 51.05], [13.81, 51.05], [13.81, 51.06], [13.8, 51.06], [13.8, 51.05]]]},
                "properties": properties})
        };
        let found = |code: &str, name: Option<&str>, short: &str| {
            source()
                .findings("waldfunktionen", &[feature(code, name, short)])
                .into_iter()
                .map(|f| f.kind)
                .collect::<Vec<_>>()
        };
        assert!(found("7212", None, "ERH II").is_empty());
        assert!(found("2111", Some("x"), "§ TW GW Z I").is_empty());
        assert!(found("211111", Some("x"), "§ TW GW Z I").is_empty());
        assert!(found("311xx", Some("x"), "§ TW GW Z I").is_empty());
        assert_eq!(
            found("7110", Some("Park"), ""),
            [FindingKind::ForestFunction {
                kind: ForestFunctionKind::Recreation,
                name: Some("Park".into())
            }]
        );
        assert_eq!(
            found("7110", None, ""),
            [FindingKind::ForestFunction {
                kind: ForestFunctionKind::Recreation,
                name: None
            }]
        );
        // A zone without the text " Z " in its short name has no zone.
        assert_eq!(
            found("21199", Some("Quelle"), "§ TW"),
            [FindingKind::WaterProtection {
                name: Some("Quelle".into()),
                zone: None,
                citation: None,
                authority: None
            }]
        );
    }

    /// The name of the national park's office is cut by the Reviere at 40 characters; whatever
    /// starts with "Nationalpark" gets the whole name, every other office its prefix.
    #[test]
    fn the_office_of_the_national_park_is_named_whole() {
        let revier = |office: &str| {
            let feature = serde_json::json!({"type": "Feature", "geometry": {"type": "Polygon",
                "coordinates": [[[13.8, 51.05], [13.81, 51.05], [13.81, 51.06], [13.8, 51.06], [13.8, 51.05]]]},
                "properties": {"FBZ_NAME": office, "REV_NAME": "R"}});
            source().findings("reviere", &[feature])[0].kind.clone()
        };
        let office = |kind| match kind {
            FindingKind::ForestUnit { office, .. } => office,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            office(revier("Nationalpark")),
            "Nationalpark- und Forstverwaltung Sächsische Schweiz"
        );
        assert_eq!(office(revier("Leipzig")), "Forstbezirk Leipzig");
        assert_eq!(office(revier("Neustadt")), "Forstbezirk Neustadt");
        assert_eq!(
            office(revier("Mein Nationalpark")),
            "Forstbezirk Mein Nationalpark"
        );
    }
}
