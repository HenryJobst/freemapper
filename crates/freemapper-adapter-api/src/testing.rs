// SPDX-License-Identifier: AGPL-3.0-or-later

//! A fetcher for tests: it answers from memory. Behind the feature `testing`, which the
//! dev-dependencies of the crates that fetch turn on.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::error::FetchError;
use crate::fetch::{Fetcher, status_answer_error};

/// Serves fixed responses by URL and counts the requests.
#[derive(Default)]
pub struct MapFetcher {
    pub responses: RefCell<HashMap<String, Vec<u8>>>,
    /// Answers for URLs starting with the key, when no exact response matches.
    pub prefixes: RefCell<HashMap<String, Vec<u8>>>,
    /// `Last-Modified` dates by URL.
    pub modified: RefCell<HashMap<String, chrono::NaiveDate>>,
    pub calls: Cell<usize>,
    /// URLs that fail with a server error.
    pub failing: RefCell<Vec<String>>,
    /// URL prefixes the server refuses with HTTP 400.
    pub refusing: RefCell<Vec<String>>,
    /// The bodies of the refusals, by URL prefix (none: an empty body).
    pub refusal_bodies: RefCell<HashMap<String, Vec<u8>>>,
    /// URL prefixes that fail with a reset this many more times before they answer.
    pub flaky: RefCell<HashMap<String, u32>>,
    /// Every URL requested with `get`, in order.
    pub log: RefCell<Vec<String>>,
}

impl MapFetcher {
    pub fn insert(&self, url: &str, bytes: Vec<u8>) {
        self.responses.borrow_mut().insert(url.to_string(), bytes);
    }

    pub fn insert_prefix(&self, prefix: &str, bytes: Vec<u8>) {
        self.prefixes.borrow_mut().insert(prefix.to_string(), bytes);
    }

    fn fail_if_failing(&self, url: &str) -> Result<(), FetchError> {
        if self
            .refusing
            .borrow()
            .iter()
            .any(|p| url.starts_with(p.as_str()))
        {
            let body = self
                .refusal_bodies
                .borrow()
                .iter()
                .find(|(p, _)| url.starts_with(p.as_str()))
                .map(|(_, body)| body.clone())
                .unwrap_or_default();
            return Err(status_answer_error(url, 400, &body));
        }
        if let Some(left) = self
            .flaky
            .borrow_mut()
            .iter_mut()
            .find(|(p, left)| url.starts_with(p.as_str()) && **left > 0)
            .map(|(_, left)| left)
        {
            *left -= 1;
            return Err(FetchError::Http {
                url: url.to_string(),
                message: "Connection reset by peer".to_string(),
            });
        }
        if self.failing.borrow().iter().any(|u| u == url) {
            return Err(FetchError::Http {
                url: url.to_string(),
                message: "503 Service Unavailable".to_string(),
            });
        }
        Ok(())
    }

    pub fn set_modified(&self, url: &str, date: chrono::NaiveDate) {
        self.modified.borrow_mut().insert(url.to_string(), date);
    }
}

impl Fetcher for MapFetcher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.calls.set(self.calls.get() + 1);
        self.log.borrow_mut().push(url.to_string());
        self.fail_if_failing(url)?;
        let by_prefix = || {
            let prefixes = self.prefixes.borrow();
            prefixes
                .iter()
                .filter(|(prefix, _)| url.starts_with(prefix.as_str()))
                .max_by_key(|(prefix, _)| prefix.len())
                .map(|(_, bytes)| bytes.clone())
        };
        self.responses
            .borrow()
            .get(url)
            .cloned()
            .or_else(by_prefix)
            .ok_or_else(|| FetchError::NotFound(url.to_string()))
    }

    fn last_modified(&self, url: &str) -> Result<Option<chrono::NaiveDate>, FetchError> {
        self.calls.set(self.calls.get() + 1);
        self.fail_if_failing(url)?;
        if !self.responses.borrow().contains_key(url) {
            return Err(FetchError::NotFound(url.to_string()));
        }
        Ok(self.modified.borrow().get(url).copied())
    }
}

/// A region adapter without data, for the tests of the engine: a boundary, a CRS rule and nothing
/// else. It has no ALKIS and no rasters.
pub struct StubRegion {
    info: crate::info::RegionInfo,
    boundary: geo_types::MultiPolygon<f64>,
    zone_by_position: bool,
}

impl StubRegion {
    /// A region `id` with `boundary` (EPSG:25833). With `zone_by_position` its maps take the zone of
    /// their centre ([`freemapper_georef::crs::Crs::for_position`]), otherwise `crs`.
    pub fn new(
        id: &'static str,
        crs: freemapper_georef::crs::Crs,
        boundary: geo_types::MultiPolygon<f64>,
        zone_by_position: bool,
    ) -> Self {
        let id = crate::records::AdapterId::built_in(id);
        Self {
            info: crate::info::RegionInfo {
                id: id.clone(),
                provider: id,
                names: crate::info::Names {
                    en: "Stub".into(),
                    de: "Stub".into(),
                    pl: None,
                    cs: None,
                },
                crs,
                country: crate::info::Country::De,
                coverage: crate::info::Coverage::Full,
                object_id_prefix: None,
                osm: Vec::new(),
                credit_short: None,
                preview: None,
            },
            boundary,
            zone_by_position,
        }
    }
}

impl crate::adapter::RegionAdapter for StubRegion {
    fn info(&self) -> &crate::info::RegionInfo {
        &self.info
    }

    fn boundary(&self) -> &geo_types::MultiPolygon<f64> {
        &self.boundary
    }

    fn alkis(&self) -> Option<&dyn crate::alkis::AlkisSource> {
        None
    }

    fn attribution(
        &self,
        _used: &crate::credit::UsedSources<'_>,
    ) -> Vec<crate::credit::CreditLine> {
        Vec::new()
    }

    fn rasters(&self) -> Option<&dyn crate::rasters::RasterBackend> {
        None
    }

    fn estimate(
        &self,
        _area: &crate::estimate::EstimateArea,
        _env: &crate::estimate::EstimateEnv<'_>,
    ) -> Vec<crate::records::EstimateItem> {
        Vec::new()
    }

    fn crs_for(&self, centre: freemapper_georef::crs::LatLon) -> freemapper_georef::crs::Crs {
        if self.zone_by_position {
            freemapper_georef::crs::Crs::for_position(centre)
        } else {
            self.info.crs
        }
    }
}
