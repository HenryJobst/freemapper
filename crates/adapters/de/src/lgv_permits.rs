//! Hamburg's sources of the permit notes (ADR 0129): six WFS on `geodienste.hamburg.de`
//! (dl-de/by-2-0) for the Bezirke, the public green spaces, the monuments, the Revierförstereien
//! and the Bannwald of the Stadtwald, the Erholungswald and the water protection areas.
//!
//! All are WFS 2.0.0 that answer GeoJSON as `application/geo+json` and read a box in WGS 84
//! latitude first with its CRS at the end, the generic reader's way ([`get_features`]). Asked so,
//! they write the positions latitude first; the parser of the Berlin sources turns them round.
//! The fetch keeps of every feature only the properties a finding reads ([`Layer::kept`]): the
//! monument list describes each building with its architects, which the notes never carry.

use std::path::Path;

use freemapper_adapter_api::permits::{
    DistrictOffices, FindingKind, ForestFunctionKind, PermitFinding, PermitSection, PermitSource,
    PermitSourceInfo, Wording,
};
use freemapper_sources::geojson_wfs::{GeoJsonWfs, LatLonBox, get_features};
use geo_types::{Geometry, Rect};
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::fetch::{FetchError, Fetcher};

const LABEL: Wording = Wording {
    en: "Geoportal Hamburg",
    de: "Geoportal Hamburg",
};

/// The credit lines, verbatim from the capabilities (`Fees`) of each service.
const LGV: &str = "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt \
    Hamburg, Landesbetrieb Geoinformation und Vermessung";
const BUE: &str = "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt \
    Hamburg, Behörde für Umwelt und Energie";
const BUKEA: &str = "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt \
    Hamburg, Behörde für Umwelt, Klima, Energie und Agrarwirtschaft (BUKEA)";
const DENKMAL: &str = "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und \
    Hansestadt Hamburg, zuständige Behörde";

/// The seven Bezirke by the number the green spaces carry.
const BEZIRKE: [&str; 7] = [
    "Hamburg-Mitte",
    "Altona",
    "Eimsbüttel",
    "Hamburg-Nord",
    "Wandsbek",
    "Bergedorf",
    "Harburg",
];

/// How the features of a layer become findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    District,
    GreenSpace,
    /// A monument of the kind given ("Gartendenkmal").
    Monument(&'static str),
    /// A Revierförsterei of the Stadtwald, run by its Bezirksamt.
    Stadtwald,
    Bannwald,
    Erholungswald,
    WaterProtection,
}

impl Layer {
    /// The properties a feature keeps.
    fn kept(self) -> &'static [&'static str] {
        match self {
            Layer::District => &["bezirk", "bezirk_name"],
            Layer::GreenSpace => &["benennung", "gruen_art", "bezirk"],
            Layer::Monument(_) => &["kdstatus", "bezeichnung", "fisid"],
            Layer::Stadtwald => &["bezirk", "foersterei"],
            Layer::Bannwald => &["bezirk", "name"],
            Layer::Erholungswald => &["bezirk", "erhl_text"],
            Layer::WaterProtection => &["gebietsname", "schutzzone", "rechtsgrundlage"],
        }
    }
}

/// One service of the Geoportal Hamburg: its layers with key, WFS type and reading.
pub struct LgvPermitSource {
    info: PermitSourceInfo,
    service: GeoJsonWfs,
    layers: &'static [(&'static str, &'static str, Layer)],
}

const fn wfs(base: &'static str) -> GeoJsonWfs {
    GeoJsonWfs {
        base,
        // The `+` written as `%2B`, or it reads as a space.
        output_format: "application/geo%2Bjson",
        bbox_crs_suffix: true,
    }
}

#[allow(clippy::too_many_arguments)]
fn source(
    key: &'static str,
    what: Wording,
    credit: &'static str,
    layers: &'static [&'static str],
    sections: &'static [PermitSection],
    offices: Option<DistrictOffices>,
    base: &'static str,
    table: &'static [(&'static str, &'static str, Layer)],
) -> Box<dyn PermitSource> {
    debug_assert!(
        layers.len() == table.len() && layers.iter().zip(table).all(|(a, (b, _, _))| a == b)
    );
    Box::new(LgvPermitSource {
        info: PermitSourceInfo {
            key,
            label: LABEL,
            what,
            credit,
            note: None,
            layers,
            sections,
            offices,
        },
        service: wfs(base),
        layers: table,
    })
}

/// The six services, in the order the notes list their sections.
pub fn sources() -> Vec<Box<dyn PermitSource>> {
    vec![
        source(
            "bezirke",
            Wording {
                en: "Bezirk borders",
                de: "Bezirksgrenzen",
            },
            LGV,
            &["bezirke"],
            &[PermitSection::Districts],
            Some(DistrictOffices::Hamburg),
            "https://geodienste.hamburg.de/HH_WFS_Verwaltungsgrenzen",
            &[("bezirke", "de.hh.up:bezirke", Layer::District)],
        ),
        source(
            "gruenanlagen",
            Wording {
                en: "list of public green spaces (Grünplan)",
                de: "Verzeichnis der öffentlichen Grünanlagen (Grünplan)",
            },
            BUE,
            &["gruenanlagen"],
            &[PermitSection::GreenSpaces],
            None,
            "https://geodienste.hamburg.de/HH_WFS_Gruenplan",
            &[(
                "gruenanlagen",
                "de.hh.up:verzeichnis_oeffentlicher_gruenanlagen",
                Layer::GreenSpace,
            )],
        ),
        source(
            "denkmale",
            Wording {
                en: "monuments (Denkmalschutz)",
                de: "Denkmale (Denkmalschutz)",
            },
            DENKMAL,
            &["gartendenkmale", "ensembles", "gebaeude"],
            &[PermitSection::Monuments],
            None,
            "https://geodienste.hamburg.de/HH_WFS_Denkmalschutz",
            &[
                (
                    "gartendenkmale",
                    "de.hh.up:flaechen_gartendenkmale",
                    Layer::Monument("Gartendenkmal"),
                ),
                (
                    "ensembles",
                    "de.hh.up:ensembles",
                    Layer::Monument("Ensemble"),
                ),
                (
                    "gebaeude",
                    "de.hh.up:gebaeude",
                    Layer::Monument("Baudenkmal"),
                ),
            ],
        ),
        source(
            "wald",
            Wording {
                en: "forest in Hamburg (Revierförstereien and Bannwald)",
                de: "Wald in Hamburg (Revierförstereien und Bannwald)",
            },
            BUKEA,
            &["stadtwald", "bannwald", "bannwald-heuckenlock"],
            &[PermitSection::ForestUnits, PermitSection::ForestFunctions],
            None,
            "https://geodienste.hamburg.de/HH_WFS_Wald_in_Hamburg",
            &[
                (
                    "stadtwald",
                    "de.hh.up:wald_stadtwald_hamburg",
                    Layer::Stadtwald,
                ),
                (
                    "bannwald",
                    "de.hh.up:bannwald_stadtwald_hamburg",
                    Layer::Bannwald,
                ),
                (
                    "bannwald-heuckenlock",
                    "de.hh.up:bannwald_heuckenlock",
                    Layer::Bannwald,
                ),
            ],
        ),
        source(
            "waldfunktionen",
            Wording {
                en: "forest functions (Erholungswald)",
                de: "Waldfunktionen (Erholungswald)",
            },
            BUKEA,
            &["erholungswald"],
            &[PermitSection::ForestFunctions],
            None,
            "https://geodienste.hamburg.de/HH_WFS_Waldfunktionen",
            &[(
                "erholungswald",
                "de.hh.up:waldfunktionen_erholungswald",
                Layer::Erholungswald,
            )],
        ),
        source(
            "wasserschutzgebiete",
            Wording {
                en: "water protection areas",
                de: "Wasserschutzgebiete",
            },
            BUE,
            &["wasserschutzgebiete"],
            &[PermitSection::WaterProtection],
            None,
            "https://geodienste.hamburg.de/HH_WFS_Wasserschutzgebiete",
            &[(
                "wasserschutzgebiete",
                "de.hh.up:wasserschutzgebiete",
                Layer::WaterProtection,
            )],
        ),
    ]
}

impl PermitSource for LgvPermitSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let bbox = LatLonBox {
            min_lat: bbox.min().y,
            min_lon: bbox.min().x,
            max_lat: bbox.max().y,
            max_lon: bbox.max().x,
        };
        self.layers
            .iter()
            .map(|(_, type_name, layer)| {
                let features = get_features(fetcher, &self.service, type_name, &bbox, part)?;
                Ok(features
                    .iter()
                    .filter_map(|f| stored(f, layer.kept()))
                    .collect())
            })
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let Some((_, _, reading)) = self.layers.iter().find(|(key, _, _)| *key == layer) else {
            return Vec::new();
        };
        features
            .iter()
            .filter_map(|feature| finding(*reading, feature))
            .collect()
    }
}

/// A feature as it is stored: the properties of `kept` and the area as longitude, latitude.
fn stored(feature: &Value, kept: &[&str]) -> Option<Value> {
    let area = geometry(feature).filter(|g| matches!(g, Geometry::MultiPolygon(_)))?;
    let properties: serde_json::Map<String, Value> = kept
        .iter()
        .filter_map(|key| {
            Some((
                key.to_string(),
                feature.get("properties")?.get(*key)?.clone(),
            ))
        })
        .collect();
    Some(serde_json::json!({
        "type": "Feature",
        "geometry": geojson::Geometry::new(geojson::GeometryValue::from(&area)),
        "properties": properties,
    }))
}

/// The text of property `key`, trimmed; `None` when it is missing, not text or blank.
fn text(feature: &Value, key: &str) -> Option<String> {
    let value = feature.get("properties")?.get(key)?;
    let text = match value {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}

/// The Bezirk of a green space: the green spaces carry its number.
fn bezirk(number: &str) -> Option<String> {
    let index: usize = number.trim().parse().ok()?;
    BEZIRKE
        .get(index.checked_sub(1)?)
        .map(|name| (*name).to_string())
}

fn finding(layer: Layer, feature: &Value) -> Option<PermitFinding> {
    let geometry = geometry(feature).filter(|g| matches!(g, Geometry::MultiPolygon(_)))?;
    let kind = match layer {
        Layer::District => FindingKind::District {
            name: text(feature, "bezirk_name")?,
        },
        Layer::GreenSpace => FindingKind::GreenSpace {
            name: text(feature, "benennung")
                .or_else(|| text(feature, "gruen_art"))
                .unwrap_or_default(),
            district: text(feature, "bezirk").and_then(|n| bezirk(&n)),
            dedication: None,
        },
        Layer::Monument(kind) => FindingKind::Monument {
            kind: kind.to_string(),
            link: None,
        },
        Layer::Stadtwald => FindingKind::ForestUnit {
            administration: format!("Bezirksamt {}", text(feature, "bezirk")?),
            office: format!("Revierförsterei {}", text(feature, "foersterei")?),
            unit: None,
        },
        Layer::Bannwald => FindingKind::ForestFunction {
            kind: ForestFunctionKind::Protected,
            name: text(feature, "name"),
        },
        Layer::Erholungswald => FindingKind::ForestFunction {
            kind: ForestFunctionKind::Recreation,
            name: text(feature, "erhl_text"),
        },
        Layer::WaterProtection => FindingKind::WaterProtection {
            name: text(feature, "gebietsname"),
            zone: text(feature, "schutzzone"),
            citation: text(feature, "rechtsgrundlage"),
        },
    };
    Some(PermitFinding { kind, geometry })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

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
                "wasserschutzgebiete"
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
        assert_eq!(bezirk("7").as_deref(), Some("Harburg"));
        assert_eq!(bezirk("0"), None);
        assert_eq!(bezirk("8"), None);
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
        let keys: Vec<&String> = layers[0][0]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .collect();
        assert_eq!(keys, ["bezeichnung", "fisid", "kdstatus"]);
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
