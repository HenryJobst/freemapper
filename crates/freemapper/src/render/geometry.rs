//! Path geometry for rendering (`docs/render/oom-render-reference.md` §2, §7): parts, Bézier
//! flattening, arc length, dashes, mid-symbol positions and border offsets.
//!
//! Coordinates are paper millimetres with y down, as in OOM.

use super::symbols::{Dash, Mid};
use crate::omap::MapCoord;
use crate::omap::coord::{FLAG_CLOSE, FLAG_CURVE_START, FLAG_HOLE};

pub type Pt = (f64, f64);

/// One part of a path: a polyline (Béziers flattened) and whether it is closed.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub points: Vec<Pt>,
    pub closed: bool,
}

/// Bézier flattening: chords at most 1 mm, control polygon at most 0.005 mm longer than the chord.
const MAX_CHORD: f64 = 1.0;
const MAX_ERROR: f64 = 0.005;
/// Points closer than this are skipped when looking for a direction (0.025 mm).
const TANGENT_EPSILON: f64 = 0.025;

fn mm(c: MapCoord) -> Pt {
    (f64::from(c.x) / 1000.0, f64::from(c.y) / 1000.0)
}

fn dist(a: Pt, b: Pt) -> f64 {
    (b.0 - a.0).hypot(b.1 - a.1)
}

fn sub(a: Pt, b: Pt) -> Pt {
    (a.0 - b.0, a.1 - b.1)
}

fn normalize(v: Pt) -> Pt {
    let l = v.0.hypot(v.1);
    if l > 0.0 {
        (v.0 / l, v.1 / l)
    } else {
        (0.0, 0.0)
    }
}

/// Right of travel with y down: (x, y) → (−y, x).
fn perp_right(v: Pt) -> Pt {
    (-v.1, v.0)
}

fn lerp(a: Pt, b: Pt, t: f64) -> Pt {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

/// Appends the cubic Bézier p0…p3 without p0, halving until flat enough.
fn flatten_cubic(p0: Pt, p1: Pt, p2: Pt, p3: Pt, out: &mut Vec<Pt>, depth: u32) {
    let chord = dist(p0, p3);
    let polygon = dist(p0, p1) + dist(p1, p2) + dist(p2, p3);
    if depth >= 16 || (chord <= MAX_CHORD && polygon - chord <= MAX_ERROR) {
        out.push(p3);
        return;
    }
    let (a, b, c) = (lerp(p0, p1, 0.5), lerp(p1, p2, 0.5), lerp(p2, p3, 0.5));
    let (d, e) = (lerp(a, b, 0.5), lerp(b, c, 0.5));
    let m = lerp(d, e, 0.5);
    flatten_cubic(p0, a, d, m, out, depth + 1);
    flatten_cubic(m, e, c, p3, out, depth + 1);
}

/// The parts of an OOM path: they end at coordinates flagged `FLAG_HOLE`; a part is closed when
/// its last coordinate is flagged `FLAG_CLOSE`. Béziers are flattened.
pub fn parts(coords: &[MapCoord]) -> Vec<Part> {
    let n = coords.len();
    let mut parts = Vec::new();
    let mut points: Vec<Pt> = Vec::new();
    let mut i = 0;
    // At each step `coords[i]` is the last point of the part so far.
    while i < n {
        if points.is_empty() {
            points.push(mm(coords[i]));
        }
        let c = coords[i];
        if c.flags & FLAG_HOLE != 0 || i + 1 == n {
            parts.push(Part {
                points: std::mem::take(&mut points),
                closed: c.flags & FLAG_CLOSE != 0,
            });
            i += 1;
        } else if c.flags & FLAG_CURVE_START != 0 && i + 3 < n {
            let p0 = mm(c);
            let (p1, p2, p3) = (mm(coords[i + 1]), mm(coords[i + 2]), mm(coords[i + 3]));
            flatten_cubic(p0, p1, p2, p3, &mut points, 0);
            i += 3;
        } else {
            points.push(mm(coords[i + 1]));
            i += 1;
        }
    }
    parts
}

pub fn length(points: &[Pt]) -> f64 {
    points.windows(2).map(|w| dist(w[0], w[1])).sum()
}

/// Cumulative arc length at every point.
fn cumulative(points: &[Pt]) -> Vec<f64> {
    let mut cum = Vec::with_capacity(points.len());
    let mut total = 0.0;
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            total += dist(points[i - 1], *p);
        }
        cum.push(total);
    }
    cum
}

/// The point at arc length `s` and the index of the segment it lies on.
fn locate(points: &[Pt], cum: &[f64], s: f64) -> (Pt, usize) {
    let s = s.clamp(0.0, *cum.last().unwrap_or(&0.0));
    let i = cum
        .partition_point(|&c| c < s)
        .saturating_sub(1)
        .min(points.len().saturating_sub(2));
    let seg = cum[i + 1] - cum[i];
    let t = if seg > 0.0 { (s - cum[i]) / seg } else { 0.0 };
    (lerp(points[i], points[i + 1], t), i)
}

/// Position and direction (radians, y down) at arc length `s`: the segment direction, the
/// bisector at a vertex, the single direction at an open end.
pub fn at(part: &Part, s: f64) -> (Pt, f64) {
    let points = &part.points;
    if points.len() < 2 {
        return (points.first().copied().unwrap_or((0.0, 0.0)), 0.0);
    }
    let cum = cumulative(points);
    let (pos, _) = locate(points, &cum, s);
    let n = points.len();
    // The nearest points before and after `pos` that are not within the epsilon.
    let after = |from: usize| -> Option<Pt> {
        let mut k = from;
        loop {
            if k < n && dist(points[k], pos) > TANGENT_EPSILON {
                return Some(points[k]);
            }
            k += 1;
            if k >= n {
                return None;
            }
        }
    };
    let i_next = cum.partition_point(|&c| c <= s);
    let i_prev = cum.partition_point(|&c| c < s);
    let mut next = after(i_next.min(n));
    let mut prev = (0..i_prev.min(n))
        .rev()
        .map(|k| points[k])
        .find(|&p| dist(p, pos) > TANGENT_EPSILON);
    if part.closed {
        if next.is_none() {
            next = points
                .iter()
                .skip(1)
                .copied()
                .find(|&p| dist(p, pos) > TANGENT_EPSILON);
        }
        if prev.is_none() {
            prev = points
                .iter()
                .rev()
                .skip(1)
                .copied()
                .find(|&p| dist(p, pos) > TANGENT_EPSILON);
        }
    }
    let out = next.map(|p| normalize(sub(p, pos))).unwrap_or((0.0, 0.0));
    let inc = prev.map(|p| normalize(sub(pos, p))).unwrap_or((0.0, 0.0));
    let t = (out.0 + inc.0, out.1 + inc.1);
    (pos, t.1.atan2(t.0))
}

/// The polyline between arc lengths `a` and `b`, through the vertices in between.
fn between(points: &[Pt], cum: &[f64], a: f64, b: f64) -> Vec<Pt> {
    let (start, i) = locate(points, cum, a);
    let (end, j) = locate(points, cum, b);
    let mut out = vec![start];
    for k in i + 1..=j {
        if cum[k] > a && cum[k] < b {
            out.push(points[k]);
        }
    }
    out.push(end);
    out
}

/// The dashes of a part as polylines; one polyline (the whole part) when it is too short
/// (reference §2.2). The dash length is stretched so that the line starts and ends with a full
/// dash; on closed parts the two halves at the closing point form one dash.
pub fn dashes(part: &Part, dash: &Dash) -> Vec<Vec<Pt>> {
    let points = &part.points;
    let cum = cumulative(points);
    let l = *cum.last().unwrap_or(&0.0);
    let (d, b, g) = (
        dash.dash / 1000.0,
        dash.gap / 1000.0,
        dash.in_group_gap / 1000.0,
    );
    let k = dash.group.max(1);
    let kf = f64::from(k);
    let half = dash.half_outer || part.closed;
    let h = if half { 2u32 } else { 0 };
    let hf = f64::from(h);
    let group = kf * d + (kf - 1.0) * g;
    let period = group + b;
    let nf = hf + (l + b - hf * d / 2.0) / period;
    let lo = nf.floor();
    let min_opt_dashes = f64::from(2 * k - h / 2);
    let sw = 0.2 * period / kf;
    if l <= 0.0 || (lo <= 1.0 && l < 2.0 * period - min_opt_dashes * sw) {
        return vec![points.clone()];
    }
    let hi = nf.ceil();
    let n = if lo < 1.0 {
        hi
    } else {
        let dlo = (l + b - lo * period) / lo;
        let dhi = (hi * period - (l + b)) / hi;
        if dlo > dhi { hi } else { lo }
    };
    let dash_len = ((l - (n - 1.0) * b - n * (kf - 1.0) * g) * 2.0 / (2.0 * n * kf - hf)).max(0.0);
    let groups = n as u32;
    let mut out = Vec::new();
    let mut pos = 0.0;
    for grp in 1..=groups {
        for i in 1..=k {
            let first = grp == 1 && i == 1;
            let last = grp == groups && i == k;
            let halved = (first && half) != (last && half);
            let len = if halved { dash_len / 2.0 } else { dash_len };
            out.push(between(points, &cum, pos, pos + len));
            pos += len;
            if i < k {
                pos += g;
            }
        }
        if grp < groups {
            pos += b;
        }
    }
    if part.closed && out.len() > 1 {
        let first = out.remove(0);
        let last = out.last_mut().expect("more than one");
        last.extend(first.into_iter().skip(1));
    }
    out
}

/// Positions and directions of a line's mid symbols (reference §2.4).
pub fn mid_positions(part: &Part, mid: &Mid) -> Vec<(Pt, f64)> {
    let cum = cumulative(&part.points);
    let l = *cum.last().unwrap_or(&0.0);
    let (s, e, q) = (
        mid.segment / 1000.0,
        mid.end / 1000.0,
        mid.distance / 1000.0,
    );
    let m = mid.per_spot.max(1);
    let msl = f64::from(m - 1) * q;
    let mut spots = Vec::new();
    if s <= 0.0 || l <= 0.0 {
        return spots;
    }
    let mut place = |p: f64| spots.push(at(part, p));
    if e == 0.0 && !part.closed {
        place(0.0);
    }
    let seg = (l - 2.0 * e).max(0.0) - msl;
    let raw = (seg / (s + msl)).max(if e == 0.0 { 1.0 } else { 0.0 });
    let (lo, hi) = (raw.floor(), raw.ceil());
    if e > 0.0 {
        if l <= msl {
            if mid.at_least_one {
                place(0.0);
                place(l);
            }
        } else {
            let dlo = (l - lo * s - (lo + 1.0) * msl - 2.0 * e).abs();
            let dhi = (l - hi * s - (hi + 1.0) * msl - 2.0 * e).abs();
            let (n, dev) = if dlo >= dhi { (hi, -dhi) } else { (lo, dlo) };
            let ideal = n * s + 2.0 * e;
            let e2 = e + dev * e / ideal;
            let s2 = s + dev * s / ideal;
            if s2 >= 0.0 && (mid.at_least_one || hi > 0.0 || l > 2.0 * e - (s + msl) / 2.0) {
                for i in 0..=n as u32 {
                    for j in 0..m {
                        place(e2 + f64::from(i) * (s2 + msl) + f64::from(j) * q);
                    }
                }
            }
        }
    } else {
        if l > msl {
            let dlo = (l - lo * s - (lo + 1.0) * msl).abs() / lo;
            let dhi = (l - hi * s - (hi + 1.0) * msl).abs() / hi;
            let n = if dlo > dhi { hi } else { lo };
            let a = (l - (n + 1.0) * msl) / n + msl;
            if a >= msl {
                'spots: for i in 0..=n as u32 {
                    for j in 0..m {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        if i == n as u32 && j == m - 1 {
                            break 'spots;
                        }
                        place(f64::from(i) * a + f64::from(j) * q);
                    }
                }
            }
        }
        place(l);
    }
    spots
}

/// The polyline offset by `shift` mm to the right of travel (negative: left), mitred at corners
/// (reference §2.3).
pub fn offset(points: &[Pt], closed: bool, shift: f64) -> Vec<Pt> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    // Without the duplicated closing point, so that tangents wrap around.
    let pts: Vec<Pt> = if closed && points[0] == points[n - 1] {
        points[..n - 1].to_vec()
    } else {
        points.to_vec()
    };
    let m = pts.len();
    let add = |a: Pt, v: Pt, f: f64| (a.0 + v.0 * f, a.1 + v.1 * f);
    let mut out = Vec::with_capacity(m + 1);
    for i in 0..m {
        let c = pts[i];
        let prev = if i > 0 {
            Some(pts[i - 1])
        } else if closed {
            Some(pts[m - 1])
        } else {
            None
        };
        let next = if i + 1 < m {
            Some(pts[i + 1])
        } else if closed {
            Some(pts[0])
        } else {
            None
        };
        let t_in = prev.map(|p| normalize(sub(c, p)));
        let t_out = next.map(|p| normalize(sub(p, c)));
        match (t_in, t_out) {
            (None, Some(t)) | (Some(t), None) => out.push(add(c, perp_right(t), shift)),
            (Some(t_in), Some(t_out)) => {
                let mid = normalize((t_in.0 + t_out.0, t_in.1 + t_out.1));
                let cross = t_out.0 * t_in.1 - t_in.0 * t_out.1;
                let base = add(c, perp_right(t_out), shift);
                if cross.abs() < 1e-12 && mid != (0.0, 0.0) {
                    out.push(base);
                } else if cross * shift > 0.0 {
                    // Outer side: the miter point, or a bevel for very sharp corners.
                    if mid.0 * t_in.0 + mid.1 * t_in.1 <= 0.4472 {
                        out.push(add(c, perp_right(t_in), shift));
                        out.push(base);
                    } else {
                        let pr = perp_right(mid);
                        let phi = (pr.0 * t_in.0 + pr.1 * t_in.1).clamp(-1.0, 1.0).acos();
                        let off = (shift / phi.tan()).abs();
                        out.push(add(base, t_out, -off));
                    }
                } else {
                    // Inner side: the intersection of the offset lines, bounded by the segments.
                    let pr = perp_right(mid);
                    let phi = (pr.0 * t_in.0 + pr.1 * t_in.1).clamp(-1.0, 1.0).acos();
                    let off = -(shift / phi.tan()).abs();
                    let len_in = prev.map_or(f64::INFINITY, |p| dist(p, c));
                    let len_out = next.map_or(f64::INFINITY, |p| dist(p, c));
                    if phi.tan() >= 1.0 || off.abs() < len_in.min(len_out) {
                        out.push(add(base, t_out, -off));
                    } else if len_in < len_out {
                        out.push(add(base, t_out, len_in));
                    } else {
                        out.push(add(add(c, perp_right(t_in), shift), t_in, -len_out));
                    }
                }
            }
            (None, None) => out.push(c),
        }
    }
    if closed {
        let first = out[0];
        out.push(first);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::symbols::PointDef;

    fn line(points: &[Pt]) -> Part {
        Part {
            points: points.to_vec(),
            closed: false,
        }
    }

    fn dash_506() -> Dash {
        Dash {
            dash: 1500.0,
            gap: 370.0,
            group: 1,
            in_group_gap: 0.0,
            half_outer: false,
        }
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn parts_split_at_holes_and_know_closed() {
        let mut coords = vec![
            MapCoord::new(0, 0),
            MapCoord::new(1000, 0),
            MapCoord::new(1000, 1000),
            MapCoord::new(0, 0),
        ];
        coords[3].flags = FLAG_HOLE | FLAG_CLOSE;
        coords.extend([MapCoord::new(5000, 0), MapCoord::new(6000, 0)]);
        let parts = parts(&coords);
        assert_eq!(parts.len(), 2);
        assert!(parts[0].closed && !parts[1].closed);
        assert_eq!(parts[1].points, [(5.0, 0.0), (6.0, 0.0)]);
    }

    #[test]
    fn bezier_is_flattened_within_tolerance() {
        // A quarter circle of radius 10 mm as one cubic Bézier.
        let k = 0.552_284_75 * 10_000.0;
        let mut coords = vec![
            MapCoord::new(10_000, 0),
            MapCoord::new(10_000, k as i32),
            MapCoord::new(k as i32, 10_000),
            MapCoord::new(0, 10_000),
        ];
        coords[0].flags = FLAG_CURVE_START;
        let part = &parts(&coords)[0];
        assert!(part.points.len() > 10);
        for w in part.points.windows(2) {
            let chord = ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
            assert!(chord <= MAX_CHORD + 1e-9);
        }
        for p in &part.points {
            let r = (p.0 * p.0 + p.1 * p.1).sqrt();
            assert!((r - 10.0).abs() < 0.01, "{p:?} at r {r}");
        }
        let quarter = std::f64::consts::FRAC_PI_2 * 10.0;
        assert!((length(&part.points) - quarter).abs() < 0.01);
    }

    #[test]
    fn dashes_are_stretched_to_whole_dashes() {
        let dashes = dashes(&line(&[(0.0, 0.0), (10.0, 0.0)]), &dash_506());
        assert_eq!(dashes.len(), 6);
        let lengths: Vec<f64> = dashes.iter().map(|d| length(d)).collect();
        let d = (10.0 - 5.0 * 0.37) / 6.0;
        assert!(lengths.iter().all(|&l| near(l, d)), "{lengths:?}");
        assert!(near(dashes[0][0].0, 0.0) && near(dashes[5].last().unwrap().0, 10.0));
        assert!(near(dashes[1][0].0, d + 0.37));
    }

    #[test]
    fn dashes_run_through_corners() {
        // 6 mm: three dashes of 1.753 mm; the second spans the corner at 3 mm.
        let dashes = dashes(&line(&[(0.0, 0.0), (3.0, 0.0), (3.0, 3.0)]), &dash_506());
        assert_eq!(dashes.len(), 3);
        assert!(dashes[1].contains(&(3.0, 0.0)), "{dashes:?}");
        let total: f64 = dashes.iter().map(|d| length(d)).sum();
        assert!(near(total + (dashes.len() - 1) as f64 * 0.37, 6.0));
    }

    #[test]
    fn short_dashed_line_is_solid() {
        let dashes = dashes(&line(&[(0.0, 0.0), (2.5, 0.0)]), &dash_506());
        assert_eq!(dashes, [vec![(0.0, 0.0), (2.5, 0.0)]]);
    }

    #[test]
    fn closed_dash_halves_meet() {
        let square = Part {
            points: vec![(0.0, 0.0), (5.0, 0.0), (5.0, 5.0), (0.0, 5.0), (0.0, 0.0)],
            closed: true,
        };
        let dashes = dashes(&square, &dash_506());
        // The first and last half dashes form one dash across the closing point.
        let at_origin = |p: &Pt| p.0.abs() < 1e-9 && p.1.abs() < 1e-9;
        let joined = dashes.iter().find(|d| d.iter().any(at_origin)).unwrap();
        let first = joined.first().unwrap();
        let last = joined.last().unwrap();
        assert!(
            first.1 > 0.0 && first.0 == 0.0,
            "starts on the left edge: {joined:?}"
        );
        assert!(
            last.0 > 0.0 && last.1 == 0.0,
            "ends on the top edge: {joined:?}"
        );
        let total: f64 = dashes.iter().map(|d| length(d)).sum();
        assert!(near(total + dashes.len() as f64 * 0.37, 20.0), "{total}");
    }

    fn fence_mid() -> Mid {
        Mid {
            symbol: Box::new(PointDef {
                rotatable: true,
                dot: None,
                ring: None,
                elements: Vec::new(),
            }),
            segment: 3750.0,
            end: 2325.0,
            per_spot: 1,
            distance: 0.0,
            at_least_one: false,
        }
    }

    #[test]
    fn mid_symbols_fill_the_line() {
        // 20 mm: n·3.75 + 4.65 closest to 20 for n = 4 (19.65); scaled by 20/19.65.
        let spots = mid_positions(&line(&[(0.0, 0.0), (20.0, 0.0)]), &fence_mid());
        assert_eq!(spots.len(), 5);
        let f = 20.0 / 19.65;
        assert!(near(spots[0].0.0, 2.325 * f));
        assert!(near(spots[1].0.0 - spots[0].0.0, 3.75 * f));
        assert!(near(spots[4].0.0, 20.0 - 2.325 * f));
        assert!(spots.iter().all(|s| near(s.1, 0.0)));
    }

    #[test]
    fn short_fence_has_no_tick() {
        assert!(mid_positions(&line(&[(0.0, 0.0), (2.5, 0.0)]), &fence_mid()).is_empty());
        let one = mid_positions(&line(&[(0.0, 0.0), (3.0, 0.0)]), &fence_mid());
        assert_eq!(one.len(), 1);
        assert!(near(one[0].0.0, 1.5));
    }

    #[test]
    fn tangent_at_vertex_is_the_bisector() {
        let part = line(&[(0.0, 0.0), (5.0, 0.0), (5.0, 5.0)]);
        let (p, angle) = at(&part, 5.0);
        assert_eq!(p, (5.0, 0.0));
        assert!(near(angle, std::f64::consts::FRAC_PI_4));
        let (_, along) = at(&part, 2.0);
        assert!(near(along, 0.0));
        let (q, down) = at(&part, 7.5);
        assert!(near(q.1, 2.5) && near(down, std::f64::consts::FRAC_PI_2));
    }

    #[test]
    fn offset_of_a_right_angle_is_mitred() {
        // Going east then south (y down): the right side is the inside of the corner.
        let right = offset(&[(0.0, 0.0), (5.0, 0.0), (5.0, 5.0)], false, 1.0);
        assert_eq!(right.len(), 3);
        assert!(near(right[0].0, 0.0) && near(right[0].1, 1.0));
        assert!(near(right[1].0, 4.0) && near(right[1].1, 1.0), "{right:?}");
        let left = offset(&[(0.0, 0.0), (5.0, 0.0), (5.0, 5.0)], false, -1.0);
        assert!(near(left[1].0, 6.0) && near(left[1].1, -1.0), "{left:?}");
        assert!(near(left[2].0, 6.0) && near(left[2].1, 5.0));
    }
}
