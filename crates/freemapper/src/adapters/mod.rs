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

/// What `work` comes to when it asks for the adapter `id` of the registry: through `fetcher`,
/// behind the policy of the adapter ([`RegionAdapter::policy`]) when it has one. For what the
/// engine fetches itself for one region (the registers and the K5 of Berlin).
pub(crate) fn asking_for<T>(
    id: &str,
    fetcher: &dyn crate::fetch::Fetcher,
    work: impl FnOnce(&dyn crate::fetch::Fetcher) -> T,
) -> T {
    match registry().get(id).and_then(|adapter| adapter.policy()) {
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

    /// What the engine fetches itself for one region goes through that region's hosts: a
    /// request off them is not sent, one on them reaches the fetcher, and an id the registry
    /// does not have is asked plainly.
    #[test]
    fn asking_for_an_adapter_keeps_to_its_hosts() {
        use crate::fetch::FetchError;
        use crate::fetch::test_support::MapFetcher;
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", b"ok".to_vec());
        let elsewhere = "https://geo.example.org/wms?token=abc";
        let refused = asking_for("berlin", &fetcher, |fetcher| fetcher.get(elsewhere));
        let Err(FetchError::HostNotNamed(refused)) = refused else {
            panic!("{refused:?}");
        };
        assert_eq!(refused.target, "https://geo.example.org/wms");
        assert_eq!(refused.adapter, "berlin");
        assert_eq!(fetcher.calls.get(), 0);
        let own = "https://gdi.berlin.de/services/wms/k5";
        assert!(asking_for("berlin", &fetcher, |fetcher| fetcher.get(own)).is_ok());
        assert!(asking_for("nowhere", &fetcher, |fetcher| fetcher.get(elsewhere)).is_ok());
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
             North Rhine-Westphalia, Lower Saxony, Rhineland-Palatinate); freemapper has data \
             sources for those only so far"
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
