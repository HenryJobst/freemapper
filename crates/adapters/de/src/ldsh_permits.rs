// SPDX-License-Identifier: AGPL-3.0-or-later

//! The monuments of Schleswig-Holstein (ADR 0145): the Landesamt für Denkmalpflege publishes its
//! list of monuments only as one GeoJSON file for the whole state (about 52 MB, EPSG:25832, a new
//! one each week, CC BY 4.0). The file is kept in the shared download cache as the statewide
//! files of ADR 0132 are, downloaded again only when the server has a newer one, and cut to the
//! map box; a monument keeps only its kind (`LayerName`).
//!
//! The archaeological monuments are not in it: their office allows their publication only with
//! its written consent. Lübeck keeps its own list.

use std::path::Path;

use freemapper_adapter_api::permits::{
    PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use geo::BoundingRect as _;
use geo_types::{Geometry, Rect};
use serde::Deserialize;
use serde_json::Value;
use serde_json::value::RawValue;

use crate::crs::Crs;
use crate::fetch::{CacheDir, FetchError, Fetcher, get_limited_with_retries};
use crate::shape_permits::current_file;
use crate::wfs_permits::{Reading, finding, in_wgs84, projected_box, stored};

/// The current file; the server sends it on to the file of the week.
pub const URL: &str =
    "https://opendata.schleswig-holstein.de/collection/geodaten-denkmalliste-sh/aktuell.geojson";

/// The largest file read; the list of 2026 has 52 MB.
const MAX_FILE: u64 = 150 * 1024 * 1024;

const READING: Reading = Reading::Monument {
    kind: "Denkmal",
    kind_from: Some("LayerName"),
    link: None,
};

/// The source of Schleswig-Holstein.
pub fn source() -> Box<dyn PermitSource> {
    Box::new(LdshSource {
        info: PermitSourceInfo {
            key: "denkmale",
            label: Wording {
                en: "Landesamt für Denkmalpflege Schleswig-Holstein",
                de: "Landesamt für Denkmalpflege Schleswig-Holstein",
            },
            what: Wording {
                en: "list of monuments (one file for the state, kept in the cache)",
                de: "Denkmalliste (eine Datei für das Bundesland, im Zwischenspeicher)",
            },
            credit: "Landesamt für Denkmalpflege Schleswig-Holstein; Lizenz: Creative Commons Namensnennung 4.0 International (CC BY 4.0)",
            note: None,
            sampled: false,
            gaps: Some(Wording {
                en: "the archaeological monuments are not in it, and the Hansestadt Lübeck keeps its own list",
                de: "Die Bodendenkmale fehlen darin, und die Hansestadt Lübeck führt ihre Denkmalliste selbst",
            }),
            layers: &["denkmale"],
            sections: &[PermitSection::Monuments],
            offices: None,
        },
    })
}

struct LdshSource {
    info: PermitSourceInfo,
}

impl PermitSource for LdshSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let features = get_limited_with_retries(fetcher, URL, part, MAX_FILE, |bytes| {
            monuments_in(bytes, bbox)
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
            URL,
            part,
            &dir,
            "geojson",
            MAX_FILE,
            looks_like_geojson,
        )?;
        Ok(vec![monuments_in(&bytes, bbox)?])
    }

    fn findings(&self, _layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        features
            .iter()
            .filter_map(|feature| finding(READING, feature))
            .collect()
    }
}

/// Whether `bytes` look like a GeoJSON FeatureCollection, the check before the file is kept:
/// an object that names the type (a maintenance page is HTML).
fn looks_like_geojson(bytes: &[u8]) -> Result<(), String> {
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

/// The monuments of the file whose box meets `bbox` (WGS 84, x = longitude), as stored
/// features in WGS 84 with their kind.
fn monuments_in(bytes: &[u8], bbox: &Rect<f64>) -> Result<Vec<Value>, FetchError> {
    let wrong = |message: String| FetchError::Http {
        url: URL.to_string(),
        message,
    };
    let collection: Collection<'_> = serde_json::from_slice(bytes)
        .map_err(|e| wrong(format!("the list of monuments is not GeoJSON: {e}")))?;
    let cut = projected_box(Crs::Utm32, bbox);
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
        if let Some(stored) = in_wgs84(&projected, Crs::Utm32).and_then(|f| stored(&f, READING)) {
            kept.push(stored);
        }
    }
    Ok(kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::fetch::test_support::MapFetcher;
    use freemapper_adapter_api::permits::FindingKind;

    /// A file of three monuments in EPSG:25832: two in Kiel, one far away.
    fn file() -> Vec<u8> {
        let at = |lat: f64, lon: f64| {
            let p = Crs::Utm32.forward(LatLon { lat, lon });
            [p.easting, p.northing]
        };
        let square = |[e, n]: [f64; 2]| {
            serde_json::json!({"type": "MultiPolygon", "coordinates": [[[
                [e, n], [e + 10.0, n], [e + 10.0, n + 10.0], [e, n + 10.0], [e, n]]]]})
        };
        serde_json::to_vec(&serde_json::json!({
            "type": "FeatureCollection",
            "crs": {"type": "name", "properties": {"name": "urn:ogc:def:crs:EPSG::25832"}},
            "features": [
                {"type": "Feature", "properties": {"LayerName": "Baudenkmal", "Strasse": "x"},
                 "geometry": square(at(54.3230, 10.1390))},
                {"type": "Feature", "properties": {"LayerName": "Gründenkmal"},
                 "geometry": square(at(54.3235, 10.1395))},
                {"type": "Feature", "properties": {"LayerName": "Baudenkmal"},
                 "geometry": square(at(54.78, 9.43))},
            ]
        }))
        .unwrap()
    }

    #[test]
    fn the_file_is_cut_to_the_map_and_kept_in_the_cache() {
        let fetcher = MapFetcher::default();
        fetcher.insert(URL, file());
        let cache_root = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(cache_root.path().to_path_buf());
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            geo_types::Coord { x: 10.13, y: 54.32 },
            geo_types::Coord { x: 10.15, y: 54.33 },
        );
        let source = source();
        let answers = source
            .fetch_cached(&fetcher, &bbox, &part.path().join("p.part"), &cache)
            .unwrap();
        assert_eq!(answers[0].len(), 2);
        assert!(answers[0][0]["properties"].get("Strasse").is_none());
        let kinds: Vec<FindingKind> = source
            .findings("denkmale", &answers[0])
            .into_iter()
            .map(|f| f.kind)
            .collect();
        assert_eq!(
            kinds,
            ["Baudenkmal", "Gründenkmal"].map(|kind| FindingKind::Monument {
                kind: kind.into(),
                link: None
            })
        );
        // Kept: a second run with the server down reads the copy.
        let down = MapFetcher::default();
        down.failing.borrow_mut().push(URL.to_string());
        let again = source
            .fetch_cached(&down, &bbox, &part.path().join("p.part"), &cache)
            .unwrap();
        assert_eq!(again[0].len(), 2);
    }

    #[test]
    fn an_answer_that_is_no_geojson_is_not_kept() {
        assert!(looks_like_geojson(b"<html>maintenance</html>").is_err());
        assert!(looks_like_geojson(br#"{"type": "FeatureCollection", "features": []}"#).is_ok());
    }
}
