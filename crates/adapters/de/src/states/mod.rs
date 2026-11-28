// SPDX-License-Identifier: AGPL-3.0-or-later

//! The German states as region adapters. Each state is its manifest
//! (`manifests/<id>/manifest.toml` with its boundary file) plus what no manifest says: the
//! raster code of its agency, the credit lines that depend on more than a year, the ALKIS of
//! Berlin and the download estimate.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::error::Remedy;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::extras::{ExtraSet, NasService, RasterTemplate, TreeDataset};
use freemapper_adapter_api::geometry::Outline;
#[cfg(test)]
use freemapper_adapter_api::info::PreviewInfo;
use freemapper_adapter_api::info::{Declared, RegionInfo};
use freemapper_adapter_api::permits::PermitSource;
use freemapper_adapter_api::pointcloud::PointCloudBackend;
use freemapper_adapter_api::policy::UrlPolicy;
use freemapper_adapter_api::rasters::{RasterBackend, RasterConfig};
use freemapper_adapter_api::records::EstimateItem;
use freemapper_adapter_api::registry::{DuplicateBackend, RasterFactory, Registry};
use freemapper_manifest::adapter::{AdapterFiles, ManifestAdapter, Parse};
use freemapper_manifest::guard::GuardedAlkis;
use geo_types::MultiPolygon;

use crate::boundaries::{StateOutline, boundary_text, outline};
use crate::crs::{BoundingBox, Crs};

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
    Saarland,
    BadenWuerttemberg,
    Bremen,
    Bayern,
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
    /// LVGL, Saarland (EPSG:25832).
    Lvgl,
    /// LGL, Baden-Württemberg (EPSG:25832).
    LglBw,
    /// Landesamt GeoInformation Bremen (EPSG:25832).
    GeoBremen,
    /// Bayerische Vermessungsverwaltung, by its state office the LDBV (EPSG:25832).
    Ldbv,
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
            Self::Lvgl => "lvgl",
            Self::LglBw => "lglbw",
            Self::GeoBremen => "geobremen",
            Self::Ldbv => "ldbv",
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
            Self::Lvgl => "lvgl-dgm1",
            Self::LglBw => "lgl-bw-dgm1",
            Self::GeoBremen => "geobremen-dgm1",
            Self::Ldbv => "ldbv-dgm1",
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
                | Self::Lvgl
                | Self::LglBw
                | Self::GeoBremen
                | Self::Ldbv
        )
    }
}

impl State {
    /// All states, in the order they are listed and queried.
    pub const ALL: [Self; 15] = [
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
        Self::Saarland,
        Self::BadenWuerttemberg,
        Self::Bremen,
        Self::Bayern,
    ];

    /// The id of the adapter of the state: its name in the project file.
    pub const fn id(self) -> &'static str {
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
            Self::Saarland => "saarland",
            Self::BadenWuerttemberg => "badenwuerttemberg",
            Self::Bremen => "bremen",
            Self::Bayern => "bayern",
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
            Self::Saarland => "DESL",
            Self::BadenWuerttemberg => "DEBW",
            Self::Bremen => "DEHB",
            Self::Bayern => "DEBY",
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
            Self::Saarland => Agency::Lvgl,
            Self::BadenWuerttemberg => Agency::LglBw,
            Self::Bremen => Agency::GeoBremen,
            Self::Bayern => Agency::Ldbv,
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
            | Self::RheinlandPfalz
            | Self::Saarland
            | Self::BadenWuerttemberg
            | Self::Bremen
            | Self::Bayern => Crs::Utm32,
            _ => Crs::Utm33,
        }
    }
}

/// The manifests of the states, in the order of [`State::ALL`]. A manifest holds what a state
/// declares: who it is, its OSM extracts, its ALKIS service, its credit, its preview and which
/// code fetches its rasters.
const MANIFESTS: [&str; 15] = [
    include_str!("../../manifests/brandenburg/manifest.toml"),
    include_str!("../../manifests/berlin/manifest.toml"),
    include_str!("../../manifests/saxony/manifest.toml"),
    include_str!("../../manifests/saxonyanhalt/manifest.toml"),
    include_str!("../../manifests/hamburg/manifest.toml"),
    include_str!("../../manifests/hessen/manifest.toml"),
    include_str!("../../manifests/schleswigholstein/manifest.toml"),
    include_str!("../../manifests/mecklenburgvorpommern/manifest.toml"),
    include_str!("../../manifests/nordrheinwestfalen/manifest.toml"),
    include_str!("../../manifests/niedersachsen/manifest.toml"),
    include_str!("../../manifests/rheinlandpfalz/manifest.toml"),
    include_str!("../../manifests/saarland/manifest.toml"),
    include_str!("../../manifests/badenwuerttemberg/manifest.toml"),
    include_str!("../../manifests/bremen/manifest.toml"),
    include_str!("../../manifests/bayern/manifest.toml"),
];

/// The boundaries of the five OSM extracts of North Rhine-Westphalia: Geofabrik's
/// `<district>-regbez.poly` in EPSG:25833, simplified 25 m (ODbL). Neighbouring boundaries
/// overlap by a few hundred metres, as the extracts do.
const NRW_EXTRACT_BOUNDARIES: [(&str, &str); 5] = [
    (
        "osm-arnsberg.geojson",
        include_str!("../../manifests/nordrheinwestfalen/osm-arnsberg.geojson"),
    ),
    (
        "osm-detmold.geojson",
        include_str!("../../manifests/nordrheinwestfalen/osm-detmold.geojson"),
    ),
    (
        "osm-duesseldorf.geojson",
        include_str!("../../manifests/nordrheinwestfalen/osm-duesseldorf.geojson"),
    ),
    (
        "osm-koeln.geojson",
        include_str!("../../manifests/nordrheinwestfalen/osm-koeln.geojson"),
    ),
    (
        "osm-muenster.geojson",
        include_str!("../../manifests/nordrheinwestfalen/osm-muenster.geojson"),
    ),
];

/// The boundaries of the four OSM extracts of Baden-Württemberg: Geofabrik's
/// `<district>-regbez.poly` in EPSG:25833, simplified 25 m (ODbL). Together they cover the
/// state; none reaches across the Rhine or into Bavaria.
const BW_EXTRACT_BOUNDARIES: [(&str, &str); 4] = [
    (
        "osm-freiburg.geojson",
        include_str!("../../manifests/badenwuerttemberg/osm-freiburg.geojson"),
    ),
    (
        "osm-karlsruhe.geojson",
        include_str!("../../manifests/badenwuerttemberg/osm-karlsruhe.geojson"),
    ),
    (
        "osm-stuttgart.geojson",
        include_str!("../../manifests/badenwuerttemberg/osm-stuttgart.geojson"),
    ),
    (
        "osm-tuebingen.geojson",
        include_str!("../../manifests/badenwuerttemberg/osm-tuebingen.geojson"),
    ),
];

/// The boundaries of the seven OSM extracts of Bavaria: Geofabrik's `<district>.poly` in
/// EPSG:25833, simplified 25 m (ODbL). Together they leave 0.02 km² of the state uncovered,
/// where the nearest extract is taken.
const BY_EXTRACT_BOUNDARIES: [(&str, &str); 7] = [
    (
        "osm-mittelfranken.geojson",
        include_str!("../../manifests/bayern/osm-mittelfranken.geojson"),
    ),
    (
        "osm-niederbayern.geojson",
        include_str!("../../manifests/bayern/osm-niederbayern.geojson"),
    ),
    (
        "osm-oberbayern.geojson",
        include_str!("../../manifests/bayern/osm-oberbayern.geojson"),
    ),
    (
        "osm-oberfranken.geojson",
        include_str!("../../manifests/bayern/osm-oberfranken.geojson"),
    ),
    (
        "osm-oberpfalz.geojson",
        include_str!("../../manifests/bayern/osm-oberpfalz.geojson"),
    ),
    (
        "osm-schwaben.geojson",
        include_str!("../../manifests/bayern/osm-schwaben.geojson"),
    ),
    (
        "osm-unterfranken.geojson",
        include_str!("../../manifests/bayern/osm-unterfranken.geojson"),
    ),
];

/// The manifests as corrections of addresses changed them (issue #273, stage 4), by state id; set
/// once, before anything reads a manifest.
static CORRECTED: OnceLock<BTreeMap<String, &'static str>> = OnceLock::new();

/// Puts the corrected manifests `texts` (by state id) in the place of the built-in ones, for the
/// rest of the process. A text must be the built-in manifest with addresses changed and must
/// parse; the caller checked both. Returns `false` when corrections were set before: the first
/// ones stay, so that every reader of the process sees the same manifests.
pub fn set_corrected_manifests(texts: BTreeMap<String, String>) -> bool {
    let leaked = texts
        .into_iter()
        .map(|(id, text)| (id, &*Box::leak(text.into_boxed_str())))
        .collect();
    CORRECTED.set(leaked).is_ok()
}

/// The text of the manifest of `state` as the program shipped it, whatever corrections say.
pub fn shipped_manifest_text(state: State) -> &'static str {
    let index = State::ALL
        .iter()
        .position(|other| *other == state)
        .expect("every state is listed");
    MANIFESTS[index]
}

/// The texts of all manifests in force, in the order of [`State::ALL`], for what reads them
/// without a state (the reachability probe).
pub fn manifest_texts() -> Vec<&'static str> {
    State::ALL.into_iter().map(manifest_text).collect()
}

/// The text of the manifest of `state` that is in force: the corrected one when a list of
/// corrections changed it, else the one the program shipped.
pub fn manifest_text(state: State) -> &'static str {
    CORRECTED
        .get()
        .and_then(|corrected| corrected.get(state.id()))
        .copied()
        .unwrap_or_else(|| shipped_manifest_text(state))
}

/// The files the manifest of a state names, embedded in the program.
pub struct StateFiles(pub State);

impl AdapterFiles for StateFiles {
    fn read(&self, name: &str) -> Result<Cow<'static, str>, String> {
        // The boundaries of the OSM extracts of the states that have more than one.
        let extracts: &[(&str, &str)] = match self.0 {
            State::NordrheinWestfalen => &NRW_EXTRACT_BOUNDARIES,
            State::BadenWuerttemberg => &BW_EXTRACT_BOUNDARIES,
            State::Bayern => &BY_EXTRACT_BOUNDARIES,
            _ => &[],
        };
        let text = match name {
            "boundary.geojson" => boundary_text(self.0.id()),
            _ => extracts
                .iter()
                .find(|(file, _)| *file == name)
                .map(|(_, text)| *text),
        };
        text.map(Cow::Borrowed)
            .ok_or_else(|| "the file is not built in".to_string())
    }
}

/// The files of the adapter folder that holds the state `id`: its manifest and every file the
/// manifest names (the boundary, the boundaries of OSM extracts), under the names the manifest
/// uses. `adapters export` writes them; the folder, named after the id, loads as an adapter
/// that replaces the built-in one (ADR 0188). A manifest that declares its orthophoto as a
/// service must have a provider of its own to load as a replacement, so its `provider` is
/// the id of the state in the export (Saxony-Anhalt). `None` for an id that is no state of
/// this crate.
pub fn export_files(id: &str) -> Option<Vec<(String, Cow<'static, str>)>> {
    let state = State::ALL.into_iter().find(|state| state.id() == id)?;
    let files = StateFiles(state);
    let mut names = vec!["boundary.geojson".to_string()];
    names.extend(
        match state {
            State::NordrheinWestfalen => &NRW_EXTRACT_BOUNDARIES[..],
            State::BadenWuerttemberg => &BW_EXTRACT_BOUNDARIES[..],
            State::Bayern => &BY_EXTRACT_BOUNDARIES[..],
            _ => &[],
        }
        .iter()
        .map(|(name, _)| (*name).to_string()),
    );
    let text = manifest_text(state);
    let own_provider = freemapper_manifest::parse(text, Path::new(state.id()))
        .ok()
        .filter(|manifest| {
            matches!(
                manifest.orthophoto,
                Some(
                    freemapper_manifest::model::Orthophoto::Wms { .. }
                        | freemapper_manifest::model::Orthophoto::Wcs { .. }
                )
            )
        })
        .map(|manifest| manifest.adapter.provider);
    let manifest = match own_provider {
        Some(provider) => Cow::Owned(text.replacen(
            &format!("provider = \"{provider}\""),
            &format!("provider = \"{}\"", state.id()),
            1,
        )),
        None => Cow::Borrowed(text),
    };
    let mut out = vec![("manifest.toml".to_string(), manifest)];
    for name in names {
        let text = files
            .read(&name)
            .expect("a file the export names is built in");
        out.push((name, text));
    }
    Some(out)
}

/// The name a manifest calls the raster backend of `agency` by; `None` for LVermGeo, whose
/// rasters the manifest of Saxony-Anhalt declares as services.
pub fn backend_name(agency: Agency) -> Option<&'static str> {
    match agency {
        Agency::Lgb => Some("lgb"),
        Agency::GeoSn => Some("geosn"),
        Agency::LvermGeo => None,
        Agency::Lgv => Some("lgv"),
        Agency::Hvbg => Some("hvbg"),
        Agency::LvermGeoSh => Some("lvermgeo-sh"),
        Agency::LaivMv => Some("laiv-mv"),
        Agency::GeobasisNrw => Some("geobasis-nrw"),
        Agency::Lgln => Some("lgln"),
        Agency::LvermGeoRp => Some("lvermgeo-rp"),
        Agency::Lvgl => Some("lvgl"),
        Agency::LglBw => Some("lgl-bw"),
        Agency::GeoBremen => Some("geobremen"),
        Agency::Ldbv => Some("ldbv"),
    }
}

/// The agencies whose rasters code of this crate fetches, in the order of the states.
const AGENCIES: [Agency; 14] = [
    Agency::Lgb,
    Agency::GeoSn,
    Agency::LvermGeo,
    Agency::Lgv,
    Agency::Hvbg,
    Agency::LvermGeoSh,
    Agency::LaivMv,
    Agency::GeobasisNrw,
    Agency::Lgln,
    Agency::LvermGeoRp,
    Agency::Lvgl,
    Agency::LglBw,
    Agency::GeoBremen,
    Agency::Ldbv,
];

/// Adds the raster backends of this crate to `registry`, each under the name the manifests
/// call it by.
pub fn register_backends(registry: &mut Registry) -> Result<(), DuplicateBackend> {
    registry.set_permit_factory(Arc::new(crate::wfs_permits::TableFactory));
    for agency in AGENCIES {
        if let (Some(name), Some((state, crs))) = (backend_name(agency), rasters::addressed(agency))
        {
            registry.add_raster_factory(name, raster_factory(agency, state, crs))?;
        }
    }
    Ok(())
}

/// The `[orthophoto]` of the built-in manifest of `state`: the addresses a manifest of a folder
/// that replaces the state falls back on where it states none.
pub fn builtin_raster_config(state: State) -> RasterConfig {
    match freemapper_manifest::parse(manifest_text(state), Path::new(state.id()))
        .expect("the built-in manifests are valid")
        .orthophoto
    {
        Some(freemapper_manifest::model::Orthophoto::Backend { config, .. }) => config,
        _ => RasterConfig::default(),
    }
}

/// The WMS of the orthophoto of `state` in the CRS `crs`: the addresses `config` states, and for
/// the keys it leaves out those of the built-in manifest of the state.
fn wms_of(
    config: &RasterConfig,
    state: State,
    crs: Crs,
) -> Result<freemapper_sources::wms::WmsService, String> {
    let merged = config.or(&builtin_raster_config(state));
    let (Some(url), Some(layer), Some(max_px)) = (merged.url, merged.layer, merged.max_px) else {
        return Err(format!(
            "the built-in manifest of {} lacks the `url`, `layer` or `max_px` of its orthophoto",
            state.id()
        ));
    };
    Ok(freemapper_sources::wms::WmsService {
        url: Cow::Owned(url),
        layer: Cow::Owned(layer),
        crs,
        extra: Cow::Borrowed(""),
        max_px,
    })
}

/// What makes the raster backend of `agency` from the addresses of an `[orthophoto]`.
fn raster_factory(agency: Agency, state: State, crs: Crs) -> RasterFactory {
    Arc::new(move |config: &RasterConfig| {
        let merged = config.or(&builtin_raster_config(state));
        let inner = if rasters::takes_wms(agency) {
            rasters::backend_with_wms(agency, wms_of(config, state, crs)?, &merged)?
        } else {
            rasters::backend_without_wms(agency, &merged)?
        }
        .ok_or_else(|| format!("{agency:?} takes no addresses"))?;
        Ok(Arc::new(Estimating { inner, agency }) as Arc<dyn RasterBackend>)
    })
}

/// A built-in raster backend that also answers the download estimate, with the raster items
/// of its agency (issue #71). A state answers its estimate itself ([`estimate::items`]); an
/// adapter of a folder that names the backend reaches the numbers only through here.
struct Estimating {
    inner: Box<dyn RasterBackend>,
    agency: Agency,
}

impl RasterBackend for Estimating {
    fn provider(&self) -> &freemapper_adapter_api::rasters::ProviderInfo {
        self.inner.provider()
    }

    fn fetch(
        &self,
        env: &mut freemapper_adapter_api::env::AdapterEnv<'_>,
        project: &mut dyn freemapper_adapter_api::project::RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), freemapper_adapter_api::error::AdapterError> {
        self.inner.fetch(env, project, bbox)
    }

    fn trust(&self) -> Option<&freemapper_adapter_api::trust::TrustAnchors> {
        self.inner.trust()
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        estimate::raster_items(self.agency, area, env)
    }

    fn derive_inputs(
        &self,
        env: &freemapper_adapter_api::rasters::DeriveEnv<'_>,
        stored: &freemapper_adapter_api::rasters::StoredRasters<'_>,
        area: &freemapper_adapter_api::rasters::RasterArea,
    ) -> freemapper_adapter_api::rasters::DeriveInputs {
        self.inner.derive_inputs(env, stored, area)
    }
}

/// The adapter of one state: its manifest, and what no manifest says.
///
/// From the manifest come the identity, the OSM extracts and their boundaries, the ALKIS
/// service, the full ALKIS and the tree cadastres, the raster backend (by name, or the services
/// the manifest declares) and the credit of the states whose credit is one line with a year.
/// Code are the ALKIS of Berlin, of Baden-Württemberg, of Bremen and of Bavaria, the credits of Brandenburg, Berlin and Saxony (they depend on
/// tiles, on the neighbour and on the layers that drew objects) and the download estimate.
/// Whatever is asked for the state, by its manifest or by its code, stays on the hosts the
/// manifest declares.
pub struct StateAdapter {
    state: State,
    manifest: ManifestAdapter,
    /// The ALKIS source where the manifest declares none (Berlin, Baden-Württemberg, Bremen,
    /// Bavaria).
    alkis: Option<Box<dyn AlkisSource>>,
    /// The sets of extra layers the state fetches with code (Berlin's registers and surveys).
    extras: Vec<Box<dyn ExtraSet>>,
    /// The raster templates the state fetches with code (Berlin's K5).
    templates: Vec<Box<dyn RasterTemplate>>,
    /// The sources of the permit notes the state brings (Berlin's Geoportal, ADR 0120; the
    /// forest administrations of Brandenburg and North Rhine-Westphalia, ADR 0124).
    permits: Vec<Box<dyn PermitSource>>,
    /// The point-cloud backend of the state, made of the addresses of its `[point_cloud]`.
    point_cloud: Option<Box<dyn PointCloudBackend>>,
}

/// The `[point_cloud]` of the manifest of `state`, parsed again: for the tests of the backends,
/// which are held to the addresses the manifest declares.
#[cfg(test)]
pub(crate) fn point_cloud_of(state: State) -> Option<freemapper_manifest::PointCloud> {
    freemapper_manifest::parse(manifest_text(state), Path::new(state.id()))
        .expect("the built-in manifests are valid")
        .point_cloud
}

/// The point-cloud backend of `state` for the addresses in `cloud`: the code that reads what the
/// state publishes, kept in the adapter, and where it reads it, taken from the manifest.
fn point_cloud_backend(
    state: State,
    cloud: &freemapper_manifest::PointCloud,
) -> Option<Box<dyn PointCloudBackend>> {
    Some(match state {
        State::NordrheinWestfalen => Box::new(crate::nrw_3dm::Nrw3dmBackend::new(cloud)),
        State::RheinlandPfalz => Box::new(crate::lvermgeo_rp_las::RpLasBackend::new(cloud)),
        State::Berlin => Box::new(crate::berlin_als::BerlinAlsBackend::new(cloud)),
        State::Saarland => Box::new(crate::lvgl_las::LvglLasBackend::new(cloud)),
        State::Bayern => Box::new(crate::ldbv_las::LdbvLasBackend::new(cloud)),
        State::Brandenburg => Box::new(crate::lgb_als::LgbAlsBackend::new(cloud)),
        _ => return None,
    })
}

impl StateAdapter {
    /// The adapter of `state`, with its raster backend taken from `backends`
    /// ([`register_backends`]).
    pub fn new(state: State, backends: &Registry) -> Self {
        Self::with_hosts(state, backends, &[])
    }

    /// [`Self::new`] with the hosts in `extra` allowed besides those of the manifest: what a
    /// person allowed for the state after a service moved (ADR 0110).
    pub fn with_hosts(state: State, backends: &Registry, extra: &[String]) -> Self {
        let broken = |findings: Vec<freemapper_manifest::ManifestError>| -> ! {
            let lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
            panic!(
                "the built-in manifest of {}: {}",
                state.id(),
                lines.join("; ")
            )
        };
        let manifest = freemapper_manifest::parse(manifest_text(state), Path::new(state.id()))
            .unwrap_or_else(|findings| broken(findings));
        // A state asks the hosts of its manifest and no other, redirects included (ADR 0106):
        // what the manifest declares, and what is code here.
        let mut manifest =
            ManifestAdapter::new(manifest, &StateFiles(state), Parse::OnFirstUse, backends)
                .unwrap_or_else(|findings| broken(findings))
                .held_to_its_hosts(Remedy::AllowHost, extra);
        // The permit sources the manifest declares (issue #273) come after those of the code.
        let declared_permits = manifest.take_permit_sources();
        let policy = manifest
            .policy()
            .expect("the manifest was just bound to its hosts")
            .clone();
        let alkis = match manifest.alkis() {
            Some(_) => None,
            None => Some(Box::new(GuardedAlkis {
                inner: alkis::source(state)
                    .unwrap_or_else(|| panic!("{}: no [alkis] and no code source", state.id())),
                policy: Arc::new(policy),
            }) as Box<dyn AlkisSource>),
        };
        let extras: Vec<Box<dyn ExtraSet>> = match state {
            State::Berlin => vec![Box::new(crate::berlin::Registers)],
            State::Hamburg => vec![Box::new(crate::lgv_fk::Feinkartierung)],
            _ => Vec::new(),
        };
        let templates: Vec<Box<dyn RasterTemplate>> = match state {
            State::Berlin => vec![Box::new(crate::berlin::K5)],
            _ => Vec::new(),
        };
        // The BfG's protection areas where the state's own services give none (ADR 0142), the
        // monuments of the state offices where they are open (ADR 0144); the monuments of the
        // seven states that have them are declared in their manifests (issue #273).
        let permits: Vec<Box<dyn PermitSource>> = match state {
            // The sources of Berlin are declared in its manifest (issue #273).
            State::Berlin => Vec::new(),
            // The Reviere and the forest functions of Brandenburg are declared in its manifest
            // (issue #273); its water protection areas are a shapefile.
            State::Brandenburg => Vec::new(),
            // The sources of Hamburg are declared in its manifest (issue #273).
            State::Hamburg => Vec::new(),
            // The sources of Saarland, Hesse and Schleswig-Holstein that had tables in the code are
            // declared in their manifests (issue #273).
            State::Saarland | State::Hessen => Vec::new(),
            // Its monuments are a GeoJSON file, declared in its manifest too (issue #273).
            State::SchleswigHolstein => Vec::new(),
            // The protected biotopes of North Rhine-Westphalia are declared in its manifest (issue #273).
            State::NordrheinWestfalen => Vec::new(),
            // The sources of Baden-Württemberg are declared in its manifest (issue #273).
            State::BadenWuerttemberg => Vec::new(),
            // The Sachsenforst, the monuments, the Flächennaturdenkmale and the biotopes are
            // declared in its manifest (issue #273).
            State::Saxony => vec![crate::bfg_permits::source()],
            State::Bremen => vec![crate::bfg_permits::source()],
            // The Naturdenkmale of Rhineland-Palatinate are declared in its manifest (issue #273).
            State::RheinlandPfalz => vec![crate::bfg_permits::source()],
            // The Naturdenkmale of Saxony-Anhalt are declared in its manifest (issue #273).
            State::SaxonyAnhalt => vec![crate::bfg_permits::source()],
            // The Naturdenkmale of the LfU are declared in its manifest (issue #273).
            State::Bayern => vec![crate::bfg_permits::source()],
            // The Naturdenkmale of the NLWKN are declared in its manifest (issue #273).
            State::Niedersachsen => vec![crate::bfg_permits::source()],
            State::MecklenburgVorpommern => vec![crate::bfg_permits::source()],
        };
        let mut permits = permits;
        permits.extend(declared_permits);
        let point_cloud = manifest
            .manifest()
            .point_cloud
            .as_ref()
            .and_then(|cloud| point_cloud_backend(state, cloud));
        Self {
            point_cloud,
            state,
            alkis,
            extras,
            templates,
            permits,
            manifest,
        }
    }

    /// The outline of the state, from the embedded boundary: parsed once for the adapter and
    /// for the modules of the agencies, which ask about their own state.
    fn outline(&self) -> &'static StateOutline {
        outline(self.state.id()).expect("every state has a boundary")
    }
}

impl RegionAdapter for StateAdapter {
    fn info(&self) -> &RegionInfo {
        self.manifest.info()
    }

    fn declared(&self) -> Option<&Declared> {
        self.manifest.declared()
    }

    fn policy(&self) -> Option<&UrlPolicy> {
        self.manifest.policy()
    }

    fn boundary(&self) -> &MultiPolygon<f64> {
        Outline::boundary(self.outline())
    }

    fn alkis(&self) -> Option<&dyn AlkisSource> {
        self.alkis.as_deref().or_else(|| self.manifest.alkis())
    }

    fn nas_services(&self) -> &[NasService] {
        self.manifest.nas_services()
    }

    fn tree_datasets(&self) -> &[TreeDataset] {
        self.manifest.tree_datasets()
    }

    fn extras(&self) -> &[Box<dyn ExtraSet>] {
        &self.extras
    }

    fn templates(&self) -> &[Box<dyn RasterTemplate>] {
        &self.templates
    }

    fn permit_sources(&self) -> &[Box<dyn PermitSource>] {
        &self.permits
    }

    fn no_buildings_warning(&self) -> Option<&'static str> {
        (self.state == State::Berlin).then_some("warning-berlin-buildings-missing")
    }

    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        match self.state {
            State::Brandenburg | State::Berlin | State::Saxony => credit::lines(self.state, used),
            _ => self.manifest.attribution(used),
        }
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        self.manifest.rasters()
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        estimate::items(self.state, area, env)
    }

    fn extract_boundary(&self, index: usize) -> Option<&MultiPolygon<f64>> {
        self.manifest.extract_boundary(index)
    }

    fn bounds(&self) -> BoundingBox {
        Outline::bounds(self.outline())
    }

    fn point_cloud(&self) -> Option<&dyn PointCloudBackend> {
        self.point_cloud.as_deref()
    }
}

/// Adds the raster backends and the adapters of the states to `registry`, the adapters in the
/// order the region functions go through them.
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    register_with_hosts(registry, &BTreeMap::new())
}

/// [`register`] with the hosts a person allowed for a state besides those of its manifest, by
/// the id of the state.
pub fn register_with_hosts(
    registry: &mut Registry,
    extra: &BTreeMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    register_backends(registry)?;
    for state in State::ALL {
        let hosts = extra.get(state.id()).map_or(&[][..], Vec::as_slice);
        let adapter = StateAdapter::with_hosts(state, registry, hosts);
        registry.add(Arc::new(adapter))?;
    }
    Ok(())
}

/// The adapters of the states, in the order the region functions go through them.
pub fn adapters() -> Vec<Arc<dyn RegionAdapter>> {
    let mut backends = Registry::new();
    register_backends(&mut backends).expect("the backends have distinct names");
    State::ALL
        .into_iter()
        .map(|state| Arc::new(StateAdapter::new(state, &backends)) as Arc<dyn RegionAdapter>)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the manifest of each state with an addressed orthophoto says is what the code
    /// held before the addresses moved there (issue #273): the same WMS, layer, size and CRS.
    #[test]
    fn the_orthophoto_wms_of_each_manifest_is_the_one_the_code_held() {
        let held: [(Agency, &freemapper_sources::wms::WmsService); 10] = [
            (Agency::GeoSn, &crate::geosn::WMS),
            (Agency::Hvbg, &crate::hvbg::WMS),
            (Agency::LvermGeoSh, &crate::lvermgeo_sh::WMS),
            (Agency::GeobasisNrw, &crate::geobasis_nrw::WMS),
            (Agency::Lgln, &crate::lgln::WMS),
            (Agency::LvermGeoRp, &crate::lvermgeo_rp::WMS),
            (Agency::Lvgl, &crate::lvgl::WMS),
            (Agency::LglBw, &crate::lgl_bw::WMS),
            (Agency::Ldbv, &crate::ldbv::WMS),
            (Agency::Lgv, &crate::lgv::WMS),
        ];
        for (agency, code) in held {
            let (state, crs) = rasters::addressed(agency).expect("an addressed agency");
            let wms = wms_of(&RasterConfig::default(), state, crs).unwrap();
            assert_eq!(wms.url, code.url, "{agency:?}");
            assert_eq!(wms.layer, code.layer, "{agency:?}");
            assert_eq!(wms.crs, code.crs, "{agency:?}");
            // What the request adds is the code of the backend (the vintage of Hamburg).
            if agency != Agency::Lgv {
                assert_eq!(wms.extra, code.extra, "{agency:?}");
            }
            assert_eq!(wms.max_px, code.max_px, "{agency:?}");
        }
        // LVermGeo declares its rasters as services in its manifest; every other agency is
        // made by its factory.
        for agency in AGENCIES {
            assert_eq!(
                rasters::addressed(agency).is_some(),
                agency != Agency::LvermGeo,
                "{agency:?}"
            );
        }
    }

    /// What the manifests say besides the WMS is what the code held: the addresses of the files.
    #[test]
    fn the_endpoints_of_each_manifest_are_the_ones_the_code_held() {
        let endpoints = |state| builtin_raster_config(state).endpoints;
        let named = |pairs: &[(&str, &str)]| {
            pairs
                .iter()
                .map(|(name, url)| (name.to_string(), url.to_string()))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        assert_eq!(
            endpoints(State::Hamburg),
            named(&[
                ("metadata", crate::lgv::METADATA_URL),
                ("dgm1", crate::lgv::Model::Dgm1.archive_url()),
                ("bdom", crate::lgv::Model::Bdom.archive_url()),
            ])
        );
        assert_eq!(
            endpoints(State::NordrheinWestfalen),
            named(&[("hm", crate::geobasis_nrw::HM)])
        );
        assert_eq!(
            endpoints(State::Bremen),
            named(&[
                ("wms", crate::geobremen::ORTHOPHOTOS),
                ("files", crate::geobremen::FILES),
            ])
        );
        assert_eq!(
            endpoints(State::MecklenburgVorpommern),
            named(&[
                ("overview", crate::laiv_mv::OVERVIEW),
                ("dop-wcs", &crate::laiv_mv::DOP.url),
                ("dgm-wcs", &crate::laiv_mv::Model::Dgm1.wcs().url),
                ("dom-wcs", &crate::laiv_mv::Model::Dom1.wcs().url),
            ])
        );
        assert_eq!(
            endpoints(State::Niedersachsen),
            named(&[
                ("dop-stac", crate::lgln::DOP_STAC),
                ("dgm-stac", crate::lgln::DGM_STAC),
                ("dom-stac", crate::lgln::DOM_STAC),
                ("dgm-wcs", &crate::lgln::DGM1_WCS.url),
                ("dom-wcs", &crate::lgln::DOM1_WCS.url),
            ])
        );
        assert_eq!(
            endpoints(State::BadenWuerttemberg),
            named(&[
                ("flights", crate::lgl_bw::FLIGHTS),
                ("files", crate::lgl_bw::FILES),
            ])
        );
        assert_eq!(
            endpoints(State::Saarland),
            named(&[("share", crate::lvgl::SHARE)])
        );
        assert_eq!(
            endpoints(State::RheinlandPfalz),
            named(&[
                ("tile-list", crate::lvermgeo_rp::TILE_LIST_URL),
                ("data", crate::lvermgeo_rp::DATA),
            ])
        );
        assert_eq!(
            endpoints(State::Hessen),
            named(&[("rest", crate::hvbg::REST_URL)])
        );
        assert_eq!(
            endpoints(State::SchleswigHolstein),
            named(&[
                ("portal", crate::lvermgeo_sh::PORTAL),
                ("atom", crate::lvermgeo_sh::ATOM_DATA),
            ])
        );
        assert_eq!(
            endpoints(State::Saxony),
            named(&[
                ("index", crate::geosn::INDEX_URL),
                ("dgm1", crate::lgb::GEOSN_DGM1_SHARE),
                ("dom1", crate::lgb::GEOSN_DOM1_SHARE),
            ])
        );
        assert_eq!(
            endpoints(State::Bayern),
            named(&[("files", crate::ldbv::FILES)])
        );
        // The folders of LGB, which the constants of `lgb.rs` held; Berlin asks the same.
        for state in [State::Brandenburg, State::Berlin] {
            assert_eq!(
                endpoints(state),
                named(&[
                    ("data", "https://data.geobasis-bb.de/geobasis/daten"),
                    (
                        "currency",
                        "https://data.geobasis-bb.de/geobasis/information/aktualitaeten"
                    ),
                ]),
                "{state:?}"
            );
        }
    }

    /// A folder that replaces a state states the keys it changes; the others come from the
    /// built-in manifest, key by key.
    #[test]
    fn a_config_overrides_key_by_key_and_the_rest_is_the_built_in_one() {
        let (state, crs) = (State::Saxony, Crs::Utm33);
        let base = wms_of(&RasterConfig::default(), state, crs).unwrap();
        let url = wms_of(
            &RasterConfig {
                url: Some("https://example.org/wms".to_string()),
                ..RasterConfig::default()
            },
            state,
            crs,
        )
        .unwrap();
        assert_eq!(url.url, "https://example.org/wms");
        assert_eq!((&url.layer, url.max_px), (&base.layer, base.max_px));
        let layer = wms_of(
            &RasterConfig {
                layer: Some("other".to_string()),
                ..RasterConfig::default()
            },
            state,
            crs,
        )
        .unwrap();
        assert_eq!(layer.layer, "other");
        assert_eq!((&layer.url, layer.max_px), (&base.url, base.max_px));
        let size = wms_of(
            &RasterConfig {
                max_px: Some(1234),
                ..RasterConfig::default()
            },
            state,
            crs,
        )
        .unwrap();
        assert_eq!(size.max_px, 1234);
        assert_eq!((&size.url, &size.layer), (&base.url, &base.layer));
    }

    /// A project folder with its raster records, for the backends to fetch into.
    struct BareProject {
        dir: std::path::PathBuf,
        tiles: Vec<freemapper_adapter_api::records::TileRecord>,
        ndsm_tiles: Vec<freemapper_adapter_api::records::NdsmRecord>,
        orthophoto: Option<freemapper_adapter_api::records::WmsRecord>,
        terrain: Option<freemapper_adapter_api::records::WcsRecord>,
        surface: Option<freemapper_adapter_api::records::WcsRecord>,
    }

    impl freemapper_adapter_api::project::RasterProject for BareProject {
        fn records(&mut self) -> freemapper_adapter_api::project::RasterRecords<'_> {
            freemapper_adapter_api::project::RasterRecords {
                tiles: &mut self.tiles,
                ndsm_tiles: &mut self.ndsm_tiles,
                orthophoto: &mut self.orthophoto,
                terrain: &mut self.terrain,
                surface: &mut self.surface,
            }
        }

        fn save(&mut self) -> Result<(), freemapper_adapter_api::error::AdapterError> {
            Ok(())
        }

        fn dir(&self) -> &Path {
            &self.dir
        }

        fn region_bbox(&self) -> BoundingBox {
            BoundingBox {
                min_e: 500_000.0,
                min_n: 5_500_000.0,
                max_e: 500_100.0,
                max_n: 5_500_100.0,
            }
        }

        fn crs(&self) -> Crs {
            Crs::Utm32
        }
    }

    /// A backend that takes addresses asks the server when it fetches, and asks the WMS its
    /// config names instead of the one of the built-in manifest.
    #[test]
    fn an_addressed_backend_fetches_from_the_wms_its_config_states() {
        use crate::fetch::test_support::MapFetcher;
        use freemapper_adapter_api::env::AdapterEnv;
        use freemapper_adapter_api::fetch::FetchMode;
        use freemapper_adapter_api::project::RasterProject as _;
        let mut registry = Registry::new();
        register_backends(&mut registry).unwrap();
        for agency in AGENCIES {
            let Some((state, _)) = rasters::addressed(agency) else {
                continue;
            };
            let name = backend_name(agency).unwrap();
            // A backend with a WMS asks the one its config names instead of the built-in one.
            let builtin = builtin_raster_config(state).url.unwrap_or_default();
            let config = RasterConfig {
                url: rasters::takes_wms(agency).then(|| "https://wms.example.org/dop".to_string()),
                ..RasterConfig::default()
            };
            let backend = registry
                .build_backend(name, &config)
                .unwrap_or_else(|| panic!("{name}"))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            let scratch = tempfile::tempdir().unwrap();
            let cache = crate::fetch::Cache::new(scratch.path().join("cache"));
            let fetcher = MapFetcher::default();
            let mut project = BareProject {
                dir: scratch.path().join("project"),
                tiles: Vec::new(),
                ndsm_tiles: Vec::new(),
                orthophoto: None,
                terrain: None,
                surface: None,
            };
            std::fs::create_dir_all(&project.dir).unwrap();
            let mut notes = Vec::new();
            let mut env = AdapterEnv {
                fetcher: &fetcher,
                regions: &registry,
                cache: &cache,
                mode: FetchMode::Online,
                today: "2026-11-13".parse().unwrap(),
                skip_surface: false,
                progress: &mut |n| notes.push(n.to_string()),
            };
            // The server serves nothing, so the fetch may fail; what it asked is the point.
            let bbox = project.region_bbox();
            let _ = backend.fetch(&mut env, &mut project, &bbox);
            let log = fetcher.log.borrow();
            assert!(!log.is_empty(), "{name} asked nothing");
            assert!(
                builtin.is_empty() || !log.iter().any(|url| url.starts_with(&builtin)),
                "{name} asked the WMS of the built-in manifest: {log:?}"
            );
        }
    }

    /// A state whose manifest names no addressed backend has no addresses to fall back on, and
    /// an agency that keeps its addresses in code makes no backend from a WMS.
    #[test]
    fn without_addresses_there_is_no_wms_and_no_backend() {
        let none = wms_of(&RasterConfig::default(), State::Brandenburg, Crs::Utm32).unwrap_err();
        assert!(none.contains("brandenburg"), "{none}");
        // Brandenburg states the folders of LGB's tiles, but no WMS.
        let brandenburg = builtin_raster_config(State::Brandenburg);
        assert!(brandenburg.url.is_none() && !brandenburg.endpoints.is_empty());
        assert_ne!(
            builtin_raster_config(State::Saxony),
            RasterConfig::default()
        );
        let wms = wms_of(
            &RasterConfig {
                url: Some("https://example.org/wms".to_string()),
                layer: Some("l".to_string()),
                max_px: Some(500),
                ..RasterConfig::default()
            },
            State::Brandenburg,
            Crs::Utm32,
        )
        .unwrap();
        assert_eq!(
            (wms.url.as_ref(), wms.layer.as_ref(), wms.max_px),
            ("https://example.org/wms", "l", 500)
        );
        assert!(
            rasters::backend_with_wms(Agency::LvermGeo, wms.clone(), &RasterConfig::default())
                .unwrap()
                .is_none()
        );
        let built =
            raster_factory(Agency::LvermGeo, State::Brandenburg, Crs::Utm32)(&RasterConfig {
                url: Some("https://example.org/wms".to_string()),
                layer: Some("l".to_string()),
                max_px: Some(500),
                ..RasterConfig::default()
            });
        assert!(
            built
                .err()
                .expect("an error")
                .contains("takes no addresses")
        );
    }

    /// The backends that take addresses keep their provider and cache folder, which the cache
    /// of every person already holds.
    #[test]
    fn an_addressed_backend_keeps_its_provider_and_its_cache_folder() {
        let mut registry = Registry::new();
        register_backends(&mut registry).unwrap();
        for (name, id, cache) in [
            ("geosn", "geosn", Some("geosn")),
            ("hvbg", "hvbg", Some("hvbg")),
            ("lvermgeo-sh", "lvermgeosh", Some("lvermgeo_sh")),
            ("geobasis-nrw", "geobasisnrw", Some("geobasis_nrw")),
            ("lgln", "lgln", None),
            ("lvermgeo-rp", "lvermgeorp", Some("lvermgeo_rp")),
            ("lvgl", "lvgl", Some("lvgl")),
            ("lgl-bw", "lglbw", Some("lglbw")),
            ("ldbv", "ldbv", Some("ldbv")),
            ("lgv", "lgv", Some("lgv")),
            ("laiv-mv", "laivmv", Some("laiv_mv")),
            ("geobremen", "geobremen", Some("geobremen")),
            ("lgb", "lgb", Some("lgb")),
        ] {
            let backend = registry.backend(name).expect("a backend of that name");
            assert_eq!(backend.provider().id.as_str(), id, "{name}");
            assert_eq!(backend.provider().cache_dir.as_deref(), cache, "{name}");
        }
    }

    /// Each state with point clouds reads its own backend, made of the table of its manifest;
    /// the others have none and no table.
    #[test]
    fn a_state_reads_the_point_cloud_backend_of_its_table() {
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        for state in State::ALL {
            let source = StateAdapter::new(state, &backends)
                .point_cloud()
                .map(|backend| backend.info().source.clone());
            let expected = match state {
                State::Brandenburg => Some("lgb-als"),
                State::Berlin => Some("berlin-als"),
                State::NordrheinWestfalen => Some("geobasisnrw-3dm"),
                State::RheinlandPfalz => Some("lvermgeorp-las"),
                State::Saarland => Some("lvgl-las"),
                State::Bayern => Some("ldbv-las"),
                _ => None,
            };
            assert_eq!(source.as_deref(), expected, "{state:?}");
            assert_eq!(
                point_cloud_of(state).is_some(),
                expected.is_some(),
                "{state:?}"
            );
        }
    }

    /// The export of a state is its manifest and the files the manifest names, nothing else;
    /// only Saxony-Anhalt, which declares its orthophoto as a service, gets a provider of its
    /// own, and every export parses and finds its files.
    #[test]
    fn the_export_of_a_state_is_its_manifest_and_its_files() {
        assert!(export_files("polska").is_none());
        assert!(export_files("").is_none());
        for state in State::ALL {
            let files = export_files(state.id()).unwrap();
            let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
            let extracts = match state {
                State::NordrheinWestfalen => 5,
                State::BadenWuerttemberg => 4,
                State::Bayern => 7,
                _ => 0,
            };
            assert_eq!(files.len(), 2 + extracts, "{state:?}: {names:?}");
            assert_eq!(names[0], "manifest.toml");
            assert_eq!(names[1], "boundary.geojson");
            assert!(files[1].1.contains("coordinates"), "{state:?}");
            let manifest = freemapper_manifest::parse(&files[0].1, Path::new(state.id()))
                .unwrap_or_else(|findings| panic!("{state:?}: {findings:?}"));
            assert_eq!(manifest.adapter.id, state.id());
            let built_in = manifest_text(state);
            if state == State::SaxonyAnhalt {
                assert_eq!(manifest.adapter.provider, "saxonyanhalt");
                assert_eq!(
                    files[0]
                        .1
                        .replace("provider = \"saxonyanhalt\"", "provider = \"lvermgeo\""),
                    built_in
                );
            } else {
                assert_eq!(files[0].1, built_in, "{state:?}");
            }
            // Every file the export holds is a file the state reads under that name.
            for (name, text) in &files[1..] {
                assert_eq!(&StateFiles(state).read(name).unwrap(), text, "{name}");
            }
        }
    }

    #[test]
    fn manifest_texts_are_the_manifests_of_all_states_in_order() {
        let texts = manifest_texts();
        assert_eq!(texts.len(), State::ALL.len());
        for (text, state) in texts.iter().zip(State::ALL) {
            assert_eq!(*text, manifest_text(state));
            assert_eq!(*text, shipped_manifest_text(state));
            assert!(text.len() > 100, "{state:?}");
        }
    }

    /// The area page draws one layer per aerial photo. Two states with the same service and
    /// layer (Brandenburg and Berlin) must name it alike in every field, or the page would
    /// show it twice.
    #[test]
    fn states_that_share_a_preview_share_all_of_it() {
        let adapters = adapters();
        let previews: Vec<&PreviewInfo> = adapters
            .iter()
            .map(|adapter| {
                adapter
                    .info()
                    .preview
                    .as_ref()
                    .expect("every state has one")
            })
            .collect();
        let mut services = Vec::new();
        for (index, one) in previews.iter().enumerate() {
            for other in &previews[index + 1..] {
                if (&one.wms, &one.layer) == (&other.wms, &other.layer) {
                    assert_eq!(one, other);
                }
            }
            assert!(one.wms.starts_with("https://"), "{}", one.wms);
            if !services.contains(&(&one.wms, &one.layer)) {
                services.push((&one.wms, &one.layer));
            }
        }
        assert_eq!(services.len(), 14);
        assert_eq!(previews[0], previews[1]);
    }

    /// Berlin's registers and surveys are one set of extra layers, with the layers of each map
    /// kind in the order they are fetched. Hamburg's Feinkartierung is another. Both bring their
    /// rules (ADR 0149). No other state has one.
    #[test]
    fn berlin_and_hamburg_have_a_set_of_extra_layers() {
        use crate::project::MapKind;
        for adapter in adapters() {
            let sets = adapter.extras();
            if adapter.info().id.as_str() == "hamburg" {
                assert_eq!(sets.len(), 1);
                assert_eq!(sets[0].info().key, "lgv-feinkartierung");
                assert!(sets[0].drawing().is_some());
                continue;
            }
            if adapter.info().id.as_str() != "berlin" {
                assert!(sets.is_empty(), "{}", adapter.info().id);
                continue;
            }
            // Berlin's registers and surveys are a set that brings its rules (ADR 0149).
            assert!(sets[0].drawing().is_some_and(|d| d.generic));
            assert_eq!(sets.len(), 1);
            let set = &sets[0];
            let info = set.info();
            assert_eq!(info.key, "berlin");
            assert_eq!(info.crs, crate::crs::Crs::Utm33);
            assert_eq!(
                info.estimate.what,
                "Berlin ALKIS structures, tree cadastre, street survey and point layers"
            );
            assert_eq!(info.estimate.megabytes_per_km2, 12.0);
            assert_eq!(info.notes.download, "progress-berlin-download");
            for kind in [MapKind::Sprint, MapKind::Schoolyard] {
                let keys: Vec<&str> = set.layers(kind).iter().map(|layer| layer.key).collect();
                let files: Vec<&str> = crate::berlin::extra_layers(kind)
                    .map(|layer| layer.file)
                    .collect();
                assert_eq!(keys, files, "{kind:?}");
                assert!(!keys.is_empty());
            }
            assert!(set.has("gebaeudehoehen"));
            assert!(!set.has("notes"));
        }
    }

    /// A built-in raster backend answers the estimate with the raster items of its state, so
    /// an adapter of a folder that names it counts them too (issue #71); the cadastre files of
    /// Bavaria are no rasters and stay out. The states keep their own estimate.
    #[test]
    fn built_in_backends_estimate_their_rasters() {
        use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
        use freemapper_adapter_api::records::Part;
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        let root = tempfile::tempdir().unwrap();
        let cache = freemapper_adapter_api::cache::CacheDir::new(root.path().to_path_buf());
        let env = EstimateEnv { cache: &cache };
        let mut checked = 0;
        for state in State::ALL {
            let Some(name) = backend_name(state.agency()) else {
                continue;
            };
            let backend = backends.backend(name).expect("registered");
            // 2 km around the middle of the state's box (EPSG:25833).
            let b = outline(state.id()).unwrap().bounds();
            let (e, n) = ((b.min_e + b.max_e) / 2.0, (b.min_n + b.max_n) / 2.0);
            let bbox = BoundingBox {
                min_e: e - 1000.0,
                min_n: n - 1000.0,
                max_e: e + 1000.0,
                max_n: n + 1000.0,
            };
            let area = EstimateArea {
                bbox_25833: bbox,
                map_bbox: bbox,
                today: chrono::NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
            };
            let of_backend = backend.estimate(&area, &env);
            let rasters: Vec<_> = estimate::items(state, &area, &env)
                .into_iter()
                .filter(|i| matches!(i.part, Part::Orthophoto | Part::Terrain | Part::Surface))
                .collect();
            assert_eq!(of_backend, rasters, "{}", state.id());
            assert!(
                of_backend.iter().any(|i| i.part == Part::Orthophoto),
                "{}: no orthophoto",
                state.id()
            );
            checked += 1;
        }
        assert_eq!(checked, 14, "every state but Saxony-Anhalt names a backend");
    }

    /// Berlin brings the six services of the Geoportal Berlin for the permit notes (ADR 0120,
    /// ADR 0124), Brandenburg the Reviere of its forest administration and North
    /// Rhine-Westphalia the forest districts and the Staatswald (ADR 0124), all on hosts of
    /// their manifest. No other state brings one yet.
    #[test]
    fn permit_sources_by_state() {
        for adapter in adapters() {
            let id = adapter.info().id.as_str();
            let keys: Vec<&str> = adapter
                .permit_sources()
                .iter()
                .map(|s| s.info().key)
                .collect();
            let (expected, url): (&[&str], &str) = match id {
                "berlin" => (
                    &[
                        "bezirke",
                        "gruenanlagen",
                        "liegenschaftsplan",
                        "denkmale",
                        "schutzgebiete",
                        "forstverwalt",
                        "wsg",
                        "biotope",
                    ],
                    "https://gdi.berlin.de/services/wfs/forstverwalt",
                ),
                "brandenburg" => (
                    &[
                        "reviere",
                        "waldfunktionen",
                        "baudenkmale",
                        "bodendenkmale",
                        "naturdenkmale",
                        "biotope",
                        "wasserschutzgebiete",
                    ],
                    "https://gis-bldam-brandenburg.de/ows/baudenkmale",
                ),
                "hamburg" => (
                    &[
                        "bezirke",
                        "gruenanlagen",
                        "denkmale",
                        "wald",
                        "waldfunktionen",
                        "wasserschutzgebiete",
                        "naturdenkmale",
                        "biotope",
                    ],
                    "https://geodienste.hamburg.de/HH_WFS_Wasserschutzgebiete",
                ),
                "saarland" => (
                    &[
                        "forst",
                        "schutzgebiete",
                        "denkmale",
                        "naturdenkmale",
                        "biotope",
                    ],
                    "https://geoportal.saarland.de/arcgis/services/Internet/Denkmalschutz/MapServer/WFSServer",
                ),
                "hessen" => (
                    &["wasserschutzgebiete", "denkmale", "biotope"],
                    "https://geodienste.denkx.de/geoserver/denkx/wfs",
                ),
                "schleswigholstein" => (
                    &["umwelt", "naturschutz", "denkmale"],
                    "https://opendata.schleswig-holstein.de/collection/geodaten-denkmalliste-sh/aktuell.geojson",
                ),
                "badenwuerttemberg" => (
                    &[
                        "lubw",
                        "denkmale",
                        "archaeologie",
                        "naturdenkmale",
                        "flaechennaturdenkmale",
                    ],
                    "https://rips-gdi.lubw.baden-wuerttemberg.de/arcgis/services/wms/UIS_0100007500200036_AGGR/MapServer/WFSServer",
                ),
                "saxony" => (
                    &[
                        "bfg",
                        "denkmale",
                        "naturdenkmale",
                        "biotope",
                        "sachsenforst",
                    ],
                    "https://cardomap3.idu.de/lfds/public/ogc.ashx?PKGID=5",
                ),
                "bremen" => (
                    &["bfg", "denkmale"],
                    "https://geodienste.bremen.de/wfs_kulturdenkmale",
                ),
                "rheinlandpfalz" => (
                    &[
                        "bfg",
                        "denkmale",
                        "naturdenkmale",
                        "landschaftsbestandteile",
                    ],
                    "https://www.geoportal.rlp.de/owsproxy/00000000000000000000000000000000/a2e0fb04feed3a72cf42d53cf6e2165e",
                ),
                "bayern" | "saxonyanhalt" => (
                    &["bfg", "naturschutz"],
                    "https://geoportal.bafg.de/arcgis1/rest/services/INSPIRE/AM/MapServer/1/query",
                ),
                "niedersachsen" => (
                    &["bfg", "naturschutz"],
                    "https://www.umweltkarten-niedersachsen.de/arcgis/rest/services/Natur_wms/MapServer",
                ),
                "mecklenburgvorpommern" => (
                    &["bfg"],
                    "https://geoportal.bafg.de/arcgis1/rest/services/INSPIRE/AM/MapServer/1/query",
                ),
                "nordrheinwestfalen" => (
                    &[
                        "linfos",
                        "denkmale",
                        "biotope",
                        "forstbetriebsbezirke",
                        "staatswald",
                        "wasserschutzgebiete",
                    ],
                    "https://www.opengeodata.nrw.de/produkte/umwelt_klima/wald_forst/wald/x.zip",
                ),
                _ => {
                    assert!(keys.is_empty(), "{id}");
                    continue;
                }
            };
            assert_eq!(keys, expected, "{id}");
            assert!(adapter.policy().unwrap().check_url(url).is_ok(), "{id}");
        }
    }

    /// Berlin's buildings come from a service of its own, which a fetch can miss while the
    /// rest arrives: Berlin alone has a warning for an area of its ground without buildings.
    #[test]
    fn berlin_alone_warns_of_an_area_without_buildings() {
        for adapter in adapters() {
            let berlin = adapter.info().id.as_str() == "berlin";
            assert_eq!(
                adapter.no_buildings_warning(),
                berlin.then_some("warning-berlin-buildings-missing"),
                "{}",
                adapter.info().id
            );
        }
    }

    /// Berlin has a map of its own to lay under a map: the K5 (1:5000) is its one raster
    /// template. No other state has one.
    #[test]
    fn berlin_alone_has_a_raster_template() {
        for adapter in adapters() {
            let templates = adapter.templates();
            if adapter.info().id.as_str() != "berlin" {
                assert!(templates.is_empty(), "{}", adapter.info().id);
                continue;
            }
            assert_eq!(templates.len(), 1);
            let info = templates[0].info();
            assert_eq!(info.key, "k5");
            assert_eq!(info.crs, crate::crs::Crs::Utm33);
            assert_eq!(info.dataset_date, "2026-01-02");
            assert_eq!(info.dataset_label, "K5 dataset");
            assert_eq!(info.estimate.what, "Berlin K5 map (1:5000) template");
            assert_eq!(info.estimate.megabytes_per_km2, 15.0);
            assert_eq!(info.notes.download, "progress-k-five-download");
        }
    }

    /// Saxony-Anhalt has no raster code: its manifest declares the services. Every other
    /// state names a backend of this crate, and Brandenburg and Berlin name the same.
    #[test]
    fn every_state_has_its_rasters() {
        let adapters = adapters();
        for (state, adapter) in State::ALL.into_iter().zip(&adapters) {
            let provider = adapter
                .rasters()
                .expect("every state has rasters")
                .provider();
            assert_eq!(provider.id.as_str(), state.agency().id(), "{}", state.id());
            assert_eq!(
                provider.terrain_source,
                state.agency().terrain_source(),
                "{}",
                state.id()
            );
        }
        let mut registry = Registry::new();
        register(&mut registry).unwrap();
        assert_eq!(registry.len(), 15);
        assert_eq!(
            registry.backend_names().collect::<Vec<_>>(),
            [
                "geobasis-nrw",
                "geobremen",
                "geosn",
                "hvbg",
                "laiv-mv",
                "ldbv",
                "lgb",
                "lgl-bw",
                "lgln",
                "lgv",
                "lvermgeo-rp",
                "lvermgeo-sh",
                "lvgl"
            ]
        );
        // A second registration is refused: a name is taken once.
        assert!(register(&mut registry).is_err());
    }
}
