//! Map objects → drawing primitives tagged with a colour priority (`docs/render/oom-render-reference.md`).
//!
//! Paper millimetres with y down.

use super::geometry::{self, Part, Pt};
use super::metrics::{ASCENT, LINE_SPACING, text_width};
use super::symbols::{
    Cap, Join, LineDef, Part as SymbolPart, PointDef, PointPattern, RenderSymbols, Symbol, TextDef,
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
                out = point_symbol(def, (mm(f64::from(c.x)), mm(f64::from(c.y))), r, symbols, 0);
            }
        }
        MapObjectKind::Path => path_symbol(symbol, &object.coords, symbols, &mut out, 0),
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
    // A colour missing from the colour table is not drawn (as in OOM).
    out.retain(|p| symbols.colour(p.priority).is_some());
    out
}

/// Whether `p` lies inside `rings` by the even-odd rule.
fn inside(rings: &[Vec<Pt>], p: Pt) -> bool {
    let mut inside = false;
    for ring in rings {
        let n = ring.len();
        for k in 0..n {
            let (a, b) = (ring[k], ring[(k + 1) % n]);
            if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
                inside = !inside;
            }
        }
    }
    inside
}

/// The points of `pattern` (rows through the map origin, as OOM places them without a pattern
/// origin of the object) whose centre lies in `rings` (mm). OOM clips the points at the outline;
/// a point whose centre lies outside is left out instead.
fn pattern_points(rings: &[Vec<Pt>], pattern: &PointPattern) -> Vec<Pt> {
    let (spacing, distance) = (mm(pattern.line_spacing), mm(pattern.point_distance));
    // Along the rows and across them; OOM's angle turns counter-clockwise on paper (y down).
    let (sin, cos) = pattern.angle.sin_cos();
    let (along, across) = ((cos, -sin), (sin, cos));
    let project = |p: Pt, d: Pt| p.0 * d.0 + p.1 * d.1;
    let (mut u, mut v) = ((f64::MAX, f64::MIN), (f64::MAX, f64::MIN));
    for &p in rings.iter().flatten() {
        let (pu, pv) = (project(p, along), project(p, across));
        u = (u.0.min(pu), u.1.max(pu));
        v = (v.0.min(pv), v.1.max(pv));
    }
    let (line_offset, point_offset) = (mm(pattern.line_offset), mm(pattern.offset_along_line));
    let mut points = Vec::new();
    let first_row = ((v.0 - line_offset) / spacing).floor() as i64;
    let last_row = ((v.1 - line_offset) / spacing).ceil() as i64;
    let first_point = ((u.0 - point_offset) / distance).floor() as i64;
    let last_point = ((u.1 - point_offset) / distance).ceil() as i64;
    for row in first_row..=last_row {
        let b = line_offset + row as f64 * spacing;
        for k in first_point..=last_point {
            let a = point_offset + k as f64 * distance;
            let p = (a * along.0 + b * across.0, a * along.1 + b * across.1);
            if inside(rings, p) {
                points.push(p);
            }
        }
    }
    points
}

/// Nesting depth of symbols (combined parts, point elements) beyond which drawing stops: a
/// symbol set that refers to itself must not overflow the stack.
const MAX_DEPTH: u32 = 8;

/// A symbol drawn on path coordinates (map objects and point symbol elements).
fn path_symbol(
    symbol: &Symbol,
    coords: &[MapCoord],
    symbols: &RenderSymbols,
    out: &mut Vec<Primitive>,
    depth: u32,
) {
    if depth > MAX_DEPTH {
        return;
    }
    match symbol {
        Symbol::Area { color, patterns } => {
            let parts = geometry::parts(coords);
            if !parts.first().is_some_and(|p| p.points.len() > 2) {
                return;
            }
            let rings: Vec<Vec<Pt>> = parts
                .into_iter()
                .map(|p| p.points)
                .filter(|r| r.len() > 2)
                .collect();
            for pattern in patterns {
                for at in pattern_points(&rings, pattern) {
                    out.extend(point_symbol(&pattern.point, at, 0.0, symbols, depth + 1));
                }
            }
            if *color >= 0 {
                out.push(Primitive {
                    priority: *color,
                    shape: Shape::Fill { rings },
                });
            }
        }
        Symbol::Line(def) => line(def, coords, symbols, out, depth),
        Symbol::Combined(parts) => {
            for part in parts {
                let part_symbol = match part {
                    SymbolPart::Ref(id) => symbols.symbols.get(id),
                    SymbolPart::Inline(symbol) => Some(&**symbol),
                };
                if let Some(part_symbol) = part_symbol {
                    path_symbol(part_symbol, coords, symbols, out, depth + 1);
                }
            }
        }
        // A point symbol element on a point object: drawn at its coordinate, unrotated.
        Symbol::Point(def) => {
            if let Some(c) = coords.first() {
                out.extend(point_symbol(
                    def,
                    (mm(f64::from(c.x)), mm(f64::from(c.y))),
                    0.0,
                    symbols,
                    depth + 1,
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

fn line(
    def: &LineDef,
    coords: &[MapCoord],
    symbols: &RenderSymbols,
    out: &mut Vec<Primitive>,
    depth: u32,
) {
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
                out.extend(point_symbol(&mid.symbol, at, r, symbols, depth + 1));
            }
        }
    }
    // Start symbol on the first part (outgoing direction), end symbol on the last (incoming
    // direction), also on closed lines (reference §2.6).
    if let (Some(start), Some(first)) = (&def.start, parts.first()) {
        let angle = if start.rotatable {
            geometry::start_angle(first)
        } else {
            0.0
        };
        out.extend(point_symbol(
            start,
            first.points[0],
            angle,
            symbols,
            depth + 1,
        ));
    }
    if let (Some(end), Some(last)) = (&def.end, parts.last()) {
        let angle = if end.rotatable {
            geometry::end_angle(last)
        } else {
            0.0
        };
        let at = *last.points.last().expect("parts have two points");
        out.extend(point_symbol(end, at, angle, symbols, depth + 1));
    }
}

/// The primitives of a point symbol at `at` (mm), rotated by `rotation` (radians, y down).
pub fn point_primitives(
    def: &PointDef,
    at: Pt,
    rotation: f64,
    symbols: &RenderSymbols,
) -> Vec<Primitive> {
    point_symbol(def, at, rotation, symbols, 0)
}

fn point_symbol(
    def: &PointDef,
    at: Pt,
    rotation: f64,
    symbols: &RenderSymbols,
    depth: u32,
) -> Vec<Primitive> {
    let mut out = Vec::new();
    if depth > MAX_DEPTH {
        return out;
    }
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
        path_symbol(&element.symbol, &coords, symbols, &mut out, depth + 1);
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
    fn training_pdf_paints_water_under_paved_areas_and_buildings() {
        let (r, s) = set();
        // Painted highest priority number first, so a larger number lies lower.
        let fill_priority = |code: &str| {
            let p = primitives(&object(&s, code, MapObjectKind::Path, square(10_000)), &r);
            p.iter()
                .filter(|p| matches!(p.shape, Shape::Fill { .. }))
                .map(|p| p.priority)
                .max()
                .unwrap_or_else(|| panic!("{code} has a fill"))
        };
        let water = fill_priority("301.1");
        assert!(water > fill_priority("501.5"), "paved areas cover water");
        assert!(water > fill_priority("521.3"), "buildings cover water");
        assert!(water < fill_priority("520"), "water covers out of bounds");
        let bank = primitives(
            &object(&s, "301.6", MapObjectKind::Path, square(10_000)),
            &r,
        );
        assert!(
            bank.iter().all(|p| p.priority < water),
            "the bank line is on top of the fill"
        );
    }

    #[test]
    fn sprint_paint_order_walkable_area_park_forest_then_paths() {
        let s = crate::project::MapKind::Sprint.symbol_set();
        let r = RenderSymbols::from_set(&s);
        // Painted highest priority number first, so a larger number lies lower.
        let lowest = |code: &str| {
            primitives(&object(&s, code, MapObjectKind::Path, square(10_000)), &r)
                .iter()
                .map(|p| p.priority)
                .max()
                .unwrap_or_else(|| panic!("{code} draws"))
        };
        let highest = |code: &str| {
            primitives(&object(&s, code, MapObjectKind::Path, square(10_000)), &r)
                .iter()
                .map(|p| p.priority)
                .min()
                .unwrap_or_else(|| panic!("{code} draws"))
        };
        let base = lowest("990");
        // The walkable area lies under every colour of the norm set.
        assert_eq!(base, 32);
        assert!(r.colours.iter().all(|c| c.priority <= base));
        assert_eq!(r.colour(base).map(|c| c.cmyk), r.colour(14).map(|c| c.cmyk));
        let park = lowest("401");
        let forest = lowest("405");
        assert!(base > park, "the park lies on the walkable area");
        assert!(park > highest("402"), "the 402 screen lies on the park");
        assert!(park > forest, "the forest lies on the park");
        for code in ["501.5", "505.1", "506", "521.3", "301.1"] {
            assert!(forest > highest(code), "{code} lies on the forest");
        }
        for code in ["520", "416", "417", "101"] {
            assert!(base > lowest(code), "{code} lies on the walkable area");
        }
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
        // The fill is black 50 % (priority 10), as in ISSprOM 2019-2.
        let Shape::Fill { rings } = &p.iter().find(|p| p.priority == 10).unwrap().shape else {
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

    fn closed_square(side: i32) -> Vec<MapCoord> {
        closed_ring(&[(0, 0), (side, 0), (side, side), (0, side)])
    }

    fn stroke_paths(p: &[Primitive], width: f64) -> Vec<(&Vec<Pt>, bool)> {
        p.iter()
            .filter_map(|p| match &p.shape {
                Shape::Stroke {
                    path,
                    closed,
                    width: w,
                    ..
                } if (w - width).abs() < 1e-9 => Some((path, *closed)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn fence_ring_does_not_wrap_around() {
        let (r, s) = set();
        // 80 mm ring: ticks keep e2 from the closing point (0, 0) on both sides.
        let p = primitives(
            &object(&s, "518", MapObjectKind::Path, closed_square(20_000)),
            &r,
        );
        let ticks = stroke_paths(&p, 0.14);
        assert_eq!(ticks.len() % 2, 0);
        let spots = ticks.len() / 2;
        assert_eq!(spots, 21, "n·3.75 + 4.65 closest to 80 mm");
        for (path, _) in &ticks {
            let d = path
                .iter()
                .map(|q| q.0.hypot(q.1))
                .fold(f64::INFINITY, f64::min);
            assert!(d > 1.5, "no tick at the closing point: {path:?}");
        }
        let main = stroke_paths(&p, 0.4);
        assert_eq!(main.len(), 1);
        assert!(main[0].1, "the ring itself is closed");
    }

    #[test]
    fn path_ring_borders_are_dashed_alike_on_both_sides() {
        let (r, s) = set();
        let p = primitives(
            &object(&s, "505.1", MapObjectKind::Path, closed_square(10_000)),
            &r,
        );
        let borders = stroke_paths(&p, 0.1);
        assert!(borders.len() > 4);
        assert_eq!(borders.len() % 2, 0, "one dash per side and position");
        assert!(
            borders.iter().all(|(_, closed)| !closed),
            "dashes are open strokes"
        );
    }

    #[test]
    fn short_path_ring_has_one_border_dash_per_side() {
        let (r, s) = set();
        // 2.4 mm around: two half dashes joined at the closing point and one gap (reference §2.2:
        // closed parts always have H = 2, so they are never drawn solid).
        let p = primitives(
            &object(&s, "505.1", MapObjectKind::Path, closed_square(600)),
            &r,
        );
        let borders = stroke_paths(&p, 0.1);
        assert_eq!(borders.len(), 2);
        assert!(borders.iter().all(|(_, closed)| !closed));
    }

    #[test]
    fn self_referencing_combined_symbol_stops() {
        let (mut r, s) = set();
        let id = s.id_for_code("521").unwrap();
        r.symbols
            .insert(id, Symbol::Combined(vec![SymbolPart::Ref(id)]));
        let p = primitives(&object(&s, "521", MapObjectKind::Path, square(10_000)), &r);
        assert!(p.is_empty());
    }

    #[test]
    fn end_bars_of_a_closed_line_follow_their_segments() {
        let set = SymbolSet::school_2019();
        let r = RenderSymbols::from_set(&set);
        let bench = object(&set, "543", MapObjectKind::Path, closed_square(5_000));
        let p = primitives(&bench, &r);
        // Bars along local y, 0.8 mm (enlarged 125 % in the school set? no: school additions are
        // final sizes): the start bar is perpendicular to the first segment (east), so vertical.
        let bars: Vec<&Vec<Pt>> = p
            .iter()
            .filter_map(|p| match &p.shape {
                Shape::Stroke { path, .. } if path.len() == 2 && geometry::length(path) < 1.0 => {
                    Some(path)
                }
                _ => None,
            })
            .collect();
        assert_eq!(bars.len(), 2, "{p:?}");
        let vertical = |b: &Vec<Pt>| (b[0].0 - b[1].0).abs() < 1e-6;
        let horizontal = |b: &Vec<Pt>| (b[0].1 - b[1].1).abs() < 1e-6;
        assert!(
            bars.iter().any(|b| vertical(b)),
            "start bar ⟂ east: {bars:?}"
        );
        assert!(
            bars.iter().any(|b| horizontal(b)),
            "end bar ⟂ north: {bars:?}"
        );
    }

    #[test]
    fn unknown_colours_are_left_out() {
        let (mut r, s) = set();
        let id = s.id_for_code("520").unwrap();
        r.symbols.insert(
            id,
            Symbol::Area {
                color: 99,
                patterns: Vec::new(),
            },
        );
        let p = primitives(&object(&s, "520", MapObjectKind::Path, square(10_000)), &r);
        assert!(p.is_empty());
    }

    #[test]
    fn scattered_trees_draw_white_dots_on_yellow() {
        let (r, s) = set();
        // 402 on a 10 × 10 mm square: yellow fill and white dots 0.75 mm apart.
        let p = primitives(&object(&s, "402", MapObjectKind::Path, square(10_000)), &r);
        let fills: Vec<i32> = p
            .iter()
            .filter(|p| matches!(p.shape, Shape::Fill { .. }))
            .map(|p| p.priority)
            .collect();
        let Symbol::Area { color, patterns } = &r.symbols[&s.id_for_code("402").unwrap()] else {
            panic!("402 is an area");
        };
        assert_eq!(patterns.len(), 1);
        let dot = patterns[0].point.dot.expect("a dot").0;
        let dots = fills.iter().filter(|&&c| c == dot).count();
        // 100 mm² at one dot per 0.75 × 0.75 mm: about 178.
        assert!((150..=200).contains(&dots), "{dots}");
        assert_eq!(fills.iter().filter(|&&c| c == *color).count(), 1);
        // 401 has no pattern.
        let p = primitives(&object(&s, "401", MapObjectKind::Path, square(10_000)), &r);
        assert_eq!(p.len(), 1);
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
