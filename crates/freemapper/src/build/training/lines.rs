//! Middle lines and offsets for the training level (spec 2026-11-14, "The second half, as
//! designed on 2026-11-15"): the geometry with which two lines side by side become their middle
//! and a stretch of a line moves away from another. Paper mm, y down, as `conflicts::Footprints`
//! uses. Only +, −, ×, ÷ and `sqrt` decide a point, so that every system draws the same.

use geo_types::Coord;

/// A polyline in paper mm with the length along it at each point (by `sqrt`, the same on every
/// system).
#[derive(Debug, Clone)]
pub(super) struct Polyline {
    pub(super) points: Vec<Coord<f64>>,
    pub(super) at: Vec<f64>,
}

impl Polyline {
    pub(super) fn new(points: Vec<Coord<f64>>) -> Self {
        let mut at = Vec::with_capacity(points.len());
        let mut total = 0.0;
        for (i, p) in points.iter().enumerate() {
            if i > 0 {
                let (dx, dy) = (p.x - points[i - 1].x, p.y - points[i - 1].y);
                total += (dx * dx + dy * dy).sqrt();
            }
            at.push(total);
        }
        Self { points, at }
    }

    pub(super) fn length(&self) -> f64 {
        self.at.last().copied().unwrap_or(0.0)
    }

    /// The point `s` along the line.
    pub(super) fn point(&self, s: f64) -> Coord<f64> {
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

    /// The length along the line to the point of it nearest `c`.
    pub(super) fn locate(&self, c: Coord<f64>) -> f64 {
        let mut best = (f64::INFINITY, 0.0);
        for (i, w) in self.points.windows(2).enumerate() {
            let span = self.at[i + 1] - self.at[i];
            let (dx, dy) = (w[1].x - w[0].x, w[1].y - w[0].y);
            let t = if span > 0.0 {
                (((c.x - w[0].x) * dx + (c.y - w[0].y) * dy) / (span * span)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (ex, ey) = (w[0].x + t * dx - c.x, w[0].y + t * dy - c.y);
            let distance = (ex * ex + ey * ey).sqrt();
            if distance < best.0 {
                best = (distance, self.at[i] + t * span);
            }
        }
        best.1
    }

    /// The line from `from` to `to` along it.
    pub(super) fn between(&self, from: f64, to: f64) -> Vec<Coord<f64>> {
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

/// The spacing of the samples `beside` takes along `a`, paper mm.
const BESIDE_STEP_MM: f64 = 0.05;

/// The stretch of `a` that lies within `within` mm of `b`, as arc-length ranges on `a` (sorted,
/// merged, each at least `min_len` long). `a` is sampled every 0.05 mm; two stretches with a gap
/// below `min_len / 2` between them are one.
pub(super) fn beside(a: &Polyline, b: &Polyline, within: f64, min_len: f64) -> Vec<(f64, f64)> {
    if a.points.len() < 2 || b.points.is_empty() {
        return Vec::new();
    }
    let reach = within * within;
    let mut runs: Vec<(f64, f64)> = Vec::new();
    let mut open: Option<(f64, f64)> = None;
    for s in samples(0.0, a.length(), BESIDE_STEP_MM) {
        let p = a.point(s);
        if distance2(p, closest(b, p)) <= reach {
            open = Some(open.map_or((s, s), |(from, _)| (from, s)));
        } else if let Some(run) = open.take() {
            runs.push(run);
        }
    }
    runs.extend(open);
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for run in runs {
        match merged.last_mut() {
            Some(last) if run.0 - last.1 < min_len / 2.0 => last.1 = run.1,
            _ => merged.push(run),
        }
    }
    merged.retain(|&(from, to)| to - from >= min_len);
    merged
}

/// The middle of `a` and `b` over `range` of `a`: samples every `step` mm on `a`, each paired
/// with the closest point of `b`, the midpoints simplified with tolerance `step / 4`
/// ([`simplify`]).
pub(super) fn middle(a: &Polyline, b: &Polyline, range: (f64, f64), step: f64) -> Vec<(f64, f64)> {
    if a.points.len() < 2 || b.points.is_empty() {
        return Vec::new();
    }
    let from = range.0.clamp(0.0, a.length());
    let to = range.1.clamp(from, a.length());
    let midpoints: Vec<Coord<f64>> = samples(from, to, step)
        .map(|s| {
            let p = a.point(s);
            let q = closest(b, p);
            Coord {
                x: (p.x + q.x) / 2.0,
                y: (p.y + q.y) / 2.0,
            }
        })
        .collect();
    simplify(&midpoints, step / 4.0)
        .into_iter()
        .map(|c| (c.x, c.y))
        .collect()
}

/// `points` simplified by Ramer–Douglas–Peucker with `tolerance` (mm): between two points kept,
/// the point farthest from the segment joining them (of as far, the last) is kept when it lies
/// more than `tolerance` from it, and the stretches on either side are simplified the same way;
/// the first and the last point always stay. As `geo::Simplify`, but it compares squared
/// distances, so only +, −, × and ÷ decide and every system keeps the same points.
pub(super) fn simplify(points: &[Coord<f64>], tolerance: f64) -> Vec<Coord<f64>> {
    if points.len() <= 2 || tolerance <= 0.0 {
        return points.to_vec();
    }
    let limit = tolerance * tolerance;
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut open = vec![(0, points.len() - 1)];
    while let Some((first, last)) = open.pop() {
        if last <= first + 1 {
            continue;
        }
        let (mut farthest, mut most) = (first, 0.0);
        for (k, &p) in points.iter().enumerate().take(last).skip(first + 1) {
            let d = segment_distance2(p, points[first], points[last]);
            if d >= most {
                (farthest, most) = (k, d);
            }
        }
        if most > limit {
            keep[farthest] = true;
            open.push((first, farthest));
            open.push((farthest, last));
        }
    }
    points
        .iter()
        .zip(&keep)
        .filter_map(|(&p, &kept)| kept.then_some(p))
        .collect()
}

/// The squared distance from `p` to the segment from `a` to `b`.
fn segment_distance2(p: Coord<f64>, a: Coord<f64>, b: Coord<f64>) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length2 = dx * dx + dy * dy;
    if length2 == 0.0 {
        return distance2(p, a);
    }
    let r = ((p.x - a.x) * dx + (p.y - a.y) * dy) / length2;
    if r <= 0.0 {
        distance2(p, a)
    } else if r >= 1.0 {
        distance2(p, b)
    } else {
        let cross = (a.y - p.y) * dx - (a.x - p.x) * dy;
        cross * cross / length2
    }
}

/// `line` with the stretch `range` moved by `shift` mm to the side `side` (+1 right, -1 left of
/// the direction, y down), with linear ramps of length `ramp` before and after; points are
/// resampled every `step` in the moved part, and the vertices there stay. A point inside a
/// segment moves along the segment's normal, a vertex along the normalized sum of the normals of
/// its two segments. A closed line (first point equal to the last) stays closed, and a stretch
/// or ramp runs on over its seam. `shift` is not negative; `side` gives the direction.
pub(super) fn shifted(
    line: &Polyline,
    range: (f64, f64),
    shift: f64,
    side: f64,
    ramp: f64,
    step: f64,
) -> Vec<(f64, f64)> {
    let mut points = line.points.clone();
    points.dedup();
    if points.len() < 2 {
        return points.iter().map(|c| (c.x, c.y)).collect();
    }
    let closed = points.len() > 2 && points[0] == points[points.len() - 1];
    let line = Polyline::new(points);
    let total = line.length();
    let (lo, hi) = (range.0 - ramp, range.1 + ramp);
    // The shift at `s` along the line, with the ramps.
    let open_offset = |s: f64| {
        if s < lo || s > hi {
            0.0
        } else if s < range.0 {
            shift * (s - lo) / ramp
        } else if s <= range.1 {
            shift
        } else {
            shift * (hi - s) / ramp
        }
    };
    let offset = |s: f64| {
        if closed {
            open_offset(s)
                .max(open_offset(s - total))
                .max(open_offset(s + total))
        } else {
            open_offset(s)
        }
    };
    let normals: Vec<Coord<f64>> = line
        .points
        .windows(2)
        .map(|w| right_normal(w[0], w[1]))
        .collect();
    let last = normals.len() - 1;
    // Where the line moves: the stretch with its ramps, on a closed line also beyond its seam.
    let laps: &[f64] = if closed { &[-1.0, 0.0, 1.0] } else { &[0.0] };
    let mut parts: Vec<(f64, f64)> = laps
        .iter()
        .map(|&lap| ((lo + lap * total).max(0.0), (hi + lap * total).min(total)))
        .filter(|&(from, to)| from <= to)
        .collect();
    parts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut moving: Vec<(f64, f64)> = Vec::new();
    for part in parts {
        match moving.last_mut() {
            Some(last) if part.0 <= last.1 => last.1 = last.1.max(part.1),
            _ => moving.push(part),
        }
    }
    let mut stations: Vec<(f64, Option<usize>)> = moving
        .iter()
        .flat_map(|&(from, to)| samples(from, to, step).map(|s| (s, None)))
        .collect();
    stations.extend(line.at.iter().enumerate().map(|(i, &s)| (s, Some(i))));
    stations.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.is_some().cmp(&a.1.is_some())));
    // A sample on a vertex, or on the end of another sample's part, is one station; a vertex wins.
    stations.dedup_by(|later, kept| {
        let same = later.0 - kept.0 <= 1e-9;
        if same && later.1.is_some() && kept.1.is_none() {
            *kept = *later;
        }
        same && (later.1.is_none() || kept.1 == later.1)
    });
    let mut moved: Vec<(f64, f64)> = Vec::with_capacity(stations.len());
    for (s, vertex) in stations {
        let by = side * offset(s);
        let (point, normal) = match vertex {
            Some(i) => {
                let normal = match (i, closed) {
                    (0, false) => normals[0],
                    (i, false) if i > last => normals[last],
                    (0, true) => bisector(normals[last], normals[0]),
                    (i, true) if i > last => bisector(normals[last], normals[0]),
                    (i, _) => bisector(normals[i - 1], normals[i]),
                };
                (line.points[i], normal)
            }
            None => {
                let i = line.at.partition_point(|&a| a < s).clamp(1, last + 1);
                (line.point(s), normals[i - 1])
            }
        };
        if by == 0.0 {
            moved.push((point.x, point.y));
        } else {
            moved.push((point.x + by * normal.x, point.y + by * normal.y));
        }
    }
    moved.dedup();
    if closed && let Some(&first) = moved.first() {
        let end = moved.len() - 1;
        moved[end] = first;
    }
    moved
}

/// `from` to `to` in equal steps of at most `step`, both ends included.
fn samples(from: f64, to: f64, step: f64) -> impl Iterator<Item = f64> {
    let n = if to > from && step > 0.0 {
        ((to - from) / step).ceil().max(1.0) as usize
    } else {
        0
    };
    (0..=n).map(move |i| {
        if i == n {
            to
        } else {
            from + (to - from) * i as f64 / n as f64
        }
    })
}

/// The point of `line` nearest `c` (the first of equally near ones).
fn closest(line: &Polyline, c: Coord<f64>) -> Coord<f64> {
    let mut best = (f64::INFINITY, line.points[0]);
    for w in line.points.windows(2) {
        let (dx, dy) = (w[1].x - w[0].x, w[1].y - w[0].y);
        let length2 = dx * dx + dy * dy;
        let t = if length2 > 0.0 {
            (((c.x - w[0].x) * dx + (c.y - w[0].y) * dy) / length2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let q = Coord {
            x: w[0].x + t * dx,
            y: w[0].y + t * dy,
        };
        let d = distance2(q, c);
        if d < best.0 {
            best = (d, q);
        }
    }
    best.1
}

fn distance2(a: Coord<f64>, b: Coord<f64>) -> f64 {
    (a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)
}

/// The unit normal to the right of the direction from `a` to `b` (y down); zero for equal points.
fn right_normal(a: Coord<f64>, b: Coord<f64>) -> Coord<f64> {
    unit(Coord {
        x: -(b.y - a.y),
        y: b.x - a.x,
    })
}

/// The normalized sum of two unit normals; the first one where they cancel out.
fn bisector(n1: Coord<f64>, n2: Coord<f64>) -> Coord<f64> {
    let sum = unit(Coord {
        x: n1.x + n2.x,
        y: n1.y + n2.y,
    });
    if sum.x == 0.0 && sum.y == 0.0 {
        n1
    } else {
        sum
    }
}

fn unit(v: Coord<f64>) -> Coord<f64> {
    let length = (v.x * v.x + v.y * v.y).sqrt();
    if length > 0.0 {
        Coord {
            x: v.x / length,
            y: v.y / length,
        }
    } else {
        Coord { x: 0.0, y: 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn polyline(points: &[(f64, f64)]) -> Polyline {
        Polyline::new(points.iter().map(|&(x, y)| Coord { x, y }).collect())
    }

    /// The distance from `p` to the nearest point of `line`.
    fn distance(p: (f64, f64), line: &[(f64, f64)]) -> f64 {
        line.windows(2)
            .map(|w| crate::build::geometry::distance_to_segment(p, w[0], w[1]))
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn simplify_keeps_what_geo_keeps() {
        use geo::Simplify as _;
        // A wave with a kink, sampled every 0.25 mm, and a straight run with one bump.
        let wave: Vec<Coord<f64>> = (0..200)
            .map(|i| {
                let x = f64::from(i) * 0.25;
                let y = (x / 3.0).sin() * 0.4 + if x > 20.0 { (x - 20.0) * 0.1 } else { 0.0 };
                Coord { x, y }
            })
            .collect();
        let bump: Vec<Coord<f64>> = (0..40)
            .map(|i| Coord {
                x: f64::from(i) * 0.25,
                y: if i == 17 { 0.2 } else { 0.0 },
            })
            .collect();
        for (points, tolerance) in [(&wave, 0.0625), (&wave, 0.01), (&bump, 0.0625)] {
            let ours = simplify(points, tolerance);
            let theirs = geo_types::LineString::new(points.clone())
                .simplify(tolerance)
                .0;
            assert_eq!(ours, theirs, "tolerance {tolerance}");
        }
        assert_eq!(simplify(&bump, 0.0625).len(), 5);
        assert_eq!(simplify(&bump, 0.25).len(), 2);
    }

    #[test]
    fn beside_finds_the_parallel_stretch() {
        // Two 20 mm lines 0.3 mm apart that overlap over 12 mm (x 8 to 20).
        let a = polyline(&[(0.0, 0.0), (20.0, 0.0)]);
        let b = polyline(&[(8.0, 0.3), (28.0, 0.3)]);
        let ranges = beside(&a, &b, 0.4, 1.0);
        assert_eq!(ranges.len(), 1, "{ranges:?}");
        let (from, to) = ranges[0];
        assert!((to - from - 12.0).abs() < 0.5, "{ranges:?}");
        assert!((7.5..=8.0).contains(&from), "{ranges:?}");
        assert!((to - 20.0).abs() < 1e-9, "{ranges:?}");
        // From b's side the stretch is its first 12 mm.
        let ranges = beside(&b, &a, 0.4, 1.0);
        assert_eq!(ranges.len(), 1, "{ranges:?}");
        assert!(
            ranges[0].0.abs() < 1e-9 && (ranges[0].1 - 12.0).abs() < 0.5,
            "{ranges:?}"
        );
    }

    #[test]
    fn beside_merges_small_gaps_and_drops_short_stretches() {
        // b runs beside a, leaves it for 0.4 mm and comes back: one stretch.
        let a = polyline(&[(0.0, 0.0), (20.0, 0.0)]);
        let b = polyline(&[
            (0.0, 0.3),
            (9.5, 0.3),
            (10.0, 2.0),
            (10.5, 0.3),
            (20.0, 0.3),
        ]);
        let ranges = beside(&a, &b, 0.4, 2.0);
        assert_eq!(ranges.len(), 1, "{ranges:?}");
        assert!(ranges[0].0.abs() < 1e-9 && (ranges[0].1 - 20.0).abs() < 1e-9);
        // A stretch shorter than `min_len` is no stretch.
        let short = polyline(&[(5.0, 0.3), (5.5, 0.3)]);
        assert!(beside(&a, &short, 0.4, 2.0).is_empty());
        // Lines apart are not beside each other.
        let far = polyline(&[(0.0, 1.0), (20.0, 1.0)]);
        assert!(beside(&a, &far, 0.4, 1.0).is_empty());
    }

    #[test]
    fn middle_of_two_parallels_lies_between() {
        let a = polyline(&[(0.0, 0.0), (20.0, 0.0)]);
        // b against the direction of a, with a vertex in between.
        let b_points = [(20.0, 0.3), (11.0, 0.3), (0.0, 0.3)];
        let b = polyline(&b_points);
        let middle = middle(&a, &b, (2.0, 18.0), 0.05);
        assert!(middle.len() >= 2, "{middle:?}");
        assert!((middle[0].0 - 2.0).abs() < 1e-9, "{middle:?}");
        assert!(
            (middle[middle.len() - 1].0 - 18.0).abs() < 1e-9,
            "{middle:?}"
        );
        for &p in &middle {
            assert!(
                (distance(p, &[(0.0, 0.0), (20.0, 0.0)]) - 0.15).abs() < 1e-9,
                "{p:?}"
            );
            assert!((distance(p, &b_points) - 0.15).abs() < 1e-9, "{p:?}");
        }
        // A straight middle needs no points in between.
        assert_eq!(middle.len(), 2, "{middle:?}");
    }

    #[test]
    fn shifted_moves_only_the_stretch() {
        let line = polyline(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)]);
        let moved = shifted(&line, (8.0, 12.0), 0.3, 1.0, 2.0, 0.1);
        assert_eq!(moved[0], (0.0, 0.0));
        assert_eq!(moved[moved.len() - 1], (20.0, 0.0));
        for &(x, y) in &moved {
            if x <= 6.0 || x >= 14.0 {
                assert_eq!(y, 0.0, "({x}, {y}) lies outside the ramps");
            } else if (8.0..=12.0).contains(&x) {
                // Right of the direction +x is +y, y down.
                assert!((y - 0.3).abs() < 0.002, "({x}, {y}) lies on the plateau");
            } else {
                assert!((0.0..=0.3).contains(&y), "({x}, {y}) lies on a ramp");
            }
        }
        assert!(
            moved.iter().any(|&(x, _)| (x - 6.0).abs() < 1e-9),
            "{moved:?}"
        );
        assert!(
            moved.iter().any(|&(x, _)| (x - 14.0).abs() < 1e-9),
            "{moved:?}"
        );
        assert!(
            moved
                .iter()
                .filter(|&&(x, _)| (8.0..=12.0).contains(&x))
                .count()
                >= 40
        );
        // The other side.
        let left = shifted(&line, (8.0, 12.0), 0.3, -1.0, 2.0, 0.1);
        let at_ten = left
            .iter()
            .find(|&&(x, _)| (x - 10.0).abs() < 1e-9)
            .unwrap();
        assert!((at_ten.1 + 0.3).abs() < 0.002, "{at_ten:?}");
        // A bend inside the plateau keeps the shift on both legs and around the corner (moved to
        // its outer side, where every point stays 0.3 mm off the line).
        let bend_points: [(f64, f64); 3] = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let moved = shifted(&polyline(&bend_points), (8.0, 12.0), 0.3, -1.0, 2.0, 0.1);
        for &p in &moved {
            assert!(distance(p, &bend_points) <= 0.3 + 0.002, "{p:?}");
        }
        let plateau: Vec<_> = moved
            .iter()
            .filter(|&&(x, y)| (8.0..=10.4).contains(&x) && (-0.4..=2.0).contains(&y))
            .collect();
        assert!(plateau.len() >= 40, "{moved:?}");
        for &&p in &plateau {
            assert!((distance(p, &bend_points) - 0.3).abs() < 0.002, "{p:?}");
        }
    }

    #[test]
    fn shifted_keeps_a_closed_line_closed() {
        let square = [
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ];
        let line = polyline(&square);
        // A stretch on the seam, and one whose ramp runs over it.
        for range in [(39.0, 41.0), (0.5, 4.0), (36.0, 39.5), (0.0, 40.0)] {
            let moved = shifted(&line, range, 0.3, 1.0, 2.0, 0.1);
            assert!(moved.len() > square.len(), "{range:?}: {moved:?}");
            assert_eq!(moved[0], moved[moved.len() - 1], "{range:?}");
        }
        // The seam moves with a stretch over it: inwards, right of the clockwise ring (y down).
        let moved = shifted(&line, (39.0, 41.0), 0.3, 1.0, 2.0, 0.1);
        assert_ne!(moved[0], (0.0, 0.0));
        assert!(moved[0].0 > 0.0 && moved[0].1 > 0.0, "{:?}", moved[0]);
    }

    #[test]
    fn the_same_input_gives_the_same_points() {
        let a = polyline(&[(0.0, 0.0), (7.3, 1.1), (13.9, -0.7), (20.0, 0.4)]);
        let b = polyline(&[(0.4, 0.35), (6.9, 1.4), (14.2, -0.4), (21.0, 0.7)]);
        let bits = |points: &[(f64, f64)]| -> Vec<(u64, u64)> {
            points
                .iter()
                .map(|p| (p.0.to_bits(), p.1.to_bits()))
                .collect()
        };
        let range_bits = |ranges: &[(f64, f64)]| bits(ranges);
        let first = beside(&a, &b, 0.5, 1.0);
        assert!(!first.is_empty());
        assert_eq!(range_bits(&first), range_bits(&beside(&a, &b, 0.5, 1.0)));
        let range = first[0];
        assert_eq!(
            bits(&middle(&a, &b, range, 0.05)),
            bits(&middle(&a, &b, range, 0.05))
        );
        assert_eq!(
            bits(&shifted(&a, range, 0.25, -1.0, 1.0, 0.05)),
            bits(&shifted(&a, range, 0.25, -1.0, 1.0, 0.05))
        );
    }
}
