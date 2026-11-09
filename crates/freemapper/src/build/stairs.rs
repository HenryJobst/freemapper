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
//! The width across the walking direction, as paper width at the map scale, picks the footprint
//! symbol nearest to it. A stair wider than [`WIDE_STAIR_FACTOR`] × the widest footprint (a
//! stadium stand) is drawn as parallel 532.9 lines across its width.

use chrono::NaiveDate;
use geo::{Distance as _, Euclidean, MinimumRotatedRect as _};
use geo_types::{Coord, Polygon};
use serde_json::Value;

use crate::build::berlin::{feature_lines, feature_polygons, provenance_tags};
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
/// At most this many parallel lines for one wide stair; wider ones get them farther apart.
const MAX_WIDE_LINES: usize = 100;
/// A flat stair whose side lies this close to a building or monument faces it.
const FACADE_DISTANCE_M: f64 = 0.5;
/// Axis ends this much farther from a facade than the nearest end count as touching it too.
const FACADE_TIE_M: f64 = 0.05;
/// `bezbwf` of the monument areas (flat stairs at them face them).
const MONUMENT_KIND: &str = "Denkmal, Denkstein, Standbild";
/// The layer and `bezbwf` of the step edges.
pub const STEP_EDGE_LAYER: &str = "bauwerke-linien";
const STEP_EDGE_KIND: &str = "Freitreppe";
/// Step edges whose middle lies this close to another edge's line belong to one flight.
const STEP_GAP_M: f64 = 2.0;
/// Step edges of one flight are parallel within about 15° (the sine of the angle).
const STEP_PARALLEL_SIN: f64 = 0.26;
/// Step edges whose ends are this close are the same edge (ALKIS has some twice).
const DUPLICATE_EDGE_M: f64 = 0.05;

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

/// Whether any of the `features` of a layer is a stair area (`derive` keeps the terrain then).
pub fn has_stairs(features: &[Value]) -> bool {
    features.iter().any(is_stair)
}

/// What the stairs are built from.
pub struct StairInput<'a> {
    /// The `bauwerke-flaechen` features (EPSG:25833); only the stair areas are used.
    pub features: &'a [Value],
    /// The `bauwerke-linien` features (EPSG:25833); only the `Freitreppe` step edges are used.
    pub step_edges: &'a [Value],
    /// The DGM1 in the map's CRS, if one is stored.
    pub terrain: Option<&'a Raster<f32>>,
    /// The 521 footprints in the map's CRS, for the direction of flat entrance steps.
    pub buildings: &'a [Polygon<f64>],
}

/// One axis of a stair rectangle: the centre line from side to side, and the width across it.
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
/// footprint becomes parallel 532.9 lines one footprint apart, tagged `freemapper:size=wide`.
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
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0 * kind.symbol_enlargement();
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
            stairs.push(&axis, tags)?;
        }
    }
    let edges = step_edges(input.step_edges, crs);
    for flight in flights(&edges) {
        let (axis, outline) = flight_axis(&flight, input.terrain);
        if Euclidean.distance(&outline, map_area) > buffer_m {
            continue;
        }
        stairs.push(
            &axis,
            provenance_tags(STEP_EDGE_LAYER, flight[0].feature, currency),
        )?;
    }
    Ok(stairs.objects)
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
    /// one footprint width apart and centred, tagged `freemapper:size=wide`.
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
            let count = ((axis.width / widest_m).floor() as usize).min(MAX_WIDE_LINES);
            let spacing = widest_m.max(axis.width / MAX_WIDE_LINES as f64);
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
        (georef, Polygon::new(ring.into(), vec![]), kind.symbol_set())
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
        let (georef, area, set) = setup(kind);
        stair_objects(
            &StairInput {
                features,
                step_edges,
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

    fn code(kind: MapKind, object: &MapObject) -> String {
        let set = kind.symbol_set();
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
        // the stand is wider than 3 × 4 m: 33 lines 4 m apart across it, each 532.9.
        let found = objects(kind, &[rect_stair(134.0, 5.3)], Some(&north));
        assert_eq!(found.len(), 33);
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
            let expected = 67.0 + (i as f64 - 16.0) * 4.0;
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
}
