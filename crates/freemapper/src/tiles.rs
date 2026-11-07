//! Tile grids in EPSG:25833 (LGB 1 km, GeoSN 2 km); a tile is named after its south-west corner in
//! km.

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

    /// Bounds of the 1 km tile.
    pub fn bounds(&self) -> BoundingBox {
        self.bounds_km(1)
    }

    /// Bounds of the tile in a `km` grid.
    pub fn bounds_km(&self, km: u32) -> BoundingBox {
        let size = TILE_SIZE_M * f64::from(km);
        let min_e = f64::from(self.east_km) * TILE_SIZE_M;
        let min_n = f64::from(self.north_km) * TILE_SIZE_M;
        BoundingBox {
            min_e,
            min_n,
            max_e: min_e + size,
            max_n: min_n + size,
        }
    }
}

/// 1 km tiles covering `bbox`, sorted; a box ending exactly on a tile edge does not touch the
/// next tile.
pub fn tiles_covering(bbox: &BoundingBox) -> Vec<TileId> {
    tiles_covering_grid(bbox, 1)
}

/// Tiles of a `km` grid (south-west corners on multiples of `km`) covering `bbox`, sorted.
pub fn tiles_covering_grid(bbox: &BoundingBox, km: u32) -> Vec<TileId> {
    let size = TILE_SIZE_M * f64::from(km);
    let range = |min: f64, max: f64| {
        let first = (min / size).floor().max(0.0) as u32;
        let last = ((max / size).ceil() as u32).saturating_sub(1).max(first);
        (first..=last).map(move |i| i * km)
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
    fn two_km_tiles_cover_the_box() {
        // The Kreuzgymnasium Dresden straddles E 416 000.
        let b = bbox(415_878.0, 5_655_655.0, 416_069.0, 5_655_834.0);
        let at = |east_km, north_km| TileId { east_km, north_km };
        assert_eq!(
            tiles_covering_grid(&b, 2),
            vec![at(414, 5654), at(416, 5654)]
        );
        assert_eq!(tiles_covering_grid(&b, 1), tiles_covering(&b));
        // Ending exactly on an even edge does not touch the next tile.
        let edge = bbox(414_100.0, 5_654_100.0, 416_000.0, 5_656_000.0);
        assert_eq!(tiles_covering_grid(&edge, 2), vec![at(414, 5654)]);
        assert_eq!(
            at(414, 5654).bounds_km(2),
            bbox(414_000.0, 5_654_000.0, 416_000.0, 5_656_000.0)
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
