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
    /// The Czech name; a region without one is named in English in Czech texts.
    pub cs: Option<String>,
}

impl Names {
    /// The name in the language with the code `code` (`de`, `pl`, `cs`), English when there is none.
    pub fn in_lang(&self, code: &str) -> &str {
        match code {
            "de" => &self.de,
            "pl" => self.pl.as_deref().unwrap_or(&self.en),
            "cs" => self.cs.as_deref().unwrap_or(&self.en),
            _ => &self.en,
        }
    }
}

/// One OSM extract of Geofabrik a region reads.
#[derive(Debug, Clone, PartialEq)]
pub struct OsmExtractInfo {
    /// The path of the extract on the Geofabrik server, without `-latest.osm.pbf`
    /// (`europe/germany/brandenburg`); `europe/poland/lubuskie/lubuskie` names the folder too.
    pub geofabrik_path: String,
    /// The size of the download, for the estimate.
    pub megabytes: f64,
    /// The English name in progress lines and in the estimate.
    pub label: String,
    /// What the name of the cached file starts with (`brandenburg_`).
    pub prefix: String,
}

impl OsmExtractInfo {
    /// The folder of the extract and its name. Geofabrik keeps every extract in a folder of its
    /// own, `europe/germany/hessen/hessen-latest.osm.pbf`; the old address
    /// `europe/germany/hessen-latest.osm.pbf` redirects to a place that does not exist (checked
    /// 21 November 2026). A path that already ends in `<name>/<name>` stays as it is; one that
    /// ends in `<name>` only, as the manifests of earlier versions and the adapters of a person's
    /// own folder write it, gets the folder.
    fn folder_and_name(&self) -> (&str, &str) {
        match self.geofabrik_path.rsplit_once('/') {
            Some((folder, name)) if folder.rsplit('/').next() == Some(name) => (folder, name),
            _ => (
                self.geofabrik_path.as_str(),
                self.geofabrik_path.rsplit('/').next().unwrap_or_default(),
            ),
        }
    }

    /// Where the extract is downloaded from.
    pub fn url(&self) -> String {
        let (folder, name) = self.folder_and_name();
        format!("{GEOFABRIK}/{folder}/{name}-latest.osm.pbf")
    }

    /// Where Geofabrik says how current the extract is: in the folder of the extract.
    pub fn state_url(&self) -> String {
        let (folder, _) = self.folder_and_name();
        format!("{GEOFABRIK}/{folder}/updates/state.txt")
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
    Cz,
    Se,
}

impl Country {
    /// The ISO 3166 code in lower case.
    pub fn code(self) -> &'static str {
        match self {
            Self::De => "de",
            Self::Pl => "pl",
            Self::Cz => "cz",
            Self::Se => "se",
        }
    }
}

/// What a region supplies besides its OSM extracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// Rasters (and, where it has them, ALKIS): a map can have its centre here.
    Full,
    /// Rasters but no cadastre: the engine fetches and derives the templates, draws the buildings
    /// and land use from OSM, and refuses an area that reaches over the border of the country.
    Rasters,
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
            cs: None,
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
        // The Czech name stands apart from the Polish one.
        let czech = Names {
            cs: Some("Polsko".into()),
            ..names(Some("Polska"))
        };
        assert_eq!(czech.in_lang("cs"), "Polsko");
        assert_eq!(names(Some("Polska")).in_lang("cs"), "Poland");
    }

    /// The ISO 3166 codes, in lower case, as they name the country in files and messages.
    #[test]
    fn an_extract_lies_in_a_folder_of_its_own_whichever_way_the_path_is_written() {
        let extract = |path: &str| OsmExtractInfo {
            geofabrik_path: path.into(),
            megabytes: 1.0,
            label: String::new(),
            prefix: String::new(),
        };
        let own = extract("europe/poland/lubuskie/lubuskie");
        assert_eq!(
            own.url(),
            "https://download.geofabrik.de/europe/poland/lubuskie/lubuskie-latest.osm.pbf"
        );
        assert_eq!(
            own.state_url(),
            "https://download.geofabrik.de/europe/poland/lubuskie/updates/state.txt"
        );
        // The path without the folder name, as the manifests write it.
        let short = extract("europe/germany/brandenburg");
        assert_eq!(
            short.url(),
            "https://download.geofabrik.de/europe/germany/brandenburg/brandenburg-latest.osm.pbf"
        );
        assert_eq!(
            short.state_url(),
            "https://download.geofabrik.de/europe/germany/brandenburg/updates/state.txt"
        );
    }

    #[test]
    fn a_country_has_its_iso_code() {
        assert_eq!(Country::De.code(), "de");
        assert_eq!(Country::Pl.code(), "pl");
        assert_eq!(Country::Cz.code(), "cz");
        assert_eq!(Country::Se.code(), "se");
    }
}
