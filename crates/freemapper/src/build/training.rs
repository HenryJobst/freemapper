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
//!    stay ([`clear_olive`]);
//! 5. what keeps no gap to another object ([`conflicts`], the pairs that keep apart) gives way:
//!    it is cut out there, or dropped when more than half of it is in conflict;
//! 6. lines and areas below the norm's minimum size go.
//!
//! What a step drops is gone from the training map; nothing is tagged hidden. Lines of one
//! network that run side by side are left to the second half and only counted in the report.

use std::collections::{BTreeMap, BTreeSet};

use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Contains as _, Intersects as _,
    MapCoords as _, unary_union,
};
use geo_types::{Coord, LineString, MultiPolygon, Point, Polygon, Rect};

use crate::build::candidates::MIN_LINE_MM;
use crate::build::conflicts::{
    Class, Classes, Conflict, Footprints, Grade, Relation, TOLERANCE_MM, conflicts, gap,
    meeting_points, rank, relation,
};
use crate::build::dedup::tag_holds;
use crate::build::geometry::{distance_to_segment, object_polygon, xy};
use crate::build::line_cuts::cut_lines_at_ways;
use crate::build::line_join::{TOUCH_M, join_lines};
use crate::build::open_land::MIN_PIECE_SIDE_MM;
use crate::build::out_of_bounds::{area_object, short_side};
use crate::build::points::{FOREST, PAVED_AREA_CODES, SCATTERED_TREES, projected};
use crate::derive::vegetation::{VegetationClass, majority, min_area_m2, outlines};
use crate::georef::Georeferencing;
use crate::measure::{
    PaperMm, PaperMm2, SymbolMm, SymbolMm2, SymbolScale, map_units_per_metre, metres_per_mm,
};
use crate::omap::coord::FLAG_CLOSE;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::raster::{GridSpec, Raster};
use crate::render::geometry::{length, parts};

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
/// How far inside the olive green an object still lies on its edge.
pub const OLIVE_EDGE_MM: PaperMm = PaperMm(0.4);
/// The beginning drawn of a way that allows no crossing: this, or the symbol's minimum length if longer.
pub const WAY_STUB_MM: PaperMm = PaperMm(1.0);
/// Area that shall not be entered: the olive green of [`clear_olive`].
const OLIVE_CODE: &str = "520";
/// Canopy: roofs, carports and the passages through buildings.
const CANOPY_CODE: &str = "522";
/// Impassable wall and impassable fence: they bar a way into the olive green.
const IMPASSABLE_CODES: [&str; 2] = ["515", "518"];
/// A canopy whose `freemapper:alkis` names one of these is a passage through a building
/// (`build/state.rs`, `build/berlin.rs`).
const PASSAGE_NAMES: [&str; 2] = ["Durchfahrt", "Arkade"];
/// A canopy whose `freemapper:osm` holds one of these is a passage from OSM (the 522 rules of
/// `assets/rules/osm-candidates.toml`).
const OSM_PASSAGES: [&str; 2] = ["tunnel=building_passage", "covered=*"];

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
    (report.gave_way, report.cut) = give_way(&mut objects, &classes, kind, georef, symbol_set);
    report.too_small = drop_too_small(&mut objects, &classes, kind, georef, symbol_set);
    (report.overlap_left, report.gap_left, report.apart_left) =
        left(&objects, kind, georef, symbol_set);
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

/// Whether `object` is the open land of a park: from the land use of recreation grounds, or an
/// OSM park.
fn is_park(object: &MapObject) -> bool {
    object.tags.iter().any(|(k, v)| {
        (k == "freemapper:nutzart" && v.starts_with(PARK_NUTZART))
            || (k == "freemapper:osm" && tag_holds(v, OSM_PARK))
    })
}

/// Step 3: open land (401) of a park ([`is_park`]) keeps only its open ground: the cells of
/// `tree_cover` with a share below [`OPEN_BELOW_SHARE`] (and those it does not know), smoothed
/// with [`majority`] and outlined as the vegetation classes are. Pieces below the vegetation's
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
/// shrunk by one cell: the cells along its edge step across it, and the outline of trees beside
/// the park follows the steps.
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
    let core = area.buffer(-tree_cover.grid.pixel_size);
    let lost = core
        .difference(&MultiPolygon(pieces.clone()))
        .unsigned_area();
    (lost >= min_m2).then_some(pieces)
}

/// The cells of `tree_cover` round `frame` (map CRS) and [`COVER_MARGIN_M`] beyond it: open
/// ground (`Some(Open)`) where the share is below [`OPEN_BELOW_SHARE`], unknown (`NaN`) or the
/// raster has no cell, else `None`.
fn open_mask(tree_cover: &Raster<f32>, frame: &Rect<f64>) -> Raster<Option<VegetationClass>> {
    let grid = tree_cover.grid;
    let size = grid.pixel_size;
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
    let mut mask = Raster::filled(window, Some(VegetationClass::Open));
    for y in 0..window.height {
        let yy = y0 + i64::from(y);
        if yy < 0 || yy >= i64::from(grid.height) {
            continue;
        }
        for x in 0..window.width {
            let xx = x0 + i64::from(x);
            if xx >= 0
                && xx < i64::from(grid.width)
                && tree_cover.get(xx as u32, yy as u32) >= OPEN_BELOW_SHARE
            {
                mask.set(x, y, None);
            }
        }
    }
    mask
}

/// Step 4: inside olive green the training map draws no details (spec 2026-11-14, "Inside olive
/// green"). The olive green is the union of the [`OLIVE_CODE`] objects; what lies within
/// [`OLIVE_EDGE_MM`] of its edge is on the edge and stays, so the step works on the union shrunk
/// by that much ([`Inner`]). Text, sheet objects, north lines, buildings, water, watercourses,
/// contours, railways, the olive green itself and the canopies stay. Areas lose their part
/// inside; pieces below the minimum of [`open_ground`] go. Points inside go. The other lines are
/// cut; pieces below their minimum length go. Ways follow [`olive_ways`]. A passage through a
/// building ([`is_passage`]) that reaches inside goes unless a way that stays runs through it.
/// Returns how many objects went or were cut.
fn clear_olive(
    objects: &mut Vec<MapObject>,
    classes: &Classes,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let Some(olive_id) = symbol_set.id_for_code(OLIVE_CODE) else {
        return 0;
    };
    let olive: Vec<Polygon<f64>> = objects
        .iter()
        .filter(|o| o.symbol_id == olive_id)
        .filter_map(object_polygon)
        .map(|p| in_mm(&p))
        .collect();
    if olive.is_empty() {
        return 0;
    }
    let inner = Inner::new(unary_union(&olive).buffer(-OLIVE_EDGE_MM.0));
    if inner.polygons.is_empty() {
        return 0;
    }
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let min_area = min_area_m2(scale) / metres_per_mm(scale_denominator).powi(2);
    let min_side = MIN_PIECE_SIDE_MM.on_paper(scale).0;
    let canopy = symbol_set.id_for_code(CANOPY_CODE);
    let is_way = |o: &MapObject| matches!(classes.class(o.symbol_id), Class::Road | Class::Path);

    let mut replaced = olive_ways(objects, &inner, classes, georef, symbol_set);
    let passages: Vec<(usize, Polygon<f64>)> = objects
        .iter()
        .enumerate()
        .filter(|(_, o)| Some(o.symbol_id) == canopy && is_passage(o))
        .filter_map(|(i, o)| Some((i, in_mm(&object_polygon(o)?))))
        .filter(|(_, p)| inner.intersects(p))
        .collect();
    if !passages.is_empty() {
        let ways: Vec<(Rect<f64>, LineString<f64>)> = objects
            .iter()
            .enumerate()
            .filter(|(_, o)| is_way(o))
            .flat_map(|(i, o)| match replaced.get(&i) {
                Some(pieces) => pieces.iter().collect::<Vec<_>>(),
                None => vec![o],
            })
            .flat_map(line_parts)
            .filter_map(|(line, _)| {
                let way = LineString::new(line.points);
                Some((way.bounding_rect()?, way))
            })
            .collect();
        for (i, passage) in passages {
            let Some(frame) = passage.bounding_rect() else {
                continue;
            };
            let through = ways
                .iter()
                .any(|(f, way)| f.intersects(&frame) && way.intersects(&passage));
            if !through {
                replaced.insert(i, Vec::new());
            }
        }
    }
    for (i, object) in objects.iter().enumerate() {
        if replaced.contains_key(&i)
            || Some(object.symbol_id) == canopy
            || is_way(object)
            || stays_in_olive(object, classes, olive_id)
        {
            continue;
        }
        let class = classes.class(object.symbol_id);
        let change = match object.kind {
            MapObjectKind::Point { .. } => object
                .coords
                .first()
                .is_some_and(|c| {
                    let (x, y) = xy(c);
                    inner.contains(Coord {
                        x: x / 1000.0,
                        y: y / 1000.0,
                    })
                })
                .then(Vec::new),
            MapObjectKind::Text { .. } => None,
            MapObjectKind::Path
                if class == Class::OtherArea
                    || (class == Class::Other
                        && !classes.is_line(object.symbol_id)
                        && is_closed(object)) =>
            {
                object_polygon(object)
                    .and_then(|p| inner.cut(&in_mm(&p)))
                    .map(|pieces| {
                        let side = (min_area.sqrt() - TOLERANCE_MM).max(0.0);
                        pieces
                            .iter()
                            .filter(|p| {
                                p.unsigned_area() >= side * side
                                    && short_side(p) >= min_side - TOLERANCE_MM
                            })
                            .map(|p| area_like(object, p))
                            .collect()
                    })
            }
            MapObjectKind::Path => {
                let parts = line_parts(object);
                let runs: Vec<Vec<Run>> = parts.iter().map(|(line, _)| inner.runs(line)).collect();
                runs.iter().flatten().any(|r| r.inside).then(|| {
                    let keep: Vec<Vec<(f64, f64)>> = runs
                        .iter()
                        .map(|runs| {
                            runs.iter()
                                .filter(|r| !r.inside)
                                .map(|r| (r.from, r.to))
                                .collect()
                        })
                        .collect();
                    let min = min_length(object, classes, symbol_set, scale).0;
                    line_pieces(object, &parts, &keep, min)
                })
            }
        };
        if let Some(pieces) = change {
            replaced.insert(i, pieces);
        }
    }
    let changed = replaced.len();
    let before = std::mem::take(objects);
    for (i, object) in before.into_iter().enumerate() {
        match replaced.remove(&i) {
            Some(pieces) => objects.extend(pieces),
            None => objects.push(object),
        }
    }
    changed
}

/// Whether `object` stays inside olive green whatever lies there: text, sheet objects, north
/// lines, buildings, water, watercourses, contours, railways and the olive green itself.
fn stays_in_olive(object: &MapObject, classes: &Classes, olive_id: u32) -> bool {
    use Class::*;
    use freemapper_contract::tags::{SOURCE_KEY, SOURCE_NORTH_LINES, SOURCE_SHEET};
    matches!(object.kind, MapObjectKind::Text { .. })
        || object.symbol_id == olive_id
        || object
            .tags
            .iter()
            .any(|(k, v)| k == SOURCE_KEY && (v == SOURCE_SHEET || v == SOURCE_NORTH_LINES))
        || matches!(
            classes.class(object.symbol_id),
            Building | WaterArea | BankLine | Watercourse | Contour | Railway
        )
}

/// Whether the canopy `object` is a passage through a building: from the cadastre
/// ([`PASSAGE_NAMES`]) or from OSM ([`OSM_PASSAGES`]). Roofs and carports are not.
fn is_passage(object: &MapObject) -> bool {
    object.tags.iter().any(|(k, v)| {
        (k == "freemapper:alkis" && PASSAGE_NAMES.iter().any(|name| v.contains(name)))
            || (k == "freemapper:osm" && OSM_PASSAGES.iter().any(|tag| tag_holds(v, tag)))
    })
}

/// The ways (Road, Path) inside `inner` (step 4). Their parts inside are taken together as they
/// join: an end within [`TOUCH_M`] of another part. Such a group is a crossing when it meets the
/// edge of `inner` at two places or more that are not barred (an end of a line inside is no
/// place); a crossing stays in full. Of a group that is no crossing, the parts inside go, except
/// the first [`WAY_STUB_MM`] (or the symbol's minimum length if longer) inward from each place
/// that is not barred. A place is barred when an impassable wall or fence ([`IMPASSABLE_CODES`])
/// crosses the way within [`OLIVE_EDGE_MM`] of the edge of `inner` and within twice that along
/// the way from the place; the way then ends at the barrier. Pieces below the minimum length go.
/// Returns the pieces of the ways that changed, by index in `objects`.
fn olive_ways(
    objects: &[MapObject],
    inner: &Inner,
    classes: &Classes,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> BTreeMap<usize, Vec<MapObject>> {
    struct WayLine {
        object: usize,
        line: Polyline,
        closed: bool,
        runs: Vec<Run>,
    }
    struct InsidePart {
        line: usize,
        from: f64,
        to: f64,
    }
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let edge = OLIVE_EDGE_MM.0;
    let touch = PaperMm::of_metres(TOUCH_M, scale_denominator).0 + TOLERANCE_MM;
    let lines: Vec<WayLine> = objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            matches!(o.kind, MapObjectKind::Path)
                && matches!(classes.class(o.symbol_id), Class::Road | Class::Path)
        })
        .flat_map(|(object, o)| {
            line_parts(o).into_iter().map(move |(line, closed)| {
                let runs = inner.runs(&line);
                WayLine {
                    object,
                    line,
                    closed,
                    runs,
                }
            })
        })
        .collect();
    let parts: Vec<InsidePart> = lines
        .iter()
        .enumerate()
        .flat_map(|(k, l)| {
            l.runs.iter().filter(|r| r.inside).map(move |r| InsidePart {
                line: k,
                from: r.from,
                to: r.to,
            })
        })
        .collect();
    if parts.is_empty() {
        return BTreeMap::new();
    }

    // The groups of parts that join.
    let stretches: Vec<Vec<Coord<f64>>> = parts
        .iter()
        .map(|p| lines[p.line].line.between(p.from, p.to))
        .collect();
    let frames: Vec<Option<Rect<f64>>> = stretches
        .iter()
        .map(|s| LineString::new(s.clone()).bounding_rect())
        .collect();
    let mut parent: Vec<usize> = (0..parts.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut join = |a: usize, b: usize| {
        let (a, b) = (root(&mut parent, a), root(&mut parent, b));
        parent[a.max(b)] = a.min(b);
    };
    for a in 0..parts.len() {
        for b in 0..parts.len() {
            let near = match (frames[a], frames[b]) {
                (Some(fa), Some(fb)) => grown(fa, touch).intersects(&fb),
                _ => false,
            };
            let ends = [stretches[a][0], stretches[a][stretches[a].len() - 1]];
            if a != b
                && near
                && ends
                    .iter()
                    .any(|&end| distance_to_line(end, &stretches[b]) <= touch)
            {
                join(a, b);
            }
        }
    }
    // A closed line that starts inside: its first and last part are one.
    for (k, l) in lines.iter().enumerate() {
        let total = l.line.length();
        let first = parts.iter().position(|p| p.line == k && p.from <= 0.0);
        let last = parts
            .iter()
            .position(|p| p.line == k && p.to >= total - 1e-9);
        if l.closed
            && let (Some(first), Some(last)) = (first, last)
        {
            join(first, last);
        }
    }

    // The places where a part meets the edge, and whether each is barred: `Some(at)` with the
    // crossing of the barrier along the line.
    let impassable: Vec<u32> = IMPASSABLE_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let barriers: Vec<(Rect<f64>, Vec<Coord<f64>>)> = objects
        .iter()
        .filter(|o| impassable.contains(&o.symbol_id) && matches!(o.kind, MapObjectKind::Path))
        .flat_map(line_parts)
        .filter_map(|(line, _)| {
            let frame = LineString::new(line.points.clone()).bounding_rect()?;
            Some((frame, line.points))
        })
        .collect();
    let mut crossings: BTreeMap<usize, Vec<f64>> = BTreeMap::new();
    for part in &parts {
        crossings
            .entry(part.line)
            .or_insert_with(|| barrier_crossings(&lines[part.line].line, &barriers));
    }
    let place = |line: usize, at: f64| -> Option<Option<f64>> {
        let way = &lines[line].line;
        if at <= TOLERANCE_MM || at >= way.length() - TOLERANCE_MM {
            return None;
        }
        let barred = crossings[&line]
            .iter()
            .copied()
            .filter(|&q| {
                (q - at).abs() <= 2.0 * edge + TOLERANCE_MM
                    && inner.boundary_distance(way.point(q), edge + TOLERANCE_MM)
                        <= edge + TOLERANCE_MM
            })
            .min_by(|a, b| (a - at).abs().total_cmp(&(b - at).abs()));
        Some(barred)
    };
    let places: Vec<[Option<Option<f64>>; 2]> = parts
        .iter()
        .map(|p| [place(p.line, p.from), place(p.line, p.to)])
        .collect();
    let mut open_places: BTreeMap<usize, usize> = BTreeMap::new();
    for (i, ends) in places.iter().enumerate() {
        let open = ends.iter().filter(|e| matches!(e, Some(None))).count();
        *open_places.entry(root(&mut parent, i)).or_default() += open;
    }

    // What goes of each line: the parts of the groups that are no crossing, less the stubs.
    let mut removed: BTreeMap<usize, Vec<(f64, f64)>> = BTreeMap::new();
    for (i, part) in parts.iter().enumerate() {
        if open_places[&root(&mut parent, i)] >= 2 {
            continue;
        }
        let object = &objects[lines[part.line].object];
        let stub = WAY_STUB_MM
            .0
            .max(min_length(object, classes, symbol_set, scale).0);
        let from = match places[i][0] {
            Some(None) => part.from + stub,
            Some(Some(barrier)) => part.from.min(barrier),
            None => part.from,
        };
        let to = match places[i][1] {
            Some(None) => part.to - stub,
            Some(Some(barrier)) => part.to.max(barrier),
            None => part.to,
        };
        if to > from {
            removed.entry(part.line).or_default().push((from, to));
        }
    }

    let mut changed: BTreeMap<usize, Vec<MapObject>> = BTreeMap::new();
    let mut by_object: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for &line in removed.keys() {
        by_object.entry(lines[line].object).or_default().push(line);
    }
    for (object_index, _) in by_object {
        let object = &objects[object_index];
        let own: Vec<usize> = (0..lines.len())
            .filter(|&k| lines[k].object == object_index)
            .collect();
        let parts: Vec<(Polyline, bool)> = own
            .iter()
            .map(|&k| (lines[k].line.clone(), lines[k].closed))
            .collect();
        let keep: Vec<Vec<(f64, f64)>> = own
            .iter()
            .map(|k| {
                let total = lines[*k].line.length();
                complement(removed.get(k).map_or(&[][..], Vec::as_slice), total)
            })
            .collect();
        let min = min_length(object, classes, symbol_set, scale).0;
        changed.insert(object_index, line_pieces(object, &parts, &keep, min));
    }
    changed
}

/// The places along `line` (length from its start) where a line of `barriers` crosses it.
fn barrier_crossings(line: &Polyline, barriers: &[(Rect<f64>, Vec<Coord<f64>>)]) -> Vec<f64> {
    let mut found = Vec::new();
    for (i, w) in line.points.windows(2).enumerate() {
        let segment = geo_types::Line::new(w[0], w[1]);
        let frame = segment.bounding_rect();
        for (barrier_frame, barrier) in barriers {
            if !barrier_frame.intersects(&frame) {
                continue;
            }
            for b in barrier.windows(2) {
                let hits = match line_intersection(segment, geo_types::Line::new(b[0], b[1])) {
                    None => continue,
                    Some(LineIntersection::SinglePoint { intersection, .. }) => {
                        [intersection, intersection]
                    }
                    Some(LineIntersection::Collinear { intersection }) => {
                        [intersection.start, intersection.end]
                    }
                };
                found.extend(
                    hits.iter()
                        .map(|hit| line.at[i] + (hit.x - w[0].x).hypot(hit.y - w[0].y)),
                );
            }
        }
    }
    found
}

/// The stretches of `0..total` that `removed` leaves.
fn complement(removed: &[(f64, f64)], total: f64) -> Vec<(f64, f64)> {
    let mut removed = removed.to_vec();
    removed.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut keep = Vec::new();
    let mut at = 0.0;
    for (from, to) in removed {
        if from > at {
            keep.push((at, from));
        }
        at = f64::max(at, to);
    }
    if at < total {
        keep.push((at, total));
    }
    keep
}

/// The distance from `point` to the polyline `line`.
fn distance_to_line(point: Coord<f64>, line: &[Coord<f64>]) -> f64 {
    line.windows(2)
        .map(|w| distance_to_segment((point.x, point.y), (w[0].x, w[0].y), (w[1].x, w[1].y)))
        .fold(f64::INFINITY, f64::min)
}

/// `rect` grown by `by` on every side.
fn grown(rect: Rect<f64>, by: f64) -> Rect<f64> {
    Rect::new(
        Coord {
            x: rect.min().x - by,
            y: rect.min().y - by,
        },
        Coord {
            x: rect.max().x + by,
            y: rect.max().y + by,
        },
    )
}

/// A polygon in map coordinates in paper mm.
fn in_mm(polygon: &Polygon<f64>) -> Polygon<f64> {
    polygon.map_coords(|c| Coord {
        x: c.x / 1000.0,
        y: c.y / 1000.0,
    })
}

/// The area `polygon` (paper mm) drawn as `object`.
fn area_like(object: &MapObject, polygon: &Polygon<f64>) -> MapObject {
    let polygon = in_map_units(polygon);
    let mut coords = Vec::new();
    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
        let points: Vec<(i32, i32)> = ring.coords().map(|c| (c.x as i32, c.y as i32)).collect();
        coords.extend(closed_ring(&points));
    }
    MapObject {
        symbol_id: object.symbol_id,
        kind: MapObjectKind::Path,
        coords,
        tags: object.tags.clone(),
    }
}

/// A polyline in paper mm with the length along it at each point.
#[derive(Debug, Clone)]
struct Polyline {
    points: Vec<Coord<f64>>,
    at: Vec<f64>,
}

impl Polyline {
    fn new(points: Vec<Coord<f64>>) -> Self {
        let mut at = Vec::with_capacity(points.len());
        let mut total = 0.0;
        for (i, p) in points.iter().enumerate() {
            if i > 0 {
                total += (p.x - points[i - 1].x).hypot(p.y - points[i - 1].y);
            }
            at.push(total);
        }
        Self { points, at }
    }

    fn length(&self) -> f64 {
        self.at.last().copied().unwrap_or(0.0)
    }

    /// The point `s` along the line.
    fn point(&self, s: f64) -> Coord<f64> {
        let i = self
            .at
            .partition_point(|&a| a < s)
            .clamp(1, self.points.len() - 1);
        let (a, b) = (self.points[i - 1], self.points[i]);
        let span = self.at[i] - self.at[i - 1];
        let t = if span > 0.0 {
            ((s - self.at[i - 1]) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Coord {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
        }
    }

    /// The line from `from` to `to` along it.
    fn between(&self, from: f64, to: f64) -> Vec<Coord<f64>> {
        let mut points = vec![self.point(from)];
        points.extend(
            self.points
                .iter()
                .zip(&self.at)
                .filter(|&(_, &a)| a > from + 1e-9 && a < to - 1e-9)
                .map(|(&p, _)| p),
        );
        points.push(self.point(to));
        points
    }
}

/// The parts of the line `object` (Béziers flattened), paper mm, and whether each is closed.
fn line_parts(object: &MapObject) -> Vec<(Polyline, bool)> {
    parts(&object.coords)
        .into_iter()
        .filter(|p| p.points.len() >= 2)
        .map(|p| {
            let points = p.points.iter().map(|&(x, y)| Coord { x, y }).collect();
            (Polyline::new(points), p.closed)
        })
        .collect()
}

/// The pieces of the line `object` that keep the stretches `keep` of each of its `parts`; a
/// closed part cut on both sides of its start is joined there. Pieces of a cut part shorter than
/// `min` (mm on paper) go.
fn line_pieces(
    object: &MapObject,
    parts: &[(Polyline, bool)],
    keep: &[Vec<(f64, f64)>],
    min: f64,
) -> Vec<MapObject> {
    let mut pieces = Vec::new();
    for ((line, closed), keep) in parts.iter().zip(keep) {
        let total = line.length();
        let whole = keep.len() == 1 && keep[0].0 <= 0.0 && keep[0].1 >= total;
        let mut stretches: Vec<Vec<Coord<f64>>> =
            keep.iter().map(|&(a, b)| line.between(a, b)).collect();
        if *closed && keep.len() > 1 && keep[0].0 <= 0.0 && keep[keep.len() - 1].1 >= total {
            let first = stretches.remove(0);
            let last = stretches.last_mut().expect("two stretches or more");
            last.extend_from_slice(&first[1..]);
        }
        for stretch in stretches {
            let drawn = Polyline::new(stretch);
            if !whole && drawn.length() < min - TOLERANCE_MM {
                continue;
            }
            let points: Vec<(i32, i32)> = drawn
                .points
                .iter()
                .map(|c| ((c.x * 1000.0).round() as i32, (c.y * 1000.0).round() as i32))
                .collect();
            pieces.push(MapObject {
                symbol_id: object.symbol_id,
                kind: MapObjectKind::Path,
                coords: if whole && *closed {
                    closed_ring(&points)
                } else {
                    points.iter().map(|&(x, y)| MapCoord::new(x, y)).collect()
                },
                tags: object.tags.clone(),
            });
        }
    }
    pieces
}

/// A stretch of a line, by the length along it, inside or outside [`Inner`].
#[derive(Debug, Clone, Copy)]
struct Run {
    from: f64,
    to: f64,
    inside: bool,
}

/// The olive green shrunk by its edge ([`OLIVE_EDGE_MM`]), paper mm (y down), each polygon with
/// its frame.
struct Inner {
    polygons: Vec<(Rect<f64>, Polygon<f64>)>,
}

impl Inner {
    fn new(union: MultiPolygon<f64>) -> Self {
        let polygons = union
            .0
            .into_iter()
            .filter(|p| p.unsigned_area() > 0.0)
            .filter_map(|p| Some((p.bounding_rect()?, p)))
            .collect();
        Self { polygons }
    }

    /// The polygons whose frame comes within `reach` of `frame`.
    fn near(&self, frame: Rect<f64>, reach: f64) -> impl Iterator<Item = &Polygon<f64>> {
        let frame = grown(frame, reach);
        self.polygons
            .iter()
            .filter(move |(f, _)| f.intersects(&frame))
            .map(|(_, p)| p)
    }

    fn contains(&self, c: Coord<f64>) -> bool {
        let point = Point(c);
        self.near(Rect::new(c, c), 0.0).any(|p| p.contains(&point))
    }

    fn intersects(&self, polygon: &Polygon<f64>) -> bool {
        polygon
            .bounding_rect()
            .is_some_and(|frame| self.near(frame, 0.0).any(|p| p.intersects(polygon)))
    }

    /// The distance from `c` to the edge of the polygons within `reach` of it (infinite when
    /// there is none).
    fn boundary_distance(&self, c: Coord<f64>, reach: f64) -> f64 {
        self.near(Rect::new(c, c), reach)
            .flat_map(|p| std::iter::once(p.exterior()).chain(p.interiors()))
            .map(|ring| distance_to_line(c, &ring.0))
            .fold(f64::INFINITY, f64::min)
    }

    /// The pieces of `polygon` (paper mm) outside, or `None` when no more than the tolerance of
    /// it lies inside.
    fn cut(&self, polygon: &Polygon<f64>) -> Option<Vec<Polygon<f64>>> {
        let near = MultiPolygon(self.near(polygon.bounding_rect()?, 0.0).cloned().collect());
        let area = MultiPolygon(vec![polygon.clone()]);
        (area.intersection(&near).unsigned_area() > TOLERANCE_MM * TOLERANCE_MM)
            .then(|| area.difference(&near).0)
    }

    /// The stretches of `line` inside and outside, in order; a stretch shorter than the
    /// tolerance takes the side of the one before it (the first one that of the next).
    fn runs(&self, line: &Polyline) -> Vec<Run> {
        let mut cuts = vec![0.0];
        for (i, w) in line.points.windows(2).enumerate() {
            let length = line.at[i + 1] - line.at[i];
            if length > 0.0 {
                let segment = geo_types::Line::new(w[0], w[1]);
                let frame = segment.bounding_rect();
                for polygon in self.near(frame, 0.0) {
                    let rings = std::iter::once(polygon.exterior()).chain(polygon.interiors());
                    for edge in rings.flat_map(|r| r.lines()) {
                        if !edge.bounding_rect().intersects(&frame) {
                            continue;
                        }
                        let hits = match line_intersection(segment, edge) {
                            None => continue,
                            Some(LineIntersection::SinglePoint { intersection, .. }) => {
                                [intersection, intersection]
                            }
                            Some(LineIntersection::Collinear { intersection }) => {
                                [intersection.start, intersection.end]
                            }
                        };
                        cuts.extend(hits.iter().map(|hit| {
                            line.at[i] + (hit.x - w[0].x).hypot(hit.y - w[0].y).min(length)
                        }));
                    }
                }
            }
            cuts.push(line.at[i + 1]);
        }
        cuts.sort_by(f64::total_cmp);
        cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let mut runs: Vec<Run> = cuts
            .windows(2)
            .map(|w| Run {
                from: w[0],
                to: w[1],
                inside: self.contains(line.point((w[0] + w[1]) / 2.0)),
            })
            .collect();
        for k in 0..runs.len() {
            if runs[k].to - runs[k].from < TOLERANCE_MM {
                let side = match k {
                    0 => runs.get(1).map(|r| r.inside),
                    _ => Some(runs[k - 1].inside),
                };
                runs[k].inside = side.unwrap_or(false);
            }
        }
        let mut merged: Vec<Run> = Vec::with_capacity(runs.len());
        for run in runs {
            match merged.last_mut() {
                Some(last) if last.inside == run.inside => last.to = run.to,
                _ => merged.push(run),
            }
        }
        merged
    }
}

/// Step 5: every object that keeps no gap to another object it must keep apart from gives way,
/// both grades (the training map keeps every gap). Losers go by ascending rank; a line with more
/// than half its length in conflict, a point and an area go whole, any other line is cut where it
/// is in conflict. A loser whose winner gives way itself in the round waits for the next round,
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

/// Step 6: a line shorter than its minimum length goes unless an end continues on another line
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

/// Step 7: the conflict length left on `objects`, mm on paper: overlap and gap of the pairs of
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
    fn tagged(mut object: MapObject, key: &str, value: &str) -> MapObject {
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
    fn area_mm2(object: &MapObject) -> f64 {
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
        let objects = vec![
            line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "505", &[(0.0, 0.3), (20.0, 0.3)]),
        ];
        let (out, report) = run(&set, objects.clone());
        assert_eq!(out, objects);
        assert!(report.gap_left + report.overlap_left > 0.0, "{report:?}");
    }

    /// The scale of the sprint tests.
    const SPRINT: u32 = 4000;

    /// Only the olive step on `objects`, a map of `set` at `scale`; returns how many went or
    /// were cut.
    fn olive_at(set: &SymbolSet, scale: u32, objects: Vec<MapObject>) -> (Vec<MapObject>, usize) {
        let mut objects = objects;
        let n = clear_olive(&mut objects, &Classes::of(set), &georef_at(scale), set);
        (objects, n)
    }

    fn olive(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, usize) {
        olive_at(set, SPRINT, objects)
    }

    /// A garden of 20 mm × 20 mm (80 m at 1:4000); its inner part starts 0.4 mm inside.
    fn garden(set: &SymbolSet) -> MapObject {
        area(set, "520", (0.0, 0.0), (20.0, 20.0))
    }

    /// The coordinates of a line, mm on paper.
    fn mm_of(object: &MapObject) -> Vec<(f64, f64)> {
        object
            .coords
            .iter()
            .map(|c| (f64::from(c.x) / 1000.0, f64::from(c.y) / 1000.0))
            .collect()
    }

    /// Whether `object` runs straight from `from` to `to` (mm, within 2 map units).
    fn runs_from_to(object: &MapObject, from: (f64, f64), to: (f64, f64)) -> bool {
        let points = mm_of(object);
        let near = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).hypot(a.1 - b.1) <= 0.002;
        points.len() >= 2 && near(points[0], from) && near(points[points.len() - 1], to)
    }

    #[test]
    fn a_fence_inside_goes_and_the_one_on_the_edge_stays() {
        let set = SymbolSet::issprom_2019();
        let edge = line(&set, "518", &[(-5.0, 0.0), (25.0, 0.0)]);
        let objects = vec![
            garden(&set),
            edge.clone(),
            line(&set, "518", &[(5.0, 10.0), (15.0, 10.0)]),
            line(&set, "516", &[(10.0, -10.0), (10.0, 10.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 2, "{out:?}");
        assert!(
            out.contains(&garden(&set)) && out.contains(&edge),
            "{out:?}"
        );
        assert_eq!(of_code(&set, &out, "518"), vec![&edge]);
        let across = of_code(&set, &out, "516");
        assert_eq!(across.len(), 1, "{out:?}");
        assert!(
            runs_from_to(across[0], (10.0, -10.0), (10.0, 0.4)),
            "{:?}",
            mm_of(across[0])
        );
    }

    #[test]
    fn a_fence_ring_round_a_corner_is_one_piece_after() {
        // Closed, starting outside: what is left of it runs on through its start.
        let set = SymbolSet::issprom_2019();
        let ring = MapObject {
            coords: closed_ring(&[(-5000, -5000), (5000, -5000), (5000, 5000), (-5000, 5000)]),
            ..line(&set, "516", &[])
        };
        let (out, n) = olive(&set, vec![garden(&set), ring]);
        assert_eq!(n, 1, "{out:?}");
        let fences = of_code(&set, &out, "516");
        assert_eq!(fences.len(), 1, "{out:?}");
        assert!(
            runs_from_to(fences[0], (0.4, 5.0), (5.0, 0.4)),
            "{:?}",
            mm_of(fences[0])
        );
        assert!((length_mm(fences[0]) - (40.0 - 2.0 * 4.6)).abs() < 0.01);
    }

    #[test]
    fn a_point_inside_goes() {
        let set = SymbolSet::issprom_2019();
        let on_edge = point(&set, "526", (10.0, 0.2));
        let outside = point(&set, "526", (10.0, 30.0));
        let objects = vec![
            garden(&set),
            point(&set, "526", (10.0, 10.0)),
            on_edge.clone(),
            outside.clone(),
        ];
        let (out, report) = train(objects, None, MapKind::Sprint, &georef_at(SPRINT), &set);
        assert_eq!(out, vec![garden(&set), on_edge, outside]);
        assert_eq!(report.inside_olive, 1, "{report:?}");
    }

    #[test]
    fn a_building_and_a_pond_inside_stay() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            area(&set, "521", (5.0, 5.0), (10.0, 10.0)),
            area(&set, "301", (12.0, 12.0), (16.0, 16.0)),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_contour_runs_through() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "101", &[(-5.0, 10.0), (25.0, 10.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn open_land_is_cut_at_the_edge() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            area(&set, "401", (-10.0, 5.0), (10.0, 15.0)),
            area(&set, "401", (5.0, 16.0), (7.0, 18.0)),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 2, "{out:?}");
        let open = of_code(&set, &out, "401");
        assert_eq!(open.len(), 1, "{out:?}");
        let east = open[0].coords.iter().map(|c| c.x).max().unwrap();
        assert!((east - 400).abs() <= 2, "{east}");
        assert!(
            (area_mm2(open[0]) - 104.0).abs() < 0.1,
            "{}",
            area_mm2(open[0])
        );
    }

    #[test]
    fn a_path_that_crosses_stays_in_full() {
        for (set, scale, path) in [
            (SymbolSet::issprom_2019(), SPRINT, "505.1"),
            (isom(), SCALE, "505"),
        ] {
            let objects = vec![
                garden(&set),
                line(&set, path, &[(-5.0, 10.0), (25.0, 10.0)]),
            ];
            assert_eq!(olive_at(&set, scale, objects.clone()), (objects, 0));
        }
    }

    #[test]
    fn a_spur_that_joins_a_crossing_inside_stays() {
        // The spur ends inside on the crossing path: one group with two open places.
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            line(&set, "505.1", &[(-5.0, 10.0), (25.0, 10.0)]),
            line(&set, "506", &[(10.0, 10.0), (10.0, 16.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_driveway_keeps_only_its_beginning() {
        // ISSprOM 505.1 has no minimum length: the stub is 1.0 mm. ISOM 505 is at least 4.25 mm,
        // at 150 % 6.375 mm: the stub is that long.
        for (set, scale, path, stub) in [
            (SymbolSet::issprom_2019(), SPRINT, "505.1", 1.0),
            (isom(), SCALE, "505", 6.375),
        ] {
            let objects = vec![
                garden(&set),
                line(&set, path, &[(10.0, -2.0), (10.0, 12.0)]),
            ];
            let (out, n) = olive_at(&set, scale, objects);
            assert_eq!(n, 1, "{}: {out:?}", set.name);
            let ways = of_code(&set, &out, path);
            assert_eq!(ways.len(), 1, "{}: {out:?}", set.name);
            assert!(
                runs_from_to(ways[0], (10.0, -2.0), (10.0, 0.4 + stub)),
                "{}: {:?}",
                set.name,
                mm_of(ways[0])
            );
        }
    }

    #[test]
    fn a_driveway_behind_an_impassable_fence_goes_whole() {
        let set = SymbolSet::issprom_2019();
        for wall in ["518", "515"] {
            let barrier = line(&set, wall, &[(-5.0, 0.0), (25.0, 0.0)]);
            let objects = vec![
                garden(&set),
                barrier.clone(),
                line(&set, "505.1", &[(10.0, 0.0), (10.0, 12.0)]),
                // From the street: the part on the street side ends at the fence.
                line(&set, "505.1", &[(15.0, -3.0), (15.0, 12.0)]),
            ];
            let (out, n) = olive(&set, objects);
            assert_eq!(n, 2, "{wall}: {out:?}");
            assert!(out.contains(&barrier), "{wall}: {out:?}");
            let ways = of_code(&set, &out, "505.1");
            assert_eq!(ways.len(), 1, "{wall}: {out:?}");
            assert!(
                runs_from_to(ways[0], (15.0, -3.0), (15.0, 0.0)),
                "{wall}: {:?}",
                mm_of(ways[0])
            );
        }
    }

    #[test]
    fn a_crossing_barred_at_one_end_is_no_crossing() {
        let set = SymbolSet::issprom_2019();
        let fence = line(&set, "518", &[(20.0, -5.0), (20.0, 25.0)]);
        let objects = vec![
            garden(&set),
            fence.clone(),
            line(&set, "505.1", &[(-5.0, 10.0), (25.0, 10.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 1, "{out:?}");
        assert!(out.contains(&fence), "{out:?}");
        let ways = of_code(&set, &out, "505.1");
        assert_eq!(ways.len(), 2, "{out:?}");
        assert!(
            runs_from_to(ways[0], (-5.0, 10.0), (1.4, 10.0)),
            "{:?}",
            mm_of(ways[0])
        );
        assert!(
            runs_from_to(ways[1], (20.0, 10.0), (25.0, 10.0)),
            "{:?}",
            mm_of(ways[1])
        );
    }

    #[test]
    fn a_passable_fence_does_not_bar() {
        let set = SymbolSet::issprom_2019();
        let fence = line(&set, "516", &[(-5.0, 0.0), (25.0, 0.0)]);
        let objects = vec![
            garden(&set),
            fence.clone(),
            line(&set, "505.1", &[(10.0, 0.0), (10.0, 12.0)]),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(n, 1, "{out:?}");
        assert!(out.contains(&fence), "{out:?}");
        let ways = of_code(&set, &out, "505.1");
        assert_eq!(ways.len(), 1, "{out:?}");
        assert!(
            runs_from_to(ways[0], (10.0, 0.0), (10.0, 1.4)),
            "{:?}",
            mm_of(ways[0])
        );
    }

    /// A passage through the building at the street front into the garden, tagged `key=value`.
    fn passage(set: &SymbolSet, key: &str, value: &str) -> MapObject {
        tagged(area(set, "522", (8.0, -1.0), (12.0, 4.0)), key, value)
    }

    #[test]
    fn a_passage_into_a_yard_goes() {
        let set = SymbolSet::issprom_2019();
        let building = area(&set, "521", (0.0, -1.0), (20.0, 4.0));
        let objects = vec![
            garden(&set),
            building.clone(),
            passage(&set, "freemapper:alkis", "Durchfahrt im Gebäude"),
            passage(&set, "freemapper:alkis", "Arkade"),
            passage(
                &set,
                "freemapper:osm",
                "highway=service; tunnel=building_passage",
            ),
            passage(&set, "freemapper:osm", "highway=footway; covered=arcade"),
        ];
        let (out, n) = olive(&set, objects);
        assert_eq!(out, vec![garden(&set), building]);
        assert_eq!(n, 4);
    }

    #[test]
    fn a_passage_on_a_crossing_stays() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            passage(&set, "freemapper:alkis", "Durchfahrt im Gebäude"),
            line(&set, "505.1", &[(10.0, -5.0), (10.0, 25.0)]),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn a_roof_inside_stays() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            garden(&set),
            tagged(
                area(&set, "522", (5.0, 5.0), (8.0, 8.0)),
                "freemapper:alkis",
                "Überdachung",
            ),
            tagged(
                area(&set, "522", (12.0, 12.0), (15.0, 15.0)),
                "freemapper:alkis",
                "Carport",
            ),
        ];
        assert_eq!(olive(&set, objects.clone()), (objects, 0));
    }

    #[test]
    fn without_olive_nothing_changes() {
        let set = SymbolSet::issprom_2019();
        let objects = vec![
            line(&set, "518", &[(0.0, 0.0), (10.0, 0.0)]),
            point(&set, "526", (5.0, 10.0)),
            area(&set, "401", (20.0, 20.0), (30.0, 30.0)),
            line(&set, "505.1", &[(50.0, 0.0), (50.0, 10.0)]),
        ];
        let (out, report) = train(
            objects.clone(),
            None,
            MapKind::Sprint,
            &georef_at(SPRINT),
            &set,
        );
        assert_eq!(out, objects);
        assert_eq!(report, TrainingReport::default());
    }
}
