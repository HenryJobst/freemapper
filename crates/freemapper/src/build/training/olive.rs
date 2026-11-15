//! Step 4 of the training level: inside olive green the training map draws no details (spec
//! 2026-11-14, "Inside olive green").

use std::collections::{BTreeMap, BTreeSet};

use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Contains as _, Intersects as _,
    MapCoords as _, unary_union,
};
use geo_types::{Coord, LineString, MultiPolygon, Point, Polygon, Rect};

use super::{in_map_units, is_closed, min_length};
use crate::build::conflicts::{Class, Classes, TOLERANCE_MM};
use crate::build::dedup::tag_holds;
use crate::build::geometry::{distance_to_segment, object_polygon, xy};
use crate::build::line_join::TOUCH_M;
use crate::build::open_land::MIN_PIECE_SIDE_MM;
use crate::build::out_of_bounds::short_side;
use crate::derive::vegetation::min_area_m2;
use crate::georef::Georeferencing;
use crate::measure::{PaperMm, SymbolScale, metres_per_mm};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::render::geometry::parts;

/// How far inside the olive green an object still lies on its edge.
pub const OLIVE_EDGE_MM: PaperMm = PaperMm(0.4);
/// The beginning drawn of a way that allows no crossing: this, or the symbol's minimum length if longer.
pub const WAY_STUB_MM: PaperMm = PaperMm(1.0);
/// Area that shall not be entered: the olive green of [`clear_olive`].
const OLIVE_CODE: &str = "520";
/// Canopy: roofs, carports and the passages through buildings.
const CANOPY_CODE: &str = "522";
/// Impassable wall and impassable fence: they bar a way into the olive green.
const IMPASSABLE_CODES: [&str; 2] = ["515", "518"];
/// A canopy whose `freemapper:alkis` names one of these is a passage through a building
/// (`build/state.rs`, `build/berlin.rs`).
const PASSAGE_NAMES: [&str; 2] = ["Durchfahrt", "Arkade"];
/// A canopy whose `freemapper:osm` holds one of these is a passage from OSM (the 522 rules of
/// `assets/rules/osm-candidates.toml`).
const OSM_PASSAGES: [&str; 2] = ["tunnel=building_passage", "covered=*"];

/// Step 4: inside olive green the training map draws no details (spec 2026-11-14, "Inside olive
/// green"). The olive green is the union of the [`OLIVE_CODE`] objects; what lies within
/// [`OLIVE_EDGE_MM`] of its edge is on the edge and stays, so the step works on the union shrunk
/// by that much ([`Inner`]). Text, sheet objects, north lines, buildings, water, watercourses,
/// contours, railways, the olive green itself and the canopies stay. Areas lose their part
/// inside; pieces below the minimum of [`open_ground`](super::open_ground) go. Points inside go. The other lines are
/// cut; pieces below their minimum length go. Ways follow [`olive_ways`]. A passage through a
/// building ([`is_passage`]) that reaches inside goes unless a way that stays runs through it.
/// Returns how many objects went or were cut.
pub(super) fn clear_olive(
    objects: &mut Vec<MapObject>,
    classes: &Classes,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let Some(olive_id) = symbol_set.id_for_code(OLIVE_CODE) else {
        return 0;
    };
    let olive: Vec<Polygon<f64>> = objects
        .iter()
        .filter(|o| o.symbol_id == olive_id)
        .filter_map(object_polygon)
        .map(|p| in_mm(&p))
        .collect();
    if olive.is_empty() {
        return 0;
    }
    let inner = Inner::new(unary_union(&olive).buffer(-OLIVE_EDGE_MM.0));
    if inner.polygons.is_empty() {
        return 0;
    }
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let min_area = min_area_m2(scale) / metres_per_mm(scale_denominator).powi(2);
    let min_side = MIN_PIECE_SIDE_MM.on_paper(scale).0;
    let canopy = symbol_set.id_for_code(CANOPY_CODE);
    let is_way = |o: &MapObject| {
        matches!(
            classes.class(o.symbol_id),
            Class::Road | Class::Path | Class::Stairway
        )
    };

    let mut replaced = olive_ways(objects, &inner, classes, georef, symbol_set);
    let passages: Vec<(usize, Polygon<f64>)> = objects
        .iter()
        .enumerate()
        .filter(|(_, o)| Some(o.symbol_id) == canopy && is_passage(o))
        .filter_map(|(i, o)| Some((i, in_mm(&object_polygon(o)?))))
        .filter(|(_, p)| inner.intersects(p))
        .collect();
    if !passages.is_empty() {
        let ways: Vec<(Rect<f64>, LineString<f64>)> = objects
            .iter()
            .enumerate()
            .filter(|(_, o)| is_way(o))
            .flat_map(|(i, o)| match replaced.get(&i) {
                Some(pieces) => pieces.iter().collect::<Vec<_>>(),
                None => vec![o],
            })
            .flat_map(line_parts)
            .filter_map(|(line, _)| {
                let way = LineString::new(line.points);
                Some((way.bounding_rect()?, way))
            })
            .collect();
        for (i, passage) in passages {
            let Some(frame) = passage.bounding_rect() else {
                continue;
            };
            let through = ways
                .iter()
                .any(|(f, way)| f.intersects(&frame) && way.intersects(&passage));
            if !through {
                replaced.insert(i, Vec::new());
            }
        }
    }
    for (i, object) in objects.iter().enumerate() {
        if replaced.contains_key(&i)
            || Some(object.symbol_id) == canopy
            || is_way(object)
            || stays_in_olive(object, classes, olive_id)
        {
            continue;
        }
        let class = classes.class(object.symbol_id);
        let change = match object.kind {
            MapObjectKind::Point { .. } => object
                .coords
                .first()
                .is_some_and(|c| {
                    let (x, y) = xy(c);
                    inner.contains(Coord {
                        x: x / 1000.0,
                        y: y / 1000.0,
                    })
                })
                .then(Vec::new),
            MapObjectKind::Text { .. } => None,
            MapObjectKind::Path
                if class == Class::OtherArea
                    || (class == Class::Other
                        && !classes.is_line(object.symbol_id)
                        && is_closed(object)) =>
            {
                object_polygon(object)
                    .and_then(|p| inner.cut(&in_mm(&p)))
                    .map(|pieces| {
                        let side = (min_area.sqrt() - TOLERANCE_MM).max(0.0);
                        pieces
                            .iter()
                            .filter(|p| {
                                p.unsigned_area() >= side * side
                                    && short_side(p) >= min_side - TOLERANCE_MM
                            })
                            .map(|p| area_like(object, p))
                            .collect()
                    })
            }
            MapObjectKind::Path => {
                let parts = line_parts(object);
                let runs: Vec<Vec<Run>> = parts.iter().map(|(line, _)| inner.runs(line)).collect();
                runs.iter().flatten().any(|r| r.inside).then(|| {
                    let keep: Vec<Vec<(f64, f64)>> = runs
                        .iter()
                        .map(|runs| {
                            runs.iter()
                                .filter(|r| !r.inside)
                                .map(|r| (r.from, r.to))
                                .collect()
                        })
                        .collect();
                    let min = min_length(object, classes, symbol_set, scale).0;
                    line_pieces(object, &parts, &keep, min)
                })
            }
        };
        if let Some(pieces) = change {
            replaced.insert(i, pieces);
        }
    }
    let changed = replaced.len();
    let before = std::mem::take(objects);
    for (i, object) in before.into_iter().enumerate() {
        match replaced.remove(&i) {
            Some(pieces) => objects.extend(pieces),
            None => objects.push(object),
        }
    }
    changed
}

/// Whether `object` stays inside olive green whatever lies there: text, sheet objects, north
/// lines, buildings, water, watercourses, contours, railways and the olive green itself.
fn stays_in_olive(object: &MapObject, classes: &Classes, olive_id: u32) -> bool {
    use Class::*;
    use freemapper_contract::tags::{SOURCE_KEY, SOURCE_NORTH_LINES, SOURCE_SHEET};
    matches!(object.kind, MapObjectKind::Text { .. })
        || object.symbol_id == olive_id
        || object
            .tags
            .iter()
            .any(|(k, v)| k == SOURCE_KEY && (v == SOURCE_SHEET || v == SOURCE_NORTH_LINES))
        || matches!(
            classes.class(object.symbol_id),
            Building | WaterArea | BankLine | Watercourse | Contour | Railway
        )
}

/// Whether the canopy `object` is a passage through a building: from the cadastre
/// ([`PASSAGE_NAMES`]) or from OSM ([`OSM_PASSAGES`]). Roofs and carports are not.
fn is_passage(object: &MapObject) -> bool {
    object.tags.iter().any(|(k, v)| {
        (k == "freemapper:alkis" && PASSAGE_NAMES.iter().any(|name| v.contains(name)))
            || (k == "freemapper:osm" && OSM_PASSAGES.iter().any(|tag| tag_holds(v, tag)))
    })
}

/// The ways (Road, Path, and Stairway, which links them) inside `inner` (step 4). Their parts
/// inside are taken together as they join: an end within [`TOUCH_M`] of another part. Such a group
/// is a crossing when it meets the edge of `inner` at two places or more that are not barred (an
/// end of a line inside is no place); a crossing stays in full. Of a group that is no crossing, the
/// parts inside go, except the first [`WAY_STUB_MM`] (or the minimum length of the way at the
/// place if longer) inward from each place that is not barred, measured on into the parts it
/// joins ([`StubParts::walk`]). A place is barred when an impassable wall or fence
/// ([`IMPASSABLE_CODES`]) crosses the way within [`OLIVE_EDGE_MM`] of the edge of `inner` and
/// within twice that along the way from the place; the way then ends at the barrier. Pieces below
/// the minimum length go unless an end continues on another way that stays. Returns the pieces of
/// the ways that changed, by index in `objects`.
fn olive_ways(
    objects: &[MapObject],
    inner: &Inner,
    classes: &Classes,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> BTreeMap<usize, Vec<MapObject>> {
    struct WayLine {
        object: usize,
        line: Polyline,
        closed: bool,
        runs: Vec<Run>,
    }
    struct InsidePart {
        line: usize,
        from: f64,
        to: f64,
    }
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let edge = OLIVE_EDGE_MM.0;
    let touch = PaperMm::of_metres(TOUCH_M, scale_denominator).0 + TOLERANCE_MM;
    let lines: Vec<WayLine> = objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            matches!(o.kind, MapObjectKind::Path)
                && matches!(
                    classes.class(o.symbol_id),
                    Class::Road | Class::Path | Class::Stairway
                )
        })
        .flat_map(|(object, o)| {
            line_parts(o).into_iter().map(move |(line, closed)| {
                let runs = inner.runs(&line);
                WayLine {
                    object,
                    line,
                    closed,
                    runs,
                }
            })
        })
        .collect();
    let parts: Vec<InsidePart> = lines
        .iter()
        .enumerate()
        .flat_map(|(k, l)| {
            l.runs.iter().filter(|r| r.inside).map(move |r| InsidePart {
                line: k,
                from: r.from,
                to: r.to,
            })
        })
        .collect();
    if parts.is_empty() {
        return BTreeMap::new();
    }

    // The groups of parts that join.
    let stretches: Vec<Vec<Coord<f64>>> = parts
        .iter()
        .map(|p| lines[p.line].line.between(p.from, p.to))
        .collect();
    let frames: Vec<Option<Rect<f64>>> = stretches
        .iter()
        .map(|s| LineString::new(s.clone()).bounding_rect())
        .collect();
    let mut parent: Vec<usize> = (0..parts.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut join = |a: usize, b: usize| {
        let (a, b) = (root(&mut parent, a), root(&mut parent, b));
        parent[a.max(b)] = a.min(b);
    };
    for a in 0..parts.len() {
        for b in 0..parts.len() {
            let near = match (frames[a], frames[b]) {
                (Some(fa), Some(fb)) => grown(fa, touch).intersects(&fb),
                _ => false,
            };
            let ends = [stretches[a][0], stretches[a][stretches[a].len() - 1]];
            if a != b
                && near
                && ends
                    .iter()
                    .any(|&end| distance_to_line(end, &stretches[b]) <= touch)
            {
                join(a, b);
            }
        }
    }
    // A closed line that starts inside: its first and last part are one.
    for (k, l) in lines.iter().enumerate() {
        let total = l.line.length();
        let first = parts.iter().position(|p| p.line == k && p.from <= 0.0);
        let last = parts
            .iter()
            .position(|p| p.line == k && p.to >= total - 1e-9);
        if l.closed
            && let (Some(first), Some(last)) = (first, last)
        {
            join(first, last);
        }
    }

    // The places where a part meets the edge, and whether each is barred: `Some(at)` with the
    // crossing of the barrier along the line.
    let impassable: Vec<u32> = IMPASSABLE_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let barriers: Vec<(Rect<f64>, Vec<Coord<f64>>)> = objects
        .iter()
        .filter(|o| impassable.contains(&o.symbol_id) && matches!(o.kind, MapObjectKind::Path))
        .flat_map(line_parts)
        .filter_map(|(line, _)| {
            let frame = LineString::new(line.points.clone()).bounding_rect()?;
            Some((frame, line.points))
        })
        .collect();
    let mut crossings: BTreeMap<usize, Vec<f64>> = BTreeMap::new();
    for part in &parts {
        crossings
            .entry(part.line)
            .or_insert_with(|| barrier_crossings(&lines[part.line].line, &barriers));
    }
    let place = |line: usize, at: f64| -> Option<Option<f64>> {
        let way = &lines[line].line;
        if at <= TOLERANCE_MM || at >= way.length() - TOLERANCE_MM {
            return None;
        }
        let barred = crossings[&line]
            .iter()
            .copied()
            .filter(|&q| {
                (q - at).abs() <= 2.0 * edge + TOLERANCE_MM
                    && inner.boundary_distance(way.point(q), edge + TOLERANCE_MM)
                        <= edge + TOLERANCE_MM
            })
            .min_by(|a, b| (a - at).abs().total_cmp(&(b - at).abs()));
        Some(barred)
    };
    let places: Vec<[Option<Option<f64>>; 2]> = parts
        .iter()
        .map(|p| [place(p.line, p.from), place(p.line, p.to)])
        .collect();
    let mut open_places: BTreeMap<usize, usize> = BTreeMap::new();
    for (i, ends) in places.iter().enumerate() {
        let open = ends.iter().filter(|e| matches!(e, Some(None))).count();
        *open_places.entry(root(&mut parent, i)).or_default() += open;
    }

    // What is drawn of the parts of the groups that are no crossing: the stubs, by the length
    // along each part.
    let groups: Vec<usize> = (0..parts.len()).map(|i| root(&mut parent, i)).collect();
    let part_lines: Vec<Polyline> = stretches.iter().cloned().map(Polyline::new).collect();
    let stub_parts = StubParts {
        lines: &part_lines,
        groups: &groups,
        touch,
    };
    let mut drawn: Vec<Vec<(f64, f64)>> = vec![Vec::new(); parts.len()];
    for (i, part) in parts.iter().enumerate() {
        if open_places[&groups[i]] >= 2 {
            continue;
        }
        let object = &objects[lines[part.line].object];
        let stub = WAY_STUB_MM
            .0
            .max(min_length(object, classes, symbol_set, scale).0);
        for (end, at) in [(0, 0.0), (1, part_lines[i].length())] {
            if places[i][end] == Some(None) {
                stub_parts.walk(i, at, stub, &mut BTreeSet::new(), &mut drawn);
            }
        }
    }
    // What goes of each line: the rest of those parts, and at a barred place the stretch to the
    // barrier.
    let mut removed: BTreeMap<usize, Vec<(f64, f64)>> = BTreeMap::new();
    for (i, part) in parts.iter().enumerate() {
        if open_places[&groups[i]] >= 2 {
            continue;
        }
        let gone = removed.entry(part.line).or_default();
        gone.extend(
            complement(&drawn[i], part_lines[i].length())
                .into_iter()
                .map(|(a, b)| (part.from + a, part.from + b)),
        );
        if let Some(Some(barrier)) = places[i][0]
            && barrier < part.from
        {
            gone.push((barrier, part.from));
        }
        if let Some(Some(barrier)) = places[i][1]
            && barrier > part.to
        {
            gone.push((part.to, barrier));
        }
    }
    removed.retain(|_, gone| !gone.is_empty());

    let mut changed: BTreeMap<usize, Vec<MapObject>> = BTreeMap::new();
    let mut by_object: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for &line in removed.keys() {
        by_object.entry(lines[line].object).or_default().push(line);
    }
    for (object_index, _) in by_object {
        let object = &objects[object_index];
        let own: Vec<usize> = (0..lines.len())
            .filter(|&k| lines[k].object == object_index)
            .collect();
        let parts: Vec<(Polyline, bool)> = own
            .iter()
            .map(|&k| (lines[k].line.clone(), lines[k].closed))
            .collect();
        let keep: Vec<Vec<(f64, f64)>> = own
            .iter()
            .map(|k| {
                let total = lines[*k].line.length();
                complement(removed.get(k).map_or(&[][..], Vec::as_slice), total)
            })
            .collect();
        changed.insert(object_index, line_pieces(object, &parts, &keep, 0.0));
    }

    // A piece below the minimum length goes unless an end continues on another way that stays.
    // Each way line that stays, with the changed object and piece it belongs to.
    type Owned = (Option<(usize, usize)>, Vec<Coord<f64>>);
    let left: Vec<Owned> = lines
        .iter()
        .filter(|l| !changed.contains_key(&l.object))
        .map(|l| (None, l.line.points.clone()))
        .chain(changed.iter().flat_map(|(&i, pieces)| {
            pieces.iter().enumerate().flat_map(move |(j, piece)| {
                line_parts(piece)
                    .into_iter()
                    .map(move |(line, _)| (Some((i, j)), line.points))
            })
        }))
        .collect();
    for (&i, pieces) in changed.iter_mut() {
        let min = min_length(&objects[i], classes, symbol_set, scale).0;
        let stays: Vec<bool> = pieces
            .iter()
            .enumerate()
            .map(|(j, piece)| {
                line_parts(piece).iter().all(|(line, _)| {
                    let ends = [line.points[0], line.points[line.points.len() - 1]];
                    line.length() >= min - TOLERANCE_MM
                        || ends.iter().any(|&end| {
                            left.iter().any(|(owner, other)| {
                                *owner != Some((i, j)) && distance_to_line(end, other) <= touch
                            })
                        })
                })
            })
            .collect();
        let mut stays = stays.into_iter();
        pieces.retain(|_| stays.next().unwrap_or(false));
    }
    changed
}

/// The parts of the ways inside olive green along which a stub may run: the line of each part
/// (paper mm, from its start) and its group.
struct StubParts<'a> {
    lines: &'a [Polyline],
    groups: &'a [usize],
    touch: f64,
}

impl StubParts<'_> {
    /// Draws `length` of part `i` from `at` inward (both ways from a point within the part), and
    /// what is left of it on into each part of the group whose line comes within `touch` of the
    /// end reached: at a fork every branch gets the rest. `visited` holds the parts walked.
    fn walk(
        &self,
        i: usize,
        at: f64,
        length: f64,
        visited: &mut BTreeSet<usize>,
        drawn: &mut [Vec<(f64, f64)>],
    ) {
        visited.insert(i);
        let line = &self.lines[i];
        let total = line.length();
        let directions: &[f64] = if at <= TOLERANCE_MM {
            &[1.0]
        } else if at >= total - TOLERANCE_MM {
            &[-1.0]
        } else {
            &[1.0, -1.0]
        };
        for &direction in directions {
            let reach = at + direction * length;
            let end = reach.clamp(0.0, total);
            drawn[i].push((at.min(end), at.max(end)));
            let rest = (reach - end).abs();
            if rest <= TOLERANCE_MM {
                continue;
            }
            let point = line.point(end);
            for k in 0..self.lines.len() {
                if self.groups[k] == self.groups[i]
                    && !visited.contains(&k)
                    && distance_to_line(point, &self.lines[k].points) <= self.touch
                {
                    let from = self.lines[k].locate(point);
                    self.walk(k, from, rest, visited, drawn);
                }
            }
        }
    }
}

/// The places along `line` (length from its start) where a line of `barriers` crosses it.
fn barrier_crossings(line: &Polyline, barriers: &[(Rect<f64>, Vec<Coord<f64>>)]) -> Vec<f64> {
    let mut found = Vec::new();
    for (i, w) in line.points.windows(2).enumerate() {
        let segment = geo_types::Line::new(w[0], w[1]);
        let frame = segment.bounding_rect();
        for (barrier_frame, barrier) in barriers {
            if !barrier_frame.intersects(&frame) {
                continue;
            }
            for b in barrier.windows(2) {
                let hits = match line_intersection(segment, geo_types::Line::new(b[0], b[1])) {
                    None => continue,
                    Some(LineIntersection::SinglePoint { intersection, .. }) => {
                        [intersection, intersection]
                    }
                    Some(LineIntersection::Collinear { intersection }) => {
                        [intersection.start, intersection.end]
                    }
                };
                found.extend(
                    hits.iter()
                        .map(|hit| line.at[i] + (hit.x - w[0].x).hypot(hit.y - w[0].y)),
                );
            }
        }
    }
    found
}

/// The stretches of `0..total` that `removed` leaves.
fn complement(removed: &[(f64, f64)], total: f64) -> Vec<(f64, f64)> {
    let mut removed = removed.to_vec();
    removed.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut keep = Vec::new();
    let mut at = 0.0;
    for (from, to) in removed {
        if from > at {
            keep.push((at, from));
        }
        at = f64::max(at, to);
    }
    if at < total {
        keep.push((at, total));
    }
    keep
}

/// The distance from `point` to the polyline `line`.
fn distance_to_line(point: Coord<f64>, line: &[Coord<f64>]) -> f64 {
    line.windows(2)
        .map(|w| distance_to_segment((point.x, point.y), (w[0].x, w[0].y), (w[1].x, w[1].y)))
        .fold(f64::INFINITY, f64::min)
}

/// `rect` grown by `by` on every side.
fn grown(rect: Rect<f64>, by: f64) -> Rect<f64> {
    Rect::new(
        Coord {
            x: rect.min().x - by,
            y: rect.min().y - by,
        },
        Coord {
            x: rect.max().x + by,
            y: rect.max().y + by,
        },
    )
}

/// A polygon in map coordinates in paper mm.
fn in_mm(polygon: &Polygon<f64>) -> Polygon<f64> {
    polygon.map_coords(|c| Coord {
        x: c.x / 1000.0,
        y: c.y / 1000.0,
    })
}

/// The area `polygon` (paper mm) drawn as `object`.
fn area_like(object: &MapObject, polygon: &Polygon<f64>) -> MapObject {
    let polygon = in_map_units(polygon);
    let mut coords = Vec::new();
    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
        let points: Vec<(i32, i32)> = ring.coords().map(|c| (c.x as i32, c.y as i32)).collect();
        coords.extend(closed_ring(&points));
    }
    MapObject {
        symbol_id: object.symbol_id,
        kind: MapObjectKind::Path,
        coords,
        tags: object.tags.clone(),
    }
}

/// A polyline in paper mm with the length along it at each point.
#[derive(Debug, Clone)]
struct Polyline {
    points: Vec<Coord<f64>>,
    at: Vec<f64>,
}

impl Polyline {
    fn new(points: Vec<Coord<f64>>) -> Self {
        let mut at = Vec::with_capacity(points.len());
        let mut total = 0.0;
        for (i, p) in points.iter().enumerate() {
            if i > 0 {
                total += (p.x - points[i - 1].x).hypot(p.y - points[i - 1].y);
            }
            at.push(total);
        }
        Self { points, at }
    }

    fn length(&self) -> f64 {
        self.at.last().copied().unwrap_or(0.0)
    }

    /// The point `s` along the line.
    fn point(&self, s: f64) -> Coord<f64> {
        let i = self
            .at
            .partition_point(|&a| a < s)
            .clamp(1, self.points.len() - 1);
        let (a, b) = (self.points[i - 1], self.points[i]);
        let span = self.at[i] - self.at[i - 1];
        let t = if span > 0.0 {
            ((s - self.at[i - 1]) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Coord {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
        }
    }

    /// The length along the line to the point of it nearest `c`.
    fn locate(&self, c: Coord<f64>) -> f64 {
        let mut best = (f64::INFINITY, 0.0);
        for (i, w) in self.points.windows(2).enumerate() {
            let span = self.at[i + 1] - self.at[i];
            let (dx, dy) = (w[1].x - w[0].x, w[1].y - w[0].y);
            let t = if span > 0.0 {
                (((c.x - w[0].x) * dx + (c.y - w[0].y) * dy) / (span * span)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let distance = (w[0].x + t * dx - c.x).hypot(w[0].y + t * dy - c.y);
            if distance < best.0 {
                best = (distance, self.at[i] + t * span);
            }
        }
        best.1
    }

    /// The line from `from` to `to` along it.
    fn between(&self, from: f64, to: f64) -> Vec<Coord<f64>> {
        let mut points = vec![self.point(from)];
        points.extend(
            self.points
                .iter()
                .zip(&self.at)
                .filter(|&(_, &a)| a > from + 1e-9 && a < to - 1e-9)
                .map(|(&p, _)| p),
        );
        points.push(self.point(to));
        points
    }
}

/// The parts of the line `object` (Béziers flattened), paper mm, and whether each is closed.
fn line_parts(object: &MapObject) -> Vec<(Polyline, bool)> {
    parts(&object.coords)
        .into_iter()
        .filter(|p| p.points.len() >= 2)
        .map(|p| {
            let points = p.points.iter().map(|&(x, y)| Coord { x, y }).collect();
            (Polyline::new(points), p.closed)
        })
        .collect()
}

/// The pieces of the line `object` that keep the stretches `keep` of each of its `parts`; a
/// closed part cut on both sides of its start is joined there. Pieces of a cut part shorter than
/// `min` (mm on paper) go.
fn line_pieces(
    object: &MapObject,
    parts: &[(Polyline, bool)],
    keep: &[Vec<(f64, f64)>],
    min: f64,
) -> Vec<MapObject> {
    let mut pieces = Vec::new();
    for ((line, closed), keep) in parts.iter().zip(keep) {
        let total = line.length();
        let whole = keep.len() == 1 && keep[0].0 <= 0.0 && keep[0].1 >= total;
        let mut stretches: Vec<Vec<Coord<f64>>> =
            keep.iter().map(|&(a, b)| line.between(a, b)).collect();
        if *closed && keep.len() > 1 && keep[0].0 <= 0.0 && keep[keep.len() - 1].1 >= total {
            let first = stretches.remove(0);
            let last = stretches.last_mut().expect("two stretches or more");
            last.extend_from_slice(&first[1..]);
        }
        for stretch in stretches {
            let drawn = Polyline::new(stretch);
            if !whole && drawn.length() < min - TOLERANCE_MM {
                continue;
            }
            let points: Vec<(i32, i32)> = drawn
                .points
                .iter()
                .map(|c| ((c.x * 1000.0).round() as i32, (c.y * 1000.0).round() as i32))
                .collect();
            pieces.push(MapObject {
                symbol_id: object.symbol_id,
                kind: MapObjectKind::Path,
                coords: if whole && *closed {
                    closed_ring(&points)
                } else {
                    points.iter().map(|&(x, y)| MapCoord::new(x, y)).collect()
                },
                tags: object.tags.clone(),
            });
        }
    }
    pieces
}

/// A stretch of a line, by the length along it, inside or outside [`Inner`].
#[derive(Debug, Clone, Copy)]
struct Run {
    from: f64,
    to: f64,
    inside: bool,
}

/// The olive green shrunk by its edge ([`OLIVE_EDGE_MM`]), paper mm (y down), each polygon with
/// its frame.
struct Inner {
    polygons: Vec<(Rect<f64>, Polygon<f64>)>,
}

impl Inner {
    fn new(union: MultiPolygon<f64>) -> Self {
        let polygons = union
            .0
            .into_iter()
            .filter(|p| p.unsigned_area() > 0.0)
            .filter_map(|p| Some((p.bounding_rect()?, p)))
            .collect();
        Self { polygons }
    }

    /// The polygons whose frame comes within `reach` of `frame`.
    fn near(&self, frame: Rect<f64>, reach: f64) -> impl Iterator<Item = &Polygon<f64>> {
        let frame = grown(frame, reach);
        self.polygons
            .iter()
            .filter(move |(f, _)| f.intersects(&frame))
            .map(|(_, p)| p)
    }

    fn contains(&self, c: Coord<f64>) -> bool {
        let point = Point(c);
        self.near(Rect::new(c, c), 0.0).any(|p| p.contains(&point))
    }

    fn intersects(&self, polygon: &Polygon<f64>) -> bool {
        polygon
            .bounding_rect()
            .is_some_and(|frame| self.near(frame, 0.0).any(|p| p.intersects(polygon)))
    }

    /// The distance from `c` to the edge of the polygons within `reach` of it (infinite when
    /// there is none).
    fn boundary_distance(&self, c: Coord<f64>, reach: f64) -> f64 {
        self.near(Rect::new(c, c), reach)
            .flat_map(|p| std::iter::once(p.exterior()).chain(p.interiors()))
            .map(|ring| distance_to_line(c, &ring.0))
            .fold(f64::INFINITY, f64::min)
    }

    /// The pieces of `polygon` (paper mm) outside, or `None` when no more than the tolerance of
    /// it lies inside.
    fn cut(&self, polygon: &Polygon<f64>) -> Option<Vec<Polygon<f64>>> {
        let near = MultiPolygon(self.near(polygon.bounding_rect()?, 0.0).cloned().collect());
        let area = MultiPolygon(vec![polygon.clone()]);
        (area.intersection(&near).unsigned_area() > TOLERANCE_MM * TOLERANCE_MM)
            .then(|| area.difference(&near).0)
    }

    /// The stretches of `line` inside and outside, in order; a stretch shorter than the
    /// tolerance takes the side of the one before it (the first one that of the next).
    fn runs(&self, line: &Polyline) -> Vec<Run> {
        let mut cuts = vec![0.0];
        for (i, w) in line.points.windows(2).enumerate() {
            let length = line.at[i + 1] - line.at[i];
            if length > 0.0 {
                let segment = geo_types::Line::new(w[0], w[1]);
                let frame = segment.bounding_rect();
                for polygon in self.near(frame, 0.0) {
                    let rings = std::iter::once(polygon.exterior()).chain(polygon.interiors());
                    for edge in rings.flat_map(|r| r.lines()) {
                        if !edge.bounding_rect().intersects(&frame) {
                            continue;
                        }
                        let hits = match line_intersection(segment, edge) {
                            None => continue,
                            Some(LineIntersection::SinglePoint { intersection, .. }) => {
                                [intersection, intersection]
                            }
                            Some(LineIntersection::Collinear { intersection }) => {
                                [intersection.start, intersection.end]
                            }
                        };
                        cuts.extend(hits.iter().map(|hit| {
                            line.at[i] + (hit.x - w[0].x).hypot(hit.y - w[0].y).min(length)
                        }));
                    }
                }
            }
            cuts.push(line.at[i + 1]);
        }
        cuts.sort_by(f64::total_cmp);
        cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let mut runs: Vec<Run> = cuts
            .windows(2)
            .map(|w| Run {
                from: w[0],
                to: w[1],
                inside: self.contains(line.point((w[0] + w[1]) / 2.0)),
            })
            .collect();
        for k in 0..runs.len() {
            if runs[k].to - runs[k].from < TOLERANCE_MM {
                let side = match k {
                    0 => runs.get(1).map(|r| r.inside),
                    _ => Some(runs[k - 1].inside),
                };
                runs[k].inside = side.unwrap_or(false);
            }
        }
        let mut merged: Vec<Run> = Vec::with_capacity(runs.len());
        for run in runs {
            match merged.last_mut() {
                Some(last) if last.inside == run.inside => last.to = run.to,
                _ => merged.push(run),
            }
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::training::tests::{
        SCALE, area, area_mm2, georef_at, isom, length_mm, line, of_code, point, tagged,
    };
    use crate::build::training::{TrainingReport, train};
    use crate::project::MapKind;

    /// The scale of the sprint tests.
    const SPRINT: u32 = 4000;

    /// Only the olive step on `objects`, a map of `set` at `scale`; returns how many went or
    /// were cut.
    fn olive_at(set: &SymbolSet, scale: u32, objects: Vec<MapObject>) -> (Vec<MapObject>, usize) {
        let mut objects = objects;
        let n = clear_olive(&mut objects, &Classes::of(set), &georef_at(scale), set);
        (objects, n)
    }

    fn olive(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, usize) {
        olive_at(set, SPRINT, objects)
    }

    /// A garden of 20 mm × 20 mm (80 m at 1:4000); its inner part starts 0.4 mm inside.
    fn garden(set: &SymbolSet) -> MapObject {
        area(set, "520", (0.0, 0.0), (20.0, 20.0))
    }

    /// The coordinates of a line, mm on paper.
    fn mm_of(object: &MapObject) -> Vec<(f64, f64)> {
        object
            .coords
            .iter()
            .map(|c| (f64::from(c.x) / 1000.0, f64::from(c.y) / 1000.0))
            .collect()
    }

    /// Whether `object` runs straight from `from` to `to` (mm, within 2 map units).
    fn runs_from_to(object: &MapObject, from: (f64, f64), to: (f64, f64)) -> bool {
        let points = mm_of(object);
        let near = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).hypot(a.1 - b.1) <= 0.002;
        points.len() >= 2 && near(points[0], from) && near(points[points.len() - 1], to)
    }

    #[test]
    fn a_fence_inside_goes_and_the_one_on_the_edge_stays() {
        let set = SymbolSet::issprom_2019();
        let edge = line(&set, "518", &[(-5.0, 0.0), (25.0, 0.0)]);
        let objects = vec![
            garden(&set),
            edge.clone(),
            line(&set, "518", &[(5.0, 10.0), (15.0, 10.0)]),
            line(&set, "516", &[(10.0, -10.0), (10.0, 10.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 2, "{out:?}");
        assert!(
            out.contains(&garden(&set)) && out.contains(&edge),
            "{out:?}"
        );
        assert_eq!(of_code(&set, &out, "518"), vec![&edge]);
        let across = of_code(&set, &out, "516");
        assert_eq!(across.len(), 1, "{out:?}");
        assert!(
            runs_from_to(across[0], (10.0, -10.0), (10.0, 0.4)),
            "{:?}",
            mm_of(across[0])
        );
    }

    #[test]
    fn a_fence_ring_round_a_corner_is_one_piece_after() {
        // Closed, starting outside: what is left of it runs on through its start.
        let set = SymbolSet::issprom_2019();
        let ring = MapObject {
            coords: closed_ring(&[(-5000, -5000), (5000, -5000), (5000, 5000), (-5000, 5000)]),
            ..line(&set, "516", &[])
        };
        let (out, n) = olive(&set, vec![garden(&set), ring]);
        assert_eq!(n, 1, "{out:?}");
        let fences = of_code(&set, &out, "516");
        assert_eq!(fences.len(), 1, "{out:?}");
        assert!(
            runs_from_to(fences[0], (0.4, 5.0), (5.0, 0.4)),
            "{:?}",
            mm_of(fences[0])
        );
        assert!((length_mm(fences[0]) - (40.0 - 2.0 * 4.6)).abs() < 0.01);
    }

    #[test]
    fn a_point_inside_goes() {
        let set = SymbolSet::issprom_2019();
        let on_edge = point(&set, "526", (10.0, 0.2));
        let outside = point(&set, "526", (10.0, 30.0));
        let objects = vec![
            garden(&set),
            point(&set, "526", (10.0, 10.0)),
            on_edge.clone(),
            outside.clone(),
        ];
        let (out, report) = train(objects, None, MapKind::Sprint, &georef_at(SPRINT), &set);
        assert_eq!(out, vec![garden(&set), on_edge, outside]);
        assert_eq!(report.inside_olive, 1, "{report:?}");
    }

    #[test]
    fn a_building_and_a_pond_inside_stay() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            area(&set, "521", (5.0, 5.0), (10.0, 10.0)),
            area(&set, "301", (12.0, 12.0), (16.0, 16.0)),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_contour_runs_through() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "101", &[(-5.0, 10.0), (25.0, 10.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn open_land_is_cut_at_the_edge() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            area(&set, "401", (-10.0, 5.0), (10.0, 15.0)),
            area(&set, "401", (5.0, 16.0), (7.0, 18.0)),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 2, "{out:?}");
        let open = of_code(&set, &out, "401");
        assert_eq!(open.len(), 1, "{out:?}");
        let east = open[0].coords.iter().map(|c| c.x).max().unwrap();
        assert!((east - 400).abs() <= 2, "{east}");
        assert!(
            (area_mm2(open[0]) - 104.0).abs() < 0.1,
            "{}",
            area_mm2(open[0])
        );
    }

    #[test]
    fn a_path_that_crosses_stays_in_full() {
        for (set, scale, path) in [
            (SymbolSet::issprom_2019(), SPRINT, "505.1"),
            (isom(), SCALE, "505"),
        ] {
            let objects = vec![
                garden(&set),
                line(&set, path, &[(-5.0, 10.0), (25.0, 10.0)]),
            ];
            assert_eq!(olive_at(&set, scale, objects.clone()), (objects, 0));
        }
    }

    #[test]
    fn a_spur_that_joins_a_crossing_inside_stays() {
        // The spur ends inside on the crossing path: one group with two open places.
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "505.1", &[(-5.0, 10.0), (25.0, 10.0)]),
            line(&set, "506", &[(10.0, 10.0), (10.0, 16.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_driveway_keeps_only_its_beginning() {
        // ISSprOM 505.1 has no minimum length: the stub is 1.0 mm. ISOM 505 is at least 4.25 mm,
        // at 150 % 6.375 mm: the stub is that long.
        for (set, scale, path, stub) in [
            (SymbolSet::issprom_2019(), SPRINT, "505.1", 1.0),
            (isom(), SCALE, "505", 6.375),
        ] {
            let objects = vec![
                garden(&set),
                line(&set, path, &[(10.0, -2.0), (10.0, 12.0)]),
            ];
            let (out, n) = olive_at(&set, scale, objects);
            assert_eq!(n, 1, "{}: {out:?}", set.name);
            let ways = of_code(&set, &out, path);
            assert_eq!(ways.len(), 1, "{}: {out:?}", set.name);
            assert!(
                runs_from_to(ways[0], (10.0, -2.0), (10.0, 0.4 + stub)),
                "{}: {:?}",
                set.name,
                mm_of(ways[0])
            );
        }
    }

    #[test]
    fn a_driveway_behind_an_impassable_fence_goes_whole() {
        let set = SymbolSet::issprom_2019();
        for wall in ["518", "515"] {
            let barrier = line(&set, wall, &[(-5.0, 0.0), (25.0, 0.0)]);
            let objects = vec![
                garden(&set),
                barrier.clone(),
                line(&set, "505.1", &[(10.0, 0.0), (10.0, 12.0)]),
                // From the street: the part on the street side ends at the fence.
                line(&set, "505.1", &[(15.0, -3.0), (15.0, 12.0)]),
            ];
            let (out, n) = olive(&set, objects);
            assert_eq!(n, 2, "{wall}: {out:?}");
            assert!(out.contains(&barrier), "{wall}: {out:?}");
            let ways = of_code(&set, &out, "505.1");
            assert_eq!(ways.len(), 1, "{wall}: {out:?}");
            assert!(
                runs_from_to(ways[0], (15.0, -3.0), (15.0, 0.0)),
                "{wall}: {:?}",
                mm_of(ways[0])
            );
        }
    }

    #[test]
    fn a_crossing_barred_at_one_end_is_no_crossing() {
        let set = SymbolSet::issprom_2019();
        let fence = line(&set, "518", &[(20.0, -5.0), (20.0, 25.0)]);
        let objects = vec![
            garden(&set),
            fence.clone(),
            line(&set, "505.1", &[(-5.0, 10.0), (25.0, 10.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 1, "{out:?}");
        assert!(out.contains(&fence), "{out:?}");
        let ways = of_code(&set, &out, "505.1");
        assert_eq!(ways.len(), 2, "{out:?}");
        assert!(
            runs_from_to(ways[0], (-5.0, 10.0), (1.4, 10.0)),
            "{:?}",
            mm_of(ways[0])
        );
        assert!(
            runs_from_to(ways[1], (20.0, 10.0), (25.0, 10.0)),
            "{:?}",
            mm_of(ways[1])
        );
    }

    #[test]
    fn a_passable_fence_does_not_bar() {
        let set = SymbolSet::issprom_2019();
        let fence = line(&set, "516", &[(-5.0, 0.0), (25.0, 0.0)]);
        let objects = vec![
            garden(&set),
            fence.clone(),
            line(&set, "505.1", &[(10.0, 0.0), (10.0, 12.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 1, "{out:?}");
        assert!(out.contains(&fence), "{out:?}");
        let ways = of_code(&set, &out, "505.1");
        assert_eq!(ways.len(), 1, "{out:?}");
        assert!(
            runs_from_to(ways[0], (10.0, 0.0), (10.0, 1.4)),
            "{:?}",
            mm_of(ways[0])
        );
    }

    /// A passage through the building at the street front into the garden, tagged `key=value`.
    fn passage(set: &SymbolSet, key: &str, value: &str) -> MapObject {
        tagged(area(set, "522", (8.0, -1.0), (12.0, 4.0)), key, value)
    }

    #[test]
    fn a_passage_into_a_yard_goes() {
        let set = SymbolSet::issprom_2019();
        let building = area(&set, "521", (0.0, -1.0), (20.0, 4.0));
        let objects = vec![
            garden(&set),
            building.clone(),
            passage(&set, "freemapper:alkis", "Durchfahrt im Gebäude"),
            passage(&set, "freemapper:alkis", "Arkade"),
            passage(
                &set,
                "freemapper:osm",
                "highway=service; tunnel=building_passage",
            ),
            passage(&set, "freemapper:osm", "highway=footway; covered=arcade"),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(out, vec![garden(&set), building]);
        assert_eq!(n, 4);
    }

    #[test]
    fn a_passage_on_a_crossing_stays() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            passage(&set, "freemapper:alkis", "Durchfahrt im Gebäude"),
            line(&set, "505.1", &[(10.0, -5.0), (10.0, 25.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_roof_inside_stays() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            tagged(
                area(&set, "522", (5.0, 5.0), (8.0, 8.0)),
                "freemapper:alkis",
                "Überdachung",
            ),
            tagged(
                area(&set, "522", (12.0, 12.0), (15.0, 15.0)),
                "freemapper:alkis",
                "Carport",
            ),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn without_olive_nothing_changes() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            line(&set, "518", &[(0.0, 0.0), (10.0, 0.0)]),
            point(&set, "526", (5.0, 10.0)),
            area(&set, "401", (20.0, 20.0), (30.0, 30.0)),
            line(&set, "505.1", &[(50.0, 0.0), (50.0, 10.0)]),
        ];
        let (out, report) = train(
            objects.clone(),
            None,
            MapKind::Sprint,
            &georef_at(SPRINT),
            &set,
        );
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
    }

    #[test]
    fn a_crossing_with_steps_stays() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "505.1", &[(-5.0, 10.0), (8.0, 10.0)]),
            line(&set, "532.6", &[(8.0, 10.0), (12.0, 10.0)]),
            line(&set, "505.1", &[(12.0, 10.0), (25.0, 10.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn steps_in_a_dead_end_go_with_it() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "505.1", &[(10.0, -2.0), (10.0, 8.0)]),
            line(&set, "532.6", &[(10.0, 8.0), (10.0, 12.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 2, "{out:?}");
        assert!(of_code(&set, &out, "532.6").is_empty(), "{out:?}");
        let ways = of_code(&set, &out, "505.1");
        assert_eq!(ways.len(), 1, "{out:?}");
        assert!(
            runs_from_to(ways[0], (10.0, -2.0), (10.0, 1.4)),
            "{:?}",
            mm_of(ways[0])
        );
    }

    #[test]
    fn a_stub_runs_on_into_the_next_piece() {
        // OSM splits the driveway 0.3 mm inside: the beginning drawn is still 1.0 mm long.
        let set = SymbolSet::issprom_2019();
        let first = line(&set, "505.1", &[(10.0, -2.0), (10.0, 0.7)]);
        let objects = vec![
            garden(&set),
            first.clone(),
            line(&set, "505.1", &[(10.0, 0.7), (10.0, 12.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 1, "{out:?}");
        let ways = of_code(&set, &out, "505.1");
        assert_eq!(ways.len(), 2, "{out:?}");
        assert_eq!(ways[0], &first);
        assert!(
            runs_from_to(ways[1], (10.0, 0.7), (10.0, 1.4)),
            "{:?}",
            mm_of(ways[1])
        );
    }

    #[test]
    fn a_way_along_the_edge_that_dips_inside_stays_whole() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(
                &set,
                "505.1",
                &[(-5.0, 0.2), (5.0, 0.2), (6.0, 1.0), (7.0, 0.2), (25.0, 0.2)],
            ),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_driveway_drawn_from_inside_out_keeps_its_outer_end() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "505.1", &[(10.0, 12.0), (10.0, -2.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 1, "{out:?}");
        let ways = of_code(&set, &out, "505.1");
        assert_eq!(ways.len(), 1, "{out:?}");
        assert!(
            runs_from_to(ways[0], (10.0, 1.4), (10.0, -2.0)),
            "{:?}",
            mm_of(ways[0])
        );
    }

    #[test]
    fn a_way_inside_goes_whole() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "505.1", &[(5.0, 5.0), (15.0, 5.0)]),
        ];
        assert_eq!(olive(&set, objects), (vec![garden(&set)], 1));
    }

    #[test]
    fn another_line_inside_goes() {
        // 528 Prominent line feature: of class Other.
        let set = SymbolSet::issprom_2019();
        let objects = vec![garden(&set), line(&set, "528", &[(5.0, 5.0), (15.0, 5.0)])];
        assert_eq!(olive(&set, objects), (vec![garden(&set)], 1));
    }

    #[test]
    fn a_paved_area_is_cut_at_the_edge() {
        // ISSprOM 501 is a combined symbol (class Other), 501.5 an area (class OtherArea).
        let set = SymbolSet::issprom_2019();
        for code in ["501", "501.5"] {
            let objects = vec![garden(&set), area(&set, code, (-10.0, 5.0), (10.0, 15.0))];
            let (out, n) = olive(&set, objects);
            assert_eq!(n, 1, "{code}: {out:?}");
            let paved = of_code(&set, &out, code);
            assert_eq!(paved.len(), 1, "{code}: {out:?}");
            let east = paved[0].coords.iter().map(|c| c.x).max().unwrap();
            assert!((east - 400).abs() <= 2, "{code}: {east}");
        }
    }

    #[test]
    fn a_watercourse_and_a_railway_inside_stay() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "305", &[(5.0, 5.0), (15.0, 5.0)]),
            line(&set, "509.1", &[(5.0, 10.0), (15.0, 10.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_passage_whose_way_went_behind_a_fence_goes() {
        let set = SymbolSet::issprom_2019();
        let fence = line(&set, "518", &[(-5.0, 0.0), (25.0, 0.0)]);
        let objects = vec![
            garden(&set),
            fence.clone(),
            passage(&set, "freemapper:alkis", "Durchfahrt im Gebäude"),
            line(&set, "505.1", &[(10.0, 0.0), (10.0, 12.0)]),
        ];
        assert_eq!(olive(&set, objects), (vec![garden(&set), fence], 2));
    }
}
