//! Where a map area lies among the regions: which adapters it touches, whose rasters it uses,
//! which tiles of a grid a provider publishes. All boxes and points are in EPSG:25833, where
//! the boundaries are. The functions go through the adapters in the order of the registry, so
//! ties break the same way every time.

use freemapper_georef::crs::{self, BoundingBox};
use freemapper_georef::tiles::{self, TileId};
use geo::{Contains, Intersects};
use geo_types::{MultiPolygon, Point, Polygon, Rect, coord};

use crate::adapter::RegionAdapter;
use crate::registry::Registry;

/// The outline of a region: its boundary in EPSG:25833 and the box around it. Every adapter has
/// one; a crate of adapters can ask the questions below of an outline it holds itself.
pub trait Outline {
    /// The boundary in EPSG:25833.
    fn boundary(&self) -> &MultiPolygon<f64>;

    /// The box around the boundary.
    fn bounds(&self) -> BoundingBox;
}

impl Outline for dyn RegionAdapter + '_ {
    fn boundary(&self) -> &MultiPolygon<f64> {
        RegionAdapter::boundary(self)
    }

    fn bounds(&self) -> BoundingBox {
        RegionAdapter::bounds(self)
    }
}

/// The area is widened by this much before testing, so the 25 m simplification never drops
/// buildings right at the border.
pub const MARGIN_M: f64 = 100.0;

/// Whether `bbox` can meet the boundary of `adapter` at all: the two boxes overlap. Asked
/// before the boundary itself, which has tens of thousands of points; it never changes an
/// answer, since what lies outside the box of a boundary cannot touch it.
pub fn near_bounds<O: Outline + ?Sized>(adapter: &O, bbox: &BoundingBox) -> bool {
    let bounds = adapter.bounds();
    bounds.min_e <= bbox.max_e
        && bbox.min_e <= bounds.max_e
        && bounds.min_n <= bbox.max_n
        && bbox.min_n <= bounds.max_n
}

/// A box that is one point.
pub fn point_box(easting: f64, northing: f64) -> BoundingBox {
    BoundingBox {
        min_e: easting,
        min_n: northing,
        max_e: easting,
        max_n: northing,
    }
}

/// The adapters of `registry` whose ALKIS data covers `bbox` (EPSG:25833), in registry order.
pub fn regions_for_in<'a>(
    registry: &'a Registry,
    bbox: &BoundingBox,
) -> Vec<&'a dyn RegionAdapter> {
    registry
        .adapters()
        .filter(|adapter| touches_region_in(*adapter, bbox))
        .collect()
}

/// Whether `boundary` (EPSG:25833) lies within [`MARGIN_M`] of `bbox`: the rule by which an area
/// needs a region, and an OSM extract.
pub fn within_margin(boundary: &MultiPolygon<f64>, bbox: &BoundingBox) -> bool {
    boundary.intersects(&rectangle(&bbox.buffered(MARGIN_M)))
}

/// Whether the boundary of `adapter` lies within [`MARGIN_M`] of `bbox` (EPSG:25833):
/// [`regions_for_in`] for one adapter.
pub fn touches_region_in<O: Outline + ?Sized>(adapter: &O, bbox: &BoundingBox) -> bool {
    near_bounds(adapter, &bbox.buffered(MARGIN_M)) && within_margin(adapter.boundary(), bbox)
}

/// Whether the point (EPSG:25833) lies inside the boundary of `adapter` itself (no margin).
pub fn contains_point_in<O: Outline + ?Sized>(adapter: &O, easting: f64, northing: f64) -> bool {
    near_bounds(adapter, &point_box(easting, northing))
        && adapter.boundary().contains(&Point::new(easting, northing))
}

/// Whether the centre of the 1 km zone-32 tile (EPSG:25832) lies in or near the region of
/// `adapter`, so its provider must have the tile.
pub fn tile_centre_near<O: Outline + ?Sized>(adapter: &O, tile: TileId) -> bool {
    let centre = crs::ProjectedPoint {
        easting: f64::from(tile.east_km) * 1000.0 + 500.0,
        northing: f64::from(tile.north_km) * 1000.0 + 500.0,
    };
    crs::Crs::Utm33
        .reproject(crs::Crs::Utm32, centre)
        .is_some_and(|p| touches_region_in(adapter, &point_box(p.easting, p.northing)))
}

/// Whether the 1 km zone-32 tile (EPSG:25832) touches the region of `adapter` at all: every
/// tile that can exist for it (a provider delivers whole or clipped tiles for each square that
/// intersects).
pub fn tile_touches_in<O: Outline + ?Sized>(adapter: &O, tile: TileId) -> bool {
    let corner = |de: u32, dn: u32| {
        crs::Crs::Utm33.reproject(
            crs::Crs::Utm32,
            crs::ProjectedPoint {
                easting: f64::from(tile.east_km + de) * 1000.0,
                northing: f64::from(tile.north_km + dn) * 1000.0,
            },
        )
    };
    let corners = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
    let Some(corners) = corners.into_iter().collect::<Option<Vec<_>>>() else {
        return false;
    };
    let around = corners.iter().fold(
        BoundingBox {
            min_e: f64::INFINITY,
            min_n: f64::INFINITY,
            max_e: f64::NEG_INFINITY,
            max_n: f64::NEG_INFINITY,
        },
        |b, p| BoundingBox {
            min_e: b.min_e.min(p.easting),
            min_n: b.min_n.min(p.northing),
            max_e: b.max_e.max(p.easting),
            max_n: b.max_n.max(p.northing),
        },
    );
    if !near_bounds(adapter, &around) {
        return false;
    }
    let ring: Vec<_> = corners
        .iter()
        .map(|p| coord! { x: p.easting, y: p.northing })
        .collect();
    let square = Polygon::new(geo::LineString::new(ring), Vec::new());
    adapter.boundary().intersects(&square)
}

/// The adapter of `registry` whose rasters a map of `bbox` uses: the first that contains its
/// centre.
pub fn raster_region_in<'a>(
    registry: &'a Registry,
    bbox: &BoundingBox,
) -> Option<&'a dyn RegionAdapter> {
    let (e, n) = (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    if !e.is_finite() || !n.is_finite() {
        return None;
    }
    registry
        .adapters()
        .find(|adapter| contains_point_in(*adapter, e, n))
        // The boundaries are simplified separately, so a few metres along a border belong to no
        // region: take a region within the margin.
        .or_else(|| {
            regions_for_in(registry, &point_box(e, n))
                .into_iter()
                .next()
        })
}

/// The tiles of a `km` grid over `bbox` that touch a region of `registry` whose provider is
/// `provider`; providers publish only those.
pub fn provider_tiles_in(
    registry: &Registry,
    bbox: &BoundingBox,
    provider: &str,
    km: u32,
) -> Vec<TileId> {
    let regions: Vec<&dyn RegionAdapter> = registry
        .adapters()
        .filter(|adapter| adapter.info().provider.as_str() == provider)
        .collect();
    tiles::tiles_covering_grid(bbox, km)
        .into_iter()
        .filter(|tile| {
            let bounds = tile.bounds_km(km);
            let rect = rectangle(&bounds);
            regions.iter().any(|adapter| {
                near_bounds(*adapter, &bounds) && adapter.boundary().intersects(&rect)
            })
        })
        .collect()
}

/// How far a point must lie from every supported region to count as outside them all. The
/// boundaries are simplified separately (up to 25 m each), so a shared border can leave a gap of
/// up to about 50 m between two states.
const UNCOVERED_M: f64 = 50.0;
/// The spacing of the points [`reaches_beyond_coverage`] tests, half of [`UNCOVERED_M`].
const COVERAGE_STEP_M: f64 = 25.0;

/// Whether part of `bbox` (EPSG:25833) lies outside every region of `registry`: a point of it
/// more than [`UNCOVERED_M`] from all their boundaries, e.g. in Bremen (a hole in
/// Niedersachsen), in Thuringia or another state that is no region yet, abroad or at sea.
pub fn reaches_beyond_coverage_in(registry: &Registry, bbox: &BoundingBox) -> bool {
    use geo::{BooleanOps as _, Distance as _, Euclidean};
    // Every boundary part within UNCOVERED_M of a point of `bbox` lies in this rectangle.
    let grown = bbox.buffered(UNCOVERED_M);
    let near = rectangle(&grown);
    let parts: Vec<MultiPolygon<f64>> = registry
        .adapters()
        .filter(|adapter| near_bounds(*adapter, &grown) && adapter.boundary().intersects(&near))
        .map(|adapter| adapter.boundary().intersection(&near))
        .collect();
    let steps = |min: f64, max: f64| {
        let n = ((max - min) / COVERAGE_STEP_M).ceil().max(1.0) as u32;
        (0..=n).map(move |i| (min + f64::from(i) * COVERAGE_STEP_M).min(max))
    };
    steps(bbox.min_e, bbox.max_e).any(|e| {
        steps(bbox.min_n, bbox.max_n).any(|n| {
            let point = Point::new(e, n);
            !parts.iter().any(|part| part.contains(&point))
                && parts
                    .iter()
                    .all(|part| Euclidean.distance(&point, part) > UNCOVERED_M)
        })
    })
}

/// Whether `bbox` itself (without the fetch margin) overlaps the boundary of `adapter`.
pub fn overlaps_in<O: Outline + ?Sized>(adapter: &O, bbox: &BoundingBox) -> bool {
    near_bounds(adapter, bbox) && adapter.boundary().intersects(&rectangle(bbox))
}

pub fn rectangle(bbox: &BoundingBox) -> Polygon<f64> {
    Rect::new(
        coord! { x: bbox.min_e, y: bbox.min_n },
        coord! { x: bbox.max_e, y: bbox.max_n },
    )
    .to_polygon()
}
