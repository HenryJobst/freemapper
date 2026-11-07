//! Map objects → drawing primitives tagged with a colour priority (`docs/render/oom-render-reference.md`).
//!
//! Paper millimetres with y down.

use super::geometry::{self, Part, Pt};
use super::metrics::{ASCENT, LINE_SPACING, text_width};
use super::symbols::{
    Cap, Join, LineDef, Part as SymbolPart, PointDef, RenderSymbols, Symbol, TextDef,
};
use crate::omap::{MapCoord, MapObject, MapObjectKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Even-odd fill of closed rings.
    Fill { rings: Vec<Vec<Pt>> },
    Stroke {
        path: Vec<Pt>,
        closed: bool,
        width: f64,
        cap: Cap,
        join: Join,
    },
    /// One line of text; `origin` is the start of the baseline.
    Text {
        origin: Pt,
        size: f64,
        bold: bool,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Primitive {
    /// Colour priority (= colour index); 0 is painted last, on top.
    pub priority: i32,
    pub shape: Shape,
}

/// Segments of a circle drawn as a polygon.
const CIRCLE_SEGMENTS: usize = 48;

fn circle(centre: Pt, radius: f64) -> Vec<Pt> {
    (0..=CIRCLE_SEGMENTS)
        .map(|i| {
            let a = i as f64 / CIRCLE_SEGMENTS as f64 * std::f64::consts::TAU;
            (centre.0 + radius * a.cos(), centre.1 + radius * a.sin())
        })
        .collect()
}

/// OOM 0.001 mm → mm.
fn mm(v: f64) -> f64 {
    v / 1000.0
}

/// The primitives of one map object; none when its symbol is unknown or its geometry unusable.
pub fn primitives(object: &MapObject, symbols: &RenderSymbols) -> Vec<Primitive> {
    let Some(symbol) = symbols.symbols.get(&object.symbol_id) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    match &object.kind {
        MapObjectKind::Point { rotation_rad } => {
            if let (Symbol::Point(def), Some(c)) = (symbol, object.coords.first()) {
                let r = if def.rotatable { -rotation_rad } else { 0.0 };
                out = point_primitives(def, (mm(f64::from(c.x)), mm(f64::from(c.y))), r, symbols);
            }
        }
        MapObjectKind::Path => path_symbol(symbol, &object.coords, symbols, &mut out),
        MapObjectKind::Text {
            text,
            h_align,
            v_align,
        } => {
            if let (Symbol::Text(def), Some(c)) = (symbol, object.coords.first()) {
                text_primitives(
                    def,
                    text,
                    *h_align,
                    *v_align,
                    (mm(f64::from(c.x)), mm(f64::from(c.y))),
                    &mut out,
                );
            }
        }
    }
    out
}

/// A symbol drawn on path coordinates (map objects and point symbol elements).
fn path_symbol(
    symbol: &Symbol,
    coords: &[MapCoord],
    symbols: &RenderSymbols,
    out: &mut Vec<Primitive>,
) {
    match symbol {
        Symbol::Area { color } => {
            let parts = geometry::parts(coords);
            if *color >= 0 && parts.first().is_some_and(|p| p.points.len() > 2) {
                out.push(Primitive {
                    priority: *color,
                    shape: Shape::Fill {
                        rings: parts
                            .into_iter()
                            .map(|p| p.points)
                            .filter(|r| r.len() > 2)
                            .collect(),
                    },
                });
            }
        }
        Symbol::Line(def) => line(def, coords, symbols, out),
        Symbol::Combined(parts) => {
            for part in parts {
                let part_symbol = match part {
                    SymbolPart::Ref(id) => symbols.symbols.get(id),
                    SymbolPart::Inline(symbol) => Some(&**symbol),
                };
                if let Some(part_symbol) = part_symbol {
                    path_symbol(part_symbol, coords, symbols, out);
                }
            }
        }
        // A point symbol element on a point object: drawn at its coordinate, unrotated.
        Symbol::Point(def) => {
            if let Some(c) = coords.first() {
                out.extend(point_primitives(
                    def,
                    (mm(f64::from(c.x)), mm(f64::from(c.y))),
                    0.0,
                    symbols,
                ));
            }
        }
        Symbol::Text(_) => {}
    }
}

fn stroke(
    color: i32,
    path: Vec<Pt>,
    closed: bool,
    width: f64,
    cap: Cap,
    join: Join,
) -> Option<Primitive> {
    (color >= 0 && width > 0.0 && path.len() >= 2).then_some(Primitive {
        priority: color,
        shape: Shape::Stroke {
            path,
            closed,
            width,
            cap,
            join,
        },
    })
}

fn line(def: &LineDef, coords: &[MapCoord], symbols: &RenderSymbols, out: &mut Vec<Primitive>) {
    let parts: Vec<Part> = geometry::parts(coords)
        .into_iter()
        .filter(|p| p.points.len() >= 2 && geometry::length(&p.points) > 0.0)
        .collect();
    let width = mm(def.width);
    for part in &parts {
        // Main line, whole or dashed.
        match &def.dash {
            Some(dash) => {
                let dashes = geometry::dashes(part, dash);
                let solid = dashes.len() == 1 && dashes[0] == part.points;
                for d in dashes {
                    out.extend(stroke(
                        def.color,
                        d,
                        solid && part.closed,
                        width,
                        def.cap,
                        def.join,
                    ));
                }
            }
            None => out.extend(stroke(
                def.color,
                part.points.clone(),
                part.closed,
                width,
                def.cap,
                def.join,
            )),
        }
        // Borders on both sides, flat caps, miter (round if the line is round).
        if let Some(border) = &def.border {
            let shift = width / 2.0 + mm(border.shift);
            let join = if def.join == Join::Round {
                Join::Round
            } else {
                Join::Miter
            };
            let pieces: Vec<(Vec<Pt>, bool)> = match &border.dash {
                Some(dash) => {
                    let dashes = geometry::dashes(part, dash);
                    let solid = dashes.len() == 1 && dashes[0] == part.points;
                    dashes
                        .into_iter()
                        .map(|d| (d, solid && part.closed))
                        .collect()
                }
                None => vec![(part.points.clone(), part.closed)],
            };
            for (piece, closed) in pieces {
                for side in [-1.0, 1.0] {
                    let path = geometry::offset(&piece, closed, side * shift);
                    out.extend(stroke(
                        border.color,
                        path,
                        closed,
                        mm(border.width),
                        Cap::Flat,
                        join,
                    ));
                }
            }
        }
        // Mid symbols, local +y to the right of travel.
        if let Some(mid) = &def.mid {
            for (at, angle) in geometry::mid_positions(part, mid) {
                let r = if mid.symbol.rotatable { angle } else { 0.0 };
                out.extend(point_primitives(&mid.symbol, at, r, symbols));
            }
        }
    }
    // Start symbol on the first part, end symbol on the last, along the direction of travel.
    if let (Some(start), Some(first)) = (&def.start, parts.first()) {
        let (at, angle) = geometry::at(first, 0.0);
        out.extend(point_primitives(
            start,
            at,
            if start.rotatable { angle } else { 0.0 },
            symbols,
        ));
    }
    if let (Some(end), Some(last)) = (&def.end, parts.last()) {
        let (at, angle) = geometry::at(last, geometry::length(&last.points));
        out.extend(point_primitives(
            end,
            at,
            if end.rotatable { angle } else { 0.0 },
            symbols,
        ));
    }
}

/// The primitives of a point symbol at `at` (mm), rotated by `rotation` (radians, y down).
pub fn point_primitives(
    def: &PointDef,
    at: Pt,
    rotation: f64,
    symbols: &RenderSymbols,
) -> Vec<Primitive> {
    let mut out = Vec::new();
    if let Some((color, radius)) = def.dot {
        out.push(Primitive {
            priority: color,
            shape: Shape::Fill {
                rings: vec![circle(at, mm(radius))],
            },
        });
    }
    if let Some((color, radius, width)) = def.ring {
        out.extend(stroke(
            color,
            circle(at, mm(radius)),
            true,
            mm(width),
            Cap::Flat,
            Join::Miter,
        ));
    }
    let (sin, cos) = rotation.sin_cos();
    for element in &def.elements {
        // Element coordinates rotated about the centre and moved there, back in 0.001 mm.
        let coords: Vec<MapCoord> = element
            .coords
            .iter()
            .map(|c| {
                let (ox, oy) = (f64::from(c.x), f64::from(c.y));
                MapCoord {
                    x: (ox * cos - oy * sin + at.0 * 1000.0).round() as i32,
                    y: (oy * cos + ox * sin + at.1 * 1000.0).round() as i32,
                    flags: c.flags,
                }
            })
            .collect();
        path_symbol(&element.symbol, &coords, symbols, &mut out);
    }
    out
}

/// One text primitive per line, laid out per reference §6 (no rotation: the sheet texts are not
/// rotatable).
fn text_primitives(
    def: &TextDef,
    text: &str,
    h_align: u8,
    v_align: u8,
    anchor: Pt,
    out: &mut Vec<Primitive>,
) {
    if def.color < 0 {
        return;
    }
    let em = mm(def.size);
    let ascent = ASCENT * em;
    let line_step = def.line_spacing * LINE_SPACING * em;
    let lines: Vec<&str> = text.split('\n').collect();
    let mut y = if v_align == 0 { 0.0 } else { ascent };
    if v_align == 2 || v_align == 3 {
        let height = ascent + (lines.len().saturating_sub(1)) as f64 * line_step;
        y -= if v_align == 2 { height / 2.0 } else { height };
    }
    for line in lines {
        let width = text_width(line, def.bold) * em;
        let x = match h_align {
            1 => -width / 2.0,
            2 => -width,
            _ => 0.0,
        };
        if !line.is_empty() {
            out.push(Primitive {
                priority: def.color,
                shape: Shape::Text {
                    origin: (anchor.0 + x, anchor.1 + y),
                    size: em,
                    bold: def.bold,
                    text: line.to_string(),
                },
            });
        }
        y += line_step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
    use crate::omap::{SymbolSet, closed_ring};

    fn set() -> (RenderSymbols, SymbolSet) {
        let set = SymbolSet::issprom_2019().with_sheet_symbols();
        (RenderSymbols::from_set(&set), set)
    }

    fn object(
        set: &SymbolSet,
        code: &str,
        kind: MapObjectKind,
        coords: Vec<MapCoord>,
    ) -> MapObject {
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind,
            coords,
            tags: Vec::new(),
        }
    }

    fn square(side: i32) -> Vec<MapCoord> {
        closed_ring(&[(0, 0), (side, 0), (side, side), (0, side)])
    }

    fn strokes(p: &[Primitive]) -> Vec<(i32, f64)> {
        p.iter()
            .filter_map(|p| match &p.shape {
                Shape::Stroke { width, .. } => {
                    Some((p.priority, (width * 1000.0).round() / 1000.0))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn paved_area_edge_lies_below_its_fill() {
        let (r, s) = set();
        let p = primitives(&object(&s, "501", MapObjectKind::Path, square(10_000)), &r);
        let fills: Vec<i32> = p
            .iter()
            .filter(|p| matches!(p.shape, Shape::Fill { .. }))
            .map(|p| p.priority)
            .collect();
        assert_eq!(fills, [14]);
        assert_eq!(strokes(&p), [(15, 0.2)]);
        let closed = p
            .iter()
            .any(|p| matches!(p.shape, Shape::Stroke { closed: true, .. }));
        assert!(closed, "the edge of an area is a closed line");
    }

    #[test]
    fn bush_is_dot_and_ring() {
        let (r, s) = set();
        let p = primitives(
            &object(
                &s,
                "418",
                MapObjectKind::Point { rotation_rad: 0.0 },
                vec![MapCoord::new(5_000, 5_000)],
            ),
            &r,
        );
        assert_eq!(p.len(), 2);
        let Shape::Fill { rings } = &p.iter().find(|p| p.priority == 21).unwrap().shape else {
            panic!()
        };
        let r_dot = rings[0]
            .iter()
            .map(|q| (q.0 - 5.0).hypot(q.1 - 5.0))
            .fold(0.0, f64::max);
        assert!((r_dot - 0.1).abs() < 1e-6);
        let Shape::Stroke {
            path,
            width,
            closed,
            ..
        } = &p.iter().find(|p| p.priority == 3).unwrap().shape
        else {
            panic!()
        };
        assert!(*closed && (*width - 0.3).abs() < 1e-9);
        assert!((((path[0].0 - 5.0).hypot(path[0].1 - 5.0)) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn fence_ticks_on_the_right() {
        let (r, s) = set();
        let fence = object(
            &s,
            "518",
            MapObjectKind::Path,
            vec![MapCoord::new(0, 0), MapCoord::new(20_000, 0)],
        );
        let p = primitives(&fence, &r);
        // Main line 0.4 mm and five ticks of two lines 0.14 mm each.
        let widths = strokes(&p);
        assert_eq!(widths.iter().filter(|w| w.1 == 0.4).count(), 1);
        assert_eq!(widths.iter().filter(|w| w.1 == 0.14).count(), 10);
        for prim in &p {
            if let Shape::Stroke { path, width, .. } = &prim.shape
                && (*width - 0.14).abs() < 1e-9
            {
                assert!(
                    path.iter().all(|q| q.1 >= -1e-9),
                    "right of travel (south): {path:?}"
                );
            }
        }
    }

    #[test]
    fn building_has_fill_and_outline() {
        let (r, s) = set();
        let mut coords = square(10_000);
        coords.extend(closed_ring(&[
            (2_000, 2_000),
            (4_000, 2_000),
            (4_000, 4_000),
            (2_000, 4_000),
        ]));
        let p = primitives(&object(&s, "521", MapObjectKind::Path, coords), &r);
        let Shape::Fill { rings } = &p.iter().find(|p| p.priority == 9).unwrap().shape else {
            panic!()
        };
        assert_eq!(rings.len(), 2, "outer ring and courtyard");
        assert_eq!(strokes(&p), [(2, 0.14), (2, 0.14)]);
        let _ = (FLAG_CLOSE, FLAG_HOLE);
    }

    #[test]
    fn text_alignment_uses_widths() {
        let (r, s) = set();
        let text = |h_align| {
            object(
                &s,
                "992",
                MapObjectKind::Text {
                    text: "Maßstab".into(),
                    h_align,
                    v_align: 0,
                },
                vec![MapCoord::new(50_000, 20_000)],
            )
        };
        let em = 2.5;
        let width = text_width("Maßstab", false) * em;
        for (h_align, x) in [(0, 50.0), (1, 50.0 - width / 2.0), (2, 50.0 - width)] {
            let p = primitives(&text(h_align), &r);
            let Shape::Text { origin, size, .. } = &p[0].shape else {
                panic!()
            };
            assert!(
                (origin.0 - x).abs() < 1e-9 && (origin.1 - 20.0).abs() < 1e-9,
                "{h_align}: {origin:?}"
            );
            assert_eq!(*size, em);
            assert_eq!(p[0].priority, 2);
        }
    }

    #[test]
    fn rotation_of_rotatable_points() {
        let (r, _) = set();
        let arrow = PointDef {
            rotatable: true,
            dot: None,
            ring: None,
            elements: vec![super::super::symbols::Element {
                symbol: Symbol::Line(LineDef {
                    color: 2,
                    width: 100.0,
                    cap: Cap::Flat,
                    join: Join::Miter,
                    dash: None,
                    border: None,
                    mid: None,
                    start: None,
                    end: None,
                }),
                coords: vec![MapCoord::new(0, 0), MapCoord::new(1_000, 0)],
            }],
        };
        // Rotation −π/2 (y down) turns local +x to paper up.
        let p = point_primitives(&arrow, (10.0, 10.0), -std::f64::consts::FRAC_PI_2, &r);
        let Shape::Stroke { path, .. } = &p[0].shape else {
            panic!()
        };
        assert!(
            (path[1].0 - 10.0).abs() < 1e-9 && (path[1].1 - 9.0).abs() < 1e-9,
            "{path:?}"
        );
    }
}
