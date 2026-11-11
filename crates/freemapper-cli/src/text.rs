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

use std::sync::OnceLock;

use freemapper_text::Catalog;
pub use freemapper_text::{FluentValue, Lang};

/// The embedded catalogs: language code and file content. English comes first.
pub static TEXTS: Catalog = Catalog::new(&[
    ("en", include_str!("../i18n/en/freemapper.ftl")),
    ("de", include_str!("../i18n/de/freemapper.ftl")),
]);

/// The language of this run, set once at start.
static LANG: OnceLock<Lang> = OnceLock::new();

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
    Ok(())
}

/// The language of this run; English before [`set_lang`].
pub fn lang() -> Lang {
    LANG.get().cloned().unwrap_or_else(Lang::english)
}

/// The text `id` in the language of this run.
pub fn text(id: &str, args: &[(&str, FluentValue)]) -> String {
    TEXTS.text(&lang(), id, args)
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
        ];
        assert_eq!(TEXTS.problems(&sample), Vec::<String>::new());
    }

    #[test]
    fn every_id_used_in_the_code_exists() {
        let english = TEXTS.message_ids("en");
        for source in [
            include_str!("main.rs"),
            include_str!("text.rs"),
            include_str!("lock.rs"),
            include_str!("select_run.rs"),
        ] {
            for id in freemapper_text::ids_in_source(source, &["cli-"]) {
                assert!(english.contains(&id), "{id} is used but not in the catalog");
            }
        }
        // And nothing is left over.
        let used: std::collections::BTreeSet<String> = [
            include_str!("main.rs"),
            include_str!("text.rs"),
            include_str!("lock.rs"),
            include_str!("select_run.rs"),
        ]
        .iter()
        .flat_map(|source| freemapper_text::ids_in_source(source, &["cli-"]))
        .collect();
        for id in &english {
            assert!(used.contains(id), "{id} is in the catalog but not used");
        }
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
    }
}
