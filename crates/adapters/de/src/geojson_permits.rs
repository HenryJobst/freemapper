// SPDX-License-Identifier: AGPL-3.0-or-later

//! A permit source that is one GeoJSON file for a whole state (ADR 0145, issue #273): the file
//! is kept in the shared download cache as the statewide files of ADR 0132 are, downloaded
//! again only when the server has a newer one, and cut to the map box; a feature keeps only the
//! attributes its reading names. The Landesamt für Denkmalpflege of Schleswig-Holstein
//! publishes its list of monuments this way (about 52 MB, EPSG:25832, a new file each week).

use std::path::Path;

use freemapper_adapter_api::permits::{DeclaredWfs, PermitFinding, PermitSource, PermitSourceInfo};
use geo::BoundingRect as _;
use geo_types::{Geometry, Rect};
use serde::Deserialize;
use serde_json::Value;
use serde_json::value::RawValue;

use crate::crs::Crs;
use crate::fetch::{CacheDir, FetchError, Fetcher, get_limited_with_retries};
use crate::shape_permits::current_file;
use crate::wfs_permits::{
    Reading, finding, in_wgs84, kept, kept_all, projected_box, reading, stored, wording,
};

/// The largest file read; the list of 2026 has 52 MB.
const MAX_FILE: u64 = 150 * 1024 * 1024;

/// A GeoJSON file of a state read by one [`Reading`], as the table sources of WFS are.
pub struct GeoJsonSource {
    info: PermitSourceInfo,
    url: &'static str,
    crs: Crs,
    reading: Reading,
}

/// The source a manifest declares as `[[permits.geojson]]` (issue #273): the file, its CRS and
/// its one layer with the reading it names. The validator holds a declaration to one layer and
/// to a CRS of Germany, so a declaration that does not is a mistake of the program.
pub(crate) fn declared_geojson_source(declared: &DeclaredWfs) -> Box<dyn PermitSource> {
    let layer = &declared.layers[0];
    Box::new(GeoJsonSource {
        info: PermitSourceInfo {
            key: kept(&declared.key),
            label: wording(&declared.label),
            what: wording(&declared.what),
            credit: kept(&declared.credit),
            note: declared.note.as_deref().map(kept),
            sampled: false,
            gaps: declared.gaps.as_ref().map(wording),
            layers: kept_all(vec![kept(&layer.key)]),
            sections: kept_all(declared.sections.clone()),
            offices: None,
        },
        url: kept(&declared.url),
        crs: declared
            .native_crs
            .and_then(Crs::from_epsg)
            .unwrap_or(Crs::Utm32),
        reading: reading(&layer.reading),
    })
}

impl PermitSource for GeoJsonSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let features = get_limited_with_retries(fetcher, self.url, part, MAX_FILE, |bytes| {
            self.features_in(bytes, bbox)
        })?;
        Ok(vec![features])
    }

    fn fetch_cached(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
        cache: &CacheDir,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let dir = cache.provider_dir("permits");
        let bytes = current_file(
            fetcher,
            self.url,
            part,
            &dir,
            "geojson",
            MAX_FILE,
            looks_like_geojson,
        )?;
        Ok(vec![self.features_in(&bytes, bbox)?])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != self.info.layers[0] {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|feature| finding(self.reading, feature))
            .collect()
    }
}

/// Whether `bytes` look like a GeoJSON FeatureCollection, the check before the file is kept:
/// an object that names the type (a maintenance page is HTML).
pub(crate) fn looks_like_geojson(bytes: &[u8]) -> Result<(), String> {
    let text = bytes.trim_ascii();
    let named = text
        .windows(b"FeatureCollection".len())
        .any(|w| w == b"FeatureCollection");
    if text.starts_with(b"{") && text.ends_with(b"}") && named {
        Ok(())
    } else {
        Err("the answer is not a GeoJSON FeatureCollection".to_string())
    }
}

#[derive(Deserialize)]
struct Collection<'a> {
    #[serde(borrow)]
    features: Vec<Feature<'a>>,
}

#[derive(Deserialize)]
struct Feature<'a> {
    #[serde(default)]
    properties: serde_json::Map<String, Value>,
    #[serde(borrow)]
    geometry: Option<&'a RawValue>,
}

impl GeoJsonSource {
    /// The features of the file whose box meets `bbox` (WGS 84, x = longitude), as stored
    /// features in WGS 84 with the attributes of the reading.
    fn features_in(&self, bytes: &[u8], bbox: &Rect<f64>) -> Result<Vec<Value>, FetchError> {
        let wrong = |message: String| FetchError::Http {
            url: self.url.to_string(),
            message,
        };
        let collection: Collection<'_> = serde_json::from_slice(bytes)
            .map_err(|e| wrong(format!("the file is not GeoJSON: {e}")))?;
        let cut = projected_box(self.crs, bbox);
        let mut kept = Vec::new();
        for feature in collection.features {
            let Some(raw) = feature.geometry else {
                continue;
            };
            let Ok(geometry) = serde_json::from_str::<geojson::Geometry>(raw.get()) else {
                continue;
            };
            let Ok(shape) = Geometry::<f64>::try_from(geometry) else {
                continue;
            };
            let Some(r) = shape.bounding_rect() else {
                continue;
            };
            if r.max().x < cut.min_x
                || r.min().x > cut.max_x
                || r.max().y < cut.min_y
                || r.min().y > cut.max_y
            {
                continue;
            }
            let projected = serde_json::json!({
                "type": "Feature",
                "geometry": geojson::Geometry::new(geojson::GeometryValue::from(&shape)),
                "properties": feature.properties,
            });
            if let Some(stored) =
                in_wgs84(&projected, self.crs).and_then(|f| stored(&f, self.reading))
            {
                kept.push(stored);
            }
        }
        Ok(kept)
    }
}
