# ADR 0172: README and manual in German and English only

## Status

Accepted, 21 November 2026. Supersedes the paragraph "Documents" of
[ADR 0168](0168-polish-program-language.md).

## Context

ADR 0168 added a Polish README and a shortened Polish manual next to the program texts. The manual
was already out of date after Stage 2 and 3 of Poland: it still described a map from OpenStreetMap
alone. Nobody who can read Polish checks these files, and every change of behaviour would need a
third and, with more countries, a twentieth copy. A README in one extra language also looks odd
beside two others.

## Decision

- README and user manual exist in German and English. Both are kept up to date with every change.
- The program itself (messages, help, area page, MCP descriptions) keeps all its languages. That is
  what a user sees on every run, and the catalogs are checked by the tests.
- A new country gets no README and no manual of its own language. It is described in the English
  ones.
- `README.pl.md` and `docs/manual/pl.md` are removed. There is no Polish changelog either.
- A document in another language is added only when people living there ask for it and someone who
  reads the language agrees to keep it up to date.

## Consequences

- Fewer files to keep in step; the manual of Stage 2 and 3 is written twice, not three times.
- A reader in Poland who does not read English finds the program in Polish but the documents in
  English.
- The Polish glossary section stays: the catalogs follow it.
