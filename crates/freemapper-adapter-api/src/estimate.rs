//! The download estimate: what an adapter is asked and the sizes most regions share.

use chrono::NaiveDate;
use freemapper_georef::crs::BoundingBox;

use crate::cache::CacheDir;
use crate::records::{EstimateItem, Part, Store};

/// The area a download estimate is asked for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EstimateArea {
    /// The map area plus buffer in EPSG:25833, where the boundaries are.
    pub bbox_25833: BoundingBox,
    /// The same in the CRS of the map.
    pub map_bbox: BoundingBox,
    /// The day of the estimate, from the clock of the program.
    pub today: NaiveDate,
}

/// What an adapter may look at for the estimate.
#[derive(Debug, Clone, Copy)]
pub struct EstimateEnv<'a> {
    /// The shared cache: a file that is there is not downloaded again.
    pub cache: &'a CacheDir,
}

/// A WMS orthophoto per km² of map area plus buffer, in MB (all states).
pub const ORTHOPHOTO_MB_PER_KM2: f64 = 8.0;
/// A 1 m Float32 height model from a WCS per km², in MB (LVermGeo, LAiV M-V, LGLN).
pub const WCS_MB_PER_KM2: f64 = 4.0;

/// A download that grows with the area: `mb_per_km2` over `bbox` (at least 0.1 MB), requested
/// for the area itself and stored in the project folder.
pub fn per_km2_item(bbox: &BoundingBox, what: &str, mb_per_km2: f64, part: Part) -> EstimateItem {
    EstimateItem {
        what: what.to_string(),
        megabytes: (bbox.width() * bbox.height() / 1e6 * mb_per_km2).max(0.1),
        cached: false,
        part,
        store: Store::Project,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_per_km2_item_grows_with_the_area_and_has_a_floor() {
        let square = |side: f64| BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: side,
            max_n: side,
        };
        let item = per_km2_item(&square(2_000.0), "orthophoto", 8.0, Part::Orthophoto);
        assert_eq!(item.megabytes, 32.0);
        assert_eq!(
            (item.what.as_str(), item.cached, item.part, item.store),
            ("orthophoto", false, Part::Orthophoto, Store::Project)
        );
        assert_eq!(
            per_km2_item(&square(10.0), "small", 8.0, Part::Terrain).megabytes,
            0.1
        );
    }
}
