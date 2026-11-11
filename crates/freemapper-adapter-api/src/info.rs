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
    /// What the ALKIS object ids of the region start with (`DEBB`), if it has ALKIS.
    pub object_id_prefix: Option<String>,
    /// The OSM extracts that cover the region.
    pub osm: Vec<OsmExtractInfo>,
    /// A short credit line for where the full one has no room.
    pub credit_short: Option<String>,
}
