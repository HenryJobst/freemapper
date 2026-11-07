//! Colours and placeholder symbols for OCAD, read from the OOM symbol set, and the freemapper code
//! → OCAD symbol number table.
//!
//! The placeholders carry the right number, name, type and main colour. In OCAD the official
//! ISSprOM 2019 or school symbol set replaces them by number ("Load colors and symbols from").

use std::collections::{HashMap, HashSet};

use super::file::{OcdSymbolKind, TextAlign};
use crate::omap::SymbolSet;
use crate::project::MapKind;

/// freemapper codes whose OCAD number differs from the plain `code × 1000` rule, checked against
/// OCAD's official ISSprOM 2019-2 set (`docs/ocad/ocd12-reference.md` §9).
const NUMBERS: [(&str, i32); 9] = [
    ("501.1", 501_100),
    ("501.5", 501_001),
    ("501.10", 501_020),
    ("505.1", 505_000),
    ("505.2", 505_001),
    ("513.1", 513_100),
    ("513.2", 513_200),
    ("521.1", 521_003),
    ("410.1", 411_002),
];

/// The OCAD number of a freemapper symbol code. OCAD reads the part after the dot as a three-place
/// decimal ("501.1" = 501.100); the hedge is 411.002 in ISSprOM 2019-2 but 410.002 in the school
/// set, which follows the first 2019 edition.
pub fn ocad_number(code: &str, kind: MapKind) -> Option<i32> {
    if code == "410.1" && kind == MapKind::Schoolyard {
        return Some(410_002);
    }
    if let Some(&(_, number)) = NUMBERS.iter().find(|(c, _)| *c == code) {
        return Some(number);
    }
    let (main, sub) = code.split_once('.').unwrap_or((code, ""));
    let main: i32 = main.parse().ok()?;
    if sub.len() > 3 || !sub.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let sub: i32 = if sub.is_empty() {
        0
    } else {
        format!("{sub:0<3}").parse().ok()?
    };
    (main > 0 || sub > 0).then_some(main * 1000 + sub)
}

/// Hands out unique OCAD numbers: the table or decimal rule, else the next free number.
#[derive(Debug, Default)]
pub struct Numbers {
    taken: HashSet<i32>,
}

impl Numbers {
    pub fn assign(&mut self, code: &str, kind: MapKind) -> i32 {
        let mut number = ocad_number(code, kind).unwrap_or(999_000);
        while !self.taken.insert(number) {
            number += 1;
        }
        number
    }
}

/// A colour of the OCAD colour table.
#[derive(Debug, Clone, PartialEq)]
pub struct OcdColour {
    /// OOM colour priority, which OOM symbols refer to; also the OCAD colour number.
    pub number: i16,
    pub name: String,
    /// CMYK in percent.
    pub cmyk: [f64; 4],
    pub opacity: f64,
}

impl OcdColour {
    /// The type 9 parameter string.
    pub fn parameter_string(&self) -> String {
        let [c, m, y, k] = self.cmyk;
        format!(
            "{}\tn{}\tc{c:.1}\tm{m:.1}\ty{y:.1}\tk{k:.1}\to0\tt{:.0}",
            self.name,
            self.number,
            self.opacity * 100.0
        )
    }

    /// Index in OCAD's 5 × 5 × 5 icon palette.
    pub fn icon_index(&self) -> u8 {
        let [c, m, y, k] = self.cmyk.map(|v| v / 100.0);
        let level = |v: f64| ((255.0 * (1.0 - v) * (1.0 - k) + 32.0) / 64.0).floor() as u8;
        25 * level(c).min(4) + 5 * level(m).min(4) + level(y).min(4)
    }
}

/// ASCII for parameter strings and symbol names: German letters transliterated, other non-ASCII
/// characters dropped, tabs replaced by spaces.
pub fn ascii(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            'ä' => out.push_str("ae"),
            'ö' => out.push_str("oe"),
            'ü' => out.push_str("ue"),
            'Ä' => out.push_str("Ae"),
            'Ö' => out.push_str("Oe"),
            'Ü' => out.push_str("Ue"),
            '©' => out.push_str("(c)"),
            'ß' => out.push_str("ss"),
            '–' | '—' => out.push('-'),
            '·' => out.push('-'),
            '\t' => out.push(' '),
            c if c.is_ascii() && !c.is_ascii_control() || c == '\r' || c == '\n' => out.push(c),
            _ => {}
        }
    }
    out
}

fn attr(e: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<String> {
    crate::omap::symbol_set::attribute(e, name).ok().flatten()
}

fn num(e: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<f64> {
    attr(e, name).and_then(|v| v.parse().ok())
}

/// The colour table in OOM priority order (first is on top).
pub fn colours(set: &SymbolSet) -> Vec<OcdColour> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(&set.colors_xml);
    let mut colours = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e) | Event::Empty(e)) if e.name().as_ref() == "color" => {
                let percent = |n| num(&e, n).unwrap_or(0.0) * 100.0;
                colours.push(OcdColour {
                    number: num(&e, "priority").unwrap_or(0.0) as i16,
                    name: ascii(&attr(&e, "name").unwrap_or_default()),
                    cmyk: [percent("c"), percent("m"), percent("y"), percent("k")],
                    opacity: num(&e, "opacity").unwrap_or(1.0),
                });
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    colours.sort_by_key(|c| c.number);
    colours
}

/// What a placeholder looks like, in OCAD units (0.01 mm).
#[derive(Debug, Clone, PartialEq)]
pub enum PlaceholderKind {
    Point {
        color: i16,
        diameter: i16,
    },
    Line {
        color: i16,
        width: i16,
        dash: Option<(i16, i16)>,
    },
    Area {
        color: i16,
    },
    Text {
        color: i16,
        size: i16,
        bold: bool,
    },
}

impl PlaceholderKind {
    pub fn color(&self) -> i16 {
        match self {
            Self::Point { color, .. }
            | Self::Line { color, .. }
            | Self::Area { color }
            | Self::Text { color, .. } => *color,
        }
    }

    /// The OCAD symbol kind; texts get the alignment of their objects.
    pub fn ocd_kind(&self, align: TextAlign) -> OcdSymbolKind {
        match *self {
            Self::Point { color, diameter } => OcdSymbolKind::Point { color, diameter },
            Self::Line { color, width, dash } => OcdSymbolKind::Line { color, width, dash },
            Self::Area { color } => OcdSymbolKind::Area { color },
            Self::Text { color, size, bold } => OcdSymbolKind::Text {
                color,
                size,
                bold,
                align,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Placeholder {
    pub code: String,
    pub name: String,
    pub kind: PlaceholderKind,
}

/// Default sizes when the OOM symbol gives none: 0.5 mm dot, 0.1 mm line.
const DEFAULT_DOT: i16 = 50;
const DEFAULT_WIDTH: i16 = 10;

/// OOM 0.001 mm → OCAD 0.01 mm.
fn ocad_units(v: f64) -> i16 {
    (v / 10.0).round().clamp(1.0, f64::from(i16::MAX)) as i16
}

/// Main line of an OOM line symbol: colour, width, (dash, gap) when dashed.
type OomLine = (i16, f64, Option<(f64, f64)>);

/// One OOM symbol as far as placeholders need it.
#[derive(Debug, Clone, Default)]
struct OomSymbol {
    code: String,
    name: String,
    kind: u32,
    line: Option<OomLine>,
    border: Option<(i16, f64)>,
    area_color: Option<i16>,
    pattern_color: Option<i16>,
    point: Option<(i16, f64, i16, f64)>,
    first_color: Option<i16>,
    text: Option<(f64, bool)>,
    text_color: Option<i16>,
    parts: Vec<u32>,
}

fn color_of(e: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<i16> {
    num(e, name).map(|v| v as i16).filter(|&c| c >= 0)
}

/// Parses the symbols of an OOM symbol set by id.
fn oom_symbols(set: &SymbolSet) -> HashMap<u32, OomSymbol> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(&set.symbols_xml);
    let mut symbols = HashMap::new();
    let mut current: Option<(u32, OomSymbol)> = None;
    // Open elements inside the current top-level symbol (0: its direct children). Point symbol
    // elements nest further `<symbol>`s, which must not end the current one.
    let mut depth = 0usize;
    while let Ok(event) = reader.read_event() {
        let (e, empty) = match event {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(_) => {
                if depth == 0 {
                    if let Some((id, symbol)) = current.take() {
                        symbols.insert(id, symbol);
                    }
                } else {
                    depth -= 1;
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };
        let name = e.name().as_ref().to_string();
        let just_opened = current.is_none();
        match (name.as_str(), current.as_mut()) {
            ("symbol", None) => {
                let id = num(&e, "id").map(|v| v as u32);
                if let Some(id) = id {
                    current = Some((
                        id,
                        OomSymbol {
                            code: attr(&e, "code").unwrap_or_default(),
                            name: attr(&e, "name").unwrap_or_default(),
                            kind: num(&e, "type").unwrap_or(0.0) as u32,
                            ..OomSymbol::default()
                        },
                    ));
                    depth = 0;
                }
            }
            (_, Some((_, symbol))) => {
                if let Some(c) = color_of(&e, "color")
                    .or_else(|| color_of(&e, "inner_color"))
                    .or_else(|| color_of(&e, "outer_color"))
                {
                    symbol.first_color.get_or_insert(c);
                }
                match name.as_str() {
                    // Combined symbols may define private parts inline.
                    "line_symbol" if symbol.line.is_none() && (depth == 0 || symbol.kind == 16) => {
                        let dash = (attr(&e, "dashed").as_deref() == Some("true")).then(|| {
                            (
                                num(&e, "dash_length").unwrap_or(0.0),
                                num(&e, "break_length").unwrap_or(0.0),
                            )
                        });
                        symbol.line = Some((
                            color_of(&e, "color").unwrap_or(-1),
                            num(&e, "line_width").unwrap_or(0.0),
                            dash,
                        ));
                    }
                    "border" if symbol.border.is_none() => {
                        if let Some(c) = color_of(&e, "color") {
                            symbol.border = Some((c, num(&e, "width").unwrap_or(0.0)));
                        }
                    }
                    "area_symbol"
                        if symbol.area_color.is_none() && (depth == 0 || symbol.kind == 16) =>
                    {
                        symbol.area_color = color_of(&e, "inner_color");
                    }
                    "pattern" if symbol.pattern_color.is_none() => {
                        symbol.pattern_color = color_of(&e, "color");
                    }
                    "point_symbol" if symbol.point.is_none() && depth == 0 => {
                        symbol.point = Some((
                            color_of(&e, "inner_color").unwrap_or(-1),
                            num(&e, "inner_radius").unwrap_or(0.0),
                            color_of(&e, "outer_color").unwrap_or(-1),
                            num(&e, "outer_width").unwrap_or(0.0),
                        ));
                    }
                    "font" => {
                        symbol.text = Some((
                            num(&e, "size").unwrap_or(4000.0),
                            attr(&e, "bold").as_deref() == Some("true"),
                        ));
                    }
                    "text" if symbol.text_color.is_none() => {
                        symbol.text_color = color_of(&e, "color");
                    }
                    "part" => {
                        if let Some(id) = num(&e, "symbol") {
                            symbol.parts.push(id as u32);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        let opened_top_symbol = name == "symbol" && depth == 0 && just_opened;
        if !empty && current.is_some() && !opened_top_symbol {
            depth += 1;
        }
    }
    symbols
}

fn placeholder_kind(symbols: &HashMap<u32, OomSymbol>, s: &OomSymbol) -> Option<PlaceholderKind> {
    let fallback = s.first_color.unwrap_or(0);
    match s.kind {
        1 => {
            let (inner, radius, outer, outer_width) = s.point.unwrap_or((-1, 0.0, -1, 0.0));
            let (color, diameter) = if outer >= 0 && outer_width > 0.0 {
                (outer, 2.0 * (radius + outer_width))
            } else if inner >= 0 && radius > 0.0 {
                (inner, 2.0 * radius)
            } else {
                (fallback, f64::from(DEFAULT_DOT) * 10.0)
            };
            Some(PlaceholderKind::Point {
                color,
                diameter: ocad_units(diameter),
            })
        }
        2 => {
            let (color, width, dash) = match s.line {
                Some((color, width, dash)) if color >= 0 && width > 0.0 => (color, width, dash),
                _ => match s.border {
                    Some((color, width)) if width > 0.0 => (color, width, None),
                    _ => (fallback, f64::from(DEFAULT_WIDTH) * 10.0, None),
                },
            };
            Some(PlaceholderKind::Line {
                color,
                width: ocad_units(width),
                dash: dash.map(|(d, g)| (ocad_units(d), ocad_units(g))),
            })
        }
        4 => Some(PlaceholderKind::Area {
            color: s.area_color.or(s.pattern_color).unwrap_or(fallback),
        }),
        8 => {
            let (size_um, bold) = s.text.unwrap_or((4000.0, false));
            // Em size in mm → points × 10.
            let size = (10.0 * size_um / 1000.0 / 25.4 * 72.0).round() as i16;
            Some(PlaceholderKind::Text {
                color: s.text_color.unwrap_or(fallback),
                size,
                bold,
            })
        }
        16 => {
            let parts: Vec<PlaceholderKind> = s
                .parts
                .iter()
                .filter_map(|id| symbols.get(id))
                .filter_map(|part| placeholder_kind(symbols, part))
                .collect();
            let own = [
                s.area_color.map(|color| PlaceholderKind::Area { color }),
                s.line.map(|(color, width, dash)| PlaceholderKind::Line {
                    color: if color >= 0 { color } else { fallback },
                    width: ocad_units(width.max(f64::from(DEFAULT_WIDTH) * 10.0)),
                    dash: dash.map(|(d, g)| (ocad_units(d), ocad_units(g))),
                }),
            ];
            let parts: Vec<PlaceholderKind> =
                parts.into_iter().chain(own.into_iter().flatten()).collect();
            parts
                .iter()
                .find(|k| matches!(k, PlaceholderKind::Area { .. }))
                .or_else(|| {
                    parts
                        .iter()
                        .find(|k| matches!(k, PlaceholderKind::Line { .. }))
                })
                .or(parts.first())
                .cloned()
        }
        _ => None,
    }
}

/// Placeholders for the symbols of `set`, by OOM symbol id.
pub fn placeholders(set: &SymbolSet) -> HashMap<u32, Placeholder> {
    let symbols = oom_symbols(set);
    symbols
        .iter()
        .filter_map(|(&id, s)| {
            Some((
                id,
                Placeholder {
                    code: s.code.clone(),
                    name: ascii(s.name.trim()),
                    kind: placeholder_kind(&symbols, s)?,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_follow_ocads_decimal_rule() {
        for (code, number) in [
            ("101", 101_000),
            ("501.1", 501_100),
            ("501.5", 501_001),
            ("501.10", 501_020),
            ("505.1", 505_000),
            ("505.2", 505_001),
            ("513.2", 513_200),
            ("521", 521_000),
            ("521.1", 521_003),
            ("993", 993_000),
            ("530.25", 530_250),
        ] {
            assert_eq!(ocad_number(code, MapKind::Sprint), Some(number), "{code}");
        }
        assert_eq!(ocad_number("abc", MapKind::Sprint), None);
        assert_eq!(ocad_number("1.2345", MapKind::Sprint), None);
    }

    #[test]
    fn hedge_number_depends_on_kind() {
        assert_eq!(ocad_number("410.1", MapKind::Sprint), Some(411_002));
        assert_eq!(ocad_number("410.1", MapKind::Schoolyard), Some(410_002));
    }

    #[test]
    fn unknown_codes_get_unique_numbers() {
        let mut numbers = Numbers::default();
        assert_eq!(numbers.assign("530.1", MapKind::Sprint), 530_100);
        assert_eq!(numbers.assign("530.10", MapKind::Sprint), 530_101);
        assert_eq!(numbers.assign("x", MapKind::Sprint), 999_000);
        assert_eq!(numbers.assign("y", MapKind::Sprint), 999_001);
    }

    #[test]
    fn colours_keep_priority_order() {
        let colours = colours(&SymbolSet::issprom_2019());
        assert_eq!(colours.len(), 32);
        assert_eq!(colours[0].number, 0);
        assert_eq!(colours[0].name, "Purple for course overprint");
        let black = &colours[2];
        assert_eq!(
            black.parameter_string(),
            "Black 100%\tn2\tc0.0\tm0.0\ty0.0\tk100.0\to0\tt100"
        );
        assert_eq!(black.icon_index(), 0);
        let white = &colours[1];
        assert_eq!(white.icon_index(), 124);
        assert!(colours.windows(2).all(|w| w[0].number < w[1].number));
    }

    #[test]
    fn names_are_ascii() {
        assert_eq!(
            ascii("Straßenverkehr · Äquidistanz\tgrün"),
            "Strassenverkehr - Aequidistanz gruen"
        );
        assert_eq!(ascii("© GeoBasis-DE/LGB"), "(c) GeoBasis-DE/LGB");
        assert_eq!(ascii("Ω"), "");
    }

    #[test]
    fn line_area_point_text_placeholders() {
        let set = SymbolSet::issprom_2019().with_sheet_symbols();
        let all = placeholders(&set);
        let by_code = |code: &str| {
            all.values()
                .find(|p| p.code == code)
                .unwrap_or_else(|| panic!("{code}"))
                .clone()
        };
        assert_eq!(
            by_code("505.1").kind,
            PlaceholderKind::Line {
                color: 14,
                width: 35,
                dash: None
            }
        );
        assert_eq!(by_code("520").kind, PlaceholderKind::Area { color: 22 });
        assert_eq!(
            by_code("418").kind,
            PlaceholderKind::Point {
                color: 3,
                diameter: 80
            }
        );
        assert_eq!(
            by_code("991").kind,
            PlaceholderKind::Text {
                color: 2,
                size: 142,
                bold: true
            }
        );
        assert_eq!(
            by_code("505.1").name,
            "Unpaved footpath or track, footprint: 1.4 m"
        );
    }

    #[test]
    fn every_symbol_gets_a_placeholder_despite_nested_elements() {
        let set = SymbolSet::issprom_2019();
        let all = placeholders(&set);
        assert_eq!(all.len(), set.symbol_count());
        for (code, _) in [("543", ()), ("418", ()), ("521", ())] {
            if let Some(id) = set.id_for_code(code) {
                assert_eq!(all[&id].code, code);
            }
        }
        for id in 0..set.symbol_count() as u32 {
            if let Some(p) = all.get(&id) {
                assert_eq!(set.id_for_code(&p.code), Some(id), "{p:?}");
            }
        }
    }

    #[test]
    fn combined_symbols_use_their_area_part() {
        let set = SymbolSet::issprom_2019();
        let all = placeholders(&set);
        let building = all.values().find(|p| p.code == "521").unwrap();
        assert!(
            matches!(building.kind, PlaceholderKind::Area { .. }),
            "{building:?}"
        );
        let paved = all.values().find(|p| p.code == "501.10").unwrap();
        assert!(
            matches!(paved.kind, PlaceholderKind::Area { .. }),
            "{paved:?}"
        );
    }
}
