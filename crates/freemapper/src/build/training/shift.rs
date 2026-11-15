//! Step 6 of the training level: shift before cutting (spec 2026-11-14, "The second half, as
//! designed on 2026-11-15", "Shift"). A line that keeps no gap to an object it keeps apart from
//! is moved away from it by what is missing to the gap, at most [`MAX_SHIFT_MM`] on paper, when
//! the moved line makes no new conflict with anything; else it is left to give way, which cuts it
//! as before. Paper mm, y down, as `conflicts::Footprints` uses; only +, −, ×, ÷ and `sqrt`
//! decide, and every collection is iterated in a sorted order.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use geo::{BoundingRect as _, Intersects as _};
use geo_types::{Coord, MultiPolygon, Point, Rect};

use super::lines::{Polyline, shifted};
use super::points::MAX_SHIFT_MM;
use super::{MAX_ROUNDS, add_generalized, class_name};
use crate::build::conflicts::{
    Class, Classes, Conflict, Footprints, Grade, MIN_CONFLICT, PairMemo, Relation, TOLERANCE_MM,
    conflict_of_first, gap, meeting_points, rank, relation,
};
use crate::build::dedup::tag_holds;
use crate::georef::Georeferencing;
use crate::omap::coord::FLAG_CLOSE;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::render::symbols::RenderSymbols;

/// A stretch is moved with a ramp of this many times the shift at each end (spec: "a ramp of four
/// times the shift").
const RAMP_FACTOR: f64 = 4.0;
/// A line whose stretches in conflict cover more than this share of it is moved whole.
const WHOLE_SHARE: f64 = 0.75;
/// The spacing of the samples along a line that find its stretch in conflict (mm on paper).
const SAMPLE_MM: f64 = 0.05;
/// The spacing of the points of a moved stretch and its ramps (mm on paper).
const STEP_MM: f64 = 0.25;
/// The objects round a line that a shift is checked against: those whose points lie this close
/// to its frame (mm on paper), beyond every symbol's reach, the gap and the shift.
const REACH_MM: f64 = 3.0;
/// The beginning of the `freemapper:generalized` value of a shifted line; the class it keeps the
/// gap to follows.
const SHIFTED: &str = "shifted: gap to ";

/// Shifts lines off what they keep no gap to. For every conflict of two objects that keep apart
/// ([`Relation::Apart`]) whose loser is a line of one part (the losers by ascending rank, of one
/// rank the longer first, as give way orders them; the winner stays): the stretches of the
/// loser's centre line inside the winner's footprint grown by the loser's half width and the gap
/// (as the measure draws the zone, the places where the centre lines cross left out, stretches
/// below [`MIN_CONFLICT`] ignored) are moved away from the winner by what is missing to the gap
/// plus [`TOLERANCE_MM`], with ramps of [`RAMP_FACTOR`] times the shift ([`shifted`]); the whole
/// line, without ramps, when they cover more than [`WHOLE_SHARE`] of it. A ramp that would
/// reach an open end moves the end with the stretch. The side is away from the winner's nearest
/// point (into it from inside its footprint). No shift beyond [`MAX_SHIFT_MM`], and a stretch of
/// a line moves once: a later shift whose moving part overlaps an earlier one is not made. On a
/// closed line a stretch in conflict across its seam is one stretch.
///
/// A shift is kept only when the moved line has no conflict (of either relation, points and
/// contours among the others) that it did not have before, none of a worse grade, and none with
/// the winner; else the line stays as it was for give way. The direction of the line is kept, so
/// a sided line (fence, wall, earth bank) keeps its tick side. The rounds measure again and stop
/// when no line moves, at the latest after [`MAX_ROUNDS`]. Returns how many objects moved.
///
/// A shifted line is drawn again from its flattened centre line: Bézier and dash flags of its
/// points are dropped, the closing flag is kept. An end that moved with its stretch and touched a
/// line before may be pulled back to that junction by `Junctions::repair` afterwards; what that
/// leaves too close, give way settles.
pub(super) fn shift_lines(
    objects: &mut [MapObject],
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let shifter = Shifter {
        symbols: RenderSymbols::from_set(symbol_set),
        classes: Classes::of(symbol_set),
        kind,
        scale_denominator: georef.scale_denominator,
        gap: gap(kind, georef.scale_denominator).0,
    };
    let mut moved: BTreeMap<usize, Ranges> = BTreeMap::new();
    let mut scene = Scene::new(objects, &shifter);
    let mut memo = PairMemo::default();
    for _ in 0..MAX_ROUNDS {
        let footprints = &scene.footprints;
        let mut found: Vec<Conflict> = memo
            .conflicts_where(footprints, kind, georef.scale_denominator, |r| {
                r == Relation::Apart
            })
            .into_iter()
            .filter(|c| footprints.centre(c.loser).0.len() == 1 && footprints.length(c.loser) > 0.0)
            .collect();
        if found.is_empty() {
            break;
        }
        found.sort_by(|a, b| {
            rank(a.classes.0)
                .cmp(&rank(b.classes.0))
                .then(
                    footprints
                        .length(b.loser)
                        .total_cmp(&footprints.length(a.loser)),
                )
                .then(a.loser.cmp(&b.loser))
                .then(b.length.0.total_cmp(&a.length.0))
                .then(a.winner.cmp(&b.winner))
        });
        let mut changed: BTreeSet<usize> = BTreeSet::new();
        for conflict in &found {
            let ranges = moved.get(&conflict.loser).map_or(&[][..], Vec::as_slice);
            if let Some(shift) =
                shifter.shift(objects, &scene, conflict.loser, conflict.winner, ranges)
            {
                scene.update(conflict.loser, &shift.object, &shift.footprints);
                objects[conflict.loser] = shift.object;
                moved.insert(conflict.loser, shift.ranges);
                changed.insert(conflict.loser);
            }
        }
        if changed.is_empty() {
            break;
        }
        memo.forget(&changed);
    }
    moved.len()
}

/// The ranges along a line that have moved (mm on paper, sorted).
type Ranges = Vec<(f64, f64)>;

/// The side of a cell of the grid that finds the objects near a line (map units).
const CELL_UNITS: f64 = 20_000.0;

/// What the shift step keeps of the objects between its rounds: the footprint and the frame of
/// every object, and a uniform grid of the frames. Only an object that moves is drawn again.
struct Scene {
    footprints: Footprints,
    frames: Vec<Option<Rect<f64>>>,
    grid: BTreeMap<(i64, i64), Vec<usize>>,
    /// The conflict of a line with an object near it before a shift, as `conflict_of_first`
    /// finds it (the line first), kept until one of the two moves.
    before: RefCell<HashMap<(usize, usize), Option<Conflict>>>,
    /// How many shifts were kept so far, and after how many each object last moved (0: never).
    stamp: usize,
    moved_at: Vec<usize>,
    /// The shifts refused, by line and winner: when, and the objects near the line then. Asked
    /// again while none of them moved and no other came near, the answer is the same.
    refused: RefCell<HashMap<(usize, usize), Refusal>>,
}

/// When a shift was refused (the stamp of [`Scene`]), and the objects near the line then.
type Refusal = (usize, Vec<usize>);

impl Scene {
    fn new(objects: &[MapObject], shifter: &Shifter) -> Self {
        let refs: Vec<&MapObject> = objects.iter().collect();
        let mut scene = Self {
            footprints: Footprints::with(&refs, &shifter.symbols, &shifter.classes),
            frames: objects.iter().map(frame).collect(),
            grid: BTreeMap::new(),
            before: RefCell::new(HashMap::new()),
            stamp: 0,
            moved_at: vec![0; objects.len()],
            refused: RefCell::new(HashMap::new()),
        };
        for i in 0..objects.len() {
            for cell in cells(scene.frames[i], 0.0) {
                scene.grid.entry(cell).or_default().push(i);
            }
        }
        scene
    }

    /// The object `i` is now `object`, drawn as the object 0 of `footprints`.
    fn update(&mut self, i: usize, object: &MapObject, footprints: &Footprints) {
        for cell in cells(self.frames[i], 0.0) {
            if let Some(ids) = self.grid.get_mut(&cell) {
                ids.retain(|&j| j != i);
            }
        }
        self.frames[i] = frame(object);
        for cell in cells(self.frames[i], 0.0) {
            self.grid.entry(cell).or_default().push(i);
        }
        self.footprints.adopt(i, footprints, 0);
        self.stamp += 1;
        self.moved_at[i] = self.stamp;
        self.before.get_mut().retain(|&(a, b), _| a != i && b != i);
    }

    /// Whether the shift of `loser` off `winner` was refused with the objects `near` round it,
    /// and neither it nor one of them has moved since.
    fn refused_before(&self, loser: usize, winner: usize, near: &[usize]) -> bool {
        self.refused
            .borrow()
            .get(&(loser, winner))
            .is_some_and(|(at, then)| {
                then.as_slice() == near
                    && std::iter::once(&loser)
                        .chain(near)
                        .all(|&j| self.moved_at[j] <= *at)
            })
    }

    /// The conflicts of the object `i` with the objects `near` (sorted, without `i`), as
    /// `conflicts_of` finds them for the object 0 of the set of `i` and `near`, by index in it.
    fn conflicts_before(
        &self,
        i: usize,
        near: &[usize],
        kind: MapKind,
        scale_denominator: u32,
    ) -> Vec<Conflict> {
        let mut memo = self.before.borrow_mut();
        near.iter()
            .enumerate()
            .filter_map(|(k, &j)| {
                let mut c = memo
                    .entry((i, j))
                    .or_insert_with(|| {
                        conflict_of_first(&self.footprints, i, j, kind, scale_denominator)
                    })
                    .clone()?;
                let index = |n: usize| if n == 0 { 0 } else { k + 1 };
                (c.loser, c.winner) = (index(c.loser), index(c.winner));
                Some(c)
            })
            .collect()
    }

    /// The objects whose frame may come within `reach` of the frame `own`, sorted, each once.
    fn around(&self, own: Rect<f64>, reach: f64) -> Vec<usize> {
        let mut ids: Vec<usize> = cells(Some(own), reach)
            .filter_map(|cell| self.grid.get(&cell))
            .flatten()
            .copied()
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }
}

/// The cells of the grid that the frame `frame` grown by `grow` covers (map units).
fn cells(frame: Option<Rect<f64>>, grow: f64) -> impl Iterator<Item = (i64, i64)> {
    let cell = |v: f64| (v / CELL_UNITS).floor() as i64;
    let (x, y) = frame.map_or(((0, -1), (0, -1)), |f| {
        (
            (cell(f.min().x - grow), cell(f.max().x + grow)),
            (cell(f.min().y - grow), cell(f.max().y + grow)),
        )
    });
    (x.0..=x.1).flat_map(move |cx| (y.0..=y.1).map(move |cy| (cx, cy)))
}

/// A shift that is kept: the moved line, the ranges along it that have moved, and its footprint
/// as the object 0 of a set.
struct Shift {
    object: MapObject,
    ranges: Ranges,
    footprints: Footprints,
}

/// What a shift needs to know.
struct Shifter {
    symbols: RenderSymbols,
    classes: Classes,
    kind: MapKind,
    scale_denominator: u32,
    gap: f64,
}

/// A stretch of a line to move: its range along the line, and the side (+1 right, -1 left of the
/// direction).
#[derive(Debug, Clone, Copy)]
struct Stretch {
    range: (f64, f64),
    side: f64,
}

impl Shifter {
    /// The line `objects[loser]` moved off `objects[winner]`, with the ranges along it that have
    /// moved so far (`moved` before, sorted), when the shift is allowed and makes no new
    /// conflict.
    fn shift(
        &self,
        objects: &[MapObject],
        scene: &Scene,
        loser: usize,
        winner: usize,
        moved: &[(f64, f64)],
    ) -> Option<Shift> {
        let class = self.classes.class(objects[loser].symbol_id);
        let near = self.near(objects, scene, loser, class);
        if scene.refused_before(loser, winner, &near) {
            return None;
        }
        let shift = self.attempt(objects, scene, loser, winner, moved, &near);
        if shift.is_none() {
            scene
                .refused
                .borrow_mut()
                .insert((loser, winner), (scene.stamp, near));
        }
        shift
    }

    /// [`shift`](Self::shift) with the objects `near` the line found.
    fn attempt(
        &self,
        objects: &[MapObject],
        scene: &Scene,
        loser: usize,
        winner: usize,
        moved: &[(f64, f64)],
        near: &[usize],
    ) -> Option<Shift> {
        let object = &objects[loser];
        let w = 1 + near.iter().position(|&j| j == winner)?;
        let mut set: Vec<usize> = vec![loser];
        set.extend(near);
        let before = scene.footprints.pick(&set);
        let had_conflicts = scene.conflicts_before(loser, near, self.kind, self.scale_denominator);
        // The conflict as it is now: the line still gives way to the winner.
        if !had_conflicts.iter().any(|c| c.loser == 0 && c.winner == w) {
            return None;
        }
        let had: BTreeMap<usize, Grade> =
            had_conflicts.iter().map(|c| (other(c), c.grade)).collect();
        let (points, ranges) = self.plan(&before, w, object, moved)?;
        let mut line = with_points(object, &points);
        let why = format!("{SHIFTED}{}", class_name(before.class(w)));
        let generalized = line
            .tags
            .iter()
            .find(|(k, _)| k == super::GENERALIZED_KEY)
            .is_some_and(|(_, v)| tag_holds(v, &why));
        if !generalized {
            add_generalized(&mut line, &why);
        }
        let mut after = before.clone();
        after.redraw(0, &line, &self.symbols, &self.classes);
        // A conflict with the winner counts in either role: of two lines of one rank and one
        // length the roles can swap after the shift, and such a pair falls to give way.
        // Measured one object after the other, the winner first, until one is worse: the same
        // verdict as all of `conflicts_of` at once.
        let worse = std::iter::once(w)
            .chain((1..set.len()).filter(|&k| k != w))
            .filter_map(|k| {
                Some((
                    k,
                    conflict_of_first(&after, 0, k, self.kind, self.scale_denominator)?,
                ))
            })
            .any(|(k, c)| {
                k == w
                    || match had.get(&k) {
                        None => true,
                        Some(&grade) => grade == Grade::Gap && c.grade == Grade::Overlap,
                    }
            });
        (!worse).then_some(Shift {
            object: line,
            ranges,
            footprints: after,
        })
    }

    /// The objects round `objects[i]` (of class `class`) that it may conflict with, sorted: those
    /// of a class that is not free to it whose frame (of the coordinates, in `scene`) comes within
    /// [`REACH_MM`] of its frame.
    fn near(&self, objects: &[MapObject], scene: &Scene, i: usize, class: Class) -> Vec<usize> {
        let frames = &scene.frames;
        let Some(own) = frames[i] else {
            return Vec::new();
        };
        let reach = REACH_MM * 1000.0;
        scene
            .around(own, reach)
            .into_iter()
            .filter(|&j| j != i)
            .filter(|&j| {
                relation(class, self.classes.class(objects[j].symbol_id)) != Relation::Free
            })
            .filter(|&j| {
                frames[j].is_some_and(|f| {
                    f.min().x <= own.max().x + reach
                        && f.max().x >= own.min().x - reach
                        && f.min().y <= own.max().y + reach
                        && f.max().y >= own.min().y - reach
                })
            })
            .collect()
    }

    /// The moved centre line of the line 0 of `footprints` (the object `object`) off the object
    /// `w`, and the ranges along it that have moved then (`moved` before); `None` when it cannot
    /// move.
    fn plan(
        &self,
        footprints: &Footprints,
        w: usize,
        object: &MapObject,
        moved: &[(f64, f64)],
    ) -> Option<(Vec<Coord<f64>>, Ranges)> {
        let [centre] = footprints.centre(0).0.as_slice() else {
            return None;
        };
        let line = Polyline::new(centre.0.clone());
        let total = line.length();
        if line.points.len() < 2 || total <= 0.0 {
            return None;
        }
        let closed = super::is_closed(object);
        let half = footprints.half_width(0);
        let zone = half + self.gap;
        let frame = centre.bounding_rect()?;
        let winner = Boundary::of(footprints.footprint(w), &frame, zone + MAX_SHIFT_MM.0 + 1.0);
        let radius = half + footprints.half_width(w) + self.gap;
        let discs = meeting_points(footprints.centre(0), footprints.centre(w));
        let n = (total / SAMPLE_MM).ceil().max(1.0) as usize;
        let step = total / n as f64;
        // The runs of samples in the zone, with the most missing to the gap in each.
        let mut runs: Vec<(f64, f64, f64)> = Vec::new();
        let mut open: Option<(f64, f64, f64)> = None;
        for k in 0..=n {
            let s = if k == n { total } else { step * k as f64 };
            let p = line.point(s);
            let crossing = discs.iter().any(|d| distance2(p, *d) <= radius * radius);
            let missing = (!crossing)
                .then(|| winner.clearance(p))
                .flatten()
                .map(|(clearance, _, _)| zone - clearance)
                .filter(|&m| m > TOLERANCE_MM);
            match (missing, open.as_mut()) {
                (Some(m), Some(run)) => {
                    run.1 = s;
                    run.2 = run.2.max(m);
                }
                (Some(m), None) => open = Some((s, s, m)),
                (None, _) => runs.extend(open.take()),
            }
        }
        runs.extend(open);
        let runs: Vec<(f64, f64, f64)> = runs
            .into_iter()
            .map(|(from, to, m)| {
                (
                    (from - step / 2.0).max(0.0),
                    (to + step / 2.0).min(total),
                    m,
                )
            })
            .filter(|&(from, to, _)| to - from >= MIN_CONFLICT.0)
            .collect();
        let shift = runs.iter().map(|r| r.2).reduce(f64::max)? + TOLERANCE_MM;
        if shift > MAX_SHIFT_MM.0 {
            return None;
        }
        let ramp = RAMP_FACTOR * shift;
        // Runs closer than two ramps are one stretch.
        let mut groups: Vec<(f64, f64)> = Vec::new();
        for (from, to, _) in runs {
            match groups.last_mut() {
                Some(last) if from - last.1 < 2.0 * ramp => last.1 = to,
                _ => groups.push((from, to)),
            }
        }
        // On a closed line a stretch at its start and one at its end meet across the seam: one
        // stretch that runs on over it (`shifted` takes a range beyond the length).
        if closed && groups.len() > 1 {
            let (first, last) = (groups[0], groups[groups.len() - 1]);
            if total - last.1 + first.0 < 2.0 * ramp {
                groups.remove(0);
                let end = groups.len() - 1;
                groups[end] = (last.0, total + first.1);
            }
        }
        let stretches: Vec<Stretch> = groups
            .into_iter()
            .map(|range| {
                Some(Stretch {
                    range,
                    side: side(&line, &winner, range)?,
                })
            })
            .collect::<Option<_>>()?;
        let covered: f64 = stretches.iter().map(|s| s.range.1 - s.range.0).sum();
        let one_side = stretches.windows(2).all(|w| w[0].side == w[1].side);
        let overlaps = |ranges: &[(f64, f64)], lo: f64, hi: f64| {
            ranges
                .iter()
                .any(|&(a, b)| lo < b - TOLERANCE_MM && hi > a + TOLERANCE_MM)
        };
        if covered > WHOLE_SHARE * total && one_side {
            if overlaps(moved, 0.0, total) {
                return None;
            }
            let side = stretches[0].side;
            let points = shifted(&line, (0.0, total), shift, side, 0.0, total);
            let points: Vec<Coord<f64>> = points.into_iter().map(|(x, y)| Coord { x, y }).collect();
            let length = Polyline::new(points.clone()).length();
            return Some((points, vec![(0.0, length)]));
        }
        let mut ranges: Ranges = moved.to_vec();
        let mut line = line;
        // From the last stretch back, so that the ranges before each stay where they were.
        for stretch in stretches.iter().rev() {
            let (mut from, mut to) = stretch.range;
            if !closed {
                if from - ramp < 0.0 {
                    from = 0.0;
                }
                if to + ramp > line.length() {
                    to = line.length();
                }
            }
            let before = line.length();
            let parts = moving_parts(from - ramp, to + ramp, before, closed);
            // Against what moved in earlier rounds and in this call.
            if parts.iter().any(|&(lo, hi)| overlaps(&ranges, lo, hi)) {
                return None;
            }
            let points = shifted(&line, (from, to), shift, stretch.side, ramp, STEP_MM);
            line = Polyline::new(points.into_iter().map(|(x, y)| Coord { x, y }).collect());
            let delta = line.length() - before;
            let hi = parts.iter().map(|p| p.1).fold(0.0, f64::max);
            for range in &mut ranges {
                if range.0 >= hi - TOLERANCE_MM {
                    range.0 += delta;
                    range.1 += delta;
                }
            }
            for (lo, hi) in parts {
                let end = if hi >= before - TOLERANCE_MM {
                    hi + delta
                } else {
                    hi
                };
                ranges.push((lo, end));
            }
        }
        ranges.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
        Some((line.points, ranges))
    }
}

/// The side away from the winner of the stretch `range` of `line`: the sign of the cross product
/// of the line's direction at the stretch's middle and the vector to the winner's nearest point,
/// turned round when the middle lies inside the winner's footprint (it leaves it the shortest
/// way). `None` when the winner lies straight ahead or nothing of it is near.
fn side(line: &Polyline, winner: &Boundary, range: (f64, f64)) -> Option<f64> {
    let mut middle = (range.0 + range.1) / 2.0;
    if middle > line.length() {
        // A stretch across the seam of a closed line.
        middle -= line.length();
    }
    let p = line.point(middle);
    let i = line
        .at
        .partition_point(|&a| a < middle)
        .clamp(1, line.points.len() - 1);
    let (a, b) = (line.points[i - 1], line.points[i]);
    let (_, q, inside) = winner.clearance(p)?;
    let cross = (b.x - a.x) * (q.y - p.y) - (b.y - a.y) * (q.x - p.x);
    if cross == 0.0 {
        return None;
    }
    let right = if cross > 0.0 { 1.0 } else { -1.0 };
    Some(if inside { right } else { -right })
}

/// The outline of a footprint near a line: its rings' segments within a frame, and the footprint
/// itself for the test inside or out.
struct Boundary<'a> {
    footprint: &'a MultiPolygon<f64>,
    segments: Vec<(Coord<f64>, Coord<f64>)>,
}

impl<'a> Boundary<'a> {
    /// The segments of the rings of `footprint` that reach within `margin` of `frame`.
    fn of(footprint: &'a MultiPolygon<f64>, frame: &Rect<f64>, margin: f64) -> Self {
        let (min, max) = (frame.min(), frame.max());
        let segments = footprint
            .iter()
            .flat_map(|p| std::iter::once(p.exterior()).chain(p.interiors()))
            .flat_map(|ring| ring.lines())
            .filter(|l| {
                l.start.x.max(l.end.x) >= min.x - margin
                    && l.start.x.min(l.end.x) <= max.x + margin
                    && l.start.y.max(l.end.y) >= min.y - margin
                    && l.start.y.min(l.end.y) <= max.y + margin
            })
            .map(|l| (l.start, l.end))
            .collect();
        Self {
            footprint,
            segments,
        }
    }

    /// How far `p` lies from the footprint (negative inside, as deep as it lies), the nearest
    /// point of its outline and whether `p` is inside; `None` when no outline is near.
    fn clearance(&self, p: Coord<f64>) -> Option<(f64, Coord<f64>, bool)> {
        let (d2, q) = self
            .segments
            .iter()
            .map(|&(a, b)| {
                let q = foot(p, a, b);
                (distance2(p, q), q)
            })
            .min_by(|x, y| x.0.total_cmp(&y.0))?;
        let inside = self.footprint.intersects(&Point(p));
        let d = d2.sqrt();
        Some((if inside { -d } else { d }, q, inside))
    }
}

/// The parts of a line of length `total` that move from `lo` to `hi` along it: clipped to the
/// line when it is open; on a closed line what runs on over the seam is a part of its own at the
/// other end.
fn moving_parts(lo: f64, hi: f64, total: f64, closed: bool) -> Vec<(f64, f64)> {
    if !closed || hi - lo >= total {
        return vec![(lo.max(0.0), hi.min(total))];
    }
    let mut parts = vec![(lo.max(0.0), hi.min(total))];
    if lo < 0.0 {
        parts.push((total + lo, total));
    }
    if hi > total {
        parts.push((0.0, hi - total));
    }
    parts.retain(|p| p.1 > p.0);
    parts.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    parts
}

/// The other object of a conflict of the object 0.
fn other(conflict: &Conflict) -> usize {
    if conflict.loser == 0 {
        conflict.winner
    } else {
        conflict.loser
    }
}

/// The point of the segment from `a` to `b` nearest `p`.
fn foot(p: Coord<f64>, a: Coord<f64>, b: Coord<f64>) -> Coord<f64> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length2 = dx * dx + dy * dy;
    let t = if length2 > 0.0 {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / length2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Coord {
        x: a.x + t * dx,
        y: a.y + t * dy,
    }
}

fn distance2(a: Coord<f64>, b: Coord<f64>) -> f64 {
    (a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)
}

/// The frame of the coordinates of `object`, map units.
fn frame(object: &MapObject) -> Option<Rect<f64>> {
    let xs = object.coords.iter().map(|c| f64::from(c.x));
    let ys = object.coords.iter().map(|c| f64::from(c.y));
    Some(Rect::new(
        Coord {
            x: xs.clone().reduce(f64::min)?,
            y: ys.clone().reduce(f64::min)?,
        },
        Coord {
            x: xs.reduce(f64::max)?,
            y: ys.reduce(f64::max)?,
        },
    ))
}

/// `object` (symbol, tags) along `points` (mm on paper), consecutive equal coordinates once; a
/// closed line keeps its closing flag.
fn with_points(object: &MapObject, points: &[Coord<f64>]) -> MapObject {
    let mut coords: Vec<MapCoord> = points
        .iter()
        .map(|c| MapCoord::new((c.x * 1000.0).round() as i32, (c.y * 1000.0).round() as i32))
        .collect();
    coords.dedup_by(|a, b| (a.x, a.y) == (b.x, b.y));
    if object
        .coords
        .last()
        .is_some_and(|c| c.flags & FLAG_CLOSE != 0)
        && let Some(last) = coords.last_mut()
    {
        last.flags |= FLAG_CLOSE;
    }
    MapObject {
        symbol_id: object.symbol_id,
        kind: MapObjectKind::Path,
        coords,
        tags: object.tags.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::conflicts::conflicts;
    use crate::build::line_join::TOUCH_M;
    use crate::build::training::tests::{SCALE, georef_at, isom, length_mm, line, of_code, point};
    use crate::build::training::{GENERALIZED_KEY, TrainingReport, train};
    use crate::measure::PaperMm;

    fn run(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, TrainingReport) {
        train(objects, None, MapKind::Forest, &georef_at(SCALE), set)
    }

    fn half_width(set: &SymbolSet, code: &str) -> f64 {
        let object = line(set, code, &[(0.0, 0.0), (10.0, 0.0)]);
        Footprints::of(&[&object], set).half_width(0)
    }

    fn mm(c: &MapCoord) -> (f64, f64) {
        (f64::from(c.x) / 1000.0, f64::from(c.y) / 1000.0)
    }

    /// The y of the line `object` at `x` (mm on paper), for a line that runs along x.
    fn y_at(object: &MapObject, x: f64) -> f64 {
        let points: Vec<(f64, f64)> = object.coords.iter().map(mm).collect();
        let w = points
            .windows(2)
            .find(|w| w[0].0.min(w[1].0) <= x && x <= w[0].0.max(w[1].0))
            .expect("the line reaches x");
        let t = if w[1].0 == w[0].0 {
            0.0
        } else {
            (x - w[0].0) / (w[1].0 - w[0].0)
        };
        w[0].1 + t * (w[1].1 - w[0].1)
    }

    fn apart_left(set: &SymbolSet, objects: &[MapObject]) -> Vec<Conflict> {
        let refs: Vec<&MapObject> = objects.iter().collect();
        conflicts(&Footprints::of(&refs, set), MapKind::Forest, SCALE)
            .into_iter()
            .filter(|c| relation(c.classes.0, c.classes.1) == Relation::Apart)
            .collect()
    }

    fn generalized(object: &MapObject) -> Option<&str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == GENERALIZED_KEY)
            .map(|(_, v)| v.as_str())
    }

    /// How far the drawn symbol of the line `object` reaches to either side of its centre
    /// (mm on paper, above and below), for a line that runs straight along x.
    fn reach(set: &SymbolSet, object: &MapObject) -> (f64, f64) {
        let footprints = Footprints::of(&[object], set);
        let frame = footprints.footprint(0).bounding_rect().unwrap();
        let y = mm(&object.coords[0]).1;
        (y - frame.min().y, frame.max().y - y)
    }

    #[test]
    fn an_earth_bank_beside_a_ditch_moves_off() {
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let apart = half_width(&set, "306") + half_width(&set, "104");
        let ditch = line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]);
        let bank = line(&set, "104", &[(0.0, apart + 0.1), (20.0, apart + 0.1)]);
        let (out, report) = run(&set, vec![ditch.clone(), bank]);
        assert_eq!(of_code(&set, &out, "306"), vec![&ditch]);
        let banks = of_code(&set, &out, "104");
        assert_eq!(banks.len(), 1, "{out:?}");
        for x in [0.0, 10.0, 20.0] {
            let y = y_at(banks[0], x);
            assert!(
                (apart + gap..apart + gap + 0.01).contains(&y),
                "{x}: {y} ({:?})",
                banks[0].coords
            );
        }
        assert!((length_mm(banks[0]) - 20.0).abs() < 0.01, "{out:?}");
        assert_eq!(generalized(banks[0]), Some("shifted: gap to watercourse"));
        assert_eq!(
            (report.shifted, report.gave_way, report.cut),
            (1, 0, 0),
            "{report:?}"
        );
        assert_eq!(apart_left(&set, &out), Vec::new(), "{out:?}");
    }

    #[test]
    fn a_fence_beside_a_path_moves_off_and_keeps_its_ticks_side() {
        // Ticks to the right of the direction: on both sides of the path, towards it and away.
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let apart = half_width(&set, "505") + half_width(&set, "518");
        let path = line(&set, "505", &[(-5.0, 0.0), (25.0, 0.0)]);
        for fence in [
            line(&set, "518", &[(0.0, 0.45), (20.0, 0.45)]),
            line(&set, "518", &[(20.0, 0.45), (0.0, 0.45)]),
            line(&set, "518", &[(0.0, -0.45), (20.0, -0.45)]),
        ] {
            let (out, report) = run(&set, vec![path.clone(), fence.clone()]);
            let fences = of_code(&set, &out, "518");
            assert_eq!(fences.len(), 1, "{out:?}");
            let moved = fences[0];
            let side = mm(&fence.coords[0]).1.signum();
            let y = y_at(moved, 10.0);
            assert!(
                (apart + gap..apart + gap + 0.01).contains(&(side * y)),
                "{y}: {:?}",
                moved.coords
            );
            // The same direction, so the ticks lie on the same side.
            let (first, last) = (
                mm(&moved.coords[0]),
                mm(&moved.coords[moved.coords.len() - 1]),
            );
            let (was_first, was_last) = (
                mm(&fence.coords[0]),
                mm(&fence.coords[fence.coords.len() - 1]),
            );
            assert_eq!(
                (first.0 - last.0).signum(),
                (was_first.0 - was_last.0).signum()
            );
            let (above, below) = reach(&set, &fence);
            let (now_above, now_below) = reach(&set, moved);
            assert!(
                (above - now_above).abs() < 0.01 && (below - now_below).abs() < 0.01,
                "{:?} → {:?}",
                (above, below),
                (now_above, now_below)
            );
            assert_eq!(generalized(moved), Some("shifted: gap to path"));
            assert_eq!(report.shifted, 1, "{report:?}");
            assert_eq!(apart_left(&set, &out), Vec::new(), "{out:?}");
        }
    }

    #[test]
    fn a_shift_that_would_hit_a_contour_is_refused() {
        // The bank's ticks point to the ditch (to the right of its direction, -x); beyond it, a
        // contour just outside its gap: moved off the ditch, the bank would come too close to
        // the contour, so it gives way as before.
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let (h306, h104, h101) = (
            half_width(&set, "306"),
            half_width(&set, "104"),
            half_width(&set, "101"),
        );
        let y = h306 + h104 + 0.1;
        let ditch = line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]);
        let bank = line(&set, "104", &[(20.0, y), (0.0, y)]);
        let beyond = y + h104 + h101 + gap + 0.01;
        let contour = line(&set, "101", &[(0.0, beyond), (20.0, beyond)]);
        assert_eq!(
            apart_left(&set, &[bank.clone(), contour.clone()]),
            Vec::new(),
            "the contour keeps the gap at first"
        );
        let (out, report) = run(&set, vec![ditch.clone(), bank, contour.clone()]);
        assert_eq!(out, vec![ditch, contour]);
        assert_eq!((report.shifted, report.gave_way), (0, 1), "{report:?}");
    }

    #[test]
    fn a_shift_beyond_half_a_millimetre_is_refused_and_cut() {
        // The fence lies 0.05 mm off the path's centre over 8 mm: it would have to move more
        // than 0.5 mm to keep the gap, so it is cut there.
        let set = isom();
        let path = line(&set, "505", &[(12.0, 0.0), (40.0, 0.0)]);
        let fence = line(&set, "518", &[(0.0, 0.05), (20.0, 0.05)]);
        let (out, report) = run(&set, vec![path.clone(), fence]);
        let fences = of_code(&set, &out, "518");
        assert_eq!(fences.len(), 1, "{out:?}");
        assert!((length_mm(fences[0]) - 12.0).abs() < 1.0, "{out:?}");
        assert!(fences[0].coords.iter().all(|c| c.y == 50), "{out:?}");
        assert_eq!((report.shifted, report.cut), (0, 1), "{report:?}");
        assert_eq!(of_code(&set, &out, "505"), vec![&path]);
    }

    #[test]
    fn a_partly_close_fence_moves_only_there() {
        // A path along 12 of the fence's 40 mm, in its middle: the stretch moves, with ramps
        // inside the fence; its ends and the rest stay.
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let apart = half_width(&set, "505") + half_width(&set, "518");
        let path = line(&set, "505", &[(14.0, 0.0), (26.0, 0.0)]);
        let fence = line(&set, "518", &[(0.0, 0.45), (40.0, 0.45)]);
        let (out, report) = run(&set, vec![path.clone(), fence.clone()]);
        let fences = of_code(&set, &out, "518");
        assert_eq!(fences.len(), 1, "{out:?}");
        let moved = fences[0];
        assert_eq!(moved.coords[0], fence.coords[0]);
        assert_eq!(moved.coords[moved.coords.len() - 1], fence.coords[1]);
        for x in [0.0, 5.0, 12.0, 28.0, 35.0, 40.0] {
            assert!(
                (y_at(moved, x) - 0.45).abs() < 1e-9,
                "{x}: {:?}",
                moved.coords
            );
        }
        for x in [14.0, 20.0, 26.0] {
            let y = y_at(moved, x);
            assert!((apart + gap..apart + gap + 0.01).contains(&y), "{x}: {y}");
        }
        assert!((length_mm(moved) - 40.0).abs() < 0.1, "{out:?}");
        assert_eq!(
            (report.shifted, report.cut, report.gave_way),
            (1, 0, 0),
            "{report:?}"
        );
        assert_eq!(apart_left(&set, &out), Vec::new(), "{out:?}");
        assert_eq!(of_code(&set, &out, "505"), vec![&path]);
    }

    #[test]
    fn a_path_end_on_the_shifted_fence_follows() {
        // A path comes from beyond the fence and ends on it; the fence moves off the path along
        // it, towards the coming path, and the path's end follows onto the moved fence.
        let set = isom();
        let path = line(&set, "505", &[(-5.0, 0.0), (25.0, 0.0)]);
        let fence = line(&set, "518", &[(0.0, 0.45), (20.0, 0.45)]);
        let coming = line(&set, "505", &[(10.0, 10.0), (10.0, 0.45)]);
        let (out, report) = run(&set, vec![path, fence, coming]);
        let fences = of_code(&set, &out, "518");
        assert_eq!(fences.len(), 1, "{out:?}");
        let moved_to = y_at(fences[0], 10.0);
        assert!(moved_to > 0.5, "{:?}", fences[0].coords);
        let id = set.id_for_code("505").unwrap();
        let comings: Vec<&MapObject> = out
            .iter()
            .filter(|o| o.symbol_id == id && mm(&o.coords[0]) == (10.0, 10.0))
            .collect();
        assert_eq!(comings.len(), 1, "{out:?}");
        let end = mm(&comings[0].coords[comings[0].coords.len() - 1]);
        let touch = PaperMm::of_metres(TOUCH_M, SCALE).0 + TOLERANCE_MM;
        assert!(
            (end.0 - 10.0).abs() < 1e-9 && (end.1 - moved_to).abs() <= touch,
            "{end:?} vs {moved_to}"
        );
        assert_eq!(report.shifted, 1, "{report:?}");
        assert!(report.junctions_repaired >= 1, "{report:?}");
        assert_eq!(report.junctions_free, 0, "{report:?}");
        assert_eq!(apart_left(&set, &out), Vec::new(), "{out:?}");
    }

    #[test]
    fn a_shift_that_would_bring_a_fence_near_a_point_is_refused() {
        // The fence runs towards -x, so its ticks lie on its right on paper, towards -y: on the
        // side of the path. On its far side a point symbol lies just beyond the gap: moved off
        // the path, the fence would come too close to it, so it is cut as before. The control
        // run, with the point further off, shows that the fence shifts without it.
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let marker = point(&set, "418", (0.0, 0.0));
        let frame = Footprints::of(&[&marker], &set)
            .footprint(0)
            .bounding_rect()
            .unwrap();
        let radius = frame.max().y;
        let path = line(&set, "505", &[(-5.0, 0.0), (12.0, 0.0)]);
        let fence = line(&set, "518", &[(20.0, 0.45), (0.0, 0.45)]);
        let beyond = 0.45 + half_width(&set, "518") + gap + radius + 0.01;
        let far = point(&set, "418", (6.0, beyond + 1.0));
        let (out, report) = run(&set, vec![path.clone(), fence.clone(), far.clone()]);
        assert_eq!((report.shifted, report.cut), (1, 0), "{report:?} {out:?}");
        assert_eq!(of_code(&set, &out, "418"), vec![&far]);
        let marker = point(&set, "418", (6.0, beyond));
        assert_eq!(
            apart_left(&set, &[fence.clone(), marker.clone()]),
            Vec::new(),
            "the point keeps the gap at first"
        );
        let (out, report) = run(&set, vec![path.clone(), fence, marker.clone()]);
        let fences = of_code(&set, &out, "518");
        assert_eq!(fences.len(), 1, "{out:?}");
        assert!(fences[0].coords.iter().all(|c| c.y == 450), "{out:?}");
        assert!((length_mm(fences[0]) - 7.6).abs() < 0.5, "{out:?}");
        assert_eq!((report.shifted, report.cut), (0, 1), "{report:?}");
        assert_eq!(of_code(&set, &out, "418"), vec![&marker]);
        assert_eq!(of_code(&set, &out, "505"), vec![&path]);
    }

    #[test]
    fn a_shift_that_would_make_a_joined_conflict_is_refused() {
        // A ditch beside a path, a second ditch just beyond its gap on the far side: moved off
        // the path, the first ditch would run too close to the second, which it joins (a new
        // conflict of the network of the water), so it gives way as before.
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let (h505, h306) = (half_width(&set, "505"), half_width(&set, "306"));
        let y = h505 + h306 + 0.1;
        let beyond = y + 2.0 * h306 + gap + 0.01;
        let path = line(&set, "505", &[(-5.0, 0.0), (25.0, 0.0)]);
        let ditch = line(&set, "306", &[(0.0, y), (20.0, y)]);
        let other = line(&set, "306", &[(0.0, beyond), (20.0, beyond)]);
        let refs = [&ditch, &other];
        assert_eq!(
            conflicts(&Footprints::of(&refs, &set), MapKind::Forest, SCALE),
            Vec::new(),
            "the ditches keep the gap at first"
        );
        let (out, report) = run(&set, vec![path.clone(), ditch, other.clone()]);
        assert_eq!(out, vec![path, other]);
        assert_eq!((report.shifted, report.gave_way), (0, 1), "{report:?}");
    }

    #[test]
    fn a_shift_across_the_seam_of_a_closed_fence_keeps_the_bound() {
        // A closed fence round a yard whose first point lies in the middle of the side along a
        // path: the stretch across the seam moves once, by the shift, and what moved is
        // recorded on both sides of the seam, so it does not move again.
        let set = isom();
        let symbols = RenderSymbols::from_set(&set);
        let classes = Classes::of(&set);
        let gap = gap(MapKind::Forest, SCALE).0;
        let path = line(&set, "505", &[(-5.0, 0.0), (25.0, 0.0)]);
        let corners = [
            (10.0, 0.3),
            (20.0, 0.3),
            (20.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.3),
            (10.0, 0.3),
        ];
        let mut fence = line(&set, "518", &corners);
        let end = fence.coords.len() - 1;
        fence.coords[end].flags |= FLAG_CLOSE;
        let objects = vec![path.clone(), fence.clone()];
        let shifter = Shifter {
            symbols,
            classes,
            kind: MapKind::Forest,
            scale_denominator: SCALE,
            gap,
        };
        let scene = Scene::new(&objects, &shifter);
        let Shift {
            object: moved,
            ranges,
            ..
        } = shifter
            .shift(&objects, &scene, 1, 0, &[])
            .expect("the fence moves off the path");
        let missing = half_width(&set, "505") + half_width(&set, "518") + gap - 0.3;
        let seam = mm(&moved.coords[0]);
        assert!((seam.0 - 10.0).abs() < 1e-3, "{seam:?}");
        assert!(
            (seam.1 - (0.3 + missing)).abs() < 0.005,
            "{seam:?}: once, by {missing}"
        );
        let original = Polyline::new(corners.iter().map(|&(x, y)| Coord { x, y }).collect());
        for c in &moved.coords {
            let (x, y) = mm(c);
            let p = Coord { x, y };
            let off = distance2(p, original.point(original.locate(p))).sqrt();
            assert!(off <= MAX_SHIFT_MM.0 + TOLERANCE_MM, "{p:?} moved {off}");
        }
        assert!(moved.coords[moved.coords.len() - 1].flags & FLAG_CLOSE != 0);
        assert_eq!(
            mm(&moved.coords[0]),
            mm(&moved.coords[moved.coords.len() - 1])
        );
        assert_eq!(
            apart_left(&set, &[path, moved.clone()]),
            Vec::new(),
            "{moved:?}"
        );
        let total = original.length();
        assert!(
            ranges.iter().any(|r| r.0 <= TOLERANCE_MM) && ranges.iter().any(|r| r.1 >= total - 0.1),
            "{ranges:?} of {total}"
        );
        assert!(shifter.shift(&objects, &scene, 1, 0, &ranges).is_none());
    }
}
