// SPDX-License-Identifier: AGPL-3.0-or-later

//! Permit sources from ArcGIS REST map services (ADR 0148): a layer's `query` asked by a box in
//! WGS 84, answered as GeoJSON in WGS 84, page by page while the service says more follow
//! (`exceededTransferLimit`). The BfG's water protection areas (ADR 0142) are read so; a table
//! of layers with the readings of [`wfs_permits`](crate::wfs_permits) serves the states whose
//! services are REST only.

use std::path::Path;

use freemapper_adapter_api::permits::{DeclaredWfs, PermitFinding, PermitSource, PermitSourceInfo};
use geo_types::Rect;
use serde_json::Value;

use crate::fetch::{FetchError, Fetcher, get_limited_with_retries};
use crate::wfs_permits::{Reading, finding, kept, kept_all, reading, stored, wording};

/// Features per page; the services give at most 1000 or 2000.
pub(crate) const PAGE: usize = 1000;
const MAX_PAGES: usize = 20;
const MAX_BYTES: u64 = 64 << 20;

/// The query of `layer` of the map service `base` for what meets `bbox`, from the feature
/// `offset` on, with the attributes `out_fields`, as GeoJSON in WGS 84.
pub(crate) fn query_url(
    base: &str,
    layer: u32,
    out_fields: &str,
    bbox: &Rect<f64>,
    offset: usize,
) -> String {
    format!(
        "{base}/{layer}/query?where=1%3D1&geometry={:.6},{:.6},{:.6},{:.6}\
         &geometryType=esriGeometryEnvelope&inSR=4326&spatialRel=esriSpatialRelIntersects\
         &outFields={out_fields}&returnGeometry=true&outSR=4326&geometryPrecision=6\
         &orderByFields=OBJECTID&resultOffset={offset}&resultRecordCount={PAGE}&f=geojson",
        bbox.min().x,
        bbox.min().y,
        bbox.max().x,
        bbox.max().y,
    )
}

/// All features of `layer` that meet `bbox`, page by page.
pub(crate) fn features(
    fetcher: &dyn Fetcher,
    base: &str,
    layer: u32,
    out_fields: &str,
    bbox: &Rect<f64>,
    part: &Path,
) -> Result<Vec<Value>, FetchError> {
    let mut features = Vec::new();
    for page in 0..MAX_PAGES {
        let url = query_url(base, layer, out_fields, bbox, page * PAGE);
        let (found, more) = get_limited_with_retries(fetcher, &url, part, MAX_BYTES, |bytes| {
            read_page(&url, bytes)
        })?;
        features.extend(found);
        if !more {
            break;
        }
    }
    Ok(features)
}

/// The features of one page and whether more follow (`exceededTransferLimit`, which ArcGIS
/// writes at the top and in `properties`).
pub(crate) fn read_page(url: &str, bytes: &[u8]) -> Result<(Vec<Value>, bool), FetchError> {
    let wrong = |message: &str| FetchError::Http {
        url: url.to_string(),
        message: message.to_string(),
    };
    let mut value: Value =
        serde_json::from_slice(bytes).map_err(|_| wrong("the answer is not GeoJSON"))?;
    if let Some(error) = value.get("error") {
        return Err(wrong(&format!("the service answers an error: {error}")));
    }
    let more = [
        &value["exceededTransferLimit"],
        &value["properties"]["exceededTransferLimit"],
    ]
    .iter()
    .any(|v| v.as_bool() == Some(true));
    match value.get_mut("features").map(Value::take) {
        Some(Value::Array(features)) => Ok((features, more)),
        _ => Err(wrong("the answer is not a FeatureCollection")),
    }
}

/// One layer of a REST table: its key (a plain name), its number in the map service and its
/// reading.
pub type RestLayer = (&'static str, u32, Reading);

/// A map service of a state read by a table.
struct RestSource {
    info: PermitSourceInfo,
    base: &'static str,
    layers: &'static [RestLayer],
}

/// The source a manifest declares as `[[permits.arcgis]]` (issue #273): the map service, and
/// for each layer its number (the `type_name`) and the reading it names. The validator holds a
/// layer's number to digits, so one that is not a number is a mistake of the program.
pub(crate) fn declared_source(declared: &DeclaredWfs) -> Box<dyn PermitSource> {
    let layers: &'static [RestLayer] = kept_all(
        declared
            .layers
            .iter()
            .map(|layer| {
                (
                    kept(&layer.key),
                    layer.type_name.parse().unwrap_or(0),
                    reading(&layer.reading),
                )
            })
            .collect(),
    );
    Box::new(RestSource {
        info: PermitSourceInfo {
            key: kept(&declared.key),
            label: wording(&declared.label),
            what: wording(&declared.what),
            credit: kept(&declared.credit),
            note: declared.note.as_deref().map(kept),
            sampled: false,
            gaps: declared.gaps.as_ref().map(wording),
            layers: kept_all(declared.layers.iter().map(|l| kept(&l.key)).collect()),
            sections: kept_all(declared.sections.clone()),
            offices: None,
        },
        base: kept(&declared.url),
        layers,
    })
}

impl PermitSource for RestSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        self.layers
            .iter()
            .map(|(_, layer, reading)| {
                let out_fields = reading.kept().join(",");
                let out_fields = if out_fields.is_empty() {
                    "OBJECTID".to_string()
                } else {
                    out_fields
                };
                Ok(
                    features(fetcher, self.base, *layer, &out_fields, bbox, part)?
                        .iter()
                        .filter_map(|f| stored(f, *reading))
                        .collect(),
                )
            })
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let Some((_, _, reading)) = self.layers.iter().find(|(key, _, _)| *key == layer) else {
            return Vec::new();
        };
        features
            .iter()
            .filter_map(|feature| finding(*reading, feature))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use freemapper_adapter_api::permits::{
        FindingKind, PermitSection, ProtectedObjectKind, Wording,
    };

    const BASE: &str = "https://example.org/arcgis/rest/services/Natur/MapServer";
    const LAYERS: &[RestLayer] = &[(
        "naturdenkmale",
        7,
        Reading::Object {
            kind: ProtectedObjectKind::NaturalMonument,
            name: Some("name"),
            citation: None,
            only: None,
            point_names: &[],
        },
    )];

    #[test]
    fn a_rest_table_pages_and_keeps_what_it_reads() {
        let fetcher = MapFetcher::default();
        let bbox = Rect::new(
            geo_types::Coord { x: 9.9, y: 51.52 },
            geo_types::Coord { x: 9.98, y: 51.56 },
        );
        let page = |name: &str, more: bool| {
            serde_json::to_vec(&serde_json::json!({"type": "FeatureCollection",
                "exceededTransferLimit": more,
                "features": [{"type": "Feature",
                    "properties": {"name": name, "vollzug": "Stadt Göttingen"},
                    "geometry": {"type": "MultiPoint", "coordinates": [[9.95, 51.54]]}}]}))
            .unwrap()
        };
        fetcher.insert(
            &query_url(BASE, 7, "name", &bbox, 0),
            page("Liasscholle", true),
        );
        fetcher.insert(
            &query_url(BASE, 7, "name", &bbox, PAGE),
            page("Wall", false),
        );
        let source = RestSource {
            info: PermitSourceInfo {
                key: "natur",
                label: Wording { en: "N", de: "N" },
                what: Wording { en: "n", de: "n" },
                credit: "dl-de/by-2-0",
                note: None,
                sampled: false,
                gaps: None,
                layers: &["naturdenkmale"],
                sections: &[PermitSection::ProtectedObjects],
                offices: None,
            },
            base: BASE,
            layers: LAYERS,
        };
        let part = tempfile::tempdir().unwrap();
        let answers = source
            .fetch(&fetcher, &bbox, &part.path().join("p.part"))
            .unwrap();
        assert_eq!(answers[0].len(), 2);
        assert!(answers[0][0]["properties"].get("vollzug").is_none());
        let names: Vec<Option<String>> = source
            .findings("naturdenkmale", &answers[0])
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::ProtectedObject { name, .. } => name,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(names, [Some("Liasscholle".into()), Some("Wall".into())]);
    }

    #[test]
    fn an_error_answer_is_an_error() {
        assert!(read_page("u", br#"{"error":{"code":400,"message":"x"}}"#).is_err());
        assert!(read_page("u", b"<html>").is_err());
    }
}
