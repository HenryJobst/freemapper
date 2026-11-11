# 0092. Program language

Date: 2026-11-11
Status: Accepted

## Context

freemapper spoke English only, except for the area page, which follows the browser. The user asked on 2026-11-08 for
German and English program texts in the web page and in the console, chosen as in the rest of the suite. The suite's
rule comes from freemapper-courseplan (its ADR 0021 and 0022), and the code behind it lives in `freemapper-text` of
freemapper-core.

## Decision

- The language of a run is the first of these that names a language freemapper has texts for: `--lang`, then
  `FREEMAPPER_LANG`, then `LC_ALL`, `LC_MESSAGES`, `LANG`; else English. Only the language part counts, so `de`,
  `de-DE` and `de_DE.UTF-8` all mean German. `C`, `POSIX`, an empty value and a language without texts pass to the next
  source.
- `--lang` is a global switch. A value that names no available language is a usage error with exit code 2, worded in
  the language the environment gives.
- Texts live in Fluent files, `crates/freemapper-cli/i18n/<language>/freemapper.ftl`, embedded at build time. English
  is the reference and the fallback. Tests fail when a language lacks a message or uses other variables, when the code
  uses an id the catalog lacks, and when the catalog holds an id the code does not use.
- Counts choose singular or plural in the catalog ("1 tile", "2 tiles").
- The work comes in stages, because the texts sit in four places:

  | Stage | Texts | State |
  |---|---|---|
  | 1 | the lines the CLI prints itself: what was written or kept, the hints after `run`, `export` and `select` | this ADR |
  | 2 | error messages (the `Display` texts of the library's error types stay English for logs and tests) | done, see below |
  | 3 | progress lines and warnings of the engine | done, see below |
  | 4 | `--help`, the MCP texts, and `lang` in `api/project` so the page follows `--lang` | done, see below |

- Tests of the CLI set `FREEMAPPER_LANG=en`, so they pass on a machine with any system language.

### Stage 2: errors (2026-11-11)

- The CLI crate has a trait `Localize` (`src/errors.rs`) and implements it for the library's error types. It is the
  CLI's own trait, as in freemapper-courseplan, because the types belong to another crate. The library does not learn
  about languages.
- In English the catalog text equals the type's `Display` text. A test builds a value of every variant and compares
  the two, so a reworded library message fails the test until the catalog follows.
- `main` prints `error: <text>` or `Fehler: <text>` with the causes after colons and exits with 1. Before, `anyhow`
  printed `Error: …` and a "Caused by:" list.
- `pipeline::describe` uses the same texts, so the MCP server and the area page report errors in the program's
  language.
- What stays as it is inside a translated error: words of the operating system, of parsers and of servers, and the
  details the library carries as free text (the `message` fields). `AreaError::Place` and `ProjectError::Layer`
  were free text too; since stage 3 they carry a `Note` with an `error-…` id from the engine's catalog, so the
  place search and the previews are worded in both languages.

### Stage 3: progress lines and warnings (2026-11-11)

- The engine no longer hands its callback a finished English line. It hands over a `Note`
  (`crates/freemapper/src/note.rs`): a message id and the values the message names. A value is a text, a count, an
  error, or another note.
- The texts of the notes live beside the engine, in `crates/freemapper/i18n/<language>/notes.ftl` (since ADR 0082,
  stage B, in `crates/freemapper-adapter-api/i18n/`, with the note type, so that readers and adapters share them). This differs
  from stage 2, where the CLI holds the texts of the library's errors. The reasons: a progress line is one
  statement at its call site, and a closed type with about 200 variants would cost five edits per line; and the
  region adapters are to become crates of their own, which then bring their texts with them.
- The engine still picks no language. The caller renders a note with `Note::text(lang, error)`; `error` words
  the errors a note carries, so the CLI passes its `errors::message` and a failed download inside a German warning
  is German too. `Display` gives the English line, so logs and the engine's tests read as before.
- The id decides the kind: `progress-…` is a progress line, `warning-…` a warning, `word-…` a name inside another
  message. A warning gets its prefix (`warning:`, `Warnung:`) from the catalog. The CLI no longer recognizes
  warnings by that prefix: `Note::is_warning` says so, and the pipeline sends them as `Event::Warning`.
- Names that are English words in the code (a state, a map kind, the kind of a height model) travel as keys, and
  each catalog words them (`hesse`, `schoolyard`, `terrain`).
- Tests: every language has every message with the same variables, every id in the engine's code has a text,
  every text is used, and in debug builds a note fails that names a value its message lacks.
- The height-model sources collect their warnings (tile lists, dates, the Metalink) as notes too and hand them
  on in `HeightModel::warnings`. Two notes count as equal when they read the same in English; the sources use
  that to say a thing once. No line of the engine is a finished English string any more.

### Stage 4, first part: the language of the selection page (2026-11-11)

- `api/project` carries `lang` when `--lang` or one of the environment variables named a language freemapper has
  texts for. The page then uses it. When nothing named one, `lang` is null and the page follows the browser as before.
- English as the mere fallback does not count as named. Otherwise a German browser on Windows, where `LANG` is
  usually missing, would get an English page after this change.

### Stage 4, second part: `--help` and the parse errors (2026-11-11)

- The doc comments of the clap derive stay the English help. The catalog holds the same texts under `help-…` ids
  (`help-<command>`, `help-<command>-<argument>`, `help-arg-<argument>` for an argument that reads the same in
  every command), and a test compares the English catalog with the doc comments. A new option without a catalog
  text fails that test.
- `src/help.rs` rebuilds the derived command with the catalog's texts, as freemapper-courseplan does: the headings,
  "possible values" and "default", the `help` command and the `-h`/`-V` flags. `--lang` is read from the raw
  arguments before clap parses, because the help itself needs the language.
- The common parse errors (unknown argument or command, missing argument, invalid value, conflict) come from the
  catalog and start with `error:` or `Fehler:`; the rare kinds keep clap's English text. The exit code stays 2.
- Still clap's: the placeholders in the usage line (`[OPTIONS]`, `<COMMAND>`, `<DIR>`).

### Stage 4, third part: the MCP server and the summary of a run (2026-11-11)

- The server instructions, the descriptions of the tools and their parameters, the results in words and the
  errors come in the language chosen at start. Tool names, parameter names and JSON field names stay English, and
  so do the values an agent compares (`running`, `finished`, `schoolyard`).
- The descriptions stay in `src/mcp.rs`, where the rmcp macros and the doc comments of the parameter types need
  them, and the catalog holds the same texts under `mcp-tool-<tool>` and `mcp-param-<tool>-<parameter>`. At start
  the server replaces the descriptions in the router by the catalog's. A test compares the English catalog with
  the macros, as for the help.
- The summary of a run (what was written or kept, a failed export step, the OCAD hints) comes from the catalog
  (`cli-summary-…`); the area page shows the same text.
- The note after `init` about 1:5000 on a sprint map is a `Note` of the engine (`note-scale-outside-norm`).
- `OcadOutcome::warnings` holds notes as well. Still English: the `message` fields named under stage 2.

## Consequences

- An AI agent on a German system now reads German tool descriptions and results. `AGENTS.md` quotes the English
  messages and says how to get them (`--lang en` after `mcp`).
- Windows and macOS set `LANG` only in some shells. Where it is missing, the program is English unless
  `FREEMAPPER_LANG` or `--lang` is given. That is the suite's rule; reading the system language through the
  operating system would be a change for all three programs.
- A new message needs its text in every language in the same commit; the tests enforce it.
- Cost if wrong: Fluent and its language crates are four more dependencies in the CLI. Going back means putting the
  English texts back into the code.

Source: the user on 2026-11-08 and 2026-11-11; freemapper-courseplan ADR 0021 and 0022; freemapper-core ADR 0010.
