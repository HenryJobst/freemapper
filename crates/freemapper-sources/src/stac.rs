// SPDX-License-Identifier: AGPL-3.0-or-later

//! A STAC API (SpatioTemporal Asset Catalog): the items of a collection under a box, with their
//! assets. Lantmäteriet publishes its height, vector and image products this way. The reader
//! takes the address of the API from the manifest of the adapter and builds no other address:
//! an asset is downloaded from the `href` the catalogue gives.

use std::collections::BTreeMap;

use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::fetch::Fetcher;
use serde_json::Value;

/// The most pages one search follows. The answer has no total, so a catalogue that links its
/// pages in a loop would never end.
const MAX_PAGES: usize = 20;

/// One file of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StacAsset {
    pub href: String,
    /// `file:size`, where the catalogue tells it.
    pub size: Option<u64>,
    pub media_type: Option<String>,
}

/// One item of a collection: a tile, a municipality, a point cloud.
#[derive(Debug, Clone, PartialEq)]
pub struct StacItem {
    pub id: String,
    pub collection: String,
    /// `properties.datetime`.
    pub datetime: Option<String>,
    /// `properties["proj:bbox"]`, in the CRS of the item; not every collection has it.
    pub bbox_proj: Option<[f64; 4]>,
    pub assets: BTreeMap<String, StacAsset>,
}

fn http(url: &str, message: String) -> FetchError {
    FetchError::Http {
        url: url.to_string(),
        message,
    }
}

fn four(value: Option<&Value>) -> Option<[f64; 4]> {
    let numbers: Vec<f64> = value?
        .as_array()?
        .iter()
        .filter_map(Value::as_f64)
        .collect();
    <[f64; 4]>::try_from(numbers).ok()
}

fn item_of(feature: &Value) -> Option<StacItem> {
    let properties = feature.get("properties");
    let assets = feature
        .get("assets")
        .and_then(Value::as_object)
        .map(|assets| {
            assets
                .iter()
                .filter_map(|(name, asset)| {
                    Some((
                        name.clone(),
                        StacAsset {
                            href: asset.get("href")?.as_str()?.to_string(),
                            size: asset.get("file:size").and_then(Value::as_u64),
                            media_type: asset
                                .get("type")
                                .and_then(Value::as_str)
                                .map(str::to_string),
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    Some(StacItem {
        id: feature.get("id")?.as_str()?.to_string(),
        collection: feature
            .get("collection")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        datetime: properties
            .and_then(|p| p.get("datetime"))
            .and_then(Value::as_str)
            .map(str::to_string),
        bbox_proj: four(properties.and_then(|p| p.get("proj:bbox"))),
        assets,
    })
}

/// The items of `collection` under `bbox_wgs84` (west, south, east, north in degrees), asked
/// of the STAC API `base` (no trailing slash), `limit` to a page. The pages are followed by
/// their `next` links, at most `MAX_PAGES`.
pub fn search(
    fetcher: &dyn Fetcher,
    base: &str,
    collection: &str,
    bbox_wgs84: [f64; 4],
    limit: u32,
) -> Result<Vec<StacItem>, FetchError> {
    if collection.is_empty()
        || !collection
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(http(
            base,
            format!("`{collection}` is not the name of a collection"),
        ));
    }
    let [west, south, east, north] = bbox_wgs84;
    let mut url = format!(
        "{base}/search?collections={collection}&bbox={west:.6},{south:.6},{east:.6},{north:.6}&limit={limit}"
    );
    let mut items = Vec::new();
    for _ in 0..MAX_PAGES {
        let body = fetcher.get(&url)?;
        let answer: Value = serde_json::from_slice(&body)
            .map_err(|e| http(&url, format!("the answer is no JSON: {e}")))?;
        let Some(features) = answer.get("features").and_then(Value::as_array) else {
            return Err(http(
                &url,
                format!("the answer for the collection `{collection}` has no features"),
            ));
        };
        items.extend(features.iter().filter_map(item_of));
        let next = answer
            .get("links")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|link| link.get("rel").and_then(Value::as_str) == Some("next"))
            .and_then(|link| link.get("href")?.as_str());
        match next {
            Some(next) => url = next.to_string(),
            None => return Ok(items),
        }
    }
    Err(http(
        &url,
        format!("the search for `{collection}` has more than {MAX_PAGES} pages"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::testing::MapFetcher;

    const BASE: &str = "https://stac.example.org/stac-hojd/v1";
    const BBOX: [f64; 4] = [13.0, 55.6, 13.3, 55.8];

    fn first_url() -> String {
        format!(
            "{BASE}/search?collections=dtm-cog&bbox=13.000000,55.600000,13.300000,55.800000&limit=100"
        )
    }

    fn page(items: &[&str], next: Option<&str>) -> Vec<u8> {
        let links = next.map_or(String::new(), |href| {
            format!(r#",{{"rel":"next","href":"{href}"}}"#)
        });
        format!(
            r#"{{"type":"FeatureCollection","features":[{}],"links":[{{"rel":"self","href":"x"}}{links}]}}"#,
            items.join(",")
        )
        .into_bytes()
    }

    fn item(id: &str) -> String {
        format!(
            r#"{{"id":"{id}","collection":"dtm-cog","bbox":[13.08,55.66,13.25,55.75],
            "properties":{{"datetime":"2025-09-29","proj:bbox":[380000,6170000,390000,6180000]}},
            "assets":{{"data":{{"href":"https://dl.example.org/{id}.tif","type":"image/tiff","file:size":298640561}},
            "info":{{"href":"https://dl.example.org/{id}_info.json"}}}}}}"#
        )
    }

    #[test]
    fn search_reads_items_with_their_assets_and_projected_box() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&first_url(), page(&[&item("617_38")], None));
        let items = search(&fetcher, BASE, "dtm-cog", BBOX, 100).unwrap();
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.id, "617_38");
        assert_eq!(item.collection, "dtm-cog");
        assert_eq!(item.datetime.as_deref(), Some("2025-09-29"));
        assert_eq!(
            item.bbox_proj,
            Some([380000.0, 6170000.0, 390000.0, 6180000.0])
        );
        let data = &item.assets["data"];
        assert_eq!(data.href, "https://dl.example.org/617_38.tif");
        assert_eq!(data.size, Some(298_640_561));
        assert_eq!(data.media_type.as_deref(), Some("image/tiff"));
        assert_eq!(item.assets["info"].size, None);
    }

    #[test]
    fn search_follows_next_links_until_there_is_none() {
        let fetcher = MapFetcher::default();
        let second = format!("{BASE}/search?page=2");
        fetcher.insert(&first_url(), page(&[&item("617_38")], Some(&second)));
        fetcher.insert(&second, page(&[&item("617_39")], None));
        let items = search(&fetcher, BASE, "dtm-cog", BBOX, 100).unwrap();
        let ids: Vec<&str> = items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["617_38", "617_39"]);
    }

    #[test]
    fn a_loop_of_next_links_stops_after_twenty_pages() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&first_url(), page(&[&item("617_38")], Some(&first_url())));
        let error = search(&fetcher, BASE, "dtm-cog", BBOX, 100).unwrap_err();
        assert!(error.to_string().contains("more than 20 pages"), "{error}");
    }

    #[test]
    fn an_item_without_assets_is_kept_with_none() {
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &first_url(),
            page(
                &[r#"{"id":"a","collection":"dtm-cog","properties":{}}"#],
                None,
            ),
        );
        let items = search(&fetcher, BASE, "dtm-cog", BBOX, 100).unwrap();
        assert!(items[0].assets.is_empty());
        assert_eq!(items[0].bbox_proj, None);
    }

    #[test]
    fn an_answer_that_is_no_feature_collection_names_the_collection() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&first_url(), br#"{"code":"NotFound"}"#.to_vec());
        let error = search(&fetcher, BASE, "dtm-cog", BBOX, 100).unwrap_err();
        assert!(error.to_string().contains("dtm-cog"), "{error}");
    }

    #[test]
    fn a_collection_with_odd_characters_is_refused_before_a_request() {
        let fetcher = MapFetcher::default();
        let error = search(&fetcher, BASE, "dtm cog&x=1", BBOX, 100).unwrap_err();
        assert!(error.to_string().contains("collection"), "{error}");
        assert_eq!(fetcher.calls.get(), 0);
    }
}
