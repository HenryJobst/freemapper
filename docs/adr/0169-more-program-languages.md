# 0169. More program languages

Date: 2026-11-21

Status: Accepted, supplements ADR 0168

## Context

ADR 0168 added Polish by adding catalogs, with no change in freemapper-core. The countries that issue
#227 lists as candidates, and the large orienteering countries, speak more languages.

## Decision

- **Eighteen catalogs** are added for the CLI and the notes: cs, nl, da, fr, sv, nb, fi, sk, et, it,
  sl, hu, lt, lv, es, pt, ru and uk. Norwegian is `nb` (Bokmål), because systems report `nb_NO`.
  Sweden is not in issue #227, but a research branch exists, so `sv` is included.
- **Texts only.** There is no README and no manual in these languages; the documents stay
  German, English and Polish.
- **Machine drafts.** No native speaker has read them. The chosen terms are in
  `docs/glossary-languages.md`. A language is called finished only after a read-through.
- **Plural forms** follow the CLDR categories of each language.
- **The question of `freemapper update`** takes the yes of every language (`o`, `s`, `d`, `i`, `t` and
  others, besides `y` and `j`).
- **Tests** use `ja` as the language without texts, since French is available now.

## Consequences

- The binary grows by about 2 MB of text.
- A catalog that is wrong in a language only shows in that language; the tests check ids and
  variables, not the wording.
- A second machine read (21 November 2026) fixed wrong terms, grammar and plural branches in all 19 catalogs. It is no substitute for a native speaker; the doubtful terms are listed for them.
