# Program texts

One folder per language, each with `freemapper.ftl` in [Fluent](https://projectfluent.org/) syntax. English (`en`)
is the reference and the fallback; every other language has the same message ids and the same variables
(ADR 0092).

To add a text: give it an id that starts with `cli-`, write it in every language, and use it in the code as
`text("cli-…", &[("name", v(value))])`. Ids are lower-case letters and hyphens only.

Errors of the library have ids that start with `error-`, words inside them (a map kind, a product) ids that start
with `word-`; `src/errors.rs` maps each error variant to its text. The English text of an error equals the
library's own message, and a test compares the two. To add a language: add a folder, list it in `TEXTS` in `src/text.rs`,
and run `cargo test -p freemapper-cli`. The tests name every missing message, every variable that differs
between languages, every id the code uses but the catalog lacks, and every id the catalog holds but the code
does not use.

The help has ids that start with `help-`: `help-<command>` for a command, `help-<command>-<argument>` for an
argument, `help-arg-<argument>` for one that reads the same in every command. The English text equals the doc
comment in `src/main.rs`; `cargo test -p freemapper-cli print_the_english_help -- --ignored --nocapture` prints
the entries for new options.

The MCP server's texts have ids that start with `mcp-`: `mcp-tool-<tool>` and `mcp-param-<tool>-<parameter>`
for the descriptions, whose English text equals the one in `src/mcp.rs`
(`cargo test -p freemapper-cli print_the_english_descriptions -- --ignored --nocapture` prints them).

Progress lines and warnings of the engine are not in this folder. Their texts live beside the engine, in
`crates/freemapper/i18n/<language>/notes.ftl`, with ids that start with `progress-`, `warning-` and `word-`
(`crates/freemapper/src/note.rs`); `cargo test -p freemapper` checks them the same way.

Texts are written for the person at the terminal: in German with "Sie", terms as in `docs/glossary.md`.
