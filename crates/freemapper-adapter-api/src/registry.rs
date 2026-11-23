// SPDX-License-Identifier: AGPL-3.0-or-later

//! The adapters a program knows, in the order the region functions go through them.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::adapter::RegionAdapter;
use crate::permits::PermitSourceFactory;
use crate::rasters::RasterBackend;
use crate::records::AdapterId;

/// The list of region adapters. The order decides ties: where two regions both hold a point,
/// the earlier one is its raster region. The registry knows no adapter itself; the program
/// that starts fills it.
#[derive(Default, Clone)]
pub struct Registry {
    adapters: Vec<Arc<dyn RegionAdapter>>,
    /// The raster backends of the adapter crates, by the name a manifest calls them.
    backends: BTreeMap<String, Arc<dyn RasterBackend>>,
    /// The ids of the adapters built into the program, which an adapter from a folder may
    /// replace (ADR 0188); see [`Registry::mark_builtin`].
    builtin: BTreeSet<String>,
    /// What turns the permit sources a manifest declares into sources.
    permit_factory: Option<Arc<dyn PermitSourceFactory>>,
}

/// A second raster backend with a name the registry holds already.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a raster backend with the name {0} is registered already")]
pub struct DuplicateBackend(pub String);

/// A second adapter with an id the registry holds already.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("an adapter with the id {0} is registered already")]
pub struct DuplicateAdapter(pub AdapterId);

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `adapter` after those already there. An id is taken once: a later adapter cannot
    /// replace an earlier one.
    pub fn add(&mut self, adapter: Arc<dyn RegionAdapter>) -> Result<(), DuplicateAdapter> {
        let id = &adapter.info().id;
        if self.get(id.as_str()).is_some() {
            return Err(DuplicateAdapter(id.clone()));
        }
        self.adapters.push(adapter);
        Ok(())
    }

    /// Notes the adapters now in the registry as the built-in ones: the program calls it once
    /// when it has added them. Only these can be replaced by [`Registry::replace`].
    pub fn mark_builtin(&mut self) {
        self.builtin = self
            .adapters
            .iter()
            .map(|adapter| adapter.info().id.to_string())
            .collect();
    }

    /// Whether `id` is an adapter built into the program that no folder has replaced yet.
    pub fn is_builtin(&self, id: &str) -> bool {
        self.builtin.contains(id)
    }

    /// Puts `adapter` in the place of the built-in adapter with the same id, so that the
    /// order stays as it was, and returns the one it replaced. An id that is not built in
    /// (or was replaced already) is [`DuplicateAdapter`]: a folder replaces a built-in
    /// adapter once and no other.
    pub fn replace(
        &mut self,
        adapter: Arc<dyn RegionAdapter>,
    ) -> Result<Arc<dyn RegionAdapter>, DuplicateAdapter> {
        let id = adapter.info().id.clone();
        if !self.builtin.remove(id.as_str()) {
            return Err(DuplicateAdapter(id));
        }
        let place = self
            .adapters
            .iter()
            .position(|other| other.info().id == id)
            .expect("a built-in id has its adapter");
        Ok(std::mem::replace(&mut self.adapters[place], adapter))
    }

    /// Makes the raster backend `backend` known as `name`: the code of an adapter crate that
    /// a manifest names in a raster table (`backend = "<name>"`). A name is taken once.
    pub fn add_backend(
        &mut self,
        name: &str,
        backend: Arc<dyn RasterBackend>,
    ) -> Result<(), DuplicateBackend> {
        if self.backends.contains_key(name) {
            return Err(DuplicateBackend(name.to_string()));
        }
        self.backends.insert(name.to_string(), backend);
        Ok(())
    }

    /// Makes `factory` the maker of the permit sources that manifests declare. A second one
    /// replaces the first.
    pub fn set_permit_factory(&mut self, factory: Arc<dyn PermitSourceFactory>) {
        self.permit_factory = Some(factory);
    }

    /// The maker of declared permit sources, if the program has one.
    pub fn permit_factory(&self) -> Option<Arc<dyn PermitSourceFactory>> {
        self.permit_factory.clone()
    }

    /// The raster backend with this name.
    pub fn backend(&self, name: &str) -> Option<Arc<dyn RasterBackend>> {
        self.backends.get(name).cloned()
    }

    /// The names of the raster backends, sorted.
    pub fn backend_names(&self) -> impl Iterator<Item = &str> {
        self.backends.keys().map(String::as_str)
    }

    /// The adapters in their order.
    pub fn adapters(&self) -> impl Iterator<Item = &dyn RegionAdapter> {
        self.adapters.iter().map(|adapter| adapter.as_ref())
    }

    /// The adapter with this id.
    pub fn get(&self, id: &str) -> Option<&dyn RegionAdapter> {
        self.adapters()
            .find(|adapter| adapter.info().id.as_str() == id)
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use geo_types::MultiPolygon;

    use super::*;
    use crate::info::{Names, RegionInfo};

    struct Plain(RegionInfo, MultiPolygon<f64>);

    impl RegionAdapter for Plain {
        fn info(&self) -> &RegionInfo {
            &self.0
        }

        fn boundary(&self) -> &MultiPolygon<f64> {
            &self.1
        }

        fn alkis(&self) -> Option<&dyn crate::alkis::AlkisSource> {
            None
        }

        fn rasters(&self) -> Option<&dyn crate::rasters::RasterBackend> {
            None
        }

        fn attribution(
            &self,
            _used: &crate::credit::UsedSources<'_>,
        ) -> Vec<crate::credit::CreditLine> {
            Vec::new()
        }

        fn estimate(
            &self,
            _area: &crate::estimate::EstimateArea,
            _env: &crate::estimate::EstimateEnv<'_>,
        ) -> Vec<crate::records::EstimateItem> {
            Vec::new()
        }
    }

    fn plain(id: &'static str) -> Arc<dyn RegionAdapter> {
        Arc::new(Plain(
            RegionInfo {
                id: AdapterId::built_in(id),
                provider: AdapterId::built_in("provider"),
                names: Names {
                    en: id.to_string(),
                    de: id.to_string(),
                    pl: None,
                },
                crs: freemapper_georef::crs::Crs::Utm33,
                country: crate::info::Country::De,
                coverage: crate::info::Coverage::Full,
                object_id_prefix: None,
                osm: Vec::new(),
                credit_short: None,
                preview: None,
            },
            MultiPolygon(Vec::new()),
        ))
    }

    #[test]
    fn the_registry_keeps_the_order_and_finds_by_id() {
        let mut registry = Registry::new();
        assert!(registry.is_empty());
        registry.add(plain("beta")).unwrap();
        registry.add(plain("alpha")).unwrap();
        let ids: Vec<&str> = registry.adapters().map(|a| a.info().id.as_str()).collect();
        assert_eq!(ids, ["beta", "alpha"]);
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get("alpha").unwrap().info().names.en, "alpha");
        assert!(registry.get("gamma").is_none());
    }

    #[test]
    fn an_id_is_taken_once() {
        let mut registry = Registry::new();
        registry.add(plain("beta")).unwrap();
        let refused = registry.add(plain("beta")).unwrap_err();
        assert_eq!(refused.0.as_str(), "beta");
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn a_built_in_adapter_is_replaced_once_in_its_place() {
        let mut registry = Registry::new();
        registry.add(plain("beta")).unwrap();
        registry.add(plain("alpha")).unwrap();
        registry.mark_builtin();
        registry.add(plain("gamma")).unwrap();
        assert!(registry.is_builtin("beta") && !registry.is_builtin("gamma"));

        let before = registry.replace(plain("beta")).unwrap();
        assert_eq!(before.info().id.as_str(), "beta");
        let ids: Vec<&str> = registry.adapters().map(|a| a.info().id.as_str()).collect();
        assert_eq!(ids, ["beta", "alpha", "gamma"]);
        // Once replaced it is no built-in any more, and an adapter that was added later never was.
        assert!(!registry.is_builtin("beta"));
        assert_eq!(
            registry.replace(plain("beta")).err().unwrap().0.as_str(),
            "beta"
        );
        assert!(registry.replace(plain("gamma")).is_err());
        assert!(registry.replace(plain("unknown")).is_err());
    }

    struct NoFactory;

    impl crate::permits::PermitSourceFactory for NoFactory {
        fn wfs(
            &self,
            _declared: &crate::permits::DeclaredWfs,
        ) -> Box<dyn crate::permits::PermitSource> {
            unreachable!("the registry only hands the factory on")
        }
    }

    #[test]
    fn the_permit_factory_is_kept_and_a_second_one_replaces_it() {
        let mut registry = Registry::new();
        assert!(registry.permit_factory().is_none());
        let first: Arc<dyn crate::permits::PermitSourceFactory> = Arc::new(NoFactory);
        registry.set_permit_factory(first.clone());
        assert!(Arc::ptr_eq(&registry.permit_factory().unwrap(), &first));
        let second: Arc<dyn crate::permits::PermitSourceFactory> = Arc::new(NoFactory);
        registry.set_permit_factory(second.clone());
        assert!(Arc::ptr_eq(&registry.permit_factory().unwrap(), &second));
    }
}
