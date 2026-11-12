//! An adapter from a folder asks its declared hosts and no other: the fetcher it is handed is
//! wrapped, so a URL that a service's answer leads to (a `next` link, an address in an index)
//! is judged like the URLs of the manifest.

use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::fetch::{ByteRange, Fetcher};
use freemapper_adapter_api::note::Note;
use freemapper_adapter_api::policy::{PolicyError, UrlPolicy};
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{
    DeriveEnv, DeriveInputs, ProviderInfo, RasterArea, RasterBackend, StoredRasters,
};
use freemapper_adapter_api::records::EstimateItem;
use freemapper_adapter_api::trust::TrustAnchors;
use freemapper_georef::crs::BoundingBox;

/// A fetcher that sends a request on only when its URL passes `policy`.
pub struct PolicyFetcher<'a> {
    pub inner: &'a dyn Fetcher,
    pub policy: &'a UrlPolicy,
}

impl PolicyFetcher<'_> {
    fn check(&self, url: &str) -> Result<(), FetchError> {
        self.policy
            .check_url(url)
            .map_err(|refused: PolicyError| FetchError::NotAllowed {
                url: refused.subject,
                reason: refused.reason.to_string(),
            })
    }
}

impl Fetcher for PolicyFetcher<'_> {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.check(url)?;
        self.inner.get(url)
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        self.check(url)?;
        self.inner.last_modified(url)
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        self.check(url)?;
        self.inner.get_range(url, range)
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        self.check(url)?;
        self.inner.download_to(url, path, max_bytes, progress)
    }
}

/// `env` with its fetcher behind `policy`, handed to `work`.
fn guarded<T>(
    env: &mut AdapterEnv<'_>,
    policy: &UrlPolicy,
    work: impl FnOnce(&mut AdapterEnv<'_>) -> T,
) -> T {
    let fetcher = PolicyFetcher {
        inner: env.fetcher,
        policy,
    };
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

#[cfg(test)]
mod tests {
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::fetch::FetchMode;
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

    /// A request to a host the manifest did not declare is not sent, whoever names the address;
    /// one to a declared host goes on to the fetcher.
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
        // Every way to ask is judged.
        let guarded = PolicyFetcher {
            inner: &fetcher,
            policy: &policy,
        };
        let evil = "https://evil.example.net/x";
        assert!(matches!(
            guarded.last_modified(evil),
            Err(FetchError::NotAllowed { .. })
        ));
        assert!(matches!(
            guarded.get_range(evil, ByteRange::Last(4)),
            Err(FetchError::NotAllowed { .. })
        ));
        assert!(matches!(
            guarded.download_to(evil, Path::new("nowhere"), 10, &mut |_| {}),
            Err(FetchError::NotAllowed { .. })
        ));
        assert_eq!(fetcher.log.borrow().len(), 1);
    }
}
