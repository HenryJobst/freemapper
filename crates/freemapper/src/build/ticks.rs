//! Which way the ticks of fences and earth banks point (ISSprOM 2019-2 §104, §201, §518).
//!
//! The ticks of a fence (518), an earth bank (104, 104.1), an impassable cliff (201) and a
//! passable retained wall (513.2) sit on
//! the right of the line, seen on paper, so the way a line runs says on which side they are. The
//! sources give the lines in any direction. The last step before the map is written turns them:
//!
//! * The side of a fence's ticks follows what lies beside it, the norm first: a closed fence (a
//!   rectangle, say) has them inside (ISSprOM 2019-2 §516, §518), and so has one whose ends
//!   nearly meet or that is bent round an area (three sides of a plot, an arc round a sports
//!   ground). Else a fence beside an area that shall not be entered (520, olive green) has its
//!   ticks towards that area, else they point away from a road or path running along one side,
//!   else towards a park (401) beside it. Where none of these decides, the same area lies on both sides: the ticks go to the side
//!   where they meet no other symbol that keeps its gap to a fence ([`conflicts::relation`]);
//!   with room on both sides or on neither the fence stays as it is. A decided side stays where
//!   its ticks overdraw something: turning is no means against overdrawing. A fence of the map
//!   looks at the ways and symbols of the map only, one of the candidates at both maps.
//! * The ticks of a bank point down the slope. The terrain model says which side is lower; where
//!   it is missing or flat, the bank takes the way round of the banks beside it that are decided.

use geo::algorithm::buffer::{BufferStyle, LineCap};
use geo::{BooleanOps as _, BoundingRect as _, Buffer as _, Contains as _, Intersects as _};
use geo_types::{Coord, Line, MultiPolygon, Point, Polygon, Rect};

use crate::build::cliffs::{TICKS_BY_PROFILE, TICKS_KEY};
use crate::build::conflicts::{self, Class, Classes, Footprints, Relation, TOLERANCE_MM};
use crate::build::points::HIDDEN_KEY;
use crate::build::symbol_measures::{FENCE_TICK_MM, LINE_GAP_MM};
use crate::crs::ProjectedPoint;
use crate::derive::ndsm::bilinear;
use crate::georef::Georeferencing;
use crate::measure::{PaperMm, SymbolMm, SymbolScale};
use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::raster::Raster;

/// The fence whose ticks point to an area that shall not be entered.
const FENCE_CODE: &str = "518";
/// The lines whose ticks point down the slope (the retained wall has them on its lower side).
const BANK_CODES: [&str; 4] = ["104", "104.1", "201", "513.2"];
/// The index of the map among the lists; the candidates follow.
const MAP_LIST: usize = 0;
/// The area that shall not be entered.
const OLIVE_CODE: &str = "520";
/// The park.
const PARK_CODE: &str = "401";

/// A fence whose ends are no farther apart than this (ground metres) and than [`CLOSE_SHARE`] of
/// its length counts as closed.
const CLOSE_GAP_M: f64 = 3.0;
/// See [`CLOSE_GAP_M`].
const CLOSE_SHARE: f64 = 0.1;
/// A fence is bent round an area when the area between it and the line joining its ends is at
/// least this share of its squared length (a right angle of equal legs: 1/8; a half circle:
/// about 1/20).
const BEND_SHARE: f64 = 0.04;

/// Distance between the places along a line where the sides are looked at (ground metres).
const STEP_M: f64 = 2.0;
/// How far from the fence the sides are probed for an olive green or park area (ground metres).
const AREA_PROBE_M: f64 = 1.0;
/// An object counts as near a fence when its coordinates come within the reach of the ticks plus
/// this much: the largest point symbols reach about as far from their coordinate.
const NEAR_MM: SymbolMm = SymbolMm(3.0);
/// How far from a bank the heights of its two sides are read (ground metres).
const HEIGHT_PROBE_M: f64 = 1.5;
/// The terrain decides when its mean drop across the bank is at least this (metres).
const MIN_DROP_M: f64 = 0.1;
/// Banks within this distance (ground metres) and nearly parallel are neighbours.
const NEIGHBOUR_M: f64 = 3.0;
/// Two directions are nearly parallel above this cosine (about 45 degrees).
const PARALLEL_COS: f64 = 0.7;

type P = (f64, f64);

/// How many lines were turned round.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Turned {
    pub fences: usize,
    pub banks: usize,
}

/// A place on a line and the unit vector of the line there (east, north).
type Sample = (P, P);

/// Turns the fences and banks among `lists` (the objects of the map, first, and of the
/// candidates map) so that their ticks point the right way. `terrain` is the height model of the
/// project.
pub fn orient_ticks(
    lists: &mut [&mut Vec<MapObject>],
    terrain: Option<&Raster<f32>>,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Turned {
    let mut turned = Turned::default();
    let id = |code: &str| symbol_set.id_for_code(code);
    let fence = id(FENCE_CODE);
    let banks: Vec<u32> = BANK_CODES.iter().filter_map(|c| id(c)).collect();
    let areas = |code: &str| -> Vec<Polygon<f64>> {
        let wanted = id(code);
        lists
            .iter()
            .flat_map(|list| list.iter())
            .filter(|o| Some(o.symbol_id) == wanted && matches!(o.kind, MapObjectKind::Path))
            .flat_map(|o| polygons(&o.coords, georef))
            .collect()
    };
    let (olive, park) = (areas(OLIVE_CODE), areas(PARK_CODE));

    let scale = SymbolScale::drawn_with(symbol_set, georef.scale_denominator);
    let grow = (FENCE_TICK_MM + LINE_GAP_MM + NEAR_MM).on_paper(scale).0;
    // The fences that can be turned and their votes: closed or bent → inside, then olive green.
    let mut fences: Vec<FenceLine> = Vec::new();
    for (l, list) in lists.iter().enumerate() {
        for (i, object) in list.iter().enumerate() {
            if Some(object.symbol_id) != fence
                || !matches!(object.kind, MapObjectKind::Path)
                || object.coords.len() < 2
                || !turnable(object)
            {
                continue;
            }
            let line = ground(&object.coords, georef);
            let samples = samples(&line);
            let closed = object
                .coords
                .last()
                .is_some_and(|c| c.flags & FLAG_CLOSE != 0)
                || nearly_closed(&line);
            // The norm first: a closed fence has its ticks inside (§516, §518).
            let vote = match closed {
                true => -signed_area(&line).signum() as i32,
                false => 0,
            };
            let vote = match vote {
                0 => bend_vote(&line),
                vote => vote,
            };
            let vote = match vote {
                0 => area_vote(&samples, &olive),
                vote => vote,
            };
            fences.push(FenceLine {
                list: l,
                index: i,
                coords: object.coords.clone(),
                samples,
                frame: frame(&object.coords, grow),
                vote,
            });
        }
    }
    // Then away from a way, then towards a park, then the side with room.
    if fences.iter().any(|f| f.vote == 0) {
        // A way counts within the reach of the ticks plus the gap; one at the gap does not.
        let reach = (FENCE_TICK_MM + LINE_GAP_MM).on_paper(scale).0 - TOLERANCE_MM;
        let reach_m = PaperMm(reach).metres(georef.scale_denominator);
        let others = Others::of(lists, &fences, symbol_set);
        for f in fences.iter_mut().filter(|f| f.vote == 0) {
            f.vote = others.way_vote(f, reach_m, georef);
        }
        for f in fences.iter_mut().filter(|f| f.vote == 0) {
            f.vote = area_vote(&f.samples, &park);
        }
        // The ticks of the fences still open with something near, drawn either way round.
        let open: Vec<usize> = (0..fences.len())
            .filter(|&n| {
                let f = &fences[n];
                f.vote == 0 && !others.near(f, false).is_empty()
            })
            .collect();
        let both_ways: Vec<MapObject> = open
            .iter()
            .flat_map(|&n| {
                let object = &lists[fences[n].list][fences[n].index];
                let mut reversed = object.clone();
                turn(&mut reversed);
                [object.clone(), reversed]
            })
            .collect();
        let drawn = Footprints::of(&both_ways.iter().collect::<Vec<_>>(), symbol_set);
        for (k, &n) in open.iter().enumerate() {
            fences[n].vote = others.room_vote(&fences[n], &drawn, 2 * k);
        }
    }
    for f in &fences {
        if f.vote < 0 && turn(&mut lists[f.list][f.index]) {
            turned.fences += 1;
        }
    }

    let mut bank_lines: Vec<BankLine> = Vec::new();
    for (l, list) in lists.iter_mut().enumerate() {
        for (i, object) in list.iter_mut().enumerate() {
            if !matches!(object.kind, MapObjectKind::Path) || object.coords.len() < 2 {
                continue;
            }
            if banks.contains(&object.symbol_id) {
                let samples = samples(&ground(&object.coords, georef));
                // A forest cliff turned by its cross profile (`build/cliffs.rs`) is decided.
                let by_profile = object
                    .tags
                    .iter()
                    .any(|(k, v)| k == TICKS_KEY && v == TICKS_BY_PROFILE);
                let turn = if by_profile {
                    Some(false)
                } else {
                    terrain.and_then(|t| downhill_vote(&samples, t))
                };
                bank_lines.push(BankLine {
                    list: l,
                    index: i,
                    samples,
                    turn,
                });
            }
        }
    }

    settle_by_neighbours(&mut bank_lines);
    for bank in bank_lines {
        if bank.turn == Some(true) && turn(&mut lists[bank.list][bank.index]) {
            turned.banks += 1;
        }
    }
    turned
}

/// A fence and its vote: negative when its ticks shall go to the left (it is turned), positive
/// when they stay on the right, 0 while open.
struct FenceLine {
    list: usize,
    index: usize,
    coords: Vec<MapCoord>,
    samples: Vec<Sample>,
    /// The frame of the fence on paper (mm, y down), grown by the reach of its ticks and
    /// [`NEAR_MM`].
    frame: Option<Rect<f64>>,
    vote: i32,
}

/// The objects near the open fences that keep their gap to a fence (their relation to a
/// barrier is not free), as they are drawn: paper mm, y down.
struct Others {
    footprints: Footprints,
    items: Vec<Other>,
}

/// One of [`Others`]: the list and index it came from, whether it is a road or path, the frame
/// of its footprint, and the coordinates of a fence.
struct Other {
    at: (usize, usize),
    way: bool,
    bounds: Option<Rect<f64>>,
    fence: Option<Vec<MapCoord>>,
}

impl Others {
    fn of(lists: &[&mut Vec<MapObject>], fences: &[FenceLine], symbol_set: &SymbolSet) -> Self {
        let classes = Classes::of(symbol_set);
        let fence = symbol_set.id_for_code(FENCE_CODE);
        let frames: Vec<Rect<f64>> = fences
            .iter()
            .filter(|f| f.vote == 0)
            .filter_map(|f| f.frame)
            .collect();
        let (mut objects, mut at) = (Vec::new(), Vec::new());
        for (l, list) in lists.iter().enumerate() {
            for (i, o) in list.iter().enumerate() {
                let class = classes.class(o.symbol_id);
                if conflicts::relation(Class::Barrier, class) == Relation::Free
                    || o.tags.iter().any(|(k, _)| k == HIDDEN_KEY)
                {
                    continue;
                }
                let near = frame(&o.coords, 0.0)
                    .is_some_and(|bounds| frames.iter().any(|f| f.intersects(&bounds)));
                if near {
                    let way = matches!(class, Class::Road | Class::Path)
                        && matches!(o.kind, MapObjectKind::Path);
                    let coords = (Some(o.symbol_id) == fence).then(|| o.coords.clone());
                    objects.push(o);
                    at.push(((l, i), way, coords));
                }
            }
        }
        let footprints = Footprints::of(&objects, symbol_set);
        let items = at
            .into_iter()
            .enumerate()
            .map(|(k, (at, way, fence))| Other {
                at,
                way,
                bounds: footprints.footprint(k).bounding_rect(),
                fence,
            })
            .collect();
        Self { footprints, items }
    }

    /// The footprints of the objects whose frame meets that of `fence`, without the fence itself
    /// and its copy in another list; `ways`: only the roads and paths. A fence of the map looks at
    /// the map only: an uncertain candidate does not turn a sure fence.
    fn near(&self, fence: &FenceLine, ways: bool) -> Vec<&MultiPolygon<f64>> {
        let Some(frame) = fence.frame else {
            return Vec::new();
        };
        self.items
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.at != (fence.list, fence.index)
                    && (fence.list != MAP_LIST || o.at.0 == MAP_LIST)
                    && (o.way || !ways)
                    && !o
                        .fence
                        .as_ref()
                        .is_some_and(|c| same_line(c, &fence.coords))
                    && o.bounds.is_some_and(|b| b.intersects(&frame))
            })
            .map(|(k, _)| self.footprints.footprint(k))
            .collect()
    }

    /// Away from a way: negative when a road or path runs along the right of the fence within
    /// `reach` (ground metres) at no less than half of its samples and along the left at less
    /// than half, positive the other way round, else 0.
    fn way_vote(&self, fence: &FenceLine, reach: f64, georef: &Georeferencing) -> i32 {
        let ways = self.near(fence, true);
        if ways.is_empty() {
            return 0;
        }
        let meets = |s: Sample, distance: f64| {
            let (Some(a), Some(b)) = (paper(s.0, georef), paper(beside(s, distance), georef))
            else {
                return false;
            };
            let probe = Line::new(a, b);
            ways.iter().any(|w| w.intersects(&probe))
        };
        let (mut right, mut left) = (0, 0);
        for &s in &fence.samples {
            right += usize::from(meets(s, reach));
            left += usize::from(meets(s, -reach));
        }
        let n = fence.samples.len();
        let along = |count: usize| 2 * count >= n;
        match (along(right), along(left)) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        }
    }

    /// The side with room: negative when the ticks of `fence` on its left would meet no other
    /// object and on its right they do, positive the other way round, else 0. `drawn` holds the
    /// fence at `k` and the fence run the other way round at `k + 1`.
    fn room_vote(&self, fence: &FenceLine, drawn: &Footprints, k: usize) -> i32 {
        let others = self.near(fence, false);
        let free = |k: usize| {
            let ticks = ticks_of(drawn, k);
            !others.iter().any(|o| o.intersects(&ticks))
        };
        match (free(k), free(k + 1)) {
            (false, true) => -1,
            (true, false) => 1,
            _ => 0,
        }
    }
}

/// The ticks of the line `k` of `drawn` without its line: the footprint less the stroke along the
/// centre line, shrunk by the tolerance so that a symbol that only touches them does not meet
/// them.
fn ticks_of(drawn: &Footprints, k: usize) -> MultiPolygon<f64> {
    let style = BufferStyle::new(drawn.half_width(k) + TOLERANCE_MM).line_cap(LineCap::Butt);
    let line = drawn.centre(k).buffer_with_style(style);
    drawn.footprint(k).difference(&line).buffer(-TOLERANCE_MM)
}

/// Whether two lines run through the same points within the tolerance, either way round.
fn same_line(a: &[MapCoord], b: &[MapCoord]) -> bool {
    let tolerance = PaperMm(TOLERANCE_MM).map_units();
    let close = |p: &MapCoord, q: &MapCoord| {
        f64::from(p.x - q.x).abs() <= tolerance && f64::from(p.y - q.y).abs() <= tolerance
    };
    a.len() == b.len()
        && (a.iter().zip(b).all(|(p, q)| close(p, q))
            || a.iter().zip(b.iter().rev()).all(|(p, q)| close(p, q)))
}

/// The frame of `coords` on paper (mm, y down), grown by `grow` (mm).
fn frame(coords: &[MapCoord], grow: f64) -> Option<Rect<f64>> {
    let mm = |v: i32| f64::from(v) / 1000.0;
    let first = coords.first()?;
    let (mut min, mut max) = ((mm(first.x), mm(first.y)), (mm(first.x), mm(first.y)));
    for c in coords {
        min = (min.0.min(mm(c.x)), min.1.min(mm(c.y)));
        max = (max.0.max(mm(c.x)), max.1.max(mm(c.y)));
    }
    Some(Rect::new(
        Coord {
            x: min.0 - grow,
            y: min.1 - grow,
        },
        Coord {
            x: max.0 + grow,
            y: max.1 + grow,
        },
    ))
}

/// The place on paper (mm, y down) of the ground point `p`.
fn paper(p: P, georef: &Georeferencing) -> Option<Coord<f64>> {
    let c = georef
        .projected_to_map(ProjectedPoint {
            easting: p.0,
            northing: p.1,
        })
        .ok()?;
    Some(Coord {
        x: f64::from(c.x) / 1000.0,
        y: f64::from(c.y) / 1000.0,
    })
}

/// A bank and what is known about its way round.
struct BankLine {
    list: usize,
    index: usize,
    samples: Vec<Sample>,
    /// `Some(true)`: the ticks point the wrong way; `Some(false)`: the right way; `None`: open.
    turn: Option<bool>,
}

/// The ground points (east, north) of a path.
fn ground(coords: &[MapCoord], georef: &Georeferencing) -> Vec<P> {
    coords
        .iter()
        .map(|c| {
            let p = georef.map_to_projected(*c);
            (p.easting, p.northing)
        })
        .collect()
}

/// The areas of a path object: the first ring of a polygon is its outline, the following ones
/// are holes (a ring ends at a close or hole flag).
fn polygons(coords: &[MapCoord], georef: &Georeferencing) -> Vec<Polygon<f64>> {
    let mut rings: Vec<Vec<P>> = Vec::new();
    let mut ring: Vec<P> = Vec::new();
    for c in coords {
        let p = georef.map_to_projected(*c);
        ring.push((p.easting, p.northing));
        if c.flags & (FLAG_CLOSE | FLAG_HOLE) != 0 {
            rings.push(std::mem::take(&mut ring));
        }
    }
    let mut rings = rings.into_iter().filter(|r| r.len() >= 3);
    match rings.next() {
        Some(outline) => vec![Polygon::new(
            outline.into(),
            rings.map(geo_types::LineString::from).collect(),
        )],
        None => Vec::new(),
    }
}

/// Places every [`STEP_M`] along the line, each with the direction of its segment.
fn samples(line: &[P]) -> Vec<Sample> {
    let mut out = Vec::new();
    for w in line.windows(2) {
        let (dx, dy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        let length = dx.hypot(dy);
        if length < 1e-6 {
            continue;
        }
        let direction = (dx / length, dy / length);
        let count = (length / STEP_M).ceil().max(1.0) as usize;
        for k in 0..count {
            let t = (k as f64 + 0.5) / count as f64;
            out.push(((w[0].0 + dx * t, w[0].1 + dy * t), direction));
        }
    }
    out
}

/// The point `distance` to the right of `at` (positive) or to the left (negative); the right of
/// the way east, north is `(north, −east)`.
fn beside((at, d): Sample, distance: f64) -> P {
    (at.0 + d.1 * distance, at.1 - d.0 * distance)
}

/// Negative when the `olive` areas lie mostly on the left of the line, positive on the right.
fn area_vote(samples: &[Sample], olive: &[Polygon<f64>]) -> i32 {
    let inside = |p: P| {
        let point = Point::new(p.0, p.1);
        olive.iter().any(|area| area.contains(&point))
    };
    samples
        .iter()
        .map(|&s| {
            match (
                inside(beside(s, AREA_PROBE_M)),
                inside(beside(s, -AREA_PROBE_M)),
            ) {
                (true, false) => 1,
                (false, true) => -1,
                _ => 0,
            }
        })
        .sum()
}

/// The length of a line (ground metres).
fn length(line: &[P]) -> f64 {
    line.windows(2)
        .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
        .sum()
}

/// Whether the ends of a line meet or nearly meet ([`CLOSE_GAP_M`], [`CLOSE_SHARE`]).
fn nearly_closed(line: &[P]) -> bool {
    let (Some(a), Some(b)) = (line.first(), line.last()) else {
        return false;
    };
    let gap = (b.0 - a.0).hypot(b.1 - a.1);
    gap <= CLOSE_GAP_M && gap <= CLOSE_SHARE * length(line)
}

/// The inner side of a line bent round an area ([`BEND_SHARE`]): negative when the area lies on
/// its left, positive on its right, else 0.
fn bend_vote(line: &[P]) -> i32 {
    let area = signed_area(line) / 2.0;
    let length = length(line);
    match length > 0.0 && area.abs() >= BEND_SHARE * length * length {
        true => -area.signum() as i32,
        false => 0,
    }
}

/// Twice the signed area of a ring (positive when it runs counter-clockwise, north up); an open
/// line is closed from its last point to its first.
fn signed_area(ring: &[P]) -> f64 {
    (0..ring.len())
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum()
}

/// `Some(true)` when the terrain is lower on the left of the line, `Some(false)` when lower on
/// the right, `None` when it says nothing clear.
fn downhill_vote(samples: &[Sample], terrain: &Raster<f32>) -> Option<bool> {
    let height = |p: P| bilinear(terrain, p.0, p.1).map(f64::from);
    let drops: Vec<f64> = samples
        .iter()
        .filter_map(|&s| {
            Some(height(beside(s, -HEIGHT_PROBE_M))? - height(beside(s, HEIGHT_PROBE_M))?)
        })
        .collect();
    if drops.is_empty() {
        return None;
    }
    let mean = drops.iter().sum::<f64>() / drops.len() as f64;
    (mean.abs() >= MIN_DROP_M).then_some(mean < 0.0)
}

/// Banks the terrain leaves open take the way round of the decided banks beside them, round by
/// round, so a run of banks gets one direction.
fn settle_by_neighbours(banks: &mut [BankLine]) {
    loop {
        let mut changed = false;
        for b in 0..banks.len() {
            if banks[b].turn.is_some() {
                continue;
            }
            let mut vote = 0.0;
            for (c, other) in banks.iter().enumerate() {
                let Some(turned) = other.turn else { continue };
                if c == b {
                    continue;
                }
                let sign = if turned { -1.0 } else { 1.0 };
                for &(p, d) in &banks[b].samples {
                    for &(q, e) in &other.samples {
                        let cos = d.0 * e.0 + d.1 * e.1;
                        if cos.abs() >= PARALLEL_COS && (p.0 - q.0).hypot(p.1 - q.1) <= NEIGHBOUR_M
                        {
                            vote += sign * cos;
                        }
                    }
                }
            }
            if vote != 0.0 {
                banks[b].turn = Some(vote < 0.0);
                changed = true;
            }
        }
        if !changed {
            return;
        }
    }
}

/// Runs the line the other way round; a line with curves or dashes is left alone. Whether it
/// was turned.
fn turn(object: &mut MapObject) -> bool {
    if !turnable(object) {
        return false;
    }
    let end = object.coords.last().map_or(0, |c| c.flags);
    object.coords.reverse();
    for c in &mut object.coords {
        c.flags = 0;
    }
    if let Some(last) = object.coords.last_mut() {
        last.flags = end;
    }
    true
}

/// Whether the line has no curves, dashes or holes: only such a line is turned. The last point
/// may close it, also with the hole flag that ends a ring as the map writes it.
fn turnable(object: &MapObject) -> bool {
    let Some((last, rest)) = object.coords.split_last() else {
        return true;
    };
    rest.iter().all(|c| c.flags == 0) && last.flags & !(FLAG_CLOSE | FLAG_HOLE) == 0
}

/// The height model of the project, when it is stored and can be read.
pub fn read_terrain(dir: &std::path::Path) -> Option<Raster<f32>> {
    crate::derive::terrain::read_float_raster(&dir.join(crate::project::TERRAIN_RELPATH)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::project::MapKind;
    use crate::raster::GridSpec;

    fn path(f: &Fixture, code: &str, points: &[P]) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| f.at(p)).collect(),
            tags: Vec::new(),
        }
    }

    /// An olive green square (a closed ring) from `(e0, n0)` of side `side`.
    fn olive(f: &Fixture, (e0, n0): P, side: f64) -> MapObject {
        let corners = [
            (e0, n0),
            (e0 + side, n0),
            (e0 + side, n0 + side),
            (e0, n0 + side),
        ];
        let mut o = path(f, "520", &corners);
        o.coords.last_mut().unwrap().flags = FLAG_CLOSE;
        o
    }

    /// The first ground point of the object, relative to the reference point.
    fn start(f: &Fixture, o: &MapObject) -> P {
        let p = f.georef.map_to_projected(o.coords[0]);
        (
            p.easting - f.georef.ref_projected.easting,
            p.northing - f.georef.ref_projected.northing,
        )
    }

    /// A terrain falling to the east by 1 m per 2 m, 200 m square around the reference point.
    fn terrain_falling_east(f: &Fixture) -> Raster<f32> {
        let (e, n) = (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        );
        let grid = GridSpec {
            west: e - 100.0,
            north: n + 100.0,
            pixel_size: 1.0,
            width: 200,
            height: 200,
        };
        let mut raster = Raster::filled(grid, 0.0f32);
        for x in 0..200 {
            for y in 0..200 {
                raster.set(x, y, 100.0 - 0.5 * x as f32);
            }
        }
        raster
    }

    fn run(f: &Fixture, objects: &mut Vec<MapObject>, terrain: Option<&Raster<f32>>) -> Turned {
        orient_ticks(&mut [objects], terrain, &f.georef, &f.set)
    }

    #[test]
    fn fence_turns_its_ticks_to_the_olive_area() {
        let f = fixture(MapKind::Sprint);
        // The olive area lies north of the fence. Running east, the right side is south.
        let mut objects = vec![
            olive(&f, (0.0, 0.0), 40.0),
            path(&f, "518", &[(5.0, -0.2), (35.0, -0.2)]),
        ];
        // The fence runs along y = −0.2: the area lies on its left; the ticks must turn.
        let turned = run(&f, &mut objects, None);
        assert_eq!(turned.fences, 1);
        assert!(start(&f, &objects[1]).0 > 30.0, "{:?}", objects[1].coords);
        // Now the area lies on its right: nothing more to do.
        assert_eq!(run(&f, &mut objects, None), Turned::default());
    }

    #[test]
    fn closed_fence_has_its_ticks_inside() {
        let f = fixture(MapKind::Sprint);
        let ring = |points: &[P]| {
            let mut o = path(&f, "518", points);
            o.coords.last_mut().unwrap().flags = FLAG_CLOSE;
            o
        };
        // Counter-clockwise: the inside is on the left, so it turns; clockwise stays.
        let ccw = ring(&[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)]);
        let cw = ring(&[(0.0, 0.0), (0.0, 20.0), (20.0, 20.0), (20.0, 0.0)]);
        let mut objects = vec![ccw.clone(), cw.clone()];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert_ne!(objects[0].coords, ccw.coords);
        assert_eq!(objects[1].coords, cw.coords);
        assert_eq!(objects[0].coords.last().unwrap().flags, FLAG_CLOSE);
    }

    #[test]
    fn closed_ring_with_hole_flag_has_its_ticks_inside() {
        let f = fixture(MapKind::Sprint);
        // A ring as the map writes it ends with close and hole (18); it turns all the same.
        let mut ccw = path(
            &f,
            "518",
            &[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)],
        );
        ccw.coords.last_mut().unwrap().flags = FLAG_CLOSE | FLAG_HOLE;
        let mut objects = vec![ccw.clone()];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert_ne!(objects[0].coords, ccw.coords);
        assert_eq!(
            objects[0].coords.last().unwrap().flags,
            FLAG_CLOSE | FLAG_HOLE
        );
        assert!(objects[0].coords[..3].iter().all(|c| c.flags == 0));
        assert_eq!(run(&f, &mut objects, None), Turned::default());
    }

    #[test]
    fn nearly_closed_fence_has_its_ticks_inside() {
        let f = fixture(MapKind::Sprint);
        // Counter-clockwise round a 20 m square with a gap of 1.5 m: the inside is on the left.
        let ccw = path(
            &f,
            "518",
            &[
                (1.5, 0.0),
                (20.0, 0.0),
                (20.0, 20.0),
                (0.0, 20.0),
                (0.0, 0.0),
            ],
        );
        let mut objects = vec![ccw.clone()];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert_ne!(objects[0].coords, ccw.coords);
    }

    #[test]
    fn bent_fence_has_its_ticks_on_its_inner_side() {
        let f = fixture(MapKind::Sprint);
        // East, then north: the bend holds the area on its left, so the ticks turn there.
        let left = path(&f, "518", &[(0.0, 0.0), (30.0, 0.0), (30.0, 30.0)]);
        let right = path(&f, "518", &[(30.0, 30.0), (30.0, 0.0), (0.0, 0.0)]);
        let mut objects = vec![left.clone()];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert_ne!(objects[0].coords, left.coords);
        let mut objects = vec![right];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        // Its bend counts before an olive green area outside it.
        let mut objects = vec![
            olive(&f, (0.0, -40.0), 39.8),
            path(&f, "518", &[(0.0, 0.0), (30.0, 0.0), (30.0, 30.0)]),
        ];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        // A line bent only a little decides nothing by its bend.
        let mut objects = vec![path(&f, "518", &[(0.0, 0.0), (30.0, 0.0), (60.0, 2.0)])];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
    }

    #[test]
    fn fence_along_a_park_has_its_ticks_to_the_park() {
        let f = fixture(MapKind::Sprint);
        let mut park = olive(&f, (0.0, 0.0), 40.0);
        park.symbol_id = f.id("401");
        // The park lies north of the fence: running east it is on the left, so it turns.
        let mut objects = vec![park, path(&f, "518", &[(5.0, -0.2), (35.0, -0.2)])];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert!(start(&f, &objects[1]).0 > 30.0);
    }

    #[test]
    fn fence_without_olive_on_one_side_stays() {
        let f = fixture(MapKind::Sprint);
        let fence = path(&f, "518", &[(5.0, 0.0), (35.0, 0.0)]);
        let mut objects = vec![fence.clone()];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        // An olive area on both sides says nothing.
        let mut objects = vec![
            olive(&f, (0.0, 0.5), 20.0),
            olive(&f, (0.0, -20.5), 20.0),
            fence.clone(),
        ];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        assert_eq!(objects[2].coords, fence.coords);
    }

    #[test]
    fn bank_ticks_point_down_the_slope() {
        let f = fixture(MapKind::Sprint);
        let terrain = terrain_falling_east(&f);
        // North, the right side is east and lower: stays. South, the right side is west: turns.
        let mut objects = vec![
            path(&f, "104", &[(0.0, 0.0), (0.0, 30.0)]),
            path(&f, "104", &[(10.0, 30.0), (10.0, 0.0)]),
        ];
        let turned = run(&f, &mut objects, Some(&terrain));
        assert_eq!(turned.banks, 1);
        assert!(start(&f, &objects[0]).1 < 5.0);
        assert!(start(&f, &objects[1]).1 < 5.0, "turned to run north");
    }

    #[test]
    fn bank_the_terrain_leaves_open_follows_its_neighbour() {
        let f = fixture(MapKind::Sprint);
        let mut terrain = terrain_falling_east(&f);
        // No heights beyond 48 m east: a bank there has no terrain on its right.
        for x in 148..200 {
            for y in 0..200 {
                terrain.set(x, y, f32::NAN);
            }
        }
        // The first bank runs south with the lower side on its left: the terrain turns it. The
        // second lies 2.5 m beside it, runs north and gets the way round of the first.
        let mut objects = vec![
            path(&f, "104", &[(45.0, 30.0), (45.0, 0.0)]),
            path(&f, "104", &[(47.5, 0.0), (47.5, 30.0)]),
        ];
        let turned = run(&f, &mut objects, Some(&terrain));
        assert_eq!(turned.banks, 1, "only the first is turned");
        let north = |o: &MapObject| o.coords[0].y > o.coords[1].y;
        assert!(north(&objects[0]) && north(&objects[1]), "{objects:?}");
    }

    /// A point symbol `code` at `at`.
    fn point(f: &Fixture, code: &str, at: P) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![f.at(at)],
            tags: Vec::new(),
        }
    }

    /// A fence running east from (5, 0) to (35, 0): its ticks are on the south side.
    fn east_fence(f: &Fixture) -> MapObject {
        path(f, "518", &[(5.0, 0.0), (35.0, 0.0)])
    }

    #[test]
    fn a_fence_along_a_path_turns_away_from_it() {
        let f = fixture(MapKind::Sprint);
        // The footpath runs 1.5 m south of the fence, on the side of its ticks.
        let mut objects = vec![
            path(&f, "501.6", &[(0.0, -1.5), (40.0, -1.5)]),
            east_fence(&f),
        ];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert!(start(&f, &objects[1]).0 > 30.0, "{:?}", objects[1].coords);
        // Now the path lies on the side without ticks: nothing more to do.
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        // A way on the other side keeps the direction.
        let mut objects = vec![
            path(&f, "501.8", &[(0.0, 2.0), (40.0, 2.0)]),
            east_fence(&f),
        ];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
    }

    #[test]
    fn a_path_along_less_than_half_the_fence_gives_no_vote() {
        let f = fixture(MapKind::Sprint);
        // A path along the last 10 m of the 30 m fence on its left, and one along its first
        // 10 m on its right: neither counts, and the ticks meet a path on both sides.
        let left = path(&f, "501.6", &[(25.0, 1.5), (40.0, 1.5)]);
        let mut objects = vec![
            left.clone(),
            path(&f, "501.6", &[(0.0, -1.5), (15.0, -1.5)]),
            east_fence(&f),
        ];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        // Along 20 m on the right it counts: the ticks turn away from it.
        let mut objects = vec![
            left,
            path(&f, "501.6", &[(0.0, -1.5), (25.0, -1.5)]),
            east_fence(&f),
        ];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
    }

    #[test]
    fn olive_green_wins_over_a_path_on_the_same_side() {
        let f = fixture(MapKind::Sprint);
        // Olive green and a path, both north of the fence: on its left.
        let mut objects = vec![
            olive(&f, (0.0, 0.0), 40.0),
            path(&f, "501.6", &[(0.0, 1.5), (40.0, 1.5)]),
            path(&f, "518", &[(5.0, -0.2), (35.0, -0.2)]),
        ];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert!(
            start(&f, &objects[2]).0 > 30.0,
            "ticks left, to the olive green"
        );
    }

    #[test]
    fn a_closed_fence_keeps_its_ticks_inside_with_a_path_inside() {
        let f = fixture(MapKind::Sprint);
        // Clockwise: the inside is on the right, where the ticks are.
        let mut fence = path(
            &f,
            "518",
            &[(0.0, 0.0), (0.0, 20.0), (20.0, 20.0), (20.0, 0.0)],
        );
        fence.coords.last_mut().unwrap().flags = FLAG_CLOSE;
        // A path runs round inside, 1.5 m from the fence.
        let mut ring = path(
            &f,
            "501.6",
            &[(1.5, 1.5), (1.5, 18.5), (18.5, 18.5), (18.5, 1.5)],
        );
        ring.coords.last_mut().unwrap().flags = FLAG_CLOSE;
        let mut objects = vec![ring, fence.clone()];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        assert_eq!(objects[1].coords, fence.coords);
    }

    #[test]
    fn a_fence_between_two_paths_keeps_its_direction() {
        let f = fixture(MapKind::Sprint);
        let fence = east_fence(&f);
        let mut objects = vec![
            path(&f, "501.6", &[(0.0, 1.5), (40.0, 1.5)]),
            path(&f, "501.6", &[(0.0, -1.5), (40.0, -1.5)]),
            fence.clone(),
        ];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        assert_eq!(objects[2].coords, fence.coords);
    }

    #[test]
    fn in_the_forest_the_side_with_room_is_taken() {
        let f = fixture(MapKind::Forest);
        // A boulder south of the fence, under its ticks; nothing to the north.
        let mut objects = vec![point(&f, "205", (20.0, -8.0)), east_fence(&f)];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert!(
            start(&f, &objects[1]).0 > 30.0,
            "ticks left, to the free side"
        );
        // On the free side now: it stays.
        assert_eq!(run(&f, &mut objects, None), Turned::default());
    }

    #[test]
    fn in_the_forest_with_no_room_the_direction_stays() {
        let f = fixture(MapKind::Forest);
        let fence = east_fence(&f);
        let mut objects = vec![
            point(&f, "205", (20.0, -8.0)),
            point(&f, "205", (20.0, 8.0)),
            fence.clone(),
        ];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        assert_eq!(objects[2].coords, fence.coords);
    }

    #[test]
    fn a_retaining_wall_is_not_turned_by_a_path() {
        let f = fixture(MapKind::Sprint);
        let terrain = terrain_falling_east(&f);
        // The wall runs north, its half dots on the right: east, the lower side. A path runs
        // along that side.
        let wall = path(&f, "513.2", &[(0.0, 0.0), (0.0, 30.0)]);
        let mut objects = vec![path(&f, "501.6", &[(1.5, -5.0), (1.5, 35.0)]), wall.clone()];
        assert_eq!(run(&f, &mut objects, Some(&terrain)), Turned::default());
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        assert_eq!(objects[1].coords, wall.coords);
    }

    #[test]
    fn the_side_given_by_olive_green_stays_when_its_ticks_overdraw() {
        let f = fixture(MapKind::Sprint);
        // Olive green north of the fence, a cairn in it right at the fence; nothing south.
        let mut objects = vec![
            olive(&f, (0.0, 0.0), 40.0),
            point(&f, "526", (20.0, 1.5)),
            path(&f, "518", &[(5.0, -0.2), (35.0, -0.2)]),
        ];
        assert_eq!(run(&f, &mut objects, None).fences, 1);
        assert!(
            start(&f, &objects[2]).0 > 30.0,
            "ticks left, to the olive green"
        );
    }

    #[test]
    fn a_closed_fence_with_olive_green_outside_keeps_its_ticks_inside() {
        let f = fixture(MapKind::Sprint);
        // Clockwise: the inside is on the right. Olive green lies west of its west side, outside.
        let mut fence = path(
            &f,
            "518",
            &[(0.0, 0.0), (0.0, 20.0), (20.0, 20.0), (20.0, 0.0)],
        );
        fence.coords.last_mut().unwrap().flags = FLAG_CLOSE;
        let mut objects = vec![olive(&f, (-40.0, -10.0), 40.0), fence.clone()];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        assert_eq!(objects[1].coords, fence.coords);
    }

    #[test]
    fn a_candidate_path_does_not_turn_a_map_fence() {
        let f = fixture(MapKind::Sprint);
        let fence = east_fence(&f);
        let along = path(&f, "501.6", &[(0.0, -1.5), (40.0, -1.5)]);
        // The path along the side of the ticks is a candidate: the fence of the map stays.
        let (mut map, mut candidates) = (vec![fence.clone()], vec![along.clone()]);
        let turned = orient_ticks(&mut [&mut map, &mut candidates], None, &f.georef, &f.set);
        assert_eq!(turned, Turned::default());
        assert_eq!(map[0].coords, fence.coords);
        // A candidate fence looks at the map too: it turns away from the path of the map.
        let (mut map, mut candidates) = (vec![along], vec![fence]);
        let turned = orient_ticks(&mut [&mut map, &mut candidates], None, &f.georef, &f.set);
        assert_eq!(turned.fences, 1);
        assert!(start(&f, &candidates[0]).0 > 30.0);
    }

    #[test]
    fn a_copy_of_the_fence_in_the_other_list_is_no_obstacle() {
        let f = fixture(MapKind::Forest);
        // The map holds the fence run the other way round and 1 map unit off; the candidates
        // hold it with a boulder under its ticks.
        let fence = east_fence(&f);
        let mut copy = fence.clone();
        copy.coords.reverse();
        for c in &mut copy.coords {
            c.x += 1;
        }
        let mut map = vec![copy.clone()];
        let mut candidates = vec![fence, point(&f, "205", (20.0, -8.0))];
        let turned = orient_ticks(&mut [&mut map, &mut candidates], None, &f.georef, &f.set);
        // The candidate turns to the free side; the map's fence sees no boulder and stays.
        assert_eq!(turned.fences, 1);
        assert!(start(&f, &candidates[0]).0 > 30.0);
        assert_eq!(map[0].coords, copy.coords);
    }

    #[test]
    fn a_curve_is_never_turned() {
        let f = fixture(MapKind::Sprint);
        let mut o = path(&f, "518", &[(5.0, -0.2), (20.0, -0.2), (35.0, -0.2)]);
        o.coords[0].flags = 1;
        let mut objects = vec![olive(&f, (0.0, 0.0), 40.0), o.clone()];
        assert_eq!(run(&f, &mut objects, None), Turned::default());
        assert_eq!(objects[1].coords, o.coords);
    }
}
