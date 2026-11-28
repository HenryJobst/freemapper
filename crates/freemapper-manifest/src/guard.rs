// SPDX-License-Identifier: AGPL-3.0-or-later

//! An adapter asks its declared hosts and no other: the fetcher it is handed is put behind
//! its policy, so a URL that a service's answer leads to (a `next` link, an address
//! in an index) and the target of a redirect are judged like the URLs of the manifest.

use std::path::PathBuf;
use std::sync::Arc;

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::cache::CacheDir;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::pointcloud::{
    PointCloudBackend, PointCloudInfo, PointCloudRecord, PointCloudTile,
};
use freemapper_adapter_api::policy::{PolicyFetcher, UrlPolicy};
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{
    DeriveEnv, DeriveInputs, ProviderInfo, RasterArea, RasterBackend, StoredRasters,
};
use freemapper_adapter_api::records::EstimateItem;
use freemapper_adapter_api::trust::TrustAnchors;
use freemapper_georef::crs::BoundingBox;

/// `env` with its fetcher behind `policy`, handed to `work`.
fn guarded<T>(
    env: &mut AdapterEnv<'_>,
    policy: &UrlPolicy,
    work: impl FnOnce(&mut AdapterEnv<'_>) -> T,
) -> T {
    let fetcher = PolicyFetcher::new(env.fetcher, policy);
    let mut inner = AdapterEnv {
        fetcher: &fetcher,
        regions: env.regions,
        cache: env.cache,
        mode: env.mode,
        today: env.today,
        skip_surface: env.skip_surface,
        progress: &mut *env.progress,
    };
    work(&mut inner)
}

/// An ALKIS source whose requests stay on the hosts of a policy.
pub struct GuardedAlkis {
    pub inner: Box<dyn AlkisSource>,
    pub policy: Arc<UrlPolicy>,
}

impl AlkisSource for GuardedAlkis {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        guarded(env, &self.policy, |env| self.inner.buildings(env, area))
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        guarded(env, &self.policy, |env| self.inner.land_use(env, area))
    }
}

/// A raster backend whose requests stay on the hosts of a policy.
pub struct GuardedRasters {
    pub inner: Arc<dyn RasterBackend>,
    pub policy: Arc<UrlPolicy>,
}

impl RasterBackend for GuardedRasters {
    fn provider(&self) -> &ProviderInfo {
        self.inner.provider()
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        guarded(env, &self.policy, |env| {
            self.inner.fetch(env, project, bbox)
        })
    }

    fn trust(&self) -> Option<&TrustAnchors> {
        self.inner.trust()
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        self.inner.estimate(area, env)
    }

    fn derive_inputs(
        &self,
        env: &DeriveEnv<'_>,
        stored: &StoredRasters<'_>,
        area: &RasterArea,
    ) -> DeriveInputs {
        self.inner.derive_inputs(env, stored, area)
    }
}

/// A point-cloud backend whose requests stay on the hosts of a policy.
pub struct GuardedPointCloud {
    pub inner: Box<dyn PointCloudBackend>,
    pub policy: Arc<UrlPolicy>,
}

impl PointCloudBackend for GuardedPointCloud {
    fn info(&self) -> &PointCloudInfo {
        self.inner.info()
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        guarded(env, &self.policy, |env| self.inner.fetch(env, bbox))
    }

    fn cached(&self, cache: &CacheDir, record: &PointCloudRecord) -> Option<PathBuf> {
        self.inner.cached(cache, record)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        self.inner.estimate(area, env)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::error::FetchError;
    use freemapper_adapter_api::fetch::{FetchMode, Fetcher};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_georef::crs::Crs;

    use super::*;

    /// An ALKIS source that asks one address for its buildings and another for its land use,
    /// as a service does whose answer names where to go on.
    struct Asks(&'static str, &'static str);

    impl AlkisSource for Asks {
        fn buildings(
            &self,
            env: &mut AdapterEnv<'_>,
            _: &FetchArea,
        ) -> Result<AlkisPart, AdapterError> {
            env.fetcher.get(self.0)?;
            Ok(AlkisPart {
                features: Vec::new(),
                crs: Crs::Utm33,
            })
        }

        fn land_use(
            &self,
            env: &mut AdapterEnv<'_>,
            _: &FetchArea,
        ) -> Result<AlkisPart, AdapterError> {
            env.fetcher.get(self.1)?;
            Ok(AlkisPart {
                features: Vec::new(),
                crs: Crs::Utm33,
            })
        }
    }

    /// A fetcher that can hold itself to a policy and counts how often it was asked to.
    struct Binds<'a> {
        inner: &'a MapFetcher,
        bound: &'a Cell<usize>,
    }

    impl Fetcher for Binds<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.inner.get(url)
        }

        fn bound_to(&self, policy: &UrlPolicy) -> Option<Box<dyn Fetcher + '_>> {
            assert!(policy.check_url("https://geo.example.org/x").is_ok());
            assert!(policy.check_url("https://download.geofabrik.de/x").is_err());
            self.bound.set(self.bound.get() + 1);
            Some(Box::new(Binds {
                inner: self.inner,
                bound: self.bound,
            }))
        }
    }

    /// A request to a host the manifest did not declare is not sent, whoever names the address;
    /// one to a declared host goes on, through the fetcher bound to the policy of the adapter,
    /// which judges the redirects.
    #[test]
    fn a_guarded_source_asks_declared_hosts_only() {
        let policy = Arc::new(UrlPolicy::new(["geo.example.org"]).unwrap());
        let source = GuardedAlkis {
            inner: Box::new(Asks(
                "https://geo.example.org/items?page=2",
                "https://evil.example.net/items?token=abc",
            )),
            policy: policy.clone(),
        };
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", b"{}".to_vec());
        let bound = Cell::new(0);
        let binds = Binds {
            inner: &fetcher,
            bound: &bound,
        };
        let registry = Registry::new();
        let cache = CacheDir::new("no-cache".into());
        let mut env = AdapterEnv {
            fetcher: &binds,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: chrono::NaiveDate::from_ymd_opt(2026, 11, 11).unwrap(),
            skip_surface: false,
            progress: &mut |_| {},
        };
        let area = FetchArea::new(
            BoundingBox {
                min_e: 0.0,
                min_n: 0.0,
                max_e: 1.0,
                max_n: 1.0,
            },
            Crs::Utm33,
        );
        assert!(source.buildings(&mut env, &area).is_ok());
        let refused = source.land_use(&mut env, &area).unwrap_err();
        let AdapterError::Fetch(FetchError::NotAllowed { url, reason }) = refused else {
            panic!("{refused:?}");
        };
        assert_eq!(url, "https://evil.example.net/items");
        assert_eq!(reason, "the host is not one of the declared hosts");
        assert_eq!(
            *fetcher.log.borrow(),
            ["https://geo.example.org/items?page=2"]
        );
        assert_eq!(bound.get(), 2);
    }

    /// A point-cloud backend that asks one address.
    struct AsksCloud(
        &'static str,
        freemapper_adapter_api::pointcloud::PointCloudInfo,
    );

    impl freemapper_adapter_api::pointcloud::PointCloudBackend for AsksCloud {
        fn info(&self) -> &freemapper_adapter_api::pointcloud::PointCloudInfo {
            &self.1
        }

        fn fetch(
            &self,
            env: &mut AdapterEnv<'_>,
            _: &BoundingBox,
        ) -> Result<Vec<freemapper_adapter_api::pointcloud::PointCloudTile>, AdapterError> {
            env.fetcher.get(self.0)?;
            Ok(Vec::new())
        }

        fn cached(
            &self,
            _: &freemapper_adapter_api::cache::CacheDir,
            _: &freemapper_adapter_api::pointcloud::PointCloudRecord,
        ) -> Option<std::path::PathBuf> {
            Some(std::path::PathBuf::from("a-tile.laz"))
        }

        fn estimate(&self, _: &EstimateArea, _: &EstimateEnv<'_>) -> Vec<EstimateItem> {
            vec![EstimateItem {
                what: "a tile".to_string(),
                megabytes: 1.0,
                cached: false,
                part: freemapper_adapter_api::records::Part::PointCloud,
                store: freemapper_adapter_api::records::Store::Cache,
            }]
        }
    }

    #[test]
    fn a_guarded_point_cloud_asks_declared_hosts_only() {
        use freemapper_adapter_api::pointcloud::{PointCloudBackend as _, PointCloudInfo};
        let info = PointCloudInfo {
            id: freemapper_adapter_api::records::AdapterId::built_in("sweden"),
            cache_dir: "sweden".to_string(),
            source: "x".to_string(),
            epsg: 25833,
            grid_km: 3,
            left_out: Vec::new(),
        };
        let policy = Arc::new(UrlPolicy::new(["geo.example.org"]).unwrap());
        let cloud = |url| GuardedPointCloud {
            inner: Box::new(AsksCloud(url, info.clone())),
            policy: policy.clone(),
        };
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", b"{}".to_vec());
        let registry = Registry::new();
        let cache = CacheDir::new("no-cache".into());
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: chrono::NaiveDate::from_ymd_opt(2026, 11, 11).unwrap(),
            skip_surface: false,
            progress: &mut |_| {},
        };
        let area = BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: 1.0,
            max_n: 1.0,
        };
        assert_eq!(cloud("https://geo.example.org/a").info().epsg, 25833);
        // The cache and the estimate come from the backend as they are.
        let record = freemapper_adapter_api::pointcloud::PointCloudRecord {
            tile: "t".to_string(),
            url: String::new(),
            published: None,
        };
        assert_eq!(
            cloud("https://geo.example.org/a").cached(&cache, &record),
            Some(std::path::PathBuf::from("a-tile.laz"))
        );
        let estimate = cloud("https://geo.example.org/a").estimate(
            &EstimateArea {
                bbox_25833: area,
                map_bbox: area,
                today: chrono::NaiveDate::from_ymd_opt(2026, 11, 11).unwrap(),
            },
            &EstimateEnv { cache: &cache },
        );
        assert_eq!(estimate.len(), 1);
        assert!(
            cloud("https://geo.example.org/a")
                .fetch(&mut env, &area)
                .is_ok()
        );
        let refused = cloud("https://evil.example.net/a")
            .fetch(&mut env, &area)
            .unwrap_err();
        assert!(
            matches!(refused, AdapterError::Fetch(FetchError::NotAllowed { .. })),
            "{refused:?}"
        );
    }
}
