//! The German states as region adapters. Each state is its manifest
//! (`manifests/<id>/manifest.toml` with its boundary file) plus what no manifest says: the
//! raster code of its agency, the credit lines that depend on more than a year, the ALKIS of
//! Berlin and the download estimate.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::error::Remedy;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::extras::{NasService, TreeDataset};
use freemapper_adapter_api::geometry::Outline;
#[cfg(test)]
use freemapper_adapter_api::info::PreviewInfo;
use freemapper_adapter_api::info::{Declared, RegionInfo};
use freemapper_adapter_api::pointcloud::PointCloudBackend;
use freemapper_adapter_api::policy::UrlPolicy;
use freemapper_adapter_api::rasters::RasterBackend;
use freemapper_adapter_api::records::EstimateItem;
use freemapper_adapter_api::registry::{DuplicateBackend, Registry};
use freemapper_manifest::adapter::{AdapterFiles, ManifestAdapter, Parse};
use freemapper_manifest::guard::GuardedAlkis;
use freemapper_sources::trees::POTSDAM;
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
        )
    }
}

impl State {
    /// All states, in the order they are listed and queried.
    pub const ALL: [Self; 12] = [
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
            | Self::Saarland => Crs::Utm32,
            _ => Crs::Utm33,
        }
    }
}

/// The manifests of the states, in the order of [`State::ALL`]. A manifest holds what a state
/// declares: who it is, its OSM extracts, its ALKIS service, its credit, its preview and which
/// code fetches its rasters.
const MANIFESTS: [&str; 12] = [
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

/// The text of the manifest of `state`.
pub fn manifest_text(state: State) -> &'static str {
    let index = State::ALL
        .iter()
        .position(|other| *other == state)
        .expect("every state is listed");
    MANIFESTS[index]
}

/// The files the manifest of a state names, embedded in the program.
pub struct StateFiles(pub State);

impl AdapterFiles for StateFiles {
    fn read(&self, name: &str) -> Result<Cow<'static, str>, String> {
        let text = match name {
            "boundary.geojson" => boundary_text(self.0.id()),
            _ if self.0 == State::NordrheinWestfalen => NRW_EXTRACT_BOUNDARIES
                .iter()
                .find(|(file, _)| *file == name)
                .map(|(_, text)| *text),
            _ => None,
        };
        text.map(Cow::Borrowed)
            .ok_or_else(|| "the file is not built in".to_string())
    }
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
    }
}

/// The agencies whose rasters code of this crate fetches, in the order of the states.
const AGENCIES: [Agency; 11] = [
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
];

/// Adds the raster backends of this crate to `registry`, each under the name the manifests
/// call it by.
pub fn register_backends(registry: &mut Registry) -> Result<(), DuplicateBackend> {
    for agency in AGENCIES {
        if let Some(name) = backend_name(agency) {
            registry.add_backend(name, Arc::from(rasters::backend(agency)))?;
        }
    }
    Ok(())
}

/// The adapter of one state: its manifest, and what no manifest says.
///
/// From the manifest come the identity, the OSM extracts and their boundaries, the ALKIS
/// service, the full ALKIS and the tree cadastres, the raster backend (by name, or the services
/// the manifest declares) and the credit of the states whose credit is one line with a year.
/// Code are the ALKIS of Berlin, the credits of Brandenburg, Berlin and Saxony (they depend on
/// tiles, on the neighbour and on the layers that drew objects) and the download estimate.
/// Whatever is asked for the state, by its manifest or by its code, stays on the hosts the
/// manifest declares.
pub struct StateAdapter {
    state: State,
    manifest: ManifestAdapter,
    /// The ALKIS source where the manifest declares none (Berlin).
    alkis: Option<Box<dyn AlkisSource>>,
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
        let manifest =
            ManifestAdapter::new(manifest, &StateFiles(state), Parse::OnFirstUse, backends)
                .unwrap_or_else(|findings| broken(findings))
                .held_to_its_hosts(Remedy::AllowHost, extra);
        let policy = manifest
            .policy()
            .expect("the manifest was just bound to its hosts")
            .clone();
        let alkis = manifest.alkis().is_none().then(|| {
            Box::new(GuardedAlkis {
                inner: alkis::source(state),
                policy: Arc::new(policy),
            }) as Box<dyn AlkisSource>
        });
        Self {
            state,
            alkis,
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

    fn nas(&self) -> Option<&NasService> {
        self.manifest.nas()
    }

    fn tree_datasets(&self) -> &[TreeDataset] {
        self.manifest.tree_datasets()
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
        match self.state {
            State::NordrheinWestfalen => Some(crate::nrw_3dm::backend()),
            State::RheinlandPfalz => Some(crate::lvermgeo_rp_las::backend()),
            State::Berlin => Some(crate::berlin_als::backend()),
            state => crate::lgb_als::point_cloud(state),
        }
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

/// The tree cadastres of all states, in the order of the states.
pub(crate) fn all_tree_datasets() -> Vec<TreeDataset> {
    vec![POTSDAM]
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(services.len(), 11);
        assert_eq!(previews[0], previews[1]);
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
        assert_eq!(registry.len(), 12);
        assert_eq!(
            registry.backend_names().collect::<Vec<_>>(),
            [
                "geobasis-nrw",
                "geosn",
                "hvbg",
                "laiv-mv",
                "lgb",
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
