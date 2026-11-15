//! The training level: the objects of the map and the visible candidates become the objects of
//! the training map, which is printed and run on without rework, so no drawn symbol may lie on
//! another or inside its gap (spec 2026-11-14, Part 3).
//!
//! The steps run in a fixed order, each on what the one before left:
//! 1. lines of one symbol that meet end to end are joined ([`join_lines`]);
//! 2. street furniture (531) on a paved area goes;
//! 3. (open land under trees: a later step);
//! 4. what keeps no gap to another object ([`conflicts`], the pairs that keep apart) gives way:
//!    it is cut out there, or dropped when more than half of it is in conflict;
//! 5. lines and areas below the norm's minimum size go.
//!
//! What a step drops is gone from the training map; nothing is tagged hidden. Lines of one
//! network that run side by side are left to the second half and only counted in the report.

use std::collections::{BTreeMap, BTreeSet};

use geo::{Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Intersects as _};
use geo_types::{Coord, LineString, Point, Polygon, Rect};

use crate::build::candidates::MIN_LINE_MM;
use crate::build::conflicts::{
    Class, Classes, Conflict, Footprints, Grade, Relation, TOLERANCE_MM, conflicts, gap,
    meeting_points, rank, relation,
};
use crate::build::geometry::{distance_to_segment, object_polygon, xy};
use crate::build::line_cuts::cut_lines_at_ways;
use crate::build::line_join::{TOUCH_M, join_lines};
use crate::build::points::{FOREST, PAVED_AREA_CODES, SCATTERED_TREES, projected};
use crate::derive::vegetation::min_area_m2;
use crate::georef::Georeferencing;
use crate::measure::{PaperMm, PaperMm2, SymbolMm, SymbolMm2, SymbolScale, map_units_per_metre};
use crate::omap::coord::FLAG_CLOSE;
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::raster::Raster;
use crate::render::geometry::{length, parts};

/// The give-way step stops after this many rounds; what is left then is in the report.
const MAX_ROUNDS: usize = 20;
/// Street furniture: the x of symbol 531, Prominent man-made feature.
const FURNITURE_CODE: &str = "531";
/// ISOM 2017-2 and ISSprOM 2019-2, 532 Stairway: "Minimum length: 3 (graphical) steps". The
/// steps of 532 lie 0.5 mm apart in both bundled symbol sets (`segment_length`), so three steps
/// span 1.0 mm.
const STAIR_MIN_MM: SymbolMm = SymbolMm(1.0);
/// ISSprOM 2019-2, 401 Open land: "Minimum area: 0.5 mm² (footprint 8 m²)".
const OPEN_LAND_MIN_SPRINT: SymbolMm2 = SymbolMm2(0.5);
/// ISOM 2017-2, 401 Open land: "Minimum area: 0.55 mm x 0.55 mm (footprint 8 m x 8 m)".
const OPEN_LAND_MIN_FOREST: SymbolMm2 = SymbolMm2(0.55 * 0.55);
/// 403 Rough open land: ISSprOM 2019-2 "Minimum area: 1 mm² (footprint 16 m²)", ISOM 2017-2
/// "1 mm x 1 mm (footprint 15 m x 15m)".
const ROUGH_OPEN_LAND_MIN: SymbolMm2 = SymbolMm2(1.0);
/// The area symbols with a minimum area: open land (401 to 403), forest (405) and the green
/// shades (406 to 410).
const MIN_AREA_CODES: [&str; 9] = [
    "401", "402", "403", "405", "406", "407", "408", "409", "410",
];

/// What the training level did, for the summary line and the ADR.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TrainingReport {
    /// Objects that went into another by a join.
    pub joined: usize,
    /// Objects dropped whole because they kept no gap.
    pub gave_way: usize,
    /// Objects cut where they kept no gap.
    pub cut: usize,
    /// Lines and areas below the norm's minimum size.
    pub too_small: usize,
    /// Open land reduced to its open ground (a later step).
    pub open_land_cut: usize,
    /// Street furniture on paved areas.
    pub furniture: usize,
    /// Conflict length left, mm on paper, by grade (the pairs of one network, left to the second half).
    pub overlap_left: f64,
    pub gap_left: f64,
}

/// The objects of the training map from the objects of the map and the visible candidates.
/// `tree_cover` is the raster of Task 5 (share of ground under trees per cell), when there is one.
pub fn train(
    objects: Vec<MapObject>,
    tree_cover: Option<&Raster<f32>>,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> (Vec<MapObject>, TrainingReport) {
    let classes = Classes::of(symbol_set);
    let mut objects = objects;
    let mut report = TrainingReport {
        joined: join(&mut objects, &classes, georef),
        furniture: drop_furniture(&mut objects, symbol_set),
        ..TrainingReport::default()
    };
    // 3. Open land under trees (Task 5) reads `tree_cover` here.
    let _ = tree_cover;
    (report.gave_way, report.cut) = give_way(&mut objects, &classes, kind, georef, symbol_set);
    report.too_small = drop_too_small(&mut objects, &classes, kind, georef, symbol_set);
    (report.overlap_left, report.gap_left) = left(&objects, kind, georef, symbol_set);
    (objects, report)
}

/// Step 1: the lines of the networks, of the water and of the landforms that meet end to end
/// are joined, until no join is left (one call joins one pair at a junction of four ends).
/// Returns how many objects went.
fn join(objects: &mut Vec<MapObject>, classes: &Classes, georef: &Georeferencing) -> usize {
    use Class::*;
    let ids_of = |of: &[Class]| -> Vec<u32> { of.iter().flat_map(|&c| classes.ids(c)).collect() };
    let ids = ids_of(&[
        Road,
        Path,
        Railway,
        Watercourse,
        EarthBank,
        Barrier,
        Contour,
    ]);
    let sided = ids_of(&[EarthBank, Cliff, Barrier]);
    let mut joined = 0;
    loop {
        let n = join_lines(objects, &ids, &sided, georef);
        if n == 0 {
            return joined;
        }
        joined += n;
    }
}

/// Step 2: a point of symbol 531 whose centre lies on a paved area goes; in a park it stays.
/// Returns how many went.
fn drop_furniture(objects: &mut Vec<MapObject>, symbol_set: &SymbolSet) -> usize {
    let Some(furniture) = symbol_set.id_for_code(FURNITURE_CODE) else {
        return 0;
    };
    let paved_ids: BTreeSet<u32> = PAVED_AREA_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let paved: Vec<(Rect<f64>, Polygon<f64>)> = objects
        .iter()
        .filter(|o| paved_ids.contains(&o.symbol_id))
        .filter_map(object_polygon)
        .filter_map(|p| Some((p.bounding_rect()?, p)))
        .collect();
    let before = objects.len();
    objects.retain(|o| {
        let is_furniture =
            o.symbol_id == furniture && matches!(o.kind, MapObjectKind::Point { .. });
        let Some(centre) = o.coords.first().filter(|_| is_furniture) else {
            return true;
        };
        let (x, y) = xy(centre);
        let centre = Point::new(x, y);
        !paved
            .iter()
            .any(|(frame, area)| frame.intersects(&centre) && area.intersects(&centre))
    });
    before - objects.len()
}

/// Step 4: every object that keeps no gap to another object it must keep apart from gives way,
/// both grades (the training map keeps every gap). Losers go by ascending rank; a line with more
/// than half its length in conflict, a point and an area go whole, any other line is cut where it
/// is in conflict. A loser whose winner changed earlier in a round waits for the next round,
/// which measures again. Returns how many objects were dropped and how many cut.
fn give_way(
    objects: &mut Vec<MapObject>,
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> (usize, usize) {
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let gap = gap(kind, scale_denominator).0;
    let (mut dropped, mut cut) = (0, 0);
    for _ in 0..MAX_ROUNDS {
        let refs: Vec<&MapObject> = objects.iter().collect();
        let footprints = Footprints::of(&refs, symbol_set);
        let found: Vec<Conflict> = conflicts(&footprints, kind, scale_denominator)
            .into_iter()
            .filter(|c| relation(c.classes.0, c.classes.1) == Relation::Apart)
            .collect();
        if found.is_empty() {
            break;
        }
        let mut by_loser: BTreeMap<usize, Vec<&Conflict>> = BTreeMap::new();
        for conflict in &found {
            by_loser.entry(conflict.loser).or_default().push(conflict);
        }
        // Ascending rank; of one rank the longer first, so that what it gives way to is settled.
        let mut order: Vec<usize> = by_loser.keys().copied().collect();
        order.sort_by(|&a, &b| {
            rank(footprints.class(a))
                .cmp(&rank(footprints.class(b)))
                .then(footprints.length(b).total_cmp(&footprints.length(a)))
                .then(a.cmp(&b))
        });
        let mut changed: BTreeSet<usize> = BTreeSet::new();
        let mut replaced: BTreeMap<usize, Vec<MapObject>> = BTreeMap::new();
        for loser in order {
            let mine = &by_loser[&loser];
            if mine.iter().any(|c| changed.contains(&c.winner)) {
                continue;
            }
            changed.insert(loser);
            let total = footprints.length(loser);
            let in_conflict: f64 = mine.iter().map(|c| c.length.0).sum();
            let pieces = if total <= 0.0 || in_conflict > total / 2.0 + TOLERANCE_MM {
                Vec::new()
            } else {
                let winners: Vec<usize> = mine.iter().map(|c| c.winner).collect();
                let object = &objects[loser];
                let min_m =
                    min_length(object, classes, symbol_set, scale).metres(scale_denominator);
                let pieces = cut_out(
                    object,
                    loser,
                    &winners,
                    &footprints,
                    gap,
                    min_m,
                    georef,
                    symbol_set,
                );
                // A cut that changes nothing would come back every round: the object goes.
                if pieces.len() == 1 && pieces[0] == *object {
                    Vec::new()
                } else {
                    pieces
                }
            };
            if pieces.is_empty() {
                dropped += 1;
            } else {
                cut += 1;
            }
            replaced.insert(loser, pieces);
        }
        let before = std::mem::take(objects);
        for (i, object) in before.into_iter().enumerate() {
            match replaced.remove(&i) {
                Some(pieces) => objects.extend(pieces),
                None => objects.push(object),
            }
        }
    }
    (dropped, cut)
}

/// The pieces of the line `object` (index `loser` in `footprints`) outside the footprints of
/// `winners` grown by its half width and `gap` (mm on paper), except near the places where the
/// centre lines cross: there the drawing joins them, as the measure counts it. Pieces shorter than
/// `min_m` ground metres go.
#[allow(clippy::too_many_arguments)]
fn cut_out(
    object: &MapObject,
    loser: usize,
    winners: &[usize],
    footprints: &Footprints,
    gap: f64,
    min_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    let Some(code) = symbol_set.code_for_id(object.symbol_id) else {
        return Vec::new();
    };
    let centre = footprints.centre(loser);
    let Some(frame) = centre.bounding_rect() else {
        return Vec::new();
    };
    // Beyond the measure's zone by the tolerance, so that the cut ends are clear of it.
    let grow = footprints.half_width(loser) + gap + TOLERANCE_MM;
    let margin = grow + 1.0;
    let around = Rect::new(
        Coord {
            x: frame.min().x - margin,
            y: frame.min().y - margin,
        },
        Coord {
            x: frame.max().x + margin,
            y: frame.max().y + margin,
        },
    )
    .to_polygon();
    let mut ways: Vec<Polygon<f64>> = Vec::new();
    for &winner in winners {
        let other = footprints.footprint(winner).intersection(&around);
        if other.0.is_empty() {
            continue;
        }
        let mut zone = other.buffer(grow);
        let radius = footprints.half_width(loser) + footprints.half_width(winner) + gap;
        for crossing in meeting_points(centre, footprints.centre(winner)) {
            zone = zone.difference(&Point(crossing).buffer(radius));
        }
        ways.extend(zone.0.iter().map(|p| projected(&in_map_units(p), georef)));
    }
    let mut pieces = vec![object.clone()];
    cut_lines_at_ways(&mut pieces, &[code], &ways, min_m, georef, symbol_set);
    pieces
}

/// A polygon in paper mm (y down) in map coordinates.
fn in_map_units(polygon: &Polygon<f64>) -> Polygon<f64> {
    let ring = |ring: &LineString<f64>| -> LineString<f64> {
        ring.coords()
            .map(|c| Coord {
                x: (c.x * 1000.0).round(),
                y: (c.y * 1000.0).round(),
            })
            .collect()
    };
    Polygon::new(
        ring(polygon.exterior()),
        polygon.interiors().iter().map(ring).collect(),
    )
}

/// The minimum length of the line `object` on paper: the norm's for stairs, the one the symbol
/// set gives, else [`MIN_LINE_MM`].
fn min_length(
    object: &MapObject,
    classes: &Classes,
    symbol_set: &SymbolSet,
    scale: SymbolScale,
) -> PaperMm {
    if classes.class(object.symbol_id) == Class::Stairway {
        return STAIR_MIN_MM.on_paper(scale);
    }
    symbol_set
        .minimum_length(object.symbol_id)
        .unwrap_or(MIN_LINE_MM)
}

/// The minimum areas (ground m²) of the area symbols of [`MIN_AREA_CODES`] by symbol id: the
/// values the build uses where it has them (402 and 405 as the point generalization, the green
/// shades as the vegetation areas), else the norm's of the map kind.
fn min_areas(kind: MapKind, symbol_set: &SymbolSet, scale: SymbolScale) -> BTreeMap<u32, f64> {
    let open_land = if kind == MapKind::Forest {
        OPEN_LAND_MIN_FOREST
    } else {
        OPEN_LAND_MIN_SPRINT
    };
    let mut areas = BTreeMap::new();
    for code in MIN_AREA_CODES {
        let min = match code {
            "401" => open_land.square_metres(scale),
            "402" => SCATTERED_TREES.min_area_mm2.square_metres(scale),
            "403" => ROUGH_OPEN_LAND_MIN.square_metres(scale),
            "405" => FOREST.min_area_mm2.square_metres(scale),
            _ => min_area_m2(scale),
        };
        if let Some(id) = symbol_set.id_for_code(code) {
            areas.entry(id).or_insert(min);
        }
    }
    areas
}

/// Whether `object` is a closed line: its last coordinate closes it, or it ends where it starts.
fn is_closed(object: &MapObject) -> bool {
    let (Some(first), Some(last)) = (object.coords.first(), object.coords.last()) else {
        return false;
    };
    last.flags & FLAG_CLOSE != 0 || (first.x, first.y) == (last.x, last.y)
}

/// Step 5: a line shorter than its minimum length goes unless an end continues on another line
/// of its class (within [`TOUCH_M`]); an area of [`MIN_AREA_CODES`] below its minimum area goes.
/// Comparisons are tolerant by [`TOLERANCE_MM`]. Returns how many went.
fn drop_too_small(
    objects: &mut Vec<MapObject>,
    classes: &Classes,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let areas = min_areas(kind, symbol_set, scale);
    let is_line = |o: &MapObject| {
        matches!(o.kind, MapObjectKind::Path) && o.coords.len() >= 2 && classes.is_line(o.symbol_id)
    };
    let short: Vec<bool> = objects
        .iter()
        .map(|o| {
            is_line(o) && {
                let drawn: f64 = parts(&o.coords).iter().map(|p| length(&p.points)).sum();
                drawn < min_length(o, classes, symbol_set, scale).0 - TOLERANCE_MM
            }
        })
        .collect();
    let touch =
        TOUCH_M * map_units_per_metre(scale_denominator) + PaperMm(TOLERANCE_MM).map_units();
    let continues = |i: usize| -> bool {
        let object = &objects[i];
        if is_closed(object) {
            return false;
        }
        let class = classes.class(object.symbol_id);
        let ends = [object.coords[0], object.coords[object.coords.len() - 1]].map(|c| xy(&c));
        objects.iter().enumerate().any(|(j, other)| {
            j != i
                && is_line(other)
                && classes.class(other.symbol_id) == class
                && other.coords.windows(2).any(|w| {
                    ends.iter()
                        .any(|&end| distance_to_segment(end, xy(&w[0]), xy(&w[1])) <= touch)
                })
        })
    };
    let tolerance_m = PaperMm(TOLERANCE_MM).metres(scale_denominator);
    let too_small: Vec<bool> = (0..objects.len())
        .map(|i| {
            let object = &objects[i];
            if short[i] {
                return !continues(i);
            }
            let Some(&min_m2) = areas.get(&object.symbol_id) else {
                return false;
            };
            let Some(polygon) = object_polygon(object) else {
                return false;
            };
            let area_m2 = PaperMm2(polygon.unsigned_area() / 1e6).square_metres(scale_denominator);
            // Tolerant on the side of the smallest square.
            let side = (min_m2.sqrt() - tolerance_m).max(0.0);
            area_m2 < side * side
        })
        .collect();
    let before = objects.len();
    let mut flags = too_small.into_iter();
    objects.retain(|_| !flags.next().unwrap_or(false));
    before - objects.len()
}

/// Step 6: the conflict length left on `objects`, mm on paper: overlap and gap.
fn left(
    objects: &[MapObject],
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> (f64, f64) {
    let refs: Vec<&MapObject> = objects.iter().collect();
    let footprints = Footprints::of(&refs, symbol_set);
    conflicts(&footprints, kind, georef.scale_denominator)
        .iter()
        .fold((0.0, 0.0), |(overlap, gap), c| match c.grade {
            Grade::Overlap => (overlap + c.length.0, gap),
            Grade::Gap => (overlap, gap + c.length.0),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::{Crs, ProjectedPoint};
    use crate::omap::{MapCoord, closed_ring};

    /// The scale of the tests: a forest map at 1:10 000, symbols at 150 %.
    const SCALE: u32 = 10_000;

    fn isom() -> SymbolSet {
        SymbolSet::isom_2017_2(MapKind::Forest.symbol_enlargement(SCALE))
    }

    fn georef_at(scale: u32) -> Georeferencing {
        let centre = Crs::Utm33
            .inverse(ProjectedPoint {
                easting: 392_400.0,
                northing: 5_820_400.0,
            })
            .unwrap();
        Georeferencing::new(centre, scale, "2026-11-08".parse().unwrap()).unwrap()
    }

    fn at((x, y): (f64, f64)) -> MapCoord {
        MapCoord::new((x * 1000.0).round() as i32, (y * 1000.0).round() as i32)
    }

    /// A line `code` through `points` (paper mm, y down).
    fn line(set: &SymbolSet, code: &str, points: &[(f64, f64)]) -> MapObject {
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| at(p)).collect(),
            tags: Vec::new(),
        }
    }

    /// An area `code`: the rectangle from `min` to `max` (paper mm).
    fn area(set: &SymbolSet, code: &str, min: (f64, f64), max: (f64, f64)) -> MapObject {
        let ring: Vec<(i32, i32)> = [
            (min.0, min.1),
            (max.0, min.1),
            (max.0, max.1),
            (min.0, max.1),
        ]
        .iter()
        .map(|&p| {
            let c = at(p);
            (c.x, c.y)
        })
        .collect();
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Path,
            coords: closed_ring(&ring),
            tags: Vec::new(),
        }
    }

    fn point(set: &SymbolSet, code: &str, p: (f64, f64)) -> MapObject {
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![at(p)],
            tags: Vec::new(),
        }
    }

    fn run(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, TrainingReport) {
        train(objects, None, MapKind::Forest, &georef_at(SCALE), set)
    }

    fn of_code<'a>(set: &SymbolSet, objects: &'a [MapObject], code: &str) -> Vec<&'a MapObject> {
        let id = set.id_for_code(code).unwrap();
        objects.iter().filter(|o| o.symbol_id == id).collect()
    }

    /// Length of a line, mm on paper.
    fn length_mm(object: &MapObject) -> f64 {
        parts(&object.coords)
            .iter()
            .map(|p| length(&p.points))
            .sum()
    }

    fn half_width(set: &SymbolSet, code: &str) -> f64 {
        let object = line(set, code, &[(0.0, 0.0), (10.0, 0.0)]);
        Footprints::of(&[&object], set).half_width(0)
    }

    fn apart_left(set: &SymbolSet, objects: &[MapObject]) -> Vec<Conflict> {
        let refs: Vec<&MapObject> = objects.iter().collect();
        conflicts(&Footprints::of(&refs, set), MapKind::Forest, SCALE)
            .into_iter()
            .filter(|c| relation(c.classes.0, c.classes.1) == Relation::Apart)
            .collect()
    }

    #[test]
    fn an_earth_bank_on_a_ditch_goes() {
        let set = isom();
        let ditch = line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]);
        let objects = vec![ditch.clone(), line(&set, "104", &[(0.0, 0.2), (20.0, 0.2)])];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![ditch]);
        assert_eq!(report.gave_way, 1, "{report:?}");
        assert_eq!((report.overlap_left, report.gap_left), (0.0, 0.0));
    }

    #[test]
    fn an_earth_bank_inside_the_gap_of_a_ditch_goes_too() {
        let set = isom();
        let y = half_width(&set, "306") + half_width(&set, "104") + 0.1;
        let ditch = line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]);
        let objects = vec![ditch.clone(), line(&set, "104", &[(0.0, y), (20.0, y)])];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![ditch]);
        assert_eq!(report.gave_way, 1, "{report:?}");
    }

    #[test]
    fn an_earth_bank_that_crosses_a_path_stays_whole() {
        let set = isom();
        let path = line(&set, "505", &[(0.0, 10.0), (20.0, 10.0)]);
        let right = vec![
            path.clone(),
            line(&set, "104", &[(10.0, 2.0), (10.0, 18.0)]),
        ];
        assert_eq!(run(&set, right.clone()).0, right);
        let (sin, cos) = 30f64.to_radians().sin_cos();
        let slant = vec![
            path,
            line(
                &set,
                "104",
                &[
                    (10.0 - 8.0 * cos, 10.0 - 8.0 * sin),
                    (10.0 + 8.0 * cos, 10.0 + 8.0 * sin),
                ],
            ),
        ];
        assert_eq!(run(&set, slant.clone()).0, slant);
    }

    #[test]
    fn an_earth_bank_that_runs_into_a_path_is_cut() {
        let set = isom();
        let path = line(&set, "505", &[(14.0, 0.3), (40.0, 0.3)]);
        let objects = vec![path.clone(), line(&set, "104", &[(0.0, 0.0), (20.0, 0.0)])];
        let (out, report) = run(&set, objects);
        let banks = of_code(&set, &out, "104");
        assert_eq!(banks.len(), 1, "{out:?}");
        assert!((length_mm(banks[0]) - 14.0).abs() < 1.0, "{out:?}");
        assert_eq!(report.cut, 1, "{report:?}");
        assert_eq!(report.gave_way, 0, "{report:?}");
        assert_eq!(of_code(&set, &out, "505"), vec![&path]);
    }

    #[test]
    fn a_fence_on_a_path_goes_where_it_lies_on_it() {
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(12.0, 0.0), (40.0, 0.0)]),
            line(&set, "518", &[(0.0, 0.0), (20.0, 0.0)]),
        ];
        let (out, report) = run(&set, objects);
        let fences = of_code(&set, &out, "518");
        assert_eq!(fences.len(), 1, "{out:?}");
        assert!((length_mm(fences[0]) - 12.0).abs() < 1.0, "{out:?}");
        assert_eq!(report.cut, 1, "{report:?}");
    }

    #[test]
    fn of_two_earth_banks_inside_the_gap_the_shorter_goes() {
        let set = isom();
        // On the side of the long bank without ticks: its ticks point to +y (down).
        let y = -(2.0 * half_width(&set, "104") + 0.1);
        let long = line(&set, "104", &[(0.0, 0.0), (20.0, 0.0)]);
        let objects = vec![line(&set, "104", &[(2.0, y), (18.0, y)]), long.clone()];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![long]);
        assert_eq!(report.gave_way, 1, "{report:?}");
    }

    #[test]
    fn a_point_on_a_building_goes() {
        let set = isom();
        let building = area(&set, "521", (0.0, 0.0), (10.0, 10.0));
        let objects = vec![building.clone(), point(&set, "526", (5.0, 5.0))];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![building]);
        assert_eq!(report.gave_way, 1, "{report:?}");
    }

    #[test]
    fn a_point_beside_it_stays() {
        let set = isom();
        let objects = vec![
            area(&set, "521", (0.0, 0.0), (10.0, 10.0)),
            point(&set, "526", (5.0, 15.0)),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
    }

    #[test]
    fn a_contour_runs_through_a_building_and_a_lake() {
        let set = isom();
        let objects = vec![
            area(&set, "521", (5.0, 0.0), (15.0, 10.0)),
            area(&set, "301", (25.0, 0.0), (35.0, 10.0)),
            line(&set, "101", &[(0.0, 5.0), (40.0, 5.0)]),
        ];
        assert_eq!(run(&set, objects.clone()).0, objects);
    }

    #[test]
    fn a_contour_on_an_earth_bank_goes() {
        let set = isom();
        let bank = line(&set, "104", &[(0.0, 0.0), (20.0, 0.0)]);
        let objects = vec![bank.clone(), line(&set, "101", &[(0.0, 0.1), (20.0, 0.1)])];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![bank]);
        assert_eq!(report.gave_way, 1, "{report:?}");
    }

    #[test]
    fn furniture_on_a_street_goes_and_in_the_park_stays() {
        let set = isom();
        let street = area(&set, "501.5", (0.0, 0.0), (10.0, 10.0));
        let park = point(&set, "531", (30.0, 30.0));
        let objects = vec![street.clone(), point(&set, "531", (5.0, 5.0)), park.clone()];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![street, park]);
        assert_eq!(report.furniture, 1, "{report:?}");
    }

    #[test]
    fn a_patch_of_open_land_below_the_minimum_goes() {
        // ISOM 401: 0.55 mm × 0.55 mm, at 150 % 0.83 mm × 0.83 mm.
        let set = isom();
        let objects = vec![area(&set, "401", (0.0, 0.0), (0.7, 0.7))];
        let (out, report) = run(&set, objects);
        assert_eq!(out, Vec::new());
        assert_eq!(report.too_small, 1, "{report:?}");
    }

    #[test]
    fn a_patch_above_it_stays() {
        let set = isom();
        let objects = vec![area(&set, "401", (0.0, 0.0), (0.9, 0.9))];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report.too_small, 0, "{report:?}");
    }

    #[test]
    fn a_stair_below_the_minimum_goes() {
        // Three steps span 1.0 mm, at 150 % 1.5 mm.
        let set = isom();
        let long = line(&set, "532", &[(0.0, 10.0), (1.6, 10.0)]);
        let objects = vec![line(&set, "532", &[(0.0, 0.0), (1.4, 0.0)]), long.clone()];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![long]);
        assert_eq!(report.too_small, 1, "{report:?}");
    }

    #[test]
    fn a_short_path_that_continues_stays() {
        // A 505 is at least 4.25 mm long, at 150 % 6.4 mm.
        let set = isom();
        let main = line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]);
        let spur = line(&set, "505", &[(10.0, 0.0), (10.0, 2.0)]);
        let alone = line(&set, "505", &[(0.0, 30.0), (2.0, 30.0)]);
        let (out, report) = run(&set, vec![main.clone(), spur.clone(), alone]);
        assert_eq!(out, vec![main, spur]);
        assert_eq!(report.too_small, 1, "{report:?}");
    }

    #[test]
    fn pieces_of_a_path_are_one_object_after() {
        let set = isom();
        let objects: Vec<MapObject> = (0..5)
            .map(|k| {
                let x = 4.0 * f64::from(k);
                line(&set, "505", &[(x, 0.0), (x + 4.0, 0.0)])
            })
            .collect();
        let (out, report) = run(&set, objects);
        assert_eq!(out.len(), 1, "{out:?}");
        assert!((length_mm(&out[0]) - 20.0).abs() < 1e-9);
        assert_eq!(report.joined, 4, "{report:?}");
    }

    #[test]
    fn carriageways_and_tracks_are_left_alone() {
        let set = isom();
        let objects = vec![
            line(&set, "502", &[(0.0, 0.0), (30.0, 0.0)]),
            line(&set, "502", &[(0.0, 0.5), (30.0, 0.5)]),
            line(&set, "509", &[(0.0, 20.0), (30.0, 20.0)]),
            line(&set, "509", &[(0.0, 20.5), (30.0, 20.5)]),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!((report.gave_way, report.cut, report.joined), (0, 0, 0));
    }

    #[test]
    fn no_apart_conflict_is_left() {
        let set = isom();
        let objects = vec![
            line(&set, "306", &[(0.0, 0.0), (30.0, 0.0)]),
            line(&set, "104", &[(-5.0, 0.2), (12.0, 0.2), (12.0, 15.0)]),
            line(&set, "505", &[(0.0, 10.0), (40.0, 10.0)]),
            line(&set, "518", &[(20.0, 2.0), (20.0, 10.0), (35.0, 10.2)]),
            area(&set, "521", (22.0, 12.0), (30.0, 20.0)),
            point(&set, "526", (26.0, 16.0)),
            point(&set, "526", (26.0, 11.0)),
            line(&set, "101", &[(0.0, 20.5), (40.0, 20.5)]),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(apart_left(&set, &out), Vec::new(), "{out:?}");
        assert!(report.gave_way + report.cut > 0, "{report:?}");
    }

    #[test]
    fn the_school_set_does_not_fail() {
        let set = SymbolSet::school_2019();
        let objects: Vec<MapObject> = ["521", "501.6", "531", "401", "532.6"]
            .iter()
            .filter(|c| set.id_for_code(c).is_some())
            .enumerate()
            .map(|(k, code)| {
                let x = 10.0 * k as f64;
                match set.symbol_type(set.id_for_code(code).unwrap()) {
                    Some(1) => point(&set, code, (x, 0.0)),
                    Some(4) => area(&set, code, (x, 0.0), (x + 5.0, 5.0)),
                    _ => line(&set, code, &[(x, 0.0), (x + 5.0, 0.0)]),
                }
            })
            .collect();
        assert!(objects.len() >= 4);
        let (out, _) = train(objects, None, MapKind::Schoolyard, &georef_at(1000), &set);
        assert!(!out.is_empty());
    }

    #[test]
    fn the_report_counts_what_is_left() {
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "505", &[(0.0, 0.3), (20.0, 0.3)]),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert!(report.gap_left + report.overlap_left > 0.0, "{report:?}");
    }
}
