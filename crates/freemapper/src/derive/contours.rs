//! Contour lines from the DTM (spec §4.4): smoothed, traced, simplified; candidates only.

use geo::{Length as _, Simplify as _};

use crate::measure::{PaperMm, PaperMm2};
use crate::project::MapKind;
use crate::raster::Raster;

/// Simplification tolerance and shortest kept line, on paper.
const SIMPLIFY_MM: PaperMm = PaperMm(0.1);
const MIN_LENGTH_MM: PaperMm = PaperMm(2.0);
/// Closed rings enclosing less than this on paper are noise at a 1 m interval.
const MIN_RING_AREA_MM2: PaperMm2 = PaperMm2(1.5);

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

/// Simplifies the lines for the map scale and drops lines shorter than 2 mm and closed rings
/// enclosing less than 1.5 mm² on paper.
pub fn generalize(lines: Vec<ContourLine>, scale_denominator: u32) -> Vec<ContourLine> {
    use geo::Area as _;
    lines
        .into_iter()
        .filter_map(|line| {
            let ls = geo_types::LineString::from(line.points.clone());
            let simplified = ls.simplify(SIMPLIFY_MM.metres(scale_denominator));
            let closed = simplified.0.len() > 3 && simplified.is_closed();
            let ring_area = geo_types::Polygon::new(simplified.clone(), vec![]).unsigned_area();
            if closed && ring_area < MIN_RING_AREA_MM2.square_metres(scale_denominator) {
                return None;
            }
            (geo::Euclidean.length(&simplified) >= MIN_LENGTH_MM.metres(scale_denominator)).then(
                || ContourLine {
                    height: line.height,
                    points: simplified.0.iter().map(|c| (c.x, c.y)).collect(),
                },
            )
        })
        .collect()
}

/// A forest map gets half the interval where the map area is this flat: the 5th to the 95th
/// percentile of its heights span less (spec 2026-11-13).
const FLAT_FOREST_RANGE_M: f64 = 15.0;
const FLAT_FOREST_INTERVAL_M: f64 = 2.5;
/// The flatness test samples the terrain at about this spacing; a percentile needs no more.
const HEIGHT_SAMPLE_M: f64 = 5.0;

/// The contour interval of a map of `kind` over `heights_in_area` (the smoothed DTM inside the
/// map area): a forest map 2.5 m where it is flat, the kind's interval otherwise and without
/// heights. `NaN` heights are skipped.
pub fn contour_interval_for(kind: MapKind, heights_in_area: &[f32]) -> f64 {
    let default = kind.contour_interval_m();
    if kind != MapKind::Forest {
        return default;
    }
    let mut heights: Vec<f32> = heights_in_area
        .iter()
        .copied()
        .filter(|h| h.is_finite())
        .collect();
    if heights.is_empty() {
        return default;
    }
    heights.sort_by(f32::total_cmp);
    let at = |share: usize| f64::from(heights[(heights.len() - 1) * share / 100]);
    if at(95) - at(5) < FLAT_FOREST_RANGE_M {
        FLAT_FOREST_INTERVAL_M
    } else {
        default
    }
}

/// The heights of every cell about [`HEIGHT_SAMPLE_M`] apart of `raster` whose centre lies in
/// `area` (same CRS), `NaN` included.
pub fn heights_inside(raster: &Raster<f32>, area: &geo_types::Polygon<f64>) -> Vec<f32> {
    use geo::{BoundingRect as _, Contains as _};
    let Some(bounds) = area.bounding_rect() else {
        return Vec::new();
    };
    let grid = raster.grid;
    let column = |e: f64| ((e - grid.west) / grid.pixel_size).floor();
    let row = |n: f64| ((grid.north - n) / grid.pixel_size).floor();
    let clamp = |v: f64, size: u32| v.clamp(0.0, f64::from(size) - 1.0) as u32;
    let (x0, x1) = (
        clamp(column(bounds.min().x), grid.width),
        clamp(column(bounds.max().x), grid.width),
    );
    let (y0, y1) = (
        clamp(row(bounds.max().y), grid.height),
        clamp(row(bounds.min().y), grid.height),
    );
    let step = ((HEIGHT_SAMPLE_M / grid.pixel_size).round() as usize).max(1);
    let mut heights = Vec::new();
    for y in (y0..=y1).step_by(step) {
        for x in (x0..=x1).step_by(step) {
            let centre = geo_types::Point::new(
                grid.west + (f64::from(x) + 0.5) * grid.pixel_size,
                grid.north - (f64::from(y) + 0.5) * grid.pixel_size,
            );
            if area.contains(&centre) {
                heights.push(raster.get(x, y));
            }
        }
    }
    heights
}

/// The lines as GeoJSON LineStrings in EPSG:25833 with a `height` property; the interval they
/// were traced at as the member `interval_m` when it is not the map kind's.
pub fn contours_to_geojson(lines: &[ContourLine], interval_m: Option<f64>) -> String {
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
    let mut collection = serde_json::json!({ "type": "FeatureCollection", "features": features });
    if let Some(interval) = interval_m {
        collection["interval_m"] = serde_json::json!(interval);
    }
    collection.to_string()
}

/// Reads lines and interval written by [`contours_to_geojson`].
pub fn contours_from_geojson(text: &str) -> Result<(Vec<ContourLine>, Option<f64>), ContourError> {
    let collection: serde_json::Value =
        serde_json::from_str(text).map_err(|e| ContourError(e.to_string()))?;
    let features = collection["features"]
        .as_array()
        .ok_or_else(|| ContourError("no features".to_string()))?;
    let interval = collection["interval_m"].as_f64();
    let lines = features
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
        .collect::<Result<_, _>>()?;
    Ok((lines, interval))
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
        let two_metre_lines = lines.iter().filter(|l| l.height == 2.0).count();
        assert_eq!(two_metre_lines, 2, "the 2 m line is cut in two by the hole");
        assert!(smooth.get(25, 25).is_nan());
        assert!((smooth.get(10, 10) - 52.3).abs() < 1e-4);
        // Next to the hole and on the border: neither pulled down by nodata nor biased.
        assert!(
            (smooth.get(19, 25) - 52.3).abs() < 1e-4,
            "{}",
            smooth.get(19, 25)
        );
        assert!(
            (smooth.get(0, 0) - 52.3).abs() < 1e-4,
            "{}",
            smooth.get(0, 0)
        );
    }

    #[test]
    fn runs_wrap_around_the_ring_start() {
        // A closed ring starting inside a usable stretch, with one unusable point in the middle.
        let ring = [
            (1.0, 1.0),
            (2.0, 1.0),
            (9.0, 9.0),
            (2.0, 2.0),
            (1.0, 2.0),
            (1.0, 1.0),
        ];
        let runs = usable_runs(&ring, &|x, _| x > 5.0);
        assert_eq!(runs, [vec![(2.0, 2.0), (1.0, 2.0), (1.0, 1.0), (2.0, 1.0)]]);
    }

    #[test]
    fn tiny_closed_rings_are_dropped() {
        let ring = |side: f64| ContourLine {
            height: 60.0,
            points: vec![
                (0.0, 0.0),
                (side, 0.0),
                (side, side),
                (0.0, side),
                (0.0, 0.0),
            ],
        };
        // At 1:1000, 1.5 mm² on paper are 1.5 m²: a 1 m square ring goes, a 2 m one stays.
        let kept = generalize(vec![ring(1.0), ring(2.0)], 1000);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].points[1], (2.0, 0.0));
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
            contours_from_geojson(&contours_to_geojson(&lines, None)).unwrap(),
            (lines, None)
        );
    }

    #[test]
    fn the_interval_is_stored_with_the_contours() {
        let lines = contour_lines(&dtm(|e, _| (e / 4.0) as f32), 2.5);
        let text = contours_to_geojson(&lines, Some(2.5));
        assert!(text.contains(r#""interval_m":2.5"#), "{text}");
        assert_eq!(contours_from_geojson(&text).unwrap(), (lines, Some(2.5)));
        // Without a stored interval the file is as before (the golden net).
        assert!(!contours_to_geojson(&[], None).contains("interval_m"));
    }

    #[test]
    fn flat_forest_gets_2_5_m_and_hilly_5_m() {
        let range = |low: f32, high: f32| -> Vec<f32> {
            (0..=100)
                .map(|i| low + (high - low) * i as f32 / 100.0)
                .collect()
        };
        assert_eq!(
            contour_interval_for(MapKind::Forest, &range(40.0, 52.0)),
            2.5
        );
        assert_eq!(
            contour_interval_for(MapKind::Forest, &range(40.0, 80.0)),
            5.0
        );
        // A few outliers (a deep pit, a tower of noise) do not decide: the 5th to 95th percentile.
        let mut flat = range(40.0, 50.0);
        flat.extend([10.0, 120.0, f32::NAN]);
        assert_eq!(contour_interval_for(MapKind::Forest, &flat), 2.5);
        assert_eq!(contour_interval_for(MapKind::Forest, &[]), 5.0);
        assert_eq!(
            contour_interval_for(MapKind::Sprint, &range(40.0, 41.0)),
            2.0
        );
        assert_eq!(
            contour_interval_for(MapKind::Schoolyard, &range(40.0, 80.0)),
            1.0
        );
    }

    #[test]
    fn heights_inside_sample_the_cells_whose_centre_lies_in_the_area() {
        let raster = dtm(|e, _| e as f32);
        let area = geo_types::Polygon::new(
            geo_types::LineString::from(vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 50.0),
                (0.0, 50.0),
                (0.0, 0.0),
            ]),
            vec![],
        );
        let heights = heights_inside(&raster, &area);
        // Every 5th cell of a 1 m raster: 2 columns × 10 rows.
        assert_eq!(heights.len(), 2 * 10);
        assert!(heights.iter().all(|h| *h < 10.0));
    }
}
