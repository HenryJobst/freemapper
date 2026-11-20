// SPDX-License-Identifier: AGPL-3.0-or-later

//! The static facts of a region adapter.

use freemapper_georef::crs::Crs;

use crate::records::AdapterId;

/// Where Geofabrik keeps its extracts.
const GEOFABRIK: &str = "https://download.geofabrik.de";

/// The name of a region in the languages of the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names {
    pub en: String,
    pub de: String,
    /// The Polish name; a region without one is named in English in Polish texts.
    pub pl: Option<String>,
}

impl Names {
    /// The name in the language with the code `code` (`de`, `pl`), English when there is none.
    pub fn in_lang(&self, code: &str) -> &str {
        match code {
            "de" => &self.de,
            "pl" => self.pl.as_deref().unwrap_or(&self.en),
            _ => &self.en,
        }
    }
}

/// One OSM extract of Geofabrik a region reads.
#[derive(Debug, Clone, PartialEq)]
pub struct OsmExtractInfo {
    /// The path of the extract on the Geofabrik server, without `-latest.osm.pbf`
    /// (`europe/germany/brandenburg`).
    pub geofabrik_path: String,
    /// The size of the download, for the estimate.
    pub megabytes: f64,
    /// The English name in progress lines and in the estimate.
    pub label: String,
    /// What the name of the cached file starts with (`brandenburg_`).
    pub prefix: String,
}

impl OsmExtractInfo {
    /// Where the extract is downloaded from.
    pub fn url(&self) -> String {
        format!("{GEOFABRIK}/{}-latest.osm.pbf", self.geofabrik_path)
    }

    /// Where Geofabrik says how current the extract is.
    pub fn state_url(&self) -> String {
        format!("{GEOFABRIK}/{}-updates/state.txt", self.geofabrik_path)
    }
}

/// The aerial photo of a region for the map of the area page: a WMS the browser asks for tiles.
/// It is not the service the orthophoto is fetched from; most regions have a mosaic for viewing
/// with its own address and layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewInfo {
    /// The address of the WMS, without a query.
    pub wms: String,
    pub layer: String,
    /// The image format asked for: `image/png` or `image/jpeg`.
    pub format: String,
    /// Further query parameters of the WMS request, by name (Hamburg: `time` = `2026`). They
    /// reach the request and nothing else: the page drops a name that is an option of its map
    /// library.
    pub params: Vec<(String, String)>,
    /// The credit shown on the map. `{year}` stands for the current year.
    pub attribution: String,
    /// The short name of the agency that serves it, for the name of the layer (`LGB`).
    pub agency: String,
}

/// What the manifest of an adapter declares about itself, for the list of the adapters and
/// for the rule that keeps requests on declared hosts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// Counts the data changes of the adapter.
    pub revision: u32,
    /// The hosts the adapter asks.
    pub hosts: Vec<String>,
    /// The day someone last checked the adapter against its services.
    pub last_verified: Option<chrono::NaiveDate>,
}

/// The country a region lies in; the engine words what a person reaches by it (no German
/// law for a Polish map).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Country {
    De,
    Pl,
}

impl Country {
    /// The ISO 3166 code in lower case.
    pub fn code(self) -> &'static str {
        match self {
            Self::De => "de",
            Self::Pl => "pl",
        }
    }
}

/// What a region supplies besides its OSM extracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// Rasters (and, where it has them, ALKIS): a map can have its centre here.
    Full,
    /// The OSM extracts only: a map is made from OSM alone, without templates.
    OsmOnly,
}

/// Identity and static facts of a region.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionInfo {
    /// The id of the adapter; the project file names its regions by it.
    pub id: AdapterId,
    /// The id of the provider of the region's rasters. Two regions may share one (Brandenburg
    /// and Berlin).
    pub provider: AdapterId,
    pub names: Names,
    /// The CRS of a map whose centre lies in the region.
    pub crs: Crs,
    /// The country of the region.
    pub country: Country,
    /// What the region supplies besides OSM.
    pub coverage: Coverage,
    /// What the ALKIS object ids of the region start with (`DEBB`), if it has ALKIS.
    pub object_id_prefix: Option<String>,
    /// The OSM extracts that cover the region.
    pub osm: Vec<OsmExtractInfo>,
    /// A short credit line for where the full one has no room.
    pub credit_short: Option<String>,
    /// The aerial photo for the map of the area page. Two regions may share one (Brandenburg
    /// and Berlin).
    pub preview: Option<PreviewInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(pl: Option<&str>) -> Names {
        Names {
            en: "Poland".into(),
            de: "Polen".into(),
            pl: pl.map(String::from),
        }
    }

    #[test]
    fn a_region_is_named_in_the_language_asked_for_and_in_english_otherwise() {
        let with = names(Some("Polska"));
        assert_eq!(with.in_lang("de"), "Polen");
        assert_eq!(with.in_lang("pl"), "Polska");
        assert_eq!(with.in_lang("en"), "Poland");
        assert_eq!(with.in_lang("fr"), "Poland");
        // No Polish name: English, not German.
        assert_eq!(names(None).in_lang("pl"), "Poland");
    }

    /// The ISO 3166 codes, in lower case, as they name the country in files and messages.
    #[test]
    fn a_country_has_its_iso_code() {
        assert_eq!(Country::De.code(), "de");
        assert_eq!(Country::Pl.code(), "pl");
    }
}
