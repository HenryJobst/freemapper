// SPDX-License-Identifier: AGPL-3.0-or-later

//! Sources of the permit notes a region adapter brings (ADR 0120): services of a state that say
//! more about a map area than the national sources do, such as the Bezirk of a city state, its
//! green spaces, the land the state owns, the monuments, the protected objects and the offices
//! of the forest administration (ADR 0124).
//!
//! An adapter names its sources ([`crate::adapter::RegionAdapter::permit_sources`]); the engine
//! asks them, behind the hosts of the adapter, only when permit notes are asked for and the map
//! area touches the ground of the adapter. A source fetches raw GeoJSON features, one list per
//! layer, and turns stored features into typed [`PermitFinding`]s. What a finding means is
//! the same for every region; only the source knows the attribute names of its service.

use std::path::Path;

use geo_types::{Geometry, Rect};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cache::CacheDir;
use crate::error::FetchError;
use crate::fetch::Fetcher;

/// A service of a region adapter that gives findings for the permit notes.
pub trait PermitSource: Send + Sync {
    /// What the engine needs to store the layers and to name the source.
    fn info(&self) -> &PermitSourceInfo;

    /// The features of every layer of [`PermitSourceInfo::layers`] inside `bbox` (WGS 84,
    /// x = longitude, y = latitude), one list per layer in that order. Each answer may stream
    /// into the file `part`. Stops at the first layer that fails: a service that is down is not
    /// asked again for its other layers.
    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError>;

    /// [`Self::fetch`] with the shared download cache at hand (ADR 0132), what the engine
    /// calls. A source that downloads a file for the whole state keeps it there and asks the
    /// server again only for a newer one; every other source fetches as [`Self::fetch`] does.
    fn fetch_cached(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
        _cache: &CacheDir,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        self.fetch(fetcher, bbox, part)
    }

    /// The findings of the stored features of the layer `layer` (a key of
    /// [`PermitSourceInfo::layers`]), geometries in WGS 84 with x = longitude. A feature that
    /// says nothing usable is left out.
    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding>;
}

/// The facts of a [`PermitSource`].
#[derive(Debug, Clone, PartialEq)]
pub struct PermitSourceInfo {
    /// The key of the source: a plain name (lower-case letters, digits, `-`), unique within the
    /// adapter. The engine stores its layers as `<key>-<layer>.geojson`.
    pub key: &'static str,
    /// The name of the publisher, the same for all sources of one portal ("Geoportal Berlin");
    /// the notes give the sources of one name one line in their table.
    pub label: Wording,
    /// What the source holds, short ("district borders").
    pub what: Wording,
    /// The credit line or licence text, verbatim as the publisher gives it.
    pub credit: &'static str,
    /// A note of the publisher on what the data are worth, verbatim; `None` when it has none.
    pub note: Option<&'static str>,
    /// The source answers only at points (ADR 0139): its findings are the cells around the
    /// points it was asked at, so a share is an estimate and what lies between the points can
    /// be missed. The notes say so.
    pub sampled: bool,
    /// Where the data of the source are not complete although they cover its state ("the city
    /// of Potsdam keeps its own list"), in the words of the notes; `None` for complete data
    /// (ADR 0144).
    pub gaps: Option<Wording>,
    /// The keys of the layers (plain names), in the order of [`PermitSource::fetch`].
    pub layers: &'static [&'static str],
    /// The sections of the notes the findings of the source fill. A section is left out of the
    /// notes when one source that fills it could not be read.
    pub sections: &'static [PermitSection],
    /// For a source of districts: whose offices the hints of its districts name (ADR 0129).
    /// `None` for every other source.
    pub offices: Option<DistrictOffices>,
}

/// The offices of a city state's districts, as the hints name them: each city state splits the
/// tasks between its Bezirksämter and its central offices in its own way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DistrictOffices {
    /// Berlin: Ordnungsamt and Straßen- und Grünflächenamt of the Bezirksamt, and its lower
    /// monument and nature conservation authorities.
    Berlin,
    /// Hamburg: the Bezirksamt with its Fachamt Management des öffentlichen Raumes; the
    /// monuments are the central Denkmalschutzamt's.
    Hamburg,
}

impl DistrictOffices {
    /// Whether one central office looks after the monuments of all districts, so the notes
    /// name it once and not per district.
    pub fn central_monuments(self) -> bool {
        matches!(self, DistrictOffices::Hamburg)
    }

    /// The name of the offices in the arguments of a hint, as `permits.json` stores it.
    pub fn key(self) -> &'static str {
        match self {
            DistrictOffices::Berlin => "berlin",
            DistrictOffices::Hamburg => "hamburg",
        }
    }
}

/// A text in the languages of the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wording {
    pub en: &'static str,
    pub de: &'static str,
}

/// The sections of the permit notes a regional source can fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermitSection {
    /// The districts of a city state (Bezirke).
    Districts,
    /// Public green spaces, and playgrounds as a count.
    GreenSpaces,
    /// Land of public owners.
    Owners,
    Monuments,
    /// Protected objects the national sources do not have.
    ProtectedObjects,
    /// The offices of the forest administration and their units (ADR 0124).
    ForestUnits,
    /// The forest the state owns.
    StateForest,
    /// Water protection areas with their zone (ADR 0129).
    WaterProtection,
    /// Forest with a function or a protection of forest law: Erholungswald, Bannwald,
    /// Naturwald, areas closed to the public.
    ForestFunctions,
    /// The administrative units the area lies in (ADR 0181): where the national VG250 does not
    /// apply, a region brings its own.
    AdminUnits,
    /// The register of monuments of a region that gives no position (ADR 0184): the monuments
    /// in the rectangle around the map area, counted, with their names.
    MonumentRegister,
}

/// One finding of a regional source with its geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct PermitFinding {
    pub kind: FindingKind,
    /// WGS 84, x = longitude, y = latitude: an area, or a point for an object.
    pub geometry: Geometry<f64>,
}

/// What a finding says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingKind {
    /// A district with an office of its own (a Bezirk with its Bezirksamt).
    District { name: String },
    /// A public green space.
    GreenSpace {
        name: String,
        /// The district whose office looks after it.
        district: Option<String>,
        /// Whether it is dedicated for public use, as the data write it ("gewidmet").
        dedication: Option<String>,
    },
    /// A public playground; the notes count them.
    Playground,
    /// A parcel of a public owner. `owner` is the name as the data write it ("Land Berlin")
    /// for an owner the source names; `None` for another public owner, whose name the notes
    /// leave out. A source names only owners from a fixed list of public bodies, so the notes
    /// never carry the name of a person, whatever the data hold.
    OwnerShare { owner: Option<String> },
    /// A listed monument.
    Monument {
        /// The kind as the data write it ("Baudenkmal", "Gartendenkmal").
        kind: String,
        /// The entry of the monument in the list of the state.
        link: Option<String>,
    },
    /// A monument of a register that gives no position: it lies somewhere in the rectangle the
    /// source asked, which the finding's geometry then is (ADR 0184). The name as the register
    /// writes it; `None` when it writes none.
    RegisterMonument { name: Option<String> },
    /// The unit of a forest administration the area belongs to. All three as the notes print
    /// them, proper names in German in every language.
    ForestUnit {
        /// The administration ("Landesbetrieb Forst Brandenburg").
        administration: String,
        /// Its office in charge ("Oberförsterei Barnim").
        office: String,
        /// The unit below the office ("Revier Werneuchen").
        unit: Option<String>,
    },
    /// Forest of the state. `owner` is the body as the notes name it ("Bundesland
    /// Nordrhein-Westfalen"), `office` the forest office that looks after it.
    StateForest {
        owner: String,
        office: Option<String>,
    },
    /// A water protection area or one of its zones.
    WaterProtection {
        /// The name of the area ("Baursberg").
        name: Option<String>,
        /// The zone as the data write it ("Schutzzone III", "Zone IIIB").
        zone: Option<String>,
        /// The ordinance, as the data cite it.
        citation: Option<String>,
        /// The authority in charge of the area, as the data name it ("Landratsamt
        /// Ludwigsburg"); `None` when the data name none (ADR 0143).
        authority: Option<String>,
    },
    /// Forest with a function or a protection of forest law.
    ForestFunction {
        kind: ForestFunctionKind,
        /// The name of the area, or what the data say about it ("von hoher Bedeutung").
        name: Option<String>,
    },
    /// An administrative unit the area lies in, the name as the register writes it, with the
    /// code of the register (ADR 0181).
    AdminUnit {
        level: AdminUnitLevel,
        name: String,
        code: String,
    },
    /// A protected object of nature conservation law.
    ProtectedObject {
        kind: ProtectedObjectKind,
        /// The name of the object or, for a tree, its species.
        name: Option<String>,
        /// The legal act, as the data cite it.
        citation: Option<String>,
    },
}

/// The levels of administrative unit a region brings, top down (ADR 0181).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdminUnitLevel {
    /// The first level below the state (Polish województwo).
    Region,
    /// The second level (Polish powiat).
    County,
    /// The municipality (Polish gmina).
    Municipality,
}

/// The kinds of protected object the national sources do not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtectedObjectKind {
    /// A Naturdenkmal: a tree, a boulder or a small area (§ 28 BNatSchG).
    NaturalMonument,
    /// A geschützter Landschaftsbestandteil (§ 29 BNatSchG).
    ProtectedLandscapePart,
    /// A gesetzlich geschütztes Biotop (§ 30 BNatSchG or the state's law); its name is the type
    /// of biotope (ADR 0146).
    Biotope,
    /// An area with special rules inside a protected area.
    SpecialRules,
    /// A Polish national park (park narodowy), without its buffer zone (ADR 0175).
    NationalPark,
    /// The buffer zone (otulina) of a Polish national or landscape park.
    ParkBufferZone,
    /// A Polish landscape park (park krajobrazowy).
    LandscapePark,
    /// A Polish nature reserve (rezerwat przyrody).
    NatureReserve,
    /// A Polish protected landscape area (obszar chronionego krajobrazu).
    ProtectedLandscapeArea,
    /// A Natura 2000 site for habitats (special area of conservation, `PLH` or `PLC`).
    Natura2000Habitats,
    /// A Natura 2000 site for birds (special protection area, `PLB`).
    Natura2000Birds,
    /// A Polish ecological site (użytek ekologiczny).
    EcologicalSite,
    /// A Polish nature and landscape complex (zespół przyrodniczo-krajobrazowy).
    LandscapeComplex,
    /// A Polish documentation site (stanowisko dokumentacyjne): a geological or similar
    /// site of scientific interest.
    DocumentationSite,
    /// A Polish natural monument (pomnik przyrody): a tree, a boulder or a small area. Not
    /// [`Self::NaturalMonument`], whose notes name the German term and law.
    NaturalMonumentPl,
}

/// The functions and protections of forest the notes know.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ForestFunctionKind {
    /// Erholungswald: forest declared or mapped for recreation.
    Recreation,
    /// Bannwald or Schutzwald: forest that may not be cleared.
    Protected,
    /// Naturwald, Naturwaldzelle, Totalreservat, Wildnisentwicklungsgebiet: left to itself.
    Natural,
    /// An area of forest closed to the public.
    Closed,
}

/// Whether a water protection zone is one of the inner zones (I or II), where events are
/// usually ruled out; `None` when the text names no zone the notes can read. "IIIA", "III B"
/// and "Zone 3" are outer zones.
pub fn inner_zone(zone: &str) -> Option<bool> {
    let upper = zone.to_uppercase();
    let words: Vec<&str> = upper
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let level = words.iter().rev().find_map(|word| {
        let roman = word.trim_end_matches(['A', 'B', 'C']);
        match roman {
            "I" | "1" => Some(1),
            "II" | "2" => Some(2),
            "III" | "3" | "IV" | "4" => Some(3),
            _ => None,
        }
    })?;
    Some(level <= 2)
}

impl ProtectedObjectKind {
    /// Whether the lower nature conservation authority looks after the object itself. Every
    /// Polish kind counts: the notes name the park administration and the regional
    /// directorate for them (ADR 0175).
    pub fn needs_nature_authority(self) -> bool {
        !matches!(self, ProtectedObjectKind::SpecialRules)
    }
}

/// Whether `name` can be the key of a source or a layer: lower-case letters, digits and `-`,
/// starting with a letter or a digit. It becomes part of a file name.
pub fn plain_key(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A WFS permit source as a manifest declares it (`[[permits.wfs]]`, issue #273): the same table
/// the code of a state wrote in Rust, as owned data. A [`PermitSourceFactory`] turns it into a
/// source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredWfs {
    /// The key of the source, as [`PermitSourceInfo::key`].
    pub key: String,
    pub label: DeclaredWording,
    pub what: DeclaredWording,
    pub credit: String,
    /// Where the data are not complete although they cover the state; see
    /// [`PermitSourceInfo::gaps`].
    pub gaps: Option<DeclaredWording>,
    pub sections: Vec<PermitSection>,
    /// The address of the service.
    pub url: String,
    pub output_format: String,
    /// The service answers GML 3.2 only (ADR 0137).
    pub gml: bool,
    /// The EPSG number of the CRS the service is asked and answers in, where it is not WGS 84
    /// (ADR 0144).
    pub native_crs: Option<u32>,
    pub layers: Vec<DeclaredLayer>,
}

/// [`Wording`] as owned text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredWording {
    pub en: String,
    pub de: String,
}

/// One layer of a declared source: its key, its WFS type and how its features are read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredLayer {
    pub key: String,
    pub type_name: String,
    pub reading: DeclaredReading,
}

/// How the features of a declared layer become findings, with the attributes they read; the
/// kinds are those of the table of a state (ADR 0130).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclaredReading {
    /// A water protection area or zone.
    Water {
        name: String,
        zone: String,
        citation: Option<String>,
        /// The attribute and the value that mark a planned zone.
        planned: Option<(String, String)>,
    },
    /// Forest with a function or protection.
    Function {
        kind: ForestFunctionKind,
        name: Option<String>,
    },
    /// A unit of a forest administration.
    Unit {
        administration: String,
        prefix: String,
        office: String,
    },
    /// Forest of the state.
    StateForest {
        owner: String,
        prefix: String,
        office: String,
    },
    /// A protected object of nature conservation law (ADR 0146).
    Object {
        kind: ProtectedObjectKind,
        name: Option<String>,
        citation: Option<String>,
        /// The attribute and the values that make a feature count.
        only: Option<(String, Vec<String>)>,
    },
    /// A listed monument.
    Monument {
        kind: String,
        kind_from: Option<String>,
        /// The attribute of the link and the start it must have.
        link: Option<(String, String)>,
    },
}

/// Makes the sources a manifest declares. The adapter crate that reads the permit services
/// registers one in the registry; an adapter whose manifest declares permit sources needs it.
pub trait PermitSourceFactory: Send + Sync {
    /// The source of `declared`.
    fn wfs(&self, declared: &DeclaredWfs) -> Box<dyn PermitSource>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::RegionAdapter;
    use crate::credit::{CreditLine, UsedSources};
    use crate::estimate::{EstimateArea, EstimateEnv};
    use crate::info::{Names, RegionInfo};
    use crate::records::{AdapterId, EstimateItem};
    use freemapper_georef::crs::Crs;
    use geo_types::MultiPolygon;

    /// An adapter that says only what it must.
    struct Bare {
        info: RegionInfo,
        boundary: MultiPolygon<f64>,
    }

    impl RegionAdapter for Bare {
        fn info(&self) -> &RegionInfo {
            &self.info
        }
        fn boundary(&self) -> &MultiPolygon<f64> {
            &self.boundary
        }
        fn alkis(&self) -> Option<&dyn crate::alkis::AlkisSource> {
            None
        }
        fn attribution(&self, _used: &UsedSources<'_>) -> Vec<CreditLine> {
            Vec::new()
        }
        fn rasters(&self) -> Option<&dyn crate::rasters::RasterBackend> {
            None
        }
        fn estimate(&self, _area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
            Vec::new()
        }
    }

    #[test]
    fn an_adapter_brings_no_permit_source_unless_it_says_so() {
        let id = AdapterId::built_in("bare");
        let bare = Bare {
            info: RegionInfo {
                id: id.clone(),
                provider: id,
                names: Names {
                    en: "Bare".into(),
                    de: "Kahl".into(),
                    pl: None,
                },
                crs: Crs::Utm33,
                country: crate::info::Country::De,
                coverage: crate::info::Coverage::Full,
                object_id_prefix: None,
                osm: Vec::new(),
                credit_short: None,
                preview: None,
            },
            boundary: MultiPolygon::new(vec![]),
        };
        assert!(bare.permit_sources().is_empty());
    }

    #[test]
    fn keys_are_plain_names() {
        assert!(plain_key("bezirke") && plain_key("a-nd") && plain_key("0x"));
        for bad in ["", "Bezirke", "a_nd", "../x", "a.b", "-a", "a b"] {
            assert!(!plain_key(bad), "{bad}");
        }
    }

    #[test]
    fn inner_zones_are_one_and_two() {
        for inner in ["Schutzzone I", "Zone II", "II", "Zone 2", "Schutzzone IIA"] {
            assert_eq!(inner_zone(inner), Some(true), "{inner}");
        }
        for outer in ["Schutzzone III", "Zone IIIB", "III A", "Zone 3", "IV"] {
            assert_eq!(inner_zone(outer), Some(false), "{outer}");
        }
        for unknown in ["", "WSG", "Zone"] {
            assert_eq!(inner_zone(unknown), None, "{unknown}");
        }
    }

    #[test]
    fn natural_monuments_and_landscape_parts_name_the_nature_authority() {
        assert!(ProtectedObjectKind::NaturalMonument.needs_nature_authority());
        assert!(ProtectedObjectKind::ProtectedLandscapePart.needs_nature_authority());
        assert!(!ProtectedObjectKind::SpecialRules.needs_nature_authority());
    }

    #[test]
    fn every_polish_kind_names_the_nature_authority() {
        for kind in [
            ProtectedObjectKind::NationalPark,
            ProtectedObjectKind::ParkBufferZone,
            ProtectedObjectKind::LandscapePark,
            ProtectedObjectKind::NatureReserve,
            ProtectedObjectKind::ProtectedLandscapeArea,
            ProtectedObjectKind::Natura2000Habitats,
            ProtectedObjectKind::Natura2000Birds,
            ProtectedObjectKind::EcologicalSite,
            ProtectedObjectKind::LandscapeComplex,
            ProtectedObjectKind::DocumentationSite,
            ProtectedObjectKind::NaturalMonumentPl,
        ] {
            assert!(kind.needs_nature_authority(), "{kind:?}");
        }
    }

    #[test]
    fn the_names_of_the_kinds_in_permits_json_are_kebab_case() {
        let name = |kind| serde_json::to_value(kind).unwrap();
        assert_eq!(name(ProtectedObjectKind::NationalPark), "national-park");
        assert_eq!(
            name(ProtectedObjectKind::Natura2000Birds),
            "natura2000-birds"
        );
        assert_eq!(
            name(ProtectedObjectKind::ProtectedLandscapeArea),
            "protected-landscape-area"
        );
        assert_eq!(
            name(ProtectedObjectKind::NaturalMonument),
            "natural-monument"
        );
    }
}
