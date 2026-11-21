# 0168. Polish as a program language

Date: 2026-11-21

Status: Accepted, supplements ADR 0092 and ADR 0164

## Context

Poland is a preview region (ADR 0164): a Polish user got German or English texts. The plan of
stage 4 expected a release of freemapper-core with a new language. That is not needed:
`freemapper-text` lists the languages of a `Catalog` from the catalogs that are built in, so
`--lang pl`, `FREEMAPPER_LANG` and the system language work as soon as a catalog `pl` is listed
(`TEXTS` in `src/text.rs`, `NOTES` in `note.rs`).

## Decision

- **Catalogs.** `crates/freemapper-cli/i18n/pl/freemapper.ftl` and
  `crates/freemapper-adapter-api/i18n/pl/notes.ftl` hold every id of the English reference. The
  existing tests check ids and variables of every listed language. Counts carry the Polish forms
  `one`, `few`, `many` and `other` where the English text has `one` and `other`.
- **Names of regions.** `Names` gets `pl: Option<String>` and `Names::in_lang(code)`: German
  and Polish name the region in their language, every other language uses English, and a region
  without a Polish name is named in English in Polish texts. Only Poland has one (`Polska`).
  `adapters list --json` and the area page's `api/regions` print `names.pl` only when it is set.
- **Language switches are codes.** The functions that took `german: bool` take the language
  code. Words the code picks between German and English (`Arg::Words`) fall to English in
  Polish. The decimal comma holds for German and Polish.
- **Answer to the update question.** `t` and `tak` mean yes besides `y` and `j`.
- **Wording.** A state of Poland is a `województwo`, a state of Germany `kraj związkowy`; the
  glossary has a Polish section that the catalogs follow.
- **Documents.** (Superseded by ADR 0172.) `README.pl.md` was a translation of the English README. `docs/manual/pl.md` is a
  shortened manual (what a user in Poland needs, same chapter numbers as the English one), not a
  translation of the 190 KB of the English manual, most of which describes German states.
  There is no Polish changelog.

## Consequences

- The texts are machine-drafted and not yet read by a native speaker; that read-through is a
  condition for removing "preview" from Poland (the person who reads is the owner's choice).
- freemapper-courseplan and freemapper-print use the same `Lang` and can add `pl` the same way;
  nothing forces them to.
