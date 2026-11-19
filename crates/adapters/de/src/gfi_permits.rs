// SPDX-License-Identifier: AGPL-3.0-or-later

//! Permit sources that answer only at points (ADR 0139): a WMS whose GetFeatureInfo names what
//! lies at a point, without its geometry. The fetch lays a grid of cells over the map box, at
//! most [`MAX_CELLS`], asks at the centre of each cell and stores each answer with its cell as
//! geometry. The engine then measures the cells as it measures areas, so a share is an estimate
//! from the points, and what lies between two points can be missed.
//!
//! As in [`wfs_permits`](crate::wfs_permits) a feature keeps only the attributes its query
//! names.

use std::path::Path;

use freemapper_adapter_api::permits::{
    FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use geo_types::{Coord, Rect};
use serde_json::Value;

use crate::berlin_permits::geometry;
use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};

/// The most cells the fetch asks at, for every query.
pub const MAX_CELLS: usize = 25;

/// The shortest side of a cell in metres: a small map gets fewer points.
const MIN_CELL: f64 = 50.0;

/// Half the side of the box a point is asked with, in degrees (about a metre).
const POINT: f64 = 0.00001;

/// The largest answer to one point.
const MAX_BYTES: u64 = 1 << 20;

/// One GetFeatureInfo query: its key (a layer key of the source), the WMS, the layers it asks
/// at once, the attributes a feature keeps and how a stored feature becomes a finding.
pub struct Query {
    pub key: &'static str,
    pub base: &'static str,
    pub layers: &'static str,
    pub kept: &'static [&'static str],
    /// The layers do not overlap (the Reviere of a state): a point keeps its first feature
    /// only. Near a border a server answers for both sides.
    pub one: bool,
    pub read: fn(&Value) -> Option<FindingKind>,
}

/// A WMS of a state asked at points.
pub struct GfiSource {
    info: PermitSourceInfo,
    queries: &'static [Query],
}

/// A source of `queries`, one stored layer each, in the order of `keys`.
pub fn source(
    key: &'static str,
    label: Wording,
    what: Wording,
    credit: &'static str,
    keys: &'static [&'static str],
    sections: &'static [PermitSection],
    queries: &'static [Query],
) -> Box<dyn PermitSource> {
    debug_assert!(
        keys.len() == queries.len() && keys.iter().zip(queries).all(|(k, q)| *k == q.key)
    );
    Box::new(GfiSource {
        info: PermitSourceInfo {
            key,
            label,
            what,
            credit,
            note: None,
            sampled: true,
            gaps: None,
            layers: keys,
            sections,
            offices: None,
        },
        queries,
    })
}

impl PermitSource for GfiSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let cells = cells(bbox);
        self.queries
            .iter()
            .map(|query| {
                let mut stored = Vec::new();
                for cell in &cells {
                    let url = point_url(query, cell.center());
                    let features =
                        get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| {
                            read_answer(&url, bytes)
                        })?;
                    let mut kept: Vec<Value> = Vec::new();
                    for feature in &features {
                        let properties = kept_properties(feature, query.kept);
                        // A layer asked twice, or two layers with one feature, answer it twice.
                        if !kept.contains(&properties) {
                            kept.push(properties);
                        }
                        if query.one {
                            break;
                        }
                    }
                    stored.extend(kept.into_iter().map(|p| with_cell(p, cell)));
                }
                Ok(stored)
            })
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let Some(query) = self.queries.iter().find(|q| q.key == layer) else {
            return Vec::new();
        };
        features
            .iter()
            .filter_map(|feature| {
                Some(PermitFinding {
                    kind: (query.read)(feature)?,
                    geometry: geometry(feature)?,
                })
            })
            .collect()
    }
}

/// The cells over `bbox` (WGS 84, x = longitude): about square on the ground, no side shorter
/// than [`MIN_CELL`], at most [`MAX_CELLS`].
pub(crate) fn cells(bbox: &Rect<f64>) -> Vec<Rect<f64>> {
    let metres_per_degree = 111_320.0;
    let width = bbox.width() * metres_per_degree * bbox.center().y.to_radians().cos();
    let height = bbox.height() * metres_per_degree;
    let side = (width * height / MAX_CELLS as f64).sqrt().max(MIN_CELL);
    let columns = ((width / side).round() as usize).clamp(1, MAX_CELLS);
    let rows = ((height / side).round() as usize).clamp(1, (MAX_CELLS / columns).max(1));
    let (dx, dy) = (bbox.width() / columns as f64, bbox.height() / rows as f64);
    let min = bbox.min();
    (0..rows)
        .flat_map(|row| {
            (0..columns).map(move |column| {
                let corner = Coord {
                    x: min.x + column as f64 * dx,
                    y: min.y + row as f64 * dy,
                };
                Rect::new(
                    corner,
                    Coord {
                        x: corner.x + dx,
                        y: corner.y + dy,
                    },
                )
            })
        })
        .collect()
}

/// The GetFeatureInfo of `query` at `point`: a box of about two metres around it in CRS:84
/// (longitude first in WMS 1.3.0), three pixels wide, asked at its middle pixel.
fn point_url(query: &Query, point: Coord<f64>) -> String {
    format!(
        "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={layers}&QUERY_LAYERS={layers}\
         &STYLES=&CRS=CRS:84&BBOX={:.6},{:.6},{:.6},{:.6}&WIDTH=3&HEIGHT=3&I=1&J=1\
         &INFO_FORMAT=application/geojson&FEATURE_COUNT=50&FORMAT=image/png",
        query.base,
        point.x - POINT,
        point.y - POINT,
        point.x + POINT,
        point.y + POINT,
        layers = query.layers,
    )
}

/// The features of one answer, a GeoJSON FeatureCollection (an ArcGIS server writes
/// `"geometry": null`).
fn read_answer(url: &str, bytes: &[u8]) -> Result<Vec<Value>, FetchError> {
    let not_geojson = |message: &str| FetchError::Http {
        url: url.to_string(),
        message: message.to_string(),
    };
    let mut value: Value =
        serde_json::from_slice(bytes).map_err(|_| not_geojson("the answer is not GeoJSON"))?;
    match value.get_mut("features").map(Value::take) {
        Some(Value::Array(features)) => Ok(features),
        None if value.get("type").and_then(Value::as_str) == Some("FeatureCollection") => {
            Ok(Vec::new())
        }
        _ => Err(not_geojson("the answer is not a FeatureCollection")),
    }
}

/// The attributes of `feature` that `kept` names.
fn kept_properties(feature: &Value, kept: &[&str]) -> Value {
    Value::Object(
        kept.iter()
            .filter_map(|key| {
                Some((
                    key.to_string(),
                    feature.get("properties")?.get(*key)?.clone(),
                ))
            })
            .collect(),
    )
}

/// A stored feature: `properties` with `cell` as its area, longitude first.
fn with_cell(properties: Value, cell: &Rect<f64>) -> Value {
    serde_json::json!({
        "type": "Feature",
        "geometry": geojson::Geometry::new(geojson::GeometryValue::from(&cell.to_polygon())),
        "properties": properties,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use freemapper_adapter_api::permits::ForestFunctionKind;

    fn read_name(feature: &Value) -> Option<FindingKind> {
        Some(FindingKind::ForestFunction {
            kind: ForestFunctionKind::Natural,
            name: feature["properties"]["NAME"].as_str().map(str::to_string),
        })
    }

    const QUERIES: &[Query] = &[Query {
        key: "wald",
        base: "https://example.org/wms",
        layers: "1,2",
        kept: &["NAME"],
        one: false,
        read: read_name,
    }];

    fn source_of_test() -> Box<dyn PermitSource> {
        source(
            "test",
            Wording { en: "T", de: "T" },
            Wording { en: "t", de: "t" },
            "credit",
            &["wald"],
            &[PermitSection::ForestFunctions],
            QUERIES,
        )
    }

    fn bbox(w: f64, s: f64, e: f64, n: f64) -> Rect<f64> {
        Rect::new(Coord { x: w, y: s }, Coord { x: e, y: n })
    }

    #[test]
    fn a_square_box_gets_the_most_cells_and_they_tile_it() {
        // About 2.1 by 2.2 km in Saxony.
        let b = bbox(13.80, 51.05, 13.83, 51.07);
        let cells = cells(&b);
        assert_eq!(cells.len(), MAX_CELLS);
        let area: f64 = cells.iter().map(|c| c.width() * c.height()).sum();
        assert!((area - b.width() * b.height()).abs() < 1e-12);
        assert!(
            cells
                .iter()
                .all(|c| geo::Contains::contains(&b, &c.center()))
        );
    }

    #[test]
    fn a_long_box_stays_within_the_most_cells() {
        for b in [bbox(13.0, 51.0, 13.5, 51.01), bbox(13.0, 51.0, 13.01, 51.4)] {
            let n = cells(&b).len();
            assert!((10..=MAX_CELLS).contains(&n), "{n}");
        }
    }

    #[test]
    fn a_schoolyard_gets_a_few_cells() {
        // About 140 by 110 m.
        assert_eq!(cells(&bbox(13.800, 51.050, 13.802, 51.051)).len(), 6);
        assert_eq!(cells(&bbox(13.8, 51.05, 13.8001, 51.0501)).len(), 1);
    }

    #[test]
    fn each_answer_is_stored_with_its_cell_and_only_the_kept_attributes() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://example.org/wms?SERVICE=WMS",
            br#"{"type":"FeatureCollection","features":[
                {"type":"Feature","geometry":null,"properties":{"NAME":"Wodrich","OWNER":"x"},"layerName":"1"},
                {"type":"Feature","geometry":null,"properties":{"NAME":"Wodrich","OWNER":"y"},"layerName":"2"}]}"#
                .to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let b = bbox(13.8, 51.05, 13.8001, 51.0501);
        let source = source_of_test();
        let answers = source
            .fetch(&fetcher, &b, &part.path().join("p.part"))
            .unwrap();
        assert_eq!(answers.len(), 1);
        // One cell, the feature once.
        assert_eq!(answers[0].len(), 1);
        assert_eq!(
            answers[0][0]["properties"],
            serde_json::json!({"NAME": "Wodrich"})
        );
        let url = fetcher.log.borrow()[0].clone();
        assert!(url.contains("QUERY_LAYERS=1,2&"), "{url}");
        assert!(
            url.contains("CRS=CRS:84&BBOX=13.800040,51.050040,13.800060,51.050060"),
            "{url}"
        );
        let found = source.findings("wald", &answers[0]);
        assert_eq!(
            found.iter().map(|f| &f.kind).collect::<Vec<_>>(),
            [&FindingKind::ForestFunction {
                kind: ForestFunctionKind::Natural,
                name: Some("Wodrich".into())
            }]
        );
        let corners: Vec<Coord<f64>> = geo::CoordsIter::coords_iter(&found[0].geometry).collect();
        assert!(
            corners.contains(&Coord { x: 13.8, y: 51.05 }),
            "{corners:?}"
        );
    }

    #[test]
    fn a_query_of_one_feature_keeps_the_first_at_a_border() {
        const ONE: &[Query] = &[Query {
            key: "wald",
            base: "https://example.org/wms",
            layers: "1",
            kept: &["NAME"],
            one: true,
            read: read_name,
        }];
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://example.org/wms?SERVICE=WMS",
            br#"{"type":"FeatureCollection","features":[
                {"type":"Feature","geometry":null,"properties":{"NAME":"Bautzen"}},
                {"type":"Feature","geometry":null,"properties":{"NAME":"Malschwitz"}}]}"#
                .to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let source = source(
            "test",
            Wording { en: "T", de: "T" },
            Wording { en: "t", de: "t" },
            "credit",
            &["wald"],
            &[PermitSection::ForestUnits],
            ONE,
        );
        let answers = source
            .fetch(
                &fetcher,
                &bbox(13.8, 51.05, 13.8001, 51.0501),
                &part.path().join("p.part"),
            )
            .unwrap();
        assert_eq!(answers[0].len(), 1);
        assert_eq!(answers[0][0]["properties"]["NAME"], "Bautzen");
    }

    #[test]
    fn an_answer_without_features_is_empty_and_another_is_an_error() {
        let ok = read_answer("u", br#"{"type":"FeatureCollection"}"#).unwrap();
        assert!(ok.is_empty());
        assert!(read_answer("u", b"<html>").is_err());
    }
}
