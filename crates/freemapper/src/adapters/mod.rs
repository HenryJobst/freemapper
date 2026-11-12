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
    let mut registry = Registry::new();
    freemapper_adapters_de::states::register(&mut registry)
        .expect("the built-in states have distinct ids and backends");
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

/// The hosts the requests of a run may go to when they are held to what was declared: the hosts
/// every adapter of the registry declares, and those of the requests that belong to no adapter
/// (OSM extracts, the place search, the map tiles). No run is held to it yet: an adapter from a
/// folder asks through its own policy ([`RegionAdapter::policy`]), and the built-in ones are
/// held to their lists once live runs of all of them have shown the redirects (ADR 0106).
pub fn declared_policy() -> freemapper_adapter_api::policy::UrlPolicy {
    let hosts = registry()
        .adapters()
        .filter_map(|adapter| adapter.declared())
        .flat_map(|declared| declared.hosts.iter().map(String::as_str));
    freemapper_adapter_api::policy::UrlPolicy::core()
        .with_hosts(hosts)
        .expect("the hosts of a manifest are DNS names")
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

    use freemapper_adapter_api::extras::{NasService, TreeDataset};
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

    /// The declared hosts of the built-in states and the hosts of the program's own requests
    /// pass the policy of all adapters; nothing else does.
    #[test]
    fn the_declared_policy_is_the_hosts_of_the_adapters_and_of_the_program() {
        let policy = declared_policy();
        for url in [
            "https://isk.geobasis-bb.de/ows/alkis_vereinf_wfs",
            "https://download.geofabrik.de/europe/germany/berlin-latest.osm.pbf",
            "https://nominatim.openstreetmap.org/search",
        ] {
            assert!(policy.check_url(url).is_ok(), "{url}");
        }
        assert!(policy.check_url("https://geo.example.org/wfs").is_err());
        // A built-in adapter is held to no list of its own.
        assert!(registry().adapters().all(|a| a.policy().is_none()));
    }

    /// The error for an area outside every region names the regions of the registry; with the
    /// built-in states it reads as it always did.
    #[test]
    fn the_outside_coverage_error_names_the_states_as_before() {
        assert_eq!(
            crate::project::ProjectError::OutsideCoverage.to_string(),
            "the map area lies outside the supported states (Brandenburg, Berlin, Saxony, \
             Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, \
             North Rhine-Westphalia, Lower Saxony, Rhineland-Palatinate); freemapper has data \
             sources for those only so far"
        );
        assert_eq!(region_names(true)[2], "Sachsen");
    }

    #[test]
    fn registry_lists_the_eleven_in_legacy_order() {
        let built_in = builtin();
        let ids: Vec<&str> = built_in.adapters().map(|a| a.info().id.as_str()).collect();
        let legacy: Vec<String> = regions::ALL.iter().map(ToString::to_string).collect();
        assert_eq!(ids, legacy);
        assert_eq!(ids.len(), 11);
        // The registry of the process is the built-in one unless another was installed.
        assert_eq!(registry().len(), 11);
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
        assert_eq!(recorded.len(), registry.len());
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
