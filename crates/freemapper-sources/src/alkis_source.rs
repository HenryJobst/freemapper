//! Ready-made ALKIS sources for an adapter: a classic "ALKIS vereinfacht" WFS, and an OGC API
//! Features service with an optional WFS of the same data behind it. Both deliver features in
//! the LGB schema.

use std::borrow::Cow;

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use serde_json::Value;

use crate::crs::Crs;
use crate::ogc_api::collection_features_in;
use crate::wfs::{self, WfsService};

/// The ALKIS of a state from its WFS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfsAlkis {
    pub service: WfsService,
}

impl AlkisSource for WfsAlkis {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let crs = self.service.crs;
        let features = wfs::fetch_buildings(env.fetcher, &self.service, &area.bbox_for(crs))?;
        Ok(AlkisPart { features, crs })
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let crs = self.service.crs;
        let features = wfs::fetch_land_use(env.fetcher, &self.service, &area.bbox_for(crs))?;
        Ok(AlkisPart { features, crs })
    }
}

/// How the features of an OGC API become features in the LGB spellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normaliser {
    /// The service writes the LGB spellings itself.
    Lgb,
    /// The service writes the class names in ASCII (Hamburg: `Wohnbauflaeche`).
    HamburgAscii,
}

impl Normaliser {
    fn building(self, feature: Value) -> Value {
        match self {
            Self::Lgb => feature,
            Self::HamburgAscii => normalise_building(feature),
        }
    }

    fn land_use(self, feature: Value) -> Value {
        match self {
            Self::Lgb => feature,
            Self::HamburgAscii => normalise_land_use(feature),
        }
    }
}

/// The ALKIS of a state from its OGC API Features service ("ALKIS vereinfacht" collections).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OgcApiAlkis {
    /// The items endpoint of the buildings collection.
    pub buildings: Cow<'static, str>,
    /// The items endpoint of the land use collection.
    pub land_use: Cow<'static, str>,
    /// The CRS asked for and answered in.
    pub crs: Crs,
    /// The classic WFS with the same data, in the same CRS: asked when the OGC API fails. When
    /// both fail, the error of the OGC API is reported.
    pub fallback: Option<WfsService>,
    pub normalise: Normaliser,
}

impl OgcApiAlkis {
    /// The features of `endpoint` in `area`, or what `from_wfs` reads from the fallback.
    fn features(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
        endpoint: &str,
        from_wfs: impl FnOnce(&WfsService) -> Result<Vec<Value>, crate::fetch::FetchError>,
    ) -> Result<Vec<Value>, AdapterError> {
        let bbox = area.bbox_for(self.crs);
        let features = collection_features_in(env.fetcher, endpoint, &bbox, self.crs).or_else(
            |e| match &self.fallback {
                Some(service) => from_wfs(service).map_err(|_| e),
                None => Err(e),
            },
        )?;
        Ok(features)
    }
}

impl AlkisSource for OgcApiAlkis {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let (fetcher, bbox) = (env.fetcher, area.bbox_for(self.crs));
        let features = self.features(env, area, &self.buildings, |service| {
            wfs::fetch_buildings(fetcher, service, &bbox)
        })?;
        Ok(AlkisPart {
            features: features
                .into_iter()
                .map(|f| self.normalise.building(f))
                .collect(),
            crs: self.crs,
        })
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let (fetcher, bbox) = (env.fetcher, area.bbox_for(self.crs));
        let features = self.features(env, area, &self.land_use, |service| {
            wfs::fetch_land_use(fetcher, service, &bbox)
        })?;
        Ok(AlkisPart {
            features: features
                .into_iter()
                .map(|f| self.normalise.land_use(f))
                .collect(),
            crs: self.crs,
        })
    }
}

/// The LGB spellings of `gebnutzbez` the build rules know.
const BUILDING_KINDS: [&str; 3] = [
    "Gebäude",
    "Bauteil",
    "Sonstiges Bauwerk oder sonstige Einrichtung",
];
/// The LGB spellings of `nutzart` (the AdV class names).
const LAND_USE_CLASSES: [&str; 26] = [
    "Wohnbaufläche",
    "Industrie- und Gewerbefläche",
    "Halde",
    "Bergbaubetrieb",
    "Tagebau, Grube, Steinbruch",
    "Fläche gemischter Nutzung",
    "Fläche besonderer funktionaler Prägung",
    "Sport-, Freizeit- und Erholungsfläche",
    "Friedhof",
    "Straßenverkehr",
    "Weg",
    "Platz",
    "Bahnverkehr",
    "Flugverkehr",
    "Schiffsverkehr",
    "Landwirtschaft",
    "Wald",
    "Gehölz",
    "Heide",
    "Moor",
    "Sumpf",
    "Unland/Vegetationslose Fläche",
    "Fließgewässer",
    "Hafenbecken",
    "Stehendes Gewässer",
    "Meer",
];

/// A Hamburg building feature with the LGB spellings (`Gebaeude` → `Gebäude`).
pub fn normalise_building(feature: Value) -> Value {
    normalise(feature, "gebnutzbez", &BUILDING_KINDS)
}

/// A Hamburg land use feature with the LGB `nutzart` spellings (`Wohnbauflaeche` →
/// `Wohnbaufläche`); unknown classes are kept.
pub fn normalise_land_use(feature: Value) -> Value {
    normalise(feature, "nutzart", &LAND_USE_CLASSES)
}

/// The land use class `text` in the LGB spelling (`Strassenverkehr` → `Straßenverkehr`); an
/// unknown class as it is.
pub fn land_use_class(text: &str) -> String {
    let key = spelling_key(text);
    LAND_USE_CLASSES
        .iter()
        .find(|name| spelling_key(name) == key)
        .map_or_else(|| text.to_string(), |name| (*name).to_string())
}

/// `feature` with `property` replaced by the entry of `names` it spells in ASCII.
fn normalise(mut feature: Value, property: &str, names: &[&str]) -> Value {
    let Some(value) = feature
        .get_mut("properties")
        .and_then(|p| p.get_mut(property))
    else {
        return feature;
    };
    if let Some(text) = value.as_str() {
        let key = spelling_key(text);
        if let Some(name) = names.iter().find(|name| spelling_key(name) == key) {
            *value = Value::from(*name);
        }
    }
    feature
}

/// A key equal for "Industrie- und Gewerbefläche" and "Industrie Und Gewerbeflaeche": lower
/// case, umlauts and ß written out, punctuation dropped, single spaces.
pub fn spelling_key(text: &str) -> String {
    let mut key = String::new();
    for c in text.to_lowercase().chars() {
        match c {
            'ä' => key.push_str("ae"),
            'ö' => key.push_str("oe"),
            'ü' => key.push_str("ue"),
            'ß' => key.push_str("ss"),
            '-' | ',' | '/' => key.push(' '),
            c => key.push(c),
        }
    }
    key.split_whitespace().collect::<Vec<_>>().join(" ")
}
