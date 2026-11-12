//! The steps of a raster fetch that name no state: the orthophoto from a WMS or WCS, the terrain
//! and the surface model of a provider that serves them per area, and what happens to a surface
//! model that is left out or fails. The raster backends of the adapters are built from them.

use std::path::Path;

use chrono::NaiveDate;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, tile_names};
use freemapper_adapter_api::geometry::{provider_tiles_in, regions_for_in};
use freemapper_adapter_api::project::{RasterProject, RasterRecords};
use freemapper_adapter_api::records::{TileRecord, WcsRecord, WmsRecord, stored_bbox};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode, FetchedTile};
use crate::height_model::HeightModel;
use crate::note::{Arg, Note, note};
use crate::tiles::{TileId, tiles_covering_grid};
use crate::wms::{DateLabel, OrthophotoSource};

/// The WMS orthophoto (GeoSN, LVermGeo) as fetched, before it becomes the template.
pub const WMS_ORTHOPHOTO_RELPATH: &str = "data/orthophoto-wms.jpg";
/// The WCS terrain model (LVermGeo DGM1, 1 m) with its world file.
pub const TERRAIN_RELPATH: &str = "data/dgm1.tif";
/// The WCS surface model (LVermGeo DOM1, 1 m) with its world file.
pub const SURFACE_RELPATH: &str = "data/dom1.tif";
/// How many days a stored orthophoto is reused while the service gives no flight date (none known,
/// or the query failed) before the next online fetch downloads it again.
pub const UNDATED_ORTHOPHOTO_MAX_AGE_DAYS: i64 = 30;

/// An orthophoto of an earlier map area in another state (GeoSN and the per-area providers) is
/// no longer used by an LGB project.
pub fn drop_wms_orthophoto(project: &mut dyn RasterProject) {
    *project.records().orthophoto = None;
    let dir = project.dir();
    let wms = dir.join(WMS_ORTHOPHOTO_RELPATH);
    for stale in [wms.clone(), wms.with_extension("jgw")] {
        freemapper_fs::remove_inside(dir, &stale);
    }
}

/// The orthophoto of `bbox` from a WMS or WCS into [`WMS_ORTHOPHOTO_RELPATH`]: downloaded again only
/// when the flight date changed (the date tells whether the state flew the area again);
/// offline, the stored image must cover `bbox`. `label` names a date that is no flight date as
/// such, in the record and in the log.
pub fn fetch_wms_orthophoto(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
    (source, provider, flight_date, label): (
        &OrthophotoSource,
        &str,
        &dyn Fn() -> Option<String>,
        Option<&DateLabel>,
    ),
) -> Result<(), AdapterError> {
    let dir = project.dir().to_path_buf();
    let path = dir.join(WMS_ORTHOPHOTO_RELPATH);
    let records = project.records();
    match env.mode {
        FetchMode::Online => {
            let flight = flight_date();
            // Reused while the stored image covers the area and the flight date is the same.
            // Without a date from the service (none known, or the query failed), the image is
            // reused only while it is recent, and keeps the date it has.
            let today = env.today;
            let recent =
                |r: &WmsRecord| (today - r.fetched).num_days() < UNDATED_ORTHOPHOTO_MAX_AGE_DAYS;
            let unchanged = records.orthophoto.as_ref().is_some_and(|r| {
                r.covers(bbox)
                    && if flight.is_some() {
                        r.flight == flight
                    } else {
                        recent(r)
                    }
            }) && path.exists();
            if unchanged {
                let kept = flight
                    .as_deref()
                    .or_else(|| records.orthophoto.as_ref()?.flight.as_deref());
                let shown = label.map_or("flight", |l| l.shown);
                (env.progress)(match kept {
                    Some(date) => {
                        note!(
                            "progress-orthophoto-unchanged-dated",
                            label = shown,
                            date = date
                        )
                    }
                    None => note!("progress-orthophoto-unchanged"),
                });
            } else {
                (env.progress)(note!(
                    "progress-orthophoto-download",
                    provider = provider,
                    service = source.kind()
                ));
                crate::wms::fetch_orthophoto_from(env.fetcher, source, bbox, &dir, &path)?;
                let flight_label = label
                    .filter(|_| flight.is_some())
                    .map(|l| l.record.to_string());
                *records.orthophoto = Some(WmsRecord {
                    flight,
                    flight_label,
                    fetched: env.today,
                    bbox: stored_bbox(bbox),
                });
            }
        }
        FetchMode::Offline => {
            let covered = records.orthophoto.as_ref().is_some_and(|r| r.covers(bbox));
            if !path.exists() || !covered {
                return Err(AdapterError::OrthophotoNotFetched);
            }
        }
    }
    Ok(())
}

/// The record of a height model just read; offline the tiles come from the cache, so the record
/// keeps the date they were fetched.
pub fn height_record(
    mode: FetchMode,
    today: NaiveDate,
    bbox: &BoundingBox,
    found: &HeightModel,
    old: &Option<WcsRecord>,
) -> WcsRecord {
    WcsRecord {
        fetched: match (mode, old) {
            (FetchMode::Offline, Some(old)) => old.fetched,
            _ => today,
        },
        bbox: stored_bbox(bbox),
        // Offline without the portal's tile list there is no date: the stored file is unchanged.
        date: found.date.clone().or_else(|| {
            old.as_ref()
                .filter(|_| mode == FetchMode::Offline)?
                .date
                .clone()
        }),
    }
}

/// The warning for the tiles of `label` that the provider lacks inside `state`.
pub fn warn_missing_tiles(
    progress: &mut dyn FnMut(Note),
    state: &str,
    label: &str,
    found: &HeightModel,
) {
    if !found.missing.is_empty() {
        progress(note!(
            "warning-model-tiles-missing",
            model = crate::note::model(label),
            count = found.missing.len(),
            names = crate::height_model::names(&found.missing),
            // The catalogs name the state by this key.
            state = state.to_lowercase().replace(' ', "-")
        ));
    }
}

/// The terrain model (required) and the surface model (a template: a failure costs a warning;
/// with `skip_surface` it is not read, and the stored one stays while it covers `bbox`) of a
/// per-area provider whose fetch reports a [`HeightModel`](HeightModel).
/// `fetch(surface, out, progress)` reads one model into `out`; `after(surface, found, project, progress)`
/// says what is specific to the provider once a model was read.
#[allow(clippy::type_complexity)]
pub fn fetch_area_heights(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
    state: &str,
    (terrain_label, surface_label): (&str, &str),
    fetch: &mut dyn FnMut(bool, &Path, &mut dyn FnMut(Note)) -> Result<HeightModel, FetchError>,
    after: &mut dyn FnMut(bool, &HeightModel, &RasterRecords<'_>, &mut dyn FnMut(Note)),
) -> Result<(), AdapterError> {
    let dir = project.dir().to_path_buf();
    (env.progress)(note!(
        "progress-model-read",
        model = crate::note::model(terrain_label)
    ));
    let found = fetch(false, &dir.join(TERRAIN_RELPATH), env.progress)?;
    warn_missing_tiles(env.progress, state, terrain_label, &found);
    let records = project.records();
    after(false, &found, &records, env.progress);
    *records.terrain = Some(height_record(
        env.mode,
        env.today,
        bbox,
        &found,
        records.terrain,
    ));
    if env.skip_surface {
        keep_stored_surface(env, project, bbox);
        project.save()?;
        return Ok(());
    }
    (env.progress)(note!(
        "progress-model-read",
        model = crate::note::model(surface_label)
    ));
    match fetch(true, &dir.join(SURFACE_RELPATH), env.progress) {
        Ok(found) => {
            warn_missing_tiles(env.progress, state, surface_label, &found);
            let records = project.records();
            after(true, &found, &records, env.progress);
            *records.surface = Some(height_record(
                env.mode,
                env.today,
                bbox,
                &found,
                records.surface,
            ));
        }
        Err(e) => keep_surface(env, project, bbox, surface_label, Arg::error(e)),
    }
    project.save()?;
    Ok(())
}

/// After the surface model `label` of a per-area provider failed with `e`: the stored one is
/// kept when its record still covers `bbox` (derive uses it), and dropped with its file when it
/// does not; offline nothing is deleted (derive ignores a record that does not cover the area).
pub fn keep_surface(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
    label: &str,
    e: Arg,
) {
    let model = crate::note::model(label);
    match stored_surface_or_drop(env.mode, project, bbox) {
        Some(fetched) => (env.progress)(note!(
            "warning-model-unavailable-kept",
            model = model,
            error = e,
            fetched = fetched.to_string()
        )),
        None => (env.progress)(note!("warning-model-unavailable", model = model, error = e)),
    }
}

/// The surface model of a per-area provider left out for lack of disk space: the stored one stays
/// while it covers `bbox`, and the user hears so.
pub fn keep_stored_surface(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
) {
    if let Some(fetched) = stored_surface_or_drop(env.mode, project, bbox) {
        (env.progress)(note!(
            "progress-surface-kept",
            fetched = fetched.to_string()
        ));
    }
}

/// After the surface model of a tile provider was left out: the cached tiles that still serve.
pub fn say_kept_surface_tiles(tiles: &[&str], progress: &mut dyn FnMut(Note)) {
    if !tiles.is_empty() {
        progress(note!(
            "progress-surface-tiles-kept",
            count = tiles.len(),
            names = tiles.join(", ")
        ));
    }
}

/// The surface model of a per-area provider that is not read this time: the stored one stays
/// while its record covers `bbox` (the day it was fetched is returned); else it is dropped with
/// its file, online only (offline derive ignores a record that does not cover the area).
pub fn stored_surface_or_drop(
    mode: FetchMode,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
) -> Option<NaiveDate> {
    let dir = project.dir().to_path_buf();
    let records = project.records();
    let stored = records
        .surface
        .as_ref()
        .filter(|r| r.covers(bbox) && dir.join(SURFACE_RELPATH).exists());
    if let Some(record) = stored {
        return Some(record.fetched);
    }
    if mode == FetchMode::Online {
        *records.surface = None;
        let surface = dir.join(SURFACE_RELPATH);
        for file in [surface.clone(), surface.with_extension("tfw")] {
            freemapper_fs::remove_inside(&dir, &file);
        }
    }
    None
}

/// The record of a fetched tile; warns when the tile was republished since `previous`.
pub fn tile_record(
    fetched: FetchedTile,
    previous: &[TileRecord],
    progress: &mut dyn FnMut(Note),
) -> TileRecord {
    let record = TileRecord {
        product: fetched.product,
        tile: fetched.tile.name(),
        url: fetched.url,
        currency: fetched.currency.currency(),
        published: fetched.currency.published,
    };
    if let Some(old) = previous.iter().find(|p| {
        p.product == record.product && p.tile == record.tile && p.published != record.published
    }) {
        progress(note!(
            "warning-tile-changed",
            product = format!("{:?}", record.product),
            tile = &record.tile,
            old = old.published.to_string(),
            new = record.published.to_string()
        ));
    }
    record
}

/// Reports once that the map area reaches into a region of another provider than `provider`.
/// Whether the `name` rasters end at the border depends on the service (the LGLN WMS and WCS
/// cover the Hamburg corner but the WMS is white in Bremen; the LVermGeo RP WMS is white across
/// the Rhine at Mainz while its tiles cover Wiesbaden; tiles may reach slightly past), so the
/// line promises no empty templates. ALKIS and OSM are read from both regions.
pub fn warn_beyond_state(
    env: &mut AdapterEnv<'_>,
    project: &dyn RasterProject,
    provider: &str,
    name: &str,
) {
    if regions_for_in(env.regions, &project.region_bbox())
        .iter()
        .any(|adapter| adapter.info().provider.as_str() != provider)
    {
        (env.progress)(note!("progress-beyond-border", provider = name));
    }
}

/// The tiles of the `km` grid of `provider` over `bbox` that touch its regions; the others are
/// not published, and the templates stay empty there (reported once, with the provider's
/// `name`).
pub fn raster_tiles(
    env: &mut AdapterEnv<'_>,
    bbox: &BoundingBox,
    provider: &str,
    name: &str,
    km: u32,
) -> Vec<TileId> {
    let tiles = provider_tiles_in(env.regions, bbox, provider, km);
    let outside: Vec<TileId> = tiles_covering_grid(bbox, km)
        .into_iter()
        .filter(|t| !tiles.contains(t))
        .collect();
    if !outside.is_empty() {
        (env.progress)(note!(
            "progress-beyond-border-no-data",
            provider = name,
            tiles = tile_names(&outside)
        ));
    }
    tiles
}
