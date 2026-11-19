// SPDX-License-Identifier: AGPL-3.0-or-later

//! Step 7 of the training level: no point symbol is drawn on a way; a point that comes too
//! close to a symbol it keeps apart from is moved off to the nearest spot that frees it (spec
//! 2026-11-14, "Trees", for every point symbol).

use std::collections::BTreeMap;

use geo::{BooleanOps as _, BoundingRect as _, Intersects as _};
use geo_types::{Coord, MultiPolygon, Point, Rect};

use super::olive::{OLIVE_CODE, in_mm};
use crate::build::barrier_gaps::MOVED_KEY;
use crate::build::conflicts::{Class, Classes, Footprints, Relation, TOLERANCE_MM, gap, relation};
use crate::build::geometry::object_polygon;
use crate::georef::Georeferencing;
use crate::measure::PaperMm;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;

/// A shift on the training map moves an object by at most this (spec: 0.5 mm on paper).
pub const MAX_SHIFT_MM: PaperMm = PaperMm(0.5);
/// A point moves by at most this. A point symbol stands for an object beside the way within its
/// own size; the bound covers the symbol's radius, the gap and half the widest way it is moved
/// off at every scale freemapper writes (at most 1.24 mm: a 417 centred on a 505 at ISOM
/// 1:10 000), and the point is still moved to the nearest spot that frees it.
pub const MAX_POINT_SHIFT_MM: PaperMm = PaperMm(1.5);
/// The `freemapper:moved` value of a point moved off on the training map.
pub const MOVED: &str = "gap to the symbols round it (training map)";

/// Whether a point keeps the gap to an object of `class`: to every class a point keeps apart
/// from ([`relation`]), other point symbols among them. Paved areas are not: a tree on a street or
/// square stays (the norm knows trees on paved areas).
fn is_obstacle(class: Class) -> bool {
    relation(Class::Point, class) == Relation::Apart
}
/// The search for a free spot goes out in steps of this (mm on paper).
const STEP_MM: f64 = 0.02;
/// The directions of the ring of the search: every 22.5°, the unit vectors written out so every
/// system searches alike.
const RING: [(f64, f64); 16] = [
    (1.0, 0.0),
    (COS_22_5, SIN_22_5),
    (SQRT_HALF, SQRT_HALF),
    (SIN_22_5, COS_22_5),
    (0.0, 1.0),
    (-SIN_22_5, COS_22_5),
    (-SQRT_HALF, SQRT_HALF),
    (-COS_22_5, SIN_22_5),
    (-1.0, 0.0),
    (-COS_22_5, -SIN_22_5),
    (-SQRT_HALF, -SQRT_HALF),
    (-SIN_22_5, -COS_22_5),
    (0.0, -1.0),
    (SIN_22_5, -COS_22_5),
    (SQRT_HALF, -SQRT_HALF),
    (COS_22_5, -SIN_22_5),
];
const COS_22_5: f64 = 0.923_879_532_511_286_7;
const SIN_22_5: f64 = 0.382_683_432_365_089_8;
const SQRT_HALF: f64 = std::f64::consts::FRAC_1_SQRT_2;
/// The footprints round a point are cut to its reach and this much more (mm on paper).
const MARGIN_MM: f64 = 1.0;

/// A point as the step sees it: a disc round its centre (mm on paper).
#[derive(Clone, Copy)]
struct Disc {
    centre: Coord<f64>,
    radius: f64,
}

/// What a point may not come near: the footprints it keeps the gap to, and the olive green
/// (520) its symbol may not overlap. Each with its frame, mm on paper.
struct Obstacles<'a> {
    walls: Vec<(Rect<f64>, &'a MultiPolygon<f64>)>,
    olive: Vec<(Rect<f64>, MultiPolygon<f64>)>,
}

/// Step 7: a point symbol (an object of [`Class::Point`] drawn as a point) that keeps less than
/// the gap to the footprint of an object it keeps apart from ([`is_obstacle`]) or to another
/// point that wins against it is moved to the nearest spot that keeps the gap to all of them and
/// whose symbol does not overlap olive green, within [`MAX_POINT_SHIFT_MM`] ([`spot`]), and
/// tagged `freemapper:moved`. Of two points the larger symbol wins, of symbols as large the
/// earlier object: the points go by the extent of their symbol, the largest first, then by
/// index, and each keeps the gap to the points before it where they now stand. A point that
/// finds no spot stays where it is, for the give-way step to drop, and is no obstacle to the
/// points after it. The symbol of a point is the disc round its centre that holds its
/// footprint. Returns how many points moved.
pub(super) fn move_points(
    objects: &mut [MapObject],
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let is_point = |o: &MapObject| {
        matches!(o.kind, MapObjectKind::Point { .. })
            && !o.coords.is_empty()
            && classes.class(o.symbol_id) == Class::Point
    };
    let chosen: Vec<usize> = (0..objects.len())
        .filter(|&i| {
            let o = &objects[i];
            is_point(o) || is_obstacle(classes.class(o.symbol_id))
        })
        .collect();
    let points: Vec<usize> = (0..chosen.len())
        .filter(|&k| is_point(&objects[chosen[k]]))
        .collect();
    if points.is_empty() {
        return 0;
    }
    let extent = extents(
        points.iter().map(|&k| objects[chosen[k]].symbol_id),
        symbol_set,
    );
    let mut order: Vec<(f64, usize, usize)> = points
        .iter()
        .filter_map(|&k| {
            let i = chosen[k];
            Some((*extent.get(&objects[i].symbol_id)?, i, k))
        })
        .collect();
    order.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let refs: Vec<&MapObject> = chosen.iter().map(|&i| &objects[i]).collect();
    let footprints = Footprints::of(&refs, symbol_set);
    let walls: Vec<(Rect<f64>, &MultiPolygon<f64>)> = (0..chosen.len())
        .filter(|&k| !is_point(refs[k]))
        .filter_map(|k| {
            let footprint = footprints.footprint(k);
            Some((footprint.bounding_rect()?, footprint))
        })
        .collect();
    let olive_id = symbol_set.id_for_code(OLIVE_CODE);
    let olive: Vec<(Rect<f64>, MultiPolygon<f64>)> = objects
        .iter()
        .filter(|o| olive_id == Some(o.symbol_id))
        .filter_map(object_polygon)
        .map(|p| in_mm(&p))
        .filter_map(|p| Some((p.bounding_rect()?, MultiPolygon(vec![p]))))
        .collect();
    let obstacles = Obstacles { walls, olive };
    let gap = gap(kind, georef.scale_denominator).0;
    let mut placed: Vec<Disc> = Vec::new();
    let mut shifts: Vec<(usize, (i32, i32))> = Vec::new();
    for (_, i, k) in order {
        let at = objects[i].coords[0];
        let centre = Coord {
            x: f64::from(at.x) / 1000.0,
            y: f64::from(at.y) / 1000.0,
        };
        let Some(radius) = radius(footprints.footprint(k), centre) else {
            continue;
        };
        let point = Disc { centre, radius };
        let Some(offset) = spot(point, &obstacles, &placed, gap) else {
            continue;
        };
        if offset != (0, 0) {
            shifts.push((i, offset));
        }
        placed.push(Disc {
            centre: Coord {
                x: f64::from(at.x + offset.0) / 1000.0,
                y: f64::from(at.y + offset.1) / 1000.0,
            },
            radius,
        });
    }
    for &(i, (dx, dy)) in &shifts {
        let object = &mut objects[i];
        object.coords[0].x += dx;
        object.coords[0].y += dy;
        match object.tags.iter_mut().find(|(k, _)| k == MOVED_KEY) {
            Some((_, v)) => {
                v.push_str("; ");
                v.push_str(MOVED);
            }
            None => object.tags.push((MOVED_KEY.to_string(), MOVED.to_string())),
        }
    }
    shifts.len()
}

/// The extent of each point symbol of `symbol_ids`: the radius of its symbol drawn unturned at
/// the origin (mm on paper), the same for every object of the symbol; none for a symbol that
/// draws nothing.
fn extents(symbol_ids: impl Iterator<Item = u32>, symbol_set: &SymbolSet) -> BTreeMap<u32, f64> {
    let mut ids: Vec<u32> = symbol_ids.collect();
    ids.sort_unstable();
    ids.dedup();
    let samples: Vec<MapObject> = ids
        .iter()
        .map(|&symbol_id| MapObject {
            symbol_id,
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![MapCoord::new(0, 0)],
            tags: Vec::new(),
        })
        .collect();
    let refs: Vec<&MapObject> = samples.iter().collect();
    let footprints = Footprints::of(&refs, symbol_set);
    ids.iter()
        .enumerate()
        .filter_map(|(k, &id)| Some((id, radius(footprints.footprint(k), Coord::zero())?)))
        .collect()
}

/// The radius of the disc round `centre` that holds `footprint` (mm on paper); `None` when the
/// symbol draws nothing.
fn radius(footprint: &MultiPolygon<f64>, centre: Coord<f64>) -> Option<f64> {
    footprint
        .iter()
        .flat_map(|p| p.exterior().coords())
        .map(|c| distance(*c, centre))
        .reduce(f64::max)
}

/// Where `point` goes, as an offset in map units: `(0, 0)` when it keeps the gap to every wall
/// and placed point, `None` when no spot within [`MAX_POINT_SHIFT_MM`] does. The search tries
/// spots in a fixed order and takes the first that keeps the gap to all and whose symbol does
/// not overlap the olive green: out in steps of [`STEP_MM`], at each distance first straight
/// away from the nearest offender (the one the point reaches deepest into), then the directions
/// of [`RING`]. A spot keeps the gap when its clearance is at least the gap; the point needs no
/// move while it is above the gap less [`TOLERANCE_MM`]. One set of obstacles serves both.
fn spot(point: Disc, obstacles: &Obstacles, placed: &[Disc], gap: f64) -> Option<(i32, i32)> {
    let reach = point.radius + gap + MAX_POINT_SHIFT_MM.0 + MARGIN_MM;
    let around = Rect::new(
        Coord {
            x: point.centre.x - reach,
            y: point.centre.y - reach,
        },
        Coord {
            x: point.centre.x + reach,
            y: point.centre.y + reach,
        },
    );
    let near_walls = near(
        obstacles.walls.iter().map(|(frame, zone)| (frame, *zone)),
        &around,
    );
    let near_points: Vec<Disc> = placed
        .iter()
        .copied()
        .filter(|other| around.intersects(&Point(other.centre)))
        .collect();
    // The nearest offender at the centre and the way straight away from it. Of two as deep (a
    // point on the centre line of a way is as deep in both sides of it) the first in the order
    // of the footprints and their edges wins.
    let mut deepest: Option<(f64, Option<Coord<f64>>)> = None;
    let mut offend = |clearance: f64, away: Option<Coord<f64>>| {
        if clearance < gap - TOLERANCE_MM && deepest.is_none_or(|(c, _)| clearance < c) {
            deepest = Some((clearance, away));
        }
    };
    for footprint in &near_walls {
        if let Some((clearance, edge, inside)) = wall_clearance(point, footprint) {
            let away = if inside {
                edge - point.centre
            } else {
                point.centre - edge
            };
            offend(clearance, unit(away));
        }
    }
    for other in &near_points {
        offend(
            distance(point.centre, other.centre) - other.radius - point.radius,
            unit(point.centre - other.centre),
        );
    }
    let Some((_, straight)) = deepest else {
        return Some((0, 0));
    };
    let clip = |zones: Vec<&MultiPolygon<f64>>| -> Vec<MultiPolygon<f64>> {
        zones
            .into_iter()
            .map(|zone| zone.intersection(&around.to_polygon()))
            .filter(|zone| !zone.0.is_empty())
            .collect()
    };
    let local = clip(near_walls);
    let olive = clip(near(
        obstacles.olive.iter().map(|(frame, zone)| (frame, zone)),
        &around,
    ));
    let free = |offset: (i32, i32)| -> bool {
        let moved = Disc {
            centre: Coord {
                x: point.centre.x + f64::from(offset.0) / 1000.0,
                y: point.centre.y + f64::from(offset.1) / 1000.0,
            },
            radius: point.radius,
        };
        local
            .iter()
            .all(|footprint| wall_clearance(moved, footprint).is_none_or(|(c, ..)| c >= gap))
            && olive
                .iter()
                .all(|zone| wall_clearance(moved, zone).is_none_or(|(c, ..)| c >= 0.0))
            && near_points.iter().all(|other| {
                distance(moved.centre, other.centre) - other.radius - moved.radius >= gap
            })
    };
    candidates(straight).find(|&offset| free(offset))
}

/// The zones of `zones` (each with its frame) whose frame meets `around`.
fn near<'a>(
    zones: impl Iterator<Item = (&'a Rect<f64>, &'a MultiPolygon<f64>)>,
    around: &Rect<f64>,
) -> Vec<&'a MultiPolygon<f64>> {
    zones
        .filter(|(frame, _)| frame.intersects(around))
        .map(|(_, zone)| zone)
        .collect()
}

/// The offsets the search tries, in map units, in its order: distance by distance, at each
/// along `straight` first, then round the ring; none longer than [`MAX_POINT_SHIFT_MM`].
fn candidates(straight: Option<Coord<f64>>) -> impl Iterator<Item = (i32, i32)> {
    let steps = (MAX_POINT_SHIFT_MM.0 / STEP_MM).round() as i32;
    let along = |(x, y): (f64, f64), k: i32| -> (i32, i32) {
        let d = f64::from(k) * STEP_MM * 1000.0;
        ((x * d).round() as i32, (y * d).round() as i32)
    };
    let limit = (MAX_POINT_SHIFT_MM.0 + TOLERANCE_MM) * 1000.0;
    let directions: Vec<(f64, f64)> = straight
        .map(|u| (u.x, u.y))
        .into_iter()
        .chain(RING)
        .collect();
    (1..=steps)
        .flat_map(move |k| directions.clone().into_iter().map(move |u| along(u, k)))
        .filter(move |&(x, y)| {
            let (x, y) = (f64::from(x), f64::from(y));
            (x * x + y * y).sqrt() <= limit
        })
}

/// The clearance between the symbol of `point` and `footprint` (mm on paper, negative where they
/// overlap), the point of the footprint's edge nearest the centre, and whether the centre lies
/// inside; `None` for an empty footprint.
fn wall_clearance(point: Disc, footprint: &MultiPolygon<f64>) -> Option<(f64, Coord<f64>, bool)> {
    let mut nearest: Option<(f64, Coord<f64>)> = None;
    for polygon in footprint {
        for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
            for line in ring.lines() {
                let foot = foot_on(point.centre, line.start, line.end);
                let d = distance(point.centre, foot);
                if nearest.is_none_or(|(n, _)| d < n) {
                    nearest = Some((d, foot));
                }
            }
        }
    }
    let (d, edge) = nearest?;
    let inside = footprint.intersects(&Point(point.centre));
    let signed = if inside { -d } else { d };
    Some((signed - point.radius, edge, inside))
}

/// The point of the segment from `a` to `b` nearest `p`.
fn foot_on(p: Coord<f64>, a: Coord<f64>, b: Coord<f64>) -> Coord<f64> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length2 = dx * dx + dy * dy;
    if length2 <= 0.0 {
        return a;
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / length2).clamp(0.0, 1.0);
    Coord {
        x: a.x + t * dx,
        y: a.y + t * dy,
    }
}

/// The distance from `a` to `b`, by the square root alone, which every system rounds alike.
fn distance(a: Coord<f64>, b: Coord<f64>) -> f64 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    (dx * dx + dy * dy).sqrt()
}

/// `v` scaled to length 1; `None` when it has no direction.
fn unit(v: Coord<f64>) -> Option<Coord<f64>> {
    let length = (v.x * v.x + v.y * v.y).sqrt();
    (length > 1e-9).then(|| Coord {
        x: v.x / length,
        y: v.y / length,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::conflicts::{Relation, conflicts, relation};
    use crate::build::training::tests::{SCALE, area, georef_at, isom, line, of_code, point};
    use crate::build::training::{TrainingReport, train};

    /// The scale of the sprint test.
    const SPRINT: u32 = 4000;
    /// The step of the search for a free spot, as the tests allow it.
    const STEP: f64 = 0.02;

    fn run_at(
        set: &SymbolSet,
        kind: MapKind,
        scale: u32,
        objects: Vec<MapObject>,
    ) -> (Vec<MapObject>, TrainingReport) {
        train(objects, None, kind, &georef_at(scale), set)
    }

    fn run(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, TrainingReport) {
        run_at(set, MapKind::Forest, SCALE, objects)
    }

    /// The radius of the drawn symbol of `code`, mm on paper.
    fn radius_of(set: &SymbolSet, code: &str) -> f64 {
        let tree = point(set, code, (0.0, 0.0));
        let footprints = Footprints::of(&[&tree], set);
        footprints
            .footprint(0)
            .iter()
            .flat_map(|p| p.exterior().coords())
            .map(|c| (c.x * c.x + c.y * c.y).sqrt())
            .fold(0.0, f64::max)
    }

    /// How far the drawing of a line of `code` reaches from its centre line, mm on paper: the
    /// widest stroke, which on some paths is not the one along the centre line.
    fn reach_of(set: &SymbolSet, code: &str) -> f64 {
        let path = line(set, code, &[(0.0, 0.0), (10.0, 0.0)]);
        let footprints = Footprints::of(&[&path], set);
        let reach = footprints
            .footprint(0)
            .iter()
            .flat_map(|p| p.exterior().coords())
            .map(|c| c.y.abs())
            .fold(0.0, f64::max);
        assert!(reach >= footprints.half_width(0));
        reach
    }

    /// Paper position (mm) of a point object.
    fn mm(object: &MapObject) -> (f64, f64) {
        let c = object.coords[0];
        (f64::from(c.x) / 1000.0, f64::from(c.y) / 1000.0)
    }

    fn shift(from: (f64, f64), to: &MapObject) -> f64 {
        let (x, y) = mm(to);
        ((x - from.0) * (x - from.0) + (y - from.1) * (y - from.1)).sqrt()
    }

    /// The pairs that keep apart and come too close on `objects`.
    fn apart(set: &SymbolSet, kind: MapKind, scale: u32, objects: &[MapObject]) -> usize {
        let refs: Vec<&MapObject> = objects.iter().collect();
        conflicts(&Footprints::of(&refs, set), kind, scale)
            .iter()
            .filter(|c| relation(c.classes.0, c.classes.1) == Relation::Apart)
            .count()
    }

    fn is_moved(object: &MapObject) -> bool {
        object
            .tags
            .iter()
            .any(|(k, v)| k == "freemapper:moved" && v == MOVED)
    }

    /// Checks a point of `code` at `from` moved off by about `least` (mm): the shift is at least
    /// that and at most one step more, within the bound, the gaps kept, the point tagged.
    #[allow(clippy::too_many_arguments)]
    fn assert_moved(
        set: &SymbolSet,
        kind: MapKind,
        scale: u32,
        code: &str,
        from: (f64, f64),
        least: f64,
        out: &[MapObject],
        report: &TrainingReport,
    ) {
        let trees = of_code(set, out, code);
        assert_eq!(trees.len(), 1, "{code}: {out:?}");
        let moved = shift(from, trees[0]);
        assert!(
            moved >= least - TOLERANCE_MM && moved <= least + STEP + TOLERANCE_MM,
            "{code}: {moved} for {least}"
        );
        assert!(moved < MAX_POINT_SHIFT_MM.0, "{code}: {moved}");
        assert!(is_moved(trees[0]), "{code}: {:?}", trees[0].tags);
        assert_eq!(report.points_moved, 1, "{code}: {report:?}");
        assert_eq!(report.gave_way, 0, "{code}: {report:?}");
        assert_eq!(apart(set, kind, scale, out), 0, "{code}: {out:?}");
    }

    #[test]
    fn a_tree_on_a_path_moves_beside_it() {
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        for code in ["417", "418"] {
            let path = line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]);
            let (out, report) = run(&set, vec![path.clone(), point(&set, code, (20.0, 0.0))]);
            let least = radius_of(&set, code) + gap + reach_of(&set, "505");
            assert_moved(
                &set,
                MapKind::Forest,
                SCALE,
                code,
                (20.0, 0.0),
                least,
                &out,
                &report,
            );
            assert_eq!(of_code(&set, &out, "505"), vec![&path]);
        }
    }

    #[test]
    fn a_tree_touching_a_path_moves_a_little() {
        // 418 at 150 %: 0.375 + 0.225 + 0.1875 mm from the centre line, 0.6 mm there.
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]),
            point(&set, "418", (20.0, 0.6)),
        ];
        let (out, report) = run(&set, objects);
        let least = radius_of(&set, "418") + gap + reach_of(&set, "505") - 0.6;
        assert!((least - 0.19).abs() < 0.01, "{least}");
        assert_moved(
            &set,
            MapKind::Forest,
            SCALE,
            "418",
            (20.0, 0.6),
            least,
            &out,
            &report,
        );
        let (_, y) = mm(of_code(&set, &out, "418")[0]);
        assert!(y > 0.6, "it moves away from the path: {y}");
    }

    #[test]
    fn a_tree_between_two_close_paths_goes() {
        // A 417 needs 1.24 mm from each centre line; 2 mm apart there is no room between, and
        // beyond either path it would be more than 1.5 mm away.
        let set = isom();
        let paths = vec![
            line(&set, "505", &[(0.0, -1.0), (40.0, -1.0)]),
            line(&set, "505", &[(0.0, 1.0), (40.0, 1.0)]),
        ];
        let mut objects = paths.clone();
        objects.push(point(&set, "417", (20.0, 0.0)));
        let (out, report) = run(&set, objects);
        assert_eq!(out, paths);
        assert_eq!(report.points_moved, 0, "{report:?}");
        assert_eq!(report.gave_way, 1, "{report:?}");
    }

    #[test]
    fn a_tree_on_a_square_stays() {
        let set = isom();
        let objects = vec![
            area(&set, "501.5", (0.0, 0.0), (10.0, 10.0)),
            point(&set, "417", (5.0, 5.0)),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
    }

    #[test]
    fn a_tree_at_a_building_moves_off() {
        let set = isom();
        let building = area(&set, "521", (0.0, 0.0), (10.0, 10.0));
        let objects = vec![building.clone(), point(&set, "418", (10.3, 5.0))];
        let (out, report) = run(&set, objects);
        let tree = of_code(&set, &out, "418");
        assert_eq!(tree.len(), 1, "{out:?}");
        let (x, y) = mm(tree[0]);
        assert!(x > 10.3 && (y - 5.0).abs() < TOLERANCE_MM, "({x}, {y})");
        assert!(shift((10.3, 5.0), tree[0]) < MAX_POINT_SHIFT_MM.0);
        assert!(is_moved(tree[0]));
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(of_code(&set, &out, "521"), vec![&building]);
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn two_trees_touching_the_smaller_moves() {
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        // The 417 comes later, and still the 418 moves.
        let large = point(&set, "417", (20.9, 10.0));
        let (out, report) = run(&set, vec![point(&set, "418", (20.0, 10.0)), large.clone()]);
        assert_eq!(of_code(&set, &out, "417"), vec![&large]);
        let least = radius_of(&set, "417") + radius_of(&set, "418") + gap - 0.9;
        assert_moved(
            &set,
            MapKind::Forest,
            SCALE,
            "418",
            (20.0, 10.0),
            least,
            &out,
            &report,
        );
        // Of the same symbol the later one moves.
        let first = point(&set, "418", (20.0, 10.0));
        let (out, report) = run(&set, vec![first.clone(), point(&set, "418", (20.5, 10.0))]);
        assert_eq!(out[0], first, "{out:?}");
        assert!(is_moved(&out[1]), "{out:?}");
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn a_tree_that_keeps_the_gap_is_not_touched() {
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]),
            area(&set, "521", (0.0, 5.0), (10.0, 10.0)),
            point(&set, "418", (20.0, 1.0)),
            point(&set, "417", (5.0, 2.5)),
            point(&set, "418", (6.6, 2.5)),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
    }

    #[test]
    fn a_tree_on_a_sprint_path_moves_beside_it() {
        let set = SymbolSet::issprom_2019();
        let gap = gap(MapKind::Sprint, SPRINT).0;
        let objects = vec![
            line(&set, "501.6", &[(0.0, 0.0), (40.0, 0.0)]),
            point(&set, "418", (20.0, 0.0)),
        ];
        let (out, report) = run_at(&set, MapKind::Sprint, SPRINT, objects);
        let least = radius_of(&set, "418") + gap + reach_of(&set, "501.6");
        assert_moved(
            &set,
            MapKind::Sprint,
            SPRINT,
            "418",
            (20.0, 0.0),
            least,
            &out,
            &report,
        );
    }

    #[test]
    fn a_moved_tree_makes_room_for_the_next() {
        // Two 418 on one path: each moves off it, and the second keeps the gap to the first.
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]),
            point(&set, "418", (20.0, 0.0)),
            point(&set, "418", (20.5, 0.3)),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(of_code(&set, &out, "418").len(), 2, "{out:?}");
        assert_eq!(report.points_moved, 2, "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn a_tree_beside_a_path_and_a_fence_moves_between_them_or_goes() {
        // A 417 needs 1.24 mm from the centre line of the 505. With the fence 2.1 mm off the path
        // there is room between them; at 0.7 mm there is none, and across the path or beyond the
        // fence lies more than 1.5 mm away.
        let set = isom();
        let path = line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]);
        let wide = line(&set, "518", &[(0.0, 3.0), (40.0, 3.0)]);
        let narrow = line(&set, "518", &[(0.0, 1.6), (40.0, 1.6)]);
        let tree = point(&set, "417", (20.0, 0.9));
        let (out, report) = run(&set, vec![path.clone(), wide.clone(), tree.clone()]);
        let moved = of_code(&set, &out, "417");
        assert_eq!(moved.len(), 1, "{out:?}");
        let (_, y) = mm(moved[0]);
        assert!(y > 0.9 && y < 3.0, "between the path and the fence: {y}");
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
        let (out, report) = run(&set, vec![path.clone(), narrow.clone(), tree]);
        assert_eq!(out, vec![path, narrow]);
        assert_eq!(report.points_moved, 0, "{report:?}");
        assert_eq!(report.gave_way, 1, "{report:?}");
    }

    #[test]
    fn a_tree_moves_off_a_railway() {
        let set = isom();
        let railway = line(&set, "509", &[(0.0, 0.0), (40.0, 0.0)]);
        let (out, report) = run(&set, vec![railway.clone(), point(&set, "418", (20.0, 0.0))]);
        assert_eq!(of_code(&set, &out, "509"), vec![&railway]);
        let tree = of_code(&set, &out, "418");
        assert_eq!(tree.len(), 1, "{out:?}");
        assert!(is_moved(tree[0]));
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn a_tree_does_not_land_on_a_531() {
        // Straight off the path the 418 would land on the 531; it goes to a spot that keeps the
        // gap to both.
        let set = isom();
        let path = line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]);
        let mark = point(&set, "531", (20.0, 1.6));
        let objects = vec![path.clone(), mark.clone(), point(&set, "418", (20.0, 0.6))];
        let (out, report) = run(&set, objects);
        assert_eq!(of_code(&set, &out, "531"), vec![&mark]);
        let tree = of_code(&set, &out, "418");
        assert_eq!(tree.len(), 1, "{out:?}");
        assert!(is_moved(tree[0]));
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(report.gave_way, 0, "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn a_near_spot_round_the_ring_beats_a_far_one_straight_on() {
        // A 418 at the east face of a building; straight east a path runs past, so the first free
        // spot that way lies beyond the path, some 1.3 mm off. The path ends just north of the
        // tree, and a step to the south-east, past the building's corner, is free about 0.5 mm off.
        let set = isom();
        let objects = vec![
            area(&set, "521", (0.0, 0.0), (10.0, 10.0)),
            line(&set, "505", &[(10.9, 0.0), (10.9, 9.4)]),
            point(&set, "418", (10.3, 9.7)),
        ];
        let (out, report) = run(&set, objects);
        let tree = of_code(&set, &out, "418");
        assert_eq!(tree.len(), 1, "{out:?}");
        let moved = shift((10.3, 9.7), tree[0]);
        assert!(moved < 0.8, "{moved}: {:?}", mm(tree[0]));
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn a_monument_beside_a_path_moves() {
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        let path = line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]);
        let (out, report) = run(&set, vec![path.clone(), point(&set, "526", (20.0, 0.0))]);
        let least = radius_of(&set, "526") + gap + reach_of(&set, "505");
        assert_moved(
            &set,
            MapKind::Forest,
            SCALE,
            "526",
            (20.0, 0.0),
            least,
            &out,
            &report,
        );
        assert_eq!(of_code(&set, &out, "505"), vec![&path]);
    }

    #[test]
    fn landforms_and_springs_move_like_every_point() {
        // A knoll or a spring that give way would drop moves off the path instead.
        let set = isom();
        let path = line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]);
        for code in ["109", "110", "111", "112", "115", "312"] {
            let (out, report) = run(&set, vec![path.clone(), point(&set, code, (20.0, 0.3))]);
            let moved = of_code(&set, &out, code);
            assert_eq!(moved.len(), 1, "{code}: {out:?}");
            assert!(is_moved(moved[0]), "{code}: {out:?}");
            assert!(
                shift((20.0, 0.3), moved[0]) <= MAX_POINT_SHIFT_MM.0,
                "{code}"
            );
            assert_eq!(report.points_moved, 1, "{code}: {report:?}");
            assert_eq!(report.apart_left, 0.0, "{code}: {report:?}");
        }
    }

    #[test]
    fn a_fountain_at_a_building_moves() {
        let set = isom();
        let building = area(&set, "521", (0.0, 0.0), (10.0, 10.0));
        let objects = vec![building.clone(), point(&set, "311", (10.3, 5.0))];
        let (out, report) = run(&set, objects);
        let fountain = of_code(&set, &out, "311");
        assert_eq!(fountain.len(), 1, "{out:?}");
        let (x, y) = mm(fountain[0]);
        assert!(x > 10.3 && (y - 5.0).abs() < TOLERANCE_MM, "({x}, {y})");
        assert!(shift((10.3, 5.0), fountain[0]) < MAX_POINT_SHIFT_MM.0);
        assert!(is_moved(fountain[0]));
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(report.gave_way, 0, "{report:?}");
        assert_eq!(of_code(&set, &out, "521"), vec![&building]);
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn of_two_points_too_close_the_smaller_moves() {
        // A 418 and a 526 close to each other: the one with the smaller symbol moves, whichever
        // comes first.
        let set = isom();
        let (small, large) = if radius_of(&set, "418") < radius_of(&set, "526") {
            ("418", "526")
        } else {
            ("526", "418")
        };
        let fixed = point(&set, large, (20.5, 10.0));
        let (out, report) = run(&set, vec![point(&set, small, (20.0, 10.0)), fixed.clone()]);
        assert_eq!(of_code(&set, &out, large), vec![&fixed]);
        let moved = of_code(&set, &out, small);
        assert_eq!(moved.len(), 1, "{out:?}");
        assert!(is_moved(moved[0]), "{out:?}");
        assert_eq!(report.points_moved, 1, "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
        // Of two monuments the later one moves.
        let first = point(&set, "526", (20.0, 10.0));
        let (out, report) = run(&set, vec![first.clone(), point(&set, "526", (20.5, 10.0))]);
        assert_eq!(out[0], first, "{out:?}");
        assert!(is_moved(&out[1]), "{out:?}");
        assert_eq!((report.points_moved, report.gave_way), (1, 0), "{report:?}");
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }

    #[test]
    fn a_tree_does_not_move_into_olive_green() {
        // A 418 on a path, a little towards the olive green beside it: straight away from the
        // path it would land on the 520; it moves to the other side.
        let set = isom();
        let path = line(&set, "505", &[(0.0, 0.0), (40.0, 0.0)]);
        let olive = area(&set, "520", (0.0, 0.4), (40.0, 10.0));
        let objects = vec![path, olive, point(&set, "418", (20.0, 0.2))];
        let (out, report) = run(&set, objects);
        let radius = radius_of(&set, "418");
        for tree in of_code(&set, &out, "418") {
            let (_, y) = mm(tree);
            assert!(
                y + radius <= 0.4 + TOLERANCE_MM,
                "inside the olive green: {y}"
            );
            assert!(is_moved(tree), "{out:?}");
        }
        assert_eq!(
            report.points_moved + report.gave_way,
            1,
            "it moves or goes: {report:?}"
        );
        assert_eq!(apart(&set, MapKind::Forest, SCALE, &out), 0, "{out:?}");
    }
}
