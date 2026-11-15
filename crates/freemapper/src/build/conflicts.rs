//! Where drawn symbols come too close: the yardstick of a readable map.
//!
//! Every object is drawn as the renderer draws it (`render/draw.rs`) and becomes its footprint on
//! paper. Two objects whose classes may not lie on each other ([`relation`]) are in conflict where
//! the centre line of the one that gives way runs inside the footprint of the other (an overlap),
//! or closer to it than the gap between two line symbols (ISSprOM §3.3, [`LINE_GAP_MM`]). Where
//! two centre lines cross or meet at [`MIN_CROSSING_DEG`] or more there is no conflict: a crossing
//! is drawn that way. Nothing is changed here; the training level reads the conflicts and joins,
//! cuts or drops objects.

use std::collections::HashMap;

use geo::algorithm::buffer::{BufferStyle, LineCap};
use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{
    BooleanOps as _, BoundingRect as _, Buffer as _, Distance as _, Euclidean, Intersects as _,
    Length as _,
};
use geo_types::{Coord, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect};

use crate::build::barriers::BARRIER_SYMBOLS;
use crate::build::content::PATH_CODES;
use crate::build::line_cuts::CONTOUR_CODES;
use crate::build::symbol_measures::LINE_GAP_MM;
use crate::measure::{PaperMm, SymbolScale};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::render::draw::{Shape, primitives};
use crate::render::geometry::{Part, Pt, parts};
use crate::render::symbols::RenderSymbols;

/// A stretch of a centre line in conflict counts from this length on paper.
pub const MIN_CONFLICT: PaperMm = PaperMm(1.0);
/// Comparisons at a threshold are tolerant by 2 map units.
pub const TOLERANCE_MM: f64 = 0.002;
/// The side of a cell of the grid that finds the pairs (mm on paper).
const CELL_MM: f64 = 10.0;
/// A point of a stroke this close to the centre line lies on it (mm on paper).
const ON_LINE_MM: f64 = 1e-6;
/// Two centre lines cross from this angle on; a flatter meeting is measured as running side by
/// side, so two digitisations of one feature that zigzag across each other do not escape.
pub const MIN_CROSSING_DEG: f64 = 20.0;
/// sin 20°, written out so every system compares alike.
const MIN_CROSSING_SIN: f64 = 0.342_020_143_325_668_7;
/// The footprint of the other object is cut to this much around the one that gives way before
/// it is grown (mm on paper, beyond the growth itself).
const LOCAL_MARGIN_MM: f64 = 1.0;

/// What a symbol is for the conflict measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class {
    Building,
    WaterArea,
    BankLine,
    Railway,
    Road,
    Path,
    Watercourse,
    Stairway,
    Barrier,
    Cliff,
    EarthBank,
    Contour,
    Point,
    /// An area symbol outside the table: ways pass under and over it (522 canopies, 501 paved
    /// areas).
    OtherArea,
    Other,
}

impl Class {
    /// Every class, in the order of the declaration.
    pub const ALL: [Class; 15] = [
        Class::Building,
        Class::WaterArea,
        Class::BankLine,
        Class::Railway,
        Class::Road,
        Class::Path,
        Class::Watercourse,
        Class::Stairway,
        Class::Barrier,
        Class::Cliff,
        Class::EarthBank,
        Class::Contour,
        Class::Point,
        Class::OtherArea,
        Class::Other,
    ];
}

/// How two classes may lie to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// They may lie on each other: no conflict.
    Free,
    /// They belong to one network: where they lie side by side they are joined.
    Joined,
    /// They keep the gap between them.
    Apart,
}

/// How close the one that gives way comes to the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    /// The one that gives way lies on the other.
    Overlap,
    /// It lies closer to the other than the gap.
    Gap,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    /// Index of the object that gives way (lower rank; same rank: the shorter) and of the other.
    pub loser: usize,
    pub winner: usize,
    pub classes: (Class, Class),
    pub grade: Grade,
    /// Length of the loser's centre line in conflict, mm on paper (0 for areas and points).
    pub length: PaperMm,
}

/// The classes by symbol code, first row first: a later row of the same symbol is ignored (on a
/// forest map 501.8 and 505.2 are roads by their alias, before the path codes come).
fn class_rows() -> Vec<(&'static str, Class)> {
    use Class::*;
    let mut rows: Vec<(&str, Class)> = vec![("521", Building)];
    // 301 and 301.1 are also the codes of the water areas of `build/water.rs`.
    rows.extend(["301", "301.1", "302"].map(|c| (c, WaterArea)));
    rows.extend(["301.4", "301.6"].map(|c| (c, BankLine)));
    rows.extend(["509", "509.1", "509.2"].map(|c| (c, Railway)));
    rows.extend(["502", "503", "504"].map(|c| (c, Road)));
    rows.extend(["505", "506", "507"].map(|c| (c, Path)));
    rows.extend(PATH_CODES.iter().map(|&c| (c, Path)));
    rows.extend(["304", "305", "306"].map(|c| (c, Watercourse)));
    rows.extend(["532", "532.6", "532.7", "532.8", "532.9"].map(|c| (c, Stairway)));
    rows.extend(["513", "515", "516", "518"].map(|c| (c, Barrier)));
    rows.extend(BARRIER_SYMBOLS.iter().map(|&(c, _, _)| (c, Barrier)));
    rows.extend(["201", "202"].map(|c| (c, Cliff)));
    rows.extend(["104", "104.1"].map(|c| (c, EarthBank)));
    rows.extend(CONTOUR_CODES.map(|c| (c, Contour)));
    rows
}

/// OOM symbol types (`SymbolSet::symbol_type`).
const POINT_TYPE: u32 = 1;
const LINE_TYPE: u32 = 2;
const AREA_TYPE: u32 = 4;
const COMBINED_TYPE: u32 = 16;

/// The classes of the symbols of a set: symbol id → class.
pub struct Classes {
    classes: HashMap<u32, Class>,
    /// The OOM type of every top-level symbol.
    types: HashMap<u32, u32>,
}

impl Classes {
    pub fn of(symbol_set: &SymbolSet) -> Self {
        let types: HashMap<u32, u32> = (0..=symbol_set.max_id())
            .filter_map(|id| Some((id, symbol_set.symbol_type(id)?)))
            .collect();
        let mut classes = HashMap::new();
        for (code, class) in class_rows() {
            if let Some(id) = symbol_set.id_for_code(code) {
                classes.entry(id).or_insert(class);
            }
        }
        for (&id, &kind) in &types {
            classes.entry(id).or_insert(match kind {
                POINT_TYPE => Class::Point,
                AREA_TYPE => Class::OtherArea,
                _ => Class::Other,
            });
        }
        Self { classes, types }
    }

    pub fn class(&self, symbol_id: u32) -> Class {
        self.classes
            .get(&symbol_id)
            .copied()
            .unwrap_or(Class::Other)
    }

    /// The ids of the line and area symbols of `class` in this set.
    pub fn ids(&self, class: Class) -> Vec<u32> {
        let mut ids: Vec<u32> = self
            .classes
            .iter()
            .filter(|&(id, &c)| {
                c == class
                    && matches!(
                        self.types.get(id),
                        Some(&(LINE_TYPE | AREA_TYPE | COMBINED_TYPE))
                    )
            })
            .map(|(&id, _)| id)
            .collect();
        ids.sort_unstable();
        ids
    }

    /// Whether an object of `symbol_id` drawn along a path is a line: its centre line counts.
    fn is_line(&self, symbol_id: u32) -> bool {
        use Class::*;
        match self.class(symbol_id) {
            BankLine | Railway | Road | Path | Watercourse | Stairway | Barrier | Cliff
            | EarthBank | Contour => true,
            Building | WaterArea | OtherArea | Point => false,
            Other => self.types.get(&symbol_id) == Some(&LINE_TYPE),
        }
    }
}

/// One object as the measure sees it.
struct Footprint {
    class: Class,
    footprint: MultiPolygon<f64>,
    /// Empty for areas and points.
    centre: MultiLineString<f64>,
    half_width: f64,
    length: f64,
    bounds: Option<Rect<f64>>,
}

/// Per object: its class, its centre lines and its drawn footprint, in paper mm (y down).
pub struct Footprints {
    items: Vec<Footprint>,
}

impl Footprints {
    pub fn of(objects: &[&MapObject], symbol_set: &SymbolSet) -> Self {
        let symbols = RenderSymbols::from_set(symbol_set);
        let classes = Classes::of(symbol_set);
        let items = objects
            .iter()
            .map(|object| footprint(object, &symbols, &classes))
            .collect();
        Self { items }
    }

    pub fn class(&self, i: usize) -> Class {
        self.items[i].class
    }

    pub fn footprint(&self, i: usize) -> &MultiPolygon<f64> {
        &self.items[i].footprint
    }

    /// Half the width of the stroke along the centre line of a line object; 0 for others.
    pub fn half_width(&self, i: usize) -> f64 {
        self.items[i].half_width
    }

    /// Length of the centre line, mm on paper; 0 for areas and points.
    pub fn length(&self, i: usize) -> f64 {
        self.items[i].length
    }
}

/// The area a stroke along `path` of half width `half` covers, ends cut straight. A closed path
/// runs on over its first segment, so that its start is joined like every other corner.
fn stroke_area(path: &[Pt], closed: bool, half: f64) -> MultiPolygon<f64> {
    let mut coords: Vec<Coord<f64>> = path.iter().map(|&(x, y)| Coord { x, y }).collect();
    if closed && coords.len() > 2 {
        if coords.first() != coords.last() {
            coords.push(coords[0]);
        }
        coords.push(coords[1]);
    }
    LineString(coords).buffer_with_style(BufferStyle::new(half).line_cap(LineCap::Butt))
}

/// The union of `shapes`, merged pairwise.
fn union_all(mut shapes: Vec<MultiPolygon<f64>>) -> MultiPolygon<f64> {
    while shapes.len() > 1 {
        shapes = shapes
            .chunks(2)
            .map(|pair| match pair {
                [a, b] => a.union(b),
                [a] => a.clone(),
                _ => unreachable!("chunks of two"),
            })
            .collect();
    }
    shapes
        .pop()
        .unwrap_or_else(|| MultiPolygon::new(Vec::new()))
}

fn footprint(object: &MapObject, symbols: &RenderSymbols, classes: &Classes) -> Footprint {
    let class = classes.class(object.symbol_id);
    let drawn = primitives(object, symbols);
    let is_line = matches!(object.kind, MapObjectKind::Path) && classes.is_line(object.symbol_id);
    let pieces: Vec<Part> = if is_line {
        parts(&object.coords)
            .into_iter()
            .filter(|p| p.points.len() >= 2)
            .collect()
    } else {
        Vec::new()
    };
    let centre = MultiLineString(
        pieces
            .iter()
            .map(|p| p.points.iter().map(|&(x, y)| Coord { x, y }).collect())
            .collect(),
    );
    let mut half_width: f64 = 0.0;
    let mut shapes = Vec::new();
    for primitive in &drawn {
        match &primitive.shape {
            Shape::Stroke {
                path,
                closed,
                width,
                ..
            } => {
                let half = width / 2.0;
                if is_line
                    && half > half_width
                    && path
                        .iter()
                        .all(|&(x, y)| Euclidean.distance(&centre, &Point::new(x, y)) <= ON_LINE_MM)
                {
                    half_width = half;
                }
                shapes.push(stroke_area(path, *closed, half));
            }
            Shape::Fill { rings } => {
                let mut rings = rings.iter().filter(|r| r.len() > 2).map(|r| {
                    LineString(r.iter().map(|&(x, y)| Coord { x, y }).collect::<Vec<_>>())
                });
                if let Some(exterior) = rings.next() {
                    // Even-odd, as the renderer fills: a union resolves the rings.
                    let fill = Polygon::new(exterior, rings.collect());
                    shapes.push(fill.union(&MultiPolygon::new(Vec::new())));
                }
            }
            Shape::Text { .. } => {}
        }
    }
    if half_width > 0.0 {
        // The centre line closes the gaps between the dashes.
        for part in &pieces {
            shapes.push(stroke_area(&part.points, part.closed, half_width));
        }
    }
    let footprint = union_all(shapes);
    let length = centre.iter().map(|l| Euclidean.length(l)).sum();
    let bounds = footprint.bounding_rect();
    Footprint {
        class,
        footprint,
        centre,
        half_width,
        length,
        bounds,
    }
}

/// The rank of a class: where two meet, the higher one stays.
pub fn rank(class: Class) -> u8 {
    use Class::*;
    match class {
        Building => 9,
        WaterArea | BankLine => 8,
        Railway => 7,
        Road => 6,
        Path => 5,
        Watercourse | Stairway => 4,
        Barrier | Cliff => 3,
        EarthBank => 2,
        Contour => 1,
        Point | OtherArea | Other => 0,
    }
}

/// How symbols of the classes `a` and `b` may lie to each other; symmetric.
pub fn relation(a: Class, b: Class) -> Relation {
    use Class::*;
    match (a, b) {
        (OtherArea | Other, _) | (_, OtherArea | Other) => Relation::Free,
        // Contours run through buildings (ISSprOM 2019-2, 101): only landforms keep them apart.
        (Contour, EarthBank | Cliff) | (EarthBank | Cliff, Contour) => Relation::Apart,
        (Contour, _) | (_, Contour) => Relation::Free,
        (Road | Path | Railway | Stairway, Road | Path | Railway | Stairway) => Relation::Joined,
        (Watercourse, Watercourse | WaterArea | BankLine) | (WaterArea | BankLine, Watercourse) => {
            Relation::Joined
        }
        (WaterArea, BankLine) | (BankLine, WaterArea) => Relation::Free,
        // Bridges, and a network is never cut by a building: at forest scale a road at its
        // minimum width touches the houses along it.
        (Road | Path | Railway, WaterArea | BankLine | Building)
        | (WaterArea | BankLine | Building, Road | Path | Railway) => Relation::Free,
        (Building, Building) | (WaterArea, WaterArea) | (BankLine, BankLine) | (Cliff, Cliff) => {
            Relation::Free
        }
        _ => Relation::Apart,
    }
}

/// The gap between two line symbols on the sheet of a map of `kind` at `scale_denominator`.
pub fn gap(kind: MapKind, scale_denominator: u32) -> PaperMm {
    LINE_GAP_MM.on_paper(SymbolScale::new(kind, scale_denominator))
}

/// Where the drawn symbols of `footprints` come too close: one [`Conflict`] per pair and grade,
/// in the order of the pairs.
pub fn conflicts(footprints: &Footprints, kind: MapKind, scale_denominator: u32) -> Vec<Conflict> {
    let gap = gap(kind, scale_denominator).0;
    let items = &footprints.items;
    // A uniform grid over the bounding boxes, grown by the gap.
    let cell = |v: f64| (v / CELL_MM).floor() as i64;
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, item) in items.iter().enumerate() {
        if matches!(item.class, Class::OtherArea | Class::Other) {
            continue;
        }
        let Some(b) = item.bounds else { continue };
        for x in cell(b.min().x - gap)..=cell(b.max().x + gap) {
            for y in cell(b.min().y - gap)..=cell(b.max().y + gap) {
                grid.entry((x, y)).or_default().push(i);
            }
        }
    }
    let mut pairs: Vec<(usize, usize)> = grid
        .values()
        .flat_map(|ids| {
            ids.iter()
                .enumerate()
                .flat_map(|(k, &a)| ids[k + 1..].iter().map(move |&b| (a.min(b), a.max(b))))
        })
        .collect();
    pairs.sort_unstable();
    pairs.dedup();
    let mut found = Vec::new();
    for (a, b) in pairs {
        let (fa, fb) = (&items[a], &items[b]);
        if relation(fa.class, fb.class) == Relation::Free {
            continue;
        }
        let (Some(ba), Some(bb)) = (fa.bounds, fb.bounds) else {
            continue;
        };
        if ba.min().x - gap > bb.max().x
            || bb.min().x - gap > ba.max().x
            || ba.min().y - gap > bb.max().y
            || bb.min().y - gap > ba.max().y
        {
            continue;
        }
        // The lower rank gives way; of the same rank the shorter, of the same length (within
        // the tolerance) the later.
        let b_loses = match rank(fb.class).cmp(&rank(fa.class)) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => fb.length <= fa.length + TOLERANCE_MM,
        };
        let (loser, winner) = if b_loses { (b, a) } else { (a, b) };
        let grades = measure(&items[loser], &items[winner], gap);
        for (grade, length) in [Grade::Overlap, Grade::Gap].into_iter().zip(grades) {
            if let Some(length) = length {
                found.push(Conflict {
                    loser,
                    winner,
                    classes: (items[loser].class, items[winner].class),
                    grade,
                    length: PaperMm(length),
                });
            }
        }
    }
    found
}

/// The length of the overlap and of the gap conflict of `loser` with `winner` (mm on paper), or
/// `None` where there is none.
fn measure(loser: &Footprint, winner: &Footprint, gap: f64) -> [Option<f64>; 2] {
    let Some(bounds) = loser.bounds else {
        return [None, None];
    };
    let grow = loser.half_width + gap + LOCAL_MARGIN_MM;
    let around = Rect::new(
        Coord {
            x: bounds.min().x - grow,
            y: bounds.min().y - grow,
        },
        Coord {
            x: bounds.max().x + grow,
            y: bounds.max().y + grow,
        },
    )
    .to_polygon();
    let other = winner.footprint.intersection(&around);
    if other.0.is_empty() {
        return [None, None];
    }
    let grown = |by: f64| other.buffer(by);
    if loser.centre.0.is_empty() || loser.length <= 0.0 {
        // An area or a point: does its footprint reach into the other, or closer than the gap?
        if grown(-TOLERANCE_MM).intersects(&loser.footprint) {
            return [Some(0.0), None];
        }
        let near = grown(gap - TOLERANCE_MM).intersects(&loser.footprint);
        return [None, near.then_some(0.0)];
    }
    let overlap_zone = grown(loser.half_width - TOLERANCE_MM);
    let gap_zone = grown(loser.half_width + gap - TOLERANCE_MM);
    // Where the two centre lines cross or meet, the drawing joins them as it must.
    let radius = loser.half_width + winner.half_width + gap;
    let crossings: Vec<MultiPolygon<f64>> = meeting_points(&loser.centre, &winner.centre)
        .into_iter()
        .map(|c| Point(c).buffer(radius))
        .collect();
    let centre = if crossings.is_empty() {
        loser.centre.clone()
    } else {
        union_all(crossings).clip(&loser.centre, true)
    };
    let overlap = overlap_zone.clip(&centre, false);
    let near = overlap_zone.clip(&gap_zone.clip(&centre, false), true);
    [counted(&overlap), counted(&near)]
}

/// The total length of the stretches of `pieces` from [`MIN_CONFLICT`] on, if any.
fn counted(pieces: &MultiLineString<f64>) -> Option<f64> {
    let total: f64 = stretches(pieces)
        .into_iter()
        .filter(|&l| l > MIN_CONFLICT.0 + TOLERANCE_MM)
        .sum();
    (total > 0.0).then_some(total)
}

/// The lengths of the stretches `pieces` form: pieces that share an end are one stretch.
fn stretches(pieces: &MultiLineString<f64>) -> Vec<f64> {
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let key = |c: &Coord<f64>| ((c.x * 1e6).round() as i64, (c.y * 1e6).round() as i64);
    let mut parent: Vec<usize> = (0..pieces.0.len()).collect();
    let mut ends: HashMap<(i64, i64), usize> = HashMap::new();
    for (i, piece) in pieces.iter().enumerate() {
        for end in [piece.0.first(), piece.0.last()].into_iter().flatten() {
            let other = *ends.entry(key(end)).or_insert(i);
            let (a, b) = (root(&mut parent, i), root(&mut parent, other));
            parent[a] = b;
        }
    }
    let mut lengths: HashMap<usize, f64> = HashMap::new();
    for (i, piece) in pieces.iter().enumerate() {
        *lengths.entry(root(&mut parent, i)).or_default() += Euclidean.length(piece);
    }
    let mut lengths: Vec<f64> = lengths.into_values().collect();
    lengths.sort_by(f64::total_cmp);
    lengths
}

/// The points where the centre lines `a` and `b` cross or meet at [`MIN_CROSSING_DEG`] or more;
/// lines that run on each other or meet flatter do not meet there: they run side by side.
pub fn meeting_points(a: &MultiLineString<f64>, b: &MultiLineString<f64>) -> Vec<Coord<f64>> {
    let (Some(frame_a), Some(frame_b)) = (a.bounding_rect(), b.bounding_rect()) else {
        return Vec::new();
    };
    let near = |line: &Line<f64>, frame: &Rect<f64>| line.bounding_rect().intersects(frame);
    let lines_b: Vec<Line<f64>> = b
        .iter()
        .flat_map(|l| l.lines())
        .filter(|l| near(l, &frame_a))
        .collect();
    let mut points = Vec::new();
    for la in a
        .iter()
        .flat_map(|l| l.lines())
        .filter(|l| near(l, &frame_b))
    {
        for &lb in &lines_b {
            let (da, db) = (la.delta(), lb.delta());
            let cross = da.x * db.y - da.y * db.x;
            // sin² of the angle between the two segments, without a library function.
            let lengths2 = (da.x * da.x + da.y * da.y) * (db.x * db.x + db.y * db.y);
            if cross * cross < MIN_CROSSING_SIN * MIN_CROSSING_SIN * lengths2 {
                continue;
            }
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                line_intersection(la, lb)
            {
                points.push(intersection);
            }
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::omap::{MapCoord, MapObjectKind, closed_ring};

    /// The scale of the tests: a forest map at 1:10 000, symbols at 150 %.
    const SCALE: u32 = 10_000;

    fn isom() -> SymbolSet {
        SymbolSet::isom_2017_2(MapKind::Forest.symbol_enlargement(SCALE))
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

    fn measure(set: &SymbolSet, objects: &[MapObject]) -> Vec<Conflict> {
        let objects: Vec<&MapObject> = objects.iter().collect();
        conflicts(&Footprints::of(&objects, set), MapKind::Forest, SCALE)
    }

    fn half_width(set: &SymbolSet, code: &str) -> f64 {
        let object = line(set, code, &[(0.0, 0.0), (10.0, 0.0)]);
        Footprints::of(&[&object], set).half_width(0)
    }

    #[test]
    fn an_earth_bank_on_a_ditch_is_an_overlap() {
        let set = isom();
        let objects = [
            line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "104", &[(0.0, 0.2), (20.0, 0.2)]),
        ];
        let found = measure(&set, &objects);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!((found[0].loser, found[0].winner), (1, 0));
        assert_eq!(found[0].classes, (Class::EarthBank, Class::Watercourse));
        assert_eq!(found[0].grade, Grade::Overlap);
        assert!((found[0].length.0 - 20.0).abs() < 1.0, "{found:?}");
    }

    #[test]
    fn an_earth_bank_beside_a_ditch_is_a_gap() {
        let set = isom();
        let reach = half_width(&set, "306") + half_width(&set, "104");
        let beside = |apart: f64| {
            let y = reach + apart;
            measure(
                &set,
                &[
                    line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]),
                    line(&set, "104", &[(0.0, y), (20.0, y)]),
                ],
            )
        };
        let found = beside(0.1);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].loser, 1);
        assert_eq!(found[0].grade, Grade::Gap);
        assert!((found[0].length.0 - 20.0).abs() < 1.0, "{found:?}");
        assert_eq!(beside(1.0), Vec::new());
    }

    #[test]
    fn two_paths_that_cross_are_no_conflict() {
        let set = isom();
        let objects = [
            line(&set, "505", &[(0.0, 10.0), (20.0, 10.0)]),
            line(&set, "505", &[(10.0, 0.0), (10.0, 20.0)]),
        ];
        assert_eq!(measure(&set, &objects), Vec::new());
    }

    #[test]
    fn two_paths_side_by_side_are() {
        let set = isom();
        let objects = [
            line(&set, "505", &[(0.0, 0.0), (10.0, 0.0)]),
            line(&set, "505", &[(-1.0, 0.3), (11.0, 0.3)]),
        ];
        let found = measure(&set, &objects);
        assert_eq!(found.len(), 1, "{found:?}");
        // Of the same rank the shorter gives way.
        assert_eq!((found[0].loser, found[0].winner), (0, 1));
        assert_eq!(found[0].classes, (Class::Path, Class::Path));
        assert!((found[0].length.0 - 10.0).abs() < 1.0, "{found:?}");
    }

    #[test]
    fn a_dashed_path_has_no_holes() {
        let set = isom();
        let objects = [
            line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "518", &[(5.0, 0.0), (15.0, 0.0)]),
        ];
        let found = measure(&set, &objects);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!((found[0].loser, found[0].winner), (1, 0));
        assert_eq!(found[0].classes, (Class::Barrier, Class::Path));
        assert!((found[0].length.0 - 10.0).abs() < 1.0, "{found:?}");
    }

    #[test]
    fn a_path_over_water_is_free() {
        let set = isom();
        let objects = [
            area(&set, "301", (5.0, 0.0), (15.0, 10.0)),
            line(&set, "505", &[(0.0, 5.0), (20.0, 5.0)]),
        ];
        assert_eq!(measure(&set, &objects), Vec::new());
    }

    #[test]
    fn a_contour_under_a_path_is_free() {
        let set = isom();
        let objects = [
            line(&set, "505", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "101", &[(0.0, 0.1), (20.0, 0.1)]),
        ];
        assert_eq!(measure(&set, &objects), Vec::new());
    }

    #[test]
    fn a_contour_through_a_building_is_free() {
        // ISSprOM 2019-2, 101: contours run through all symbols, also Building (521).
        let set = isom();
        let objects = [
            area(&set, "521", (5.0, 0.0), (15.0, 10.0)),
            line(&set, "101", &[(0.0, 5.0), (20.0, 5.0)]),
        ];
        assert_eq!(measure(&set, &objects), Vec::new());
    }

    #[test]
    fn a_contour_on_an_earth_bank_is_an_overlap() {
        let set = isom();
        let objects = [
            line(&set, "104", &[(0.0, 0.0), (20.0, 0.0)]),
            line(&set, "101", &[(0.0, 0.1), (20.0, 0.1)]),
        ];
        let found = measure(&set, &objects);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].classes, (Class::Contour, Class::EarthBank));
        assert_eq!(found[0].grade, Grade::Overlap);
    }

    #[test]
    fn lengths_within_the_tolerance_are_equal() {
        // The later path is longer by one map unit: of equals the later gives way.
        let set = isom();
        let objects = [
            line(&set, "505", &[(0.0, 0.0), (10.0, 0.0)]),
            line(&set, "505", &[(0.0, 0.3), (10.001, 0.3)]),
        ];
        let found = measure(&set, &objects);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!((found[0].loser, found[0].winner), (1, 0));
    }

    #[test]
    fn a_fence_zigzagging_along_a_path_is_one_conflict() {
        // Two digitisations of one feature: the fence crosses the path every 2 mm at under 6°.
        let set = isom();
        let zigzag: Vec<(f64, f64)> = (0..=10)
            .map(|k| (2.0 * f64::from(k), if k % 2 == 0 { -0.1 } else { 0.1 }))
            .collect();
        let objects = [
            line(&set, "505", &[(-5.0, 0.0), (25.0, 0.0)]),
            line(&set, "518", &zigzag),
        ];
        let found = measure(&set, &objects);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!((found[0].loser, found[0].winner), (1, 0));
        assert_eq!(found[0].grade, Grade::Overlap);
        assert!((found[0].length.0 - 20.0).abs() < 1.0, "{found:?}");
    }

    #[test]
    fn a_ditch_that_crosses_a_path_is_no_conflict() {
        let set = isom();
        let right = [
            line(&set, "505", &[(0.0, 10.0), (20.0, 10.0)]),
            line(&set, "306", &[(10.0, 0.0), (10.0, 20.0)]),
        ];
        assert_eq!(measure(&set, &right), Vec::new());
        let (sin, cos) = 30f64.to_radians().sin_cos();
        let slant = [
            line(&set, "505", &[(0.0, 10.0), (20.0, 10.0)]),
            line(
                &set,
                "306",
                &[
                    (10.0 - 8.0 * cos, 10.0 - 8.0 * sin),
                    (10.0 + 8.0 * cos, 10.0 + 8.0 * sin),
                ],
            ),
        ];
        assert_eq!(measure(&set, &slant), Vec::new());
    }

    #[test]
    fn a_road_along_a_house_is_free() {
        let set = isom();
        let objects = [
            area(&set, "521", (5.0, 0.0), (15.0, 5.0)),
            line(&set, "503", &[(0.0, 5.1), (20.0, 5.1)]),
        ];
        assert_eq!(measure(&set, &objects), Vec::new());
    }

    #[test]
    fn row_houses_are_free() {
        let set = isom();
        let objects = [
            area(&set, "521", (0.0, 0.0), (5.0, 5.0)),
            area(&set, "521", (5.0, 0.0), (10.0, 5.0)),
            area(&set, "521", (10.05, 0.0), (15.0, 5.0)),
        ];
        assert_eq!(measure(&set, &objects), Vec::new());
    }

    #[test]
    fn a_path_under_a_canopy_is_free() {
        let set = isom();
        let objects = [
            area(&set, "522", (5.0, 0.0), (15.0, 10.0)),
            line(&set, "505", &[(0.0, 5.0), (20.0, 5.0)]),
        ];
        assert_eq!(measure(&set, &objects), Vec::new());
    }

    #[test]
    fn a_point_on_a_building_is_an_overlap() {
        let set = isom();
        let objects = [
            area(&set, "521", (0.0, 0.0), (10.0, 10.0)),
            point(&set, "526", (5.0, 5.0)),
        ];
        let found = measure(&set, &objects);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!((found[0].loser, found[0].winner), (1, 0));
        assert_eq!(found[0].classes, (Class::Point, Class::Building));
        assert_eq!(found[0].grade, Grade::Overlap);
        assert_eq!(found[0].length, PaperMm(0.0));
    }

    #[test]
    fn the_verdict_at_the_gap_is_tolerant() {
        let set = isom();
        let gap = gap(MapKind::Forest, SCALE).0;
        // Centre lines this far apart (map units) leave their stroke edges exactly `gap` apart.
        let apart = (half_width(&set, "306") + half_width(&set, "104") + gap) * 1000.0;
        assert!(
            (apart - apart.round()).abs() < 1e-6,
            "{apart}: whole map units"
        );
        let beside = |shift: f64| {
            let y = (apart.round() + shift) / 1000.0;
            measure(
                &set,
                &[
                    line(&set, "306", &[(0.0, 0.0), (20.0, 0.0)]),
                    line(&set, "104", &[(0.0, y), (20.0, y)]),
                ],
            )
        };
        for shift in [-1.0, 0.0, 1.0] {
            assert_eq!(beside(shift), Vec::new(), "shift {shift}");
        }
        // Beyond the tolerance it is a gap again.
        let closer = beside(-3.0);
        assert_eq!(closer.len(), 1, "{closer:?}");
        assert_eq!(closer[0].grade, Grade::Gap);
    }

    #[test]
    fn the_classes_follow_the_aliases() {
        let set = isom();
        let classes = Classes::of(&set);
        let class = |code: &str| classes.class(set.id_for_code(code).unwrap());
        assert_eq!(class("501.8"), Class::Road);
        assert_eq!(class("501.6"), Class::Path);
        let set = SymbolSet::issprom_2019();
        let classes = Classes::of(&set);
        let class = |code: &str| classes.class(set.id_for_code(code).unwrap());
        assert_eq!(class("501.6"), Class::Path);
        assert_eq!(class("501.8"), Class::Path);
        assert_eq!(class("532.6"), Class::Stairway);
        assert_eq!(class("509.1"), Class::Railway);
        assert_eq!(class("301.6"), Class::BankLine);
        assert!(
            classes
                .ids(Class::Path)
                .contains(&set.id_for_code("501.6").unwrap())
        );
    }

    #[test]
    fn a_set_without_the_code_gives_no_conflict() {
        let set = SymbolSet::school_2019();
        let none: Vec<&MapObject> = Vec::new();
        let footprints = Footprints::of(&none, &set);
        assert_eq!(
            conflicts(&footprints, MapKind::Schoolyard, 1000),
            Vec::new()
        );
        let unknown = |symbol_id: u32, x: f64| MapObject {
            symbol_id,
            kind: MapObjectKind::Path,
            coords: vec![at((x, 0.0)), at((x + 10.0, 0.0))],
            tags: Vec::new(),
        };
        let objects = [unknown(9999, 0.0), unknown(9999, 0.0), unknown(9998, 0.1)];
        let objects: Vec<&MapObject> = objects.iter().collect();
        let footprints = Footprints::of(&objects, &set);
        assert_eq!(footprints.class(0), Class::Other);
        assert_eq!(
            conflicts(&footprints, MapKind::Schoolyard, 1000),
            Vec::new()
        );
    }

    #[test]
    fn the_relation_is_symmetric() {
        for a in Class::ALL {
            for b in Class::ALL {
                assert_eq!(relation(a, b), relation(b, a), "{a:?} {b:?}");
            }
        }
    }
}
