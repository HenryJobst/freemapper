//! Step 5 of the training level: merge what is alike (spec 2026-11-14, "The second half, as
//! designed on 2026-11-15", "Merge"). Two lines of one class (roads, railways, paths) that run
//! side by side inside the gap become one line on their middle over that stretch, and on a
//! forest map a path or a tram in the gap zone of a road (a sidewalk, a cycleway) goes. It runs before the points
//! move, so that they move off the merged line. Paper mm, y down, as `conflicts::Footprints`
//! uses; only +, −, ×, ÷ and `sqrt` decide, and every collection is iterated in a sorted order.

use std::collections::{BTreeMap, BTreeSet};

use geo_types::Coord;

use super::junctions::{JUNCTION_TAG, Key, key_of};
use super::lines::{Polyline, beside, middle};
use super::{MAX_ROUNDS, add_generalized, is_closed, min_length};
use crate::build::conflicts::{
    Class, Classes, Conflict, Footprints, MIN_CONFLICT, Relation, TOLERANCE_MM, conflicts, gap,
    meeting_points, relation,
};
use crate::build::dedup::{add_also, tag_holds};
use crate::georef::Georeferencing;
use crate::measure::SymbolScale;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;

/// The spacing of the samples of a middle line (mm on paper); [`middle`] simplifies them with a
/// quarter of it.
const MIDDLE_STEP_MM: f64 = 0.25;
/// The line kept leaves its old course for the middle over this many times the distance between
/// them, as the shift step ramps its stretches (spec: "a ramp of four times the shift").
const RAMP_FACTOR: f64 = 4.0;
/// A tram (`freemapper:osm` holds it): on a forest map it goes where it runs inside a road.
const OSM_TRAM: &str = "railway=tram";
/// The `freemapper:generalized` value of a merged line; ` with ` and the source and id of the
/// other line follow when it has an id.
const MERGED: &str = "merged";
/// The `freemapper:generalized` value of a piece of a path or tram cut where it ran in the gap
/// zone of a road.
const CUT_INSIDE_ROAD: &str = "cut: in the gap of a road";

/// What [`merge`] did.
#[derive(Debug, Default, Clone, PartialEq)]
pub(super) struct Merged {
    /// Pairs that became one line.
    pub(super) merged: usize,
    /// Paths and trams that went because they ran in the gap zone of a road.
    pub(super) dropped_beside_roads: usize,
    /// Paths and trams cut where they ran in the gap zone of a road.
    pub(super) cut: usize,
    /// The line that took the place of another, by junction key: of a merged pair the one that
    /// went (where it ran beside) → the one kept; of a path or tram inside a road → the road.
    pub(super) replaced_by: BTreeMap<Key, Key>,
}

/// Merges what is alike among `objects`. On a forest map first a path (505 to 507 and the
/// aliases of the path class) or a tram (a railway whose `freemapper:osm` holds [`OSM_TRAM`])
/// whose centre line runs in the gap zone of a road (within both half widths and the gap of the
/// road's centre line, the zone of a Joined conflict: a sidewalk or cycleway, which ISOM does
/// not map apart from the road) over more than half its length goes, else its stretches inside
/// are cut out (stretches of at least [`MIN_CONFLICT`], the places
/// where the centre lines cross left out; pieces below their minimum length go, and the end of
/// a piece next to the road is drawn onto the road's centre line). On a sprint map the street
/// space is a paved area (501.5), on which paths and trams are free: it is left alone.
///
/// Then, in rounds, two lines of the same class Road, Railway or Path in conflict (relation
/// Joined) merge where they run within both half widths and the gap of each other over at
/// least the minimum length of the narrower symbol ([`beside`]): the stretch of the kept line
/// (the wider symbol, of the same width the longer, else the earlier) becomes the middle of the
/// two ([`middle`]), joined to its own course before and after by a ramp of [`RAMP_FACTOR`]
/// times the distance; the other line keeps its pieces outside the stretch (each at least its
/// minimum length) and their cut ends are drawn onto the kept line. The kept line carries the
/// tags of the longer one, the other's source and id in `freemapper:also` and `freemapper:
/// generalized` [`MERGED`] with that id; it keeps its own junction key. Closed lines and lines
/// of several parts do not merge. An object merges once per round; the rounds stop when none
/// merges, at the latest after [`MAX_ROUNDS`].
pub(super) fn merge(
    objects: &mut Vec<MapObject>,
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Merged {
    let mut merged = Merged::default();
    if kind == MapKind::Forest {
        inside_roads(objects, classes, kind, georef, symbol_set, &mut merged);
    }
    for _ in 0..MAX_ROUNDS {
        let n = merge_round(objects, classes, kind, georef, symbol_set, &mut merged);
        if n == 0 {
            break;
        }
        merged.merged += n;
    }
    merged
}

/// The centre line of object `i` as one polyline, when it has exactly one part.
fn polyline(footprints: &Footprints, i: usize) -> Option<Polyline> {
    match footprints.centre(i).0.as_slice() {
        [one] if one.0.len() >= 2 => Some(Polyline::new(one.0.clone())),
        _ => None,
    }
}

/// The source and id of `object` as `freemapper:also` names it (as `line_join` does), when it
/// has an id.
fn also_id(object: &MapObject) -> Option<String> {
    let get = |key: &str| {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    };
    let id = get("freemapper:source_id")?;
    Some(match get("freemapper:source") {
        Some(source) => format!("{source}/{id}"),
        None => id.to_string(),
    })
}

/// Whether `object` is a tram by its OSM tags.
fn is_tram(object: &MapObject) -> bool {
    object
        .tags
        .iter()
        .any(|(k, v)| k == "freemapper:osm" && tag_holds(v, OSM_TRAM))
}

/// The forest rule of [`merge`]: paths and trams in the gap zone of a road.
fn inside_roads(
    objects: &mut Vec<MapObject>,
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    out: &mut Merged,
) {
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let gap = gap(kind, scale_denominator).0;
    let refs: Vec<&MapObject> = objects.iter().collect();
    let footprints = Footprints::of(&refs, symbol_set);
    // The roads each path or tram is in conflict with.
    let mut roads_of: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for c in conflicts(&footprints, kind, scale_denominator) {
        let (rider, road) = match c.classes {
            (Class::Road, other) if other != Class::Road => (c.winner, c.loser),
            (other, Class::Road) if other != Class::Road => (c.loser, c.winner),
            _ => continue,
        };
        let rides = match footprints.class(rider) {
            Class::Path => true,
            Class::Railway => is_tram(&objects[rider]),
            _ => false,
        };
        if rides && relation(c.classes.0, c.classes.1) == Relation::Joined {
            roads_of.entry(rider).or_default().insert(road);
        }
    }
    let mut replaced: BTreeMap<usize, Vec<MapObject>> = BTreeMap::new();
    for (&rider, roads) in &roads_of {
        let Some(line) = polyline(&footprints, rider) else {
            continue;
        };
        let rider_half = footprints.half_width(rider);
        let mut inside: Vec<(f64, f64)> = Vec::new();
        let mut crossings: Vec<(f64, f64)> = Vec::new();
        // The road the rider lies in longest (of as long, the earlier), and the road lines.
        let mut longest: Option<(f64, usize)> = None;
        let mut road_lines: Vec<(usize, Polyline)> = Vec::new();
        for &road in roads {
            let Some(road_line) = polyline(&footprints, road) else {
                continue;
            };
            let road_half = footprints.half_width(road);
            // The zone of a Joined conflict: the road's footprint grown by the rider's half
            // width and the gap.
            let radius = rider_half + road_half + gap;
            let mine = beside(&line, &road_line, radius, 0.0);
            for m in meeting_points(footprints.centre(rider), footprints.centre(road)) {
                crossings.extend(beside(&line, &Polyline::new(vec![m]), radius, 0.0));
            }
            let length: f64 = mine.iter().map(|r| r.1 - r.0).sum();
            if length > 0.0 && longest.is_none_or(|(l, _)| length > l + TOLERANCE_MM) {
                longest = Some((length, road));
            }
            inside.extend(mine);
            road_lines.push((road, road_line));
        }
        let mut inside = without(&union(inside), &union(crossings));
        inside.retain(|r| r.1 - r.0 > MIN_CONFLICT.0 + TOLERANCE_MM);
        let in_road: f64 = inside.iter().map(|r| r.1 - r.0).sum();
        let Some((_, road)) = longest else { continue };
        if inside.is_empty() {
            continue;
        }
        let object = &objects[rider];
        let total = line.length();
        let pieces = if in_road > total / 2.0 + TOLERANCE_MM {
            Vec::new()
        } else {
            let min = min_length(object, classes, symbol_set, scale).0;
            // A cut end next to a road is drawn onto the road's centre line.
            let reach = |road_half: f64| rider_half + road_half + gap + TOLERANCE_MM;
            let onto = |c: Coord<f64>| -> Option<Coord<f64>> {
                road_lines
                    .iter()
                    .map(|(road, l)| {
                        let q = l.point(l.locate(c));
                        (distance(c, q), *road, q)
                    })
                    .filter(|&(d, road, _)| d <= reach(footprints.half_width(road)))
                    .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                    .map(|(_, _, q)| q)
            };
            outside(&line, &inside, min)
                .into_iter()
                .map(|(from, to)| {
                    let mut points = line.between(from, to);
                    if from > 0.0
                        && let Some(q) = onto(points[0])
                    {
                        points.insert(0, q);
                    }
                    if to < total
                        && let Some(q) = onto(points[points.len() - 1])
                    {
                        points.push(q);
                    }
                    let mut piece = with_points(object, &points);
                    add_generalized(&mut piece, CUT_INSIDE_ROAD);
                    piece
                })
                .collect()
        };
        if pieces.is_empty() {
            out.dropped_beside_roads += 1;
        } else {
            out.cut += 1;
        }
        if let (Some(from), Some(to)) = (key_of(object), key_of(&objects[road])) {
            out.replaced_by.entry(from).or_insert(to);
        }
        replaced.insert(rider, pieces);
    }
    rebuild(objects, replaced);
}

/// One round of the merge of [`merge`]; returns how many pairs merged.
fn merge_round(
    objects: &mut Vec<MapObject>,
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    out: &mut Merged,
) -> usize {
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let gap = gap(kind, scale_denominator).0;
    let refs: Vec<&MapObject> = objects.iter().collect();
    let footprints = Footprints::of(&refs, symbol_set);
    let found = conflicts(&footprints, kind, scale_denominator);
    let mut pairs: Vec<&Conflict> = found
        .iter()
        .filter(|c| {
            c.classes.0 == c.classes.1
                && matches!(c.classes.0, Class::Road | Class::Railway | Class::Path)
                && relation(c.classes.0, c.classes.1) == Relation::Joined
        })
        .collect();
    // The longest conflicts first.
    pairs.sort_by(|a, b| {
        b.length
            .0
            .total_cmp(&a.length.0)
            .then((a.loser, a.winner).cmp(&(b.loser, b.winner)))
    });
    let mut used: BTreeSet<usize> = BTreeSet::new();
    let mut replaced: BTreeMap<usize, Vec<MapObject>> = BTreeMap::new();
    let mut n = 0;
    for c in pairs {
        if used.contains(&c.loser) || used.contains(&c.winner) {
            continue;
        }
        let pair = Pair {
            objects,
            footprints: &footprints,
            classes,
            symbol_set,
            scale,
            gap,
        };
        let Some((kept, line, other, pieces)) = pair.merged(c.loser, c.winner) else {
            continue;
        };
        used.extend([kept, other]);
        if let (Some(from), Some(to)) = (key_of(&objects[other]), key_of(&objects[kept])) {
            out.replaced_by.entry(from).or_insert(to);
        }
        replaced.insert(kept, vec![line]);
        replaced.insert(other, pieces);
        n += 1;
    }
    rebuild(objects, replaced);
    n
}

/// What one merge of two lines needs to know.
struct Pair<'a> {
    objects: &'a [MapObject],
    footprints: &'a Footprints,
    classes: &'a Classes,
    symbol_set: &'a SymbolSet,
    scale: SymbolScale,
    gap: f64,
}

impl Pair<'_> {
    /// The merge of the lines `a` and `b`, when they run beside each other long enough: the
    /// index of the line kept and its new object, the index of the other and its pieces left.
    fn merged(&self, a: usize, b: usize) -> Option<(usize, MapObject, usize, Vec<MapObject>)> {
        let objects = self.objects;
        if is_closed(&objects[a]) || is_closed(&objects[b]) {
            return None;
        }
        let (line_a, line_b) = (polyline(self.footprints, a)?, polyline(self.footprints, b)?);
        let (half_a, half_b) = (self.footprints.half_width(a), self.footprints.half_width(b));
        let (length_a, length_b) = (line_a.length(), line_b.length());
        let same_width = (half_a - half_b).abs() <= TOLERANCE_MM;
        let same_length = (length_a - length_b).abs() <= TOLERANCE_MM;
        let a_kept = if !same_width {
            half_a > half_b
        } else if !same_length {
            length_a > length_b
        } else {
            a < b
        };
        let min = |i: usize| min_length(&objects[i], self.classes, self.symbol_set, self.scale).0;
        let min_len = if same_width {
            min(a).min(min(b))
        } else if half_a < half_b {
            min(a)
        } else {
            min(b)
        };
        let ((kept, kept_line), (other, other_line)) = if a_kept {
            ((a, line_a), (b, line_b))
        } else {
            ((b, line_b), (a, line_a))
        };
        let within = half_a + half_b + self.gap;
        let on_kept = beside(&kept_line, &other_line, within, min_len);
        let on_other = beside(&other_line, &kept_line, within, min_len);
        if on_kept.is_empty() || on_other.is_empty() {
            return None;
        }
        let points = merged_line(&kept_line, &other_line, &on_kept);
        // The tags of the longer line; of the same length, the kept one's.
        let (longer, shorter) = if same_length || kept_line.length() > other_line.length() {
            (kept, other)
        } else {
            (other, kept)
        };
        let mut tags: Vec<(String, String)> = objects[longer]
            .tags
            .iter()
            .filter(|(k, _)| k != JUNCTION_TAG)
            .cloned()
            .collect();
        tags.extend(
            objects[kept]
                .tags
                .iter()
                .filter(|(k, _)| k == JUNCTION_TAG)
                .cloned(),
        );
        let id = also_id(&objects[shorter]);
        if let Some(id) = &id {
            add_also(&mut tags, id);
        }
        if let Some((_, older)) = objects[shorter]
            .tags
            .iter()
            .find(|(k, _)| k == "freemapper:also")
        {
            for also in older.split("; ") {
                add_also(&mut tags, also);
            }
        }
        let mut line = with_points(&objects[kept], &points);
        line.tags = tags;
        match id {
            Some(id) => add_generalized(&mut line, &format!("{MERGED} with {id}")),
            None => add_generalized(&mut line, MERGED),
        }
        // The pieces of the other line outside the stretch, their cut ends on the kept line.
        let merged = Polyline::new(points);
        let total = other_line.length();
        let onto = |c: Coord<f64>| {
            let q = merged.point(merged.locate(c));
            (distance(c, q) <= within + TOLERANCE_MM).then_some(q)
        };
        let pieces = outside(&other_line, &on_other, min(other))
            .into_iter()
            .map(|(from, to)| {
                let mut points = other_line.between(from, to);
                if from > 0.0
                    && let Some(q) = onto(points[0])
                {
                    points.insert(0, q);
                }
                if to < total
                    && let Some(q) = onto(points[points.len() - 1])
                {
                    points.push(q);
                }
                with_points(&objects[other], &points)
            })
            .collect();
        Some((kept, line, other, pieces))
    }
}

/// The kept line `kept` with its stretches `ranges` (on it, sorted) replaced by the middle of it
/// and `other`; before and after each stretch it leaves its own course over a ramp of
/// [`RAMP_FACTOR`] times the distance to the middle, where there is room for it.
fn merged_line(kept: &Polyline, other: &Polyline, ranges: &[(f64, f64)]) -> Vec<Coord<f64>> {
    let total = kept.length();
    let mut points: Vec<Coord<f64>> = Vec::new();
    // Where the kept line's own course resumes, and where the last stretch ended.
    let (mut cursor, mut last_end) = (0.0, 0.0);
    for (i, &range) in ranges.iter().enumerate() {
        let mid: Vec<Coord<f64>> = middle(kept, other, range, MIDDLE_STEP_MM)
            .into_iter()
            .map(|(x, y)| Coord { x, y })
            .collect();
        let (Some(&first), Some(&last)) = (mid.first(), mid.last()) else {
            continue;
        };
        let lead = RAMP_FACTOR * distance(kept.point(range.0), first);
        let from = range.0 - lead;
        if from > cursor {
            points.extend(kept.between(cursor, from));
        } else if i == 0 && range.0 > TOLERANCE_MM {
            points.push(kept.point(0.0));
        }
        points.extend(mid);
        cursor = range.1 + RAMP_FACTOR * distance(kept.point(range.1), last);
        last_end = range.1;
    }
    if cursor < total {
        points.extend(kept.between(cursor, total));
    } else if last_end < total - TOLERANCE_MM {
        points.push(kept.point(total));
    }
    points
}

/// The stretches of `line` outside `ranges` (sorted, apart), each at least `min` long.
fn outside(line: &Polyline, ranges: &[(f64, f64)], min: f64) -> Vec<(f64, f64)> {
    let total = line.length();
    let mut pieces = Vec::new();
    let mut cursor = 0.0;
    for &(from, to) in ranges.iter().chain([&(total, total)]) {
        if from - cursor >= min - TOLERANCE_MM && from > cursor {
            pieces.push((cursor, from));
        }
        cursor = cursor.max(to);
    }
    pieces
}

/// The union of the ranges `ranges`: sorted, overlapping ones as one.
fn union(mut ranges: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    ranges.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for r in ranges {
        match merged.last_mut() {
            Some(last) if r.0 <= last.1 => last.1 = last.1.max(r.1),
            _ => merged.push(r),
        }
    }
    merged
}

/// The ranges `ranges` without `holes` (both sorted and apart).
fn without(ranges: &[(f64, f64)], holes: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut left = Vec::new();
    for &(from, to) in ranges {
        let mut start = from;
        for &(a, b) in holes {
            if b <= start || a >= to {
                continue;
            }
            if a > start {
                left.push((start, a));
            }
            start = start.max(b);
        }
        if to > start {
            left.push((start, to));
        }
    }
    left
}

/// `object` (symbol, tags) along `points` (mm on paper), consecutive equal coordinates once.
fn with_points(object: &MapObject, points: &[Coord<f64>]) -> MapObject {
    let mut coords: Vec<MapCoord> = points
        .iter()
        .map(|c| MapCoord::new((c.x * 1000.0).round() as i32, (c.y * 1000.0).round() as i32))
        .collect();
    coords.dedup_by(|a, b| (a.x, a.y) == (b.x, b.y));
    MapObject {
        symbol_id: object.symbol_id,
        kind: MapObjectKind::Path,
        coords,
        tags: object.tags.clone(),
    }
}

/// `objects` with each object of `replaced` replaced by its objects there, in order.
fn rebuild(objects: &mut Vec<MapObject>, mut replaced: BTreeMap<usize, Vec<MapObject>>) {
    if replaced.is_empty() {
        return;
    }
    let before = std::mem::take(objects);
    for (i, object) in before.into_iter().enumerate() {
        match replaced.remove(&i) {
            Some(pieces) => objects.extend(pieces),
            None => objects.push(object),
        }
    }
}

fn distance(a: Coord<f64>, b: Coord<f64>) -> f64 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::conflicts::Conflict;
    use crate::build::line_join::TOUCH_M;
    use crate::build::training::tests::{
        SCALE, area, georef_at, isom, length_mm, line, of_code, point, tagged,
    };
    use crate::build::training::{TrainingReport, train};
    use crate::measure::PaperMm;

    /// The sprint scale of the tests.
    const SPRINT: u32 = 4000;

    fn forest(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, TrainingReport) {
        train(objects, None, MapKind::Forest, &georef_at(SCALE), set)
    }

    fn sprint(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, TrainingReport) {
        train(objects, None, MapKind::Sprint, &georef_at(SPRINT), set)
    }

    /// The ISSprOM set of the sprint tests.
    fn issprom() -> SymbolSet {
        SymbolSet::issprom_2019()
    }

    /// A line `code` through `points` with an OSM source and the id `id`.
    fn sourced(set: &SymbolSet, code: &str, id: &str, points: &[(f64, f64)]) -> MapObject {
        tagged(
            tagged(line(set, code, points), "freemapper:source", "osm"),
            "freemapper:source_id",
            id,
        )
    }

    fn mm(c: &MapCoord) -> Coord<f64> {
        Coord {
            x: f64::from(c.x) / 1000.0,
            y: f64::from(c.y) / 1000.0,
        }
    }

    /// The distance from `p` to the line `object` (mm on paper).
    fn off(object: &MapObject, p: (f64, f64)) -> f64 {
        let line = Polyline::new(object.coords.iter().map(mm).collect());
        let p = Coord { x: p.0, y: p.1 };
        distance(p, line.point(line.locate(p)))
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Whether the end of `object` (the last when `last`) lies on another line of `objects`.
    fn end_touches(objects: &[MapObject], i: usize, last: bool, scale: u32) -> bool {
        let object = &objects[i];
        let c = if last {
            object.coords[object.coords.len() - 1]
        } else {
            object.coords[0]
        };
        let at = mm(&c);
        let touch = PaperMm::of_metres(TOUCH_M, scale).0 + TOLERANCE_MM;
        (0..objects.len())
            .filter(|&j| j != i && objects[j].coords.len() >= 2)
            .any(|j| off(&objects[j], (at.x, at.y)) <= touch)
    }

    fn apart_left(
        set: &SymbolSet,
        objects: &[MapObject],
        kind: MapKind,
        scale: u32,
    ) -> Vec<Conflict> {
        let refs: Vec<&MapObject> = objects.iter().collect();
        conflicts(&Footprints::of(&refs, set), kind, scale)
            .into_iter()
            .filter(|c| relation(c.classes.0, c.classes.1) == Relation::Apart)
            .collect()
    }

    #[test]
    fn two_carriageways_become_one_road() {
        // A road of the forest map and a road of the sprint map, both drawn as ways of 501.8.
        for (set, code, apart, kind, scale) in [
            (isom(), "503", 0.5, MapKind::Forest, SCALE),
            (issprom(), "501.8", 0.6, MapKind::Sprint, SPRINT),
        ] {
            let objects = vec![
                sourced(&set, code, "way/1", &[(0.0, 0.0), (30.0, 0.0)]),
                sourced(&set, code, "way/2", &[(0.0, apart), (30.0, apart)]),
            ];
            let (out, report) = train(objects, None, kind, &georef_at(scale), &set);
            assert_eq!(out.len(), 1, "{}: {out:?}", set.name);
            assert_eq!(report.merged, 1, "{}: {report:?}", set.name);
            let road = &out[0];
            assert_eq!(road.symbol_id, set.id_for_code(code).unwrap());
            for c in &road.coords {
                assert!((mm(c).y - apart / 2.0).abs() < 0.002, "{}: {c:?}", set.name);
            }
            assert!((length_mm(road) - 30.0).abs() < 0.01, "{}", set.name);
            assert_eq!(tag(road, "freemapper:source_id"), Some("way/1"));
            assert_eq!(tag(road, "freemapper:also"), Some("osm/way/2"));
            assert_eq!(
                tag(road, "freemapper:generalized"),
                Some("merged with osm/way/2")
            );
            assert_eq!((report.gap_left, report.overlap_left), (0.0, 0.0));
        }
    }

    #[test]
    fn the_wider_symbol_is_kept_with_the_tags_of_the_longer_line() {
        let set = isom();
        let objects = vec![
            sourced(&set, "505", "way/1", &[(0.0, 0.3), (40.0, 0.3)]),
            sourced(&set, "503", "way/2", &[(5.0, 0.0), (30.0, 0.0)]),
        ];
        // A road and a path are not alike: they do not merge, the road stays as it is (the
        // path goes in its gap zone).
        let road = objects[1].clone();
        let (out, report) = forest(&set, objects);
        assert_eq!(report.merged, 0, "{report:?}");
        assert_eq!(of_code(&set, &out, "503"), [&road]);
        // Two roads of different widths: the wider symbol, the tags of the longer line.
        let objects = vec![
            sourced(&set, "503", "way/1", &[(0.0, 0.5), (40.0, 0.5)]),
            sourced(&set, "502", "way/2", &[(5.0, 0.0), (30.0, 0.0)]),
        ];
        let narrow = Footprints::of(&[&objects[1]], &set).half_width(0);
        let wide = Footprints::of(&[&objects[0]], &set).half_width(0);
        assert!(wide > narrow, "{wide} {narrow}");
        let (out, report) = forest(&set, objects);
        assert_eq!(report.merged, 1, "{report:?}");
        let roads = of_code(&set, &out, "503");
        assert_eq!(roads.len(), 1, "{out:?}");
        assert_eq!(tag(roads[0], "freemapper:source_id"), Some("way/1"));
        assert_eq!(tag(roads[0], "freemapper:also"), Some("osm/way/2"));
        assert!(off(roads[0], (15.0, 0.25)) < 0.01, "{:?}", roads[0]);
        assert!(off(roads[0], (35.0, 0.5)) < 0.01, "{:?}", roads[0]);
        assert!(of_code(&set, &out, "502").is_empty(), "{out:?}");
    }

    #[test]
    fn two_tracks_become_one_line() {
        for (set, code, apart, kind, scale) in [
            (isom(), "509", 0.5, MapKind::Forest, SCALE),
            (issprom(), "509.1", 0.4, MapKind::Sprint, SPRINT),
        ] {
            let objects = vec![
                line(&set, code, &[(0.0, 20.0), (30.0, 20.0)]),
                line(&set, code, &[(0.0, 20.0 + apart), (30.0, 20.0 + apart)]),
            ];
            let (out, report) = train(objects, None, kind, &georef_at(scale), &set);
            assert_eq!(out.len(), 1, "{}: {out:?}", set.name);
            assert_eq!(report.merged, 1, "{}: {report:?}", set.name);
            assert!(off(&out[0], (15.0, 20.0 + apart / 2.0)) < 0.002);
        }
    }

    #[test]
    fn two_paths_too_short_side_by_side_stay() {
        // 505 has a minimum length of 6.375 mm in the set at 1:10 000: 5 mm side by side is less.
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "505", &[(15.0, 0.3), (20.0, 0.3), (30.0, 10.0)]),
        ];
        let (out, report) = forest(&set, objects.clone());
        assert_eq!(report.merged, 0, "{report:?}");
        assert_eq!(out, objects);
    }

    #[test]
    fn a_sidewalk_beside_a_road_goes() {
        let set = isom();
        let road = line(&set, "503", &[(0.0, 0.0), (40.0, 0.0)]);
        let objects = vec![road.clone(), line(&set, "505", &[(5.0, 0.2), (35.0, 0.2)])];
        let (out, report) = forest(&set, objects);
        assert_eq!(out, vec![road]);
        assert_eq!(report.dropped_beside_roads, 1, "{report:?}");
    }

    #[test]
    fn a_sidewalk_five_metres_from_the_road_goes() {
        // 5 m at 1:10 000 is 0.5 mm: beyond the drawn half width of 503, inside its gap zone.
        let set = isom();
        let road = line(&set, "503", &[(0.0, 0.0), (40.0, 0.0)]);
        assert!(Footprints::of(&[&road], &set).half_width(0) < 0.5);
        let objects = vec![road.clone(), line(&set, "505", &[(5.0, 0.5), (35.0, 0.5)])];
        let (out, report) = forest(&set, objects);
        assert_eq!(out, vec![road]);
        assert_eq!(report.dropped_beside_roads, 1, "{report:?}");
        assert_eq!((report.gap_left, report.overlap_left), (0.0, 0.0));
    }

    #[test]
    fn a_path_that_leaves_the_road_keeps_its_free_part() {
        let set = isom();
        let objects = vec![
            line(&set, "503", &[(0.0, 0.0), (50.0, 0.0)]),
            line(&set, "505", &[(30.0, 0.5), (40.0, 0.5), (40.0, 20.0)]),
        ];
        let (out, report) = forest(&set, objects);
        assert_eq!(report.dropped_beside_roads, 0, "{report:?}");
        assert_eq!(report.cut, 1, "{report:?}");
        let paths = of_code(&set, &out, "505");
        assert_eq!(paths.len(), 1, "{out:?}");
        assert!(off(paths[0], (35.0, 0.5)) > 1.0, "{:?}", paths[0]);
        assert!(off(paths[0], (40.0, 10.0)) < 0.002, "{:?}", paths[0]);
        let index = out.iter().position(|o| o == paths[0]).unwrap();
        assert!(end_touches(&out, index, false, SCALE), "{:?}", paths[0]);
        assert_eq!((report.gap_left, report.overlap_left), (0.0, 0.0));
    }

    #[test]
    fn a_path_partly_inside_a_road_is_cut_there() {
        let set = isom();
        let objects = vec![
            line(&set, "503", &[(0.0, 0.0), (50.0, 0.0)]),
            line(&set, "505", &[(30.0, 0.2), (40.0, 0.2), (40.0, 20.0)]),
        ];
        let (out, report) = forest(&set, objects);
        assert_eq!(report.dropped_beside_roads, 0, "{report:?}");
        assert_eq!(report.cut, 1, "{report:?}");
        let paths = of_code(&set, &out, "505");
        assert_eq!(paths.len(), 1, "{out:?}");
        assert!(off(paths[0], (35.0, 0.2)) > 1.0, "{:?}", paths[0]);
        assert!(off(paths[0], (40.0, 10.0)) < 0.002, "{:?}", paths[0]);
        let index = out.iter().position(|o| o == paths[0]).unwrap();
        assert!(end_touches(&out, index, false, SCALE), "{:?}", paths[0]);
        assert!(
            tag(paths[0], "freemapper:generalized").is_some_and(|v| v.contains(CUT_INSIDE_ROAD))
        );
    }

    #[test]
    fn a_tram_in_a_road_goes_on_the_forest_map_only() {
        let set = isom();
        let road = line(&set, "503", &[(0.0, 0.0), (40.0, 0.0)]);
        let tram = tagged(
            line(&set, "509", &[(0.0, 0.1), (40.0, 0.1)]),
            "freemapper:osm",
            "railway=tram",
        );
        let (out, report) = forest(&set, vec![road.clone(), tram.clone()]);
        assert_eq!(out, vec![road.clone()]);
        assert_eq!(report.dropped_beside_roads, 1, "{report:?}");
        // A railway that is no tram stays.
        let train_line = line(&set, "509", &[(0.0, 0.1), (40.0, 0.1)]);
        let (out, _) = forest(&set, vec![road, train_line.clone()]);
        assert!(out.contains(&train_line), "{out:?}");
        // On the sprint map the tram runs on the street area, which is free.
        let set = issprom();
        let street = area(&set, "501.5", (0.0, -2.0), (40.0, 2.0));
        let tram = tagged(
            line(&set, "509.1", &[(0.0, 0.0), (40.0, 0.0)]),
            "freemapper:osm",
            "railway=tram",
        );
        let objects = vec![street, tram];
        let (out, report) = sprint(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
    }

    #[test]
    fn a_side_road_stays_connected_to_the_merged_road() {
        let set = isom();
        let objects = vec![
            line(&set, "503", &[(0.0, 0.0), (40.0, 0.0)]),
            line(&set, "503", &[(0.0, 0.5), (40.0, 0.5)]),
            // On the second carriageway, which goes, and on the first, which is kept.
            line(&set, "505", &[(20.0, 10.0), (20.0, 0.5)]),
            line(&set, "505", &[(10.0, -10.0), (10.0, 0.0)]),
        ];
        let (out, report) = forest(&set, objects);
        assert_eq!(report.merged, 1, "{report:?}");
        assert_eq!(of_code(&set, &out, "503").len(), 1, "{out:?}");
        for (i, object) in out.iter().enumerate() {
            if object.symbol_id == set.id_for_code("505").unwrap() {
                assert!(end_touches(&out, i, true, SCALE), "{object:?}");
            }
        }
        assert_eq!(
            (report.junctions_repaired, report.junctions_free),
            (2, 0),
            "{report:?}"
        );
    }

    #[test]
    fn a_pair_that_parts_merges_only_where_it_runs_together() {
        let set = isom();
        let objects = vec![
            line(&set, "503", &[(0.0, 0.0), (40.0, 0.0)]),
            line(&set, "503", &[(0.0, 0.5), (20.0, 0.5), (40.0, 15.0)]),
        ];
        let (out, report) = forest(&set, objects);
        assert_eq!(report.merged, 1, "{report:?}");
        let roads = of_code(&set, &out, "503");
        assert_eq!(roads.len(), 2, "{out:?}");
        let merged: Vec<&&MapObject> = roads
            .iter()
            .filter(|r| tag(r, "freemapper:generalized").is_some())
            .collect();
        assert_eq!(merged.len(), 1, "{roads:?}");
        // Where they run together, one line on the middle and none on the old lines.
        assert!(off(merged[0], (10.0, 0.25)) < 0.002, "{:?}", merged[0]);
        for road in &roads {
            assert!(off(road, (10.0, 0.0)) > 0.2, "{road:?}");
            assert!(off(road, (10.0, 0.5)) > 0.2, "{road:?}");
        }
        // Where they part, both branches.
        assert!(
            roads.iter().any(|r| off(r, (35.0, 0.0)) < 0.002),
            "{roads:?}"
        );
        assert!(
            roads.iter().any(|r| off(r, (35.0, 11.375)) < 0.002),
            "{roads:?}"
        );
        // The branch that is left starts on the merged line.
        let branch = roads
            .iter()
            .find(|r| tag(r, "freemapper:generalized").is_none());
        let index = out.iter().position(|o| Some(&o) == branch).unwrap();
        assert!(end_touches(&out, index, false, SCALE), "{out:?}");
    }

    #[test]
    fn paths_beside_a_street_area_are_left_on_the_sprint_map() {
        let set = issprom();
        let objects = vec![
            area(&set, "501.5", (0.0, 0.0), (40.0, 6.0)),
            line(&set, "501.6", &[(0.0, 6.2), (40.0, 6.2)]),
            line(&set, "501.6", &[(0.0, 3.0), (40.0, 3.0)]),
        ];
        let (out, report) = sprint(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
    }

    #[test]
    fn nothing_new_conflicts_with_a_moved_point() {
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (30.0, 0.0)]),
            line(&set, "505", &[(0.0, 0.3), (30.0, 0.3)]),
            point(&set, "526", (15.0, 1.0)),
        ];
        let (out, report) = forest(&set, objects);
        assert_eq!(report.merged, 1, "{report:?}");
        assert_eq!(report.gave_way, 0, "{report:?}");
        assert_eq!(of_code(&set, &out, "526").len(), 1, "{out:?}");
        assert_eq!(
            apart_left(&set, &out, MapKind::Forest, SCALE),
            Vec::new(),
            "{out:?}"
        );
    }

    #[test]
    fn junctions_survive_a_merge_and_a_dropped_sidewalk() {
        // Record, merge, repair and forget in one run: a pair that parts (the second
        // carriageway is cut into the branch it keeps), a sidewalk inside the first that goes,
        // and the ways that ended on them.
        let set = isom();
        let objects = vec![
            line(&set, "503", &[(0.0, 0.0), (60.0, 0.0)]),
            line(&set, "503", &[(0.0, 0.5), (30.0, 0.5), (60.0, 20.0)]),
            line(&set, "505", &[(10.0, -0.2), (25.0, -0.2)]),
            // Ends on the sidewalk, on the second carriageway and on its branch.
            line(&set, "506", &[(15.0, -10.0), (15.0, -0.2)]),
            line(&set, "505", &[(20.0, 10.0), (20.0, 0.5)]),
            line(&set, "505", &[(45.0, 25.0), (45.0, 10.25)]),
        ];
        let (out, report) = forest(&set, objects);
        assert_eq!(
            (report.merged, report.dropped_beside_roads),
            (1, 1),
            "{report:?}"
        );
        assert_eq!(report.junctions_free, 0, "{report:?}");
        assert_eq!(report.junctions_repaired, 2, "{report:?}");
        assert_eq!(of_code(&set, &out, "503").len(), 2, "{out:?}");
        for code in ["505", "506"] {
            for (i, object) in out.iter().enumerate() {
                if object.symbol_id == set.id_for_code(code).unwrap() {
                    assert!(end_touches(&out, i, true, SCALE), "{object:?}");
                }
            }
        }
        assert!(
            out.iter()
                .all(|o| o.tags.iter().all(|(k, _)| k != JUNCTION_TAG)),
            "{out:?}"
        );
    }
}
