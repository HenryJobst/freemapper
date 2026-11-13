//! Point symbols keep the norm's gap to walls and fences (ISSprOM 2019-2 §3.3).
//!
//! ADR 0066 gives two point symbols a gap of 0.15 mm between their outer edges. The same gap
//! holds between a point symbol and a wall or fence line: from the symbol's outer edge to the
//! edge of the line, which on the tick side of a fence is the end of its ticks. A point that
//! comes closer is moved away from the line, straight across it, to just that distance; it keeps
//! its side, and the spot also keeps the gap to every other point symbol: if the first spot is
//! taken by another symbol, the nearest free one along the line or further away is used; if it is
//! taken by the same symbol the moved point is reduced (left out) and the other names it in
//! `freemapper:generalized`. A moved point carries `freemapper:moved`. It runs after the ticks are turned
//! (`build/ticks.rs`), because the tick side needs more room.

use crate::build::points::{HIDDEN_KEY, POINT_GAP_MM, POINT_SIZES_MM, point_size_mm};
use crate::measure::{SymbolMm, SymbolScale};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;

/// The lines and how far their drawing reaches from the centre line on paper at 1:4000 (mm):
/// to the left and to the right of the way they run. The ticks of the fence and of the retained
/// wall (`build/ticks.rs`) sit on the right.
const LINES: [(&str, SymbolMm, SymbolMm); 3] = [
    // Impassable wall: 0.4 mm wide.
    ("515", SymbolMm(0.20), SymbolMm(0.20)),
    // Impassable fence: 0.4 mm wide, ticks 0.65 mm.
    ("518", SymbolMm(0.20), SymbolMm(0.65)),
    // Passable retained wall: 0.21 mm wide, ticks 0.3 mm.
    ("513.2", SymbolMm(0.105), SymbolMm(0.30)),
];

type P = (f64, f64);

/// A distance this much (map units, 0.002 mm) under its limit still counts as kept. A moved point
/// sits exactly at the limit, where the last digits of a sum differ between systems.
const TOLERANCE: f64 = 2.0;
/// How many steps of half a symbol size the search for a free spot goes along the line and away
/// from it.
const SEARCH_STEPS: i32 = 10;
/// How often a point is moved again when the move puts it too close to another line.
const MAX_MOVES: usize = 4;
/// The `freemapper:moved` value of a point moved off a wall or fence.
pub const MOVED: &str = "gap to a wall or fence (ISSprOM 3.3)";

/// Moves the point symbols among `lists` that come closer than the gap to a wall or fence line
/// among them. Returns how many moved.
pub fn move_points_off_barriers(
    lists: &mut [&mut Vec<MapObject>],
    kind: MapKind,
    scale: u32,
    symbol_set: &SymbolSet,
) -> usize {
    let scale = SymbolScale::new(kind, scale);
    // The reach of each line symbol in map units (1/1000 mm).
    let reach: Vec<(u32, f64, f64)> = LINES
        .iter()
        .filter_map(|&(code, left, right)| {
            Some((
                symbol_set.id_for_code(code)?,
                left.map_units(scale),
                right.map_units(scale),
            ))
        })
        .collect();
    let radius: std::collections::HashMap<u32, f64> = POINT_SIZES_MM
        .iter()
        .filter_map(|&(code, _, _)| {
            Some((
                symbol_set.id_for_code(code)?,
                point_size_mm(code, scale)?.map_units() / 2.0,
            ))
        })
        .collect();
    let gap = POINT_GAP_MM.map_units(scale);
    let xy = |c: &crate::omap::MapCoord| (f64::from(c.x), f64::from(c.y));

    // (left, right, a, b) of every segment of a wall or fence.
    let mut segments: Vec<Segment> = Vec::new();
    for list in lists.iter() {
        for o in list.iter() {
            let Some(&(_, left, right)) = reach.iter().find(|(id, _, _)| *id == o.symbol_id) else {
                continue;
            };
            if matches!(o.kind, MapObjectKind::Path) {
                for w in o.coords.windows(2) {
                    segments.push(Segment {
                        left,
                        right,
                        a: xy(&w[0]),
                        b: xy(&w[1]),
                    });
                }
            }
        }
    }
    if segments.is_empty() {
        return 0;
    }
    // Every point symbol with a size: (list, index, position, radius, symbol).
    let mut points: Vec<(usize, usize, P, f64, u32)> = Vec::new();
    for (l, list) in lists.iter().enumerate() {
        for (i, o) in list.iter().enumerate() {
            if let (MapObjectKind::Point { .. }, Some(c), Some(&r)) =
                (&o.kind, o.coords.first(), radius.get(&o.symbol_id))
                && !o.tags.iter().any(|(k, _)| k == HIDDEN_KEY)
            {
                points.push((l, i, xy(c), r, o.symbol_id));
            }
        }
    }
    let mut moved = 0;
    let mut removed = vec![false; points.len()];
    let mut named: std::collections::HashMap<(usize, usize), Vec<String>> =
        std::collections::HashMap::new();
    for n in 0..points.len() {
        let (l, i, from, r, symbol) = points[n];
        let mut at = from;
        for _ in 0..MAX_MOVES {
            match away_from_lines(at, r + gap, &segments) {
                Some(next) => at = next,
                None => break,
            }
        }
        if at == from {
            continue;
        }
        // The point symbols standing too close to `q`.
        let crowd = |q: P| -> Vec<usize> {
            points
                .iter()
                .enumerate()
                .filter(|&(m, &(.., other, r2, _))| {
                    m != n
                        && !removed[m]
                        && (q.0 - other.0).hypot(q.1 - other.1) < r + r2 + gap - TOLERANCE
                })
                .map(|(m, _)| m)
                .collect()
        };
        let free = |q: P| away_from_lines(q, r + gap, &segments).is_none() && crowd(q).is_empty();
        if !free(at) {
            // The same symbol too close: one of them goes, the moved one. Another symbol: the
            // nearest spot that keeps both gaps, along the line and away from it.
            if let Some(&same) = crowd(at).iter().find(|&&m| points[m].4 == symbol) {
                removed[n] = true;
                named
                    .entry((points[same].0, points[same].1))
                    .or_default()
                    .push(name(&lists[l][i]));
                continue;
            }
            let (dx, dy) = (at.0 - from.0, at.1 - from.1);
            let length = dx.hypot(dy).max(f64::EPSILON);
            let (nx, ny) = (dx / length, dy / length);
            let (tx, ty) = (-ny, nx);
            let step = (r + gap) / 2.0;
            let mut offsets: Vec<(f64, f64)> = (-SEARCH_STEPS..=SEARCH_STEPS)
                .flat_map(|s| (0..=SEARCH_STEPS).map(move |a| (f64::from(s), f64::from(a))))
                .collect();
            offsets.sort_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)));
            if let Some(q) = offsets
                .into_iter()
                .map(|(s, a)| {
                    (
                        at.0 + (tx * s + nx * a) * step,
                        at.1 + (ty * s + ny * a) * step,
                    )
                })
                .find(|&q| free(q))
            {
                at = q;
            }
        }
        points[n].2 = at;
        let o = &mut lists[l][i];
        o.coords[0].x = at.0.round() as i32;
        o.coords[0].y = at.1.round() as i32;
        o.tags
            .push(("freemapper:moved".to_string(), MOVED.to_string()));
        moved += 1;
    }
    for ((l, i), mut names) in named {
        names.sort_unstable();
        names.dedup();
        let o = &mut lists[l][i];
        let joined = names.join("; ");
        match o
            .tags
            .iter_mut()
            .find(|(k, _)| k == "freemapper:generalized")
        {
            Some((_, v)) => {
                v.push_str("; ");
                v.push_str(&joined);
            }
            None => o.tags.push(("freemapper:generalized".to_string(), joined)),
        }
    }
    // Drop the removed points, from the back so the indices stay valid.
    let mut gone: Vec<(usize, usize)> = points
        .iter()
        .zip(&removed)
        .filter(|(_, r)| **r)
        .map(|(p, _)| (p.0, p.1))
        .collect();
    gone.sort_unstable_by(|a, b| b.cmp(a));
    for (l, i) in gone {
        lists[l].remove(i);
    }
    moved
}

/// `<source>/<source_id>` of an object.
fn name(o: &MapObject) -> String {
    let tag = |key: &str| {
        o.tags
            .iter()
            .find(|(k, _)| k == key)
            .map_or("", |(_, v)| v.as_str())
    };
    format!(
        "{}/{}",
        tag("freemapper:source"),
        tag("freemapper:source_id")
    )
}

/// A segment of a wall or fence and how far its drawing reaches to either side.
struct Segment {
    left: f64,
    right: f64,
    a: P,
    b: P,
}

/// Where the point `p` goes when the drawing of the nearest segment, with the point's radius and
/// the gap (`room`), reaches it: along the normal from the segment, to the clearance of its
/// side. `None` when no line is too close.
fn away_from_lines(p: P, room: f64, segments: &[Segment]) -> Option<P> {
    let mut best: Option<(f64, P)> = None;
    for s in segments {
        let (dx, dy) = (s.b.0 - s.a.0, s.b.1 - s.a.1);
        let length2 = dx * dx + dy * dy;
        let t = if length2 > 0.0 {
            (((p.0 - s.a.0) * dx + (p.1 - s.a.1) * dy) / length2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let foot = (s.a.0 + t * dx, s.a.1 + t * dy);
        let d = (p.0 - foot.0).hypot(p.1 - foot.1);
        // On paper (y down) the right of the way is where the cross product is positive.
        let cross = dx * (p.1 - s.a.1) - dy * (p.0 - s.a.0);
        let on_right = cross > 0.0;
        let clearance = room + if on_right { s.right } else { s.left };
        if d >= clearance - TOLERANCE || best.is_some_and(|(b, _)| d >= b) {
            continue;
        }
        // The direction from the line to the point; on the line itself, the side without ticks.
        let length = length2.sqrt().max(f64::EPSILON);
        let (ux, uy) = if d > 1e-6 {
            ((p.0 - foot.0) / d, (p.1 - foot.1) / d)
        } else {
            (dy / length, -dx / length)
        };
        best = Some((d, (foot.0 + ux * clearance, foot.1 + uy * clearance)));
    }
    best.map(|(_, to)| to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};

    fn fence(f: &Fixture) -> MapObject {
        MapObject {
            symbol_id: f.id("518"),
            kind: MapObjectKind::Path,
            coords: vec![f.at((0.0, 0.0)), f.at((40.0, 0.0))],
            tags: Vec::new(),
        }
    }

    fn tree(f: &Fixture, at: (f64, f64)) -> MapObject {
        MapObject {
            symbol_id: f.id("417"),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![f.at(at)],
            tags: vec![("freemapper:source_id".to_string(), "t".to_string())],
        }
    }

    fn run(f: &Fixture, objects: Vec<MapObject>) -> (usize, Vec<MapObject>) {
        let mut objects = objects;
        let n =
            move_points_off_barriers(&mut [&mut objects], f.kind, f.kind.default_scale(), &f.set);
        (n, objects)
    }

    /// The metres the point at the end of `objects` lies from the line y = 0 (north positive).
    fn north_of_the_line(f: &Fixture, objects: &[MapObject]) -> f64 {
        -f64::from(objects.last().unwrap().coords[0].y) / f.metre()
    }

    #[test]
    fn tree_touching_a_fence_moves_to_the_gap() {
        let f = fixture(MapKind::Sprint);
        // 417 is 1.4 mm (5.6 m) wide: its edge plus the gap needs 0.7 + 0.15 + 0.2 mm = 4.2 m
        // from a line on the side without ticks.
        let (n, objects) = run(&f, vec![fence(&f), tree(&f, (20.0, 3.0))]);
        assert_eq!(n, 1);
        assert_eq!(objects.len(), 2, "nothing goes");
        let d = north_of_the_line(&f, &objects);
        assert!((d - 4.2).abs() < 0.05, "{d}");
        assert!(objects[1].tags.iter().any(|(k, _)| k == "freemapper:moved"));
    }

    #[test]
    fn a_moved_point_keeps_the_gap_to_another_symbol() {
        let f = fixture(MapKind::Sprint);
        // A tree that would land on the spot of a tree already at the gap: it slides along the
        // fence to a spot that clears both.
        let mut neighbour = tree(&f, (20.0, 4.5));
        neighbour.symbol_id = f.id("418");
        let (n, objects) = run(&f, vec![fence(&f), neighbour, tree(&f, (21.0, 3.0))]);
        assert_eq!(n, 1);
        let (a, b) = (&objects[1].coords[0], &objects[2].coords[0]);
        let d = (f64::from(a.x - b.x)).hypot(f64::from(a.y - b.y)) / f.metre();
        // 417 is 5.6 m wide, 418 3.2 m, the gap 0.6 m: 2.8 + 1.6 + 0.6.
        assert!(d >= 5.0 - 0.05, "{d}");
        assert!(north_of_the_line(&f, &objects) >= 4.2 - 0.05);
    }

    #[test]
    fn a_moved_point_of_the_same_symbol_is_reduced() {
        let f = fixture(MapKind::Sprint);
        // The same symbol (417) on the spot: the moved one is left out, the other names it.
        let (_, kept) = run(
            &f,
            vec![fence(&f), tree(&f, (20.0, 4.5)), tree(&f, (21.0, 3.0))],
        );
        assert_eq!(kept.len(), 2);
        assert!(
            kept[1]
                .tags
                .iter()
                .any(|(k, _)| k == "freemapper:generalized")
        );
    }

    #[test]
    fn tree_with_room_stays() {
        let f = fixture(MapKind::Sprint);
        let (n, objects) = run(&f, vec![fence(&f), tree(&f, (20.0, 4.6))]);
        assert_eq!(n, 0);
        assert!((north_of_the_line(&f, &objects) - 4.6).abs() < 0.01);
    }

    #[test]
    fn the_tick_side_needs_more_room() {
        let f = fixture(MapKind::Sprint);
        // Running east, the right of the line (on paper) is south. The ticks reach 0.65 mm
        // (2.6 m): 0.7 + 0.15 + 0.65 mm = 6.0 m.
        let (n, south) = run(&f, vec![fence(&f), tree(&f, (20.0, -5.0))]);
        assert_eq!(n, 1);
        assert!((north_of_the_line(&f, &south) + 6.0).abs() < 0.05);
        let (n, north) = run(&f, vec![fence(&f), tree(&f, (20.0, 5.0))]);
        assert_eq!(n, 0);
        assert!((north_of_the_line(&f, &north) - 5.0).abs() < 0.01);
    }
}
