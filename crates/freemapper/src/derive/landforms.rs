//! Landform candidates from the DTM: small knolls, small depressions, earth banks and cliffs.
//!
//! Contours at 2 m (sprint) or 1 m (schoolyard) miss the small forms of a park: a play mound, a
//! hollow, the bank of a terrace. Ideas from karttapullautin (knolls as small closed rings around
//! a top, cliffs as height jumps) and omapmaker (drop measured across the steep part, the low
//! side decides the ticks); the code is our own. Everything here is a candidate (spec §4.5).

use geo::{Area as _, Contains as _, Simplify as _};

use super::contours::{contour_lines, gaussian_smooth};
use super::terrain::gradient;
use crate::raster::{GridSpec, Raster};

/// Smoothing before knolls and depressions are looked for: DGM1 noise makes false tops.
const KNOLL_SIGMA_M: f64 = 1.0;
/// Step of the fine contours that outline a knoll.
const RING_STEP_M: f64 = 0.25;
/// Widest knoll or depression, on paper and at least on the ground: wider ones are shown by
/// the contours.
const MAX_DIAMETER_MM: f64 = 5.0;
const MAX_DIAMETER_M: f64 = 10.0;
/// Narrowest knoll or depression on the ground.
const MIN_DIAMETER_M: f64 = 2.0;
/// A knoll must rise at least this much per metre from its outline to its top: the top of a
/// wide hill is no knoll.
const MIN_STEEPNESS: f64 = 0.15;
/// Only this much below a top is searched for its outline.
const MAX_FORM_HEIGHT_M: f32 = 6.0;
/// A knoll whose outline is this much longer than wide is elongated (ISSprOM 110).
const ELONGATED_ASPECT: f64 = 1.5;
/// Gradient from which a cell is steep (about 27°): DGM1 at 1 m blurs a bank over 2 to 3 cells,
/// so a 45° bank of 1.5 m comes out at about 30°.
const STEEP_GRADIENT: f64 = 0.5;
/// The drop of a steep cell is measured this far up and down the slope.
const DROP_REACH_M: f64 = 3.0;
/// A bank holds most of the drop of twice the reach; a plain slope only half.
const MIN_DROP_SHARE: f64 = 0.7;
/// Shortest bank on paper, and on the ground.
const MIN_BANK_MM: f64 = 2.0;
const MIN_BANK_M: f64 = 4.0;
/// A steep patch at least this much longer than wide is a bank; a rounder one is a hillside.
const MIN_BANK_ELONGATION: f64 = 2.0;
/// Banks are simplified to this, on the ground.
const BANK_SIMPLIFY_M: f64 = 0.5;
/// The centre line is averaged over this many 1-cell bins on each side before it is simplified:
/// the cells of a wide or uneven bank make it jump sideways from bin to bin.
const BANK_SMOOTH_BINS: usize = 2;
/// From this drop on a height jump is an impassable cliff (201), below an earth bank (104).
const CLIFF_DROP_M: f64 = 3.0;
/// A bank's cells lie at most this far from its line; a patch with cells farther off bends and is
/// split (ground metres).
const MAX_OFF_LINE_M: f64 = 2.0;
/// From this share of cells rising to each side of the axis, a patch is two banks back to back.
const TWO_SIDED_SHARE: f64 = 0.3;
/// How often a patch is split at most.
const MAX_SPLIT_DEPTH: u32 = 6;

/// What a landform candidate is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandformKind {
    Knoll,
    ElongatedKnoll,
    Depression,
    Bank,
    Cliff,
}

impl LandformKind {
    /// The name in `data/landforms.geojson`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Knoll => "knoll",
            Self::ElongatedKnoll => "elongated-knoll",
            Self::Depression => "depression",
            Self::Bank => "bank",
            Self::Cliff => "cliff",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        [
            Self::Knoll,
            Self::ElongatedKnoll,
            Self::Depression,
            Self::Bank,
            Self::Cliff,
        ]
        .into_iter()
        .find(|k| k.name() == name)
    }

    /// A point symbol (knolls, depressions) rather than a line.
    pub fn is_point(self) -> bool {
        matches!(self, Self::Knoll | Self::ElongatedKnoll | Self::Depression)
    }
}

/// One landform candidate in the map's CRS. A point kind has one point, a line kind runs with
/// its lower side on the right.
#[derive(Debug, Clone, PartialEq)]
pub struct Landform {
    pub kind: LandformKind,
    pub points: Vec<(f64, f64)>,
    /// Direction of an elongated knoll, degrees counter-clockwise from east; else 0.
    pub rotation_deg: f64,
    /// Height of a knoll, depth of a depression, drop of a bank or cliff.
    pub height_m: f64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid landforms: {0}")]
pub struct LandformError(String);

/// The knolls and depressions at least `min_height_m` high or deep and narrow enough that the
/// contours of the map do not show them.
pub fn knolls_and_depressions(
    dtm: &Raster<f32>,
    scale_denominator: u32,
    min_height_m: f64,
) -> Vec<Landform> {
    let smoothed = gaussian_smooth(dtm, KNOLL_SIGMA_M);
    let max_diameter_m =
        (MAX_DIAMETER_MM * f64::from(scale_denominator) / 1000.0).max(MAX_DIAMETER_M);
    let limits = FormLimits {
        min_height_m,
        min_area_m2: std::f64::consts::PI * (MIN_DIAMETER_M / 2.0).powi(2),
        max_area_m2: std::f64::consts::PI * (max_diameter_m / 2.0).powi(2),
        half_window: (max_diameter_m / 2.0 / dtm.grid.pixel_size).ceil() as i64 + 2,
    };
    let mut forms = tops(&smoothed, &limits, LandformKind::Knoll);
    let lowered = Raster {
        grid: smoothed.grid,
        data: smoothed.data.iter().map(|z| -z).collect(),
    };
    forms.extend(tops(&lowered, &limits, LandformKind::Depression));
    forms
}

struct FormLimits {
    min_height_m: f64,
    min_area_m2: f64,
    max_area_m2: f64,
    half_window: i64,
}

/// The tops of `dtm` that stand out by at least the minimum height inside a ring small enough;
/// `kind` is `Knoll` on the terrain, `Depression` on the negated terrain.
fn tops(dtm: &Raster<f32>, limits: &FormLimits, kind: LandformKind) -> Vec<Landform> {
    let (w, h) = (i64::from(dtm.grid.width), i64::from(dtm.grid.height));
    let z = |x: i64, y: i64| -> f32 {
        if x < 0 || y < 0 || x >= w || y >= h {
            f32::NAN
        } else {
            dtm.data[(y * w + x) as usize]
        }
    };
    let mut found: Vec<(Landform, geo_types::Polygon<f64>)> = Vec::new();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let top = z(x, y);
            if top.is_nan() {
                continue;
            }
            // A top is no lower than its neighbours; of equal ones the first in raster order.
            let highest = (-1..=1)
                .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
                .filter(|&d| d != (0, 0))
                .all(|(dx, dy)| {
                    let n = z(x + dx, y + dy);
                    !n.is_nan() && (n < top || (n == top && (dy, dx) > (0, 0)))
                });
            if !highest {
                continue;
            }
            let r = limits.half_window;
            let window_min = (y - r..=y + r)
                .flat_map(|yy| (x - r..=x + r).map(move |xx| (xx, yy)))
                .map(|(xx, yy)| z(xx, yy))
                .filter(|v| !v.is_nan())
                .fold(f32::MAX, f32::min);
            if f64::from(top - window_min) < limits.min_height_m {
                continue;
            }
            if let Some(form) = outline(dtm, (x, y), limits, kind) {
                found.push(form);
            }
        }
    }
    // Two tops inside one ring are one form: the higher one stays.
    found.sort_by(|a, b| b.0.height_m.total_cmp(&a.0.height_m));
    let mut kept: Vec<(Landform, geo_types::Polygon<f64>)> = Vec::new();
    for (form, ring) in found {
        let p = geo_types::Point::new(form.points[0].0, form.points[0].1);
        if kept.iter().all(|(_, r)| !r.contains(&p)) {
            kept.push((form, ring));
        }
    }
    kept.into_iter().map(|(form, _)| form).collect()
}

/// The lowest closed fine contour around the top at `(x, y)` that is small enough, as a form.
fn outline(
    dtm: &Raster<f32>,
    (x, y): (i64, i64),
    limits: &FormLimits,
    kind: LandformKind,
) -> Option<(Landform, geo_types::Polygon<f64>)> {
    let grid = dtm.grid;
    let (w, h) = (i64::from(grid.width), i64::from(grid.height));
    let r = limits.half_window;
    let (x0, y0) = ((x - r).max(0), (y - r).max(0));
    let (x1, y1) = ((x + r).min(w - 1), (y + r).min(h - 1));
    let top = dtm.get(x as u32, y as u32);
    let floor = top - MAX_FORM_HEIGHT_M;
    let window_grid = GridSpec {
        west: grid.west + x0 as f64 * grid.pixel_size,
        north: grid.north - y0 as f64 * grid.pixel_size,
        pixel_size: grid.pixel_size,
        width: (x1 - x0 + 1) as u32,
        height: (y1 - y0 + 1) as u32,
    };
    let mut window = Raster::filled(window_grid, f32::NAN);
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            let v = dtm.get(xx as u32, yy as u32);
            window.set(
                (xx - x0) as u32,
                (yy - y0) as u32,
                if v.is_nan() { v } else { v.max(floor) },
            );
        }
    }
    let (e, n) = cell_centre(&grid, x, y);
    let centre = geo_types::Point::new(e, n);
    let best = contour_lines(&window, RING_STEP_M)
        .into_iter()
        .filter(|line| line.points.len() > 3 && line.points.first() == line.points.last())
        .filter(|line| line.height > f64::from(floor) && line.height < f64::from(top))
        .filter_map(|line| {
            let ring = geo_types::Polygon::new(line.points.clone().into(), vec![]);
            let area = ring.unsigned_area();
            let height_m = f64::from(top) - line.height;
            let radius_m = (area / std::f64::consts::PI).sqrt();
            let fits = (limits.min_area_m2..=limits.max_area_m2).contains(&area)
                && height_m >= limits.min_height_m
                && height_m >= MIN_STEEPNESS * radius_m
                && ring.contains(&centre);
            fits.then_some((line, ring, height_m))
        })
        .min_by(|a, b| a.0.height.total_cmp(&b.0.height))?;
    let (line, ring, height_m) = best;
    let (aspect, rotation_deg) = principal_axes(&line.points);
    let kind = if kind == LandformKind::Knoll && aspect >= ELONGATED_ASPECT {
        LandformKind::ElongatedKnoll
    } else {
        kind
    };
    let rotation_deg = if kind == LandformKind::ElongatedKnoll {
        rotation_deg
    } else {
        0.0
    };
    Some((
        Landform {
            kind,
            points: vec![(e, n)],
            rotation_deg,
            height_m,
        },
        ring,
    ))
}

/// The ratio of the long to the short principal axis of `points`, and the direction of the long
/// one in degrees counter-clockwise from east, in `[0, 180)`.
fn principal_axes(points: &[(f64, f64)]) -> (f64, f64) {
    let count = points.len() as f64;
    let (me, mn) = points
        .iter()
        .fold((0.0, 0.0), |(a, b), (e, n)| (a + e / count, b + n / count));
    let (mut see, mut snn, mut sen) = (0.0, 0.0, 0.0);
    for (e, n) in points {
        let (de, dn) = (e - me, n - mn);
        see += de * de;
        snn += dn * dn;
        sen += de * dn;
    }
    let half_trace = (see + snn) / 2.0;
    let root = (((see - snn) / 2.0).powi(2) + sen * sen).sqrt();
    let (major, minor) = (half_trace + root, (half_trace - root).max(1e-12));
    let angle = 0.5 * (2.0 * sen).atan2(see - snn);
    ((major / minor).sqrt(), angle.to_degrees().rem_euclid(180.0))
}

fn cell_centre(grid: &GridSpec, x: i64, y: i64) -> (f64, f64) {
    (
        grid.west + (x as f64 + 0.5) * grid.pixel_size,
        grid.north - (y as f64 + 0.5) * grid.pixel_size,
    )
}

/// Earth banks and cliffs: steep, long and narrow patches of the DTM whose drop is at least
/// `min_height_m` and lies mostly within the patch.
pub fn height_jumps(dtm: &Raster<f32>, scale_denominator: u32, min_height_m: f64) -> Vec<Landform> {
    let grid = dtm.grid;
    let (w, h) = (grid.width as usize, grid.height as usize);
    let px = grid.pixel_size;
    let gradients: Vec<Option<(f64, f64)>> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
                return None;
            }
            gradient(dtm, x as u32, y as u32)
        })
        .collect();
    let steep: Vec<bool> = gradients
        .iter()
        .map(|g| g.is_some_and(|(de, dn)| de.hypot(dn) >= STEEP_GRADIENT))
        .collect();
    let min_length_m = (MIN_BANK_MM * f64::from(scale_denominator) / 1000.0).max(MIN_BANK_M);
    // The height at a point, `None` off the raster or on a cell without data.
    let height_at = |(e, n): (f64, f64)| -> Option<f64> {
        let x = ((e - grid.west) / px).floor();
        let y = ((grid.north - n) / px).floor();
        if x < 0.0 || y < 0.0 || x >= w as f64 || y >= h as f64 {
            return None;
        }
        let v = dtm.get(x as u32, y as u32);
        (!v.is_nan()).then_some(f64::from(v))
    };
    let mut seen = vec![false; w * h];
    let mut jumps = Vec::new();
    for start in 0..w * h {
        if !steep[start] || seen[start] {
            continue;
        }
        let cells = component(start, w, h, &steep, &mut seen);
        if cells.len() < 3 {
            continue;
        }
        let mut drops = Vec::new();
        let mut shares = Vec::new();
        let mut uphill = Vec::with_capacity(cells.len());
        for &c in &cells {
            let (dz_de, dz_dn) = gradients[c].expect("steep cells have a gradient");
            let length = dz_de.hypot(dz_dn);
            let u = (dz_de / length, dz_dn / length);
            uphill.push((c, u));
            let p = cell_centre(&grid, (c % w) as i64, (c / w) as i64);
            let at = |d: f64| height_at((p.0 + u.0 * d, p.1 + u.1 * d));
            let (Some(high), Some(low), Some(far_high), Some(far_low)) = (
                at(DROP_REACH_M),
                at(-DROP_REACH_M),
                at(2.0 * DROP_REACH_M),
                at(-2.0 * DROP_REACH_M),
            ) else {
                continue;
            };
            let drop = high - low;
            drops.push(drop);
            let far = far_high - far_low;
            shares.push(if far > 0.0 { drop / far } else { 0.0 });
        }
        // Half the cells at least must say how high the jump is.
        if drops.len() * 2 < cells.len() {
            continue;
        }
        let drop = median(&mut drops);
        if drop < min_height_m || median(&mut shares) < MIN_DROP_SHARE {
            continue;
        }
        let mut pieces = Vec::new();
        bank_pieces(&uphill, &grid, min_length_m, 0, &mut pieces);
        for (points, uphill) in pieces {
            // The ticks sit on the right of the line and point down: the low side on the right.
            let (first, last) = (points[0], points[points.len() - 1]);
            let direction = (last.0 - first.0, last.1 - first.1);
            let right = (direction.1, -direction.0);
            let points = if right.0 * uphill.0 + right.1 * uphill.1 > 0.0 {
                points.into_iter().rev().collect()
            } else {
                points
            };
            jumps.push(Landform {
                kind: if drop >= CLIFF_DROP_M {
                    LandformKind::Cliff
                } else {
                    LandformKind::Bank
                },
                points,
                rotation_deg: 0.0,
                height_m: drop,
            });
        }
    }
    jumps
}

/// The 8-connected steep cells around `start`, marking them seen.
fn component(start: usize, w: usize, h: usize, steep: &[bool], seen: &mut [bool]) -> Vec<usize> {
    let mut cells = Vec::new();
    let mut stack = vec![start];
    seen[start] = true;
    while let Some(c) = stack.pop() {
        cells.push(c);
        let (x, y) = ((c % w) as i64, (c / w) as i64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (xx, yy) = (x + dx, y + dy);
                if xx < 0 || yy < 0 || xx >= w as i64 || yy >= h as i64 {
                    continue;
                }
                let n = yy as usize * w + xx as usize;
                if steep[n] && !seen[n] {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
    }
    cells
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// A bank's centre line and the uphill vectors of its cells, summed.
type BankPiece = (Vec<(f64, f64)>, (f64, f64));

/// The pieces of a steep patch (`cells` with the unit uphill vector of each) that each make one
/// bank with one low side, as `(centre line, summed uphill vector)`. A patch that falls to both
/// sides of its axis (a dyke, a ditch) is split by side; one whose cells lie farther than
/// [`MAX_OFF_LINE_M`] from its centre line (a corner, a curve) is split across its axis at the
/// cell farthest off. Pieces too short for a bank are left out.
fn bank_pieces(
    cells: &[(usize, (f64, f64))],
    grid: &GridSpec,
    min_length_m: f64,
    depth: u32,
    out: &mut Vec<BankPiece>,
) {
    use geo::{Distance as _, Euclidean};
    if cells.len() < 3 {
        return;
    }
    let w = grid.width as usize;
    let centre = |c: usize| cell_centre(grid, (c % w) as i64, (c / w) as i64);
    let points: Vec<(f64, f64)> = cells.iter().map(|&(c, _)| centre(c)).collect();
    let (_, axis_deg) = principal_axes(&points);
    let axis = (axis_deg.to_radians().cos(), axis_deg.to_radians().sin());
    let normal = (-axis.1, axis.0);
    let deeper = depth < MAX_SPLIT_DEPTH;
    let split = |keep: &dyn Fn(usize) -> bool, out: &mut Vec<_>| {
        let (a, b): (Vec<_>, Vec<_>) = (0..cells.len()).partition(|&i| keep(i));
        if a.is_empty() || b.is_empty() {
            return false;
        }
        for part in [a, b] {
            let part: Vec<_> = part.into_iter().map(|i| cells[i]).collect();
            bank_pieces(&part, grid, min_length_m, depth + 1, out);
        }
        true
    };
    let side = |i: usize| cells[i].1.0 * normal.0 + cells[i].1.1 * normal.1;
    let rising = (0..cells.len()).filter(|&i| side(i) > 0.0).count();
    let falling = cells.len() - rising;
    if deeper
        && rising.min(falling) as f64 >= TWO_SIDED_SHARE * cells.len() as f64
        && split(&|i| side(i) > 0.0, out)
    {
        return;
    }
    let ids: Vec<usize> = cells.iter().map(|&(c, _)| c).collect();
    let Some(line) = centre_line(&ids, grid, min_length_m) else {
        return;
    };
    let path = geo_types::LineString::from(line.clone());
    let (worst, off) = points
        .iter()
        .enumerate()
        .map(|(i, p)| (i, Euclidean.distance(&geo_types::Point::from(*p), &path)))
        .fold(
            (0, 0.0),
            |best, item| if item.1 > best.1 { item } else { best },
        );
    if deeper && off > MAX_OFF_LINE_M {
        let along = |p: &(f64, f64)| p.0 * axis.0 + p.1 * axis.1;
        let cut = along(&points[worst]);
        if split(&|i| along(&points[i]) < cut, out) {
            return;
        }
    }
    if off > MAX_OFF_LINE_M {
        return;
    }
    let uphill = cells
        .iter()
        .fold((0.0, 0.0), |s, &(_, u)| (s.0 + u.0, s.1 + u.1));
    out.push((line, uphill));
}

/// The line along the long axis of the patch: the mean of its cells per pixel along the axis,
/// smoothed and simplified; `None` when the patch is too short or not long and narrow enough.
fn centre_line(cells: &[usize], grid: &GridSpec, min_length_m: f64) -> Option<Vec<(f64, f64)>> {
    let w = grid.width as usize;
    let points: Vec<(f64, f64)> = cells
        .iter()
        .map(|&c| cell_centre(grid, (c % w) as i64, (c / w) as i64))
        .collect();
    let (_, axis_deg) = principal_axes(&points);
    let axis = (axis_deg.to_radians().cos(), axis_deg.to_radians().sin());
    let along = |p: &(f64, f64)| p.0 * axis.0 + p.1 * axis.1;
    let (lo, hi) = points
        .iter()
        .map(along)
        .fold((f64::MAX, f64::MIN), |(a, b), t| (a.min(t), b.max(t)));
    let px = grid.pixel_size;
    let length = hi - lo + px;
    let width = cells.len() as f64 * px * px / length;
    if length < min_length_m || length < MIN_BANK_ELONGATION * width {
        return None;
    }
    let bins = ((hi - lo) / px).floor() as usize + 1;
    let mut sums = vec![(0.0, 0.0, 0usize); bins];
    for p in &points {
        let bin = (((along(p) - lo) / px).floor() as usize).min(bins - 1);
        sums[bin] = (sums[bin].0 + p.0, sums[bin].1 + p.1, sums[bin].2 + 1);
    }
    let raw: Vec<(f64, f64)> = sums
        .into_iter()
        .filter(|s| s.2 > 0)
        .map(|(e, n, k)| (e / k as f64, n / k as f64))
        .collect();
    // A moving mean; the ends stay where they are, so the line keeps its length.
    let last = raw.len() - 1;
    let smoothed: Vec<(f64, f64)> = (0..raw.len())
        .map(|i| {
            if i == 0 || i == last {
                return raw[i];
            }
            let window =
                &raw[i.saturating_sub(BANK_SMOOTH_BINS)..(i + BANK_SMOOTH_BINS + 1).min(raw.len())];
            let k = window.len() as f64;
            (
                window.iter().map(|p| p.0).sum::<f64>() / k,
                window.iter().map(|p| p.1).sum::<f64>() / k,
            )
        })
        .collect();
    let line = geo_types::LineString::<f64>::from(smoothed).simplify(BANK_SIMPLIFY_M);
    (line.0.len() >= 2).then(|| line.0.iter().map(|c| (c.x, c.y)).collect())
}

/// The landforms as GeoJSON in the map's CRS, with `kind`, `height` and `rotation` properties.
pub fn landforms_to_geojson(forms: &[Landform]) -> String {
    let features: Vec<serde_json::Value> = forms
        .iter()
        .map(|f| {
            let geometry = if f.kind.is_point() {
                serde_json::json!({ "type": "Point", "coordinates": [f.points[0].0, f.points[0].1] })
            } else {
                serde_json::json!({
                    "type": "LineString",
                    "coordinates": f.points.iter().map(|(e, n)| [*e, *n]).collect::<Vec<_>>()
                })
            };
            serde_json::json!({
                "type": "Feature",
                "properties": {
                    "kind": f.kind.name(),
                    "height": f.height_m,
                    "rotation": f.rotation_deg,
                },
                "geometry": geometry
            })
        })
        .collect();
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// Reads landforms written by [`landforms_to_geojson`].
pub fn landforms_from_geojson(text: &str) -> Result<Vec<Landform>, LandformError> {
    let collection: serde_json::Value =
        serde_json::from_str(text).map_err(|e| LandformError(e.to_string()))?;
    let features = collection["features"]
        .as_array()
        .ok_or_else(|| LandformError("no features".to_string()))?;
    features
        .iter()
        .map(|f| {
            let properties = &f["properties"];
            let kind = properties["kind"]
                .as_str()
                .and_then(LandformKind::from_name)
                .ok_or_else(|| LandformError("feature without a known kind".to_string()))?;
            let number = |key: &str| {
                properties[key]
                    .as_f64()
                    .ok_or_else(|| LandformError(format!("feature without {key}")))
            };
            let coordinates = &f["geometry"]["coordinates"];
            let pair = |c: &serde_json::Value| Some((c.get(0)?.as_f64()?, c.get(1)?.as_f64()?));
            let points = if kind.is_point() {
                pair(coordinates).map(|p| vec![p])
            } else {
                coordinates
                    .as_array()
                    .and_then(|cs| cs.iter().map(pair).collect::<Option<Vec<_>>>())
                    .filter(|ps| ps.len() >= 2)
            }
            .ok_or_else(|| LandformError("invalid coordinates".to_string()))?;
            Ok(Landform {
                kind,
                points,
                rotation_deg: number("rotation")?,
                height_m: number("height")?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPRINT: u32 = 4000;

    /// `size` × `size` DTM with 1 m pixels, west 0, north `size`; `z(e, n)` at pixel centres.
    fn dtm(size: u32, z: impl Fn(f64, f64) -> f64) -> Raster<f32> {
        let grid = GridSpec {
            west: 0.0,
            north: f64::from(size),
            pixel_size: 1.0,
            width: size,
            height: size,
        };
        let mut r = Raster::filled(grid, 0.0);
        for y in 0..size {
            for x in 0..size {
                let (e, n) = cell_centre(&grid, i64::from(x), i64::from(y));
                r.set(x, y, z(e, n) as f32);
            }
        }
        r
    }

    /// A Gaussian bump of `height` with `sigma_e`, `sigma_n` metres at `(ce, cn)`.
    fn bump(ce: f64, cn: f64, height: f64, sigma_e: f64, sigma_n: f64) -> impl Fn(f64, f64) -> f64 {
        move |e, n| {
            let q = ((e - ce) / sigma_e).powi(2) + ((n - cn) / sigma_n).powi(2);
            height * (-q / 2.0).exp()
        }
    }

    fn near(p: (f64, f64), q: (f64, f64), tolerance: f64) -> bool {
        (p.0 - q.0).hypot(p.1 - q.1) <= tolerance
    }

    #[test]
    fn a_two_metre_bump_is_a_knoll() {
        let b = bump(30.0, 30.0, 2.0, 2.5, 2.5);
        let forms = knolls_and_depressions(&dtm(60, |e, n| 40.0 + b(e, n)), SPRINT, 1.0);
        assert_eq!(forms.len(), 1, "{forms:?}");
        assert_eq!(forms[0].kind, LandformKind::Knoll);
        assert!(near(forms[0].points[0], (30.0, 30.0), 1.0), "{forms:?}");
        assert!(
            forms[0].height_m >= 1.0 && forms[0].height_m <= 2.0,
            "{forms:?}"
        );
    }

    #[test]
    fn a_low_bump_is_no_knoll() {
        let b = bump(30.0, 30.0, 0.6, 2.5, 2.5);
        let forms = knolls_and_depressions(&dtm(60, |e, n| 40.0 + b(e, n)), SPRINT, 1.0);
        assert!(forms.is_empty(), "{forms:?}");
        // On a schoolyard map half a metre is enough.
        let b = bump(30.0, 30.0, 1.2, 2.0, 2.0);
        let forms = knolls_and_depressions(&dtm(60, |e, n| 40.0 + b(e, n)), 1000, 0.5);
        assert_eq!(forms.len(), 1, "{forms:?}");
    }

    #[test]
    fn a_wide_hill_is_no_knoll() {
        // 3 m high and some 40 m across: the 2 m contours show it.
        let b = bump(50.0, 50.0, 3.0, 10.0, 10.0);
        let forms = knolls_and_depressions(&dtm(100, |e, n| 40.0 + b(e, n)), SPRINT, 1.0);
        assert!(forms.is_empty(), "{forms:?}");
    }

    #[test]
    fn an_elongated_bump_has_its_rotation() {
        // Long along the north-south axis.
        let b = bump(30.0, 30.0, 2.0, 1.5, 4.0);
        let forms = knolls_and_depressions(&dtm(60, |e, n| 40.0 + b(e, n)), SPRINT, 1.0);
        assert_eq!(forms.len(), 1, "{forms:?}");
        assert_eq!(forms[0].kind, LandformKind::ElongatedKnoll);
        assert!((forms[0].rotation_deg - 90.0).abs() < 10.0, "{forms:?}");
    }

    #[test]
    fn a_pit_is_a_depression() {
        let b = bump(30.0, 30.0, 1.8, 2.5, 2.5);
        let forms = knolls_and_depressions(&dtm(60, |e, n| 40.0 - b(e, n)), SPRINT, 1.0);
        assert_eq!(forms.len(), 1, "{forms:?}");
        assert_eq!(forms[0].kind, LandformKind::Depression);
        assert!(near(forms[0].points[0], (30.0, 30.0), 1.0), "{forms:?}");
    }

    #[test]
    fn two_bumps_are_two_knolls() {
        let (a, b) = (
            bump(20.0, 30.0, 2.0, 2.0, 2.0),
            bump(42.0, 30.0, 1.8, 2.0, 2.0),
        );
        let forms = knolls_and_depressions(&dtm(60, |e, n| 40.0 + a(e, n) + b(e, n)), SPRINT, 1.0);
        assert_eq!(forms.len(), 2, "{forms:?}");
    }

    #[test]
    fn a_knoll_on_a_slope_is_found() {
        let b = bump(30.0, 30.0, 2.0, 2.5, 2.5);
        let forms = knolls_and_depressions(&dtm(60, |e, n| 40.0 + 0.05 * e + b(e, n)), SPRINT, 1.0);
        assert_eq!(forms.len(), 1, "{forms:?}");
    }

    #[test]
    fn noise_gives_nothing() {
        // A few centimetres of DGM1 noise on a gentle slope.
        let noise = |e: f64, n: f64| 0.05 * ((e * 12.9898 + n * 78.233).sin() * 43758.5453).fract();
        let terrain = dtm(80, |e, n| 40.0 + 0.03 * n + noise(e, n));
        assert!(knolls_and_depressions(&terrain, SPRINT, 1.0).is_empty());
        assert!(height_jumps(&terrain, SPRINT, 1.0).is_empty());
    }

    /// A step of `drop` metres down to the east at e = 30, spread over two metres.
    fn step(drop: f64) -> impl Fn(f64, f64) -> f64 {
        move |e, _| 40.0 - drop * ((e - 29.0) / 2.0).clamp(0.0, 1.0)
    }

    #[test]
    fn a_step_is_a_bank_with_the_low_side_right() {
        let jumps = height_jumps(&dtm(60, step(1.5)), SPRINT, 1.0);
        assert_eq!(jumps.len(), 1, "{jumps:?}");
        let bank = &jumps[0];
        assert_eq!(bank.kind, LandformKind::Bank);
        assert!((bank.height_m - 1.5).abs() < 0.2, "{bank:?}");
        assert!(
            bank.points.iter().all(|p| (p.0 - 30.0).abs() < 1.5),
            "{bank:?}"
        );
        // The low side (east) is on the right: the line runs north.
        let (first, last) = (bank.points[0], bank.points[bank.points.len() - 1]);
        assert!(last.1 > first.1 + 20.0, "{bank:?}");
    }

    #[test]
    fn a_ragged_edge_gives_a_smooth_bank() {
        // The edge jumps 2 m east and back every 2 m, as the cells of a wide, uneven bank do.
        let ragged = |e: f64, n: f64| {
            let edge = 30.0 + 2.0 * ((n / 2.0).floor() as i64 % 2) as f64;
            40.0 + 1.5 / (1.0 + (-(e - edge) * 3.0).exp())
        };
        let jumps = height_jumps(&dtm(60, ragged), SPRINT, 1.0);
        assert_eq!(jumps.len(), 1, "{jumps:?}");
        let points = &jumps[0].points;
        assert!(points.len() <= 4, "{points:?}");
        assert!(
            points.iter().all(|p| (p.0 - 31.0).abs() < 0.8),
            "{points:?}"
        );
    }

    /// A stretch of an edge, from one end to the other.
    type Edge = ((f64, f64), (f64, f64));

    /// Every point of `jumps` lies within 2.5 m of `edges`, and the lines cover at least
    /// `share` of their length.
    fn follow(jumps: &[Landform], edges: &[Edge], share: f64) {
        use geo::{Distance as _, Euclidean, Length as _};
        let edges: Vec<geo_types::Line<f64>> = edges
            .iter()
            .map(|&(a, b)| geo_types::Line::new(a, b))
            .collect();
        for jump in jumps {
            for p in &jump.points {
                let off = edges
                    .iter()
                    .map(|l| Euclidean.distance(&geo_types::Point::from(*p), l))
                    .fold(f64::MAX, f64::min);
                assert!(off < 2.5, "{p:?} is {off} m off: {jumps:?}");
            }
        }
        let drawn: f64 = jumps
            .iter()
            .map(|j| Euclidean.length(&geo_types::LineString::from(j.points.clone())))
            .sum();
        let total: f64 = edges.iter().map(|l| Euclidean.length(l)).sum();
        assert!(drawn >= share * total, "{drawn} of {total} m: {jumps:?}");
    }

    /// A terrace raised by 1.5 m wherever `inside` says, its edge spread over two metres.
    fn terrace(inside: impl Fn(f64, f64) -> f64) -> impl Fn(f64, f64) -> f64 {
        move |e, n| 40.0 + 1.5 * ((inside(e, n) + 1.0) / 2.0).clamp(0.0, 1.0)
    }

    #[test]
    fn an_l_shaped_step_gives_banks_along_both_arms() {
        // The south-west quarter is raised: one edge runs north from (30, 0), one east from
        // (0, 30), and they meet at a corner.
        let jumps = height_jumps(
            &dtm(60, terrace(|e, n| (30.0 - e).min(30.0 - n))),
            SPRINT,
            1.0,
        );
        follow(
            &jumps,
            &[((30.0, 0.0), (30.0, 30.0)), ((0.0, 30.0), (30.0, 30.0))],
            0.6,
        );
    }

    #[test]
    fn a_u_shaped_step_follows_its_three_sides_with_the_low_side_inside() {
        // A channel 20 m wide sunk into a terrace, open to the south.
        let jumps = height_jumps(
            &dtm(60, terrace(|e, n| (20.0 - e).max(e - 40.0).max(n - 35.0))),
            SPRINT,
            1.0,
        );
        follow(
            &jumps,
            &[
                ((20.0, 0.0), (20.0, 35.0)),
                ((40.0, 0.0), (40.0, 35.0)),
                ((20.0, 35.0), (40.0, 35.0)),
            ],
            0.6,
        );
        // The low side (the channel) is on the right of every bank.
        for jump in &jumps {
            let (a, b) = (jump.points[0], jump.points[jump.points.len() - 1]);
            let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
            let right = (b.1 - a.1, a.0 - b.0);
            let norm = right.0.hypot(right.1);
            let probe = (mid.0 + 3.0 * right.0 / norm, mid.1 + 3.0 * right.1 / norm);
            let inside = (20.0..40.0).contains(&probe.0) && probe.1 < 35.0;
            assert!(inside, "{jump:?}");
        }
    }

    #[test]
    fn a_high_step_is_a_cliff() {
        let jumps = height_jumps(&dtm(60, step(4.0)), SPRINT, 1.0);
        assert_eq!(jumps.len(), 1, "{jumps:?}");
        assert_eq!(jumps[0].kind, LandformKind::Cliff);
    }

    #[test]
    fn a_low_step_is_no_bank() {
        assert!(height_jumps(&dtm(60, step(0.6)), SPRINT, 1.0).is_empty());
    }

    #[test]
    fn a_gentle_slope_gives_nothing() {
        assert!(height_jumps(&dtm(60, |e, _| 40.0 + 0.3 * e), SPRINT, 1.0).is_empty());
    }

    #[test]
    fn a_wide_steep_hillside_gives_nothing() {
        // A uniform 40° slope: steep everywhere, but no jump.
        assert!(height_jumps(&dtm(60, |e, _| 40.0 + 0.85 * e), SPRINT, 1.0).is_empty());
    }

    #[test]
    fn nan_cells_give_nothing() {
        let mut terrain = dtm(60, step(1.5));
        for y in 0..60 {
            for x in 26..34 {
                terrain.set(x, y, f32::NAN);
            }
        }
        assert!(height_jumps(&terrain, SPRINT, 1.0).is_empty());
    }

    #[test]
    fn geojson_round_trip() {
        let forms = vec![
            Landform {
                kind: LandformKind::ElongatedKnoll,
                points: vec![(1.5, 2.5)],
                rotation_deg: 30.0,
                height_m: 1.25,
            },
            Landform {
                kind: LandformKind::Bank,
                points: vec![(0.0, 0.0), (5.0, 1.0), (9.0, 0.0)],
                rotation_deg: 0.0,
                height_m: 1.5,
            },
        ];
        assert_eq!(
            landforms_from_geojson(&landforms_to_geojson(&forms)).unwrap(),
            forms
        );
        assert!(landforms_from_geojson(r#"{"features":[{"properties":{"kind":"x"}}]}"#).is_err());
    }
}
