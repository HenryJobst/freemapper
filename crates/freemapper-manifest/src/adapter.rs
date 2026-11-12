//! The region adapter a manifest gives.
//!
//! [`ManifestAdapter`] answers what the manifest declares: who the region is, where it lies,
//! its OSM extracts, its preview and its credit. The services (ALKIS, rasters, extra sources)
//! are built from the manifest by the other modules of the crate.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::info::{Names, OsmExtractInfo, PreviewInfo, RegionInfo};
use freemapper_adapter_api::rasters::RasterBackend;
use freemapper_adapter_api::records::{AdapterId, EstimateItem};
use freemapper_georef::crs::BoundingBox;
use geo::BoundingRect as _;
use geo_types::MultiPolygon;

use crate::boundary::parse_boundary;
use crate::error::ManifestError;
use crate::model::{BoundaryCrs, Manifest};

/// The files a manifest names, by their plain name in the folder of the adapter.
pub trait AdapterFiles {
    /// The text of the file `name`, or why it cannot be had.
    fn read(&self, name: &str) -> Result<Cow<'static, str>, String>;
}

/// The files of an adapter built into the program: name and text.
pub struct EmbeddedFiles(pub &'static [(&'static str, &'static str)]);

impl AdapterFiles for EmbeddedFiles {
    fn read(&self, name: &str) -> Result<Cow<'static, str>, String> {
        self.0
            .iter()
            .find(|(file, _)| *file == name)
            .map(|(_, text)| Cow::Borrowed(*text))
            .ok_or_else(|| "the file is not built in".to_string())
    }
}

/// When the boundary files of an adapter are parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parse {
    /// When the adapter is made, so a broken file is a finding: for an adapter from a folder.
    Now,
    /// When the boundary is asked for the first time: for the built-in adapters, whose files
    /// the tests have checked. A broken file then stops the program.
    OnFirstUse,
}

/// A boundary and the box around it.
struct Shape {
    boundary: MultiPolygon<f64>,
    bounds: BoundingBox,
}

impl Shape {
    fn of(boundary: MultiPolygon<f64>) -> Self {
        let rect = boundary
            .bounding_rect()
            .expect("a boundary that passed the check has an extent");
        Self {
            bounds: BoundingBox {
                min_e: rect.min().x,
                min_n: rect.min().y,
                max_e: rect.max().x,
                max_n: rect.max().y,
            },
            boundary,
        }
    }
}

/// A boundary file, parsed once.
struct BoundaryFile {
    /// The text, until it is parsed.
    text: Cow<'static, str>,
    crs: BoundaryCrs,
    path: PathBuf,
    shape: OnceLock<Shape>,
}

impl BoundaryFile {
    fn shape(&self) -> &Shape {
        self.shape.get_or_init(|| {
            match parse_boundary(&self.text, self.crs, &self.path) {
                Ok(boundary) => Shape::of(boundary),
                // Only a built-in file gets here unparsed, and the tests parse each of them.
                Err(finding) => panic!("{finding}"),
            }
        })
    }
}

/// The adapter of one manifest.
pub struct ManifestAdapter {
    manifest: Manifest,
    info: RegionInfo,
    boundary: BoundaryFile,
    /// The boundary of each OSM extract that has one, in the order of the extracts.
    extracts: Vec<Option<BoundaryFile>>,
}

impl ManifestAdapter {
    /// The adapter of `manifest` with the files it names, read through `files`. A file that is
    /// missing is a finding; with [`Parse::Now`] a file that is no valid boundary is one too.
    pub fn new(
        manifest: Manifest,
        files: &dyn AdapterFiles,
        parse: Parse,
    ) -> Result<Self, Vec<ManifestError>> {
        let folder = Path::new(&manifest.adapter.id).to_path_buf();
        let crs = manifest.boundary.crs;
        let mut findings = Vec::new();
        let mut read = |name: &str, key: &str| -> Option<BoundaryFile> {
            let path = folder.join(name);
            let text = match files.read(name) {
                Ok(text) => text,
                Err(why) => {
                    findings.push(ManifestError {
                        code: "manifest.file-missing",
                        path,
                        line: 0,
                        key: key.to_string(),
                        message: why,
                    });
                    return None;
                }
            };
            let file = BoundaryFile {
                text,
                crs,
                path,
                shape: OnceLock::new(),
            };
            if parse == Parse::OnFirstUse {
                return Some(file);
            }
            match parse_boundary(&file.text, crs, &file.path) {
                Ok(boundary) => Some(BoundaryFile {
                    text: Cow::Borrowed(""),
                    shape: OnceLock::from(Shape::of(boundary)),
                    ..file
                }),
                Err(finding) => {
                    findings.push(finding);
                    None
                }
            }
        };
        let boundary = read(&manifest.boundary.file, "boundary.file");
        let extracts: Vec<Option<BoundaryFile>> = manifest
            .osm
            .iter()
            .map(|extract| {
                extract
                    .boundary
                    .as_deref()
                    .and_then(|name| read(name, "osm.extract.boundary"))
            })
            .collect();
        let (Some(boundary), true) = (boundary, findings.is_empty()) else {
            return Err(findings);
        };
        Ok(Self {
            info: info_of(&manifest),
            manifest,
            boundary,
            extracts,
        })
    }

    /// The manifest the adapter was made of.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

/// The static facts of the region `manifest` declares.
fn info_of(manifest: &Manifest) -> RegionInfo {
    let id = |id: &str| -> AdapterId {
        id.parse()
            .expect("the validator accepts the ids the registry accepts")
    };
    let adapter = &manifest.adapter;
    RegionInfo {
        id: id(&adapter.id),
        provider: id(&adapter.provider),
        names: Names {
            en: adapter.names.en.clone(),
            de: adapter.names.de.clone(),
        },
        crs: adapter.crs,
        object_id_prefix: adapter.object_id_prefix.clone(),
        osm: manifest
            .osm
            .iter()
            .map(|extract| OsmExtractInfo {
                geofabrik_path: extract.geofabrik.clone(),
                megabytes: extract.megabytes,
                label: extract.label.clone(),
                prefix: format!("{}_", extract.cache_prefix),
            })
            .collect(),
        credit_short: manifest.attribution.short.clone(),
        preview: manifest.preview.as_ref().map(|preview| PreviewInfo {
            wms: preview.wms.clone(),
            layer: preview.layer.clone(),
            format: preview.format.clone(),
            params: preview.params.clone(),
            attribution: preview.attribution.clone(),
            agency: preview.agency.clone(),
        }),
    }
}

impl RegionAdapter for ManifestAdapter {
    fn info(&self) -> &RegionInfo {
        &self.info
    }

    fn boundary(&self) -> &MultiPolygon<f64> {
        &self.boundary.shape().boundary
    }

    fn extract_boundary(&self, index: usize) -> Option<&MultiPolygon<f64>> {
        self.extracts
            .get(index)?
            .as_ref()
            .map(|file| &file.shape().boundary)
    }

    fn alkis(&self) -> Option<&dyn AlkisSource> {
        None
    }

    /// The credit of the manifest once the project holds something of the region, with the
    /// year of the newest fetch in place of `{year}`.
    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        let Some(year) = used.newest_fetch_year(self.info.provider.as_str(), self.info.id.as_str())
        else {
            return Vec::new();
        };
        let credit = &self.manifest.attribution;
        let text = credit.text.replacen("{year}", &year.to_string(), 1);
        vec![match &credit.note {
            Some(note) => CreditLine::with_licence(text, note),
            None => CreditLine::new(text),
        }]
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        None
    }

    fn estimate(&self, _area: &EstimateArea, _env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        Vec::new()
    }

    fn bounds(&self) -> BoundingBox {
        self.boundary.shape().bounds
    }
}
