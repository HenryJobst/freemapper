//! The region adapters built into freemapper and the registry the engine asks (ADR 0082).
//!
//! The registry is a type of `freemapper-adapter-api` and knows no adapter. This module fills it
//! with the built-in states, so the library and its tests work without the command line; the
//! command line may install another registry once, before the first use.

use std::sync::OnceLock;

pub use freemapper_adapter_api::adapter::RegionAdapter;
pub use freemapper_adapter_api::registry::Registry;

/// The adapters built into the program: the German states of `freemapper-adapters-de` with
/// their raster backends, in the order the region functions always went through them.
pub fn builtin() -> Registry {
    builtin_with_hosts(&ExtraHosts::new())
}

/// The hosts a person allowed for built-in adapters besides those of their manifests, by the
/// id of the adapter (ADR 0110).
pub type ExtraHosts = std::collections::BTreeMap<String, Vec<String>>;

/// [`builtin`] with the hosts of `extra` allowed for the adapters it names. A host that is no
/// DNS name as a manifest's hosts are is left out; the check of the address a connection goes
/// to holds for these hosts as for all.
pub fn builtin_with_hosts(extra: &ExtraHosts) -> Registry {
    let mut registry = Registry::new();
    freemapper_adapters_de::states::register_with_hosts(&mut registry, extra)
        .expect("the built-in states have distinct ids and backends");
    registry
}

/// A registry with the raster backends of the program and no adapter: what the manifest of an
/// adapter that is checked on its own may name.
pub fn backends() -> Registry {
    let mut registry = Registry::new();
    freemapper_adapters_de::states::register_backends(&mut registry)
        .expect("the built-in backends have distinct names");
    registry
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

/// The registry of this process: the one installed, else the built-in states.
pub fn registry() -> &'static Registry {
    REGISTRY.get_or_init(builtin)
}

/// Makes `registry` the registry of this process. It works once and only before the first
/// use; afterwards the registry given comes back.
pub fn install_registry(registry: Registry) -> Result<(), Registry> {
    REGISTRY.set(registry)
}

/// What `work` comes to when it asks for `adapter`: through `fetcher`, behind the policy of
/// the adapter ([`RegionAdapter::policy`]) when it has one. For what the engine fetches for
/// one adapter: its sets of extra layers, its raster templates, its point clouds.
pub(crate) fn asking<T>(
    adapter: &dyn RegionAdapter,
    fetcher: &dyn crate::fetch::Fetcher,
    work: impl FnOnce(&dyn crate::fetch::Fetcher) -> T,
) -> T {
    match adapter.policy() {
        Some(policy) => work(&freemapper_adapter_api::policy::PolicyFetcher::new(
            fetcher, policy,
        )),
        None => work(fetcher),
    }
}

/// The names of the regions of the registry in its order, German with `german`, else English:
/// the list of what freemapper supports, for messages.
pub fn region_names(german: bool) -> Vec<&'static str> {
    registry()
        .adapters()
        .map(|adapter| {
            let names = &adapter.info().names;
            if german {
                names.de.as_str()
            } else {
                names.en.as_str()
            }
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::Arc;

    use freemapper_adapter_api::extras::{ExtraSet, NasService, RasterTemplate, TreeDataset};
    use freemapper_adapter_api::info::{Names, RegionInfo};
    use freemapper_adapter_api::policy::UrlPolicy;
    use freemapper_adapter_api::records::AdapterId;

    use super::*;

    /// The id of the region of [`square`].
    pub(crate) const SQUARE: AdapterId = AdapterId::built_in("square");

    /// A square region with a NAS service and the hosts its requests are held to, as an
    /// adapter from a folder has them.
    struct Square {
        info: RegionInfo,
        boundary: geo_types::MultiPolygon<f64>,
        nas: Option<NasService>,
        policy: Option<UrlPolicy>,
        extras: Vec<Box<dyn ExtraSet>>,
        templates: Vec<Box<dyn RasterTemplate>>,
    }

    impl RegionAdapter for Square {
        fn info(&self) -> &RegionInfo {
            &self.info
        }

        fn policy(&self) -> Option<&UrlPolicy> {
            self.policy.as_ref()
        }

        fn boundary(&self) -> &geo_types::MultiPolygon<f64> {
            &self.boundary
        }

        fn alkis(&self) -> Option<&dyn freemapper_adapter_api::alkis::AlkisSource> {
            None
        }

        fn nas(&self) -> Option<&NasService> {
            self.nas.as_ref()
        }

        fn tree_datasets(&self) -> &[TreeDataset] {
            &[]
        }

        fn extras(&self) -> &[Box<dyn ExtraSet>] {
            &self.extras
        }

        fn templates(&self) -> &[Box<dyn RasterTemplate>] {
            &self.templates
        }

        fn rasters(&self) -> Option<&dyn freemapper_adapter_api::rasters::RasterBackend> {
            None
        }

        fn attribution(
            &self,
            _used: &freemapper_adapter_api::credit::UsedSources<'_>,
        ) -> Vec<freemapper_adapter_api::credit::CreditLine> {
            Vec::new()
        }

        fn estimate(
            &self,
            _area: &freemapper_adapter_api::estimate::EstimateArea,
            _env: &freemapper_adapter_api::estimate::EstimateEnv<'_>,
        ) -> Vec<freemapper_adapter_api::records::EstimateItem> {
            Vec::new()
        }
    }

    /// A registry with the one region [`SQUARE`], E and N from 0 to 10 km (EPSG:25833), with
    /// the NAS service `nas`. `hosts`: the hosts its requests are held to, as for an adapter
    /// from a folder; `None`: it asks as a built-in adapter does.
    pub(crate) fn square(nas: Option<NasService>, hosts: Option<&[&str]>) -> Registry {
        square_with(nas, hosts, Vec::new())
    }

    /// [`square`] with the sets of extra layers `extras`.
    pub(crate) fn square_with(
        nas: Option<NasService>,
        hosts: Option<&[&str]>,
        extras: Vec<Box<dyn ExtraSet>>,
    ) -> Registry {
        square_of(nas, hosts, extras, Vec::new())
    }

    /// [`square`] with the raster templates `templates`.
    pub(crate) fn square_with_templates(
        hosts: Option<&[&str]>,
        templates: Vec<Box<dyn RasterTemplate>>,
    ) -> Registry {
        square_of(None, hosts, Vec::new(), templates)
    }

    fn square_of(
        nas: Option<NasService>,
        hosts: Option<&[&str]>,
        extras: Vec<Box<dyn ExtraSet>>,
        templates: Vec<Box<dyn RasterTemplate>>,
    ) -> Registry {
        let mut registry = Registry::new();
        registry
            .add(Arc::new(Square {
                info: RegionInfo {
                    id: SQUARE,
                    provider: SQUARE,
                    names: Names {
                        en: "Square".to_string(),
                        de: "Quadrat".to_string(),
                    },
                    crs: crate::crs::Crs::Utm33,
                    object_id_prefix: None,
                    osm: Vec::new(),
                    credit_short: None,
                    preview: None,
                },
                boundary: geo_types::MultiPolygon(vec![
                    geo_types::Rect::new(
                        geo_types::coord! { x: 0.0, y: 0.0 },
                        geo_types::coord! { x: 10_000.0, y: 10_000.0 },
                    )
                    .to_polygon(),
                ]),
                nas,
                policy: hosts.map(|hosts| UrlPolicy::new(hosts.iter().copied()).unwrap()),
                extras,
                templates,
            }))
            .unwrap();
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::sources::region::ids::{IdFacts as _, regions};

    /// The full ALKIS of LGB and the tree cadastre of Potsdam belong to Brandenburg; the
    /// queries over the registry find them for an area there and nowhere else.
    #[test]
    fn brandenburg_names_the_nas_service_and_the_tree_cadastre() {
        use crate::crs::BoundingBox;
        use crate::sources::{alkis_nas, trees};
        for adapter in registry().adapters() {
            let brandenburg = adapter.info().id.as_str() == "brandenburg";
            assert_eq!(
                adapter.nas().is_some(),
                brandenburg,
                "{}",
                adapter.info().id
            );
            assert_eq!(
                adapter.tree_datasets().len(),
                usize::from(brandenburg),
                "{}",
                adapter.info().id
            );
        }
        assert_eq!(
            trees::all_tree_datasets()
                .map(|dataset| dataset.id.as_ref())
                .collect::<Vec<_>>(),
            ["potsdam"]
        );
        // Sanssouci, Potsdam, and the Zwinger, Dresden (EPSG:25833).
        let sanssouci = BoundingBox {
            min_e: 366_200.0,
            min_n: 5_807_100.0,
            max_e: 367_100.0,
            max_n: 5_807_700.0,
        };
        let zwinger = BoundingBox {
            min_e: 410_800.0,
            min_n: 5_656_300.0,
            max_e: 411_200.0,
            max_n: 5_656_700.0,
        };
        assert_eq!(alkis_nas::nas_services_for(&sanssouci).len(), 1);
        assert_eq!(trees::tree_datasets_for(&sanssouci).len(), 1);
        assert!(alkis_nas::nas_services_for(&zwinger).is_empty());
        assert!(trees::tree_datasets_for(&zwinger).is_empty());
    }

    /// Each built-in adapter is held to the hosts of its own manifest, not to those of
    /// another state.
    #[test]
    fn every_built_in_adapter_has_the_policy_of_its_manifest() {
        assert!(registry().adapters().all(|a| a.policy().is_some()));
        let hamburg = registry().get("hamburg").unwrap().policy().unwrap();
        assert!(hamburg.check_url("https://geodienste.hamburg.de/x").is_ok());
        assert!(hamburg.check_url("https://isk.geobasis-bb.de/x").is_err());
        assert!(
            hamburg
                .check_url("https://download.geofabrik.de/x")
                .is_err()
        );
    }

    /// The lines of `source` that name a region or reach into the adapter crate: the id of an
    /// adapter of `ids` as a text; a constant of the boundaries, a state or an agency; a path
    /// into the adapter crate; a path into one of its `modules` (`berlin::`, `lgb::`). Tests do
    /// not count (everything from the first test module on), nor do comments, nor the modules
    /// `ids` and `golden`, which hold the names the tests go by.
    fn naming_a_region(source: &str, ids: &[&str], modules: &[String]) -> Vec<String> {
        let names_one = |line: &str| {
            let followed_by_upper = |name: &str| {
                line.match_indices(name).any(|(at, _)| {
                    line[at + name.len()..].starts_with(|c: char| c.is_ascii_uppercase())
                })
            };
            // `berlin::` as a path of its own: not the end of a longer name, and not the
            // engine's own `build::berlin`, the draw rules.
            let path_into = |module: &String| {
                let path = format!("{module}::");
                line.match_indices(&path).any(|(at, _)| {
                    let before = &line[..at];
                    !before.ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
                        && !before.ends_with("build::")
                })
            };
            ids.iter().any(|id| line.contains(&format!("\"{id}\"")))
                || followed_by_upper("boundaries::")
                || followed_by_upper("State::")
                || followed_by_upper("Agency::")
                || line.contains("freemapper_adapters_de::")
                || modules.iter().any(path_into)
        };
        let lines: Vec<&str> = source.lines().collect();
        let tests = lines
            .iter()
            .enumerate()
            .position(|(at, line)| {
                let line = line.trim_start();
                line.starts_with("mod tests")
                    || (line.starts_with("#[cfg(test)]")
                        && lines
                            .get(at + 1)
                            .is_some_and(|next| next.trim_start().contains("mod ")))
            })
            .unwrap_or(lines.len());
        let mut found = Vec::new();
        // Inside `pub mod ids {` or `pub mod golden {`: how many braces are open.
        let mut names: Option<i32> = None;
        for line in &lines[..tests] {
            let trimmed = line.trim_start();
            if names.is_none()
                && (trimmed.starts_with("pub mod ids {") || trimmed.starts_with("pub mod golden {"))
            {
                names = Some(0);
            }
            if let Some(open) = names.as_mut() {
                *open += line.matches('{').count() as i32 - line.matches('}').count() as i32;
                if *open <= 0 {
                    names = None;
                }
                continue;
            }
            if !trimmed.starts_with("//") && names_one(line) {
                found.push(line.trim().to_string());
            }
        }
        found
    }

    /// What the engine still takes from the adapter crate by name: the file and a part of the
    /// line, each with its reason. A line that is not here fails
    /// [`the_engine_names_no_region`].
    const STILL_NAMED: &[(&str, &str)] = &[
        // The adapters built into the program are linked here.
        (
            "adapters/mod.rs",
            "freemapper_adapters_de::states::register_with_hosts(",
        ),
        (
            "adapters/mod.rs",
            "freemapper_adapters_de::states::register_backends(",
        ),
        // The key of the one set of extra layers the project file has a field for.
        ("project.rs", "const REGISTERS_SET: &str = \"berlin\";"),
        // The table of Berlin's layers, typed, for the inputs of the draw rules (naming a layer
        // is allowed; the rules leave when adapters bring rules).
        ("project.rs", "use freemapper_adapters_de::berlin;"),
        ("project.rs", "berlin::ExtraLayer"),
        ("project.rs", "berlin::EXTRA_LAYERS"),
        (
            "project.rs",
            "berlin::extra_layers(project.map_kind.contract())",
        ),
        ("project/export.rs", "berlin::join_footprint_dates("),
        // Which registers drew objects, for the credit: open with the rank of `dedup` (ADR 0114).
        (
            "project.rs",
            "use freemapper_adapters_de::berlin::BERLIN_REGISTERS;",
        ),
        // The tile download of the tile providers, re-exported for the engine's tests of it.
        ("fetch.rs", "pub use freemapper_adapters_de::tile_fetch::{"),
        // How a product names its tiles, a trait of the tile providers that `derive` reads.
        (
            "derive/orthophoto.rs",
            "use freemapper_adapters_de::lgb::ProductTiles as _;",
        ),
        (
            "derive/terrain.rs",
            "use freemapper_adapters_de::lgb::ProductTiles as _;",
        ),
        // The endpoints of the cadastre of two states, for the helpers the engine's tests use.
        (
            "sources/alkis.rs",
            "pub use freemapper_adapters_de::states::alkis::{",
        ),
    ];

    /// The engine asks the adapters of the registry and decides nothing by the name of a
    /// region (ADR 0082, ADR 0114). Outside the draw rules (`build/`, which read the layers of
    /// Berlin by name until adapters bring rules), its code that is no test names no region
    /// and reaches into the adapter crate only at the lines of [`STILL_NAMED`].
    #[test]
    fn the_engine_names_no_region() {
        let ids: Vec<&str> = registry()
            .adapters()
            .map(|adapter| adapter.info().id.as_str())
            .collect();
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        // The modules of the adapter crate, from its `lib.rs`.
        let modules: Vec<String> =
            std::fs::read_to_string(manifest.join("../adapters/de/src/lib.rs"))
                .unwrap()
                .lines()
                .filter_map(|line| line.strip_prefix("pub mod ")?.strip_suffix(';'))
                .map(str::to_string)
                .collect();
        assert!(
            modules.iter().any(|module| module == "berlin"),
            "{modules:?}"
        );
        // The detector finds what it is for.
        let sample = "let a = x == \"hessen\";\n// \"berlin\" in a comment\nuse b::State::Berlin;\n\
                      let c = boundaries::BERLIN;\nlet near = berlin::near_berlin(&bbox);\n\
                      use crate::build::berlin::LayerInput;\nuse freemapper_adapters_de::lgb::Product;\n\
                      let plain = boundaries_of(id);\npub mod ids {\n    pub const B: &str = \"berlin\";\n}\n\
                      let d = lgb::Product::Dop;\n#[cfg(test)]\nmod tests {\n    let e = \"saxony\";\n}\n";
        assert_eq!(
            naming_a_region(sample, &ids, &modules),
            [
                "let a = x == \"hessen\";",
                "use b::State::Berlin;",
                "let c = boundaries::BERLIN;",
                "let near = berlin::near_berlin(&bbox);",
                "use freemapper_adapters_de::lgb::Product;",
                "let d = lgb::Product::Dop;",
            ]
        );
        /// Whether the module `file` is declared for tests only (`#[cfg(test)] mod name;`).
        fn for_tests_only(file: &std::path::Path) -> bool {
            let (Some(dir), Some(name)) = (file.parent(), file.file_stem()) else {
                return false;
            };
            let declaration = format!("#[cfg(test)]\nmod {};", name.to_string_lossy());
            [dir.join("mod.rs"), dir.with_extension("rs")]
                .iter()
                .filter_map(|parent| std::fs::read_to_string(parent).ok())
                // A Windows checkout may turn LF into CRLF.
                .any(|parent| parent.replace("\r\n", "\n").contains(&declaration))
        }
        fn visit(
            dir: &std::path::Path,
            ids: &[&str],
            modules: &[String],
            found: &mut Vec<(String, String)>,
        ) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    if path.file_name().is_some_and(|name| name != "build") {
                        visit(&path, ids, modules, found);
                    }
                } else if path.extension().is_some_and(|e| e == "rs") && !for_tests_only(&path) {
                    let source = std::fs::read_to_string(&path).unwrap();
                    for line in naming_a_region(&source, ids, modules) {
                        found.push((path.to_string_lossy().replace('\\', "/"), line));
                    }
                }
            }
        }
        let mut found = Vec::new();
        visit(&manifest.join("src"), &ids, &modules, &mut found);
        let known = |file: &str, line: &str| {
            STILL_NAMED.iter().any(|(suffix, part)| {
                file.ends_with(&format!("/src/{suffix}")) && line.contains(part)
            })
        };
        let new: Vec<String> = found
            .iter()
            .filter(|(file, line)| !known(file, line))
            .map(|(file, line)| format!("{file}: {line}"))
            .collect();
        assert!(new.is_empty(), "{new:#?}");
        // And the list holds nothing that is gone.
        for (suffix, part) in STILL_NAMED {
            assert!(
                found
                    .iter()
                    .any(|(file, line)| file.ends_with(&format!("/src/{suffix}"))
                        && line.contains(part)),
                "{suffix}: {part} is no longer there; take it off the list"
            );
        }
    }

    /// What the engine fetches for one adapter goes through that adapter's hosts: a request
    /// off them is not sent, one on them reaches the fetcher. The caller names the adapter
    /// itself, so there is no id that could miss the registry and be asked plainly.
    #[test]
    fn asking_an_adapter_keeps_to_its_hosts() {
        use crate::fetch::FetchError;
        use crate::fetch::test_support::MapFetcher;
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", b"ok".to_vec());
        let berlin = registry().get("berlin").expect("built in");
        let elsewhere = "https://geo.example.org/wms?token=abc";
        let refused = asking(berlin, &fetcher, |fetcher| fetcher.get(elsewhere));
        let Err(FetchError::HostNotNamed(refused)) = refused else {
            panic!("{refused:?}");
        };
        assert_eq!(refused.target, "https://geo.example.org/wms");
        assert_eq!(refused.adapter, "berlin");
        assert_eq!(fetcher.calls.get(), 0);
        let own = "https://gdi.berlin.de/services/wms/k5";
        assert!(asking(berlin, &fetcher, |fetcher| fetcher.get(own)).is_ok());
        assert_eq!(fetcher.calls.get(), 1);
        // An adapter that is held to no hosts (none of the program is) asks as the fetcher does.
        let free = test_support::square(None, None);
        let square = free.get("square").unwrap();
        assert!(asking(square, &fetcher, |fetcher| fetcher.get(elsewhere)).is_ok());
        assert_eq!(fetcher.calls.get(), 2);
    }

    /// The error for an area outside every region names the regions of the registry; with the
    /// built-in states it reads as it always did.
    #[test]
    fn the_outside_coverage_error_names_the_states_as_before() {
        assert_eq!(
            crate::project::ProjectError::OutsideCoverage.to_string(),
            "the map area lies outside the supported states (Brandenburg, Berlin, Saxony, \
             Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, \
             North Rhine-Westphalia, Lower Saxony, Rhineland-Palatinate, Saarland); freemapper \
             has data sources for those only so far"
        );
        assert_eq!(region_names(true)[2], "Sachsen");
    }

    /// The backends alone: no adapter, and every backend a built-in manifest names.
    #[test]
    fn the_backends_come_without_an_adapter() {
        let backends = backends();
        assert_eq!(backends.len(), 0);
        for name in ["lgb", "geosn", "lgv", "hvbg", "lgln"] {
            assert!(backends.backend(name).is_some(), "{name}");
        }
        assert!(backends.backend("lvermgeo").is_none());
    }

    #[test]
    fn registry_lists_the_twelve() {
        let built_in = builtin();
        let ids: Vec<&str> = built_in.adapters().map(|a| a.info().id.as_str()).collect();
        let all: Vec<String> = regions::ALL.iter().map(ToString::to_string).collect();
        assert_eq!(ids, all);
        assert_eq!(ids.len(), 12);
        // The eleven that were code first, in the order the region functions always went
        // through them; a state added since comes after them.
        let legacy: Vec<String> = regions::LEGACY.iter().map(ToString::to_string).collect();
        assert_eq!(ids[..11], legacy);
        assert_eq!(ids[11], "saarland");
        // The registry of the process is the built-in one unless another was installed.
        assert_eq!(registry().len(), 12);
    }

    /// G1 records what the enums of v0.12.0 answered; the adapters must say the same.
    #[test]
    fn legacy_info_matches_g1() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/golden/adapters/g1-regions.json");
        let recorded: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let recorded = recorded.as_array().unwrap();
        let registry = builtin();
        // The eleven of v0.12.0 come first in the registry; what was added since is not in G1.
        assert_eq!(recorded.len(), regions::LEGACY.len());
        for (facts, adapter) in recorded.iter().zip(registry.adapters()) {
            let info = adapter.info();
            assert_eq!(facts["id"], info.id.as_str());
            assert_eq!(facts["provider"], info.provider.as_str(), "{}", info.id);
            assert_eq!(facts["crs_epsg"], info.crs.epsg(), "{}", info.id);
            assert_eq!(
                facts["object_id_prefix"],
                info.object_id_prefix.as_deref().unwrap(),
                "{}",
                info.id
            );
            let osm = facts["osm"].as_array().unwrap();
            assert_eq!(osm.len(), info.osm.len(), "{}", info.id);
            for (extract, known) in osm.iter().zip(&info.osm) {
                assert_eq!(extract["url"], known.url());
                assert_eq!(extract["state_url"], known.state_url());
                assert_eq!(extract["prefix"], known.prefix);
                assert_eq!(extract["megabytes"], known.megabytes);
                assert_eq!(extract["label"], known.label);
            }
            assert!(!adapter.boundary().0.is_empty(), "{}", info.id);
        }
    }

    /// G1 records who serves the rasters of each state and how; the backends say the same.
    #[test]
    fn provider_info_matches_g1() {
        use crate::sources::region::golden::legacy_facts;
        use freemapper_adapter_api::rasters::RasterKind;
        for facts in legacy_facts() {
            let adapter = registry().get(&facts.id).unwrap();
            let info = adapter.rasters().unwrap().provider();
            assert_eq!(info.id.as_str(), facts.provider, "{}", facts.id);
            assert_eq!(info.cache_dir, facts.cache_dir, "{}", facts.id);
            assert_eq!(info.terrain_source, facts.terrain_source, "{}", facts.id);
            assert_eq!(
                info.kind == RasterKind::PerArea,
                facts.per_area,
                "{}",
                facts.id
            );
        }
        let kind = |id: &str| {
            registry()
                .get(id)
                .unwrap()
                .rasters()
                .unwrap()
                .provider()
                .kind
        };
        assert_eq!(kind("berlin"), RasterKind::Tiles { grid_km: 1 });
        assert_eq!(kind("saxony"), RasterKind::Tiles { grid_km: 2 });
    }

    /// The engine and the crates beside it name no state through an enum of the engine: the
    /// enums `Region` and `Provider` are gone (plan C14), and nothing may bring them back. The
    /// test reads every source file of the crates outside their `tests` folders.
    #[test]
    fn no_enum_matches_left() {
        fn sources(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    if path
                        .file_name()
                        .is_some_and(|name| name != "tests" && name != "target")
                    {
                        sources(&path, found);
                    }
                } else if path.extension().is_some_and(|e| e == "rs") {
                    found.push(path);
                }
            }
        }
        let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut files = Vec::new();
        sources(&crates, &mut files);
        assert!(files.len() > 80, "{} files", files.len());
        // Written in two parts, so that this file does not name them itself.
        let names = [["Region", "::"].concat(), ["Provider", "::"].concat()];
        let mut left = Vec::new();
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            for (index, line) in text.lines().enumerate() {
                let named = names.iter().any(|name| {
                    line.match_indices(name.as_str()).any(|(at, _)| {
                        // `TileProvider::Lgb` of the adapter crate is another type.
                        !line[..at]
                            .chars()
                            .next_back()
                            .is_some_and(|c| c.is_alphanumeric() || c == '_')
                    })
                });
                if named {
                    left.push(format!("{}:{}", file.display(), index + 1));
                }
            }
        }
        assert!(left.is_empty(), "the enums are named in: {left:?}");
    }
}
