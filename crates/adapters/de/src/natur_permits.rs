// SPDX-License-Identifier: AGPL-3.0-or-later

//! The Naturdenkmale and geschützte Landschaftsbestandteile of Lower Saxony (ADR 0146, 0148):
//! the NLWKN's map service (ArcGIS REST) is declared in the manifest of the state as
//! `[[permits.arcgis]]` (issue #273). These tests hold the declared table to what the table in
//! the code gave, and read the service through it.

use std::path::Path;

use freemapper_adapter_api::permits::{PermitSection, PermitSource, Wording};

/// The map service of the NLWKN, which Lower Saxony's manifest names as a host.
const NIEDERSACHSEN: &str =
    "https://www.umweltkarten-niedersachsen.de/arcgis/rest/services/Natur_wms/MapServer";

/// The source of Lower Saxony, as its manifest declares it.
fn niedersachsen() -> Box<dyn PermitSource> {
    let text = include_str!("../manifests/niedersachsen/manifest.toml");
    freemapper_manifest::parse(text, Path::new("niedersachsen"))
        .unwrap()
        .permits
        .iter()
        .find(|w| w.key == "naturschutz")
        .map(crate::wfs_permits::declared_source)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arcgis_permits::query_url;
    use crate::fetch::test_support::MapFetcher;
    use freemapper_adapter_api::permits::{FindingKind, ProtectedObjectKind};
    use geo_types::{Coord, Rect};

    /// The table the code had, word for word: the declared source says the same.
    #[test]
    fn the_declared_source_is_the_table_the_code_had() {
        let source = niedersachsen();
        let info = source.info();
        assert_eq!(info.key, "naturschutz");
        assert_eq!(
            info.label,
            Wording {
                en: "NLWKN",
                de: "NLWKN"
            }
        );
        assert_eq!(
            info.what,
            Wording {
                en: "Naturdenkmale and geschützte Landschaftsbestandteile",
                de: "Naturdenkmale und geschützte Landschaftsbestandteile",
            }
        );
        assert_eq!(
            info.credit,
            "Datenlizenz Deutschland – Namensnennung – Version 2.0 (dl-de/by-2-0); Quelle: Niedersächsischer Landesbetrieb für Wasserwirtschaft, Küsten- und Naturschutz (NLWKN)"
        );
        assert_eq!(
            info.layers,
            [
                "naturdenkmale-flaechen",
                "naturdenkmale-punkte",
                "landschaftsbestandteile-flaechen",
                "landschaftsbestandteile-punkte",
            ]
        );
        assert_eq!(info.sections, [PermitSection::ProtectedObjects]);
        assert_eq!(info.note, None);
        assert!(!info.sampled);
        assert_eq!(info.offices, None);
    }

    #[test]
    fn lower_saxony_names_the_gap_of_its_biotopes() {
        let source = niedersachsen();
        let gaps = source.info().gaps.expect("the biotopes are missing");
        assert!(gaps.en.contains("biotopes"));
        assert!(gaps.de.contains("Biotope"));
    }

    /// Each layer is asked by its number, for the attribute `name` only, and reads the kind the
    /// code gave it.
    #[test]
    fn it_asks_the_four_layers_by_their_numbers_and_reads_their_kinds() {
        let source = niedersachsen();
        let bbox = Rect::new(Coord { x: 9.9, y: 51.52 }, Coord { x: 9.98, y: 51.56 });
        let fetcher = MapFetcher::default();
        let page = |kind: &str| {
            serde_json::to_vec(&serde_json::json!({"type": "FeatureCollection",
                "features": [{"type": "Feature",
                    "properties": {"name": kind, "vollzug": "x"},
                    "geometry": {"type": "Point", "coordinates": [9.95, 51.54]}}]}))
            .unwrap()
        };
        for (number, kind) in [(190, "A"), (7, "B"), (10, "C"), (12, "D")] {
            fetcher.insert(
                &query_url(NIEDERSACHSEN, number, "name", &bbox, 0),
                page(kind),
            );
        }
        let part = tempfile::tempdir().unwrap();
        let answers = source
            .fetch(&fetcher, &bbox, &part.path().join("p.part"))
            .unwrap();
        assert_eq!(answers.len(), 4);
        let kinds = [
            ProtectedObjectKind::NaturalMonument,
            ProtectedObjectKind::NaturalMonument,
            ProtectedObjectKind::ProtectedLandscapePart,
            ProtectedObjectKind::ProtectedLandscapePart,
        ];
        for (i, layer) in source.info().layers.iter().enumerate() {
            assert_eq!(answers[i].len(), 1, "{layer}");
            assert!(answers[i][0]["properties"].get("vollzug").is_none());
            let found = source.findings(layer, &answers[i]);
            match &found[0].kind {
                FindingKind::ProtectedObject { kind, name, .. } => {
                    assert_eq!(*kind, kinds[i], "{layer}");
                    assert_eq!(name.as_deref(), Some(["A", "B", "C", "D"][i]));
                }
                other => panic!("{other:?}"),
            }
        }
    }
}
