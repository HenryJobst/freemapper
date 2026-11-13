//! The permit notes as Markdown (`permits.md`, ADR 0117), and the command that fetches and
//! writes them: `freemapper permits`, `run --permits` and, later, the MCP tool.
//!
//! The engine assembles the notes (`freemapper::permits`); this file words them in the language
//! of the run. Every text is in the catalog under `permits-*`; the hints are stored in
//! `permits.json` as their catalog id with arguments.

use std::path::{Path, PathBuf};

use freemapper::fetch::{Cache, Fetcher};
use freemapper::note::{NOTES, Note};
use freemapper::permits::PermitsError;
use freemapper::permits::fetch::fetch_inputs;
use freemapper::permits::notes::{
    AdminEntry, AdminLevel, Hint, OperatorEntry, OperatorRole, OsmEntry, PermitNotes,
    ProtectedEntry, Share, SourceStatus, hint_key,
};
use freemapper::permits::regional::{
    GreenSpaceEntry, ProtectedObjectEntry, ProtectedObjectKind, RegionalNotes, RegionalStatus,
    Words,
};
use freemapper::permits::store::{PERMITS_JSON, PERMITS_MD, write_permit_notes};
use freemapper::project::{MapKind, ProjectFile};

use crate::lock::{self, Who};
use crate::text::{FluentValue, Lang, TEXTS};

/// What writing the permit notes left behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermitsWritten {
    /// `permits.md`.
    pub markdown: PathBuf,
    /// `permits.json`.
    pub json: PathBuf,
    /// Warning lines in the language of the run for the sources an offline run found missing;
    /// online, the fetch says that itself while it works.
    pub warnings: Vec<String>,
}

/// Takes the lock of the project folder `dir` for `who`, then [`write_permits`]. A folder that
/// does not exist is left to the project file, which reports its usual error.
pub fn run_permits(
    dir: &Path,
    offline: bool,
    cache: &Cache,
    fetcher: &dyn Fetcher,
    who: Who,
    lang: &Lang,
    progress: &mut dyn FnMut(Note),
) -> anyhow::Result<PermitsWritten> {
    let _lock = match dir.is_dir() {
        // The command line lets go of the lock on Ctrl+C too.
        true if who == Who::Cli => Some(lock::acquire_for_command(dir, who)?),
        true => Some(lock::acquire(dir, who)?),
        false => None,
    };
    write_permits(dir, offline, cache, fetcher, lang, progress)
}

/// Fetches the inputs (or reads the stored ones `offline`), writes `permits.json` and
/// `permits.md` into the project folder `dir`, without taking the folder lock: the caller holds
/// it ([`run_permits`], or the run the notes belong to).
pub fn write_permits(
    dir: &Path,
    offline: bool,
    cache: &Cache,
    fetcher: &dyn Fetcher,
    lang: &Lang,
    progress: &mut dyn FnMut(Note),
) -> anyhow::Result<PermitsWritten> {
    let project = ProjectFile::load(dir)?;
    let inputs = fetch_inputs(&project, dir, fetcher, cache, offline, progress)?;
    let notes = write_permit_notes(dir, &project, &inputs, freemapper::clock::today())?;
    let markdown = dir.join(PERMITS_MD);
    freemapper_fs::write_inside(dir, &markdown, render_markdown(&notes, lang).as_bytes())
        .map_err(|(path, e)| PermitsError::Io(path, e))?;
    let warnings = if offline {
        left_out(&notes, lang)
    } else {
        Vec::new()
    };
    Ok(PermitsWritten {
        markdown,
        json: dir.join(PERMITS_JSON),
        warnings,
    })
}

/// A warning line for each source the notes had to leave out.
fn left_out(notes: &PermitNotes, lang: &Lang) -> Vec<String> {
    let regional = notes
        .regional
        .iter()
        .flat_map(|r| &r.sources)
        .map(|r| (regional_name(lang, r), &r.status));
    notes
        .sources
        .iter()
        .map(|s| (source_name(lang, &s.id), s))
        .chain(regional)
        .filter(|(_, s)| s.state != "read")
        .map(|(name, s)| {
            let reason = match &s.reason {
                Some(reason) if s.state == "unreachable" => reason.clone(),
                _ => t(lang, "cli-permits-not-read", &[]),
            };
            let body = t(
                lang,
                "cli-permits-source-left-out",
                &[("source", name.into()), ("reason", reason.into())],
            );
            NOTES.text(lang, "note-warning", &[("text", body.into())])
        })
        .collect()
}

fn t(lang: &Lang, id: &str, args: &[(&str, FluentValue)]) -> String {
    TEXTS.text(lang, id, args)
}

fn arg(value: impl ToString) -> FluentValue<'static> {
    FluentValue::from(value.to_string())
}

/// Text from the data (a name, a title, the reason a server gave) with the characters that
/// Markdown would read as markup escaped, and line breaks as spaces: it stays on its line and
/// cannot start a list item or a rule. A leading `-` or `+` is escaped too, since a name can
/// open a list line.
fn md(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, c) in text.chars().enumerate() {
        if c == '\r' || c == '\n' {
            out.push(' ');
            continue;
        }
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#'
        ) || (i == 0 && matches!(c, '-' | '+'))
        {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// A table cell of a trusted text (a credit line): only the column bar is escaped.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\r', '\n'], " ")
}

/// A link from the data: an autolink when it is a plain web address.
fn link(url: &str) -> String {
    let plain = (url.starts_with("https://") || url.starts_with("http://"))
        && !url.contains(|c: char| c.is_whitespace() || c == '<' || c == '>');
    if plain { format!("<{url}>") } else { md(url) }
}

/// A number with `decimals` decimals in the way of `lang` (a decimal comma in German).
fn decimal(lang: &Lang, value: f64, decimals: usize) -> String {
    let text = format!("{value:.decimals$}");
    if lang.code() == "de" {
        text.replace('.', ",")
    } else {
        text
    }
}

/// A share of the map area: whole percent, "< 1 %" below one.
fn share_text(lang: &Lang, percent: f64) -> String {
    if percent < 1.0 {
        t(lang, "permits-share-below-one", &[])
    } else {
        t(
            lang,
            "permits-share",
            &[("percent", arg(percent.round() as i64))],
        )
    }
}

/// The serde name of a level or kind (`gemeinde`, `naturschutzgebiet`): the selector of its word.
fn serde_name(key: impl serde::Serialize) -> String {
    match serde_json::to_value(key) {
        Ok(serde_json::Value::String(name)) => name,
        _ => String::new(),
    }
}

/// The short name of a source (`vg250`, `bfn`, `osm`).
fn source_name(lang: &Lang, id: &str) -> String {
    t(lang, "permits-source", &[("source", arg(id))])
}

/// The status of the source `id`, if the notes have one.
fn status<'a>(notes: &'a PermitNotes, id: &str) -> Option<&'a SourceStatus> {
    notes.sources.iter().find(|s| s.id == id)
}

fn was_read(notes: &PermitNotes, id: &str) -> bool {
    status(notes, id).is_some_and(|s| s.state == "read")
}

/// The block of a section whose source gave nothing.
fn not_read(lang: &Lang, id: &str) -> String {
    t(
        lang,
        "permits-section-not-read",
        &[("source", arg(source_name(lang, id)))],
    )
}

/// `kind name`, or the name alone.
fn unit_name(unit: &AdminEntry) -> String {
    format!("{} {}", unit.kind.trim(), unit.name.trim())
        .trim()
        .to_string()
}

fn admin_line(lang: &Lang, s: &Share<AdminEntry>) -> String {
    let level = t(
        lang,
        "permits-level",
        &[("level", arg(serde_name(s.item.level)))],
    );
    format!(
        "- {}",
        t(
            lang,
            "permits-admin-line",
            &[
                ("level", arg(level)),
                ("name", arg(md(&unit_name(&s.item)))),
                ("share", arg(share_text(lang, s.percent))),
            ],
        )
    )
}

/// The entries of a section and, below them, those that only touch the edge.
fn with_edge<T>(
    lang: &Lang,
    entries: &[Share<T>],
    line: impl Fn(&Share<T>) -> String,
    edge_note: Option<String>,
) -> Vec<String> {
    let (edge, main): (Vec<&Share<T>>, Vec<&Share<T>>) = entries.iter().partition(|s| s.at_edge);
    let mut blocks = Vec::new();
    if !main.is_empty() {
        blocks.push(main.iter().map(|s| line(s)).collect::<Vec<_>>().join("\n"));
    }
    if !edge.is_empty() {
        blocks.push(t(lang, "permits-at-edge", &[]));
        blocks.push(edge.iter().map(|s| line(s)).collect::<Vec<_>>().join("\n"));
        blocks.extend(edge_note);
    }
    blocks
}

fn protected_lines(lang: &Lang, s: &Share<ProtectedEntry>) -> String {
    let p = &s.item;
    let kind = t(lang, "permits-kind", &[("kind", arg(serde_name(p.kind)))]);
    let mut lines = vec![format!(
        "- {}",
        t(
            lang,
            "permits-protected-line",
            &[
                ("kind", arg(kind)),
                ("name", arg(md(&p.name))),
                ("share", arg(share_text(lang, s.percent))),
            ],
        )
    )];
    let mut sub = |id: &str, name: &str, value: String| {
        lines.push(format!("  - {}", t(lang, id, &[(name, arg(value))])));
    };
    if let Some(date) = p.legal_date {
        sub("permits-protected-date", "date", date.to_string());
    }
    if let Some(title) = p.legal_title.as_deref().filter(|t| !t.trim().is_empty()) {
        sub("permits-protected-title", "title", md(title));
    }
    if let Some(url) = p.legal_link.as_deref().filter(|u| !u.trim().is_empty()) {
        sub("permits-protected-link", "url", link(url.trim()));
    }
    if let Some(url) = p.site_link.as_deref().filter(|u| !u.trim().is_empty()) {
        sub("permits-protected-site-link", "url", link(url.trim()));
    }
    if let Some(year) = p.year {
        sub("permits-protected-year", "year", year.to_string());
    }
    lines.join("\n")
}

fn osm_lines(lang: &Lang, s: &Share<OsmEntry>) -> String {
    let o = &s.item;
    let kind = match (&o.protection_title, &o.protect_class) {
        (Some(title), _) if !title.trim().is_empty() => md(title),
        (_, Some(class)) if !class.trim().is_empty() => t(
            lang,
            "permits-osm-protect-class",
            &[("class", arg(md(class)))],
        ),
        _ => format!("`{}`", o.object.replace('`', "")),
    };
    let name = match o.name.as_deref().filter(|n| !n.trim().is_empty()) {
        Some(name) => t(lang, "permits-quoted", &[("text", arg(md(name)))]),
        None => t(lang, "permits-osm-unnamed", &[]),
    };
    let mut lines = vec![format!(
        "- {}",
        t(
            lang,
            "permits-osm-line",
            &[
                ("kind", arg(kind)),
                ("name", arg(name)),
                ("share", arg(share_text(lang, s.percent))),
                ("id", arg(md(&o.osm_id))),
            ],
        )
    )];
    if let Some(operator) = o.operator.as_deref().filter(|n| !n.trim().is_empty()) {
        lines.push(format!(
            "  - {}",
            t(
                lang,
                "permits-osm-operator",
                &[("operator", arg(md(operator)))]
            )
        ));
    }
    lines.join("\n")
}

fn operator_line(lang: &Lang, o: &OperatorEntry) -> String {
    let roles = match (
        o.roles.contains(&OperatorRole::Operator),
        o.roles.contains(&OperatorRole::Owner),
    ) {
        (true, true) => "both",
        (false, true) => "owner",
        _ => "operator",
    };
    let objects: Vec<String> = o
        .objects
        .iter()
        .map(|object| format!("`{}`", object.replace('`', "")))
        .collect();
    format!(
        "- {}",
        t(
            lang,
            "permits-operator-line",
            &[
                ("name", arg(md(&o.name))),
                ("roles", arg(roles)),
                ("objects", arg(objects.join(", "))),
            ],
        )
    )
}

/// A hint in words. The nature authority without a Kreis and the order office without an Amt
/// pick the general variant of their text; a Landkreis is named with its kind.
fn hint_text(lang: &Lang, notes: &PermitNotes, hint: &Hint) -> String {
    let mut args: Vec<(String, String)> =
        hint.args.iter().map(|(k, v)| (k.clone(), md(v))).collect();
    let has = |args: &[(String, String)], name: &str| args.iter().any(|(k, _)| k == name);
    if hint.key == hint_key::NATURE_AUTHORITY {
        if !has(&args, "kreisfrei") {
            args.push(("kreisfrei".into(), "unknown".into()));
        }
        let kreisfrei = hint.args.get("kreisfrei").map(String::as_str);
        if let (Some("no"), Some(name)) = (kreisfrei, hint.args.get("kreis"))
            && let Some(kreis) = notes
                .admin
                .iter()
                .map(|s| &s.item)
                .find(|u| u.level == AdminLevel::Kreis && &u.name == name)
        {
            for (k, v) in &mut args {
                if k == "kreis" {
                    *v = md(&unit_name(kreis));
                }
            }
        }
    }
    if hint.key == hint_key::ORDER_OFFICE && !has(&args, "amt") {
        args.push(("amt".into(), "none".into()));
    }
    if hint.key == hint_key::OWNERS_UNKNOWN && !has(&args, "rest") {
        args.push(("rest".into(), "no".into()));
    }
    if hint.key == hint_key::OWNER_SHARE {
        let percent = hint.args.get("percent").and_then(|p| p.parse::<f64>().ok());
        args.retain(|(k, _)| k != "percent");
        args.push(("share".into(), share_text(lang, percent.unwrap_or(0.0))));
    }
    let args: Vec<(&str, FluentValue)> = args.iter().map(|(k, v)| (k.as_str(), arg(v))).collect();
    t(lang, &hint.key, &args)
}

fn sources_table(lang: &Lang, notes: &PermitNotes) -> String {
    let mut rows = vec![
        format!(
            "| {} | {} | {} |",
            t(lang, "permits-column-source", &[]),
            t(lang, "permits-column-state", &[]),
            t(lang, "permits-column-credit", &[])
        ),
        "|---|---|---|".to_string(),
    ];
    for s in &notes.sources {
        let what = t(lang, "permits-source-what", &[("source", arg(&s.id))]);
        let state = match (s.state.as_str(), s.fetched) {
            ("read", Some(date)) if s.id == "osm" => {
                t(lang, "permits-state-read-osm", &[("date", arg(date))])
            }
            ("read", Some(date)) => t(lang, "permits-state-read", &[("date", arg(date))]),
            ("unreachable", _) => t(
                lang,
                "permits-state-unreachable",
                &[("reason", arg(md(s.reason.as_deref().unwrap_or_default())))],
            ),
            _ => t(lang, "permits-state-not-read", &[]),
        };
        let credit = match &s.credit {
            Some(credit) => credit.clone(),
            None => t(lang, "permits-no-credit", &[]),
        };
        rows.push(format!(
            "| {}: {} | {} | {} |",
            cell(&source_name(lang, &s.id)),
            cell(&what),
            // The reason in it is escaped already; the catalog text has no bar.
            state,
            cell(&credit)
        ));
    }
    rows.extend(regional_rows(lang, notes));
    rows.join("\n")
}

/// A text of the notes in `lang`.
fn words<'a>(lang: &Lang, words: &'a Words) -> &'a str {
    if lang.code() == "de" {
        &words.de
    } else {
        &words.en
    }
}

/// The name of a regional source in a warning: its publisher and what it holds.
fn regional_name(lang: &Lang, r: &RegionalStatus) -> String {
    format!(
        "{}: {}",
        words(lang, &r.source.label),
        words(lang, &r.source.what)
    )
}

/// The state of one source in the table.
fn state_text(lang: &Lang, s: &SourceStatus) -> String {
    match (s.state.as_str(), s.fetched) {
        ("read", Some(date)) => t(lang, "permits-state-read", &[("date", arg(date))]),
        ("unreachable", _) => t(
            lang,
            "permits-state-unreachable",
            &[("reason", arg(md(s.reason.as_deref().unwrap_or_default())))],
        ),
        _ => t(lang, "permits-state-not-read", &[]),
    }
}

/// One row per publisher and credit of the regional sources: the sources of one portal share
/// a line, and a source that was not read is named in it.
fn regional_rows(lang: &Lang, notes: &PermitNotes) -> Vec<String> {
    let mut groups: Vec<(String, String, Vec<&RegionalStatus>)> = Vec::new();
    for r in notes.regional.iter().flat_map(|r| &r.sources) {
        let label = words(lang, &r.source.label).to_string();
        match groups
            .iter_mut()
            .find(|(l, c, _)| *l == label && *c == r.source.credit)
        {
            Some((_, _, members)) => members.push(r),
            None => groups.push((label, r.source.credit.clone(), vec![r])),
        }
    }
    groups
        .into_iter()
        .map(|(label, credit, members)| {
            let what: Vec<String> = members
                .iter()
                .map(|m| md(words(lang, &m.source.what)))
                .collect();
            let states: Vec<String> = members
                .iter()
                .map(|m| state_text(lang, &m.status))
                .collect();
            let state = if states.iter().all(|s| *s == states[0]) {
                states[0].clone()
            } else {
                members
                    .iter()
                    .zip(&states)
                    .map(|(m, s)| format!("{}: {s}", md(words(lang, &m.source.what))))
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            let read = members.iter().any(|m| m.status.state == "read");
            let credit = if read {
                cell(&credit)
            } else {
                t(lang, "permits-no-credit", &[])
            };
            format!(
                "| {}: {} | {} | {} |",
                cell(&md(&label)),
                cell(&what.join(", ")),
                state,
                credit
            )
        })
        .collect()
}

/// The notes of the publishers on what their data are worth, verbatim, below the table.
fn regional_source_notes(lang: &Lang, notes: &PermitNotes) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for r in notes.regional.iter().flat_map(|r| &r.sources) {
        if let Some(note) = r
            .source
            .note
            .as_deref()
            .filter(|_| r.status.state == "read")
        {
            let line = t(
                lang,
                "permits-source-note",
                &[
                    ("source", arg(md(&regional_name(lang, r)))),
                    ("note", arg(md(note))),
                ],
            );
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
    }
    lines
}

/// The publisher of the sources of a regional part, for its headings.
fn regional_label(lang: &Lang, part: &RegionalNotes) -> String {
    part.sources
        .first()
        .map(|r| md(words(lang, &r.source.label)))
        .unwrap_or_default()
}

fn green_space_line(lang: &Lang, s: &Share<GreenSpaceEntry>) -> String {
    let g = &s.item;
    let name = if g.name.trim().is_empty() {
        t(lang, "permits-osm-unnamed", &[])
    } else {
        t(lang, "permits-quoted", &[("text", arg(md(&g.name)))])
    };
    let mut details = Vec::new();
    if let Some(district) = &g.district {
        details.push(t(
            lang,
            "permits-green-space-district",
            &[("district", arg(md(district)))],
        ));
    }
    if let Some(dedication) = &g.dedication {
        details.push(t(
            lang,
            "permits-green-space-dedication",
            &[("dedication", arg(md(dedication)))],
        ));
    }
    let name = if details.is_empty() {
        name
    } else {
        format!("{name} ({})", details.join("; "))
    };
    format!(
        "- {}",
        t(
            lang,
            "permits-share-line",
            &[
                ("name", arg(name)),
                ("share", arg(share_text(lang, s.percent))),
            ],
        )
    )
}

fn object_kind(lang: &Lang, kind: ProtectedObjectKind) -> String {
    t(
        lang,
        "permits-object-kind",
        &[("kind", arg(serde_name(kind)))],
    )
}

fn object_line(lang: &Lang, o: &ProtectedObjectEntry) -> String {
    let mut text = object_kind(lang, o.kind);
    if let Some(name) = o.name.as_deref().filter(|n| !n.trim().is_empty()) {
        text = format!(
            "{text} {}",
            t(lang, "permits-quoted", &[("text", arg(md(name)))])
        );
    }
    match o.percent {
        Some(percent) => format!("- {text}, {}", share_text(lang, percent)),
        None if o.count > 1 => format!(
            "- {text}, {}",
            t(lang, "permits-object-count", &[("count", arg(o.count))])
        ),
        None => format!("- {text}"),
    }
}

/// The sections of one regional part; a section whose source was not read is left out.
fn regional_blocks(lang: &Lang, part: &RegionalNotes) -> Vec<String> {
    let source = regional_label(lang, part);
    let heading = |id: &str| format!("## {}", t(lang, id, &[("source", arg(&source))]));
    let none = |id: &str| t(lang, id, &[("source", arg(&source))]);
    let mut blocks = Vec::new();
    if let Some(districts) = &part.districts {
        blocks.push(heading("permits-section-districts"));
        if districts.is_empty() {
            blocks.push(none("permits-none-districts"));
        } else {
            blocks.extend(with_edge(
                lang,
                districts,
                |s| {
                    format!(
                        "- {}",
                        t(
                            lang,
                            "permits-share-line",
                            &[
                                ("name", arg(md(&s.item.name))),
                                ("share", arg(share_text(lang, s.percent))),
                            ],
                        )
                    )
                },
                None,
            ));
        }
    }
    if let Some(green) = &part.green_spaces {
        blocks.push(heading("permits-section-green-spaces"));
        if green.entries.is_empty() {
            blocks.push(none("permits-none-green-spaces"));
        } else {
            blocks.extend(with_edge(
                lang,
                &green.entries,
                |s| green_space_line(lang, s),
                None,
            ));
        }
        blocks.push(match green.playgrounds {
            0 => t(lang, "permits-playgrounds-none", &[]),
            1 => t(lang, "permits-playgrounds-one", &[]),
            n => t(lang, "permits-playgrounds-many", &[("count", arg(n))]),
        });
    }
    if let Some(owners) = &part.owners {
        blocks.push(heading("permits-section-owners"));
        if owners.is_empty() {
            blocks.push(none("permits-none-owners"));
        } else {
            let lines: Vec<String> = owners
                .iter()
                .map(|s| {
                    format!(
                        "- {}",
                        t(
                            lang,
                            "permits-share-line",
                            &[
                                ("name", arg(md(&s.item.owner))),
                                ("share", arg(share_text(lang, s.percent))),
                            ],
                        )
                    )
                })
                .collect();
            blocks.push(lines.join("\n"));
            blocks.push(t(lang, "permits-owners-rest", &[]));
        }
    }
    if let Some(monuments) = &part.monuments {
        blocks.push(heading("permits-section-monuments"));
        if monuments.total == 0 {
            blocks.push(none("permits-none-monuments"));
        } else {
            blocks.push(t(
                lang,
                "permits-monuments-total",
                &[("count", arg(monuments.total))],
            ));
            let mut lines = Vec::new();
            for group in &monuments.groups {
                lines.push(format!(
                    "- {}",
                    t(
                        lang,
                        "permits-count-line",
                        &[("name", arg(md(&group.kind))), ("count", arg(group.count))],
                    )
                ));
                for url in &group.links {
                    lines.push(format!("  - {}", link(url)));
                }
            }
            blocks.push(lines.join("\n"));
            if monuments.links_left > 0 {
                blocks.push(t(
                    lang,
                    "permits-monuments-links-left",
                    &[("count", arg(monuments.links_left))],
                ));
            }
        }
    }
    if let Some(objects) = &part.protected_objects {
        blocks.push(heading("permits-section-protected-objects"));
        if objects.is_empty() {
            blocks.push(none("permits-none-protected-objects"));
        } else {
            let lines: Vec<String> = objects.iter().map(|o| object_line(lang, o)).collect();
            blocks.push(lines.join("\n"));
            let mut citations: Vec<&str> = Vec::new();
            for c in objects.iter().filter_map(|o| o.citation.as_deref()) {
                if !citations.contains(&c) {
                    citations.push(c);
                }
            }
            if !citations.is_empty() {
                blocks.push(t(lang, "permits-object-citations", &[]));
                let lines: Vec<String> = citations.iter().map(|c| format!("- {}", md(c))).collect();
                blocks.push(lines.join("\n"));
            }
        }
    }
    blocks
}

/// The notes as Markdown in `lang`. The disclaimer is the first paragraph after the title.
pub fn render_markdown(notes: &PermitNotes, lang: &Lang) -> String {
    let heading = |id: &str| format!("## {}", t(lang, id, &[]));
    let kind = match notes.map_kind {
        MapKind::Schoolyard => "schoolyard",
        MapKind::Sprint => "sprint",
        MapKind::Forest => "forest",
    };
    let mut blocks = vec![
        format!("# {}", t(lang, "permits-title", &[])),
        t(lang, "permits-disclaimer", &[]),
        t(lang, "permits-bfn-merge", &[]),
        t(
            lang,
            "permits-created",
            &[
                ("date", arg(notes.created)),
                ("kind", arg(kind)),
                ("area", arg(decimal(lang, notes.area_m2 / 10_000.0, 1))),
            ],
        ),
    ];

    blocks.push(heading("permits-section-admin"));
    if !was_read(notes, "vg250") {
        blocks.push(not_read(lang, "vg250"));
    } else if notes.admin.is_empty() {
        blocks.push(t(lang, "permits-none-admin", &[]));
    } else {
        blocks.extend(with_edge(
            lang,
            &notes.admin,
            |s| admin_line(lang, s),
            Some(t(lang, "permits-at-edge-generalised", &[])),
        ));
    }

    for part in &notes.regional {
        blocks.extend(regional_blocks(lang, part));
    }

    blocks.push(heading("permits-section-protected"));
    if !was_read(notes, "bfn") {
        blocks.push(not_read(lang, "bfn"));
    } else if notes.protected.is_empty() {
        blocks.push(t(lang, "permits-none-protected", &[]));
    } else {
        blocks.extend(with_edge(
            lang,
            &notes.protected,
            |s| protected_lines(lang, s),
            None,
        ));
    }

    let osm = was_read(notes, "osm");
    let gap = t(lang, "permits-osm-enclosing-gap", &[]);
    blocks.push(heading("permits-section-osm-protected"));
    if !osm {
        blocks.push(not_read(lang, "osm"));
    } else {
        if notes.osm_protected.is_empty() {
            blocks.push(t(lang, "permits-none-osm-protected", &[]));
        } else {
            blocks.extend(with_edge(
                lang,
                &notes.osm_protected,
                |s| osm_lines(lang, s),
                None,
            ));
        }
        blocks.push(gap.clone());
    }

    blocks.push(heading("permits-section-operators"));
    if !osm {
        blocks.push(not_read(lang, "osm"));
    } else {
        if notes.operators.is_empty() {
            blocks.push(t(lang, "permits-none-operators", &[]));
        } else {
            let lines: Vec<String> = notes
                .operators
                .iter()
                .map(|o| operator_line(lang, o))
                .collect();
            blocks.push(lines.join("\n"));
        }
        blocks.push(gap);
    }

    blocks.push(heading("permits-section-hints"));
    if notes.hints.is_empty() {
        blocks.push(t(lang, "permits-none-hints", &[]));
    } else {
        let lines: Vec<String> = notes
            .hints
            .iter()
            .map(|h| format!("- {}", hint_text(lang, notes, h)))
            .collect();
        blocks.push(lines.join("\n"));
    }

    blocks.push(heading("permits-section-sources"));
    blocks.push(sources_table(lang, notes));
    blocks.extend(regional_source_notes(lang, notes));

    let mut text = blocks.join("\n\n");
    text.push('\n');
    text
}

/// The `permits-*` ids this file uses: the literals of the code above the tests, and the hints
/// the engine stores.
#[cfg(test)]
fn used_ids() -> Vec<String> {
    // The scan stops at the first `#[cfg(test)]`, this function.
    let mut ids = freemapper_text::ids_in_source(include_str!("permits.rs"), &["permits-"]);
    ids.extend(
        [
            hint_key::CITY_STATE,
            hint_key::NATURE_AUTHORITY,
            hint_key::NATIONAL_PARK,
            hint_key::NSG_PATHS,
            hint_key::NATURA2000,
            hint_key::ORDER_OFFICE,
            hint_key::PARK_OPERATOR,
            hint_key::SCHOOL_OPERATOR,
            hint_key::DISTRICT_OFFICE,
            hint_key::MONUMENT_AUTHORITY,
            hint_key::NATURE_OBJECT_AUTHORITY,
            hint_key::OWNER_SHARE,
            hint_key::OWNERS_UNKNOWN,
        ]
        .map(String::from),
    );
    ids.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::TEXTS;
    use chrono::NaiveDate;
    use freemapper::permits::notes::{
        AdminEntry, AdminLevel, Hint, OperatorEntry, OperatorRole, OsmEntry, ProtectedEntry,
        ProtectedKind, Share, SourceStatus, hint_key,
    };
    use freemapper::project::MapKind;

    fn de() -> Lang {
        TEXTS.parse_lang_arg("de").unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    const BKG: &str = "© BKG (2026) dl-de/by-2-0, Datenquellen: \
        https://sg.geodatenzentrum.de/web_public/gdz/datenquellen/datenquellen_vg_nuts.pdf";
    const BFN: &str = "© Bundesamt für Naturschutz (BfN) (2026), Schutzgebiete in Deutschland; \
        Nutzungsbestimmungen für die Bereitstellung von Geodaten des Bundes (GeoNutzV). Nicht für \
        Planungszwecke geeignet.";
    const OSM: &str = "© OpenStreetMap contributors, ODbL";

    fn share<T>(item: T, percent: f64) -> Share<T> {
        Share {
            item,
            percent,
            at_edge: percent < 0.5,
        }
    }

    fn admin(level: AdminLevel, kind: &str, name: &str, percent: f64) -> Share<AdminEntry> {
        share(
            AdminEntry {
                level,
                name: name.into(),
                kind: kind.into(),
                ags: String::new(),
            },
            percent,
        )
    }

    fn protected(kind: ProtectedKind, name: &str, percent: f64) -> Share<ProtectedEntry> {
        share(
            ProtectedEntry {
                kind,
                name: name.into(),
                legal_date: None,
                legal_title: None,
                legal_link: None,
                site_link: None,
                year: Some(2024),
            },
            percent,
        )
    }

    fn hint(key: &str, args: &[(&str, &str)]) -> Hint {
        Hint {
            key: key.into(),
            args: args
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    fn source(id: &str, fetched: NaiveDate, credit: &str) -> SourceStatus {
        SourceStatus {
            id: id.into(),
            state: "read".into(),
            fetched: Some(fetched),
            reason: None,
            credit: Some(credit.into()),
        }
    }

    /// A sprint map at Buckow (Märkische Schweiz): every section has entries, one Gemeinde
    /// only touches the edge.
    pub(crate) fn buckow() -> PermitNotes {
        let mut nsg = protected(ProtectedKind::Naturschutzgebiet, "Buckower Kessel", 42.4);
        nsg.item.legal_date = Some(date(1966, 8, 1));
        nsg.item.legal_title =
            Some("Verordnung über das Naturschutzgebiet „Buckower Kessel“".into());
        nsg.item.legal_link =
            Some("https://bravors.brandenburg.de/verordnungen/nsg_buckower_kessel".into());
        PermitNotes {
            schema: 1,
            created: date(2026, 11, 13),
            map_kind: MapKind::Sprint,
            area_m2: 412_345.0,
            admin: vec![
                admin(AdminLevel::Land, "Land", "Brandenburg", 100.0),
                admin(AdminLevel::Kreis, "Landkreis", "Märkisch-Oderland", 100.0),
                admin(
                    AdminLevel::Verwaltungsgemeinschaft,
                    "Amt",
                    "Märkische Schweiz",
                    100.0,
                ),
                admin(
                    AdminLevel::Gemeinde,
                    "Stadt",
                    "Buckow (Märkische Schweiz)",
                    99.7,
                ),
                admin(AdminLevel::Gemeinde, "Gemeinde", "Waldsieversdorf", 0.3),
            ],
            protected: vec![
                nsg,
                protected(ProtectedKind::Ffh, "Buckower Kessel", 42.6),
                protected(
                    ProtectedKind::Landschaftsschutzgebiet,
                    "Märkische Schweiz",
                    100.0,
                ),
                protected(ProtectedKind::Naturpark, "Märkische Schweiz", 100.0),
            ],
            osm_protected: vec![share(
                OsmEntry {
                    osm_id: "way/123".into(),
                    object: "boundary=protected_area".into(),
                    name: Some("Alte Eichen".into()),
                    operator: Some("Stadt Buckow".into()),
                    protect_class: Some("4".into()),
                    protection_title: Some("Geschützter Landschaftsbestandteil".into()),
                },
                0.7,
            )],
            operators: vec![
                OperatorEntry {
                    name: "Evangelische Kirchengemeinde Buckow".into(),
                    roles: vec![OperatorRole::Owner],
                    objects: vec!["amenity=place_of_worship".into()],
                },
                OperatorEntry {
                    name: "Stadt Buckow".into(),
                    roles: vec![OperatorRole::Operator, OperatorRole::Owner],
                    objects: vec!["leisure=park".into(), "leisure=pitch".into()],
                },
            ],
            hints: vec![
                hint(
                    hint_key::NATURE_AUTHORITY,
                    &[("kreis", "Märkisch-Oderland"), ("kreisfrei", "no")],
                ),
                hint(hint_key::NSG_PATHS, &[]),
                hint(hint_key::NATURA2000, &[]),
                hint(
                    hint_key::ORDER_OFFICE,
                    &[
                        ("amt", "Amt Märkische Schweiz"),
                        ("gemeinde", "Buckow (Märkische Schweiz)"),
                    ],
                ),
                hint(hint_key::PARK_OPERATOR, &[("operator", "Stadt Buckow")]),
                hint(hint_key::OWNERS_UNKNOWN, &[]),
            ],
            sources: vec![
                source("vg250", date(2026, 11, 13), BKG),
                source("bfn", date(2026, 11, 13), BFN),
                source("osm", date(2026, 11, 6), OSM),
            ],
            regional: Vec::new(),
        }
    }

    /// The live answer for Sanssouci (Task 5): Potsdam, no protected area, the SPSG.
    pub(crate) fn sanssouci() -> PermitNotes {
        PermitNotes {
            schema: 1,
            created: date(2026, 11, 13),
            map_kind: MapKind::Sprint,
            area_m2: 1_032_000.0,
            admin: vec![
                admin(AdminLevel::Land, "Land", "Brandenburg", 100.0),
                admin(AdminLevel::Kreis, "Kreisfreie Stadt", "Potsdam", 100.0),
                admin(AdminLevel::Gemeinde, "Stadt", "Potsdam", 100.0),
            ],
            protected: vec![],
            osm_protected: vec![],
            operators: vec![OperatorEntry {
                name: "Stiftung Preußische Schlösser und Gärten Berlin-Brandenburg".into(),
                roles: vec![OperatorRole::Operator],
                objects: vec!["historic=building".into(), "leisure=park".into()],
            }],
            hints: vec![
                hint(hint_key::ORDER_OFFICE, &[("gemeinde", "Potsdam")]),
                hint(
                    hint_key::PARK_OPERATOR,
                    &[(
                        "operator",
                        "Stiftung Preußische Schlösser und Gärten Berlin-Brandenburg",
                    )],
                ),
                hint(hint_key::OWNERS_UNKNOWN, &[]),
            ],
            sources: vec![
                source("vg250", date(2026, 11, 13), BKG),
                source("bfn", date(2026, 11, 13), BFN),
                source("osm", date(2026, 11, 6), OSM),
            ],
            regional: Vec::new(),
        }
    }

    const LICENCE: &str = "Für die Nutzung der Daten ist die Datenlizenz Deutschland - Zero - \
        Version 2.0 anzuwenden. Die Lizenz ist über https://www.govdata.de/dl-de/zero-2-0 abrufbar.";
    const ANALOGUE: &str = "Nur diese amtlichen analogen Karten sind rechtsverbindlich.";

    fn regional_status(key: &str, what: &str, note: Option<&str>) -> RegionalStatus {
        RegionalStatus {
            source: freemapper::permits::regional::RegionalSource {
                key: key.into(),
                label: Words {
                    en: "Geoportal Berlin".into(),
                    de: "Geoportal Berlin".into(),
                },
                what: Words {
                    en: what.into(),
                    de: what.into(),
                },
                credit: LICENCE.into(),
                note: note.map(str::to_string),
                sections: Vec::new(),
            },
            status: SourceStatus {
                id: key.into(),
                ..source(key, date(2026, 11, 13), LICENCE)
            },
        }
    }

    /// A sprint map at the Viktoriapark, Berlin, as the fixtures of the Geoportal Berlin give
    /// it: two Bezirke, the park, land of the Land Berlin and of the Bund, monuments and trees.
    pub(crate) fn viktoriapark() -> PermitNotes {
        use freemapper::permits::regional::{
            DistrictEntry, GreenSpaces, MonumentGroup, Monuments, OwnerEntry,
        };
        let green = |name: &str, district: &str, percent: f64| {
            share(
                GreenSpaceEntry {
                    name: name.into(),
                    district: Some(district.into()),
                    dedication: Some("gewidmet".into()),
                },
                percent,
            )
        };
        let tree = |name: &str, count: usize| ProtectedObjectEntry {
            kind: ProtectedObjectKind::NaturalMonument,
            name: Some(name.into()),
            citation: Some(
                "Verordnung zum Schutz von Naturdenkmalen in Berlin vom 20.05.2021 (GVBl.S. 536)"
                    .into(),
            ),
            count,
            percent: None,
            at_edge: false,
        };
        let links = |n: usize, from: u32| -> Vec<String> {
            (0..n as u32)
                .map(|i| {
                    format!(
                        "https://denkmaldatenbank.berlin.de/daobj.php?obj_dok_nr={:08}",
                        9_030_000 + from + i
                    )
                })
                .collect()
        };
        PermitNotes {
            schema: 1,
            created: date(2026, 11, 13),
            map_kind: MapKind::Sprint,
            area_m2: 664_000.0,
            admin: vec![
                admin(AdminLevel::Land, "Land", "Berlin", 100.0),
                admin(AdminLevel::Kreis, "Kreisfreie Stadt", "Berlin", 100.0),
                admin(AdminLevel::Gemeinde, "Stadt", "Berlin", 100.0),
            ],
            protected: vec![],
            osm_protected: vec![],
            operators: vec![],
            hints: vec![
                hint(
                    hint_key::DISTRICT_OFFICE,
                    &[("district", "Friedrichshain-Kreuzberg")],
                ),
                hint(
                    hint_key::DISTRICT_OFFICE,
                    &[("district", "Tempelhof-Schöneberg")],
                ),
                hint(
                    hint_key::MONUMENT_AUTHORITY,
                    &[("district", "Friedrichshain-Kreuzberg")],
                ),
                hint(
                    hint_key::NATURE_OBJECT_AUTHORITY,
                    &[("district", "Friedrichshain-Kreuzberg")],
                ),
                hint(
                    hint_key::OWNER_SHARE,
                    &[("owner", "Land Berlin"), ("percent", "41.3")],
                ),
                hint(
                    hint_key::OWNER_SHARE,
                    &[("owner", "Bundesrepublik Deutschland"), ("percent", "0.4")],
                ),
                hint(hint_key::OWNERS_UNKNOWN, &[("rest", "yes")]),
            ],
            sources: vec![
                source("vg250", date(2026, 11, 13), BKG),
                source("bfn", date(2026, 11, 13), BFN),
                source("osm", date(2026, 11, 6), OSM),
            ],
            regional: vec![RegionalNotes {
                adapter: "berlin".into(),
                districts: Some(vec![
                    share(
                        DistrictEntry {
                            name: "Friedrichshain-Kreuzberg".into(),
                        },
                        86.0,
                    ),
                    share(
                        DistrictEntry {
                            name: "Tempelhof-Schöneberg".into(),
                        },
                        14.0,
                    ),
                ]),
                green_spaces: Some(GreenSpaces {
                    entries: vec![
                        green("Viktoriapark", "Friedrichshain-Kreuzberg", 18.7),
                        green("Dora-Duncker-Park", "Tempelhof-Schöneberg", 3.1),
                        green("Mehringdamm 90", "Friedrichshain-Kreuzberg", 0.3),
                    ],
                    playgrounds: 4,
                }),
                owners: Some(vec![
                    share(
                        OwnerEntry {
                            owner: "Land Berlin".into(),
                        },
                        41.3,
                    ),
                    share(
                        OwnerEntry {
                            owner: "Bundesrepublik Deutschland".into(),
                        },
                        0.4,
                    ),
                ]),
                monuments: Some(Monuments {
                    total: 42,
                    groups: vec![
                        MonumentGroup {
                            kind: "Baudenkmal".into(),
                            count: 29,
                            links: links(7, 0),
                        },
                        MonumentGroup {
                            kind: "Gesamtanlage".into(),
                            count: 8,
                            links: links(8, 100),
                        },
                        MonumentGroup {
                            kind: "Ensemble".into(),
                            count: 4,
                            links: links(4, 200),
                        },
                        MonumentGroup {
                            kind: "Gartendenkmal".into(),
                            count: 1,
                            links: vec![
                                "https://denkmaldatenbank.berlin.de/daobj.php?obj_dok_nr=09046157"
                                    .into(),
                            ],
                        },
                    ],
                    links_left: 22,
                }),
                protected_objects: Some(vec![
                    tree("Ahornblättrige Platane", 3),
                    tree("Amerikanischer Zürgelbaum", 3),
                    tree("Gewöhnliche Rosskastanie", 1),
                    tree("Weisse Maulbeere", 1),
                ]),
                sources: vec![
                    regional_status("bezirke", "Bezirksgrenzen (ALKIS Berlin)", None),
                    regional_status("gruenanlagen", "Grünanlagenbestand", None),
                    regional_status("liegenschaftsplan", "Liegenschaftsplan", None),
                    regional_status("denkmale", "Denkmale (Denkmalkarte)", None),
                    regional_status(
                        "schutzgebiete",
                        "Schutzobjekte nach Naturschutzrecht",
                        Some(ANALOGUE),
                    ),
                ],
            }],
        }
    }

    #[test]
    fn berlin_sections_follow_the_administrative_units() {
        let text = render_markdown(&viktoriapark(), &de());
        let at = |needle: &str| {
            text.find(needle)
                .unwrap_or_else(|| panic!("{needle}\n{text}"))
        };
        assert!(at("## Verwaltung") < at("## Bezirke (Geoportal Berlin)"));
        assert!(at("## Bezirke (Geoportal Berlin)") < at("## Grünanlagen (Geoportal Berlin)"));
        assert!(at("## Denkmale (Geoportal Berlin)") < at("## Schutzgebiete (BfN)"));
        assert!(
            text.contains("- Friedrichshain-Kreuzberg: 86 %\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "- „Viktoriapark“ (Bezirk Friedrichshain-Kreuzberg; Widmung: gewidmet): 19 %\n"
            ),
            "{text}"
        );
        assert!(
            text.contains("Im Kartengebiet liegen 4 öffentliche Spielplätze."),
            "{text}"
        );
        assert!(
            text.contains("- Land Berlin: 41 %\n- Bundesrepublik Deutschland: < 1 %\n"),
            "{text}"
        );
        assert!(text.contains("- Gartendenkmal: 1\n  - <https://denkmaldatenbank.berlin.de/daobj.php?obj_dok_nr=09046157>"), "{text}");
        assert!(text.contains("Weitere 22 Einträge"), "{text}");
        assert!(
            text.contains("- Naturdenkmal „Ahornblättrige Platane“, 3 Objekte\n"),
            "{text}"
        );
        assert_eq!(
            text.matches("Verordnung zum Schutz von Naturdenkmalen")
                .count(),
            1
        );
        // The hints name the Bezirksamt and the owners.
        assert!(
            text.contains("des Bezirksamts Friedrichshain-Kreuzberg zuständig."),
            "{text}"
        );
        assert!(
            text.contains("gehören 41 % des Kartengebiets dem Eigentümer „Land Berlin“."),
            "{text}"
        );
        assert!(
            text.contains(
                "Die Eigentümer der übrigen Flächen sind aus offenen Daten nicht bestimmbar."
            ),
            "{text}"
        );
        assert!(
            !text.contains("Den Bezirk bestimmen diese Hinweise noch nicht"),
            "{text}"
        );
    }

    #[test]
    fn the_geoportal_has_one_line_with_its_licence_verbatim() {
        for lang in [de(), Lang::english()] {
            let text = render_markdown(&viktoriapark(), &lang);
            let rows: Vec<&str> = text
                .lines()
                .filter(|l| l.starts_with("| Geoportal Berlin:"))
                .collect();
            assert_eq!(rows.len(), 1, "{text}");
            assert!(rows[0].contains(LICENCE), "{}", rows[0]);
            assert_eq!(column_bars(rows[0]), 4, "{}", rows[0]);
            assert!(
                rows[0].contains("Bezirksgrenzen (ALKIS Berlin), Grünanlagenbestand"),
                "{}",
                rows[0]
            );
            assert!(text.contains(ANALOGUE), "{text}");
        }
        // A service that was down is named in the line, and its section drops.
        let mut notes = viktoriapark();
        notes.regional[0].monuments = None;
        notes.regional[0].sources[3].status = SourceStatus {
            id: "denkmale".into(),
            state: "unreachable".into(),
            fetched: None,
            reason: Some("HTTP 503".into()),
            credit: None,
        };
        let text = render_markdown(&notes, &de());
        assert!(!text.contains("## Denkmale"), "{text}");
        assert!(
            text.contains("Denkmale (Denkmalkarte): nicht erreichbar (HTTP 503)"),
            "{text}"
        );
        assert!(
            text.contains("Bezirksgrenzen (ALKIS Berlin): gelesen, abgerufen am 2026-11-13"),
            "{text}"
        );
    }

    #[test]
    fn berlin_names_cannot_break_the_markdown() {
        let mut notes = viktoriapark();
        let part = &mut notes.regional[0];
        part.districts.as_mut().unwrap()[0].item.name = "A | *B*\n- fake".into();
        part.protected_objects.as_mut().unwrap()[0].citation = Some("[x](y) <b>".into());
        part.monuments.as_mut().unwrap().groups[0].kind = "# Kind".into();
        let text = render_markdown(&notes, &Lang::english());
        assert!(text.contains(r"- A \| \*B\* - fake: 86%"), "{text}");
        assert!(text.contains(r"- \[x\](y) \<b\>"), "{text}");
        assert!(text.contains(r"- \# Kind: 29"), "{text}");
        for line in text.lines() {
            assert!(!line.trim_start().starts_with("- fake"), "{line}");
        }
        assert!(!text.contains('—'), "{text}");
        assert!(!render_markdown(&viktoriapark(), &de()).contains('—'));
    }

    #[test]
    fn markdown_starts_with_the_disclaimer() {
        for lang in [Lang::english(), de()] {
            let text = render_markdown(&buckow(), &lang);
            let title = TEXTS.text(&lang, "permits-title", &[]);
            let disclaimer = TEXTS.text(&lang, "permits-disclaimer", &[]);
            assert!(
                text.starts_with(&format!("# {title}\n\n{disclaimer}\n\n")),
                "{text}"
            );
        }
        let german = render_markdown(&buckow(), &de());
        assert!(german.starts_with(
            "# Hinweise zu Genehmigungen\n\nHinweis, keine Rechtsauskunft. Maßgeblich sind die \
             zuständigen Behörden."
        ));
        let english = render_markdown(&buckow(), &Lang::english());
        assert!(english.starts_with("# Permit notes\n\nA hint, not legal advice."));
    }

    #[test]
    fn no_protected_area_is_said() {
        for (lang, said) in [
            (
                Lang::english(),
                "## Protected areas (BfN)\n\nAccording to the BfN, no protected area lies in the map area.\n",
            ),
            (
                de(),
                "## Schutzgebiete (BfN)\n\nIm Kartengebiet liegt laut BfN kein Schutzgebiet.\n",
            ),
        ] {
            let text = render_markdown(&sanssouci(), &lang);
            assert!(text.contains(said), "{text}");
        }
        // Each empty section says so in words.
        let text = render_markdown(&sanssouci(), &de());
        assert!(
            text.contains("OpenStreetMap nennt im Kartengebiet kein Schutzgebiet, das das BfN nicht schon führt."),
            "{text}"
        );
    }

    #[test]
    fn the_bfn_line_is_verbatim() {
        for lang in [Lang::english(), de()] {
            let text = render_markdown(&buckow(), &lang);
            assert!(text.contains(BFN), "{text}");
            // The disclaimer quotes the BfN in its own words, in every language.
            let disclaimer = text.split("\n\n").nth(1).unwrap();
            assert!(
                disclaimer.contains("Nicht für Planungszwecke geeignet."),
                "{disclaimer}"
            );
        }
        let english = render_markdown(&buckow(), &Lang::english());
        assert!(
            english.contains("(BfN: merged once a year, \"Nicht für Planungszwecke geeignet.\"")
        );
    }

    #[test]
    fn a_source_that_was_down_is_named() {
        let mut notes = buckow();
        notes.protected.clear();
        notes.sources[1] = SourceStatus {
            id: "bfn".into(),
            state: "unreachable".into(),
            fetched: None,
            reason: Some("HTTP 503 Service Unavailable".into()),
            credit: None,
        };
        notes.osm_protected.clear();
        notes.operators.clear();
        notes.sources[2] = SourceStatus {
            id: "osm".into(),
            state: "not-read".into(),
            fetched: None,
            reason: None,
            credit: None,
        };
        let text = render_markdown(&notes, &Lang::english());
        assert!(
            text.contains("| BfN: Protected areas in Germany of the BfN | not reachable (HTTP 503 Service Unavailable) | not used |"),
            "{text}"
        );
        assert!(
            text.contains("| OpenStreetMap: OpenStreetMap, from the extract freemapper keeps in its cache | not read | not used |"),
            "{text}"
        );
        // The empty sections do not claim there is nothing.
        assert!(
            text.contains(
                "## Protected areas (BfN)\n\nThe data of BfN were not read, so this section is empty, although there may be entries.\n"
            ),
            "{text}"
        );
        assert!(!text.contains("no protected area lies"), "{text}");
        assert!(!text.contains("names no operator"), "{text}");
        let german = render_markdown(&notes, &de());
        assert!(
            german.contains("| BfN: Schutzgebiete in Deutschland des BfN | nicht erreichbar (HTTP 503 Service Unavailable) | nicht verwendet |"),
            "{german}"
        );
    }

    #[test]
    fn shares_have_no_decimals_and_the_edge_has_its_own_lines() {
        let text = render_markdown(&buckow(), &de());
        assert!(
            text.contains("- Bundesland: Land Brandenburg, 100 %\n"),
            "{text}"
        );
        assert!(
            text.contains("- Gemeinde: Stadt Buckow (Märkische Schweiz), 100 %\n"),
            "{text}"
        );
        assert!(
            text.contains("Am Rand berührt (weniger als 0,5 % des Kartengebiets):\n\n- Gemeinde: Gemeinde Waldsieversdorf, < 1 %\n"),
            "{text}"
        );
        assert!(text.contains("„Alte Eichen“"), "{text}");
        assert!(
            text.contains("„Alte Eichen“, < 1 % (OSM way/123)"),
            "{text}"
        );
        let english = render_markdown(&buckow(), &Lang::english());
        assert!(
            english.contains("- State: Land Brandenburg, 100%\n"),
            "{english}"
        );
        assert!(
            english.contains("(less than 0.5% of the map area)"),
            "{english}"
        );
        assert!(
            english.contains("Nature reserve (Naturschutzgebiet) \"Buckower Kessel\", 42%"),
            "{english}"
        );
        assert!(english.contains("(41.2 ha)"), "{english}");
        assert!(render_markdown(&buckow(), &de()).contains("(41,2 ha)"));
    }

    #[test]
    fn the_hints_name_the_units() {
        let text = render_markdown(&buckow(), &de());
        assert!(
            text.contains("Schutzgebiete betreffen die untere Naturschutzbehörde (meist der Landkreis Märkisch-Oderland)."),
            "{text}"
        );
        assert!(
            text.contains("ist meist das Ordnungsamt zuständig: Amt Märkische Schweiz, für die Gemeinde Buckow (Märkische Schweiz)."),
            "{text}"
        );
        let potsdam = render_markdown(&sanssouci(), &de());
        assert!(
            potsdam.contains("ist meist das Ordnungsamt der Gemeinde Potsdam zuständig."),
            "{potsdam}"
        );
        // Without VG250 the nature authority stays general (ruling 2).
        let mut notes = buckow();
        notes.hints[0] = hint(hint_key::NATURE_AUTHORITY, &[]);
        let text = render_markdown(&notes, &de());
        assert!(
            text.contains(
                "die zuständige untere Naturschutzbehörde (meist Landkreis oder kreisfreie Stadt)"
            ),
            "{text}"
        );
        // A kreisfreie Stadt is its own authority.
        notes.hints[0] = hint(
            hint_key::NATURE_AUTHORITY,
            &[("kreis", "Potsdam"), ("kreisfrei", "yes")],
        );
        let text = render_markdown(&notes, &Lang::english());
        assert!(
            text.contains("Here that is the city of Potsdam itself."),
            "{text}"
        );
    }

    #[test]
    fn the_osm_sections_say_what_can_be_missing() {
        for lang in [Lang::english(), de()] {
            let text = render_markdown(&sanssouci(), &lang);
            let gap = TEXTS.text(&lang, "permits-osm-enclosing-gap", &[]);
            assert_eq!(text.matches(&gap).count(), 2, "{text}");
            // The extract reader keeps multipolygon relations only.
            assert!(gap.contains("`type=boundary`"), "{gap}");
        }
    }

    #[test]
    fn names_from_the_data_cannot_break_the_markdown() {
        let mut notes = sanssouci();
        notes.operators[0].name = "A | B *C* [x](y)".into();
        let text = render_markdown(&notes, &Lang::english());
        assert!(text.contains(r"A \| B \*C\* \[x\](y)"), "{text}");
    }

    #[test]
    fn a_forest_map_is_named_as_such() {
        let mut notes = buckow();
        notes.map_kind = MapKind::Forest;
        let german = render_markdown(&notes, &de());
        assert!(
            german.contains("für das Kartengebiet einer Waldkarte (41,2 ha)."),
            "{german}"
        );
        let english = render_markdown(&notes, &Lang::english());
        assert!(
            english.contains("for the map area of a forest map (41.2 ha)."),
            "{english}"
        );
    }

    /// The column bars of a table row that Markdown reads as bars (not escaped).
    fn column_bars(row: &str) -> usize {
        let mut bars = 0;
        let mut escaped = false;
        for c in row.chars() {
            if c == '|' && !escaped {
                bars += 1;
            }
            escaped = c == '\\' && !escaped;
        }
        bars
    }

    /// The reason of a failure can be text of the server (an OGC exception, a redirect
    /// target).
    #[test]
    fn a_failure_reason_stays_one_cell_of_plain_text() {
        let mut notes = buckow();
        notes.protected.clear();
        notes.sources[1] = SourceStatus {
            id: "bfn".into(),
            state: "unreachable".into(),
            fetched: None,
            reason: Some("a | b <b>bold</b>\r\nnext [x](y)".into()),
            credit: None,
        };
        for lang in [Lang::english(), de()] {
            let text = render_markdown(&notes, &lang);
            let rows: Vec<&str> = text.lines().filter(|l| l.starts_with("| BfN:")).collect();
            assert_eq!(rows.len(), 1, "{text}");
            assert_eq!(column_bars(rows[0]), 4, "{}", rows[0]);
            assert!(
                rows[0].contains(r"a \| b \<b\>bold\</b\>  next \[x\](y)"),
                "{}",
                rows[0]
            );
            assert!(!text.contains("<b>") && !text.contains("[x](y)"), "{text}");
        }
    }

    #[test]
    fn a_newline_in_a_name_starts_no_list_item() {
        let mut notes = buckow();
        notes.osm_protected[0].item.name = Some("Alte Eichen\n- fake".into());
        notes.operators[0].name = "Kirche\r\n- fake\n\n---".into();
        notes.operators[1].name = "- fake".into();
        for lang in [Lang::english(), de()] {
            let text = render_markdown(&notes, &lang);
            for line in text.lines() {
                let line = line.trim_start();
                assert!(!line.starts_with("- fake"), "{line}");
                assert!(!line.starts_with("- - "), "{line}");
                assert_ne!(line, "---", "{text}");
            }
            assert!(text.contains("Alte Eichen - fake"), "{text}");
        }
    }

    #[test]
    fn a_natura2000_page_is_no_legal_text() {
        let mut notes = buckow();
        notes.protected[1].item.site_link = Some("https://natura2000.example/DE3450301".into());
        let german = render_markdown(&notes, &de());
        assert!(
            german.contains(
                "  - Gebietsseite (Natura 2000): <https://natura2000.example/DE3450301>\n"
            ),
            "{german}"
        );
        let english = render_markdown(&notes, &Lang::english());
        assert!(
            english
                .contains("  - Site page (Natura 2000): <https://natura2000.example/DE3450301>\n"),
            "{english}"
        );
        // Only the NSG has a legal text.
        assert_eq!(english.matches("Legal text:").count(), 1, "{english}");
        assert_eq!(german.matches("Rechtstext:").count(), 1, "{german}");
    }

    #[test]
    fn no_em_dash_in_the_texts() {
        for lang in [Lang::english(), de()] {
            for notes in [buckow(), sanssouci()] {
                let text = render_markdown(&notes, &lang);
                assert!(!text.contains('—'), "{text}");
            }
        }
    }

    /// The `permits-*` messages are used by this file, and the hints of the engine have a text.
    #[test]
    fn every_permits_text_is_used() {
        let english = TEXTS.message_ids("en");
        let used = used_ids();
        for id in &used {
            assert!(english.contains(id), "{id} is used but not in the catalog");
        }
        for id in english.iter().filter(|id| id.starts_with("permits-")) {
            assert!(used.contains(id), "{id} is in the catalog but not used");
        }
        for key in [
            hint_key::CITY_STATE,
            hint_key::NATURE_AUTHORITY,
            hint_key::NATIONAL_PARK,
            hint_key::NSG_PATHS,
            hint_key::NATURA2000,
            hint_key::ORDER_OFFICE,
            hint_key::PARK_OPERATOR,
            hint_key::SCHOOL_OPERATOR,
            hint_key::DISTRICT_OFFICE,
            hint_key::MONUMENT_AUTHORITY,
            hint_key::NATURE_OBJECT_AUTHORITY,
            hint_key::OWNER_SHARE,
            hint_key::OWNERS_UNKNOWN,
        ] {
            assert!(english.contains(key), "{key}");
        }
    }

    /// Prints the two samples of the report: `cargo test -p freemapper-cli
    /// print_the_sample_notes -- --ignored --nocapture`.
    #[test]
    #[ignore = "a tool for the report, not a test"]
    fn print_the_sample_notes() {
        for lang in [de(), Lang::english()] {
            println!("{}\n-----", render_markdown(&buckow(), &lang));
        }
        for lang in [de(), Lang::english()] {
            println!("{}\n-----", render_markdown(&viktoriapark(), &lang));
        }
    }
}
