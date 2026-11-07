//! LGB tile grid: 1 × 1 km tiles in EPSG:25833, named after their south-west corner in km.

use crate::crs::BoundingBox;

/// Edge length of a tile in metres.
const TILE_SIZE_M: f64 = 1000.0;
/// UTM zone prefix of LGB tile names.
const ZONE_PREFIX: &str = "33";

/// Tile id: south-west corner of the tile in km, e.g. `33421-5821`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileId {
    pub east_km: u32,
    pub north_km: u32,
}

impl TileId {
    pub fn name(&self) -> String {
        format!("{ZONE_PREFIX}{:03}-{:04}", self.east_km, self.north_km)
    }

    pub fn parse(s: &str) -> Option<Self> {
        let (east, north) = s.strip_prefix(ZONE_PREFIX)?.split_once('-')?;
        if east.len() != 3 || north.len() != 4 {
            return None;
        }
        Some(Self {
            east_km: east.parse().ok()?,
            north_km: north.parse().ok()?,
        })
    }

    pub fn bounds(&self) -> BoundingBox {
        let min_e = f64::from(self.east_km) * TILE_SIZE_M;
        let min_n = f64::from(self.north_km) * TILE_SIZE_M;
        BoundingBox {
            min_e,
            min_n,
            max_e: min_e + TILE_SIZE_M,
            max_n: min_n + TILE_SIZE_M,
        }
    }
}

/// Tiles covering `bbox`, sorted; a box ending exactly on a tile edge does not touch the next tile.
pub fn tiles_covering(bbox: &BoundingBox) -> Vec<TileId> {
    let range = |min: f64, max: f64| {
        let first = (min / TILE_SIZE_M).floor().max(0.0) as u32;
        let last = ((max / TILE_SIZE_M).ceil() as u32)
            .saturating_sub(1)
            .max(first);
        first..=last
    };
    let mut tiles = Vec::new();
    for east_km in range(bbox.min_e, bbox.max_e) {
        for north_km in range(bbox.min_n, bbox.max_n) {
            tiles.push(TileId { east_km, north_km });
        }
    }
    tiles.sort();
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHOOL_TILE: TileId = TileId {
        east_km: 421,
        north_km: 5821,
    };

    fn bbox(min_e: f64, min_n: f64, max_e: f64, max_n: f64) -> BoundingBox {
        BoundingBox {
            min_e,
            min_n,
            max_e,
            max_n,
        }
    }

    #[test]
    fn tile_name_and_parse_roundtrip() {
        assert_eq!(SCHOOL_TILE.name(), "33421-5821");
        assert_eq!(TileId::parse("33421-5821"), Some(SCHOOL_TILE));
        assert_eq!(TileId::parse("421-5821"), None);
        assert_eq!(TileId::parse("34421-5821"), None);
        assert_eq!(
            SCHOOL_TILE.bounds(),
            bbox(421_000.0, 5_821_000.0, 422_000.0, 5_822_000.0)
        );
    }

    #[test]
    fn tiles_covering_school_is_one_tile() {
        let b = bbox(421_400.0, 5_821_550.0, 421_800.0, 5_821_900.0);
        assert_eq!(tiles_covering(&b), vec![SCHOOL_TILE]);
    }

    #[test]
    fn tiles_covering_edge() {
        let across = bbox(421_990.0, 5_821_500.0, 422_010.0, 5_821_600.0);
        assert_eq!(
            tiles_covering(&across),
            vec![
                SCHOOL_TILE,
                TileId {
                    east_km: 422,
                    north_km: 5821
                }
            ]
        );
        let up_to_edge = bbox(421_500.0, 5_821_500.0, 422_000.0, 5_822_000.0);
        assert_eq!(tiles_covering(&up_to_edge), vec![SCHOOL_TILE]);
    }
}
