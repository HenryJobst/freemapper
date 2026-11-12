//! Drawing primitives as a raster image: the map preview next to the map file (`tiny-skia`, pure
//! Rust). The same primitives and the same painting order as [`super::pdf`], in sRGB on white.
//! Texts are left out: there is no font here, and the map part holds no text objects.

use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform,
};

use super::draw::{Primitive, Shape};
use super::geometry::Pt;
use super::pdf::{MITER_LIMIT, PaperBox};
use super::symbols::{Cap, Join, RenderColour};

/// At most this many pixels per millimetre of paper.
const MAX_PX_PER_MM: f64 = 8.0;
/// At most this many pixels on the long side of the image.
const MAX_LONG_SIDE_PX: f64 = 2048.0;

/// Pixels per millimetre for a preview of `clip`: the long side gets at most 2048 px, and no more
/// than 8 px per mm.
pub fn preview_px_per_mm(clip: &PaperBox) -> f64 {
    let long_side = (clip.max.0 - clip.min.0).max(clip.max.1 - clip.min.1);
    MAX_PX_PER_MM.min(MAX_LONG_SIDE_PX / long_side)
}

/// The image of `primitives` within `clip` at `px_per_mm`: white background, the primitives
/// painted by colour priority as the PDF paints them. Its top-left corner is the top-left corner
/// of `clip` as the PDF page shows it (paper mm, y down).
pub fn rasterize(
    primitives: &[Primitive],
    clip: PaperBox,
    colours: &[RenderColour],
    px_per_mm: f64,
) -> image::RgbaImage {
    let side = |mm: f64| ((mm * px_per_mm).round() as u32).max(1);
    let (width, height) = (side(clip.max.0 - clip.min.0), side(clip.max.1 - clip.min.1));
    let Some(mut pixmap) = Pixmap::new(width, height) else {
        // Too large for a pixmap; the preview size never gets there.
        return image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]));
    };
    pixmap.fill(Color::WHITE);
    let px = |p: Pt| {
        (
            ((p.0 - clip.min.0) * px_per_mm) as f32,
            ((p.1 - clip.min.1) * px_per_mm) as f32,
        )
    };
    // Highest priority first: priority 0 is painted last, on top. Stable, so objects keep their
    // order within a colour (as in `pdf::write_pdf`).
    let mut items: Vec<&Primitive> = primitives.iter().collect();
    items.sort_by_key(|p| std::cmp::Reverse(p.priority));
    for primitive in items {
        let [r, g, b] = srgb(
            colours
                .iter()
                .find(|c| c.priority == primitive.priority)
                .map_or([0.0, 0.0, 0.0, 1.0], |c| c.cmyk),
        );
        let mut paint = Paint::default();
        paint.set_color_rgba8(r, g, b, 255);
        paint.anti_alias = true;
        match &primitive.shape {
            Shape::Fill { rings } => {
                let mut builder = PathBuilder::new();
                for ring in rings {
                    add_points(&mut builder, ring.iter().map(|&p| px(p)), true);
                }
                if let Some(path) = builder.finish() {
                    pixmap.fill_path(
                        &path,
                        &paint,
                        FillRule::EvenOdd,
                        Transform::identity(),
                        None,
                    );
                }
            }
            Shape::Stroke {
                path,
                closed,
                width,
                cap,
                join,
            } => {
                let mut builder = PathBuilder::new();
                add_points(&mut builder, path.iter().map(|&p| px(p)), *closed);
                let Some(path) = builder.finish() else {
                    continue;
                };
                let stroke = Stroke {
                    width: (width * px_per_mm) as f32,
                    miter_limit: MITER_LIMIT,
                    line_cap: match cap {
                        Cap::Flat => LineCap::Butt,
                        Cap::Round => LineCap::Round,
                        Cap::Square => LineCap::Square,
                    },
                    line_join: match join {
                        Join::Miter => LineJoin::Miter,
                        Join::Round => LineJoin::Round,
                        Join::Bevel => LineJoin::Bevel,
                    },
                    ..Stroke::default()
                };
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
            // No font here; the map part holds no text objects (the sheet is not drawn).
            Shape::Text { .. } => {}
        }
    }
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    image::RgbaImage::from_raw(width, height, rgba)
        .unwrap_or_else(|| image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255])))
}

/// Adds `points` as one subpath, closed if `close`.
fn add_points(builder: &mut PathBuilder, points: impl Iterator<Item = (f32, f32)>, close: bool) {
    let mut first = true;
    for (x, y) in points {
        if first {
            builder.move_to(x, y);
            first = false;
        } else {
            builder.line_to(x, y);
        }
    }
    if close && !first {
        builder.close();
    }
}

/// `cmyk` (each 0 to 1) as sRGB: r = 255 · (1 − c) · (1 − k), g and b alike from m and y,
/// rounded, values outside 0 to 1 clamped. The formula of `freemapper_render::svg::srgb` in
/// freemapper-core (v0.21.1), which draws the same colours as SVG; freemapper does not depend on
/// that crate, so it is repeated here.
fn srgb(cmyk: [f64; 4]) -> [u8; 3] {
    let [c, m, y, k] = cmyk.map(|v| v.clamp(0.0, 1.0));
    // The product lies in 0..=255, so the cast is exact after rounding.
    let channel = |v: f64| (255.0 * (1.0 - v) * (1.0 - k)).round() as u8;
    [channel(c), channel(m), channel(y)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::draw::Shape;
    use crate::render::symbols::{Cap, Join};

    const RED: [u8; 4] = [255, 0, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];
    const WHITE: [u8; 4] = [255, 255, 255, 255];

    fn colours() -> Vec<RenderColour> {
        vec![
            RenderColour {
                priority: 1,
                cmyk: [0.0, 1.0, 1.0, 0.0],
            },
            RenderColour {
                priority: 2,
                cmyk: [1.0, 1.0, 0.0, 0.0],
            },
        ]
    }

    fn clip(width: f64, height: f64) -> PaperBox {
        PaperBox {
            min: (0.0, 0.0),
            max: (width, height),
        }
    }

    fn square(priority: i32, from: f64, to: f64) -> Primitive {
        Primitive {
            priority,
            shape: Shape::Fill {
                rings: vec![vec![(from, from), (to, from), (to, to), (from, to)]],
            },
        }
    }

    fn pixel(image: &image::RgbaImage, x: u32, y: u32) -> [u8; 4] {
        image.get_pixel(x, y).0
    }

    #[test]
    fn a_fill_is_painted_on_white() {
        let image = rasterize(&[square(1, 0.0, 10.0)], clip(20.0, 20.0), &colours(), 4.0);
        assert_eq!(image.dimensions(), (80, 80));
        assert_eq!(pixel(&image, 20, 20), RED);
        assert_eq!(pixel(&image, 70, 70), WHITE);
    }

    #[test]
    fn the_image_starts_at_the_clip_corner() {
        let shifted = PaperBox {
            min: (100.0, 200.0),
            max: (120.0, 220.0),
        };
        let corner = Primitive {
            priority: 1,
            shape: Shape::Fill {
                rings: vec![vec![
                    (110.0, 210.0),
                    (120.0, 210.0),
                    (120.0, 220.0),
                    (110.0, 220.0),
                ]],
            },
        };
        let image = rasterize(&[corner], shifted, &colours(), 4.0);
        assert_eq!(pixel(&image, 20, 20), WHITE, "top left of the clip");
        assert_eq!(
            pixel(&image, 60, 60),
            RED,
            "bottom right, y down as on paper"
        );
    }

    #[test]
    fn rings_are_filled_even_odd() {
        let ring = Primitive {
            priority: 1,
            shape: Shape::Fill {
                rings: vec![
                    vec![(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)],
                    vec![(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0)],
                ],
            },
        };
        let image = rasterize(&[ring], clip(20.0, 20.0), &colours(), 4.0);
        assert_eq!(pixel(&image, 8, 8), RED);
        assert_eq!(pixel(&image, 40, 40), WHITE, "the hole");
    }

    #[test]
    fn the_lower_priority_number_is_on_top() {
        // Priority 1 first in the list, 2 after it: the PDF still paints 2 first.
        let image = rasterize(
            &[square(1, 0.0, 15.0), square(2, 5.0, 20.0)],
            clip(20.0, 20.0),
            &colours(),
            4.0,
        );
        assert_eq!(pixel(&image, 40, 40), RED, "overlap");
        assert_eq!(pixel(&image, 70, 70), BLUE);
    }

    #[test]
    fn a_stroke_has_its_width() {
        let line = Primitive {
            priority: 1,
            shape: Shape::Stroke {
                path: vec![(0.0, 10.0), (20.0, 10.0)],
                closed: false,
                width: 1.0,
                cap: Cap::Flat,
                join: Join::Miter,
            },
        };
        let image = rasterize(&[line], clip(20.0, 20.0), &colours(), 4.0);
        assert_eq!(pixel(&image, 40, 39), RED, "on the line");
        assert_eq!(pixel(&image, 40, 40), RED, "on the line");
        assert_eq!(pixel(&image, 40, 48), WHITE, "2 mm away");
    }

    #[test]
    fn texts_are_skipped() {
        let text = Primitive {
            priority: 1,
            shape: Shape::Text {
                origin: (0.0, 10.0),
                size: 10.0,
                bold: true,
                text: "████".into(),
            },
        };
        let image = rasterize(&[text], clip(20.0, 20.0), &colours(), 4.0);
        assert!(image.pixels().all(|p| p.0 == WHITE));
    }

    #[test]
    fn a_colour_not_in_the_set_is_black() {
        let image = rasterize(&[square(7, 0.0, 10.0)], clip(20.0, 20.0), &colours(), 4.0);
        assert_eq!(pixel(&image, 20, 20), [0, 0, 0, 255]);
    }

    #[test]
    fn pixels_per_mm_follow_the_long_side() {
        let wide = preview_px_per_mm(&clip(300.0, 200.0));
        assert!((wide - 2048.0 / 300.0).abs() < 1e-9, "{wide}");
        assert!((wide - 6.826).abs() < 0.001, "{wide}");
        assert_eq!(preview_px_per_mm(&clip(100.0, 50.0)), 8.0);
        assert!((preview_px_per_mm(&clip(50.0, 400.0)) - 5.12).abs() < 1e-9);
    }

    #[test]
    fn cmyk_becomes_srgb_as_in_freemapper_render() {
        assert_eq!(srgb([0.0, 0.0, 0.0, 1.0]), [0, 0, 0]);
        assert_eq!(srgb([0.0, 0.0, 0.0, 0.0]), [255, 255, 255]);
        assert_eq!(srgb([0.0, 0.27, 0.79, 0.0]), [255, 186, 54]);
        assert_eq!(srgb([0.0, 0.0, 0.0, 0.5]), [128, 128, 128]);
        assert_eq!(srgb([2.0, -1.0, 0.0, 0.0]), [0, 255, 255], "clamped");
    }
}
