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

use std::collections::{BTreeMap, BTreeSet, HashMap};

use geo_types::Coord;

use super::lines::Polyline;
use super::{FreeEnd, is_closed};
use crate::build::conflicts::{
    Class, Classes, Footprints, Grade, Relation, TOLERANCE_MM, conflict_of_first, relation,
};
use crate::build::line_join::TOUCH_M;
use crate::georef::Georeferencing;
use crate::measure::PaperMm;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::render::geometry::parts;
use crate::render::symbols::RenderSymbols;

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

/// One recorded end: of which object, whether it is the last coordinate, where it lies now (mm on
/// paper; [`Junctions::follow`] and the repairs keep it up to date, and `gone` is set once the
/// end is no longer an end of its line) and the lines it touched (sorted), with their source ids
/// (sorted, those with an id) and whether one of them was a way, for the report.
#[derive(Debug, Clone)]
struct End {
    object: Key,
    last: bool,
    at: Coord<f64>,
    gone: bool,
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
    /// object, or of its own line's body away from the end (a loop). Ends of closed lines are not recorded. Every line object of these classes gets
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
                let mut near: Vec<usize> = grid
                    .near(at, touch)
                    .into_iter()
                    .filter(|&j| j != i && distance(&objects[j], at).0 <= touch)
                    .filter(|&j| key_of(&objects[j]).is_some())
                    .collect();
                // An end on its own line's body closes a loop (a road joined round a
                // roundabout): it is a junction too.
                if meets_own_body(object, last, touch) {
                    near.push(i);
                }
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
                        gone: false,
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

    /// The ways each end of each way of `objects` is linked to by a recorded junction, by object
    /// index (sorted), for the minimum sizes after give way: a short way piece that still links
    /// ways that stay is no dead end, even where a shift or a cut moved its ends off them, since
    /// the last repair draws the junctions together again. The keys are read from the tag `tag`
    /// (the keys set aside while give way runs). A recorded end links the end of its line nearest
    /// where it was, when within `reach` mm, to each line it touched that lies within `reach` of
    /// that end, and back when the nearest end of that line lies there too. A closed line has no
    /// ends: it is linked by both slots to every way whose end touched it.
    pub(super) fn links(
        &self,
        objects: &[MapObject],
        tag: &str,
        reach: f64,
    ) -> Vec<[Vec<usize>; 2]> {
        let within = reach + TOLERANCE_MM;
        let mut pieces: BTreeMap<Key, Vec<usize>> = BTreeMap::new();
        for (i, object) in objects.iter().enumerate() {
            if is_line(object, &self.way_ids)
                && let Some(key) = key_in(object, tag)
            {
                pieces.entry(key).or_default().push(i);
            }
        }
        let of = |family: &BTreeSet<Key>| -> Vec<usize> {
            family
                .iter()
                .filter_map(|k| pieces.get(k))
                .flatten()
                .copied()
                .collect()
        };
        let mut links: Vec<[BTreeSet<usize>; 2]> = vec![Default::default(); objects.len()];
        for end in self.ends.iter().filter(|e| !e.gone) {
            let own = self.family(end.object);
            let mine = of(&own);
            for &t in &end.touched {
                let theirs = self.family(t);
                if !own.is_disjoint(&theirs) {
                    continue;
                }
                let theirs = of(&theirs);
                for &a in &mine {
                    for &b in &theirs {
                        for (x, y) in [(a, b), (b, a)] {
                            if is_closed(&objects[x]) {
                                links[x][0].insert(y);
                                links[x][1].insert(y);
                                continue;
                            }
                            // The end of `x` nearest the junction (of two as near, the first).
                            let [first, last] =
                                [false, true].map(|l| length(end_point(&objects[x], l), end.at));
                            let (last, d) = if last < first {
                                (true, last)
                            } else {
                                (false, first)
                            };
                            let p = end_point(&objects[x], last);
                            if d <= within && distance(&objects[y], p).0 <= within {
                                links[x][usize::from(last)].insert(y);
                            }
                        }
                    }
                }
            }
        }
        links
            .into_iter()
            .map(|[first, last]| [first.into_iter().collect(), last.into_iter().collect()])
            .collect()
    }

    /// Finds every recorded end again after a step that moved lines: the end of a piece of its
    /// line (or of a line its line went into by a join, keys read from the tag `tag`) within
    /// `within` mm of where it lay is where it lies now. An end found nowhere is gone: a step cut
    /// it off with its stretch, or a join made it a vertex inside a line. A step that moves no
    /// line is followed with the touch distance, one that moves lines with the reach of the
    /// repairs, so an end is found by its key and its position in turn, never by a distance
    /// that several steps add up.
    pub(super) fn follow(&mut self, objects: &[MapObject], tag: &str, within: f64) {
        let pieces = pieces_by(objects, tag, &self.line_ids);
        for k in 0..self.ends.len() {
            if self.ends[k].gone {
                continue;
            }
            match self.resolve(objects, &pieces, &self.ends[k], within) {
                Some((i, last)) => self.ends[k].at = end_point(&objects[i], last),
                None => self.ends[k].gone = true,
            }
        }
    }

    /// Where `end` lies now: the end of a piece of its line, or of a line its line went into by a
    /// join (`pieces` by key), nearest its position and within `within` mm of it (of two as
    /// near, the earlier object, then the same end as recorded).
    fn resolve(
        &self,
        objects: &[MapObject],
        pieces: &BTreeMap<Key, Vec<usize>>,
        end: &End,
        within: f64,
    ) -> Option<(usize, bool)> {
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
                gone: false,
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
        &mut self,
        objects: &mut [MapObject],
        replaced_by: &BTreeMap<Key, Key>,
        reach: f64,
        georef: &Georeferencing,
    ) -> (usize, usize) {
        let after = After {
            replaced_by,
            cut_for: &BTreeMap::new(),
            rider_reach: &BTreeMap::new(),
        };
        let within = reach + TOLERANCE_MM;
        let (repaired, free) = self.settle(objects, &after, reach, within, georef, None);
        (repaired, free.len())
    }

    /// As [`repair`](Self::repair), after lines were cut where they kept no gap
    /// ([`After::cut_for`]): an end is not drawn onto a line that was cut for the end's own
    /// line, since that gap was opened on purpose (two lines that lay on each other). Such an end
    /// stays where it is, and it is not counted free when every line it could move onto was cut
    /// for its own line.
    ///
    /// An end that met a path or tram that went or was cut in the gap zone of a road is drawn
    /// onto the road that replaced it from as far as that line lay from the road
    /// ([`After::rider_reach`]), beyond `reach`, when the moved line makes no new conflict with
    /// an object it keeps apart from.
    ///
    /// A line that a join took into another ([`absorb`](Self::absorb)) is found on the line it
    /// went into, both as the end's own line and as a line the end touched. One end recorded
    /// more than once is counted free once.
    ///
    /// Returns how many ends were repaired and the ends left free, in the order of the objects,
    /// each named by its line's source id and symbol code (of `symbol_set`).
    pub(super) fn repair_after_cuts(
        &mut self,
        objects: &mut [MapObject],
        after: &After,
        reach: f64,
        georef: &Georeferencing,
        kind: MapKind,
        symbol_set: &SymbolSet,
    ) -> (usize, Vec<FreeEnd>) {
        let mut checker = Checker::new(objects, kind, georef.scale_denominator, symbol_set);
        // Followed after every step that moved lines, an end lies where it lay or is gone.
        let touch = touch_mm(georef);
        let (repaired, free) =
            self.settle(objects, after, reach, touch, georef, Some(&mut checker));
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
    /// repaired, and the ends left free by object index and end. An end is found within `found`
    /// mm of where it lay ([`resolve`](Self::resolve)); afterwards every end lies where it lies
    /// now, or is gone. With a `checker`, an end moves only when the moved line makes no new
    /// conflict; without one, no end moves beyond `reach`.
    fn settle(
        &mut self,
        objects: &mut [MapObject],
        after: &After,
        reach: f64,
        found: f64,
        georef: &Georeferencing,
        mut checker: Option<&mut Checker>,
    ) -> (usize, BTreeMap<(usize, bool), Loose>) {
        let After {
            replaced_by,
            cut_for,
            rider_reach,
        } = after;
        let touch = touch_mm(georef);
        let within = reach + TOLERANCE_MM;
        // The line objects of each key, in order.
        let pieces = pieces_by(objects, JUNCTION_TAG, &self.line_ids);
        // Where each recorded end is now.
        let held: Vec<Option<(usize, bool)>> = self
            .ends
            .iter()
            .map(|end| {
                if end.gone {
                    None
                } else {
                    self.resolve(objects, &pieces, end, found)
                }
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
        // The ends that one record found meeting a line or repaired: another record of the same
        // end does not leave it free.
        let mut settled: BTreeSet<(usize, bool)> = BTreeSet::new();
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
                settled.insert((i, last));
                continue;
            }
            // The lines it may move onto: those it touched (rank 0), the lines they went into by
            // a join (the same rank) and what replaced them (one rank more each step), each with
            // how far it may lie: `within`, or as far as a rider lay from the road that replaced
            // it.
            let mut ranks: BTreeMap<Key, (usize, f64)> = BTreeMap::new();
            let mut open: Vec<(Key, usize, f64)> =
                end.touched.iter().map(|&k| (k, 0, within)).collect();
            while let Some((key, rank, far)) = open.pop() {
                if ranks
                    .get(&key)
                    .is_some_and(|&(r, f)| r < rank || (r == rank && f >= far))
                {
                    continue;
                }
                ranks.insert(key, (rank, far));
                open.extend(
                    self.absorbed
                        .get(&key)
                        .into_iter()
                        .flatten()
                        .map(|&k| (k, rank, far)),
                );
                if let Some(&next) = replaced_by.get(&key) {
                    let wide = match (&checker, rider_reach.get(&key)) {
                        (Some(_), Some(&r)) => far.max(r + TOLERANCE_MM),
                        _ => far,
                    };
                    open.push((next, rank + 1, wide));
                }
            }
            let parted = |key: &Key| {
                cut_for
                    .get(key)
                    .is_some_and(|w| w.iter().any(|k| own.contains(k)))
            };
            // Every line it may move onto was cut for its own line: the gap is meant. The lines it
            // touched that a join took into its own line are not among them.
            let into_own = |k: &Key| self.family(*k).iter().any(|f| own.contains(f));
            let others: Vec<&Key> = ranks.keys().filter(|k| !into_own(k)).collect();
            let meant = !others.is_empty() && others.into_iter().all(parted);
            // Every line it touched went into its own line by a join, which closed a loop: it
            // meets that line where it meets the line's own body away from the end.
            let looped = !ranks.is_empty()
                && ranks.keys().all(into_own)
                && meets_own_body(&objects[i], last, touch);
            let best = ranks
                .iter()
                .flat_map(|(&key, &(rank, far))| {
                    let pieces = pieces.get(&key).map_or(&[][..], Vec::as_slice);
                    pieces.iter().map(move |&j| (key, rank, far, j))
                })
                .filter(|&(key, _, _, j)| {
                    j != i
                        && !own.contains(&key)
                        && is_line(&objects[j], &self.line_ids)
                        && !parted(&key)
                })
                .filter_map(|(key, rank, far, j)| {
                    let (d, q) = distance(&objects[j], at);
                    let via_end = at_recorded_end(&objects[j], recorded.get(&j), q, touch);
                    (d <= far).then_some((via_end, rank, key, d, j, q))
                })
                .min_by(|a, b| {
                    (a.0, a.1, a.2)
                        .cmp(&(b.0, b.1, b.2))
                        .then(a.3.total_cmp(&b.3))
                        .then(a.4.cmp(&b.4))
                });
            // With a checker, a move is made only when it makes no new conflict; without one,
            // none goes beyond the usual reach.
            let allowed = |checker: &mut Option<&mut Checker>, d: f64, q: Coord<f64>| -> bool {
                match checker.as_deref_mut() {
                    Some(c) => c.allows(objects, i, &with_end(&objects[i], last, q)),
                    None => d <= within,
                }
            };
            // Every line it touched is its own line now and it no longer meets that line's body:
            // it is drawn onto the body, beyond the stretch next to the end.
            let target = best.map(|b| (b.3, b.5)).or_else(|| {
                (!ranks.is_empty() && ranks.keys().all(into_own) && !looped)
                    .then(|| own_body(&objects[i], last, 2.0 * within))
                    .flatten()
                    .filter(|&(d, _)| d <= within)
            });
            match target {
                Some((d, q)) if d > touch && allowed(&mut checker, d, q) => {
                    objects[i] = with_end(&objects[i], last, q);
                    if let Some(c) = checker.as_deref_mut() {
                        c.moved(objects, i);
                    }
                    grid.add(i, &objects[i]);
                    repaired += 1;
                    settled.insert((i, last));
                }
                // It meets the line already, or meets another end and has no line in reach.
                Some((d, _)) if d <= touch => {
                    settled.insert((i, last));
                }
                None if looped => {
                    settled.insert((i, last));
                }
                None if !meets.is_empty() || meant => {}
                _ => {
                    let loose = free.entry((i, last)).or_default();
                    loose.touched.extend(end.touched_ids.iter().cloned());
                    loose.touched_way |= end.touched_way;
                }
            }
        }
        free.retain(|end, _| !settled.contains(end));
        // Every end lies where it lies now, after the moves; an end found nowhere is gone.
        for (end, held) in self.ends.iter_mut().zip(held) {
            match held {
                Some((i, last)) => end.at = end_point(&objects[i], last),
                None => end.gone = true,
            }
        }
        (repaired, free)
    }
}

/// The line objects of `objects` (symbols `line_ids`) by their key in the tag `tag`, in order.
fn pieces_by(
    objects: &[MapObject],
    tag: &str,
    line_ids: &BTreeSet<u32>,
) -> BTreeMap<Key, Vec<usize>> {
    let mut pieces: BTreeMap<Key, Vec<usize>> = BTreeMap::new();
    for (i, object) in objects.iter().enumerate() {
        if is_line(object, line_ids)
            && let Some(key) = key_in(object, tag)
        {
            pieces.entry(key).or_default().push(i);
        }
    }
    pieces
}

/// Whether the end of `object` (the last when `last`) lies within `touch` mm of the line's own
/// body, away from the end: farther along the line than `2 × touch` from it.
fn meets_own_body(object: &MapObject, last: bool, touch: f64) -> bool {
    own_body(object, last, 2.0 * touch).is_some_and(|(d, _)| d <= touch)
}

/// The distance from the end of `object` (the last when `last`) to the line's own body farther
/// along it than `skip` mm from that end, and the point of the body nearest the end (of the
/// part of the end, the first or the last; `None` when the part is no longer than `skip`).
fn own_body(object: &MapObject, last: bool, skip: f64) -> Option<(f64, Coord<f64>)> {
    let parts = polylines(object);
    let line = if last { parts.last()? } else { parts.first()? };
    let at = end_point(object, last);
    let total = line.length();
    let (from, to) = if last {
        (0.0, total - skip)
    } else {
        (skip, total)
    };
    if to <= from {
        return None;
    }
    let body = Polyline::new(line.between(from, to));
    let q = body.point(body.locate(at));
    Some((length(at, q), q))
}

/// What the last repair needs to know of the steps before it.
pub(super) struct After<'a> {
    /// The line that took the place of another, by key (`Merged::replaced_by`).
    pub(super) replaced_by: &'a BTreeMap<Key, Key>,
    /// The lines each line was cut or dropped for by give way, by key.
    pub(super) cut_for: &'a BTreeMap<Key, BTreeSet<Key>>,
    /// How far a path or tram that went in the gap zone of a road lay from it, by key
    /// (`Merged::rider_reach`).
    pub(super) rider_reach: &'a BTreeMap<Key, f64>,
}

/// `object` with its first or last vertex (`last`) at `q` (mm on paper), its flags kept.
fn with_end(object: &MapObject, last: bool, q: Coord<f64>) -> MapObject {
    let mut moved = object.clone();
    let vertex = if last { moved.coords.len() - 1 } else { 0 };
    let c = &mut moved.coords[vertex];
    c.x = (q.x * 1000.0).round() as i32;
    c.y = (q.y * 1000.0).round() as i32;
    moved
}

/// The objects round a moved line end that it may come too close to: within this much of the
/// stretch that moved (mm on paper), beyond every symbol's reach and the gap.
const CHECK_REACH_MM: f64 = 3.0;

/// Whether a line with a moved end comes too close to an object it keeps apart from, measured
/// as the conflict measure does ([`conflict_of_first`]). The footprints are drawn once per
/// object and drawn again when the object moves.
pub(super) struct Checker {
    symbols: RenderSymbols,
    classes: Classes,
    kind: MapKind,
    scale_denominator: u32,
    footprints: Footprints,
    /// The slot in `footprints` of each object drawn so far.
    slots: HashMap<usize, usize>,
    /// The frame of each object's coordinates (map units: min x, min y, max x, max y).
    frames: Vec<Option<[i32; 4]>>,
}

impl Checker {
    pub(super) fn new(
        objects: &[MapObject],
        kind: MapKind,
        scale_denominator: u32,
        symbol_set: &SymbolSet,
    ) -> Self {
        let symbols = RenderSymbols::from_set(symbol_set);
        let classes = Classes::of(symbol_set);
        let footprints = Footprints::with(&Vec::<&MapObject>::new(), &symbols, &classes);
        Self {
            symbols,
            classes,
            kind,
            scale_denominator,
            footprints,
            slots: HashMap::new(),
            frames: objects.iter().map(coord_frame).collect(),
        }
    }

    /// The slot of the footprint of `objects[j]`, drawn when it is first asked for.
    fn slot(&mut self, objects: &[MapObject], j: usize) -> usize {
        if let Some(&slot) = self.slots.get(&j) {
            return slot;
        }
        let slot = self
            .footprints
            .push(&objects[j], &self.symbols, &self.classes);
        self.slots.insert(j, slot);
        slot
    }

    /// `objects[i]` moved: its footprint and frame are drawn again when next asked for.
    pub(super) fn moved(&mut self, objects: &[MapObject], i: usize) {
        self.slots.remove(&i);
        self.frames[i] = coord_frame(&objects[i]);
    }

    /// Whether `objects[i]`, drawn as `moved` (the same line with one end moved), has no conflict
    /// with an object it keeps apart from ([`Relation::Apart`]) near the stretch that moved
    /// that it did not have, none of a worse grade and none longer by more than the tolerance.
    pub(super) fn allows(&mut self, objects: &[MapObject], i: usize, moved: &MapObject) -> bool {
        let class = self.classes.class(objects[i].symbol_id);
        // The stretch that moved: the old and the new end and the vertices next to them.
        let mut frame: Option<[i32; 4]> = None;
        for c in objects[i].coords.iter().chain(&moved.coords).filter(|c| {
            let n = moved.coords.len();
            let near_end = |o: &[MapCoord]| {
                [o.first(), o.get(1), o.get(n.saturating_sub(2)), o.last()]
                    .into_iter()
                    .flatten()
                    .any(|e| (e.x, e.y) == (c.x, c.y))
            };
            near_end(&objects[i].coords) || near_end(&moved.coords)
        }) {
            frame = Some(match frame {
                None => [c.x, c.y, c.x, c.y],
                Some([a, b, x, y]) => [a.min(c.x), b.min(c.y), x.max(c.x), y.max(c.y)],
            });
        }
        let Some([min_x, min_y, max_x, max_y]) = frame else {
            return true;
        };
        let grow = (CHECK_REACH_MM * 1000.0) as i32;
        let near: Vec<usize> = (0..objects.len())
            .filter(|&j| j != i)
            .filter(|&j| {
                self.frames[j].is_some_and(|[a, b, x, y]| {
                    a <= max_x + grow && x >= min_x - grow && b <= max_y + grow && y >= min_y - grow
                })
            })
            .filter(|&j| {
                relation(class, self.classes.class(objects[j].symbol_id)) == Relation::Apart
            })
            .collect();
        if near.is_empty() {
            return true;
        }
        let old = self.slot(objects, i);
        let new = self.footprints.push(moved, &self.symbols, &self.classes);
        let (kind, scale) = (self.kind, self.scale_denominator);
        near.into_iter().all(|j| {
            let other = self.slot(objects, j);
            let Some(now) = conflict_of_first(&self.footprints, new, other, kind, scale) else {
                return true;
            };
            match conflict_of_first(&self.footprints, old, other, kind, scale) {
                None => false,
                Some(before) => {
                    !(before.grade == Grade::Gap && now.grade == Grade::Overlap)
                        && now.length.0 <= before.length.0 + TOLERANCE_MM
                }
            }
        })
    }
}

/// The frame of the coordinates of `object` (map units: min x, min y, max x, max y).
fn coord_frame(object: &MapObject) -> Option<[i32; 4]> {
    let first = object.coords.first()?;
    Some(
        object
            .coords
            .iter()
            .fold([first.x, first.y, first.x, first.y], |[a, b, x, y], c| {
                [a.min(c.x), b.min(c.y), x.max(c.x), y.max(c.y)]
            }),
    )
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
    key_in(object, JUNCTION_TAG)
}

/// The key of `object` in the tag `tag` ([`JUNCTION_TAG`] or [`ASIDE_TAG`]), when it carries one.
fn key_in(object: &MapObject, tag: &str) -> Option<Key> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == tag)
        .and_then(|(_, v)| v.parse().ok())
}

/// Ends closer than this touch a line (mm on paper): [`TOUCH_M`] at the scale of the map, with
/// the tolerance of the comparisons.
pub(super) fn touch_mm(georef: &Georeferencing) -> f64 {
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
    use crate::build::training::tests::{SCALE, area, georef_at, isom, line, point};
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

    /// The last repair with nothing replaced, cut or dropped beside a road.
    fn none() -> After<'static> {
        static EMPTY_KEYS: BTreeMap<Key, Key> = BTreeMap::new();
        static EMPTY_CUTS: BTreeMap<Key, BTreeSet<Key>> = BTreeMap::new();
        static EMPTY_REACH: BTreeMap<Key, f64> = BTreeMap::new();
        After {
            replaced_by: &EMPTY_KEYS,
            cut_for: &EMPTY_CUTS,
            rider_reach: &EMPTY_REACH,
        }
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
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut before, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut before, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
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
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
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
        let result =
            junctions.repair_after_cuts(&mut objects, &none(), 0.5, &georef, MapKind::Forest, &set);
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
        let result =
            junctions.repair_after_cuts(&mut objects, &none(), 0.5, &georef, MapKind::Forest, &set);
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
        let result =
            junctions.repair_after_cuts(&mut after, &none(), 0.5, &georef, MapKind::Forest, &set);
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

    /// A path that ends on a sidewalk 0.8 mm from a 502, beyond the reach, after the sidewalk
    /// went beside the road (the rider rule), so that the road replaced it.
    struct OnASidewalk {
        objects: Vec<MapObject>,
        junctions: Junctions,
        replaced_by: BTreeMap<Key, Key>,
        rider_reach: BTreeMap<Key, f64>,
        cut_for: BTreeMap<Key, BTreeSet<Key>>,
    }

    impl OnASidewalk {
        fn new(set: &SymbolSet, with_point: bool) -> Self {
            let georef = georef_at(SCALE);
            let mut objects = vec![
                tagged(set, "505", "p", &[(25.0, 10.0), (20.0, 0.8)]),
                tagged(set, "505", "sidewalk", &[(5.0, 0.8), (35.0, 0.8)]),
                tagged(set, "502", "road", &[(0.0, 0.0), (40.0, 0.0)]),
            ];
            if with_point {
                // 1.06 mm from the path's end, 0.7 mm from its last stretch once the end is on
                // the road: 526 keeps about 1.01 mm from a 505.
                objects.push(point(set, "526", (19.3, 0.0)));
            }
            let junctions = Junctions::record(&mut objects, &Classes::of(set), &georef);
            let (sidewalk, road) = (key(&objects[1]), key(&objects[2]));
            objects.remove(1);
            Self {
                objects,
                junctions,
                replaced_by: BTreeMap::from([(sidewalk, road)]),
                rider_reach: BTreeMap::from([(sidewalk, 0.85)]),
                cut_for: BTreeMap::new(),
            }
        }

        fn after(&self) -> After<'_> {
            After {
                replaced_by: &self.replaced_by,
                cut_for: &self.cut_for,
                rider_reach: &self.rider_reach,
            }
        }
    }

    /// The reach of the repairs at 1:10 000 on the forest map: the shift bound plus the gap.
    fn reach() -> f64 {
        0.5 + crate::build::conflicts::gap(MapKind::Forest, SCALE).0
    }

    #[test]
    fn an_end_on_a_sidewalk_that_went_is_drawn_onto_the_road() {
        let set = isom();
        let georef = georef_at(SCALE);
        assert!(reach() < 0.8, "{}", reach());
        let f = OnASidewalk::new(&set, false);
        // The repairs before the cuts check no conflict: they do not reach that far.
        let mut early = f.objects.clone();
        assert_eq!(
            f.junctions
                .clone()
                .repair(&mut early, &f.replaced_by, reach(), &georef),
            (0, 1)
        );
        let after = f.after();
        let mut objects = f.objects.clone();
        let (repaired, free) = f.junctions.clone().repair_after_cuts(
            &mut objects,
            &after,
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!((repaired, free.len()), (1, 0), "{free:?}");
        assert_eq!(mm(&objects[0].coords[1]), (20.0, 0.0));
    }

    #[test]
    fn an_end_drawn_onto_the_road_into_the_gap_of_a_point_stays() {
        let set = isom();
        let georef = georef_at(SCALE);
        let f = OnASidewalk::new(&set, true);
        let mut objects = f.objects.clone();
        let (repaired, free) = f.junctions.clone().repair_after_cuts(
            &mut objects,
            &f.after(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!((repaired, free.len()), (0, 1), "{free:?}");
        assert_eq!(objects, f.objects, "the old end stays");
        assert!(free[0].between_ways, "{free:?}");
        assert_eq!(free[0].touched, ["sidewalk"]);
    }

    #[test]
    fn the_last_repair_keeps_the_old_end_where_the_moved_line_comes_too_close() {
        // The road the path ended on moves 0.5 mm, within reach; drawn onto it, the path's last
        // stretch would pass 0.9 mm from a point symbol (526 keeps about 1.01 mm from a 505),
        // which its old end kept clear of (1.08 mm).
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = vec![
            tagged(&set, "505", "p", &[(25.0, 10.0), (20.0, 0.0)]),
            tagged(&set, "503", "r", &[(0.0, 0.0), (40.0, 0.0)]),
            point(&set, "526", (19.1, -0.6)),
        ];
        let junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        objects[1] = moved(&set, &objects[1], &[(0.0, -0.5), (40.0, -0.5)]);
        // The repairs before give way do not check: what they leave too close, give way settles.
        let mut early = objects.clone();
        assert_eq!(
            junctions
                .clone()
                .repair(&mut early, &BTreeMap::new(), reach(), &georef),
            (1, 0)
        );
        let before = objects.clone();
        let (repaired, free) = junctions.clone().repair_after_cuts(
            &mut objects,
            &none(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!((repaired, free.len()), (0, 1), "{free:?}");
        assert_eq!(objects, before, "the old end stays");
        // Without the point the end is drawn onto the road.
        objects.pop();
        let (repaired, free) = junctions.clone().repair_after_cuts(
            &mut objects,
            &none(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!((repaired, free.len()), (1, 0), "{free:?}");
        assert_eq!(mm(&objects[0].coords[1]), (20.0, -0.5));
    }

    #[test]
    fn an_end_is_followed_through_moves_that_add_up_beyond_the_reach() {
        // The path's end moves twice by 0.5 mm (a shift and the repair after it), 1 mm in all;
        // then the road it met goes. Followed after each move, the last repair still finds the
        // end, and counts it free.
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = path_on_road(&set);
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        let within = reach() + TOLERANCE_MM;
        for y in [10.5, 11.0] {
            objects[0] = moved(&set, &objects[0], &[(20.0, 0.0), (20.0, y)]);
            junctions.follow(&objects, JUNCTION_TAG, within);
        }
        objects.pop();
        let (repaired, free) = junctions.repair_after_cuts(
            &mut objects,
            &none(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!((repaired, free.len()), (0, 1), "{free:?}");
        assert_eq!(free[0].at, (PaperMm(20.0), PaperMm(11.0)));
    }

    #[test]
    fn an_end_that_a_join_took_inside_its_line_is_gone() {
        // Two paths of one symbol meet end to end on a road; a join makes their ends one vertex
        // inside a line. That vertex is no end: neither is counted when the road goes.
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = vec![
            tagged(&set, "505", "a", &[(0.0, 0.0), (20.0, 10.0)]),
            tagged(&set, "505", "b", &[(20.0, 10.0), (40.0, 20.0)]),
            tagged(&set, "503", "r", &[(0.0, 10.0), (40.0, 10.0)]),
        ];
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        join_ways(&set, &mut objects, &mut junctions);
        assert_eq!(objects.len(), 2, "{objects:?}");
        junctions.follow(&objects, JUNCTION_TAG, touch_mm(&georef));
        objects.retain(|o| o.symbol_id == set.id_for_code("505").unwrap());
        let (_, free) = junctions.repair_after_cuts(
            &mut objects,
            &none(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!(free, []);
    }

    #[test]
    fn an_end_that_meets_a_line_by_one_record_is_not_free_by_another() {
        // Two lines `a` and `b` went into `h` by joins; `b` keeps a piece that crosses the end
        // of `h`. The end is recorded for `a` and for `b`, each touching a line that went. For
        // `a` the end meets the piece of `b`; for `b` that piece is its own line. It meets a
        // line, so it is not free.
        let set = isom();
        let georef = georef_at(SCALE);
        let path = set.id_for_code("505").unwrap();
        let mut objects = vec![
            tagged(&set, "505", "h", &[(0.0, 0.0), (10.0, 0.0)]),
            tagged(&set, "505", "b", &[(10.0, -5.0), (10.0, 5.0)]),
        ];
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        let (h, b) = (key(&objects[0]), key(&objects[1]));
        let (a, gone) = (100, 101);
        junctions.ends.clear();
        for object in [a, b] {
            junctions.ends.push(End {
                object,
                last: true,
                at: Coord { x: 10.0, y: 0.0 },
                gone: false,
                touched: vec![gone],
                touched_ids: Vec::new(),
                touched_way: true,
            });
        }
        junctions.absorbed = BTreeMap::from([(a, BTreeSet::from([h])), (b, BTreeSet::from([h]))]);
        assert!(objects.iter().all(|o| o.symbol_id == path));
        let (_, free) = junctions.repair_after_cuts(
            &mut objects,
            &none(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!(free, []);
    }

    #[test]
    fn an_end_on_a_line_that_a_join_took_into_its_own_line_meets_it() {
        // The path `p` ends on the body of the path `t`, which meets `p`'s other end end to end:
        // a join takes `t` into `p`, whose end now meets its own body. It is not free.
        let set = isom();
        let georef = georef_at(SCALE);
        let mut objects = vec![
            tagged(&set, "505", "t", &[(0.0, 0.0), (20.0, 0.0)]),
            tagged(
                &set,
                "505",
                "p",
                &[
                    (20.0, 0.0),
                    (30.0, 0.0),
                    (30.0, 10.0),
                    (10.0, 10.0),
                    (10.0, 0.0),
                ],
            ),
        ];
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        join_ways(&set, &mut objects, &mut junctions);
        assert_eq!(objects.len(), 1, "{objects:?}");
        let (_, free) = junctions.repair_after_cuts(
            &mut objects,
            &none(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!(free, []);
    }

    #[test]
    fn a_road_that_ends_on_its_own_body_is_drawn_back_onto_it() {
        // A road joined round a roundabout ends on its own body. A merge moves that body 0.07 mm
        // off its end: the end is drawn back onto it.
        let set = isom();
        let georef = georef_at(SCALE);
        let ring = [
            (20.0, 10.0),
            (30.0, 10.0),
            (30.0, 20.0),
            (20.0, 20.0),
            (25.0, 10.0),
        ];
        let mut road: Vec<(f64, f64)> = vec![(0.0, 10.0)];
        road.extend(ring);
        let mut objects = vec![tagged(&set, "503", "r", &road)];
        let mut junctions = Junctions::record(&mut objects, &Classes::of(&set), &georef);
        assert_eq!(junctions.ends.len(), 1, "{junctions:?}");
        let mut shifted = road.clone();
        for p in &mut shifted[..3] {
            p.1 += 0.07;
        }
        objects[0] = moved(&set, &objects[0], &shifted);
        let (repaired, free) = junctions.repair_after_cuts(
            &mut objects,
            &none(),
            reach(),
            &georef,
            MapKind::Forest,
            &set,
        );
        assert_eq!((repaired, free.len()), (1, 0), "{free:?}");
        assert_eq!(mm(objects[0].coords.last().unwrap()), (25.0, 10.07));
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
