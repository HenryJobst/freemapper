//! The OOM symbol definitions as far as the renderer needs them (`docs/render/oom-render-reference.md`).
//!
//! Lengths stay in OOM units (0.001 mm); colours are indices = priorities.

use std::collections::HashMap;

use crate::omap::{MapCoord, SymbolSet};

/// A colour of the map: its priority (0 is on top) and CMYK (0..1).
#[derive(Debug, Clone, PartialEq)]
pub struct RenderColour {
    pub priority: i32,
    pub cmyk: [f64; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    Flat,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Bevel,
    Miter,
    Round,
}

/// Dash pattern of a line or border.
#[derive(Debug, Clone, PartialEq)]
pub struct Dash {
    pub dash: f64,
    pub gap: f64,
    pub group: u32,
    pub in_group_gap: f64,
    pub half_outer: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    pub color: i32,
    pub width: f64,
    pub shift: f64,
    pub dash: Option<Dash>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mid {
    pub symbol: Box<PointDef>,
    pub segment: f64,
    pub end: f64,
    pub per_spot: u32,
    pub distance: f64,
    pub at_least_one: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineDef {
    /// −1: no main line.
    pub color: i32,
    pub width: f64,
    pub cap: Cap,
    pub join: Join,
    pub dash: Option<Dash>,
    pub border: Option<Border>,
    pub mid: Option<Mid>,
    pub start: Option<Box<PointDef>>,
    pub end: Option<Box<PointDef>>,
}

/// One element of a point symbol: a symbol drawn on local coordinates (0.001 mm, y down).
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub symbol: Symbol,
    pub coords: Vec<MapCoord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointDef {
    pub rotatable: bool,
    /// Filled dot: colour, radius.
    pub dot: Option<(i32, f64)>,
    /// Stroked ring: colour, centre radius, width.
    pub ring: Option<(i32, f64, f64)>,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDef {
    pub color: i32,
    /// Em size (0.001 mm).
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub line_spacing: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    /// Another symbol of the set, by index (= id).
    Ref(u32),
    Inline(Box<Symbol>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Line(LineDef),
    Area { color: i32 },
    Point(PointDef),
    Text(TextDef),
    Combined(Vec<Part>),
}

/// Colours and symbols of a symbol set, ready for drawing.
#[derive(Debug, Clone)]
pub struct RenderSymbols {
    pub colours: Vec<RenderColour>,
    pub symbols: HashMap<u32, Symbol>,
}

/// A parsed XML element.
#[derive(Debug, Default)]
struct Node {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
    text: String,
}

impl Node {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn num(&self, name: &str) -> f64 {
        self.attr(name).and_then(|v| v.parse().ok()).unwrap_or(0.0)
    }

    fn color(&self, name: &str) -> i32 {
        self.attr(name).and_then(|v| v.parse().ok()).unwrap_or(-1)
    }

    fn flag(&self, name: &str) -> bool {
        self.attr(name) == Some("true")
    }

    fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }
}

/// The element tree of an XML fragment (one synthetic root holding the top-level elements).
fn parse(xml: &str) -> Node {
    use quick_xml::events::{BytesStart, Event};
    fn node(e: &BytesStart<'_>) -> Node {
        Node {
            name: e.name().as_ref().to_string(),
            attrs: e
                .attributes()
                .flatten()
                .map(|a| {
                    let key = a.key.as_ref().to_string();
                    let value = crate::omap::symbol_set::attribute(e, &key)
                        .ok()
                        .flatten()
                        .unwrap_or_default();
                    (key, value)
                })
                .collect(),
            ..Node::default()
        }
    }
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut stack = vec![Node::default()];
    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(e) => stack.push(node(&e)),
            Event::Empty(e) => {
                let n = node(&e);
                stack.last_mut().expect("root").children.push(n);
            }
            Event::End(_) => {
                if stack.len() > 1 {
                    let n = stack.pop().expect("checked");
                    stack.last_mut().expect("root").children.push(n);
                }
            }
            Event::Text(t) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(t.as_ref());
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    while stack.len() > 1 {
        let n = stack.pop().expect("checked");
        stack.last_mut().expect("root").children.push(n);
    }
    stack.pop().expect("root")
}

/// `x y [flags];` coordinates of an OOM `<coords>` element.
fn coords(node: &Node) -> Vec<MapCoord> {
    node.text
        .split(';')
        .filter_map(|c| {
            let v: Vec<i32> = c
                .split_whitespace()
                .filter_map(|t| t.parse().ok())
                .collect();
            (v.len() >= 2).then(|| MapCoord {
                x: v[0],
                y: v[1],
                flags: v.get(2).copied().unwrap_or(0) as u8,
            })
        })
        .collect()
}

fn dash(node: &Node) -> Option<Dash> {
    (node.flag("dashed") && node.num("dash_length") > 0.0).then(|| Dash {
        dash: node.num("dash_length"),
        gap: node.num("break_length"),
        group: (node.num("dashes_in_group") as u32).max(1),
        in_group_gap: node.num("in_group_break_length"),
        half_outer: node.flag("half_outer_dashes"),
    })
}

/// The point symbol nested in `<mid_symbol>`, `<start_symbol>` … (`<symbol><point_symbol>`).
fn nested_point(node: Option<&Node>) -> Option<Box<PointDef>> {
    let point = node?.child("symbol")?.child("point_symbol")?;
    Some(Box::new(point_def(point)))
}

fn line_def(node: &Node) -> LineDef {
    let cap = match node.num("cap_style") as i32 {
        1 => Cap::Round,
        2 => Cap::Square,
        _ => Cap::Flat,
    };
    let join = match node.num("join_style") as i32 {
        0 => Join::Bevel,
        2 => Join::Round,
        _ => Join::Miter,
    };
    let border = node
        .child("borders")
        .and_then(|b| b.child("border"))
        .map(|b| Border {
            color: b.color("color"),
            width: b.num("width"),
            shift: b.num("shift"),
            dash: (b.flag("dashed") && b.num("dash_length") > 0.0 && b.num("break_length") > 0.0)
                .then(|| Dash {
                    dash: b.num("dash_length"),
                    gap: b.num("break_length"),
                    group: 1,
                    in_group_gap: 0.0,
                    half_outer: false,
                }),
        })
        .filter(|b| b.width > 0.0 && b.color >= 0);
    let dash = dash(node);
    let mid = nested_point(node.child("mid_symbol"))
        .filter(|_| node.num("segment_length") > 0.0 && dash.is_none())
        .map(|symbol| Mid {
            symbol,
            segment: node.num("segment_length"),
            end: node.num("end_length"),
            per_spot: (node.num("mid_symbols_per_spot") as u32).max(1),
            distance: node.num("mid_symbol_distance"),
            at_least_one: node.flag("show_at_least_one_symbol"),
        });
    LineDef {
        color: node.color("color"),
        width: node.num("line_width"),
        cap,
        join,
        dash,
        border,
        mid,
        start: nested_point(node.child("start_symbol")),
        end: nested_point(node.child("end_symbol")),
    }
}

fn point_def(node: &Node) -> PointDef {
    let (inner, radius) = (node.color("inner_color"), node.num("inner_radius"));
    let (outer, width) = (node.color("outer_color"), node.num("outer_width"));
    PointDef {
        rotatable: node.flag("rotatable"),
        dot: (inner >= 0 && radius > 0.0).then_some((inner, radius)),
        ring: (outer >= 0 && width > 0.0).then_some((outer, radius + width / 2.0, width)),
        elements: node
            .children
            .iter()
            .filter(|c| c.name == "element")
            .filter_map(|e| {
                Some(Element {
                    symbol: symbol(e.child("symbol")?)?,
                    coords: e.child("object")?.child("coords").map(coords)?,
                })
            })
            .collect(),
    }
}

fn symbol(node: &Node) -> Option<Symbol> {
    match node.num("type") as u32 {
        1 => node
            .child("point_symbol")
            .map(|p| Symbol::Point(point_def(p))),
        2 => node.child("line_symbol").map(|l| Symbol::Line(line_def(l))),
        4 => node.child("area_symbol").map(|a| Symbol::Area {
            color: a.color("inner_color"),
        }),
        8 => {
            let t = node.child("text_symbol")?;
            let font = t.child("font");
            let text = t.child("text");
            Some(Symbol::Text(TextDef {
                color: text.map_or(-1, |t| t.color("color")),
                size: font.map_or(4000.0, |f| f.num("size")),
                bold: font.is_some_and(|f| f.flag("bold")),
                italic: font.is_some_and(|f| f.flag("italic")),
                line_spacing: text
                    .and_then(|t| t.attr("line_spacing"))
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1.0),
            }))
        }
        16 => {
            let c = node.child("combined_symbol")?;
            Some(Symbol::Combined(
                c.children
                    .iter()
                    .filter(|p| p.name == "part")
                    .filter_map(|p| {
                        if p.flag("private") {
                            p.child("symbol")
                                .and_then(symbol)
                                .map(|s| Part::Inline(Box::new(s)))
                        } else {
                            p.attr("symbol")?.parse().ok().map(Part::Ref)
                        }
                    })
                    .collect(),
            ))
        }
        _ => None,
    }
}

impl RenderSymbols {
    pub fn from_set(set: &SymbolSet) -> Self {
        let colours = parse(&set.colors_xml)
            .children
            .iter()
            .flat_map(|c| c.children.iter().chain(std::iter::once(c)))
            .filter(|c| c.name == "color")
            .map(|c| RenderColour {
                priority: c.num("priority") as i32,
                cmyk: [c.num("c"), c.num("m"), c.num("y"), c.num("k")],
            })
            .collect();
        let root = parse(&set.symbols_xml);
        let symbols = root
            .children
            .iter()
            .flat_map(|s| s.children.iter().chain(std::iter::once(s)))
            .filter(|s| s.name == "symbol")
            .filter_map(|s| Some((s.attr("id")?.parse().ok()?, symbol(s)?)))
            .collect();
        Self { colours, symbols }
    }

    pub fn colour(&self, priority: i32) -> Option<&RenderColour> {
        self.colours.iter().find(|c| c.priority == priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbols() -> (RenderSymbols, SymbolSet) {
        let set = SymbolSet::issprom_2019().with_sheet_symbols();
        (RenderSymbols::from_set(&set), set)
    }

    fn by_code<'a>(s: &'a (RenderSymbols, SymbolSet), code: &str) -> &'a Symbol {
        &s.0.symbols[&s.1.id_for_code(code).unwrap()]
    }

    fn line<'a>(s: &'a (RenderSymbols, SymbolSet), code: &str) -> &'a LineDef {
        match by_code(s, code) {
            Symbol::Line(l) => l,
            other => panic!("{code}: {other:?}"),
        }
    }

    #[test]
    fn colours_have_priority_and_cmyk() {
        let (r, _) = symbols();
        assert_eq!(r.colours.len(), 32);
        assert_eq!(r.colour(2).unwrap().cmyk, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(r.colour(8).unwrap().cmyk, [0.25, 0.75, 1.0, 0.0]);
    }

    #[test]
    fn line_with_dashed_borders() {
        let s = symbols();
        let l = line(&s, "505.1");
        assert_eq!(
            (l.color, l.width, l.cap, l.join),
            (14, 350.0, Cap::Flat, Join::Miter)
        );
        assert!(l.dash.is_none());
        let b = l.border.as_ref().unwrap();
        assert_eq!((b.color, b.width, b.shift), (15, 100.0, 50.0));
        let d = b.dash.as_ref().unwrap();
        assert_eq!((d.dash, d.gap), (2000.0, 250.0));
    }

    #[test]
    fn dashed_main_line() {
        let s = symbols();
        let d = line(&s, "506").dash.clone().unwrap();
        assert_eq!(
            (d.dash, d.gap, d.group, d.half_outer),
            (1500.0, 370.0, 1, false)
        );
        assert_eq!(line(&s, "101").cap, Cap::Round);
        assert_eq!(line(&s, "101").join, Join::Round);
    }

    #[test]
    fn mid_symbol_of_fence() {
        let s = symbols();
        let mid = line(&s, "518").mid.clone().unwrap();
        assert_eq!((mid.segment, mid.end, mid.per_spot), (3750.0, 2325.0, 1));
        assert!(!mid.at_least_one);
        assert!(mid.symbol.rotatable);
        assert!(mid.symbol.dot.is_none(), "inner colour −1");
        assert_eq!(mid.symbol.elements.len(), 2);
        assert_eq!(
            mid.symbol.elements[0].coords,
            [MapCoord::new(450, 0), MapCoord::new(825, 650)]
        );
        assert!(matches!(mid.symbol.elements[0].symbol, Symbol::Line(ref l) if l.width == 140.0));
        assert!(line(&s, "513.2").mid.as_ref().unwrap().at_least_one);
    }

    #[test]
    fn start_and_end_symbols() {
        let set = SymbolSet::school_2019();
        let s = (RenderSymbols::from_set(&set), set);
        let l = line(&s, "543");
        assert!(l.start.is_some() && l.end.is_some());
        assert!(l.start.as_ref().unwrap().rotatable);
        assert!(l.mid.is_none());
    }

    #[test]
    fn point_with_ring_and_elements() {
        let s = symbols();
        let Symbol::Point(bush) = by_code(&s, "418") else {
            panic!()
        };
        assert_eq!(bush.dot, Some((21, 100.0)));
        assert_eq!(bush.ring, Some((3, 250.0, 300.0)));
        assert!(!bush.rotatable);
        let Symbol::Point(minimum) = by_code(&s, "521.1") else {
            panic!()
        };
        assert!(minimum.rotatable);
        assert_eq!(minimum.elements.len(), 2);
    }

    #[test]
    fn combined_parts_resolve() {
        let s = symbols();
        let Symbol::Combined(building) = by_code(&s, "521") else {
            panic!()
        };
        assert_eq!(building, &[Part::Ref(143), Part::Ref(144)]);
        let Symbol::Combined(paved) = by_code(&s, "501") else {
            panic!()
        };
        assert_eq!(paved[0], Part::Ref(105));
        assert!(
            matches!(&paved[1], Part::Inline(p) if matches!(**p, Symbol::Line(ref l) if l.color == 15))
        );
        assert!(matches!(s.0.symbols[&143], Symbol::Area { color: 10 }));
    }

    #[test]
    fn text_symbol() {
        let s = symbols();
        let Symbol::Text(title) = by_code(&s, "991") else {
            panic!()
        };
        assert_eq!((title.color, title.size, title.bold), (2, 5000.0, true));
    }
}
