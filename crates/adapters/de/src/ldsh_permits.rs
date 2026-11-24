// SPDX-License-Identifier: AGPL-3.0-or-later

//! The monuments of Schleswig-Holstein (ADR 0145): the Landesamt für Denkmalpflege publishes its
//! list of monuments only as one GeoJSON file for the whole state, now declared in the manifest
//! of the state as `[[permits.geojson]]` (issue #273). These tests hold the declared table to what
//! the table in the code gave, and read the file through it.
//!
//! The archaeological monuments are not in it: their office allows their publication only with
//! its written consent. Lübeck keeps its own list.

use std::path::Path;

use freemapper_adapter_api::permits::{PermitSection, PermitSource, Wording};
use geo_types::Rect;

use crate::crs::Crs;
use crate::fetch::CacheDir;

/// The current file; the server sends it on to the file of the week.
const URL: &str =
    "https://opendata.schleswig-holstein.de/collection/geodaten-denkmalliste-sh/aktuell.geojson";

/// The source of Schleswig-Holstein, as its manifest declares it.
fn source() -> Box<dyn PermitSource> {
    let text = include_str!("../manifests/schleswigholstein/manifest.toml");
    freemapper_manifest::parse(text, Path::new("schleswigholstein"))
        .unwrap()
        .permits
        .iter()
        .find(|w| w.key == "denkmale")
        .map(crate::wfs_permits::declared_source)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::fetch::test_support::MapFetcher;
    use crate::geojson_permits::looks_like_geojson;
    use freemapper_adapter_api::permits::FindingKind;

    /// The table the code had, word for word: the declared source says the same.
    #[test]
    fn the_declared_source_is_the_table_the_code_had() {
        let source = source();
        let info = source.info();
        assert_eq!(info.key, "denkmale");
        assert_eq!(
            info.label,
            Wording {
                en: "Landesamt für Denkmalpflege Schleswig-Holstein",
                de: "Landesamt für Denkmalpflege Schleswig-Holstein",
            }
        );
        assert_eq!(
            info.what,
            Wording {
                en: "list of monuments (one file for the state, kept in the cache)",
                de: "Denkmalliste (eine Datei für das Bundesland, im Zwischenspeicher)",
            }
        );
        assert_eq!(
            info.credit,
            "Landesamt für Denkmalpflege Schleswig-Holstein; Lizenz: Creative Commons Namensnennung 4.0 International (CC BY 4.0)"
        );
        assert_eq!(info.note, None);
        assert!(!info.sampled);
        assert_eq!(
            info.gaps,
            Some(Wording {
                en: "the archaeological monuments are not in it, and the Hansestadt Lübeck keeps its own list",
                de: "Die Bodendenkmale fehlen darin, und die Hansestadt Lübeck führt ihre Denkmalliste selbst",
            })
        );
        assert_eq!(info.layers, ["denkmale"]);
        assert_eq!(info.sections, [PermitSection::Monuments]);
        assert!(info.offices.is_none());
    }

    /// A file of three monuments in EPSG:25832: two in Kiel, one far away.
    fn file() -> Vec<u8> {
        let at = |lat: f64, lon: f64| {
            let p = Crs::Utm32.forward(LatLon { lat, lon });
            [p.easting, p.northing]
        };
        let square = |[e, n]: [f64; 2]| {
            serde_json::json!({"type": "MultiPolygon", "coordinates": [[[
                [e, n], [e + 10.0, n], [e + 10.0, n + 10.0], [e, n + 10.0], [e, n]]]]})
        };
        serde_json::to_vec(&serde_json::json!({
            "type": "FeatureCollection",
            "crs": {"type": "name", "properties": {"name": "urn:ogc:def:crs:EPSG::25832"}},
            "features": [
                {"type": "Feature", "properties": {"LayerName": "Baudenkmal", "Strasse": "x"},
                 "geometry": square(at(54.3230, 10.1390))},
                {"type": "Feature", "properties": {"LayerName": "Gründenkmal"},
                 "geometry": square(at(54.3235, 10.1395))},
                {"type": "Feature", "properties": {"LayerName": "Baudenkmal"},
                 "geometry": square(at(54.78, 9.43))},
            ]
        }))
        .unwrap()
    }

    #[test]
    fn the_file_is_cut_to_the_map_and_kept_in_the_cache() {
        let fetcher = MapFetcher::default();
        fetcher.insert(URL, file());
        let cache_root = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(cache_root.path().to_path_buf());
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            geo_types::Coord { x: 10.13, y: 54.32 },
            geo_types::Coord { x: 10.15, y: 54.33 },
        );
        let source = source();
        let answers = source
            .fetch_cached(&fetcher, &bbox, &part.path().join("p.part"), &cache)
            .unwrap();
        assert_eq!(answers[0].len(), 2);
        assert!(answers[0][0]["properties"].get("Strasse").is_none());
        let kinds: Vec<FindingKind> = source
            .findings("denkmale", &answers[0])
            .into_iter()
            .map(|f| f.kind)
            .collect();
        assert_eq!(
            kinds,
            ["Baudenkmal", "Gründenkmal"].map(|kind| FindingKind::Monument {
                kind: kind.into(),
                link: None
            })
        );
        // Kept: a second run with the server down reads the copy.
        let down = MapFetcher::default();
        down.failing.borrow_mut().push(URL.to_string());
        let again = source
            .fetch_cached(&down, &bbox, &part.path().join("p.part"), &cache)
            .unwrap();
        assert_eq!(again[0].len(), 2);
    }

    #[test]
    fn the_file_is_read_without_the_cache_and_a_foreign_layer_has_no_findings() {
        let fetcher = MapFetcher::default();
        fetcher.insert(URL, file());
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            geo_types::Coord { x: 10.13, y: 54.32 },
            geo_types::Coord { x: 10.15, y: 54.33 },
        );
        let source = source();
        let answers = source
            .fetch(&fetcher, &bbox, &part.path().join("p.part"))
            .unwrap();
        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].len(), 2);
        assert_eq!(source.findings("denkmale", &answers[0]).len(), 2);
        assert!(source.findings("other", &answers[0]).is_empty());
    }

    #[test]
    fn a_file_that_is_no_geojson_is_an_error_with_the_address() {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            URL,
            br#"{"type": "FeatureCollection", "features": 3}"#.to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            geo_types::Coord { x: 10.13, y: 54.32 },
            geo_types::Coord { x: 10.15, y: 54.33 },
        );
        let error = source()
            .fetch(&fetcher, &bbox, &part.path().join("p.part"))
            .unwrap_err()
            .to_string();
        assert!(error.contains(URL), "{error}");
    }

    #[test]
    fn an_answer_that_is_no_geojson_is_not_kept() {
        assert!(looks_like_geojson(b"<html>maintenance</html>").is_err());
        assert!(looks_like_geojson(br#"{"type": "FeatureCollection", "features": []}"#).is_ok());
    }
}
