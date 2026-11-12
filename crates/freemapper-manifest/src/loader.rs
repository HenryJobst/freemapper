//! Adapters from folders: the user's own (`<config>/freemapper/adapters/<id>/`) and, when the
//! person asks for them, those of a project (`<project>/adapters/<id>/`).
//!
//! A folder is untrusted input. Nothing in it replaces an adapter the program has, a link is
//! not followed, and a folder with a defect is left out with its findings while the others
//! load. The adapters that load ask their declared hosts and no other.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::NaiveDate;
use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::registry::Registry;
use geo_types::MultiPolygon;

use crate::adapter::{AdapterFiles, ManifestAdapter, Parse};
use crate::boundary::MAX_BOUNDARY_BYTES;
use crate::error::ManifestError;
use crate::model::{Manifest, Orthophoto, Version};
use crate::{MANIFEST_FILE, MAX_MANIFEST_BYTES};

/// The most files the folder of an adapter may hold.
pub const MAX_FILES: usize = 64;

/// Where a folder of adapters comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The folder of the user, loaded always: the person put the files there.
    User,
    /// The folder of a project, loaded only when the person asks: a project that was passed on
    /// must not bring a source along.
    Project,
}

/// A folder that holds one folder per adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterDir {
    pub path: PathBuf,
    pub origin: Origin,
}

/// An adapter that was loaded from a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    pub id: String,
    pub origin: Origin,
    pub folder: PathBuf,
    pub revision: u32,
    /// The hosts the adapter may ask.
    pub hosts: Vec<String>,
    pub last_verified: Option<NaiveDate>,
    /// The adapters loaded before it on whose ground it lies: a map there gets the objects of
    /// both.
    pub overlaps: Vec<String>,
}

/// What is wrong with the folder of an adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterWarning {
    pub folder: PathBuf,
    /// Whether the adapter was left out; `false`: it loaded, and the findings are for the
    /// person to know.
    pub skipped: bool,
    pub findings: Vec<ManifestError>,
}

/// The files of an adapter folder, read as text: plain names, no links, at most 2 MiB each.
pub struct FolderFiles {
    dir: PathBuf,
}

impl FolderFiles {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }
}

impl AdapterFiles for FolderFiles {
    fn read(&self, name: &str) -> Result<Cow<'static, str>, String> {
        if !freemapper_fs::is_plain_name(name) {
            return Err("not a plain file name".to_string());
        }
        let path = self.dir.join(name);
        let file = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !file.is_file() {
            return Err("not a file of its own (a link is not followed)".to_string());
        }
        if file.len() > MAX_BOUNDARY_BYTES as u64 {
            return Err(format!(
                "the file has {} bytes; a file of an adapter has at most {MAX_BOUNDARY_BYTES}",
                file.len()
            ));
        }
        std::fs::read_to_string(&path)
            .map(Cow::Owned)
            .map_err(|e| e.to_string())
    }
}

fn finding(code: &'static str, path: &Path, message: impl Into<String>) -> ManifestError {
    ManifestError {
        code,
        path: path.to_path_buf(),
        line: 0,
        key: String::new(),
        message: message.into(),
    }
}

/// Loads the adapters of `dirs` into `registry`, each folder of adapters in the order of the
/// names of its folders, after the adapters the registry holds. `running` is the version of
/// this freemapper. A folder of adapters that does not exist is no finding.
///
/// Findings beside those of the manifest and its boundary files: `adapter.folder` (a link, a
/// name that is no id, too many files), `adapter.needs-newer`, `adapter.duplicate` (the id is
/// taken; a folder cannot replace an adapter), `adapter.provider-shared`, and, for an adapter
/// that loads, `adapter.rules-not-read`.
pub fn load_dirs(
    registry: &mut Registry,
    dirs: &[AdapterDir],
    running: Version,
) -> (Vec<Loaded>, Vec<AdapterWarning>) {
    let (mut loaded, mut warnings) = (Vec::new(), Vec::new());
    for dir in dirs {
        let Ok(kind) = std::fs::symlink_metadata(&dir.path) else {
            continue;
        };
        if !kind.is_dir() {
            warnings.push(AdapterWarning {
                folder: dir.path.clone(),
                skipped: true,
                findings: vec![finding(
                    "adapter.folder",
                    &dir.path,
                    "not a folder of its own (a link is not followed)",
                )],
            });
            continue;
        }
        let mut entries: Vec<PathBuf> = match std::fs::read_dir(&dir.path) {
            Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
            Err(e) => {
                warnings.push(AdapterWarning {
                    folder: dir.path.clone(),
                    skipped: true,
                    findings: vec![finding("adapter.folder", &dir.path, e.to_string())],
                });
                continue;
            }
        };
        entries.sort();
        for folder in entries {
            // A plain file beside the folders (a note, a `.DS_Store`) is nobody's adapter.
            let is_file = std::fs::symlink_metadata(&folder).is_ok_and(|kind| kind.is_file());
            if is_file {
                continue;
            }
            match load_one(registry, &folder, dir.origin, running) {
                Ok((adapter, notes)) => {
                    if !notes.is_empty() {
                        warnings.push(AdapterWarning {
                            folder: folder.clone(),
                            skipped: false,
                            findings: notes,
                        });
                    }
                    loaded.push(adapter);
                }
                Err(findings) => warnings.push(AdapterWarning {
                    folder,
                    skipped: true,
                    findings,
                }),
            }
        }
    }
    (loaded, warnings)
}

/// Loads the adapter of `folder` into `registry`; the findings of an adapter that loads, or
/// those that keep it out.
fn load_one(
    registry: &mut Registry,
    folder: &Path,
    origin: Origin,
    running: Version,
) -> Result<(Loaded, Vec<ManifestError>), Vec<ManifestError>> {
    let one = |code, message: String| vec![finding(code, folder, message)];
    let kind =
        std::fs::symlink_metadata(folder).map_err(|e| one("adapter.folder", e.to_string()))?;
    if !kind.is_dir() {
        return Err(one(
            "adapter.folder",
            "not a folder of its own (a link is not followed)".to_string(),
        ));
    }
    let files: Vec<PathBuf> = std::fs::read_dir(folder)
        .map_err(|e| one("adapter.folder", e.to_string()))?
        .flatten()
        .map(|entry| entry.path())
        .collect();
    if files.len() > MAX_FILES {
        return Err(one(
            "adapter.folder",
            format!(
                "the folder holds {} files; the folder of an adapter holds at most {MAX_FILES}",
                files.len()
            ),
        ));
    }
    let path = folder.join(MANIFEST_FILE);
    let missing = |why: String| vec![finding("manifest.file-missing", &path, why)];
    let file = std::fs::symlink_metadata(&path).map_err(|e| missing(e.to_string()))?;
    if !file.is_file() {
        return Err(missing(
            "not a file of its own (a link is not followed)".to_string(),
        ));
    }
    if file.len() > MAX_MANIFEST_BYTES as u64 {
        return Err(vec![finding(
            "manifest.too-large",
            &path,
            format!(
                "the manifest has {} bytes; a manifest has at most {MAX_MANIFEST_BYTES}",
                file.len()
            ),
        )]);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| missing(e.to_string()))?;
    let manifest = crate::parse(&text, folder)?;
    check_fit(registry, &manifest, &path, running)?;
    let mut entry = Loaded {
        id: manifest.adapter.id.clone(),
        origin,
        folder: folder.to_path_buf(),
        revision: manifest.adapter.revision,
        hosts: manifest.hosts.clone(),
        last_verified: manifest.adapter.last_verified,
        overlaps: Vec::new(),
    };
    let adapter = ManifestAdapter::new(manifest, &FolderFiles::new(folder), Parse::Now, registry)
        .map_err(|findings| {
            // The files of a folder are named by their place on the disk.
            findings
                .into_iter()
                .map(|mut found| {
                    if let Some(name) = found.path.file_name() {
                        found.path = folder.join(name);
                    }
                    found
                })
                .collect::<Vec<_>>()
        })?
        .asking_declared_hosts_only();
    let adapter: Arc<dyn RegionAdapter> = Arc::new(adapter);
    entry.overlaps = sharing_ground(registry, adapter.as_ref());
    registry
        .add(adapter)
        .map_err(|taken| one("adapter.duplicate", taken.to_string()))?;
    let mut notes = Vec::new();
    if files
        .iter()
        .any(|file| file.file_name().is_some_and(|name| name == "rules.toml"))
    {
        notes.push(finding(
            "adapter.rules-not-read",
            &folder.join("rules.toml"),
            "this freemapper does not read the rules of an adapter; the adapter loads without them",
        ));
    }
    Ok((entry, notes))
}

/// The adapters of `registry` on whose ground `adapter` lies, or which lie on its ground: a
/// point inside the one is inside the other. Two neighbours whose simplified borders cross are
/// not among them.
fn sharing_ground(registry: &Registry, adapter: &dyn RegionAdapter) -> Vec<String> {
    use geo::{Contains, InteriorPoint};
    let inside = |shape: &MultiPolygon<f64>, of: &MultiPolygon<f64>| {
        shape
            .interior_point()
            .is_some_and(|point| of.contains(&point))
    };
    let bounds = adapter.bounds();
    registry
        .adapters()
        .filter(|other| {
            // The box first: the boundary of an adapter far away is not even parsed.
            let theirs = other.bounds();
            let apart = theirs.max_e < bounds.min_e
                || bounds.max_e < theirs.min_e
                || theirs.max_n < bounds.min_n
                || bounds.max_n < theirs.min_n;
            !apart
                && (inside(adapter.boundary(), other.boundary())
                    || inside(other.boundary(), adapter.boundary()))
        })
        .map(|other| other.info().id.to_string())
        .collect()
}

/// Whether the adapter of `manifest` fits beside those of `registry` and this freemapper.
fn check_fit(
    registry: &Registry,
    manifest: &Manifest,
    path: &Path,
    running: Version,
) -> Result<(), Vec<ManifestError>> {
    let id = &manifest.adapter.id;
    if manifest.adapter.min_freemapper > running {
        return Err(vec![finding(
            "adapter.needs-newer",
            path,
            format!(
                "adapter {id} needs freemapper {} or newer (this is {running})",
                manifest.adapter.min_freemapper
            ),
        )]);
    }
    if registry.get(id).is_some() {
        return Err(vec![finding(
            "adapter.duplicate",
            path,
            format!(
                "an adapter with the id {id} exists already; an adapter from a folder cannot replace it"
            ),
        )]);
    }
    // Two adapters with one provider share its cache folder and its record in the project
    // file. That is right for a backend they both name, and wrong for services of its own.
    let declares_services = matches!(
        manifest.orthophoto,
        Some(Orthophoto::Wms { .. } | Orthophoto::Wcs { .. })
    );
    let provider = &manifest.adapter.provider;
    if declares_services
        && registry
            .adapters()
            .any(|other| other.info().provider.as_str() == provider)
    {
        return Err(vec![finding(
            "adapter.provider-shared",
            path,
            format!(
                "the provider {provider} belongs to another adapter; an adapter that declares its \
                 rasters as services is its own provider"
            ),
        )]);
    }
    Ok(())
}
