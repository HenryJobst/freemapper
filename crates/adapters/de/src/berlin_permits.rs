// SPDX-License-Identifier: AGPL-3.0-or-later

//! The geometry of the Berlin sources of the permit notes (ADR 0120), and their fixture tests.
//! The seven services of the Geoportal Berlin (dl-de/zero-2-0) had their table in this file
//! until they moved into the manifest of Berlin (issue #273): GeoServer WFS 2.0.0 that answer
//! GeoJSON (`application/json`) and read a box in WGS 84 latitude first when it ends with its
//! CRS, which is how the generic reader asks. The answers come as longitude, latitude; the
//! parser still accepts EPSG:25833 (the default CRS of the services) and latitude first, so a
//! changed default does not turn the findings into nonsense. A blank value of the services is a
//! single space.

use geo_types::{Coord, Geometry};
use serde_json::Value;

use crate::crs::{Crs, ProjectedPoint};

/// The geometry of `feature` in WGS 84 with x = longitude: a MultiPolygon for an area, a
/// MultiPoint for an object.
pub(crate) fn geometry(feature: &Value) -> Option<Geometry<f64>> {
    let geometry =
        serde_json::from_value::<geojson::Geometry>(feature.get("geometry")?.clone()).ok()?;
    let geometry = match Geometry::<f64>::try_from(geometry).ok()? {
        Geometry::Polygon(p) => Geometry::MultiPolygon(geo_types::MultiPolygon(vec![p])),
        Geometry::Point(p) => Geometry::MultiPoint(geo_types::MultiPoint(vec![p])),
        g @ (Geometry::MultiPolygon(_) | Geometry::MultiPoint(_)) => g,
        _ => return None,
    };
    let geometry = lon_lat(geometry)?;
    let usable = match &geometry {
        Geometry::MultiPolygon(mp) => mp.0.iter().any(|p| p.exterior().0.len() >= 4),
        Geometry::MultiPoint(mp) => !mp.0.is_empty(),
        _ => false,
    };
    usable.then_some(geometry)
}

/// `geometry` as longitude, latitude. Coordinates in metres are EPSG:25833; degrees with the
/// first larger than the second are latitude first (in Berlin the latitude is about 52.5, the
/// longitude about 13.4). `None` when a position cannot be converted.
fn lon_lat(geometry: Geometry<f64>) -> Option<Geometry<f64>> {
    let projected = |c: &Coord<f64>| c.x.abs() > 360.0 || c.y.abs() > 360.0;
    let first = geo::CoordsIter::coords_iter(&geometry).next();
    let Some(first) = first else {
        return Some(geometry);
    };
    let convert: fn(Coord<f64>) -> Option<Coord<f64>> = if projected(&first) {
        |c| {
            let p = Crs::Utm33.inverse(ProjectedPoint {
                easting: c.x,
                northing: c.y,
            })?;
            Some(Coord { x: p.lon, y: p.lat })
        }
    } else if first.x > first.y {
        |c| Some(Coord { x: c.y, y: c.x })
    } else {
        return Some(geometry);
    };
    if geo::CoordsIter::coords_iter(&geometry).any(|c| convert(c).is_none()) {
        return None;
    }
    let mut geometry = geometry;
    geo::MapCoordsInPlace::map_coords_in_place(&mut geometry, |c| convert(c).unwrap_or(c));
    Some(geometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::fetch::test_support::MapFetcher;
    use crate::wfs_permits::declared_source;
    use freemapper_adapter_api::permits::{
        FindingKind, PermitFinding, PermitSource, ProtectedObjectKind, Wording,
    };
    use geo_types::Rect;

    const LICENCE: &str = "Für die Nutzung der Daten ist die Datenlizenz Deutschland - Zero - \
        Version 2.0 anzuwenden. Die Lizenz ist über https://www.govdata.de/dl-de/zero-2-0 abrufbar.";

    const LABEL: Wording = Wording {
        en: "Geoportal Berlin",
        de: "Geoportal Berlin",
    };

    /// What the protected objects are worth legally, verbatim from the abstract of the service
    /// `schutzgebiete`.
    const ANALOGUE_MAPS: &str = "Die amtlichen Karten zur jeweiligen Schutzverordnung können \
        bei der jeweils örtlich zuständigen unteren Behörde für Naturschutz und Landschaftspflege im \
        Bezirksamt oder bei der obersten Behörde für Naturschutz und Landschaftspflege \
        (Senatsverwaltung für Mobilität, Verkehr, Klimaschutz und Umwelt Abt. III) eingesehen \
        werden. Nur diese amtlichen analogen Karten sind rechtsverbindlich, sofern die Karten nicht \
        im Gesetz- und Verordnungsblatt für Berlin mitveröffentlicht wurden.";

    /// The sources the manifest of Berlin declares, in its order.
    fn sources() -> Vec<Box<dyn PermitSource>> {
        let text = include_str!("../manifests/berlin/manifest.toml");
        freemapper_manifest::parse(text, Path::new("berlin"))
            .unwrap()
            .permits
            .iter()
            .map(declared_source)
            .collect()
    }

    macro_rules! fixture {
        ($name:literal) => {
            include_str!(concat!("../../../../tests/fixtures/permits-berlin/", $name))
        };
    }

    fn features(collection: &str) -> Vec<Value> {
        let value: Value = serde_json::from_str(collection).unwrap();
        value["features"].as_array().unwrap().clone()
    }

    fn source(key: &str) -> Box<dyn PermitSource> {
        sources().into_iter().find(|s| s.info().key == key).unwrap()
    }

    fn read(key: &str, layer: &str, collection: &str) -> Vec<PermitFinding> {
        source(key).findings(layer, &features(collection))
    }

    /// Every position of the findings lies around the Viktoriapark, as longitude, latitude.
    fn near_viktoriapark(findings: &[PermitFinding]) {
        for f in findings {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!((13.36..13.40).contains(&c.x), "{c:?}");
                assert!((52.48..52.50).contains(&c.y), "{c:?}");
            }
        }
    }

    /// A Distrikt of the Berliner Forsten names its Forstamt and Revier (ADR 0124), from a
    /// trimmed answer of the service in the Grunewald.
    #[test]
    fn forest_distrikte_name_their_forstamt_and_revier() {
        let found = read(
            "forstverwalt",
            "forstverwalt",
            fixture!("forstverwalt.geojson"),
        );
        let kinds: Vec<&FindingKind> = found.iter().map(|f| &f.kind).collect();
        assert_eq!(kinds.len(), 3, "{kinds:?}");
        for (kind, revier) in kinds.iter().zip(["Saubucht", "Dachsberg", "Eichkamp"]) {
            assert_eq!(
                **kind,
                FindingKind::ForestUnit {
                    administration: "Berliner Forsten".into(),
                    office: "Forstamt Grunewald".into(),
                    unit: Some(format!("Revier {revier}")),
                }
            );
        }
        for f in &found {
            assert!(matches!(f.geometry, Geometry::MultiPolygon(_)));
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!(
                    (13.19..13.28).contains(&c.x) && (52.43..52.51).contains(&c.y),
                    "{c:?}"
                );
            }
        }
        // Without a Forstamt the Distrikt says nothing the notes could use.
        let mut collection: Value = serde_json::from_str(fixture!("forstverwalt.geojson")).unwrap();
        collection["features"][0]["properties"]["foa_name"] = Value::String(" ".into());
        let features = collection["features"].as_array().unwrap().clone();
        assert_eq!(
            source("forstverwalt")
                .findings("forstverwalt", &features)
                .len(),
            2
        );
    }

    #[test]
    fn seven_sources_with_plain_keys() {
        let all = sources();
        let keys: Vec<&str> = all.iter().map(|s| s.info().key).collect();
        assert_eq!(
            keys,
            [
                "bezirke",
                "gruenanlagen",
                "liegenschaftsplan",
                "denkmale",
                "schutzgebiete",
                "forstverwalt",
                "wsg",
                "biotope"
            ]
        );
        for s in &all {
            let info = s.info();
            assert!(freemapper_adapter_api::permits::plain_key(info.key));
            assert!(
                info.layers
                    .iter()
                    .all(|l| freemapper_adapter_api::permits::plain_key(l))
            );
            assert_eq!(info.credit, LICENCE);
            assert_eq!(info.label, LABEL);
        }
        // Only the protected objects carry the note on the analogue maps.
        let notes: Vec<&str> = all.iter().filter_map(|s| s.info().note).collect();
        assert_eq!(notes, [ANALOGUE_MAPS]);
        assert!(
            ANALOGUE_MAPS.contains("Nur diese amtlichen analogen Karten sind rechtsverbindlich")
        );
    }

    #[test]
    fn the_queries_ask_the_geoportal_in_wgs84() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://",
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            Coord {
                x: 13.375,
                y: 52.486,
            },
            Coord {
                x: 13.389,
                y: 52.493,
            },
        );
        let answer = source("schutzgebiete")
            .fetch(&fetcher, &bbox, &part.path().join("x.part"))
            .unwrap();
        assert_eq!(answer.len(), 4);
        let log = fetcher.log.borrow();
        assert_eq!(log.len(), 4);
        for (url, type_name) in log.iter().zip(["a_nd", "e_fnd", "g_glb", "f_fspr"]) {
            assert!(
                url.starts_with("https://gdi.berlin.de/services/wfs/schutzgebiete?"),
                "{url}"
            );
            assert!(url.contains(&format!("typeNames=schutzgebiete:{type_name}&")));
            assert!(url.contains("outputFormat=application/json"), "{url}");
            assert!(
                url.ends_with("bbox=52.486,13.375,52.493,13.389,urn:ogc:def:crs:EPSG::4326"),
                "{url}"
            );
        }
    }

    #[test]
    fn a_failing_layer_stops_the_service() {
        let fetcher = MapFetcher::default();
        fetcher
            .refusing
            .borrow_mut()
            .push("https://gdi.berlin.de/".to_string());
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(Coord { x: 13.3, y: 52.4 }, Coord { x: 13.4, y: 52.5 });
        let source = source("gruenanlagen");
        assert!(
            source
                .fetch(&fetcher, &bbox, &part.path().join("x.part"))
                .is_err()
        );
        assert_eq!(fetcher.calls.get(), 1, "the playgrounds are not asked");
    }

    #[test]
    fn bezirke_are_named_by_namgem() {
        let found = read(
            "bezirke",
            "bezirksgrenzen",
            fixture!("bezirksgrenzen.geojson"),
        );
        let names: Vec<&FindingKind> = found.iter().map(|f| &f.kind).collect();
        assert_eq!(
            names,
            [
                &FindingKind::District {
                    name: "Friedrichshain-Kreuzberg".into()
                },
                &FindingKind::District {
                    name: "Tempelhof-Schöneberg".into()
                },
            ]
        );
        assert!(
            found
                .iter()
                .all(|f| matches!(f.geometry, Geometry::MultiPolygon(_)))
        );
        near_viktoriapark(&found);
    }

    #[test]
    fn green_spaces_take_the_park_name_and_drop_blanks() {
        let found = read(
            "gruenanlagen",
            "gruenanlagen",
            fixture!("gruenanlagen.geojson"),
        );
        assert_eq!(found.len(), 7);
        let names: Vec<String> = found
            .iter()
            .map(|f| match &f.kind {
                FindingKind::GreenSpace {
                    name,
                    district,
                    dedication,
                } => format!(
                    "{name} | {} | {}",
                    district.as_deref().unwrap_or("-"),
                    dedication.as_deref().unwrap_or("-")
                ),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            names[..2],
            [
                "Dora-Duncker-Park | Tempelhof-Schöneberg | gewidmet",
                "Viktoriapark | Friedrichshain-Kreuzberg | gewidmet",
            ]
        );
        assert!(names.contains(&"Mehringdamm 90 | Friedrichshain-Kreuzberg | gewidmet".into()));
        assert_eq!(
            names
                .iter()
                .filter(|n| n.starts_with("Viktoriapark |"))
                .count(),
            5,
            "{names:?}"
        );
        near_viktoriapark(&found);

        // A blank of a single space is no value.
        let blank = serde_json::json!({
            "type": "Feature",
            "geometry": {"type": "Polygon", "coordinates": [[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]},
            "properties": {"namenr": " ", "bezirkname": " ", "widmung": " "}
        });
        let found = source("gruenanlagen").findings("gruenanlagen", &[blank]);
        assert_eq!(
            found[0].kind,
            FindingKind::GreenSpace {
                name: String::new(),
                district: None,
                dedication: None
            }
        );
    }

    /// A park is the whole name before the first comma: parks whose names start with the same
    /// words stay apart, the parts of one park come together (review finding 5).
    #[test]
    fn parks_with_the_same_first_words_stay_apart() {
        let space = |name: &str| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "Polygon", "coordinates": [[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]},
                "properties": {"namenr": name, "bezirkname": "Mitte", "widmung": "gewidmet"}
            })
        };
        let spaces: Vec<Value> = [
            "Park am Gleisdreieck, Ostpark",
            "Park am Gleisdreieck, Westpark",
            "Park am Weinberg, Spielwiese",
            "Volkspark Friedrichshain",
            "Volkspark Humboldthain, Nord",
        ]
        .map(space)
        .into();
        let names: Vec<String> = source("gruenanlagen")
            .findings("gruenanlagen", &spaces)
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::GreenSpace { name, .. } => name,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            names,
            [
                "Park am Gleisdreieck",
                "Park am Gleisdreieck",
                "Park am Weinberg",
                "Volkspark Friedrichshain",
                "Volkspark Humboldthain",
            ]
        );
    }

    #[test]
    fn playgrounds_are_findings_without_a_name() {
        let found = read(
            "gruenanlagen",
            "spielplaetze",
            fixture!("spielplaetze.geojson"),
        );
        assert_eq!(found.len(), 4);
        assert!(found.iter().all(|f| f.kind == FindingKind::Playground));
    }

    #[test]
    fn the_liegenschaftsplan_names_two_owners() {
        let found = read(
            "liegenschaftsplan",
            "liegenschaftsplan",
            fixture!("liegenschaftsplan.geojson"),
        );
        let owner = |name: &str| {
            found
                .iter()
                .filter(|f| {
                    f.kind
                        == FindingKind::OwnerShare {
                            owner: Some(name.into()),
                        }
                })
                .count()
        };
        assert_eq!(owner("Land Berlin"), 58);
        assert_eq!(owner("Bundesrepublik Deutschland"), 2);
        assert_eq!(found.len(), 60);
    }

    /// Only "Land Berlin" and "Bundesrepublik Deutschland" are named as owners; any other
    /// value counts as another public owner, without its text (review finding 8).
    #[test]
    fn only_the_land_and_the_bund_are_named_as_owners() {
        let parcel = |owner: &str| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "Polygon", "coordinates": [[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]},
                "properties": {"sch": owner}
            })
        };
        let parcels: Vec<Value> = [
            " Land Berlin ",
            "Bundesrepublik Deutschland",
            "Muster Grundbesitz GmbH",
            "Land Berlin, Beispielverwaltung",
        ]
        .map(parcel)
        .into();
        let found = source("liegenschaftsplan").findings("liegenschaftsplan", &parcels);
        assert_eq!(found.len(), 4);
        let debug = format!("{found:?}");
        assert!(
            !debug.contains("Muster") && !debug.contains("Beispiel"),
            "{debug}"
        );
        let owners: Vec<Option<&str>> = found
            .iter()
            .map(|f| match &f.kind {
                FindingKind::OwnerShare { owner } => owner.as_deref(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            owners,
            [
                Some("Land Berlin"),
                Some("Bundesrepublik Deutschland"),
                None,
                None
            ]
        );
    }

    #[test]
    fn monuments_carry_their_kind_and_link() {
        let found = read("denkmale", "denkmale", fixture!("denkmale.geojson"));
        assert_eq!(found.len(), 42);
        let garden: Vec<&FindingKind> = found
            .iter()
            .map(|f| &f.kind)
            .filter(|k| matches!(k, FindingKind::Monument { kind, .. } if kind == "Gartendenkmal"))
            .collect();
        assert_eq!(
            garden,
            [&FindingKind::Monument {
                kind: "Gartendenkmal".into(),
                link: Some(
                    "https://denkmaldatenbank.berlin.de/daobj.php?obj_dok_nr=09046157".into()
                ),
            }]
        );
        // A link into another site is not taken.
        let elsewhere = serde_json::json!({
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [13.38, 52.49]},
            "properties": {"typ": "Baudenkmal", "link": "https://example.org/x"}
        });
        let found = source("denkmale").findings("denkmale", &[elsewhere]);
        assert_eq!(
            found[0].kind,
            FindingKind::Monument {
                kind: "Baudenkmal".into(),
                link: None
            }
        );
    }

    #[test]
    fn naturdenkmal_points_name_the_tree_and_cite_the_ordinance() {
        let found = read("schutzgebiete", "a-nd", fixture!("naturdenkmale.geojson"));
        assert_eq!(found.len(), 8);
        assert!(
            found
                .iter()
                .all(|f| matches!(f.geometry, Geometry::MultiPoint(_)))
        );
        assert_eq!(
            found[0].kind,
            FindingKind::ProtectedObject {
                kind: ProtectedObjectKind::NaturalMonument,
                name: Some("Ahornblättrige Platane".into()),
                citation: Some(
                    "Verordnung zum Schutz von Naturdenkmalen in Berlin vom 20.05.2021 (GVBl.S. 536)"
                        .into()
                ),
            }
        );
        near_viktoriapark(&found);
    }

    #[test]
    fn the_area_layers_read_gebietsname_and_zitat() {
        let area = |name: &str| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "MultiPolygon", "coordinates": [[[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]]},
                "properties": {"gebietsname": name, "schutzstatus": "festgesetzt", "zitat": "VO vom 1.1.2000"}
            })
        };
        for (layer, kind) in [
            ("e-fnd", ProtectedObjectKind::NaturalMonument),
            ("g-glb", ProtectedObjectKind::ProtectedLandscapePart),
            ("f-fspr", ProtectedObjectKind::SpecialRules),
        ] {
            let found = source("schutzgebiete").findings(layer, &[area("Pfuhl")]);
            assert_eq!(
                found[0].kind,
                FindingKind::ProtectedObject {
                    kind,
                    name: Some("Pfuhl".into()),
                    citation: Some("VO vom 1.1.2000".into())
                },
                "{layer}"
            );
        }
        assert!(
            source("schutzgebiete")
                .findings("h-nsg", &[area("NSG")])
                .is_empty()
        );
    }

    /// The services' own CRS (EPSG:25833) and latitude first both come out as longitude,
    /// latitude.
    #[test]
    fn coordinates_are_normalised_to_lon_lat() {
        let lon_lat =
            features(fixture!("naturdenkmale.geojson"))[0]["geometry"]["coordinates"][0].clone();
        let (lon, lat) = (lon_lat[0].as_f64().unwrap(), lon_lat[1].as_f64().unwrap());
        let tree = |coordinates: Value| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "MultiPoint", "coordinates": [coordinates]},
                "properties": {"baum_deutsch": "Platane"}
            })
        };
        // The point as the service answers in its default CRS.
        let projected = tree(serde_json::json!([390195.19597097, 5816323.24395383]));
        let swapped = tree(serde_json::json!([lat, lon]));
        for feature in [projected, swapped] {
            let found = source("schutzgebiete").findings("a-nd", &[feature]);
            let Geometry::MultiPoint(points) = &found[0].geometry else {
                panic!("{:?}", found[0].geometry);
            };
            assert!((points.0[0].x() - lon).abs() < 1e-6, "{points:?}");
            assert!((points.0[0].y() - lat).abs() < 1e-6, "{points:?}");
        }
    }

    #[test]
    fn a_district_needs_an_area() {
        let point = serde_json::json!({
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [13.38, 52.49]},
            "properties": {"namgem": "Mitte"}
        });
        assert!(
            source("bezirke")
                .findings("bezirksgrenzen", &[point])
                .is_empty()
        );
    }

    /// The services for the Viktoriapark, asked live through the hosts of Berlin's manifest
    /// with the fixed User-Agent of the program:
    /// `cargo test -p freemapper-adapters-de live_viktoriapark_permit_sources -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn live_viktoriapark_permit_sources() {
        use crate::fetch::HttpFetcher;
        use crate::states::{State, StateAdapter, register_backends};
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        use freemapper_adapter_api::policy::PolicyFetcher;
        use freemapper_adapter_api::registry::Registry;
        assert!(!freemapper::fetch::USER_AGENT.contains('@'));
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        let berlin = StateAdapter::new(State::Berlin, &backends);
        let http = HttpFetcher::new();
        let fetcher = PolicyFetcher::new(&http, berlin.policy().unwrap());
        // The probe box 389613,5816190,390529,5816909 (EPSG:25833) in WGS 84.
        let corner = |e, n| {
            let p = Crs::Utm33
                .inverse(ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap();
            Coord { x: p.lon, y: p.lat }
        };
        let (a, b) = (corner(389613.0, 5816190.0), corner(390529.0, 5816909.0));
        let bbox = Rect::new(a, b);
        let part = tempfile::tempdir().unwrap();
        let part = part.path().join("live.part");
        let mut found = std::collections::BTreeMap::new();
        for source in berlin.permit_sources() {
            let answers = source.fetch(&fetcher, &bbox, &part).unwrap();
            for (layer, features) in source.info().layers.iter().zip(&answers) {
                found.insert(*layer, source.findings(layer, features));
            }
        }
        let districts: std::collections::BTreeSet<String> = found["bezirksgrenzen"]
            .iter()
            .filter_map(|f| match &f.kind {
                FindingKind::District { name } => Some(name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            districts.into_iter().collect::<Vec<_>>(),
            ["Friedrichshain-Kreuzberg", "Tempelhof-Schöneberg"]
        );
        let monuments = &found["denkmale"];
        let garden = monuments.iter().any(
            |f| matches!(&f.kind, FindingKind::Monument { kind, .. } if kind == "Gartendenkmal"),
        );
        assert!(garden || monuments.len() >= 30, "{}", monuments.len());
        let owners = |name: &str| {
            found["liegenschaftsplan"]
                .iter()
                .filter(|f| matches!(&f.kind, FindingKind::OwnerShare { owner } if owner.as_deref() == Some(name)))
                .count()
        };
        // The probe of 2026-11-13 found 58 and 2 parcels in the EPSG:25833 box; the box in
        // WGS 84 around its corners is a little larger and held 60 and 2 on the same day.
        assert!(owners("Land Berlin") >= 58, "{}", owners("Land Berlin"));
        assert_eq!(owners("Bundesrepublik Deutschland"), 2);
        // Whole Bezirke and parcels come back, so only the city is checked.
        for f in found.values().flatten() {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!(
                    (13.0..13.8).contains(&c.x) && (52.3..52.7).contains(&c.y),
                    "{c:?}"
                );
            }
        }
    }
}
