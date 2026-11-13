//! The extra layers of a project: sets of feature layers beside the cadastre, stored one file
//! per layer under `data/`. Three families exist: Berlin's registers and surveys, the full ALKIS
//! (NAS) of the states that serve it, and the tree cadastres of cities. They differ in what is
//! fetched; how a layer is stored, kept, removed and read back is the same and lives here.
//!
//! A layer never fails a fetch: it only gives candidates and a few map objects. One that could
//! not be fetched keeps its stored file while that still covers the area, and is left out
//! otherwise; each case is reported.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::records::AdapterId;
use serde_json::Value;

use super::{
    AlkisRecord, BERLIN_LAYERS_DIR, NAS_LAYERS_DIR, ProjectError, SKIPPED_SHOWN, TREE_LAYERS_DIR,
    alkis_nas, berlin, trees,
};
use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode, write_inside};
use crate::note::Note;

/// What the engine knows of one family of extra layers.
pub(crate) struct Family {
    /// The folder of the stored layers under the project folder.
    pub dir: &'static str,
    /// Whether `key` is a layer of the family's table. A path is only ever built from such a
    /// key, never from a name read from the project file (ADR 0059).
    pub known: fn(&str) -> bool,
    /// The name the notes give the key of a layer under.
    pub key_arg: &'static str,
    /// The file an answer streams into before it becomes a layer (ADR 0060); `None`: the
    /// answers of the family are held in memory.
    pub part: Option<&'static str>,
    /// The property of a feature that dates it; `None`: the features carry no date.
    pub dated_by: Option<&'static str>,
    pub notes: Notes,
}

/// The message ids of a family.
pub(crate) struct Notes {
    /// Offline, in reach of the family, and nothing is stored.
    pub none_stored: &'static str,
    /// The area left the reach of the family and stored layers go.
    pub removing: &'static str,
    /// The folder of the layers leads out of the project folder (families with a part file).
    pub folder_outside: Option<&'static str>,
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

fn known_berlin(key: &str) -> bool {
    berlin::EXTRA_LAYERS.iter().any(|layer| layer.file == key)
}

fn known_nas(key: &str) -> bool {
    alkis_nas::NAS_LAYERS.iter().any(|layer| layer.file == key)
}

fn known_trees(key: &str) -> bool {
    trees::all_tree_datasets().any(|dataset| dataset.id == key)
}

/// Berlin's registers, surveys and point layers.
pub(crate) const BERLIN: Family = Family {
    dir: BERLIN_LAYERS_DIR,
    known: known_berlin,
    key_arg: "file",
    part: None,
    dated_by: None,
    notes: Notes {
        none_stored: "progress-berlin-none-stored",
        removing: "progress-berlin-removing",
        folder_outside: None,
        download: "progress-berlin-download",
        features: "progress-berlin-features",
        skipped: None,
        kept: "warning-berlin-layer-kept",
        dropped: "warning-berlin-layer-skipped",
    },
};

/// The full ALKIS (NAS) of the states that serve it.
pub(crate) const NAS: Family = Family {
    dir: NAS_LAYERS_DIR,
    known: known_nas,
    key_arg: "file",
    part: Some("answer.gml"),
    dated_by: Some("beginnt"),
    notes: Notes {
        none_stored: "progress-nas-none-stored",
        removing: "progress-nas-removing",
        folder_outside: Some("warning-nas-folder-outside"),
        download: "progress-nas-download",
        features: "progress-nas-features",
        skipped: Some("progress-nas-skipped"),
        kept: "warning-nas-layer-kept",
        dropped: "warning-nas-layer-skipped",
    },
};

/// The tree cadastres of cities.
pub(crate) const TREES: Family = Family {
    dir: TREE_LAYERS_DIR,
    known: known_trees,
    key_arg: "id",
    part: Some("answer.geojson"),
    dated_by: Some("tree_date"),
    notes: Notes {
        none_stored: "progress-trees-none-stored",
        removing: "progress-trees-removing",
        folder_outside: Some("warning-trees-folder-outside"),
        download: "progress-trees-download",
        features: "progress-trees-count",
        skipped: Some("progress-trees-skipped"),
        kept: "warning-trees-kept",
        dropped: "warning-trees-skipped",
    },
};

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

/// The path of the stored layer `key` of `family`. `key` is one of the family's table.
pub(crate) fn path(dir: &Path, family: &Family, key: &str) -> PathBuf {
    dir.join(family.dir).join(format!("{key}.geojson"))
}

/// Removes the stored layer the project file records under `key`: only a layer of the table,
/// and only when its file lies inside the project folder (no link leads out of it).
pub(crate) fn remove(dir: &Path, family: &Family, key: &str) {
    if (family.known)(key) {
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
    let layers_dir = dir.join(family.dir);
    if let Some(answer) = family.part {
        // Answers stream into a part file beside the layers with a fresh name, its folder
        // checked right before each download (ADR 0060), removed after each one.
        if crate::fsutil::check_folder_inside(dir, &layers_dir).is_err() {
            if let Some(id) = family.notes.folder_outside {
                progress(Note::new(id).with("folder", family.dir));
            }
            return BTreeMap::new();
        }
        // Only after the check: a linked folder never has its part files removed.
        crate::fsutil::remove_stale_parts_inside(dir, &layers_dir, crate::fetch::STALE_PART_AGE);
        let part = || -> Result<PathBuf, FetchError> {
            let part = crate::fsutil::fresh_part_name(&layers_dir.join(answer));
            crate::fsutil::check_inside(dir, &part).map_err(|(path, e)| FetchError::Io(path, e))?;
            Ok(part)
        };
        progress(Note::new(family.notes.download));
        store(family, dir, bbox, previous, progress, fetch(&part))
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
        let file = path(dir, family, key);
        let stored = fetched
            .features
            .map_err(|e| crate::fetch::one_line(&e.to_string(), 300))
            .and_then(|fetched| {
                if !freemapper_fs::stays_inside(dir, &file) {
                    return Err(format!("{} leads out of the project folder", family.dir));
                }
                let geojson = berlin::layer_to_geojson(&fetched.features);
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
    let file = path(dir, family, key);
    let Some(record) = records
        .get(key)
        .filter(|record| file.exists() && record.covers(bbox))
    else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(&file).map_err(|e| ProjectError::Io(file.clone(), e))?;
    let features = berlin::parse_layer(&text).map_err(|e| ProjectError::StoredCandidates {
        path: file.clone(),
        message: e.to_string(),
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
        std::fs::write(file, berlin::layer_to_geojson(features)).unwrap();
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
                &BERLIN,
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
        write(dir.path(), &BERLIN, "relief", &[]);
        // A key that is no layer of the table names no file of freemapper: it stays.
        write(dir.path(), &BERLIN, "notes", &[]);
        let previous = BTreeMap::from([
            ("relief".to_string(), covering()),
            ("notes".to_string(), covering()),
        ]);
        let mut said = Vec::new();
        let records = fetch_into(
            &BERLIN,
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
        assert!(!path(dir.path(), &BERLIN, "relief").exists());
        assert!(path(dir.path(), &BERLIN, "notes").exists());
    }

    #[test]
    fn the_disk_plan_can_leave_a_family_out() {
        let dir = tempfile::tempdir().unwrap();
        let previous = BTreeMap::from([("relief".to_string(), covering())]);
        let mut said = Vec::new();
        let records = fetch_into(
            &BERLIN,
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
            write(dir.path(), &BERLIN, key, &[json!({"old": key})]);
        }
        let previous = BTreeMap::from([
            ("bauwerke-flaechen".to_string(), covering()),
            ("bauwerke-linien".to_string(), beside()),
            ("relief".to_string(), covering()),
        ]);
        let berlin_id = AdapterId::built_in("berlin");
        let mut said = Vec::new();
        let records = fetch_into(
            &BERLIN,
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
        let file = |key| path(dir.path(), &BERLIN, key);
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
        write(
            dir.path(),
            &TREES,
            "berlin-strassenbaeume",
            &[json!({"t": 1})],
        );
        let records = BTreeMap::from([
            ("berlin-strassenbaeume".to_string(), covering()),
            ("elsewhere".to_string(), beside()),
            ("gone".to_string(), covering()),
        ]);
        let read = |key| stored(&TREES, dir.path(), &records, &AREA, key);
        let layer = read("berlin-strassenbaeume").unwrap().unwrap();
        assert_eq!(layer.features, [json!({"t": 1})]);
        assert_eq!(layer.fetched, covering().fetched);
        // No record, a record of another area, a record without its file.
        write(dir.path(), &TREES, "elsewhere", &[]);
        assert!(read("unknown").unwrap().is_none());
        assert!(read("elsewhere").unwrap().is_none());
        assert!(read("gone").unwrap().is_none());
        // A file that is no layer is an error of the project, with its path.
        std::fs::write(path(dir.path(), &TREES, "gone"), "{").unwrap();
        assert!(matches!(
            read("gone"),
            Err(ProjectError::StoredCandidates { .. })
        ));
    }
}
