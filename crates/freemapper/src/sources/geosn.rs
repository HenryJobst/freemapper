//! GeoSN Saxony open data (dl-de/by-2-0, source note "GeoSN"): 2 km tiles of the terrain and
//! surface models, their currency and download links from the ArcGIS tile index.

use chrono::NaiveDate;
use serde_json::Value;

use crate::sources::lgb::Product;
use crate::tiles::TileId;

/// The ArcGIS tile index with one layer per product (`Kachel`, `Download`, `Stand`).
pub const INDEX_URL: &str = "https://geodienste.sachsen.de/ags-relay/ArcGISServer/guest/arcgis/rest/services/geosn/rest_geosn_downloadlinks/MapServer";

/// One row of the tile index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexRow {
    pub tile: TileId,
    /// The latest date of `Stand`.
    pub currency: NaiveDate,
    pub url: String,
}

/// Query of the index layer of `product` for the rows of `tiles` (2 km tiles).
pub fn index_url(product: Product, tiles: &[TileId]) -> String {
    let layer = match product {
        Product::SaxonyDsm => 4,
        _ => 6,
    };
    let km = product.grid_km();
    let bounds = tiles.iter().map(|t| t.bounds_km(km));
    let fold = |f: fn(f64, f64) -> f64, pick: fn(&crate::crs::BoundingBox) -> f64, start: f64| {
        bounds.clone().map(|b| pick(&b)).fold(start, f)
    };
    // Shrunk by 1 m: the index also returns tiles that only touch the envelope's edge.
    let min_e = fold(f64::min, |b| b.min_e, f64::INFINITY) + 1.0;
    let min_n = fold(f64::min, |b| b.min_n, f64::INFINITY) + 1.0;
    let max_e = fold(f64::max, |b| b.max_e, f64::NEG_INFINITY) - 1.0;
    let max_n = fold(f64::max, |b| b.max_n, f64::NEG_INFINITY) - 1.0;
    format!(
        "{INDEX_URL}/{layer}/query?f=json&geometry={min_e},{min_n},{max_e},{max_n}\
         &geometryType=esriGeometryEnvelope&inSR=25833&spatialRel=esriSpatialRelIntersects\
         &outFields=Kachel,Download,Stand&returnGeometry=false"
    )
}

/// The latest `YYYY-MM-DD` date in a `Stand` value such as `2024-11-27, 2024-11-30 bis 2024-12-28`.
pub fn latest_date(stand: &str) -> Option<NaiveDate> {
    stand
        .char_indices()
        .filter_map(|(i, _)| stand.get(i..i + 10)?.parse::<NaiveDate>().ok())
        .max()
}

/// The rows of an index answer; rows without a readable tile or date are skipped.
pub fn parse_index(json: &str) -> Result<Vec<IndexRow>, String> {
    let answer: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if let Some(error) = answer.get("error") {
        return Err(format!("tile index error: {error}"));
    }
    let features = answer["features"]
        .as_array()
        .ok_or("tile index answer without features")?;
    Ok(features
        .iter()
        .filter_map(|feature| {
            let attributes = &feature["attributes"];
            let kachel = attributes["Kachel"].as_str()?;
            if kachel.len() != 7 {
                return None;
            }
            Some(IndexRow {
                tile: TileId {
                    east_km: kachel[..3].parse().ok()?,
                    north_km: kachel[3..].parse().ok()?,
                },
                currency: latest_date(attributes["Stand"].as_str()?)?,
                url: attributes["Download"].as_str()?.to_string(),
            })
        })
        .collect())
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const DGM1_INDEX: &str = include_str!("../../../../tests/fixtures/geosn-index-dgm1.json");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    const KREUZGYMNASIUM: TileId = TileId {
        east_km: 414,
        north_km: 5654,
    };
    const EAST: TileId = TileId {
        east_km: 416,
        north_km: 5654,
    };

    #[test]
    fn geosn_tile_names_and_members() {
        assert_eq!(
            Product::SaxonyDtm.tile_url(KREUZGYMNASIUM),
            "https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/JCcXyifaNdLDnxZ/dgm1_33414_5654_2_sn_tiff.zip"
        );
        assert_eq!(
            Product::SaxonyDsm.tile_url(KREUZGYMNASIUM),
            "https://geocloud.landesvermessung.sachsen.de/public.php/dav/files/S6wwnFwX7882sZm/dom1_33414_5654_2_sn_tiff.zip"
        );
        assert_eq!(
            Product::SaxonyDtm.raster_member(KREUZGYMNASIUM),
            "dgm1_33414_5654_2_sn.tif"
        );
        assert_eq!(
            Product::SaxonyDsm.world_file_member(KREUZGYMNASIUM),
            "dom1_33414_5654_2_sn.tfw"
        );
        assert_eq!(Product::SaxonyDtm.grid_km(), 2);
        assert_eq!(Product::Dtm.grid_km(), 1);
        assert_eq!(Product::SaxonyDtm.cache_dir_name(), "dgm1");
        assert_eq!(Product::SaxonyDtm.provider_dir_name(), "geosn");
        assert_eq!(Product::Dtm.provider_dir_name(), "lgb");
    }

    #[test]
    fn index_url_is_an_envelope_query() {
        assert_eq!(
            index_url(Product::SaxonyDtm, &[KREUZGYMNASIUM, EAST]),
            format!(
                "{INDEX_URL}/6/query?f=json&geometry=414001,5654001,417999,5655999\
                 &geometryType=esriGeometryEnvelope&inSR=25833&spatialRel=esriSpatialRelIntersects\
                 &outFields=Kachel,Download,Stand&returnGeometry=false"
            )
        );
        assert!(
            index_url(Product::SaxonyDsm, &[EAST]).starts_with(&format!("{INDEX_URL}/4/query?"))
        );
    }

    #[test]
    fn stand_takes_the_latest_date() {
        let date = |s: &str| s.parse::<NaiveDate>().ok();
        assert_eq!(latest_date("2024-11-30"), date("2024-11-30"));
        assert_eq!(
            latest_date("2020-11-24, 2020-11-25, 2020-11-30"),
            date("2020-11-30")
        );
        assert_eq!(
            latest_date("2024-11-27, 2024-11-30 bis 2024-12-28"),
            date("2024-12-28")
        );
        assert_eq!(latest_date(""), None);
        assert_eq!(latest_date("unbekannt"), None);
    }

    #[test]
    fn index_rows_give_currency_and_url() {
        let rows = parse_index(DGM1_INDEX).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].tile, KREUZGYMNASIUM);
        assert_eq!(rows[0].currency, "2024-11-30".parse::<NaiveDate>().unwrap());
        assert!(rows[0].url.ends_with("/dgm1_33414_5654_2_sn_tiff.zip"));
        assert_eq!(rows[1].tile, EAST);
        assert_eq!(rows[2].currency, "2024-12-28".parse::<NaiveDate>().unwrap());
        assert!(parse_index(r#"{"error":{"code":400,"message":"Invalid query"}}"#).is_err());
    }
}
