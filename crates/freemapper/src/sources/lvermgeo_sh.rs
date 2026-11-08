//! The LVermGeo SH Downloadportal (`geodaten.schleswig-holstein.de`, CC BY 4.0, "© GeoBasis-DE/
//! LVermGeo SH"): the DGM1 and bDOM tiles under an area, looked up through the portal's AJAX
//! `overview.php` (EPSG:25832, 1 km tiles).
//!
//! File names carry a year that differs per tile, so they must come from the portal; the
//! `massen.php` links are built from the same attributes as the portal's own index
//! (`link_data`).

use chrono::NaiveDate;
use serde_json::Value;

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, Fetcher, get_with_retries};
use crate::tiles::TileId;

/// The Downloadportal; its host is reached through the bundled D-TRUST anchors (see `fetch`).
pub const PORTAL: &str = "https://geodaten.schleswig-holstein.de/gaialight-sh/_apps/dladownload";

/// A product of the Downloadportal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Product {
    /// DGM1, the terrain (XYZ on the portal).
    Dgm1,
    /// bDOM 0.2 m, the image-based surface (GeoTIFF).
    Bdom,
}

impl Product {
    /// The `type[]` of `overview.php`.
    fn overview_type(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Bdom => "bdom",
        }
    }
}

/// A tile of a product as the portal lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalTile {
    pub tile: TileId,
    /// File name without extension, e.g. `dgm1_32_565_5949_1_sh_2022`.
    pub name: String,
    pub year: u16,
    /// Acquisition (bDOM: flight) date, `YYYY-MM-DD`.
    pub date: Option<String>,
    /// The `massen.php` download URL.
    pub link: String,
}

/// The AJAX tile lookup of `product` over `bbox` (EPSG:25832), rounded outward to whole metres.
fn overview_url(bbox: &BoundingBox, product: Product) -> String {
    format!(
        "{PORTAL}/_ajax/overview.php?bbox={},{},{},{}&crs=EPSG:25832&type[]={}",
        bbox.min_e.floor(),
        bbox.min_n.floor(),
        bbox.max_e.ceil(),
        bbox.max_n.ceil(),
        product.overview_type()
    )
}

/// The tiles of `product` intersecting `bbox` (EPSG:25832), sorted by tile.
///
/// The portal answers at most 200 tiles (a 4 x 4 km area has 25); more, and an answer that is
/// not a tile overview (the portal reports errors as HTML with status 200), are errors.
pub fn overview(
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    product: Product,
) -> Result<Vec<PortalTile>, FetchError> {
    let url = overview_url(bbox, product);
    // Retries cover the transport only; a refused or malformed answer does not change.
    let bytes = get_with_retries(fetcher, &url, |bytes| Ok(bytes.to_vec()))?;
    let error = |message: String| FetchError::Http {
        url: url.clone(),
        message,
    };
    let json: Value = serde_json::from_slice(&bytes).map_err(|_| {
        error(format!(
            "not a tile overview: {}",
            excerpt(&String::from_utf8_lossy(&bytes))
        ))
    })?;
    if json["success"] != true {
        return Err(error(match json["reason"].as_str() {
            Some("tooManyObjects") => format!(
                "the Downloadportal lists at most {} tiles per request; reduce the map area",
                json["max"].as_u64().unwrap_or(200)
            ),
            _ => format!("not a tile overview: {}", excerpt(&json.to_string())),
        }));
    }
    let features = json["result"]["features"]
        .as_array()
        .ok_or_else(|| error("not a tile overview: no features".to_string()))?;
    let mut tiles = features
        .iter()
        .map(|feature| &feature["properties"])
        .filter(|properties| text(&properties["type"]).as_deref() == Some(product.overview_type()))
        .map(|properties| portal_tile(properties, product).map_err(error))
        .collect::<Result<Vec<_>, _>>()?;
    tiles.sort_by_key(|t| t.tile);
    Ok(tiles)
}

/// One overview entry; every part of the link is checked to be plain letters, digits and `_`.
fn portal_tile(properties: &Value, product: Product) -> Result<PortalTile, String> {
    let field = |key: &str| {
        text(&properties[key])
            .filter(|value| is_plain(value))
            .ok_or_else(|| format!("tile entry without a valid `{key}`: {properties}"))
    };
    let title = field("title")?;
    let tile = parse_tile_key(&title).ok_or_else(|| format!("unknown tile key {title}"))?;
    let year = field("jahr")?;
    let year: u16 = year.parse().map_err(|_| format!("unknown year {year}"))?;
    let date =
        text(&properties["a_datum"]).filter(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").is_ok());
    let (name, link) = match product {
        Product::Dgm1 => {
            let name = field("kaname")?;
            let km = field("filepath")?;
            let link = format!("{PORTAL}/massen.php?file={name}.xyz&id=2&live={year}&km={km}");
            (name, link)
        }
        Product::Bdom => {
            let listed = field("kachel_n")?;
            // The overview names the tile `bdom20rgbi_…`; the file is `bdom20nc_…`.
            let name = match listed.strip_prefix("bdom20rgbi_") {
                Some(rest) => format!("bdom20nc_{rest}"),
                None => listed,
            };
            let km = field("kach10km")?;
            let stack = field("filepath")?;
            let link = format!(
                "{PORTAL}/massen.php?file={name}.tif&id=6&live={year}&km={km}&stack={stack}"
            );
            (name, link)
        }
    };
    Ok(PortalTile {
        tile,
        name,
        year,
        date,
        link,
    })
}

/// A string or number property as text.
fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Safe to put into a URL unescaped.
fn is_plain(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// `325655949` (zone 32, E 565 km, N 5949 km).
fn parse_tile_key(key: &str) -> Option<TileId> {
    let digits = key.strip_prefix("32")?;
    if digits.len() != 7 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(TileId {
        east_km: digits[..3].parse().ok()?,
        north_km: digits[3..].parse().ok()?,
    })
}

/// The start of an error answer, on one line.
fn excerpt(text: &str) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    match line.char_indices().nth(200) {
        Some((end, _)) => format!("{}…", &line[..end]),
        None => line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const DGM1: &str =
        include_str!("../../../../tests/fixtures/schleswig-holstein-overview-dgm1.json");
    const BDOM: &str =
        include_str!("../../../../tests/fixtures/schleswig-holstein-overview-bdom.json");
    const TOO_MANY: &str =
        include_str!("../../../../tests/fixtures/schleswig-holstein-overview-too-many.json");

    /// The Coppernicus-Gymnasium in Norderstedt.
    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 565_100.0,
        min_n: 5_949_100.0,
        max_e: 565_300.0,
        max_n: 5_949_300.0,
    };

    const TILE: TileId = TileId {
        east_km: 565,
        north_km: 5949,
    };

    fn url(product: &str) -> String {
        format!(
            "{PORTAL}/_ajax/overview.php?bbox=565100,5949100,565300,5949300&crs=EPSG:25832\
             &type[]={product}"
        )
    }

    fn serving(product: &str, body: &str) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert(&url(product), body.as_bytes().to_vec());
        fetcher
    }

    #[test]
    fn overview_lists_dgm1_and_bdom_tiles() {
        let dgm1 = overview(&serving("dgm1", DGM1), &SCHOOL, Product::Dgm1).unwrap();
        assert_eq!(
            dgm1.iter().map(|t| t.tile).collect::<Vec<_>>(),
            [
                TILE,
                TileId {
                    east_km: 565,
                    north_km: 5950
                },
                TileId {
                    east_km: 566,
                    north_km: 5949
                },
            ]
        );
        assert_eq!(
            dgm1[0],
            PortalTile {
                tile: TILE,
                name: "dgm1_32_565_5949_1_sh_2022".to_string(),
                year: 2022,
                date: Some("2022-03-10".to_string()),
                link: format!(
                    "{PORTAL}/massen.php?file=dgm1_32_565_5949_1_sh_2022.xyz&id=2&live=2022\
                     &km=32560_5940"
                ),
            }
        );
        // The 10 km block follows the tile.
        assert!(dgm1[1].link.ends_with("&km=32560_5950"), "{}", dgm1[1].link);

        let bdom = overview(&serving("bdom", BDOM), &SCHOOL, Product::Bdom).unwrap();
        assert_eq!(
            bdom,
            [PortalTile {
                tile: TILE,
                // The overview says `bdom20rgbi_…`; the file is `bdom20nc_…`.
                name: "bdom20nc_32_565_5949_1_sh_2023".to_string(),
                year: 2023,
                date: Some("2023-04-07".to_string()),
                link: format!(
                    "{PORTAL}/massen.php?file=bdom20nc_32_565_5949_1_sh_2023.tif&id=6&live=2023\
                     &km=32560_5940&stack=32565_5949"
                ),
            }]
        );
    }

    #[test]
    fn too_many_objects_is_an_error() {
        let err = overview(&serving("dgm1", TOO_MANY), &SCHOOL, Product::Dgm1).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("at most 200 tiles"), "{message}");
        assert!(message.contains("reduce the map area"), "{message}");
    }

    #[test]
    fn html_answer_is_an_error() {
        let html = "Folgender Datensatz: foo konnte nicht heruntergeladen werden. <br> Die \
                    verwendete Massendownload-Datei ist veraltet.";
        let err = overview(&serving("bdom", html), &SCHOOL, Product::Bdom).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("not a tile overview"), "{message}");
        assert!(message.contains("Folgender Datensatz"), "{message}");
    }

    #[test]
    fn unsafe_names_are_refused() {
        let evil = DGM1.replace("dgm1_32_565_5949_1_sh_2022", "x&file=../../etc");
        let err = overview(&serving("dgm1", &evil), &SCHOOL, Product::Dgm1).unwrap_err();
        assert!(err.to_string().contains("x&file=../../etc"), "{err}");
    }

    #[test]
    #[ignore = "network"]
    fn live_overview_lists_the_school_tile() {
        let fetcher = crate::fetch::HttpFetcher::new();
        for product in [Product::Dgm1, Product::Bdom] {
            let tiles = overview(&fetcher, &SCHOOL, product).unwrap();
            assert_eq!(
                tiles.iter().map(|t| t.tile).collect::<Vec<_>>(),
                [TILE],
                "{product:?}"
            );
            assert!(tiles[0].year >= 2022, "{tiles:?}");
        }
    }
}
