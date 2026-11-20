// SPDX-License-Identifier: AGPL-3.0-or-later

//! Moves a height raster from PL-1992 (EPSG:2180, the grid of the Polish national data) into the
//! UTM grid of a project. The warp runs backwards: every pixel of the project grid is projected to
//! PL-1992 and read there, so no pixel of the result is left without a look.
//!
//! One sheet after the other goes into the same result ([`warp_into`]): a pixel that a sheet
//! already filled keeps its value, a pixel it left empty falls to the next sheet. That is the
//! mosaic, and where two sheets overlap the first one wins.

use crate::crs::{Crs, Pl1992, ProjectedPoint};
use crate::raster::{GridSpec, Raster};

/// How a height is read between the pixels of the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resample {
    /// Weighs the four nearest pixels. Heights stay smooth, the plane of a slope stays a plane.
    Bilinear,
    /// Takes the pixel the position lies in. Keeps values exact; for codes and for tests.
    Nearest,
}

/// The raster `src` (in PL-1992) on the grid `grid` of the project CRS `crs`; `NaN` where `src`
/// holds nothing.
pub fn warp(src: &Raster<f32>, grid: GridSpec, crs: Crs, how: Resample) -> Raster<f32> {
    let mut out = Raster::filled(grid, f32::NAN);
    warp_into(&mut out, crs, src, how);
    out
}

/// Writes `src` (in PL-1992) into the empty (`NaN`) pixels of `dst` (in `crs`) where `src` holds
/// a height. Pixels of `dst` that hold a height stay as they are.
pub fn warp_into(dst: &mut Raster<f32>, crs: Crs, src: &Raster<f32>, how: Resample) {
    let grid = dst.grid;
    for y in 0..grid.height {
        for x in 0..grid.width {
            if !dst.get(x, y).is_nan() {
                continue;
            }
            let Some(source) = source_position(crs, &grid, x, y) else {
                continue;
            };
            if let Some(z) = sample(src, source.easting, source.northing, how) {
                dst.set(x, y, z);
            }
        }
    }
}

/// The colour raster `src` (in PL-1992) on the grid `grid` of the project CRS `crs`, as RGBA:
/// alpha 255 where `src` holds a pixel, 0 (and black) where it does not.
pub fn warp_rgb(src: &Raster<[u8; 3]>, grid: GridSpec, crs: Crs, how: Resample) -> Raster<[u8; 4]> {
    let mut out = Raster::filled(grid, [0u8; 4]);
    warp_rgb_into(&mut out, crs, src, how);
    out
}

/// Writes `src` (in PL-1992) into the transparent (alpha 0) pixels of `dst` (in `crs`) where
/// `src` holds a pixel; a pixel that is already opaque stays, so the first sheet wins.
pub fn warp_rgb_into(dst: &mut Raster<[u8; 4]>, crs: Crs, src: &Raster<[u8; 3]>, how: Resample) {
    let grid = dst.grid;
    for y in 0..grid.height {
        for x in 0..grid.width {
            if dst.get(x, y)[3] != 0 {
                continue;
            }
            let Some(source) = source_position(crs, &grid, x, y) else {
                continue;
            };
            if let Some([r, g, b]) = sample_rgb(src, source.easting, source.northing, how) {
                dst.set(x, y, [r, g, b, 255]);
            }
        }
    }
}

/// Where the centre of pixel (`x`, `y`) of `grid` (in `crs`) lies in PL-1992.
fn source_position(crs: Crs, grid: &GridSpec, x: u32, y: u32) -> Option<ProjectedPoint> {
    let easting = grid.west + (f64::from(x) + 0.5) * grid.pixel_size;
    let northing = grid.north - (f64::from(y) + 0.5) * grid.pixel_size;
    crs.inverse(ProjectedPoint { easting, northing })
        .map(Pl1992::forward)
}

/// The colour of `src` at a position in its own grid; `None` outside it.
fn sample_rgb(
    src: &Raster<[u8; 3]>,
    easting: f64,
    northing: f64,
    how: Resample,
) -> Option<[u8; 3]> {
    let g = &src.grid;
    let fx = (easting - g.west) / g.pixel_size;
    let fy = (g.north - northing) / g.pixel_size;
    if !(0.0..f64::from(g.width)).contains(&fx) || !(0.0..f64::from(g.height)).contains(&fy) {
        return None;
    }
    if how == Resample::Nearest {
        return Some(src.get(fx as u32, fy as u32));
    }
    // The four pixel centres around the position; at the edge the outer pixel counts twice.
    let (gx, gy) = (fx - 0.5, fy - 0.5);
    let (x0, y0) = (gx.floor(), gy.floor());
    let (tx, ty) = (gx - x0, gy - y0);
    let at = |dx: f64, dy: f64| {
        let x = (x0 + dx).clamp(0.0, f64::from(g.width - 1)) as u32;
        let y = (y0 + dy).clamp(0.0, f64::from(g.height - 1)) as u32;
        src.get(x, y)
    };
    let corners = [
        (at(0.0, 0.0), (1.0 - tx) * (1.0 - ty)),
        (at(1.0, 0.0), tx * (1.0 - ty)),
        (at(0.0, 1.0), (1.0 - tx) * ty),
        (at(1.0, 1.0), tx * ty),
    ];
    let channel = |c: usize| {
        let sum: f64 = corners.iter().map(|(p, k)| f64::from(p[c]) * k).sum();
        sum.round().clamp(0.0, 255.0) as u8
    };
    Some([channel(0), channel(1), channel(2)])
}

/// The height of `src` at a position in its own grid; `None` outside it and where the pixel the
/// position lies in holds nothing.
fn sample(src: &Raster<f32>, easting: f64, northing: f64, how: Resample) -> Option<f32> {
    let g = &src.grid;
    // Position in pixels from the north-west corner; pixel `i` spans `i..i + 1`.
    let fx = (easting - g.west) / g.pixel_size;
    let fy = (g.north - northing) / g.pixel_size;
    if !(0.0..f64::from(g.width)).contains(&fx) || !(0.0..f64::from(g.height)).contains(&fy) {
        return None;
    }
    let (ix, iy) = (fx as u32, fy as u32);
    let nearest = src.get(ix, iy);
    if nearest.is_nan() || how == Resample::Nearest {
        return (!nearest.is_nan()).then_some(nearest);
    }
    // The four pixel centres around the position. At the edge of the sheet the outer pixel is
    // read twice, so the last half pixel of a sheet is not lost. A pixel without a height drops
    // out and the others are weighed again, so a hole does not pull its neighbours down.
    let (gx, gy) = (fx - 0.5, fy - 0.5);
    let (x0, y0) = (gx.floor(), gy.floor());
    let (tx, ty) = (gx - x0, gy - y0);
    let at = |dx: f64, dy: f64| {
        let x = (x0 + dx).clamp(0.0, f64::from(g.width - 1)) as u32;
        let y = (y0 + dy).clamp(0.0, f64::from(g.height - 1)) as u32;
        src.get(x, y)
    };
    let corners = [
        (at(0.0, 0.0), (1.0 - tx) * (1.0 - ty)),
        (at(1.0, 0.0), tx * (1.0 - ty)),
        (at(0.0, 1.0), (1.0 - tx) * ty),
        (at(1.0, 1.0), tx * ty),
    ];
    let (sum, weight) = corners
        .iter()
        .filter(|(z, _)| !z.is_nan())
        .fold((0.0, 0.0), |(s, w), (z, k)| (s + f64::from(*z) * k, w + k));
    // The pixel the position lies in is one of the four and holds a height, and a corner of it
    // always has a weight above zero, so `weight` is never zero here.
    Some((sum / weight) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;

    /// A position in Słubice: PL-1992 and UTM 33 of the same place.
    fn place() -> (ProjectedPoint, ProjectedPoint) {
        let ll = LatLon {
            lat: 52.35,
            lon: 14.57,
        };
        (Pl1992::forward(ll), Crs::Utm33.forward(ll))
    }

    /// A 1 m sheet in PL-1992 whose north-west corner is `dx`, `dy` from the place; the height of
    /// a pixel is `z` of its centre.
    fn sheet(
        dx: f64,
        dy: f64,
        width: u32,
        height: u32,
        z: impl Fn(f64, f64) -> f32,
    ) -> Raster<f32> {
        let (pl, _) = place();
        let grid = GridSpec {
            west: (pl.easting + dx).floor(),
            north: (pl.northing + dy).floor(),
            pixel_size: 1.0,
            width,
            height,
        };
        let mut r = Raster::filled(grid, 0.0);
        for y in 0..height {
            for x in 0..width {
                r.set(
                    x,
                    y,
                    z(
                        grid.west + f64::from(x) + 0.5,
                        grid.north - f64::from(y) - 0.5,
                    ),
                );
            }
        }
        r
    }

    /// A 1 m grid of `size` pixels in UTM 33 around the place.
    fn target(size: u32) -> GridSpec {
        let (_, utm) = place();
        GridSpec {
            west: (utm.easting - f64::from(size) / 2.0).floor(),
            north: (utm.northing + f64::from(size) / 2.0).floor(),
            pixel_size: 1.0,
            width: size,
            height: size,
        }
    }

    /// Where the centre of a pixel of `grid` lies in PL-1992.
    fn centre_in_pl(grid: &GridSpec, x: u32, y: u32) -> ProjectedPoint {
        let p = ProjectedPoint {
            easting: grid.west + (f64::from(x) + 0.5) * grid.pixel_size,
            northing: grid.north - (f64::from(y) + 0.5) * grid.pixel_size,
        };
        Pl1992::forward(Crs::Utm33.inverse(p).unwrap())
    }

    fn slope(e: f64, n: f64) -> f32 {
        let (pl, _) = place();
        (0.001 * (e - pl.easting) + 0.002 * (n - pl.northing)) as f32
    }

    #[test]
    fn the_plane_of_a_slope_keeps_its_heights_within_a_millimetre() {
        let src = sheet(-100.0, 100.0, 200, 200, slope);
        let grid = target(60);
        let out = warp(&src, grid, Crs::Utm33, Resample::Bilinear);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let p = centre_in_pl(&grid, x, y);
                let want = slope(p.easting, p.northing);
                assert!(
                    (out.get(x, y) - want).abs() < 1e-3,
                    "({x}, {y}): {} against {want}",
                    out.get(x, y)
                );
            }
        }
    }

    #[test]
    fn a_marker_pixel_lands_on_the_pixel_that_projects_to_it() {
        let grid = target(40);
        let (x, y) = (17, 23);
        let p = centre_in_pl(&grid, x, y);
        let mut src = sheet(-100.0, 100.0, 200, 200, |_, _| 0.0);
        let col = (p.easting - src.grid.west).floor() as u32;
        let row = (src.grid.north - p.northing).floor() as u32;
        src.set(col, row, 100.0);
        let out = warp(&src, grid, Crs::Utm33, Resample::Nearest);
        let marked: Vec<(u32, u32)> = (0..grid.height)
            .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
            .filter(|&(x, y)| out.get(x, y) == 100.0)
            .collect();
        assert!(marked.contains(&(x, y)), "{marked:?}");
        // One source pixel is about one target pixel: at most four are marked.
        assert!(marked.len() <= 4, "{marked:?}");
    }

    #[test]
    fn nearest_keeps_nodata_where_it_is() {
        let mut src = sheet(-100.0, 100.0, 200, 200, |_, _| 5.0);
        let grid = target(40);
        let p = centre_in_pl(&grid, 20, 20);
        src.set(
            (p.easting - src.grid.west).floor() as u32,
            (src.grid.north - p.northing).floor() as u32,
            f32::NAN,
        );
        let out = warp(&src, grid, Crs::Utm33, Resample::Nearest);
        assert!(out.get(20, 20).is_nan());
        let holes = out.data.iter().filter(|z| z.is_nan()).count();
        assert!((1..=4).contains(&holes), "{holes} holes");
    }

    #[test]
    fn bilinear_does_not_spread_nodata_beyond_the_pixel_it_sits_in() {
        let mut src = sheet(-100.0, 100.0, 200, 200, |_, _| 5.0);
        let grid = target(40);
        let p = centre_in_pl(&grid, 20, 20);
        src.set(
            (p.easting - src.grid.west).floor() as u32,
            (src.grid.north - p.northing).floor() as u32,
            f32::NAN,
        );
        let out = warp(&src, grid, Crs::Utm33, Resample::Bilinear);
        assert!(out.get(20, 20).is_nan());
        let holes = out.data.iter().filter(|z| z.is_nan()).count();
        assert!((1..=4).contains(&holes), "{holes} holes");
        // The neighbours are read from the pixels that hold a height.
        assert_eq!(out.get(22, 20), 5.0);
    }

    #[test]
    fn pixels_outside_the_sheet_stay_empty() {
        let src = sheet(-10.0, 10.0, 20, 20, |_, _| 1.0);
        let out = warp(&src, target(60), Crs::Utm33, Resample::Bilinear);
        assert!(out.get(0, 0).is_nan());
        assert!(out.get(59, 59).is_nan());
        assert!(out.get(30, 30) == 1.0);
    }

    #[test]
    fn no_hole_at_the_seam() {
        // Two sheets that overlap by 2 m; each has nodata in the corner that the other covers.
        let mut west = sheet(-30.0, 30.0, 31, 60, |_, _| 10.0);
        let mut east = sheet(-1.0, 30.0, 31, 60, |_, _| 20.0);
        for r in [&mut west, &mut east] {
            let (w, h) = (r.grid.width, r.grid.height);
            for (x, y) in [(w - 1, 0), (w - 1, h - 1), (0, 0), (0, h - 1)] {
                r.set(x, y, f32::NAN);
            }
        }
        let grid = target(30);
        let mut out = Raster::filled(grid, f32::NAN);
        for sheet in [&west, &east] {
            warp_into(&mut out, Crs::Utm33, sheet, Resample::Bilinear);
        }
        // Everything between the middle rows of the two sheets holds a height.
        for y in 8..22 {
            for x in 2..28 {
                assert!(!out.get(x, y).is_nan(), "hole at ({x}, {y})");
            }
        }
    }

    #[test]
    fn the_first_sheet_wins_where_two_hold_a_height() {
        let a = sheet(-30.0, 30.0, 60, 60, |_, _| 1.0);
        let b = sheet(-30.0, 30.0, 60, 60, |_, _| 2.0);
        let grid = target(20);
        let mut out = warp(&a, grid, Crs::Utm33, Resample::Bilinear);
        warp_into(&mut out, Crs::Utm33, &b, Resample::Bilinear);
        assert!(out.data.iter().all(|z| *z == 1.0));
    }

    #[test]
    fn a_gap_between_two_sheets_stays_nodata() {
        let a = sheet(-30.0, 30.0, 20, 60, |_, _| 1.0);
        let b = sheet(10.0, 30.0, 20, 60, |_, _| 2.0);
        let grid = target(40);
        let mut out = warp(&a, grid, Crs::Utm33, Resample::Bilinear);
        warp_into(&mut out, Crs::Utm33, &b, Resample::Bilinear);
        assert!(out.get(20, 20).is_nan());
        assert_eq!(out.get(2, 20), 1.0);
        assert_eq!(out.get(37, 20), 2.0);
    }
    /// A colour sheet like [`sheet`]: the colour of a pixel is `c` of its centre.
    fn rgb_sheet(
        dx: f64,
        dy: f64,
        width: u32,
        height: u32,
        c: impl Fn(f64, f64) -> [u8; 3],
    ) -> Raster<[u8; 3]> {
        let (pl, _) = place();
        let grid = GridSpec {
            west: (pl.easting + dx).floor(),
            north: (pl.northing + dy).floor(),
            pixel_size: 1.0,
            width,
            height,
        };
        let mut r = Raster::filled(grid, [0u8; 3]);
        for y in 0..height {
            for x in 0..width {
                let e = grid.west + f64::from(x) + 0.5;
                let n = grid.north - f64::from(y) - 0.5;
                r.set(x, y, c(e, n));
            }
        }
        r
    }

    #[test]
    fn a_flat_colour_stays_and_what_the_sheet_does_not_cover_is_transparent() {
        let src = rgb_sheet(-10.0, 10.0, 20, 20, |_, _| [10, 200, 90]);
        for how in [Resample::Bilinear, Resample::Nearest] {
            let out = warp_rgb(&src, target(60), Crs::Utm33, how);
            assert_eq!(out.get(30, 30), [10, 200, 90, 255], "{how:?}");
            assert_eq!(out.get(0, 0), [0, 0, 0, 0], "{how:?}");
            assert_eq!(out.get(59, 59), [0, 0, 0, 0], "{how:?}");
        }
    }

    #[test]
    fn a_colour_gradient_is_read_between_the_pixels_within_one_level() {
        let (pl, _) = place();
        let red = |e: f64| (128.0 + (e - pl.easting)) as u8;
        let src = rgb_sheet(-60.0, 60.0, 120, 120, |e, _| [red(e), 0, 0]);
        let grid = target(40);
        let out = warp_rgb(&src, grid, Crs::Utm33, Resample::Bilinear);
        for (x, y) in [(5, 5), (20, 20), (33, 12)] {
            let p = centre_in_pl(&grid, x, y);
            let want = 128.0 + (p.easting - pl.easting) - 0.5;
            let got = f64::from(out.get(x, y)[0]);
            assert!(
                (got - want).abs() <= 1.5,
                "({x}, {y}): {got} against {want}"
            );
        }
    }

    #[test]
    fn the_first_colour_sheet_wins_and_a_gap_stays_transparent() {
        let a = rgb_sheet(-30.0, 30.0, 20, 60, |_, _| [1, 1, 1]);
        let b = rgb_sheet(-30.0, 30.0, 20, 60, |_, _| [2, 2, 2]);
        let c = rgb_sheet(10.0, 30.0, 20, 60, |_, _| [3, 3, 3]);
        let grid = target(40);
        let mut out = warp_rgb(&a, grid, Crs::Utm33, Resample::Bilinear);
        for next in [&b, &c] {
            warp_rgb_into(&mut out, Crs::Utm33, next, Resample::Bilinear);
        }
        assert_eq!(out.get(2, 20), [1, 1, 1, 255]);
        assert_eq!(out.get(20, 20)[3], 0);
        assert_eq!(out.get(37, 20), [3, 3, 3, 255]);
    }

    /// A 4 by 3 grid of 2 m pixels with the north-west corner at (100, 200), in no CRS at all.
    fn plain_grid() -> GridSpec {
        GridSpec {
            west: 100.0,
            north: 200.0,
            pixel_size: 2.0,
            width: 4,
            height: 3,
        }
    }

    /// Positions between pixel centres (centres lie at 101, 103, 105, 107 east and 199, 197, 195
    /// north), with different fractions in the two directions.
    const PROBES: [(f64, f64); 3] = [(103.7, 197.6), (105.9, 195.4), (101.3, 198.8)];

    fn plane(e: f64, n: f64) -> f32 {
        (0.1 * (e - 100.0) + 0.3 * (200.0 - n)) as f32
    }

    #[test]
    fn a_height_between_pixels_of_two_metres_is_read_where_the_plane_has_it() {
        let mut src = Raster::filled(plain_grid(), 0.0f32);
        for y in 0..3 {
            for x in 0..4 {
                src.set(
                    x,
                    y,
                    plane(101.0 + 2.0 * f64::from(x), 199.0 - 2.0 * f64::from(y)),
                );
            }
        }
        for (e, n) in PROBES {
            let got = sample(&src, e, n, Resample::Bilinear).unwrap();
            assert!((got - plane(e, n)).abs() < 1e-4, "({e}, {n}): {got}");
        }
        // Nearest takes the pixel the position lies in: pixel (1, 1) for the first probe.
        assert_eq!(
            sample(&src, 103.7, 197.6, Resample::Nearest),
            Some(plane(103.0, 197.0))
        );
        // Within the outer half pixel the outer pixel counts twice.
        assert_eq!(
            sample(&src, 100.4, 199.5, Resample::Bilinear),
            Some(plane(101.0, 199.0))
        );
        assert_eq!(sample(&src, 99.0, 197.0, Resample::Bilinear), None);
        assert_eq!(sample(&src, 108.5, 197.0, Resample::Bilinear), None);
    }

    #[test]
    fn a_colour_between_pixels_of_two_metres_is_read_where_the_gradient_has_it() {
        let mut src = Raster::filled(plain_grid(), [0u8; 3]);
        for y in 0..3u32 {
            for x in 0..4u32 {
                src.set(x, y, [(10 + 20 * x) as u8, (5 + 30 * y) as u8, 100]);
            }
        }
        let want = [[37, 26, 100], [59, 59, 100], [13, 8, 100]];
        for ((e, n), want) in PROBES.into_iter().zip(want) {
            assert_eq!(
                sample_rgb(&src, e, n, Resample::Bilinear),
                Some(want),
                "({e}, {n})"
            );
        }
        assert_eq!(
            sample_rgb(&src, 103.7, 197.6, Resample::Nearest),
            Some([30, 35, 100])
        );
        assert_eq!(sample_rgb(&src, 99.0, 197.0, Resample::Bilinear), None);
        assert_eq!(sample_rgb(&src, 103.0, 193.5, Resample::Bilinear), None);
    }

    #[test]
    fn the_centre_of_a_pixel_of_two_metres_projects_to_the_place_it_names() {
        let (_, utm) = place();
        let grid = GridSpec {
            west: utm.easting.floor(),
            north: utm.northing.floor(),
            pixel_size: 2.0,
            width: 10,
            height: 10,
        };
        let got = source_position(Crs::Utm33, &grid, 3, 5).unwrap();
        // Pixel (3, 5) has its centre 7 m east and 11 m south of the corner.
        let want = Pl1992::forward(
            Crs::Utm33
                .inverse(ProjectedPoint {
                    easting: grid.west + 7.0,
                    northing: grid.north - 11.0,
                })
                .unwrap(),
        );
        assert!((got.easting - want.easting).abs() < 1e-6, "{got:?}");
        assert!((got.northing - want.northing).abs() < 1e-6, "{got:?}");
    }
}
