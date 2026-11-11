//! Consistency tests of the embedded page (`web/`): its texts in the catalog, ids and labels, the
//! suite's page rules. No JavaScript runner is needed; the files are read as text.
#![cfg(test)]

use std::collections::BTreeSet;

use crate::text::{Lang, TEXTS, page_texts};

fn asset(name: &str) -> String {
    let file = crate::select::Assets::get(name).unwrap_or_else(|| panic!("no asset {name}"));
    String::from_utf8(file.data.to_vec()).unwrap()
}

/// The page's own scripts.
const SCRIPTS: [&str; 2] = ["app.js", "run.js"];

/// The attributes through which the suite's `texts.js` puts a text into an element.
const TEXT_ATTRIBUTES: [&str; 4] = ["data-text", "data-label", "data-title", "data-placeholder"];

fn de() -> Lang {
    TEXTS.parse_lang_arg("de").unwrap()
}

/// The ids of the page's family (`page-*`) and the suite modules' (`suite-*`) in the catalog of
/// `code`.
fn page_ids(code: &str) -> BTreeSet<String> {
    TEXTS
        .message_ids(code)
        .into_iter()
        .filter(|id| id.starts_with("page-") || id.starts_with("suite-"))
        .collect()
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

/// The text ids written as string literals in a script: what `t`, `tn` and the id tables name.
fn script_ids(script: &str) -> BTreeSet<String> {
    freemapper_text::ids_in_source(script, &["page-", "suite-"])
}

/// Whether the catalog has `id`, or the two messages `tn` picks from for it.
fn has_text(ids: &BTreeSet<String>, id: &str) -> bool {
    ids.contains(id) || (ids.contains(&format!("{id}-one")) && ids.contains(&format!("{id}-other")))
}

/// The kinds of the files a run reports (`pipeline.rs`); the run panel names each with
/// `page-file-kind-<kind>`.
const FILE_KINDS: [&str; 6] = [
    "map",
    "template",
    "ocad",
    "ocad-candidates",
    "pdf",
    "pdf-candidates",
];

/// The steps of a run; the run panel names each with `page-<label key>`.
fn step_ids() -> Vec<String> {
    use crate::pipeline::Step;
    [
        Step::Fetch,
        Step::Derive,
        Step::Map,
        Step::Ocad,
        Step::Pdf,
        Step::PdfCandidates,
    ]
    .into_iter()
    .map(|step| format!("page-{}", step.label_key()))
    .collect()
}

#[test]
fn page_texts_have_the_same_ids_in_both_languages() {
    let english = page_ids("en");
    assert_eq!(english, page_ids("de"));
    assert!(english.len() > 130, "{}", english.len());
    assert!(english.contains("suite-banner-dismiss"));
}

#[test]
fn page_messages_have_no_selectors_and_no_functions() {
    // The page fills in `{ $name }` itself (suite texts.js): a selector or a function would reach
    // it unformatted.
    for source in [
        include_str!("../i18n/en/freemapper.ftl"),
        include_str!("../i18n/de/freemapper.ftl"),
    ] {
        let mut current = "";
        for line in source.lines() {
            if !line.starts_with([' ', '#']) && !line.is_empty() {
                current = line.split('=').next().unwrap_or_default().trim();
            }
            if current.starts_with("page-") || current.starts_with("suite-") {
                for banned in ["->", "NUMBER(", "DATETIME(", "{ -", "{-"] {
                    assert!(!line.contains(banned), "{current}: {line}");
                }
            }
        }
    }
}

#[test]
fn every_text_id_of_the_page_is_in_the_catalog() {
    let html = asset("index.html");
    let ids = page_ids("en");
    let mut seen = 0;
    for attribute in TEXT_ATTRIBUTES {
        for id in attribute_values(&html, attribute) {
            assert!(ids.contains(&id), "{attribute}={id} has no text");
            seen += 1;
        }
    }
    assert!(seen > 45, "only {seen} text ids in the page");
    assert!(html.contains(r#"<title data-text="page-title">"#));
}

#[test]
fn every_text_id_the_scripts_use_is_in_the_catalog() {
    let ids = page_ids("en");
    let mut seen = 0;
    for name in SCRIPTS {
        for id in script_ids(&asset(name)) {
            assert!(
                has_text(&ids, &id),
                "{name} uses {id}, the catalog has none"
            );
            seen += 1;
        }
    }
    assert!(seen > 60, "only {seen} ids in the scripts");
    for id in step_ids().into_iter().chain(
        FILE_KINDS
            .iter()
            .map(|kind| format!("page-file-kind-{kind}")),
    ) {
        assert!(ids.contains(&id), "{id}");
    }
    let pipeline = include_str!("pipeline.rs");
    for kind in FILE_KINDS {
        assert!(pipeline.contains(&format!("(\"{kind}\"")), "{kind}");
    }
}

#[test]
fn every_page_message_is_used() {
    let mut used: BTreeSet<String> = BTreeSet::new();
    let html = asset("index.html");
    for attribute in TEXT_ATTRIBUTES {
        used.extend(attribute_values(&html, attribute));
    }
    for name in SCRIPTS {
        used.extend(script_ids(&asset(name)));
    }
    for (name, bytes) in freemapper_web::suite_assets() {
        if name.ends_with(".js") {
            used.extend(script_ids(&String::from_utf8(bytes).unwrap()));
        }
    }
    // The server's own sentences for the page, such as a stale save.
    used.extend(freemapper_text::ids_in_source(
        include_str!("select.rs"),
        &["page-"],
    ));
    used.extend(step_ids());
    used.extend(
        FILE_KINDS
            .iter()
            .map(|kind| format!("page-file-kind-{kind}")),
    );
    for id in used.clone() {
        used.insert(format!("{id}-one"));
        used.insert(format!("{id}-other"));
    }
    for id in page_ids("en") {
        assert!(used.contains(&id), "{id} is in the catalog but not used");
    }
}

#[test]
fn every_id_the_scripts_use_exists_in_the_page() {
    let html = asset("index.html");
    let ids: BTreeSet<String> = attribute_values(&html, "id").into_iter().collect();
    for script in SCRIPTS {
        let text = asset(script);
        for opener in ["$(\"", "$r(\""] {
            for (at, _) in text.match_indices(opener) {
                let rest = &text[at + opener.len()..];
                let id = &rest[..rest.find('"').unwrap()];
                assert!(ids.contains(id), "{script} uses #{id}, the page has none");
            }
        }
    }
    // The suite's status module writes into these two.
    assert!(html.contains(r#"<div id="banner" role="alert">"#));
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
    let texts = page_texts(&de());
    assert!(
        texts["page-outside"].contains("außerhalb der unterstützten Bundesländer"),
        "{}",
        texts["page-outside"]
    );
    assert!(texts["page-why-outside"].contains("Bundesländer"));
    for (id, text) in &texts {
        assert!(!text.contains("Länder"), "{id}: {text}");
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
fn scales_come_from_the_server() {
    let app = asset("app.js");
    assert!(!app.contains("const SCALES"));
    assert!(app.contains("allowed_scales"));
}

/// The body of the top-level function `name` in `script`, up to its closing brace in column 0.
fn function_body<'a>(script: &'a str, name: &str) -> &'a str {
    let start = script
        .find(&format!("function {name}("))
        .unwrap_or_else(|| panic!("no function {name}"));
    let rest = &script[start..];
    &rest[..rest.find("\n}\n").expect("the function ends")]
}

/// The hint under the drawing buttons, the layer names of the map and the update notice are set by
/// the scripts, not by a `data-*` attribute. They were once written in the browser's language and
/// stayed German on an English page. Now every word comes from `api/texts` in the one language the
/// server chose: no script and no element carries a text of its own, nothing picks a language on
/// the page, and the script-set texts are written only after the texts are loaded.
#[test]
fn script_set_texts_come_from_the_catalog_in_the_page_language() {
    let html = asset("index.html");
    let scripts: Vec<(&str, String)> = SCRIPTS.iter().map(|name| (*name, asset(name))).collect();
    // No word of the page is written into a script or an element, in either language.
    for lang in [Lang::english(), de()] {
        for (id, text) in page_texts(&lang) {
            if text.contains("{ $") || !text.contains(' ') || text.chars().count() < 12 {
                continue;
            }
            for (name, script) in &scripts {
                assert!(
                    !script.contains(&text),
                    "{name} holds the text of {id}: {text}"
                );
            }
            assert!(
                !html.contains(&text),
                "index.html holds the text of {id}: {text}"
            );
        }
    }
    for (name, script) in &scripts {
        for own in ["TEXTS", "navigator.language", "setLanguage", "applyTexts"] {
            assert!(
                !script.contains(own),
                "{name} picks its own language: {own}"
            );
        }
        assert!(script.contains("../suite/texts.js"), "{name}");
    }
    // The texts are loaded before the page writes any of them.
    let app = &scripts[0].1;
    let start = function_body(app, "start");
    let loaded = start
        .find("await loadTexts()")
        .expect("start loads the texts");
    for call in [
        "setMode(null)",
        "showLayerNames()",
        "showUpdateNotice()",
        "api(\"api/project\")",
    ] {
        let at = start
            .find(call)
            .unwrap_or_else(|| panic!("start calls {call}"));
        assert!(loaded < at, "{call} comes before loadTexts()");
    }
    // The layer control is built once, in start, from the catalog.
    assert_eq!(app.matches("showLayerNames();").count(), 1);
    assert!(function_body(app, "showLayerNames").contains("t(\"page-layer-osm\")"));
    assert!(function_body(app, "showHint").contains("\"page-hint-idle\""));
    assert!(function_body(app, "showUpdateNotice").contains("t(\"page-update-available\""));
    // start runs after both modules are loaded.
    assert!(app.contains("document.addEventListener(\"DOMContentLoaded\", start)"));
}

#[test]
fn every_server_call_goes_through_the_suite_api() {
    for name in SCRIPTS {
        let script = asset(name);
        assert!(!script.contains("fetch("), "{name} calls fetch itself");
        assert!(!script.contains("XMLHttpRequest"), "{name}");
    }
    let app = asset("app.js");
    assert!(app.contains("from \"../suite/api.js\""));
    // The save names the revision of freemapper.toml the page last read.
    assert!(function_body(&app, "saveArea").contains("project.revision"));
}

#[test]
fn page_follows_the_suite_rules() {
    // The page's own files and the suite's, without the vendored Leaflet (its own limit below).
    let mut files: Vec<(String, Vec<u8>)> = crate::select::Assets::iter()
        .filter(|name| !name.starts_with("vendor/"))
        .map(|name| {
            let data = crate::select::Assets::get(&name).unwrap().data.into_owned();
            (name.into_owned(), data)
        })
        .collect();
    assert!(files.iter().any(|(name, _)| name == "index.html"));
    files.extend(freemapper_web::suite_assets());
    let pairs: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    if let Err(problems) = freemapper_web::check_page_rules(&pairs, 300 * 1024) {
        panic!("{}", problems.join("\n"));
    }
    // Leaflet 1.9.4 with its CSS, images and licence.
    let vendored: usize = crate::select::Assets::iter()
        .filter(|name| name.starts_with("vendor/"))
        .map(|name| crate::select::Assets::get(&name).unwrap().data.len())
        .sum();
    assert!(vendored < 300 * 1024, "{vendored} bytes");
}
