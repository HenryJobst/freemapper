//! The two edges of a NAS wall become one 515 centre line (ADR 0063, amendment of 2026-11-09).
//!
//! The full ALKIS (NAS) stores many walls as their two edges, `bauwerksfunktion` 1701
//! (Mauerkante, rechts) and 1702 (Mauerkante, links). Two 515 lines closer than the line width
//! plus the gap (0.4 mm + 0.15 mm on paper) print as one heavy line, so such edges are paired
//! here, on the stored features and before the segments are joined (`build/candidates.rs`),
//! because only the stored feature still says on which side the wall lies.

use std::collections::{BTreeSet, HashMap};

use geo::Simplify as _;
use geo_types::LineString;
use serde_json::Value;

use crate::project::MapKind;

/// Two wall lines closer than this on paper (mm) are read as one: the 515 line width (0.4 mm)
/// plus the gap between lines of one colour (§3.3, 0.15 mm).
pub const WALL_PAIR_MM: f64 = 0.55;
/// Two edges pair only where they are nearly parallel: at most 20° apart (the cosine).
const PARALLEL_COS: f64 = 0.9397;
/// The edges are walked in steps of at most this many ground metres ...
const STEP_M: f64 = 0.25;
/// ... and at most this share of the pair distance.
const STEP_SHARE: f64 = 0.25;
/// The offset to a partner point runs at most 60° off the normal of the edges (the sine of
/// 30°): two pieces of one straight line never pair.
const ACROSS: f64 = 0.5;
/// A point of an edge pairs with the same edge only this many pair widths away along it.
const SELF_APART: f64 = 1.5;
/// The centre line loses the step points within this distance (ground metres).
const SIMPLIFY_M: f64 = 0.02;
/// The NAS type of the walls.
const WALL_TYPE: &str = "AX_SonstigesBauwerkOderSonstigeEinrichtung";
/// The code property of the walls.
const CODE: &str = "bauwerksfunktion";
/// Mauer: a line is the whole wall, an area its outline.
const WALL: &str = "1700";
/// Mauerkante, rechts: the right edge, the wall lies on the left of its direction (measured on
/// the Sanssouci data of 2026-11-09: 388 m of the partners on the left, 48 m on the right).
const RIGHT_EDGE: &str = "1701";
/// Mauerkante, links: the left edge, the wall lies on the right (268 m against 14 m).
const LEFT_EDGE: &str = "1702";
/// Mauermitte: the centre line of a wall.
const CENTRE: &str = "1703";
/// The property that names the edges a centre line stands for (their `oid`s, `; `-joined); the
/// candidates name them in `freemapper:also`.
pub(crate) const PAIRED_ALSO: &str = "paired_also";
/// The property of a centre line built from two edges (`yes`).
pub(crate) const PAIRED: &str = "paired";
/// The `freemapper:alkis` label of a centre line built from two edges.
pub(crate) const PAIRED_LABEL: &str = "Mauer (Mauerkanten gepaart)";

/// The pair distance (ground metres) on a map of `kind` at `scale_denominator`:
/// [`WALL_PAIR_MM`] enlarged like the symbols (§3.1).
pub fn pair_distance_m(kind: MapKind, scale_denominator: u32) -> f64 {
    WALL_PAIR_MM * kind.symbol_enlargement() * f64::from(scale_denominator) / 1000.0
}

type P = [f64; 2];

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}

fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn cross(a: P, b: P) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

fn norm(a: P) -> f64 {
    a[0].hypot(a[1])
}

fn unit(a: P) -> P {
    let n = norm(a);
    if n > 0.0 {
        [a[0] / n, a[1] / n]
    } else {
        [0.0, 0.0]
    }
}

fn mid(a: P, b: P) -> P {
    [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]
}

/// One wall edge: a line of a 1701/1702 feature or a ring of a 1700 area.
struct Edge {
    feature: usize,
    /// The points, walked in steps; a closed edge does not repeat its first point.
    points: Vec<P>,
    /// Whether a point is a vertex of the stored line (not a step point).
    vertex: Vec<bool>,
    closed: bool,
    /// +1: the wall lies on the left of the direction, −1: on the right.
    side: f64,
    /// The distance along the edge of each point.
    at: Vec<f64>,
    length: f64,
    /// The longer side of its bounding box.
    extent: f64,
}

impl Edge {
    fn new(feature: usize, line: &[P], closed: bool, side: f64, step: f64) -> Option<Self> {
        let mut line: Vec<P> = line.to_vec();
        line.dedup();
        if closed && line.len() > 1 && line.first() == line.last() {
            line.pop();
        }
        if line.len() < 2 || (closed && line.len() < 3) {
            return None;
        }
        let (mut points, mut vertex) = (Vec::new(), Vec::new());
        let n = line.len();
        let segments = if closed { n } else { n - 1 };
        for k in 0..segments {
            let (a, b) = (line[k], line[(k + 1) % n]);
            let pieces = (norm(sub(b, a)) / step).ceil().max(1.0) as usize;
            for s in 0..pieces {
                let t = s as f64 / pieces as f64;
                points.push([a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]);
                vertex.push(s == 0);
            }
        }
        if !closed {
            points.push(line[n - 1]);
            vertex.push(true);
        }
        let mut at = Vec::with_capacity(points.len());
        let mut length = 0.0;
        for i in 0..points.len() {
            if i > 0 {
                length += norm(sub(points[i], points[i - 1]));
            }
            at.push(length);
        }
        if closed {
            length += norm(sub(points[0], points[points.len() - 1]));
        }
        let span = |axis: usize| {
            let values = points.iter().map(|p| p[axis]);
            values.clone().fold(f64::MIN, f64::max) - values.fold(f64::MAX, f64::min)
        };
        let extent = span(0).max(span(1));
        Some(Self {
            feature,
            points,
            vertex,
            closed,
            side,
            at,
            length,
            extent,
        })
    }

    fn segments(&self) -> usize {
        if self.closed {
            self.points.len()
        } else {
            self.points.len() - 1
        }
    }

    fn segment(&self, k: usize) -> (P, P) {
        (self.points[k], self.points[(k + 1) % self.points.len()])
    }

    /// The directions of the segments at point `i` (one at a free end, else two).
    fn directions(&self, i: usize) -> Vec<P> {
        let n = self.points.len();
        let before = (self.closed || i > 0).then(|| (i + n - 1) % n);
        let after = (self.closed || i + 1 < n).then(|| (i + 1) % n);
        before
            .map(|b| unit(sub(self.points[i], self.points[b])))
            .into_iter()
            .chain(after.map(|a| unit(sub(self.points[a], self.points[i]))))
            .collect()
    }

    /// The direction at point `i` most nearly parallel to `other` (either way), if it is within
    /// [`PARALLEL_COS`].
    fn along(&self, i: usize, other: P) -> Option<P> {
        self.directions(i)
            .into_iter()
            .map(|d| (dot(d, other).abs(), d))
            .filter(|(c, _)| *c >= PARALLEL_COS)
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, d)| d)
    }

    /// The distance along the edge between two positions (the shorter way round a ring).
    fn apart(&self, a: f64, b: f64) -> f64 {
        let d = (a - b).abs();
        if self.closed {
            d.min(self.length - d)
        } else {
            d
        }
    }
}

/// A grid of segments: (line, segment) by cell.
struct SegmentGrid {
    cell: f64,
    cells: HashMap<(i64, i64), Vec<(usize, usize)>>,
}

impl SegmentGrid {
    fn new(cell: f64) -> Self {
        Self {
            cell,
            cells: HashMap::new(),
        }
    }

    fn key(&self, p: P) -> (i64, i64) {
        (
            (p[0] / self.cell).floor() as i64,
            (p[1] / self.cell).floor() as i64,
        )
    }

    fn add(&mut self, line: usize, segment: usize, (a, b): (P, P)) {
        let (k0, k1) = (
            self.key([a[0].min(b[0]), a[1].min(b[1])]),
            self.key([a[0].max(b[0]), a[1].max(b[1])]),
        );
        for x in k0.0..=k1.0 {
            for y in k0.1..=k1.1 {
                self.cells.entry((x, y)).or_default().push((line, segment));
            }
        }
    }

    /// The segments in the cells within `reach` of `p`, sorted, each once.
    fn near(&self, p: P, reach: f64) -> Vec<(usize, usize)> {
        let (k0, k1) = (
            self.key([p[0] - reach, p[1] - reach]),
            self.key([p[0] + reach, p[1] + reach]),
        );
        let mut found = Vec::new();
        for x in k0.0..=k1.0 {
            for y in k0.1..=k1.1 {
                found.extend(self.cells.get(&(x, y)).into_iter().flatten().copied());
            }
        }
        found.sort_unstable();
        found.dedup();
        found
    }
}

/// The point of segment `(a, b)` nearest to `p`, and its share along the segment (unclamped).
fn foot(p: P, (a, b): (P, P)) -> (P, f64) {
    let d = sub(b, a);
    let length = dot(d, d);
    if length == 0.0 {
        return (a, 0.0);
    }
    let t = dot(sub(p, a), d) / length;
    let s = t.clamp(0.0, 1.0);
    ([a[0] + s * d[0], a[1] + s * d[1]], t)
}

/// What becomes of a point of an edge.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Fate {
    /// No partner: the edge stays here.
    Alone,
    /// Paired, and this edge draws the centre: the partner point, its edge and the pair width.
    Centre { partner: P, edge: usize, width: f64 },
    /// Paired, and the partner draws the centre.
    Gone,
    /// Along a stored centre line (1700 line or 1703): the feature of that line.
    Covered(usize),
}

/// The geometry parts of a feature as lines: (points, closed, from a polygon ring, ring is a
/// hole).
fn parts(geometry: &Value) -> Vec<(Vec<P>, bool, bool, bool)> {
    let line = |v: &Value| -> Vec<P> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| Some([p.get(0)?.as_f64()?, p.get(1)?.as_f64()?]))
            .collect()
    };
    let polygon = |rings: &Value, out: &mut Vec<(Vec<P>, bool, bool, bool)>| {
        for (i, ring) in rings.as_array().into_iter().flatten().enumerate() {
            out.push((line(ring), true, true, i > 0));
        }
    };
    let coordinates = &geometry["coordinates"];
    let mut out = Vec::new();
    match geometry["type"].as_str() {
        Some("LineString") => {
            let l = line(coordinates);
            let closed = l.len() > 3 && l.first() == l.last();
            out.push((l, closed, false, false));
        }
        Some("MultiLineString") => {
            for part in coordinates.as_array().into_iter().flatten() {
                let l = line(part);
                let closed = l.len() > 3 && l.first() == l.last();
                out.push((l, closed, false, false));
            }
        }
        Some("Polygon") => polygon(coordinates, &mut out),
        Some("MultiPolygon") => {
            for rings in coordinates.as_array().into_iter().flatten() {
                polygon(rings, &mut out);
            }
        }
        _ => {}
    }
    out
}

/// Twice the signed area of a ring (positive: counter-clockwise).
fn signed_area(ring: &[P]) -> f64 {
    ring.iter()
        .zip(ring.iter().cycle().skip(1))
        .map(|(a, b)| cross(*a, *b))
        .sum()
}

fn code(feature: &Value) -> Option<&str> {
    let properties = &feature["properties"];
    (properties["nas_type"].as_str() == Some(WALL_TYPE))
        .then(|| properties[CODE].as_str())
        .flatten()
}

/// A centre line built from a run of paired points.
struct Built {
    feature: usize,
    points: Vec<P>,
    closed: bool,
    /// The pair width at the first and the last point.
    widths: (f64, f64),
    partners: BTreeSet<usize>,
}

/// The stored NAS `features` (map CRS, metres) with the wall edges paired: two edges (1701,
/// 1702, or the rings of a 1700 area) closer than `distance` m, nearly parallel, each lying on
/// the wall side of the other, become one centre line where both overlap. What has no partner
/// stays as an edge; a gate cut in one edge is not bridged. A ring paired all round with
/// another ring becomes a closed centre ring; a thin 1700 area (or a closed edge with the wall
/// inside) becomes its centre line, unless it is shorter than `min_length` all round (a pedestal
/// or a short wall stays as stored, it prints as a dot or a bar). Where a
/// stored centre line (1700 line, 1703) runs, the edges along it are dropped and named on it.
/// Centre ends that meet at a corner are snapped together (each moves at most half the pair
/// width). A centre line is a 1700 with the older `beginnt` of its edges, [`PAIRED`] and the
/// other edges in [`PAIRED_ALSO`]. Other features, and wall features that pair with nothing,
/// come back unchanged, in their order.
pub fn pair_wall_edges(features: &[Value], distance: f64, min_length: f64) -> Vec<Value> {
    let step = (distance * STEP_SHARE).min(STEP_M);
    let mut edges: Vec<Edge> = Vec::new();
    let mut centres: Vec<(usize, Vec<P>)> = Vec::new();
    for (f, feature) in features.iter().enumerate() {
        let Some(code) = code(feature) else {
            continue;
        };
        for (line, closed, ring, hole) in parts(&feature["geometry"]) {
            let side = match (code, ring) {
                // The wall is the inside of the area: left of a counter-clockwise outer ring,
                // right of a counter-clockwise hole.
                (WALL, true) => {
                    let ccw = signed_area(&line) > 0.0;
                    if ccw != hole { 1.0 } else { -1.0 }
                }
                (WALL | CENTRE, false) => {
                    if line.len() >= 2 {
                        centres.push((f, line));
                    }
                    continue;
                }
                (RIGHT_EDGE, false) => 1.0,
                (LEFT_EDGE, false) => -1.0,
                _ => continue,
            };
            edges.extend(Edge::new(f, &line, closed, side, step));
        }
    }
    if edges.is_empty() {
        return features.to_vec();
    }
    let cell = distance.max(1.0);
    let mut grid = SegmentGrid::new(cell);
    for (e, edge) in edges.iter().enumerate() {
        for k in 0..edge.segments() {
            grid.add(e, k, edge.segment(k));
        }
    }
    let mut centre_grid = SegmentGrid::new(cell);
    for (c, (_, line)) in centres.iter().enumerate() {
        for k in 0..line.len() - 1 {
            centre_grid.add(c, k, (line[k], line[k + 1]));
        }
    }

    // The fate of every point.
    let mut fates: Vec<Vec<Fate>> = Vec::with_capacity(edges.len());
    // The partner of each paired point: its edge, the position along it and the segment.
    let mut partner_at: Vec<Vec<Option<(usize, f64, usize)>>> = Vec::with_capacity(edges.len());
    for (e, edge) in edges.iter().enumerate() {
        let mut fate = Vec::with_capacity(edge.points.len());
        let mut found_at = Vec::with_capacity(edge.points.len());
        for (i, &p) in edge.points.iter().enumerate() {
            // Along a stored centre line on its wall side: the edge of a wall at most the pair
            // distance thick lies at most half of it off the centre.
            let covered = centre_grid.near(p, distance).into_iter().find(|&(c, k)| {
                let line = &centres[c].1;
                let (q, _) = foot(p, (line[k], line[k + 1]));
                norm(sub(p, q)) <= distance / 2.0
                    && edge
                        .along(i, unit(sub(line[k + 1], line[k])))
                        .is_some_and(|d| cross(d, sub(q, p)) * edge.side > 0.0)
            });
            if let Some((c, _)) = covered {
                fate.push(Fate::Covered(centres[c].0));
                found_at.push(None);
                continue;
            }
            // The nearest partner point: nearly parallel, each on the wall side of the other,
            // and a foot inside the partner (not beyond its free end).
            // The nearest point of each other edge (of the same edge: away from `p` along it).
            let mut nearest: Vec<(usize, f64, usize, P, f64)> = Vec::new();
            for (o, k) in grid.near(p, distance) {
                let other = &edges[o];
                let segment = other.segment(k);
                let (q, t) = foot(p, segment);
                let d = norm(sub(p, q));
                let along = other.at[k] + t.clamp(0.0, 1.0) * norm(sub(segment.1, segment.0));
                if o == e
                    && (edge.extent < min_length || edge.apart(edge.at[i], along) <= SELF_APART * d)
                {
                    continue;
                }
                match nearest.last_mut() {
                    Some(n) if n.0 == o => {
                        if d < n.1 {
                            *n = (o, d, k, q, t);
                        }
                    }
                    _ => nearest.push((o, d, k, q, t)),
                }
            }
            let mut best: Option<(f64, usize, P, f64, usize)> = None;
            for (o, d, k, q, t) in nearest {
                let other = &edges[o];
                // A foot beyond a free end: there is no partner.
                let last = other.segments() - 1;
                if !other.closed && ((k == 0 && t < 0.0) || (k == last && t > 1.0)) {
                    continue;
                }
                if d > distance || d < 1e-6 {
                    continue;
                }
                // At a corner of the partner either of its segments there may be the parallel one.
                let n = other.segments();
                let mut beside = vec![k];
                if t <= 0.0 && (other.closed || k > 0) {
                    beside.push((k + n - 1) % n);
                }
                if t >= 1.0 && (other.closed || k < last) {
                    beside.push((k + 1) % n);
                }
                let Some((direction, tangent)) = beside.into_iter().find_map(|j| {
                    let (a, b) = other.segment(j);
                    let direction = unit(sub(b, a));
                    edge.along(i, direction).map(|tangent| (direction, tangent))
                }) else {
                    continue;
                };
                let segment = other.segment(k);
                // Each on the wall side of the other, the offset mostly across the edges.
                if cross(tangent, sub(q, p)) * edge.side < ACROSS * d
                    || cross(direction, sub(p, q)) * other.side < ACROSS * d
                {
                    continue;
                }
                let along = other.at[k] + t.clamp(0.0, 1.0) * norm(sub(segment.1, segment.0));
                if best.is_none_or(|(bd, bo, _, _, _)| (d, o) < (bd, bo)) {
                    best = Some((d, o, q, along, k));
                }
            }
            match best {
                Some((width, o, q, along, k)) => {
                    fate.push(Fate::Centre {
                        partner: q,
                        edge: o,
                        width,
                    });
                    found_at.push(Some((o, along, k)));
                }
                None => {
                    fate.push(Fate::Alone);
                    found_at.push(None);
                }
            }
        }
        fates.push(fate);
        partner_at.push(found_at);
    }

    // Only edges that pair with each other: a point whose partner pairs, at both ends of the
    // segment it meets, with a third edge (the nearer one) keeps no partner.
    let mutual: Vec<Vec<bool>> = partner_at
        .iter()
        .enumerate()
        .map(|(e, found)| {
            found
                .iter()
                .map(|f| {
                    f.is_none_or(|(o, _, k)| {
                        let n = edges[o].points.len();
                        [k, (k + 1) % n].iter().any(|&j| match fates[o][j] {
                            Fate::Centre { edge, .. } => edge == e,
                            _ => true,
                        })
                    })
                })
                .collect()
        })
        .collect();
    for (e, keep) in mutual.iter().enumerate() {
        for (i, &keep) in keep.iter().enumerate() {
            if !keep {
                fates[e][i] = Fate::Alone;
                partner_at[e][i] = None;
            }
        }
    }

    // Who draws the centre: the edge first in order, on a ring paired with itself the point
    // first along it, counted from a point without a partner.
    for (e, edge) in edges.iter_mut().enumerate() {
        let n = edge.points.len();
        // A ring starts where a stretch without a partner starts, so no stretch wraps round.
        let paired = |f: &Fate| matches!(f, Fate::Centre { .. });
        let kind = |f: &Fate| match f {
            Fate::Alone => 0,
            Fate::Centre { .. } => 1,
            Fate::Gone => 2,
            Fate::Covered(_) => 3,
        };
        let start = if edge.closed {
            (0..n)
                .find(|&i| {
                    let before = &fates[e][(i + n - 1) % n];
                    !paired(&fates[e][i]) && kind(before) != kind(&fates[e][i])
                })
                .unwrap_or(0)
        } else {
            0
        };
        if start > 0 {
            edge.points.rotate_left(start);
            edge.vertex.rotate_left(start);
            fates[e].rotate_left(start);
            partner_at[e].rotate_left(start);
        }
        let shift = edge.at[start];
        let length = edge.length;
        let along = |a: f64| (a - shift).rem_euclid(length.max(f64::MIN_POSITIVE));
        edge.at = edge.at.iter().map(|&a| along(a)).collect();
        edge.at.rotate_left(start);
        for i in 0..n {
            if let (Fate::Centre { .. }, Some((o, at, _))) = (fates[e][i], partner_at[e][i]) {
                let theirs = if o == e { along(at) } else { at };
                if (o, theirs) < (e, edge.at[i]) {
                    fates[e][i] = Fate::Gone;
                }
            }
        }
    }

    // Runs of equal fate kind along each edge.
    let mut built: Vec<Built> = Vec::new();
    let mut remnants: Vec<Vec<Vec<P>>> = vec![Vec::new(); features.len()];
    let mut touched = vec![false; features.len()];
    let mut covered: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); features.len()];
    for (e, edge) in edges.iter().enumerate() {
        let fate = &fates[e];
        if fate.iter().all(|f| *f == Fate::Alone) {
            remnants[edge.feature].push(Vec::new());
            continue;
        }
        touched[edge.feature] = true;
        let n = edge.points.len();
        let kind = |f: &Fate| match f {
            Fate::Alone => 0,
            Fate::Centre { .. } => 1,
            Fate::Gone => 2,
            Fate::Covered(_) => 3,
        };
        let mut i = 0;
        while i < n {
            let k = kind(&fate[i]);
            let mut j = i;
            while j + 1 < n && kind(&fate[j + 1]) == k {
                j += 1;
            }
            let whole = edge.closed && i == 0 && j == n - 1;
            match fate[i] {
                Fate::Alone => {
                    let mut line: Vec<P> = (i..=j)
                        .filter(|&x| x == i || x == j || edge.vertex[x])
                        .map(|x| edge.points[x])
                        .collect();
                    if whole {
                        line.push(line[0]);
                    }
                    if line.len() >= 2 {
                        remnants[edge.feature].push(line);
                    }
                }
                Fate::Centre { .. } => {
                    let mut points = Vec::new();
                    let mut partners = BTreeSet::new();
                    let mut widths = (0.0, 0.0);
                    for (x, (&f, &p)) in fate
                        .iter()
                        .zip(&edge.points)
                        .enumerate()
                        .take(j + 1)
                        .skip(i)
                    {
                        if let Fate::Centre {
                            partner,
                            edge: o,
                            width,
                        } = f
                        {
                            points.push(mid(p, partner));
                            partners.insert(edges[o].feature);
                            if x == i {
                                widths.0 = width;
                            }
                            widths.1 = width;
                        }
                    }
                    if whole {
                        points.push(points[0]);
                    }
                    partners.remove(&edge.feature);
                    if points.len() >= 2 {
                        built.push(Built {
                            feature: edge.feature,
                            points,
                            closed: whole,
                            widths,
                            partners,
                        });
                    }
                }
                Fate::Gone => {}
                Fate::Covered(_) => {
                    for f in &fate[i..=j] {
                        if let Fate::Covered(c) = f {
                            covered[*c].insert(edge.feature);
                        }
                    }
                }
            }
            i = j + 1;
        }
    }
    snap_corners(&mut built, step);

    // The features in their order: each touched one by what is left of it and the centre lines
    // it draws, a stored centre line with the edges it stands for.
    let oid = |f: usize| {
        features[f]["properties"]["oid"]
            .as_str()
            .unwrap_or_default()
    };
    let mut out = Vec::with_capacity(features.len());
    for (f, feature) in features.iter().enumerate() {
        if !covered[f].is_empty() {
            let mut kept = feature.clone();
            let names: BTreeSet<&str> = covered[f].iter().map(|&c| oid(c)).collect();
            kept["properties"][PAIRED_ALSO] =
                names.into_iter().collect::<Vec<_>>().join("; ").into();
            out.push(kept);
            continue;
        }
        if !touched[f] {
            out.push(feature.clone());
            continue;
        }
        let lines: Vec<&Vec<P>> = remnants[f].iter().filter(|l| l.len() >= 2).collect();
        if !lines.is_empty() {
            let mut rest = feature.clone();
            rest["geometry"] = if lines.len() == 1 {
                serde_json::json!({"type": "LineString", "coordinates": lines[0]})
            } else {
                serde_json::json!({"type": "MultiLineString", "coordinates": lines})
            };
            out.push(rest);
        }
        for centre in built.iter().filter(|b| b.feature == f) {
            out.push(centre_feature(features, centre));
        }
    }
    out
}

/// The feature of a centre line: the properties of its edge as a 1700 with the older `beginnt`
/// of the edges, [`PAIRED`] and the other edges in [`PAIRED_ALSO`].
fn centre_feature(features: &[Value], centre: &Built) -> Value {
    let own = &features[centre.feature];
    let mut feature = own.clone();
    let simplified = LineString::from(
        centre
            .points
            .iter()
            .map(|p| (p[0], p[1]))
            .collect::<Vec<_>>(),
    )
    .simplify(SIMPLIFY_M);
    let mut points: Vec<P> = simplified.coords().map(|c| [c.x, c.y]).collect();
    if centre.closed && points.len() < 4 {
        points = centre.points.clone();
    }
    feature["geometry"] = serde_json::json!({"type": "LineString", "coordinates": points});
    let properties = &mut feature["properties"];
    properties[CODE] = WALL.into();
    properties[PAIRED] = "yes".into();
    let oldest = std::iter::once(centre.feature)
        .chain(centre.partners.iter().copied())
        .filter_map(|f| features[f]["properties"]["beginnt"].as_str())
        .min();
    if let Some(oldest) = oldest {
        properties["beginnt"] = oldest.into();
    }
    let names: BTreeSet<&str> = centre
        .partners
        .iter()
        .filter_map(|&f| features[f]["properties"]["oid"].as_str())
        .collect();
    if !names.is_empty() {
        properties[PAIRED_ALSO] = names.into_iter().collect::<Vec<_>>().join("; ").into();
    }
    feature
}

/// Snaps the open ends of the centre lines that meet at a corner: two ends whose lines cross
/// within half the pair width (plus a `step`) of both go to the crossing; two ends that do not
/// cross but lie within half the larger pair width of each other go to their middle. Each end
/// snaps once, the ends taken in order.
fn snap_corners(built: &mut [Built], step: f64) {
    // (line, at the last point): the end point, its direction outwards and half its width.
    let ends: Vec<(usize, bool)> = built
        .iter()
        .enumerate()
        .filter(|(_, b)| !b.closed)
        .flat_map(|(i, _)| [(i, false), (i, true)])
        .collect();
    let end = |built: &[Built], (i, last): (usize, bool)| -> (P, P, f64) {
        let points = &built[i].points;
        let n = points.len();
        let (p, before, width) = if last {
            (points[n - 1], points[n - 2], built[i].widths.1)
        } else {
            (points[0], points[1], built[i].widths.0)
        };
        (p, unit(sub(p, before)), width / 2.0)
    };
    let mut snapped = vec![false; ends.len()];
    for a in 0..ends.len() {
        if snapped[a] {
            continue;
        }
        for b in a + 1..ends.len() {
            if snapped[b] || ends[a].0 == ends[b].0 {
                continue;
            }
            let (p, u, ha) = end(built, ends[a]);
            let (q, v, hb) = end(built, ends[b]);
            let denominator = cross(u, v);
            let target = if denominator.abs() > 0.17 {
                // p + s·u = q + r·v
                let s = cross(sub(q, p), v) / denominator;
                let x = [p[0] + s * u[0], p[1] + s * u[1]];
                (norm(sub(x, p)) <= ha + step && norm(sub(x, q)) <= hb + step).then_some(x)
            } else {
                (norm(sub(p, q)) <= ha.max(hb) + step).then(|| mid(p, q))
            };
            let Some(x) = target else {
                continue;
            };
            for (i, last) in [ends[a], ends[b]] {
                let points = &mut built[i].points;
                let at = if last { points.len() - 1 } else { 0 };
                points[at] = x;
            }
            snapped[a] = true;
            snapped[b] = true;
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanssouci, Potsdam (EPSG:25833).
    const AT: (f64, f64) = (366_300.0, 5_807_200.0);
    /// 2.2 m: the pair distance at 1:4000.
    const SPRINT: f64 = 2.2;
    /// 4 m: the shortest line at 1:4000 (1 mm).
    const MIN: f64 = 4.0;

    fn wall(id: &str, code: &str, beginnt: &str, geometry: Value) -> Value {
        serde_json::json!({"type": "Feature", "id": id, "geometry": geometry,
            "properties": {"oid": id, "nas_type": WALL_TYPE, CODE: code, "beginnt": beginnt}})
    }

    fn line(points: &[(f64, f64)]) -> Value {
        let points: Vec<[f64; 2]> = points.iter().map(|(x, y)| [AT.0 + x, AT.1 + y]).collect();
        serde_json::json!({"type": "LineString", "coordinates": points})
    }

    /// The points of a LineString feature off [`AT`].
    fn points(feature: &Value) -> Vec<(f64, f64)> {
        assert_eq!(feature["geometry"]["type"], "LineString", "{feature}");
        feature["geometry"]["coordinates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_f64().unwrap() - AT.0, p[1].as_f64().unwrap() - AT.1))
            .collect()
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).hypot(a.1 - b.1) < 0.05
    }

    fn centres(found: &[Value]) -> Vec<&Value> {
        found
            .iter()
            .filter(|f| f["properties"][PAIRED] == "yes")
            .collect()
    }

    /// A wall 20 m long and `width` m wide along the x axis: its right edge (1701) runs east at
    /// y = 0 with the wall on its left, its left edge (1702) runs east at y = `width` with the
    /// wall on its right.
    fn straight(width: f64) -> Vec<Value> {
        vec![
            wall(
                "r",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.0), (20.0, 0.0)]),
            ),
            wall(
                "l",
                LEFT_EDGE,
                "2013-01-29",
                line(&[(0.0, width), (20.0, width)]),
            ),
        ]
    }

    #[test]
    fn close_edges_become_one_centre_line() {
        let found = pair_wall_edges(&straight(1.0), SPRINT, MIN);
        assert_eq!(found.len(), 1, "{found:?}");
        let centre = &found[0];
        let p = points(centre);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(close(p[0], (0.0, 0.5)) && close(p[1], (20.0, 0.5)), "{p:?}");
        let properties = &centre["properties"];
        assert_eq!(properties["oid"], "r");
        assert_eq!(properties[CODE], WALL);
        assert_eq!(properties[PAIRED], "yes");
        assert_eq!(properties[PAIRED_ALSO], "l");
        // The older date of the two edges.
        assert_eq!(properties["beginnt"], "2013-01-29");
    }

    #[test]
    fn edges_further_apart_stay() {
        let features = straight(2.5);
        assert_eq!(pair_wall_edges(&features, SPRINT, MIN), features);
        // At 1:1000 on the schoolyard the pair distance is 0.69 m.
        let school = pair_distance_m(MapKind::Schoolyard, 1000);
        assert!((school - 0.6875).abs() < 1e-9);
        assert!((pair_distance_m(MapKind::Sprint, 4000) - SPRINT).abs() < 1e-9);
        let features = straight(1.0);
        assert_eq!(pair_wall_edges(&features, school, 1.0), features);
        assert_eq!(pair_wall_edges(&straight(0.5), school, 1.0).len(), 1);
    }

    #[test]
    fn walls_back_to_back_across_a_path_stay_apart() {
        // Two walls with a 1.5 m path between them: the left edge of the lower wall and the
        // right edge of the upper wall face away from each other.
        let features = vec![
            wall(
                "a",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.0), (20.0, 0.0)]),
            ),
            wall(
                "b",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 1.5), (20.0, 1.5)]),
            ),
        ];
        assert_eq!(pair_wall_edges(&features, SPRINT, MIN), features);
        // The same two codes, one of them drawn the other way round, face each other.
        let features = vec![
            wall(
                "a",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.0), (20.0, 0.0)]),
            ),
            wall(
                "b",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(20.0, 1.5), (0.0, 1.5)]),
            ),
        ];
        assert_eq!(centres(&pair_wall_edges(&features, SPRINT, MIN)).len(), 1);
    }

    #[test]
    fn edges_at_an_angle_stay() {
        let features = vec![
            wall(
                "r",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.0), (10.0, 0.0)]),
            ),
            // 30° off, within 2.2 m over its first metres.
            wall(
                "l",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.3), (8.66, 5.3)]),
            ),
        ];
        assert_eq!(pair_wall_edges(&features, SPRINT, MIN), features);
    }

    #[test]
    fn gate_in_one_edge_is_not_bridged() {
        let features = vec![
            wall(
                "r",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.0), (20.0, 0.0)]),
            ),
            wall(
                "l1",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(0.0, 1.0), (9.0, 1.0)]),
            ),
            wall(
                "l2",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(11.0, 1.0), (20.0, 1.0)]),
            ),
        ];
        let found = pair_wall_edges(&features, SPRINT, MIN);
        let built = centres(&found);
        assert_eq!(built.len(), 2, "{found:?}");
        let (a, b) = (points(built[0]), points(built[1]));
        assert!(close(a[0], (0.0, 0.5)), "{a:?}");
        assert!((a[a.len() - 1].0 - 9.0).abs() < 0.3, "{a:?}");
        assert!((b[0].0 - 11.0).abs() < 0.3, "{b:?}");
        assert!(close(b[b.len() - 1], (20.0, 0.5)), "{b:?}");
        // The edge stays only in the gate.
        let rest: Vec<&Value> = found
            .iter()
            .filter(|f| f["properties"]["oid"] == "r" && f["properties"][PAIRED].is_null())
            .collect();
        assert_eq!(rest.len(), 1, "{found:?}");
        let r = points(rest[0]);
        assert!(
            r.iter().all(|p| p.0 > 8.7 && p.0 < 11.3 && p.1 == 0.0),
            "{r:?}"
        );
    }

    #[test]
    fn centre_only_where_both_edges_overlap() {
        let features = vec![
            wall(
                "r",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.0), (20.0, 0.0)]),
            ),
            wall(
                "l",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(5.0, 1.0), (25.0, 1.0)]),
            ),
        ];
        let found = pair_wall_edges(&features, SPRINT, MIN);
        assert_eq!(found.len(), 3, "{found:?}");
        let r = points(&found[0]);
        assert!(
            close(r[0], (0.0, 0.0)) && (r[1].0 - 5.0).abs() < 0.3,
            "{r:?}"
        );
        let c = points(&found[1]);
        assert!(
            (c[0].0 - 5.0).abs() < 0.3 && close(c[1], (20.0, 0.5)),
            "{c:?}"
        );
        let l = points(&found[2]);
        assert!(
            (l[0].0 - 20.0).abs() < 0.3 && close(l[1], (25.0, 1.0)),
            "{l:?}"
        );
    }

    #[test]
    fn corner_ends_meet() {
        // An L-shaped wall 1 m thick, each leg with its own two edges: the outer edges run
        // along y = 0 and x = 0, the inner ones along y = 1 and x = 1. The wall lies inside.
        let features = vec![
            // Outer, bottom leg running west: the wall (y > 0) is on its right.
            wall(
                "ob",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(20.0, 0.0), (0.0, 0.0)]),
            ),
            // Outer, left leg running north: the wall (x > 0) is on its right.
            wall(
                "ol",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(0.0, 0.0), (0.0, 20.0)]),
            ),
            // Inner, bottom leg running west: the wall (y < 1) is on its left.
            wall(
                "ib",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(20.0, 1.0), (1.0, 1.0)]),
            ),
            // Inner, left leg running north: the wall (x < 1) is on its left.
            wall(
                "il",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(1.0, 1.0), (1.0, 20.0)]),
            ),
        ];
        let found = pair_wall_edges(&features, SPRINT, MIN);
        let built = centres(&found);
        assert_eq!(built.len(), 2, "{found:?}");
        let ends: Vec<(f64, f64)> = built
            .iter()
            .flat_map(|f| {
                let p = points(f);
                [p[0], p[p.len() - 1]]
            })
            .collect();
        assert_eq!(
            ends.iter().filter(|p| close(**p, (0.5, 0.5))).count(),
            2,
            "{ends:?}"
        );
    }

    #[test]
    fn concentric_rings_become_one_closed_centre_ring() {
        // A basin wall: the outer edge counter-clockwise with the wall on its left (inside),
        // the inner edge counter-clockwise with the wall on its right (outside).
        let square = |a: f64, b: f64| line(&[(a, a), (b, a), (b, b), (a, b), (a, a)]);
        let features = vec![
            wall("outer", RIGHT_EDGE, "2015-03-01", square(0.0, 10.0)),
            wall("inner", LEFT_EDGE, "2015-03-01", square(1.0, 9.0)),
        ];
        let found = pair_wall_edges(&features, SPRINT, MIN);
        assert_eq!(found.len(), 1, "{found:?}");
        let p = points(&found[0]);
        assert_eq!(p.first(), p.last());
        for corner in [(0.5, 0.5), (9.5, 0.5), (9.5, 9.5), (0.5, 9.5)] {
            assert!(p.iter().any(|q| close(*q, corner)), "{corner:?} {p:?}");
        }
    }

    #[test]
    fn thin_wall_area_becomes_its_centre_line() {
        let ring = |cw: bool| {
            let mut r = [(0.0, 0.0), (20.0, 0.0), (20.0, 0.6), (0.0, 0.6), (0.0, 0.0)];
            if cw {
                r.reverse();
            }
            let points: Vec<[f64; 2]> = r.iter().map(|(x, y)| [AT.0 + x, AT.1 + y]).collect();
            serde_json::json!({"type": "Polygon", "coordinates": [points]})
        };
        for cw in [false, true] {
            let found = pair_wall_edges(&[wall("w", WALL, "2015-03-01", ring(cw))], SPRINT, MIN);
            let built = centres(&found);
            assert_eq!(built.len(), 1, "{found:?}");
            let p = points(built[0]);
            assert!(p.iter().all(|q| (q.1 - 0.3).abs() < 0.05), "{p:?}");
            let xs: Vec<f64> = p.iter().map(|q| q.0).collect();
            let (lo, hi) = (
                xs.iter().copied().fold(f64::MAX, f64::min),
                xs.iter().copied().fold(f64::MIN, f64::max),
            );
            assert!(lo < 0.5 && hi > 19.5, "{p:?}");
            // The short ends (0.6 m) are left over; the 1 mm limit drops them later.
            for rest in found.iter().filter(|f| f["properties"][PAIRED].is_null()) {
                let total: f64 = rest["geometry"]["coordinates"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|l| {
                        let l: Vec<[f64; 2]> =
                            serde_json::from_value(l.clone()).unwrap_or_default();
                        l.windows(2).map(|w| norm(sub(w[1], w[0]))).sum::<f64>()
                    })
                    .sum();
                assert!(total < 2.0, "{rest}");
            }
        }
        // A wide wall area stays its outline.
        let wide = {
            let r = [(0.0, 0.0), (20.0, 0.0), (20.0, 4.0), (0.0, 4.0), (0.0, 0.0)];
            let points: Vec<[f64; 2]> = r.iter().map(|(x, y)| [AT.0 + x, AT.1 + y]).collect();
            wall(
                "w",
                WALL,
                "2015-03-01",
                serde_json::json!({"type": "Polygon", "coordinates": [points]}),
            )
        };
        assert_eq!(
            pair_wall_edges(std::slice::from_ref(&wide), SPRINT, MIN),
            [wide]
        );
    }

    #[test]
    fn an_edge_pairs_only_with_an_edge_that_pairs_back() {
        // Sanssouci, DEBBAL540001vq48/49/4a: a wall 1.4 m thick between x = 0.5 and 1.9 (its
        // right edge `a` runs south with the wall on its left, its left edge `b` runs south
        // with the wall on its right), and 0.5 m west of it another right edge `c`. `c` faces
        // `b`, but `b` pairs with `a`, the nearer one: one centre line, `c` stays.
        let features = vec![
            wall(
                "c",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 10.0), (0.0, -10.0)]),
            ),
            wall(
                "b",
                LEFT_EDGE,
                "2015-03-01",
                line(&[(1.9, 5.0), (1.9, 0.0)]),
            ),
            wall(
                "a",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.5, 5.0), (0.5, 0.0)]),
            ),
        ];
        let found = pair_wall_edges(&features, SPRINT, MIN);
        let built = centres(&found);
        assert_eq!(built.len(), 1, "{found:?}");
        assert!(
            points(built[0]).iter().all(|p| (p.0 - 1.2).abs() < 0.05),
            "{found:?}"
        );
        assert_eq!(found[0], features[0]);
    }

    #[test]
    fn small_round_wall_stays_as_stored() {
        // A round pedestal 2 m across, drawn clockwise as a left edge: the wall is inside.
        // Its centre would be a point, so the ring stays; it prints as a dot.
        let mut ring: Vec<(f64, f64)> = (0..16)
            .map(|k| {
                let a = -std::f64::consts::TAU * f64::from(k) / 16.0;
                (a.cos(), a.sin())
            })
            .collect();
        ring.push(ring[0]);
        let features = vec![wall("p", LEFT_EDGE, "2015-03-01", line(&ring))];
        assert_eq!(pair_wall_edges(&features, SPRINT, MIN), features);
        // A wall area 3 m long stays its outline too.
        let r = [(0.0, 0.0), (3.0, 0.0), (3.0, 0.6), (0.0, 0.6), (0.0, 0.0)];
        let points: Vec<[f64; 2]> = r.iter().map(|(x, y)| [AT.0 + x, AT.1 + y]).collect();
        let short = wall(
            "w",
            WALL,
            "2015-03-01",
            serde_json::json!({"type": "Polygon", "coordinates": [points]}),
        );
        assert_eq!(
            pair_wall_edges(std::slice::from_ref(&short), SPRINT, MIN),
            [short]
        );
    }

    #[test]
    fn stored_centre_line_is_kept_and_its_edges_dropped() {
        let mut features = straight(1.0);
        features.push(wall(
            "m",
            CENTRE,
            "2015-03-01",
            line(&[(0.0, 0.5), (20.0, 0.5)]),
        ));
        let found = pair_wall_edges(&features, SPRINT, MIN);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0]["properties"]["oid"], "m");
        assert_eq!(found[0]["properties"][CODE], CENTRE);
        assert_eq!(found[0]["properties"][PAIRED_ALSO], "l; r");
        assert!(found[0]["properties"][PAIRED].is_null());
        assert_eq!(found[0]["geometry"], features[2]["geometry"]);
    }

    #[test]
    fn other_features_and_lone_edges_stay() {
        let fence = |id: &str, y: f64| wall(id, "1740", "2015-03-01", line(&[(0.0, y), (20.0, y)]));
        let retaining = |id: &str, code: &str, y: f64| {
            wall(id, code, "2015-03-01", line(&[(0.0, y), (20.0, y)]))
        };
        let features = vec![
            fence("f1", 0.0),
            fence("f2", 1.0),
            retaining("s1", "1721", 10.0),
            retaining("s2", "1722", 11.0),
            wall(
                "lone",
                RIGHT_EDGE,
                "2015-03-01",
                line(&[(0.0, 30.0), (20.0, 30.0)]),
            ),
        ];
        assert_eq!(pair_wall_edges(&features, SPRINT, MIN), features);
    }
}
