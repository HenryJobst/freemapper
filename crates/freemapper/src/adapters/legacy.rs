//! The built-in states as adapters over the `Region` and `Provider` enums: the shim that lets
//! the engine ask the registry while the code of the states still hangs on the enums. Each
//! task of stage C moves one concern from the enums into the adapters; when none is left, the
//! enums go.

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::info::{Names, OsmExtractInfo, RegionInfo};
use freemapper_adapter_api::records::AdapterId;
use geo_types::MultiPolygon;

use crate::sources::osm::ExtractSource;
use crate::sources::region::Region;

/// One built-in state.
pub struct LegacyAdapter {
    region: Region,
    info: RegionInfo,
}

/// The names of a state in English and German.
fn names(region: Region) -> Names {
    let (en, de) = match region {
        Region::Brandenburg => ("Brandenburg", "Brandenburg"),
        Region::Berlin => ("Berlin", "Berlin"),
        Region::Saxony => ("Saxony", "Sachsen"),
        Region::SaxonyAnhalt => ("Saxony-Anhalt", "Sachsen-Anhalt"),
        Region::Hamburg => ("Hamburg", "Hamburg"),
        Region::Hessen => ("Hesse", "Hessen"),
        Region::SchleswigHolstein => ("Schleswig-Holstein", "Schleswig-Holstein"),
        Region::MecklenburgVorpommern => ("Mecklenburg-Vorpommern", "Mecklenburg-Vorpommern"),
        Region::NordrheinWestfalen => ("North Rhine-Westphalia", "Nordrhein-Westfalen"),
        Region::Niedersachsen => ("Lower Saxony", "Niedersachsen"),
        Region::RheinlandPfalz => ("Rhineland-Palatinate", "Rheinland-Pfalz"),
    };
    Names {
        en: en.to_string(),
        de: de.to_string(),
    }
}

/// An extract as the info names it: the path on the Geofabrik server is what its URL holds
/// between the host and `-latest.osm.pbf`.
fn extract_info(source: ExtractSource) -> OsmExtractInfo {
    let path = source
        .url()
        .strip_prefix("https://download.geofabrik.de/")
        .and_then(|rest| rest.strip_suffix("-latest.osm.pbf"))
        .expect("an extract of Geofabrik");
    OsmExtractInfo {
        geofabrik_path: path.to_string(),
        megabytes: source.megabytes(),
        label: source.label().to_string(),
        prefix: source.prefix().to_string(),
    }
}

impl LegacyAdapter {
    pub fn new(region: Region) -> Self {
        let info = RegionInfo {
            id: AdapterId::built_in(region.id()),
            provider: AdapterId::built_in(region.provider().id()),
            names: names(region),
            crs: region.crs(),
            object_id_prefix: Some(region.object_id_prefix().to_string()),
            osm: ExtractSource::of_region(region)
                .into_iter()
                .map(extract_info)
                .collect(),
            credit_short: None,
        };
        Self { region, info }
    }

    /// The enum value the adapter stands for, for the code that still hangs on the enums.
    pub fn region(&self) -> Region {
        self.region
    }
}

impl RegionAdapter for LegacyAdapter {
    fn info(&self) -> &RegionInfo {
        &self.info
    }

    fn boundary(&self) -> &MultiPolygon<f64> {
        self.region.boundary()
    }
}
