// SPDX-License-Identifier: AGPL-3.0-or-later

//! The fixtures of North Rhine-Westphalia's permit sources (ADR 0124), read through the tables its
//! manifest declares (issue #273). The Forstbetriebsbezirke and the Staatswald of the Landesbetrieb
//! Wald und Holz NRW, the water protection areas and LINFOS had their tables in this file until
//! they moved into the manifest; these tests, which ran against the code, now run against the
//! declared ones. Each shapefile is one zip for the whole state (about 11 and 6 MB), cut to the
//! map box (EPSG:25832); every row of the Forstbetriebsbezirke also names the forester with first
//! and last name, e-mail and phone numbers, and a feature keeps only the fields its reading
//! names, so none of that is stored or reaches the notes.

mod tests {
    use std::path::Path;

    use crate::crs::{Crs, ProjectedPoint};
    use crate::fetch::{FetchError, test_support::MapFetcher};
    use crate::wfs_permits::declared_source;
    use freemapper_adapter_api::permits::{FindingKind, PermitSource};
    use geo_types::Rect;
    use serde_json::Value;

    const BASE: &str = "https://www.opengeodata.nrw.de/produkte/umwelt_klima/wald_forst/wald/";

    /// The sources the manifest of North Rhine-Westphalia declares, in its order: the services
    /// first, then the shapefiles.
    fn sources() -> Vec<Box<dyn PermitSource>> {
        let text = include_str!("../manifests/nordrheinwestfalen/manifest.toml");
        freemapper_manifest::parse(text, Path::new("nordrheinwestfalen"))
            .unwrap()
            .permits
            .iter()
            .map(declared_source)
            .collect()
    }

    fn source(key: &str) -> Box<dyn PermitSource> {
        sources().into_iter().find(|s| s.info().key == key).unwrap()
    }
    use freemapper_sources::shapefile::test_support::{Entry, shape_files};
    use geo_types::Coord;
    use std::io::Write as _;

    /// A square of 200 m in EPSG:25832 at the south-west corner `(e, n)`.
    fn square(e: f64, n: f64) -> Vec<[f64; 2]> {
        vec![
            [e, n],
            [e, n + 200.0],
            [e + 200.0, n + 200.0],
            [e + 200.0, n],
            [e, n],
        ]
    }

    /// The box of a made-up map area near Bonn (about 365 km east, 5617 km north).
    fn kottenforst() -> Rect<f64> {
        let sw = Crs::Utm32.inverse(ProjectedPoint {
            easting: 364_000.0,
            northing: 5_616_000.0,
        });
        let ne = Crs::Utm32.inverse(ProjectedPoint {
            easting: 366_000.0,
            northing: 5_618_000.0,
        });
        let (sw, ne) = (sw.unwrap(), ne.unwrap());
        Rect::new(
            Coord {
                x: sw.lon,
                y: sw.lat,
            },
            Coord {
                x: ne.lon,
                y: ne.lat,
            },
        )
    }

    /// A zip of the files of `records`, named as the state names them.
    fn zip_of(stem: &str, records: &[Entry<'_>]) -> Vec<u8> {
        let (shp, dbf) = shape_files(records);
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        zip.add_directory(format!("{stem}/"), options).unwrap();
        for (extension, bytes) in [("shp", shp), ("dbf", dbf), ("cpg", b"UTF-8".to_vec())] {
            zip.start_file(format!("{stem}/{stem}.{extension}"), options)
                .unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    fn fetch(source: &dyn PermitSource, zip: Vec<u8>) -> Vec<Value> {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &format!(
                "{BASE}{}",
                match source.info().key {
                    "staatswald" => "landeseigener_forstbetrieb_staatswald_EPSG25832_Shape.zip",
                    _ => "forstbetriebsbezirke_nrw_EPSG25832_Shape.zip",
                }
            ),
            zip,
        );
        let part = tempfile::tempdir().unwrap();
        let mut layers = source
            .fetch(&fetcher, &kottenforst(), &part.path().join("x.part"))
            .unwrap();
        assert_eq!(layers.len(), 1);
        layers.remove(0)
    }

    /// The fields of a district as the state writes them, with made-up values for the person.
    fn district_fields<'a>(rfa: &'a str, fbb: &'a str) -> Vec<(&'a str, &'a str)> {
        vec![
            ("rfa_nr", "04"),
            ("fbb_nr", "02"),
            ("rfa", rfa),
            ("fbb", fbb),
            ("vorname", "Erika"),
            ("name", "Musterfrau"),
            ("strasse", "Beispielweg"),
            ("nummer", "1"),
            ("ort", "Musterstadt"),
            ("email", "erika.musterfrau@example.org"),
            ("plz", "12345"),
            ("telefon", "+49 000 0000000"),
            ("mobil", "+49 000 0000001"),
            ("aktualitae", "2026-10-14"),
        ]
    }

    /// A district keeps its office and its name and nothing about the forester; a district far
    /// from the map is not kept at all.
    #[test]
    fn districts_keep_no_personal_field() {
        let near = district_fields("Rhein-Sieg-Erft", "Kottenforst");
        let far = district_fields("Münsterland", "Weit weg");
        let zip = zip_of(
            "forstbetriebsbezirke_nrw",
            &[
                (&near, &[square(365_000.0, 5_617_000.0)]),
                (&far, &[square(400_000.0, 5_750_000.0)]),
            ],
        );
        let source = source("forstbetriebsbezirke");
        let features = fetch(source.as_ref(), zip);
        assert_eq!(features.len(), 1, "{features:?}");
        let text = features[0].to_string();
        for personal in [
            "Erika",
            "Musterfrau",
            "example.org",
            "+49",
            "Beispielweg",
            "12345",
        ] {
            assert!(!text.contains(personal), "{personal}: {text}");
        }
        let keys: Vec<&String> = features[0]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .collect();
        assert_eq!(keys, ["fbb", "rfa"]);
        let found = source.findings("forstbetriebsbezirke", &features);
        assert_eq!(
            found.iter().map(|f| &f.kind).collect::<Vec<_>>(),
            [&FindingKind::ForestUnit {
                administration: "Landesbetrieb Wald und Holz NRW".into(),
                office: "Regionalforstamt Rhein-Sieg-Erft".into(),
                unit: Some("Forstbetriebsbezirk Kottenforst".into()),
            }]
        );
        let bbox = kottenforst();
        for c in geo::CoordsIter::coords_iter(&found[0].geometry) {
            assert!(bbox.min().x < c.x && c.x < bbox.max().x, "{c:?}");
            assert!(bbox.min().y < c.y && c.y < bbox.max().y, "{c:?}");
        }
    }

    #[test]
    fn state_forest_names_its_owner_and_office() {
        let parcel = [
            ("gemeinde", "Bonn, Stadt"),
            ("gemarkung_", "1234"),
            ("forstamt", "Rhein-Sieg-Erft"),
            ("label", "1/2/0"),
            ("gemarkung1", "Röttgen"),
        ];
        let zip = zip_of(
            "landeseigener_forstbetrieb_staatswald",
            &[(&parcel, &[square(365_000.0, 5_617_000.0)])],
        );
        let source = source("staatswald");
        let features = fetch(source.as_ref(), zip);
        let keys: Vec<&String> = features[0]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .collect();
        assert_eq!(keys, ["forstamt"], "the parcel label is not kept");
        let found = source.findings("staatswald", &features);
        assert_eq!(
            found.iter().map(|f| &f.kind).collect::<Vec<_>>(),
            [&FindingKind::StateForest {
                owner: "Bundesland Nordrhein-Westfalen".into(),
                office: Some("Regionalforstamt Rhein-Sieg-Erft".into()),
            }]
        );
    }

    /// An office is a Regionalforstamt, unless its name says it is a Forstamt, and "Arnsberger
    /// Wald" is the Lehr- und Versuchsforstamt.
    #[test]
    fn offices_are_named_as_wald_und_holz_names_them() {
        let feature = |name: &str| {
            serde_json::json!({"type": "Feature", "properties": {"forstamt": name},
                "geometry": {"type": "Polygon", "coordinates": [[
                    [7.0, 50.7], [7.0, 50.71], [7.01, 50.71], [7.0, 50.7]]]}})
        };
        let office = |name: &str| match source("staatswald")
            .findings("staatswald", &[feature(name)])
            .remove(0)
            .kind
        {
            FindingKind::StateForest { office, .. } => office.unwrap(),
            other => panic!("{other:?}"),
        };
        assert_eq!(office("Hochstift"), "Regionalforstamt Hochstift");
        assert_eq!(
            office("Nationalparkforstamt Eifel"),
            "Nationalparkforstamt Eifel"
        );
        assert_eq!(
            office("NATIONALPARKFORSTAMT EIFEL"),
            "NATIONALPARKFORSTAMT EIFEL"
        );
        assert_eq!(
            office("Arnsberger Wald"),
            "Lehr- und Versuchsforstamt Arnsberger Wald"
        );
    }

    /// A zip without the files, or a broken one, is an error with the URL, not empty notes.
    #[test]
    fn a_broken_download_is_an_error() {
        let fetcher = MapFetcher::default();
        let url = format!("{BASE}forstbetriebsbezirke_nrw_EPSG25832_Shape.zip");
        fetcher.insert(&url, b"<html>moved</html>".to_vec());
        let part = tempfile::tempdir().unwrap();
        let err = source("forstbetriebsbezirke")
            .fetch(&fetcher, &kottenforst(), &part.path().join("x.part"))
            .unwrap_err();
        assert!(
            matches!(&err, FetchError::Http { url: u, .. } if *u == url),
            "{err:?}"
        );
    }

    /// The forest districts and the Staatswald of the Kottenforst, downloaded live through the
    /// hosts of the manifest of North Rhine-Westphalia with the fixed User-Agent of the program
    /// (about 17 MB): `cargo test -p freemapper-adapters-de live_kottenforst -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn live_kottenforst() {
        use crate::fetch::HttpFetcher;
        use crate::states::{State, StateAdapter, register_backends};
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        use freemapper_adapter_api::policy::PolicyFetcher;
        use freemapper_adapter_api::registry::Registry;
        assert!(!freemapper::fetch::USER_AGENT.contains('@'));
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        let nrw = StateAdapter::new(State::NordrheinWestfalen, &backends);
        let http = HttpFetcher::new();
        let fetcher = PolicyFetcher::new(&http, nrw.policy().unwrap());
        let part = tempfile::tempdir().unwrap();
        let mut kinds = Vec::new();
        // Kottenforst, south-west of Bonn.
        let bbox = Rect::new(Coord { x: 7.04, y: 50.66 }, Coord { x: 7.08, y: 50.68 });
        for source in nrw.permit_sources() {
            let layers = source
                .fetch(&fetcher, &bbox, &part.path().join("live.part"))
                .unwrap();
            for feature in &layers[0] {
                let keys: Vec<&String> =
                    feature["properties"].as_object().unwrap().keys().collect();
                assert!(
                    keys.iter()
                        .all(|k| ["rfa", "fbb", "forstamt"].contains(&k.as_str())),
                    "{keys:?}"
                );
            }
            kinds.extend(source.findings(source.info().layers[0], &layers[0]));
        }
        let offices: std::collections::BTreeSet<String> = kinds
            .iter()
            .filter_map(|f| match &f.kind {
                FindingKind::ForestUnit { office, .. } => Some(office.clone()),
                _ => None,
            })
            .collect();
        // The Kottenforst lies in the Regionalforstamt Rhein-Sieg-Erft, with Staatswald.
        assert!(
            offices.contains("Regionalforstamt Rhein-Sieg-Erft"),
            "{offices:?}"
        );
        assert!(
            kinds
                .iter()
                .any(|f| matches!(f.kind, FindingKind::StateForest { .. })),
            "no Staatswald"
        );
    }
}
