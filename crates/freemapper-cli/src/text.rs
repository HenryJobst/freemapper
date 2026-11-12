//! Program texts from Fluent files (ADR 0092).
//!
//! The program independent part (languages, lookup with English fallback, language detection,
//! catalog checks) is `freemapper-text` of freemapper-core. This file holds the embedded catalogs
//! and the language of this run.
//!
//! Every text lives in `i18n/<language>/freemapper.ftl`. English is the reference and the
//! fallback: a message missing in another language is taken from English, and a message missing
//! everywhere comes back as its id.
//!
//! The language is the first of these that names an available language: `--lang`,
//! `FREEMAPPER_LANG`, `LC_ALL`, `LC_MESSAGES`, `LANG`; else English. Only the language part
//! counts (`de`, `de-DE` and `de_DE.UTF-8` all mean German). A `--lang` value that names no
//! available language is a usage error.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use freemapper_text::Catalog;
pub use freemapper_text::{FluentValue, Lang};

/// The English catalog, the reference.
const ENGLISH: &str = include_str!("../i18n/en/freemapper.ftl");

/// The embedded catalogs: language code and file content. English comes first.
pub static TEXTS: Catalog = Catalog::new(&[
    ("en", ENGLISH),
    ("de", include_str!("../i18n/de/freemapper.ftl")),
]);

/// The language of this run, set once at start.
static LANG: OnceLock<Lang> = OnceLock::new();

/// Whether `--lang` or the environment named the language of this run; else it is English
/// because nothing said otherwise.
static NAMED: OnceLock<bool> = OnceLock::new();

/// The variables that can name the language, in the suite's order.
const LANG_VARIABLES: [&str; 4] = ["FREEMAPPER_LANG", "LC_ALL", "LC_MESSAGES", "LANG"];

/// Whether `arg` or one of the variables read by `env` names a language with texts.
fn names_a_lang(arg: Option<&str>, env: &dyn Fn(&str) -> Option<String>) -> bool {
    arg.is_some()
        || LANG_VARIABLES
            .iter()
            .filter_map(|name| env(name))
            .any(|value| TEXTS.parse_lang_arg(&value).is_ok())
}

/// Sets the language of this run from `--lang` (`arg`) and the environment. A `--lang` value
/// that names no available language is an error, worded in the language the environment gives.
pub fn set_lang(arg: Option<&str>) -> Result<(), String> {
    let env = |name: &str| std::env::var(name).ok();
    let lang = match arg {
        Some(value) => TEXTS.parse_lang_arg(value).map_err(|available| {
            let available: Vec<String> = available.iter().map(Lang::code).collect();
            TEXTS.text(
                &TEXTS.detect_lang(None, &env),
                "cli-lang-unknown",
                &[
                    ("value", FluentValue::from(value.to_owned())),
                    ("available", FluentValue::from(available.join(", "))),
                ],
            )
        })?,
        None => TEXTS.detect_lang(None, &env),
    };
    // A second call keeps the first language; the program calls this once.
    let _ = LANG.set(lang);
    let _ = NAMED.set(names_a_lang(arg, &env));
    Ok(())
}

/// The language of this run; English before [`set_lang`].
pub fn lang() -> Lang {
    LANG.get().cloned().unwrap_or_else(Lang::english)
}

/// The language of this run when `--lang` or the environment named it. The selection page takes
/// it over; without one it follows the browser ([`page_lang`]).
pub fn named_lang() -> Option<Lang> {
    NAMED.get().copied().unwrap_or(false).then(lang)
}

/// The language of the area page's texts (core ADR 0030, the rule of ADR 0092): the language
/// `--lang` or the environment named ([`named_lang`]), else the browser's from `accept_language`,
/// else English.
pub fn page_lang(accept_language: Option<&str>) -> Lang {
    let fixed = named_lang().map(|lang| lang.code());
    let code = freemapper_web::texts_language(fixed.as_deref(), accept_language);
    TEXTS
        .parse_lang_arg(code)
        .unwrap_or_else(|_| Lang::english())
}

/// The area page's messages (`page-*`) and those of the suite's page modules (`suite-*`) in
/// `lang`, for `api/texts` (ADR 0096). A value stays as `{ $name }` in the text; the page fills it
/// in. These messages have no selectors, so every variable is such a placeholder.
pub fn page_texts(lang: &Lang) -> BTreeMap<String, String> {
    TEXTS
        .message_ids("en")
        .into_iter()
        .filter(|id| id.starts_with("page-") || id.starts_with("suite-"))
        .map(|id| {
            let names = variables_of(ENGLISH, &id);
            let placeholders: Vec<(&str, FluentValue)> = names
                .iter()
                .map(|name| (name.as_str(), v(format!("{{ ${name} }}"))))
                .collect();
            let text = TEXTS.text(lang, &id, &placeholders);
            (id, text)
        })
        .collect()
}

/// The variables (`$name`) of the message `id` in the Fluent `source`: its first line and the
/// indented lines after it.
fn variables_of(source: &str, id: &str) -> Vec<String> {
    let mut lines = source.lines().skip_while(|line| {
        line.split_once('=')
            .is_none_or(|(name, _)| name.trim_end() != id || name.starts_with(' '))
    });
    let mut message = lines.next().unwrap_or_default().to_owned();
    for line in lines.take_while(|line| line.starts_with(' ')) {
        message.push_str(line);
    }
    let mut names = Vec::new();
    for (at, _) in message.match_indices('$') {
        let name: String = message[at + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .collect();
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// The text `id` in the language of this run.
pub fn text(id: &str, args: &[(&str, FluentValue)]) -> String {
    TEXTS.text(&lang(), id, args)
}

/// The names of the supported states in `lang`, in the order of the registry and joined by
/// commas: for a list in parentheses.
pub fn states(lang: &Lang) -> String {
    freemapper::adapters::region_names(lang.code() == "de").join(", ")
}

/// [`states`] for the middle of a sentence: the last name follows the word for "and".
pub fn states_in_a_sentence(lang: &Lang) -> String {
    let names = freemapper::adapters::region_names(lang.code() == "de");
    match names.split_last() {
        Some((last, first)) if !first.is_empty() => TEXTS.text(
            lang,
            "cli-list-and",
            &[("first", v(first.join(", "))), ("last", v(last))],
        ),
        _ => names.join(", "),
    }
}

/// `body` as a warning line in the language of this run, with the prefix every warning of the
/// program has ("warning: …"), from the engine's catalog (`note-warning`).
pub fn warning(body: &str) -> String {
    freemapper::note::NOTES.text(&lang(), "note-warning", &[("text", v(body))])
}

/// A path, a name or another text as a message variable.
pub fn v(value: impl ToString) -> FluentValue<'static> {
    FluentValue::from(value.to_string())
}

/// A count as a message variable; the catalogs pick singular or plural by it.
pub fn n(count: usize) -> FluentValue<'static> {
    FluentValue::from(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de() -> Lang {
        TEXTS.parse_lang_arg("de").unwrap()
    }

    #[test]
    fn the_catalogs_are_complete_and_use_the_same_variables() {
        let sample = [
            ("path", v("a/b")),
            ("count", n(2)),
            ("skipped", n(1)),
            ("dropped", n(3)),
            ("name", v("n")),
            ("id", v("way/1")),
            ("query", v("q")),
            ("error", v("e")),
            ("value", v("fr")),
            ("available", v("en, de")),
            ("time", v("14:02")),
            ("who", v("the terminal")),
            ("job", v("job-1")),
            ("messages", v("m")),
            ("program", v("open")),
            ("files", v("a.omap")),
            ("message", v("m")),
            ("scale", v(4000)),
            ("kind", v("k")),
            ("allowed", v("1:4000")),
            ("tiles", v("33421-5821")),
            ("width", v(1)),
            ("height", v(1)),
            ("limit", v(1)),
            ("part", v("p")),
            ("needed", v("1 MB")),
            ("left", v("1 MB")),
            ("reserve", v("1 GB")),
            ("url", v("u")),
            ("product", v("p")),
            ("status", v(404)),
            ("detail", v("d")),
            ("reason", v("r")),
            ("hosts", v("geo.example.org")),
            ("folder", v("/adapters/x")),
            ("finding", v("f")),
            ("others", v("berlin")),
            ("capability", v("rasters")),
            ("layers", n(6)),
            ("source", v("built in")),
            ("revision", v(1)),
            ("checked", v("2026-11-12")),
            ("target", v("https://x.example.org/y: r")),
            ("label", v("l")),
            ("range", v("0-1")),
            ("line", v(1)),
            ("parts", v(2)),
            ("lat", v(1)),
            ("lon", v(1)),
            ("date", v("2026-01-01")),
            ("easting", v(1)),
            ("northing", v(1)),
            ("symbol", v(1)),
            ("arg", v("--x")),
            ("job", v("job-1")),
            ("area", v("way/1")),
            ("layer", v("ndsm")),
            ("args", v("<DIR>")),
            ("values", v("a, b")),
            ("other", v("--y")),
            ("new", v("0.12.0")),
            ("old", v("0.11.0")),
            ("released", v("2026-11-20")),
            (
                "notes",
                v("https://github.com/HenryJobst/freemapper/releases/tag/v0.12.0"),
            ),
            ("size", v("11.8")),
            ("version", v("0.12.0")),
            ("mode", v("notify")),
            ("platform", v("linux-riscv")),
            (
                "page",
                v("https://github.com/HenryJobst/freemapper/releases/latest"),
            ),
            // The area page's patterns (`page-*`); the page fills them in itself.
            ("n", n(2)),
            ("names", v("33421-5821, 33421-5822")),
            ("mb", v(350)),
            ("volumes", v("1 GB free, about 350 MB needed")),
            ("free", v("1 GB")),
            ("max", v(1500)),
            ("dir", v("/p")),
            ("index", v(1)),
            ("total", v(3)),
            // The content setting (`--content`).
            ("summary", v("balanced, walls=map")),
            ("detail", v("unknown profile `huge`")),
            // `freemapper find`.
            ("category", v("amenity")),
            ("place", v("Strausberg")),
            // The supported states, from the registry.
            ("states", v("Brandenburg, Berlin")),
            ("first", v("Brandenburg")),
            ("last", v("Berlin")),
            ("state", v("Brandenburg/Berlin")),
            ("agency", v("LGB")),
            ("adapter", v("czfoo")),
        ];
        assert_eq!(TEXTS.problems(&sample), Vec::<String>::new());
    }

    /// The id families the Rust code names: `page-` for the area page's sentences the server
    /// writes (a stale save), the others as in `i18n/README.md`.
    const PREFIXES: [&str; 6] = ["cli-", "error-", "word-", "mcp-", "update-", "page-"];

    #[test]
    fn every_id_used_in_the_code_exists() {
        let english = TEXTS.message_ids("en");
        for source in [
            include_str!("main.rs"),
            include_str!("text.rs"),
            include_str!("lock.rs"),
            include_str!("select.rs"),
            include_str!("select_run.rs"),
            include_str!("adapters.rs"),
            include_str!("adapter_check.rs"),
            include_str!("errors.rs"),
            include_str!("find.rs"),
            include_str!("pipeline.rs"),
            include_str!("estimate.rs"),
            include_str!("help.rs"),
            include_str!("mcp.rs"),
            include_str!("jobs.rs"),
            include_str!("update/mod.rs"),
            include_str!("update/background.rs"),
            include_str!("update/command.rs"),
            include_str!("update/source.rs"),
        ] {
            for id in freemapper_text::ids_in_source(source, &PREFIXES) {
                assert!(english.contains(&id), "{id} is used but not in the catalog");
            }
        }
        // And nothing is left over.
        let used: std::collections::BTreeSet<String> = [
            include_str!("main.rs"),
            include_str!("text.rs"),
            include_str!("lock.rs"),
            include_str!("select.rs"),
            include_str!("select_run.rs"),
            include_str!("adapters.rs"),
            include_str!("adapter_check.rs"),
            include_str!("errors.rs"),
            include_str!("find.rs"),
            include_str!("pipeline.rs"),
            include_str!("estimate.rs"),
            include_str!("help.rs"),
            include_str!("mcp.rs"),
            include_str!("jobs.rs"),
            include_str!("update/mod.rs"),
            include_str!("update/background.rs"),
            include_str!("update/command.rs"),
            include_str!("update/source.rs"),
        ]
        .iter()
        .flat_map(|source| freemapper_text::ids_in_source(source, &PREFIXES))
        .collect();
        // The help and the MCP descriptions are looked up by name; `help.rs` and `mcp.rs` test
        // those. The area page's texts are used by its scripts; `select_page.rs` tests those.
        for id in english.iter().filter(|id| {
            !["help-", "mcp-tool-", "mcp-param-", "page-", "suite-"]
                .iter()
                .any(|p| id.starts_with(p))
        }) {
            assert!(used.contains(id), "{id} is in the catalog but not used");
        }
    }

    /// The lists of the supported states read as the fixed lists read before they came from
    /// the registry.
    #[test]
    fn the_state_lists_read_as_before() {
        let (english, german) = (Lang::english(), de());
        assert_eq!(
            states(&english),
            "Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, \
             Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony, Rhineland-Palatinate"
        );
        assert_eq!(
            states_in_a_sentence(&english),
            "Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, \
             Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony and \
             Rhineland-Palatinate"
        );
        assert_eq!(
            states_in_a_sentence(&german),
            "Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, \
             Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen und Rheinland-Pfalz"
        );
        let instructions = TEXTS.text(
            &english,
            "mcp-instructions",
            &[("states", v(states_in_a_sentence(&english)))],
        );
        assert!(
            instructions.contains("geodata of Brandenburg, Berlin, Saxony,")
                && instructions.contains("Lower Saxony and Rhineland-Palatinate, and from OSM"),
            "{instructions}"
        );
    }

    #[test]
    fn german_and_english_differ_and_fill_in_variables() {
        let args = [("path", v("karte.omap"))];
        assert_eq!(
            TEXTS.text(&Lang::english(), "cli-wrote", &args),
            "Wrote karte.omap"
        );
        assert_eq!(
            TEXTS.text(&de(), "cli-wrote", &args),
            "karte.omap geschrieben"
        );
    }

    #[test]
    fn counts_pick_singular_and_plural() {
        let one = [("count", n(1))];
        let two = [("count", n(2))];
        assert_eq!(
            TEXTS.text(&Lang::english(), "cli-fetched-tiles", &one),
            "Fetched 1 tile"
        );
        assert_eq!(
            TEXTS.text(&Lang::english(), "cli-fetched-tiles", &two),
            "Fetched 2 tiles"
        );
        assert_eq!(
            TEXTS.text(&de(), "cli-fetched-tiles", &one),
            "1 Kachel geladen"
        );
        assert_eq!(
            TEXTS.text(&de(), "cli-fetched-tiles", &two),
            "2 Kacheln geladen"
        );
    }

    #[test]
    fn page_texts_keep_their_placeholders_for_the_page() {
        let english = page_texts(&Lang::english());
        assert_eq!(
            english["page-size"],
            "Size with buffer: { $width } × { $height } m"
        );
        assert_eq!(
            english["page-step-line"],
            "Step { $index } of { $total }: { $label }"
        );
        assert_eq!(english["suite-banner-dismiss"], "Dismiss");
        let german = page_texts(&de());
        assert_eq!(german["page-tiles-one"], "{ $n } Kachel: { $names }");
        assert_eq!(german["page-tiles-other"], "{ $n } Kacheln: { $names }");
        assert_eq!(german["page-next"], r#"Weiter: freemapper run "{ $dir }""#);
        // Only the page's families, and every message of them.
        assert!(
            english
                .keys()
                .all(|id| id.starts_with("page-") || id.starts_with("suite-"))
        );
        assert_eq!(english.len(), german.len());
        assert!(english.contains_key("page-save-stale"));
    }

    #[test]
    fn variables_are_read_from_the_message_only() {
        let source = "a = x { $one }\n    and { $two }\nb = { $three }\nab = { $four }\n";
        assert_eq!(variables_of(source, "a"), ["one", "two"]);
        assert_eq!(variables_of(source, "ab"), ["four"]);
        assert!(variables_of(source, "c").is_empty());
    }

    #[test]
    fn the_language_comes_from_the_environment_in_the_suite_order() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |name: &str| {
                pairs
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| (*v).to_owned())
            }
        };
        let code = |pairs| TEXTS.detect_lang(None, &env(pairs)).code();
        assert_eq!(code(&[]), "en");
        assert_eq!(code(&[("LANG", "de_DE.UTF-8")]), "de");
        assert_eq!(
            code(&[("FREEMAPPER_LANG", "en"), ("LANG", "de_DE.UTF-8")]),
            "en"
        );
        // A language without texts passes to the next source.
        assert_eq!(code(&[("FREEMAPPER_LANG", "fr"), ("LANG", "de_AT")]), "de");
        assert_eq!(code(&[("LANG", "C")]), "en");
        assert!(TEXTS.parse_lang_arg("fr").is_err());
        assert_eq!(TEXTS.parse_lang_arg("DE").unwrap().code(), "de");
        // English by default is not a named language; English by `LANG` is.
        let named = |pairs| names_a_lang(None, &env(pairs));
        assert!(!named(&[]));
        assert!(!named(&[("LANG", "C")]));
        assert!(!named(&[("LANG", "fr_FR.UTF-8")]));
        assert!(named(&[("LANG", "en_US.UTF-8")]));
        assert!(named(&[("FREEMAPPER_LANG", "de")]));
        assert!(names_a_lang(Some("en"), &env(&[])));
    }
}
