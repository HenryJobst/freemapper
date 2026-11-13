//! The extra layers of a project: sets of feature layers beside the cadastre, stored one file
//! per layer under `data/`. Three families exist: Berlin's registers and surveys, the full ALKIS
//! (NAS) of the states that serve it, and the tree cadastres of cities. They differ in what is
//! fetched; how a layer is stored, kept, removed and read back is the same and lives here.
//!
//! A layer never fails a fetch: it only gives candidates and a few map objects. One that could
//! not be fetched keeps its stored file while that still covers the area, and is left out
//! otherwise; each case is reported.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::extras::{ExtraSet, RasterTemplate};
use freemapper_adapter_api::geometry::touches_region_in;
use freemapper_adapter_api::policy::PolicyFetcher;
use freemapper_adapter_api::records::{AdapterId, WmsRecord, stored_bbox};
use serde_json::Value;

use super::{
    AlkisRecord, MapKind, NAS_LAYERS_DIR, ProjectError, SKIPPED_SHOWN, TREE_LAYERS_DIR, alkis_nas,
    trees,
};
use crate::adapters::{RegionAdapter, Registry};
use crate::crs::{BoundingBox, Crs};
use crate::fetch::{FetchError, FetchMode, Fetcher, write_inside};
use crate::note::Note;

/// What the engine knows of one family of extra layers.
pub(crate) struct Family<'a> {
    /// The folder of the stored layers under the project folder.
    pub dir: Cow<'a, str>,
    /// The table of the family's layers. A file is only ever read, written or removed under a
    /// key of it, never under a name read from the project file (ADR 0059): [`stored`],
    /// [`is_stored`], [`remove`] and the storing of a fetch ask it first ([`Family::has`]).
    pub table: Table<'a>,
    /// The name the notes give the key of a layer under.
    pub key_arg: &'static str,
    /// Where an answer streams into before it becomes a layer (ADR 0060); `None`: the answers
    /// of the family are held in memory.
    pub part: Option<Part>,
    /// The property of a feature that dates it; `None`: the features carry no date.
    pub dated_by: Option<&'static str>,
    pub notes: Notes,
}

/// The layers a family has.
pub(crate) enum Table<'a> {
    /// A table of the engine, for a family fed by data of the adapters.
    Fixed(fn(&str) -> bool),
    /// The layers of a set an adapter fetches itself.
    Set(&'a dyn ExtraSet),
}

impl Family<'_> {
    /// Whether `key` is a layer of the family.
    pub(crate) fn has(&self, key: &str) -> bool {
        match self.table {
            Table::Fixed(known) => known(key),
            // A set is not taken on trust: the key becomes a file name.
            Table::Set(set) => plain(key) && set.has(key),
        }
    }
}

/// The part file of a family whose answers are streamed.
pub(crate) struct Part {
    /// The name of the answer file, which gets a fresh name per download.
    pub answer: &'static str,
    /// The message when the folder of the layers leads out of the project folder: nothing is
    /// asked then, and the family has no layers.
    pub folder_outside: &'static str,
}

/// The message ids of a family.
pub(crate) struct Notes {
    /// Offline, in reach of the family, and nothing is stored.
    pub none_stored: &'static str,
    /// The area left the reach of the family and stored layers go.
    pub removing: &'static str,
    pub download: &'static str,
    /// A layer was stored: its count and key.
    pub features: &'static str,
    /// Features of a stored layer that could not be read: count, key and a list.
    pub skipped: Option<&'static str>,
    /// A layer failed and its stored file stays: key and detail.
    pub kept: &'static str,
    /// A layer failed and nothing of it is stored: key and detail.
    pub dropped: &'static str,
}

fn known_nas(key: &str) -> bool {
    alkis_nas::NAS_LAYERS.iter().any(|layer| layer.file == key)
}

fn known_trees(key: &str) -> bool {
    trees::all_tree_datasets().any(|dataset| dataset.id == key)
}

/// The full ALKIS (NAS) of the states that serve it.
pub(crate) const NAS: Family<'static> = Family {
    dir: Cow::Borrowed(NAS_LAYERS_DIR),
    table: Table::Fixed(known_nas),
    key_arg: "file",
    part: Some(Part {
        answer: "answer.gml",
        folder_outside: "warning-nas-folder-outside",
    }),
    dated_by: Some("beginnt"),
    notes: Notes {
        none_stored: "progress-nas-none-stored",
        removing: "progress-nas-removing",
        download: "progress-nas-download",
        features: "progress-nas-features",
        skipped: Some("progress-nas-skipped"),
        kept: "warning-nas-layer-kept",
        dropped: "warning-nas-layer-skipped",
    },
};

/// The tree cadastres of cities.
pub(crate) const TREES: Family<'static> = Family {
    dir: Cow::Borrowed(TREE_LAYERS_DIR),
    table: Table::Fixed(known_trees),
    key_arg: "id",
    part: Some(Part {
        answer: "answer.geojson",
        folder_outside: "warning-trees-folder-outside",
    }),
    dated_by: Some("tree_date"),
    notes: Notes {
        none_stored: "progress-trees-none-stored",
        removing: "progress-trees-removing",
        download: "progress-trees-download",
        features: "progress-trees-count",
        skipped: Some("progress-trees-skipped"),
        kept: "warning-trees-kept",
        dropped: "warning-trees-skipped",
    },
};

/// Whether `name` can be the name of a folder or a file of freemapper: lower-case letters,
/// digits, `-` and `_`, starting with a letter or a digit. The key of a set becomes a folder
/// and the key of a layer a file; a name with a dot or a separator would leave its place.
fn plain(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'))
}

/// The adapter of `registry` that has a set of extra layers with `key`, and the set. A set
/// whose key is no plain name is no set for the engine.
pub(crate) fn set_in<'a>(
    registry: &'a Registry,
    key: &str,
) -> Option<(&'a dyn RegionAdapter, &'a dyn ExtraSet)> {
    if !plain(key) {
        return None;
    }
    registry.adapters().find_map(|adapter| {
        let set = adapter.extras().iter().find(|set| set.info().key == key)?;
        Some((adapter, set.as_ref()))
    })
}

/// [`fetch_set_into`] for the set with `key` of the adapter of `registry` that has it. When no
/// adapter of the run has the set (the check of one adapter on its own), nothing of it is
/// known: nothing is asked or touched, and `previous` stays.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fetch_key_into(
    registry: &Registry,
    key: &str,
    dir: &Path,
    fetcher: &dyn Fetcher,
    region_bbox: &BoundingBox,
    kind: MapKind,
    mode: FetchMode,
    skip: bool,
    previous: &BTreeMap<String, AlkisRecord>,
    progress: &mut dyn FnMut(Note),
) -> BTreeMap<String, AlkisRecord> {
    match set_in(registry, key) {
        Some((adapter, set)) => fetch_set_into(
            adapter,
            set,
            dir,
            fetcher,
            region_bbox,
            kind,
            mode,
            skip,
            previous,
            progress,
        ),
        None => previous.clone(),
    }
}

/// The family of a set of extra layers an adapter fetches itself: its layers lie in the folder
/// `data/<key>`, held in memory while they are fetched, without a date of their own.
pub(crate) fn of_set(set: &dyn ExtraSet) -> Family<'_> {
    let info = set.info();
    Family {
        dir: Cow::Owned(format!("data/{}", info.key)),
        table: Table::Set(set),
        key_arg: "file",
        part: None,
        dated_by: None,
        notes: Notes {
            none_stored: info.notes.none_stored,
            removing: info.notes.removing,
            download: info.notes.download,
            features: info.notes.features,
            skipped: None,
            kept: info.notes.kept,
            dropped: info.notes.dropped,
        },
    }
}

/// Fetches the layers of `set`, a set of `adapter`, for a map of `kind` into its folder and
/// returns the records that replace `previous`, as [`fetch_into`] does for any family. The set
/// is in reach of an area that touches the region of its adapter, and it is asked through the
/// hosts the adapter is held to.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fetch_set_into(
    adapter: &dyn RegionAdapter,
    set: &dyn ExtraSet,
    dir: &Path,
    fetcher: &dyn Fetcher,
    region_bbox: &BoundingBox,
    kind: MapKind,
    mode: FetchMode,
    skip: bool,
    previous: &BTreeMap<String, AlkisRecord>,
    progress: &mut dyn FnMut(Note),
) -> BTreeMap<String, AlkisRecord> {
    fetch_into(
        &of_set(set),
        dir,
        touches_region_in(adapter, region_bbox),
        mode,
        skip,
        region_bbox,
        previous,
        progress,
        |_| {
            let fetch = |fetcher: &dyn Fetcher| set.fetch(fetcher, region_bbox, kind.contract());
            let layers = match adapter.policy() {
                Some(policy) => fetch(&PolicyFetcher::new(fetcher, policy)),
                None => fetch(fetcher),
            };
            layers
                .into_iter()
                .map(|fetched| Fetched {
                    key: fetched.key.to_string(),
                    features: fetched.features.map(|features| Features {
                        features,
                        skipped: Vec::new(),
                        regions: vec![adapter.info().id.clone()],
                    }),
                })
                .collect()
        },
    )
}

/// The adapter of `registry` that has a raster template with `key`, and the template. A
/// template whose key is no plain name is no template for the engine.
pub(crate) fn template_in<'a>(
    registry: &'a Registry,
    key: &str,
) -> Option<(&'a dyn RegionAdapter, &'a dyn RasterTemplate)> {
    if !plain(key) {
        return None;
    }
    registry.adapters().find_map(|adapter| {
        let template = adapter
            .templates()
            .iter()
            .find(|template| template.info().key == key)?;
        Some((adapter, template.as_ref()))
    })
}

/// The file of the raster template `key` of a project; its world file lies beside it.
pub(crate) fn template_path(dir: &Path, key: &str) -> PathBuf {
    dir.join("templates").join(format!("{key}.png"))
}

/// Fetches the raster template `key` of the adapter of `registry` that has it over `bbox`
/// (the map area with its buffer, in the CRS of the map) and keeps `record` in step with the
/// file. A template never fails a fetch: one that cannot be fetched is a warning and is
/// removed.
///
/// The template is fetched for an area that touches the region of its adapter
/// (`region_bbox`, EPSG:25833) and for a map in the CRS of the template (`map_crs`), through
/// the hosts the adapter is held to; otherwise, and when the disk plan left it out (`skip`),
/// its file and record go. A stored template that still covers the area stays as it is.
/// Offline nothing changes. When no adapter of the run has the template (the check of one
/// adapter on its own), nothing of it is known and nothing is touched.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fetch_template_key_into(
    registry: &Registry,
    key: &str,
    dir: &Path,
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    skip: bool,
    map_crs: Crs,
    region_bbox: &BoundingBox,
    bbox: &BoundingBox,
    record: &mut Option<WmsRecord>,
    progress: &mut dyn FnMut(Note),
) {
    let Some((adapter, template)) = template_in(registry, key) else {
        return;
    };
    if mode == FetchMode::Offline {
        return;
    }
    let info = template.info();
    let path = template_path(dir, key);
    let remove = |record: &mut Option<WmsRecord>| {
        for file in [path.clone(), path.with_extension("pgw")] {
            freemapper_fs::remove_inside(dir, &file);
        }
        *record = None;
    };
    if !touches_region_in(adapter, region_bbox) || map_crs != info.crs {
        remove(record);
        return;
    }
    if skip {
        // The disk plan already said that the template is left out.
        remove(record);
        return;
    }
    let unchanged = record.as_ref().is_some_and(|r| r.covers(bbox))
        && super::check_template(&path, bbox).is_ok();
    if unchanged {
        progress(Note::new(info.notes.unchanged));
        return;
    }
    progress(Note::new(info.notes.download));
    let fetch = |fetcher: &dyn Fetcher| template.fetch(fetcher, bbox, dir, &path);
    let fetched = match adapter.policy() {
        Some(policy) => fetch(&PolicyFetcher::new(fetcher, policy)),
        None => fetch(fetcher),
    };
    match fetched {
        Ok(()) => {
            *record = Some(WmsRecord {
                flight: Some(info.dataset_date.to_string()),
                flight_label: Some(info.dataset_label.to_string()),
                fetched: crate::clock::today(),
                bbox: stored_bbox(bbox),
            });
        }
        Err(e) => {
            progress(
                Note::new(info.notes.unavailable)
                    .with("detail", crate::fetch::one_line(&e.to_string(), 300)),
            );
            remove(record);
        }
    }
}

/// One layer as fetched; an empty list of features is a layer without objects in the area.
pub(crate) struct Fetched {
    /// The key of the layer in the family's table.
    pub key: String,
    pub features: Result<Features, FetchError>,
}

/// The features of a fetched layer.
pub(crate) struct Features {
    pub features: Vec<Value>,
    /// Features no geometry could be read for.
    pub skipped: Vec<String>,
    /// The adapters whose services were asked.
    pub regions: Vec<AdapterId>,
}

/// One stored layer as read back.
pub(crate) struct Stored {
    pub features: Vec<Value>,
    /// The day of the fetch, the currency of features without a date of their own.
    pub fetched: NaiveDate,
}

/// The path of the stored layer `key` of `family`. It does not ask whether `key` is a layer of
/// the table: a caller takes the key from the table, or uses [`stored`], [`is_stored`] and
/// [`remove`], which ask.
pub(crate) fn path(dir: &Path, family: &Family, key: &str) -> PathBuf {
    dir.join(family.dir.as_ref()).join(format!("{key}.geojson"))
}

/// Whether the layer `key` of `family` has a stored file.
pub(crate) fn is_stored(dir: &Path, family: &Family, key: &str) -> bool {
    family.has(key) && path(dir, family, key).exists()
}

/// Removes the stored layer the project file records under `key`: only a layer of the table,
/// and only when its file lies inside the project folder (no link leads out of it).
pub(crate) fn remove(dir: &Path, family: &Family, key: &str) {
    if family.has(key) {
        freemapper_fs::remove_inside(dir, &path(dir, family, key));
    }
}

/// Fetches the layers of `family` into its folder and returns the records that replace
/// `previous`. `near` says whether the area is in reach of the family; `bbox` is the area the
/// records are kept for. `fetch` is called when layers are to be downloaded, with what names a
/// fresh part file for an answer.
///
/// Offline nothing is downloaded and `previous` stays. Out of reach nothing is requested and
/// stored layers of an earlier area are removed. When the disk plan left the family out
/// (`skip`), nothing is downloaded and `previous` stays. A layer that fails keeps its stored
/// file while that covers `bbox`; layers that are no longer fetched are removed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fetch_into(
    family: &Family,
    dir: &Path,
    near: bool,
    mode: FetchMode,
    skip: bool,
    bbox: &BoundingBox,
    previous: &BTreeMap<String, AlkisRecord>,
    progress: &mut dyn FnMut(Note),
    fetch: impl FnOnce(&dyn Fn() -> Result<PathBuf, FetchError>) -> Vec<Fetched>,
) -> BTreeMap<String, AlkisRecord> {
    if mode == FetchMode::Offline {
        if near && previous.is_empty() {
            progress(Note::new(family.notes.none_stored));
        }
        return previous.clone();
    }
    if !near {
        if !previous.is_empty() {
            progress(Note::new(family.notes.removing));
            for key in previous.keys() {
                remove(dir, family, key);
            }
        }
        return BTreeMap::new();
    }
    if skip {
        // The disk plan already said that the layers are left out; stored ones stay.
        return previous.clone();
    }
    let layers_dir = dir.join(family.dir.as_ref());
    if let Some(part) = &family.part {
        // Answers stream into a part file beside the layers with a fresh name, its folder
        // checked right before each download (ADR 0060), removed after each one.
        if crate::fsutil::check_folder_inside(dir, &layers_dir).is_err() {
            progress(Note::new(part.folder_outside).with("folder", family.dir.as_ref()));
            return BTreeMap::new();
        }
        // Only after the check: a linked folder never has its part files removed.
        crate::fsutil::remove_stale_parts_inside(dir, &layers_dir, crate::fetch::STALE_PART_AGE);
        let fresh = || -> Result<PathBuf, FetchError> {
            let fresh = crate::fsutil::fresh_part_name(&layers_dir.join(part.answer));
            crate::fsutil::check_inside(dir, &fresh)
                .map_err(|(path, e)| FetchError::Io(path, e))?;
            Ok(fresh)
        };
        progress(Note::new(family.notes.download));
        store(family, dir, bbox, previous, progress, fetch(&fresh))
    } else {
        // A family that holds its answers in memory asks for no part file.
        let part = || -> Result<PathBuf, FetchError> {
            Err(FetchError::Io(
                layers_dir.clone(),
                std::io::Error::other("the layers of this folder are not streamed"),
            ))
        };
        progress(Note::new(family.notes.download));
        store(family, dir, bbox, previous, progress, fetch(&part))
    }
}

/// Stores the fetched `layers` and returns their records; a layer that failed keeps the record
/// of `previous` while its file covers `bbox`, and the files of layers without a record go.
fn store(
    family: &Family,
    dir: &Path,
    bbox: &BoundingBox,
    previous: &BTreeMap<String, AlkisRecord>,
    progress: &mut dyn FnMut(Note),
    layers: Vec<Fetched>,
) -> BTreeMap<String, AlkisRecord> {
    let rounded = [
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil(),
    ];
    let mut records = BTreeMap::new();
    for fetched in layers {
        let key = fetched.key.as_str();
        if !family.has(key) {
            // No file is written under a name that is no layer of the table.
            progress(
                Note::new(family.notes.dropped)
                    .with(family.key_arg, key)
                    .with(
                        "detail",
                        Note::new("error-extra-layer-unknown").with("folder", family.dir.as_ref()),
                    ),
            );
            continue;
        }
        let file = path(dir, family, key);
        let stored = fetched
            .features
            .map_err(|e| crate::fetch::one_line(&e.to_string(), 300))
            .and_then(|fetched| {
                if !freemapper_fs::stays_inside(dir, &file) {
                    return Err(format!("{} leads out of the project folder", family.dir));
                }
                let geojson = freemapper_sources::layer_file::layer_to_geojson(&fetched.features);
                write_inside(dir, &file, geojson.as_bytes()).map_err(|e| e.to_string())?;
                Ok(fetched)
            });
        match stored {
            Ok(fetched) => {
                let count = fetched.features.len();
                progress(
                    Note::new(family.notes.features)
                        .with("count", count)
                        .with(family.key_arg, key),
                );
                if let Some(id) = family.notes.skipped
                    && !fetched.skipped.is_empty()
                {
                    let shown = fetched.skipped.iter().take(SKIPPED_SHOWN);
                    let more = fetched.skipped.len() > SKIPPED_SHOWN;
                    progress(
                        Note::new(id)
                            .with("count", fetched.skipped.len())
                            .with(family.key_arg, key)
                            .with(
                                "list",
                                format!(
                                    "{}{}",
                                    shown
                                        .map(|s| crate::fetch::one_line(s, 120))
                                        .collect::<Vec<_>>()
                                        .join("; "),
                                    if more { "; …" } else { "" }
                                ),
                            ),
                    );
                }
                let latest_update = family.dated_by.and_then(|property| {
                    fetched
                        .features
                        .iter()
                        .filter_map(|f| f["properties"][property].as_str()?.parse().ok())
                        .max()
                });
                records.insert(
                    key.to_string(),
                    AlkisRecord {
                        bbox: rounded,
                        fetched: crate::clock::today(),
                        latest_update,
                        count,
                        regions: fetched.regions,
                    },
                );
            }
            Err(e) => match previous
                .get(key)
                .filter(|record| file.exists() && record.covers(bbox))
            {
                Some(record) => {
                    progress(
                        Note::new(family.notes.kept)
                            .with(family.key_arg, key)
                            .with("detail", e),
                    );
                    records.insert(key.to_string(), record.clone());
                }
                None => {
                    // Nothing was written for it; a stale file is removed with the unused layers.
                    progress(
                        Note::new(family.notes.dropped)
                            .with(family.key_arg, key)
                            .with("detail", e),
                    );
                }
            },
        }
    }
    // Layers that are no longer fetched, and failed ones that no longer cover the area.
    for key in previous.keys().filter(|key| !records.contains_key(*key)) {
        remove(dir, family, key);
    }
    records
}

/// The stored layer `key` of `family`, when `records` has it, its file is there and it covers
/// `bbox`; `None` otherwise (an optional source: `fetch` reported it, the export goes on).
pub(crate) fn stored(
    family: &Family,
    dir: &Path,
    records: &BTreeMap<String, AlkisRecord>,
    bbox: &BoundingBox,
    key: &str,
) -> Result<Option<Stored>, ProjectError> {
    if !family.has(key) {
        return Ok(None);
    }
    let file = path(dir, family, key);
    let Some(record) = records
        .get(key)
        .filter(|record| file.exists() && record.covers(bbox))
    else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(&file).map_err(|e| ProjectError::Io(file.clone(), e))?;
    let features = freemapper_sources::layer_file::parse_layer(&text).map_err(|e| {
        ProjectError::StoredCandidates {
            path: file.clone(),
            message: e.to_string(),
        }
    })?;
    Ok(Some(Stored {
        features,
        fetched: record.fetched,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The family of Berlin's registers and surveys, the one set there is.
    fn berlin() -> Family<'static> {
        static SET: freemapper_adapters_de::berlin::Registers =
            freemapper_adapters_de::berlin::Registers;
        of_set(&SET)
    }

    const AREA: BoundingBox = BoundingBox {
        min_e: 390_000.4,
        min_n: 5_820_000.6,
        max_e: 390_300.2,
        max_n: 5_820_200.9,
    };

    fn record(bbox: [f64; 4]) -> AlkisRecord {
        AlkisRecord {
            bbox,
            fetched: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            latest_update: None,
            count: 7,
            regions: Vec::new(),
        }
    }

    /// A record that covers [`AREA`], and one of an area beside it.
    fn covering() -> AlkisRecord {
        record([390_000.0, 5_820_000.0, 390_301.0, 5_820_201.0])
    }

    fn beside() -> AlkisRecord {
        record([380_000.0, 5_820_000.0, 380_301.0, 5_820_201.0])
    }

    fn write(dir: &Path, family: &Family, key: &str, features: &[Value]) {
        let file = path(dir, family, key);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            file,
            freemapper_sources::layer_file::layer_to_geojson(features),
        )
        .unwrap();
    }

    fn failed(key: &str) -> Fetched {
        Fetched {
            key: key.to_string(),
            features: Err(FetchError::Http {
                url: "https://example.org/wfs".to_string(),
                message: "status 503".to_string(),
            }),
        }
    }

    fn never(_: &dyn Fn() -> Result<PathBuf, FetchError>) -> Vec<Fetched> {
        panic!("nothing is fetched here")
    }

    #[test]
    fn offline_keeps_what_is_stored_and_says_when_nothing_is() {
        let dir = tempfile::tempdir().unwrap();
        let mut said = Vec::new();
        let mut fetch = |near, previous: &BTreeMap<String, AlkisRecord>| {
            fetch_into(
                &berlin(),
                dir.path(),
                near,
                FetchMode::Offline,
                false,
                &AREA,
                previous,
                &mut |note: Note| said.push(note.to_string()),
                never,
            )
        };
        assert!(fetch(true, &BTreeMap::new()).is_empty());
        let previous = BTreeMap::from([("relief".to_string(), covering())]);
        assert_eq!(fetch(true, &previous), previous);
        // Out of reach nothing is missing.
        assert!(fetch(false, &BTreeMap::new()).is_empty());
        assert_eq!(
            said,
            [
                "no Berlin ALKIS structures stored; run `freemapper fetch` online for stairs, \
                 walls, trees and canopies"
            ]
        );
    }

    #[test]
    fn the_layers_of_an_area_out_of_reach_are_removed() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), &berlin(), "relief", &[]);
        // A key that is no layer of the table names no file of freemapper: it stays.
        write(dir.path(), &berlin(), "notes", &[]);
        let previous = BTreeMap::from([
            ("relief".to_string(), covering()),
            ("notes".to_string(), covering()),
        ]);
        let mut said = Vec::new();
        let records = fetch_into(
            &berlin(),
            dir.path(),
            false,
            FetchMode::Online,
            false,
            &AREA,
            &previous,
            &mut |note: Note| said.push(note.to_string()),
            never,
        );
        assert!(records.is_empty());
        assert_eq!(
            said,
            ["removing the stored Berlin layers: the map area is no longer near Berlin"]
        );
        assert!(!path(dir.path(), &berlin(), "relief").exists());
        assert!(path(dir.path(), &berlin(), "notes").exists());
    }

    #[test]
    fn the_disk_plan_can_leave_a_family_out() {
        let dir = tempfile::tempdir().unwrap();
        let previous = BTreeMap::from([("relief".to_string(), covering())]);
        let mut said = Vec::new();
        let records = fetch_into(
            &berlin(),
            dir.path(),
            true,
            FetchMode::Online,
            true,
            &AREA,
            &previous,
            &mut |note: Note| said.push(note.to_string()),
            never,
        );
        assert_eq!(records, previous);
        assert!(said.is_empty(), "{said:?}");
    }

    #[test]
    fn a_layer_that_fails_stays_while_its_file_covers_the_area() {
        let dir = tempfile::tempdir().unwrap();
        for key in ["bauwerke-flaechen", "bauwerke-linien", "relief"] {
            write(dir.path(), &berlin(), key, &[json!({"old": key})]);
        }
        let previous = BTreeMap::from([
            ("bauwerke-flaechen".to_string(), covering()),
            ("bauwerke-linien".to_string(), beside()),
            ("relief".to_string(), covering()),
        ]);
        let berlin_id = AdapterId::built_in("berlin");
        let mut said = Vec::new();
        let records = fetch_into(
            &berlin(),
            dir.path(),
            true,
            FetchMode::Online,
            false,
            &AREA,
            &previous,
            &mut |note: Note| said.push(note.to_string()),
            |_| {
                vec![
                    failed("bauwerke-flaechen"),
                    failed("bauwerke-linien"),
                    Fetched {
                        key: "vegetation-linien".to_string(),
                        features: Ok(Features {
                            features: vec![json!({"a": 1}), json!({"a": 2})],
                            skipped: Vec::new(),
                            regions: vec![berlin_id.clone()],
                        }),
                    },
                ]
            },
        );
        let detail = "download of https://example.org/wfs failed: status 503";
        assert_eq!(
            said,
            [
                "downloading Berlin ALKIS structures, tree cadastre, street survey and point \
                 layers"
                    .to_string(),
                format!(
                    "warning: Berlin bauwerke-flaechen unavailable ({detail}); keeping the stored layer"
                ),
                format!("warning: Berlin bauwerke-linien unavailable ({detail}); layer skipped"),
                "2 Berlin vegetation-linien features".to_string(),
            ]
        );
        assert_eq!(
            records,
            BTreeMap::from([
                ("bauwerke-flaechen".to_string(), covering()),
                (
                    "vegetation-linien".to_string(),
                    AlkisRecord {
                        bbox: [390_000.0, 5_820_000.0, 390_301.0, 5_820_201.0],
                        fetched: crate::clock::today(),
                        latest_update: None,
                        count: 2,
                        regions: vec![berlin_id],
                    }
                ),
            ])
        );
        let file = |key| path(dir.path(), &berlin(), key);
        assert!(file("bauwerke-flaechen").exists());
        // The failed layer whose file is of another area, and the layer no longer fetched.
        assert!(!file("bauwerke-linien").exists());
        assert!(!file("relief").exists());
        assert_eq!(
            std::fs::read_to_string(file("vegetation-linien")).unwrap(),
            r#"{"features":[{"a":1},{"a":2}],"type":"FeatureCollection"}"#
        );
    }

    #[test]
    fn a_family_with_a_part_file_dates_its_layers_and_lists_what_it_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let mut said = Vec::new();
        let mut part_dir = None;
        let records = fetch_into(
            &NAS,
            dir.path(),
            true,
            FetchMode::Online,
            false,
            &AREA,
            &BTreeMap::new(),
            &mut |note: Note| said.push(note.to_string()),
            |part| {
                part_dir = part().unwrap().parent().map(Path::to_path_buf);
                vec![Fetched {
                    key: "sonstiges-bauwerk".to_string(),
                    features: Ok(Features {
                        features: vec![
                            json!({"properties": {"beginnt": "2024-05-01"}}),
                            json!({"properties": {"beginnt": "2025-01-02"}}),
                            json!({"properties": {}}),
                        ],
                        skipped: vec!["DEBB1: no geometry".to_string()],
                        regions: Vec::new(),
                    }),
                }]
            },
        );
        assert_eq!(part_dir, Some(dir.path().join("data/alkis-nas")));
        assert_eq!(
            said,
            [
                "downloading the full ALKIS structures (NAS)",
                "3 ALKIS NAS sonstiges-bauwerk features",
                "skipped 1 ALKIS NAS sonstiges-bauwerk features: DEBB1: no geometry",
            ]
        );
        assert_eq!(
            records["sonstiges-bauwerk"].latest_update,
            NaiveDate::from_ymd_opt(2025, 1, 2)
        );
    }

    #[test]
    fn a_stored_layer_is_read_when_its_record_covers_the_area() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), &berlin(), "relief", &[json!({"t": 1})]);
        write(dir.path(), &berlin(), "bauwerke-linien", &[]);
        let records = BTreeMap::from([
            ("relief".to_string(), covering()),
            ("bauwerke-linien".to_string(), beside()),
            ("bauwerke-flaechen".to_string(), covering()),
        ]);
        let read = |key| stored(&berlin(), dir.path(), &records, &AREA, key);
        let layer = read("relief").unwrap().unwrap();
        assert_eq!(layer.features, [json!({"t": 1})]);
        assert_eq!(layer.fetched, covering().fetched);
        // No record, a record of another area, a record without its file.
        assert!(read("vegetation-linien").unwrap().is_none());
        assert!(read("bauwerke-linien").unwrap().is_none());
        assert!(read("bauwerke-flaechen").unwrap().is_none());
        // A file that is no layer is an error of the project, with its path.
        std::fs::write(path(dir.path(), &berlin(), "bauwerke-flaechen"), "{").unwrap();
        assert!(matches!(
            read("bauwerke-flaechen"),
            Err(ProjectError::StoredCandidates { .. })
        ));
    }

    /// ADR 0059: a path is built from a key of the table only. A name the project file brings
    /// along is never read, stored under or removed, whatever lies at its place.
    #[test]
    fn a_key_that_is_no_layer_of_the_table_names_no_file() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), &berlin(), "notes", &[json!({"t": 1})]);
        let records = BTreeMap::from([("notes".to_string(), covering())]);
        assert!(
            stored(&berlin(), dir.path(), &records, &AREA, "notes")
                .unwrap()
                .is_none()
        );
        assert!(!is_stored(dir.path(), &berlin(), "notes"));
        write(dir.path(), &berlin(), "relief", &[]);
        assert!(is_stored(dir.path(), &berlin(), "relief"));
        assert!(!is_stored(dir.path(), &berlin(), "bauwerke-linien"));
        // A fetch that hands over such a key stores nothing under it.
        let mut said = Vec::new();
        let records = fetch_into(
            &berlin(),
            dir.path(),
            true,
            FetchMode::Online,
            false,
            &AREA,
            &BTreeMap::new(),
            &mut |note: Note| said.push(note.to_string()),
            |_| {
                vec![Fetched {
                    key: "../outside".to_string(),
                    features: Ok(Features {
                        features: vec![json!({"a": 1})],
                        skipped: Vec::new(),
                        regions: Vec::new(),
                    }),
                }]
            },
        );
        assert!(records.is_empty());
        assert!(!dir.path().join("data/outside.geojson").exists());
        assert_eq!(
            said[1..],
            ["warning: Berlin ../outside unavailable (not a layer of data/berlin); layer skipped"]
        );
    }

    #[test]
    fn a_long_list_of_skipped_features_is_cut() {
        let dir = tempfile::tempdir().unwrap();
        let mut said = Vec::new();
        fetch_into(
            &NAS,
            dir.path(),
            true,
            FetchMode::Online,
            false,
            &AREA,
            &BTreeMap::new(),
            &mut |note: Note| said.push(note.to_string()),
            |_| {
                vec![Fetched {
                    key: "sonstiges-bauwerk".to_string(),
                    features: Ok(Features {
                        features: Vec::new(),
                        skipped: (1..=7).map(|n| format!("f{n}")).collect(),
                        regions: Vec::new(),
                    }),
                }]
            },
        );
        assert_eq!(
            said[2],
            "skipped 7 ALKIS NAS sonstiges-bauwerk features: f1; f2; f3; f4; f5; …"
        );
    }

    /// A folder of layers that is a link out of the project folder: a family with a part file
    /// asks nothing and says so; one without writes nothing through the link.
    #[cfg(unix)]
    #[test]
    fn a_folder_that_leads_out_of_the_project_is_not_written_to() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("data")).unwrap();
        for family in [&TREES, &berlin()] {
            std::os::unix::fs::symlink(outside.path(), dir.path().join(family.dir.as_ref()))
                .unwrap();
        }
        let previous = BTreeMap::from([("potsdam".to_string(), covering())]);
        let mut said = Vec::new();
        let records = fetch_into(
            &TREES,
            dir.path(),
            true,
            FetchMode::Online,
            false,
            &AREA,
            &previous,
            &mut |note: Note| said.push(note.to_string()),
            never,
        );
        assert!(records.is_empty());
        assert_eq!(
            said,
            ["warning: city tree cadastres skipped (data/trees leads out of the project folder)"]
        );
        said.clear();
        let records = fetch_into(
            &berlin(),
            dir.path(),
            true,
            FetchMode::Online,
            false,
            &AREA,
            &BTreeMap::new(),
            &mut |note: Note| said.push(note.to_string()),
            |_| {
                vec![Fetched {
                    key: "relief".to_string(),
                    features: Ok(Features {
                        features: vec![json!({"a": 1})],
                        skipped: Vec::new(),
                        regions: Vec::new(),
                    }),
                }]
            },
        );
        assert!(records.is_empty());
        assert_eq!(
            said[1..],
            [
                "warning: Berlin relief unavailable (data/berlin leads out of the project \
                 folder); layer skipped"
            ]
        );
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }

    /// A set of extra layers of an adapter the engine has never heard of. It is asked through
    /// the hosts its adapter is held to, only for an area its region touches, and its layers
    /// are stored, kept and removed like those of any family.
    mod a_set_of_an_adapter {
        use std::sync::Mutex;

        use freemapper_adapter_api::extras::{
            ExtraEstimate, ExtraLayerInfo, ExtraNotes, ExtraSet, ExtraSetInfo, FetchedExtra,
        };

        use super::*;
        use crate::adapters::test_support::{SQUARE, square_with};
        use crate::crs::Crs;
        use crate::fetch::Fetcher;
        use crate::fetch::test_support::MapFetcher;
        use crate::project::MapKind;

        /// The wrecks of a made-up coast, one layer for sprint maps and one for both kinds.
        struct Wrecks {
            asked: std::sync::Arc<Mutex<Vec<String>>>,
        }

        static INFO: ExtraSetInfo = ExtraSetInfo {
            key: "wrecks",
            crs: Crs::Utm33,
            notes: ExtraNotes {
                none_stored: "progress-berlin-none-stored",
                removing: "progress-berlin-removing",
                download: "progress-berlin-download",
                features: "progress-berlin-features",
                kept: "warning-berlin-layer-kept",
                dropped: "warning-berlin-layer-skipped",
            },
            estimate: ExtraEstimate {
                what: "wrecks",
                megabytes_per_km2: 1.0,
            },
        };

        impl ExtraSet for Wrecks {
            fn info(&self) -> &ExtraSetInfo {
                &INFO
            }

            fn layers(&self, kind: freemapper_contract::MapKind) -> Vec<ExtraLayerInfo> {
                let mut layers = vec![ExtraLayerInfo { key: "hulls" }];
                if kind == freemapper_contract::MapKind::Sprint {
                    layers.push(ExtraLayerInfo { key: "anchors" });
                }
                layers
            }

            fn has(&self, key: &str) -> bool {
                matches!(key, "hulls" | "anchors")
            }

            fn fetch(
                &self,
                fetcher: &dyn Fetcher,
                _region_bbox: &BoundingBox,
                kind: freemapper_contract::MapKind,
            ) -> Vec<FetchedExtra> {
                self.layers(kind)
                    .into_iter()
                    .map(|layer| {
                        let url = format!("https://wrecks.example.org/{}", layer.key);
                        self.asked.lock().unwrap().push(url.clone());
                        FetchedExtra {
                            key: layer.key,
                            features: fetcher.get(&url).map(|bytes| {
                                freemapper_sources::layer_file::parse_layer(
                                    &String::from_utf8_lossy(&bytes),
                                )
                                .unwrap()
                            }),
                        }
                    })
                    .collect()
            }
        }

        /// An area inside the square region, and one far from it.
        const INSIDE: BoundingBox = BoundingBox {
            min_e: 4_000.0,
            min_n: 4_000.0,
            max_e: 4_300.0,
            max_n: 4_200.0,
        };
        const FAR: BoundingBox = BoundingBox {
            min_e: 50_000.0,
            min_n: 50_000.0,
            max_e: 50_300.0,
            max_n: 50_200.0,
        };

        #[test]
        fn is_fetched_stored_and_removed_through_the_registry() {
            let asked = std::sync::Arc::new(Mutex::new(Vec::new()));
            let registry = square_with(
                None,
                Some(&["wrecks.example.org"]),
                vec![Box::new(Wrecks {
                    asked: asked.clone(),
                })],
            );
            let (adapter, set) = set_in(&registry, "wrecks").expect("the square has the set");
            assert_eq!(adapter.info().id, SQUARE);
            assert!(set_in(&registry, "berlin").is_none());
            let dir = tempfile::tempdir().unwrap();
            let fetcher = MapFetcher::default();
            fetcher.insert(
                "https://wrecks.example.org/hulls",
                freemapper_sources::layer_file::layer_to_geojson(&[json!({"hull": 1})])
                    .into_bytes(),
            );
            fetcher.insert(
                "https://wrecks.example.org/anchors",
                freemapper_sources::layer_file::layer_to_geojson(&[]).into_bytes(),
            );
            let mut said = Vec::new();
            let mut fetch = |bbox: &BoundingBox, kind, previous: &BTreeMap<String, AlkisRecord>| {
                fetch_set_into(
                    adapter,
                    set,
                    dir.path(),
                    &fetcher,
                    bbox,
                    kind,
                    FetchMode::Online,
                    false,
                    previous,
                    &mut |note: Note| said.push(note.to_string()),
                )
            };
            let sprint = fetch(&INSIDE, MapKind::Sprint, &BTreeMap::new());
            assert_eq!(
                sprint.keys().collect::<Vec<_>>(),
                ["anchors", "hulls"],
                "both layers of a sprint map"
            );
            assert_eq!(sprint["hulls"].count, 1);
            assert_eq!(sprint["hulls"].regions, [SQUARE]);
            let family = of_set(set);
            assert!(path(dir.path(), &family, "hulls").exists());
            assert_eq!(
                path(dir.path(), &family, "hulls"),
                dir.path().join("data/wrecks/hulls.geojson")
            );
            // A schoolyard map uses one layer: the other goes.
            let schoolyard = fetch(&INSIDE, MapKind::Schoolyard, &sprint);
            assert_eq!(schoolyard.keys().collect::<Vec<_>>(), ["hulls"]);
            assert!(!path(dir.path(), &family, "anchors").exists());
            // Far from the region nothing is asked and what is stored goes.
            let before = asked.lock().unwrap().len();
            let far = fetch(&FAR, MapKind::Sprint, &schoolyard);
            assert!(far.is_empty());
            assert_eq!(asked.lock().unwrap().len(), before);
            assert!(!path(dir.path(), &family, "hulls").exists());
            assert_eq!(
                said.last().map(String::as_str),
                Some("removing the stored Berlin layers: the map area is no longer near Berlin")
            );
        }

        /// The set asks through the hosts of its adapter: a layer on another host is refused
        /// and reported as any layer that fails.
        #[test]
        fn asks_only_the_hosts_of_its_adapter() {
            let asked = std::sync::Arc::new(Mutex::new(Vec::new()));
            let registry = square_with(
                None,
                Some(&["other.example.org"]),
                vec![Box::new(Wrecks { asked })],
            );
            let (adapter, set) = set_in(&registry, "wrecks").unwrap();
            let dir = tempfile::tempdir().unwrap();
            let fetcher = MapFetcher::default();
            fetcher.insert("https://wrecks.example.org/hulls", b"{}".to_vec());
            let mut said = Vec::new();
            let records = fetch_set_into(
                adapter,
                set,
                dir.path(),
                &fetcher,
                &INSIDE,
                MapKind::Schoolyard,
                FetchMode::Online,
                false,
                &BTreeMap::new(),
                &mut |note: Note| said.push(note.to_string()),
            );
            assert!(records.is_empty());
            assert_eq!(fetcher.calls.get(), 0, "the request never left");
            assert!(
                said[1].starts_with("warning: Berlin hulls unavailable (")
                    && said[1].contains("does not send requests to this address"),
                "{}",
                said[1]
            );
        }

        /// A run whose registry has no adapter with the set (the check of one adapter on its
        /// own): nothing is asked, nothing is said, and what is recorded stays.
        #[test]
        fn is_left_alone_by_a_run_without_its_adapter() {
            let registry = square_with(None, None, Vec::new());
            let dir = tempfile::tempdir().unwrap();
            let fetcher = MapFetcher::default();
            let previous = BTreeMap::from([("hulls".to_string(), covering())]);
            let mut said = Vec::new();
            let records = fetch_key_into(
                &registry,
                "wrecks",
                dir.path(),
                &fetcher,
                &INSIDE,
                MapKind::Sprint,
                FetchMode::Online,
                false,
                &previous,
                &mut |note: Note| said.push(note.to_string()),
            );
            assert_eq!(records, previous);
            assert_eq!(fetcher.calls.get(), 0);
            assert!(said.is_empty(), "{said:?}");
        }

        /// The engine builds a folder from the key of a set and a file from the key of a layer.
        /// Neither is taken on trust: a key that is no plain name is no set and no layer.
        #[test]
        fn is_named_by_plain_names_only() {
            struct Rogue(ExtraSetInfo);
            impl ExtraSet for Rogue {
                fn info(&self) -> &ExtraSetInfo {
                    &self.0
                }
                fn layers(&self, _: freemapper_contract::MapKind) -> Vec<ExtraLayerInfo> {
                    Vec::new()
                }
                fn has(&self, _: &str) -> bool {
                    true
                }
                fn fetch(
                    &self,
                    _: &dyn Fetcher,
                    _: &BoundingBox,
                    _: freemapper_contract::MapKind,
                ) -> Vec<FetchedExtra> {
                    Vec::new()
                }
            }
            let named = |key: &'static str| -> Box<dyn ExtraSet> {
                Box::new(Rogue(ExtraSetInfo {
                    key,
                    ..INFO.clone()
                }))
            };
            let registry = square_with(None, None, vec![named("../up"), named("wrecks")]);
            assert!(set_in(&registry, "../up").is_none());
            let (_, set) = set_in(&registry, "wrecks").unwrap();
            let family = of_set(set);
            assert!(family.has("hulls") && family.has("old_hulls-2"));
            for key in ["../x", "a/b", "a\\b", ".hidden", "", "x.geojson", "Ä"] {
                assert!(!family.has(key), "{key:?}");
            }
        }

        /// The folder of Berlin's set is the one the engine reads the building heights from.
        #[test]
        fn of_berlin_lies_in_the_folder_the_engine_names() {
            assert_eq!(berlin().dir, crate::project::BERLIN_LAYERS_DIR);
        }
    }

    /// A raster template of an adapter the engine has never heard of: fetched for an area its
    /// region touches and a map in its CRS, through the hosts of its adapter, recorded with
    /// the date of its dataset, and removed when the area or the plan no longer wants it.
    mod a_template_of_an_adapter {
        use freemapper_adapter_api::extras::{
            ExtraEstimate, RasterTemplate, RasterTemplateInfo, TemplateNotes,
        };
        use freemapper_adapter_api::records::WmsRecord;

        use super::*;
        use crate::adapters::test_support::{SQUARE, square, square_with_templates};
        use crate::crs::Crs;
        use crate::fetch::Fetcher;
        use crate::fetch::test_support::MapFetcher;

        /// The sea chart of a made-up coast.
        struct Chart;

        static INFO: RasterTemplateInfo = RasterTemplateInfo {
            key: "chart",
            crs: Crs::Utm33,
            dataset_date: "2001-02-03",
            dataset_label: "chart edition",
            notes: TemplateNotes {
                unchanged: "progress-k-five-unchanged",
                download: "progress-k-five-download",
                unavailable: "warning-k-five-unavailable",
            },
            estimate: ExtraEstimate {
                what: "sea chart",
                megabytes_per_km2: 1.0,
            },
        };

        impl RasterTemplate for Chart {
            fn info(&self) -> &RasterTemplateInfo {
                &INFO
            }

            fn fetch(
                &self,
                fetcher: &dyn Fetcher,
                _bbox: &BoundingBox,
                root: &Path,
                out: &Path,
            ) -> Result<(), FetchError> {
                let image = fetcher.get("https://charts.example.org/chart.png")?;
                write_inside(root, out, &image)?;
                write_inside(root, &out.with_extension("pgw"), b"world")
            }
        }

        const INSIDE: BoundingBox = BoundingBox {
            min_e: 4_000.0,
            min_n: 4_000.0,
            max_e: 4_300.0,
            max_n: 4_200.0,
        };
        const FAR: BoundingBox = BoundingBox {
            min_e: 50_000.0,
            min_n: 50_000.0,
            max_e: 50_300.0,
            max_n: 50_200.0,
        };

        struct Run {
            dir: tempfile::TempDir,
            fetcher: MapFetcher,
            record: Option<WmsRecord>,
            said: Vec<String>,
        }

        impl Run {
            fn new() -> Self {
                let fetcher = MapFetcher::default();
                fetcher.insert("https://charts.example.org/chart.png", b"image".to_vec());
                Self {
                    dir: tempfile::tempdir().unwrap(),
                    fetcher,
                    record: None,
                    said: Vec::new(),
                }
            }

            fn fetch(&mut self, registry: &Registry, bbox: &BoundingBox, crs: Crs, skip: bool) {
                fetch_template_key_into(
                    registry,
                    "chart",
                    self.dir.path(),
                    &self.fetcher,
                    FetchMode::Online,
                    skip,
                    crs,
                    bbox,
                    bbox,
                    &mut self.record,
                    &mut |note: Note| self.said.push(note.to_string()),
                );
            }

            fn files(&self) -> (bool, bool) {
                let image = template_path(self.dir.path(), "chart");
                (image.exists(), image.with_extension("pgw").exists())
            }
        }

        #[test]
        fn is_fetched_recorded_and_removed_through_the_registry() {
            let registry =
                square_with_templates(Some(&["charts.example.org"]), vec![Box::new(Chart)]);
            let (adapter, _) = template_in(&registry, "chart").expect("the square has it");
            assert_eq!(adapter.info().id, SQUARE);
            assert!(template_in(&registry, "k5").is_none());
            assert!(template_in(&registry, "../chart").is_none());
            let mut run = Run::new();
            assert_eq!(
                template_path(run.dir.path(), "chart"),
                run.dir.path().join("templates/chart.png")
            );
            run.fetch(&registry, &INSIDE, Crs::Utm33, false);
            assert_eq!(run.files(), (true, true));
            assert_eq!(
                run.record,
                Some(WmsRecord {
                    flight: Some("2001-02-03".to_string()),
                    flight_label: Some("chart edition".to_string()),
                    fetched: crate::clock::today(),
                    bbox: [4_000.0, 4_000.0, 4_300.0, 4_200.0],
                })
            );
            assert_eq!(
                run.said,
                ["downloading the K5 map (1:5000) from the Geoportal Berlin WMS"]
            );
            // Far from the region the template goes, and nothing is asked.
            let calls = run.fetcher.calls.get();
            run.fetch(&registry, &FAR, Crs::Utm33, false);
            assert_eq!((run.files(), &run.record), ((false, false), &None));
            // A map in another CRS gets none, and neither does one the disk plan left it out of.
            for (crs, skip) in [(Crs::Utm32, false), (Crs::Utm33, true)] {
                run.fetch(&registry, &INSIDE, Crs::Utm33, false);
                assert_eq!(run.files(), (true, true));
                let calls = run.fetcher.calls.get();
                run.fetch(&registry, &INSIDE, crs, skip);
                assert_eq!((run.files(), &run.record), ((false, false), &None));
                assert_eq!(run.fetcher.calls.get(), calls);
            }
            assert_eq!(run.fetcher.calls.get(), calls + 2);
        }

        /// A template that cannot be fetched is a warning; a refused host is such a failure.
        #[test]
        fn that_fails_is_a_warning_and_is_held_to_the_hosts_of_its_adapter() {
            let registry =
                square_with_templates(Some(&["other.example.org"]), vec![Box::new(Chart)]);
            let mut run = Run::new();
            run.fetch(&registry, &INSIDE, Crs::Utm33, false);
            assert_eq!(run.fetcher.calls.get(), 0, "the request never left");
            assert_eq!((run.files(), &run.record), ((false, false), &None));
            assert!(
                run.said[1].starts_with("warning: the K5 map template is unavailable (")
                    && run.said[1].ends_with("); skipped"),
                "{}",
                run.said[1]
            );
        }

        /// The template recorded in `k5`: an adapter of the program has it under the key the
        /// engine looks for, its file is the one the engine lists and removes, and its credit
        /// goes by the same key.
        #[test]
        fn of_berlin_is_the_one_the_engine_records() {
            use crate::project::{K5_RELPATH, K5_TEMPLATE};
            let (adapter, template) =
                template_in(crate::adapters::registry(), K5_TEMPLATE).expect("built in");
            assert_eq!(adapter.info().id.as_str(), "berlin");
            assert_eq!(template.info().key, freemapper_adapters_de::berlin::K5_KEY);
            let dir = Path::new("project");
            assert_eq!(template_path(dir, K5_TEMPLATE), dir.join(K5_RELPATH));
        }

        /// A run whose registry has no adapter with the template leaves it alone.
        #[test]
        fn is_left_alone_by_a_run_without_its_adapter() {
            let with = square_with_templates(None, vec![Box::new(Chart)]);
            let without = square(None, None);
            let mut run = Run::new();
            run.fetch(&with, &INSIDE, Crs::Utm33, false);
            let (record, calls) = (run.record.clone(), run.fetcher.calls.get());
            run.said.clear();
            run.fetch(&without, &FAR, Crs::Utm32, true);
            assert_eq!(run.files(), (true, true));
            assert_eq!(run.record, record);
            assert_eq!(run.fetcher.calls.get(), calls);
            assert!(run.said.is_empty(), "{:?}", run.said);
        }
    }
}
