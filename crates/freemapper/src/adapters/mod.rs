//! The region adapters built into freemapper and the registry the engine asks (ADR 0082).
//!
//! The registry is a type of `freemapper-adapter-api` and knows no adapter. This module fills it
//! with the built-in states, so the library and its tests work without the command line; the
//! command line may install another registry once, before the first use.

use std::sync::{Arc, OnceLock};

pub use freemapper_adapter_api::adapter::RegionAdapter;
pub use freemapper_adapter_api::registry::Registry;

use crate::sources::region::Region;

pub(crate) mod alkis;
pub(crate) mod credit;
pub(crate) mod estimate;
pub mod legacy;
pub(crate) mod rasters;

/// The states built into the program, in the order the region functions always went through
/// them.
pub fn builtin() -> Registry {
    let mut registry = Registry::new();
    for region in Region::ALL {
        registry
            .add(Arc::new(legacy::LegacyAdapter::new(region)))
            .expect("the built-in states have distinct ids");
    }
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
mod tests {
    use super::*;

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
        let legacy: Vec<&str> = Region::ALL.into_iter().map(Region::id).collect();
        assert_eq!(ids, legacy);
        assert_eq!(ids.len(), 11);
        // The registry of the process is the built-in one unless another was installed.
        assert_eq!(registry().len(), 11);
    }

    /// G1 records what the enums answer; the adapters must say the same.
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
}
