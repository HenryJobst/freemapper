//! Junctions on the training map (spec 2026-11-14, "The second half, as designed on
//! 2026-11-15", "Junctions survive"): the line ends that touch another line are recorded before
//! lines are merged or shifted, and afterwards an end that no longer touches is moved onto the
//! line it touched or onto the line that replaced it. freemapper-courseplan tells free ends from
//! junctions by whether lines touch. Paper mm, y down; only +, −, ×, ÷ and `sqrt` decide, and
//! every collection is iterated in a sorted order, so that every system decides alike.
//!
//! [`Junctions::record`] names every line object by the temporary tag [`JUNCTION_TAG`], so that
//! the keys hold whatever the steps between drop, split or replace; `train` must call [`forget`]
//! before it returns, so that the tag never reaches a map file.

use std::collections::{BTreeMap, BTreeSet};

use geo_types::Coord;

use super::lines::Polyline;
use super::{FreeEnd, is_closed};
use crate::build::conflicts::{Class, Classes, TOLERANCE_MM};
use crate::build::line_join::TOUCH_M;
use crate::georef::Georeferencing;
use crate::measure::PaperMm;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
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

/// The classes of the way network: a free end of one of them that touched another is a break of
/// the network on print.
const WAY_CLASSES: [Class; 4] = [Class::Road, Class::Path, Class::Railway, Class::Stairway];

/// The side of a cell of the search grid (mm on paper).
const CELL_MM: f64 = 5.0;

/// The temporary tag that carries the [`Key`] of a line object while the training level runs.
pub(super) const JUNCTION_TAG: &str = "freemapper:junction";

/// The key of a line object: its index when the junctions were recorded, carried in the tag
/// [`JUNCTION_TAG`]. A step that splits or replaces a line keeps the tags on the pieces (the cut
/// helpers copy them) and reports replacements by key, so one key can name several objects.
pub(super) type Key = u32;

/// One recorded end: of which object, whether it is the last coordinate, where it lay (mm on
/// paper) and the lines it touched (sorted), with their source ids (sorted, those with an id)
/// and whether one of them was a way, for the report.
#[derive(Debug, Clone)]
struct End {
    object: Key,
    last: bool,
    at: Coord<f64>,
    touched: Vec<Key>,
    touched_ids: Vec<String>,
    touched_way: bool,
}

/// An end that a step drew onto a line while it ran (a cut end of a merged or cut line): of
/// which object, whether it is the last coordinate, where it lies (mm on paper) and the line it
/// was drawn onto.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Attached {
    pub(super) object: Key,
    pub(super) last: bool,
    pub(super) at: Coord<f64>,
    pub(super) onto: Key,
}

/// Line ends that touch another line, recorded before a change.
#[derive(Debug, Clone)]
pub(super) struct Junctions {
    ends: Vec<End>,
    /// The symbols of [`LINE_CLASSES`].
    line_ids: BTreeSet<u32>,
    /// The symbols of [`WAY_CLASSES`].
    way_ids: BTreeSet<u32>,
    /// The keys of lines that a join took into another line, each with the keys of the lines it
    /// went into ([`absorb`](Self::absorb)); a split line can go into several.
    absorbed: BTreeMap<Key, BTreeSet<Key>>,
}

impl Junctions {
    /// Every end of a line object (classes Road, Path, Railway, Stairway, Barrier, Watercourse,
    /// EarthBank) that lies within [`TOUCH_M`] (ground metres, as `line_join`) of another line
    /// object. Ends of closed lines are not recorded. Every line object of these classes gets
    /// the tag [`JUNCTION_TAG`] with its index as its [`Key`] (a tag left from before is
    /// replaced); `train` must call [`forget`] before it returns.
    pub(super) fn record(
        objects: &mut [MapObject],
        classes: &Classes,
        georef: &Georeferencing,
    ) -> Self {
        let line_ids: BTreeSet<u32> = LINE_CLASSES.iter().flat_map(|&c| classes.ids(c)).collect();
        let way_ids: BTreeSet<u32> = WAY_CLASSES.iter().flat_map(|&c| classes.ids(c)).collect();
        for (i, object) in objects.iter_mut().enumerate() {
            object.tags.retain(|(k, _)| k != JUNCTION_TAG);
            if is_line(object, &line_ids) && Key::try_from(i).is_ok() {
                object.tags.push((JUNCTION_TAG.to_string(), i.to_string()));
            }
        }
        let objects: &[MapObject] = objects;
        let touch = touch_mm(georef);
        let grid = Grid::of(objects, &line_ids);
        let mut ends = Vec::new();
        for (i, object) in objects.iter().enumerate() {
            let Some(key) = key_of(object) else {
                continue;
            };
            if !is_open_line(object, &line_ids) {
                continue;
            }
            for last in [false, true] {
                let at = end_point(object, last);
                let near: Vec<usize> = grid
                    .near(at, touch)
                    .into_iter()
                    .filter(|&j| j != i && distance(&objects[j], at).0 <= touch)
                    .filter(|&j| key_of(&objects[j]).is_some())
                    .collect();
                let touched: BTreeSet<Key> =
                    near.iter().filter_map(|&j| key_of(&objects[j])).collect();
                if !touched.is_empty() {
                    let ids: BTreeSet<String> = near
                        .iter()
                        .filter_map(|&j| source_id(&objects[j]))
                        .collect();
                    ends.push(End {
                        object: key,
                        last,
                        at,
                        touched: touched.into_iter().collect(),
                        touched_ids: ids.into_iter().collect(),
                        touched_way: near
                            .iter()
                            .any(|&j| way_ids.contains(&objects[j].symbol_id)),
                    });
                }
            }
        }
        Self {
            ends,
            line_ids,
            way_ids,
            absorbed: BTreeMap::new(),
        }
    }

    /// Records the joins of a step after [`record`](Self::record): each pair names, by the
    /// value of the key tag, a line that went into another and the line it went into. The
    /// repairs find the ends of a line that went, and the lines an end touched, on the line it
    /// went into.
    pub(super) fn absorb(&mut self, pairs: &[(String, String)]) {
        for (from, into) in pairs {
            if let (Ok(from), Ok(into)) = (from.parse::<Key>(), into.parse::<Key>())
                && from != into
            {
                self.absorbed.entry(from).or_default().insert(into);
            }
        }
    }

    /// `key` and the keys of every line it went into by a join, through every later join.
    fn family(&self, key: Key) -> BTreeSet<Key> {
        let mut family = BTreeSet::from([key]);
        let mut open = vec![key];
        while let Some(k) = open.pop() {
            for &into in self.absorbed.get(&k).into_iter().flatten() {
                if family.insert(into) {
                    open.push(into);
                }
            }
        }
        family
    }

    /// Records the ends a step drew onto a line after [`record`](Self::record), as if they had
    /// touched that line before: [`repair`](Self::repair) moves them onto it, or onto what
    /// replaced it, when it moved, and counts them. `objects` are the objects after the step:
    /// the report names the line an end was drawn onto by its source id.
    pub(super) fn attach(&mut self, ends: &[Attached], objects: &[MapObject]) {
        for e in ends {
            let onto: Vec<&MapObject> = objects
                .iter()
                .filter(|o| key_of(o) == Some(e.onto))
                .collect();
            let ids: BTreeSet<String> = onto.iter().filter_map(|o| source_id(o)).collect();
            self.ends.push(End {
                object: e.object,
                last: e.last,
                at: e.at,
                touched: vec![e.onto],
                touched_ids: ids.into_iter().collect(),
                touched_way: onto.iter().any(|o| self.way_ids.contains(&o.symbol_id)),
            });
        }
    }

    /// After a change: every recorded end whose object still exists but no longer touches a line
    /// is moved to the nearest point of the line that touched it or of the line that replaced it
    /// (`replaced_by`), when that point is within `reach` mm; returns (repaired, still free).
    ///
    /// A key can name several objects, the pieces of a split line: the end is that of the piece
    /// whose end lies nearest the recorded position, within `reach` (else the end is gone with
    /// its stretch and not counted), and of the pieces of a line it can move onto, the nearest.
    /// Only the end vertex moves (its flags kept), so a sided line keeps its direction, and the
    /// line it moves onto stays as it is. Another piece of the end's own line is no line it
    /// touches (a cut right at the end leaves one). A touch counts only on the body of a line: where an end
    /// meets just another recorded end, both may have lost the line they met on, so the end looks
    /// for its line first and stays where it is only when there is none in reach. Of the lines in
    /// reach, a body comes before a recorded end, then the line recorded before the line that
    /// replaced it (`replaced_by` is followed), then the lower key, then the nearer piece.
    pub(super) fn repair(
        &self,
        objects: &mut [MapObject],
        replaced_by: &BTreeMap<Key, Key>,
        reach: f64,
        georef: &Georeferencing,
    ) -> (usize, usize) {
        let (repaired, free) = self.settle(objects, replaced_by, &BTreeMap::new(), reach, georef);
        (repaired, free.len())
    }

    /// As [`repair`](Self::repair), after lines were cut where they kept no gap: `cut_for`
    /// names, by key, the lines each line was cut or dropped for. An end is not drawn onto a line
    /// that was cut for the end's own line: that gap was opened on purpose (two lines that lay
    /// on each other). Such an end stays where it is, and it is not counted free when every
    /// line it could move onto was cut for its own line.
    ///
    /// A line that a join took into another ([`absorb`](Self::absorb)) is found on the line it
    /// went into, both as the end's own line and as a line the end touched. One end recorded
    /// more than once is counted free once.
    ///
    /// Returns how many ends were repaired and the ends left free, in the order of the objects,
    /// each named by its line's source id and symbol code (of `symbol_set`).
    pub(super) fn repair_after_cuts(
        &self,
        objects: &mut [MapObject],
        replaced_by: &BTreeMap<Key, Key>,
        cut_for: &BTreeMap<Key, BTreeSet<Key>>,
        reach: f64,
        georef: &Georeferencing,
        symbol_set: &SymbolSet,
    ) -> (usize, Vec<FreeEnd>) {
        let (repaired, free) = self.settle(objects, replaced_by, cut_for, reach, georef);
        let free = free
            .into_iter()
            .map(|((i, last), loose)| {
                let object = &objects[i];
                let at = end_point(object, last);
                FreeEnd {
                    source_id: source_id(object),
                    symbol: symbol_set
                        .code_for_id(object.symbol_id)
                        .unwrap_or_default()
                        .to_string(),
                    at: (PaperMm(at.x), PaperMm(at.y)),
                    touched: loose.touched.into_iter().collect(),
                    between_ways: loose.touched_way && self.way_ids.contains(&object.symbol_id),
                }
            })
            .collect();
        (repaired, free)
    }

    /// The repairs of [`repair_after_cuts`](Self::repair_after_cuts): how many ends were
    /// repaired, and the ends left free by object index and end.
    fn settle(
        &self,
        objects: &mut [MapObject],
        replaced_by: &BTreeMap<Key, Key>,
        cut_for: &BTreeMap<Key, BTreeSet<Key>>,
        reach: f64,
        georef: &Georeferencing,
    ) -> (usize, BTreeMap<(usize, bool), Loose>) {
        let touch = touch_mm(georef);
        let within = reach + TOLERANCE_MM;
        // The line objects of each key, in order.
        let mut pieces: BTreeMap<Key, Vec<usize>> = BTreeMap::new();
        for (i, object) in objects.iter().enumerate() {
            if is_line(object, &self.line_ids)
                && let Some(key) = key_of(object)
            {
                pieces.entry(key).or_default().push(i);
            }
        }
        // Where each recorded end is now: the end of a piece of its object, or of a line its
        // object went into by a join, nearest the recorded position (of two as near, the
        // earlier object, then the same end as recorded).
        let held: Vec<Option<(usize, bool)>> = self
            .ends
            .iter()
            .map(|end| {
                self.family(end.object)
                    .iter()
                    .filter_map(|key| pieces.get(key))
                    .flatten()
                    .filter(|&&i| is_open_line(&objects[i], &self.line_ids))
                    .flat_map(|&i| [(i, false), (i, true)])
                    .map(|(i, last)| (length(end_point(&objects[i], last), end.at), i, last))
                    .filter(|&(d, _, _)| d <= within)
                    .min_by(|a, b| {
                        a.0.total_cmp(&b.0)
                            .then(a.1.cmp(&b.1))
                            .then((a.2 != end.last).cmp(&(b.2 != end.last)))
                    })
                    .map(|(_, i, last)| (i, last))
            })
            .collect();
        // The recorded ends of the objects now: object index → [first, last].
        let mut recorded: BTreeMap<usize, [bool; 2]> = BTreeMap::new();
        for &(i, last) in held.iter().flatten() {
            recorded.entry(i).or_default()[usize::from(last)] = true;
        }
        let mut grid = Grid::of(objects, &self.line_ids);
        let mut repaired = 0;
        // The ends left free, by object index and end: one end recorded twice (by `record` and
        // by `attach`, or for two pieces of one key) is counted once, and not when another of
        // its records repaired it.
        let mut free: BTreeMap<(usize, bool), Loose> = BTreeMap::new();
        for (end, held) in self.ends.iter().zip(&held) {
            let Some(&(i, last)) = held.as_ref() else {
                continue;
            };
            // The end's own line: the key recorded and the key of the line that holds the end
            // now (another, when a join took the end's line into it).
            let own: BTreeSet<Key> = std::iter::once(end.object)
                .chain(key_of(&objects[i]))
                .collect();
            let at = end_point(&objects[i], last);
            // Every line the end meets: on its body (`false`) or only at a recorded end (`true`).
            // The other pieces of its own line do not count: a cut right at the end leaves one.
            let meets: Vec<bool> = grid
                .near(at, touch)
                .into_iter()
                .filter(|&j| {
                    j != i
                        && is_line(&objects[j], &self.line_ids)
                        && key_of(&objects[j]).is_none_or(|k| !own.contains(&k))
                })
                .filter_map(|j| {
                    let (d, q) = distance(&objects[j], at);
                    (d <= touch).then(|| at_recorded_end(&objects[j], recorded.get(&j), q, touch))
                })
                .collect();
            if meets.contains(&false) {
                continue;
            }
            // The lines it may move onto: those it touched (rank 0), the lines they went into by
            // a join (the same rank) and what replaced them (one rank more each step).
            let mut ranks: BTreeMap<Key, usize> = BTreeMap::new();
            let mut open: Vec<(Key, usize)> = end.touched.iter().map(|&k| (k, 0)).collect();
            while let Some((key, rank)) = open.pop() {
                if ranks.get(&key).is_some_and(|&r| r <= rank) {
                    continue;
                }
                ranks.insert(key, rank);
                open.extend(
                    self.absorbed
                        .get(&key)
                        .into_iter()
                        .flatten()
                        .map(|&k| (k, rank)),
                );
                open.extend(replaced_by.get(&key).map(|&k| (k, rank + 1)));
            }
            let parted = |key: &Key| {
                cut_for
                    .get(key)
                    .is_some_and(|w| w.iter().any(|k| own.contains(k)))
            };
            // Every line it may move onto was cut for its own line: the gap is meant.
            let meant = !ranks.is_empty() && ranks.keys().all(parted);
            let best = ranks
                .into_iter()
                .flat_map(|(key, rank)| {
                    let pieces = pieces.get(&key).map_or(&[][..], Vec::as_slice);
                    pieces.iter().map(move |&j| (key, rank, j))
                })
                .filter(|&(key, _, j)| {
                    j != i
                        && !own.contains(&key)
                        && is_line(&objects[j], &self.line_ids)
                        && !parted(&key)
                })
                .filter_map(|(key, rank, j)| {
                    let (d, q) = distance(&objects[j], at);
                    let via_end = at_recorded_end(&objects[j], recorded.get(&j), q, touch);
                    (d <= within).then_some((via_end, rank, key, d, j, q))
                })
                .min_by(|a, b| {
                    (a.0, a.1, a.2)
                        .cmp(&(b.0, b.1, b.2))
                        .then(a.3.total_cmp(&b.3))
                        .then(a.4.cmp(&b.4))
                });
            match best {
                Some((_, _, _, d, _, q)) if d > touch => {
                    let vertex = if last { objects[i].coords.len() - 1 } else { 0 };
                    let c = &mut objects[i].coords[vertex];
                    c.x = (q.x * 1000.0).round() as i32;
                    c.y = (q.y * 1000.0).round() as i32;
                    grid.add(i, &objects[i]);
                    repaired += 1;
                    free.remove(&(i, last));
                }
                // It meets the line already, or meets another end and has no line in reach.
                Some(_) => {}
                None if !meets.is_empty() || meant => {}
                None => {
                    let loose = free.entry((i, last)).or_default();
                    loose.touched.extend(end.touched_ids.iter().cloned());
                    loose.touched_way |= end.touched_way;
                }
            }
        }
        (repaired, free)
    }
}

/// What the records of one free end name: the lines it touched and whether one was a way.
#[derive(Debug, Default)]
struct Loose {
    touched: BTreeSet<String>,
    touched_way: bool,
}

/// The source and id of `object` as `freemapper:also` names them (`osm/way/1`), when it has an
/// id.
fn source_id(object: &MapObject) -> Option<String> {
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

/// The tag that holds the [`Key`] of a line object while another recording of the junctions
/// uses [`JUNCTION_TAG`] ([`set_aside`], [`bring_back`]).
pub(super) const ASIDE_TAG: &str = "freemapper:junction-aside";

/// Removes the tag [`JUNCTION_TAG`] (and a key set aside) from every object.
pub(super) fn forget(objects: &mut [MapObject]) {
    for object in objects {
        object
            .tags
            .retain(|(k, _)| k != JUNCTION_TAG && k != ASIDE_TAG);
    }
}

/// The key of `object` set aside by [`set_aside`], when it carries one.
pub(super) fn aside_key_of(object: &MapObject) -> Option<Key> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == ASIDE_TAG)
        .and_then(|(_, v)| v.parse().ok())
}

/// Sets the keys of a recording aside, so that another can record and repair in between; the
/// steps keep the tag on the pieces as they keep [`JUNCTION_TAG`]. [`bring_back`] restores
/// them.
pub(super) fn set_aside(objects: &mut [MapObject]) {
    for object in objects {
        object.tags.retain(|(k, _)| k != ASIDE_TAG);
        for (k, _) in &mut object.tags {
            if k == JUNCTION_TAG {
                *k = ASIDE_TAG.to_string();
            }
        }
    }
}

/// The keys set aside by [`set_aside`] are the keys again; the keys of a recording in between
/// go.
pub(super) fn bring_back(objects: &mut [MapObject]) {
    for object in objects {
        object.tags.retain(|(k, _)| k != JUNCTION_TAG);
        for (k, _) in &mut object.tags {
            if k == ASIDE_TAG {
                *k = JUNCTION_TAG.to_string();
            }
        }
    }
}

/// The key of `object`, when it carries one.
pub(super) fn key_of(object: &MapObject) -> Option<Key> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == JUNCTION_TAG)
        .and_then(|(_, v)| v.parse().ok())
}

/// Ends closer than this touch a line (mm on paper): [`TOUCH_M`] at the scale of the map, with
/// the tolerance of the comparisons.
fn touch_mm(georef: &Georeferencing) -> f64 {
    PaperMm::of_metres(TOUCH_M, georef.scale_denominator).0 + TOLERANCE_MM
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
    use crate::build::line_join::join_lines_keyed;
    use crate::build::training::tests::{SCALE, area, georef_at, isom, line};
    use crate::omap::SymbolSet;

    /// A line `code` through `points` (paper mm) with the source id `id`.
    fn tagged(set: &SymbolSet, code: &str, id: &str, points: &[(f64, f64)]) -> MapObject {
        let mut object = line(set, code, points);
        object
            .tags
            .push(("freemapper:source_id".to_string(), id.to_string()));
        object
    }

    /// `object` with its coordinates through `points` (paper mm), its tags kept.
    fn moved(set: &SymbolSet, object: &MapObject, points: &[(f64, f64)]) -> MapObject {
        let mut moved = object.clone();
        moved.coords = line(set, "505", points).coords;
        moved
    }

    fn key(object: &MapObject) -> Key {
        key_of(object).unwrap()
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
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 1, "{junctions:?}");
        objects[1] = moved(&set, &objects[1], &[(0.0, 10.3), (40.0, 10.3)]);
        let flags = objects[0].coords[1].flags;
        let result = junctions.repair(&mut objects, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (1, 0));
        assert_eq!(mm(&objects[0].coords[1]), (20.0, 10.3));
        assert_eq!(objects[0].coords[1].flags, flags);
        assert_eq!(mm(&objects[0].coords[0]), (20.0, 0.0), "the free end stays");
        assert_eq!(mm(&objects[1].coords[0]), (0.0, 10.3), "the line stays");
    }

    #[test]
    fn an_end_moves_to_the_replacing_middle_line() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut before = vec![
            tagged(&set, "505", "p", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "503", "a", &[(0.0, 10.0), (40.0, 10.0)]),
            tagged(&set, "503", "b", &[(0.0, 10.4), (40.0, 10.4)]),
        ];
        let junctions = Junctions::record(&mut before, &Classes::of(&set), &georef);
        // The pair becomes its middle line, which keeps the tags of `b`; `a` goes.
        let mut after = vec![
            before[0].clone(),
            moved(&set, &before[2], &[(0.0, 10.2), (40.0, 10.2)]),
        ];
        let replaced_by = BTreeMap::from([(key(&before[1]), key(&before[2]))]);
        let result = junctions.repair(&mut after, &replaced_by, 0.5, &georef);
        assert_eq!(result, (1, 0));
        assert_eq!(mm(&after[0].coords[1]), (20.0, 10.2));
    }

    #[test]
    fn an_end_too_far_stays_free_and_is_counted() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        objects[1] = moved(&set, &objects[1], &[(0.0, 11.0), (40.0, 11.0)]);
        let result = junctions.repair(&mut objects, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (0, 1));
        assert_eq!(mm(&objects[0].coords[1]), (20.0, 10.0));
        assert_eq!(mm(&objects[0].coords[0]), (20.0, 0.0), "the free end stays");
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
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        assert!(junctions.ends.is_empty(), "{junctions:?}");
        objects[1] = moved(&set, &objects[1], &[(0.0, 10.3), (40.0, 10.3)]);
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
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 2, "{junctions:?}");
        objects[2] = moved(&set, &objects[2], &[(0.0, 10.3), (40.0, 10.3)]);
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
        let mut before = vec![
            tagged(&set, "505", "p", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "506", "q", &[(20.0, 10.0), (30.0, 20.0)]),
        ];
        let junctions = Junctions::record(&mut before, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 2, "{junctions:?}");
        let mut after = before.clone();
        let result = junctions.repair(&mut after, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (0, 0));
        assert_eq!(after, before);
    }

    #[test]
    fn a_dropped_neighbour_does_not_shift_the_keys() {
        let set = isom();
        let georef = georef_at(SCALE);
        // Two paths of one source: the first ends on the road, the second stops 0.2 mm short.
        let mut objects = vec![
            tagged(&set, "505", "s", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "505", "s", &[(30.0, 0.0), (30.0, 9.8)]),
            tagged(&set, "503", "r", &[(0.0, 10.0), (40.0, 10.0)]),
        ];
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 1, "{junctions:?}");
        // The first path goes and the road shifts: the second path is not the first.
        let mut after = vec![
            objects[1].clone(),
            moved(&set, &objects[2], &[(0.0, 10.3), (40.0, 10.3)]),
        ];
        let result = junctions.repair(&mut after, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (0, 0));
        assert_eq!(after[0], objects[1]);
    }

    #[test]
    fn a_split_line_is_found_by_its_pieces() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        // Both lines are cut in two, and the pieces of the road shift.
        let (path, road) = (&objects[0], &objects[1]);
        let mut after = vec![
            moved(&set, path, &[(20.0, 0.0), (20.0, 5.0)]),
            moved(&set, road, &[(0.0, 10.3), (10.0, 10.3)]),
            moved(&set, path, &[(20.0, 5.0), (20.0, 10.0)]),
            moved(&set, road, &[(10.0, 10.3), (40.0, 10.3)]),
        ];
        let result = junctions.repair(&mut after, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (1, 0));
        assert_eq!(mm(&after[2].coords[1]), (20.0, 10.3));
        assert_eq!(mm(&after[2].coords[0]), (20.0, 5.0));
        assert_eq!(
            mm(&after[0].coords[1]),
            (20.0, 5.0),
            "the other piece stays"
        );
    }

    #[test]
    fn a_line_cut_at_its_recorded_end_still_follows() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        // The path is cut right at its end on the road, and the road shifts: the other piece
        // of the path, which meets the end, is no line the end touches.
        let (path, road) = (&objects[0], &objects[1]);
        let mut after = vec![
            moved(&set, path, &[(20.0, 0.0), (20.0, 9.995)]),
            moved(&set, path, &[(20.0, 9.995), (20.0, 10.0)]),
            moved(&set, road, &[(0.0, 10.3), (40.0, 10.3)]),
        ];
        let result = junctions.repair(&mut after, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (1, 0));
        assert_eq!(mm(&after[1].coords[1]), (20.0, 10.3));
    }

    /// Joins the roads (503) and paths (505) of `objects` as `train` does after the merge: with
    /// the keys set aside, and the joins passed to `junctions`.
    fn join_ways(set: &SymbolSet, objects: &mut Vec<MapObject>, junctions: &mut Junctions) {
        let ids = ["503", "505"].map(|c| set.id_for_code(c).unwrap());
        set_aside(objects);
        let georef = georef_at(SCALE);
        let (_, absorbed) = join_lines_keyed(objects, &ids, &[], &georef, Some(ASIDE_TAG));
        bring_back(objects);
        junctions.absorb(&absorbed);
    }

    #[test]
    fn an_end_on_a_line_that_a_join_took_follows_the_joined_line() {
        // The path ends on the short road `s`, which a join takes into the longer road `l`
        // (whose key the joined road keeps); then the joined road shifts.
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = vec![
            tagged(&set, "505", "p", &[(5.0, 0.0), (5.0, 10.0)]),
            tagged(&set, "503", "s", &[(0.0, 10.0), (10.0, 10.0)]),
            tagged(&set, "503", "l", &[(10.0, 10.0), (40.0, 10.0)]),
        ];
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        let short = key(&objects[1]);
        join_ways(&set, &mut objects, &mut junctions);
        assert_eq!(objects.len(), 2, "{objects:?}");
        let road = objects.iter().position(|o| key_of(o) != Some(0)).unwrap();
        assert_ne!(
            key(&objects[road]),
            short,
            "the joined road keeps the key of `l`"
        );
        objects[road] = moved(&set, &objects[road], &[(0.0, 10.3), (40.0, 10.3)]);
        let result = junctions.repair_after_cuts(
            &mut objects,
            &BTreeMap::new(),
            &BTreeMap::new(),
            0.5,
            &georef,
            &set,
        );
        assert_eq!((result.0, result.1.len()), (1, 0));
        let path = objects.iter().position(|o| key_of(o) == Some(0)).unwrap();
        assert_eq!(mm(&objects[path].coords[1]), (5.0, 10.3));
    }

    #[test]
    fn the_end_of_a_line_that_a_join_took_is_found_on_the_joined_line() {
        // The short path ends on the road; a join takes it into the longer path before it, and
        // the road shifts: the end is the joined path's now.
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = vec![
            tagged(&set, "505", "long", &[(5.0, -30.0), (5.0, 0.0)]),
            tagged(&set, "505", "short", &[(5.0, 0.0), (5.0, 10.0)]),
            tagged(&set, "503", "r", &[(0.0, 10.0), (40.0, 10.0)]),
        ];
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        join_ways(&set, &mut objects, &mut junctions);
        assert_eq!(objects.len(), 2, "{objects:?}");
        objects[1] = moved(&set, &objects[1], &[(0.0, 10.3), (40.0, 10.3)]);
        let result = junctions.repair_after_cuts(
            &mut objects,
            &BTreeMap::new(),
            &BTreeMap::new(),
            0.5,
            &georef,
            &set,
        );
        assert_eq!((result.0, result.1.len()), (1, 0));
        let end = mm(objects[0].coords.last().unwrap());
        assert_eq!(end, (5.0, 10.3));
    }

    #[test]
    fn an_end_on_the_line_that_took_another_piece_of_its_line_meets_it() {
        // The path ends on the body of the path `l`. It is split, and a join takes its other
        // piece into `l`: the end still meets `l`, which is no piece of its own line.
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = vec![
            tagged(&set, "505", "p", &[(20.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "505", "l", &[(0.0, 10.0), (40.0, 10.0)]),
        ];
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 1, "{junctions:?}");
        let mut after = vec![
            moved(&set, &objects[0], &[(20.0, 5.0), (20.0, 10.0)]),
            moved(&set, &objects[0], &[(-10.0, 10.0), (0.0, 10.0)]),
            objects[1].clone(),
        ];
        join_ways(&set, &mut after, &mut junctions);
        assert_eq!(after.len(), 2, "{after:?}");
        let before = after.clone();
        let result = junctions.repair_after_cuts(
            &mut after,
            &BTreeMap::new(),
            &BTreeMap::new(),
            0.5,
            &georef,
            &set,
        );
        assert_eq!((result.0, result.1.len()), (0, 0));
        assert_eq!(after, before);
    }

    #[test]
    fn an_end_recorded_twice_is_counted_free_once() {
        // The path's end is recorded by `record` and again by `attach`; the road goes.
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        junctions.attach(
            &[Attached {
                object: key(&objects[0]),
                last: true,
                at: Coord { x: 20.0, y: 10.0 },
                onto: key(&objects[1]),
            }],
            &objects,
        );
        assert_eq!(junctions.ends.len(), 2, "{junctions:?}");
        let mut after = vec![objects[0].clone()];
        let result = junctions.repair(&mut after, &BTreeMap::new(), 0.5, &georef);
        assert_eq!(result, (0, 1));
    }

    #[test]
    fn forget_removes_the_tag() {
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        objects.push(area(&set, "521", (50.0, 0.0), (60.0, 10.0)));
        let _ = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        let tagged: Vec<bool> = objects.iter().map(|o| key_of(o).is_some()).collect();
        assert_eq!(tagged, [true, true, false]);
        forget(&mut objects);
        assert!(
            objects
                .iter()
                .all(|o| o.tags.iter().all(|(k, _)| k != JUNCTION_TAG)),
            "{objects:?}"
        );
    }
}
