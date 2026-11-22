// SPDX-License-Identifier: AGPL-3.0-or-later

//! The sources of the permit notes of Poland (ADR 0175): the protected areas and objects of the
//! GDOŚ (General Directorate for Environmental Protection) and the units of the State Forests
//! from the national register of administrative boundaries (PRG) of GUGiK.
//!
//! The GDOŚ WFS (`sdi.gdos.gov.pl`) is a GeoServer. It answers GeoJSON; asked with the plain
//! `srsName=EPSG:4326` and a box that ends in `EPSG:4326`, it writes longitude first, the order
//! GeoJSON has (with the URN `urn:ogc:def:crs:EPSG::4326` it writes latitude first). The PRG WFS
//! is a MapServer that answers GML 3.2 only, in EPSG:2180 with the northing first.
//!
//! The Nadleśnictwo (forest district) of the State Forests belongs to a regional directorate
//! (RDLP) whose code is the part of the district's code before the hyphen (`17-02` in `17`).
//! The fetch asks both layers, the second without geometry, and stores the name of the
//! directorate in each district, so a stored district says all the notes print.

use std::path::Path;

use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::fetch::{Fetcher, get_limited_with_retries, get_with_retries};
use freemapper_adapter_api::permits::{
    FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, ProtectedObjectKind,
    Wording,
};
use freemapper_georef::crs::{LatLon, Pl1992};
use geo_types::{Geometry, Rect};
use serde_json::{Value, json};

use crate::egib::{Feature, pl1992_inverse, read_members};
use crate::gugik_index::http_error;

/// The host of the GDOŚ services.
pub const GDOS_HOST: &str = "sdi.gdos.gov.pl";

/// The WFS of the GDOŚ.
const GDOS_WFS: &str = "https://sdi.gdos.gov.pl/wfs";

/// The WFS of the administrative boundaries of the PRG.
const PRG_WFS: &str =
    "https://mapy.geoportal.gov.pl/wss/service/PZGIK/PRG/WFS/AdministrativeBoundaries";

/// The credit of the GDOŚ. Its page on the reuse of information allows reuse, also commercial
/// and in derivatives, without a fee (ADR 0175).
pub const GDOS_CREDIT: &str = "© GDOŚ";

/// The credit of GUGiK, as on the map ([`crate::polska::CREDIT`]).
pub const PRG_CREDIT: &str = "© GUGiK, geoportal.gov.pl";

/// Features asked for per page of the GDOŚ.
const GDOS_PAGE: usize = 500;

/// Pages of one layer read at most (2 500 features); more is a box far larger than a map area.
const GDOS_MAX_PAGES: usize = 5;

/// The largest answer read: the polygons of a landscape protection area are several megabytes.
const MAX_BYTES: u64 = 50 * 1024 * 1024;

/// Districts asked for at most in one request: a map area touches a few.
const PRG_COUNT: usize = 100;

/// The name of the State Forests where no regional directorate is found for a district.
const STATE_FORESTS: &str = "Lasy Państwowe";

/// The layers of the GDOŚ in the order of the source: key, type name, kind.
const GDOS_LAYERS: [(&str, &str, ProtectedObjectKind); 11] = [
    (
        "parki-narodowe",
        "GDOS:ParkiNarodowe",
        ProtectedObjectKind::NationalPark,
    ),
    (
        "parki-krajobrazowe",
        "GDOS:ParkiKrajobrazowe",
        ProtectedObjectKind::LandscapePark,
    ),
    (
        "rezerwaty",
        "GDOS:Rezerwaty",
        ProtectedObjectKind::NatureReserve,
    ),
    (
        "obszary-chronionego-krajobrazu",
        "GDOS:ObszaryChronionegoKrajobrazu",
        ProtectedObjectKind::ProtectedLandscapeArea,
    ),
    (
        "natura2000-ptaki",
        "GDOS:ObszarySpecjalnejOchrony",
        ProtectedObjectKind::Natura2000Birds,
    ),
    (
        "natura2000-siedliska",
        "GDOS:SpecjalneObszaryOchrony",
        ProtectedObjectKind::Natura2000Habitats,
    ),
    (
        "pomniki-powierzchniowe",
        "GDOS:PomnikiPrzyrodyPowierzchniowe",
        ProtectedObjectKind::NaturalMonumentPl,
    ),
    (
        "pomniki-punktowe",
        "GDOS:PomnikiPrzyrodyPunktowe",
        ProtectedObjectKind::NaturalMonumentPl,
    ),
    (
        "uzytki-ekologiczne",
        "GDOS:UzytkiEkologiczne",
        ProtectedObjectKind::EcologicalSite,
    ),
    (
        "zespoly-przyrodniczo-krajobrazowe",
        "GDOS:ZespolyPrzyrodniczoKrajobrazowe",
        ProtectedObjectKind::LandscapeComplex,
    ),
    (
        "stanowiska-dokumentacyjne",
        "GDOS:StanowiskaDokumentacyjne",
        ProtectedObjectKind::DocumentationSite,
    ),
];

/// The keys of [`GDOS_LAYERS`].
const GDOS_KEYS: [&str; 11] = [
    "parki-narodowe",
    "parki-krajobrazowe",
    "rezerwaty",
    "obszary-chronionego-krajobrazu",
    "natura2000-ptaki",
    "natura2000-siedliska",
    "pomniki-powierzchniowe",
    "pomniki-punktowe",
    "uzytki-ekologiczne",
    "zespoly-przyrodniczo-krajobrazowe",
    "stanowiska-dokumentacyjne",
];

/// The sources of Poland.
pub fn sources() -> Vec<Box<dyn PermitSource>> {
    vec![
        Box::new(Gdos {
            info: PermitSourceInfo {
                key: "gdos",
                label: Wording {
                    en: "GDOŚ",
                    de: "GDOŚ",
                },
                what: Wording {
                    en: "protected areas and objects of nature conservation",
                    de: "Schutzgebiete und Schutzobjekte des Naturschutzes",
                },
                credit: GDOS_CREDIT,
                note: None,
                sampled: false,
                gaps: None,
                layers: &GDOS_KEYS,
                sections: &[PermitSection::ProtectedObjects],
                offices: None,
            },
        }),
        Box::new(Prg {
            info: PermitSourceInfo {
                key: "prg",
                label: Wording {
                    en: "GUGiK (PRG)",
                    de: "GUGiK (PRG)",
                },
                what: Wording {
                    en: "forest districts (Nadleśnictwa) of the State Forests",
                    de: "Forstbezirke (Nadleśnictwa) der Staatsforsten",
                },
                credit: PRG_CREDIT,
                note: None,
                sampled: false,
                gaps: Some(Wording {
                    en: "forest that does not belong to the State Forests, such as private forest and the forest of the national parks, is not in it",
                    de: "Wald außerhalb der Staatsforsten, etwa Privatwald und der Wald der Nationalparks, fehlt darin",
                }),
                layers: &["nadlesnictwa"],
                sections: &[PermitSection::ForestUnits],
                offices: None,
            },
        }),
    ]
}

/// The protected areas and objects of the GDOŚ.
struct Gdos {
    info: PermitSourceInfo,
}

/// The URL of the page of `type_name` in `bbox` that starts at `start`.
fn gdos_url(type_name: &str, bbox: &Rect<f64>, start: usize) -> String {
    format!(
        "{GDOS_WFS}?service=WFS&version=2.0.0&request=GetFeature&outputFormat=application/json\
         &srsName=EPSG:4326&typeNames={type_name}&count={GDOS_PAGE}&startIndex={start}\
         &bbox={},{},{},{},EPSG:4326",
        bbox.min().x,
        bbox.min().y,
        bbox.max().x,
        bbox.max().y,
    )
}

/// The `features` of a GeoJSON feature collection.
fn read_collection(url: &str, bytes: &[u8]) -> Result<Vec<Value>, FetchError> {
    let mut value: Value = serde_json::from_slice(bytes)
        .map_err(|e| http_error(url, format!("the answer is not GeoJSON: {e}")))?;
    match value.get_mut("features").map(Value::take) {
        Some(Value::Array(features)) => Ok(features),
        _ => Err(http_error(url, "the answer is not a FeatureCollection")),
    }
}

impl PermitSource for Gdos {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        GDOS_LAYERS
            .iter()
            .map(|(_, type_name, _)| {
                let mut features = Vec::new();
                for _ in 0..GDOS_MAX_PAGES {
                    let url = gdos_url(type_name, bbox, features.len());
                    let page = get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| {
                        read_collection(&url, bytes)
                    })?;
                    let short = page.len() < GDOS_PAGE;
                    features.extend(page);
                    if short {
                        return Ok(features);
                    }
                }
                Err(http_error(
                    GDOS_WFS,
                    format!(
                        "the service sent more than {GDOS_MAX_PAGES} full pages of {type_name}"
                    ),
                ))
            })
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let Some((_, _, kind)) = GDOS_LAYERS.iter().find(|(key, ..)| *key == layer) else {
            return Vec::new();
        };
        features
            .iter()
            .filter_map(|feature| {
                let geometry = geometry_of(feature)?;
                let kind = object_kind(*kind, feature);
                Some(PermitFinding {
                    kind: FindingKind::ProtectedObject {
                        kind,
                        name: object_name(kind, feature),
                        citation: None,
                    },
                    geometry,
                })
            })
            .collect()
    }
}

/// The text of the property `key`, trimmed; `None` when it is missing, not text or blank.
fn property(feature: &Value, key: &str) -> Option<String> {
    let text = feature.get("properties")?.get(key)?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// The geometry of a feature as GeoJSON in WGS 84, longitude first.
fn geometry_of(feature: &Value) -> Option<Geometry<f64>> {
    let geometry =
        serde_json::from_value::<geojson::Geometry>(feature.get("geometry")?.clone()).ok()?;
    Geometry::<f64>::try_from(geometry).ok()
}

/// The buffer zone of a park is a feature of its own, named like the park with "- otulina".
fn is_buffer_zone(name: &str) -> bool {
    name.to_lowercase().trim_end().ends_with("otulina")
}

/// The kind of a feature: the layer's, except for the buffer zone of a park.
fn object_kind(kind: ProtectedObjectKind, feature: &Value) -> ProtectedObjectKind {
    let buffer = matches!(
        kind,
        ProtectedObjectKind::NationalPark | ProtectedObjectKind::LandscapePark
    ) && property(feature, "nazwa").is_some_and(|name| is_buffer_zone(&name));
    if buffer {
        ProtectedObjectKind::ParkBufferZone
    } else {
        kind
    }
}

/// The name of an object. A natural monument often has none; the data then give its species
/// (`Lipa drobnolistna - Tilia cordata`), as the German sources do for a tree.
fn object_name(kind: ProtectedObjectKind, feature: &Value) -> Option<String> {
    property(feature, "nazwa").or_else(|| {
        (kind == ProtectedObjectKind::NaturalMonumentPl)
            .then(|| property(feature, "gatunek"))
            .flatten()
    })
}

/// The forest districts of the State Forests.
struct Prg {
    info: PermitSourceInfo,
}

/// The box in EPSG:2180 that holds the box `bbox` (WGS 84), with the margin of a metre.
fn box_2180(bbox: &Rect<f64>) -> (f64, f64, f64, f64) {
    let corners = [
        (bbox.min().x, bbox.min().y),
        (bbox.min().x, bbox.max().y),
        (bbox.max().x, bbox.min().y),
        (bbox.max().x, bbox.max().y),
    ];
    let mut out = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (lon, lat) in corners {
        let p = Pl1992::forward(LatLon { lat, lon });
        out = (
            out.0.min(p.northing),
            out.1.min(p.easting),
            out.2.max(p.northing),
            out.3.max(p.easting),
        );
    }
    (out.0.floor(), out.1.floor(), out.2.ceil(), out.3.ceil())
}

/// The request for `type_name` in the box; `properties` limits what the service sends (a
/// regional directorate has a polygon of hundreds of kilobytes that is not needed).
fn prg_url(type_name: &str, properties: &str, bbox: &Rect<f64>) -> String {
    let crs = "urn:ogc:def:crs:EPSG::2180";
    let (min_n, min_e, max_n, max_e) = box_2180(bbox);
    format!(
        "{PRG_WFS}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}\
         &COUNT={PRG_COUNT}&PROPERTYNAME={properties}&SRSNAME={crs}\
         &BBOX={min_n},{min_e},{max_n},{max_e},{crs}"
    )
}

/// The features of one layer of the PRG: the code and the name of the unit, with polygons.
fn prg_features(
    fetcher: &dyn Fetcher,
    element: &str,
    properties: &str,
    bbox: &Rect<f64>,
) -> Result<Vec<Feature>, FetchError> {
    let url = prg_url(&format!("ms:{element}"), properties, bbox);
    let page = get_with_retries(fetcher, &url, |body| {
        read_members(element, "JPT_KOD_JE", "JPT_NAZWA_", body, &url)
    })?;
    Ok(page.features)
}

/// The polygons of a feature in WGS 84 as GeoJSON; `None` when it has none or a point has no
/// position.
fn prg_geometry(feature: &Feature) -> Option<Value> {
    let polygons = feature
        .polygons
        .iter()
        .map(|rings| {
            rings
                .iter()
                .map(|ring| {
                    ring.iter()
                        .map(|&(north, east)| pl1992_inverse(east, north).map(|p| [p.lon, p.lat]))
                        .collect::<Option<Vec<_>>>()
                })
                .collect::<Option<Vec<_>>>()
        })
        .collect::<Option<Vec<_>>>()?;
    (!polygons.is_empty()).then(|| json!({"type": "MultiPolygon", "coordinates": polygons}))
}

/// The code of the regional directorate of a district code: `17` of `17-02`.
fn directorate_code(district_code: &str) -> &str {
    district_code.split('-').next().unwrap_or_default()
}

impl PermitSource for Prg {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        _part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let districts = prg_features(
            fetcher,
            "U06_Nadlesnictwo",
            "msGeometry,JPT_KOD_JE,JPT_NAZWA_",
            bbox,
        )?;
        let directorates = if districts.is_empty() {
            Vec::new()
        } else {
            prg_features(
                fetcher,
                "U07_Regionalna_dyrekcja_lasow_panstwowych",
                "JPT_KOD_JE,JPT_NAZWA_",
                bbox,
            )?
        };
        let features = districts
            .iter()
            .filter_map(|district| {
                let geometry = prg_geometry(district)?;
                let directorate = directorates
                    .iter()
                    .find(|d| d.id == directorate_code(&district.id))
                    .map(|d| d.kind.as_str());
                Some(json!({
                    "type": "Feature",
                    "properties": {
                        "kod": district.id,
                        "nazwa": district.kind,
                        "rdlp": directorate,
                    },
                    "geometry": geometry,
                }))
            })
            .collect();
        Ok(vec![features])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != "nadlesnictwa" {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|feature| {
                let office = title_case(&property(feature, "nazwa")?);
                let administration =
                    property(feature, "rdlp").unwrap_or_else(|| STATE_FORESTS.to_string());
                Some(PermitFinding {
                    kind: FindingKind::ForestUnit {
                        administration,
                        office,
                        unit: None,
                    },
                    geometry: geometry_of(feature)?,
                })
            })
            .collect()
    }
}

/// `NADLEŚNICTWO GOLUB-DOBRZYŃ` as `Nadleśnictwo Golub-Dobrzyń`: the register writes the names
/// of the districts in capitals. A name that is not all capitals stays as it is.
fn title_case(name: &str) -> String {
    if name.chars().any(char::is_lowercase) {
        return name.to_string();
    }
    let mut out = String::with_capacity(name.len());
    let mut word_start = true;
    for c in name.chars() {
        if c.is_alphabetic() {
            if word_start {
                out.extend(c.to_uppercase());
            } else {
                out.extend(c.to_lowercase());
            }
            word_start = false;
        } else {
            out.push(c);
            word_start = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::testing::MapFetcher;
    use geo_types::{Coord, Point};

    const GDOS_FIXTURES: [(&str, &str); 11] = [
        (
            "parki-narodowe",
            include_str!("../tests/fixtures/permits/parki-narodowe.json"),
        ),
        (
            "parki-krajobrazowe",
            include_str!("../tests/fixtures/permits/parki-krajobrazowe.json"),
        ),
        (
            "rezerwaty",
            include_str!("../tests/fixtures/permits/rezerwaty.json"),
        ),
        (
            "obszary-chronionego-krajobrazu",
            include_str!("../tests/fixtures/permits/obszary-chronionego-krajobrazu.json"),
        ),
        (
            "natura2000-ptaki",
            include_str!("../tests/fixtures/permits/natura2000-ptaki.json"),
        ),
        (
            "natura2000-siedliska",
            include_str!("../tests/fixtures/permits/natura2000-siedliska.json"),
        ),
        (
            "pomniki-powierzchniowe",
            include_str!("../tests/fixtures/permits/pomniki-powierzchniowe.json"),
        ),
        (
            "pomniki-punktowe",
            include_str!("../tests/fixtures/permits/pomniki-punktowe.json"),
        ),
        (
            "uzytki-ekologiczne",
            include_str!("../tests/fixtures/permits/uzytki-ekologiczne.json"),
        ),
        (
            "zespoly-przyrodniczo-krajobrazowe",
            include_str!("../tests/fixtures/permits/zespoly-przyrodniczo-krajobrazowe.json"),
        ),
        (
            "stanowiska-dokumentacyjne",
            include_str!("../tests/fixtures/permits/stanowiska-dokumentacyjne.json"),
        ),
    ];
    const DISTRICTS: &str = include_str!("../tests/fixtures/permits/prg-nadlesnictwa.xml");
    const DIRECTORATES: &str = include_str!("../tests/fixtures/permits/prg-rdlp.xml");

    fn gdos() -> Gdos {
        let Some(source) = sources().into_iter().next() else {
            unreachable!("the source list has the GDOŚ");
        };
        Gdos {
            info: source.info().clone(),
        }
    }

    fn prg() -> Prg {
        let source = sources().into_iter().nth(1).expect("the PRG is the second");
        Prg {
            info: source.info().clone(),
        }
    }

    /// The features of a fixture.
    fn features(text: &str) -> Vec<Value> {
        read_collection("fixture", text.as_bytes()).unwrap()
    }

    fn unwrap_object(finding: &PermitFinding) -> (ProtectedObjectKind, Option<&str>) {
        match &finding.kind {
            FindingKind::ProtectedObject { kind, name, .. } => (*kind, name.as_deref()),
            other => panic!("not a protected object: {other:?}"),
        }
    }

    #[test]
    fn the_keys_of_the_layers_are_plain_and_match_the_table() {
        let source = gdos();
        assert_eq!(source.info.layers.len(), GDOS_LAYERS.len());
        for (key, (table_key, ..)) in source.info.layers.iter().zip(GDOS_LAYERS) {
            assert_eq!(*key, table_key);
            assert!(freemapper_adapter_api::permits::plain_key(key), "{key}");
        }
        for (key, _) in GDOS_FIXTURES {
            assert!(GDOS_KEYS.contains(&key), "{key}");
        }
    }

    #[test]
    fn each_layer_of_the_gdos_gives_its_kind_and_the_name_of_the_data() {
        let source = gdos();
        let expected = [
            ("rezerwaty", ProtectedObjectKind::NatureReserve),
            (
                "obszary-chronionego-krajobrazu",
                ProtectedObjectKind::ProtectedLandscapeArea,
            ),
            ("natura2000-ptaki", ProtectedObjectKind::Natura2000Birds),
            (
                "natura2000-siedliska",
                ProtectedObjectKind::Natura2000Habitats,
            ),
            ("uzytki-ekologiczne", ProtectedObjectKind::EcologicalSite),
            (
                "zespoly-przyrodniczo-krajobrazowe",
                ProtectedObjectKind::LandscapeComplex,
            ),
            (
                "stanowiska-dokumentacyjne",
                ProtectedObjectKind::DocumentationSite,
            ),
        ];
        for (layer, kind) in expected {
            let text = GDOS_FIXTURES.iter().find(|(k, _)| *k == layer).unwrap().1;
            let found = source.findings(layer, &features(text));
            assert_eq!(found.len(), 2, "{layer}");
            for finding in &found {
                let (got, name) = unwrap_object(finding);
                assert_eq!(got, kind, "{layer}");
                assert!(name.is_some_and(|n| !n.is_empty()), "{layer}");
                assert!(
                    matches!(finding.geometry, Geometry::MultiPolygon(_)),
                    "{layer}"
                );
            }
        }
    }

    #[test]
    fn natura_2000_names_come_from_the_nazwa_property() {
        let found = gdos().findings("natura2000-ptaki", &features(GDOS_FIXTURES[4].1));
        let names: Vec<_> = found.iter().map(|f| unwrap_object(f).1.unwrap()).collect();
        assert_eq!(names, ["Zalew Wiślany", "Bagienna Dolina Narwi"]);
    }

    /// The two layers of parks hold the park and its buffer zone; the park is its layer's kind.
    #[test]
    fn a_park_without_otulina_is_the_kind_of_its_layer() {
        let park = |name: &str| {
            json!({"type": "Feature", "properties": {"nazwa": name},
                "geometry": {"type": "Point", "coordinates": [20.0, 52.0]}})
        };
        let found = gdos().findings(
            "parki-krajobrazowe",
            &[park("Mazowiecki Park Krajobrazowy")],
        );
        assert_eq!(
            unwrap_object(&found[0]).0,
            ProtectedObjectKind::LandscapePark
        );
        let found = gdos().findings("parki-narodowe", &[park("Kampinoski Park Narodowy")]);
        assert_eq!(
            unwrap_object(&found[0]).0,
            ProtectedObjectKind::NationalPark
        );
    }

    /// The buffer zone of a park is a feature named "… - otulina": its own kind.
    #[test]
    fn the_buffer_zone_of_a_park_is_its_own_kind() {
        let found = gdos().findings("parki-narodowe", &features(GDOS_FIXTURES[0].1));
        let kinds: Vec<_> = found
            .iter()
            .map(|f| (unwrap_object(f).0, unwrap_object(f).1.unwrap().to_string()))
            .collect();
        assert_eq!(
            kinds,
            [
                (
                    ProtectedObjectKind::NationalPark,
                    "Roztoczański Park Narodowy".to_string()
                ),
                (
                    ProtectedObjectKind::ParkBufferZone,
                    "Kampinoski Park Narodowy - otulina".to_string()
                ),
            ]
        );
        let found = gdos().findings("parki-krajobrazowe", &features(GDOS_FIXTURES[1].1));
        assert!(
            found
                .iter()
                .all(|f| unwrap_object(f).0 == ProtectedObjectKind::ParkBufferZone)
        );
    }

    /// A natural monument without a name gets its species; a point layer gives points.
    #[test]
    fn natural_monuments_without_a_name_give_their_species() {
        let source = gdos();
        let points = source.findings("pomniki-punktowe", &features(GDOS_FIXTURES[7].1));
        assert_eq!(points.len(), 2);
        assert_eq!(
            unwrap_object(&points[0]),
            (ProtectedObjectKind::NaturalMonumentPl, Some("Wojtek"))
        );
        assert_eq!(
            unwrap_object(&points[1]),
            (
                ProtectedObjectKind::NaturalMonumentPl,
                Some("Lipa drobnolistna - Tilia cordata")
            )
        );
        assert!(matches!(points[0].geometry, Geometry::Point(_)));
        let areas = source.findings("pomniki-powierzchniowe", &features(GDOS_FIXTURES[6].1));
        // The first area has neither a name nor a species in the sample.
        assert_eq!(unwrap_object(&areas[0]).1, None);
        assert_eq!(unwrap_object(&areas[1]).1, Some("Aleja Lipowa"));
    }

    #[test]
    fn a_feature_without_a_geometry_is_left_out() {
        let found = gdos().findings(
            "rezerwaty",
            &[json!({"type": "Feature", "properties": {"nazwa": "X"}, "geometry": null})],
        );
        assert!(found.is_empty());
        assert!(
            gdos()
                .findings("unknown", &features(GDOS_FIXTURES[2].1))
                .is_empty()
        );
    }

    #[test]
    fn the_url_asks_longitude_first_in_the_plain_crs() {
        let bbox = Rect::new(Coord { x: 20.55, y: 52.30 }, Coord { x: 20.57, y: 52.32 });
        let url = gdos_url("GDOS:ParkiNarodowe", &bbox, 500);
        assert!(url.starts_with("https://sdi.gdos.gov.pl/wfs?service=WFS&version=2.0.0&"));
        assert!(
            url.contains("&outputFormat=application/json&srsName=EPSG:4326&"),
            "{url}"
        );
        assert!(url.contains("&typeNames=GDOS:ParkiNarodowe&"), "{url}");
        assert!(url.contains("&startIndex=500&"), "{url}");
        assert!(
            url.ends_with("&bbox=20.55,52.3,20.57,52.32,EPSG:4326"),
            "{url}"
        );
    }

    #[test]
    fn the_fetch_asks_every_layer_once_and_returns_them_in_order() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(Coord { x: 20.55, y: 52.30 }, Coord { x: 20.57, y: 52.32 });
        for (key, text) in GDOS_FIXTURES {
            let (_, type_name, _) = GDOS_LAYERS.iter().find(|(k, ..)| *k == key).unwrap();
            fetcher.insert(&gdos_url(type_name, &bbox, 0), text.as_bytes().to_vec());
        }
        let part = tempfile::tempdir().unwrap().keep().join("page.part");
        let lists = gdos().fetch(&fetcher, &bbox, &part).unwrap();
        assert_eq!(lists.len(), 11);
        assert!(lists.iter().all(|l| l.len() == 2));
        assert_eq!(fetcher.calls.get(), 11);
    }

    #[test]
    fn the_fetch_stops_at_the_first_layer_that_fails() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(Coord { x: 20.55, y: 52.30 }, Coord { x: 20.57, y: 52.32 });
        let part = tempfile::tempdir().unwrap().keep().join("page.part");
        assert!(gdos().fetch(&fetcher, &bbox, &part).is_err());
        // An exception report with status 200 is no collection either.
        fetcher.insert(
            &gdos_url("GDOS:ParkiNarodowe", &bbox, 0),
            b"<ows:ExceptionReport/>".to_vec(),
        );
        assert!(gdos().fetch(&fetcher, &bbox, &part).is_err());
    }

    #[test]
    fn a_full_page_asks_for_the_next_one() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(Coord { x: 20.55, y: 52.30 }, Coord { x: 20.57, y: 52.32 });
        let feature = json!({"type": "Feature", "properties": {"nazwa": "X"},
            "geometry": {"type": "Point", "coordinates": [20.56, 52.31]}});
        let full = json!({"type": "FeatureCollection", "features": vec![feature; GDOS_PAGE]});
        let first = gdos_url("GDOS:ParkiNarodowe", &bbox, 0);
        fetcher.insert(&first, serde_json::to_vec(&full).unwrap());
        fetcher.insert(
            &gdos_url("GDOS:ParkiNarodowe", &bbox, GDOS_PAGE),
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        fetcher.insert_prefix(
            "https://sdi.gdos.gov.pl/wfs?service=WFS&version=2.0.0&request=GetFeature&outputFormat=application/json&srsName=EPSG:4326&typeNames=GDOS:",
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        let part = tempfile::tempdir().unwrap().keep().join("page.part");
        let lists = gdos().fetch(&fetcher, &bbox, &part).unwrap();
        assert_eq!(lists[0].len(), GDOS_PAGE);
    }

    /// A collection of `n` point features, each with `pad` bytes of padding.
    fn collection(n: usize, pad: usize) -> Vec<u8> {
        let feature = json!({"type": "Feature",
            "properties": {"nazwa": "X", "pad": "x".repeat(pad)},
            "geometry": {"type": "Point", "coordinates": [20.56, 52.31]}});
        serde_json::to_vec(&json!({"type": "FeatureCollection", "features": vec![feature; n]}))
            .unwrap()
    }

    /// Answers every layer at its first page and nothing else, with `first` for the first layer.
    fn only_first_pages(bbox: &Rect<f64>, first: Vec<u8>) -> MapFetcher {
        let fetcher = MapFetcher::default();
        for (i, (_, type_name, _)) in GDOS_LAYERS.iter().enumerate() {
            let body = if i == 0 {
                first.clone()
            } else {
                collection(0, 0)
            };
            fetcher.insert(&gdos_url(type_name, bbox, 0), body);
        }
        fetcher
    }

    #[test]
    fn a_page_one_short_of_full_is_the_last_one() {
        let bbox = Rect::new(Coord { x: 20.55, y: 52.30 }, Coord { x: 20.57, y: 52.32 });
        let fetcher = only_first_pages(&bbox, collection(GDOS_PAGE - 1, 0));
        let part = tempfile::tempdir().unwrap().keep().join("page.part");
        let lists = gdos().fetch(&fetcher, &bbox, &part).unwrap();
        assert_eq!(lists[0].len(), GDOS_PAGE - 1);
        assert_eq!(fetcher.calls.get(), 11);
    }

    #[test]
    fn a_full_page_is_not_the_last_one() {
        let bbox = Rect::new(Coord { x: 20.55, y: 52.30 }, Coord { x: 20.57, y: 52.32 });
        // The second page of the first layer is not served: asking for it fails the fetch.
        let fetcher = only_first_pages(&bbox, collection(GDOS_PAGE, 0));
        let part = tempfile::tempdir().unwrap().keep().join("page.part");
        assert!(gdos().fetch(&fetcher, &bbox, &part).is_err());
    }

    #[test]
    fn an_answer_of_several_megabytes_is_read() {
        // Landscape protection areas come as polygons of several megabytes.
        let bbox = Rect::new(Coord { x: 20.55, y: 52.30 }, Coord { x: 20.57, y: 52.32 });
        let big = collection(1, 3 * 1024 * 1024);
        assert!(big.len() > 3 * 1024 * 1024);
        let fetcher = only_first_pages(&bbox, big);
        let part = tempfile::tempdir().unwrap().keep().join("page.part");
        let lists = gdos().fetch(&fetcher, &bbox, &part).unwrap();
        assert_eq!(lists[0].len(), 1);
    }

    #[test]
    fn the_districts_come_with_their_directorate_and_positions_in_wgs_84() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(Coord { x: 20.9, y: 52.2 }, Coord { x: 21.0, y: 52.3 });
        fetcher.insert(
            &prg_url(
                "ms:U06_Nadlesnictwo",
                "msGeometry,JPT_KOD_JE,JPT_NAZWA_",
                &bbox,
            ),
            DISTRICTS.as_bytes().to_vec(),
        );
        fetcher.insert(
            &prg_url(
                "ms:U07_Regionalna_dyrekcja_lasow_panstwowych",
                "JPT_KOD_JE,JPT_NAZWA_",
                &bbox,
            ),
            DIRECTORATES.as_bytes().to_vec(),
        );
        let part = Path::new("unused");
        let lists = prg().fetch(&fetcher, &bbox, part).unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].len(), 2);
        let found = prg().findings("nadlesnictwa", &lists[0]);
        let units: Vec<_> = found
            .iter()
            .map(|f| match &f.kind {
                FindingKind::ForestUnit {
                    administration,
                    office,
                    unit,
                } => (administration.as_str(), office.as_str(), unit.is_none()),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            units,
            [
                (
                    "Regionalna Dyrekcja Lasów Państwowych w Warszawie",
                    "Nadleśnictwo Chojnów",
                    true
                ),
                (
                    "Regionalna Dyrekcja Lasów Państwowych w Warszawie",
                    "Nadleśnictwo Jabłonna",
                    true
                ),
            ]
        );
        // The sample rectangle lies around Warsaw: positions are longitude, latitude.
        let Geometry::MultiPolygon(area) = &found[0].geometry else {
            panic!("an area")
        };
        let corner = area.0[0].exterior().0[0];
        assert!((19.0..22.0).contains(&corner.x), "{corner:?}");
        assert!((51.5..53.0).contains(&corner.y), "{corner:?}");
        assert!(geo::Contains::contains(area, &Point::new(21.0, 52.2)));
    }

    /// No district near, no second request: a town without State Forests costs one.
    #[test]
    fn without_a_district_the_directorates_are_not_asked() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(Coord { x: 20.9, y: 52.2 }, Coord { x: 21.0, y: 52.3 });
        let empty = DISTRICTS.split("<wfs:member>").next().unwrap().to_string()
            + "</wfs:FeatureCollection>";
        fetcher.insert(
            &prg_url(
                "ms:U06_Nadlesnictwo",
                "msGeometry,JPT_KOD_JE,JPT_NAZWA_",
                &bbox,
            ),
            empty.into_bytes(),
        );
        let lists = prg().fetch(&fetcher, &bbox, Path::new("unused")).unwrap();
        assert_eq!(lists, vec![Vec::<Value>::new()]);
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn a_district_without_a_known_directorate_names_the_state_forests() {
        let feature = json!({
            "type": "Feature",
            "properties": {"kod": "99-01", "nazwa": "NADLEŚNICTWO ZGORZELEC", "rdlp": null},
            "geometry": {"type": "MultiPolygon", "coordinates": [[[[15.0, 51.0], [15.1, 51.0], [15.1, 51.1], [15.0, 51.0]]]]},
        });
        let found = prg().findings("nadlesnictwa", &[feature]);
        assert_eq!(
            found[0].kind,
            FindingKind::ForestUnit {
                administration: "Lasy Państwowe".to_string(),
                office: "Nadleśnictwo Zgorzelec".to_string(),
                unit: None,
            }
        );
    }

    #[test]
    fn the_names_of_the_register_are_cased_for_reading() {
        assert_eq!(
            title_case("NADLEŚNICTWO GOLUB-DOBRZYŃ"),
            "Nadleśnictwo Golub-Dobrzyń"
        );
        assert_eq!(
            title_case("NADLEŚNICTWO ŚWIERADÓW"),
            "Nadleśnictwo Świeradów"
        );
        assert_eq!(title_case("Nadleśnictwo Kłobuck"), "Nadleśnictwo Kłobuck");
        assert_eq!(directorate_code("17-02"), "17");
        assert_eq!(directorate_code("17"), "17");
    }

    #[test]
    fn the_box_in_the_polish_grid_holds_the_box_in_degrees() {
        let bbox = Rect::new(Coord { x: 20.9, y: 52.2 }, Coord { x: 21.0, y: 52.3 });
        let (min_n, min_e, max_n, max_e) = box_2180(&bbox);
        // Warsaw lies near 487 000 N, 637 000 E.
        assert!((470_000.0..500_000.0).contains(&min_n) && max_n > min_n);
        assert!((620_000.0..650_000.0).contains(&min_e) && max_e > min_e);
    }

    #[test]
    fn an_exception_report_of_the_prg_is_an_error() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(Coord { x: 20.9, y: 52.2 }, Coord { x: 21.0, y: 52.3 });
        fetcher.insert(
            &prg_url("ms:U06_Nadlesnictwo", "msGeometry,JPT_KOD_JE,JPT_NAZWA_", &bbox),
            br#"<?xml version="1.0"?><ows:ExceptionReport xmlns:ows="http://www.opengis.net/ows/1.1"><ows:Exception exceptionCode="NoApplicableCode"><ows:ExceptionText>x</ows:ExceptionText></ows:Exception></ows:ExceptionReport>"#.to_vec(),
        );
        assert!(prg().fetch(&fetcher, &bbox, Path::new("unused")).is_err());
    }

    #[test]
    fn the_sources_name_their_credit_and_sections() {
        let all = sources();
        let infos: Vec<_> = all.iter().map(|s| s.info()).collect();
        assert_eq!(infos[0].credit, "© GDOŚ");
        assert_eq!(infos[0].sections, [PermitSection::ProtectedObjects]);
        assert_eq!(infos[1].credit, "© GUGiK, geoportal.gov.pl");
        assert_eq!(infos[1].sections, [PermitSection::ForestUnits]);
    }
}
