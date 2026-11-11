//! Consistency tests of the embedded page (`web/`): texts in both languages, ids and labels.
//! No JavaScript runner is needed; the tables are read as text.
#![cfg(test)]

use std::collections::BTreeSet;

fn asset(name: &str) -> String {
    let file = crate::select::Assets::get(name).unwrap_or_else(|| panic!("no asset {name}"));
    String::from_utf8(file.data.to_vec()).unwrap()
}

/// The keys of the language table `lang` (`de` or `en`) inside the object `table` (`TEXTS` or
/// `RUN_TEXTS`) of a script. Keys sit at four spaces of indentation.
fn keys_of(script: &str, table: &str, lang: &str) -> BTreeSet<String> {
    let start = script
        .find(&format!("const {table} = {{"))
        .unwrap_or_else(|| panic!("no table {table}"));
    let mut in_lang = false;
    let mut keys = BTreeSet::new();
    for line in script[start..].lines().skip(1) {
        if line == format!("  {lang}: {{") {
            in_lang = true;
        } else if in_lang && line == "  }," {
            return keys;
        } else if in_lang {
            let Some(rest) = line.strip_prefix("    ") else {
                continue;
            };
            if rest.starts_with(' ') {
                continue;
            }
            if let Some(end) = rest.find(':') {
                let key = rest[..end].trim_matches('"');
                if !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    keys.insert(key.to_string());
                }
            }
        }
    }
    panic!("table {table} has no language {lang}");
}

fn attribute_values(html: &str, attribute: &str) -> Vec<String> {
    let needle = format!("{attribute}=\"");
    html.match_indices(&needle)
        .filter(|(at, _)| {
            html[..*at]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace)
        })
        .map(|(at, _)| {
            let rest = &html[at + needle.len()..];
            rest[..rest.find('"').unwrap()].to_string()
        })
        .collect()
}

/// The part of the page that holds the run options (from `<section id="run-options"` to its end).
fn options_section(html: &str) -> &str {
    let start = html
        .find("<div id=\"run-options\"")
        .expect("block #run-options");
    let end = html[start..].find("<!-- end run-options -->").unwrap();
    &html[start..start + end]
}

#[test]
fn page_texts_have_the_same_keys_in_both_languages() {
    let app = asset("app.js");
    assert_eq!(keys_of(&app, "TEXTS", "de"), keys_of(&app, "TEXTS", "en"));
    assert!(keys_of(&app, "TEXTS", "de").len() > 40);
}

#[test]
fn run_texts_have_the_same_keys_in_both_languages() {
    let run = asset("run.js");
    let de = keys_of(&run, "RUN_TEXTS", "de");
    assert_eq!(de, keys_of(&run, "RUN_TEXTS", "en"));
    assert!(de.len() > 30, "{de:?}");
    // The two tables never share a key, so one cannot hide the other.
    let app = keys_of(&asset("app.js"), "TEXTS", "de");
    assert!(app.intersection(&de).next().is_none());
}

#[test]
fn every_text_key_and_placeholder_of_the_page_has_a_text() {
    let html = asset("index.html");
    let mut known = keys_of(&asset("app.js"), "TEXTS", "de");
    known.extend(keys_of(&asset("run.js"), "RUN_TEXTS", "de"));
    for attribute in ["data-text", "data-placeholder"] {
        for key in attribute_values(&html, attribute) {
            assert!(known.contains(&key), "{attribute}={key} has no text");
        }
    }
}

#[test]
fn every_id_the_scripts_use_exists_in_the_page() {
    let html = asset("index.html");
    let ids: BTreeSet<String> = attribute_values(&html, "id").into_iter().collect();
    for script in ["app.js", "run.js"] {
        let text = asset(script);
        for opener in ["$(\"", "$r(\""] {
            for (at, _) in text.match_indices(opener) {
                let rest = &text[at + opener.len()..];
                let id = &rest[..rest.find('"').unwrap()];
                assert!(ids.contains(id), "{script} uses #{id}, the page has none");
            }
        }
    }
}

#[test]
fn index_has_labels_for_every_option() {
    let html = asset("index.html");
    let section = options_section(&html);
    let ids = attribute_values(section, "id");
    for control in ["input", "select"] {
        for (at, _) in section.match_indices(&format!("<{control}")) {
            let tag = &section[at..at + section[at..].find('>').unwrap()];
            let id = attribute_values(tag, "id");
            assert_eq!(id.len(), 1, "{tag} needs an id");
            let wrapped = section[..at]
                .rfind("<label")
                .is_some_and(|open| !section[open..at].contains("</label>"));
            let by_for = section.contains(&format!("for=\"{}\"", id[0]));
            assert!(wrapped || by_for, "{tag} has no label");
        }
    }
    for option in [
        "run-ocad",
        "run-pdf",
        "run-pdf-candidates",
        "run-pdf-scale-bar",
        "run-pdf-sources",
        "run-offline",
        "run-force",
    ] {
        assert!(ids.contains(&option.to_string()), "{option}");
    }
}

#[test]
fn run_js_names_no_path_field() {
    let run = asset("run.js");
    let start = run
        .find("function optionsFromForm")
        .expect("optionsFromForm");
    let end = start + run[start..].find("\n}\n").unwrap();
    let body = &run[start..end];
    for word in ["dir", "path", "folder", "kind", "scale", "title"] {
        assert!(
            !body.contains(&format!("{word}:")) && !body.contains(&format!("\"{word}\"")),
            "optionsFromForm names {word}"
        );
    }
    for key in [
        "force",
        "offline",
        "ocad",
        "pdf",
        "pdf_candidates",
        "pdf_scale_bar",
        "pdf_sources",
    ] {
        assert!(body.contains(key), "{key}");
    }
}

#[test]
fn german_text_says_bundesland_not_land_for_states() {
    let app = asset("app.js");
    assert!(app.contains("Bundesländer"));
    assert!(!app.contains("unterstützten Länder"));
    let run = asset("run.js");
    assert!(!run.contains("Länder"));
}

#[test]
fn step_names_exist_for_all_steps_in_both_languages() {
    use crate::pipeline::Step;
    let run = asset("run.js");
    for step in [
        Step::Fetch,
        Step::Derive,
        Step::Map,
        Step::Ocad,
        Step::Pdf,
        Step::PdfCandidates,
    ] {
        let key = step.label_key();
        assert_eq!(run.matches(&format!("\"{key}\":")).count(), 2, "{key}");
    }
}

#[test]
fn progress_has_no_percent_and_is_announced() {
    let html = asset("index.html");
    let progress = html.find("<progress").expect("a progress element");
    let tag = &html[progress..progress + html[progress..].find('>').unwrap()];
    assert!(!tag.contains("value="), "{tag}");
    assert!(html.contains("aria-live=\"polite\""));
    let run = asset("run.js");
    assert!(
        !run.contains("run-bar"),
        "a script touches the progress bar"
    );
    assert!(!run.contains("innerHTML"));
}

#[test]
fn dialog_default_focus_is_cancel() {
    let html = asset("index.html");
    let dialog = &html[html.find("<dialog").expect("a dialog")..];
    let dialog = &dialog[..dialog.find("</dialog>").unwrap()];
    let cancel = dialog.find("id=\"force-cancel\"").unwrap();
    let tag_start = dialog[..cancel].rfind('<').unwrap();
    let tag = &dialog[tag_start..tag_start + dialog[tag_start..].find('>').unwrap()];
    assert!(tag.contains("autofocus"), "{tag}");
    assert!(!dialog[..tag_start].contains("autofocus"));
    assert!(dialog.contains("aria-labelledby"));
}

#[test]
fn page_has_no_inline_handlers_and_no_external_requests() {
    let html = asset("index.html");
    for handler in [" onclick=", " onchange=", " onload=", " onsubmit="] {
        assert!(!html.contains(handler), "{handler}");
    }
    assert!(!html.contains("<script>"));
    let run = asset("run.js");
    assert!(!run.contains("http://") && !run.contains("https://"));
    assert!(html.contains("assets/run.js"));
    assert!(!run.contains('@'));
}

#[test]
fn outside_text_uses_bundesland() {
    let app = asset("app.js");
    assert!(app.contains("außerhalb der unterstützten Bundesländer"));
}

#[test]
fn scales_come_from_the_server() {
    let app = asset("app.js");
    assert!(!app.contains("const SCALES"));
    assert!(app.contains("allowed_scales"));
}

#[test]
fn project_language_wins_over_the_browser_language() {
    let app = asset("app.js");
    assert!(app.contains("project.lang"));
    assert!(app.contains("navigator.language"));
}
