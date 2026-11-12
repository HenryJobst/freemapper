//! What `freemapper adapters check` finds in the folder of an adapter without asking a server:
//! whether the folder loads as it would from the adapter folder of a person, and what its
//! author should know before passing it on.

use std::path::Path;

use chrono::{Months, NaiveDate};
use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::geometry::rectangle;
use freemapper_adapter_api::registry::Registry;
use freemapper_georef::crs::BoundingBox;
use geo::{Area, Contains};

use crate::MANIFEST_FILE;
use crate::error::ManifestError;
use crate::loader::{Loaded, Origin, load_folder, sharing_ground};
use crate::model::Version;

/// A region smaller than this, in km², is more likely a boundary in the wrong unit than a
/// region.
const MIN_AREA_KM2: f64 = 1.0;
/// A region larger than this, in km², is larger than any country an adapter is written for.
const MAX_AREA_KM2: f64 = 2_000_000.0;

/// The folder of an adapter that loads.
pub struct Checked {
    pub loaded: Loaded,
    /// The raster backends it was given and the adapter itself, alone.
    pub registry: Registry,
    /// What does not keep the adapter from loading and its author should still know.
    pub warnings: Vec<ManifestError>,
}

impl Checked {
    /// The adapter that was checked.
    pub fn adapter(&self) -> &dyn RegionAdapter {
        self.registry
            .get(&self.loaded.id)
            .expect("the adapter that loaded is in the registry")
    }
}

/// Checks the folder `folder` of one adapter: the rules of the loader (a folder of its own
/// named like the id, a valid manifest, valid boundary files, known backends, a freemapper new
/// enough), then the warnings. `backends` holds the raster backends a manifest may name and no
/// adapter; `built_in` holds the adapters of the program, which the adapter is compared with.
///
/// Warnings: `check.never-verified`, `check.verified-long-ago` (more than twelve months before
/// `today`), `check.built-in-id`, `check.prefix-shared`, `check.boundary-area`,
/// `check.shares-ground` and the loader's `adapter.rules-not-read`.
pub fn check_folder(
    folder: &Path,
    backends: Registry,
    built_in: &Registry,
    running: Version,
    today: NaiveDate,
) -> Result<Checked, Vec<ManifestError>> {
    let mut registry = backends;
    let (loaded, mut warnings) = load_folder(&mut registry, folder, Origin::User, running)?;
    let manifest = folder.join(MANIFEST_FILE);
    let mut warn = |code: &'static str, key: &str, message: String| {
        warnings.push(ManifestError {
            code,
            path: manifest.clone(),
            line: 0,
            key: key.to_string(),
            message,
        });
    };
    match loaded.last_verified {
        None => warn(
            "check.never-verified",
            "adapter.last_verified",
            "the manifest does not say when its services were last checked against it".to_string(),
        ),
        Some(day) if today.checked_sub_months(Months::new(12)) > Some(day) => warn(
            "check.verified-long-ago",
            "adapter.last_verified",
            format!("the services were last checked on {day}, more than twelve months ago"),
        ),
        Some(_) => {}
    }
    let adapter = registry
        .get(&loaded.id)
        .expect("the adapter that loaded is in the registry");
    if built_in.get(&loaded.id).is_some() {
        warn(
            "check.built-in-id",
            "adapter.id",
            format!(
                "an adapter with the id {} is built into this freemapper; from an adapter \
                 folder this one does not load",
                loaded.id
            ),
        );
    }
    let prefix = adapter.info().object_id_prefix.as_deref();
    for other in built_in.adapters() {
        let info = other.info();
        if prefix.is_some()
            && info.object_id_prefix.as_deref() == prefix
            && info.id != adapter.info().id
        {
            warn(
                "check.prefix-shared",
                "adapter.object_id_prefix",
                format!(
                    "the built-in adapter {} has the same object id prefix: an object is \
                     counted to one region by it",
                    info.id
                ),
            );
        }
    }
    let km2 = adapter.boundary().unsigned_area() / 1e6;
    if !(MIN_AREA_KM2..=MAX_AREA_KM2).contains(&km2) {
        warn(
            "check.boundary-area",
            "boundary.file",
            format!("the boundary encloses {km2:.3} km², which is no size for a region"),
        );
    }
    let neighbours: Vec<String> = sharing_ground(built_in, adapter)
        .into_iter()
        .filter(|other| *other != loaded.id)
        .collect();
    if !neighbours.is_empty() {
        warn(
            "check.shares-ground",
            "boundary.file",
            format!(
                "the adapter lies on the ground of {}: a map there gets the objects of both",
                neighbours.join(", ")
            ),
        );
    }
    Ok(Checked {
        loaded,
        registry,
        warnings,
    })
}

/// Whether `bbox` (EPSG:25833) lies inside the boundary of `adapter`.
pub fn covers(adapter: &dyn RegionAdapter, bbox: &BoundingBox) -> bool {
    adapter.boundary().contains(&rectangle(bbox))
}
