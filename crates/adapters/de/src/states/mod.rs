//! The German states as region adapters: for each state its identity, boundary, OSM extracts,
//! ALKIS service, credit lines, raster backend and download estimate.

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

use crate::crs::{BoundingBox, Crs};
use std::sync::{Arc, OnceLock};

use crate::boundaries::{StateOutline, outline, parse_boundary};
use freemapper_sources::nas::LGB_NAS;
use freemapper_sources::trees::POTSDAM;

pub mod alkis;
pub mod credit;
pub mod estimate;
pub mod rasters;

/// A German state freemapper has data for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Brandenburg,
    Berlin,
    Saxony,
    SaxonyAnhalt,
    Hamburg,
    Hessen,
    SchleswigHolstein,
    MecklenburgVorpommern,
    NordrheinWestfalen,
    Niedersachsen,
    RheinlandPfalz,
}

/// Who serves the rasters of a state (orthophoto, terrain and surface models).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agency {
    /// LGB Brandenburg, also for Berlin.
    Lgb,
    /// GeoSN, Saxony.
    GeoSn,
    /// LVermGeo, Saxony-Anhalt (EPSG:25832).
    LvermGeo,
    /// LGV, Hamburg (EPSG:25832).
    Lgv,
    /// HVBG, Hesse (EPSG:25832).
    Hvbg,
    /// LVermGeo SH, Schleswig-Holstein (EPSG:25832).
    LvermGeoSh,
    /// LAiV M-V, Mecklenburg-Vorpommern (EPSG:25833, the first per-area provider in zone 33).
    LaivMv,
    /// Geobasis NRW (Bezirksregierung Köln), North Rhine-Westphalia (EPSG:25832).
    GeobasisNrw,
    /// LGLN, Niedersachsen (EPSG:25832).
    Lgln,
    /// LVermGeo Rheinland-Pfalz (EPSG:25832).
    LvermGeoRp,
}

impl Agency {
    /// The id of the provider: its name in the project file.
    pub fn id(self) -> &'static str {
        match self {
            Self::Lgb => "lgb",
            Self::GeoSn => "geosn",
            Self::LvermGeo => "lvermgeo",
            Self::Lgv => "lgv",
            Self::Hvbg => "hvbg",
            Self::LvermGeoSh => "lvermgeosh",
            Self::LaivMv => "laivmv",
            Self::GeobasisNrw => "geobasisnrw",
            Self::Lgln => "lgln",
            Self::LvermGeoRp => "lvermgeorp",
        }
    }

    /// The terrain model the contours come from, for `freemapper:source` on contour candidates.
    pub fn terrain_source(self) -> &'static str {
        match self {
            Self::Lgb => "lgb-dgm1",
            Self::GeoSn => "geosn-dgm1",
            Self::LvermGeo => "lvermgeo-st-dgm1",
            Self::Lgv => "lgv-dgm1",
            Self::Hvbg => "hvbg-dgm1",
            Self::LvermGeoSh => "lvermgeo-sh-dgm1",
            Self::LaivMv => "laiv-mv-dgm1",
            Self::GeobasisNrw => "geobasis-nrw-dgm1",
            Self::Lgln => "lgln-dgm1",
            Self::LvermGeoRp => "lvermgeo-rp-dgm1",
        }
    }

    /// Whether the provider's rasters are fetched per map area (WMS, WCS, zip members) instead of
    /// as recorded tiles.
    pub fn per_area(self) -> bool {
        matches!(
            self,
            Self::LvermGeo
                | Self::Lgv
                | Self::Hvbg
                | Self::LvermGeoSh
                | Self::LaivMv
                | Self::GeobasisNrw
                | Self::Lgln
                | Self::LvermGeoRp
        )
    }
}

impl State {
    /// All states, in the order they are listed and queried.
    pub const ALL: [Self; 11] = [
        Self::Brandenburg,
        Self::Berlin,
        Self::Saxony,
        Self::SaxonyAnhalt,
        Self::Hamburg,
        Self::Hessen,
        Self::SchleswigHolstein,
        Self::MecklenburgVorpommern,
        Self::NordrheinWestfalen,
        Self::Niedersachsen,
        Self::RheinlandPfalz,
    ];

    /// The id of the adapter of the state: its name in the project file.
    pub fn id(self) -> &'static str {
        match self {
            Self::Brandenburg => "brandenburg",
            Self::Berlin => "berlin",
            Self::Saxony => "saxony",
            Self::SaxonyAnhalt => "saxonyanhalt",
            Self::Hamburg => "hamburg",
            Self::Hessen => "hessen",
            Self::SchleswigHolstein => "schleswigholstein",
            Self::MecklenburgVorpommern => "mecklenburgvorpommern",
            Self::NordrheinWestfalen => "nordrheinwestfalen",
            Self::Niedersachsen => "niedersachsen",
            Self::RheinlandPfalz => "rheinlandpfalz",
        }
    }

    /// What the ALKIS object ids of the region start with.
    pub fn object_id_prefix(self) -> &'static str {
        match self {
            Self::Brandenburg => "DEBB",
            Self::Berlin => "DEBE",
            Self::Saxony => "DESN",
            Self::SaxonyAnhalt => "DEST",
            Self::Hamburg => "DEHH",
            Self::Hessen => "DEHE",
            Self::SchleswigHolstein => "DESH",
            Self::MecklenburgVorpommern => "DEMV",
            Self::NordrheinWestfalen => "DENW",
            Self::Niedersachsen => "DENI",
            Self::RheinlandPfalz => "DERP",
        }
    }

    pub fn agency(self) -> Agency {
        match self {
            Self::Brandenburg | Self::Berlin => Agency::Lgb,
            Self::Saxony => Agency::GeoSn,
            Self::SaxonyAnhalt => Agency::LvermGeo,
            Self::Hamburg => Agency::Lgv,
            Self::Hessen => Agency::Hvbg,
            Self::SchleswigHolstein => Agency::LvermGeoSh,
            Self::MecklenburgVorpommern => Agency::LaivMv,
            Self::NordrheinWestfalen => Agency::GeobasisNrw,
            Self::Niedersachsen => Agency::Lgln,
            Self::RheinlandPfalz => Agency::LvermGeoRp,
        }
    }

    /// The CRS the region's rasters come in, and so the CRS of maps whose centre lies in it.
    pub fn crs(self) -> Crs {
        match self {
            Self::SaxonyAnhalt
            | Self::Hamburg
            | Self::Hessen
            | Self::SchleswigHolstein
            | Self::NordrheinWestfalen
            | Self::Niedersachsen
            | Self::RheinlandPfalz => Crs::Utm32,
            _ => Crs::Utm33,
        }
    }
}

/// The adapter of one state.
pub struct StateAdapter {
    state: State,
    info: RegionInfo,
    alkis: Box<dyn AlkisSource>,
    nas: Option<NasService>,
    tree_datasets: Vec<TreeDataset>,
    rasters: Box<dyn RasterBackend>,
}

/// The names of a state in English and German.
fn names(state: State) -> Names {
    let (en, de) = match state {
        State::Brandenburg => ("Brandenburg", "Brandenburg"),
        State::Berlin => ("Berlin", "Berlin"),
        State::Saxony => ("Saxony", "Sachsen"),
        State::SaxonyAnhalt => ("Saxony-Anhalt", "Sachsen-Anhalt"),
        State::Hamburg => ("Hamburg", "Hamburg"),
        State::Hessen => ("Hesse", "Hessen"),
        State::SchleswigHolstein => ("Schleswig-Holstein", "Schleswig-Holstein"),
        State::MecklenburgVorpommern => ("Mecklenburg-Vorpommern", "Mecklenburg-Vorpommern"),
        State::NordrheinWestfalen => ("North Rhine-Westphalia", "Nordrhein-Westfalen"),
        State::Niedersachsen => ("Lower Saxony", "Niedersachsen"),
        State::RheinlandPfalz => ("Rhineland-Palatinate", "Rheinland-Pfalz"),
    };
    Names {
        en: en.to_string(),
        de: de.to_string(),
    }
}

/// The aerial photo of a state for the map of the area page.
fn preview(state: State) -> PreviewInfo {
    let jpeg = "image/jpeg";
    let (wms, layer, format, params, attribution, agency): (_, _, _, &[(&str, &str)], _, _) =
        match state {
            // The map proxy of LGB serves Brandenburg and Berlin.
            State::Brandenburg | State::Berlin => (
                "https://isk.geobasis-bb.de/mapproxy/dop20c/service/wms",
                "bebb_dop20c",
                "image/png",
                &[],
                "© GeoBasis-DE/LGB, dl-de/by-2-0",
                "LGB",
            ),
            State::Saxony => (
                "https://geodienste.sachsen.de/wms_geosn_dop-rgb/guest",
                "sn_dop_020",
                jpeg,
                &[],
                "Quelle: GeoSN, dl-de/by-2-0",
                "GeoSN",
            ),
            State::SaxonyAnhalt => (
                "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest",
                "lsa_lvermgeo_dop20_2",
                jpeg,
                &[],
                "© GeoBasis-DE / LVermGeo ST, dl-de/by-2-0",
                "LVermGeo",
            ),
            // The open 2026 vintage; 2023 is a closed-licence satellite image.
            State::Hamburg => (
                "https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt",
                "dop_zeitreihe_unbelaubt",
                jpeg,
                &[("time", "2026")],
                "Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0",
                "LGV",
            ),
            // `he_dop_rgb` is served at all scales (`he_dop20_rgb` only below 1:9000, blank when
            // zoomed out).
            State::Hessen => (
                "https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows",
                "he_dop_rgb",
                jpeg,
                &[],
                "Geobasisdaten © HVBG, dl-de/zero-2-0",
                "HVBG",
            ),
            // The current DOP20 mosaic (EPSG:3857 is among the layer's CRS).
            State::SchleswigHolstein => (
                "https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD",
                "sh_dop20_rgb",
                jpeg,
                &[],
                "© GeoBasis-DE/LVermGeo SH/CC BY 4.0",
                "LVermGeo SH",
            ),
            // The ADV DOP20 mosaic (EPSG:3857 is offered); the WMS stamps a watermark, fine for
            // a preview.
            State::MecklenburgVorpommern => (
                "https://www.geodaten-mv.de/dienste/adv_dop",
                "mv_dop",
                jpeg,
                &[],
                "©GeoBasis-DE/MV/CC BY 4.0",
                "LAiV M-V",
            ),
            // The current DOP mosaic (EPSG:3857 is among the layer's CRS).
            State::NordrheinWestfalen => (
                "https://www.wms.nrw.de/geobasis/wms_nw_dop",
                "nw_dop_rgb",
                jpeg,
                &[],
                "Geobasisdaten © Geobasis NRW (Bezirksregierung Köln), dl-de/zero-2-0",
                "Geobasis NRW",
            ),
            // The newest DOP20 (EPSG:3857 is among the service's CRS); Bremen is left white.
            State::Niedersachsen => (
                "https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms",
                "ni_dop20",
                jpeg,
                &[],
                "© GeoBasis-DE/LGLN, CC BY 4.0",
                "LGLN",
            ),
            // The current DOP20 (EPSG:3857 is among the layer's CRS); white beyond the state
            // border. The credit carries the year without a blank, as the licence page asks.
            State::RheinlandPfalz => (
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
        include_str!("../../../../../assets/regions/osm/nrw-arnsberg.geojson"),
        include_str!("../../../../../assets/regions/osm/nrw-detmold.geojson"),
        include_str!("../../../../../assets/regions/osm/nrw-duesseldorf.geojson"),
        include_str!("../../../../../assets/regions/osm/nrw-koeln.geojson"),
        include_str!("../../../../../assets/regions/osm/nrw-muenster.geojson"),
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
fn extracts(state: State) -> Vec<OsmExtractInfo> {
    let one = |path: &str, megabytes: f64, label: &str, prefix: &str| {
        vec![OsmExtractInfo {
            geofabrik_path: format!("europe/germany/{path}"),
            megabytes,
            label: label.to_string(),
            prefix: prefix.to_string(),
        }]
    };
    match state {
        State::Brandenburg | State::Berlin => {
            one("brandenburg", 300.0, "Brandenburg", "brandenburg_")
        }
        State::Saxony => one("sachsen", 270.0, "Saxony", "sachsen_"),
        State::SaxonyAnhalt => one("sachsen-anhalt", 177.0, "Saxony-Anhalt", "sachsen-anhalt_"),
        State::Hamburg => one("hamburg", 54.0, "Hamburg", "hamburg_"),
        State::Hessen => one("hessen", 348.0, "Hesse", "hessen_"),
        State::SchleswigHolstein => one(
            "schleswig-holstein",
            159.0,
            "Schleswig-Holstein",
            "schleswig-holstein_",
        ),
        State::MecklenburgVorpommern => one(
            "mecklenburg-vorpommern",
            128.8,
            "Mecklenburg-Vorpommern",
            "mecklenburg-vorpommern_",
        ),
        State::Niedersachsen => one("niedersachsen", 510.0, "Lower Saxony", "niedersachsen_"),
        State::RheinlandPfalz => one(
            "rheinland-pfalz",
            275.0,
            "Rhineland-Palatinate",
            "rheinland-pfalz_",
        ),
        State::NordrheinWestfalen => NRW_DISTRICTS
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

impl StateAdapter {
    pub fn new(state: State) -> Self {
        let info = RegionInfo {
            id: AdapterId::built_in(state.id()),
            provider: AdapterId::built_in(state.agency().id()),
            names: names(state),
            crs: state.crs(),
            object_id_prefix: Some(state.object_id_prefix().to_string()),
            osm: extracts(state),
            credit_short: None,
            preview: Some(preview(state)),
        };
        Self {
            state,
            info,
            alkis: alkis::source(state),
            // The full ALKIS of LGB and the tree cadastre of Potsdam are Brandenburg's.
            nas: (state == State::Brandenburg).then_some(LGB_NAS),
            tree_datasets: if state == State::Brandenburg {
                vec![POTSDAM]
            } else {
                Vec::new()
            },
            rasters: rasters::backend(state.agency()),
        }
    }

    /// The outline of the state, from the embedded boundary.
    fn outline(&self) -> &'static StateOutline {
        outline(self.state.id()).expect("every state has a boundary")
    }
}

impl RegionAdapter for StateAdapter {
    fn info(&self) -> &RegionInfo {
        &self.info
    }

    fn boundary(&self) -> &MultiPolygon<f64> {
        Outline::boundary(self.outline())
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
        credit::lines(self.state, used)
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        Some(&*self.rasters)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        estimate::items(self.state, area, env)
    }

    fn extract_boundary(&self, index: usize) -> Option<&MultiPolygon<f64>> {
        match self.state {
            State::NordrheinWestfalen => nrw_district_boundary(index),
            _ => None,
        }
    }

    fn bounds(&self) -> BoundingBox {
        Outline::bounds(self.outline())
    }
}

/// The adapters of the states, in the order the region functions go through them.
pub fn adapters() -> Vec<Arc<dyn RegionAdapter>> {
    State::ALL
        .into_iter()
        .map(|state| Arc::new(StateAdapter::new(state)) as Arc<dyn RegionAdapter>)
        .collect()
}

/// The tree cadastres of all states, in the order of the states.
pub(crate) fn all_tree_datasets() -> Vec<TreeDataset> {
    vec![POTSDAM]
}
