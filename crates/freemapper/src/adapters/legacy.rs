//! The built-in states as adapters over the `Region` and `Provider` enums: the shim that lets
//! the engine ask the registry while the code of the states still hangs on the enums. Each
//! task of stage C moves one concern from the enums into the adapters; when none is left, the
//! enums go.

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::info::{Names, OsmExtractInfo, RegionInfo};
use freemapper_adapter_api::records::{AdapterId, EstimateItem};
use geo_types::MultiPolygon;

use crate::adapters::{alkis, credit, estimate};
use crate::crs::BoundingBox;
use std::sync::OnceLock;

use crate::sources::region::{Region, parse_boundary};

/// One built-in state.
pub struct LegacyAdapter {
    region: Region,
    info: RegionInfo,
    /// The box around the boundary, found when first asked for.
    alkis: Box<dyn AlkisSource>,
    bounds: OnceLock<BoundingBox>,
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

/// The government districts (Regierungsbezirke) of North Rhine-Westphalia: Geofabrik's slug, the
/// size of the extract in MB (Geofabrik, 2026-11-08) and the name. Geofabrik serves each as its
/// own extract, so a map reads one or two of them instead of the 917 MB state extract.
const NRW_DISTRICTS: [(&str, f64, &str); 5] = [
    ("arnsberg", 195.2, "Arnsberg"),
    ("detmold", 137.3, "Detmold"),
    ("duesseldorf", 217.2, "Düsseldorf"),
    ("koeln", 225.2, "Köln"),
    ("muenster", 140.9, "Münster"),
];

/// The extract boundary of the NRW district `index`: Geofabrik's `<district>-regbez.poly` in
/// EPSG:25833, simplified 25 m (ODbL), parsed once. Neighbouring boundaries overlap by a few
/// hundred metres, as the extracts do.
fn nrw_district_boundary(index: usize) -> Option<&'static MultiPolygon<f64>> {
    static BOUNDARIES: [OnceLock<MultiPolygon<f64>>; 5] = [
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    const TEXTS: [&str; 5] = [
        include_str!("../../../../assets/regions/osm/nrw-arnsberg.geojson"),
        include_str!("../../../../assets/regions/osm/nrw-detmold.geojson"),
        include_str!("../../../../assets/regions/osm/nrw-duesseldorf.geojson"),
        include_str!("../../../../assets/regions/osm/nrw-koeln.geojson"),
        include_str!("../../../../assets/regions/osm/nrw-muenster.geojson"),
    ];
    Some(
        BOUNDARIES
            .get(index)?
            .get_or_init(|| parse_boundary(TEXTS[index])),
    )
}

/// The Geofabrik extracts of a state: the path on the server, the size in MB, the English label
/// and what the cached file's name starts with. Berlin reads the extract of Brandenburg, which
/// holds it; the extract of Niedersachsen holds Bremen.
fn extracts(region: Region) -> Vec<OsmExtractInfo> {
    let one = |path: &str, megabytes: f64, label: &str, prefix: &str| {
        vec![OsmExtractInfo {
            geofabrik_path: format!("europe/germany/{path}"),
            megabytes,
            label: label.to_string(),
            prefix: prefix.to_string(),
        }]
    };
    match region {
        Region::Brandenburg | Region::Berlin => {
            one("brandenburg", 300.0, "Brandenburg", "brandenburg_")
        }
        Region::Saxony => one("sachsen", 270.0, "Saxony", "sachsen_"),
        Region::SaxonyAnhalt => one("sachsen-anhalt", 177.0, "Saxony-Anhalt", "sachsen-anhalt_"),
        Region::Hamburg => one("hamburg", 54.0, "Hamburg", "hamburg_"),
        Region::Hessen => one("hessen", 348.0, "Hesse", "hessen_"),
        Region::SchleswigHolstein => one(
            "schleswig-holstein",
            159.0,
            "Schleswig-Holstein",
            "schleswig-holstein_",
        ),
        Region::MecklenburgVorpommern => one(
            "mecklenburg-vorpommern",
            128.8,
            "Mecklenburg-Vorpommern",
            "mecklenburg-vorpommern_",
        ),
        Region::Niedersachsen => one("niedersachsen", 510.0, "Lower Saxony", "niedersachsen_"),
        Region::RheinlandPfalz => one(
            "rheinland-pfalz",
            275.0,
            "Rhineland-Palatinate",
            "rheinland-pfalz_",
        ),
        Region::NordrheinWestfalen => NRW_DISTRICTS
            .iter()
            .map(|(slug, megabytes, name)| OsmExtractInfo {
                geofabrik_path: format!("europe/germany/nordrhein-westfalen/{slug}-regbez"),
                megabytes: *megabytes,
                label: format!("North Rhine-Westphalia, {name}"),
                prefix: format!("nrw-{slug}_"),
            })
            .collect(),
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
            osm: extracts(region),
            credit_short: None,
        };
        Self {
            region,
            info,
            alkis: alkis::source(region),
            bounds: OnceLock::new(),
        }
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

    fn alkis(&self) -> Option<&dyn AlkisSource> {
        Some(&*self.alkis)
    }

    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        credit::lines(self.region, used)
    }

    fn tile_grid_km(&self) -> Option<u32> {
        estimate::tile_grid_km(self.region)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        estimate::items(self.region, area, env)
    }

    fn extract_boundary(&self, index: usize) -> Option<&MultiPolygon<f64>> {
        match self.region {
            Region::NordrheinWestfalen => nrw_district_boundary(index),
            _ => None,
        }
    }

    fn bounds(&self) -> BoundingBox {
        *self.bounds.get_or_init(|| {
            use geo::BoundingRect as _;
            let rect = self
                .boundary()
                .bounding_rect()
                .expect("a boundary has an extent");
            BoundingBox {
                min_e: rect.min().x,
                min_n: rect.min().y,
                max_e: rect.max().x,
                max_n: rect.max().y,
            }
        })
    }
}
