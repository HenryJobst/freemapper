//! The built-in states as adapters over the `Region` and `Provider` enums: the shim that lets
//! the engine ask the registry while the code of the states still hangs on the enums. Each
//! task of stage C moves one concern from the enums into the adapters; when none is left, the
//! enums go.

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::extras::{NasService, TreeDataset};
use freemapper_adapter_api::geometry::Outline;
use freemapper_adapter_api::info::{Names, OsmExtractInfo, PreviewInfo, RegionInfo};
use freemapper_adapter_api::rasters::RasterBackend;
use freemapper_adapter_api::records::{AdapterId, EstimateItem};
use geo_types::MultiPolygon;

use crate::adapters::{alkis, credit, estimate, rasters};
use crate::crs::BoundingBox;
use std::sync::OnceLock;

use freemapper_adapters_de::boundaries::parse_boundary;
use freemapper_sources::nas::LGB_NAS;
use freemapper_sources::trees::POTSDAM;

use crate::sources::region::Region;

/// One built-in state.
pub struct LegacyAdapter {
    region: Region,
    info: RegionInfo,
    /// The box around the boundary, found when first asked for.
    alkis: Box<dyn AlkisSource>,
    nas: Option<NasService>,
    tree_datasets: Vec<TreeDataset>,
    rasters: Box<dyn RasterBackend>,
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

/// The aerial photo of a state for the map of the area page.
fn preview(region: Region) -> PreviewInfo {
    let jpeg = "image/jpeg";
    let (wms, layer, format, params, attribution, agency): (_, _, _, &[(&str, &str)], _, _) =
        match region {
            // The map proxy of LGB serves Brandenburg and Berlin.
            Region::Brandenburg | Region::Berlin => (
                "https://isk.geobasis-bb.de/mapproxy/dop20c/service/wms",
                "bebb_dop20c",
                "image/png",
                &[],
                "© GeoBasis-DE/LGB, dl-de/by-2-0",
                "LGB",
            ),
            Region::Saxony => (
                "https://geodienste.sachsen.de/wms_geosn_dop-rgb/guest",
                "sn_dop_020",
                jpeg,
                &[],
                "Quelle: GeoSN, dl-de/by-2-0",
                "GeoSN",
            ),
            Region::SaxonyAnhalt => (
                "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest",
                "lsa_lvermgeo_dop20_2",
                jpeg,
                &[],
                "© GeoBasis-DE / LVermGeo ST, dl-de/by-2-0",
                "LVermGeo",
            ),
            // The open 2026 vintage; 2023 is a closed-licence satellite image.
            Region::Hamburg => (
                "https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt",
                "dop_zeitreihe_unbelaubt",
                jpeg,
                &[("time", "2026")],
                "Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0",
                "LGV",
            ),
            // `he_dop_rgb` is served at all scales (`he_dop20_rgb` only below 1:9000, blank when
            // zoomed out).
            Region::Hessen => (
                "https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows",
                "he_dop_rgb",
                jpeg,
                &[],
                "Geobasisdaten © HVBG, dl-de/zero-2-0",
                "HVBG",
            ),
            // The current DOP20 mosaic (EPSG:3857 is among the layer's CRS).
            Region::SchleswigHolstein => (
                "https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD",
                "sh_dop20_rgb",
                jpeg,
                &[],
                "© GeoBasis-DE/LVermGeo SH/CC BY 4.0",
                "LVermGeo SH",
            ),
            // The ADV DOP20 mosaic (EPSG:3857 is offered); the WMS stamps a watermark, fine for
            // a preview.
            Region::MecklenburgVorpommern => (
                "https://www.geodaten-mv.de/dienste/adv_dop",
                "mv_dop",
                jpeg,
                &[],
                "©GeoBasis-DE/MV/CC BY 4.0",
                "LAiV M-V",
            ),
            // The current DOP mosaic (EPSG:3857 is among the layer's CRS).
            Region::NordrheinWestfalen => (
                "https://www.wms.nrw.de/geobasis/wms_nw_dop",
                "nw_dop_rgb",
                jpeg,
                &[],
                "Geobasisdaten © Geobasis NRW (Bezirksregierung Köln), dl-de/zero-2-0",
                "Geobasis NRW",
            ),
            // The newest DOP20 (EPSG:3857 is among the service's CRS); Bremen is left white.
            Region::Niedersachsen => (
                "https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms",
                "ni_dop20",
                jpeg,
                &[],
                "© GeoBasis-DE/LGLN, CC BY 4.0",
                "LGLN",
            ),
            // The current DOP20 (EPSG:3857 is among the layer's CRS); white beyond the state
            // border. The credit carries the year without a blank, as the licence page asks.
            Region::RheinlandPfalz => (
                "https://geo4.service24.rlp.de/wms/rp_dop20.fcgi",
                "rp_dop20",
                jpeg,
                &[],
                "©GeoBasis-DE / LVermGeoRP{year}, dl-de/by-2-0, www.lvermgeo.rlp.de",
                "LVermGeo RP",
            ),
        };
    PreviewInfo {
        wms: wms.to_string(),
        layer: layer.to_string(),
        format: format.to_string(),
        params: params
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect(),
        attribution: attribution.to_string(),
        agency: agency.to_string(),
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
            preview: Some(preview(region)),
        };
        Self {
            region,
            info,
            alkis: alkis::source(region),
            // The full ALKIS of LGB and the tree cadastre of Potsdam are Brandenburg's.
            nas: (region == Region::Brandenburg).then_some(LGB_NAS),
            tree_datasets: if region == Region::Brandenburg {
                vec![POTSDAM]
            } else {
                Vec::new()
            },
            rasters: rasters::backend(region.provider()),
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

    fn nas(&self) -> Option<&NasService> {
        self.nas.as_ref()
    }

    fn tree_datasets(&self) -> &[TreeDataset] {
        &self.tree_datasets
    }

    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        credit::lines(self.region, used)
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        Some(&*self.rasters)
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
        Outline::bounds(self.region.outline())
    }
}
