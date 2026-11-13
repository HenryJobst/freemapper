//! Geometry of map objects that several build modules share.

use crate::omap::{MapObject, MapObjectKind};

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
