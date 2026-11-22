// SPDX-License-Identifier: AGPL-3.0-or-later

//! Drawing primitives as a one-page PDF: DeviceCMYK colours, painted by colour priority, texts in
//! the font Arimo, embedded as a subset by freemapper-core (`freemapper_render::fonts`, core ADR
//! 0045), so letters beyond WinAnsi such as ł, č or ő are shown. Paths go through the compact path
//! writer of freemapper-core (`freemapper_render::pdf_path`, core ADR 0033).

use freemapper_render::fonts::{PageFonts, face};
use freemapper_render::pdf::PdfError;
use freemapper_render::pdf_path::{Paint, write_path};
use pdf_writer::types::{LineCapStyle, LineJoinStyle};
use pdf_writer::{Content, Filter, Finish, Pdf, Rect, Ref, Str, TextStr};

use super::draw::{Primitive, Shape};
use super::geometry::Pt;
use super::symbols::{Cap, Join, RenderColour};

/// Points per millimetre.
const PT_PER_MM: f64 = 72.0 / 25.4;
/// Margin around everything drawn (mm).
const MARGIN_MM: f64 = 5.0;
/// Page sizes tried, smallest first (mm, portrait).
const PAGES: [(f64, f64); 2] = [(210.0, 297.0), (297.0, 420.0)];
/// OOM's miter limit for PDF (reference §2.1); the raster preview uses it too.
pub(super) const MITER_LIMIT: f32 = 2.236_068;

/// An axis-parallel rectangle on paper (mm, y down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaperBox {
    pub min: Pt,
    pub max: Pt,
}

/// The page (mm) for content of `width` × `height` mm plus margin: A4 or A3 in either
/// orientation, the smallest that fits, else exactly the content size.
pub fn page_size(width: f64, height: f64) -> (f64, f64) {
    let (w, h) = (width + 2.0 * MARGIN_MM, height + 2.0 * MARGIN_MM);
    PAGES
        .iter()
        .flat_map(|&(a, b)| [(a, b), (b, a)])
        .find(|&(pw, ph)| w <= pw && h <= ph)
        .unwrap_or((w, h))
}

/// The box around everything a primitive paints (mm), widened by half a stroke.
fn bounds(p: &Primitive) -> Option<PaperBox> {
    let (points, extra): (Vec<Pt>, f64) = match &p.shape {
        Shape::Fill { rings } => (rings.iter().flatten().copied().collect(), 0.0),
        Shape::Stroke { path, width, .. } => (path.clone(), width / 2.0),
        Shape::Text {
            origin,
            size,
            bold,
            text,
        } => {
            let w = super::metrics::text_width(text, *bold) * size;
            (
                vec![
                    (origin.0, origin.1 - size),
                    (origin.0 + w, origin.1 + size / 3.0),
                ],
                0.0,
            )
        }
    };
    let first = *points.first()?;
    let (min, max) = points.iter().fold((first, first), |(min, max), q| {
        (
            (min.0.min(q.0), min.1.min(q.1)),
            (max.0.max(q.0), max.1.max(q.1)),
        )
    });
    Some(PaperBox {
        min: (min.0 - extra, min.1 - extra),
        max: (max.0 + extra, max.1 + extra),
    })
}

fn union(a: PaperBox, b: PaperBox) -> PaperBox {
    PaperBox {
        min: (a.min.0.min(b.min.0), a.min.1.min(b.min.1)),
        max: (a.max.0.max(b.max.0), a.max.1.max(b.max.1)),
    }
}

/// Paper mm (y down) → page points (y up).
struct Transform {
    offset: Pt,
    page_height: f64,
}

impl Transform {
    fn pt(&self, p: Pt) -> (f32, f32) {
        (
            ((p.0 + self.offset.0) * PT_PER_MM) as f32,
            ((self.page_height - (p.1 + self.offset.1)) * PT_PER_MM) as f32,
        )
    }
}

/// Writes the rings of one fill or stroke with core's compact path writer (core ADR 0033): numbers
/// rounded to 1/1000 pt, a small shape relative to its first point inside `q … Q`, then `paint`.
/// A primitive with a coordinate that is not finite is left out; `write_pdf` has no error path and
/// a PDF number cannot hold it (before, `NaN` went into the content stream and broke the page).
fn path(content: &mut Content, t: &Transform, rings: &[&[Pt]], closed: bool, paint: Paint) {
    let rings: Vec<Vec<(f32, f32)>> = rings
        .iter()
        .map(|ring| ring.iter().map(|&p| t.pt(p)).collect())
        .collect();
    // Err(NotFinite) wrote nothing: the primitive is skipped on purpose.
    let _ = write_path(content, &rings, closed, paint);
}

fn paint(
    content: &mut Content,
    fonts: &mut PageFonts,
    t: &Transform,
    p: &Primitive,
    colours: &[RenderColour],
) {
    let [c, m, y, k] = colours
        .iter()
        .find(|c| c.priority == p.priority)
        .map_or([0.0, 0.0, 0.0, 1.0], |c| c.cmyk)
        .map(|v| v as f32);
    match &p.shape {
        Shape::Fill { rings } => {
            content.set_fill_cmyk(c, m, y, k);
            let rings: Vec<&[Pt]> = rings.iter().map(Vec::as_slice).collect();
            path(content, t, &rings, true, Paint::FillEvenOdd);
        }
        Shape::Stroke {
            path: points,
            closed,
            width,
            cap,
            join,
        } => {
            content.set_stroke_cmyk(c, m, y, k);
            content.set_line_width((width * PT_PER_MM) as f32);
            content.set_line_cap(match cap {
                Cap::Flat => LineCapStyle::ButtCap,
                Cap::Round => LineCapStyle::RoundCap,
                Cap::Square => LineCapStyle::ProjectingSquareCap,
            });
            content.set_line_join(match join {
                Join::Miter => LineJoinStyle::MiterJoin,
                Join::Round => LineJoinStyle::RoundJoin,
                Join::Bevel => LineJoinStyle::BevelJoin,
            });
            content.set_miter_limit(MITER_LIMIT);
            path(content, t, &[points], *closed, Paint::Stroke);
        }
        Shape::Text {
            origin,
            size,
            bold,
            text,
        } => {
            let (x, y0) = t.pt(*origin);
            let font = face(*bold, false);
            let bytes = fonts.encode(font, text);
            content.set_fill_cmyk(c, m, y, k);
            content.begin_text();
            content.set_font(PageFonts::resource_name(font), (size * PT_PER_MM) as f32);
            content.set_text_matrix([1.0, 0.0, 0.0, 1.0, x, y0]);
            content.show(Str(&bytes));
            content.end_text();
        }
    }
}

/// The PDF of `map` primitives (clipped to `clip`) and `sheet` primitives (not clipped). It fails
/// only when the subsets of the bundled font cannot be written, which is a bug.
pub fn write_pdf(
    map: &[Primitive],
    sheet: &[Primitive],
    clip: PaperBox,
    colours: &[RenderColour],
    title: &str,
) -> Result<Vec<u8>, PdfError> {
    let content_box = sheet.iter().filter_map(bounds).fold(clip, union);
    let (width, height) = (
        content_box.max.0 - content_box.min.0,
        content_box.max.1 - content_box.min.1,
    );
    let (page_w, page_h) = page_size(width, height);
    let t = Transform {
        offset: (
            (page_w - width) / 2.0 - content_box.min.0,
            (page_h - height) / 2.0 - content_box.min.1,
        ),
        page_height: page_h,
    };

    // Highest priority first: priority 0 is painted last, on top. Stable, so objects keep their
    // order within a colour.
    let mut items: Vec<(&Primitive, bool)> = map
        .iter()
        .map(|p| (p, true))
        .chain(sheet.iter().map(|p| (p, false)))
        .collect();
    items.sort_by_key(|(p, _)| std::cmp::Reverse(p.priority));

    let mut content = Content::new();
    let mut fonts = PageFonts::new();
    let (cx0, cy0) = t.pt((clip.min.0, clip.max.1));
    let (cx1, cy1) = t.pt((clip.max.0, clip.min.1));
    // One clip block for each run of consecutive map primitives.
    let mut clipping = false;
    for (primitive, clipped) in items {
        if clipped && !clipping {
            content.save_state();
            content.rect(cx0, cy0, cx1 - cx0, cy1 - cy0);
            content.clip_nonzero();
            content.end_path();
        } else if !clipped && clipping {
            content.restore_state();
        }
        clipping = clipped;
        paint(&mut content, &mut fonts, &t, primitive, colours);
    }
    if clipping {
        content.restore_state();
    }

    let (catalog, pages, page, contents, info) = (
        Ref::new(1),
        Ref::new(2),
        Ref::new(3),
        Ref::new(4),
        Ref::new(5),
    );
    let mut next = Ref::new(6);
    let font_objects = fonts.finish(&mut next);
    let mut pdf = Pdf::new();
    pdf.catalog(catalog).pages(pages);
    pdf.pages(pages).kids([page]).count(1);
    let mut p = pdf.page(page);
    p.media_box(Rect::new(
        0.0,
        0.0,
        (page_w * PT_PER_MM) as f32,
        (page_h * PT_PER_MM) as f32,
    ));
    p.parent(pages);
    p.contents(contents);
    let mut resources = p.resources();
    let mut font_dict = resources.fonts();
    for (name, font) in font_objects.resources() {
        font_dict.pair(name, font);
    }
    font_dict.finish();
    resources.finish();
    p.finish();
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&content.finish(), 6);
    pdf.stream(contents, &compressed)
        .filter(Filter::FlateDecode);
    pdf.document_info(info)
        .title(TextStr(title))
        .creator(TextStr(
            "freemapper – https://github.com/HenryJobst/freemapper",
        ));
    // After the content stream: the first stream of the file stays the page content.
    font_objects.write(&mut pdf)?;
    Ok(pdf.finish())
}

/// The text a PDF written by [`write_pdf`] shows, one line for each shown string (for tests). The
/// glyph ids of the content are read back to characters through the `ToUnicode` table of the font
/// each string is set in.
#[cfg(test)]
pub(crate) fn shown_text(pdf: &[u8]) -> String {
    use std::collections::HashMap;

    fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }
    /// The number that follows `marker` in `haystack`, after spaces.
    fn number_after(haystack: &[u8], marker: &[u8]) -> Option<u32> {
        let rest = &haystack[find(haystack, marker)? + marker.len()..];
        let digits: Vec<u8> = rest
            .iter()
            .skip_while(|b| **b == b' ' || **b == b'\n')
            .take_while(|b| b.is_ascii_digit())
            .copied()
            .collect();
        String::from_utf8(digits).ok()?.parse().ok()
    }
    /// The body of the object `number` (between `obj` and `endobj`).
    fn object(pdf: &[u8], number: u32) -> &[u8] {
        let head = format!("\n{number} 0 obj");
        let from = find(pdf, head.as_bytes()).expect("object") + head.len();
        let len = find(&pdf[from..], b"endobj").expect("end of object");
        &pdf[from..from + len]
    }
    /// glyph id → character of the font the resource `name` (with its `/`) stands for.
    fn table(pdf: &[u8], name: &[u8]) -> HashMap<u16, char> {
        let font = number_after(pdf, &[name, b" "].concat()).expect("font resource");
        let to_unicode =
            number_after(object(pdf, font), b"/ToUnicode").expect("ToUnicode of the font");
        let body = object(pdf, to_unicode);
        let start = find(body, b"stream\n").expect("stream") + 7;
        let end = find(body, b"\nendstream").expect("end of stream");
        let cmap =
            miniz_oxide::inflate::decompress_to_vec_zlib(&body[start..end]).expect("FlateDecode");
        let text = String::from_utf8_lossy(&cmap);
        let section = text
            .split("beginbfchar")
            .nth(1)
            .and_then(|s| s.split("endbfchar").next())
            .unwrap_or("");
        let hex: Vec<u32> = section
            .split(['<', '>'])
            .filter_map(|t| u32::from_str_radix(t.trim(), 16).ok())
            .collect();
        hex.chunks(2)
            .filter_map(|pair| Some((u16::try_from(pair[0]).ok()?, char::from_u32(*pair.get(1)?)?)))
            .collect()
    }
    /// The bytes of a string operand, literal or hexadecimal.
    fn string_bytes(operand: &[u8]) -> Vec<u8> {
        if operand.first() == Some(&b'<') {
            let digits: Vec<u8> = operand[1..operand.len() - 1].to_vec();
            return digits
                .chunks(2)
                .map(|d| u8::from_str_radix(std::str::from_utf8(d).unwrap_or("0"), 16).unwrap_or(0))
                .collect();
        }
        let inner = &operand[1..operand.len() - 1];
        let mut out = Vec::new();
        let mut i = 0;
        while i < inner.len() {
            if inner[i] != b'\\' {
                out.push(inner[i]);
                i += 1;
                continue;
            }
            i += 1;
            match inner[i] {
                b'n' => out.push(b'\n'),
                b'r' => out.push(b'\r'),
                b't' => out.push(b'\t'),
                b'b' => out.push(8),
                b'f' => out.push(12),
                d if d.is_ascii_digit() => {
                    let mut value = 0u32;
                    let mut taken = 0;
                    while taken < 3 && i < inner.len() && inner[i].is_ascii_digit() {
                        value = value * 8 + u32::from(inner[i] - b'0');
                        i += 1;
                        taken += 1;
                    }
                    out.push(value as u8);
                    continue;
                }
                other => out.push(other),
            }
            i += 1;
        }
        out
    }

    let content = content_of(pdf);
    let mut tables: HashMap<Vec<u8>, HashMap<u16, char>> = HashMap::new();
    let mut font: Vec<u8> = Vec::new();
    let mut shown = String::new();
    for line in content.split(|b| *b == b'\n') {
        if let Some(rest) = line.strip_suffix(b" Tf") {
            font = rest.split(|b| *b == b' ').next().unwrap_or(&[]).to_vec();
        } else if let Some(operand) = line.strip_suffix(b" Tj") {
            let glyphs = tables
                .entry(font.clone())
                .or_insert_with(|| table(pdf, &font));
            for pair in string_bytes(operand).chunks(2) {
                let id = u16::from_be_bytes([pair[0], *pair.get(1).unwrap_or(&0)]);
                shown.push(glyphs.get(&id).copied().unwrap_or('\u{FFFD}'));
            }
            shown.push('\n');
        }
    }
    shown
}

/// The decompressed page content of a PDF written by [`write_pdf`] (for tests).
#[cfg(test)]
pub(crate) fn content_of(pdf: &[u8]) -> Vec<u8> {
    let find = |needle: &[u8]| pdf.windows(needle.len()).position(|w| w == needle);
    let start = find(b"stream\n").expect("content stream") + 7;
    let end = find(b"\nendstream").expect("end of stream");
    miniz_oxide::inflate::decompress_to_vec_zlib(&pdf[start..end]).expect("FlateDecode")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`super::write_pdf`] for a test: the bundled font is always written.
    fn write_pdf(
        map: &[Primitive],
        sheet: &[Primitive],
        clip: PaperBox,
        colours: &[RenderColour],
        title: &str,
    ) -> Vec<u8> {
        super::write_pdf(map, sheet, clip, colours, title).expect("the PDF is written")
    }

    fn colours() -> Vec<RenderColour> {
        vec![
            RenderColour {
                priority: 2,
                cmyk: [0.0, 0.0, 0.0, 1.0],
            },
            RenderColour {
                priority: 9,
                cmyk: [0.0, 0.0, 0.0, 0.6],
            },
        ]
    }

    fn square(priority: i32) -> Primitive {
        Primitive {
            priority,
            shape: Shape::Fill {
                rings: vec![vec![(10.0, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)]],
            },
        }
    }

    fn clip() -> PaperBox {
        PaperBox {
            min: (0.0, 0.0),
            max: (100.0, 100.0),
        }
    }

    fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }

    #[test]
    fn page_size_is_a4_or_a3() {
        assert_eq!(page_size(150.0, 200.0), (210.0, 297.0));
        assert_eq!(page_size(250.0, 150.0), (297.0, 210.0));
        assert_eq!(page_size(350.0, 250.0), (420.0, 297.0));
        assert_eq!(page_size(500.0, 100.0), (510.0, 110.0));
    }

    #[test]
    fn primitives_are_painted_by_priority() {
        let pdf = content_of(&write_pdf(
            &[square(2), square(9)],
            &[],
            clip(),
            &colours(),
            "Test",
        ));
        let black = find(&pdf, b"0 0 0 1 k").expect("black fill");
        let grey = find(&pdf, b"0 0 0 0.6 k").expect("grey fill");
        assert!(
            grey < black,
            "priority 9 is painted before 2, so 2 is on top"
        );
        assert!(find(&pdf, b"f*").is_some(), "even-odd fill");
        assert!(
            find(&pdf, b"re\nW\nn").is_some(),
            "map primitives are clipped"
        );
    }

    #[test]
    fn strokes_carry_width_caps_and_miter_limit() {
        let line = Primitive {
            priority: 2,
            shape: Shape::Stroke {
                path: vec![(0.0, 0.0), (10.0, 0.0)],
                closed: false,
                width: 0.35,
                cap: Cap::Round,
                join: Join::Miter,
            },
        };
        let pdf = content_of(&write_pdf(&[line], &[], clip(), &colours(), "Test"));
        assert!(find(&pdf, b"0 0 0 1 K").is_some(), "stroke colour");
        assert!(find(&pdf, b"1 J").is_some(), "round cap");
        assert!(find(&pdf, b"2.236068 M").is_some(), "miter limit");
        assert!(find(&pdf, b"0.992126 w").is_some(), "0.35 mm in points");
    }

    #[test]
    fn text_is_embedded_arimo_with_the_letters_winansi_lacks() {
        let text = |size: f64, bold: bool, text: &str| Primitive {
            priority: 2,
            shape: Shape::Text {
                origin: (10.0, 10.0),
                size,
                bold,
                text: text.into(),
            },
        };
        let pdf = write_pdf(
            &[],
            &[
                text(2.5, true, "Słubice ©"),
                text(2.0, false, "Říčany Šiauliai"),
            ],
            clip(),
            &colours(),
            "Test",
        );
        assert!(find(&pdf, b"/Identity-H").is_some(), "Type0 font");
        assert!(find(&pdf, b"Arimo").is_some(), "Arimo is embedded");
        assert!(find(&pdf, b"/Helvetica").is_none(), "no standard font");
        assert!(find(&pdf, b"/WinAnsiEncoding").is_none());
        // Read back through the ToUnicode table: the letters themselves are shown.
        let shown = shown_text(&pdf);
        assert!(shown.contains("Słubice ©"), "{shown:?}");
        assert!(shown.contains("Říčany Šiauliai"), "{shown:?}");
    }

    #[test]
    fn content_is_compressed_and_clip_blocks_are_shared() {
        let many: Vec<Primitive> = (0..200).map(|_| square(9)).collect();
        let pdf = write_pdf(&many, &[], clip(), &colours(), "Test");
        assert!(find(&pdf, b"/FlateDecode").is_some());
        let start = find(&pdf, b"stream\n").unwrap() + 7;
        let end = find(&pdf, b"\nendstream").unwrap();
        let content = miniz_oxide::inflate::decompress_to_vec_zlib(&pdf[start..end]).unwrap();
        let clips = content.windows(5).filter(|w| w == b"re\nW\n").count();
        assert_eq!(clips, 1, "one clip block for a run of map primitives");
        assert!(find(&content, b"0 0 0 0.6 k").is_some());
    }

    fn fill(priority: i32, ring: Vec<Pt>) -> Primitive {
        Primitive {
            priority,
            shape: Shape::Fill { rings: vec![ring] },
        }
    }

    /// The text between the first `start` and the next `end` after it.
    fn between<'a>(haystack: &'a [u8], start: &[u8], end: &[u8]) -> Option<&'a [u8]> {
        let from = find(haystack, start)?;
        let to = from + find(&haystack[from..], end)? + end.len();
        Some(&haystack[from..to])
    }

    #[test]
    fn path_numbers_have_at_most_three_decimals() {
        // Odd coordinates, one large ring and one small, filled and stroked.
        let ring = vec![
            (10.123_457, 13.765_432),
            (61.987_654, 14.111_111),
            (59.333_333, 70.777_777),
        ];
        let dot = vec![
            (30.123_457, 40.765_432),
            (31.111_111, 40.765_432),
            (31.111_111, 41.333_333),
            (30.123_457, 41.333_333),
        ];
        let line = Primitive {
            priority: 2,
            shape: Shape::Stroke {
                path: ring.clone(),
                closed: false,
                width: 0.35,
                cap: Cap::Flat,
                join: Join::Miter,
            },
        };
        let pdf = content_of(&write_pdf(
            &[fill(9, ring), fill(2, dot), line],
            &[],
            clip(),
            &colours(),
            "Test",
        ));
        let text = String::from_utf8(pdf).expect("ASCII content");
        let path_lines: Vec<&str> = text
            .lines()
            .filter(|l| l.ends_with(" m") || l.ends_with(" l") || l.ends_with(" cm"))
            .collect();
        assert!(path_lines.len() >= 10, "{text}");
        for line in path_lines {
            for number in line.split(' ').filter(|n| n.parse::<f32>().is_ok()) {
                let decimals = number.split_once('.').map_or(0, |(_, d)| d.len());
                assert!(decimals <= 3, "{number} in {line:?}");
            }
        }
    }

    #[test]
    fn small_shapes_are_written_relative_with_their_paint_inside() {
        // A dot of 1 mm, four points: filled, and stroked as a closed ring.
        let dot = vec![(30.0, 40.0), (31.0, 40.0), (31.0, 41.0), (30.0, 41.0)];
        let ring = Primitive {
            priority: 9,
            shape: Shape::Stroke {
                path: dot.clone(),
                closed: true,
                width: 0.1,
                cap: Cap::Flat,
                join: Join::Miter,
            },
        };
        let pdf = content_of(&write_pdf(
            &[fill(2, dot), ring],
            &[],
            clip(),
            &colours(),
            "Test",
        ));
        let blocks: Vec<&[u8]> = [&b"0 0 0 0.6 K\n"[..], &b"0 0 0 1 k\n"[..]]
            .iter()
            .map(|colour| {
                let after = &pdf[find(&pdf, colour).expect("colour")..];
                between(after, b"q\n1 0 0 1 ", b"\nQ").expect("a relative block")
            })
            .collect();
        let (stroke, fill) = (blocks[0], blocks[1]);
        for (block, paint) in [(stroke, &b"h\nS\nQ"[..]), (fill, &b"h\nf*\nQ"[..])] {
            assert!(find(block, b" cm\n0 0 m\n").is_some(), "{block:?}");
            assert!(
                block.ends_with(paint),
                "{:?}",
                String::from_utf8_lossy(block)
            );
            // 1 mm right is 2.835 pt; 1 mm down on paper is 2.835 pt down on the page.
            assert!(find(block, b"\n2.835 0 l\n2.835 -2.835 l\n0 -2.835 l\n").is_some());
        }
    }

    #[test]
    fn large_shapes_are_written_absolute() {
        let pdf = content_of(&write_pdf(&[square(9)], &[], clip(), &colours(), "Test"));
        assert!(find(&pdf, b" cm").is_none(), "no translation");
        // (10, 10) mm on the A4 page, 55 mm and 98.5 mm in from its edges: 65 mm right, 188.5 mm up.
        assert!(find(&pdf, b"184.252 534.331 m\n").is_some());
        assert!(find(&pdf, b"h\nf*\n").is_some());
    }

    #[test]
    fn clip_paths_are_never_relative() {
        // A clip box of 3 mm, smaller than the 7 mm of a small shape.
        let tiny = PaperBox {
            min: (30.0, 40.0),
            max: (33.0, 43.0),
        };
        let pdf = content_of(&write_pdf(&[square(9)], &[], tiny, &colours(), "Test"));
        let clip_end = find(&pdf, b"re\nW\nn\n").expect("clip") + 7;
        assert!(pdf.starts_with(b"q\n"), "the clip's own graphics state");
        assert!(
            find(&pdf[..clip_end], b" cm").is_none(),
            "clip not translated"
        );
    }

    #[test]
    fn primitives_with_coordinates_that_are_not_finite_are_left_out() {
        let broken = fill(
            9,
            vec![(f64::NAN, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)],
        );
        let pdf = content_of(&write_pdf(
            &[broken, square(2)],
            &[],
            clip(),
            &colours(),
            "Test",
        ));
        let text = String::from_utf8_lossy(&pdf);
        assert!(!text.to_lowercase().contains("nan"), "{text}");
        assert_eq!(text.matches("f*").count(), 1, "only the finite square");
    }

    #[test]
    fn pdf_structure_is_valid() {
        let pdf = write_pdf(&[square(9)], &[], clip(), &colours(), "Strausberg Vorstadt");
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(find(&pdf, b"xref").is_some() && find(&pdf, b"trailer").is_some());
        assert!(pdf.ends_with(b"%%EOF") || pdf.ends_with(b"%%EOF\n"));
        assert!(find(&pdf, b"(Strausberg Vorstadt)").is_some(), "title");
        // UTF-16BE (the en dash is not ASCII): "HenryJobst".
        assert!(
            find(&pdf, b"00480065006E00720079004A006F00620073007400").is_some(),
            "creator"
        );
        assert!(
            find(&pdf, b"/MediaBox [0 0 595.2756 841.8898]").is_some(),
            "A4"
        );
    }
}
