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

The self-update's texts (`freemapper update` and its notices, `src/update/`) have ids that start with `update-`.

The area page's texts (`web/`, ADR 0096) have ids that start with `page-`. The page loads every `page-*` and
`suite-*` message through `GET api/texts` and puts them into its elements (`data-text`, `data-label`, `data-title`,
`data-placeholder`) or sets them from its scripts with `t(id, args)`. A message with a value keeps `{ $name }` in the
text and the page fills it in, so these messages have no selectors and no functions; a count has two messages,
`<id>-one` and `<id>-other`, and the page picks one. `suite-*` holds the words of the suite's shared page modules
from freemapper-core (core ADR 0030), so far `suite-banner-dismiss`. The page speaks to the person with "du", as it
did before its texts moved here; AGENTS.md, the manual and `scripts/browser-smoke.md` quote its words.
`src/select_page.rs` checks that every id the page uses exists and that every `page-*` message is used.

The MCP server's texts have ids that start with `mcp-`: `mcp-tool-<tool>` and `mcp-param-<tool>-<parameter>`
for the descriptions, whose English text equals the one in `src/mcp.rs`
(`cargo test -p freemapper-cli print_the_english_descriptions -- --ignored --nocapture` prints them).

Progress lines and warnings of the engine are not in this folder. Their texts live in
`crates/freemapper-adapter-api/i18n/<language>/notes.ftl`, with ids that start with `progress-`, `warning-` and
`word-` (`crates/freemapper-adapter-api/src/note.rs`); `cargo test -p freemapper-adapter-api` checks them the
same way. The engine, the readers and the adapters built in share that one catalog (ADR 0082).

Texts are written for the person at the terminal: in German with "Sie", terms as in `docs/glossary.md`.
