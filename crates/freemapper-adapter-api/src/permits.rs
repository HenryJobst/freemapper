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
    },
    /// Forest with a function or a protection of forest law.
    ForestFunction {
        kind: ForestFunctionKind,
        /// The name of the area, or what the data say about it ("von hoher Bedeutung").
        name: Option<String>,
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

/// The kinds of protected object the national sources do not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtectedObjectKind {
    /// A Naturdenkmal: a tree, a boulder or a small area (§ 28 BNatSchG).
    NaturalMonument,
    /// A geschützter Landschaftsbestandteil (§ 29 BNatSchG).
    ProtectedLandscapePart,
    /// An area with special rules inside a protected area.
    SpecialRules,
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
    /// Whether the lower nature conservation authority looks after the object itself.
    pub fn needs_nature_authority(self) -> bool {
        matches!(
            self,
            ProtectedObjectKind::NaturalMonument | ProtectedObjectKind::ProtectedLandscapePart
        )
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
                },
                crs: Crs::Utm33,
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
}
