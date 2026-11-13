//! The full ALKIS (AAA model, a NAS-conformant WFS with the AdV namespace `adv:`) as buildings
//! and land use in the simplified schema, for a state that serves no "ALKIS vereinfacht".
//!
//! The simplified schema is derived from these objects, so the way back is short: one feature
//! type per land use class, and the AdV codes with their labels (`assets/rules/adv-codes.toml`)
//! in place of the texts. The result was compared with the two services the LGLN runs for
//! Lower Saxony, the full and the simplified one: same objects, same texts
//! (`docs/regions/bremen-reference.md`).

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use serde_json::{Map, Value};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher};
use crate::nas::{NasService, PartFile, fetch_nas_type};
use crate::note::{Note, note};

/// The ALKIS of a state from the WFS of its full ALKIS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AaaAlkis {
    pub service: NasService,
    /// `bez` of a square without a function, where the state leaves open what most squares
    /// are used for: they stay candidates.
    pub square_bez: Option<Cow<'static, str>>,
    /// The folder of the provider in the cache; an answer streams through a file there.
    pub cache_dir: Cow<'static, str>,
}

/// How many features the reader could not read are named in the log.
const SKIPPED_SHOWN: usize = 5;

impl AaaAlkis {
    /// A new file in the cache for each answer on its way: fetches at the same time never
    /// share one.
    fn part(&self, env: &AdapterEnv<'_>) -> impl Fn() -> Result<PathBuf, FetchError> + use<> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let folder = env.cache.provider_dir(&self.cache_dir);
        move || {
            std::fs::create_dir_all(&folder).map_err(|e| FetchError::Io(folder.clone(), e))?;
            Ok(folder.join(format!(
                "alkis-{}-{}.gml.part",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            )))
        }
    }

    fn part_of(
        &self,
        env: &mut AdapterEnv<'_>,
        fetched: Result<Fetched, FetchError>,
    ) -> Result<AlkisPart, AdapterError> {
        let fetched = fetched?;
        if !fetched.skipped.is_empty() {
            let shown: Vec<&str> = fetched
                .skipped
                .iter()
                .take(SKIPPED_SHOWN)
                .map(String::as_str)
                .collect();
            let more = fetched.skipped.len() > SKIPPED_SHOWN;
            (env.progress)(skipped_note(fetched.skipped.len(), &shown, more));
        }
        Ok(AlkisPart {
            features: fetched.features,
            crs: self.service.crs,
        })
    }
}

fn skipped_note(count: usize, shown: &[&str], more: bool) -> Note {
    note!(
        "progress-alkis-skipped",
        count = count,
        list = format!("{}{}", shown.join("; "), if more { "; …" } else { "" })
    )
}

impl AlkisSource for AaaAlkis {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let part = self.part(env);
        let bbox = area.bbox_for(self.service.crs);
        let fetched = fetch_buildings(env.fetcher, &self.service, &bbox, &part);
        self.part_of(env, fetched)
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let part = self.part(env);
        let bbox = area.bbox_for(self.service.crs);
        let fetched = fetch_land_use(
            env.fetcher,
            &self.service,
            self.square_bez.as_deref(),
            &bbox,
            &part,
        );
        self.part_of(env, fetched)
    }
}

/// What a fetch came to: the features in the simplified schema, and the features the reader
/// could not read (`<gml:id> (<type>): <why>`).
#[derive(Debug, Default, PartialEq)]
pub struct Fetched {
    pub features: Vec<Value>,
    pub skipped: Vec<String>,
}

/// The feature types the buildings and structures of the simplified schema come from.
pub const BUILDING_TYPES: [&str; 3] = [
    "adv:AX_Gebaeude",
    "adv:AX_Bauteil",
    "adv:AX_SonstigesBauwerkOderSonstigeEinrichtung",
];

/// The buildings, building parts and roofs of `service` in `bbox` (in its CRS), in the
/// simplified schema.
pub fn fetch_buildings(
    fetcher: &dyn Fetcher,
    service: &NasService,
    bbox: &BoundingBox,
    part: PartFile,
) -> Result<Fetched, FetchError> {
    fetch(fetcher, service, bbox, part, &BUILDING_TYPES, &building)
}

/// The land use of `service` in `bbox` (in its CRS), in the simplified schema: one request per
/// land use type.
pub fn fetch_land_use(
    fetcher: &dyn Fetcher,
    service: &NasService,
    square_bez: Option<&str>,
    bbox: &BoundingBox,
    part: PartFile,
) -> Result<Fetched, FetchError> {
    let types: Vec<String> = LAND_USE_TYPES
        .iter()
        .map(|land| format!("adv:{}", land.type_name))
        .collect();
    let types: Vec<&str> = types.iter().map(String::as_str).collect();
    fetch(fetcher, service, bbox, part, &types, &|feature| {
        land_use(feature, square_bez)
    })
}

fn fetch(
    fetcher: &dyn Fetcher,
    service: &NasService,
    bbox: &BoundingBox,
    part: PartFile,
    types: &[&str],
    simplified: &dyn Fn(&Value) -> Option<Value>,
) -> Result<Fetched, FetchError> {
    let mut fetched = Fetched::default();
    for type_name in types {
        let answer = fetch_nas_type(fetcher, service, type_name, bbox, part)?;
        fetched
            .features
            .extend(answer.features.iter().filter_map(simplified));
        fetched.skipped.extend(answer.skipped);
    }
    Ok(fetched)
}

/// `gebnutzbez` of a structure that is no building.
const STRUCTURE: &str = "Sonstiges Bauwerk oder sonstige Einrichtung";
/// The functions of a structure the simplified schema holds as an area: `Überdachung` and
/// `Carport`. Walls, fences and stairs are lines and points of the full ALKIS layers.
const ROOFS: [&str; 2] = ["1610", "1611"];
/// What the oid of a building and of a land use area ends in, as the simplified services
/// write it after the `gml:id`.
const BUILDING_SUFFIX: &str = "BL";
const LAND_USE_SUFFIX: &str = "TN";

/// A building, a building part or a roof of the full ALKIS in the simplified schema; `None`
/// for any other feature. A code the lists do not know leaves the text out.
pub fn building(feature: &Value) -> Option<Value> {
    let nas_type = text(feature, "nas_type")?;
    let label = |property: &str| label(nas_type, property, text(feature, property)?);
    let (kind, function) = match nas_type {
        "AX_Gebaeude" => ("Gebäude", label("gebaeudefunktion")),
        "AX_Bauteil" => ("Bauteil", label("bauart")),
        "AX_SonstigesBauwerkOderSonstigeEinrichtung" => {
            let code = text(feature, "bauwerksfunktion")?;
            if !ROOFS.contains(&code) || !is_area(feature) {
                return None;
            }
            (STRUCTURE, label("bauwerksfunktion"))
        }
        _ => return None,
    };
    Some(in_schema(
        feature,
        BUILDING_SUFFIX,
        vec![
            ("gebnutzbez", Some(kind)),
            ("funktion", function),
            (
                "rellage",
                label_of("AX_Gebaeude", feature, "lageZurErdoberflaeche"),
            ),
        ],
    ))
}

/// One land use type of the full ALKIS: its class in the simplified schema, and the coded
/// property whose label is the `bez`.
#[derive(Debug, PartialEq, Eq)]
pub struct LandUseType {
    /// `AX_…`, without the namespace prefix.
    pub type_name: &'static str,
    pub nutzart: &'static str,
    pub bez: Option<&'static str>,
}

const fn land(
    type_name: &'static str,
    nutzart: &'static str,
    bez: Option<&'static str>,
) -> LandUseType {
    LandUseType {
        type_name,
        nutzart,
        bez,
    }
}

/// The land use types in the order of the AdV catalogue. A railway is named by its category
/// and an airfield by its kind, as the simplified services do.
pub const LAND_USE_TYPES: [LandUseType; 26] = [
    land("AX_Wohnbauflaeche", "Wohnbaufläche", None),
    land(
        "AX_IndustrieUndGewerbeflaeche",
        "Industrie- und Gewerbefläche",
        Some("funktion"),
    ),
    land("AX_Halde", "Halde", None),
    land("AX_Bergbaubetrieb", "Bergbaubetrieb", None),
    land(
        "AX_TagebauGrubeSteinbruch",
        "Tagebau, Grube, Steinbruch",
        None,
    ),
    land(
        "AX_FlaecheGemischterNutzung",
        "Fläche gemischter Nutzung",
        Some("funktion"),
    ),
    land(
        "AX_FlaecheBesondererFunktionalerPraegung",
        "Fläche besonderer funktionaler Prägung",
        Some("funktion"),
    ),
    land(
        "AX_SportFreizeitUndErholungsflaeche",
        "Sport-, Freizeit- und Erholungsfläche",
        Some("funktion"),
    ),
    land("AX_Friedhof", "Friedhof", Some("funktion")),
    land("AX_Strassenverkehr", "Straßenverkehr", Some("funktion")),
    land("AX_Weg", "Weg", Some("funktion")),
    land("AX_Platz", "Platz", Some("funktion")),
    land("AX_Bahnverkehr", "Bahnverkehr", Some("bahnkategorie")),
    land("AX_Flugverkehr", "Flugverkehr", Some("art")),
    land("AX_Schiffsverkehr", "Schiffsverkehr", Some("funktion")),
    land(
        "AX_Landwirtschaft",
        "Landwirtschaft",
        Some("vegetationsmerkmal"),
    ),
    land("AX_Wald", "Wald", Some("vegetationsmerkmal")),
    land("AX_Gehoelz", "Gehölz", None),
    land("AX_Heide", "Heide", None),
    land("AX_Moor", "Moor", None),
    land("AX_Sumpf", "Sumpf", None),
    land(
        "AX_UnlandVegetationsloseFlaeche",
        "Unland/Vegetationslose Fläche",
        Some("funktion"),
    ),
    land("AX_Fliessgewaesser", "Fließgewässer", Some("funktion")),
    land("AX_Hafenbecken", "Hafenbecken", Some("funktion")),
    land(
        "AX_StehendesGewaesser",
        "Stehendes Gewässer",
        Some("funktion"),
    ),
    land("AX_Meer", "Meer", Some("funktion")),
];

/// `istWeitereNutzung` of a use that lies over another one (a car park below a square): the
/// simplified schema holds the use underneath only.
const OVERLYING: &str = "1000";
/// The class of the squares, which get `square_bez` when they have no function.
const SQUARE: &str = "Platz";

/// A land use area of the full ALKIS in the simplified schema; `None` for any other feature
/// and for a use that lies over another one. A square without a function gets `square_bez`.
pub fn land_use(feature: &Value, square_bez: Option<&str>) -> Option<Value> {
    let nas_type = text(feature, "nas_type")?;
    let land = LAND_USE_TYPES
        .iter()
        .find(|land| land.type_name == nas_type)?;
    if text(feature, "istWeitereNutzung") == Some(OVERLYING) {
        return None;
    }
    let bez = land
        .bez
        .and_then(|property| label_of(nas_type, feature, property))
        .or_else(|| square_bez.filter(|_| land.nutzart == SQUARE));
    Some(in_schema(
        feature,
        LAND_USE_SUFFIX,
        vec![("nutzart", Some(land.nutzart)), ("bez", bez)],
    ))
}

/// The text of the property `name` of a feature as [`crate::nas::parse_nas_gml`] reads it.
fn text<'a>(feature: &'a Value, name: &str) -> Option<&'a str> {
    feature["properties"][name]
        .as_str()
        .filter(|text| !text.is_empty())
}

/// Whether the geometry of `feature` is an area.
fn is_area(feature: &Value) -> bool {
    matches!(
        feature["geometry"]["type"].as_str(),
        Some("Polygon" | "MultiPolygon")
    )
}

/// `feature` with the attributes `properties` of the simplified schema, its oid (the `gml:id`
/// and `suffix`) and the day its version began as `aktualit`.
fn in_schema(feature: &Value, suffix: &str, properties: Vec<(&str, Option<&str>)>) -> Value {
    let oid = format!("{}{suffix}", text(feature, "oid").unwrap_or_default());
    let mut schema = Map::new();
    schema.insert("oid".to_string(), Value::from(oid.clone()));
    for (name, value) in properties {
        if let Some(value) = value {
            schema.insert(name.to_string(), Value::from(value));
        }
    }
    if let Some(day) = text(feature, "beginnt") {
        schema.insert("aktualit".to_string(), Value::from(day));
    }
    serde_json::json!({
        "type": "Feature",
        "id": oid,
        "geometry": feature["geometry"],
        "properties": schema,
    })
}

/// The AdV code lists: type (`AX_…`) → code property → code → label.
type AdvCodes = BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>;

fn adv_codes() -> &'static AdvCodes {
    static CODES: OnceLock<AdvCodes> = OnceLock::new();
    CODES.get_or_init(|| {
        toml::from_str(include_str!("../../../assets/rules/adv-codes.toml"))
            .expect("embedded AdV code lists are valid")
    })
}

/// The label of `code` in the list of `property` of `nas_type`.
fn label(nas_type: &str, property: &str, code: &str) -> Option<&'static str> {
    adv_codes()
        .get(nas_type)?
        .get(property)?
        .get(code)
        .map(String::as_str)
}

/// The label of the code `feature` has as `property`, in the list of `nas_type`.
fn label_of(nas_type: &str, feature: &Value, property: &str) -> Option<&'static str> {
    label(nas_type, property, text(feature, property)?)
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    /// Recorded from the WFS of the full ALKIS of Bremen (2026-11-13), nine objects out of
    /// three answers: three buildings, two building parts and four structures.
    pub const BREMEN_BUILDINGS: &str =
        include_str!("../../../tests/fixtures/bremen-aaa-buildings.gml");
    /// Recorded from the same service, nine areas out of six answers.
    pub const BREMEN_LAND_USE: &str =
        include_str!("../../../tests/fixtures/bremen-aaa-land-use.gml");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::crs::Crs;
    use crate::fetch::test_support::MapFetcher;
    use crate::nas::parse_nas_gml;
    use crate::nas::test_support::EMPTY;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::fetch::FetchMode;
    use freemapper_adapter_api::records::AdapterId;

    const URL: &str = "https://opendata.lgln.niedersachsen.de/doorman/noauth/alkishb_wfs_sf";

    fn service() -> NasService {
        NasService {
            name: Cow::Borrowed("GeoBremen ALKIS"),
            url: Cow::Borrowed(URL),
            region: AdapterId::built_in("bremen"),
            crs: Crs::Utm32,
            max_count: 100_000,
            max_bytes: 256 << 20,
            layers: None,
            namespaces: None,
        }
    }

    /// Around the main station of Bremen (EPSG:25832).
    const STATION: BoundingBox = BoundingBox {
        min_e: 487_000.0,
        min_n: 5_881_500.0,
        max_e: 487_500.0,
        max_n: 5_882_000.0,
    };

    fn recorded(gml: &str) -> Vec<Value> {
        let answer = parse_nas_gml(gml).unwrap();
        assert!(answer.skipped.is_empty(), "{:?}", answer.skipped);
        answer.features
    }

    fn of_type<'a>(features: &'a [Value], nas_type: &str) -> Vec<&'a Value> {
        features
            .iter()
            .filter(|f| f["properties"]["nas_type"] == nas_type)
            .collect()
    }

    /// The answers to the requests for `type_name`.
    fn serve(fetcher: &MapFetcher, type_name: &str, answer: &str) {
        fetcher.insert_prefix(
            &format!("{URL}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}&"),
            answer.as_bytes().to_vec(),
        );
    }

    #[test]
    fn a_building_of_the_full_alkis_becomes_one_of_the_schema() {
        let features = recorded(BREMEN_BUILDINGS);
        let buildings: Vec<Value> = of_type(&features, "AX_Gebaeude")
            .into_iter()
            .map(|f| building(f).unwrap())
            .collect();
        assert_eq!(buildings.len(), 3);
        // A house: the oid of the simplified services, the label of the code, the day the
        // version began.
        let house = &buildings[0];
        assert_eq!(house["id"], "DEHB01ALn0001NxBBL");
        assert_eq!(house["geometry"]["type"], "Polygon");
        let properties = house["properties"].as_object().unwrap();
        assert_eq!(properties["oid"], "DEHB01ALn0001NxBBL");
        assert_eq!(properties["gebnutzbez"], "Gebäude");
        assert_eq!(properties["funktion"], "Wohnhaus");
        assert!(!properties.contains_key("rellage"), "{properties:?}");
        let day = properties["aktualit"].as_str().unwrap();
        assert_eq!(day.len(), 10, "{day}");
        assert!(day.parse::<chrono::NaiveDate>().is_ok(), "{day}");
        // Nothing of the full ALKIS is carried on.
        let mut names: Vec<&str> = properties.keys().map(String::as_str).collect();
        names.sort_unstable();
        assert_eq!(names, ["aktualit", "funktion", "gebnutzbez", "oid"]);
        // The position to the surface is the `rellage` that marks an underground building.
        assert_eq!(
            buildings[1]["properties"]["rellage"],
            "Unter der Erdoberfläche"
        );
        assert_eq!(buildings[2]["properties"]["rellage"], "Aufgeständert");
    }

    #[test]
    fn a_building_part_is_named_by_its_kind_of_construction() {
        let features = recorded(BREMEN_BUILDINGS);
        let parts: Vec<Value> = of_type(&features, "AX_Bauteil")
            .into_iter()
            .map(|f| building(f).unwrap())
            .collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["properties"]["gebnutzbez"], "Bauteil");
        assert_eq!(parts[0]["properties"]["funktion"], "Durchfahrt im Gebäude");
        assert_eq!(parts[0]["id"], "DEHB01ALT0001l57BL");
        assert_eq!(
            parts[1]["properties"]["funktion"],
            "Geringergeschossiger Gebäudeteil"
        );
    }

    /// Of the structures only the roofs are areas of the simplified schema; stairs, their
    /// bottom edges and monuments come with the layers of the full ALKIS.
    #[test]
    fn of_the_structures_only_a_roof_is_in_the_schema() {
        let features = recorded(BREMEN_BUILDINGS);
        let structures = of_type(&features, "AX_SonstigesBauwerkOderSonstigeEinrichtung");
        assert_eq!(structures.len(), 4);
        let kept: Vec<Value> = structures.iter().filter_map(|f| building(f)).collect();
        assert_eq!(kept.len(), 1);
        assert_eq!(
            kept[0]["properties"]["gebnutzbez"],
            "Sonstiges Bauwerk oder sonstige Einrichtung"
        );
        assert_eq!(kept[0]["properties"]["funktion"], "Überdachung");
        // A roof surveyed as a line is no area.
        let mut line = structures[0].clone();
        line["geometry"] =
            serde_json::json!({"type": "LineString", "coordinates": [[0.0, 0.0], [1.0, 1.0]]});
        assert_eq!(building(&line), None);
    }

    #[test]
    fn a_land_use_area_is_named_by_its_type_and_its_function() {
        let features = recorded(BREMEN_LAND_USE);
        let named = |nas_type: &str| -> Vec<(String, Option<String>)> {
            of_type(&features, nas_type)
                .into_iter()
                .filter_map(|f| land_use(f, None))
                .map(|f| {
                    let properties = &f["properties"];
                    (
                        properties["nutzart"].as_str().unwrap().to_string(),
                        properties["bez"].as_str().map(str::to_string),
                    )
                })
                .collect()
        };
        let some = |nutzart: &str, bez: &str| (nutzart.to_string(), Some(bez.to_string()));
        let none = |nutzart: &str| (nutzart.to_string(), None);
        assert_eq!(
            named("AX_Platz"),
            [some("Platz", "Parkplatz"), none("Platz")]
        );
        // The strips along the carriageway carry the text the build leaves out by.
        assert_eq!(
            named("AX_Strassenverkehr"),
            [
                some("Straßenverkehr", "Begleitfläche Straßenverkehr"),
                none("Straßenverkehr")
            ]
        );
        assert_eq!(
            named("AX_SportFreizeitUndErholungsflaeche"),
            [some("Sport-, Freizeit- und Erholungsfläche", "Park")]
        );
        // A railway by its category, not by its function.
        assert_eq!(
            named("AX_Bahnverkehr"),
            [some("Bahnverkehr", "Straßenbahn")]
        );
        assert_eq!(named("AX_Wohnbauflaeche"), [none("Wohnbaufläche")]);
        // Of the two areas of special character one lies over another use: left out.
        assert_eq!(
            named("AX_FlaecheBesondererFunktionalerPraegung"),
            [some(
                "Fläche besonderer funktionaler Prägung",
                "Bildung und Wissenschaft"
            )]
        );
        let area = land_use(of_type(&features, "AX_Platz")[0], None).unwrap();
        assert_eq!(area["id"], "DEHB01ALT00026AdTN");
        assert_eq!(area["properties"]["oid"], "DEHB01ALT00026AdTN");
        assert_eq!(area["properties"]["aktualit"], "2018-03-13");
        assert_eq!(area["geometry"]["type"], "Polygon");
    }

    #[test]
    fn a_square_without_a_function_gets_the_bez_of_the_state() {
        let features = recorded(BREMEN_LAND_USE);
        let squares = of_type(&features, "AX_Platz");
        let bez = |feature: &Value| {
            land_use(feature, Some("ohne Unterart (Bremen)")).unwrap()["properties"]["bez"].clone()
        };
        assert_eq!(bez(squares[0]), "Parkplatz");
        assert_eq!(bez(squares[1]), "ohne Unterart (Bremen)");
        // Only a square: a street without a function has none.
        let street = of_type(&features, "AX_Strassenverkehr")[1];
        assert_eq!(bez(street), Value::Null);
    }

    #[test]
    fn a_feature_of_another_type_is_neither() {
        let features = recorded(BREMEN_BUILDINGS);
        assert_eq!(land_use(&features[0], None), None);
        let features = recorded(BREMEN_LAND_USE);
        assert_eq!(building(&features[0]), None);
    }

    /// The classes are those of the simplified schema, in the order of the catalogue, and
    /// every list a type names is in the table.
    #[test]
    fn every_land_use_type_has_its_class_and_its_list() {
        let classes: Vec<&str> = LAND_USE_TYPES.iter().map(|land| land.nutzart).collect();
        assert_eq!(classes, crate::alkis_source::LAND_USE_CLASSES);
        for land in &LAND_USE_TYPES {
            if let Some(property) = land.bez {
                let list = adv_codes()
                    .get(land.type_name)
                    .and_then(|lists| lists.get(property));
                assert!(
                    list.is_some_and(|list| !list.is_empty()),
                    "{} {property}",
                    land.type_name
                );
            }
        }
        assert_eq!(
            label("AX_Gebaeude", "gebaeudefunktion", "2463"),
            Some("Garage")
        );
        assert_eq!(label("AX_Bauteil", "bauart", "2400"), Some("Arkade"));
        assert_eq!(
            label(
                "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                "bauwerksfunktion",
                "1611"
            ),
            Some("Carport")
        );
        // The texts the build compares.
        for (nas_type, property, code, text) in [
            (
                "AX_Strassenverkehr",
                "funktion",
                "2312",
                "Begleitfläche Straßenverkehr",
            ),
            ("AX_Platz", "funktion", "5310", "Parkplatz"),
            ("AX_Platz", "funktion", "5340", "Marktplatz"),
            ("AX_Platz", "funktion", "5130", "Fußgängerzone"),
            (
                "AX_SportFreizeitUndErholungsflaeche",
                "funktion",
                "4400",
                "Grünanlage",
            ),
            (
                "AX_SportFreizeitUndErholungsflaeche",
                "funktion",
                "4410",
                "Siedlungsgrünfläche",
            ),
            (
                "AX_SportFreizeitUndErholungsflaeche",
                "funktion",
                "4440",
                "Kleingarten",
            ),
            (
                "AX_Landwirtschaft",
                "vegetationsmerkmal",
                "1020",
                "Grünland",
            ),
            ("AX_Bauteil", "bauart", "2610", "Durchfahrt im Gebäude"),
            (
                "AX_Bauteil",
                "bauart",
                "2620",
                "Durchfahrt an überbautem Verkehrsweg",
            ),
        ] {
            assert_eq!(label(nas_type, property, code), Some(text));
        }
    }

    #[test]
    fn a_code_the_lists_do_not_know_leaves_the_text_out() {
        let mut feature = recorded(BREMEN_BUILDINGS).remove(0);
        feature["properties"]["gebaeudefunktion"] = "0007".into();
        let properties = building(&feature).unwrap()["properties"].clone();
        assert_eq!(properties["gebnutzbez"], "Gebäude");
        assert_eq!(properties.get("funktion"), None);
    }

    #[test]
    fn the_buildings_are_asked_type_by_type() {
        let fetcher = MapFetcher::default();
        for type_name in BUILDING_TYPES {
            serve(&fetcher, type_name, BREMEN_BUILDINGS);
        }
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.gml.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let fetched = fetch_buildings(&fetcher, &service(), &STATION, &part).unwrap();
        assert_eq!(fetcher.calls.get(), 3);
        // The recorded file holds all three types, so each request brings the six of the
        // schema.
        assert_eq!(fetched.features.len(), 18);
        assert!(fetched.skipped.is_empty());
        let log = fetcher.log.borrow();
        assert!(
            log[0].starts_with(&format!(
                "{URL}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=adv:AX_Gebaeude\
                 &SRSNAME=urn:ogc:def:crs:EPSG::25832&BBOX=487000,5881500,487500,5882000,"
            )),
            "{log:?}"
        );
        assert!(log.iter().all(|url| url.contains("&COUNT=100000")));
    }

    #[test]
    fn the_land_use_is_asked_for_every_type() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(URL, EMPTY.as_bytes().to_vec());
        serve(&fetcher, "adv:AX_Platz", BREMEN_LAND_USE);
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.gml.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let fetched = fetch_land_use(
            &fetcher,
            &service(),
            Some("ohne Unterart (Bremen)"),
            &STATION,
            &part,
        )
        .unwrap();
        assert_eq!(fetcher.calls.get(), 26);
        // Nine recorded areas, one of them over another use.
        assert_eq!(fetched.features.len(), 8);
        assert_eq!(
            fetched.features[1]["properties"]["bez"],
            "ohne Unterart (Bremen)"
        );
        let log = fetcher.log.borrow();
        for land in &LAND_USE_TYPES {
            let asked = format!("TYPENAMES=adv:{}&", land.type_name);
            assert!(log.iter().any(|url| url.contains(&asked)), "{asked}");
        }
    }

    #[test]
    fn a_type_that_fails_fails_the_fetch() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(URL, EMPTY.as_bytes().to_vec());
        fetcher.refusing.borrow_mut().push(format!(
            "{URL}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=adv:AX_Weg&"
        ));
        let scratch = tempfile::tempdir().unwrap();
        let part_path = scratch.path().join("answer.gml.part");
        let part = || -> Result<PathBuf, FetchError> { Ok(part_path.clone()) };
        let failed = fetch_land_use(&fetcher, &service(), None, &STATION, &part);
        assert!(failed.is_err(), "{failed:?}");
    }

    /// The source of an adapter: answers in the CRS of the service, streams through the cache
    /// folder of its provider and says which objects the reader could not read.
    #[test]
    fn the_source_answers_in_the_schema_and_names_what_it_could_not_read() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(URL, EMPTY.as_bytes().to_vec());
        serve(&fetcher, "adv:AX_Gebaeude", BREMEN_BUILDINGS);
        // A building whose outline is a spline, which the reader does not know.
        let odd = r#"<?xml version="1.0" encoding="utf-8"?><wfs:FeatureCollection numberReturned="1" numberMatched="unknown" xmlns="http://www.adv-online.de/namespaces/adv/gid/7.1" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:wfs="http://www.opengis.net/wfs/2.0"><wfs:member><AX_Bauteil gml:id="DEHB01ODD0000001"><position><gml:Curve><gml:segments><gml:CubicSpline><gml:posList>0 0 1 1 2 0</gml:posList></gml:CubicSpline></gml:segments></gml:Curve></position><bauart>2610</bauart></AX_Bauteil></wfs:member></wfs:FeatureCollection>"#;
        serve(&fetcher, "adv:AX_Bauteil", odd);
        serve(&fetcher, "adv:AX_Platz", BREMEN_LAND_USE);
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().join("cache"));
        let registry = freemapper_adapter_api::registry::Registry::new();
        let mut notes = Vec::new();
        let mut progress = |note: Note| notes.push(note);
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: "2026-11-13".parse().unwrap(),
            skip_surface: false,
            progress: &mut progress,
        };
        let source = AaaAlkis {
            service: service(),
            square_bez: Some(Cow::Borrowed("ohne Unterart (Bremen)")),
            cache_dir: Cow::Borrowed("geobremen"),
        };
        let area = FetchArea::new(STATION, Crs::Utm32);
        let buildings = source.buildings(&mut env, &area).unwrap();
        assert_eq!(buildings.crs, Crs::Utm32);
        assert_eq!(buildings.features.len(), 6);
        let land_use = source.land_use(&mut env, &area).unwrap();
        assert_eq!(land_use.crs, Crs::Utm32);
        assert_eq!(land_use.features.len(), 8);
        assert_eq!(notes.len(), 1, "{notes:?}");
        let shown = format!("{:?}", notes[0]);
        assert!(shown.contains("progress-alkis-skipped"), "{shown}");
        assert!(shown.contains("DEHB01ODD0000001 (AX_Bauteil)"), "{shown}");
        // The answers streamed through the folder of the provider, and none is left there.
        let folder = dir.path().join("cache").join("geobremen");
        assert!(folder.is_dir());
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
    }
}
