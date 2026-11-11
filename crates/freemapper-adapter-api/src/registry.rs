//! The adapters a program knows, in the order the region functions go through them.

use std::sync::Arc;

use crate::adapter::RegionAdapter;
use crate::records::AdapterId;

/// The list of region adapters. The order decides ties: where two regions both hold a point,
/// the earlier one is its raster region. The registry knows no adapter itself; the program
/// that starts fills it.
#[derive(Default, Clone)]
pub struct Registry {
    adapters: Vec<Arc<dyn RegionAdapter>>,
}

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

        fn attribution(
            &self,
            _used: &crate::credit::UsedSources<'_>,
        ) -> Vec<crate::credit::CreditLine> {
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
                },
                crs: freemapper_georef::crs::Crs::Utm33,
                object_id_prefix: None,
                osm: Vec::new(),
                credit_short: None,
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
}
