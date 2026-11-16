//! The light level: the base map is cleaned lightly, and nothing on it moves (spec 2026-11-14,
//! Part 2, "The light level"; ADR 0157).
//!
//! The base map is the mapper's working material: every object lies where its source has it.
//! The light level only takes objects off it that a mapper cannot read there anyway, and puts
//! them among the candidates, where the candidate layer of their kind (ADR 0156) and the check
//! print show them. Three steps, on the objects routed to the map:
//!
//! 1. lines of one symbol and one source (`freemapper:source`) that meet end to end are joined
//!    ([`join_lines_by`]), so the dash pattern runs through and short pieces stop being objects of
//!    their own; the joined line names the others in `freemapper:also`;
//! 2. of every pair of symbols that keep apart ([`Relation::Apart`]) whose drawn footprints cover
//!    each other ([`Grade::Overlap`]), the line of lower rank ([`rank`]) gives way: it leaves whole
//!    when more than half of it lies on the other, else only its stretch on the other leaves, and
//!    what stays of it must reach its minimum length. A gap conflict without overlap stays: both
//!    objects can still be told apart. Points and areas stay, and so do the pairs of one network
//!    ([`Relation::Joined`]);
//! 3. what is below the norm's minimum size leaves ([`too_small`]).
//!
//! What leaves carries `freemapper:generalized` with the reason and no `freemapper:hidden`. No
//! vertex of what stays moves: a cut line keeps its own vertices and gets new ones only at the
//! cut ends, on its course.

use std::collections::{BTreeMap, BTreeSet};

use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{BoundingRect as _, Intersects as _};
use geo_types::{Coord, Line, Point, Polygon, Rect};

use crate::build::conflicts::{
    Class, Classes, Conflict, Footprints, Grade, MIN_CONFLICT, PairMemo, Relation, TOLERANCE_MM,
    gap, rank,
};
use crate::build::line_join::join_lines_by;
use crate::build::training::{
    TooSmall, add_generalized, class_name, conflict_zones, join_ids, min_length, too_small,
};
use crate::georef::Georeferencing;
use crate::measure::SymbolScale;
use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::render::symbols::RenderSymbols;

/// Give way stops after this many rounds, as the training level's.
const MAX_ROUNDS: usize = 20;
/// The tag whose value is the source of an object; lines join only within one.
const SOURCE_KEY: &str = "freemapper:source";
/// The beginning of the `freemapper:generalized` value of a line or stretch that gave way; the
/// classes it lay on follow.
pub const HIDDEN_BY: &str = "hidden by ";
/// The beginning of the value on the pieces of a cut line that stay on the map.
pub const CUT_WHERE_HIDDEN_BY: &str = "cut where hidden by ";
/// The value on an object below the norm's minimum size.
pub const BELOW_MINIMUM: &str = "below the minimum size";
/// Two positions on a line this close (mm on paper, half a map unit) are one.
const SAME_MM: f64 = 0.0005;

/// What the light level did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LightReport {
    /// Objects that went into another by a join.
    pub joined: usize,
    /// Lines that left the map whole because more than half of them lay on another symbol.
    pub gave_way: usize,
    /// Lines of which a stretch left the map.
    pub cut: usize,
    /// Lines and areas below the norm's minimum size.
    pub too_small: usize,
}

/// Cleans `map` lightly; returns what left it, each object tagged with the reason, in the order
/// the steps took them off.
pub fn light_level(
    map: &mut Vec<MapObject>,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> (Vec<MapObject>, LightReport) {
    let classes = Classes::of(symbol_set);
    let mut taken = Vec::new();
    let mut report = LightReport {
        joined: join(map, &classes, georef),
        ..LightReport::default()
    };
    (report.gave_way, report.cut) = give_way(map, &mut taken, &classes, kind, georef, symbol_set);
    report.too_small = leave_too_small(map, &mut taken, &classes, kind, georef, symbol_set);
    (taken, report)
}

/// Step 1: the lines the training level joins ([`join_ids`]) are joined where they have one
/// symbol and one source and meet end to end, until no join is left. Returns how many objects
/// went.
fn join(map: &mut Vec<MapObject>, classes: &Classes, georef: &Georeferencing) -> usize {
    let (ids, sided) = join_ids(classes);
    let source = |o: &MapObject| {
        o.tags
            .iter()
            .find(|(k, _)| k == SOURCE_KEY)
            .map(|(_, v)| v.clone())
    };
    let mut joined = 0;
    loop {
        let n = join_lines_by(map, &ids, &sided, georef, source);
        if n == 0 {
            return joined;
        }
        joined += n;
    }
}

/// The classes give way measures: points and the areas outside the table never give way to a
/// line or make one give way here.
const MEASURED: [Class; 12] = [
    Class::Building,
    Class::WaterArea,
    Class::BankLine,
    Class::Railway,
    Class::Road,
    Class::Path,
    Class::Watercourse,
    Class::Stairway,
    Class::Barrier,
    Class::Cliff,
    Class::EarthBank,
    Class::Contour,
];

/// Step 2: every line that lies on a symbol it keeps apart from (an Apart conflict of grade
/// Overlap, the line the loser) gives way, by [`split`]. A loser waits while one of its winners
/// still has to give way itself, since that one may go; of losers that all wait, the lowest in
/// rank goes first (the longer of one rank). A line the cut cannot reach stays as it is. The
/// rounds measure again only the pairs of what changed ([`PairMemo`]): the objects keep their
/// slots, a piece gets a new one, and the map is put together in the order of the input, each
/// line's pieces at its place. Returns how many lines left whole and how many were cut.
fn give_way(
    map: &mut Vec<MapObject>,
    taken: &mut Vec<MapObject>,
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> (usize, usize) {
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let gap = gap(kind, scale_denominator).0;
    let symbols = RenderSymbols::from_set(symbol_set);
    let (mut whole, mut cut) = (0, 0);
    let mut slots: Vec<Option<MapObject>> = std::mem::take(map).into_iter().map(Some).collect();
    // Where each slot goes in the map: the place of its line, then the piece's place among the
    // pieces, so a piece cut again stays in the order of its line.
    let mut order: Vec<Vec<usize>> = (0..slots.len()).map(|i| vec![i]).collect();
    let refs: Vec<&MapObject> = slots.iter().flatten().collect();
    let mut footprints = Footprints::of_only(&refs, symbol_set, &MEASURED);
    let mut memo = PairMemo::default();
    // Lines that stay although they lie on another symbol.
    let mut stuck: BTreeSet<usize> = BTreeSet::new();
    for _ in 0..MAX_ROUNDS {
        let found: Vec<Conflict> = memo
            .conflicts_where(&footprints, kind, scale_denominator, |r| {
                r == Relation::Apart
            })
            .into_iter()
            .filter(|c| {
                c.grade == Grade::Overlap
                    && footprints.length(c.loser) > 0.0
                    && !stuck.contains(&c.loser)
            })
            .collect();
        if found.is_empty() {
            break;
        }
        let mut by_loser: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for conflict in &found {
            by_loser
                .entry(conflict.loser)
                .or_default()
                .push(conflict.winner);
        }
        let mut ready: Vec<usize> = by_loser
            .iter()
            .filter(|(_, winners)| winners.iter().all(|w| !by_loser.contains_key(w)))
            .map(|(&loser, _)| loser)
            .collect();
        if ready.is_empty() {
            // Ascending rank; of one rank the longer first, as the training level.
            ready.extend(by_loser.keys().copied().min_by(|&a, &b| {
                rank(footprints.class(a))
                    .cmp(&rank(footprints.class(b)))
                    .then(footprints.length(b).total_cmp(&footprints.length(a)))
                    .then(a.cmp(&b))
            }));
        }
        let mut changed: BTreeSet<usize> = BTreeSet::new();
        let mut added: Vec<(usize, MapObject)> = Vec::new();
        for loser in ready {
            let winners = &by_loser[&loser];
            let Some(object) = slots[loser].as_ref() else {
                continue;
            };
            let mut names: Vec<&str> = winners
                .iter()
                .map(|&w| class_name(footprints.class(w)))
                .collect();
            names.sort_unstable();
            names.dedup();
            let names = names.join(", ");
            let zones = conflict_zones(
                loser,
                winners,
                &footprints,
                footprints.half_width(loser) + TOLERANCE_MM,
                gap,
            );
            let min_mm = min_length(object, classes, symbol_set, scale).0;
            match split(object, &zones, min_mm) {
                Split::Whole => {
                    let mut gone = object.clone();
                    add_generalized(&mut gone, &format!("{HIDDEN_BY}{names}"));
                    taken.push(gone);
                    whole += 1;
                }
                Split::Pieces { kept, gone } => {
                    for mut piece in gone {
                        add_generalized(&mut piece, &format!("{HIDDEN_BY}{names}"));
                        taken.push(piece);
                    }
                    for mut piece in kept {
                        add_generalized(&mut piece, &format!("{CUT_WHERE_HIDDEN_BY}{names}"));
                        added.push((loser, piece));
                    }
                    cut += 1;
                }
                Split::Untouched => {
                    stuck.insert(loser);
                    continue;
                }
            }
            slots[loser] = None;
            footprints.undraw(loser);
            changed.insert(loser);
        }
        memo.forget(&changed);
        let mut nth: BTreeMap<usize, usize> = BTreeMap::new();
        for (line, piece) in added {
            let n = nth.entry(line).or_default();
            let mut place = order[line].clone();
            place.push(*n);
            *n += 1;
            let i = footprints.push(&piece, &symbols, classes);
            debug_assert_eq!(i, slots.len());
            slots.push(Some(piece));
            order.push(place);
        }
    }
    let mut placed: Vec<(Vec<usize>, MapObject)> = order
        .into_iter()
        .zip(slots)
        .filter_map(|(place, object)| Some((place, object?)))
        .collect();
    placed.sort_by(|a, b| a.0.cmp(&b.0));
    *map = placed.into_iter().map(|(_, object)| object).collect();
    (whole, cut)
}

/// A stretch of a line: from and to which position along it, mm on paper.
type Stretch = (f64, f64);

/// What [`split`] makes of a line.
#[derive(Debug, PartialEq)]
enum Split {
    /// The line leaves whole.
    Whole,
    /// The pieces that stay, and those that leave.
    Pieces {
        kept: Vec<MapObject>,
        gone: Vec<MapObject>,
    },
    /// Nothing to cut: no stretch of at least [`MIN_CONFLICT`] lies in the zones, or the line is
    /// not a plain line of one part.
    Untouched,
}

/// The line `object` cut where it lies in `zones` (mm on paper): its stretches inside, each at
/// least [`MIN_CONFLICT`] long, leave, and so does a stretch between them shorter than `min_mm`;
/// the line leaves whole when more than half of it is inside or nothing reaches `min_mm`. Every
/// vertex of the line stays where it is; a piece gets a new vertex only at a cut end, on the
/// line's course. A piece of a closed line is open.
fn split(object: &MapObject, zones: &[Polygon<f64>], min_mm: f64) -> Split {
    let Some((points, closed)) = plain_line(object) else {
        return Split::Untouched;
    };
    let mm: Vec<Coord<f64>> = points
        .iter()
        .map(|c| Coord {
            x: f64::from(c.x) / 1000.0,
            y: f64::from(c.y) / 1000.0,
        })
        .collect();
    let mut along = vec![0.0];
    for w in mm.windows(2) {
        let last = along[along.len() - 1];
        along.push(last + (w[1].x - w[0].x).hypot(w[1].y - w[0].y));
    }
    let total = along[along.len() - 1];
    if total <= 0.0 {
        return Split::Untouched;
    }
    let inside = inside_stretches(&mm, &along, zones, closed);
    let in_conflict: f64 = inside.iter().map(|&(a, b)| b - a).sum();
    if in_conflict <= 0.0 {
        return Split::Untouched;
    }
    if in_conflict > total / 2.0 + TOLERANCE_MM {
        return Split::Whole;
    }
    // The stretches between, in order along the line; on a closed line the one across its start
    // runs from the last stretch inside round to the first.
    let mut outside: Vec<(f64, f64)> = Vec::new();
    let mut from = 0.0;
    for &(a, b) in &inside {
        outside.push((from, a));
        from = b;
    }
    outside.push((from, total));
    if closed {
        let last = outside.pop().unwrap_or((total, total));
        let first = outside.remove(0);
        outside.push((last.0, total + first.1));
    }
    let outside: Vec<(f64, f64)> = outside
        .into_iter()
        .filter(|&(a, b)| b - a > SAME_MM)
        .collect();
    let (kept, short): (Vec<Stretch>, Vec<Stretch>) = outside
        .into_iter()
        .partition(|&(a, b)| b - a >= min_mm - TOLERANCE_MM);
    if kept.is_empty() {
        return Split::Whole;
    }
    // What leaves: the stretches inside and the short ones between, each run of them one piece.
    let mut leaving: Vec<(f64, f64)> = inside.into_iter().chain(short).collect();
    leaving.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut runs: Vec<(f64, f64)> = Vec::new();
    for (a, b) in leaving {
        match runs.last_mut() {
            Some(run) if a - run.1 <= SAME_MM => run.1 = run.1.max(b),
            _ => runs.push((a, b)),
        }
    }
    if closed && runs.len() > 1 {
        // A run that ends at the line's end goes on at its start.
        let (first, last) = (runs[0], runs[runs.len() - 1]);
        if first.0 <= SAME_MM && total - last.1 <= SAME_MM {
            runs.remove(0);
            let end = runs.len() - 1;
            runs[end].1 = total + first.1;
        }
    }
    let piece = |(a, b): (f64, f64)| -> Option<MapObject> {
        let coords = stretch(&points, &mm, &along, a, b, closed);
        (coords.len() >= 2).then(|| MapObject {
            symbol_id: object.symbol_id,
            kind: MapObjectKind::Path,
            coords,
            tags: object.tags.clone(),
        })
    };
    Split::Pieces {
        kept: kept.into_iter().filter_map(piece).collect(),
        gone: runs.into_iter().filter_map(piece).collect(),
    }
}

/// The vertices of `object` when it is a plain line of one part without curves: open, or closed
/// (its last coordinate closes it on the first, as `closed_ring` writes it), with the closing
/// coordinate as a plain copy of the first. `None` for anything else.
fn plain_line(object: &MapObject) -> Option<(Vec<MapCoord>, bool)> {
    if !matches!(object.kind, MapObjectKind::Path) || object.coords.len() < 2 {
        return None;
    }
    let (last, inner) = object.coords.split_last()?;
    if inner.iter().any(|c| c.flags != 0) {
        return None;
    }
    let mut points: Vec<MapCoord> = inner.to_vec();
    let closed = last.flags == FLAG_HOLE | FLAG_CLOSE || last.flags == FLAG_CLOSE;
    if !closed && last.flags != 0 {
        return None;
    }
    points.push(MapCoord::new(last.x, last.y));
    if closed && (points[0].x, points[0].y) != (last.x, last.y) {
        points.push(MapCoord::new(points[0].x, points[0].y));
    }
    let closed = closed && points.len() > 3;
    Some((points, closed))
}

/// The stretches (positions along the line, mm on paper, ascending) of the line through `mm`,
/// whose positions are `along`, inside `zones`, each at least [`MIN_CONFLICT`] long; on a closed
/// line a stretch over its start counts once, with the part after the start and the part before
/// the end added.
fn inside_stretches(
    mm: &[Coord<f64>],
    along: &[f64],
    zones: &[Polygon<f64>],
    closed: bool,
) -> Vec<(f64, f64)> {
    let frames: Vec<Option<Rect<f64>>> = zones.iter().map(|z| z.bounding_rect()).collect();
    let mut runs: Vec<(f64, f64)> = Vec::new();
    for (k, w) in mm.windows(2).enumerate() {
        let segment = Line::new(w[0], w[1]);
        let frame = segment.bounding_rect();
        let near: Vec<&Polygon<f64>> = zones
            .iter()
            .zip(&frames)
            .filter(|(_, f)| f.is_some_and(|f| f.intersects(&frame)))
            .map(|(z, _)| z)
            .collect();
        if near.is_empty() {
            continue;
        }
        let length = along[k + 1] - along[k];
        if length <= 0.0 {
            continue;
        }
        // Where the segment crosses an edge of a zone, as fractions of it.
        let mut cuts = vec![0.0, 1.0];
        let d = w[1] - w[0];
        let at = |p: Coord<f64>| ((p - w[0]).x * d.x + (p - w[0]).y * d.y) / (length * length);
        for zone in &near {
            for ring in std::iter::once(zone.exterior()).chain(zone.interiors()) {
                for edge in ring.lines() {
                    if !edge.bounding_rect().intersects(&frame) {
                        continue;
                    }
                    match line_intersection(segment, edge) {
                        Some(LineIntersection::SinglePoint { intersection, .. }) => {
                            cuts.push(at(intersection));
                        }
                        Some(LineIntersection::Collinear { intersection }) => {
                            cuts.push(at(intersection.start));
                            cuts.push(at(intersection.end));
                        }
                        None => {}
                    }
                }
            }
        }
        cuts.retain(|t| (0.0..=1.0).contains(t));
        cuts.sort_by(f64::total_cmp);
        cuts.dedup_by(|a, b| (*a - *b) * length <= SAME_MM / 10.0);
        for t in cuts.windows(2) {
            let mid = w[0] + d * ((t[0] + t[1]) / 2.0);
            if near.iter().any(|z| z.intersects(&Point(mid))) {
                let (a, b) = (along[k] + t[0] * length, along[k] + t[1] * length);
                match runs.last_mut() {
                    Some(run) if a - run.1 <= SAME_MM => run.1 = b,
                    _ => runs.push((a, b)),
                }
            }
        }
    }
    let total = along[along.len() - 1];
    let counts = |l: f64| l > MIN_CONFLICT.0 + TOLERANCE_MM;
    if closed && runs.len() > 1 {
        let (first, last) = (runs[0], runs[runs.len() - 1]);
        if first.0 <= SAME_MM && total - last.1 <= SAME_MM {
            // One stretch over the start: kept, or not, as a whole.
            let over = (first.1 - first.0) + (last.1 - last.0);
            let end = runs.len() - 1;
            if !counts(over) {
                runs.remove(end);
                runs.remove(0);
            } else {
                // Each part stays a stretch of its own, both kept.
                runs[0].0 = 0.0;
                runs[end].1 = total;
                let middle: Vec<(f64, f64)> = runs[1..end]
                    .iter()
                    .copied()
                    .filter(|&(a, b)| counts(b - a))
                    .collect();
                let mut kept = vec![runs[0]];
                kept.extend(middle);
                kept.push(runs[end]);
                return kept;
            }
        }
    }
    runs.into_iter().filter(|&(a, b)| counts(b - a)).collect()
}

/// The coordinates of the line `points` (paper mm `mm`, positions `along`) from position `a` to
/// `b`: the vertices between as they are, a new vertex at each end unless a vertex lies there. On
/// a closed line `b` may pass its end and go on from its start.
fn stretch(
    points: &[MapCoord],
    mm: &[Coord<f64>],
    along: &[f64],
    a: f64,
    b: f64,
    closed: bool,
) -> Vec<MapCoord> {
    let total = along[along.len() - 1];
    let mut out: Vec<MapCoord> = Vec::new();
    let mut push = |c: MapCoord| {
        if out
            .last()
            .is_none_or(|l: &MapCoord| (l.x, l.y) != (c.x, c.y))
        {
            out.push(MapCoord::new(c.x, c.y));
        }
    };
    let at = |s: f64| -> MapCoord {
        let k = along
            .windows(2)
            .position(|w| s <= w[1] + SAME_MM)
            .unwrap_or(along.len() - 2);
        if (s - along[k]).abs() <= SAME_MM {
            return points[k];
        }
        if (s - along[k + 1]).abs() <= SAME_MM {
            return points[k + 1];
        }
        let t = (s - along[k]) / (along[k + 1] - along[k]);
        let p = mm[k] + (mm[k + 1] - mm[k]) * t;
        MapCoord::new((p.x * 1000.0).round() as i32, (p.y * 1000.0).round() as i32)
    };
    push(at(a));
    let (end, wraps) = if closed && b > total + SAME_MM {
        (total, true)
    } else {
        (b.min(total), false)
    };
    for (k, &s) in along.iter().enumerate() {
        if s > a + SAME_MM && s < end - SAME_MM {
            push(points[k]);
        }
    }
    if wraps {
        push(points[points.len() - 1]);
        let rest = b - total;
        for (k, &s) in along.iter().enumerate().skip(1) {
            if s < rest - SAME_MM {
                push(points[k]);
            }
        }
        push(at(rest));
    } else {
        push(at(end));
    }
    out
}

/// Step 3: the lines of the classes of the measure (not the others, so the north lines and the
/// borders of paved areas stay) and the areas with a minimum size that are below it leave
/// ([`too_small`]). A stairway below its minimum stays where both its ends continue on a way
/// that stays: on the base map it stays a stairway. Returns how many left.
fn leave_too_small(
    map: &mut Vec<MapObject>,
    taken: &mut Vec<MapObject>,
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let considered = |o: &MapObject| classes.class(o.symbol_id) != Class::Other;
    let TooSmall { gone, .. } = too_small(map, classes, kind, georef, symbol_set, &considered);
    let mut n = 0;
    let before = std::mem::take(map);
    for (object, gone) in before.into_iter().zip(gone) {
        if gone {
            let mut object = object;
            add_generalized(&mut object, BELOW_MINIMUM);
            taken.push(object);
            n += 1;
        } else {
            map.push(object);
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::candidate_layers::{CandidateLayer, split as layers};
    use crate::build::conflicts::conflicts;
    use crate::build::points::HIDDEN_KEY;
    use crate::build::training::tests::{
        SCALE, area, georef_at, half_width, isom, length_mm, line, of_code, point,
    };
    use crate::omap::closed_ring;

    fn run(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, Vec<MapObject>) {
        let mut map = objects;
        let (taken, _) = light_level(&mut map, MapKind::Forest, &georef_at(SCALE), set);
        (map, taken)
    }

    fn from(mut object: MapObject, source: &str, id: &str) -> MapObject {
        object.tags = vec![
            (SOURCE_KEY.to_string(), source.to_string()),
            ("freemapper:source_id".to_string(), id.to_string()),
        ];
        object
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The overlap conflicts of pairs that keep apart among `objects`.
    fn overlaps(set: &SymbolSet, objects: &[MapObject]) -> Vec<Conflict> {
        let refs: Vec<&MapObject> = objects.iter().collect();
        conflicts(&Footprints::of(&refs, set), MapKind::Forest, SCALE)
            .into_iter()
            .filter(|c| {
                c.grade == Grade::Overlap
                    && crate::build::conflicts::relation(c.classes.0, c.classes.1)
                        == Relation::Apart
            })
            .collect()
    }

    /// Every vertex of the objects `after` is one of `before`, but the ends of the pieces of a
    /// cut line, which lie on its course.
    fn no_vertex_moved(before: &[MapObject], after: &[MapObject]) {
        let vertices: BTreeSet<(i32, i32)> = before
            .iter()
            .flat_map(|o| o.coords.iter().map(|c| (c.x, c.y)))
            .collect();
        for object in after {
            let cut = tag(object, "freemapper:generalized")
                .is_some_and(|v| v.contains(CUT_WHERE_HIDDEN_BY) || v.contains(HIDDEN_BY));
            let n = object.coords.len();
            for (i, c) in object.coords.iter().enumerate() {
                let end = i == 0 || i + 1 == n;
                assert!(
                    vertices.contains(&(c.x, c.y)) || (cut && end),
                    "{c:?} of {object:?} moved"
                );
            }
        }
    }

    #[test]
    fn paths_of_one_source_that_meet_end_to_end_are_joined() {
        let set = isom();
        let a = from(line(&set, "505", &[(0.0, 0.0), (5.0, 0.0)]), "osm", "way/1");
        let b = from(
            line(&set, "505", &[(5.0, 0.0), (12.0, 1.0)]),
            "osm",
            "way/2",
        );
        let (map, taken) = run(&set, vec![a.clone(), b.clone()]);
        assert!(taken.is_empty());
        assert_eq!(map.len(), 1);
        assert_eq!(tag(&map[0], "freemapper:source_id"), Some("way/2"));
        assert_eq!(tag(&map[0], "freemapper:also"), Some("osm/way/1"));
        no_vertex_moved(&[a.clone(), b.clone()], &map);
        // Of two sources, or of two symbols, they stay two.
        let c = from(b.clone(), "alkis", "s1");
        assert_eq!(run(&set, vec![a.clone(), c]).0.len(), 2);
        let mut d = b.clone();
        d.symbol_id = set.id_for_code("506").unwrap();
        assert_eq!(run(&set, vec![a, d]).0.len(), 2);
    }

    #[test]
    fn an_earth_bank_mostly_on_a_path_leaves_whole_for_the_terrain_layer() {
        let set = isom();
        let path = line(&set, "505", &[(-5.0, 0.0), (30.0, 0.0)]);
        let bank = line(&set, "104", &[(0.0, 0.0), (10.0, 0.0)]);
        let (map, taken) = run(&set, vec![path.clone(), bank.clone()]);
        assert_eq!(map, [path]);
        assert_eq!(taken.len(), 1);
        assert_eq!(taken[0].coords, bank.coords);
        assert_eq!(
            tag(&taken[0], "freemapper:generalized"),
            Some("hidden by path")
        );
        assert_eq!(tag(&taken[0], HIDDEN_KEY), None);
        // The candidate layers put it with the terrain.
        let mut candidates = vec![area(&set, "520", (0.0, 0.0), (1.0, 1.0))];
        candidates.extend(taken);
        let layer = layers(&candidates, &set)
            .into_iter()
            .find(|(layer, _)| *layer == CandidateLayer::Terrain)
            .unwrap();
        assert!(layer.1.contains(&candidates[1]));
    }

    #[test]
    fn of_a_bank_partly_on_a_path_only_the_stretch_leaves_and_no_vertex_moves() {
        let set = isom();
        let path = line(&set, "505", &[(-5.0, 0.0), (6.0, 0.0)]);
        let bank = line(
            &set,
            "104",
            &[
                (0.0, 0.0),
                (3.0, 0.0),
                (10.0, 0.0),
                (15.0, 0.0),
                (20.0, 0.0),
            ],
        );
        let before = vec![path.clone(), bank.clone()];
        let (map, taken) = run(&set, before.clone());
        let banks = of_code(&set, &map, "104");
        assert_eq!(banks.len(), 1);
        let kept = banks[0];
        assert_eq!(
            tag(kept, "freemapper:generalized"),
            Some("cut where hidden by path")
        );
        for vertex in [(10_000, 0), (15_000, 0), (20_000, 0)] {
            assert!(kept.coords.iter().any(|c| (c.x, c.y) == vertex));
        }
        assert_eq!(taken.len(), 1);
        assert_eq!(
            tag(&taken[0], "freemapper:generalized"),
            Some("hidden by path")
        );
        assert!(taken[0].coords.iter().any(|c| (c.x, c.y) == (3_000, 0)));
        // Map and candidates together are the bank.
        let together = length_mm(kept) + length_mm(&taken[0]);
        assert!((together - 20.0).abs() < 0.01, "{together}");
        no_vertex_moved(&before, &map);
        assert!(overlaps(&set, &map).is_empty());
    }

    #[test]
    fn a_bank_beside_a_path_inside_the_gap_stays() {
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let y = half_width(&set, "104") + half_width(&set, "505") + gap / 2.0;
        let path = line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]);
        let bank = line(&set, "104", &[(0.0, y), (20.0, y)]);
        let (map, taken) = run(&set, vec![path, bank]);
        assert!(taken.is_empty());
        assert_eq!(map.len(), 2);
    }

    /// `loser` drawn on the course of `winner` for half of the winner's length leaves whole and
    /// the winner stays.
    fn gives_way(set: &SymbolSet, loser: &str, winner: &str) {
        let (Some(_), Some(_)) = (set.id_for_code(loser), set.id_for_code(winner)) else {
            return;
        };
        let w = line(set, winner, &[(0.0, 0.0), (20.0, 0.0)]);
        let l = line(set, loser, &[(2.0, 0.0), (12.0, 0.0)]);
        let (map, taken) = run(set, vec![l.clone(), w.clone()]);
        assert_eq!(map, [w], "{loser} under {winner}");
        assert_eq!(taken.len(), 1, "{loser} under {winner}");
        assert_eq!(taken[0].coords, l.coords);
    }

    #[test]
    fn the_ranks_of_the_spec() {
        let set = isom();
        // An earth bank gives way to a watercourse, a bank line, a railway, a road and a path.
        for winner in ["305", "306", "301.4", "509", "503", "505"] {
            gives_way(&set, "104", winner);
        }
        // A fence gives way to a path or road it is drawn on.
        for fence in ["516", "518"] {
            for winner in ["505", "503"] {
                gives_way(&set, fence, winner);
            }
        }
        // A contour piece on an earth bank.
        gives_way(&set, "101", "104");
        // Of two earth banks the shorter.
        gives_way(&set, "104", "104");
    }

    #[test]
    fn pairs_of_one_network_and_points_stay() {
        let set = isom();
        let a = from(
            line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]),
            "osm",
            "way/1",
        );
        let b = from(line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]), "alkis", "s1");
        let road = line(&set, "503", &[(0.0, 0.0), (20.0, 0.0)]);
        let tree = point(&set, "418", (5.0, 0.0));
        let objects = vec![a, b, road, tree];
        let (map, taken) = run(&set, objects.clone());
        assert!(taken.is_empty());
        assert_eq!(map, objects);
    }

    #[test]
    fn what_is_below_the_minimum_leaves() {
        let set = isom();
        let stair = line(&set, "532", &[(0.0, 0.0), (0.5, 0.0)]);
        let ring: Vec<(i32, i32)> = [(0, 0), (200, 0), (200, 200)].to_vec();
        let loop_ = MapObject {
            symbol_id: set.id_for_code("101").unwrap(),
            kind: MapObjectKind::Path,
            coords: closed_ring(&ring),
            tags: Vec::new(),
        };
        let patch = area(&set, "401", (50.0, 50.0), (50.2, 50.2));
        let (map, taken) = run(&set, vec![stair, loop_, patch]);
        assert!(map.is_empty(), "{map:?}");
        assert_eq!(taken.len(), 3);
        for object in &taken {
            assert_eq!(tag(object, "freemapper:generalized"), Some(BELOW_MINIMUM));
            assert_eq!(tag(object, HIDDEN_KEY), None);
        }
    }

    #[test]
    fn a_short_stair_between_two_paths_stays_a_stair() {
        let set = isom();
        let a = line(&set, "505", &[(-20.0, 30.0), (0.0, 30.0)]);
        let stair = line(&set, "532", &[(0.0, 30.0), (0.5, 30.0)]);
        let b = line(&set, "505", &[(0.5, 30.0), (20.0, 35.0)]);
        let (map, taken) = run(&set, vec![a, stair.clone(), b]);
        assert!(taken.is_empty());
        assert!(map.contains(&stair));
    }

    /// A rectangle `x0` to `x1` round the x axis, mm on paper.
    fn zone(x0: f64, x1: f64) -> Polygon<f64> {
        Rect::new(Coord { x: x0, y: -1.0 }, Coord { x: x1, y: 1.0 }).to_polygon()
    }

    #[test]
    fn split_keeps_the_rest_only_above_the_minimum() {
        let set = isom();
        let object = line(&set, "104", &[(0.0, 0.0), (4.0, 0.0), (10.0, 0.0)]);
        // Inside: [0, 1.5] and [3, 4.5]; between them 1.5 mm, below the minimum of 2.
        let Split::Pieces { kept, gone } = split(&object, &[zone(-1.0, 1.5), zone(3.0, 4.5)], 2.0)
        else {
            panic!("pieces");
        };
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].coords.first().map(|c| (c.x, c.y)), Some((4_500, 0)));
        assert_eq!(kept[0].coords.last().map(|c| (c.x, c.y)), Some((10_000, 0)));
        assert_eq!(gone.len(), 1);
        let ends: Vec<(i32, i32)> = gone[0].coords.iter().map(|c| (c.x, c.y)).collect();
        assert_eq!(ends, [(0, 0), (4_000, 0), (4_500, 0)]);
        // Nothing reaches the minimum: the line leaves whole.
        assert_eq!(
            split(&object, &[zone(-1.0, 1.5), zone(3.0, 4.5)], 6.0),
            Split::Whole
        );
        // More than half inside: whole.
        assert_eq!(split(&object, &[zone(-1.0, 5.5)], 1.0), Split::Whole);
        // A stretch under 1 mm is no conflict.
        assert_eq!(split(&object, &[zone(-1.0, 0.5)], 1.0), Split::Untouched);
    }

    #[test]
    fn a_closed_line_is_cut_across_its_start() {
        let set = isom();
        let ring: Vec<(i32, i32)> = vec![(0, 0), (10_000, 0), (10_000, 10_000), (0, 10_000)];
        let object = MapObject {
            symbol_id: set.id_for_code("104").unwrap(),
            kind: MapObjectKind::Path,
            coords: closed_ring(&ring),
            tags: Vec::new(),
        };
        // The zone covers the first 3 mm of the ring: a stretch from (0, 0) to (3, 0).
        let Split::Pieces { kept, gone } = split(&object, &[zone(-1.0, 3.0)], 1.0) else {
            panic!("pieces");
        };
        assert_eq!(gone.len(), 1);
        assert_eq!(kept.len(), 1);
        let kept: Vec<(i32, i32)> = kept[0].coords.iter().map(|c| (c.x, c.y)).collect();
        assert_eq!(
            kept,
            [
                (3_000, 0),
                (10_000, 0),
                (10_000, 10_000),
                (0, 10_000),
                (0, 1_000)
            ]
        );
        let gone: Vec<(i32, i32)> = gone[0].coords.iter().map(|c| (c.x, c.y)).collect();
        assert_eq!(gone, [(0, 1_000), (0, 0), (3_000, 0)]);
    }
}
