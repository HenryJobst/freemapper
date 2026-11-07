//! Regions: which state's ALKIS service covers an area.
//!
//! The LGB rasters cover Berlin too, but ALKIS is kept per state: the LGB service returns no
//! features inside Berlin and the Berlin service none outside, so an area near the border needs
//! both.

use std::sync::OnceLock;

use geo::{Contains, Intersects};
use geo_types::{MultiPolygon, Polygon, Rect, coord};

use crate::crs::BoundingBox;

/// Berlin boundary from ALKIS `alkis:land` (EPSG:25833, simplified 25 m; dl-de/zero-2-0).
const BERLIN_BOUNDARY: &str = include_str!("../../../../assets/regions/berlin.geojson");
/// The area is widened by this much before testing, so the 25 m simplification never drops
/// buildings right at the border.
const MARGIN_M: f64 = 100.0;

/// A state with its own ALKIS service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Brandenburg,
    Berlin,
}

impl Region {
    /// The state an ALKIS object id belongs to (`DEBB…`, `DEBE…`).
    pub fn of_object_id(oid: &str) -> Option<Self> {
        match oid.get(..4) {
            Some("DEBB") => Some(Self::Brandenburg),
            Some("DEBE") => Some(Self::Berlin),
            _ => None,
        }
    }
}

/// The regions whose ALKIS data covers `bbox` (EPSG:25833), Brandenburg first.
pub fn regions_for(bbox: &BoundingBox) -> Vec<Region> {
    let rect = rectangle(&bbox.buffered(MARGIN_M));
    let berlin = berlin();
    let mut regions = Vec::new();
    if !berlin.contains(&rect) {
        regions.push(Region::Brandenburg);
    }
    if berlin.intersects(&rect) {
        regions.push(Region::Berlin);
    }
    regions
}

/// Whether `bbox` itself (without the fetch margin) overlaps Berlin.
pub fn touches_berlin(bbox: &BoundingBox) -> bool {
    berlin().intersects(&rectangle(bbox))
}

fn rectangle(bbox: &BoundingBox) -> Polygon<f64> {
    Rect::new(
        coord! { x: bbox.min_e, y: bbox.min_n },
        coord! { x: bbox.max_e, y: bbox.max_n },
    )
    .to_polygon()
}

/// The Berlin boundary, parsed once.
fn berlin() -> &'static MultiPolygon<f64> {
    static BOUNDARY: OnceLock<MultiPolygon<f64>> = OnceLock::new();
    BOUNDARY.get_or_init(|| {
        let collection: geojson::FeatureCollection = BERLIN_BOUNDARY
            .parse()
            .expect("embedded Berlin boundary is GeoJSON");
        let geometry = collection.features[0]
            .geometry
            .clone()
            .expect("embedded Berlin boundary has a geometry");
        MultiPolygon::try_from(geometry.value).expect("embedded Berlin boundary is a MultiPolygon")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn around(e: f64, n: f64, half: f64) -> BoundingBox {
        BoundingBox {
            min_e: e - half,
            min_n: n - half,
            max_e: e + half,
            max_n: n + half,
        }
    }

    #[test]
    fn strausberg_is_brandenburg() {
        assert_eq!(
            regions_for(&around(421_582.0, 5_821_278.0, 500.0)),
            vec![Region::Brandenburg]
        );
    }

    #[test]
    fn mitte_is_berlin() {
        assert_eq!(
            regions_for(&around(391_000.0, 5_820_000.0, 500.0)),
            vec![Region::Berlin]
        );
    }

    #[test]
    fn border_needs_both() {
        // The southernmost boundary vertex, at Schönefeld.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_524.0, 300.0)),
            vec![Region::Brandenburg, Region::Berlin]
        );
    }

    #[test]
    fn margin_reaches_across_the_border() {
        // 50 m south of the boundary: still within the margin.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_424.0, 50.0)),
            vec![Region::Brandenburg, Region::Berlin]
        );
        // 300 m south: Brandenburg only.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_124.0, 50.0)),
            vec![Region::Brandenburg]
        );
    }

    #[test]
    fn object_ids_name_their_state() {
        assert_eq!(
            Region::of_object_id("DEBE03YY600004AR"),
            Some(Region::Berlin)
        );
        assert_eq!(
            Region::of_object_id("DEBBAL0100000001"),
            Some(Region::Brandenburg)
        );
        assert_eq!(Region::of_object_id("DESNAL01"), None);
    }

    #[test]
    fn touching_berlin_needs_no_margin() {
        // 50 m south of the boundary: within the fetch margin, but not in Berlin.
        assert!(!touches_berlin(&around(407_850.0, 5_799_424.0, 40.0)));
        assert!(touches_berlin(&around(407_850.0, 5_799_524.0, 40.0)));
    }

    #[test]
    fn boundary_has_all_parts() {
        assert_eq!(berlin().0.len(), 4);
        assert_eq!(berlin().0[0].exterior().0.len(), 664);
    }
}
