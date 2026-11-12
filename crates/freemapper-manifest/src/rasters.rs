//! `[orthophoto]`, `[terrain]` and `[surface]` of a manifest that are standard services: the
//! services of the readers, and the raster backend that fetches them for a map area.

use std::borrow::Cow;
use std::path::Path;

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::fetch::{FetchMode, Fetcher};
use freemapper_adapter_api::note::{self, Arg};
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::{AdapterId, EstimateItem, WcsRecord, stored_bbox};
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_sources::float_tiff::write_float_raster;
use freemapper_sources::raster::{GridSpec, Raster};
use freemapper_sources::raster_fetch::{
    SURFACE_RELPATH, TERRAIN_RELPATH, fetch_wms_orthophoto, keep_stored_surface,
    stored_surface_or_drop, warn_beyond_state,
};
use freemapper_sources::wcs::{WcsOrthophoto, WcsService};
use freemapper_sources::wms::{OrthophotoSource, WmsService};

use crate::dates;
use crate::model::{DateSource, HeightModel, Manifest, Orthophoto};

/// More parameters of a request as the readers append them: `&NAME=value`, the value
/// percent-encoded.
pub fn query_of(params: &[(String, String)]) -> String {
    let mut query = String::new();
    for (name, value) in params {
        query.push('&');
        query.push_str(name);
        query.push('=');
        for byte in value.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                query.push(char::from(byte));
            } else {
                query.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    query
}

/// The service of an `[orthophoto]` table; `None` when a backend fetches the orthophoto.
pub fn orthophoto_source(orthophoto: &Orthophoto) -> Option<OrthophotoSource> {
    match orthophoto {
        Orthophoto::Wms {
            url,
            layer,
            crs,
            params,
            max_px,
            ..
        } => Some(OrthophotoSource::Wms(WmsService {
            url: Cow::Owned(url.clone()),
            layer: Cow::Owned(layer.clone()),
            crs: *crs,
            extra: Cow::Owned(query_of(params)),
            max_px: *max_px,
        })),
        Orthophoto::Wcs {
            url,
            coverage,
            crs,
            max_px,
            ..
        } => Some(OrthophotoSource::Wcs(WcsOrthophoto {
            url: Cow::Owned(url.clone()),
            coverage: Cow::Owned(coverage.clone()),
            crs: *crs,
            max_px: *max_px,
        })),
        Orthophoto::Backend { .. } => None,
    }
}

/// The WCS of a `[terrain]` or `[surface]` table; `None` when a backend fetches the model.
pub fn height_service(model: &HeightModel) -> Option<WcsService> {
    match model {
        HeightModel::Wcs {
            url,
            coverage,
            params,
            multipart,
            envelope: [min_e, min_n, max_e, max_n],
            max_m,
            ..
        } => Some(WcsService {
            url: Cow::Owned(url.clone()),
            coverage: Cow::Owned(coverage.clone()),
            extra: Cow::Owned(query_of(params)),
            multipart: *multipart,
            envelope: BoundingBox {
                min_e: *min_e,
                min_n: *min_n,
                max_e: *max_e,
                max_n: *max_n,
            },
            max_m: *max_m,
        }),
        HeightModel::Backend { .. } => None,
    }
}

/// A height model that is a WCS: the service, its name in messages and where its date is asked.
struct Model {
    service: WcsService,
    label: String,
    date: DateSource,
}

impl Model {
    fn of(model: &HeightModel) -> Option<Self> {
        match model {
            HeightModel::Wcs { label, date, .. } => Some(Self {
                service: height_service(model)?,
                label: label.clone(),
                date: date.clone(),
            }),
            HeightModel::Backend { .. } => None,
        }
    }

    /// Downloads the model over `bbox` in pieces and writes it as a Float32 GeoTIFF `out` with
    /// its world file.
    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &BoundingBox,
        root: &Path,
        out: &Path,
    ) -> Result<(), FetchError> {
        let grid = GridSpec::covering(bbox, 1.0);
        let mut raster = Raster::filled(grid, f32::NAN);
        self.service.fill(fetcher, &mut raster, &grid.bounds())?;
        write_float_raster(&raster, root, out, &self.service.url)
    }
}

/// The rasters of a manifest whose raster tables are services: the orthophoto from a WMS or a
/// WCS, the terrain model and, if declared, the surface model from a WCS. They are requested for
/// the map area and stored in the project folder.
pub struct DeclaredRasters {
    info: ProviderInfo,
    /// The short name of who serves the rasters, in messages.
    agency: String,
    crs: Crs,
    orthophoto: OrthophotoSource,
    orthophoto_date: DateSource,
    terrain: Model,
    surface: Option<Model>,
    manifest: Manifest,
}

impl DeclaredRasters {
    /// The backend of the raster tables of `manifest`; `None` unless its orthophoto and its
    /// terrain model are services.
    pub fn new(manifest: &Manifest) -> Option<Self> {
        let orthophoto = manifest.orthophoto.as_ref()?;
        let terrain = manifest.terrain.as_ref()?;
        let orthophoto_date = match orthophoto {
            Orthophoto::Wms { date, .. } | Orthophoto::Wcs { date, .. } => date.clone(),
            Orthophoto::Backend { .. } => return None,
        };
        let provider: AdapterId = manifest.adapter.provider.parse().ok()?;
        let terrain_source = match terrain {
            HeightModel::Wcs {
                source_label: Some(label),
                ..
            } => label.clone(),
            _ => format!("{}-dgm1", manifest.adapter.provider),
        };
        Some(Self {
            info: ProviderInfo {
                id: provider,
                cache_dir: None,
                terrain_source,
                kind: RasterKind::PerArea,
            },
            agency: manifest.adapter.agency.clone(),
            crs: manifest.adapter.crs,
            orthophoto: orthophoto_source(orthophoto)?,
            orthophoto_date,
            terrain: Model::of(terrain)?,
            surface: match &manifest.surface {
                Some(surface) => Some(Model::of(surface)?),
                None => None,
            },
            manifest: manifest.clone(),
        })
    }

    /// The address and the CRS of the WMS of the orthophoto, if it is one.
    fn wms(&self) -> Option<(&str, Crs)> {
        match &self.orthophoto {
            OrthophotoSource::Wms(wms) => Some((&wms.url, wms.crs)),
            OrthophotoSource::Wcs(_) => None,
        }
    }
}

impl RasterBackend for DeclaredRasters {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    /// The orthophoto, the terrain model and the surface model of `bbox`. The surface model is
    /// only a template: an unavailable one costs a warning.
    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        let (fetcher, mode, skip_surface) = (env.fetcher, env.mode, env.skip_surface);
        let dir = &project.dir().to_path_buf();
        project.records().tiles.clear();
        project.records().ndsm_tiles.clear();
        warn_beyond_state(env, project, self.info.id.as_str(), &self.agency);
        let date_of = |date: &DateSource| dates::lookup(date, fetcher, self.wms(), self.crs, bbox);
        let flight = || date_of(&self.orthophoto_date);
        fetch_wms_orthophoto(
            env,
            project,
            bbox,
            (&self.orthophoto, &self.agency, &flight, None),
        )?;
        let record = |date: Option<String>| WcsRecord {
            fetched: env.today,
            bbox: stored_bbox(bbox),
            date,
        };
        match mode {
            FetchMode::Online => {
                (env.progress)(freemapper_adapter_api::note!(
                    "progress-model-download",
                    model = note::model(&self.terrain.label)
                ));
                self.terrain
                    .fetch(fetcher, bbox, dir, &dir.join(TERRAIN_RELPATH))?;
                *project.records().terrain = Some(record(date_of(&self.terrain.date)));
                let Some(surface) = &self.surface else {
                    // No surface model here: one stored for another area is of no use.
                    stored_surface_or_drop(mode, project, bbox);
                    project.save()?;
                    return Ok(());
                };
                if skip_surface {
                    keep_stored_surface(env, project, bbox);
                    project.save()?;
                    return Ok(());
                }
                (env.progress)(freemapper_adapter_api::note!(
                    "progress-model-download",
                    model = note::model(&surface.label)
                ));
                match surface.fetch(fetcher, bbox, dir, &dir.join(SURFACE_RELPATH)) {
                    Ok(()) => {
                        *project.records().surface = Some(record(date_of(&surface.date)));
                    }
                    Err(e) => (env.progress)(freemapper_adapter_api::note!(
                        "warning-model-unavailable",
                        model = note::model(&surface.label),
                        error = Arg::error(e)
                    )),
                }
            }
            FetchMode::Offline => {
                let stored = |r: &Option<WcsRecord>, relpath: &str| {
                    r.as_ref().is_some_and(|r| r.covers(bbox)) && dir.join(relpath).exists()
                };
                if !stored(&*project.records().terrain, TERRAIN_RELPATH) {
                    return Err(AdapterError::TerrainNotFetched);
                }
                if self.surface.is_some() && !stored(&*project.records().surface, SURFACE_RELPATH) {
                    (env.progress)(freemapper_adapter_api::note!("warning-no-surface-stored"));
                }
            }
        }
        project.save()?;
        Ok(())
    }

    fn estimate(&self, area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        crate::estimate::declared_items(&self.manifest, area)
    }
}
