# Program texts

One folder per language, each with `freemapper.ftl` in [Fluent](https://projectfluent.org/) syntax. English (`en`)
is the reference and the fallback; every other language has the same message ids and the same variables
(ADR 0092).

To add a text: give it an id that starts with `cli-`, write it in every language, and use it in the code as
`text("cli-…", &[("name", v(value))])`. To add a language: add a folder, list it in `TEXTS` in `src/text.rs`,
and run `cargo test -p freemapper-cli`. The tests name every missing message, every variable that differs
between languages, every id the code uses but the catalog lacks, and every id the catalog holds but the code
does not use.

Texts are written for the person at the terminal: in German with "Sie", terms as in `docs/glossary.md`.
