// SPDX-License-Identifier: AGPL-3.0-or-later

//! Where the orthophoto shows rock (ADR 0126): for a forest map, the share of rock-coloured
//! pixels in every metre of the orthophoto, so that a cliff the photo confirms goes first.
//!
//! Rock is light and not green: the mean of red, green and blue above 115, green no more
//! than 5 above red (leaves and needles are green, sandstone is grey or reddish),
//! the colours less than 70 apart. A pixel darker than 55 says nothing, and a
//! metre mostly in shadow has no value. Most rock faces are vertical and seen from above only at
//! their top edge, so the photo can confirm a cliff but never deny one (at the Bastei it shows
//! rock at 180 of 2149 OSM cliffs).

use image::RgbImage;

use crate::raster::{GridSpec, Raster};

const LIGHT: f64 = 115.0;
const GREEN_OVER_RED: i32 = 5;
const SPREAD: u8 = 70;
const SHADOW: f64 = 55.0;
/// A metre with less than this share of lit pixels is in shadow.
const LIT_SHARE: f64 = 0.5;
/// The cells of the evidence, about this wide (metres).
const CELL_M: f64 = 1.0;

/// The share of rock-coloured pixels of `image` (laid out on `grid`) per cell of about
/// a metre, `NaN` where the cell lies mostly in shadow.
pub fn rock_seen(image: &RgbImage, grid: &GridSpec) -> Raster<f32> {
    let f = ((CELL_M / grid.pixel_size).round() as u32).max(1);
    let (w, h) = (image.width() / f, image.height() / f);
    let mut out = Raster::filled(
        GridSpec {
            west: grid.west,
            north: grid.north,
            pixel_size: grid.pixel_size * f64::from(f),
            width: w,
            height: h,
        },
        f32::NAN,
    );
    for cy in 0..h {
        for cx in 0..w {
            let (mut lit, mut rock) = (0u32, 0u32);
            for y in cy * f..(cy + 1) * f {
                for x in cx * f..(cx + 1) * f {
                    let [r, g, b] = image.get_pixel(x, y).0;
                    let mean = (f64::from(r) + f64::from(g) + f64::from(b)) / 3.0;
                    if mean < SHADOW {
                        continue;
                    }
                    lit += 1;
                    let spread = r.max(g).max(b) - r.min(g).min(b);
                    if mean > LIGHT
                        && i32::from(g) <= i32::from(r) + GREEN_OVER_RED
                        && spread < SPREAD
                    {
                        rock += 1;
                    }
                }
            }
            if f64::from(lit) >= LIT_SHARE * f64::from(f * f) {
                out.set(cx, cy, rock as f32 / lit as f32);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    #[test]
    fn grey_rock_counts_green_and_shadow_do_not() {
        // 0.2 m pixels: one metre is 5 × 5 of them.
        let grid = GridSpec {
            west: 0.0,
            north: 3.0,
            pixel_size: 0.2,
            width: 15,
            height: 5,
        };
        let mut image = RgbImage::new(15, 5);
        for y in 0..5 {
            for x in 0..15 {
                let colour = match x / 5 {
                    0 => Rgb([180, 175, 165]), // sandstone
                    1 => Rgb([90, 140, 70]),   // needles
                    _ => Rgb([20, 25, 20]),    // shadow
                };
                image.put_pixel(x, y, colour);
            }
        }
        let seen = rock_seen(&image, &grid);
        assert_eq!((seen.grid.width, seen.grid.height), (3, 1));
        assert!((seen.grid.pixel_size - 1.0).abs() < 1e-9);
        assert_eq!(seen.get(0, 0), 1.0);
        assert_eq!(seen.get(1, 0), 0.0);
        assert!(seen.get(2, 0).is_nan(), "a metre in shadow says nothing");
    }
}
