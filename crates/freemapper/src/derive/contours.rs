//! Contour lines from the DTM (spec §4.4): smoothed, traced, simplified; candidates only.

use geo::{Length as _, Simplify as _};

use crate::raster::Raster;

/// Simplification tolerance and shortest kept line, on paper.
const SIMPLIFY_MM: f64 = 0.1;
const MIN_LENGTH_MM: f64 = 2.0;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid contours: {0}")]
pub struct ContourError(String);

/// One contour line in EPSG:25833.
#[derive(Debug, Clone, PartialEq)]
pub struct ContourLine {
    pub height: f64,
    pub points: Vec<(f64, f64)>,
}

/// Gaussian smoothing with `sigma_m` metres. Only valid cells contribute (normalized
/// convolution); `NaN` cells stay `NaN`.
pub fn gaussian_smooth(dtm: &Raster<f32>, sigma_m: f64) -> Raster<f32> {
    let (w, h) = (dtm.grid.width as usize, dtm.grid.height as usize);
    let sigma = sigma_m / dtm.grid.pixel_size;
    let radius = (3.0 * sigma).ceil() as isize;
    let kernel: Vec<f64> = (-radius..=radius)
        .map(|i| (-((i * i) as f64) / (2.0 * sigma * sigma)).exp())
        .collect();
    // Horizontal then vertical pass over (weighted sum, weight) pairs.
    let pass = |values: &[(f64, f64)], horizontal: bool| -> Vec<(f64, f64)> {
        let mut out = vec![(0.0, 0.0); w * h];
        for y in 0..h {
            for x in 0..w {
                let mut acc = (0.0, 0.0);
                for (k, weight) in kernel.iter().enumerate() {
                    let d = k as isize - radius;
                    let (xx, yy) = if horizontal {
                        (x as isize + d, y as isize)
                    } else {
                        (x as isize, y as isize + d)
                    };
                    if xx < 0 || yy < 0 || xx >= w as isize || yy >= h as isize {
                        continue;
                    }
                    let (sum, weights) = values[yy as usize * w + xx as usize];
                    acc.0 += weight * sum;
                    acc.1 += weight * weights;
                }
                out[y * w + x] = acc;
            }
        }
        out
    };
    let start: Vec<(f64, f64)> = dtm
        .data
        .iter()
        .map(|z| {
            if z.is_nan() {
                (0.0, 0.0)
            } else {
                (f64::from(*z), 1.0)
            }
        })
        .collect();
    let smoothed = pass(&pass(&start, true), false);
    Raster {
        grid: dtm.grid,
        data: dtm
            .data
            .iter()
            .zip(smoothed)
            .map(|(z, (sum, weight))| {
                if z.is_nan() || weight == 0.0 {
                    f32::NAN
                } else {
                    (sum / weight) as f32
                }
            })
            .collect(),
    }
}

/// Contour lines at the multiples of `interval_m` between the lowest and highest height. Lines
/// end at the raster border and next to cells without data, so neither creates false contours.
pub fn contour_lines(dtm: &Raster<f32>, interval_m: f64) -> Vec<ContourLine> {
    let (w, h) = (dtm.grid.width as usize, dtm.grid.height as usize);
    let valid: Vec<f64> = dtm
        .data
        .iter()
        .filter(|z| !z.is_nan())
        .map(|z| f64::from(*z))
        .collect();
    if valid.is_empty() || w < 2 || h < 2 {
        return Vec::new();
    }
    let (min, max) = valid
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), z| (lo.min(*z), hi.max(*z)));
    let mean = valid.iter().sum::<f64>() / valid.len() as f64;
    let first = (min / interval_m).ceil() as i64;
    let last = (max / interval_m).floor() as i64;
    let levels: Vec<f64> = (first..=last).map(|k| k as f64 * interval_m).collect();
    if levels.is_empty() {
        return Vec::new();
    }
    let values: Vec<f64> = dtm
        .data
        .iter()
        .map(|z| if z.is_nan() { mean } else { f64::from(*z) })
        .collect();
    let nodata = |col: isize, row: isize| {
        col >= 0
            && row >= 0
            && (col as usize) < w
            && (row as usize) < h
            && dtm.data[row as usize * w + col as usize].is_nan()
    };
    // A ring point lies between up to four pixel centres (at i + 0.5); it is unusable on the
    // raster border or next to a cell without data.
    let unusable = |gx: f64, gy: f64| {
        let eps = 1e-9;
        if gx <= eps || gy <= eps || gx >= w as f64 - eps || gy >= h as f64 - eps {
            return true;
        }
        let cols = [(gx - 0.5).floor() as isize, (gx - 0.5).ceil() as isize];
        let rows = [(gy - 0.5).floor() as isize, (gy - 0.5).ceil() as isize];
        cols.iter().any(|c| rows.iter().any(|r| nodata(*c, *r)))
    };
    let to_projected = |gx: f64, gy: f64| {
        (
            dtm.grid.west + gx * dtm.grid.pixel_size,
            dtm.grid.north - gy * dtm.grid.pixel_size,
        )
    };
    let builder = contour::ContourBuilder::new(w, h, true);
    let traced = builder
        .lines(&values, &levels)
        .expect("values match the grid");
    let mut lines = Vec::new();
    for line in traced {
        let height = line.threshold();
        for ring in &line.geometry().0 {
            let points: Vec<(f64, f64)> = ring.0.iter().map(|c| (c.x, c.y)).collect();
            for run in usable_runs(&points, &unusable) {
                lines.push(ContourLine {
                    height,
                    points: run.into_iter().map(|(x, y)| to_projected(x, y)).collect(),
                });
            }
        }
    }
    lines
}

/// The parts of a ring between unusable points, each with at least two points. A ring without
/// unusable points stays whole (closed).
fn usable_runs(ring: &[(f64, f64)], unusable: &dyn Fn(f64, f64) -> bool) -> Vec<Vec<(f64, f64)>> {
    let Some(cut) = ring.iter().position(|(x, y)| unusable(*x, *y)) else {
        return vec![ring.to_vec()];
    };
    // Start at an unusable point so that no run wraps around the ring's start.
    let closed = ring.len() > 1 && ring.first() == ring.last();
    let open: &[(f64, f64)] = if closed {
        &ring[..ring.len() - 1]
    } else {
        ring
    };
    let rotated: Vec<(f64, f64)> = if closed {
        open[cut..].iter().chain(&open[..cut]).copied().collect()
    } else {
        open.to_vec()
    };
    let mut runs = Vec::new();
    let mut current = Vec::new();
    for p in rotated {
        if unusable(p.0, p.1) {
            if current.len() >= 2 {
                runs.push(std::mem::take(&mut current));
            }
            current.clear();
        } else {
            current.push(p);
        }
    }
    if current.len() >= 2 {
        runs.push(current);
    }
    runs
}

/// Simplifies the lines for the map scale and drops lines shorter than 2 mm on paper.
pub fn generalize(lines: Vec<ContourLine>, scale_denominator: u32) -> Vec<ContourLine> {
    let metres_per_mm = f64::from(scale_denominator) / 1000.0;
    lines
        .into_iter()
        .filter_map(|line| {
            let ls = geo_types::LineString::from(line.points.clone());
            let simplified = ls.simplify(SIMPLIFY_MM * metres_per_mm);
            (geo::Euclidean.length(&simplified) >= MIN_LENGTH_MM * metres_per_mm).then(|| {
                ContourLine {
                    height: line.height,
                    points: simplified.0.iter().map(|c| (c.x, c.y)).collect(),
                }
            })
        })
        .collect()
}

/// The lines as GeoJSON LineStrings in EPSG:25833 with a `height` property.
pub fn contours_to_geojson(lines: &[ContourLine]) -> String {
    let features: Vec<serde_json::Value> = lines
        .iter()
        .map(|l| {
            serde_json::json!({
                "type": "Feature",
                "properties": { "height": l.height },
                "geometry": {
                    "type": "LineString",
                    "coordinates": l.points.iter().map(|(e, n)| [*e, *n]).collect::<Vec<_>>()
                }
            })
        })
        .collect();
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// Reads lines written by [`contours_to_geojson`].
pub fn contours_from_geojson(text: &str) -> Result<Vec<ContourLine>, ContourError> {
    let collection: serde_json::Value =
        serde_json::from_str(text).map_err(|e| ContourError(e.to_string()))?;
    let features = collection["features"]
        .as_array()
        .ok_or_else(|| ContourError("no features".to_string()))?;
    features
        .iter()
        .map(|f| {
            let height = f["properties"]["height"]
                .as_f64()
                .ok_or_else(|| ContourError("feature without height".to_string()))?;
            let points = f["geometry"]["coordinates"]
                .as_array()
                .ok_or_else(|| ContourError("feature without coordinates".to_string()))?
                .iter()
                .map(|c| Some((c.get(0)?.as_f64()?, c.get(1)?.as_f64()?)))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| ContourError("invalid coordinate".to_string()))?;
            Ok(ContourLine { height, points })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::GridSpec;

    /// 50 × 50 DTM with 1 m pixels, west 0, north 50; `z(e, n)` at pixel centres.
    fn dtm(z: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let grid = GridSpec {
            west: 0.0,
            north: 50.0,
            pixel_size: 1.0,
            width: 50,
            height: 50,
        };
        let mut r = Raster::filled(grid, 0.0);
        for y in 0..50 {
            for x in 0..50 {
                r.set(x, y, z(f64::from(x) + 0.5, 50.0 - f64::from(y) - 0.5));
            }
        }
        r
    }

    #[test]
    fn plane_gives_straight_contours() {
        let lines = contour_lines(&dtm(|e, _| (e / 10.0) as f32), 1.0);
        let heights: Vec<f64> = lines.iter().map(|l| l.height).collect();
        assert_eq!(heights, [1.0, 2.0, 3.0, 4.0]);
        for line in &lines {
            assert!(line.points.len() >= 2);
            for (e, n) in &line.points {
                assert!(
                    (e - line.height * 10.0).abs() < 0.01,
                    "{e} for {}",
                    line.height
                );
                assert!((0.0..=50.0).contains(n));
            }
        }
    }

    #[test]
    fn flat_dtm_gives_no_contours() {
        assert!(contour_lines(&dtm(|_, _| 52.3), 1.0).is_empty());
    }

    #[test]
    fn smoothing_keeps_a_plane() {
        let plane = dtm(|e, n| (e / 10.0 + n / 20.0) as f32);
        let smooth = gaussian_smooth(&plane, 1.5);
        for (y, x) in [(10, 10), (25, 30), (40, 5)] {
            assert!(
                (smooth.get(x, y) - plane.get(x, y)).abs() < 1e-4,
                "({x}, {y})"
            );
        }
    }

    #[test]
    fn nodata_does_not_create_contours() {
        let mut holed = dtm(|_, _| 52.3);
        for y in 20..30 {
            for x in 20..30 {
                holed.set(x, y, f32::NAN);
            }
        }
        assert!(contour_lines(&holed, 1.0).is_empty());
        let smooth = gaussian_smooth(&holed, 1.5);
        // On a slope the lines through the hole end at its edge instead of running around it.
        let mut slope = dtm(|e, _| (e / 10.0) as f32);
        for y in 20..30 {
            for x in 20..30 {
                slope.set(x, y, f32::NAN);
            }
        }
        let lines = contour_lines(&slope, 1.0);
        let in_hole = |(e, n): &(f64, f64)| (20.0..30.0).contains(e) && (20.0..30.0).contains(n);
        assert!(
            lines.iter().all(|l| !l.points.iter().any(in_hole)),
            "{lines:?}"
        );
        let three = lines.iter().filter(|l| l.height == 2.0).count();
        assert_eq!(three, 2, "the 2 m line is cut in two by the hole");
        assert!(smooth.get(25, 25).is_nan());
        assert!((smooth.get(10, 10) - 52.3).abs() < 1e-4);
    }

    #[test]
    fn short_lines_are_dropped() {
        let lines = vec![
            ContourLine {
                height: 1.0,
                points: vec![(0.0, 0.0), (1.0, 0.0)],
            },
            ContourLine {
                height: 2.0,
                points: vec![(0.0, 0.0), (1.0, 0.05), (3.0, 0.0)],
            },
        ];
        let kept = generalize(lines, 1000);
        assert_eq!(kept.len(), 1, "1 m is shorter than 2 mm at 1:1000");
        assert_eq!(kept[0].points, [(0.0, 0.0), (3.0, 0.0)], "simplified");
    }

    #[test]
    fn geojson_roundtrip() {
        let lines = contour_lines(&dtm(|e, _| (e / 10.0) as f32), 1.0);
        assert_eq!(
            contours_from_geojson(&contours_to_geojson(&lines)).unwrap(),
            lines
        );
    }
}
