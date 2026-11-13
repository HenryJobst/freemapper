//! ALKIS stairs (Berlin `Treppe`, `Freitreppe` areas and `Freitreppe` step edges) →
//! 532.6–532.9 candidates (Berlin plus 1).
//!
//! A stair polygon becomes one line along its walking direction. The oriented minimum rectangle
//! of the polygon gives two axes; the walking direction is the axis whose ends differ more in
//! height in the terrain model (DGM1, sampled in the map's CRS), and the line runs uphill. If
//! neither axis rises [`STAIR_MIN_RISE_M`], or no terrain model is stored, the candidate is
//! tagged `freemapper:direction=guessed`: a stair at a building or monument (entrance or plinth
//! steps) faces it, any other one runs along its short axis.
//!
//! The `Freitreppe` lines of `bauwerke-linien` are step edges. Parallel edges up to
//! [`STEP_GAP_M`] apart that overlap along their length form one flight (exact duplicates are
//! dropped), and the flight is one stair whose walking direction runs across the edges.
//!
//! The surveyed stairs (`sb-treppe`, Straßenbefahrung `cc_treppe` polygons) go the same way. ALKIS
//! wins: a surveyed stair that covers an ALKIS stair area or flight by at least
//! [`DUPLICATE_STAIR_SHARE`] of the smaller one is the same stair and is dropped; the ALKIS
//! stair names it in `freemapper:also`. The survey's `trs` (probably the number of steps) stays
//! as `berlin:trs`.
//!
//! NAS stairs (Mecklenburg-Vorpommern, Hesse; the full ALKIS of any state) come the same way:
//! the areas 1620 and 1621 of `AX_SonstigesBauwerkOderSonstigeEinrichtung`. The bottom edge
//! (1630, linked to its stair by `gehoert_zu`) gives the walking direction, across it and away
//! from it, and the direction is not guessed. A stair without an edge goes by the DGM1 as above.
//! An edge draws nothing on its own.
//!
//! The width across the walking direction, as paper width at the map scale, picks the footprint
//! symbol nearest to it. A stair wider than [`WIDE_STAIR_FACTOR`] × the widest footprint (a
//! stadium stand) is drawn as parallel 532.9 lines across its width, one footprint plus the
//! minimum gap of 0.15 mm apart, so the ticks of neighbouring lines do not run into each other.

use chrono::NaiveDate;
use geo::{Area as _, BooleanOps as _, Distance as _, Euclidean, MinimumRotatedRect as _};
use geo_types::{Coord, Polygon};
use serde_json::Value;

use crate::build::berlin::{
    LAYER_CRS, feature_lines, feature_polygons, provenance_tags, survey_tags,
};
use crate::build::dedup::add_also;
use crate::build::state::{NAS_SOURCE, nas_feature_tags};
use crate::crs::{Crs, ProjectedPoint};
use crate::derive::ndsm::bilinear;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::raster::Raster;

/// The layer the stair areas come from.
const STAIR_LAYER: &str = "bauwerke-flaechen";
/// `bezbwf` of the stair areas.
const STAIR_KINDS: [&str; 2] = ["Treppe", "Freitreppe"];
/// An axis whose ends differ less than this in height does not decide the walking direction.
pub const STAIR_MIN_RISE_M: f64 = 0.3;
/// The footprint symbols are 0.4, 0.5, 0.75 and 1 mm wide on paper (symbols at 100 %; at
/// 1:4000 the 1.6 / 2 / 3 / 4 m of their names). Upper bounds (mm, exclusive) of each: the
/// midpoints to the next one; wider stairs get 532.9.
const FOOTPRINTS: [(f64, &str); 3] = [(0.45, "532.6"), (0.625, "532.7"), (0.875, "532.8")];
const WIDEST_FOOTPRINT: &str = "532.9";
/// Paper width of 532.9 (mm, at 100 %).
const WIDEST_FOOTPRINT_MM: f64 = 1.0;
/// A stair wider than this many 532.9 widths is drawn as parallel lines across its width.
const WIDE_STAIR_FACTOR: f64 = 3.0;
/// Paper gap (mm) kept between the ticks of two neighbouring lines of a wide stair: the
/// minimum gap between two line symbols of one colour (ISSprOM 3.2, 0.15 mm).
const WIDE_LINE_GAP_MM: f64 = 0.15;
/// At most this many parallel lines for one wide stair; wider ones get them farther apart.
const MAX_WIDE_LINES: usize = 100;
/// A flat stair whose side lies this close to a building or monument faces it.
const FACADE_DISTANCE_M: f64 = 0.5;
/// Axis ends this much farther from a facade than the nearest end count as touching it too.
const FACADE_TIE_M: f64 = 0.05;
/// `bezbwf` of the monument areas (flat stairs at them face them).
const MONUMENT_KIND: &str = "Denkmal, Denkstein, Standbild";
/// The NAS layer file the stair areas and bottom edges come from (`sources/alkis_nas.rs`).
pub const NAS_STAIR_LAYER: &str = "sonstiges-bauwerk";
/// `bauwerksfunktion` of the NAS stair areas (`Treppe`, `Freitreppe`) and of the bottom edge.
const NAS_STAIR_CODES: [&str; 2] = ["1620", "1621"];
const NAS_BOTTOM_EDGE_CODE: &str = "1630";
/// The layer and `bezbwf` of the step edges.
pub const STEP_EDGE_LAYER: &str = "bauwerke-linien";
const STEP_EDGE_KIND: &str = "Freitreppe";
/// Step edges whose middle lies this close to another edge's line belong to one flight.
const STEP_GAP_M: f64 = 2.0;
/// Step edges of one flight are parallel within about 15° (the sine of the angle).
const STEP_PARALLEL_SIN: f64 = 0.26;
/// Step edges whose ends are this close are the same edge (ALKIS has some twice).
const DUPLICATE_EDGE_M: f64 = 0.05;
/// The layer file of the surveyed stairs.
pub const SURVEY_STAIR_LAYER: &str = "sb-treppe";
/// A surveyed stair covering this share of the smaller of it and an ALKIS stair is the same.
const DUPLICATE_STAIR_SHARE: f64 = 0.5;

/// The ISSprOM footprint symbol (532.6 … 532.9) nearest to a stair `paper_mm` wide on paper,
/// measured at 100 % symbol size (the ground width over metres per mm and the enlargement).
pub fn footprint_code(paper_mm: f64) -> &'static str {
    FOOTPRINTS
        .iter()
        .find(|(below, _)| paper_mm < *below)
        .map_or(WIDEST_FOOTPRINT, |(_, code)| code)
}

fn is_stair(feature: &Value) -> bool {
    feature["properties"]["bezbwf"]
        .as_str()
        .is_some_and(|kind| STAIR_KINDS.contains(&kind))
}

fn is_step_edge(feature: &Value) -> bool {
    feature["properties"]["bezbwf"] == STEP_EDGE_KIND
}

fn is_survey_stair(feature: &Value) -> bool {
    feature["properties"][freemapper_adapters_de::berlin::TYPE_KEY] == "cc_treppe"
}

fn nas_code(feature: &Value) -> Option<&str> {
    feature["properties"]["bauwerksfunktion"].as_str()
}

fn is_nas_stair(feature: &Value) -> bool {
    nas_code(feature).is_some_and(|c| NAS_STAIR_CODES.contains(&c))
}

fn is_nas_bottom_edge(feature: &Value) -> bool {
    nas_code(feature) == Some(NAS_BOTTOM_EDGE_CODE)
}

/// Whether any NAS stair area among the `sonstiges-bauwerk` `features` has no bottom edge: its
/// direction comes from the terrain model, so `derive` keeps the terrain then.
pub fn has_nas_stairs_without_edge(features: &[Value]) -> bool {
    features.iter().filter(|f| is_nas_stair(f)).any(|stair| {
        !features
            .iter()
            .any(|e| is_nas_bottom_edge(e) && names_stair(e, stair))
    })
}

/// Whether the bottom `edge` belongs to the `stair` (`gehoert_zu` names its oid).
fn names_stair(edge: &Value, stair: &Value) -> bool {
    let oid = &stair["properties"]["oid"];
    oid.is_string() && edge["properties"]["gehoert_zu"] == *oid
}

/// Whether any of the `features` of a layer is a stair area, ALKIS or surveyed (`derive` keeps
/// the terrain then).
pub fn has_stairs(features: &[Value]) -> bool {
    features.iter().any(|f| is_stair(f) || is_survey_stair(f))
}

/// What the stairs are built from.
pub struct StairInput<'a> {
    /// The `bauwerke-flaechen` features (EPSG:25833); only the stair areas are used.
    pub features: &'a [Value],
    /// The `bauwerke-linien` features (EPSG:25833); only the `Freitreppe` step edges are used.
    pub step_edges: &'a [Value],
    /// The stored `sb-treppe` features (EPSG:25833, stamped with `freemapper:layer` and the
    /// survey date); they are dropped where an ALKIS stair is.
    pub survey: &'a [Value],
    /// The stored NAS `sonstiges-bauwerk` features (the map's CRS); only the stair areas
    /// (1620, 1621) and their bottom edges (1630, by `gehoert_zu`) are used.
    pub nas: &'a [Value],
    /// The DGM1 in the map's CRS, if one is stored.
    pub terrain: Option<&'a Raster<f32>>,
    /// The 521 footprints in the map's CRS, for the direction of flat entrance steps.
    pub buildings: &'a [Polygon<f64>],
}

/// One axis of a stair rectangle: the centre line from side to side, and the width across it.
#[derive(Clone)]
struct Axis {
    from: Coord<f64>,
    to: Coord<f64>,
    width: f64,
}

impl Axis {
    fn length(&self) -> f64 {
        (self.to - self.from).x.hypot((self.to - self.from).y)
    }

    /// The height difference `to` minus `from` in `terrain`, if both ends have a height.
    fn rise(&self, terrain: &Raster<f32>) -> Option<f64> {
        let height = |c: Coord<f64>| bilinear(terrain, c.x, c.y).map(f64::from);
        Some(height(self.to)? - height(self.from)?)
    }

    fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            width: self.width,
        }
    }
}

/// The two axes of the oriented minimum rectangle of `polygon`, the long one first; none for a
/// degenerate polygon.
fn axes(polygon: &Polygon<f64>) -> Option<[Axis; 2]> {
    let rect = polygon.minimum_rotated_rect()?;
    let c: Vec<Coord<f64>> = rect.exterior().coords().copied().collect();
    let [p0, p1, p2, p3, ..] = c[..] else {
        return None;
    };
    let mid = |a: Coord<f64>, b: Coord<f64>| (a + b) / 2.0;
    let side = |a: Coord<f64>, b: Coord<f64>| (b - a).x.hypot((b - a).y);
    // Along p0 → p1, across p1 → p2, and the other way round.
    let first = Axis {
        from: mid(p0, p3),
        to: mid(p1, p2),
        width: side(p1, p2),
    };
    let second = Axis {
        from: mid(p0, p1),
        to: mid(p3, p2),
        width: side(p0, p1),
    };
    if first.length() <= 0.0 || second.length() <= 0.0 {
        return None;
    }
    Some(if first.length() >= second.length() {
        [first, second]
    } else {
        [second, first]
    })
}

/// The walking axis of a stair, uphill, and whether it is only guessed (no terrain model, or
/// flatter than [`STAIR_MIN_RISE_M`]): then towards a building or monument the stair lies at
/// ([`FACADE_DISTANCE_M`]), else the short axis. A flight long enough to be the long axis of
/// the rectangle would rise enough to show in the DGM1, so a flat stair is most likely wide.
fn walking_axis(
    [long, short]: [Axis; 2],
    terrain: Option<&Raster<f32>>,
    facades: &[Polygon<f64>],
) -> (Axis, bool) {
    let rises = terrain.and_then(|t| Some((long.rise(t)?, short.rise(t)?)));
    match rises {
        Some((a, b)) if a.abs().max(b.abs()) >= STAIR_MIN_RISE_M => {
            let (axis, rise) = if a.abs() >= b.abs() {
                (long, a)
            } else {
                (short, b)
            };
            (if rise < 0.0 { axis.reversed() } else { axis }, false)
        }
        _ => (facing_facade([long, short], facades), true),
    }
}

/// The walking axis of a stair from its bottom `edge` lines: the axis across the edge (the one
/// less parallel to the principal direction of its points), starting at the end nearest to the edge.
fn axis_from_edge([long, short]: [Axis; 2], edge: &[geo_types::LineString<f64>]) -> Option<Axis> {
    let points: Vec<Coord<f64>> = edge.iter().flat_map(|l| l.0.iter().copied()).collect();
    if points.len() < 2 {
        return None;
    }
    let centre = points.iter().fold(Coord { x: 0.0, y: 0.0 }, |s, p| s + *p) / points.len() as f64;
    // The principal direction of all edge points: a bent or multi-part edge runs the way most
    // of it does, whatever its first and last point.
    let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
    for p in &points {
        let d = *p - centre;
        (sxx, syy, sxy) = (sxx + d.x * d.x, syy + d.y * d.y, sxy + d.x * d.y);
    }
    if sxx + syy <= 0.0 {
        return None;
    }
    let angle = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    let along = Coord {
        x: angle.cos(),
        y: angle.sin(),
    };
    let parallel = |axis: &Axis| {
        let d = (axis.to - axis.from) / axis.length();
        (d.x * along.y - d.y * along.x).abs()
    };
    // The sine of the angle to the edge: the larger one runs across it.
    let axis = if parallel(&long) >= parallel(&short) {
        long
    } else {
        short
    };
    let distance = |c: Coord<f64>| (c - centre).x.hypot((c - centre).y);
    Some(if distance(axis.from) <= distance(axis.to) {
        axis
    } else {
        axis.reversed()
    })
}

/// Of the four axis ends, the one nearest to a building or monument within
/// [`FACADE_DISTANCE_M`] gives the walking axis, towards it (across the edge the stair lies
/// at); without one, the short (second) axis. Ends about as near as the nearest one (a stair
/// inside a monument area that covers its plinth touches it with every end) are told apart by
/// their distance to the centre of the area they are nearest to.
fn facing_facade([long, short]: [Axis; 2], facades: &[Polygon<f64>]) -> Axis {
    use geo::Centroid as _;
    let ends = geo_types::MultiPoint::from(vec![long.from, long.to, short.from, short.to]);
    let near: Vec<(&Polygon<f64>, Option<geo_types::Point<f64>>)> = facades
        .iter()
        .filter(|b| Euclidean.distance(&ends, *b) <= FACADE_DISTANCE_M)
        .map(|b| (b, b.centroid()))
        .collect();
    // The distance to the nearest facade, and to that facade's centre.
    let distance = |c: Coord<f64>| {
        let p = geo_types::Point(c);
        near.iter()
            .map(|(b, centre)| {
                let to_centre = centre.map_or(f64::INFINITY, |m| Euclidean.distance(&p, &m));
                (Euclidean.distance(&p, *b), to_centre)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)))
            .unwrap_or((f64::INFINITY, f64::INFINITY))
    };
    let ends = [
        (distance(long.to), 0, false),
        (distance(long.from), 0, true),
        (distance(short.to), 1, false),
        (distance(short.from), 1, true),
    ];
    let Some(nearest) = ends
        .iter()
        .map(|((d, _), _, _)| *d)
        .filter(|d| *d <= FACADE_DISTANCE_M)
        .min_by(f64::total_cmp)
    else {
        return short;
    };
    let chosen = ends
        .into_iter()
        .filter(|((d, _), _, _)| *d <= nearest + FACADE_TIE_M)
        .min_by(|a, b| a.0.1.total_cmp(&b.0.1));
    match chosen {
        Some((_, 0, true)) => long.reversed(),
        Some((_, 0, false)) => long,
        Some((_, _, true)) => short.reversed(),
        _ => short,
    }
}

/// A step edge (`Freitreppe` line) in the map's CRS: from its first to its last point.
struct StepEdge<'a> {
    a: Coord<f64>,
    b: Coord<f64>,
    feature: &'a Value,
}

impl StepEdge<'_> {
    fn length(&self) -> f64 {
        (self.b - self.a).x.hypot((self.b - self.a).y)
    }

    fn unit(&self) -> Coord<f64> {
        (self.b - self.a) / self.length()
    }

    /// Whether `other` is the same edge (both ends within [`DUPLICATE_EDGE_M`], either way).
    fn same_as(&self, other: &Self) -> bool {
        let near = |p: Coord<f64>, q: Coord<f64>| (p - q).x.hypot((p - q).y) <= DUPLICATE_EDGE_M;
        (near(self.a, other.a) && near(self.b, other.b))
            || (near(self.a, other.b) && near(self.b, other.a))
    }

    /// Whether `other` is a step of the same flight: nearly parallel, its middle within
    /// [`STEP_GAP_M`] of this edge's line, and overlapping it along its length.
    fn same_flight(&self, other: &Self) -> bool {
        let (u, v) = (self.unit(), other.unit());
        let cross = |p: Coord<f64>, q: Coord<f64>| p.x * q.y - p.y * q.x;
        let dot = |p: Coord<f64>, q: Coord<f64>| p.x * q.x + p.y * q.y;
        if cross(u, v).abs() > STEP_PARALLEL_SIN {
            return false;
        }
        let middle = (other.a + other.b) / 2.0;
        if cross(u, middle - self.a).abs() > STEP_GAP_M {
            return false;
        }
        let (s, t) = (dot(u, other.a - self.a), dot(u, other.b - self.a));
        s.max(t).min(self.length()) - s.min(t).max(0.0) > 0.0
    }
}

/// The `Freitreppe` step edges of the `bauwerke-linien` `features`, exact duplicates dropped.
fn step_edges(features: &[Value], crs: Crs) -> Vec<StepEdge<'_>> {
    let mut edges: Vec<StepEdge<'_>> = Vec::new();
    for feature in features.iter().filter(|f| is_step_edge(f)) {
        for line in feature_lines(feature, crs) {
            let (Some(&a), Some(&b)) = (line.0.first(), line.0.last()) else {
                continue;
            };
            let edge = StepEdge { a, b, feature };
            if edge.length() > DUPLICATE_EDGE_M && !edges.iter().any(|e| e.same_as(&edge)) {
                edges.push(edge);
            }
        }
    }
    edges
}

/// The flights of `edges`: groups of two or more edges linked by [`StepEdge::same_flight`], in
/// the order of their first edge. A lone edge says nothing about the stair's length.
fn flights<'e, 'a>(edges: &'e [StepEdge<'a>]) -> Vec<Vec<&'e StepEdge<'a>>> {
    // Union-find over the edges; the flights of one area are few edges.
    let mut parent: Vec<usize> = (0..edges.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            if edges[i].same_flight(&edges[j]) || edges[j].same_flight(&edges[i]) {
                let (a, b) = (root(&mut parent, i), root(&mut parent, j));
                parent[a.max(b)] = a.min(b);
            }
        }
    }
    let mut groups: Vec<(usize, Vec<&StepEdge<'a>>)> = Vec::new();
    for (i, edge) in edges.iter().enumerate() {
        let r = root(&mut parent, i);
        match groups.iter_mut().find(|(g, _)| *g == r) {
            Some((_, group)) => group.push(edge),
            None => groups.push((r, vec![edge])),
        }
    }
    groups
        .into_iter()
        .map(|(_, group)| group)
        .filter(|group| group.len() >= 2)
        .collect()
}

/// The walking axis of a flight of step edges: across the edges, through the middle of their
/// extent along them; as wide as that extent and as long as the flight. Uphill when the
/// terrain rises [`STAIR_MIN_RISE_M`] along it, otherwise in the order of the edges. The rectangle
/// of the flight is returned for the distance to the map area.
fn flight_axis(flight: &[&StepEdge<'_>], terrain: Option<&Raster<f32>>) -> (Axis, Polygon<f64>) {
    let longest = flight
        .iter()
        .max_by(|a, b| a.length().total_cmp(&b.length()))
        .expect("a flight has edges");
    let d = longest.unit();
    let n = Coord { x: -d.y, y: d.x };
    let origin = flight[0].a;
    let dot = |p: Coord<f64>, q: Coord<f64>| p.x * q.x + p.y * q.y;
    let (mut s0, mut s1, mut t0, mut t1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for edge in flight {
        for p in [edge.a, edge.b] {
            let (s, t) = (dot(p - origin, d), dot(p - origin, n));
            (s0, s1, t0, t1) = (s0.min(s), s1.max(s), t0.min(t), t1.max(t));
        }
    }
    let at = |s: f64, t: f64| origin + d * s + n * t;
    let middle = (s0 + s1) / 2.0;
    let mut axis = Axis {
        from: at(middle, t0),
        to: at(middle, t1),
        width: s1 - s0,
    };
    // The first edge lies at the start of the flight unless it is the other end.
    if dot(flight[0].a - origin, n) - t0 > t1 - dot(flight[0].a - origin, n) {
        axis = axis.reversed();
    }
    if let Some(rise) = terrain.and_then(|t| axis.rise(t))
        && rise.abs() >= STAIR_MIN_RISE_M
        && rise < 0.0
    {
        axis = axis.reversed();
    }
    let ring = vec![at(s0, t0), at(s1, t0), at(s1, t1), at(s0, t1), at(s0, t0)];
    (axis, Polygon::new(ring.into(), vec![]))
}

/// 532.6–532.9 candidates for the stair areas and the flights of step edges of `input` within
/// `buffer_m` of `map_area` (map CRS), with the footprints of the symbols of `kind`; `currency`
/// is the fetch date of the layers. A stair wider than [`WIDE_STAIR_FACTOR`] × the widest
/// footprint becomes parallel 532.9 lines one footprint plus the minimum gap apart, tagged `freemapper:size=wide`.
pub fn stair_objects(
    input: &StairInput<'_>,
    kind: MapKind,
    map_area: &Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    currency: NaiveDate,
) -> Result<Vec<MapObject>, GeorefError> {
    let crs = Crs::from_epsg(georef.epsg).unwrap_or(Crs::Utm33);
    // Ground metres per paper millimetre at 100 % symbol size.
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0
        * kind.symbol_enlargement(georef.scale_denominator);
    let mut stairs = StairWriter {
        georef,
        symbol_set,
        metres_per_mm,
        objects: Vec::new(),
    };
    // Flat stairs face the buildings and the monuments they lie at.
    let facades: Vec<Polygon<f64>> = input
        .buildings
        .iter()
        .cloned()
        .chain(
            input
                .features
                .iter()
                .filter(|f| f["properties"]["bezbwf"] == MONUMENT_KIND)
                .flat_map(|f| feature_polygons(f, crs)),
        )
        .collect();
    // The ALKIS stairs: footprint and the objects drawn for it.
    let mut alkis: Vec<(Polygon<f64>, std::ops::Range<usize>)> = Vec::new();
    for feature in input.features.iter().filter(|f| is_stair(f)) {
        for part in feature_polygons(feature, crs) {
            if Euclidean.distance(&part, map_area) > buffer_m {
                continue;
            }
            let Some(axes) = axes(&part) else {
                continue;
            };
            let (axis, guessed) = walking_axis(axes, input.terrain, &facades);
            let mut tags = provenance_tags(STAIR_LAYER, feature, currency);
            if guessed {
                tags.push(("freemapper:direction".to_string(), "guessed".to_string()));
            }
            let start = stairs.objects.len();
            stairs.push(&axis, tags)?;
            alkis.push((part, start..stairs.objects.len()));
        }
    }
    // The NAS stairs. They are stored in the map's CRS: no reprojection (`feature_polygons`
    // takes the layer CRS as given).
    for feature in input.nas.iter().filter(|f| is_nas_stair(f)) {
        let bottom: Vec<_> = input
            .nas
            .iter()
            .filter(|e| is_nas_bottom_edge(e) && names_stair(e, feature))
            .flat_map(|e| feature_lines(e, LAYER_CRS))
            .collect();
        for part in feature_polygons(feature, LAYER_CRS) {
            if Euclidean.distance(&part, map_area) > buffer_m {
                continue;
            }
            let Some(axes) = axes(&part) else {
                continue;
            };
            let from_edge = if bottom.is_empty() {
                None
            } else {
                axis_from_edge(axes.clone(), &bottom)
            };
            let (axis, guessed) = match from_edge {
                Some(axis) => (axis, false),
                None => walking_axis(axes, input.terrain, &facades),
            };
            let mut tags = nas_stair_tags(feature, currency);
            if guessed {
                tags.push(("freemapper:direction".to_string(), "guessed".to_string()));
            }
            let start = stairs.objects.len();
            stairs.push(&axis, tags)?;
            alkis.push((part, start..stairs.objects.len()));
        }
    }
    let edges = step_edges(input.step_edges, crs);
    for flight in flights(&edges) {
        let (axis, outline) = flight_axis(&flight, input.terrain);
        if Euclidean.distance(&outline, map_area) > buffer_m {
            continue;
        }
        let start = stairs.objects.len();
        stairs.push(
            &axis,
            provenance_tags(STEP_EDGE_LAYER, flight[0].feature, currency),
        )?;
        alkis.push((outline, start..stairs.objects.len()));
    }
    for feature in input.survey.iter().filter(|f| is_survey_stair(f)) {
        for part in feature_polygons(feature, crs) {
            if Euclidean.distance(&part, map_area) > buffer_m {
                continue;
            }
            let Some(axes) = axes(&part) else {
                continue;
            };
            let tags = survey_tags(SURVEY_STAIR_LAYER, feature);
            let id = tags
                .iter()
                .find(|(k, _)| k == "freemapper:source_id")
                .map_or("", |(_, v)| v.as_str());
            // ALKIS wins; it names the surveyed stair it stands for.
            if let Some((_, drawn)) = alkis
                .iter()
                .find(|(footprint, _)| same_stair(footprint, &part))
            {
                let also = format!("geoportal-berlin/{SURVEY_STAIR_LAYER}/{id}");
                for object in &mut stairs.objects[drawn.clone()] {
                    add_also(&mut object.tags, &also);
                }
                continue;
            }
            let (axis, guessed) = walking_axis(axes, input.terrain, &facades);
            let mut tags = tags;
            if guessed {
                tags.push(("freemapper:direction".to_string(), "guessed".to_string()));
            }
            stairs.push(&axis, tags)?;
        }
    }
    Ok(stairs.objects)
}

/// The tags of a NAS stair: `freemapper:source=alkis-nas/<file>`, then the tags of every NAS
/// object (`state::nas_feature_tags`); `currency` stands in for a missing `beginnt`.
fn nas_stair_tags(feature: &Value, currency: NaiveDate) -> Vec<(String, String)> {
    let mut tags = vec![(
        "freemapper:source".to_string(),
        format!("{NAS_SOURCE}/{NAS_STAIR_LAYER}"),
    )];
    let own = nas_feature_tags(feature);
    if !own.iter().any(|(k, _)| k == "freemapper:currency") {
        tags.push(("freemapper:currency".to_string(), currency.to_string()));
    }
    tags.extend(own);
    tags
}

/// Whether two stair footprints overlap by [`DUPLICATE_STAIR_SHARE`] of the smaller one.
fn same_stair(a: &Polygon<f64>, b: &Polygon<f64>) -> bool {
    let smaller = a.unsigned_area().min(b.unsigned_area());
    smaller > 0.0 && a.intersection(b).unsigned_area() >= DUPLICATE_STAIR_SHARE * smaller
}

/// Writes the lines of the stairs.
struct StairWriter<'a> {
    georef: &'a Georeferencing,
    symbol_set: &'a SymbolSet,
    metres_per_mm: f64,
    objects: Vec<MapObject>,
}

impl StairWriter<'_> {
    /// One line along `axis` with the footprint nearest to its width; a wide stair (more than
    /// [`WIDE_STAIR_FACTOR`] × the widest footprint) as parallel lines of the widest footprint,
    /// one footprint width plus the minimum gap ([`WIDE_LINE_GAP_MM`]) apart and centred, tagged `freemapper:size=wide`.
    fn push(&mut self, axis: &Axis, mut tags: Vec<(String, String)>) -> Result<(), GeorefError> {
        let to_map = |c: Coord<f64>| {
            self.georef.projected_to_map(ProjectedPoint {
                easting: c.x,
                northing: c.y,
            })
        };
        let widest_m = WIDEST_FOOTPRINT_MM * self.metres_per_mm;
        let wide = axis.width > WIDE_STAIR_FACTOR * widest_m;
        let code = if wide {
            WIDEST_FOOTPRINT
        } else {
            footprint_code(axis.width / self.metres_per_mm)
        };
        let symbol_id = self
            .symbol_set
            .id_for_code(code)
            .expect("symbol set defines 532.6-532.9");
        let offsets: Vec<f64> = if wide {
            let gap_m = WIDE_LINE_GAP_MM * self.metres_per_mm;
            let step = widest_m + gap_m;
            let count = (((axis.width + gap_m) / step).floor() as usize).clamp(1, MAX_WIDE_LINES);
            let spacing = step.max(axis.width / MAX_WIDE_LINES as f64);
            let first = -(count as f64 - 1.0) / 2.0 * spacing;
            tags.push(("freemapper:size".to_string(), "wide".to_string()));
            (0..count).map(|i| first + i as f64 * spacing).collect()
        } else {
            vec![0.0]
        };
        let along = (axis.to - axis.from) / axis.length();
        let across = Coord {
            x: -along.y,
            y: along.x,
        };
        for offset in offsets {
            let shift = across * offset;
            self.objects.push(MapObject {
                symbol_id,
                kind: MapObjectKind::Path,
                coords: vec![to_map(axis.from + shift)?, to_map(axis.to + shift)?],
                tags: tags.clone(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::GridSpec;

    const E: f64 = 392_400.0;
    const N: f64 = 5_820_400.0;

    fn setup(kind: MapKind) -> (Georeferencing, Polygon<f64>, SymbolSet) {
        let centre = Crs::Utm33
            .inverse(ProjectedPoint {
                easting: E,
                northing: N,
            })
            .unwrap();
        let georef =
            Georeferencing::new(centre, kind.default_scale(), "2026-11-08".parse().unwrap())
                .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let ring = vec![
            (e - 100.0, n - 100.0),
            (e + 100.0, n - 100.0),
            (e + 100.0, n + 100.0),
            (e - 100.0, n + 100.0),
            (e - 100.0, n - 100.0),
        ];
        (
            georef,
            Polygon::new(ring.into(), vec![]),
            kind.symbol_set(kind.default_scale()),
        )
    }

    /// A 1 m DGM around the reference point with height `h(e, n)` (metres from the reference
    /// point) at each cell centre.
    fn terrain(h: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let grid = GridSpec {
            west: E - 150.0,
            north: N + 150.0,
            pixel_size: 1.0,
            width: 300,
            height: 300,
        };
        let mut raster = Raster::filled(grid, f32::NAN);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let e = grid.west + f64::from(x) + 0.5 - E;
                let n = grid.north - f64::from(y) - 0.5 - N;
                raster.set(x, y, h(e, n));
            }
        }
        raster
    }

    /// A `Treppe` feature with the corners `points` (metres from the reference point).
    fn stair(kind: &str, points: &[(f64, f64)]) -> Value {
        let mut ring: Vec<[f64; 2]> = points.iter().map(|(e, n)| [E + e, N + n]).collect();
        ring.push(ring[0]);
        serde_json::json!({
            "type": "Feature", "id": "bauwerkeflaechen.DEBE00YYUK0000Ah",
            "geometry": {"type": "MultiPolygon", "coordinates": [[ring]]},
            "properties": {"uuid": "DEBE00YYUK0000Ah", "bwf": "1630", "bezbwf": kind}
        })
    }

    /// A `w` (east) × `h` (north) m stair with its south-west corner at the reference point.
    fn rect_stair(w: f64, h: f64) -> Value {
        stair("Treppe", &[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)])
    }

    /// A `Freitreppe` step edge of `bauwerke-linien` from `a` to `b` (metres from the reference
    /// point).
    fn step_edge(id: &str, a: (f64, f64), b: (f64, f64)) -> Value {
        serde_json::json!({
            "type": "Feature", "id": format!("bauwerkelinien.{id}"),
            "geometry": {"type": "MultiLineString",
                         "coordinates": [[[E + a.0, N + a.1], [E + b.0, N + b.1]]]},
            "properties": {"uuid": id, "bwf": "1630", "bezbwf": "Freitreppe"}
        })
    }

    fn objects(kind: MapKind, features: &[Value], dgm: Option<&Raster<f32>>) -> Vec<MapObject> {
        objects_beside(kind, features, dgm, &[])
    }

    fn objects_beside(
        kind: MapKind,
        features: &[Value],
        terrain: Option<&Raster<f32>>,
        buildings: &[Polygon<f64>],
    ) -> Vec<MapObject> {
        objects_with_edges(kind, features, &[], terrain, buildings)
    }

    fn objects_with_edges(
        kind: MapKind,
        features: &[Value],
        step_edges: &[Value],
        terrain: Option<&Raster<f32>>,
        buildings: &[Polygon<f64>],
    ) -> Vec<MapObject> {
        objects_with_survey(kind, features, step_edges, &[], terrain, buildings)
    }

    fn objects_with_survey(
        kind: MapKind,
        features: &[Value],
        step_edges: &[Value],
        survey: &[Value],
        terrain: Option<&Raster<f32>>,
        buildings: &[Polygon<f64>],
    ) -> Vec<MapObject> {
        objects_with_nas(kind, features, step_edges, survey, &[], terrain, buildings)
    }

    fn objects_with_nas(
        kind: MapKind,
        features: &[Value],
        step_edges: &[Value],
        survey: &[Value],
        nas: &[Value],
        terrain: Option<&Raster<f32>>,
        buildings: &[Polygon<f64>],
    ) -> Vec<MapObject> {
        let (georef, area, set) = setup(kind);
        stair_objects(
            &StairInput {
                features,
                step_edges,
                survey,
                nas,
                terrain,
                buildings,
            },
            kind,
            &area,
            20.0,
            &georef,
            &set,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap()
    }

    /// A stored NAS stair area 1620 with the corners `points` (metres from the reference point).
    fn nas_stair(oid: &str, points: &[(f64, f64)]) -> Value {
        let mut ring: Vec<[f64; 2]> = points.iter().map(|(e, n)| [E + e, N + n]).collect();
        ring.push(ring[0]);
        serde_json::json!({
            "type": "Feature", "id": oid,
            "geometry": {"type": "Polygon", "coordinates": [ring]},
            "properties": {"oid": oid, "nas_type": "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                           "bauwerksfunktion": "1620", "beginnt": "2018-03-19T13:09:16Z"}
        })
    }

    /// The bottom edge 1630 of the stair `stair_oid` along `points`.
    fn nas_edge(oid: &str, stair_oid: &str, points: &[(f64, f64)]) -> Value {
        let line: Vec<[f64; 2]> = points.iter().map(|(e, n)| [E + e, N + n]).collect();
        serde_json::json!({
            "type": "Feature", "id": oid,
            "geometry": {"type": "LineString", "coordinates": line},
            "properties": {"oid": oid, "nas_type": "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                           "bauwerksfunktion": "1630", "gehoert_zu": stair_oid}
        })
    }

    fn code(kind: MapKind, object: &MapObject) -> String {
        let set = kind.symbol_set(kind.default_scale());
        ["532.6", "532.7", "532.8", "532.9"]
            .into_iter()
            .find(|c| set.id_for_code(c) == Some(object.symbol_id))
            .expect("a stair symbol")
            .to_string()
    }

    /// The line ends in metres from the reference point.
    fn ends(kind: MapKind, object: &MapObject) -> [(f64, f64); 2] {
        let (georef, _, _) = setup(kind);
        let p = |i: usize| {
            let p = georef.map_to_projected(object.coords[i]);
            (p.easting - E, p.northing - N)
        };
        [p(0), p(1)]
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 0.05 && (a.1 - b.1).abs() < 0.05
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    fn direction(object: &MapObject) -> Option<&str> {
        tag(object, "freemapper:direction")
    }

    #[test]
    fn stair_direction_follows_the_dtm() {
        let kind = MapKind::Sprint;
        // 6 m east × 3 m north, the ground rising 0.2 m per metre to the north: the short axis
        // rises 0.6 m, the long one not at all. The steps go north, 6 m wide.
        let north = terrain(|_, n| 30.0 + 0.2 * n as f32);
        let found = objects(kind, &[rect_stair(6.0, 3.0)], Some(&north));
        assert_eq!(found.len(), 1);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (3.0, 0.0)) && close(to, (3.0, 3.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(code(kind, &found[0]), "532.9");
        assert_eq!(direction(&found[0]), None);
        let tags: Vec<(&str, &str)> = found[0]
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            tags,
            [
                ("freemapper:source", "geoportal-berlin/bauwerke-flaechen"),
                ("freemapper:source_id", "DEBE00YYUK0000Ah"),
                ("freemapper:currency", "2026-11-09"),
                ("freemapper:alkis", "Treppe"),
            ]
        );

        // The ground falls to the east: the long axis, uphill westwards, 3 m wide.
        let west = terrain(|e, _| 30.0 - 0.1 * e as f32);
        let found = objects(kind, &[rect_stair(6.0, 3.0)], Some(&west));
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (6.0, 1.5)) && close(to, (0.0, 1.5)),
            "{from:?} → {to:?}"
        );
        assert_eq!(code(kind, &found[0]), "532.8");

        // A Freitreppe shaped like ALKIS DEBE00YYUK0000Aj (a parallelogram, 15 m long), on a
        // slope to the north: its long axis uphill, 3 m wide across the rectangle.
        let freitreppe = stair(
            "Freitreppe",
            &[
                (9.981, -17.345),
                (6.849, -15.896),
                (8.529, -30.627),
                (11.661, -32.076),
            ],
        );
        let found = objects(kind, &[freitreppe], Some(&north));
        let [from, to] = ends(kind, &found[0]);
        assert!(to.1 - from.1 > 14.0, "{from:?} → {to:?}");
        assert_eq!(code(kind, &found[0]), "532.8");
        assert_eq!(direction(&found[0]), None);
    }

    #[test]
    fn flat_stair_uses_short_axis_and_is_tagged() {
        let kind = MapKind::Sprint;
        // 0.24 m over 3 m: below the 0.3 m rise. A flight long enough to be the long axis
        // would rise more, so a flat stair is most likely wide: the short axis.
        let gentle = terrain(|_, n| 30.0 + 0.08 * n as f32);
        let no_data = terrain(|_, _| f32::NAN);
        for dgm in [Some(&gentle), Some(&no_data), None] {
            let found = objects(kind, &[rect_stair(6.0, 3.0)], dgm);
            assert_eq!(found.len(), 1);
            let [from, to] = ends(kind, &found[0]);
            assert!(
                (from.0 - 3.0).abs() < 0.05 && (to.0 - 3.0).abs() < 0.05,
                "{from:?} → {to:?}"
            );
            assert!(
                ((to.1 - from.1).abs() - 3.0).abs() < 0.05,
                "{from:?} → {to:?}"
            );
            assert_eq!(direction(&found[0]), Some("guessed"));
            // 6 m wide at 1:4000: the widest footprint (4 m), not yet wide (12 m).
            assert_eq!(code(kind, &found[0]), "532.9");
            assert_eq!(tag(&found[0], "freemapper:size"), None);
        }
    }

    fn rect(e: f64, n: f64, w: f64, h: f64) -> Polygon<f64> {
        let ring = vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)];
        Polygon::new(ring.into(), vec![])
    }

    #[test]
    fn flat_stair_at_a_building_faces_it() {
        let kind = MapKind::Sprint;
        let flat = terrain(|_, _| 30.0);
        // Entrance steps along a façade: 8 m east × 2 m north, the building north of them.
        let building = rect(E - 5.0, N + 2.0, 20.0, 10.0);
        let found = objects_beside(kind, &[rect_stair(8.0, 2.0)], Some(&flat), &[building]);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (4.0, 0.0)) && close(to, (4.0, 2.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), Some("guessed"));
        // A building east of the stair, at its short side: the stair runs towards that edge,
        // along its long axis.
        let east = rect(E + 8.0, N - 5.0, 10.0, 12.0);
        let found = objects_beside(kind, &[rect_stair(8.0, 2.0)], Some(&flat), &[east]);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (0.0, 1.0)) && close(to, (8.0, 1.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), Some("guessed"));
        // A building 5 m away is no façade of the stair: the short axis.
        let away = rect(E - 5.0, N + 7.0, 20.0, 10.0);
        let found = objects_beside(kind, &[rect_stair(8.0, 2.0)], Some(&flat), &[away]);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            ((to.1 - from.1).abs() - 2.0).abs() < 0.05 && (to.0 - from.0).abs() < 0.05,
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), Some("guessed"));
        // A rising DGM still decides over the building.
        let rising = terrain(|e, _| 30.0 + 0.1 * e as f32);
        let building = rect(E - 5.0, N + 2.0, 20.0, 10.0);
        let found = objects_beside(kind, &[rect_stair(8.0, 2.0)], Some(&rising), &[building]);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (0.0, 1.0)) && close(to, (8.0, 1.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), None);
    }

    #[test]
    fn flat_stair_at_a_monument_faces_it() {
        let kind = MapKind::Sprint;
        let flat = terrain(|_, _| 30.0);
        // A plinth stair 8 m east × 2 m north, the monument area of the same layer east of it.
        let monument = stair(
            "Denkmal, Denkstein, Standbild",
            &[(8.0, -4.0), (14.0, -4.0), (14.0, 6.0), (8.0, 6.0)],
        );
        let found = objects(kind, &[rect_stair(8.0, 2.0), monument], Some(&flat));
        assert_eq!(found.len(), 1, "the monument is no stair");
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (0.0, 1.0)) && close(to, (8.0, 1.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), Some("guessed"));
    }

    #[test]
    fn flat_stair_inside_a_monument_area_faces_its_centre() {
        // ALKIS draws the Nationaldenkmal as one area over the plinth and its stairs: every
        // axis end of a plinth stair touches it, so the end nearest its centre decides.
        let kind = MapKind::Sprint;
        let flat = terrain(|_, _| 30.0);
        let monument = |w: f64, e: f64| {
            stair(
                "Denkmal, Denkstein, Standbild",
                &[(w, -5.0), (e, -5.0), (e, 7.0), (w, 7.0)],
            )
        };
        for (area, (from_e, to_e)) in [
            (monument(0.0, 30.0), (0.0, 8.0)),
            (monument(-22.0, 8.0), (8.0, 0.0)),
        ] {
            let found = objects(kind, &[rect_stair(8.0, 2.0), area], Some(&flat));
            assert_eq!(found.len(), 1);
            let [from, to] = ends(kind, &found[0]);
            assert!(
                close(from, (from_e, 1.0)) && close(to, (to_e, 1.0)),
                "{from:?} → {to:?}"
            );
            assert_eq!(direction(&found[0]), Some("guessed"));
        }
    }

    #[test]
    fn very_wide_stair_is_parallel_lines_tagged_wide() {
        let kind = MapKind::Sprint;
        let north = terrain(|_, n| 30.0 + 0.4 * n as f32);
        // A stand 134 m wide and 5.3 m deep, rising north. 532.9 is 4 m wide at 1:4000, so
        // the stand is wider than 3 × 4 m: lines of 532.9 across it, each 4.6 m from the next
        // (4 m footprint plus the 0.15 mm = 0.6 m minimum gap between two lines of one colour,
        // ISSprOM 3.2), so their ticks do not run into each other: 29 of them.
        let found = objects(kind, &[rect_stair(134.0, 5.3)], Some(&north));
        assert_eq!(found.len(), 29);
        let mut xs: Vec<f64> = found
            .iter()
            .map(|o| {
                let [from, to] = ends(kind, o);
                assert!(
                    (from.1).abs() < 0.05 && (to.1 - 5.3).abs() < 0.05,
                    "{from:?} → {to:?}"
                );
                assert!((from.0 - to.0).abs() < 0.05, "{from:?} → {to:?}");
                assert_eq!(code(kind, o), "532.9");
                assert_eq!(tag(o, "freemapper:size"), Some("wide"));
                assert_eq!(direction(o), None);
                from.0
            })
            .collect();
        xs.sort_by(f64::total_cmp);
        for (i, x) in xs.iter().enumerate() {
            let expected = 67.0 + (i as f64 - 14.0) * 4.6;
            assert!(
                (x - expected).abs() < 0.05,
                "line {i} at {x}, not {expected}"
            );
        }
        // 12 m (3 × 4 m) is not wide yet: one line.
        let found = objects(kind, &[rect_stair(12.0, 5.3)], Some(&north));
        assert_eq!(found.len(), 1);
        assert_eq!(tag(&found[0], "freemapper:size"), None);
    }

    #[test]
    fn the_stadium_stand_of_the_viktoriapark_keeps_a_gap_between_its_lines() {
        let kind = MapKind::Sprint;
        // The surveyed stand of the Viktoriapark stadium (ALKIS Berlin `Treppe`, 130 m × 5.3 m),
        // moved to the reference point. The stand leans 2.6° off north; its terraces rise to the
        // west, square to its long side.
        let origin = (389_888.0, 5_816_330.0);
        let ring = [
            (389_894.245, 5_816_265.815),
            (389_900.185, 5_816_395.732),
            (389_900.295, 5_816_398.027),
            (389_895.013, 5_816_398.161),
            (389_894.912, 5_816_395.973),
            (389_888.972, 5_816_266.056),
            (389_888.878, 5_816_264.009),
            (389_894.154, 5_816_263.75),
        ]
        .map(|(e, n)| (e - origin.0, n - origin.1));
        let west = terrain(|e, n| 30.0 - 0.4 * (e - 0.0457 * n) as f32);
        let found = objects(kind, &[stair("Treppe", &ring)], Some(&west));
        // Footprint 4 m (532.9 at 1:4000) plus 0.6 m of gap: at most one line per 4.6 m.
        assert!(found.len() > 20, "{}", found.len());
        let mut ys: Vec<f64> = found.iter().map(|o| ends(kind, o)[0].1).collect();
        ys.sort_by(f64::total_cmp);
        for pair in ys.windows(2) {
            assert!(pair[1] - pair[0] > 4.59, "{pair:?}");
        }
        // They still cover the stand from end to end (the outer lines half a footprint in).
        assert!((ys[ys.len() - 1] - ys[0]) + 4.0 > 0.95 * 130.0, "{ys:?}");
        assert!(found.iter().all(|o| code(kind, o) == "532.9"));
    }

    #[test]
    fn step_edges_become_one_stair_across_them() {
        let kind = MapKind::Sprint;
        // A flight of 5 step edges 3 m long (east), 0.3 m apart (north), with the middle edge
        // twice (the second one the other way round, as ALKIS has some); a collinear edge
        // beside the bottom one that overlaps none of them; a flight of two 2 m edges 1.5 m
        // apart further east; and a lone edge.
        let mut edges: Vec<Value> = (0..5)
            .map(|i| {
                let n = 0.3 * f64::from(i);
                step_edge(&format!("A{i}"), (0.0, n), (3.0, n))
            })
            .collect();
        edges.push(step_edge("A2bis", (3.0, 0.6), (0.0, 0.6)));
        edges.push(step_edge("C", (4.0, 0.0), (7.0, 0.0)));
        edges.push(step_edge("B0", (20.0, 0.0), (22.0, 0.0)));
        edges.push(step_edge("B1", (20.0, 1.5), (22.0, 1.5)));
        edges.push(step_edge("D", (50.0, 50.0), (53.0, 50.0)));
        let steep = |slope: f32| terrain(move |_, n| 30.0 + slope * n as f32);
        let (up_north, up_south) = (steep(0.5), steep(-0.5));
        for (dgm, north) in [
            (Some(&up_north), true),
            (Some(&up_south), false),
            (None, true),
        ] {
            let found = objects_with_edges(kind, &[], &edges, dgm, &[]);
            assert_eq!(found.len(), 2, "{found:?}");
            // The walking direction is across the edges (no DGM needed: never guessed), the
            // stair as wide as the edges are long and as long as the flight.
            let [from, to] = ends(kind, &found[0]);
            let (low, high) = if north { (0.0, 1.2) } else { (1.2, 0.0) };
            assert!(
                close(from, (1.5, low)) && close(to, (1.5, high)),
                "{from:?} → {to:?}"
            );
            assert_eq!(code(kind, &found[0]), "532.8");
            assert_eq!(direction(&found[0]), None);
            assert_eq!(
                tag(&found[0], "freemapper:source"),
                Some("geoportal-berlin/bauwerke-linien")
            );
            assert_eq!(tag(&found[0], "freemapper:source_id"), Some("A0"));
            assert_eq!(tag(&found[0], "freemapper:alkis"), Some("Freitreppe"));
            let [from, to] = ends(kind, &found[1]);
            assert!(
                (from.0 - 21.0).abs() < 0.05 && (to.0 - 21.0).abs() < 0.05,
                "{from:?} → {to:?}"
            );
            assert!(((to.1 - from.1).abs() - 1.5).abs() < 0.05);
            assert_eq!(code(kind, &found[1]), "532.7");
        }
        // Other lines of the layer are no step edges.
        let mut fence = step_edge("Z", (0.0, 0.0), (3.0, 0.0));
        fence["properties"]["bezbwf"] = "Zaun".into();
        let mut fence2 = step_edge("Z2", (0.0, 0.3), (3.0, 0.3));
        fence2["properties"]["bezbwf"] = "Zaun".into();
        assert!(objects_with_edges(kind, &[], &[fence, fence2], None, &[]).is_empty());
    }

    /// A Straßenbefahrung `cc_treppe` feature as it is stored (type and survey date stamped), with
    /// the corners `points` (metres from the reference point).
    fn survey_stair(gis_id: &str, trs: &str, points: &[(f64, f64)]) -> Value {
        let mut ring: Vec<[f64; 2]> = points.iter().map(|(e, n)| [E + e, N + n]).collect();
        ring.push(ring[0]);
        let mut feature = serde_json::json!({
            "type": "Feature", "id": format!("cc_treppe.{gis_id}"),
            "geometry": {"type": "MultiPolygon", "coordinates": [[ring]]},
            "properties": {"bezeichnun": "Treppe", "gis_id": gis_id, "trs": trs, "material": "08",
                           "freemapper:layer": "cc_treppe"}
        });
        freemapper_adapters_de::berlin::stamp_survey_date(&mut feature);
        feature
    }

    fn rect_points(e: f64, n: f64, w: f64, h: f64) -> [(f64, f64); 4] {
        [(e, n), (e + w, n), (e + w, n + h), (e, n + h)]
    }

    #[test]
    fn surveyed_stairs_keep_step_count() {
        let kind = MapKind::Sprint;
        let survey = [
            survey_stair("Fl_20150416_163", "5", &rect_points(0.0, 0.0, 2.0, 4.0)),
            // `NULL` is the WFS's word for "no value": no tag.
            survey_stair(
                "FL_16072014_03142",
                "NULL",
                &rect_points(20.0, 0.0, 2.0, 4.0),
            ),
            survey_stair("unreadable", "7", &rect_points(40.0, 0.0, 2.0, 4.0)),
        ];
        let found = objects_with_survey(kind, &[], &[], &survey, None, &[]);
        assert_eq!(found.len(), 3, "{found:?}");
        assert_eq!(tag(&found[0], "berlin:trs"), Some("5"));
        assert_eq!(tag(&found[1], "berlin:trs"), None);
        assert_eq!(tag(&found[0], "berlin:layer"), Some("cc_treppe"));
        assert_eq!(
            tag(&found[0], "freemapper:source"),
            Some("geoportal-berlin/sb-treppe")
        );
        assert_eq!(
            tag(&found[0], "freemapper:source_id"),
            Some("Fl_20150416_163")
        );
        // The survey date is the currency; an unreadable id gives the dataset date, guessed.
        assert_eq!(tag(&found[0], "freemapper:currency"), Some("2015-04-16"));
        assert_eq!(tag(&found[1], "freemapper:currency"), Some("2014-07-16"));
        assert_eq!(tag(&found[2], "freemapper:currency"), Some("2014-01-01"));
        assert_eq!(tag(&found[2], "freemapper:currency_guessed"), Some("yes"));
        assert_eq!(tag(&found[0], "freemapper:currency_guessed"), None);
        // Width 2 m at 1:4000, 125 % symbols: the same footprint rule as the ALKIS stairs.
        assert_eq!(
            code(kind, &found[0]),
            code(kind, &objects(kind, &[rect_stair(2.0, 4.0)], None)[0])
        );
    }

    #[test]
    fn surveyed_stairs_are_drawn_on_the_schoolyard_map_too() {
        let kind = MapKind::Schoolyard;
        let survey = [survey_stair(
            "Fl_20150416_163",
            "5",
            &rect_points(0.0, 0.0, 0.8, 3.0),
        )];
        let found = objects_with_survey(kind, &[], &[], &survey, None, &[]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(["532.6", "532.7", "532.8", "532.9"].contains(&code(kind, &found[0]).as_str()));
        assert_eq!(tag(&found[0], "berlin:trs"), Some("5"));
        // ALKIS wins here as well.
        let found = objects_with_survey(kind, &[rect_stair(0.8, 3.0)], &[], &survey, None, &[]);
        assert_eq!(found.len(), 1);
        assert_eq!(
            tag(&found[0], "freemapper:also"),
            Some("geoportal-berlin/sb-treppe/Fl_20150416_163")
        );
    }

    #[test]
    fn survey_stair_on_alkis_stair_is_dropped() {
        let kind = MapKind::Sprint;
        // ALKIS wins: a surveyed stair on an ALKIS stair area or on a flight of step edges is
        // the same stair; one beside them stays. The ALKIS stair names the survey one.
        let alkis = [rect_stair(2.0, 4.0)];
        let edges: Vec<Value> = (0..4)
            .map(|i| {
                let n = 0.3 * f64::from(i);
                step_edge(&format!("A{i}"), (20.0, n), (23.0, n))
            })
            .collect();
        let survey = [
            survey_stair("Fl_20150416_1", "5", &rect_points(0.3, 0.2, 2.0, 4.0)),
            survey_stair("Fl_20150416_2", "4", &rect_points(20.1, -0.1, 3.0, 1.2)),
            survey_stair("Fl_20150416_3", "6", &rect_points(40.0, 0.0, 2.0, 4.0)),
        ];
        let found = objects_with_survey(kind, &alkis, &edges, &survey, None, &[]);
        assert_eq!(found.len(), 3, "{found:?}");
        let sources: Vec<_> = found
            .iter()
            .map(|o| tag(o, "freemapper:source").unwrap())
            .collect();
        assert_eq!(
            sources,
            [
                "geoportal-berlin/bauwerke-flaechen",
                "geoportal-berlin/bauwerke-linien",
                "geoportal-berlin/sb-treppe"
            ]
        );
        assert_eq!(
            tag(&found[0], "freemapper:also"),
            Some("geoportal-berlin/sb-treppe/Fl_20150416_1")
        );
        assert_eq!(
            tag(&found[1], "freemapper:also"),
            Some("geoportal-berlin/sb-treppe/Fl_20150416_2")
        );
        assert_eq!(
            tag(&found[2], "freemapper:source_id"),
            Some("Fl_20150416_3")
        );
        // Without the ALKIS stairs the survey ones stay.
        assert_eq!(
            objects_with_survey(kind, &[], &[], &survey, None, &[]).len(),
            3
        );
    }

    #[test]
    fn stair_width_picks_footprint_symbol() {
        // Paper widths of 532.6-532.9 at 100 %: 0.4, 0.5, 0.75 and 1 mm; the nearest one wins.
        for (paper_mm, expected) in [
            (0.3, "532.6"),
            (0.449, "532.6"),
            (0.45, "532.7"),
            (0.62, "532.7"),
            (0.625, "532.8"),
            (0.87, "532.8"),
            (0.875, "532.9"),
            (3.0, "532.9"),
        ] {
            assert_eq!(footprint_code(paper_mm), expected, "{paper_mm} mm");
        }
        // The stairs below rise to the north, along their 8 m.
        let north = terrain(|_, n| 30.0 + 0.2 * n as f32);
        // Sprint 1:4000: the names of the symbols (1.6 / 2 / 3 / 4 m on the ground).
        for (width, expected) in [
            (1.5, "532.6"),
            (2.0, "532.7"),
            (3.0, "532.8"),
            (4.0, "532.9"),
        ] {
            let found = objects(MapKind::Sprint, &[rect_stair(width, 8.0)], Some(&north));
            assert_eq!(code(MapKind::Sprint, &found[0]), expected, "{width} m");
        }
        // Schoolyard 1:1000 with symbols at 125 %: 1.25 m per (100 %) mm, so most stairs are
        // drawn with the widest footprint, the one closest to their real width.
        for (width, expected) in [
            (0.5, "532.6"),
            (0.7, "532.7"),
            (1.5, "532.9"),
            (3.0, "532.9"),
        ] {
            let found = objects(MapKind::Schoolyard, &[rect_stair(width, 8.0)], Some(&north));
            assert_eq!(code(MapKind::Schoolyard, &found[0]), expected, "{width} m");
        }
        // Other areas are no stairs.
        let mut wall = rect_stair(8.0, 2.0);
        wall["properties"]["bezbwf"] = "Mauer".into();
        assert!(objects(MapKind::Sprint, &[wall], None).is_empty());
    }

    /// The Rostock stairs of the Mecklenburg-Vorpommern fixture: the stair area `DxDD`
    /// (about 2.3 m long from south-south-east to north-north-west), its bottom edge on the
    /// south side, and an edge of a stair that is not in the file.
    fn rostock() -> Vec<Value> {
        let text = include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-nas-stairs.gml");
        crate::sources::alkis_nas::parse_nas_gml(text)
            .unwrap()
            .features
    }

    /// The oid of a stored NAS feature.
    fn is_oid(feature: &Value, oid: &str) -> bool {
        feature["properties"]["oid"] == oid
    }

    /// The stairs built from NAS `features` around the Rostock fixture (map area 200 m square).
    /// The DGM1 gets the height `h(e, n)` (metres from the fixture's south-west corner).
    fn nas_stairs(
        features: &[Value],
        terrain: Option<&dyn Fn(f64, f64) -> f32>,
    ) -> (Vec<MapObject>, Georeferencing) {
        nas_stairs_in(Crs::Utm33, features, terrain)
    }

    /// As [`nas_stairs`] for a map in `crs`; `features` are stored in that CRS. A terrain model
    /// is placed in EPSG:25833 only, so it goes with `Crs::Utm33`.
    fn nas_stairs_in(
        crs: Crs,
        features: &[Value],
        terrain: Option<&dyn Fn(f64, f64) -> f32>,
    ) -> (Vec<MapObject>, Georeferencing) {
        let (e0, n0) = (312_090.0, 5_996_807.0);
        let kind = MapKind::Sprint;
        let centre = Crs::Utm33
            .inverse(ProjectedPoint {
                easting: e0,
                northing: n0,
            })
            .unwrap();
        let georef = Georeferencing::new_in(
            crs,
            centre,
            kind.default_scale(),
            "2026-11-09".parse().unwrap(),
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let ring = vec![
            (e - 100.0, n - 100.0),
            (e + 100.0, n - 100.0),
            (e + 100.0, n + 100.0),
            (e - 100.0, n + 100.0),
            (e - 100.0, n - 100.0),
        ];
        let area = Polygon::new(ring.into(), vec![]);
        let grid = GridSpec {
            west: e0 - 150.0,
            north: n0 + 150.0,
            pixel_size: 1.0,
            width: 300,
            height: 300,
        };
        let raster = terrain.map(|h| {
            let mut raster = Raster::filled(grid, f32::NAN);
            for y in 0..grid.height {
                for x in 0..grid.width {
                    let (de, dn) = (
                        grid.west + f64::from(x) + 0.5 - e0,
                        grid.north - f64::from(y) - 0.5 - n0,
                    );
                    raster.set(x, y, h(de, dn));
                }
            }
            raster
        });
        let objects = stair_objects(
            &StairInput {
                features: &[],
                step_edges: &[],
                survey: &[],
                nas: features,
                terrain: raster.as_ref(),
                buildings: &[],
            },
            kind,
            &area,
            20.0,
            &georef,
            &kind.symbol_set(kind.default_scale()),
            "2026-11-09".parse().unwrap(),
        )
        .unwrap();
        (objects, georef)
    }

    /// The northing of the line ends, from the first to the last.
    fn northings(object: &MapObject, georef: &Georeferencing) -> [f64; 2] {
        let n = |i: usize| georef.map_to_projected(object.coords[i]).northing;
        [n(0), n(1)]
    }

    #[test]
    fn nas_stair_runs_up_from_its_bottom_edge() {
        // The ground falls to the north, so the DGM1 alone would send the stair south. The edge
        // on the south side is the bottom: the line starts there and runs north.
        let down = |_: f64, n: f64| 30.0 - 0.5 * n as f32;
        let (found, georef) = nas_stairs(&rostock(), Some(&down));
        assert_eq!(found.len(), 1, "the edge itself is no stair");
        let [from, to] = northings(&found[0], &georef);
        assert!(from < 5_996_808.2 && to > 5_996_809.8, "{from} → {to}");
        assert_eq!(direction(&found[0]), None);
        let tags: Vec<(&str, &str)> = found[0]
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        for expected in [
            ("freemapper:source", "alkis-nas/sonstiges-bauwerk"),
            ("freemapper:source_id", "DEMVHR030000DxDD"),
            ("freemapper:currency", "2018-03-19"),
            ("freemapper:alkis", "Treppe"),
            ("alkis:type", "AX_SonstigesBauwerkOderSonstigeEinrichtung"),
        ] {
            assert!(tags.contains(&expected), "{expected:?} in {tags:?}");
        }
        // 1.2 m wide at 1:4000 (0.3 mm): the narrowest footprint.
        assert_eq!(code(MapKind::Sprint, &found[0]), "532.6");
    }

    #[test]
    fn nas_stair_without_edge_uses_the_dtm() {
        let stair: Vec<Value> = rostock()
            .into_iter()
            .filter(|f| is_oid(f, "DEMVHR030000DxDD"))
            .collect();
        assert_eq!(stair.len(), 1);
        // Rising to the south: the stair runs south. Not guessed.
        let south = |_: f64, n: f64| 30.0 - 0.5 * n as f32;
        let (found, georef) = nas_stairs(&stair, Some(&south));
        assert_eq!(found.len(), 1);
        let [from, to] = northings(&found[0], &georef);
        assert!(from > to, "{from} → {to}");
        assert_eq!(direction(&found[0]), None);
        // Rising to the north: the other way.
        let north = |_: f64, n: f64| 30.0 + 0.5 * n as f32;
        let (found, georef) = nas_stairs(&stair, Some(&north));
        let [from, to] = northings(&found[0], &georef);
        assert!(from < to, "{from} → {to}");
        // No terrain model: the short axis (across the 1.2 m), guessed.
        let (found, georef) = nas_stairs(&stair, None);
        assert_eq!(found.len(), 1);
        assert_eq!(direction(&found[0]), Some("guessed"));
        let [from, to] = northings(&found[0], &georef);
        assert!((from - to).abs() < 0.6, "{from} → {to}");
    }

    #[test]
    fn edge_of_a_stair_outside_the_area_is_ignored() {
        assert_eq!(nas_stairs(&rostock(), None).0.len(), 1, "inside the area");
        // The stair and its edge 1 km east: outside the map area plus buffer, nothing is drawn.
        fn shift(v: &mut Value) {
            match v {
                Value::Array(a) if a.len() == 2 && a.iter().all(Value::is_f64) => {
                    a[0] = serde_json::json!(a[0].as_f64().unwrap() + 1000.0);
                }
                Value::Array(a) => a.iter_mut().for_each(shift),
                Value::Object(o) => o.values_mut().for_each(shift),
                _ => {}
            }
        }
        let mut far = rostock();
        far.iter_mut().for_each(|f| shift(&mut f["geometry"]));
        assert!(nas_stairs(&far, None).0.is_empty());
        // An edge whose stair is not stored (the file's `xCDF`) draws nothing either.
        let orphan: Vec<Value> = rostock()
            .into_iter()
            .filter(|f| is_oid(f, "DEMVHR030000xCDF"))
            .collect();
        assert_eq!(orphan.len(), 1);
        assert!(nas_stairs(&orphan, None).0.is_empty());
    }

    #[test]
    fn survey_stair_on_nas_stair_is_dropped() {
        let kind = MapKind::Sprint;
        // The same ALKIS-wins rule as for the Berlin areas: the surveyed stair on the NAS
        // stair is named in `freemapper:also`; the one beside it stays.
        let nas = [nas_stair(
            "DEMVHR030000AAAA",
            &rect_points(0.0, 0.0, 2.0, 4.0),
        )];
        let survey = [
            survey_stair("Fl_20150416_1", "5", &rect_points(0.3, 0.2, 2.0, 4.0)),
            survey_stair("Fl_20150416_3", "6", &rect_points(40.0, 0.0, 2.0, 4.0)),
        ];
        let found = objects_with_nas(kind, &[], &[], &survey, &nas, None, &[]);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(
            tag(&found[0], "freemapper:source"),
            Some("alkis-nas/sonstiges-bauwerk")
        );
        assert_eq!(
            tag(&found[0], "freemapper:also"),
            Some("geoportal-berlin/sb-treppe/Fl_20150416_1")
        );
        assert_eq!(
            tag(&found[1], "freemapper:source_id"),
            Some("Fl_20150416_3")
        );
    }

    #[test]
    fn nas_bent_bottom_edge_still_gives_the_axis_across_it() {
        let kind = MapKind::Sprint;
        // 6 m east x 3 m north. The edge runs along the south side and returns (a kink): its
        // first and last point lie 0.3 m apart, but the edge as a whole runs east-west.
        let features = [
            nas_stair("DEMVHR030000BBBB", &rect_points(0.0, 0.0, 6.0, 3.0)),
            nas_edge(
                "DEMVHR030000BBBC",
                "DEMVHR030000BBBB",
                &[(0.0, 0.0), (6.0, 0.0), (0.0, 0.3)],
            ),
        ];
        let found = objects_with_nas(kind, &[], &[], &[], &features, None, &[]);
        assert_eq!(found.len(), 1);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (3.0, 0.0)) && close(to, (3.0, 3.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), None);
    }

    #[test]
    fn nas_stair_lands_in_the_same_place_in_a_utm32_map() {
        // A project in EPSG:25832 (Hesse) stores NAS geometry reprojected into it
        // (`fetch_nas_type` → `reproject_feature`), so it is built without moving it again.
        // The same stair built in both CRSs ends at the same geographic place.
        let spot = |crs: Crs, features: &[Value]| {
            let (found, georef) = nas_stairs_in(crs, features, None);
            assert_eq!(found.len(), 1);
            [0, 1].map(|i| {
                let p = georef.map_to_projected(found[0].coords[i]);
                georef.crs().inverse(p).unwrap()
            })
        };
        let utm33 = rostock();
        let utm32: Vec<Value> = utm33
            .iter()
            .cloned()
            .map(|f| crate::sources::alkis::reproject_feature(f, Crs::Utm33, Crs::Utm32))
            .collect();
        assert_ne!(utm32[0]["geometry"], utm33[0]["geometry"]);
        let (a, b) = (spot(Crs::Utm33, &utm33), spot(Crs::Utm32, &utm32));
        for (p, q) in a.iter().zip(&b) {
            assert!(
                (p.lat - q.lat).abs() < 2e-7 && (p.lon - q.lon).abs() < 3e-7,
                "{p:?} vs {q:?}"
            );
        }
    }
}
