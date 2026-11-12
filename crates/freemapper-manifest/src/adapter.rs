//! The region adapter a manifest gives.
//!
//! [`ManifestAdapter`] answers what the manifest declares: who the region is, where it lies,
//! its OSM extracts, its preview and its credit. The services (ALKIS, rasters, extra sources)
//! are built from the manifest by the other modules of the crate.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use freemapper_adapter_api::adapter::RegionAdapter;
use freemapper_adapter_api::alkis::AlkisSource;
use freemapper_adapter_api::credit::{CreditLine, UsedSources};
use freemapper_adapter_api::error::Remedy;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::extras::{NasService, TreeDataset};
use freemapper_adapter_api::info::{Declared, Names, OsmExtractInfo, PreviewInfo, RegionInfo};
use freemapper_adapter_api::policy::{Owner, UrlPolicy};
use freemapper_adapter_api::rasters::RasterBackend;
use freemapper_adapter_api::records::{AdapterId, EstimateItem};
use freemapper_adapter_api::registry::Registry;
use freemapper_georef::crs::BoundingBox;
use geo::BoundingRect as _;
use geo_types::MultiPolygon;

use crate::boundary::parse_boundary;
use crate::error::ManifestError;
use crate::guard::{GuardedAlkis, GuardedRasters};
use crate::model::{BoundaryCrs, Manifest, Orthophoto};
use crate::rasters::DeclaredRasters;

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
    alkis: Option<Box<dyn AlkisSource>>,
    rasters: Option<Arc<dyn RasterBackend>>,
    nas: Option<NasService>,
    trees: Vec<TreeDataset>,
    declared: Declared,
    /// The hosts the requests are held to, for an adapter from a folder.
    policy: Option<Arc<UrlPolicy>>,
}

impl ManifestAdapter {
    /// The adapter of `manifest` with the files it names, read through `files`, and the raster
    /// backend it names, taken from `backends`. A file that is missing is a finding, and so is a
    /// backend the program does not have; with [`Parse::Now`] a file that is no valid boundary
    /// is one too.
    pub fn new(
        manifest: Manifest,
        files: &dyn AdapterFiles,
        parse: Parse,
        backends: &Registry,
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
        let manifest_path = folder.join(crate::MANIFEST_FILE);
        let finding = |code: &'static str, key: &str, message: String| ManifestError {
            code,
            path: manifest_path.clone(),
            line: 0,
            key: key.to_string(),
            message,
        };
        let rasters: Option<Arc<dyn RasterBackend>> = match &manifest.orthophoto {
            None => None,
            Some(Orthophoto::Backend { name }) => match backends.backend(name) {
                Some(backend) if backend.provider().id.as_str() == manifest.adapter.provider => {
                    Some(backend)
                }
                Some(backend) => {
                    let message = format!(
                        "the backend `{name}` fetches for the provider `{}`, the manifest says `{}`",
                        backend.provider().id,
                        manifest.adapter.provider
                    );
                    findings.push(finding("manifest.value", "adapter.provider", message));
                    None
                }
                None => {
                    let message = format!(
                        "this freemapper has no raster backend `{name}`; a backend is code of an \
                         adapter crate, which a build of freemapper includes by a Cargo feature"
                    );
                    findings.push(finding(
                        "manifest.backend-missing",
                        "orthophoto.backend",
                        message,
                    ));
                    None
                }
            },
            Some(_) => DeclaredRasters::new(&manifest)
                .map(|declared| Arc::new(declared) as Arc<dyn RasterBackend>),
        };
        let (Some(boundary), true) = (boundary, findings.is_empty()) else {
            return Err(findings);
        };
        let info = info_of(&manifest);
        Ok(Self {
            alkis: manifest.alkis.as_ref().map(crate::alkis::alkis_source),
            rasters,
            nas: manifest
                .nas
                .first()
                .map(|nas| crate::extras::nas_service(nas, &info.id)),
            trees: manifest
                .trees
                .iter()
                .map(|trees| crate::extras::tree_dataset(trees, &info.id))
                .collect(),
            declared: Declared {
                revision: manifest.adapter.revision,
                hosts: manifest.hosts.clone(),
                last_verified: manifest.adapter.last_verified,
            },
            info,
            manifest,
            boundary,
            extracts,
            policy: None,
        })
    }

    /// The adapter with its requests kept on the hosts of `[net] hosts`, redirects included:
    /// a service may answer with an address nobody declared. Its ALKIS and its rasters ask
    /// through the policy here; the engine asks its NAS service and its tree cadastres through
    /// [`RegionAdapter::policy`]. The adapters from a folder are made this way: a host the
    /// manifest does not name belongs into the manifest.
    pub fn asking_declared_hosts_only(self) -> Self {
        self.held_to_its_hosts(Remedy::Manifest, &[])
    }

    /// [`Self::asking_declared_hosts_only`] with what a person can do about a host the adapter
    /// does not name (`remedy`), and with the hosts in `extra` allowed besides those of the
    /// manifest: the hosts a person allowed for a built-in state. An entry of `extra` that is
    /// no DNS name as the manifest's hosts are is left out.
    pub fn held_to_its_hosts(mut self, remedy: Remedy, extra: &[String]) -> Self {
        let mut policy = UrlPolicy::new(self.manifest.hosts.iter().map(String::as_str))
            .expect("the validator accepts the hosts the policy accepts");
        for host in extra {
            if let Ok(wider) = policy.clone().with_hosts([host.as_str()]) {
                policy = wider;
            }
        }
        let policy = Arc::new(policy.owned_by(Owner {
            adapter: self.info.id.to_string(),
            name: self.info.names.en.clone(),
            remedy,
        }));
        self.alkis = self.alkis.take().map(|inner| {
            Box::new(GuardedAlkis {
                inner,
                policy: policy.clone(),
            }) as Box<dyn AlkisSource>
        });
        self.rasters = self.rasters.take().map(|inner| {
            Arc::new(GuardedRasters {
                inner,
                policy: policy.clone(),
            }) as Arc<dyn RasterBackend>
        });
        self.policy = Some(policy);
        self
    }

    /// Whether the requests of the adapter are kept on its declared hosts.
    pub fn asks_declared_hosts_only(&self) -> bool {
        self.policy.is_some()
    }

    /// The manifest the adapter was made of.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

/// The static facts of the region `manifest` declares.
pub fn info_of(manifest: &Manifest) -> RegionInfo {
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

    fn declared(&self) -> Option<&Declared> {
        Some(&self.declared)
    }

    fn policy(&self) -> Option<&UrlPolicy> {
        self.policy.as_deref()
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
        self.alkis.as_deref()
    }

    fn nas(&self) -> Option<&NasService> {
        self.nas.as_ref()
    }

    fn tree_datasets(&self) -> &[TreeDataset] {
        &self.trees
    }

    /// The credit of the manifest once the project holds something of the region, with the
    /// year of the newest fetch in place of `{year}`, then the line of each tree cadastre that
    /// drew trees.
    fn attribution(&self, used: &UsedSources<'_>) -> Vec<CreditLine> {
        let credit = &self.manifest.attribution;
        let own = used
            .newest_fetch_year(self.info.provider.as_str(), self.info.id.as_str())
            .map(|year| {
                let text = credit.text.replacen("{year}", &year.to_string(), 1);
                match &credit.note {
                    Some(note) => CreditLine::with_licence(text, note),
                    None => CreditLine::new(text),
                }
            });
        let trees = self
            .trees
            .iter()
            .filter(|dataset| used.drawn_registers.contains(&dataset.id.as_ref()))
            .map(|dataset| CreditLine::new(dataset.attribution.as_ref()));
        own.into_iter().chain(trees).collect()
    }

    fn rasters(&self) -> Option<&dyn RasterBackend> {
        self.rasters.as_deref()
    }

    /// The raster downloads, as the backend of the rasters estimates them.
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        self.rasters
            .as_ref()
            .map(|rasters| rasters.estimate(area, env))
            .unwrap_or_default()
    }

    fn bounds(&self) -> BoundingBox {
        self.boundary.shape().bounds
    }
}
