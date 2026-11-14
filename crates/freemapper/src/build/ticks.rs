//! Which way the ticks of fences and earth banks point (ISSprOM 2019-2 §104, §201, §518).
//!
//! The ticks of a fence (518), an earth bank (104, 104.1), an impassable cliff (201) and a
//! passable retained wall (513.2) sit on
//! the right of the line, seen on paper, so the way a line runs says on which side they are. The
//! sources give the lines in any direction. The last step before the map is written turns them:
//!
//! * A fence beside an area that shall not be entered (520, olive green) has its ticks towards
//!   that area. Else a closed fence (a rectangle, say) has them inside, and a fence along a park
//!   (401) towards the park. A fence with no such area on one side only stays as it is.
//! * The ticks of a bank point down the slope. The terrain model says which side is lower; where
//!   it is missing or flat, the bank takes the way round of the banks beside it that are decided.

use geo::Contains as _;
use geo_types::{Point, Polygon};

use crate::build::cliffs::{TICKS_BY_PROFILE, TICKS_KEY};
use crate::derive::ndsm::bilinear;
use crate::georef::Georeferencing;
use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::raster::Raster;

/// The fence whose ticks point to an area that shall not be entered.
const FENCE_CODE: &str = "518";
/// The lines whose ticks point down the slope (the retained wall has them on its lower side).
const BANK_CODES: [&str; 4] = ["104", "104.1", "201", "513.2"];
/// The area that shall not be entered.
const OLIVE_CODE: &str = "520";
/// The park.
const PARK_CODE: &str = "401";

/// Distance between the places along a line where the sides are looked at (ground metres).
const STEP_M: f64 = 2.0;
/// How far from the fence the sides are probed for an olive green or park area (ground metres).
const AREA_PROBE_M: f64 = 1.0;
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

/// Turns the fences and banks among `lists` (the objects of the map and of the candidates map)
/// so that their ticks point the right way. `terrain` is the height model of the project.
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

    let mut bank_lines: Vec<BankLine> = Vec::new();
    for (l, list) in lists.iter_mut().enumerate() {
        for (i, object) in list.iter_mut().enumerate() {
            if !matches!(object.kind, MapObjectKind::Path) || object.coords.len() < 2 {
                continue;
            }
            if Some(object.symbol_id) == fence {
                let line = ground(&object.coords, georef);
                let samples = samples(&line);
                let closed = object
                    .coords
                    .last()
                    .is_some_and(|c| c.flags & FLAG_CLOSE != 0)
                    || line.first() == line.last();
                let vote = match area_vote(&samples, &olive) {
                    0 if closed => -signed_area(&line).signum() as i32,
                    0 => area_vote(&samples, &park),
                    vote => vote,
                };
                if vote < 0 && turn(object) {
                    turned.fences += 1;
                }
            } else if banks.contains(&object.symbol_id) {
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

/// Twice the signed area of a ring (positive when it runs counter-clockwise, north up).
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
    if object.coords.iter().any(|c| c.flags & !FLAG_CLOSE != 0) {
        return false;
    }
    let closed = object
        .coords
        .last()
        .is_some_and(|c| c.flags & FLAG_CLOSE != 0);
    object.coords.reverse();
    for c in &mut object.coords {
        c.flags = 0;
    }
    if closed && let Some(last) = object.coords.last_mut() {
        last.flags = FLAG_CLOSE;
    }
    true
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
