// SPDX-License-Identifier: AGPL-3.0-or-later

//! Lines of one symbol that meet end to end are joined into one object, so a dash pattern runs
//! through the joint and short pieces stop being objects of their own.
//!
//! The method is that of the cliff join (`build::cliffs`): ends that touch are paired, the pieces
//! are chained, and a chain that closes on itself becomes a closed path. A line with a side (ticks
//! on one side) is never reversed, so a later tick side stays right.

use std::collections::BTreeMap;

use crate::build::dedup::add_also;
use crate::georef::Georeferencing;
use crate::omap::{MapCoord, MapObject, MapObjectKind, closed_ring};

/// Ends closer than this touch (ground metres), as `JOIN_TOUCH_M` in `build/candidates.rs`.
pub const TOUCH_M: f64 = 0.1;
/// At a junction of three or more ends the straightest pair is joined when it turns less than this.
pub const MAX_TURN_DEG: f64 = 30.0;

/// Tolerance of the comparisons at a threshold, so all systems decide alike.
const EPS: f64 = 1e-9;

/// One end of a candidate line: `2 * k` is the start of candidate `k`, `2 * k + 1` its end.
type End = usize;

/// Joins the path objects of each symbol id in `ids` that meet end to end. Returns how many objects went.
///
/// Where exactly two ends meet they join whatever the angle; where more meet, the pair with the
/// smallest turn joins if it turns less than [`MAX_TURN_DEG`] and the others stay. An object whose
/// symbol id is in `sided` is never reversed: it joins only end to start. An object with a
/// coordinate flag other than the closing one is left alone, and so is a closed path. The joined
/// object keeps the tags of the longer piece, names the others in `freemapper:also`, and takes
/// the place of its first member.
pub fn join_lines(
    objects: &mut Vec<MapObject>,
    ids: &[u32],
    sided: &[u32],
    georef: &Georeferencing,
) -> usize {
    join_lines_keyed(objects, ids, sided, georef, None).0
}

/// As [`join_lines`], and with `key` the name of a tag: for every piece that went into a joined
/// object, the value of its tag `key` and the value of that tag on the joined object (the
/// longest piece's), when both have one and they differ, in the order the joins were made.
pub fn join_lines_keyed(
    objects: &mut Vec<MapObject>,
    ids: &[u32],
    sided: &[u32],
    georef: &Georeferencing,
    key: Option<&str>,
) -> (usize, Vec<(String, String)>) {
    join_grouped(objects, ids, sided, georef, key, |_| None, Joint::Merged)
}

/// As [`join_lines`], and only lines of one `group` join: the objects of a symbol are joined
/// among those `group` gives the same value. No vertex is lost: where the end of one piece and
/// the start of the next lie apart (they touch within [`TOUCH_M`]), both stay, and so does the
/// last vertex of a closed chain. Returns how many objects went.
pub fn join_lines_by(
    objects: &mut Vec<MapObject>,
    ids: &[u32],
    sided: &[u32],
    georef: &Georeferencing,
    group: impl Fn(&MapObject) -> Option<String>,
) -> usize {
    join_grouped(objects, ids, sided, georef, None, group, Joint::Kept).0
}

/// What becomes of the two vertices where the pieces of a chain meet.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Joint {
    /// The first vertex of each following piece goes, and a closed chain loses its last: the
    /// training level's join, where the line is generalized anyway.
    Merged,
    /// Only a vertex identical to the one before goes: the base map, where no vertex is lost.
    Kept,
}

/// [`join_lines_keyed`] within the groups of `group`.
fn join_grouped(
    objects: &mut Vec<MapObject>,
    ids: &[u32],
    sided: &[u32],
    georef: &Georeferencing,
    key: Option<&str>,
    group: impl Fn(&MapObject) -> Option<String>,
    joint: Joint,
) -> (usize, Vec<(String, String)>) {
    let mut absorbed = Vec::new();
    let before = objects.len();
    let mut slots: Vec<Option<MapObject>> = std::mem::take(objects).into_iter().map(Some).collect();
    let mut ids: Vec<u32> = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    for id in ids {
        let mut groups: BTreeMap<Option<String>, Vec<usize>> = BTreeMap::new();
        for (i, o) in slots.iter().enumerate() {
            if let Some(o) = o.as_ref().filter(|o| joinable(o, id)) {
                groups.entry(group(o)).or_default().push(i);
            }
        }
        for candidates in groups.values() {
            join_symbol(
                &mut slots,
                candidates,
                sided.contains(&id),
                georef,
                key.map(|key| (key, &mut absorbed)),
                joint,
            );
        }
    }
    *objects = slots.into_iter().flatten().collect();
    (before - objects.len(), absorbed)
}

/// A plain open path of symbol `id` with at least two points and no coordinate flags.
fn joinable(object: &MapObject, id: u32) -> bool {
    object.symbol_id == id
        && matches!(object.kind, MapObjectKind::Path)
        && object.coords.len() >= 2
        && object.coords.iter().all(|c| c.flags == 0)
}

/// The value of the tag `key` of `object`.
fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn join_symbol(
    slots: &mut [Option<MapObject>],
    candidates: &[usize],
    sided: bool,
    georef: &Georeferencing,
    mut absorbed: Option<(&str, &mut Vec<(String, String)>)>,
    joint: Joint,
) {
    let lines: Vec<&[MapCoord]> = candidates
        .iter()
        .map(|&i| slots[i].as_ref().map_or(&[][..], |o| o.coords.as_slice()))
        .collect();
    let partner = pair_ends(&lines, sided, georef);
    if partner.iter().all(Option::is_none) {
        return;
    }
    let chains = chains(&lines, &partner);
    let mut built: Vec<(usize, MapObject)> = Vec::new();
    let mut gone: Vec<usize> = Vec::new();
    for chain in chains {
        if chain.members.len() == 1 && !chain.closed {
            continue;
        }
        let object = build_chain(slots, candidates, &chain, joint);
        if let Some((key, absorbed)) = absorbed.as_mut()
            && let Some(kept) = tag(&object, key)
        {
            for &(k, _) in &chain.members {
                let member = slots[candidates[k]].as_ref().and_then(|o| tag(o, key));
                if let Some(member) = member.filter(|&m| m != kept) {
                    absorbed.push((member.to_string(), kept.to_string()));
                }
            }
        }
        let first = chain.members.iter().map(|&(k, _)| candidates[k]).min();
        let Some(first) = first else { continue };
        gone.extend(chain.members.iter().map(|&(k, _)| candidates[k]));
        built.push((first, object));
    }
    for i in gone {
        slots[i] = None;
    }
    for (i, object) in built {
        slots[i] = Some(object);
    }
}

/// The end each end is joined to, if any.
fn pair_ends(lines: &[&[MapCoord]], sided: bool, georef: &Georeferencing) -> Vec<Option<End>> {
    let n_ends = lines.len() * 2;
    let point = |e: End| -> (f64, f64) {
        let line = lines[e / 2];
        let c = if e.is_multiple_of(2) {
            line[0]
        } else {
            line[line.len() - 1]
        };
        let p = georef.map_to_projected(c);
        (p.easting, p.northing)
    };
    let points: Vec<(f64, f64)> = (0..n_ends).map(point).collect();
    // Ends that touch form a group (union-find over the touching pairs).
    let mut parent: Vec<usize> = (0..n_ends).collect();
    fn root(parent: &mut [usize], mut e: usize) -> usize {
        while parent[e] != e {
            parent[e] = parent[parent[e]];
            e = parent[e];
        }
        e
    }
    let mut by_x: Vec<End> = (0..n_ends).collect();
    by_x.sort_by(|&a, &b| points[a].0.total_cmp(&points[b].0).then(a.cmp(&b)));
    for (at, &a) in by_x.iter().enumerate() {
        for &b in &by_x[at + 1..] {
            if points[b].0 - points[a].0 > TOUCH_M + EPS {
                break;
            }
            let d = (points[a].0 - points[b].0).hypot(points[a].1 - points[b].1);
            if d <= TOUCH_M + EPS {
                let (ra, rb) = (root(&mut parent, a), root(&mut parent, b));
                if ra != rb {
                    parent[ra.max(rb)] = ra.min(rb);
                }
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<End>> = BTreeMap::new();
    for e in 0..n_ends {
        let r = root(&mut parent, e);
        groups.entry(r).or_default().push(e);
    }
    let mut partner: Vec<Option<End>> = vec![None; n_ends];
    for ends in groups.values().filter(|g| g.len() >= 2) {
        let mut best: Option<(f64, End, End)> = None;
        for (i, &a) in ends.iter().enumerate() {
            for &b in &ends[i + 1..] {
                let Some(turn) = turn_deg(lines, a, b, sided) else {
                    continue;
                };
                if best.is_none_or(|(t, ..)| turn < t - EPS) {
                    best = Some((turn, a, b));
                }
            }
        }
        if let Some((turn, a, b)) = best
            && (ends.len() == 2 || turn < MAX_TURN_DEG - EPS)
        {
            partner[a] = Some(b);
            partner[b] = Some(a);
        }
    }
    partner
}

/// The turn at the joint of ends `a` and `b` in degrees, or `None` when the pair may not join:
/// the ends of one object that has too few points to close, or a sided line that would have
/// to be reversed. The turn is measured between the last segment of the first piece and the
/// first of the second, after any reversal.
fn turn_deg(lines: &[&[MapCoord]], a: End, b: End, sided: bool) -> Option<f64> {
    if a / 2 == b / 2 && lines[a / 2].len() < 3 {
        return None;
    }
    // The first piece leaves through the end `x`, the second enters through `y`.
    let orders: [(End, End); 2] = [(a, b), (b, a)];
    let mut best: Option<f64> = None;
    for (x, y) in orders {
        if sided && (x % 2 == 0 || y % 2 == 1) {
            continue;
        }
        let first = oriented(lines[x / 2], x % 2 == 0);
        let second = oriented(lines[y / 2], y % 2 == 1);
        let turn = angle_deg(direction_at_end(&first), direction_at_start(&second));
        if best.is_none_or(|t| turn < t) {
            best = Some(turn);
        }
    }
    best
}

fn oriented(line: &[MapCoord], reversed: bool) -> Vec<MapCoord> {
    let mut v = line.to_vec();
    if reversed {
        v.reverse();
    }
    v
}

/// Unit direction of the last non-empty segment, `None` for a line without extent.
fn direction_at_end(line: &[MapCoord]) -> Option<(f64, f64)> {
    (1..line.len())
        .rev()
        .find_map(|i| unit(line[i - 1], line[i]))
}

fn direction_at_start(line: &[MapCoord]) -> Option<(f64, f64)> {
    (1..line.len()).find_map(|i| unit(line[i - 1], line[i]))
}

fn unit(a: MapCoord, b: MapCoord) -> Option<(f64, f64)> {
    let (dx, dy) = (f64::from(b.x - a.x), f64::from(b.y - a.y));
    let l = dx.hypot(dy);
    (l > 1e-9).then(|| (dx / l, dy / l))
}

fn angle_deg(a: Option<(f64, f64)>, b: Option<(f64, f64)>) -> f64 {
    match (a, b) {
        (Some(a), Some(b)) => (a.0 * b.0 + a.1 * b.1).clamp(-1.0, 1.0).acos().to_degrees(),
        _ => 0.0,
    }
}

/// Members in order, each with whether it is used reversed; `closed` for a ring of pieces.
struct Chain {
    members: Vec<(usize, bool)>,
    closed: bool,
}

fn chains(lines: &[&[MapCoord]], partner: &[Option<End>]) -> Vec<Chain> {
    let n = lines.len();
    let mut done = vec![false; n];
    let mut out = Vec::new();
    // Walks from the end `from` of member `k` (the end it is left through).
    let walk = |start: usize, reversed: bool, done: &mut Vec<bool>| -> Chain {
        let mut members = Vec::new();
        let (mut k, mut rev) = (start, reversed);
        let mut closed = false;
        loop {
            done[k] = true;
            members.push((k, rev));
            let tail = 2 * k + usize::from(!rev);
            let Some(next) = partner[tail] else { break };
            let (nk, enters_at_start) = (next / 2, next % 2 == 0);
            if nk == start {
                closed = true;
                break;
            }
            if done[nk] {
                break;
            }
            k = nk;
            rev = !enters_at_start;
        }
        Chain { members, closed }
    };
    // Chains start at a piece with a free end, preferably its start so a sided line stays as it
    // is; what is left are rings of pieces.
    for k in 0..n {
        if !done[k] && partner[2 * k].is_none() {
            out.push(walk(k, false, &mut done));
        }
    }
    for k in 0..n {
        if !done[k] && partner[2 * k + 1].is_none() {
            out.push(walk(k, true, &mut done));
        }
    }
    for k in 0..n {
        if !done[k] {
            out.push(walk(k, false, &mut done));
        }
    }
    out
}

fn build_chain(
    slots: &[Option<MapObject>],
    candidates: &[usize],
    chain: &Chain,
    joint: Joint,
) -> MapObject {
    let piece = |k: usize| slots[candidates[k]].as_ref().expect("candidate is present");
    let length = |o: &MapObject| -> f64 {
        o.coords
            .windows(2)
            .map(|w| f64::from(w[1].x - w[0].x).hypot(f64::from(w[1].y - w[0].y)))
            .sum()
    };
    let mut points: Vec<(i32, i32)> = Vec::new();
    for &(k, reversed) in &chain.members {
        let mut line: Vec<MapCoord> = piece(k).coords.clone();
        if reversed {
            line.reverse();
        }
        let skip = match (joint, points.last(), line.first()) {
            (Joint::Merged, Some(_), _) => 1,
            (Joint::Kept, Some(&last), Some(first)) => usize::from(last == (first.x, first.y)),
            _ => 0,
        };
        points.extend(line.iter().skip(skip).map(|c| (c.x, c.y)));
    }
    // The longest piece lends its tags; the first of equals.
    let mut main = chain.members[0].0;
    for &(k, _) in &chain.members {
        if length(piece(k)) > length(piece(main)) + EPS {
            main = k;
        }
    }
    let mut tags = piece(main).tags.clone();
    for &(k, _) in &chain.members {
        if k == main {
            continue;
        }
        let other = piece(k);
        let get = |key: &str| {
            other
                .tags
                .iter()
                .find(|(t, _)| t == key)
                .map(|(_, v)| v.as_str())
        };
        if let Some(id) = get("freemapper:source_id") {
            let also = match get("freemapper:source") {
                Some(source) => format!("{source}/{id}"),
                None => id.to_string(),
            };
            add_also(&mut tags, &also);
        }
        if let Some(older) = get("freemapper:also") {
            for also in older.split("; ") {
                add_also(&mut tags, also);
            }
        }
    }
    let coords = if chain.closed {
        // The last point lies on the first (or within `TOUCH_M` of it); the ring closes on the first.
        // Kept, it stays unless it is the first (`closed_ring` drops that one).
        if joint == Joint::Merged {
            points.pop();
        }
        closed_ring(&points)
    } else {
        points
            .into_iter()
            .map(|(x, y)| MapCoord::new(x, y))
            .collect()
    };
    MapObject {
        symbol_id: piece(chain.members[0].0).symbol_id,
        kind: MapObjectKind::Path,
        coords,
        tags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::{Crs, ProjectedPoint};
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};

    fn georef() -> Georeferencing {
        let centre = Crs::Utm33
            .inverse(ProjectedPoint {
                easting: 392_400.0,
                northing: 5_820_400.0,
            })
            .unwrap();
        Georeferencing::new(centre, 4000, "2026-11-08".parse().unwrap()).unwrap()
    }

    fn line(id: u32, pts: &[(i32, i32)], source_id: &str) -> MapObject {
        MapObject {
            symbol_id: id,
            kind: MapObjectKind::Path,
            coords: pts.iter().map(|&(x, y)| MapCoord::new(x, y)).collect(),
            tags: vec![
                ("freemapper:source".to_string(), "osm".to_string()),
                ("freemapper:source_id".to_string(), source_id.to_string()),
            ],
        }
    }

    fn xy(o: &MapObject) -> Vec<(i32, i32)> {
        o.coords.iter().map(|c| (c.x, c.y)).collect()
    }

    #[test]
    fn two_pieces_become_one() {
        let mut v = vec![
            line(1, &[(0, 0), (1000, 0), (2000, 0)], "a"),
            line(1, &[(2000, 0), (3000, 0), (4000, 0), (5000, 0)], "b"),
        ];
        assert_eq!(join_lines(&mut v, &[1], &[], &georef()), 1);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].coords.len(), 6);
        let also = v[0].tags.iter().find(|(k, _)| k == "freemapper:also");
        assert_eq!(also.map(|(_, v)| v.as_str()), Some("osm/a"));
    }

    #[test]
    fn a_keyed_join_names_the_pieces_that_went() {
        let keyed = |mut o: MapObject, key: &str| {
            o.tags.push(("k".to_string(), key.to_string()));
            o
        };
        let mut v = vec![
            keyed(line(1, &[(0, 0), (1000, 0)], "a"), "1"),
            keyed(line(1, &[(1000, 0), (5000, 0)], "b"), "2"),
            keyed(line(1, &[(5000, 0), (6000, 0)], "c"), "2"),
        ];
        let (n, absorbed) = join_lines_keyed(&mut v, &[1], &[], &georef(), Some("k"));
        assert_eq!(n, 2);
        assert_eq!(absorbed, vec![("1".to_string(), "2".to_string())]);
    }

    #[test]
    fn three_ends_join_the_straight_pair() {
        let mut v = vec![
            line(1, &[(-5000, 0), (0, 0)], "left"),
            line(1, &[(0, 0), (5000, 0)], "right"),
            line(1, &[(0, 0), (0, 5000)], "stem"),
        ];
        assert_eq!(join_lines(&mut v, &[1], &[], &georef()), 1);
        assert_eq!(v.len(), 2);
        assert_eq!(xy(&v[0]), vec![(-5000, 0), (0, 0), (5000, 0)]);
        assert_eq!(xy(&v[1]), vec![(0, 0), (0, 5000)]);
    }

    #[test]
    fn a_fork_of_60_degrees_joins_nothing() {
        // Three branches 120 degrees apart: the straightest pair turns 60 degrees.
        let mut fork = vec![
            line(1, &[(0, 5000), (0, 0)], "stem"),
            line(1, &[(0, 0), (4330, -2500)], "a"),
            line(1, &[(0, 0), (-4330, -2500)], "b"),
        ];
        assert_eq!(join_lines(&mut fork, &[1], &[], &georef()), 0);
        assert_eq!(fork.len(), 3);
    }

    #[test]
    fn different_symbols_do_not_join() {
        let mut v = vec![
            line(1, &[(0, 0), (1000, 0)], "a"),
            line(2, &[(1000, 0), (2000, 0)], "b"),
        ];
        assert_eq!(join_lines(&mut v, &[1, 2], &[], &georef()), 0);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn a_ring_of_pieces_is_closed() {
        let mut v = vec![
            line(1, &[(0, 0), (4000, 0), (4000, 4000)], "a"),
            line(1, &[(4000, 4000), (0, 4000), (0, 0)], "b"),
        ];
        assert_eq!(join_lines(&mut v, &[1], &[], &georef()), 1);
        assert_eq!(v.len(), 1);
        let last = v[0].coords.last().unwrap();
        assert_eq!(last.flags, FLAG_HOLE | FLAG_CLOSE);
        assert_eq!(xy(&v[0]).first(), xy(&v[0]).last());
        assert_eq!(v[0].coords.len(), 5);
    }

    #[test]
    fn a_sided_line_is_not_reversed() {
        let mut head_to_head = vec![
            line(104, &[(0, 0), (1000, 0)], "a"),
            line(104, &[(2000, 0), (1000, 0)], "b"),
        ];
        assert_eq!(join_lines(&mut head_to_head, &[104], &[104], &georef()), 0);
        assert_eq!(head_to_head.len(), 2);
        let mut tail_to_head = vec![
            line(104, &[(1000, 0), (2000, 0)], "b"),
            line(104, &[(0, 0), (1000, 0)], "a"),
        ];
        assert_eq!(join_lines(&mut tail_to_head, &[104], &[104], &georef()), 1);
        assert_eq!(xy(&tail_to_head[0]), vec![(0, 0), (1000, 0), (2000, 0)]);
    }

    #[test]
    fn an_unsided_line_may_be_reversed() {
        let mut v = vec![
            line(1, &[(0, 0), (1000, 0)], "a"),
            line(1, &[(2000, 0), (1000, 0)], "b"),
        ];
        assert_eq!(join_lines(&mut v, &[1], &[], &georef()), 1);
        assert_eq!(xy(&v[0]), vec![(0, 0), (1000, 0), (2000, 0)]);
    }

    #[test]
    fn an_object_with_flags_is_left_alone() {
        let mut flagged = line(1, &[(0, 0), (1000, 0)], "a");
        flagged.coords[0].flags = crate::omap::coord::FLAG_DASH;
        let mut v = vec![flagged, line(1, &[(1000, 0), (2000, 0)], "b")];
        assert_eq!(join_lines(&mut v, &[1], &[], &georef()), 0);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn ends_one_metre_apart_do_not_join() {
        // 1 m on the ground is 0.25 mm on paper at 1:4000.
        let mut v = vec![
            line(1, &[(0, 0), (1000, 0)], "a"),
            line(1, &[(1250, 0), (2000, 0)], "b"),
        ];
        assert_eq!(join_lines(&mut v, &[1], &[], &georef()), 0);
    }
}
