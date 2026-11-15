//! Junctions on the training map (spec 2026-11-14, "The second half, as designed on
//! 2026-11-15", "Junctions survive"): the line ends that touch another line are recorded before
//! lines are merged or shifted, and afterwards an end that no longer touches is moved onto the
//! line it touched or onto the line that replaced it. freemapper-courseplan tells free ends from
//! junctions by whether lines touch. Paper mm, y down; only +, −, ×, ÷ and `sqrt` decide, and
//! every collection is iterated in a sorted order, so that every system decides alike.

#![allow(dead_code)] // The merge and shift steps of the training level call it.

use std::collections::{BTreeMap, BTreeSet};

use geo_types::Coord;

use super::is_closed;
use super::lines::Polyline;
use crate::build::conflicts::{Class, Classes, TOLERANCE_MM};
use crate::build::line_join::TOUCH_M;
use crate::georef::Georeferencing;
use crate::measure::PaperMm;
use crate::omap::{MapCoord, MapObject, MapObjectKind};
use crate::render::geometry::parts;

/// The classes whose line ends are recorded and whose lines an end can touch.
const LINE_CLASSES: [Class; 7] = [
    Class::Road,
    Class::Path,
    Class::Railway,
    Class::Stairway,
    Class::Barrier,
    Class::Watercourse,
    Class::EarthBank,
];

/// The side of a cell of the search grid (mm on paper).
const CELL_MM: f64 = 5.0;

/// The key of an object that survives a change of the others: its `freemapper:source_id`, its
/// symbol and its index among the objects with the same source id and symbol, in input order.
/// A step that replaces or splits objects keeps the source id and the symbol on the pieces and
/// reports the replacements by key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key {
    pub(super) source_id: Option<String>,
    pub(super) symbol_id: u32,
    pub(super) nth: usize,
}

/// One recorded end: of which object, whether it is the last coordinate, and the lines it
/// touched (sorted).
#[derive(Debug, Clone)]
struct End {
    object: Key,
    last: bool,
    touched: Vec<Key>,
}

/// Line ends that touch another line, recorded before a change.
#[derive(Debug, Clone)]
pub(super) struct Junctions {
    ends: Vec<End>,
    /// The symbols of [`LINE_CLASSES`].
    line_ids: BTreeSet<u32>,
}

impl Junctions {
    /// Every end of a line object (classes Road, Path, Railway, Stairway, Barrier, Watercourse,
    /// EarthBank) that lies within [`TOUCH_M`] (ground metres, as `line_join`) of another line
    /// object, keyed by a stable object key ([`Key`]). Ends of closed lines are not recorded.
    pub(super) fn record(
        objects: &[MapObject],
        classes: &Classes,
        georef: &Georeferencing,
    ) -> Self {
        let line_ids: BTreeSet<u32> = LINE_CLASSES.iter().flat_map(|&c| classes.ids(c)).collect();
        let touch = touch_mm(georef);
        let grid = Grid::of(objects, &line_ids);
        let keys = keys(objects);
        let mut ends = Vec::new();
        for (i, object) in objects.iter().enumerate() {
            if !is_open_line(object, &line_ids) {
                continue;
            }
            for last in [false, true] {
                let at = end_point(object, last);
                let touched: BTreeSet<Key> = grid
                    .near(at, touch)
                    .into_iter()
                    .filter(|&j| j != i && distance(&objects[j], at).0 <= touch)
                    .map(|j| keys[j].clone())
                    .collect();
                if !touched.is_empty() {
                    ends.push(End {
                        object: keys[i].clone(),
                        last,
                        touched: touched.into_iter().collect(),
                    });
                }
            }
        }
        Self { ends, line_ids }
    }

    /// After a change: every recorded end whose object still exists but no longer touches a line
    /// is moved to the nearest point of the line that touched it or of the line that replaced it
    /// (`replaced_by`), when that point is within `reach` mm; returns (repaired, still free).
    ///
    /// Only the end vertex moves (its flags kept), so a sided line keeps its direction, and the
    /// line it moves onto stays as it is. A touch counts only on the body of a line: where an end
    /// meets just another recorded end, both may have lost the line they met on, so the end looks
    /// for its line first and stays where it is only when there is none in reach. Of the lines in
    /// reach, a body comes before a recorded end, then the line recorded before the line that
    /// replaced it (`replaced_by` is followed), then the lower key.
    pub(super) fn repair(
        &self,
        objects: &mut [MapObject],
        replaced_by: &BTreeMap<Key, Key>,
        reach: f64,
        georef: &Georeferencing,
    ) -> (usize, usize) {
        let touch = touch_mm(georef);
        let keys = keys(objects);
        let index: BTreeMap<&Key, usize> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        // The recorded ends of the objects that still exist: object index → [first, last].
        let mut recorded: BTreeMap<usize, [bool; 2]> = BTreeMap::new();
        for end in &self.ends {
            if let Some(&i) = index.get(&end.object) {
                recorded.entry(i).or_default()[usize::from(end.last)] = true;
            }
        }
        let mut grid = Grid::of(objects, &self.line_ids);
        let (mut repaired, mut free) = (0, 0);
        for end in &self.ends {
            let Some(&i) = index.get(&end.object) else {
                continue;
            };
            if !is_open_line(&objects[i], &self.line_ids) {
                continue;
            }
            let at = end_point(&objects[i], end.last);
            // Every line the end meets: on its body (`false`) or only at a recorded end (`true`).
            let meets: Vec<bool> = grid
                .near(at, touch)
                .into_iter()
                .filter(|&j| j != i && is_line(&objects[j], &self.line_ids))
                .filter_map(|j| {
                    let (d, q) = distance(&objects[j], at);
                    (d <= touch).then(|| at_recorded_end(&objects[j], recorded.get(&j), q, touch))
                })
                .collect();
            if meets.contains(&false) {
                continue;
            }
            let mut ranks: BTreeMap<&Key, usize> = BTreeMap::new();
            for touched in &end.touched {
                let mut seen = BTreeSet::new();
                let mut key = touched;
                let mut rank = 0;
                while seen.insert(key) {
                    let r = ranks.entry(key).or_insert(rank);
                    *r = (*r).min(rank);
                    let Some(next) = replaced_by.get(key) else {
                        break;
                    };
                    key = next;
                    rank += 1;
                }
            }
            let best = ranks
                .into_iter()
                .filter_map(|(key, rank)| {
                    let &j = index.get(key)?;
                    if j == i || !is_line(&objects[j], &self.line_ids) {
                        return None;
                    }
                    let (d, q) = distance(&objects[j], at);
                    let via_end = at_recorded_end(&objects[j], recorded.get(&j), q, touch);
                    (d <= reach + TOLERANCE_MM).then_some((via_end, rank, key, d, q))
                })
                .min_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)));
            match best {
                Some((_, _, _, d, q)) if d > touch => {
                    let vertex = if end.last {
                        objects[i].coords.len() - 1
                    } else {
                        0
                    };
                    let c = &mut objects[i].coords[vertex];
                    c.x = (q.x * 1000.0).round() as i32;
                    c.y = (q.y * 1000.0).round() as i32;
                    grid.add(i, &objects[i]);
                    repaired += 1;
                }
                // It meets the line already, or meets another end and has no line in reach.
                Some(_) => {}
                None if !meets.is_empty() => {}
                None => free += 1,
            }
        }
        (repaired, free)
    }
}

/// Ends closer than this touch a line (mm on paper): [`TOUCH_M`] at the scale of the map, with
/// the tolerance of the comparisons.
fn touch_mm(georef: &Georeferencing) -> f64 {
    PaperMm::of_metres(TOUCH_M, georef.scale_denominator).0 + TOLERANCE_MM
}

/// The key of every object, in order.
fn keys(objects: &[MapObject]) -> Vec<Key> {
    let mut count: BTreeMap<(Option<&str>, u32), usize> = BTreeMap::new();
    objects
        .iter()
        .map(|o| {
            let source_id = o
                .tags
                .iter()
                .find(|(k, _)| k == "freemapper:source_id")
                .map(|(_, v)| v.as_str());
            let nth = count.entry((source_id, o.symbol_id)).or_insert(0);
            let key = Key {
                source_id: source_id.map(str::to_string),
                symbol_id: o.symbol_id,
                nth: *nth,
            };
            *nth += 1;
            key
        })
        .collect()
}

/// A path of a line symbol of `line_ids` with at least two points.
fn is_line(object: &MapObject, line_ids: &BTreeSet<u32>) -> bool {
    matches!(object.kind, MapObjectKind::Path)
        && object.coords.len() >= 2
        && line_ids.contains(&object.symbol_id)
}

fn is_open_line(object: &MapObject, line_ids: &BTreeSet<u32>) -> bool {
    is_line(object, line_ids) && !is_closed(object)
}

/// The first or the last coordinate of `object`, mm on paper.
fn end_point(object: &MapObject, last: bool) -> Coord<f64> {
    let c = if last {
        object.coords[object.coords.len() - 1]
    } else {
        object.coords[0]
    };
    mm(c)
}

fn mm(c: MapCoord) -> Coord<f64> {
    Coord {
        x: f64::from(c.x) / 1000.0,
        y: f64::from(c.y) / 1000.0,
    }
}

/// The parts of the line `object` (Béziers flattened), mm on paper.
fn polylines(object: &MapObject) -> Vec<Polyline> {
    parts(&object.coords)
        .into_iter()
        .filter(|p| p.points.len() >= 2)
        .map(|p| Polyline::new(p.points.iter().map(|&(x, y)| Coord { x, y }).collect()))
        .collect()
}

/// The distance from `at` to the line `object` and the point of the line nearest it (of two
/// parts as near, the first).
fn distance(object: &MapObject, at: Coord<f64>) -> (f64, Coord<f64>) {
    let mut best = (f64::INFINITY, at);
    for line in polylines(object) {
        let q = line.point(line.locate(at));
        let d = length(at, q);
        if d < best.0 {
            best = (d, q);
        }
    }
    best
}

fn length(a: Coord<f64>, b: Coord<f64>) -> f64 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    (dx * dx + dy * dy).sqrt()
}

/// Whether `q`, a point of `object`, lies within `touch` of one of its ends that is recorded
/// (`recorded`: [first, last]).
fn at_recorded_end(
    object: &MapObject,
    recorded: Option<&[bool; 2]>,
    q: Coord<f64>,
    touch: f64,
) -> bool {
    let Some(recorded) = recorded else {
        return false;
    };
    [false, true]
        .into_iter()
        .any(|last| recorded[usize::from(last)] && length(end_point(object, last), q) <= touch)
}

/// The line objects by the cells of a square grid their bounds cover, so that the lines near a
/// point are found without measuring every line. An object moved is added again; the cells it
/// left still name it, which only costs a measure.
struct Grid {
    cells: BTreeMap<(i64, i64), BTreeSet<usize>>,
}

impl Grid {
    fn of(objects: &[MapObject], line_ids: &BTreeSet<u32>) -> Self {
        let mut grid = Self {
            cells: BTreeMap::new(),
        };
        for (i, object) in objects.iter().enumerate() {
            if is_line(object, line_ids) {
                grid.add(i, object);
            }
        }
        grid
    }

    fn add(&mut self, i: usize, object: &MapObject) {
        for line in polylines(object) {
            let (mut min, mut max) = (line.points[0], line.points[0]);
            for p in &line.points {
                (min.x, min.y) = (min.x.min(p.x), min.y.min(p.y));
                (max.x, max.y) = (max.x.max(p.x), max.y.max(p.y));
            }
            for x in cell(min.x)..=cell(max.x) {
                for y in cell(min.y)..=cell(max.y) {
                    self.cells.entry((x, y)).or_default().insert(i);
                }
            }
        }
    }

    /// The objects whose cells lie within `within` of `at`, in order.
    fn near(&self, at: Coord<f64>, within: f64) -> BTreeSet<usize> {
        let mut near = BTreeSet::new();
        for x in cell(at.x - within)..=cell(at.x + within) {
            for y in cell(at.y - within)..=cell(at.y + within) {
                if let Some(objects) = self.cells.get(&(x, y)) {
                    near.extend(objects);
                }
            }
        }
        near
    }
}

fn cell(v: f64) -> i64 {
    (v / CELL_MM).floor() as i64
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::training::tests::{SCALE, georef_at, isom, line};
    use crate::omap::SymbolSet;

    /// A line `code` through `points` (paper mm) with the source id `id`.
    fn tagged(set: &SymbolSet, code: &str, id: &str, points: &[(f64, f64)]) -> MapObject {
        let mut object = line(set, code, points);
        object
            .tags
            .push(("freemapper:source_id".to_string(), id.to_string()));
        object
    }

    fn key_of(objects: &[MapObject], i: usize) -> Key {
        keys(objects)[i].clone()
    }

    fn mm(c: &MapCoord) -> (f64, f64) {
        (f64::from(c.x) / 1000.0, f64::from(c.y) / 1000.0)
    }

    /// A path from (20, 0) whose end lies on a road along y = 10.
    fn path_on_road(set: &SymbolSet) -> Vec<MapObject> {
        vec![
            tagged(set, "505", "p", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(set, "503", "r", &[(0.0, 10.0), (40.0, 10.0)]),
        ]
    }

    #[test]
    fn an_end_follows_a_shifted_line() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        let junctions = Junctions::record(&objects, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 1, "{junctions:?}");
        objects[1] = tagged(&set, "503", "r", &[(0.0, 10.3), (40.0, 10.3)]);
        let flags = objects[0].coords[1].flags;
        let result = junctions.repair(&mut objects, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (1, 0));
        assert_eq!(mm(&objects[0].coords[1]), (20.0, 10.3));
        assert_eq!(objects[0].coords[1].flags, flags);
        assert_eq!(
            mm(&objects[0].coords[0]),
            (20.0, 0.0),
            "the other end stays"
        );
        assert_eq!(mm(&objects[1].coords[0]), (0.0, 10.3), "the line stays");
    }

    #[test]
    fn an_end_moves_to_the_replacing_middle_line() {
        let set = isom();
        let georef = georef_at(SCALE);
        let before = vec![
            tagged(&set, "505", "p", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "503", "a", &[(0.0, 10.0), (40.0, 10.0)]),
            tagged(&set, "503", "b", &[(0.0, 10.4), (40.0, 10.4)]),
        ];
        let junctions = Junctions::record(&before, &Classes::of(&set), &georef);
        // The pair becomes its middle line, which keeps the key of `b`; `a` goes.
        let mut after = vec![
            before[0].clone(),
            tagged(&set, "503", "b", &[(0.0, 10.2), (40.0, 10.2)]),
        ];
        let replaced_by = BTreeMap::from([(key_of(&before, 1), key_of(&after, 1))]);
        let result = junctions.repair(&mut after, &replaced_by, 0.5, &georef);
        assert_eq!(result, (1, 0));
        assert_eq!(mm(&after[0].coords[1]), (20.0, 10.2));
    }

    #[test]
    fn an_end_too_far_stays_free_and_is_counted() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        let junctions = Junctions::record(&objects, &Classes::of(&set), &georef);
        objects[1] = tagged(&set, "503", "r", &[(0.0, 11.0), (40.0, 11.0)]);
        let result = junctions.repair(&mut objects, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (0, 1));
        assert_eq!(mm(&objects[0].coords[1]), (20.0, 10.0));
    }

    #[test]
    fn a_closed_line_has_no_ends() {
        let set = isom();
        let georef = georef_at(SCALE);
        let ring = [(20.0, 10.0), (30.0, 0.0), (10.0, 0.0), (20.0, 10.0)];
        let mut objects = vec![
            tagged(&set, "505", "p", &ring),
            tagged(&set, "503", "r", &[(0.0, 10.0), (40.0, 10.0)]),
        ];
        let junctions = Junctions::record(&objects, &Classes::of(&set), &georef);
        assert!(junctions.ends.is_empty(), "{junctions:?}");
        objects[1] = tagged(&set, "503", "r", &[(0.0, 10.3), (40.0, 10.3)]);
        let result = junctions.repair(&mut objects, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (0, 0));
        assert_eq!(mm(&objects[0].coords[0]), (20.0, 10.0));
    }

    #[test]
    fn two_ends_that_met_on_a_shifted_line_both_follow_it() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = vec![
            tagged(&set, "505", "p", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "506", "q", &[(20.0, 10.0), (30.0, 0.0)]),
            tagged(&set, "503", "r", &[(0.0, 10.0), (40.0, 10.0)]),
        ];
        let junctions = Junctions::record(&objects, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 2, "{junctions:?}");
        objects[2] = tagged(&set, "503", "r", &[(0.0, 10.3), (40.0, 10.3)]);
        let (repaired, free) = junctions.repair(&mut objects, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(free, 0);
        assert!(repaired >= 1, "{repaired}");
        let touch = touch_mm(&georef);
        for (i, last) in [(0, true), (1, false)] {
            let at = end_point(&objects[i], last);
            let on_line = (0..objects.len())
                .filter(|&j| j != i)
                .any(|j| distance(&objects[j], at).0 <= touch);
            assert!(on_line, "{i}: {at:?}");
        }
        assert_eq!(mm(&objects[0].coords[1]), (20.0, 10.3));
    }

    #[test]
    fn ends_that_still_meet_stay_where_they_are() {
        let set = isom();
        let georef = georef_at(SCALE);
        let before = vec![
            tagged(&set, "505", "p", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "506", "q", &[(20.0, 10.0), (30.0, 20.0)]),
        ];
        let junctions = Junctions::record(&before, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 2, "{junctions:?}");
        let mut after = before.clone();
        let result = junctions.repair(&mut after, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (0, 0));
        assert_eq!(after, before);
    }
}
