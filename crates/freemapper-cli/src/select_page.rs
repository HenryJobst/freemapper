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
const FILE_KINDS: [&str; 8] = [
    "map",
    "preview",
    "template",
    "ocad",
    "ocad-candidates",
    "pdf",
    "pdf-candidates",
    "permits",
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
        Step::Permits,
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
        "permits",
    ] {
        assert!(body.contains(key), "{key}");
    }
}

/// The permit notes are an output switch like the PDF and OCAD ones: off unless the page was
/// opened with them, and sent as `permits` only as the state of the box (ADR 0116).
#[test]
fn the_permits_box_is_an_output_switch() {
    let html = asset("index.html");
    let options = options_section(&html);
    let at = options
        .find("<input id=\"run-permits\"")
        .expect("a box run-permits among the run options");
    let tag = &options[at..at + options[at..].find('>').unwrap()];
    assert!(tag.contains("type=\"checkbox\""), "{tag}");
    assert!(!tag.contains("checked"), "off by default: {tag}");
    let label = &options[at..at + options[at..].find("</label>").unwrap()];
    assert!(label.contains("data-text=\"page-opt-permits\""), "{label}");
    let run = asset("run.js");
    let start = run.find("function optionsFromForm").unwrap();
    let body = &run[start..start + run[start..].find("\n}\n").unwrap()];
    assert!(
        body.contains("permits: field(\"permits\").checked"),
        "{body}"
    );
    assert!(
        run.contains("$r(\"run-permits\").checked = !!defaults.permits;"),
        "the box starts as the page was opened"
    );
    assert_eq!(
        TEXTS.text(&Lang::english(), "page-opt-permits", &[]),
        "Permit notes"
    );
    assert_eq!(
        TEXTS.text(&de(), "page-opt-permits", &[]),
        "Hinweise zu Genehmigungen"
    );
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

/// After a stale save the run panel must not save on its own: it would replace what another window
/// wrote without the person deciding. Only a click on "Save" clears the conflict.
#[test]
fn a_conflict_blocks_the_save_before_a_run() {
    let app = asset("app.js");
    let ensure = function_body(&app, "ensureSaved");
    let check = ensure
        .find("if (conflict) return false;")
        .expect("ensureSaved checks conflict");
    assert!(check < ensure.find("saveArea(").unwrap());
    assert!(app.contains(r#"$("save").addEventListener("click", () => saveArea(true));"#));
    let save = function_body(&app, "saveArea");
    assert!(save.contains("if (explicit) conflict = false;"));
    assert!(save.contains("showConflict(e.message)"));
    // A save whose revision is not the one read right after it lost to another writer.
    assert!(save.contains("answer.revision !== project.revision"));
    assert!(function_body(&app, "showConflict").contains("conflict = true;"));
    // The declaration and the explicit save, nothing else clears it.
    assert_eq!(app.matches("conflict = false;").count(), 2);
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

#[test]
fn content_names_cover_the_library() {
    use freemapper::build::content::{Group, Place, Profile};
    // The script lists the ids literally (so `every_page_message_is_used` sees them); a group,
    // profile or place that the library gains must get its texts here first.
    let script = script_ids(&asset("app.js"));
    let catalog_en = page_ids("en");
    let catalog_de = page_ids("de");
    let mut wanted: Vec<String> = Group::ALL
        .iter()
        .map(|g| format!("page-content-group-{}", g.name()))
        .collect();
    wanted.extend(
        Place::ALL
            .iter()
            .map(|p| format!("page-content-place-{}", p.name())),
    );
    for profile in Profile::ALL {
        wanted.push(format!("page-content-profile-{}", profile.name()));
        wanted.push(format!("page-content-profile-{}-hint", profile.name()));
    }
    wanted.push("page-content-place-profile".to_string());
    for id in wanted {
        assert!(script.contains(&id), "app.js does not name {id}");
        assert!(catalog_en.contains(&id) && catalog_de.contains(&id), "{id}");
    }
}

#[test]
fn content_section_is_in_the_run_panel_before_the_start_button() {
    let html = asset("index.html");
    let run = html.find(r#"<section id="run""#).unwrap();
    let section = html.find(r#"<details id="content-section">"#).unwrap();
    let start = html.find(r#"id="run-start""#).unwrap();
    assert!(run < section && section < start);
    for id in ["content-profile", "content-groups"] {
        assert!(html.contains(&format!(r#"<fieldset id="{id}">"#)), "{id}");
    }
    // Every control the script builds has a label: the profile radios and the group selects sit
    // inside a <label>.
    let script = asset("app.js");
    assert!(script.contains("label.append(radio, text)"));
    assert!(script.contains("label.append(text, select)"));
}

#[test]
fn preview_section_is_in_the_run_panel_below_the_result() {
    let html = asset("index.html");
    let run = html.find(r#"<section id="run""#).unwrap();
    let result = html.find(r#"id="run-result""#).unwrap();
    let section = html.find(r#"<details id="preview-section""#).unwrap();
    let end = html[run..].find("</section>").unwrap() + run;
    assert!(run < result && result < section && section < end);
    for id in ["preview-state", "preview-image", "preview-open"] {
        assert!(html.contains(&format!(r#"id="{id}""#)), "{id}");
    }
    // The page asks the server for the images through its own routes only (the policy allows
    // nothing else), and the link opens in a new tab without giving the page away.
    let link = &html[html.find(r#"<a id="preview-open""#).unwrap()..];
    let link = &link[..link.find('>').unwrap()];
    assert!(link.contains(r#"rel="noopener""#), "{link}");
}

#[test]
fn preview_uses_only_the_pages_own_image_routes() {
    let run = asset("run.js");
    assert!(run.contains("api/map-image/state"));
    assert!(run.contains("api/map-image/live/"));
    assert!(!run.contains("blob:") && !run.contains("createObjectURL"));
    // A live image is not asked for while a run starts or goes on (a page opened meanwhile shows
    // the stored one), nor while the drawn area is not saved: both branches end before the
    // request.
    let during_run = run
        .find("if (running || starting) {\n    if (previewImage().hidden) {\n      previewShowStored(\"page-preview-run\");")
        .expect("the branch for a run");
    let unsaved_area = run
        .find("if (!page().areaIsSaved()) {\n    previewShowStored(\"page-preview-area-unsaved\");\n    return;\n  }")
        .expect("the branch for an unsaved area");
    let request = run.find("await requestLivePreview();").unwrap();
    assert!(during_run < unsaved_area && unsaved_area < request);
}

#[test]
fn a_save_sends_the_area_as_it_came_in() {
    let app = asset("app.js");
    assert!(app.contains("area: areaInput || area"));
}
