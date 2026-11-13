//! Geometry of map objects that several build modules share.

use crate::omap::{MapCoord, MapObject, MapObjectKind};

/// A map coordinate as a pair of numbers.
pub(crate) fn xy(c: &MapCoord) -> (f64, f64) {
    (f64::from(c.x), f64::from(c.y))
}

/// The distance from `p` to the segment from `a` to `b`.
pub(crate) fn distance_to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length2 = dx * dx + dy * dy;
    let t = if length2 > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p.0 - a.0 - t * dx).hypot(p.1 - a.1 - t * dy)
}

/// The area of a map object in map coordinates: the first closed run of coordinates is the
/// exterior, the others are holes. `None` for a point or a line too short for a ring.
pub(crate) fn object_polygon(object: &MapObject) -> Option<geo_types::Polygon<f64>> {
    use crate::omap::coord::FLAG_CLOSE;
    if !matches!(object.kind, MapObjectKind::Path) {
        return None;
    }
    let mut rings: Vec<geo_types::LineString<f64>> = Vec::new();
    let mut ring = Vec::new();
    for c in &object.coords {
        ring.push((f64::from(c.x), f64::from(c.y)));
        if c.flags & FLAG_CLOSE != 0 {
            rings.push(std::mem::take(&mut ring).into());
        }
    }
    if !ring.is_empty() {
        rings.push(ring.into());
    }
    let mut rings = rings.into_iter().filter(|r| r.0.len() >= 3);
    Some(geo_types::Polygon::new(rings.next()?, rings.collect()))
}
