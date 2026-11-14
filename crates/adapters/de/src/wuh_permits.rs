//! North Rhine-Westphalia's sources of the permit notes (ADR 0124): the Forstbetriebsbezirke of
//! the Landesbetrieb Wald und Holz NRW, each with its Regionalforstamt, and the Staatswald, both
//! as shapefiles on the open data portal of NRW (dl-de/by-2-0).
//!
//! Each source downloads one zip for the whole state (about 11 and 6 MB), reads its `.shp` and
//! `.dbf`, keeps the records whose box meets the map box (EPSG:25832) and turns them into GeoJSON
//! features as longitude, latitude. Every row of the Forstbetriebsbezirke also names the
//! forester with first and last name, e-mail, phone and mobile number: a feature keeps only the
//! fields of [`Layer::kept`], so none of that is stored or reaches the notes.

use std::path::Path;

use freemapper_adapter_api::permits::{
    FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use geo_types::Rect;
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::crs::Crs;
use crate::fetch::{FetchError, Fetcher};
use crate::shape_permits::{record_feature, shape_records};

/// The credit the licence asks for: the publisher as GovData names it, and its licence.
pub const CREDIT: &str = "Landesbetrieb Wald und Holz Nordrhein-Westfalen, Datenlizenz \
    Deutschland – Namensnennung – Version 2.0 (https://www.govdata.de/dl-de/by-2-0)";

/// The administration the notes name.
const WALD_UND_HOLZ: &str = "Landesbetrieb Wald und Holz NRW";

/// The owner of the Staatswald.
const STATE: &str = "Bundesland Nordrhein-Westfalen";

const BASE: &str = "https://www.opengeodata.nrw.de/produkte/umwelt_klima/wald_forst/wald/";

/// How the records of a shapefile become findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    /// A Forstbetriebsbezirk (`fbb`) of a Regionalforstamt (`rfa`).
    Districts,
    /// A parcel of Staatswald with the forest office (`forstamt`) that looks after it.
    StateForest,
}

impl Layer {
    /// The fields a feature keeps.
    fn kept(self) -> &'static [&'static str] {
        match self {
            Layer::Districts => &["rfa", "fbb"],
            Layer::StateForest => &["forstamt"],
        }
    }
}

/// One shapefile of Wald und Holz NRW.
pub struct WuhPermitSource {
    info: PermitSourceInfo,
    zip: &'static str,
    layer: Layer,
}

const LABEL: Wording = Wording {
    en: "Wald und Holz NRW",
    de: "Wald und Holz NRW",
};

/// The two sources, the forest offices first.
pub fn sources() -> Vec<Box<dyn PermitSource>> {
    vec![
        Box::new(WuhPermitSource {
            info: PermitSourceInfo {
                key: "forstbetriebsbezirke",
                label: LABEL,
                what: Wording {
                    en: "forest districts (Forstbetriebsbezirke) with their Regionalforstamt",
                    de: "Forstbetriebsbezirke mit ihrem Regionalforstamt",
                },
                credit: CREDIT,
                note: None,
                layers: &["forstbetriebsbezirke"],
                sections: &[PermitSection::ForestUnits],
                offices: None,
            },
            zip: "forstbetriebsbezirke_nrw_EPSG25832_Shape.zip",
            layer: Layer::Districts,
        }),
        Box::new(WuhPermitSource {
            info: PermitSourceInfo {
                key: "staatswald",
                label: LABEL,
                what: Wording {
                    en: "state forest (Landeseigener Forstbetrieb)",
                    de: "Staatswald (Landeseigener Forstbetrieb)",
                },
                credit: CREDIT,
                note: None,
                layers: &["staatswald"],
                sections: &[PermitSection::StateForest],
                offices: None,
            },
            zip: "landeseigener_forstbetrieb_staatswald_EPSG25832_Shape.zip",
            layer: Layer::StateForest,
        }),
        linfos(),
        crate::shape_permits::source(
            "wasserschutzgebiete",
            Wording {
                en: "Land NRW",
                de: "Land NRW",
            },
            Wording {
                en: "water protection areas (set and planned)",
                de: "Wasserschutzgebiete (festgesetzt und geplant)",
            },
            WSG_CREDIT,
            &["wasserschutzgebiete"],
            &[PermitSection::WaterProtection],
            "https://www.opengeodata.nrw.de/produkte/umwelt_klima/wasser/trinkwasser/wasserschutzgebiete/Wasserschutzgebiete-NRW_EPSG25832_Shape.zip",
            Crs::Utm32,
            crate::wfs_permits::Reading::Water {
                name: "name",
                zone: "wsg_zone",
                citation: None,
                planned: Some(("areastatus", "P")),
            },
        ),
    ]
}

/// The credit of the water protection areas: GovData's licence of the dataset (dl-de/zero-2-0)
/// with the source the open data of NRW name.
const WSG_CREDIT: &str = "Land NRW, Datenlizenz Deutschland – Zero – Version 2.0 \
    (https://www.govdata.de/dl-de/zero-2-0)";

/// The credit of LINFOS, verbatim from the capabilities (`AccessConstraints`) of its WMS; the
/// WFS gives none.
const LINFOS_CREDIT: &str = "Open Data - Dieses Angebot vom Landesamt für Natur, Umwelt und \
    Verbraucherschutz Nordrhein-Westfalen ist lizensiert unter der Datenlizenz Deutschland \
    Namensnennung 2.0 (https://www.govdata.de/dl-de/by-2-0). Datensätze unter dieser Lizenz \
    müssen bei Weiterverwendung mit dem Quellenverweis „Land NRW“ versehen werden.";

/// The Naturwaldzellen and Wildnisentwicklungsgebiete of LINFOS (ADR 0130), a WFS that
/// answers GeoJSON.
fn linfos() -> Box<dyn PermitSource> {
    use crate::wfs_permits::Reading;
    use freemapper_adapter_api::permits::ForestFunctionKind;
    const NATURAL: Reading = Reading::Function {
        kind: ForestFunctionKind::Natural,
        name: Some("objektbezeichnung"),
    };
    crate::wfs_permits::source(
        "linfos",
        Wording {
            en: "LINFOS (Land NRW)",
            de: "LINFOS (Land NRW)",
        },
        Wording {
            en: "Naturwaldzellen and wilderness development areas",
            de: "Naturwaldzellen und Wildnisentwicklungsgebiete",
        },
        LINFOS_CREDIT,
        &["naturwaldzellen", "wildnis"],
        &[PermitSection::ForestFunctions],
        "https://www.wfs.nrw.de/umwelt/linfos",
        "GEOJSON",
        &[
            ("naturwaldzellen", "wfs_linfos:nwz_polygon", NATURAL),
            ("wildnis", "wfs_linfos:weg_polygon", NATURAL),
        ],
    )
}

impl PermitSource for WuhPermitSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let url = format!("{BASE}{}", self.zip);
        let records = shape_records(fetcher, &url, Crs::Utm32, bbox, part)?;
        Ok(vec![
            records
                .iter()
                .filter_map(|r| record_feature(r, self.layer.kept(), Crs::Utm32))
                .collect(),
        ])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != self.info.layers[0] {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|f| finding(self.layer, f))
            .collect()
    }
}

/// The office as the notes name it: the Regionalforstamt of `name`, unless the name is an
/// office of its own ("Nationalparkforstamt Eifel"); "Arnsberger Wald" is the Lehr- und
/// Versuchsforstamt.
fn office(name: &str) -> String {
    if name.to_lowercase().contains("forstamt") {
        name.to_string()
    } else if name == "Arnsberger Wald" {
        format!("Lehr- und Versuchsforstamt {name}")
    } else {
        format!("Regionalforstamt {name}")
    }
}

fn finding(layer: Layer, feature: &Value) -> Option<PermitFinding> {
    let geometry =
        geometry(feature).filter(|g| matches!(g, geo_types::Geometry::MultiPolygon(_)))?;
    let text = |key: &str| {
        let text = feature.get("properties")?.get(key)?.as_str()?.trim();
        (!text.is_empty()).then(|| text.to_string())
    };
    let kind = match layer {
        Layer::Districts => FindingKind::ForestUnit {
            administration: WALD_UND_HOLZ.to_string(),
            office: office(&text("rfa")?),
            unit: text("fbb").map(|name| format!("Forstbetriebsbezirk {name}")),
        },
        Layer::StateForest => FindingKind::StateForest {
            owner: STATE.to_string(),
            office: text("forstamt").map(|name| office(&name)),
        },
    };
    Some(PermitFinding { kind, geometry })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::ProjectedPoint;
    use crate::fetch::test_support::MapFetcher;
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
        let source = &sources()[0];
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
        let source = &sources()[1];
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

    #[test]
    fn offices_are_named_as_wald_und_holz_names_them() {
        assert_eq!(office("Hochstift"), "Regionalforstamt Hochstift");
        assert_eq!(
            office("Nationalparkforstamt Eifel"),
            "Nationalparkforstamt Eifel"
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
        let err = sources()[0]
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
