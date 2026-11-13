//! Progress lines and warnings as values (ADR 0092, stage 3).
//!
//! The engine reports what it does through a callback. It hands over a [`Note`]: a message id
//! and the values the message names. The texts live in `i18n/<language>/notes.ftl` of this crate:
//! one catalog for the engine, the readers and the adapters built in (a code adapter adds its
//! messages there). The engine never picks a language: the caller renders a note with
//! [`Note::text`] in the language of its run, and `Display` gives English for logs and tests.
//!
//! An id starts with `progress-` or `warning-`; that prefix decides the kind. `word-` ids name
//! things inside other messages (a height model, a product), and `error-` ids word the detail of
//! an error that has many forms (a place, a layer).

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use freemapper_text::{Catalog, FluentValue};

pub use freemapper_text::Lang;

/// The embedded catalogs: language code and file content. English comes first.
pub static NOTES: Catalog = Catalog::new(&[
    ("en", include_str!("../i18n/en/notes.ftl")),
    ("de", include_str!("../i18n/de/notes.ftl")),
]);

/// What the engine reports while it works: a progress line or a warning.
#[derive(Debug, Clone)]
pub struct Note {
    id: &'static str,
    args: Vec<(&'static str, Arg)>,
}

/// A value a message names.
#[derive(Debug, Clone)]
pub enum Arg {
    /// A name, a path, a formatted number: shown as it is.
    Text(String),
    /// A count; the catalogs pick singular or plural by it.
    Count(i64),
    /// A text of its own, rendered in the same language.
    Note(Note),
    /// A text a source brings in each language of the program, such as the name of a service
    /// of a region adapter: German in a German run, else English.
    Words { en: String, de: String },
    /// An error. The caller of [`Note::text`] words it.
    Error(Arc<dyn Error + Send + Sync>),
}

impl Arg {
    /// An error as a message value.
    pub fn error(error: impl Error + Send + Sync + 'static) -> Arg {
        Arg::Error(Arc::new(error))
    }
}

impl From<String> for Arg {
    fn from(text: String) -> Arg {
        Arg::Text(text)
    }
}

impl From<&str> for Arg {
    fn from(text: &str) -> Arg {
        Arg::Text(text.to_owned())
    }
}

impl From<&String> for Arg {
    fn from(text: &String) -> Arg {
        Arg::Text(text.clone())
    }
}

impl From<Note> for Arg {
    fn from(note: Note) -> Arg {
        Arg::Note(note)
    }
}

macro_rules! count_from {
    ($($number:ty),*) => {$(
        impl From<$number> for Arg {
            fn from(count: $number) -> Arg {
                Arg::Count(i64::try_from(count).unwrap_or(i64::MAX))
            }
        }
    )*};
}
count_from!(usize, u64, u32, u16, u8, i64, i32);

/// A [`Note`] from a message id and the values the message names:
/// `note!("progress-ndsm-from-bdom", tile = tile.name())`.
#[macro_export]
macro_rules! note {
    ($id:literal $(, $name:ident = $value:expr)* $(,)?) => {
        $crate::note::Note::new($id)$(.with(stringify!($name), $value))*
    };
}

/// A height model or orthophoto by the English name its `label()` gives, such as
/// `terrain model (HVBG DGM1)`: the kind is worded in the language, the name in brackets stays.
pub fn model(label: &str) -> Arg {
    let named = label.split_once(" (").and_then(|(kind, name)| {
        let kind = match kind {
            "terrain model" => "terrain",
            "surface model" => "surface",
            "orthophoto" => "orthophoto",
            _ => return None,
        };
        Some(note!(
            "word-model",
            kind = kind,
            name = name.strip_suffix(')')?
        ))
    });
    named.map_or_else(|| Arg::from(label), Arg::from)
}

impl Note {
    /// The message `id` without values.
    pub fn new(id: &'static str) -> Note {
        Note {
            id,
            args: Vec::new(),
        }
    }

    /// Adds the value the message calls `name`.
    #[must_use]
    pub fn with(mut self, name: &'static str, value: impl Into<Arg>) -> Note {
        self.args.push((name, value.into()));
        self
    }

    /// The message id.
    pub fn id(&self) -> &'static str {
        self.id
    }

    /// Whether this is a warning. Callers collect warnings and show them again at the end.
    pub fn is_warning(&self) -> bool {
        self.id.starts_with("warning-")
    }

    /// The line in `lang`, a warning with its prefix ("warning: …"). `error` words the errors
    /// the note carries; the engine's own errors have English `Display` texts only.
    pub fn text(
        &self,
        lang: &Lang,
        error: &dyn Fn(&(dyn Error + 'static), &Lang) -> String,
    ) -> String {
        let body = self.body_text(lang, error);
        if self.is_warning() {
            NOTES.text(lang, "note-warning", &[("text", FluentValue::from(body))])
        } else {
            body
        }
    }

    /// The text without the warning prefix.
    fn body_text(
        &self,
        lang: &Lang,
        error: &dyn Fn(&(dyn Error + 'static), &Lang) -> String,
    ) -> String {
        let id = self.id;
        let values: Vec<(&str, FluentValue)> = self
            .args
            .iter()
            .map(|(name, arg)| {
                let value = match arg {
                    Arg::Text(text) => FluentValue::from(text.clone()),
                    Arg::Count(count) => FluentValue::from(*count),
                    Arg::Note(note) => FluentValue::from(note.body_text(lang, error)),
                    Arg::Words { en, de } => {
                        FluentValue::from(if lang.code() == "de" { de } else { en }.clone())
                    }
                    Arg::Error(e) => FluentValue::from(error(e.as_ref(), lang)),
                };
                (*name, value)
            })
            .collect();
        let text = NOTES.text(lang, id, &values);
        // A missing text comes back as the id, a missing value as `{$name}`.
        debug_assert!(text != id && !text.contains("{$"), "{id}: {text}");
        text
    }
}

/// For the tests of the sources, which look at a warning's English sentence without its prefix.
#[cfg(any(test, feature = "testing"))]
impl Note {
    fn sentence(&self) -> String {
        self.body_text(&Lang::english(), &|e, _| e.to_string())
    }

    pub fn contains(&self, part: &str) -> bool {
        self.sentence().contains(part)
    }

    pub fn starts_with(&self, start: &str) -> bool {
        self.sentence().starts_with(start)
    }

    pub fn ends_with(&self, end: &str) -> bool {
        self.sentence().ends_with(end)
    }
}

#[cfg(any(test, feature = "testing"))]
impl PartialEq<&str> for Note {
    fn eq(&self, other: &&str) -> bool {
        self.sentence() == *other
    }
}

/// Two notes are the same when they read the same; the sources use it to say a thing once.
impl PartialEq for Note {
    fn eq(&self, other: &Note) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Note {}

/// The English line, errors in their `Display` text.
impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text(&Lang::english(), &|e, _| e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::Path;

    fn de() -> Lang {
        NOTES.parse_lang_arg("de").unwrap()
    }

    /// The `src/` folders of the workspace crates that emit notes, from this crate's folder.
    const EMITTING_CRATES: [&str; 5] = [
        "src",
        "../freemapper/src",
        "../freemapper-sources/src",
        "../freemapper-manifest/src",
        "../adapters/de/src",
    ];

    /// The ids every `.rs` file of the emitting crates names, tests included.
    fn ids_in_the_code() -> BTreeSet<String> {
        fn visit(dir: &Path, ids: &mut BTreeSet<String>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    visit(&path, ids);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    // `ids_in_source` stops at the first test marker; these files have code below it.
                    let source = std::fs::read_to_string(&path)
                        .unwrap()
                        .replace("#[cfg(test)]", "");
                    ids.extend(freemapper_text::ids_in_source(
                        &source,
                        &["progress-", "warning-", "word-", "note-", "error-"],
                    ));
                }
            }
        }
        let mut ids = BTreeSet::new();
        for src in EMITTING_CRATES {
            visit(&Path::new(env!("CARGO_MANIFEST_DIR")).join(src), &mut ids);
        }
        ids
    }

    /// The variables the English catalog names.
    fn variables() -> BTreeSet<String> {
        include_str!("../i18n/en/notes.ftl")
            .split('$')
            .skip(1)
            .map(|rest| {
                rest.chars()
                    .take_while(|c| c.is_ascii_lowercase() || *c == '-')
                    .collect()
            })
            .collect()
    }

    #[test]
    fn the_catalogs_are_complete_and_use_the_same_variables() {
        let names = variables();
        // A number fits everywhere: counts pick their plural, the other selections their default.
        let sample: Vec<(&str, FluentValue)> = names
            .iter()
            .map(|name| (name.as_str(), FluentValue::from(2)))
            .collect();
        assert_eq!(NOTES.problems(&sample), Vec::<String>::new());
    }

    #[test]
    fn every_id_in_the_code_has_a_text_and_every_text_is_used() {
        let english = NOTES.message_ids("en");
        let used = ids_in_the_code();
        for id in &used {
            assert!(english.contains(id), "{id} is used but not in the catalog");
        }
        for id in &english {
            assert!(used.contains(id), "{id} is in the catalog but not used");
        }
    }

    #[test]
    fn every_message_is_a_progress_line_a_warning_or_a_word() {
        for id in NOTES.message_ids("en") {
            assert!(
                ["progress-", "warning-", "word-", "note-", "error-"]
                    .iter()
                    .any(|prefix| id.starts_with(prefix)),
                "{id}"
            );
        }
    }

    #[test]
    fn a_warning_carries_its_prefix_in_the_language() {
        let note = note!("warning-other-state");
        assert!(note.is_warning());
        assert!(note.to_string().starts_with("warning: "), "{note}");
        let german = note.text(&de(), &|e, _| e.to_string());
        assert!(german.starts_with("Warnung: "), "{german}");
    }

    #[test]
    fn values_fill_the_message_in_both_languages() {
        let note = note!("warning-other-zone", zone = "32", epsg = "25832");
        assert_eq!(
            note.to_string(),
            "warning: the map area lies in UTM zone 32 (EPSG:25832); freemapper fetches everything again in that zone"
        );
        let german = note.text(&de(), &|e, _| e.to_string());
        assert!(german.contains("UTM-Zone 32 (EPSG:25832)"), "{german}");
    }

    #[test]
    fn words_are_taken_in_the_language_of_the_run() {
        let note = note!(
            "progress-permits-fetch",
            source = Arg::Words {
                en: "district borders".into(),
                de: "Bezirksgrenzen".into(),
            }
        );
        assert!(note.to_string().contains("district borders"), "{note}");
        let german = note.text(&de(), &|e, _| e.to_string());
        assert!(german.contains("Bezirksgrenzen"), "{german}");
    }

    #[test]
    fn the_caller_words_the_errors() {
        let failure = std::io::Error::other("no route");
        let note = note!("warning-error", error = Arg::error(failure));
        assert_eq!(note.to_string(), "warning: no route");
        let german = note.text(&de(), &|e, _| format!("kein Weg ({e})"));
        assert_eq!(german, "Warnung: kein Weg (no route)");
    }

    #[test]
    fn a_note_inside_a_note_is_worded_in_the_same_language() {
        let inner = note!("progress-ndsm-from-bdom", tile = "33421-5821");
        let note = Note::new("warning-error").with("error", inner);
        let german = note.text(&de(), &|e, _| e.to_string());
        assert_eq!(
            german,
            "Warnung: nDSM der Kachel 33421-5821 aus dem Oberflächenmodell (bDOM)"
        );
    }
}
