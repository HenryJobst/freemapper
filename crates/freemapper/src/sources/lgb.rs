//! Raster products of all providers (LGB Brandenburg, GeoSN Saxony; dl-de/by-2-0), tile URLs,
//! and the LGB data currency.

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::sources::region::Provider;
use crate::tiles::TileId;

const DATA_URL: &str = "https://data.geobasis-bb.de/geobasis/daten";
const DOP_CURRENCY_CSV: &str =
    "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dop_aktualitaet.csv";
const DGM_CURRENCY_CSV: &str =
    "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dgm_aktualitaet.csv";
/// GeoSN GeoCloud link shares (Nextcloud); the tokens change now and then (§2.1 of the Saxony
/// reference), so the tile index is asked first.
const GEOSN_SHARES: &str = "https://geocloud.landesvermessung.sachsen.de/public.php/dav/files";
const GEOSN_DGM1_SHARE: &str = "JCcXyifaNdLDnxZ";
const GEOSN_DOM1_SHARE: &str = "S6wwnFwX7882sZm";
const BDOM_CURRENCY_CSV: &str =
    "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_bdom_aktualitaet.csv";

/// LGB product used by freemapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Product {
    /// DOP20 RGB orthophoto, JPEG + world file, 0.2 m.
    Orthophoto,
    /// DGM1 digital terrain model, GeoTIFF f32 + world file, 1 m.
    Dtm,
    /// bDOM image-matched surface model, GeoTIFF f32 + world file, 0.2 m; the nDSM source
    /// where no nDOM point cloud is published (Berlin).
    ImageDsm,
    /// GeoSN DGM1 terrain model, 2 km tiles, GeoTIFF f32 + world file, 1 m.
    SaxonyDtm,
    /// GeoSN DOM1 laser surface model, 2 km tiles, GeoTIFF f32 + world file, 1 m; the nDSM
    /// source in Saxony, where no nDOM is published.
    SaxonyDsm,
}

impl Product {
    /// Who serves the product.
    pub fn provider(self) -> Provider {
        match self {
            Self::Orthophoto | Self::Dtm | Self::ImageDsm => Provider::Lgb,
            Self::SaxonyDtm | Self::SaxonyDsm => Provider::GeoSn,
        }
    }

    /// Edge length of the product's tiles in km.
    pub fn grid_km(self) -> u32 {
        match self.provider() {
            Provider::Lgb => 1,
            Provider::GeoSn => 2,
            provider @ (Provider::LvermGeo | Provider::Lgv) => {
                unreachable!("{provider:?} rasters are fetched per area, not as tiles")
            }
        }
    }

    fn file_prefix(self) -> &'static str {
        match self {
            Self::Orthophoto => "dop",
            Self::Dtm => "dgm",
            Self::ImageDsm => "bdom",
            Self::SaxonyDtm => "dgm1",
            Self::SaxonyDsm => "dom1",
        }
    }

    /// File stem of a tile: `dgm_33421-5821` (LGB), `dgm1_33414_5654_2_sn` (GeoSN).
    fn stem(self, t: TileId) -> String {
        match self.provider() {
            Provider::Lgb => format!("{}_{}", self.file_prefix(), t.name()),
            Provider::GeoSn => format!(
                "{}_33{:03}_{:04}_2_sn",
                self.file_prefix(),
                t.east_km,
                t.north_km
            ),
            provider @ (Provider::LvermGeo | Provider::Lgv) => {
                unreachable!("{provider:?} rasters are fetched per area, not as tiles")
            }
        }
    }

    /// Download URL of a tile. GeoSN rotates its share tokens, so its tiles are downloaded from
    /// the URL in the tile index; this one is the fallback.
    pub fn tile_url(self, t: TileId) -> String {
        match self {
            Self::Orthophoto => format!("{DATA_URL}/dop/rgb_jpg/{}.zip", self.stem(t)),
            Self::Dtm => format!("{DATA_URL}/dgm/tif/{}.zip", self.stem(t)),
            Self::ImageDsm => format!("{DATA_URL}/bdom/tif/{}.zip", self.stem(t)),
            Self::SaxonyDtm => format!(
                "{GEOSN_SHARES}/{GEOSN_DGM1_SHARE}/{}_tiff.zip",
                self.stem(t)
            ),
            Self::SaxonyDsm => format!(
                "{GEOSN_SHARES}/{GEOSN_DOM1_SHARE}/{}_tiff.zip",
                self.stem(t)
            ),
        }
    }

    /// The LGB currency CSV; GeoSN products take their currency from the tile index instead.
    pub fn currency_csv_url(self) -> &'static str {
        match self {
            Self::Orthophoto => DOP_CURRENCY_CSV,
            Self::Dtm => DGM_CURRENCY_CSV,
            Self::ImageDsm => BDOM_CURRENCY_CSV,
            Self::SaxonyDtm | Self::SaxonyDsm => crate::sources::geosn::INDEX_URL,
        }
    }

    /// Name of the raster file inside the tile archive.
    pub fn raster_member(self, t: TileId) -> String {
        let ext = match self {
            Self::Orthophoto => "jpg",
            _ => "tif",
        };
        format!("{}.{ext}", self.stem(t))
    }

    /// Name of the world file inside the tile archive.
    pub fn world_file_member(self, t: TileId) -> String {
        let ext = match self {
            Self::Orthophoto => "jgw",
            _ => "tfw",
        };
        format!("{}.{ext}", self.stem(t))
    }

    /// Human-readable name for messages.
    pub fn label(self) -> &'static str {
        match self {
            Self::Orthophoto => "orthophoto (DOP20)",
            Self::Dtm => "terrain model (DGM1)",
            Self::ImageDsm => "surface model (bDOM)",
            Self::SaxonyDtm => "terrain model (GeoSN DGM1)",
            Self::SaxonyDsm => "surface model (GeoSN DOM1)",
        }
    }

    /// Folder of the product's provider in the cache.
    pub fn provider_dir_name(self) -> &'static str {
        match self.provider() {
            Provider::Lgb => "lgb",
            Provider::GeoSn => "geosn",
            Provider::LvermGeo => "lvermgeo",
            Provider::Lgv => "lgv",
        }
    }

    /// Folder of this product in its provider's cache folder.
    pub fn cache_dir_name(self) -> &'static str {
        match self {
            Self::Orthophoto => "dop20rgb",
            Self::Dtm | Self::SaxonyDtm => "dgm1",
            Self::ImageDsm => "bdom",
            Self::SaxonyDsm => "dom1",
        }
    }
}

/// First columns of an LGB currency CSV.
const CURRENCY_CSV_HEADER: &str = "sheetnr;sheetname;creationdate;revisiondate;publicationdate";

impl std::fmt::Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Data currency of one tile from the LGB currency CSV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCurrency {
    pub created: NaiveDate,
    pub revised: Option<NaiveDate>,
    pub published: NaiveDate,
}

impl TileCurrency {
    /// Date the data describes: the revision date, else the creation date.
    pub fn currency(&self) -> NaiveDate {
        self.revised.unwrap_or(self.created)
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("currency CSV line {line}: {message}")]
pub struct CurrencyCsvError {
    pub line: usize,
    pub message: String,
}

/// Parses an LGB currency CSV (`sheetnr;sheetname;creationdate;revisiondate;publicationdate;…`).
pub fn parse_currency_csv(text: &str) -> Result<HashMap<TileId, TileCurrency>, CurrencyCsvError> {
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    let header = lines.next().unwrap_or_default();
    if !header.starts_with(CURRENCY_CSV_HEADER) {
        return Err(CurrencyCsvError {
            line: 1,
            message: format!("unexpected header {header:?}"),
        });
    }
    let mut table = HashMap::new();
    for (index, line) in lines.enumerate() {
        let line_no = index + 2;
        if line.trim().is_empty() {
            continue;
        }
        let error = |message: String| CurrencyCsvError {
            line: line_no,
            message,
        };
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() < 5 {
            return Err(error(format!("expected 5 fields, found {}", fields.len())));
        }
        let tile = TileId::parse(fields[0].trim())
            .ok_or_else(|| error(format!("invalid tile id {:?}", fields[0])))?;
        let date = |field: &str| -> Result<Option<NaiveDate>, CurrencyCsvError> {
            let field = field.trim();
            if field.is_empty() {
                return Ok(None);
            }
            field
                .parse()
                .map(Some)
                .map_err(|_| error(format!("invalid date {field:?}")))
        };
        let created = date(fields[2])?.ok_or_else(|| error("missing creation date".into()))?;
        let revised = date(fields[3])?;
        let published = date(fields[4])?.ok_or_else(|| error("missing publication date".into()))?;
        table.insert(
            tile,
            TileCurrency {
                created,
                revised,
                published,
            },
        );
    }
    Ok(table)
}

/// URL of the nDSM point cloud of `t` (LAS 1.2, compressed; no zip, no currency CSV).
pub fn ndsm_tile_url(t: TileId) -> String {
    format!("{DATA_URL}/ndom/NDOM_{}_LAS12.laz", t.name())
}

/// Attribution line required by dl-de/by-2-0.
pub fn attribution(year: i32) -> String {
    format!("© GeoBasis-DE/LGB {year}, dl-de/by-2-0, Daten geändert")
}

/// The LGB attribution for tiles holding Berlin data, as the LGB licence note asks.
pub fn attribution_with_berlin(year: i32) -> String {
    format!(
        "© GeoBasis-DE/LGB {year}, dl-de/by-2-0; © Geoportal Berlin, dl-de/zero-2-0, Daten geändert"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };

    fn date(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn bdom_url_matches_lgb_layout() {
        let tile = TileId {
            east_km: 392,
            north_km: 5820,
        };
        assert_eq!(
            Product::ImageDsm.tile_url(tile),
            "https://data.geobasis-bb.de/geobasis/daten/bdom/tif/bdom_33392-5820.zip"
        );
        assert_eq!(
            Product::ImageDsm.currency_csv_url(),
            "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_bdom_aktualitaet.csv"
        );
        assert_eq!(Product::ImageDsm.raster_member(tile), "bdom_33392-5820.tif");
        assert_eq!(
            Product::ImageDsm.world_file_member(tile),
            "bdom_33392-5820.tfw"
        );
        assert_eq!(Product::ImageDsm.cache_dir_name(), "bdom");
    }

    #[test]
    fn urls_match_lgb_layout() {
        assert_eq!(
            Product::Orthophoto.tile_url(TILE),
            "https://data.geobasis-bb.de/geobasis/daten/dop/rgb_jpg/dop_33421-5821.zip"
        );
        assert_eq!(
            Product::Dtm.tile_url(TILE),
            "https://data.geobasis-bb.de/geobasis/daten/dgm/tif/dgm_33421-5821.zip"
        );
        assert_eq!(
            Product::Orthophoto.raster_member(TILE),
            "dop_33421-5821.jpg"
        );
        assert_eq!(
            Product::Orthophoto.world_file_member(TILE),
            "dop_33421-5821.jgw"
        );
        assert_eq!(Product::Dtm.raster_member(TILE), "dgm_33421-5821.tif");
        assert_eq!(Product::Dtm.world_file_member(TILE), "dgm_33421-5821.tfw");
        assert_eq!(
            Product::Dtm.currency_csv_url(),
            "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dgm_aktualitaet.csv"
        );
        assert_eq!(
            Product::Orthophoto.currency_csv_url(),
            "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dop_aktualitaet.csv"
        );
        assert_eq!(Product::Orthophoto.cache_dir_name(), "dop20rgb");
        assert_eq!(Product::Dtm.cache_dir_name(), "dgm1");
    }

    #[test]
    fn currency_csv_parses_bom_and_empty_fields() {
        let csv = "\u{feff}sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n\
                   33421-5821;;2018-11-27;2023-05-04;2026-01-26;\n\
                   33421-5822;;2026-05-01;;2026-10-09;2026-06-15\n";
        let table = parse_currency_csv(csv).unwrap();
        assert_eq!(table.len(), 2);
        let dtm = table[&TILE];
        assert_eq!(dtm.currency(), date("2023-05-04"));
        assert_eq!(dtm.published, date("2026-01-26"));
        let north = TileId {
            east_km: 421,
            north_km: 5822,
        };
        assert_eq!(table[&north].revised, None);
        assert_eq!(table[&north].currency(), date("2026-05-01"));
    }

    #[test]
    fn currency_csv_reports_bad_line() {
        let csv = "sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n\
                   33421-5821;;2018-11-27;2023-05-04;2026-01-26;\n\
                   33421-5822;;2026-13-01;;2026-10-09;\n";
        let err = parse_currency_csv(csv).unwrap_err();
        assert_eq!(err.line, 3);
    }

    #[test]
    fn currency_csv_rejects_other_documents() {
        let err = parse_currency_csv("<html>captive portal</html>").unwrap_err();
        assert_eq!(err.line, 1);
    }

    #[test]
    fn ndsm_url_matches_lgb_layout() {
        assert_eq!(
            ndsm_tile_url(TILE),
            "https://data.geobasis-bb.de/geobasis/daten/ndom/NDOM_33421-5821_LAS12.laz"
        );
    }

    #[test]
    fn attribution_text() {
        assert_eq!(
            attribution(2026),
            "© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert"
        );
    }
}
