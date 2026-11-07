//! Regions: the states freemapper has data for, and which of them cover an area.
//!
//! Every state keeps its own ALKIS service: the LGB service returns no features inside Berlin and
//! the Berlin service none outside, so an area near a border needs several. The rasters come
//! from one provider, the one of the region containing the area's centre (LGB also serves Berlin).

use std::sync::OnceLock;

use geo::{Contains, Intersects};
use geo_types::{MultiPolygon, Point, Polygon, Rect, coord};

use crate::crs::BoundingBox;
use crate::tiles::TileId;

/// Brandenburg from LGB ALKIS `verwaltungseinheit` DE12, with Berlin as a hole (dl-de/by-2-0).
const BRANDENBURG_BOUNDARY: &str = include_str!("../../../../assets/regions/brandenburg.geojson");
/// Berlin boundary from ALKIS `alkis:land` (EPSG:25833, simplified 25 m; dl-de/zero-2-0).
const BERLIN_BOUNDARY: &str = include_str!("../../../../assets/regions/berlin.geojson");
/// Saxony from GeoSN ALKIS `VerwaltungsEinheit` DE14 (dl-de/by-2-0).
const SAXONY_BOUNDARY: &str = include_str!("../../../../assets/regions/saxony.geojson");
/// The area is widened by this much before testing, so the 25 m simplification never drops
/// buildings right at the border.
const MARGIN_M: f64 = 100.0;

/// A state with its own ALKIS service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Brandenburg,
    Berlin,
    Saxony,
}

/// Who serves a region's rasters (orthophoto, terrain and surface models).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// LGB Brandenburg, also for Berlin.
    Lgb,
    /// GeoSN, Saxony.
    GeoSn,
}

impl Region {
    /// All regions, in the order they are listed and queried.
    pub const ALL: [Self; 3] = [Self::Brandenburg, Self::Berlin, Self::Saxony];

    /// The state an ALKIS object id belongs to (`DEBB…`, `DEBE…`, `DESN…`).
    pub fn of_object_id(oid: &str) -> Option<Self> {
        match oid.get(..4) {
            Some("DEBB") => Some(Self::Brandenburg),
            Some("DEBE") => Some(Self::Berlin),
            Some("DESN") => Some(Self::Saxony),
            _ => None,
        }
    }

    pub fn provider(self) -> Provider {
        match self {
            Self::Brandenburg | Self::Berlin => Provider::Lgb,
            Self::Saxony => Provider::GeoSn,
        }
    }

    /// The region's boundary, parsed once.
    fn boundary(self) -> &'static MultiPolygon<f64> {
        static BOUNDARIES: [OnceLock<MultiPolygon<f64>>; 3] =
            [OnceLock::new(), OnceLock::new(), OnceLock::new()];
        let (index, text) = match self {
            Self::Brandenburg => (0, BRANDENBURG_BOUNDARY),
            Self::Berlin => (1, BERLIN_BOUNDARY),
            Self::Saxony => (2, SAXONY_BOUNDARY),
        };
        BOUNDARIES[index].get_or_init(|| parse_boundary(text))
    }
}

/// The regions whose ALKIS data covers `bbox` (EPSG:25833), in [`Region::ALL`] order.
pub fn regions_for(bbox: &BoundingBox) -> Vec<Region> {
    let rect = rectangle(&bbox.buffered(MARGIN_M));
    Region::ALL
        .into_iter()
        .filter(|region| region.boundary().intersects(&rect))
        .collect()
}

/// The region whose rasters a map of `bbox` uses: the one containing its centre.
pub fn raster_region(bbox: &BoundingBox) -> Option<Region> {
    let centre = Point::new(
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    if !centre.x().is_finite() || !centre.y().is_finite() {
        return None;
    }
    Region::ALL
        .into_iter()
        .find(|region| region.boundary().contains(&centre))
        // The boundaries are simplified separately, so a few metres along a border belong to no
        // region: take a region within the margin.
        .or_else(|| {
            let point = BoundingBox {
                min_e: centre.x(),
                min_n: centre.y(),
                max_e: centre.x(),
                max_n: centre.y(),
            };
            regions_for(&point).into_iter().next()
        })
}

/// The tiles of a `km` grid over `bbox` that touch a region of `provider`; providers publish only
/// those.
pub fn provider_tiles(bbox: &BoundingBox, provider: Provider, km: u32) -> Vec<TileId> {
    crate::tiles::tiles_covering_grid(bbox, km)
        .into_iter()
        .filter(|tile| {
            let rect = rectangle(&tile.bounds_km(km));
            Region::ALL
                .into_iter()
                .filter(|region| region.provider() == provider)
                .any(|region| region.boundary().intersects(&rect))
        })
        .collect()
}

/// Whether `bbox` itself (without the fetch margin) overlaps Berlin.
pub fn touches_berlin(bbox: &BoundingBox) -> bool {
    Region::Berlin.boundary().intersects(&rectangle(bbox))
}

fn rectangle(bbox: &BoundingBox) -> Polygon<f64> {
    Rect::new(
        coord! { x: bbox.min_e, y: bbox.min_n },
        coord! { x: bbox.max_e, y: bbox.max_n },
    )
    .to_polygon()
}

fn parse_boundary(text: &str) -> MultiPolygon<f64> {
    let collection: geojson::FeatureCollection =
        text.parse().expect("embedded boundary is GeoJSON");
    let geometry = collection.features[0]
        .geometry
        .clone()
        .expect("embedded boundary has a geometry");
    MultiPolygon::try_from(geometry.value).expect("embedded boundary is a MultiPolygon")
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
    fn dresden_is_saxony() {
        let school = around(415_973.0, 5_655_745.0, 300.0);
        assert_eq!(regions_for(&school), vec![Region::Saxony]);
        assert_eq!(raster_region(&school), Some(Region::Saxony));
        assert_eq!(Region::Saxony.provider(), Provider::GeoSn);
    }

    #[test]
    fn lausitz_border_needs_both() {
        // A vertex shared by the Brandenburg and Saxony boundaries.
        assert_eq!(
            regions_for(&around(459_509.0, 5_710_017.0, 300.0)),
            vec![Region::Brandenburg, Region::Saxony]
        );
    }

    #[test]
    fn goerlitz_east_edge_is_covered() {
        assert_eq!(
            raster_region(&around(501_000.0, 5_676_000.0, 300.0)),
            Some(Region::Saxony)
        );
    }

    #[test]
    fn berlin_hole_is_not_brandenburg() {
        let mitte = around(391_000.0, 5_820_000.0, 500.0);
        assert!(!regions_for(&mitte).contains(&Region::Brandenburg));
        assert_eq!(raster_region(&mitte), Some(Region::Berlin));
        assert_eq!(Region::Berlin.provider(), Provider::Lgb);
        assert_eq!(Region::Brandenburg.provider(), Provider::Lgb);
    }

    #[test]
    fn raster_region_follows_the_centre() {
        // Centred 50 m south of Berlin's southernmost vertex: Brandenburg, although the box
        // reaches into Berlin.
        let south = around(407_850.0, 5_799_474.0, 200.0);
        assert!(regions_for(&south).contains(&Region::Berlin));
        assert_eq!(raster_region(&south), Some(Region::Brandenburg));
        let north = around(407_850.0, 5_799_624.0, 200.0);
        assert_eq!(raster_region(&north), Some(Region::Berlin));
    }

    #[test]
    fn outside_every_region_is_refused() {
        // Saxony-Anhalt, west of Brandenburg and Saxony.
        let elsewhere = around(230_000.0, 5_800_000.0, 300.0);
        assert_eq!(raster_region(&elsewhere), None);
        assert!(regions_for(&elsewhere).is_empty());
        let nan = around(f64::NAN, 5_800_000.0, 300.0);
        assert_eq!(raster_region(&nan), None);
    }

    #[test]
    fn centre_in_a_boundary_sliver_has_a_region() {
        // Points that the separately simplified boundaries leave in no region.
        for (e, n) in [(391_595.0, 5_834_335.0), (418_969.0, 5_692_390.0)] {
            assert!(raster_region(&around(e, n, 100.0)).is_some(), "{e} {n}");
        }
    }

    #[test]
    fn provider_tiles_skip_tiles_outside_its_regions() {
        // Centre in Brandenburg 150 m north of the 1 km line; the tile south of it lies in Saxony.
        let border = around(410_500.0, 5_691_150.0, 300.0);
        let at = |east_km, north_km| crate::tiles::TileId { east_km, north_km };
        assert_eq!(
            crate::tiles::tiles_covering(&border),
            [at(410, 5690), at(410, 5691)]
        );
        assert_eq!(provider_tiles(&border, Provider::Lgb, 1), [at(410, 5691)]);
        assert_eq!(provider_tiles(&border, Provider::GeoSn, 2), [at(410, 5690)]);
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
        assert_eq!(
            Region::of_object_id("DESNALK0CrM0003UBL"),
            Some(Region::Saxony)
        );
        assert_eq!(Region::of_object_id("DEST01"), None);
    }

    #[test]
    fn touching_berlin_needs_no_margin() {
        // 50 m south of the boundary: within the fetch margin, but not in Berlin.
        assert!(!touches_berlin(&around(407_850.0, 5_799_424.0, 40.0)));
        assert!(touches_berlin(&around(407_850.0, 5_799_524.0, 40.0)));
    }

    #[test]
    fn boundary_has_all_parts() {
        assert_eq!(Region::Berlin.boundary().0.len(), 4);
        assert_eq!(Region::Berlin.boundary().0[0].exterior().0.len(), 664);
        assert_eq!(
            Region::Brandenburg.boundary().0[0].interiors().len(),
            4,
            "Berlin and small enclaves"
        );
    }
}
