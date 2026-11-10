//! The source block of the PDF: the attribution lines, shortened when they would take too much
//! room under the title block (ADR 0081). `ATTRIBUTION.txt` and the OOM map keep the full list.
//!
//! The block has a budget of [`MAX_LINES`] lines. A block within it is printed unchanged. A
//! longer one is compacted in steps until it fits:
//!
//! 1. sources of one provider with the `dl-de/zero-2-0` licence become one line,
//! 2. long source names get their short forms,
//! 3. neighbouring lines are joined with ` · ` up to the line width,
//! 4. the `dl-de/zero-2-0` lines shrink to the provider names and a pointer to `ATTRIBUTION.txt`.
//!
//! Every other line (OpenStreetMap, `dl-de/by-2-0`, CC BY, the licence texts) and the freemapper
//! credit stay word for word in every step: only `dl-de/zero-2-0` asks for no attribution.

use crate::render::metrics::text_width;

/// Most lines the block may have, the freemapper credit line included.
pub const MAX_LINES: usize = 5;
/// The line that points to the full list (step 4).
pub const POINTER: &str = "Vollständige Quellen: ATTRIBUTION.txt im Projektordner";
const ZERO: &str = "dl-de/zero-2-0";
const JOIN: &str = " · ";
/// Long source names of the Geoportal Berlin and their short forms (step 2).
const SHORT_NAMES: [(&str, &str); 5] = [
    ("ALKIS Berlin", "ALKIS"),
    ("Karte 1:5000 von Berlin (K5)", "K5"),
    ("Straßenbefahrung 2014", "Straßenbefahrung"),
    ("Gebäudehöhen (Umweltatlas)", "Gebäudehöhen"),
    ("Öffentliche Beleuchtung", "Beleuchtung"),
];

/// What the block may take.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Budget {
    pub max_lines: usize,
    /// Widest line a compaction step produces, in mm on paper.
    pub max_width_mm: f64,
    /// Text size on paper in mm.
    pub font_mm: f64,
}

impl Budget {
    /// The budget for a map `content_width_mm` wide, with the small text of the sheet (1.8 mm).
    /// The width is that of the map content, at least 100 mm and at most 220 mm.
    pub fn for_content(content_width_mm: f64) -> Self {
        Self {
            max_lines: MAX_LINES,
            max_width_mm: content_width_mm.clamp(100.0, 220.0),
            font_mm: 1.8,
        }
    }

    fn width(&self, text: &str) -> f64 {
        text_width(text, false) * self.font_mm
    }
}

/// The `dl-de/zero-2-0` sources of one provider: `<provider> / <name>, dl-de/zero-2-0[, note]`.
struct Zero {
    provider: String,
    names: Vec<String>,
    /// How many of the merged lines carry a change note, and how many lines there were.
    changed: usize,
    lines: usize,
}

enum Entry {
    /// A line that stays as it is.
    Verbatim(String),
    Zero(Zero),
}

fn parse(line: &str) -> Entry {
    let verbatim = || Entry::Verbatim(line.to_string());
    if line.contains("dl-de/by") || line.contains(';') {
        return verbatim();
    }
    let Some((head, rest)) = line.split_once(&format!(", {ZERO}")) else {
        return verbatim();
    };
    if !(rest.is_empty() || rest.starts_with(", ")) {
        return verbatim();
    }
    let head = head.strip_prefix("© ").unwrap_or(head);
    let (provider, names) = head.split_once(" / ").unwrap_or((head, ""));
    Entry::Zero(Zero {
        provider: provider.to_string(),
        names: names
            .split(", ")
            .filter(|n| !n.is_empty())
            .map(str::to_string)
            .collect(),
        changed: usize::from(!rest.is_empty()),
        lines: 1,
    })
}

/// Step 1 and 2: one `Zero` for each provider, in the order of first appearance.
fn merge(entries: Vec<Entry>, shorten: bool) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    for entry in entries {
        let Entry::Zero(z) = entry else {
            out.push(entry);
            continue;
        };
        let names = z
            .names
            .iter()
            .map(|n| if shorten { short(n) } else { n.clone() });
        let found = out.iter_mut().find_map(|e| match e {
            Entry::Zero(o) if o.provider == z.provider => Some(o),
            _ => None,
        });
        if let Some(target) = found {
            target.changed += z.changed;
            target.lines += z.lines;
            for name in names {
                if !target.names.contains(&name) {
                    target.names.push(name);
                }
            }
        } else {
            let mut unique: Vec<String> = Vec::new();
            for name in names {
                if !unique.contains(&name) {
                    unique.push(name);
                }
            }
            out.push(Entry::Zero(Zero { names: unique, ..z }));
        }
    }
    out
}

fn short(name: &str) -> String {
    SHORT_NAMES
        .iter()
        .find(|(long, _)| *long == name)
        .map_or_else(|| name.to_string(), |(_, s)| (*s).to_string())
}

/// The lines of a merged provider, the names packed into lines of at most the budget's width;
/// every line names provider and licence.
fn render_zero(z: &Zero, budget: &Budget) -> Vec<String> {
    let note = if z.changed == 0 {
        ""
    } else if z.changed < z.lines {
        ", teils Daten geändert"
    } else {
        ", Daten geändert"
    };
    let header = format!("{} ({ZERO}{note})", z.provider);
    if z.names.is_empty() {
        return vec![header];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    for name in &z.names {
        current = Some(match current.take() {
            None => format!("{header}: {name}"),
            Some(line) => {
                let longer = format!("{line}, {name}");
                if budget.width(&longer) <= budget.max_width_mm {
                    longer
                } else {
                    lines.push(line);
                    format!("{header}: {name}")
                }
            }
        });
    }
    lines.extend(current);
    lines
}

fn render(entries: &[Entry], budget: &Budget) -> Vec<String> {
    entries
        .iter()
        .flat_map(|e| match e {
            Entry::Verbatim(text) => vec![text.clone()],
            Entry::Zero(z) => render_zero(z, budget),
        })
        .collect()
}

/// Step 3: neighbouring lines joined with ` · ` while the result stays within the width.
fn join(lines: Vec<String>, budget: &Budget) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        match out.last_mut() {
            Some(last) if budget.width(&format!("{last}{JOIN}{line}")) <= budget.max_width_mm => {
                last.push_str(JOIN);
                last.push_str(&line);
            }
            _ => out.push(line),
        }
    }
    out
}

/// Step 4: the `dl-de/zero-2-0` lines reduced to the provider names, then the pointer line.
fn pointer(entries: Vec<Entry>, budget: &Budget) -> Vec<String> {
    let mut providers: Vec<String> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for entry in entries {
        match entry {
            Entry::Verbatim(text) => lines.push(text),
            Entry::Zero(z) => {
                if !providers.contains(&z.provider) {
                    providers.push(z.provider);
                }
            }
        }
    }
    if !providers.is_empty() {
        lines.push(format!(
            "Weitere Quellen ({ZERO}): {}",
            providers.join(", ")
        ));
    }
    lines.push(POINTER.to_string());
    join(lines, budget)
}

/// The block for the PDF: `attribution` (the lines of the sheet without the credit) compacted to
/// the budget, `credit` (the freemapper line) last. A block within the budget is returned as it
/// is.
pub fn compact(attribution: &[String], credit: &str, budget: &Budget) -> Vec<String> {
    let with_credit = |mut lines: Vec<String>| {
        lines.push(credit.to_string());
        lines
    };
    if attribution.len() < budget.max_lines {
        return with_credit(attribution.to_vec());
    }
    let entries = || attribution.iter().map(|l| parse(l)).collect::<Vec<_>>();
    let steps = [
        render(&merge(entries(), false), budget),
        render(&merge(entries(), true), budget),
        join(render(&merge(entries(), true), budget), budget),
    ];
    for lines in steps {
        if lines.len() < budget.max_lines {
            return with_credit(lines);
        }
    }
    with_credit(pointer(merge(entries(), true), budget))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CREDIT: &str = "Erstellt mit freemapper – https://github.com/HenryJobst/freemapper";
    const OSM: &str = "© OpenStreetMap-Mitwirkende, ODbL";
    const LGB: &str =
        "© GeoBasis-DE/LGB 2026, dl-de/by-2-0; © Geoportal Berlin, dl-de/zero-2-0, Daten geändert";

    fn berlin() -> Vec<String> {
        [
            OSM,
            LGB,
            "© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert",
            "Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0",
            "Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0",
            "Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert",
            "Geoportal Berlin / Öffentliche Beleuchtung, Zierbrunnen, dl-de/zero-2-0",
            "Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0",
        ]
        .map(String::from)
        .to_vec()
    }

    /// A `dl-de/by-2-0` line too wide to be joined with another one on a 150 mm page.
    fn long_by(i: usize) -> String {
        format!(
            "Quelle {i} mit einem recht langen Namen, dl-de/by-2-0, Daten geändert, Landesamt für Vermessung"
        )
    }

    fn budget() -> Budget {
        Budget::for_content(150.0)
    }

    fn width(line: &str) -> f64 {
        budget().width(line)
    }

    #[test]
    fn short_blocks_stay_unchanged() {
        let lines = vec![
            OSM.to_string(),
            "© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert".to_string(),
        ];
        let block = compact(&lines, CREDIT, &budget());
        assert_eq!(block, [lines[0].as_str(), lines[1].as_str(), CREDIT]);
    }

    #[test]
    fn berlin_fits_the_budget() {
        let block = compact(&berlin(), CREDIT, &budget());
        assert!(block.len() <= MAX_LINES, "{block:#?}");
        assert!(block.len() < berlin().len() + 1);
        assert_eq!(block.last().map(String::as_str), Some(CREDIT));
    }

    #[test]
    fn berlin_zero_sources_merge_into_one_line_per_provider() {
        let block = compact(&berlin(), CREDIT, &Budget::for_content(220.0));
        let berlin_lines: Vec<&String> = block
            .iter()
            .filter(|l| l.starts_with("Geoportal Berlin ("))
            .collect();
        assert_eq!(berlin_lines.len(), 1, "{block:#?}");
        let line = berlin_lines[0];
        assert!(
            line.contains("dl-de/zero-2-0, teils Daten geändert"),
            "{line}"
        );
        for name in [
            "ALKIS Berlin",
            "Karte 1:5000 von Berlin (K5)",
            "Baumbestand Berlin",
            "Straßenbefahrung 2014",
            "Zierbrunnen",
            "Gebäudehöhen (Umweltatlas)",
        ] {
            assert!(line.contains(name), "{name} missing in {line}");
        }
    }

    #[test]
    fn required_parts_are_retained_in_every_step() {
        // Many dl-de/by-2-0 sources force the last step.
        let mut lines = berlin();
        for i in 0..6 {
            lines.push(long_by(i));
        }
        for block in [
            compact(&berlin(), CREDIT, &budget()),
            compact(&lines, CREDIT, &budget()),
        ] {
            let text = block.join("\n");
            assert!(text.contains(OSM), "{text}");
            assert!(text.contains(LGB), "{text}");
            assert!(text.contains(CREDIT), "{text}");
        }
        let text = compact(&lines, CREDIT, &budget()).join("\n");
        for i in 0..6 {
            assert!(text.contains(&long_by(i)), "{text}");
        }
        assert!(text.contains(POINTER), "{text}");
    }

    #[test]
    fn narrow_pages_join_less_and_pointer_is_the_last_resort() {
        let mut lines = berlin();
        lines.extend((0..4).map(long_by));
        let block = compact(&lines, CREDIT, &budget());
        let text = block.join("\n");
        assert!(text.contains(POINTER));
        assert!(text.contains("Weitere Quellen (dl-de/zero-2-0): Geoportal Berlin"));
        assert!(!text.contains("Zierbrunnen"));
    }

    #[test]
    fn short_names_are_used_when_the_long_ones_do_not_fit() {
        // Two providers, each with names that overflow one line without the short forms.
        let lines: Vec<String> = [
            OSM,
            LGB,
            "Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0",
            "Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0",
            "Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0",
            "Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0",
            "Geoportal Berlin / Öffentliche Beleuchtung, dl-de/zero-2-0",
        ]
        .map(String::from)
        .to_vec();
        let block = compact(&lines, CREDIT, &Budget::for_content(100.0));
        let text = block.join("\n");
        assert!(block.len() <= MAX_LINES, "{block:#?}");
        assert!(text.contains("Geoportal Berlin (dl-de/zero-2-0)"), "{text}");
    }

    #[test]
    fn lines_never_exceed_the_width() {
        for content in [100.0, 150.0, 220.0] {
            let budget = Budget::for_content(content);
            for line in compact(&berlin(), CREDIT, &budget) {
                // Lines that stay word for word are exempt; every generated one is checked.
                if berlin().contains(&line) || line == CREDIT {
                    continue;
                }
                assert!(
                    budget.width(&line) <= budget.max_width_mm,
                    "{line} is {} mm",
                    budget.width(&line)
                );
            }
        }
        assert!(width(CREDIT) < 100.0);
    }

    #[test]
    fn compaction_is_deterministic_and_idempotent() {
        let once = compact(&berlin(), CREDIT, &budget());
        assert_eq!(once, compact(&berlin(), CREDIT, &budget()));
        let body: Vec<String> = once[..once.len() - 1].to_vec();
        assert_eq!(compact(&body, CREDIT, &budget()), once);
    }

    #[test]
    fn only_unmergeable_lines_stay_when_nothing_is_zero() {
        let lines: Vec<String> = (0..6).map(long_by).collect();
        let block = compact(&lines, CREDIT, &budget());
        let text = block.join("\n");
        for line in &lines {
            assert!(text.contains(line.as_str()), "{text}");
        }
        assert!(text.contains(POINTER));
    }

    #[test]
    fn example_blocks_of_other_regions_stay_unchanged() {
        let sanssouci = [
            OSM.to_string(),
            "© GeoBasis-DE/LGB 2025, dl-de/by-2-0, Daten geändert".to_string(),
            "Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0".to_string(),
        ];
        let cochem = [
            OSM.to_string(),
            "©GeoBasis-DE / LVermGeoRP2026, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]"
                .to_string(),
            "Lizenz dl-de/by-2-0 (Datenlizenz Deutschland – Namensnennung – Version 2.0): https://www.govdata.de/dl-de/by-2-0".to_string(),
        ];
        for body in [&sanssouci[..], &cochem[..]] {
            let block = compact(body, CREDIT, &budget());
            let expected: Vec<String> = body.iter().cloned().chain([CREDIT.to_string()]).collect();
            assert_eq!(block, expected);
        }
    }
}
