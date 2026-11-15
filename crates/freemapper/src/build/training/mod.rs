//! The training level: the objects of the map and the visible candidates become the objects of
//! the training map, which is printed and run on without rework, so no drawn symbol may lie on
//! another or inside its gap (spec 2026-11-14, Part 3).
//!
//! The steps run in a fixed order, each on what the one before left:
//! 1. lines of one symbol that meet end to end are joined ([`join_lines`]);
//! 2. street furniture (531) on a paved area goes;
//! 3. open land of a park is reduced to its open ground where the tree cover shows trees
//!    ([`open_ground`]);
//! 4. inside olive green (520) only buildings, water, contours and the ways a runner can cross
//!    stay ([`clear_olive`](olive::clear_olive));
//! 5. what is alike is merged: two roads, railways or paths side by side become their middle,
//!    and on a forest map a path or tram inside the drawn width of a road goes
//!    ([`merge`](merge::merge)); the line ends that touched a line before touch one after
//!    ([`Junctions`]), and the lines that meet end to end are joined again;
//! 6. a point symbol too close to a way, a building, water or another point is moved off
//!    ([`move_points`](points::move_points));
//! 7. what keeps no gap to another object ([`conflicts`], the pairs that keep apart) gives way:
//!    it is cut out there, or dropped when more than half of it is in conflict;
//! 8. lines and areas below the norm's minimum size go.
//!
//! What a step drops is gone from the training map; nothing is tagged hidden. Lines of one
//! network that still run side by side are counted in the report.

mod junctions;
mod lines;
mod merge;
mod olive;
mod points;

use std::collections::{BTreeMap, BTreeSet};

use geo::{Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Intersects as _};
use geo_types::{Coord, MultiPolygon, Point, Polygon, Rect};

use crate::build::candidates::MIN_LINE_MM;
use crate::build::conflicts::{
    Class, Classes, Conflict, Footprints, Grade, Relation, TOLERANCE_MM, conflicts, gap,
    length_in_zones, meeting_points, rank, relation,
};
use crate::build::dedup::tag_holds;
use crate::build::geometry::{distance_to_segment, object_polygon, xy};
use crate::build::line_cuts::cut_lines_at_ways;
use crate::build::line_join::{TOUCH_M, join_lines};
use crate::build::open_land::MIN_PIECE_SIDE_MM;
use crate::build::out_of_bounds::{area_object, short_side};
use crate::build::points::{FOREST, PAVED_AREA_CODES, SCATTERED_TREES, in_map_units, projected};
use crate::derive::vegetation::{CELL_M, VegetationClass, majority, min_area_m2, outlines};
use crate::georef::Georeferencing;
use crate::measure::{PaperMm, PaperMm2, SymbolMm, SymbolMm2, SymbolScale, map_units_per_metre};
use crate::omap::coord::FLAG_CLOSE;
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::raster::{GridSpec, Raster};
use crate::render::geometry::{length, parts};
use junctions::Junctions;
use merge::merge;
use olive::clear_olive;
pub use olive::{OLIVE_EDGE_MM, WAY_STUB_MM};
use points::move_points;
pub use points::{MAX_POINT_SHIFT_MM, MAX_SHIFT_MM};

/// The give-way step stops after this many rounds; what is left then is in the report.
const MAX_ROUNDS: usize = 20;
/// Street furniture: the x of symbol 531, Prominent man-made feature.
const FURNITURE_CODE: &str = "531";
/// Open land, the only symbol [`open_ground`] reduces.
const OPEN_LAND_CODE: &str = "401";
/// The land use of a park, green space or other recreation ground (`freemapper:nutzart` starts
/// with it).
const PARK_NUTZART: &str = "Sport-, Freizeit- und Erholungsfläche";
/// An OSM park (`freemapper:osm` holds it).
const OSM_PARK: &str = "leisure=park";
/// A cell of the tree cover with a smaller share of ground under trees is open ground.
const OPEN_BELOW_SHARE: f32 = 0.5;
/// The window of tree cover read round a park reaches this far past it, beyond the blur of the
/// outlines (three sigmas of 1.5 m and a cell), so that the open ground at the park's edge is
/// not cut by the window's edge.
const COVER_MARGIN_M: f64 = 8.0;
/// The tag of a piece of open ground: what the step did to it.
const GENERALIZED_KEY: &str = "freemapper:generalized";
/// The value of [`GENERALIZED_KEY`] on a piece of open ground.
const TREE_COVER: &str = "tree cover";
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
    /// Open land of parks reduced to its open ground, or dropped where none is left.
    pub open_land_cut: usize,
    /// Street furniture on paved areas.
    pub furniture: usize,
    /// Objects inside olive green that went or were cut.
    pub inside_olive: usize,
    /// Point symbols moved off ways, buildings, water and other points.
    pub points_moved: usize,
    /// Pairs of roads, railways or paths side by side that became one line.
    pub merged: usize,
    /// Paths and trams that went because they ran inside the drawn width of a road (forest map).
    pub dropped_beside_roads: usize,
    /// Line ends moved back onto the line they touched, or onto the line that replaced it.
    pub junctions_repaired: usize,
    /// Line ends that touched a line before the merge and touch none after.
    pub junctions_free: usize,
    /// Conflict length left, mm on paper, by grade (the pairs of one network, left to the second half).
    pub overlap_left: f64,
    pub gap_left: f64,
    /// Conflict length of the pairs that keep apart left, mm on paper: 0 unless the give-way
    /// step stopped at its last round.
    pub apart_left: f64,
}

/// The objects of the training map from the objects of the map and the visible candidates.
/// `tree_cover` is the share of ground under trees per cell (`data/tree-cover.tif`), when there
/// is one.
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
    if let Some(tree_cover) = tree_cover {
        report.open_land_cut = open_ground(&mut objects, tree_cover, georef, symbol_set);
    }
    report.inside_olive = clear_olive(&mut objects, &classes, georef, symbol_set);
    let junctions = Junctions::record(&mut objects, &classes, georef);
    let merged = merge(&mut objects, &classes, kind, georef, symbol_set);
    let reach = MAX_SHIFT_MM.0 + gap(kind, georef.scale_denominator).0;
    (report.junctions_repaired, report.junctions_free) =
        junctions.repair(&mut objects, &merged.replaced_by, reach, georef);
    junctions::forget(&mut objects);
    report.merged = merged.merged;
    report.dropped_beside_roads = merged.dropped_beside_roads;
    report.cut += merged.cut;
    report.joined += join(&mut objects, &classes, georef);
    report.points_moved = move_points(&mut objects, &classes, kind, georef, symbol_set);
    let (gave_way, cut) = give_way(&mut objects, &classes, kind, georef, symbol_set);
    report.gave_way = gave_way;
    report.cut += cut;
    report.too_small = drop_too_small(&mut objects, &classes, kind, georef, symbol_set);
    report.joined += join_ways(&mut objects, &classes, georef);
    (report.overlap_left, report.gap_left, report.apart_left) =
        left(&objects, kind, georef, symbol_set);
    junctions::forget(&mut objects);
    debug_assert!(
        objects
            .iter()
            .all(|o| o.tags.iter().all(|(k, _)| k != junctions::JUNCTION_TAG)),
        "the junction keys never reach the map"
    );
    (objects, report)
}

/// Step 1: the lines of the networks (stairways among them), of the water and of the landforms
/// that meet end to end are joined, until no join is left (one call joins one pair at a junction
/// of four ends). Returns how many objects went.
fn join(objects: &mut Vec<MapObject>, classes: &Classes, georef: &Georeferencing) -> usize {
    use Class::*;
    let ids_of = |of: &[Class]| -> Vec<u32> { of.iter().flat_map(|&c| classes.ids(c)).collect() };
    let ids = ids_of(&[
        Road,
        Path,
        Railway,
        Stairway,
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

/// Whether `object` is the open land of a park: from the land use of recreation grounds, or an
/// OSM park.
fn is_park(object: &MapObject) -> bool {
    object.tags.iter().any(|(k, v)| {
        (k == "freemapper:nutzart" && v.starts_with(PARK_NUTZART))
            || (k == "freemapper:osm" && tag_holds(v, OSM_PARK))
    })
}

/// Step 3: open land (401) of a park ([`is_park`]) keeps only its open ground: the cells of
/// `tree_cover` with a share below [`OPEN_BELOW_SHARE`] (and those it does not know), taken to
/// the vegetation's cells ([`open_mask`]), smoothed with [`majority`] and outlined as the
/// vegetation classes are. Pieces below the vegetation's
/// minimum area or with a short side under [`MIN_PIECE_SIDE_MM`] go; the rest keep the tags of
/// the object and get `freemapper:generalized=tree cover`. Where the trees take less than the
/// minimum area from it ([`open_pieces`]) the object stays as it is. Returns how many objects were reduced or dropped.
fn open_ground(
    objects: &mut Vec<MapObject>,
    tree_cover: &Raster<f32>,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let Some(open_land) = symbol_set.id_for_code(OPEN_LAND_CODE) else {
        return 0;
    };
    let scale = SymbolScale::drawn_with(symbol_set, georef.scale_denominator);
    let min_m2 = min_area_m2(scale);
    let min_side_m = MIN_PIECE_SIDE_MM.metres(scale);
    let mut reduced = 0;
    let before = std::mem::take(objects);
    for object in before {
        let pieces = (object.symbol_id == open_land && is_park(&object))
            .then(|| object_polygon(&object))
            .flatten()
            .and_then(|p| open_pieces(&projected(&p, georef), tree_cover, min_m2, min_side_m));
        let Some(pieces) = pieces else {
            objects.push(object);
            continue;
        };
        let mut tags = object.tags.clone();
        match tags.iter_mut().find(|(k, _)| k == GENERALIZED_KEY) {
            Some((_, v)) => {
                v.push_str("; ");
                v.push_str(TREE_COVER);
            }
            None => tags.push((GENERALIZED_KEY.to_string(), TREE_COVER.to_string())),
        }
        let drawn: Result<Vec<MapObject>, _> = pieces
            .iter()
            .map(|piece| area_object(piece, open_land, tags.clone(), georef))
            .collect();
        match drawn {
            Ok(drawn) => {
                objects.extend(drawn);
                reduced += 1;
            }
            Err(_) => objects.push(object),
        }
    }
    reduced
}

/// The open ground of the park `area` (map CRS), pieces below `min_m2` or with a short side
/// under `min_side_m` left out; `None` when the trees take less than `min_m2` from the park
/// shrunk by one cell of [`open_mask`]: the cells along its edge step across it, and the outline
/// of trees beside the park follows the steps.
fn open_pieces(
    area: &Polygon<f64>,
    tree_cover: &Raster<f32>,
    min_m2: f64,
    min_side_m: f64,
) -> Option<Vec<Polygon<f64>>> {
    let frame = area.bounding_rect()?;
    let open = majority(&open_mask(tree_cover, &frame));
    if open.data.iter().all(Option::is_some) {
        return None;
    }
    let ground = outlines(&open, VegetationClass::Open, min_m2);
    let pieces: Vec<Polygon<f64>> = MultiPolygon(vec![area.clone()])
        .intersection(&ground)
        .0
        .into_iter()
        .filter(|p| p.unsigned_area() >= min_m2 && short_side(p) >= min_side_m)
        .collect();
    let core = area.buffer(-open.grid.pixel_size);
    let lost = core
        .difference(&MultiPolygon(pieces.clone()))
        .unsigned_area();
    (lost >= min_m2).then_some(pieces)
}

/// The open ground round `frame` (map CRS) and [`COVER_MARGIN_M`] beyond it, on cells of the
/// vegetation's size ([`CELL_M`], aligned to `tree_cover`), the grid [`majority`] and
/// [`outlines`] are made for: a cell is open ground (`Some(Open)`) when at least half of the
/// cells of `tree_cover` in it are, that is have a share below [`OPEN_BELOW_SHARE`], an unknown
/// share (`NaN`) or lie outside the raster; else `None`.
fn open_mask(tree_cover: &Raster<f32>, frame: &Rect<f64>) -> Raster<Option<VegetationClass>> {
    let grid = tree_cover.grid;
    let k = (CELL_M / grid.pixel_size).round().max(1.0) as i64;
    let size = grid.pixel_size * k as f64;
    let col = |e: f64| ((e - grid.west) / size).floor() as i64;
    let row = |n: f64| ((grid.north - n) / size).floor() as i64;
    let (x0, x1) = (
        col(frame.min().x - COVER_MARGIN_M),
        col(frame.max().x + COVER_MARGIN_M) + 1,
    );
    let (y0, y1) = (
        row(frame.max().y + COVER_MARGIN_M),
        row(frame.min().y - COVER_MARGIN_M) + 1,
    );
    let window = GridSpec {
        west: grid.west + x0 as f64 * size,
        north: grid.north - y0 as f64 * size,
        pixel_size: size,
        width: (x1 - x0) as u32,
        height: (y1 - y0) as u32,
    };
    let (width, height) = (i64::from(grid.width), i64::from(grid.height));
    let open = |xx: i64, yy: i64| {
        if xx < 0 || yy < 0 || xx >= width || yy >= height {
            return true;
        }
        let share = tree_cover.get(xx as u32, yy as u32);
        share.is_nan() || share < OPEN_BELOW_SHARE
    };
    let mut mask = Raster::filled(window, Some(VegetationClass::Open));
    for y in 0..window.height {
        let fy = (y0 + i64::from(y)) * k;
        for x in 0..window.width {
            let fx = (x0 + i64::from(x)) * k;
            let open_cells = (fy..fy + k)
                .flat_map(|yy| (fx..fx + k).map(move |xx| (xx, yy)))
                .filter(|&(xx, yy)| open(xx, yy))
                .count() as i64;
            if 2 * open_cells < k * k {
                mask.set(x, y, None);
            }
        }
    }
    mask
}

/// Step 7: every object that keeps no gap to another object it must keep apart from gives way,
/// both grades (the training map keeps every gap). Losers go by ascending rank; a line with more
/// than half its length in conflict (the zones of all its winners together, a stretch near two
/// counted once), a point and an area go whole, any other line is cut where it is in conflict.
/// Walls and impassable fences ([`KEPT_BARRIER_CODES`]) are always cut, never dropped whole: a
/// missing barrier changes the route choice. Pieces below the minimum length go in either case.
/// A loser whose winner gives way itself in the round waits for the next round, which measures
/// again. Returns how many objects were dropped and how many cut.
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
        // Ascending rank; of one rank the longer first.
        let mut order: Vec<usize> = by_loser.keys().copied().collect();
        order.sort_by(|&a, &b| {
            rank(footprints.class(a))
                .cmp(&rank(footprints.class(b)))
                .then(footprints.length(b).total_cmp(&footprints.length(a)))
                .then(a.cmp(&b))
        });
        // Winners first: a loser waits while one of its winners gives way itself, since that
        // winner may go and free it. With strict ranks and the shorter giving way some loser
        // has no such winner; should the tolerance of the tie-break make a ring of equals,
        // the first loser in the order is settled, so every round settles one.
        let mut ready: Vec<usize> = order
            .iter()
            .copied()
            .filter(|loser| {
                by_loser[loser]
                    .iter()
                    .all(|c| !by_loser.contains_key(&c.winner))
            })
            .collect();
        if ready.is_empty() {
            ready.extend(order.first());
        }
        let mut replaced: BTreeMap<usize, Vec<MapObject>> = BTreeMap::new();
        for loser in ready {
            let mine = &by_loser[&loser];
            let object = &objects[loser];
            let winners: Vec<usize> = mine.iter().map(|c| c.winner).collect();
            let total = footprints.length(loser);
            let mostly = || {
                let in_conflict = match mine.as_slice() {
                    [one] => one.length.0,
                    _ => length_in_zones(&footprints, loser, &winners, kind, scale_denominator),
                };
                in_conflict > total / 2.0 + TOLERANCE_MM
            };
            let pieces = if total <= 0.0 || (!is_kept_barrier(object, symbol_set) && mostly()) {
                Vec::new()
            } else {
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
                // A cut that changes nothing would come back every round: the object goes, a
                // barrier too (the measure found a stretch the cut cannot reach).
                if pieces.len() == 1 && pieces[0] == *object {
                    Vec::new()
                } else {
                    let mut names: Vec<&str> = winners
                        .iter()
                        .map(|&w| class_name(footprints.class(w)))
                        .collect();
                    names.sort_unstable();
                    names.dedup();
                    let why = format!("{CUT_GAP_TO}{}", names.join(", "));
                    let mut pieces = pieces;
                    for piece in &mut pieces {
                        add_generalized(piece, &why);
                    }
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

/// The beginning of the `freemapper:generalized` value of a piece cut by give way; the classes it
/// keeps the gap to follow.
const CUT_GAP_TO: &str = "cut: gap to ";

/// The name of `class` in the `freemapper:generalized` value of a cut piece.
fn class_name(class: Class) -> &'static str {
    match class {
        Class::Building => "building",
        Class::WaterArea => "water area",
        Class::BankLine => "bank line",
        Class::Railway => "railway",
        Class::Road => "road",
        Class::Path => "path",
        Class::Watercourse => "watercourse",
        Class::Stairway => "stairway",
        Class::Barrier => "barrier",
        Class::Cliff => "cliff",
        Class::EarthBank => "earth bank",
        Class::Contour => "contour",
        Class::Point => "point symbol",
        Class::OtherArea => "area",
        Class::Other => "other",
    }
}

/// Walls (513, passable or not, 515) and impassable fences (518): a runner meets them as they
/// are, so give way only cuts them where they keep no gap and never drops them whole.
const KEPT_BARRIER_CODES: [&str; 3] = ["513", "515", "518"];

/// Whether `object` is one of [`KEPT_BARRIER_CODES`] (or a variant of it such as 513.2).
fn is_kept_barrier(object: &MapObject, symbol_set: &SymbolSet) -> bool {
    symbol_set
        .code_for_id(object.symbol_id)
        .and_then(|code| code.split('.').next())
        .is_some_and(|base| KEPT_BARRIER_CODES.contains(&base))
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

/// The classes that count as one for [`drop_too_small`]: the ways of the network.
fn continues_class(class: Class) -> Class {
    match class {
        Class::Road | Class::Railway | Class::Stairway => Class::Path,
        other => other,
    }
}

/// The `freemapper:generalized` value of a stair below its minimum drawn as the way it lies in.
const STAIR_AS_PATH: &str = "stair below the minimum drawn as path";

/// Step 8: a line shorter than its minimum length goes unless an end continues (within
/// [`TOUCH_M`]) on another line that stays: of the network (roads, paths, railways and
/// stairways count as one) for a way, of its own class for any other line. A short line that
/// continues on a short line stays only when that one stays, so a chain of short pieces holds
/// when one of its ends meets a line long enough, and two short pieces do not keep each other.
/// A stairway below its minimum stays only when both ends continue on a road or path that
/// stays; it is then drawn with the symbol of the longest of them, its geometry and tags kept
/// and `freemapper:generalized` added, unless a barrier runs along it ([`barrier_along`]): then
/// it stays a stairway. A short stairway that does not goes, and what continued
/// only on it is judged again without it. An area of [`MIN_AREA_CODES`] below its minimum area
/// goes. Comparisons are tolerant by [`TOLERANCE_MM`]. Returns how many went.
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
    let drawn = |o: &MapObject| -> f64 { parts(&o.coords).iter().map(|p| length(&p.points)).sum() };
    let short: Vec<bool> = objects
        .iter()
        .map(|o| {
            is_line(o) && drawn(o) < min_length(o, classes, symbol_set, scale).0 - TOLERANCE_MM
        })
        .collect();
    let touch =
        TOUCH_M * map_units_per_metre(scale_denominator) + PaperMm(TOLERANCE_MM).map_units();
    // The lines each end of each short open line touches.
    let touched: Vec<[Vec<usize>; 2]> = (0..objects.len())
        .map(|i| {
            let object = &objects[i];
            if !short[i] || is_closed(object) {
                return [Vec::new(), Vec::new()];
            }
            let class = continues_class(classes.class(object.symbol_id));
            [object.coords[0], object.coords[object.coords.len() - 1]].map(|end| {
                let end = xy(&end);
                (0..objects.len())
                    .filter(|&j| {
                        let other = &objects[j];
                        j != i
                            && is_line(other)
                            && continues_class(classes.class(other.symbol_id)) == class
                            && other
                                .coords
                                .windows(2)
                                .any(|w| distance_to_segment(end, xy(&w[0]), xy(&w[1])) <= touch)
                    })
                    .collect()
            })
        })
        .collect();
    let is_stair = |i: usize| classes.class(objects[i].symbol_id) == Class::Stairway;
    let is_way = |j: usize| {
        matches!(
            classes.class(objects[j].symbol_id),
            Class::Road | Class::Path
        )
    };
    // The short stairways that go: found round by round, each round on what stays without them.
    let mut gone = vec![false; objects.len()];
    let stays = loop {
        // The short lines that stay: from the lines long enough along the short ones they touch,
        // until none is added.
        let mut stays: Vec<bool> = short.iter().map(|&s| !s).collect();
        loop {
            let mut added = false;
            for i in 0..objects.len() {
                if !stays[i] && !gone[i] && touched[i].iter().flatten().any(|&j| stays[j]) {
                    stays[i] = true;
                    added = true;
                }
            }
            if !added {
                break;
            }
        }
        let mut more = false;
        for i in (0..objects.len()).filter(|&i| short[i] && stays[i] && is_stair(i)) {
            let both = touched[i]
                .iter()
                .all(|end| end.iter().any(|&j| stays[j] && is_way(j)));
            if !both {
                gone[i] = true;
                more = true;
            }
        }
        if !more {
            break stays;
        }
    };
    // A short stairway that stays takes the symbol of the longest way at its ends (of two as
    // long, the earlier object).
    let as_path: Vec<(usize, u32)> = (0..objects.len())
        .filter(|&i| short[i] && stays[i] && is_stair(i))
        .filter_map(|i| {
            let way = touched[i]
                .iter()
                .flatten()
                .copied()
                .filter(|&j| stays[j] && is_way(j))
                .max_by(|&a, &b| {
                    drawn(&objects[a])
                        .total_cmp(&drawn(&objects[b]))
                        .then(b.cmp(&a))
                })?;
            Some((i, objects[way].symbol_id))
        })
        .filter(|&(i, symbol_id)| {
            !barrier_along(
                objects,
                i,
                symbol_id,
                classes,
                kind,
                scale_denominator,
                symbol_set,
            )
        })
        .collect();
    let tolerance_m = PaperMm(TOLERANCE_MM).metres(scale_denominator);
    let too_small: Vec<bool> = (0..objects.len())
        .map(|i| {
            let object = &objects[i];
            if short[i] {
                return !stays[i];
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
    for (i, symbol_id) in as_path {
        let object = &mut objects[i];
        object.symbol_id = symbol_id;
        add_generalized(object, STAIR_AS_PATH);
    }
    let before = objects.len();
    let mut flags = too_small.into_iter();
    objects.retain(|_| !flags.next().unwrap_or(false));
    before - objects.len()
}

/// Whether a barrier runs along the stairway `objects[stair]` so close that, drawn with the path
/// symbol `symbol_id`, the stairway would keep no gap to it: a stairway and a barrier may touch
/// (ISSprOM 2019-2, 532), a path and a barrier may not.
fn barrier_along(
    objects: &[MapObject],
    stair: usize,
    symbol_id: u32,
    classes: &Classes,
    kind: MapKind,
    scale_denominator: u32,
    symbol_set: &SymbolSet,
) -> bool {
    let as_path = MapObject {
        symbol_id,
        ..objects[stair].clone()
    };
    let frame = |o: &MapObject| -> Option<Rect<f64>> {
        let xs = o.coords.iter().map(|c| f64::from(c.x));
        let ys = o.coords.iter().map(|c| f64::from(c.y));
        Some(Rect::new(
            Coord {
                x: xs.clone().reduce(f64::min)?,
                y: ys.clone().reduce(f64::min)?,
            },
            Coord {
                x: xs.reduce(f64::max)?,
                y: ys.reduce(f64::max)?,
            },
        ))
    };
    let Some(own) = frame(&as_path) else {
        return false;
    };
    // Far enough for every symbol width and the gap.
    let reach = PaperMm(2.0).map_units();
    let mut near: Vec<&MapObject> = vec![&as_path];
    near.extend(objects.iter().enumerate().filter_map(|(j, o)| {
        let f = frame(o)?;
        let close = f.min().x <= own.max().x + reach
            && f.max().x >= own.min().x - reach
            && f.min().y <= own.max().y + reach
            && f.max().y >= own.min().y - reach;
        (j != stair && close && classes.class(o.symbol_id) == Class::Barrier).then_some(o)
    }));
    if near.len() == 1 {
        return false;
    }
    conflicts(&Footprints::of(&near, symbol_set), kind, scale_denominator)
        .iter()
        .any(|c| {
            (c.loser == 0 || c.winner == 0) && relation(c.classes.0, c.classes.1) == Relation::Apart
        })
}

/// Adds `value` to the `freemapper:generalized` tag of `object`.
pub(super) fn add_generalized(object: &mut MapObject, value: &str) {
    match object.tags.iter_mut().find(|(k, _)| k == GENERALIZED_KEY) {
        Some((_, v)) => {
            v.push_str("; ");
            v.push_str(value);
        }
        None => object
            .tags
            .push((GENERALIZED_KEY.to_string(), value.to_string())),
    }
}

/// Step 8, last: the ways (roads, paths, stairways) that meet end to end are joined again, so a
/// stairway drawn as path joins the path it lies in. Returns how many objects went.
fn join_ways(objects: &mut Vec<MapObject>, classes: &Classes, georef: &Georeferencing) -> usize {
    let ids: Vec<u32> = [Class::Road, Class::Path, Class::Stairway]
        .iter()
        .flat_map(|&c| classes.ids(c))
        .collect();
    let mut joined = 0;
    loop {
        let n = join_lines(objects, &ids, &[], georef);
        if n == 0 {
            return joined;
        }
        joined += n;
    }
}

/// Step 9: the conflict length left on `objects`, mm on paper: overlap and gap of the pairs of
/// one network, and of the pairs that keep apart.
fn left(
    objects: &[MapObject],
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> (f64, f64, f64) {
    let refs: Vec<&MapObject> = objects.iter().collect();
    let footprints = Footprints::of(&refs, symbol_set);
    conflicts(&footprints, kind, georef.scale_denominator)
        .iter()
        .fold((0.0, 0.0, 0.0), |(overlap, gap, apart), c| {
            let length = c.length.0;
            match (relation(c.classes.0, c.classes.1), c.grade) {
                (Relation::Apart, _) => (overlap, gap, apart + length),
                (_, Grade::Overlap) => (overlap + length, gap, apart),
                (_, Grade::Gap) => (overlap, gap + length, apart),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::{Crs, ProjectedPoint};
    use crate::omap::{MapCoord, closed_ring};

    /// The scale of the tests: a forest map at 1:10 000, symbols at 150 %.
    pub(super) const SCALE: u32 = 10_000;

    pub(super) fn isom() -> SymbolSet {
        SymbolSet::isom_2017_2(MapKind::Forest.symbol_enlargement(SCALE))
    }

    pub(super) fn georef_at(scale: u32) -> Georeferencing {
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
    pub(super) fn line(set: &SymbolSet, code: &str, points: &[(f64, f64)]) -> MapObject {
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| at(p)).collect(),
            tags: Vec::new(),
        }
    }

    /// An area `code`: the rectangle from `min` to `max` (paper mm).
    pub(super) fn area(set: &SymbolSet, code: &str, min: (f64, f64), max: (f64, f64)) -> MapObject {
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

    pub(super) fn point(set: &SymbolSet, code: &str, p: (f64, f64)) -> MapObject {
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

    pub(super) fn of_code<'a>(
        set: &SymbolSet,
        objects: &'a [MapObject],
        code: &str,
    ) -> Vec<&'a MapObject> {
        let id = set.id_for_code(code).unwrap();
        objects.iter().filter(|o| o.symbol_id == id).collect()
    }

    /// Length of a line, mm on paper.
    pub(super) fn length_mm(object: &MapObject) -> f64 {
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
        assert!(
            fences[0]
                .tags
                .contains(&(GENERALIZED_KEY.into(), "cut: gap to path".into())),
            "{:?}",
            fences[0].tags
        );
    }

    #[test]
    fn a_fence_between_a_path_and_a_ditch_is_cut_not_dropped() {
        // The path runs along the first 40 % of the fence, the ditch along the last 40 %: more
        // than half of it is in conflict, but a barrier is only cut where it is.
        let set = isom();
        let path = line(&set, "505", &[(-10.0, 0.3), (16.0, 0.3)]);
        let ditch = line(&set, "306", &[(24.0, -0.3), (50.0, -0.3)]);
        let objects = vec![
            path.clone(),
            ditch.clone(),
            line(&set, "518", &[(0.0, 0.0), (40.0, 0.0)]),
        ];
        let (out, report) = run(&set, objects);
        let fences = of_code(&set, &out, "518");
        assert_eq!(fences.len(), 1, "{out:?}");
        let kept = length_mm(fences[0]);
        assert!((6.0..8.0).contains(&kept), "{kept}");
        assert_eq!((report.cut, report.gave_way), (1, 0), "{report:?}");
        assert_eq!(apart_left(&set, &out), Vec::new(), "{out:?}");
        assert_eq!(of_code(&set, &out, "505"), vec![&path]);
        assert_eq!(of_code(&set, &out, "306"), vec![&ditch]);
    }

    #[test]
    fn an_impassable_wall_mostly_on_a_path_keeps_its_free_part() {
        // The path runs along 70 % of it; the impassable wall and fence and the passable wall
        // keep the 6 mm beyond.
        let set = isom();
        for code in ["515", "518", "513"] {
            let objects = vec![
                line(&set, "505", &[(-10.0, 0.3), (14.0, 0.3)]),
                line(&set, code, &[(0.0, 0.0), (20.0, 0.0)]),
            ];
            let (out, report) = run(&set, objects);
            let walls = of_code(&set, &out, code);
            assert_eq!(walls.len(), 1, "{code}: {out:?}");
            let kept = length_mm(walls[0]);
            assert!((4.5..6.0).contains(&kept), "{code}: {kept}");
            assert_eq!((report.cut, report.gave_way), (1, 0), "{code}: {report:?}");
        }
    }

    #[test]
    fn an_earth_bank_between_two_paths_is_measured_once() {
        // Two paths along its first 30 %, one on each side: 30 % of it is in conflict, not 60 %,
        // so it is cut, not dropped.
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(-10.0, 0.3), (9.0, 0.3)]),
            line(&set, "505", &[(-10.0, -0.3), (9.0, -0.3)]),
            line(&set, "104", &[(0.0, 0.0), (30.0, 0.0)]),
        ];
        let (out, report) = run(&set, objects);
        let banks = of_code(&set, &out, "104");
        assert_eq!(banks.len(), 1, "{out:?}");
        assert!((length_mm(banks[0]) - 20.5).abs() < 1.0, "{out:?}");
        assert_eq!((report.cut, report.gave_way), (1, 0), "{report:?}");
    }

    #[test]
    fn of_two_earth_banks_inside_the_gap_the_shorter_goes() {
        let set = isom();
        // On the tick side of the long bank (its ticks point to +y): the short one runs through
        // them, in and out of their overlap zone.
        let y = 2.0 * half_width(&set, "104") + 0.1;
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
    fn a_contour_on_a_bank_on_a_ditch_stays() {
        // The bank gives way to the ditch first; with the bank gone the contour is free (a
        // contour may lie on a ditch), so it must not go against the bank in the same round.
        let set = isom();
        let ditch = line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]);
        let contour = line(&set, "101", &[(0.0, 0.1), (20.0, 0.1)]);
        let objects = vec![
            ditch.clone(),
            line(&set, "104", &[(0.0, 0.2), (20.0, 0.2)]),
            contour.clone(),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![ditch, contour]);
        assert_eq!(report.gave_way, 1, "{report:?}");
    }

    #[test]
    fn a_lake_with_a_boathouse_stays_whole() {
        let set = isom();
        let objects = vec![
            area(&set, "301", (0.0, 0.0), (30.0, 20.0)),
            area(&set, "521", (28.0, 8.0), (32.0, 12.0)),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
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
    fn a_path_cut_by_steps_stays() {
        // Rathen: 505 – 532 – 505 – 532 – 505, every piece below its minimum (505 6.4 mm, 532
        // 1.5 mm at 150 %), the chain ending on a longer path at one end.
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(0.0, -10.0), (0.0, 10.0)]),
            line(&set, "505", &[(0.0, 0.0), (3.0, 0.0)]),
            line(&set, "532", &[(3.0, 0.0), (4.2, 0.0)]),
            line(&set, "505", &[(4.2, 0.0), (7.2, 0.0)]),
            line(&set, "532", &[(7.2, 0.0), (8.4, 0.0)]),
            line(&set, "505", &[(8.4, 0.0), (11.4, 0.0)]),
        ];
        let (out, report) = run(&set, objects.clone());
        // Every piece stays; the steps, too short to draw, are drawn as the path they lie in,
        // which then joins into one line.
        assert_eq!(out.len(), 2, "{out:?}");
        assert_eq!(out[0], objects[0]);
        assert_eq!(of_code(&set, &out, "505").len(), 2, "{out:?}");
        assert!((length_mm(&out[1]) - 11.4).abs() < 1e-9, "{out:?}");
        assert_eq!((report.too_small, report.joined), (0, 4), "{report:?}");
    }

    #[test]
    fn a_step_too_short_between_paths_becomes_path() {
        let set = isom();
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (10.0, 0.0)]),
            line(&set, "532", &[(10.0, 0.0), (10.3, 0.0)]),
            line(&set, "505", &[(10.3, 0.0), (20.0, 0.0)]),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(out.len(), 1, "{out:?}");
        assert_eq!(of_code(&set, &out, "505").len(), 1, "{out:?}");
        assert!((length_mm(&out[0]) - 20.0).abs() < 1e-9, "{out:?}");
        assert_eq!((report.too_small, report.joined), (0, 2), "{report:?}");
    }

    #[test]
    fn a_short_stair_along_a_fence_is_not_drawn_as_path() {
        // The fence runs along the stair and turns away before the paths: as a stair the two
        // may touch (ISSprOM 532), as a path they would keep no gap.
        let set = isom();
        let stair = line(&set, "532", &[(10.0, 0.0), (11.2, 0.0)]);
        let fence = line(
            &set,
            "518",
            &[(10.0, 5.0), (10.0, 0.3), (11.2, 0.3), (11.2, 5.0)],
        );
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (10.0, 0.0)]),
            stair.clone(),
            line(&set, "505", &[(11.2, 0.0), (20.0, 0.0)]),
            fence.clone(),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(of_code(&set, &out, "532"), vec![&stair], "{out:?}");
        assert_eq!(of_code(&set, &out, "518"), vec![&fence], "{out:?}");
        assert_eq!(of_code(&set, &out, "505").len(), 2, "{out:?}");
        assert_eq!(report.apart_left, 0.0, "{report:?}");
    }

    #[test]
    fn a_short_stair_at_a_dead_end_goes() {
        let set = isom();
        let path = line(&set, "505", &[(0.0, 0.0), (10.0, 0.0)]);
        let objects = vec![path.clone(), line(&set, "532", &[(10.0, 0.0), (10.3, 0.0)])];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![path]);
        assert_eq!(report.too_small, 1, "{report:?}");
    }

    #[test]
    fn two_short_stairs_do_not_keep_each_other() {
        // The end of the second touches the middle of the first; no way that stays is near.
        let set = isom();
        let long = line(&set, "505", &[(0.0, 20.0), (20.0, 20.0)]);
        let objects = vec![
            line(&set, "532", &[(0.0, 0.0), (0.4, 0.0)]),
            line(&set, "532", &[(0.2, 0.0), (0.2, 0.4)]),
            long.clone(),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(out, vec![long]);
        assert_eq!(report.too_small, 2, "{report:?}");
    }

    #[test]
    fn pieces_of_a_stair_are_joined_before_the_minimum() {
        let set = isom();
        let objects = vec![
            line(&set, "532", &[(0.0, 0.0), (0.8, 0.0)]),
            line(&set, "532", &[(0.8, 0.0), (1.6, 0.0)]),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(out.len(), 1, "{out:?}");
        assert!((length_mm(&out[0]) - 1.6).abs() < 1e-9, "{out:?}");
        assert_eq!((report.joined, report.too_small), (1, 0), "{report:?}");
    }

    #[test]
    fn a_fence_along_stairs_stays_whole() {
        let set = isom();
        let objects = vec![
            line(&set, "532", &[(0.0, 0.0), (10.0, 0.0)]),
            line(&set, "518", &[(0.0, 0.3), (10.0, 0.3)]),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
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
    fn carriageways_and_tracks_beyond_the_gap_are_left_alone() {
        let set = isom();
        let objects = vec![
            line(&set, "502", &[(0.0, 0.0), (30.0, 0.0)]),
            line(&set, "502", &[(0.0, 1.0), (30.0, 1.0)]),
            line(&set, "509", &[(0.0, 20.0), (30.0, 20.0)]),
            line(&set, "509", &[(0.0, 21.0), (30.0, 21.0)]),
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
        assert_eq!(report.apart_left, 0.0, "{report:?}");
    }

    /// The first code of `codes` the set has.
    fn first_code<'a>(set: &SymbolSet, codes: &[&'a str]) -> &'a str {
        codes
            .iter()
            .copied()
            .find(|c| set.id_for_code(c).is_some())
            .unwrap_or_else(|| panic!("none of {codes:?} in {}", set.name))
    }

    #[test]
    fn the_school_and_sprint_sets_do_not_fail() {
        // ISSprOM has no 502 to 505 and the school set no 104: give way runs on what they have.
        for (set, kind, scale) in [
            (SymbolSet::school_2019(), MapKind::Schoolyard, 1000),
            (SymbolSet::issprom_2019(), MapKind::Sprint, 4000),
        ] {
            let path = first_code(&set, &["505", "505.1", "501.6"]);
            let fence = first_code(&set, &["518", "516", "515"]);
            let mark = first_code(&set, &["526", "531", "528"]);
            let objects = vec![
                line(&set, path, &[(0.0, 0.0), (20.0, 0.0)]),
                line(&set, fence, &[(0.0, 0.1), (20.0, 0.1)]),
                area(&set, "521", (0.0, 5.0), (10.0, 10.0)),
                point(&set, mark, (5.0, 7.5)),
            ];
            let measured = |objects: &[MapObject]| -> usize {
                let refs: Vec<&MapObject> = objects.iter().collect();
                conflicts(&Footprints::of(&refs, &set), kind, scale)
                    .iter()
                    .filter(|c| relation(c.classes.0, c.classes.1) == Relation::Apart)
                    .count()
            };
            assert!(measured(&objects) >= 2, "{}", set.name);
            let (out, report) = train(objects, None, kind, &georef_at(scale), &set);
            assert_eq!(measured(&out), 0, "{}: {out:?}", set.name);
            assert_eq!(report.gave_way, 2, "{}: {report:?}", set.name);
            assert_eq!(report.apart_left, 0.0, "{}: {report:?}", set.name);
        }
    }

    /// `object` with one more tag.
    pub(super) fn tagged(mut object: MapObject, key: &str, value: &str) -> MapObject {
        object.tags.push((key.to_string(), value.to_string()));
        object
    }

    /// A tree cover of 1 m cells over the paper rectangle from `min` to `max` (mm) and 20 m
    /// round it: `share` of the paper position (mm) of each cell centre.
    fn cover(
        georef: &Georeferencing,
        min: (f64, f64),
        max: (f64, f64),
        share: impl Fn(f64, f64) -> f32,
    ) -> Raster<f32> {
        let corners = [
            (min.0, min.1),
            (max.0, min.1),
            (max.0, max.1),
            (min.0, max.1),
        ]
        .map(|p| georef.map_to_projected(at(p)));
        let west = corners.iter().map(|p| p.easting).fold(f64::MAX, f64::min) - 20.0;
        let east = corners.iter().map(|p| p.easting).fold(f64::MIN, f64::max) + 20.0;
        let south = corners.iter().map(|p| p.northing).fold(f64::MAX, f64::min) - 20.0;
        let north = corners.iter().map(|p| p.northing).fold(f64::MIN, f64::max) + 20.0;
        let (west, north) = (west.floor(), north.ceil());
        let grid = GridSpec {
            west,
            north,
            pixel_size: 1.0,
            width: (east - west).ceil() as u32,
            height: (north - south).ceil() as u32,
        };
        let mut raster = Raster::filled(grid, 0.0f32);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let centre = georef
                    .projected_to_map(ProjectedPoint {
                        easting: west + f64::from(x) + 0.5,
                        northing: north - f64::from(y) - 0.5,
                    })
                    .unwrap();
                let (x_mm, y_mm) = (f64::from(centre.x) / 1000.0, f64::from(centre.y) / 1000.0);
                raster.set(x, y, share(x_mm, y_mm));
            }
        }
        raster
    }

    fn run_with(
        set: &SymbolSet,
        objects: Vec<MapObject>,
        tree_cover: &Raster<f32>,
    ) -> (Vec<MapObject>, TrainingReport) {
        train(
            objects,
            Some(tree_cover),
            MapKind::Forest,
            &georef_at(SCALE),
            set,
        )
    }

    /// Area of an area object, mm² on paper.
    pub(super) fn area_mm2(object: &MapObject) -> f64 {
        object_polygon(object).unwrap().unsigned_area() / 1e6
    }

    /// The land use tag of a park, as `build/out_of_bounds.rs` writes it.
    const PARK: (&str, &str) = (
        "freemapper:nutzart",
        "Sport-, Freizeit- und Erholungsfläche / Park",
    );

    #[test]
    fn a_park_keeps_only_its_open_half() {
        // 10 mm on paper are 100 m at 1:10 000: trees on the west half, open ground on the east.
        let set = isom();
        let georef = georef_at(SCALE);
        let tree_cover = cover(&georef, (0.0, 0.0), (10.0, 10.0), |x, _| {
            if x < 5.0 { 1.0 } else { 0.0 }
        });
        for (key, value) in [PARK, ("freemapper:osm", "leisure=park; name=Stadtwald")] {
            let park = tagged(area(&set, "401", (0.0, 0.0), (10.0, 10.0)), key, value);
            let (out, report) = run_with(&set, vec![park], &tree_cover);
            assert_eq!(out.len(), 1, "{key}: {out:?}");
            let half = &out[0];
            assert_eq!(half.symbol_id, set.id_for_code("401").unwrap());
            assert!((area_mm2(half) - 50.0).abs() < 3.0, "{}", area_mm2(half));
            let west = half.coords.iter().map(|c| c.x).min().unwrap();
            assert!(west > 4_700, "{west}");
            assert!(half.tags.contains(&(key.to_string(), value.to_string())));
            assert!(
                half.tags
                    .contains(&("freemapper:generalized".into(), "tree cover".into())),
                "{:?}",
                half.tags
            );
            assert_eq!(report.open_land_cut, 1, "{report:?}");
        }
    }

    #[test]
    fn the_open_mask_has_the_cell_of_the_vegetation() {
        // 1 m cells of tree cover: open where x < 4 m; a 3 m cell is open when most of its
        // cells are.
        let grid = GridSpec {
            west: 0.0,
            north: 30.0,
            pixel_size: 1.0,
            width: 30,
            height: 30,
        };
        let mut tree_cover = Raster::filled(grid, 1.0f32);
        for y in 0..30 {
            for x in 0..4 {
                tree_cover.set(x, y, 0.0);
            }
        }
        let frame = Rect::new(Coord { x: 9.0, y: 9.0 }, Coord { x: 21.0, y: 21.0 });
        let mask = open_mask(&tree_cover, &frame);
        assert_eq!(mask.grid.pixel_size, CELL_M);
        // Aligned to the tree cover: the window starts on a 3 m step from its west edge.
        assert_eq!(mask.grid.west.rem_euclid(CELL_M), 0.0, "{:?}", mask.grid);
        let col = |e: f64| ((e - mask.grid.west) / CELL_M).floor() as u32;
        assert_eq!(mask.get(col(1.5), 5), Some(VegetationClass::Open));
        // 3 to 6 m: one open column of three.
        assert_eq!(mask.get(col(4.5), 5), None);
        assert_eq!(mask.get(col(10.5), 5), None);
    }

    #[test]
    fn a_meadow_of_another_use_is_left() {
        let set = isom();
        let georef = georef_at(SCALE);
        let tree_cover = cover(&georef, (0.0, 0.0), (10.0, 10.0), |_, _| 1.0);
        let meadow = tagged(
            area(&set, "401", (0.0, 0.0), (10.0, 10.0)),
            "freemapper:nutzart",
            "Landwirtschaft / Grünland",
        );
        let rough = tagged(area(&set, "403", (0.0, 0.0), (10.0, 10.0)), PARK.0, PARK.1);
        let objects = vec![meadow, rough];
        let (out, report) = run_with(&set, objects.clone(), &tree_cover);
        assert_eq!(out, objects);
        assert_eq!(report.open_land_cut, 0, "{report:?}");
    }

    #[test]
    fn without_tree_cover_nothing_changes() {
        let set = isom();
        let park = tagged(area(&set, "401", (0.0, 0.0), (10.0, 10.0)), PARK.0, PARK.1);
        let (out, report) = run(&set, vec![park.clone()]);
        assert_eq!(out, vec![park]);
        assert_eq!(report, TrainingReport::default());
    }

    #[test]
    fn a_park_without_trees_stays_as_it_is() {
        let set = isom();
        let georef = georef_at(SCALE);
        let tree_cover = cover(&georef, (0.0, 0.0), (10.0, 10.0), |_, _| f32::NAN);
        let park = tagged(area(&set, "401", (0.0, 0.0), (10.0, 10.0)), PARK.0, PARK.1);
        let (out, report) = run_with(&set, vec![park.clone()], &tree_cover);
        assert_eq!(out, vec![park]);
        assert_eq!(report.open_land_cut, 0, "{report:?}");
    }

    #[test]
    fn trees_beside_a_park_leave_it_whole() {
        // Trees all round a park of 200 m × 200 m and none in it: the cells along its edge step
        // across it, which is no tree area. ISOM 1:10 000 and ISSprOM 1:4000.
        for (set, kind, scale, side) in [
            (isom(), MapKind::Forest, SCALE, 20.0),
            (SymbolSet::issprom_2019(), MapKind::Sprint, 4000, 50.0),
        ] {
            let georef = georef_at(scale);
            let inside = |v: f64| (0.0..=side).contains(&v);
            let tree_cover = cover(&georef, (0.0, 0.0), (side, side), |x, y| {
                if inside(x) && inside(y) { 0.0 } else { 1.0 }
            });
            let park = tagged(area(&set, "401", (0.0, 0.0), (side, side)), PARK.0, PARK.1);
            let (out, report) = train(vec![park.clone()], Some(&tree_cover), kind, &georef, &set);
            assert_eq!(out, vec![park], "{}", set.name);
            assert_eq!(report.open_land_cut, 0, "{}: {report:?}", set.name);
        }
    }

    #[test]
    fn a_glade_below_the_minimum_goes() {
        // A glade of 10 m × 10 m in a wooded park; 1.5 mm × 1.5 mm at 150 % are 506 m².
        let set = isom();
        let georef = georef_at(SCALE);
        let tree_cover = cover(&georef, (0.0, 0.0), (10.0, 10.0), |x, y| {
            if (4.5..5.5).contains(&x) && (4.5..5.5).contains(&y) {
                0.0
            } else {
                0.9
            }
        });
        let park = tagged(area(&set, "401", (0.0, 0.0), (10.0, 10.0)), PARK.0, PARK.1);
        let (out, report) = run_with(&set, vec![park], &tree_cover);
        assert_eq!(out, Vec::new());
        assert_eq!(report.open_land_cut, 1, "{report:?}");
        assert_eq!(report.too_small, 0, "{report:?}");
    }

    #[test]
    fn the_report_counts_what_is_left() {
        let set = isom();
        // Two paths side by side merge; a path beside a road, outside its drawn width, is left.
        let road = line(&set, "503", &[(0.0, 10.0), (20.0, 10.0)]);
        let path = line(&set, "505", &[(0.0, 10.4), (20.0, 10.4)]);
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "505", &[(0.0, 0.3), (20.0, 0.3)]),
            road.clone(),
            path.clone(),
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(out.len(), 3, "{out:?}");
        assert_eq!(&out[1..], [road, path]);
        assert_eq!(report.merged, 1, "{report:?}");
        assert!(report.gap_left + report.overlap_left > 0.0, "{report:?}");
    }
}
