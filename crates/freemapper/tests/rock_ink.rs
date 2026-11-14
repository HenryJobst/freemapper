//! The rock of the Bastei example stays readable (ADR 0126): the ink of the impassable cliffs
//! (201 with its tags, the top line 201.3) and of the rock pillars and fins (206) on any square
//! centimetre of paper. Before the generalization the worst square centimetre held 230 % (the
//! strokes lay on each other), afterwards 31 %. The example is rebuilt by hand, so this guards
//! the example as committed.

use std::collections::HashMap;
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;

/// The Bastei is a forest map at 1:10 000: the symbols are drawn at 150 %.
const ENLARGEMENT: f64 = 1.5;
/// The line of 201 and 201.3 (mm at 100 %), and the ink of the tags of 201 per mm of line: 0.12
/// mm wide, 0.575 mm long, every 0.5 mm.
const LINE_MM: f64 = 0.35;
const TAG_INK_PER_MM: f64 = 0.12 * 0.575 / 0.5;
/// Ruling: no square centimetre of paper more than 40 % black with rock.
const MAX_INK_SHARE: f64 = 0.40;

/// The objects of `xml`: the code of the symbol and the points in mm on paper.
fn objects(xml: &str) -> Vec<(String, Vec<(f64, f64)>)> {
    let mut reader = Reader::from_str(xml);
    let mut codes: HashMap<String, String> = HashMap::new();
    let mut out = Vec::new();
    let mut symbol: Option<String> = None;
    let mut in_coords = false;
    let mut text = String::new();
    loop {
        match reader.read_event().unwrap() {
            Event::Start(e) | Event::Empty(e) => {
                let attr = |name: &str| {
                    e.attributes()
                        .flatten()
                        .find(|a| a.key.as_ref() == name)
                        .map(|a| a.value.to_string())
                };
                match e.name().as_ref() {
                    "symbol" => {
                        if let (Some(id), Some(code)) = (attr("id"), attr("code")) {
                            codes.insert(id, code);
                        }
                    }
                    "object" => symbol = attr("symbol"),
                    "coords" => {
                        in_coords = true;
                        text.clear();
                    }
                    _ => {}
                }
            }
            Event::Text(t) if in_coords => text.push_str(t.as_ref()),
            Event::End(e) if e.name().as_ref() == "coords" => in_coords = false,
            Event::End(e) if e.name().as_ref() == "object" => {
                if let Some(code) = symbol.take().and_then(|s| codes.get(&s).cloned()) {
                    let points = text
                        .split(';')
                        .filter_map(|c| {
                            let mut parts = c.split_whitespace();
                            let x: f64 = parts.next()?.parse().ok()?;
                            let y: f64 = parts.next()?.parse().ok()?;
                            Some((x / 1000.0, y / 1000.0))
                        })
                        .collect();
                    out.push((code, points));
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    out
}

#[test]
fn the_rock_of_the_bastei_stays_readable() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/sachsen-bastei/sachsen-bastei.omap");
    let xml = std::fs::read_to_string(&path).unwrap();
    let mut ink: HashMap<(i64, i64), f64> = HashMap::new();
    let cell = |x: f64, y: f64| ((x / 10.0).floor() as i64, (y / 10.0).floor() as i64);
    for (code, points) in objects(&xml) {
        match code.as_str() {
            "206" => {
                let n = points.len();
                if n < 3 {
                    continue;
                }
                let area = (0..n)
                    .map(|i| {
                        let (a, b) = (points[i], points[(i + 1) % n]);
                        a.0 * b.1 - b.0 * a.1
                    })
                    .sum::<f64>()
                    .abs()
                    / 2.0;
                let (cx, cy) = points.iter().fold((0.0, 0.0), |s, p| {
                    (s.0 + p.0 / n as f64, s.1 + p.1 / n as f64)
                });
                *ink.entry(cell(cx, cy)).or_default() += area;
            }
            "201" | "201.3" => {
                let tags = if code == "201" { TAG_INK_PER_MM } else { 0.0 };
                let per_mm = (LINE_MM + tags) * ENLARGEMENT;
                for w in points.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    let length = (b.0 - a.0).hypot(b.1 - a.1);
                    let steps = (length / 0.5).ceil().max(1.0) as usize;
                    for k in 0..steps {
                        let t = (k as f64 + 0.5) / steps as f64;
                        let at = cell(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                        *ink.entry(at).or_default() += per_mm * length / steps as f64;
                    }
                }
            }
            _ => {}
        }
    }
    assert!(!ink.is_empty(), "the Bastei has rock");
    let (worst_cell, worst) = ink
        .iter()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(c, v)| (*c, *v / 100.0))
        .unwrap();
    assert!(
        worst <= MAX_INK_SHARE,
        "the square centimetre {worst_cell:?} is {:.0} % black with rock",
        worst * 100.0
    );
}
