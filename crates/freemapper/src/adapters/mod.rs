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

#[cfg(test)]
mod tests {
    use super::*;

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
