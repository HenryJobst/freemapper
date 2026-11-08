//! ALKIS water parcels → 301.1 uncrossable water with 301.6 bank lines (spec §4.5).
//!
//! A river is split into many parcels, so the parts are dissolved into one shape first: the
//! fill and the bank line are separate symbols, and the bank line runs only along the shore,
//! never along a parcel boundary or the clip edge (the map edge). Parcels that do not quite meet,
//! such as two states' water at a border in mid-river, are closed up before the bank lines are
//! drawn.

use geo::algorithm::buffer::{Buffer as _, BufferStyle, LineJoin};
use geo::algorithm::validation::Validation as _;
use geo::orient::Direction;
use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Euclidean, Intersects as _, Length as _,
    Orient as _,
};
use geo_types::{Coord, LineString, MultiLineString, MultiPolygon, Polygon, Rect, coord};

use crate::build::out_of_bounds::{LandClip, mean_width, nutzart_label};
use crate::crs::{BoundingBox, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::sources::alkis::LandUse;

/// ISSprOM 301.1 uncrossable body of water, fill only.
const FILL_CODE: &str = "301.1";
/// ISSprOM 301.6 bank line of uncrossable water.
const BANK_CODE: &str = "301.6";
/// ISSprOM 301 uncrossable body of water, the combined symbol the OSM rules use for areas.
const OSM_AREA_CODE: &str = "301";
/// ISSprOM 311 small fountain or well, the OSM rules' symbol for `amenity=fountain` nodes and
/// `man_made=water_well`.
const OSM_POINT_CODE: &str = "311";
/// Land uses that are open water, with any `bez`. ALKIS has no depth: all of it is 301, never 302.
pub const WATER_LAND_USES: [&str; 4] =
    ["Fließgewässer", "Stehendes Gewässer", "Hafenbecken", "Meer"];
/// Parts narrower than this mean width (2·area/perimeter) are candidates: the ISSprOM 305 limit
/// for a crossable watercourse. It replaces `bez`, which BB, SN and ST do not give.
pub const NARROW_WATER_M: f64 = 2.0;
/// A segment with both ends this close to the same edge of the clip rectangle lies on it.
const EDGE_TOLERANCE_M: f64 = 0.01;
/// Parcel parts are cut down to the clip window grown by this much before they are dissolved.
const NEAR_WINDOW_M: f64 = 5.0;
/// A parcel part counts as a source of a dissolved piece above this overlap; neighbours that
/// only touch the piece share no area.
const MIN_OVERLAP_M2: f64 = 0.01;
/// Gaps between water parcels narrower than this are closed: the parcels of two states rarely
/// share their vertices at a border in mid-river. Real gaps are wider, the narrowest a bridge
/// of 0.8 m.
const SEAM_TOLERANCE_M: f64 = 0.5;
/// Corners sharper than this get a bevel instead of a mitre while gaps are closed (radians,
/// about 6°).
const MITRE_MIN_ANGLE: f64 = 0.1;
/// A closed gap is grown by this much before it joins the water, so it overlaps both sides; the
/// growth is cut back to the closing.
const GAP_OVERLAP_M: f64 = 0.01;

pub fn is_water(land: &LandUse) -> bool {
    WATER_LAND_USES.contains(&land.nutzart.as_str())
}

/// Water objects: map objects where ALKIS is reliable, candidates for the narrow parts.
#[derive(Debug)]
pub struct WaterObjects {
    /// 301.1 areas (islands as holes) and their 301.6 bank lines.
    pub map: Vec<MapObject>,
    /// The same for parts narrower than [`NARROW_WATER_M`]: ditches, or slivers of a river.
    pub candidates: Vec<MapObject>,
    /// The 301.1 areas of `map` as drawn (clipped, slivers and sliver holes dropped), in the
    /// map's CRS; OSM water areas inside it are duplicates.
    pub union: MultiPolygon<f64>,
}

/// A water part with the parcel it belongs to.
type Part<'a> = (&'a LandUse, Polygon<f64>);

/// 301.1 + 301.6 objects for the water parcels of `land_use`, clipped to `clip` (map area plus
/// buffer, the map's CRS).
pub fn water_objects(
    land_use: &[LandUse],
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<WaterObjects, GeorefError> {
    let symbols = Symbols {
        fill: symbol_set
            .id_for_code(FILL_CODE)
            .expect("ISSprOM defines 301.1"),
        bank: symbol_set
            .id_for_code(BANK_CODE)
            .expect("ISSprOM defines 301.6"),
    };
    let land_clip = LandClip::new(clip, georef);
    let near = Rect::new(
        coord! { x: clip.min_e - NEAR_WINDOW_M, y: clip.min_n - NEAR_WINDOW_M },
        coord! { x: clip.max_e + NEAR_WINDOW_M, y: clip.max_n + NEAR_WINDOW_M },
    );
    let (mut wide, mut narrow): (Vec<Part>, Vec<Part>) = (Vec::new(), Vec::new());
    for land in land_use.iter().filter(|l| is_water(l)) {
        for part in &land.footprint.0 {
            // One winding order for all parts: the union's fill rule follows the first ring.
            let part = part.orient(Direction::Default);
            // Also skips NaN areas and degenerate rings (a point, a line, a bow tie).
            let area = part.unsigned_area();
            if area.is_nan() || area <= 0.0 {
                continue;
            }
            // Classified by the whole part, so a river does not turn narrow at the map edge.
            let parts = if mean_width(&part) >= NARROW_WATER_M {
                &mut wide
            } else {
                &mut narrow
            };
            // The WFS returns whole parcels, and a `Meer` parcel can be far larger than the
            // map: cut it down near the window, so the union and the provenance test cost what
            // the map needs. The result inside the window is the same.
            if let Some(part) = cut_near(&part, &near) {
                parts.push((land, part));
            }
        }
    }
    let clipper = Clipper {
        clip: &land_clip,
        bbox: clip,
        georef,
        symbols,
    };
    // Wide water is no sliver down to 2 m on the ground, even where 1 mm on paper is more
    // (4 m at 1:4000): a mill stream or a small pond stays on the map. A candidate is narrow by
    // definition, so only the area limit drops it.
    let (map, drawn) = clipper.objects(&dissolve(&wide), &wide, NARROW_WATER_M)?;
    let (candidates, _) = clipper.objects(&dissolve(&narrow), &narrow, 0.0)?;
    Ok(WaterObjects {
        map,
        candidates,
        union: MultiPolygon(drawn),
    })
}

/// `part` cut to the rectangle `near`, or `None` if nothing of it is inside.
///
/// Each ring is cut on its own (Sutherland-Hodgman), not with a boolean operation: that snaps
/// every vertex to a grid that depends on the part's extent, so a cut parcel would no longer
/// meet its uncut neighbour on their shared edge, and the two would not dissolve. Vertices
/// inside `near` stay exactly as they are. A concave ring can leave zero-width runs along the
/// edges of `near`; they lie outside the clip window, which cuts the union again.
fn cut_near(part: &Polygon<f64>, near: &Rect<f64>) -> Option<Polygon<f64>> {
    let bounds = part.bounding_rect()?;
    if contains(near, &bounds) {
        return Some(part.clone());
    }
    if !bounds.intersects(near) {
        return None;
    }
    let exterior = cut_ring(part.exterior(), near)?;
    let holes = part
        .interiors()
        .iter()
        .filter_map(|hole| cut_ring(hole, near))
        .collect();
    Some(Polygon::new(exterior, holes))
}

/// The edges of a rectangle, each with its coordinate.
#[derive(Clone, Copy)]
enum Edge {
    West(f64),
    East(f64),
    South(f64),
    North(f64),
}

impl Edge {
    fn inside(self, c: Coord<f64>) -> bool {
        match self {
            Self::West(x) => c.x >= x,
            Self::East(x) => c.x <= x,
            Self::South(y) => c.y >= y,
            Self::North(y) => c.y <= y,
        }
    }

    /// Where the segment from `p` to `q` crosses the edge. The ends are put in one order first, so a
    /// shared edge walked the other way by the neighbour parcel gives the same point.
    fn crossing(self, p: Coord<f64>, q: Coord<f64>) -> Coord<f64> {
        let (a, b) = if (p.x, p.y) <= (q.x, q.y) {
            (p, q)
        } else {
            (q, p)
        };
        match self {
            Self::West(x) | Self::East(x) => Coord {
                x,
                y: a.y + (x - a.x) / (b.x - a.x) * (b.y - a.y),
            },
            Self::South(y) | Self::North(y) => Coord {
                x: a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x),
                y,
            },
        }
    }
}

/// The part of the closed `ring` inside `near`, or `None` if fewer than three points are left.
fn cut_ring(ring: &LineString<f64>, near: &Rect<f64>) -> Option<LineString<f64>> {
    let mut points = ring.0.clone();
    if ring.is_closed() {
        points.pop();
    }
    for edge in [
        Edge::West(near.min().x),
        Edge::East(near.max().x),
        Edge::South(near.min().y),
        Edge::North(near.max().y),
    ] {
        let mut kept = Vec::with_capacity(points.len());
        for (i, &current) in points.iter().enumerate() {
            let previous = points[(i + points.len() - 1) % points.len()];
            match (edge.inside(previous), edge.inside(current)) {
                (true, true) => kept.push(current),
                (true, false) => kept.push(edge.crossing(previous, current)),
                (false, true) => {
                    kept.push(edge.crossing(previous, current));
                    kept.push(current);
                }
                (false, false) => {}
            }
        }
        points = kept;
        if points.len() < 3 {
            return None;
        }
    }
    Some(LineString::new(points))
}

/// `outer` contains all of `inner`.
fn contains(outer: &Rect<f64>, inner: &Rect<f64>) -> bool {
    outer.min().x <= inner.min().x
        && outer.min().y <= inner.min().y
        && inner.max().x <= outer.max().x
        && inner.max().y <= outer.max().y
}

/// The union of `parts`: one shape per connected body of water, with gaps narrower than
/// [`SEAM_TOLERANCE_M`] closed.
fn dissolve(parts: &[Part]) -> MultiPolygon<f64> {
    close_gaps(&geo::unary_union(parts.iter().map(|(_, part)| part)))
}

/// `shape` with its gaps, inlets and holes narrower than [`SEAM_TOLERANCE_M`] filled.
///
/// The gaps are what a morphological closing adds: `shape` grown by half the tolerance and
/// shrunk again. Only those pieces join `shape`, so every other vertex stays exactly where the
/// union put it; the closing's own rounding leaves slivers below [`MIN_OVERLAP_M2`], which are
/// dropped.
fn close_gaps(shape: &MultiPolygon<f64>) -> MultiPolygon<f64> {
    if shape.0.is_empty() {
        return shape.clone();
    }
    let style = |distance| BufferStyle::new(distance).line_join(LineJoin::Miter(MITRE_MIN_ANGLE));
    let closed = shape
        .buffer_with_style(style(SEAM_TOLERANCE_M / 2.0))
        .buffer_with_style(style(-SEAM_TOLERANCE_M / 2.0));
    let gaps: Vec<Polygon<f64>> = closed
        .difference(shape)
        .0
        .into_iter()
        .filter(|gap| gap.unsigned_area() > MIN_OVERLAP_M2)
        .collect();
    if gaps.is_empty() {
        return shape.clone();
    }
    // Grown so they overlap the water on both sides, then cut back to the closing, so they do
    // not bulge out where a gap meets the shore.
    let grown = MultiPolygon(gaps).buffer_with_style(style(GAP_OVERLAP_M));
    shape.union(&grown.intersection(&closed))
}

fn rings(polygon: &Polygon<f64>) -> impl Iterator<Item = &LineString<f64>> {
    std::iter::once(polygon.exterior()).chain(polygon.interiors())
}

#[derive(Clone, Copy)]
struct Symbols {
    fill: u32,
    bank: u32,
}

struct Clipper<'a> {
    clip: &'a LandClip,
    bbox: &'a BoundingBox,
    georef: &'a Georeferencing,
    symbols: Symbols,
}

impl Clipper<'_> {
    /// Fill and bank line objects for the pieces of `shape` inside the clip window, tagged with
    /// the parcels of `sources` they come from, and the pieces drawn.
    ///
    /// Slivers are dropped (`LandClip::is_sliver`): pieces with the width limit capped at
    /// `width_cap_m`, and holes with it capped at [`NARROW_WATER_M`]. The hole test is meant for
    /// gaps between parcels that do not quite meet (centimetres wide); an island or a mole of
    /// 2 m mean width stays, even where that is under 1 mm on paper.
    fn objects(
        &self,
        shape: &MultiPolygon<f64>,
        sources: &[Part],
        width_cap_m: f64,
    ) -> Result<(Vec<MapObject>, Vec<Polygon<f64>>), GeorefError> {
        let mut objects = Vec::new();
        let mut drawn = Vec::new();
        if shape.0.is_empty() {
            return Ok((objects, drawn));
        }
        for piece in shape.intersection(&self.clip.polygon).0 {
            let islands = piece
                .interiors()
                .iter()
                .filter(|hole| {
                    !self
                        .clip
                        .is_sliver(&Polygon::new((*hole).clone(), vec![]), NARROW_WATER_M)
                })
                .cloned()
                .collect();
            let piece = Polygon::new(piece.exterior().clone(), islands);
            if self.clip.is_sliver(&piece, width_cap_m) {
                continue;
            }
            let tags = provenance(
                sources
                    .iter()
                    .filter(|(_, part)| overlaps(part, &piece))
                    .map(|(land, _)| *land),
            );
            let mut fill = Vec::new();
            let mut banks = Vec::new();
            for ring in rings(&piece) {
                fill.extend(closed_ring(&self.to_map(&ring.0)?));
                for (line, closed) in bank_lines(ring, self.bbox) {
                    let points = self.to_map(&line)?;
                    banks.push(if closed {
                        closed_ring(&points)
                    } else {
                        points.iter().map(|&(x, y)| MapCoord::new(x, y)).collect()
                    });
                }
            }
            objects.push(self.path(self.symbols.fill, fill, &tags));
            for coords in banks {
                objects.push(self.path(self.symbols.bank, coords, &tags));
            }
            drawn.push(piece);
        }
        Ok((objects, drawn))
    }

    fn to_map(&self, coords: &[Coord<f64>]) -> Result<Vec<(i32, i32)>, GeorefError> {
        coords
            .iter()
            .map(|c| {
                self.georef
                    .projected_to_map(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .map(|m| (m.x, m.y))
            })
            .collect()
    }

    fn path(&self, symbol_id: u32, coords: Vec<MapCoord>, tags: &[(String, String)]) -> MapObject {
        MapObject {
            symbol_id,
            kind: MapObjectKind::Path,
            coords,
            tags: tags.to_vec(),
        }
    }
}

/// `part` and `piece` share area, not only a boundary.
fn overlaps(part: &Polygon<f64>, piece: &Polygon<f64>) -> bool {
    let near = match (part.bounding_rect(), piece.bounding_rect()) {
        (Some(a), Some(b)) => a.intersects(&b),
        _ => false,
    };
    near && part.intersection(piece).unsigned_area() > MIN_OVERLAP_M2
}

/// The provenance tags of a piece dissolved from `lands`: the sorted oids, the distinct
/// `nutzart / bez` and the newest update.
fn provenance<'a>(lands: impl Iterator<Item = &'a LandUse>) -> Vec<(String, String)> {
    let mut oids = Vec::new();
    let mut nutzarten = Vec::new();
    let mut currency = None;
    for land in lands {
        oids.push(land.oid.clone());
        nutzarten.push(nutzart_label(land));
        currency = currency.max(land.updated);
    }
    for list in [&mut oids, &mut nutzarten] {
        list.sort();
        list.dedup();
    }
    let mut tags = vec![
        ("freemapper:source".to_string(), "alkis-nutzung".to_string()),
        ("freemapper:source_id".to_string(), oids.join(";")),
        ("freemapper:nutzart".to_string(), nutzarten.join(";")),
    ];
    if let Some(updated) = currency {
        tags.push(("freemapper:currency".to_string(), updated.to_string()));
    }
    tags
}

/// The bank lines of a closed `ring`, without its segments on the clip rectangle `bbox`: the
/// ring itself if no segment lies there (`true`, closed), else the open runs between them.
fn bank_lines(ring: &LineString<f64>, bbox: &BoundingBox) -> Vec<(Vec<Coord<f64>>, bool)> {
    let points = &ring.0;
    if points.len() < 3 {
        return Vec::new();
    }
    let near = |a: f64, b: f64, edge: f64| {
        (a - edge).abs() <= EDGE_TOLERANCE_M && (b - edge).abs() <= EDGE_TOLERANCE_M
    };
    let on_edge: Vec<bool> = points
        .windows(2)
        .map(|s| {
            let (p, q) = (s[0], s[1]);
            near(p.x, q.x, bbox.min_e)
                || near(p.x, q.x, bbox.max_e)
                || near(p.y, q.y, bbox.min_n)
                || near(p.y, q.y, bbox.max_n)
        })
        .collect();
    let Some(first_cut) = on_edge.iter().position(|&cut| cut) else {
        return vec![(points.clone(), true)];
    };
    // Walk once around the ring, starting after a cut segment, so no run wraps around.
    let segments = on_edge.len();
    let mut lines = Vec::new();
    let mut run: Vec<Coord<f64>> = Vec::new();
    for k in 1..=segments {
        let i = (first_cut + k) % segments;
        if on_edge[i] {
            if run.len() >= 2 {
                lines.push((std::mem::take(&mut run), false));
            }
            run.clear();
        } else {
            if run.is_empty() {
                run.push(points[i]);
            }
            run.push(points[i + 1]);
        }
    }
    lines
}

/// An OSM 301 area with at least this share of its area (inside the clip window) lying in ALKIS
/// water is a duplicate of it.
pub const OSM_WATER_COVERED: f64 = 0.5;

/// `objects` without the OSM 301 areas that ALKIS water already covers.
///
/// `union` is the drawn ALKIS water ([`WaterObjects::union`]), clipped to `clip`; the OSM area is
/// clipped to the same window first, so a lake reaching beyond it counts by its visible part.
/// Everything else (other symbols, areas that are not from OSM, areas outside the window) stays.
pub fn drop_covered_water(
    objects: Vec<MapObject>,
    union: &MultiPolygon<f64>,
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    let Some(water_id) = symbol_set.id_for_code(OSM_AREA_CODE) else {
        return objects;
    };
    if union.0.is_empty() {
        return objects;
    }
    let window = LandClip::new(clip, georef).polygon;
    objects
        .into_iter()
        .filter(|object| {
            if object.symbol_id != water_id || !from_osm(object) || object.coords.len() < 4 {
                return true;
            }
            let area = projected_area(object, georef);
            // A self-intersecting ring (a bow tie) has no reliable area: keep it for the mapper.
            if !area.is_valid() {
                return true;
            }
            let visible = area.intersection(&window);
            let visible_area = visible.unsigned_area();
            if visible_area <= 0.0 {
                return true;
            }
            visible.intersection(union).unsigned_area() / visible_area < OSM_WATER_COVERED
        })
        .collect()
}

/// The parts of `line` outside `water`: ISSprOM draws no contours inside uncrossable water.
///
/// `water` is the drawn ALKIS water ([`WaterObjects::union`]). A line that does not touch it is
/// returned unchanged; pieces of a cut line shorter than `min_piece_m` (stubs along the shore)
/// are dropped.
pub fn outside_water(
    line: &LineString<f64>,
    water: &MultiPolygon<f64>,
    min_piece_m: f64,
) -> Vec<LineString<f64>> {
    if water.0.is_empty() || line.0.len() < 2 || !line.intersects(water) {
        return vec![line.clone()];
    }
    let mut pieces: Vec<LineString<f64>> = water
        .clip(&MultiLineString(vec![line.clone()]), true)
        .0
        .into_iter()
        .filter(|piece| piece.0.len() >= 2)
        .collect();
    // A closed line is cut at its start point as well: join the pieces that meet there.
    if line.is_closed() && pieces.len() > 1 {
        let start = line.0[0];
        let at = |c: Coord<f64>| (c.x - start.x).hypot(c.y - start.y) < EDGE_TOLERANCE_M;
        let ending = pieces
            .iter()
            .position(|p| at(*p.0.last().expect("2 coords")));
        let starting = pieces.iter().position(|p| at(p.0[0]));
        if let (Some(ending), Some(starting)) = (ending, starting)
            && ending != starting
        {
            let tail = pieces[starting].0[1..].to_vec();
            pieces[ending].0.extend(tail);
            pieces.remove(starting);
        }
    }
    pieces
        .into_iter()
        .filter(|piece| Euclidean.length(piece) >= min_piece_m)
        .collect()
}

/// `objects` without the OSM 311 points that stand in water: the jets of a fountain in its basin,
/// or a well in a pond. Water is the drawn ALKIS water `union` and every OSM 301 area among
/// `objects`, so call it after [`drop_covered_water`]. A fountain without a basin stays.
pub fn drop_points_in_water(
    objects: Vec<MapObject>,
    union: &MultiPolygon<f64>,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    let (Some(area_id), Some(point_id)) = (
        symbol_set.id_for_code(OSM_AREA_CODE),
        symbol_set.id_for_code(OSM_POINT_CODE),
    ) else {
        return objects;
    };
    let basins: Vec<Polygon<f64>> = objects
        .iter()
        .filter(|o| o.symbol_id == area_id && from_osm(o) && o.coords.len() >= 4)
        .map(|o| projected_area(o, georef))
        .collect();
    objects
        .into_iter()
        .filter(|object| {
            let is_point = matches!(object.kind, MapObjectKind::Point { .. });
            if object.symbol_id != point_id || !is_point || !from_osm(object) {
                return true;
            }
            let Some(&c) = object.coords.first() else {
                return true;
            };
            let p = georef.map_to_projected(c);
            let point = geo_types::Point::new(p.easting, p.northing);
            !(point.intersects(union) || basins.iter().any(|b| point.intersects(b)))
        })
        .collect()
}

fn from_osm(object: &MapObject) -> bool {
    object
        .tags
        .iter()
        .any(|(k, v)| k == "freemapper:source" && v == "osm")
}

/// The ring of an area object in the map's CRS (no holes: OSM areas come from closed ways).
fn projected_area(object: &MapObject, georef: &Georeferencing) -> Polygon<f64> {
    let ring: Vec<Coord<f64>> = object
        .coords
        .iter()
        .map(|&c| {
            let p = georef.map_to_projected(c);
            Coord {
                x: p.easting,
                y: p.northing,
            }
        })
        .collect();
    Polygon::new(LineString(ring), vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::{Crs, LatLon, ProjectedPoint};
    use crate::omap::MapObjectKind;
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
    use crate::project::MapKind;
    use geo::Centroid as _;
    use geo_types::{LineString, Polygon};

    struct Fixture {
        georef: Georeferencing,
        clip: BoundingBox,
        centre: (f64, f64),
    }

    /// 1:4000 in Strausberg Vorstadt; clip 300 × 300 m around the reference point.
    fn fixture() -> Fixture {
        let date = "2026-11-08".parse().unwrap();
        let georef = Georeferencing::new(
            LatLon {
                lat: 52.5385,
                lon: 13.844,
            },
            4000,
            date,
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        Fixture {
            clip: BoundingBox {
                min_e: e - 150.0,
                min_n: n - 150.0,
                max_e: e + 150.0,
                max_n: n + 150.0,
            },
            georef,
            centre: (e, n),
        }
    }

    fn ring(e: f64, n: f64, w: f64, h: f64) -> LineString<f64> {
        LineString::from(vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)])
    }

    fn land(oid: &str, nutzart: &str, bez: Option<&str>, parts: Vec<Polygon<f64>>) -> LandUse {
        LandUse {
            oid: oid.to_string(),
            nutzart: nutzart.to_string(),
            bez: bez.map(str::to_string),
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(parts),
        }
    }

    /// A rectangle `w` × `h` m with its south-west corner `(de, dn)` from the fixture's centre.
    fn rect(f: &Fixture, de: f64, dn: f64, w: f64, h: f64) -> Polygon<f64> {
        let (e, n) = f.centre;
        Polygon::new(ring(e + de, n + dn, w, h), vec![])
    }

    fn water(f: &Fixture, land_use: &[LandUse]) -> (WaterObjects, SymbolSet) {
        let set = MapKind::Sprint.symbol_set();
        let objects = water_objects(land_use, &f.clip, &f.georef, &set).unwrap();
        (objects, set)
    }

    fn with_code<'a>(objects: &'a [MapObject], set: &SymbolSet, code: &str) -> Vec<&'a MapObject> {
        let id = set.id_for_code(code).unwrap();
        objects.iter().filter(|o| o.symbol_id == id).collect()
    }

    /// Number of rings (sub-paths ended by the hole flag) of a path object.
    fn rings(object: &MapObject) -> usize {
        object
            .coords
            .iter()
            .filter(|c| c.flags & FLAG_HOLE != 0)
            .count()
    }

    fn is_closed(object: &MapObject) -> bool {
        object
            .coords
            .last()
            .is_some_and(|c| c.flags == FLAG_HOLE | FLAG_CLOSE)
    }

    fn projected(f: &Fixture, object: &MapObject) -> Vec<ProjectedPoint> {
        object
            .coords
            .iter()
            .map(|c| f.georef.map_to_projected(*c))
            .collect()
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn water_parcels_become_uncrossable_water() {
        let f = fixture();
        // All four land uses, with Hamburg's `Fluß` and without `bez` (BB, SN, ST).
        for (nutzart, bez) in [
            ("Fließgewässer", Some("Fluß")),
            ("Fließgewässer", None),
            ("Stehendes Gewässer", Some("Teich")),
            ("Hafenbecken", None),
            ("Meer", None),
        ] {
            let pond = land(
                "DEBBAL01",
                nutzart,
                bez,
                vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
            );
            let (objects, set) = water(&f, &[pond]);
            assert!(objects.candidates.is_empty(), "{nutzart}");
            let fills = with_code(&objects.map, &set, "301.1");
            let banks = with_code(&objects.map, &set, "301.6");
            assert_eq!(
                (fills.len(), banks.len(), objects.map.len()),
                (1, 1, 2),
                "{nutzart}"
            );
            assert_eq!(fills[0].kind, MapObjectKind::Path);
            assert_eq!(rings(fills[0]), 1);
            assert!(is_closed(banks[0]), "a pond's bank line is closed");
            assert_eq!(banks[0].coords.len(), 5);
            assert!((objects.union.unsigned_area() - 1200.0).abs() < 1e-6);
        }
        // The school set (125 %) has both symbols too.
        let set = MapKind::Schoolyard.symbol_set();
        let pond = land(
            "DEBBAL01",
            "Meer",
            None,
            vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
        );
        let objects = water_objects(&[pond], &f.clip, &f.georef, &set).unwrap();
        assert_eq!(objects.map.len(), 2);
    }

    #[test]
    fn adjacent_parcels_are_dissolved() {
        let f = fixture();
        // One river in three parcels; the middle one overlaps its neighbour by 1 cm.
        let land_use = [
            land(
                "DEBBAL03",
                "Fließgewässer",
                None,
                vec![rect(&f, -90.0, -10.0, 60.0, 20.0)],
            ),
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -30.01, -10.0, 60.01, 20.0)],
            ),
            land(
                "DEBBAL02",
                "Fließgewässer",
                None,
                vec![rect(&f, 30.0, -10.0, 60.0, 20.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!((fills.len(), banks.len()), (1, 1));
        assert_eq!(rings(fills[0]), 1);
        // No bank line point inside the river at the parcel boundaries.
        let (e, n) = f.centre;
        for p in projected(&f, banks[0]) {
            let inside = (p.northing - n).abs() < 9.9;
            let at_boundary = [-30.0, 30.0]
                .iter()
                .any(|de| (p.easting - (e + de)).abs() < 0.1);
            assert!(!(inside && at_boundary), "{p:?}");
        }
        assert_eq!(objects.union.0.len(), 1);
    }

    /// A river 80 m wide right across the clip window, split along its axis at the state border
    /// (Rhineland-Palatinate south, Hesse north, as at Mainz), the halves `gap` m apart (negative:
    /// overlapping).
    fn river_halves(f: &Fixture, gap: f64) -> [LandUse; 2] {
        let half = |oid: &str, dn: f64| {
            land(
                oid,
                "Fließgewässer",
                Some("Fluss"),
                vec![rect(f, -1000.0, dn, 2000.0, 40.0 - gap / 2.0)],
            )
        };
        [half("DERPAL01", -40.0), half("DEHEAL01", gap / 2.0)]
    }

    /// The bank lines of `objects`, and whether any of their points lies in the river's middle.
    fn banks_and_seam(f: &Fixture, objects: &WaterObjects, set: &SymbolSet) -> (usize, bool) {
        let banks = with_code(&objects.map, set, "301.6");
        let seam = banks.iter().any(|bank| {
            projected(f, bank)
                .iter()
                .any(|p| (p.northing - f.centre.1).abs() < 30.0)
        });
        (banks.len(), seam)
    }

    #[test]
    fn no_bank_line_where_two_states_water_meets() {
        let f = fixture();
        for gap in [0.2, -0.2, 0.01, 0.45] {
            let (objects, set) = water(&f, &river_halves(&f, gap));
            assert_eq!(
                with_code(&objects.map, &set, "301.1").len(),
                1,
                "gap {gap}: one fill"
            );
            assert_eq!(
                banks_and_seam(&f, &objects, &set),
                (2, false),
                "gap {gap}: the north and the south bank only"
            );
            assert_eq!(objects.union.0.len(), 1, "gap {gap}");
            assert_eq!(objects.union.0[0].interiors().len(), 0, "gap {gap}");
        }
    }

    #[test]
    fn no_bank_line_where_the_border_zigzags_between_gap_and_overlap() {
        let f = fixture();
        let (e, n) = f.centre;
        // The south half reaches 0.2 m over the border in the west and stops 0.2 m short of it
        // in the east; the north half starts at the border.
        let south = Polygon::new(
            LineString::from(vec![
                (e - 1000.0, n - 40.0),
                (e + 1000.0, n - 40.0),
                (e + 1000.0, n - 0.2),
                (e + 7.0, n - 0.2),
                (e - 7.0, n + 0.2),
                (e - 1000.0, n + 0.2),
                (e - 1000.0, n - 40.0),
            ]),
            vec![],
        );
        let land_use = [
            land("DERPAL01", "Fließgewässer", None, vec![south]),
            land(
                "DEHEAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -1000.0, 0.0, 2000.0, 40.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        assert_eq!(with_code(&objects.map, &set, "301.1").len(), 1);
        assert_eq!(banks_and_seam(&f, &objects, &set), (2, false));
    }

    #[test]
    fn a_gap_across_the_river_leaves_the_shore_straight() {
        let f = fixture();
        // A 40 m river split across its width, the two parcels 0.2 m apart: the closed gap meets
        // both shores and must not bulge out of them.
        let part = |oid: &str, de: f64, w: f64| {
            land(
                oid,
                "Fließgewässer",
                None,
                vec![rect(&f, de, -20.0, w, 40.0)],
            )
        };
        let land_use = [
            part("DERPAL01", -1000.0, 999.9),
            part("DEHEAL01", 0.1, 999.9),
        ];
        let (objects, set) = water(&f, &land_use);
        assert_eq!(with_code(&objects.map, &set, "301.1").len(), 1);
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(banks.len(), 2, "the north and the south bank");
        let n = f.centre.1;
        for bank in banks {
            for p in projected(&f, bank) {
                let off = (p.northing - (n - 20.0))
                    .abs()
                    .min((p.northing - (n + 20.0)).abs());
                // Map coordinates are 1 µm on paper, 4 mm on the ground at 1:4000.
                assert!(off < 0.005, "{p:?} is {off} m off the shore");
            }
        }
    }

    #[test]
    fn water_across_a_real_gap_keeps_both_banks() {
        let f = fixture();
        // 5 m of land between the halves, and a bridge of 0.8 m, the narrowest real gap.
        for gap in [5.0, 0.8] {
            let (objects, set) = water(&f, &river_halves(&f, gap));
            assert_eq!(
                with_code(&objects.map, &set, "301.1").len(),
                2,
                "gap {gap}: two fills"
            );
            assert_eq!(
                banks_and_seam(&f, &objects, &set),
                (4, true),
                "gap {gap}: both banks of the land strip"
            );
            assert_eq!(objects.union.0.len(), 2, "gap {gap}");
        }
    }

    #[test]
    fn no_bank_line_on_the_clip_edge() {
        let f = fixture();
        // A 20 m river right across the clip window, and a lake reaching over its north-east corner.
        let land_use = [
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -1000.0, -10.0, 2000.0, 20.0)],
            ),
            land(
                "DEBBAL02",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, 100.0, 100.0, 200.0, 200.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 2);
        // River: the north and the south bank; lake: one line from the north to the east edge.
        assert_eq!(banks.len(), 3);
        for bank in &banks {
            assert!(!is_closed(bank), "a cut-off ring is open");
            let points = projected(&f, bank);
            for pair in points.windows(2) {
                let on_edge =
                    |a: f64, b: f64, edge: f64| (a - edge).abs() < 0.05 && (b - edge).abs() < 0.05;
                let (p, q) = (pair[0], pair[1]);
                assert!(
                    !(on_edge(p.easting, q.easting, f.clip.min_e)
                        || on_edge(p.easting, q.easting, f.clip.max_e)
                        || on_edge(p.northing, q.northing, f.clip.min_n)
                        || on_edge(p.northing, q.northing, f.clip.max_n)),
                    "segment {p:?} – {q:?} lies on the clip edge"
                );
            }
        }
        let lake = banks
            .iter()
            .find(|b| b.coords.len() == 3)
            .expect("the lake's bank line has three points");
        let corner = projected(&f, lake)[1];
        assert!(
            (corner.easting - (f.centre.0 + 100.0)).abs() < 0.01,
            "{corner:?}"
        );
        assert!(
            (corner.northing - (f.centre.1 + 100.0)).abs() < 0.01,
            "{corner:?}"
        );
    }

    #[test]
    fn islands_get_bank_lines() {
        let f = fixture();
        let (e, n) = f.centre;
        let lake = Polygon::new(
            ring(e - 50.0, n - 40.0, 100.0, 80.0),
            vec![ring(e - 10.0, n - 10.0, 20.0, 20.0)],
        );
        let (objects, set) = water(
            &f,
            &[land("DEBBAL01", "Stehendes Gewässer", None, vec![lake])],
        );
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 1);
        assert_eq!(rings(fills[0]), 2, "the island is a hole");
        assert_eq!(fills[0].coords.len(), 10);
        assert_eq!(banks.len(), 2, "shore and island");
        assert!(banks.iter().all(|b| is_closed(b)));
    }

    #[test]
    fn three_metre_island_is_kept_at_sprint_scale() {
        let f = fixture();
        let (e, n) = f.centre;
        // A 3 × 60 m island (a mole) is under 1 mm (4 m) on paper at 1:4000, but 2 m wide land
        // is a visible feature, like 2 m wide water.
        let lake = Polygon::new(
            ring(e - 50.0, n - 40.0, 100.0, 80.0),
            vec![ring(e - 1.5, n - 30.0, 3.0, 60.0)],
        );
        let (objects, set) = water(
            &f,
            &[land("DEBBAL01", "Stehendes Gewässer", None, vec![lake])],
        );
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 1);
        assert_eq!(rings(fills[0]), 2, "the island is a hole");
        assert_eq!(banks.len(), 2, "shore and island");
        assert!((objects.union.unsigned_area() - (8000.0 - 180.0)).abs() < 1e-6);
    }

    /// Two `Meer` parcels 100 × 50 km, split 10 m east of the centre, south of a coast with a
    /// vertex every metre that waves through the clip window.
    fn sea(f: &Fixture) -> Vec<LandUse> {
        let (e, n) = f.centre;
        let coast = |x: f64| n - 40.0 + 20.0 * (x / 30.0).sin();
        let parcel = |oid: &str, from: f64, to: f64| {
            let mut points: Vec<(f64, f64)> = (0..=(to - from) as i64)
                .map(|k| {
                    let x = from + k as f64;
                    (e + x, coast(x))
                })
                .collect();
            points.push((e + to, n - 50_000.0));
            points.push((e + from, n - 50_000.0));
            points.push((e + from, coast(from)));
            land(oid, "Meer", None, vec![Polygon::new(points.into(), vec![])])
        };
        vec![
            parcel("DEBBAL01", -50_000.0, 10.0),
            parcel("DEBBAL02", 10.0, 50_000.0),
        ]
    }

    /// An object's symbol, sorted coordinates and tags.
    type Signature = (u32, Vec<(i32, i32)>, Vec<(String, String)>);

    /// Every object's [`Signature`], in a comparable order.
    fn signature(objects: &[MapObject]) -> Vec<Signature> {
        let mut out: Vec<_> = objects
            .iter()
            .map(|o| {
                let mut coords: Vec<_> = o.coords.iter().map(|c| (c.x, c.y)).collect();
                coords.sort_unstable();
                (o.symbol_id, coords, o.tags.clone())
            })
            .collect();
        out.sort();
        out
    }

    #[test]
    fn the_cut_keeps_the_vertices_inside_exactly() {
        let f = fixture();
        let (e, n) = f.centre;
        let near = Rect::new(
            coord! { x: e - 155.0, y: n - 155.0 },
            coord! { x: e + 155.0, y: n + 155.0 },
        );
        // Uneven coordinates, half of the part far outside, with a hole across the edge.
        let outline: Vec<(f64, f64)> = vec![
            (e - 20.317, n + 0.693),
            (e + 20.411, n + 0.281),
            (e + 31.077, n + 140.913),
            (e + 300.123, n - 10_000.77),
            (e - 250.456, n - 10_000.33),
            (e - 20.317, n + 0.693),
        ];
        let hole = ring(e - 10.123, n - 170.456, 9.789, 40.321);
        let part = Polygon::new(LineString::from(outline.clone()), vec![hole.clone()]);
        let cut = cut_near(&part, &near).expect("part of it is inside");
        let original: Vec<Coord<f64>> = super::rings(&part).flat_map(|r| r.0.clone()).collect();
        let strictly_inside = |c: &Coord<f64>| {
            c.x > near.min().x && c.x < near.max().x && c.y > near.min().y && c.y < near.max().y
        };
        for c in super::rings(&cut)
            .flat_map(|r| r.0.iter())
            .filter(|c| strictly_inside(c))
        {
            assert!(original.contains(c), "{c:?} moved");
        }
        let expected = part.intersection(&near.to_polygon()).unsigned_area();
        // The boolean operation snaps to a grid, so it is only close.
        assert!((cut.unsigned_area() - expected).abs() < 0.01);
        assert_eq!(cut.interiors().len(), 1);
        // Inside or outside as a whole: unchanged or nothing.
        let pond = Polygon::new(ring(e, n, 10.0, 10.0), vec![]);
        assert_eq!(cut_near(&pond, &near), Some(pond));
        let far = Polygon::new(ring(e + 1000.0, n, 10.0, 10.0), vec![]);
        assert_eq!(cut_near(&far, &near), None);
    }

    #[test]
    fn a_long_parcel_still_joins_its_neighbour() {
        let f = fixture();
        let (e, n) = f.centre;
        // A pond inside the window on a river parcel that runs 10 km south: they share an edge
        // with uneven coordinates, so a cut that moves the shared vertices leaves a gap.
        let (west, east) = ((e - 20.317, n + 0.693), (e + 20.411, n + 0.281));
        let pond = Polygon::new(
            LineString::from(vec![
                west,
                east,
                (e + 20.411, n + 30.587),
                (e - 20.317, n + 30.587),
                west,
            ]),
            vec![],
        );
        let river = Polygon::new(
            LineString::from(vec![
                west,
                (e - 250.456, n - 10_000.33),
                (e + 300.123, n - 10_000.77),
                east,
                west,
            ]),
            vec![],
        );
        let (objects, set) = water(
            &f,
            &[
                land("DEBBAL01", "Stehendes Gewässer", None, vec![pond]),
                land("DEBBAL02", "Fließgewässer", None, vec![river]),
            ],
        );
        let fills = with_code(&objects.map, &set, "301.1");
        assert_eq!(fills.len(), 1, "one body of water");
        assert_eq!(
            tag(fills[0], "freemapper:source_id"),
            Some("DEBBAL01;DEBBAL02")
        );
        assert_eq!(objects.union.0.len(), 1);
    }

    #[test]
    fn parcels_far_larger_than_the_window_give_the_same_water() {
        let f = fixture();
        let sea = sea(&f);
        // The same parcels cut down to the window plus 50 m by hand.
        let grown = geo_types::Rect::new(
            geo_types::coord! { x: f.clip.min_e - 50.0, y: f.clip.min_n - 50.0 },
            geo_types::coord! { x: f.clip.max_e + 50.0, y: f.clip.max_n + 50.0 },
        )
        .to_polygon();
        let cut: Vec<LandUse> = sea
            .iter()
            .map(|l| LandUse {
                footprint: l.footprint.intersection(&grown),
                ..l.clone()
            })
            .collect();
        let started = std::time::Instant::now();
        let (whole, set) = water(&f, &sea);
        let elapsed = started.elapsed();
        let (small, _) = water(&f, &cut);
        assert_eq!(with_code(&whole.map, &set, "301.1").len(), 1);
        assert_eq!(with_code(&whole.map, &set, "301.6").len(), 1);
        // Equal up to the rounding of the boolean operations: 1 map unit is 4 mm at 1:4000.
        let (a, b) = (signature(&whole.map), signature(&small.map));
        assert_eq!(a.len(), b.len());
        for ((sa, ca, ta), (sb, cb, tb)) in a.iter().zip(&b) {
            assert_eq!((sa, ta, ca.len()), (sb, tb, cb.len()));
            for (p, q) in ca.iter().zip(cb) {
                assert!(
                    (p.0 - q.0).abs() <= 2 && (p.1 - q.1).abs() <= 2,
                    "{p:?} {q:?}"
                );
            }
        }
        assert!(whole.candidates.is_empty() && small.candidates.is_empty());
        assert!((whole.union.unsigned_area() - small.union.unsigned_area()).abs() < 1e-3);
        assert_eq!(
            tag(&whole.map[0], "freemapper:source_id"),
            Some("DEBBAL01;DEBBAL02")
        );
        // About 0.1 s in a debug build; the whole parcels took twice as long before the cut.
        assert!(elapsed < std::time::Duration::from_secs(3), "{elapsed:?}");
    }

    #[test]
    fn three_metre_stream_is_drawn() {
        let f = fixture();
        // 3 m is wide water, but under 1 mm (4 m) on paper at 1:4000. The 0.5 m left of a lake
        // inside the clip window is a sliver.
        let land_use = [
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -30.0, 20.0, 60.0, 3.0)],
            ),
            land(
                "DEBBAL02",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, 149.5, -20.0, 100.0, 40.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        assert!(objects.candidates.is_empty());
        let fills = with_code(&objects.map, &set, "301.1");
        assert_eq!(fills.len(), 1);
        assert_eq!(tag(fills[0], "freemapper:source_id"), Some("DEBBAL01"));
        assert_eq!(with_code(&objects.map, &set, "301.6").len(), 1);
        // The union is what is drawn: the stream, not the lake's sliver.
        assert!(
            (objects.union.unsigned_area() - 180.0).abs() < 1e-6,
            "{}",
            objects.union.unsigned_area()
        );
    }

    #[test]
    fn sliver_gaps_inside_water_are_dropped() {
        let f = fixture();
        // Two parcels of a lake around a third that leaves a 5 cm gap: a hole that is no island.
        let land_use = [
            land(
                "DEBBAL01",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -50.0, -40.0, 100.0, 30.0)],
            ),
            land(
                "DEBBAL02",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -50.0, 10.0, 100.0, 30.0)],
            ),
            land(
                "DEBBAL03",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -50.0, -10.0, 30.0, 20.0)],
            ),
            land(
                "DEBBAL04",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -19.95, -10.0, 69.95, 20.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 1);
        assert_eq!(rings(fills[0]), 1);
        assert_eq!(banks.len(), 1);
    }

    #[test]
    fn narrow_water_is_a_candidate() {
        let f = fixture();
        // A 1.5 m ditch in two parcels (no `bez`, as in BB) beside a pond.
        let land_use = [
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -80.0, 50.0, 40.0, 1.5)],
            ),
            land(
                "DEBBAL02",
                "Fließgewässer",
                None,
                vec![rect(&f, -40.0, 50.0, 40.0, 1.5)],
            ),
            land(
                "DEBBAL03",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        assert_eq!(objects.map.len(), 2, "the pond");
        assert_eq!(with_code(&objects.candidates, &set, "301.1").len(), 1);
        assert_eq!(with_code(&objects.candidates, &set, "301.6").len(), 1);
        assert_eq!(objects.candidates.len(), 2);
        assert_eq!(
            tag(&objects.candidates[0], "freemapper:source_id"),
            Some("DEBBAL01;DEBBAL02")
        );
        assert!(
            (objects.union.unsigned_area() - 1200.0).abs() < 1e-6,
            "the ditch is not part of the union"
        );
    }

    #[test]
    fn other_land_use_is_not_water() {
        let f = fixture();
        let land_use: Vec<LandUse> = [
            ("Wald", None),
            ("Wohnbaufläche", None),
            ("Straßenverkehr", None),
            ("Moor", None),
            ("Sumpf", None),
            (
                "Unland/Vegetationslose Fläche",
                Some("Gewässerbegleitfläche"),
            ),
        ]
        .into_iter()
        .map(|(nutzart, bez)| {
            land(
                "DEBBAL01",
                nutzart,
                bez,
                vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
            )
        })
        .collect();
        let (objects, _) = water(&f, &land_use);
        assert!(objects.map.is_empty() && objects.candidates.is_empty());
        assert!(objects.union.0.is_empty());
        assert!(land_use.iter().all(|l| !is_water(l)));
    }

    #[test]
    fn water_tags_carry_provenance() {
        let f = fixture();
        let mut river = land(
            "DEHHAL02",
            "Fließgewässer",
            Some("Fluß"),
            vec![rect(&f, -60.0, -10.0, 60.0, 20.0)],
        );
        river.updated = "2024-01-02".parse().ok();
        let mut stretch = land(
            "DEHHAL01",
            "Fließgewässer",
            None,
            vec![rect(&f, 0.0, -10.0, 60.0, 20.0)],
        );
        stretch.updated = "2025-09-30".parse().ok();
        let mut far = land(
            "DEHHAL03",
            "Hafenbecken",
            None,
            vec![rect(&f, -60.0, 60.0, 30.0, 30.0)],
        );
        far.updated = None;
        let (objects, set) = water(&f, &[river, stretch, far]);
        let expected = [
            ("freemapper:source", "alkis-nutzung"),
            ("freemapper:source_id", "DEHHAL01;DEHHAL02"),
            ("freemapper:nutzart", "Fließgewässer;Fließgewässer / Fluß"),
            ("freemapper:currency", "2025-09-30"),
        ];
        for code in ["301.1", "301.6"] {
            let river = with_code(&objects.map, &set, code)
                .into_iter()
                .find(|o| tag(o, "freemapper:source_id") != Some("DEHHAL03"))
                .unwrap();
            let tags: Vec<(&str, &str)> = river
                .tags
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            assert_eq!(tags, expected, "{code}");
        }
        let harbour = objects
            .map
            .iter()
            .find(|o| tag(o, "freemapper:source_id") == Some("DEHHAL03"))
            .unwrap();
        assert_eq!(tag(harbour, "freemapper:nutzart"), Some("Hafenbecken"));
        assert_eq!(tag(harbour, "freemapper:currency"), None);
    }

    #[test]
    fn invalid_parcels_do_not_crash() {
        let f = fixture();
        let (e, n) = f.centre;
        let bowtie = Polygon::new(
            LineString::from(vec![
                (e, n),
                (e + 30.0, n + 30.0),
                (e + 30.0, n),
                (e, n + 30.0),
                (e, n),
            ]),
            vec![],
        );
        let point = Polygon::new(LineString::from(vec![(e, n), (e, n), (e, n)]), vec![]);
        let flat = Polygon::new(
            LineString::from(vec![(e, n), (e + 10.0, n), (e + 20.0, n), (e, n)]),
            vec![],
        );
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        let land_use = [land(
            "DEBBAL01",
            "Stehendes Gewässer",
            None,
            vec![bowtie, point, flat, empty],
        )];
        let (objects, _) = water(&f, &land_use);
        assert!(objects.map.is_empty() && objects.candidates.is_empty());
        assert!(objects.union.0.is_empty());
    }

    #[test]
    fn ni_fixture_pond_is_drawn() {
        use crate::sources::alkis::parse_land_use;
        use crate::sources::alkis_wfs::{parse_gml, test_support::NI_LAND_USE};
        let features = parse_gml(NI_LAND_USE).unwrap();
        let collection =
            serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string();
        let land_use = parse_land_use(&collection).unwrap();
        let pond = land_use
            .iter()
            .find(|l| l.nutzart == "Stehendes Gewässer")
            .expect("the fixture has a pond");
        let centre = pond.footprint.centroid().unwrap();
        let bounds = pond.footprint.bounding_rect().unwrap();
        let georef = Georeferencing::new_in(
            Crs::Utm32,
            Crs::Utm32
                .inverse(ProjectedPoint {
                    easting: centre.x(),
                    northing: centre.y(),
                })
                .unwrap(),
            4000,
            "2026-11-08".parse().unwrap(),
        )
        .unwrap();
        let clip = BoundingBox {
            min_e: bounds.min().x - 100.0,
            min_n: bounds.min().y - 100.0,
            max_e: bounds.max().x + 100.0,
            max_n: bounds.max().y + 100.0,
        };
        let set = MapKind::Sprint.symbol_set();
        let objects = water_objects(&land_use, &clip, &georef, &set).unwrap();
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 1);
        assert_eq!(
            tag(fills[0], "freemapper:nutzart"),
            Some("Stehendes Gewässer / Teich")
        );
        assert_eq!(
            tag(fills[0], "freemapper:source_id"),
            Some(pond.oid.as_str())
        );
        assert_eq!(tag(fills[0], "freemapper:currency"), Some("2011-09-16"));
        // The pond has an island.
        assert_eq!(rings(fills[0]), 2);
        assert_eq!(banks.len(), 2);
        assert!(banks.iter().all(|b| is_closed(b)));
        assert!(objects.candidates.is_empty());
    }

    /// A rectangle of OSM water (301) with its south-west corner at `(e, n)`.
    fn osm_water(g: &Georeferencing, set: &SymbolSet, e: f64, n: f64, w: f64, h: f64) -> MapObject {
        let pts: Vec<(i32, i32)> = ring(e, n, w, h)
            .coords()
            .map(|c| {
                let m = g
                    .projected_to_map(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .unwrap();
                (m.x, m.y)
            })
            .collect();
        MapObject {
            symbol_id: set.id_for_code("301").unwrap(),
            kind: MapObjectKind::Path,
            coords: closed_ring(&pts),
            tags: vec![("freemapper:source".into(), "osm".into())],
        }
    }

    #[test]
    fn osm_water_inside_alkis_water_is_dropped() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        // ALKIS lake: 100 x 100 m, x and y from -50 to 50 m around the centre.
        let union = MultiPolygon(vec![Polygon::new(
            ring(e - 50.0, n - 50.0, 100.0, 100.0),
            vec![],
        )]);
        let inside = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        // Three fifths in: a duplicate.
        let mostly_inside = osm_water(&f.georef, &set, e + 38.0, n - 10.0, 20.0, 20.0);
        // Only a quarter in.
        let quarter = osm_water(&f.georef, &set, e + 40.0, n + 40.0, 20.0, 20.0);
        let far = osm_water(&f.georef, &set, e - 120.0, n - 120.0, 20.0, 20.0);
        let mut bridge = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        bridge.symbol_id = set.id_for_code("501.5").unwrap();
        let mut alkis = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        alkis.tags = vec![("freemapper:source".into(), "alkis-nutzung".into())];
        let kept = drop_covered_water(
            vec![
                inside,
                mostly_inside,
                quarter.clone(),
                far.clone(),
                bridge.clone(),
                alkis.clone(),
            ],
            &union,
            &f.clip,
            &f.georef,
            &set,
        );
        assert_eq!(kept, vec![quarter, far, bridge, alkis]);
    }

    #[test]
    fn osm_water_is_measured_inside_the_clip_window() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        // The OSM lake lies mostly outside the clip (x from 100 to 300 m); the window ends at
        // 150 m, where the drawn ALKIS water (clipped to it) covers the whole visible part.
        let union = MultiPolygon(vec![Polygon::new(
            ring(e + 100.0, n - 50.0, 50.0, 100.0),
            vec![],
        )]);
        let lake = osm_water(&f.georef, &set, e + 100.0, n - 20.0, 200.0, 40.0);
        let kept = drop_covered_water(vec![lake], &union, &f.clip, &f.georef, &set);
        assert!(kept.is_empty());
    }

    #[test]
    fn drop_covered_water_guards_keep_the_object() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        let union = MultiPolygon(vec![Polygon::new(
            ring(e - 50.0, n - 50.0, 100.0, 100.0),
            vec![],
        )]);
        let lake = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        let keep = |objects: Vec<MapObject>, union: &MultiPolygon<f64>| {
            drop_covered_water(objects.clone(), union, &f.clip, &f.georef, &set) == objects
        };
        // No ALKIS water: nothing to compare with.
        assert!(keep(vec![lake.clone()], &MultiPolygon(vec![])));
        // Fewer than 4 coordinates is no ring.
        let mut short = lake.clone();
        short.coords.truncate(3);
        assert!(keep(vec![short], &union));
        // Entirely outside the clip window: nothing visible to measure.
        let outside = osm_water(&f.georef, &set, e + 400.0, n, 20.0, 20.0);
        assert!(keep(vec![outside], &union));
        // Sanity: the same lake with a valid ring is dropped.
        assert!(!keep(vec![lake], &union));
    }

    #[test]
    fn self_intersecting_osm_ring_is_kept() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        let union = MultiPolygon(vec![Polygon::new(
            ring(e - 50.0, n - 50.0, 100.0, 100.0),
            vec![],
        )]);
        // A bow tie inside the ALKIS lake: invalid, so not measured and not dropped.
        let mut bow = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        bow.coords.swap(1, 2);
        let kept = drop_covered_water(vec![bow.clone()], &union, &f.clip, &f.georef, &set);
        assert_eq!(kept, vec![bow]);
    }

    /// A 100 × 100 m lake around the origin.
    fn lake() -> MultiPolygon<f64> {
        MultiPolygon(vec![Polygon::new(ring(-50.0, -50.0, 100.0, 100.0), vec![])])
    }

    fn length(line: &LineString<f64>) -> f64 {
        Euclidean.length(line)
    }

    #[test]
    fn contours_stop_at_the_shore() {
        // Across the lake from west to east, with a vertex in the water.
        let line = LineString::from(vec![(-80.0, 10.0), (0.0, 10.0), (80.0, 10.0)]);
        let pieces = outside_water(&line, &lake(), 1.0);
        assert_eq!(pieces.len(), 2, "{pieces:?}");
        for piece in &pieces {
            assert!((length(piece) - 30.0).abs() < 1e-6, "{piece:?}");
            assert!(
                piece.coords().all(|c| c.x.abs() >= 50.0 - 1e-9),
                "{piece:?}"
            );
        }
        // A ring inside the water is gone.
        let inside = ring(-20.0, -20.0, 40.0, 40.0);
        assert!(outside_water(&inside, &lake(), 1.0).is_empty());
    }

    #[test]
    fn contours_without_water_unchanged() {
        let line = LineString::from(vec![(-80.0, 60.0), (0.0, 70.0), (80.0, 60.0)]);
        assert_eq!(outside_water(&line, &lake(), 1.0), vec![line.clone()]);
        assert_eq!(
            outside_water(&line, &MultiPolygon(vec![]), 1.0),
            vec![line.clone()]
        );
        // A closed ring around the lake stays one closed ring.
        let around = ring(-60.0, -60.0, 120.0, 120.0);
        assert_eq!(outside_water(&around, &lake(), 1.0), vec![around]);
    }

    #[test]
    fn closed_contour_cut_by_water_is_one_line() {
        // A 20 × 40 m ring starting at (60, 10); only its east part (x > 50) lies outside the
        // lake, across the start point.
        let closed = LineString::from(vec![
            (60.0, 10.0),
            (60.0, 30.0),
            (40.0, 30.0),
            (40.0, -10.0),
            (60.0, -10.0),
            (60.0, 10.0),
        ]);
        let pieces = outside_water(&closed, &lake(), 1.0);
        assert_eq!(pieces.len(), 1, "{pieces:?}");
        assert!((length(&pieces[0]) - 60.0).abs() < 1e-6, "{pieces:?}");
        assert!(!pieces[0].is_closed());
    }

    #[test]
    fn shore_stubs_are_dropped() {
        // Crosses the lake and turns back on the east shore: a 0.6 m stub outside.
        let line = LineString::from(vec![(-80.0, 0.0), (50.2, 0.0), (50.2, 0.2), (40.0, 0.2)]);
        let pieces = outside_water(&line, &lake(), 1.0);
        assert_eq!(pieces.len(), 1, "{pieces:?}");
        assert!((length(&pieces[0]) - 30.0).abs() < 1e-6, "{pieces:?}");
    }

    fn osm_point(g: &Georeferencing, set: &SymbolSet, code: &str, e: f64, n: f64) -> MapObject {
        let m = g
            .projected_to_map(ProjectedPoint {
                easting: e,
                northing: n,
            })
            .unwrap();
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![MapCoord::new(m.x, m.y)],
            tags: vec![("freemapper:source".into(), "osm".into())],
        }
    }

    #[test]
    fn fountain_jets_inside_a_basin_are_dropped() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        // An OSM fountain basin (20 × 20 m) and an ALKIS pond (x from 50 to 80 m).
        let basin = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        let union = MultiPolygon(vec![Polygon::new(
            ring(e + 50.0, n - 15.0, 30.0, 30.0),
            vec![],
        )]);
        let jets = [(0.0, 0.0), (5.0, -5.0), (-8.0, 8.0)]
            .map(|(de, dn)| osm_point(&f.georef, &set, "311", e + de, n + dn));
        let well_in_pond = osm_point(&f.georef, &set, "311", e + 60.0, n);
        let spring_in_pond = osm_point(&f.georef, &set, "312", e + 60.0, n);
        let mut objects = vec![basin.clone(), well_in_pond, spring_in_pond.clone()];
        objects.extend(jets);
        let kept = drop_points_in_water(objects, &union, &f.georef, &set);
        assert_eq!(kept, vec![basin, spring_in_pond]);
    }

    #[test]
    fn free_standing_fountain_is_kept() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        let basin = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        let union = MultiPolygon(vec![Polygon::new(
            ring(e + 50.0, n - 15.0, 30.0, 30.0),
            vec![],
        )]);
        let fountain = osm_point(&f.georef, &set, "311", e - 40.0, n);
        let well = osm_point(&f.georef, &set, "311", e + 30.0, n);
        let objects = vec![basin, fountain, well];
        assert_eq!(
            drop_points_in_water(objects.clone(), &union, &f.georef, &set),
            objects
        );
        // Without any water nothing is dropped either.
        let lone = vec![osm_point(&f.georef, &set, "311", e, n)];
        assert_eq!(
            drop_points_in_water(lone.clone(), &MultiPolygon(vec![]), &f.georef, &set),
            lone
        );
    }
}
