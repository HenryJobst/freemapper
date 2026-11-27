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
//! The administrative units come from the same register (ADR 0181): the fetch asks the
//! municipalities (gminy) of the box with their polygons and the names of the counties (powiaty)
//! and voivodeships (województwa) without polygons. A municipality's code (TERYT) starts with the
//! code of its county (four digits) and its voivodeship (two), so each municipality gives three
//! findings with its own polygon and the notes sum the shares.
//!
//! The register of immovable monuments of the National Heritage Institute (NID, ADR 0184) has no
//! geometry in its WMS. The fetch asks `GetFeatureInfo` with the whole map box as one pixel and
//! gets every monument of the box with its name; the notes count them and name the first few,
//! and say that the data give no position. The register of archaeological monuments (the book C
//! of the register, ADR 0190) is a second service of the same host with the same kind of answer
//! and gets the same treatment in a section of its own. The monuments of history (Pomniki
//! historii, ADR 0191) are a WFS with the true shapes; the fetch asks the map box in EPSG:2180
//! (northing first, the CRS at the end of the box) and converts the GeoJSON, which has the
//! easting first, to WGS 84, so the notes can tell a monument in the map area from one in the
//! rectangle around it.
//!
//! The Nadleśnictwo (forest district) of the State Forests belongs to a regional directorate
//! (RDLP) whose code is the part of the district's code before the hyphen (`17-02` in `17`).
//! The fetch asks both layers, the second without geometry, and stores the name of the
//! directorate in each district, so a stored district says all the notes print.

use std::path::Path;

use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::fetch::{
    Fetcher, get_limited_with_retries, get_with_retries, is_ogc_report,
};
use freemapper_adapter_api::permits::{
    AdminUnitLevel, FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo,
    ProtectedObjectKind, Wording,
};
use freemapper_georef::crs::{LatLon, Pl1992};
use geo_types::{Geometry, Rect};
use quick_xml::events::Event;
use serde_json::{Value, json};

use crate::egib::{Feature, pl1992_inverse, read_members};
use crate::gugik_index::{attribute, entity, http_error, local, report_error};

/// The host of the GDOŚ services.
pub const GDOS_HOST: &str = "sdi.gdos.gov.pl";

/// The WFS of the GDOŚ.
const GDOS_WFS: &str = "https://sdi.gdos.gov.pl/wfs";

/// The WFS of the administrative boundaries of the PRG.
const PRG_WFS: &str =
    "https://mapy.geoportal.gov.pl/wss/service/PZGIK/PRG/WFS/AdministrativeBoundaries";

/// The host of the register of monuments of the NID.
pub const NID_HOST: &str = "usluga.zabytek.gov.pl";

/// The WMS of the register of immovable monuments (INSPIRE, layer `Immovable_Monuments`).
const NID_WMS: &str = "https://usluga.zabytek.gov.pl/INSPIRE_IMD/service.svc/get";

/// The WMS of the register of archaeological monuments (INSPIRE, layer `Archaeological_Monuments`,
/// ADR 0190): the sites entered in the book C of the register.
const NID_ARCHAEOLOGY_WMS: &str = "https://usluga.zabytek.gov.pl/INSPIRE_AMD/service.svc/get";

/// The WFS of the monuments of history (INSPIRE, type `gmgml:Monuments_of_History`, ADR 0191):
/// GeoJSON with true polygons, lines and points, only in EPSG:2180.
const NID_HISTORY_WFS: &str = "https://usluga.zabytek.gov.pl/INSPIRE_WFS_MHD/service.svc/get";

/// The credit of the NID: its data are open under CC BY 4.0 (dane.gov.pl, dataset 1130); the
/// licence names no wording, so the notes name the institute and the licence (ADR 0184).
pub const NID_CREDIT: &str = "© Narodowy Instytut Dziedzictwa (CC BY 4.0)";

/// The most monuments one request asks for. The service answers one monument without it; a
/// full answer means the box is far larger than a map area and is an error, not a partial list.
const NID_COUNT: usize = 5000;

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
        Box::new(PrgAdmin {
            info: PermitSourceInfo {
                key: "prg-granice",
                label: Wording {
                    en: "GUGiK (PRG)",
                    de: "GUGiK (PRG)",
                },
                what: Wording {
                    en: "administrative units (voivodeship, powiat, gmina)",
                    de: "Verwaltungseinheiten (Woiwodschaft, Powiat, Gmina)",
                },
                credit: PRG_CREDIT,
                note: None,
                sampled: false,
                gaps: None,
                layers: &["gminy"],
                sections: &[PermitSection::AdminUnits],
                offices: None,
            },
        }),
        Box::new(NidRegister {
            info: PermitSourceInfo {
                key: "nid-zabytki",
                label: Wording {
                    en: "NID",
                    de: "NID",
                },
                what: Wording {
                    en: "register of immovable monuments (names only, no positions)",
                    de: "Denkmalregister (nur Namen, keine Positionen)",
                },
                credit: NID_CREDIT,
                note: None,
                sampled: false,
                gaps: Some(Wording {
                    en: "archaeological sites and the Historic Monuments (Pomniki historii) are not in it",
                    de: "archäologische Stätten und die Pomniki historii fehlen darin",
                }),
                layers: &["zabytki"],
                sections: &[PermitSection::MonumentRegister],
                offices: None,
            },
            register: Register::Immovable,
        }),
        Box::new(NidRegister {
            info: PermitSourceInfo {
                key: "nid-archeologia",
                label: Wording {
                    en: "NID",
                    de: "NID",
                },
                what: Wording {
                    en: "register of archaeological monuments (names only, no positions)",
                    de: "Register der archäologischen Denkmale (nur Namen, keine Positionen)",
                },
                credit: NID_CREDIT,
                note: None,
                sampled: false,
                gaps: Some(Wording {
                    en: "sites that are only in the archaeological record (AZP) and not entered in the register are not in it",
                    de: "Fundstellen, die nur in der archäologischen Kartei (AZP) stehen und nicht im Register eingetragen sind, fehlen darin",
                }),
                layers: &["archeologia"],
                sections: &[PermitSection::ArchaeologyRegister],
                offices: None,
            },
            register: Register::Archaeological,
        }),
        Box::new(NidHistory {
            info: PermitSourceInfo {
                key: "nid-pomniki",
                label: Wording {
                    en: "NID",
                    de: "NID",
                },
                what: Wording {
                    en: "monuments of history (Pomniki historii, with their positions)",
                    de: "Denkmäler der Geschichte (Pomniki historii, mit Positionen)",
                },
                credit: NID_CREDIT,
                note: None,
                sampled: false,
                gaps: None,
                layers: &["pomniki"],
                sections: &[PermitSection::HistoryRegister],
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

/// The administrative units of the PRG: voivodeship, powiat and gmina (ADR 0181).
struct PrgAdmin {
    info: PermitSourceInfo,
}

/// The kind of a gmina as the last digit of its TERYT code says it, in front of its name.
fn gmina_kind(code: &str) -> &'static str {
    match code.chars().last() {
        Some('1') => "gmina miejska",
        Some('2') => "gmina wiejska",
        Some('3') => "gmina miejsko-wiejska",
        Some('4') => "miasto",
        Some('5') => "obszar wiejski",
        Some('8') => "dzielnica",
        Some('9') => "delegatura",
        _ => "gmina",
    }
}

/// `name` with `prefix` and a space in front, unless it begins with it already (the register
/// writes `powiat słubicki` but `lubuskie`).
fn with_prefix(prefix: &str, name: &str) -> String {
    if name.to_lowercase().starts_with(prefix) {
        name.to_string()
    } else {
        format!("{prefix} {name}")
    }
}

impl PermitSource for PrgAdmin {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        _part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let municipalities = prg_features(
            fetcher,
            "A03_Granice_gmin",
            "msGeometry,JPT_KOD_JE,JPT_NAZWA_",
            bbox,
        )?;
        if municipalities.is_empty() {
            return Ok(vec![Vec::new()]);
        }
        let counties = prg_features(
            fetcher,
            "A02_Granice_powiatow",
            "JPT_KOD_JE,JPT_NAZWA_",
            bbox,
        )?;
        let voivodeships = prg_features(
            fetcher,
            "A01_Granice_wojewodztw",
            "JPT_KOD_JE,JPT_NAZWA_",
            bbox,
        )?;
        // The name of the unit whose code is the first `len` characters of `code`.
        let name_of = |units: &[Feature], code: &str, len: usize| {
            let prefix = code.get(..len)?;
            units
                .iter()
                .find(|u| u.id == prefix)
                .map(|u| (u.id.clone(), u.kind.clone()))
        };
        let features = municipalities
            .iter()
            .filter_map(|m| {
                let geometry = prg_geometry(m)?;
                let county = name_of(&counties, &m.id, 4);
                let voivodeship = name_of(&voivodeships, &m.id, 2);
                Some(json!({
                    "type": "Feature",
                    "properties": {
                        "kod": m.id,
                        "nazwa": m.kind,
                        "powiat_kod": county.as_ref().map(|c| &c.0),
                        "powiat": county.as_ref().map(|c| &c.1),
                        "woj_kod": voivodeship.as_ref().map(|v| &v.0),
                        "woj": voivodeship.as_ref().map(|v| &v.1),
                    },
                    "geometry": geometry,
                }))
            })
            .collect();
        Ok(vec![features])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != "gminy" {
            return Vec::new();
        }
        let mut found = Vec::new();
        for feature in features {
            let Some(geometry) = geometry_of(feature) else {
                continue;
            };
            let unit = |level, code: &str, name: String| PermitFinding {
                kind: FindingKind::AdminUnit {
                    level,
                    name,
                    code: code.to_string(),
                },
                geometry: geometry.clone(),
            };
            // A unit whose name the register did not answer is left out, not guessed.
            if let (Some(code), Some(name)) =
                (property(feature, "woj_kod"), property(feature, "woj"))
            {
                found.push(unit(
                    AdminUnitLevel::Region,
                    &code,
                    with_prefix("województwo", &name),
                ));
            }
            if let (Some(code), Some(name)) =
                (property(feature, "powiat_kod"), property(feature, "powiat"))
            {
                found.push(unit(
                    AdminUnitLevel::County,
                    &code,
                    with_prefix("powiat", &name),
                ));
            }
            if let (Some(code), Some(name)) = (property(feature, "kod"), property(feature, "nazwa"))
            {
                let name = format!("{} {name}", gmina_kind(&code));
                found.push(unit(AdminUnitLevel::Municipality, &code, name));
            }
        }
        found
    }
}

/// The two registers of the NID that answer alike: the immovable monuments (ADR 0184) and the
/// archaeological ones (ADR 0190).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Register {
    Immovable,
    Archaeological,
}

impl Register {
    /// The WMS of the register.
    fn wms(self) -> &'static str {
        match self {
            Self::Immovable => NID_WMS,
            Self::Archaeological => NID_ARCHAEOLOGY_WMS,
        }
    }

    /// The layer of the WMS.
    fn layer(self) -> &'static str {
        match self {
            Self::Immovable => "Immovable_Monuments",
            Self::Archaeological => "Archaeological_Monuments",
        }
    }

    /// The layer of the source whose findings the register gives.
    fn source_layer(self) -> &'static str {
        match self {
            Self::Immovable => "zabytki",
            Self::Archaeological => "archeologia",
        }
    }
}

/// A register of monuments of the NID.
struct NidRegister {
    info: PermitSourceInfo,
    register: Register,
}

/// The `GetFeatureInfo` request that asks for every monument of `bbox`: the whole box as one
/// pixel. EPSG:4326 in WMS 1.3.0 has the latitude first.
fn nid_url(register: Register, bbox: &Rect<f64>) -> String {
    let (wms, layer) = (register.wms(), register.layer());
    format!(
        "{wms}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={layer}\
         &QUERY_LAYERS={layer}&CRS=EPSG:4326&BBOX={},{},{},{}\
         &WIDTH=1&HEIGHT=1&I=0&J=0&INFO_FORMAT=text/xml&FEATURE_COUNT={NID_COUNT}",
        bbox.min().y,
        bbox.min().x,
        bbox.max().y,
        bbox.max().x,
    )
}

/// One monument of the register: its INSPIRE id, its name and its kind (both may be blank).
type Monument = (String, String, String);

/// The monuments of a `GetFeatureInfo` answer in the order they come. The service answers with
/// status 200 also for an error, then with an exception report instead of the collection.
fn read_monuments(url: &str, body: &[u8]) -> Result<Vec<Monument>, FetchError> {
    if is_ogc_report(body) {
        return Err(report_error(url, body));
    }
    let mut reader = quick_xml::Reader::from_reader(body);
    let mut buffer = Vec::new();
    let mut depth = 0_usize;
    let mut root = false;
    let mut field: Option<String> = None;
    let mut text = String::new();
    let mut values: std::collections::BTreeMap<String, String> = Default::default();
    let mut monuments = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|e| http_error(url, format!("the answer is no XML: {e}")))?;
        match event {
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                if depth == 0 {
                    if name != "FeatureCollection" {
                        return Err(http_error(
                            url,
                            format!("the answer is no feature collection but <{name}>"),
                        ));
                    }
                    root = true;
                } else if name == "Layer" {
                    values.clear();
                } else if name == "Attribute" {
                    field = attribute(&e, "Name");
                    text.clear();
                }
                depth += 1;
            }
            Event::Empty(e) => {
                // An answer with no monument is an empty root element.
                let name = local(e.name().as_ref());
                if depth == 0 && name == "FeatureCollection" {
                    root = true;
                }
            }
            Event::Text(t) if field.is_some() => text.push_str(t.as_ref()),
            Event::CData(c) if field.is_some() => {
                text.push_str(c.as_ref());
            }
            Event::GeneralRef(r) if field.is_some() => {
                if let Some(c) = entity(r.as_ref()) {
                    text.push(c);
                }
            }
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                let name = local(e.name().as_ref());
                if name == "Attribute" {
                    if let Some(field) = field.take() {
                        // The two registers write the names of the attributes in different cases.
                        values.insert(field.to_ascii_lowercase(), text.trim().to_string());
                    }
                } else if name == "Layer" {
                    let get = |key: &str| values.get(key).cloned().unwrap_or_default();
                    let id = get("inspireid");
                    if !id.is_empty() {
                        monuments.push((id, get("sitename"), get("nationalmonumentsrecord")));
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root {
        return Err(http_error(url, "the answer is no feature collection"));
    }
    if monuments.len() >= NID_COUNT {
        return Err(http_error(
            url,
            format!("the service sent {NID_COUNT} monuments or more, the box is too large"),
        ));
    }
    Ok(monuments)
}

/// The polygon of `bbox` as GeoJSON: the position a monument without one gets.
fn rectangle(bbox: &Rect<f64>) -> Value {
    let (w, s, e, n) = (bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y);
    json!({"type": "Polygon", "coordinates": [[[w, s], [e, s], [e, n], [w, n], [w, s]]]})
}

impl PermitSource for NidRegister {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let url = nid_url(self.register, bbox);
        let found = get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| {
            read_monuments(&url, bytes)
        })?;
        // One monument of the register can come several times (an entry with several parts):
        // merged by its INSPIREID, the first name that is not blank stays.
        let mut merged: Vec<Monument> = Vec::new();
        for (id, name, kind) in found {
            match merged.iter_mut().find(|m| m.0 == id) {
                Some(known) if known.1.is_empty() => known.1 = name,
                Some(_) => {}
                None => merged.push((id, name, kind)),
            }
        }
        let rectangle = rectangle(bbox);
        let features = merged
            .into_iter()
            .map(|(id, name, kind)| {
                json!({
                    "type": "Feature",
                    "properties": {"id": id, "nazwa": name, "rodzaj": kind},
                    "geometry": rectangle,
                })
            })
            .collect();
        Ok(vec![features])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != self.register.source_layer() {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|feature| {
                let name = property(feature, "nazwa");
                Some(PermitFinding {
                    kind: match self.register {
                        Register::Immovable => FindingKind::RegisterMonument { name },
                        Register::Archaeological => FindingKind::RegisterArchaeology { name },
                    },
                    geometry: geometry_of(feature)?,
                })
            })
            .collect()
    }
}

/// The monuments of history of the NID (ADR 0191): a WFS with the true shapes.
struct NidHistory {
    info: PermitSourceInfo,
}

/// The request for every monument of history in `bbox`. The service takes the box only with the
/// CRS at its end and the northing first; without it the box is read in another way.
fn history_url(bbox: &Rect<f64>) -> String {
    let (min_n, min_e, max_n, max_e) = box_2180(bbox);
    format!(
        "{NID_HISTORY_WFS}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
         &TYPENAMES=gmgml:Monuments_of_History&OUTPUTFORMAT=application/vnd.geo%2Bjson\
         &BBOX={min_n},{min_e},{max_n},{max_e},EPSG:2180"
    )
}

/// The coordinates of a GeoJSON geometry in EPSG:2180 (easting first, as the service writes
/// them) as WGS 84 with the longitude first; `None` for a position that is no number or no
/// position.
fn history_coordinates(coordinates: &Value) -> Option<Value> {
    let list = coordinates.as_array()?;
    if list.first()?.is_number() {
        let east = list.first()?.as_f64()?;
        let north = list.get(1)?.as_f64()?;
        let p = pl1992_inverse(east, north)?;
        return Some(json!([p.lon, p.lat]));
    }
    list.iter()
        .map(history_coordinates)
        .collect::<Option<Vec<_>>>()
        .map(Value::Array)
}

/// The monuments of an answer, one feature per part: its INSPIREID, its name and its shape in
/// WGS 84. A part without an id is left out; an answer that is no feature collection, or a part
/// whose shape cannot be read, fails the whole answer, because a count would be wrong.
fn read_history(url: &str, body: &[u8]) -> Result<Vec<Value>, FetchError> {
    if is_ogc_report(body) {
        return Err(report_error(url, body));
    }
    // An empty answer carries a box of the largest numbers (`1.79769313486232e308`), which is
    // more than an f64 holds: the members are read raw, so only the ones needed are parsed.
    let members: std::collections::BTreeMap<String, &serde_json::value::RawValue> =
        serde_json::from_slice(body)
            .map_err(|e| http_error(url, format!("the answer is no JSON object: {e}")))?;
    let member =
        |key: &str| -> Option<Value> { serde_json::from_str(members.get(key)?.get()).ok() };
    let parts = member("features")
        .and_then(|features| match features {
            Value::Array(parts) => Some(parts),
            _ => None,
        })
        .filter(|_| member("type").as_ref().and_then(Value::as_str) == Some("FeatureCollection"))
        .ok_or_else(|| http_error(url, "the answer is no feature collection"))?;
    let mut out = Vec::new();
    for part in &parts {
        let Some(id) = property(part, "INSPIREID") else {
            continue;
        };
        let geometry = part
            .get("geometry")
            .and_then(|g| {
                Some(json!({
                    "type": g.get("type")?,
                    "coordinates": history_coordinates(g.get("coordinates")?)?,
                }))
            })
            .ok_or_else(|| http_error(url, format!("the monument {id} has no usable position")))?;
        let feature = json!({
            "type": "Feature",
            "properties": {"id": id, "nazwa": property(part, "SITENAME")},
            "geometry": geometry,
        });
        if geometry_of(&feature).is_none() {
            return Err(http_error(
                url,
                format!("the monument {id} has no usable position"),
            ));
        }
        out.push(feature);
    }
    Ok(out)
}

impl PermitSource for NidHistory {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let url = history_url(bbox);
        let features = get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| {
            read_history(&url, bytes)
        })?;
        Ok(vec![features])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != "pomniki" {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|feature| {
                Some(PermitFinding {
                    kind: FindingKind::RegisterHistory {
                        id: property(feature, "id")?,
                        name: property(feature, "nazwa"),
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
    const GMINY_SLUBICE: &str = include_str!("../tests/fixtures/permits/prg-gminy-slubice.xml");
    const POWIATY_SLUBICE: &str = include_str!("../tests/fixtures/permits/prg-powiaty-slubice.xml");
    const WOJ_SLUBICE: &str = include_str!("../tests/fixtures/permits/prg-woj-slubice.xml");
    const GMINY_KOSCIAN: &str = include_str!("../tests/fixtures/permits/prg-gminy-koscian.xml");
    const POWIATY_KOSCIAN: &str = include_str!("../tests/fixtures/permits/prg-powiaty-koscian.xml");
    const WOJ_KOSCIAN: &str = include_str!("../tests/fixtures/permits/prg-woj-koscian.xml");

    fn prg_admin() -> PrgAdmin {
        let source = sources()
            .into_iter()
            .nth(2)
            .expect("the units are the third");
        PrgAdmin {
            info: source.info().clone(),
        }
    }

    /// A fetcher that answers the three layers of units for `bbox` with the fixtures.
    fn admin_fetcher(bbox: &Rect<f64>, answers: [&str; 3]) -> MapFetcher {
        let fetcher = MapFetcher::default();
        let props = "JPT_KOD_JE,JPT_NAZWA_";
        fetcher.insert(
            &prg_url(
                "ms:A03_Granice_gmin",
                "msGeometry,JPT_KOD_JE,JPT_NAZWA_",
                bbox,
            ),
            answers[0].as_bytes().to_vec(),
        );
        fetcher.insert(
            &prg_url("ms:A02_Granice_powiatow", props, bbox),
            answers[1].as_bytes().to_vec(),
        );
        fetcher.insert(
            &prg_url("ms:A01_Granice_wojewodztw", props, bbox),
            answers[2].as_bytes().to_vec(),
        );
        fetcher
    }

    /// The units as (level, name, code) of the findings of a fetch.
    fn units(source: &PrgAdmin, lists: &[Vec<Value>]) -> Vec<(AdminUnitLevel, String, String)> {
        source
            .findings("gminy", &lists[0])
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::AdminUnit { level, name, code } => (level, name, code),
                other => panic!("{other:?}"),
            })
            .collect()
    }

    #[test]
    fn a_map_area_in_slubice_names_voivodeship_powiat_and_gmina() {
        let bbox = Rect::new(
            Coord {
                x: 14.558,
                y: 52.349,
            },
            Coord {
                x: 14.562,
                y: 52.351,
            },
        );
        let fetcher = admin_fetcher(&bbox, [GMINY_SLUBICE, POWIATY_SLUBICE, WOJ_SLUBICE]);
        let lists = prg_admin()
            .fetch(&fetcher, &bbox, Path::new("unused"))
            .unwrap();
        assert_eq!(fetcher.calls.get(), 3);
        assert_eq!(
            units(&prg_admin(), &lists),
            [
                (
                    AdminUnitLevel::Region,
                    "województwo lubuskie".into(),
                    "08".into()
                ),
                (
                    AdminUnitLevel::County,
                    "powiat słubicki".into(),
                    "0805".into()
                ),
                (
                    AdminUnitLevel::Municipality,
                    "gmina miejsko-wiejska Słubice".into(),
                    "0805053".into()
                ),
            ]
        );
        // The sample polygon lies around Słubice: positions are longitude, latitude.
        let found = prg_admin().findings("gminy", &lists[0]);
        assert!(geo::Contains::contains(
            &found[2].geometry,
            &Point::new(14.56, 52.35)
        ));
    }

    /// A map area on the border of two gminas of one powiat: two municipalities, and the county
    /// and voivodeship appear once per municipality for the notes to sum.
    #[test]
    fn a_map_area_that_straddles_two_gminas_gives_both() {
        let bbox = Rect::new(Coord { x: 16.64, y: 52.07 }, Coord { x: 16.66, y: 52.09 });
        let fetcher = admin_fetcher(&bbox, [GMINY_KOSCIAN, POWIATY_KOSCIAN, WOJ_KOSCIAN]);
        let lists = prg_admin()
            .fetch(&fetcher, &bbox, Path::new("unused"))
            .unwrap();
        let found = units(&prg_admin(), &lists);
        assert_eq!(found.len(), 6);
        let names: Vec<&str> = found
            .iter()
            .filter(|(level, ..)| *level == AdminUnitLevel::Municipality)
            .map(|(_, name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["gmina wiejska Kościan", "gmina miejska Kościan"]);
        assert!(
            found
                .iter()
                .any(|(_, n, c)| n == "powiat kościański" && c == "3011")
        );
        assert!(
            found
                .iter()
                .any(|(_, n, c)| n == "województwo wielkopolskie" && c == "30")
        );
    }

    #[test]
    fn without_a_gmina_the_names_of_the_upper_levels_are_not_asked() {
        let bbox = Rect::new(
            Coord {
                x: 14.558,
                y: 52.349,
            },
            Coord {
                x: 14.562,
                y: 52.351,
            },
        );
        let empty = GMINY_SLUBICE
            .split("<wfs:member>")
            .next()
            .unwrap()
            .to_string()
            + "</wfs:FeatureCollection>";
        let fetcher = admin_fetcher(&bbox, [&empty, "", ""]);
        let lists = prg_admin()
            .fetch(&fetcher, &bbox, Path::new("unused"))
            .unwrap();
        assert_eq!(lists, vec![Vec::<Value>::new()]);
        assert_eq!(fetcher.calls.get(), 1);
    }

    /// A level whose name the register did not answer is left out; the others stay.
    #[test]
    fn a_missing_name_of_a_level_leaves_that_level_out() {
        let bbox = Rect::new(
            Coord {
                x: 14.558,
                y: 52.349,
            },
            Coord {
                x: 14.562,
                y: 52.351,
            },
        );
        let none = POWIATY_SLUBICE
            .split("<wfs:member>")
            .next()
            .unwrap()
            .to_string()
            + "</wfs:FeatureCollection>";
        let fetcher = admin_fetcher(&bbox, [GMINY_SLUBICE, &none, WOJ_SLUBICE]);
        let lists = prg_admin()
            .fetch(&fetcher, &bbox, Path::new("unused"))
            .unwrap();
        let levels: Vec<_> = units(&prg_admin(), &lists).iter().map(|u| u.0).collect();
        assert_eq!(
            levels,
            [AdminUnitLevel::Region, AdminUnitLevel::Municipality]
        );
    }

    #[test]
    fn a_failing_service_is_an_error_of_the_source() {
        let bbox = Rect::new(
            Coord {
                x: 14.558,
                y: 52.349,
            },
            Coord {
                x: 14.562,
                y: 52.351,
            },
        );
        let fetcher = MapFetcher::default();
        assert!(
            prg_admin()
                .fetch(&fetcher, &bbox, Path::new("unused"))
                .is_err()
        );
        // The second request fails: no half answer.
        let fetcher = admin_fetcher(
            &bbox,
            [GMINY_SLUBICE, "<ows:ExceptionReport/>", WOJ_SLUBICE],
        );
        assert!(
            prg_admin()
                .fetch(&fetcher, &bbox, Path::new("unused"))
                .is_err()
        );
    }

    #[test]
    fn the_kind_of_a_gmina_follows_the_last_digit_of_its_code() {
        assert_eq!(gmina_kind("3011011"), "gmina miejska");
        assert_eq!(gmina_kind("3011032"), "gmina wiejska");
        assert_eq!(gmina_kind("0805053"), "gmina miejsko-wiejska");
        assert_eq!(gmina_kind("1465011"), "gmina miejska");
        assert_eq!(gmina_kind("1465014"), "miasto");
        assert_eq!(gmina_kind("1465015"), "obszar wiejski");
        assert_eq!(gmina_kind("1465018"), "dzielnica");
        assert_eq!(gmina_kind("1465019"), "delegatura");
        assert_eq!(gmina_kind("1465016"), "gmina");
        assert_eq!(gmina_kind("x"), "gmina");
        assert_eq!(with_prefix("powiat", "powiat Poznań"), "powiat Poznań");
        assert_eq!(
            with_prefix("województwo", "lubuskie"),
            "województwo lubuskie"
        );
    }

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
        assert_eq!(infos[2].credit, "© GUGiK, geoportal.gov.pl");
        assert_eq!(infos[2].sections, [PermitSection::AdminUnits]);
        assert_eq!(infos[2].layers, ["gminy"]);
    }

    const NID_SLUBICE: &str = include_str!("../tests/fixtures/permits/nid-slubice.xml");
    const NID_KRAKOW: &str = include_str!("../tests/fixtures/permits/nid-krakow.xml");
    const NID_EMPTY: &str = include_str!("../tests/fixtures/permits/nid-empty.xml");

    const NID_BISKUPIN: &str =
        include_str!("../tests/fixtures/permits/nid-archeologia-biskupin.xml");

    fn register(key: &str, register: Register) -> NidRegister {
        let source = sources()
            .into_iter()
            .find(|s| s.info().key == key)
            .expect("the register is a source");
        NidRegister {
            info: source.info().clone(),
            register,
        }
    }

    fn nid() -> NidRegister {
        register("nid-zabytki", Register::Immovable)
    }

    fn archaeology() -> NidRegister {
        register("nid-archeologia", Register::Archaeological)
    }

    fn nid_box() -> Rect<f64> {
        Rect::new(
            Coord {
                x: 19.93,
                y: 50.055,
            },
            Coord {
                x: 19.96,
                y: 50.073,
            },
        )
    }

    fn nid_answer(bbox: &Rect<f64>, body: &str) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &nid_url(Register::Immovable, bbox),
            body.as_bytes().to_vec(),
        );
        fetcher
    }

    fn archaeology_answer(bbox: &Rect<f64>, body: &str) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &nid_url(Register::Archaeological, bbox),
            body.as_bytes().to_vec(),
        );
        fetcher
    }

    #[test]
    fn the_register_asks_the_whole_box_as_one_pixel_with_the_latitude_first() {
        let url = nid_url(Register::Immovable, &nid_box());
        assert!(url.contains("BBOX=50.055,19.93,50.073,19.96"), "{url}");
        assert!(url.contains("WIDTH=1&HEIGHT=1&I=0&J=0"), "{url}");
        assert!(
            url.starts_with("https://usluga.zabytek.gov.pl/INSPIRE_IMD/service.svc/get?"),
            "{url}"
        );
        assert!(
            url.contains("LAYERS=Immovable_Monuments&QUERY_LAYERS=Immovable_Monuments&"),
            "{url}"
        );
    }

    #[test]
    fn the_archaeological_register_asks_its_own_service_and_layer() {
        let url = nid_url(Register::Archaeological, &nid_box());
        assert!(
            url.starts_with("https://usluga.zabytek.gov.pl/INSPIRE_AMD/service.svc/get?"),
            "{url}"
        );
        assert!(
            url.contains("LAYERS=Archaeological_Monuments&QUERY_LAYERS=Archaeological_Monuments&"),
            "{url}"
        );
        assert!(url.contains("BBOX=50.055,19.93,50.073,19.96"), "{url}");
        assert!(url.ends_with("FEATURE_COUNT=5000"), "{url}");
        assert!(url.starts_with(&format!("https://{NID_HOST}/")), "{url}");
    }

    #[test]
    fn the_archaeological_register_counts_a_site_once_and_keeps_every_name() {
        let bbox = Rect::new(Coord { x: 17.74, y: 52.79 }, Coord { x: 17.752, y: 52.8 });
        let fetcher = archaeology_answer(&bbox, NID_BISKUPIN);
        let lists = archaeology()
            .fetch(&fetcher, &bbox, Path::new("unused-archaeology-1"))
            .unwrap();
        // Nine entries, one id twice: eight sites.
        assert_eq!(lists[0].len(), 8);
        let names: Vec<Option<String>> = archaeology()
            .findings("archeologia", &lists[0])
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::RegisterArchaeology { name } => name,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(names.len(), 8);
        assert_eq!(
            names
                .iter()
                .filter(|n| n.as_deref() == Some("Biskupin"))
                .count(),
            2
        );
        assert!(names.contains(&Some("Biskupin, st. 15,15a".to_string())));
        // The register gives no position: the rectangle of the box stands in.
        assert!(lists[0].iter().all(|f| f["geometry"]["type"] == "Polygon"));
        // The kind of an entry stays: id 1069 is a settlement.
        let kind = |id: &str| {
            let hit = lists[0]
                .iter()
                .find(|f| f["properties"]["id"] == format!("PL.1.9.ZIPOZ.NID_A_04_AR.{id}"));
            hit.map(|f| f["properties"]["rodzaj"].clone())
        };
        assert_eq!(kind("1069"), Some(json!("settlement")));
        assert_eq!(kind("0"), None);
    }

    #[test]
    fn each_register_reads_only_its_own_layer() {
        let bbox = nid_box();
        let fetcher = nid_answer(&bbox, NID_KRAKOW);
        let lists = nid()
            .fetch(&fetcher, &bbox, Path::new("unused-archaeology-2"))
            .unwrap();
        assert!(archaeology().findings("zabytki", &lists[0]).is_empty());
        assert!(nid().findings("archeologia", &lists[0]).is_empty());
        assert_eq!(nid().findings("zabytki", &lists[0]).len(), 8);
        // A feature without a position gives no finding.
        let bare = [json!({"properties": {"nazwa": "x"}})];
        assert!(archaeology().findings("archeologia", &bare).is_empty());
    }

    #[test]
    fn the_archaeological_register_answers_empty_and_fails_like_the_other() {
        let bbox = nid_box();
        let fetcher = archaeology_answer(&bbox, NID_EMPTY);
        let lists = archaeology()
            .fetch(&fetcher, &bbox, Path::new("unused-archaeology-3"))
            .unwrap();
        assert!(lists[0].is_empty());
        let report = r#"<?xml version="1.0"?><ServiceExceptionReport version="1.3.0"><ServiceException code="LayerNotDefined">no such layer</ServiceException></ServiceExceptionReport>"#;
        let fetcher = archaeology_answer(&bbox, report);
        assert!(
            archaeology()
                .fetch(&fetcher, &bbox, Path::new("unused-archaeology-4"))
                .is_err()
        );
        // The immovable register's URL is not served by this fetcher: nothing to read.
        let fetcher = nid_answer(&bbox, NID_KRAKOW);
        assert!(
            archaeology()
                .fetch(&fetcher, &bbox, Path::new("unused-archaeology-5"))
                .is_err()
        );
    }

    #[test]
    fn the_two_registers_are_two_sources_with_their_own_sections() {
        let infos: Vec<_> = sources().into_iter().map(|s| s.info().clone()).collect();
        let nid = infos.iter().find(|i| i.key == "nid-zabytki").unwrap();
        let arch = infos.iter().find(|i| i.key == "nid-archeologia").unwrap();
        assert_eq!(nid.sections, [PermitSection::MonumentRegister]);
        assert_eq!(arch.sections, [PermitSection::ArchaeologyRegister]);
        assert_eq!(arch.layers, ["archeologia"]);
        assert_eq!(arch.credit, "© Narodowy Instytut Dziedzictwa (CC BY 4.0)");
        assert!(!arch.sampled);
    }

    #[test]
    fn a_monument_that_comes_twice_counts_once_and_names_stay() {
        let bbox = nid_box();
        let fetcher = nid_answer(&bbox, NID_KRAKOW);
        let lists = nid()
            .fetch(&fetcher, &bbox, Path::new("unused-nid-1"))
            .unwrap();
        assert_eq!(lists[0].len(), 8);
        let names: Vec<Option<String>> = nid()
            .findings("zabytki", &lists[0])
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::RegisterMonument { name } => name,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(names.len(), 8);
        assert_eq!(
            names
                .iter()
                .filter(|n| n.as_deref() == Some("działka"))
                .count(),
            2
        );
        assert_eq!(
            names
                .iter()
                .filter(|n| n.as_deref() == Some("kamienica, ob. muzeum farmacji"))
                .count(),
            1
        );
    }

    #[test]
    fn the_register_in_slubice_has_eleven_monuments_in_seventeen_entries() {
        let bbox = Rect::new(Coord { x: 14.55, y: 52.34 }, Coord { x: 14.57, y: 52.36 });
        let fetcher = nid_answer(&bbox, NID_SLUBICE);
        let lists = nid()
            .fetch(&fetcher, &bbox, Path::new("unused-nid-2"))
            .unwrap();
        assert_eq!(lists[0].len(), 11);
    }

    /// A register answer with `layers` as (id, name) entries; no name is a blank one.
    fn nid_xml(layers: &[(&str, Option<&str>)]) -> String {
        let members: String = layers
            .iter()
            .map(|(id, name)| {
                format!(
                    r#"<gml:featureMember><Layer Name="Immovable_Monuments"><Attribute Name="INSPIREID">{id}</Attribute><Attribute Name="SITENAME">{}</Attribute></Layer></gml:featureMember>"#,
                    name.unwrap_or_default()
                )
            })
            .collect();
        format!(
            r#"<?xml version="1.0"?><FeatureCollection xmlns="http://www.intergraph.com/geomedia/gml" xmlns:gml="http://www.opengis.net/gml">{members}</FeatureCollection>"#
        )
    }

    #[test]
    fn every_entry_of_the_answer_is_read_once() {
        let url = "https://example.test/nid";
        let xml = nid_xml(&[("a", Some("one")), ("b", Some("two"))]);
        let found = read_monuments(url, xml.as_bytes()).unwrap();
        assert_eq!(
            found,
            [
                ("a".to_string(), "one".to_string(), String::new()),
                ("b".to_string(), "two".to_string(), String::new()),
            ]
        );
        // A single entry is one monument, not one more for each element that closes after it.
        let xml = nid_xml(&[("a", Some("one"))]);
        assert_eq!(read_monuments(url, xml.as_bytes()).unwrap().len(), 1);
    }

    #[test]
    fn an_empty_element_of_another_name_is_no_feature_collection() {
        let url = "https://example.test/nid";
        assert!(read_monuments(url, b"<Other/>").is_err());
        assert!(
            read_monuments(url, b"<FeatureCollection/>")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_name_in_a_cdata_section_or_with_an_entity_is_read_whole() {
        let url = "https://example.test/nid";
        let xml = r#"<FeatureCollection><Layer><Attribute Name="INSPIREID">a</Attribute><Attribute Name="SITENAME"><![CDATA[Dwór <A>]]></Attribute></Layer><Layer><Attribute Name="INSPIREID">b</Attribute><Attribute Name="SITENAME">Młyn &amp; spichlerz</Attribute></Layer></FeatureCollection>"#;
        let names: Vec<String> = read_monuments(url, xml.as_bytes())
            .unwrap()
            .into_iter()
            .map(|m| m.1)
            .collect();
        assert_eq!(names, ["Dwór <A>", "Młyn & spichlerz"]);
    }

    #[test]
    fn an_entry_that_comes_twice_keeps_its_first_name_that_is_not_blank() {
        let bbox = nid_box();
        let fetch = |layers: &[(&str, Option<&str>)], part: &str| {
            let fetcher = nid_answer(&bbox, &nid_xml(layers));
            let lists = nid().fetch(&fetcher, &bbox, Path::new(part)).unwrap();
            lists[0]
                .iter()
                .map(|f| f["properties"]["nazwa"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        // A name that is there stays; a blank one gives way to the next.
        assert_eq!(
            fetch(
                &[("a", Some("first")), ("a", Some("second"))],
                "unused-nid-6"
            ),
            ["first"]
        );
        assert_eq!(
            fetch(&[("a", None), ("a", Some("second"))], "unused-nid-7"),
            ["second"]
        );
        assert_eq!(
            fetch(&[("a", Some("first")), ("a", None)], "unused-nid-8"),
            ["first"]
        );
    }

    #[test]
    fn an_empty_answer_is_no_monument_and_no_failure() {
        let bbox = nid_box();
        let fetcher = nid_answer(&bbox, NID_EMPTY);
        let lists = nid()
            .fetch(&fetcher, &bbox, Path::new("unused-nid-3"))
            .unwrap();
        assert!(lists[0].is_empty());
    }

    #[test]
    fn an_error_in_the_body_of_a_200_answer_is_a_failure() {
        let bbox = nid_box();
        let report = r#"<?xml version="1.0"?><ServiceExceptionReport version="1.3.0"><ServiceException code="LayerNotDefined">no such layer</ServiceException></ServiceExceptionReport>"#;
        let fetcher = nid_answer(&bbox, report);
        assert!(
            nid()
                .fetch(&fetcher, &bbox, Path::new("unused-nid-4"))
                .is_err()
        );
        let fetcher = nid_answer(&bbox, "<html>nope</html>");
        assert!(
            nid()
                .fetch(&fetcher, &bbox, Path::new("unused-nid-5"))
                .is_err()
        );
    }
    const HISTORY_KRAKOW: &str = include_str!("../tests/fixtures/permits/nid-pomniki-krakow.json");
    const HISTORY_EMPTY: &str = include_str!("../tests/fixtures/permits/nid-pomniki-empty.json");

    fn history() -> NidHistory {
        let source = sources()
            .into_iter()
            .find(|s| s.info().key == "nid-pomniki")
            .expect("the monuments of history are a source");
        NidHistory {
            info: source.info().clone(),
        }
    }

    /// The box of the Kraków fixture: Kopiec Kościuszki.
    fn history_box() -> Rect<f64> {
        Rect::new(Coord { x: 19.89, y: 50.05 }, Coord { x: 19.91, y: 50.07 })
    }

    fn history_answer(body: &str) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert(&history_url(&history_box()), body.as_bytes().to_vec());
        fetcher
    }

    fn history_fetch(body: &str, part: &str) -> Result<Vec<Value>, FetchError> {
        let fetcher = history_answer(body);
        history()
            .fetch(&fetcher, &history_box(), Path::new(part))
            .map(|mut lists| lists.remove(0))
    }

    /// A feature as the service writes it, with `coordinates` in EPSG:2180, easting first.
    fn history_part(id: &str, name: Option<&str>, geometry: &str) -> String {
        let name = name.map_or(String::new(), |n| format!(r#", "SITENAME": "{n}""#));
        format!(
            r#"{{"type": "Feature", "id": "x", "properties": {{"INSPIREID": "{id}"{name}}}, "geometry": {geometry}}}"#
        )
    }

    fn history_collection(parts: &[String]) -> String {
        format!(
            r#"{{"type": "FeatureCollection", "crs": "EPSG:2180", "features": [{}]}}"#,
            parts.join(",")
        )
    }

    #[test]
    fn the_monuments_of_history_ask_the_box_in_2180_with_the_northing_first() {
        let url = history_url(&history_box());
        // The corners of the box in EPSG:2180 (cs2cs): the northings 242905 and 245146 come
        // first and third, the eastings 563671 and 565130 second and fourth.
        assert_eq!(
            url,
            "https://usluga.zabytek.gov.pl/INSPIRE_WFS_MHD/service.svc/get?SERVICE=WFS\
             &VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=gmgml:Monuments_of_History\
             &OUTPUTFORMAT=application/vnd.geo%2Bjson\
             &BBOX=242905,563671,245146,565130,EPSG:2180"
        );
        assert!(url.starts_with(&format!("https://{NID_HOST}/")), "{url}");
    }

    #[test]
    fn the_monuments_of_history_come_with_their_positions_in_wgs_84() {
        let features = history_fetch(HISTORY_KRAKOW, "unused-history-1").unwrap();
        // Two parts of one monument; each keeps its id and name.
        assert_eq!(features.len(), 2);
        for f in &features {
            assert_eq!(
                f["properties"]["id"], "PL.1.9.ZIPOZ.NID_N_12_PH.15423",
                "{f}"
            );
            assert_eq!(
                f["properties"]["nazwa"],
                "Kraków - Kopiec Kościuszki z otoczeniem"
            );
            assert_eq!(f["geometry"]["type"], "Polygon");
        }
        // The first corner is 563837.95 E, 243217.53 N: longitude first, in the box.
        let corner = &features[0]["geometry"]["coordinates"][0][0];
        let (lon, lat) = (corner[0].as_f64().unwrap(), corner[1].as_f64().unwrap());
        assert!((19.89..19.91).contains(&lon), "{lon}");
        assert!((50.05..50.07).contains(&lat), "{lat}");
        let findings = history().findings("pomniki", &features);
        assert_eq!(findings.len(), 2);
        for f in &findings {
            assert_eq!(
                f.kind,
                FindingKind::RegisterHistory {
                    id: "PL.1.9.ZIPOZ.NID_N_12_PH.15423".into(),
                    name: Some("Kraków - Kopiec Kościuszki z otoczeniem".into()),
                }
            );
            assert!(matches!(f.geometry, Geometry::Polygon(_)));
        }
    }

    #[test]
    fn no_monument_of_history_is_no_failure() {
        let features = history_fetch(HISTORY_EMPTY, "unused-history-2").unwrap();
        assert!(features.is_empty());
        assert!(history().findings("pomniki", &features).is_empty());
    }

    #[test]
    fn every_shape_of_a_monument_of_history_is_read() {
        let polygon = r#"{"type": "Polygon", "coordinates": [[[563837.9, 243217.5], [563845.7, 243219.3], [563848.9, 243220.3], [563837.9, 243217.5]]]}"#;
        let multi = format!(
            r#"{{"type": "MultiPolygon", "coordinates": [{}]}}"#,
            &polygon[polygon.find("[[[").unwrap()..polygon.len() - 1]
        );
        let point = r#"{"type": "Point", "coordinates": [563837.9, 243217.5]}"#;
        let line = r#"{"type": "LineString", "coordinates": [[563837.9, 243217.5], [563845.7, 243219.3]]}"#;
        let multi_line = r#"{"type": "MultiLineString", "coordinates": [[[563837.9, 243217.5], [563845.7, 243219.3]]]}"#;
        let multi_point = r#"{"type": "MultiPoint", "coordinates": [[563837.9, 243217.5]]}"#;
        let body = history_collection(&[
            history_part("a", Some("A"), polygon),
            history_part("b", Some("B"), &multi),
            history_part("c", Some("C"), point),
            history_part("d", Some("D"), line),
            history_part("e", Some("E"), multi_line),
            history_part("f", Some("F"), multi_point),
        ]);
        let features = history_fetch(&body, "unused-history-3").unwrap();
        let kinds: Vec<&str> = features
            .iter()
            .map(|f| f["geometry"]["type"].as_str().unwrap())
            .collect();
        assert_eq!(
            kinds,
            [
                "Polygon",
                "MultiPolygon",
                "Point",
                "LineString",
                "MultiLineString",
                "MultiPoint"
            ]
        );
        let findings = history().findings("pomniki", &features);
        assert_eq!(findings.len(), 6);
        let point = findings
            .iter()
            .find_map(|f| match &f.geometry {
                Geometry::Point(p) => Some(*p),
                _ => None,
            })
            .unwrap();
        // 563837.9 E, 243217.5 N is 19.89201 E, 50.05279 N (cs2cs).
        assert!((point.x() - 19.89201).abs() < 0.0001, "{point:?}");
        assert!((point.y() - 50.05279).abs() < 0.0001, "{point:?}");
        // A third value (a height) does not matter.
        let z = history_part(
            "z",
            None,
            r#"{"type": "Point", "coordinates": [563837.9, 243217.5, 200.0]}"#,
        );
        assert_eq!(
            history_fetch(&history_collection(&[z]), "unused-history-4")
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn a_part_without_an_id_is_left_out_and_a_blank_name_is_none() {
        let point = r#"{"type": "Point", "coordinates": [563837.9, 243217.5]}"#;
        let no_id = format!(
            r#"{{"type": "Feature", "properties": {{"SITENAME": "ohne"}}, "geometry": {point}}}"#
        );
        let blank = history_part("b", Some("  "), point);
        let body = history_collection(&[no_id, blank]);
        let features = history_fetch(&body, "unused-history-5").unwrap();
        assert_eq!(features.len(), 1);
        let findings = history().findings("pomniki", &features);
        assert_eq!(
            findings[0].kind,
            FindingKind::RegisterHistory {
                id: "b".into(),
                name: None
            }
        );
        // Another layer name gives nothing; a feature without an id gives no finding.
        assert!(history().findings("zabytki", &features).is_empty());
        assert!(
            history()
                .findings(
                    "pomniki",
                    &[json!({"properties": {}, "geometry": features[0]["geometry"]})]
                )
                .is_empty()
        );
    }

    #[test]
    fn a_broken_answer_fails_the_source() {
        let point = r#"{"type": "Point", "coordinates": [563837.9, 243217.5]}"#;
        let report = r#"<?xml version="1.0"?><ExceptionReport><Exception exceptionCode="InvalidParameterValue"/></ExceptionReport>"#;
        let bad: [(&str, String); 10] = [
            ("report", report.to_string()),
            ("html", "<html>nope</html>".to_string()),
            ("empty object", "{}".to_string()),
            (
                "no features",
                r#"{"type": "FeatureCollection"}"#.to_string(),
            ),
            (
                "other type",
                r#"{"type": "Feature", "features": []}"#.to_string(),
            ),
            (
                "features no list",
                r#"{"type": "FeatureCollection", "features": {}}"#.to_string(),
            ),
            (
                "no geometry",
                history_collection(&[
                    r#"{"type": "Feature", "properties": {"INSPIREID": "x"}}"#.to_string()
                ]),
            ),
            (
                "text coordinates",
                history_collection(&[history_part(
                    "x",
                    None,
                    r#"{"type": "Point", "coordinates": ["a", "b"]}"#,
                )]),
            ),
            (
                "one number",
                history_collection(&[history_part(
                    "x",
                    None,
                    r#"{"type": "Point", "coordinates": [563837.9]}"#,
                )]),
            ),
            (
                "unknown type",
                history_collection(&[history_part(
                    "x",
                    None,
                    r#"{"type": "Circle", "coordinates": [563837.9, 243217.5]}"#,
                )]),
            ),
        ];
        for (n, (what, body)) in bad.iter().enumerate() {
            let part = format!("unused-history-bad-{n}");
            assert!(history_fetch(body, &part).is_err(), "{what}");
        }
        // One bad part among good ones fails the whole answer: a count would be wrong.
        let body = history_collection(&[
            history_part("a", None, point),
            history_part("x", None, r#"{"type": "Point", "coordinates": ["a", 1]}"#),
        ]);
        assert!(history_fetch(&body, "unused-history-6").is_err());
    }
}
