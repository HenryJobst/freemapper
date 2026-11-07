//! LGB Brandenburg open data (dl-de/by-2-0): products, tile URLs and data currency.

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::tiles::TileId;

const DATA_URL: &str = "https://data.geobasis-bb.de/geobasis/daten";
const DOP_CURRENCY_CSV: &str =
    "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dop_aktualitaet.csv";
const DGM_CURRENCY_CSV: &str =
    "https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dgm_aktualitaet.csv";

/// LGB product used by freemapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Product {
    /// DOP20 RGB orthophoto, JPEG + world file, 0.2 m.
    Orthophoto,
    /// DGM1 digital terrain model, GeoTIFF f32 + world file, 1 m.
    Dtm,
}

impl Product {
    fn file_prefix(self) -> &'static str {
        match self {
            Self::Orthophoto => "dop",
            Self::Dtm => "dgm",
        }
    }

    pub fn tile_url(self, t: TileId) -> String {
        let dir = match self {
            Self::Orthophoto => "dop/rgb_jpg",
            Self::Dtm => "dgm/tif",
        };
        format!("{DATA_URL}/{dir}/{}_{}.zip", self.file_prefix(), t.name())
    }

    pub fn currency_csv_url(self) -> &'static str {
        match self {
            Self::Orthophoto => DOP_CURRENCY_CSV,
            Self::Dtm => DGM_CURRENCY_CSV,
        }
    }

    /// Name of the raster file inside the tile archive.
    pub fn raster_member(self, t: TileId) -> String {
        let ext = match self {
            Self::Orthophoto => "jpg",
            Self::Dtm => "tif",
        };
        format!("{}_{}.{ext}", self.file_prefix(), t.name())
    }

    /// Name of the world file inside the tile archive.
    pub fn world_file_member(self, t: TileId) -> String {
        let ext = match self {
            Self::Orthophoto => "jgw",
            Self::Dtm => "tfw",
        };
        format!("{}_{}.{ext}", self.file_prefix(), t.name())
    }

    /// Folder of this product in the cache.
    pub fn cache_dir_name(self) -> &'static str {
        match self {
            Self::Orthophoto => "dop20rgb",
            Self::Dtm => "dgm1",
        }
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
    let mut table = HashMap::new();
    for (index, line) in text
        .trim_start_matches('\u{feff}')
        .lines()
        .enumerate()
        .skip(1)
    {
        let line_no = index + 1;
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

/// Attribution line required by dl-de/by-2-0.
pub fn attribution(year: i32) -> String {
    format!("© GeoBasis-DE/LGB {year}, dl-de/by-2-0, Daten geändert")
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
    fn attribution_text() {
        assert_eq!(
            attribution(2026),
            "© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert"
        );
    }
}
