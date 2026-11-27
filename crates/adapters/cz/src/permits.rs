// SPDX-License-Identifier: AGPL-3.0-or-later

//! The sources of the permit notes of Czechia (ADR 0201): the protected areas and objects of the
//! Nature Conservation Agency (AOPK ČR), the administrative units of RÚIAN and the monuments of
//! the National Heritage Institute (NPÚ). All three are ArcGIS REST map services that answer
//! `query` as GeoJSON in WGS 84 (`outSR=4326`, longitude first) without a login.
//!
//! The administrative units (ADR 0201) come from the layers of RÚIAN: the fetch asks the
//! municipalities (obce) of the map box with their polygons and, without geometry, the
//! municipalities with an authorised municipal office (POU), the municipalities with extended
//! powers (ORP), the districts (okresy) and the regions (kraje) of the box. A municipality gives
//! four findings with its own polygon (region, district, ORP, municipality), so the notes sum
//! the shares as they do for Poland. The chain is `obec.pou` to `pou.orp` to the ORP, `obec.okres`
//! to the district and `okres.vusc` to the region; the capital has no district and its ORP names
//! its region.

use std::path::Path;

use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::fetch::{Fetcher, get_limited_with_retries, one_line};
use freemapper_adapter_api::info::Country;
use freemapper_adapter_api::permits::{
    AdminUnitLevel, FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo,
    ProtectedObjectKind, Wording,
};
use geo_types::{Geometry, Rect};
use serde_json::{Value, json};

/// The host of the AOPK map services.
pub const AOPK_HOST: &str = "gis.nature.cz";

/// The host of the NPÚ map services.
pub const NPU_HOST: &str = "geoportal.npu.cz";

/// The credit of the AOPK: the line its services print (`copyrightText`); its open data carry the
/// licence CC BY 4.0 (metadata of the services at geoportal.gov.cz, ADR 0201).
pub const AOPK_CREDIT: &str = "© AOPK ČR (CC BY 4.0)";

/// The credit of ČÚZK, as on the map ([`crate::cesko::CREDIT`]).
pub const RUIAN_CREDIT: &str = "© ČÚZK (CC BY 4.0)";

/// The credit of the NPÚ: its INSPIRE service of protected sites names the licence CC BY-SA 4.0
/// in its metadata record (ADR 0201).
pub const NPU_CREDIT: &str = "© Národní památkový ústav (CC BY-SA 4.0)";

/// The map services of the AOPK.
const AOPK_PROTECTION: &str =
    "https://gis.nature.cz/arcgis/rest/services/UzemniOchrana/ChranUzemi/MapServer";
const AOPK_NATURA: &str =
    "https://gis.nature.cz/arcgis/rest/services/UzemniOchrana/Natura2000/MapServer";
const AOPK_TREES: &str =
    "https://gis.nature.cz/arcgis/rest/services/PamatneStromy/PamatneStromy/MapServer";

/// RÚIAN, as [`crate::cadastre::RUIAN`].
const RUIAN: &str = crate::cadastre::RUIAN;

/// The INSPIRE service of the protected sites of the NPÚ (layer 0: monuments and areas).
const NPU_SITES: &str =
    "https://geoportal.npu.cz/arcgis/rest/services/INSPIRE/ProtectedSites/MapServer";

/// Features asked for per page; the services answer 1000 to 2000.
const PAGE: usize = 1000;

/// Pages of one layer read at most (20 000 features); more is a box far larger than a map area.
const MAX_PAGES: usize = 20;

/// The largest answer read: 50 MiB.
const MAX_BYTES: u64 = 52_428_800;

/// A line of a polygon is simplified to about 5 m (degrees): shares of a map area do not need
/// more, and the boundary of a landscape protection area is several hundred kilobytes.
const SIMPLIFY: &str = "0.00005";

/// How an AOPK layer is read: the map service, the layer, the attributes asked for and the way
/// the kind and the name come out.
#[derive(Clone, Copy)]
struct AopkLayer {
    key: &'static str,
    service: &'static str,
    layer: u32,
    fields: &'static str,
    kind: KindBy,
}

/// Where the kind of an object comes from.
#[derive(Clone, Copy)]
enum KindBy {
    /// The attribute `KAT` of a small-area protected area: `NPR`, `NPP`, `PR`, `PP`.
    SmallArea,
    /// The attribute `KAT` of a large-area protected area: `NP`, `CHKO`, and `OP` for the
    /// protective zone of a national park.
    LargeArea,
    /// The protective zone of a reserve or natural monument; its name carries the category.
    ProtectiveZone,
    /// A zone of a national park or a protected landscape area; its name carries the zone.
    Zone,
    /// One fixed kind.
    Fixed(ProtectedObjectKind),
}

/// The layers of the AOPK in the order of the source.
const AOPK_LAYERS: [AopkLayer; 8] = [
    AopkLayer {
        key: "maloplosna",
        service: AOPK_PROTECTION,
        layer: 0,
        fields: "KAT,NAZEV",
        kind: KindBy::SmallArea,
    },
    AopkLayer {
        key: "ochranna-pasma",
        service: AOPK_PROTECTION,
        layer: 1,
        fields: "KAT_ZCHU,NAZEV",
        kind: KindBy::ProtectiveZone,
    },
    AopkLayer {
        key: "velkoplosna",
        service: AOPK_PROTECTION,
        layer: 3,
        fields: "KAT,NAZEV",
        kind: KindBy::LargeArea,
    },
    AopkLayer {
        key: "zony",
        service: AOPK_PROTECTION,
        layer: 2,
        fields: "KAT,NAZEV,ZONA",
        kind: KindBy::Zone,
    },
    AopkLayer {
        key: "natura-evl",
        service: AOPK_NATURA,
        layer: 1,
        fields: "NAZEV,SITECODE",
        kind: KindBy::Fixed(ProtectedObjectKind::Natura2000HabitatsCz),
    },
    AopkLayer {
        key: "natura-po",
        service: AOPK_NATURA,
        layer: 0,
        fields: "NAZEV,SITECODE",
        kind: KindBy::Fixed(ProtectedObjectKind::Natura2000BirdsCz),
    },
    AopkLayer {
        key: "pamatne-stromy",
        service: AOPK_TREES,
        layer: 1,
        fields: "NAZEV,DRUH",
        kind: KindBy::Fixed(ProtectedObjectKind::MemorialTree),
    },
    AopkLayer {
        key: "pamatne-stromy-skupiny",
        service: AOPK_TREES,
        layer: 3,
        fields: "NAZEV",
        kind: KindBy::Fixed(ProtectedObjectKind::MemorialTree),
    },
];

/// The keys of [`AOPK_LAYERS`].
const AOPK_KEYS: [&str; 8] = [
    "maloplosna",
    "ochranna-pasma",
    "velkoplosna",
    "zony",
    "natura-evl",
    "natura-po",
    "pamatne-stromy",
    "pamatne-stromy-skupiny",
];

/// The sources of Czechia.
pub fn sources() -> Vec<Box<dyn PermitSource>> {
    vec![
        Box::new(Aopk {
            info: PermitSourceInfo {
                key: "aopk",
                label: Wording {
                    en: "AOPK ČR",
                    de: "AOPK ČR",
                },
                what: Wording {
                    en: "protected areas and objects of nature conservation",
                    de: "Schutzgebiete und Schutzobjekte des Naturschutzes",
                },
                credit: AOPK_CREDIT,
                note: None,
                sampled: false,
                gaps: Some(Wording {
                    en: "the natural parks of the regions (přírodní parky) are not in it",
                    de: "die Naturparks der Kreise (přírodní parky) fehlen darin",
                }),
                layers: &AOPK_KEYS,
                sections: &[PermitSection::ProtectedObjects],
                offices: None,
            },
        }),
        Box::new(Ruian {
            info: PermitSourceInfo {
                key: "ruian-hranice",
                label: Wording {
                    en: "ČÚZK (RÚIAN)",
                    de: "ČÚZK (RÚIAN)",
                },
                what: Wording {
                    en: "administrative units (region, district, municipality with extended powers, municipality)",
                    de: "Verwaltungseinheiten (Region, Bezirk, Gemeinde mit erweiterter Zuständigkeit, Gemeinde)",
                },
                credit: RUIAN_CREDIT,
                note: None,
                sampled: false,
                gaps: None,
                layers: &["obce"],
                sections: &[PermitSection::AdminUnits],
                offices: None,
            },
        }),
        Box::new(Npu {
            info: PermitSourceInfo {
                key: "npu-pamatky",
                label: Wording {
                    en: "NPÚ",
                    de: "NPÚ",
                },
                what: Wording {
                    en: "cultural monuments and monument areas of the state list",
                    de: "Kulturdenkmale und Denkmalgebiete der staatlichen Liste",
                },
                credit: NPU_CREDIT,
                note: None,
                sampled: false,
                gaps: Some(Wording {
                    en: "the protective zones of monuments (ochranná pásma) are not in it",
                    de: "die Schutzzonen der Denkmale (ochranná pásma) fehlen darin",
                }),
                layers: &["pamatky"],
                sections: &[PermitSection::Monuments],
                offices: None,
            },
        }),
    ]
}

/// The URL of the page of `layer` of `service` in `bbox` that starts at `offset`, asking for the
/// attributes `fields` and, when `geometry` is true, the shapes in WGS 84.
fn query_url(
    service: &str,
    layer: u32,
    fields: &str,
    geometry: bool,
    bbox: &Rect<f64>,
    offset: usize,
) -> String {
    format!(
        "{service}/{layer}/query?where=1%3D1&geometry={:.6},{:.6},{:.6},{:.6}\
         &geometryType=esriGeometryEnvelope&inSR=4326&spatialRel=esriSpatialRelIntersects\
         &outFields={fields}&returnGeometry={geometry}&outSR=4326&geometryPrecision=6\
         &maxAllowableOffset={SIMPLIFY}&orderByFields=OBJECTID&resultOffset={offset}\
         &resultRecordCount={PAGE}&f=geojson",
        bbox.min().x,
        bbox.min().y,
        bbox.max().x,
        bbox.max().y,
    )
}

/// The features of one answer and whether more follow (`exceededTransferLimit`, which ArcGIS
/// writes at the top and in `properties`). An error object in a 200 answer is an error.
fn read_page(url: &str, bytes: &[u8]) -> Result<(Vec<Value>, bool), FetchError> {
    let wrong = |message: String| FetchError::Http {
        url: url.to_string(),
        message,
    };
    let mut value: Value = serde_json::from_slice(bytes).map_err(|e| {
        let start = one_line(
            &String::from_utf8_lossy(&bytes[..bytes.len().min(200)]),
            200,
        );
        wrong(format!("the answer is not GeoJSON ({e}): {start}"))
    })?;
    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(wrong(format!(
            "the service answered with an error: {message}"
        )));
    }
    let more = [
        &value["exceededTransferLimit"],
        &value["properties"]["exceededTransferLimit"],
    ]
    .iter()
    .any(|v| v.as_bool() == Some(true));
    match value.get_mut("features").map(Value::take) {
        Some(Value::Array(features)) => Ok((features, more)),
        _ => Err(wrong("the answer is not a FeatureCollection".to_string())),
    }
}

/// Every feature of `layer` in `bbox`, page by page.
fn fetch_layer(
    fetcher: &dyn Fetcher,
    service: &str,
    layer: u32,
    fields: &str,
    geometry: bool,
    bbox: &Rect<f64>,
    part: &Path,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    for _ in 0..MAX_PAGES {
        let url = query_url(service, layer, fields, geometry, bbox, features.len());
        let (page, more) = get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| {
            read_page(&url, bytes)
        })?;
        let empty = page.is_empty();
        features.extend(page);
        if !more || empty {
            return Ok(features);
        }
    }
    Err(FetchError::Http {
        url: format!("{service}/{layer}"),
        message: format!("the service sent more than {MAX_PAGES} pages of {layer}"),
    })
}

/// The text of the property `key`, trimmed; `None` when it is missing, not text or blank.
fn property(feature: &Value, key: &str) -> Option<String> {
    let text = feature.get("properties")?.get(key)?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// A whole number property; RÚIAN writes its codes as numbers.
fn number(feature: &Value, key: &str) -> Option<i64> {
    feature.get("properties")?.get(key)?.as_i64()
}

/// The geometry of a feature as GeoJSON in WGS 84, longitude first.
fn geometry_of(feature: &Value) -> Option<Geometry<f64>> {
    let geometry =
        serde_json::from_value::<geojson::Geometry>(feature.get("geometry")?.clone()).ok()?;
    Geometry::<f64>::try_from(geometry).ok()
}

/// The protected areas and objects of the AOPK.
struct Aopk {
    info: PermitSourceInfo,
}

impl PermitSource for Aopk {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn country(&self) -> Option<Country> {
        Some(Country::Cz)
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        AOPK_LAYERS
            .iter()
            .map(|l| fetch_layer(fetcher, l.service, l.layer, l.fields, true, bbox, part))
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let Some(layer) = AOPK_LAYERS.iter().find(|l| l.key == layer) else {
            return Vec::new();
        };
        features
            .iter()
            .filter_map(|feature| {
                let geometry = geometry_of(feature)?;
                let (kind, name) = object(layer.kind, feature)?;
                Some(PermitFinding {
                    kind: FindingKind::ProtectedObject {
                        kind,
                        name,
                        citation: None,
                    },
                    geometry,
                })
            })
            .collect()
    }
}

/// The kind and the name of an object of the AOPK; `None` for a category the notes do not know.
fn object(by: KindBy, feature: &Value) -> Option<(ProtectedObjectKind, Option<String>)> {
    let category = property(feature, "KAT");
    let name = property(feature, "NAZEV");
    Some(match by {
        KindBy::SmallArea => (
            match category.as_deref()? {
                "NPR" => ProtectedObjectKind::NationalNatureReserve,
                "NPP" => ProtectedObjectKind::NationalNatureMonument,
                "PR" => ProtectedObjectKind::NatureReserveCz,
                "PP" => ProtectedObjectKind::NatureMonumentCz,
                _ => return None,
            },
            name,
        ),
        KindBy::LargeArea => (
            match category.as_deref()? {
                "NP" => ProtectedObjectKind::NationalParkCz,
                "CHKO" => ProtectedObjectKind::ProtectedLandscapeAreaCz,
                "OP" => ProtectedObjectKind::ProtectiveZoneCz,
                _ => return None,
            },
            name,
        ),
        KindBy::ProtectiveZone => (
            ProtectedObjectKind::ProtectiveZoneCz,
            match (name, property(feature, "KAT_ZCHU")) {
                (Some(name), Some(category)) => Some(format!("{name} ({category})")),
                (name, _) => name,
            },
        ),
        KindBy::Zone => (
            ProtectedObjectKind::NatureZoneCz,
            match (category, name, property(feature, "ZONA")) {
                (Some(category), Some(name), Some(zone)) => {
                    Some(format!("{category} {name}, zóna {zone}"))
                }
                (_, name, _) => name,
            },
        ),
        KindBy::Fixed(kind) => (kind, name.or_else(|| property(feature, "DRUH"))),
    })
}

/// The administrative units of RÚIAN.
struct Ruian {
    info: PermitSourceInfo,
}

/// The code of the capital as a municipality: it is a municipality, a region and an ORP at once.
const PRAGUE: i64 = 554_782;

/// The features of a name layer of the map box, without shapes, whose records are valid: not
/// marked wrong and not ended (`nespravny`, `platido`).
fn names(
    fetcher: &dyn Fetcher,
    layer: u32,
    fields: &str,
    bbox: &Rect<f64>,
    part: &Path,
) -> Result<Vec<Value>, FetchError> {
    Ok(
        fetch_layer(fetcher, RUIAN, layer, fields, false, bbox, part)?
            .into_iter()
            .filter(valid)
            .collect(),
    )
}

/// Whether the record is the current one: neither marked wrong nor ended.
fn valid(feature: &Value) -> bool {
    let properties = &feature["properties"];
    properties["nespravny"].is_null() && properties["platido"].is_null()
}

/// The record with the code `code` in `units`.
fn lookup(units: &[Value], code: i64) -> Option<&Value> {
    units.iter().find(|u| number(u, "kod") == Some(code))
}

impl PermitSource for Ruian {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn country(&self) -> Option<Country> {
        Some(Country::Cz)
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let municipalities: Vec<Value> = fetch_layer(
            fetcher,
            RUIAN,
            12,
            "kod,nazev,okres,pou,nespravny,platido",
            true,
            bbox,
            part,
        )?
        .into_iter()
        .filter(valid)
        .collect();
        if municipalities.is_empty() {
            return Ok(vec![Vec::new()]);
        }
        let authorised = names(fetcher, 13, "kod,orp,nespravny,platido", bbox, part)?;
        let extended = names(fetcher, 14, "kod,nazev,vusc,nespravny,platido", bbox, part)?;
        let districts = names(fetcher, 15, "kod,nazev,vusc,nespravny,platido", bbox, part)?;
        let regions = names(fetcher, 17, "kod,nazev,nespravny,platido", bbox, part)?;
        let features = municipalities
            .iter()
            .filter_map(|m| {
                let geometry = m.get("geometry").filter(|g| !g.is_null())?;
                let code = number(m, "kod")?;
                let district = number(m, "okres").and_then(|c| lookup(&districts, c));
                let orp = number(m, "pou")
                    .and_then(|c| lookup(&authorised, c))
                    .and_then(|p| number(p, "orp"))
                    .and_then(|c| lookup(&extended, c));
                // The region: that of the district; the capital has no district, its ORP names it.
                let region_code = district
                    .and_then(|d| number(d, "vusc"))
                    .or_else(|| orp.and_then(|o| number(o, "vusc")));
                let region = region_code.and_then(|c| lookup(&regions, c));
                let pick = |unit: Option<&Value>, key: &str| unit.and_then(|u| property(u, key));
                Some(json!({
                    "type": "Feature",
                    "properties": {
                        "kod": code,
                        "nazev": property(m, "nazev"),
                        "okres_kod": district.and_then(|d| number(d, "kod")),
                        "okres": pick(district, "nazev"),
                        "orp_kod": orp.and_then(|o| number(o, "kod")),
                        "orp": pick(orp, "nazev"),
                        "kraj_kod": region.and_then(|r| number(r, "kod")),
                        "kraj": pick(region, "nazev"),
                    },
                    "geometry": geometry,
                }))
            })
            .collect();
        Ok(vec![features])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != "obce" {
            return Vec::new();
        }
        let mut found = Vec::new();
        for feature in features {
            let Some(geometry) = geometry_of(feature) else {
                continue;
            };
            let unit = |level, code: i64, name: String| PermitFinding {
                kind: FindingKind::AdminUnit {
                    level,
                    name,
                    code: code.to_string(),
                },
                geometry: geometry.clone(),
            };
            // A unit whose name the register did not answer is left out, not guessed.
            if let (Some(code), Some(name)) =
                (number(feature, "kraj_kod"), property(feature, "kraj"))
            {
                found.push(unit(AdminUnitLevel::Region, code, name));
            }
            if let (Some(code), Some(name)) =
                (number(feature, "okres_kod"), property(feature, "okres"))
            {
                found.push(unit(AdminUnitLevel::County, code, format!("okres {name}")));
            }
            if let (Some(code), Some(name)) = (number(feature, "orp_kod"), property(feature, "orp"))
            {
                found.push(unit(
                    AdminUnitLevel::AuthorityDistrict,
                    code,
                    format!("ORP {name}"),
                ));
            }
            if let (Some(code), Some(name)) = (number(feature, "kod"), property(feature, "nazev")) {
                let name = if code == PRAGUE {
                    "hlavní město Praha".to_string()
                } else {
                    format!("obec {name}")
                };
                found.push(unit(AdminUnitLevel::Municipality, code, name));
            }
        }
        found
    }
}

/// The cultural monuments and monument areas of the NPÚ.
struct Npu {
    info: PermitSourceInfo,
}

/// The kind of a site as the register's INSPIRE service designates it: a monument, a monument
/// area (reservation, zone) or a site of the world heritage list.
fn site_kind(designation: &str) -> Option<&'static str> {
    match designation {
        "monument" => Some("kulturní památka"),
        "settlement" => Some("památková rezervace nebo zóna"),
        "cultural" => Some("světové dědictví UNESCO"),
        _ => None,
    }
}

impl PermitSource for Npu {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn country(&self) -> Option<Country> {
        Some(Country::Cz)
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        Ok(vec![fetch_layer(
            fetcher,
            NPU_SITES,
            0,
            "SD_DESIGNATION",
            true,
            bbox,
            part,
        )?])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != "pamatky" {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|feature| {
                let geometry = geometry_of(feature)?;
                let kind = site_kind(&property(feature, "SD_DESIGNATION")?)?;
                Some(PermitFinding {
                    kind: FindingKind::Monument {
                        kind: kind.to_string(),
                        link: None,
                    },
                    geometry,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::permits::plain_key;
    use freemapper_adapter_api::testing::MapFetcher;
    use geo_types::Coord;

    const AOPK_FIXTURES: [(&str, &str); 8] = [
        (
            "maloplosna",
            include_str!("../tests/fixtures/permits/aopk-maloplosna.json"),
        ),
        (
            "ochranna-pasma",
            include_str!("../tests/fixtures/permits/aopk-ochranna-pasma.json"),
        ),
        (
            "velkoplosna",
            include_str!("../tests/fixtures/permits/aopk-velkoplosna.json"),
        ),
        (
            "zony",
            include_str!("../tests/fixtures/permits/aopk-zony.json"),
        ),
        (
            "natura-evl",
            include_str!("../tests/fixtures/permits/aopk-natura-evl.json"),
        ),
        (
            "natura-po",
            include_str!("../tests/fixtures/permits/aopk-natura-po.json"),
        ),
        (
            "pamatne-stromy",
            include_str!("../tests/fixtures/permits/aopk-pamatne-stromy.json"),
        ),
        (
            "pamatne-stromy-skupiny",
            include_str!("../tests/fixtures/permits/aopk-pamatne-stromy-skupiny.json"),
        ),
    ];
    const OBCE: &str = include_str!("../tests/fixtures/permits/ruian-obce.json");
    const POU: &str = include_str!("../tests/fixtures/permits/ruian-pou.json");
    const ORP: &str = include_str!("../tests/fixtures/permits/ruian-orp.json");
    const OKRES: &str = include_str!("../tests/fixtures/permits/ruian-okres.json");
    const KRAJ: &str = include_str!("../tests/fixtures/permits/ruian-kraj.json");
    const PRAHA_OBCE: &str = include_str!("../tests/fixtures/permits/ruian-praha-obce.json");
    const PRAHA_POU: &str = include_str!("../tests/fixtures/permits/ruian-praha-pou.json");
    const PRAHA_ORP: &str = include_str!("../tests/fixtures/permits/ruian-praha-orp.json");
    const PRAHA_OKRES: &str = include_str!("../tests/fixtures/permits/ruian-praha-okres.json");
    const PRAHA_KRAJ: &str = include_str!("../tests/fixtures/permits/ruian-praha-kraj.json");
    const NPU_ADRSPACH: &str = include_str!("../tests/fixtures/permits/npu-adrspach.json");
    const NPU_PRAHA: &str = include_str!("../tests/fixtures/permits/npu-praha.json");

    const MUNICIPALITY_FIELDS: &str = "kod,nazev,okres,pou,nespravny,platido";

    fn bbox() -> Rect<f64> {
        Rect::new(
            Coord {
                x: 16.105,
                y: 50.608,
            },
            Coord {
                x: 16.122,
                y: 50.617,
            },
        )
    }

    fn part() -> std::path::PathBuf {
        tempfile::tempdir().unwrap().keep().join("page.part")
    }

    fn source(key: &str) -> Box<dyn PermitSource> {
        sources()
            .into_iter()
            .find(|s| s.info().key == key)
            .expect("the source exists")
    }

    fn features(text: &str) -> Vec<Value> {
        let mut value: Value = serde_json::from_str(text).unwrap();
        match value["features"].take() {
            Value::Array(features) => features,
            other => panic!("{other:?}"),
        }
    }

    fn feature(properties: Value) -> Value {
        json!({"type": "Feature", "properties": properties,
            "geometry": {"type": "Point", "coordinates": [16.1, 50.6]}})
    }

    fn objects(layer: &str, list: &[Value]) -> Vec<(ProtectedObjectKind, Option<String>)> {
        source("aopk")
            .findings(layer, list)
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::ProtectedObject { kind, name, .. } => (kind, name),
                other => panic!("{other:?}"),
            })
            .collect()
    }

    #[test]
    fn every_source_says_it_is_czech() {
        for source in sources() {
            assert_eq!(source.country(), Some(Country::Cz), "{}", source.info().key);
        }
    }

    #[test]
    fn the_keys_of_the_layers_are_plain_and_match_the_table() {
        let all = sources();
        let keys: Vec<_> = all.iter().map(|s| s.info().key).collect();
        assert_eq!(keys, ["aopk", "ruian-hranice", "npu-pamatky"]);
        let layers = all[0].info().layers;
        assert_eq!(layers, AOPK_KEYS);
        assert_eq!(
            layers,
            AOPK_LAYERS.map(|l| l.key),
            "the table and the keys agree"
        );
        for source in &all {
            assert!(plain_key(source.info().key));
            assert!(source.info().layers.iter().all(|l| plain_key(l)));
        }
        assert_eq!(layers.len(), AOPK_FIXTURES.len());
    }

    #[test]
    fn the_sources_name_their_credit_sections_and_hosts() {
        let all = sources();
        let infos: Vec<_> = all.iter().map(|s| s.info()).collect();
        assert_eq!(infos[0].credit, "© AOPK ČR (CC BY 4.0)");
        assert_eq!(infos[0].sections, [PermitSection::ProtectedObjects]);
        assert_eq!(infos[1].credit, "© ČÚZK (CC BY 4.0)");
        assert_eq!(infos[1].sections, [PermitSection::AdminUnits]);
        assert_eq!(infos[1].layers, ["obce"]);
        assert_eq!(infos[2].credit, "© Národní památkový ústav (CC BY-SA 4.0)");
        assert_eq!(infos[2].sections, [PermitSection::Monuments]);
        assert_eq!(infos[2].layers, ["pamatky"]);
        assert!(infos[2].gaps.is_some() && infos[0].gaps.is_some() && infos[1].gaps.is_none());
        for service in [AOPK_PROTECTION, AOPK_NATURA, AOPK_TREES, RUIAN, NPU_SITES] {
            assert!(service.starts_with("https://"), "{service}");
            assert!(service.ends_with("/MapServer"), "{service}");
        }
        for service in [AOPK_PROTECTION, AOPK_NATURA, AOPK_TREES] {
            assert!(service.contains(AOPK_HOST), "{service}");
        }
        assert!(NPU_SITES.contains(NPU_HOST));
    }

    #[test]
    fn each_layer_of_the_aopk_gives_its_kind_and_the_name_of_the_data() {
        let expected = [
            (
                "maloplosna",
                ProtectedObjectKind::NationalNatureReserve,
                "Adršpašsko-teplické skály",
            ),
            (
                "ochranna-pasma",
                ProtectedObjectKind::ProtectiveZoneCz,
                "Křížová cesta (PR)",
            ),
            (
                "velkoplosna",
                ProtectedObjectKind::ProtectedLandscapeAreaCz,
                "Broumovsko",
            ),
            (
                "natura-evl",
                ProtectedObjectKind::Natura2000HabitatsCz,
                "Adršpašsko-teplické skály",
            ),
            (
                "natura-po",
                ProtectedObjectKind::Natura2000BirdsCz,
                "Broumovsko",
            ),
        ];
        let source = source("aopk");
        for (key, kind, name) in expected {
            let text = AOPK_FIXTURES.iter().find(|(k, _)| *k == key).unwrap().1;
            let found = objects(key, &features(text));
            assert_eq!(
                found.first(),
                Some(&(kind, Some(name.to_string()))),
                "{key}"
            );
            assert!(
                !source.findings(key, &features(text)).is_empty(),
                "{key} gives findings"
            );
        }
    }

    #[test]
    fn the_zones_of_a_protected_landscape_area_name_the_area_and_the_zone() {
        let text = AOPK_FIXTURES.iter().find(|(k, _)| *k == "zony").unwrap().1;
        let found = objects("zony", &features(text));
        assert_eq!(found.len(), 2);
        assert!(
            found
                .iter()
                .all(|(kind, _)| *kind == ProtectedObjectKind::NatureZoneCz)
        );
        let names: Vec<_> = found.iter().filter_map(|(_, n)| n.as_deref()).collect();
        assert!(names.contains(&"CHKO Broumovsko, zóna I"), "{names:?}");
        assert!(names.contains(&"CHKO Broumovsko, zóna III"), "{names:?}");
        // Without the zone the name is the area's.
        let bare = feature(json!({"KAT": "NP", "NAZEV": "Šumava"}));
        assert_eq!(
            objects("zony", &[bare]),
            [(
                ProtectedObjectKind::NatureZoneCz,
                Some("Šumava".to_string())
            )]
        );
    }

    #[test]
    fn a_small_area_takes_its_kind_from_its_category() {
        for (category, kind) in [
            ("NPR", ProtectedObjectKind::NationalNatureReserve),
            ("NPP", ProtectedObjectKind::NationalNatureMonument),
            ("PR", ProtectedObjectKind::NatureReserveCz),
            ("PP", ProtectedObjectKind::NatureMonumentCz),
        ] {
            let list = [feature(json!({"KAT": category, "NAZEV": "X"}))];
            assert_eq!(
                objects("maloplosna", &list),
                [(kind, Some("X".to_string()))],
                "{category}"
            );
        }
        // A category the notes do not know, and a feature without one, are left out.
        let unknown = [
            feature(json!({"KAT": "ZZ", "NAZEV": "X"})),
            feature(json!({"NAZEV": "Y"})),
        ];
        assert!(objects("maloplosna", &unknown).is_empty());
    }

    #[test]
    fn a_large_area_takes_its_kind_from_its_category() {
        for (category, kind) in [
            ("NP", ProtectedObjectKind::NationalParkCz),
            ("CHKO", ProtectedObjectKind::ProtectedLandscapeAreaCz),
            ("OP", ProtectedObjectKind::ProtectiveZoneCz),
        ] {
            let list = [feature(json!({"KAT": category, "NAZEV": "X"}))];
            assert_eq!(
                objects("velkoplosna", &list),
                [(kind, Some("X".to_string()))],
                "{category}"
            );
        }
        let unknown = [feature(json!({"KAT": "NPR", "NAZEV": "X"}))];
        assert!(objects("velkoplosna", &unknown).is_empty());
    }

    #[test]
    fn a_protective_zone_without_its_category_is_named_as_the_data_do() {
        let list = [feature(json!({"NAZEV": "Křížová cesta"}))];
        assert_eq!(
            objects("ochranna-pasma", &list),
            [(
                ProtectedObjectKind::ProtectiveZoneCz,
                Some("Křížová cesta".to_string())
            )]
        );
    }

    #[test]
    fn memorial_trees_come_by_name_and_fall_back_to_the_species() {
        let text = AOPK_FIXTURES
            .iter()
            .find(|(k, _)| *k == "pamatne-stromy")
            .unwrap()
            .1;
        let found = objects("pamatne-stromy", &features(text));
        assert_eq!(found.len(), 3);
        assert!(
            found
                .iter()
                .all(|(kind, _)| *kind == ProtectedObjectKind::MemorialTree)
        );
        assert_eq!(
            found[0].1.as_deref(),
            Some("Hrušeň obecná nad Zdíkovskou ulicí")
        );
        let species = [feature(json!({"DRUH": "dub letní"}))];
        assert_eq!(
            objects("pamatne-stromy", &species)[0].1.as_deref(),
            Some("dub letní")
        );
        let none = [feature(json!({}))];
        assert_eq!(objects("pamatne-stromy", &none)[0].1, None);
        let groups = AOPK_FIXTURES
            .iter()
            .find(|(k, _)| *k == "pamatne-stromy-skupiny")
            .unwrap()
            .1;
        let found = objects("pamatne-stromy-skupiny", &features(groups));
        assert_eq!(found.len(), 2);
        assert_eq!(found[1].1.as_deref(), Some("Duby pod táborem"));
    }

    #[test]
    fn a_feature_without_a_geometry_is_left_out_and_an_unknown_layer_gives_nothing() {
        let source = source("aopk");
        let bare = json!({"type": "Feature", "geometry": null,
            "properties": {"KAT": "NPR", "NAZEV": "X"}});
        assert!(source.findings("maloplosna", &[bare]).is_empty());
        let one = [feature(json!({"KAT": "NPR", "NAZEV": "X"}))];
        assert!(source.findings("nothing", &one).is_empty());
        assert_eq!(source.findings("maloplosna", &one).len(), 1);
    }

    #[test]
    fn the_url_asks_longitude_first_in_wgs_84_with_the_fields_and_the_page() {
        let url = query_url(AOPK_PROTECTION, 3, "KAT,NAZEV", true, &bbox(), 2000);
        assert_eq!(
            url,
            "https://gis.nature.cz/arcgis/rest/services/UzemniOchrana/ChranUzemi/MapServer/3/query\
             ?where=1%3D1&geometry=16.105000,50.608000,16.122000,50.617000\
             &geometryType=esriGeometryEnvelope&inSR=4326&spatialRel=esriSpatialRelIntersects\
             &outFields=KAT,NAZEV&returnGeometry=true&outSR=4326&geometryPrecision=6\
             &maxAllowableOffset=0.00005&orderByFields=OBJECTID&resultOffset=2000\
             &resultRecordCount=1000&f=geojson"
        );
        let plain = query_url(RUIAN, 15, "kod", false, &bbox(), 0);
        assert!(plain.contains("returnGeometry=false"), "{plain}");
        assert!(plain.contains("resultOffset=0"), "{plain}");
    }

    fn aopk_fetcher(bbox: &Rect<f64>) -> MapFetcher {
        let fetcher = MapFetcher::default();
        for (key, text) in AOPK_FIXTURES {
            let layer = AOPK_LAYERS.iter().find(|l| l.key == key).unwrap();
            fetcher.insert(
                &query_url(layer.service, layer.layer, layer.fields, true, bbox, 0),
                text.as_bytes().to_vec(),
            );
        }
        fetcher
    }

    #[test]
    fn the_fetch_asks_every_layer_once_and_returns_them_in_order() {
        let bbox = bbox();
        let fetcher = aopk_fetcher(&bbox);
        let lists = source("aopk").fetch(&fetcher, &bbox, &part()).unwrap();
        assert_eq!(lists.len(), 8);
        assert_eq!(fetcher.calls.get(), 8);
        let lengths: Vec<usize> = lists.iter().map(Vec::len).collect();
        assert_eq!(lengths, [1, 1, 1, 2, 1, 1, 3, 2]);
    }

    #[test]
    fn the_fetch_stops_at_the_first_layer_that_fails() {
        let bbox = bbox();
        let fetcher = MapFetcher::default();
        assert!(source("aopk").fetch(&fetcher, &bbox, &part()).is_err());
        // An error object with status 200 is an error, whatever the retries do.
        let first = &AOPK_LAYERS[0];
        fetcher.insert(
            &query_url(first.service, first.layer, first.fields, true, &bbox, 0),
            br#"{"error":{"code":400,"message":"Failed to execute query."}}"#.to_vec(),
        );
        let error = source("aopk")
            .fetch(&fetcher, &bbox, &part())
            .unwrap_err()
            .to_string();
        assert!(error.contains("Failed to execute query."), "{error}");
    }

    #[test]
    fn an_answer_that_is_no_collection_is_an_error() {
        let url = "https://example.org/q";
        assert!(read_page(url, b"<html>down</html>").is_err());
        assert!(read_page(url, br#"{"type":"x"}"#).is_err());
        let (features, more) = read_page(url, br#"{"features":[]}"#).unwrap();
        assert!(features.is_empty() && !more);
    }

    #[test]
    fn the_transfer_limit_says_more_follow_wherever_it_is_written() {
        let url = "https://example.org/q";
        let top = br#"{"features":[],"exceededTransferLimit":true}"#;
        assert!(read_page(url, top).unwrap().1);
        let inner = br#"{"features":[],"properties":{"exceededTransferLimit":true}}"#;
        assert!(read_page(url, inner).unwrap().1);
        let no = br#"{"features":[],"exceededTransferLimit":false}"#;
        assert!(!read_page(url, no).unwrap().1);
    }

    #[test]
    fn a_page_that_says_more_follow_asks_for_the_next_one() {
        let bbox = bbox();
        let fetcher = MapFetcher::default();
        let one = json!({"type": "Feature", "properties": {"SD_DESIGNATION": "monument"},
            "geometry": {"type": "Point", "coordinates": [16.1, 50.6]}});
        let first = json!({"features": [one.clone(), one.clone()], "exceededTransferLimit": true});
        let second = json!({"features": [one.clone()]});
        let url = |offset| query_url(NPU_SITES, 0, "SD_DESIGNATION", true, &bbox, offset);
        fetcher.insert(&url(0), first.to_string().into_bytes());
        fetcher.insert(&url(2), second.to_string().into_bytes());
        let lists = source("npu-pamatky")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert_eq!(lists[0].len(), 3);
        assert_eq!(fetcher.calls.get(), 2);
    }

    #[test]
    fn an_empty_page_ends_the_list_even_when_it_says_more_follow() {
        let bbox = bbox();
        let fetcher = MapFetcher::default();
        let empty = json!({"features": [], "exceededTransferLimit": true});
        fetcher.insert(
            &query_url(NPU_SITES, 0, "SD_DESIGNATION", true, &bbox, 0),
            empty.to_string().into_bytes(),
        );
        let lists = source("npu-pamatky")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert!(lists[0].is_empty());
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn a_list_that_never_ends_is_an_error() {
        let bbox = bbox();
        let fetcher = MapFetcher::default();
        let one = json!({"type": "Feature", "properties": {},
            "geometry": {"type": "Point", "coordinates": [16.1, 50.6]}});
        let page = json!({"features": [one], "exceededTransferLimit": true}).to_string();
        for n in 0..MAX_PAGES {
            fetcher.insert(
                &query_url(NPU_SITES, 0, "SD_DESIGNATION", true, &bbox, n),
                page.clone().into_bytes(),
            );
        }
        let error = source("npu-pamatky")
            .fetch(&fetcher, &bbox, &part())
            .unwrap_err()
            .to_string();
        assert!(error.contains("more than 20 pages"), "{error}");
        assert_eq!(fetcher.calls.get(), MAX_PAGES);
    }

    fn admin_fetcher(bbox: &Rect<f64>, answers: [&str; 5]) -> MapFetcher {
        let fetcher = MapFetcher::default();
        let layers = [
            (12, MUNICIPALITY_FIELDS, true),
            (13, "kod,orp,nespravny,platido", false),
            (14, "kod,nazev,vusc,nespravny,platido", false),
            (15, "kod,nazev,vusc,nespravny,platido", false),
            (17, "kod,nazev,nespravny,platido", false),
        ];
        for ((layer, fields, geometry), answer) in layers.into_iter().zip(answers) {
            fetcher.insert(
                &query_url(RUIAN, layer, fields, geometry, bbox, 0),
                answer.as_bytes().to_vec(),
            );
        }
        fetcher
    }

    /// The units as (level, name, code) of the findings of a fetch.
    fn units(lists: &[Vec<Value>]) -> Vec<(AdminUnitLevel, String, String)> {
        source("ruian-hranice")
            .findings("obce", &lists[0])
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::AdminUnit { level, name, code } => (level, name, code),
                other => panic!("{other:?}"),
            })
            .collect()
    }

    fn unit(level: AdminUnitLevel, name: &str, code: &str) -> (AdminUnitLevel, String, String) {
        (level, name.to_string(), code.to_string())
    }

    #[test]
    fn a_map_area_in_adrspach_names_region_district_orp_and_municipality() {
        let bbox = bbox();
        let fetcher = admin_fetcher(&bbox, [OBCE, POU, ORP, OKRES, KRAJ]);
        let lists = source("ruian-hranice")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert_eq!(fetcher.calls.get(), 5);
        assert_eq!(
            units(&lists),
            [
                unit(AdminUnitLevel::Region, "Královéhradecký kraj", "86"),
                unit(AdminUnitLevel::County, "okres Náchod", "3605"),
                unit(AdminUnitLevel::AuthorityDistrict, "ORP Broumov", "1031"),
                unit(AdminUnitLevel::Municipality, "obec Adršpach", "547786"),
            ]
        );
        // Every unit lies where the municipality does.
        let findings = source("ruian-hranice").findings("obce", &lists[0]);
        assert_eq!(findings.len(), 4);
        assert!(findings.iter().all(|f| f.geometry == findings[0].geometry));
    }

    #[test]
    fn the_capital_has_no_district_and_its_orp_names_the_region() {
        let bbox = bbox();
        let fetcher = admin_fetcher(
            &bbox,
            [PRAHA_OBCE, PRAHA_POU, PRAHA_ORP, PRAHA_OKRES, PRAHA_KRAJ],
        );
        let lists = source("ruian-hranice")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert_eq!(
            units(&lists),
            [
                unit(AdminUnitLevel::Region, "Hlavní město Praha", "19"),
                unit(
                    AdminUnitLevel::AuthorityDistrict,
                    "ORP Hlavní město Praha",
                    "19"
                ),
                unit(AdminUnitLevel::Municipality, "hlavní město Praha", "554782"),
            ]
        );
    }

    #[test]
    fn without_a_municipality_the_names_of_the_upper_levels_are_not_asked() {
        let bbox = bbox();
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &query_url(RUIAN, 12, MUNICIPALITY_FIELDS, true, &bbox, 0),
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        let lists = source("ruian-hranice")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert_eq!(lists.len(), 1);
        assert!(lists[0].is_empty());
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn a_missing_name_of_a_level_leaves_that_level_out() {
        let bbox = bbox();
        let empty = r#"{"type":"FeatureCollection","features":[]}"#;
        // The region comes by the district, so a missing district takes the region with it.
        for (answers, left_out) in [
            (
                [OBCE, POU, ORP, empty, KRAJ],
                vec!["okres Náchod", "Královéhradecký kraj"],
            ),
            ([OBCE, empty, ORP, OKRES, KRAJ], vec!["ORP Broumov"]),
            ([OBCE, POU, empty, OKRES, KRAJ], vec!["ORP Broumov"]),
            ([OBCE, POU, ORP, OKRES, empty], vec!["Královéhradecký kraj"]),
        ] {
            let fetcher = admin_fetcher(&bbox, answers);
            let lists = source("ruian-hranice")
                .fetch(&fetcher, &bbox, &part())
                .unwrap();
            let found = units(&lists);
            assert_eq!(found.len(), 4 - left_out.len(), "{left_out:?}: {found:?}");
            assert!(
                found.iter().all(|u| !left_out.contains(&u.1.as_str())),
                "{left_out:?}"
            );
            assert_eq!(found.last().unwrap().1, "obec Adršpach");
        }
    }

    #[test]
    fn a_record_that_ended_or_is_marked_wrong_is_left_out() {
        let bbox = bbox();
        let ended = OBCE.replace(r#""platido": null"#, r#""platido": 1700000000000"#);
        let wrong = OBCE.replace(r#""nespravny": null"#, r#""nespravny": true"#);
        for answer in [ended.as_str(), wrong.as_str()] {
            let fetcher = admin_fetcher(&bbox, [answer, POU, ORP, OKRES, KRAJ]);
            let lists = source("ruian-hranice")
                .fetch(&fetcher, &bbox, &part())
                .unwrap();
            assert!(lists[0].is_empty());
        }
        // The same holds for the name layers: a district that ended names nobody.
        let ended = OKRES.replace(r#""platido": null"#, r#""platido": 1700000000000"#);
        let fetcher = admin_fetcher(&bbox, [OBCE, POU, ORP, &ended, KRAJ]);
        let lists = source("ruian-hranice")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        let found = units(&lists);
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|u| u.0 != AdminUnitLevel::County));
        // An ORP that ended names no ORP.
        let ended = ORP.replace(r#""platido": null"#, r#""platido": 1700000000000"#);
        let fetcher = admin_fetcher(&bbox, [OBCE, POU, &ended, OKRES, KRAJ]);
        let lists = source("ruian-hranice")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert!(
            units(&lists)
                .iter()
                .all(|u| u.0 != AdminUnitLevel::AuthorityDistrict)
        );
    }

    #[test]
    fn a_municipality_without_a_geometry_or_a_code_is_left_out() {
        let bbox = bbox();
        let mut list = features(OBCE);
        list[0]["geometry"] = Value::Null;
        let text = json!({"type": "FeatureCollection", "features": list}).to_string();
        let fetcher = admin_fetcher(&bbox, [&text, POU, ORP, OKRES, KRAJ]);
        let lists = source("ruian-hranice")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert!(lists[0].is_empty());
        let mut list = features(OBCE);
        list[0]["properties"]["kod"] = Value::Null;
        let text = json!({"type": "FeatureCollection", "features": list}).to_string();
        let fetcher = admin_fetcher(&bbox, [&text, POU, ORP, OKRES, KRAJ]);
        let lists = source("ruian-hranice")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert!(lists[0].is_empty());
        // And findings of stored features without a shape give none.
        let stored = [json!({"type": "Feature", "geometry": null,
            "properties": {"kod": 1, "nazev": "X"}})];
        assert!(source("ruian-hranice").findings("obce", &stored).is_empty());
        assert!(
            source("ruian-hranice")
                .findings("other", &features(OBCE))
                .is_empty()
        );
    }

    #[test]
    fn a_failing_service_is_an_error_of_the_source() {
        let bbox = bbox();
        let fetcher = MapFetcher::default();
        assert!(
            source("ruian-hranice")
                .fetch(&fetcher, &bbox, &part())
                .is_err()
        );
        // The municipality answers, a name layer does not.
        fetcher.insert(
            &query_url(RUIAN, 12, MUNICIPALITY_FIELDS, true, &bbox, 0),
            OBCE.as_bytes().to_vec(),
        );
        assert!(
            source("ruian-hranice")
                .fetch(&fetcher, &bbox, &part())
                .is_err()
        );
    }

    #[test]
    fn monuments_come_by_the_designation_of_the_register() {
        let npu = source("npu-pamatky");
        let kinds = |text: &str| -> Vec<String> {
            npu.findings("pamatky", &features(text))
                .into_iter()
                .map(|f| match f.kind {
                    FindingKind::Monument { kind, link } => {
                        assert_eq!(link, None);
                        kind
                    }
                    other => panic!("{other:?}"),
                })
                .collect()
        };
        assert_eq!(kinds(NPU_ADRSPACH), ["kulturní památka"; 5]);
        assert_eq!(
            kinds(NPU_PRAHA),
            [
                "světové dědictví UNESCO",
                "památková rezervace nebo zóna",
                "kulturní památka",
                "kulturní památka",
                "kulturní památka",
                "kulturní památka",
            ]
        );
        // A designation the notes do not know, and a feature without one, are left out; so is
        // one without a shape.
        let list = [
            feature(json!({"SD_DESIGNATION": "ruin"})),
            feature(json!({})),
            json!({"type": "Feature", "geometry": null,
                "properties": {"SD_DESIGNATION": "monument"}}),
            feature(json!({"SD_DESIGNATION": "monument"})),
        ];
        assert_eq!(npu.findings("pamatky", &list).len(), 1);
        assert!(npu.findings("other", &list).is_empty());
    }

    #[test]
    fn the_fetch_of_the_monuments_asks_the_inspire_layer_once() {
        let bbox = bbox();
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &query_url(NPU_SITES, 0, "SD_DESIGNATION", true, &bbox, 0),
            NPU_ADRSPACH.as_bytes().to_vec(),
        );
        let lists = source("npu-pamatky")
            .fetch(&fetcher, &bbox, &part())
            .unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].len(), 5);
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn the_kind_of_a_site_follows_its_designation() {
        assert_eq!(site_kind("monument"), Some("kulturní památka"));
        assert_eq!(
            site_kind("settlement"),
            Some("památková rezervace nebo zóna")
        );
        assert_eq!(site_kind("cultural"), Some("světové dědictví UNESCO"));
        assert_eq!(site_kind("other"), None);
    }

    #[test]
    fn the_numbers_and_names_of_a_feature_are_read_as_such() {
        let f = feature(json!({"kod": 5, "text": "  x  ", "blank": "  ", "n": "5"}));
        assert_eq!(number(&f, "kod"), Some(5));
        assert_eq!(number(&f, "n"), None);
        assert_eq!(number(&f, "missing"), None);
        assert_eq!(property(&f, "text").as_deref(), Some("x"));
        assert_eq!(property(&f, "blank"), None);
        assert_eq!(property(&f, "kod"), None);
        assert!(valid(&feature(json!({"nespravny": null, "platido": null}))));
        assert!(valid(&feature(json!({}))));
        assert!(!valid(&feature(json!({"nespravny": true}))));
        assert!(!valid(&feature(json!({"platido": 1}))));
    }
}
