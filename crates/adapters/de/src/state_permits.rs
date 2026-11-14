//! The permit sources of the Saarland and Hesse (ADR 0130), described as tables for
//! [`wfs_permits`](crate::wfs_permits): GeoJSON WFS of the Geoportal Saarland (CC BY 4.0) and of
//! the HLNUG (CC BY 4.0). The credit lines are verbatim from the services' capabilities or
//! their catalogue entries.

use freemapper_adapter_api::permits::{ForestFunctionKind, PermitSection, PermitSource, Wording};

use crate::wfs_permits::{Reading, source};

const SAARLAND: Wording = Wording {
    en: "Geoportal Saarland",
    de: "Geoportal Saarland",
};

/// The credit of the Geoportal Saarland as its catalogue gives it; the year is that of the
/// fetch.
const SAARLAND_CREDIT: &str = "Quellenvermerk: © GDI-SL ({year}); Lizenz: Creative Commons Namensnennung 4.0 International (CC BY 4.0)";

/// The three services of the Saarland.
pub fn saarland() -> Vec<Box<dyn PermitSource>> {
    vec![
        source(
            "forst",
            SAARLAND,
            Wording {
                en: "forest districts of the SaarForst Landesbetrieb (state forest)",
                de: "Reviere des SaarForst Landesbetriebs (Staatswald)",
            },
            SAARLAND_CREDIT,
            &["reviere"],
            &[PermitSection::ForestUnits],
            "https://geoportal.saarland.de/arcgis/services/Internet/Forstwirtschaft/MapServer/WFSServer",
            "GEOJSON",
            &[(
                "reviere",
                "Forstwirtschaft:Reviergrenze_Staatswald",
                Reading::Unit {
                    administration: "SaarForst Landesbetrieb",
                    prefix: "Revier ",
                    office: "NAME",
                },
            )],
        ),
        source(
            "schutzgebiete",
            SAARLAND,
            Wording {
                en: "water protection areas, Naturwaldzellen and Urwald",
                de: "Wasserschutzgebiete, Naturwaldzellen und Urwald",
            },
            SAARLAND_CREDIT,
            &["wasserschutzgebiete", "naturwaldzellen", "urwald"],
            &[
                PermitSection::WaterProtection,
                PermitSection::ForestFunctions,
            ],
            "https://geoportal.saarland.de/arcgis/services/Internet/Schutzgebiete_INS/MapServer/WFSServer",
            "GEOJSON",
            &[
                (
                    "wasserschutzgebiete",
                    "Schutzgebiete_INS:Wasserschutzgebiet",
                    Reading::Water {
                        name: "NAME",
                        zone: "SCHUTZZONE",
                        citation: Some("RECHTSGRUNDLAGE"),
                    },
                ),
                (
                    "naturwaldzellen",
                    "Schutzgebiete_INS:Naturwaldzellen",
                    Reading::Function {
                        kind: ForestFunctionKind::Natural,
                        name: Some("NWZ"),
                    },
                ),
                (
                    "urwald",
                    "Schutzgebiete_INS:Urwald_SL",
                    Reading::Function {
                        kind: ForestFunctionKind::Natural,
                        name: Some("INFO"),
                    },
                ),
            ],
        ),
        source(
            "denkmale",
            SAARLAND,
            Wording {
                en: "listed buildings (Baudenkmale)",
                de: "Baudenkmale",
            },
            SAARLAND_CREDIT,
            &["baudenkmale"],
            &[PermitSection::Monuments],
            "https://geoportal.saarland.de/arcgis/services/Internet/Denkmalschutz/MapServer/WFSServer",
            "GEOJSON",
            &[(
                "baudenkmale",
                "Denkmalschutz:Baudenkmale",
                Reading::Monument {
                    kind: "Baudenkmal",
                    link: Some(("Baud_Land", "https://www.saarland.de/")),
                },
            )],
        ),
    ]
}

/// The credit of the HLNUG, verbatim from the capabilities (`Fees`) of its service.
const HLNUG_CREDIT: &str = "Lizenz: Creative Commons BY 4.0 \
    (https://creativecommons.org/licenses/by/4.0/deed.de). Quellenvermerk für Daten: \
    „Darstellung auf der Grundlage von Daten des Hessischen Landesamtes für Naturschutz, Umwelt \
    und Geologie (HLNUG)“.";

/// The one service of Hesse.
pub fn hessen() -> Vec<Box<dyn PermitSource>> {
    vec![source(
        "wasserschutzgebiete",
        Wording {
            en: "HLNUG",
            de: "HLNUG",
        },
        Wording {
            en: "drinking water and medicinal spring protection areas",
            de: "Trinkwasser- und Heilquellenschutzgebiete",
        },
        HLNUG_CREDIT,
        &["wasserschutzgebiete"],
        &[PermitSection::WaterProtection],
        "https://geodienste-umwelt.hessen.de/arcgis/services/inspire/bewirtschaftungsgebiete/MapServer/WFSServer",
        "GEOJSON",
        &[(
            "wasserschutzgebiete",
            "inspire_bewirtschaftungsgebiete:TWS_HQS_ALK",
            Reading::Water {
                name: "WSG_KURZNAME",
                zone: "ZONE",
                citation: Some("STAATSANZEIGER"),
            },
        )],
    )]
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::permits::FindingKind;
    use serde_json::Value;

    macro_rules! fixture {
        ($name:literal) => {
            include_str!(concat!("../../../../tests/fixtures/permits-states/", $name))
        };
    }

    fn features(collection: &str) -> Vec<Value> {
        let value: Value = serde_json::from_str(collection).unwrap();
        value["features"].as_array().unwrap().clone()
    }

    /// The findings of `layer` of the source `key` among `sources`, from a trimmed live answer.
    fn read(
        sources: Vec<Box<dyn PermitSource>>,
        key: &str,
        layer: &str,
        collection: &str,
    ) -> Vec<FindingKind> {
        let source = sources.into_iter().find(|s| s.info().key == key).unwrap();
        let found = source.findings(layer, &features(collection));
        for f in &found {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                // Germany, longitude first.
                assert!(
                    (5.8..15.1).contains(&c.x) && (47.2..55.1).contains(&c.y),
                    "{c:?}"
                );
            }
        }
        found.into_iter().map(|f| f.kind).collect()
    }

    #[test]
    fn saarland_reviere_name_saarforst() {
        assert_eq!(
            read(
                saarland(),
                "forst",
                "reviere",
                fixture!("saarland-reviere.geojson")
            ),
            ["Grossrosseln", "Rastpfuhl/Puettlingen"].map(|name| FindingKind::ForestUnit {
                administration: "SaarForst Landesbetrieb".into(),
                office: format!("Revier {name}"),
                unit: None,
            })
        );
    }

    #[test]
    fn saarland_water_zones_get_their_zone_word() {
        let found = read(
            saarland(),
            "schutzgebiete",
            "wasserschutzgebiete",
            fixture!("saarland-wsg.geojson"),
        );
        let zones: Vec<Option<&str>> = found
            .iter()
            .map(|k| match k {
                FindingKind::WaterProtection { zone, .. } => zone.as_deref(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(zones, [Some("Schutzzone III"), Some("Schutzzone II")]);
    }

    #[test]
    fn saarland_naturwald_and_urwald_are_natural_forest() {
        let cells = read(
            saarland(),
            "schutzgebiete",
            "naturwaldzellen",
            fixture!("saarland-naturwaldzellen.geojson"),
        );
        assert_eq!(
            cells,
            ["Frankenbacher Hof", "Gersweiler Weiher"].map(|name| FindingKind::ForestFunction {
                kind: ForestFunctionKind::Natural,
                name: Some(name.into()),
            })
        );
        // The Urwald has a blank INFO: no name.
        let urwald = read(
            saarland(),
            "schutzgebiete",
            "urwald",
            fixture!("saarland-urwald.geojson"),
        );
        assert!(urwald.iter().all(|k| *k
            == FindingKind::ForestFunction {
                kind: ForestFunctionKind::Natural,
                name: None
            }));
    }

    #[test]
    fn saarland_baudenkmale_are_points_with_the_list() {
        let found = read(
            saarland(),
            "denkmale",
            "baudenkmale",
            fixture!("saarland-baudenkmale.geojson"),
        );
        assert_eq!(found.len(), 2);
        let FindingKind::Monument { kind, link } = &found[0] else {
            panic!("{found:?}");
        };
        assert_eq!(kind, "Baudenkmal");
        assert!(
            link.as_deref()
                .unwrap()
                .starts_with("https://www.saarland.de/")
        );
    }

    #[test]
    fn hessen_water_zones_keep_their_zone() {
        let found = read(
            hessen(),
            "wasserschutzgebiete",
            "wasserschutzgebiete",
            fixture!("hessen-wsg.geojson"),
        );
        let zones: Vec<(Option<&str>, Option<&str>)> = found
            .iter()
            .map(|k| match k {
                FindingKind::WaterProtection { zone, citation, .. } => {
                    (zone.as_deref(), citation.as_deref())
                }
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            zones,
            [
                (Some("Schutzzone II"), None),
                (Some("Schutzzone IIIB"), Some("1986/27 S.1381"))
            ]
        );
    }

    #[test]
    fn the_credits_carry_their_licence() {
        for s in saarland() {
            assert!(s.info().credit.contains("CC BY 4.0") && s.info().credit.contains("{year}"));
        }
        assert!(hessen()[0].info().credit.contains("HLNUG"));
    }

    #[test]
    fn brandenburg_forest_functions() {
        let sources = crate::lfb_permits::sources();
        let one = |layer: &str, file: &str| {
            read(crate::lfb_permits::sources(), "waldfunktionen", layer, file)
        };
        assert_eq!(sources.len(), 2);
        assert_eq!(
            one("naturwald", fixture!("brandenburg-naturwald.geojson")),
            ["Fünfeichen", "Dubrow"].map(|n| FindingKind::ForestFunction {
                kind: ForestFunctionKind::Natural,
                name: Some(n.into()),
            })
        );
        assert_eq!(
            one(
                "totalreservat",
                fixture!("brandenburg-totalreservat.geojson")
            )
            .len(),
            1
        );
        assert_eq!(
            one(
                "nicht-betretbar",
                fixture!("brandenburg-nicht-betretbar.geojson")
            ),
            ["Sperrbereich Altbergbau", "Militärischer Sperrbereich"].map(|n| {
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Closed,
                    name: Some(n.into()),
                }
            })
        );
        for (layer, file) in [
            (
                "erholungswald",
                fixture!("brandenburg-erholungswald.geojson"),
            ),
            (
                "erholungswald-rechtlich",
                fixture!("brandenburg-erholungswald-rechtlich.geojson"),
            ),
        ] {
            assert!(one(layer, file).iter().all(|k| matches!(
                k,
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Recreation,
                    name: Some(_)
                }
            )));
        }
    }

    #[test]
    fn nrw_naturwaldzellen_and_wildnis_are_natural_forest() {
        for (layer, file, first) in [
            (
                "naturwaldzellen",
                fixture!("nrw-naturwaldzellen.geojson"),
                "Naturwaldzelle Sandkaul",
            ),
            (
                "wildnis",
                fixture!("nrw-wildnis.geojson"),
                "Rosenberg bei Bilstein",
            ),
        ] {
            let found = read(crate::wuh_permits::sources(), "linfos", layer, file);
            assert_eq!(
                found[0],
                FindingKind::ForestFunction {
                    kind: ForestFunctionKind::Natural,
                    name: Some(first.into()),
                }
            );
        }
    }

    #[test]
    fn berlin_water_zones() {
        let found = read(
            crate::berlin_permits::sources(),
            "wsg",
            "wsg",
            include_str!("../../../../tests/fixtures/permits-berlin/wsg.geojson"),
        );
        assert_eq!(
            found,
            ["Schutzzone I", "Schutzzone II"].map(|zone| FindingKind::WaterProtection {
                name: Some("Beelitzhof".into()),
                zone: Some(zone.into()),
                citation: Some("Wasserschutzgebietsverordnung Beelitzhof".into()),
            })
        );
    }

    /// The table sources, asked live through the hosts of their state's manifest with the
    /// fixed User-Agent of the program, each at a place where it has findings:
    /// `cargo test -p freemapper-adapters-de live_table_sources -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn live_table_sources() {
        use crate::fetch::HttpFetcher;
        use crate::states::{State, StateAdapter, register_backends};
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        use freemapper_adapter_api::policy::PolicyFetcher;
        use freemapper_adapter_api::registry::Registry;
        use geo_types::{Coord, Rect};
        assert!(!freemapper::fetch::USER_AGENT.contains('@'));
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        let http = HttpFetcher::new();
        let part = tempfile::tempdir().unwrap();
        let part = part.path().join("live.part");
        // State, source key, box (min lon, min lat, max lon, max lat).
        let probes = [
            (State::Berlin, "wsg", [13.15, 52.40, 13.30, 52.50]),
            (
                State::Brandenburg,
                "waldfunktionen",
                [13.0, 52.0, 14.5, 53.0],
            ),
            (State::NordrheinWestfalen, "linfos", [6.5, 50.5, 8.0, 51.5]),
            (State::Saarland, "forst", [6.9, 49.2, 7.1, 49.3]),
            (State::Saarland, "schutzgebiete", [6.8, 49.2, 7.2, 49.5]),
            (State::Saarland, "denkmale", [6.99, 49.23, 7.00, 49.24]),
            (State::Hessen, "wasserschutzgebiete", [8.5, 50.0, 8.8, 50.2]),
        ];
        for (state, key, [w, s, e, n]) in probes {
            let adapter = StateAdapter::new(state, &backends);
            let fetcher = PolicyFetcher::new(&http, adapter.policy().unwrap());
            let source = adapter
                .permit_sources()
                .iter()
                .find(|s| s.info().key == key)
                .unwrap();
            let bbox = Rect::new(Coord { x: w, y: s }, Coord { x: e, y: n });
            let answers = source.fetch(&fetcher, &bbox, &part).unwrap();
            let found: usize = source
                .info()
                .layers
                .iter()
                .zip(&answers)
                .map(|(layer, features)| source.findings(layer, features).len())
                .sum();
            assert!(found > 0, "{state:?} {key}: no finding");
        }
    }
}
