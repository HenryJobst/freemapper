# 0096. The area page on the suite frame

Date: 2026-11-11
Status: Accepted

Amends [0009](0009-area-selection-in-a-local-browser-page.md) and the page mechanism of
[0092](0092-program-language.md). Carries out step 4 of freemapper-core's ADR 0030 ("One look and one frame for the
suite's local pages").

## Context

freemapper-core v0.21.0 serves a shared page frame below the page token at `/{token}/suite/`: `suite.css` with the
suite's tokens in light and dark, `api.js`, `texts.js`, `status.js` and `tree.js`, and on the Rust side
`texts_language`, `revision`, `stale`, `refused` and the test helper `check_page_rules` (core ADR 0030 with its
addendum). freemapper-courseplan's editor and freemapper-print's editor use it; freemapper's area page did not.
The page kept its texts in two tables, `TEXTS` in `app.js` and `RUN_TEXTS` in `run.js`, picked its language from
`project.lang` or the browser, called `fetch` itself, had fixed colours without a dark mode, and saved
`freemapper.toml` without checking whether another window or program had written it in the meantime.

## Decision

- **Look.** The page loads `suite/suite.css` before `app.css`. `app.css` keeps what belongs to the page alone (the map
  and the side panel, the drawing buttons, the run panel) and uses the suite tokens instead of fixed colours, so the
  page follows the system's dark mode. One app token, `--ok`, colours a save that worked. The Leaflet map stays light;
  its zoom buttons, layer control and attribution take the tokens. Lines drawn on the map keep their fixed colours.
- **Modules.** `app.js` and `run.js` are ES modules (`<script type="module">`) and import `api.js`, `texts.js` and
  `status.js` from `../suite/`. Leaflet stays a classic script from `web/vendor/`. Every server call goes through
  `api.js`; `app.js` wraps it in one `api()` that throws on an error answer, as before, so run.js keeps its error
  handling. The two scripts still talk through `window.freemapperPage` and `window.freemapperRun`; the start waits for
  `DOMContentLoaded`, which comes after both modules ran.
- **Texts.** Every text of the two tables is a Fluent message of the family `page-*` in
  `crates/freemapper-cli/i18n/<language>/freemapper.ftl`, word for word, because AGENTS.md, the manual and
  `scripts/browser-smoke.md` quote them. `suite-banner-dismiss` joins as the first `suite-*` message. A value stays as
  `{ $name }` and the page fills it in with `t`; a count has two messages, `<id>-one` and `<id>-other`, and the page
  picks one with `tn`. Static texts use `data-text`, `data-label`, `data-title` and `data-placeholder`; the elements
  in `index.html` are empty until the texts arrive. The three prefixes the scripts put in front of a value
  ("Fehler: ", "Aus OpenStreetMap: ", "Speicherplatz: ") became one message with a value each; the update notice
  ("… Aktualisieren mit ") lost its trailing space to the script, because Fluent drops it. `failedPrefix` of `run.js` was never
  used and is gone; `madeFiles`, unused in `app.js`, now words the closing line of a finished run, which `run.js` had
  written in German and English by hand.
- **Language.** `GET api/texts` answers `{ lang, texts }` with every `page-*` and `suite-*` message. The language is
  `freemapper_web::texts_language(fixed, Accept-Language)`: `fixed` is the language `--lang` or the environment named
  (what `project.lang` carried before, ADR 0092 stage 4), otherwise the browser's language from the request, otherwise
  English. The rule of ADR 0092 is unchanged; the server now applies it, and the page no longer reads
  `navigator.language`. `project.lang`, `setLanguage()` and the two tables are gone. The page loads the texts first
  and only then writes the hint, the layer names and the update notice, so no script-set text can stay in another
  language.
- **Revision on save.** `GET api/project` carries `revision`, the FNV-1a revision of `freemapper.toml` as the page read
  it (`freemapper_web::revision`), or null while there is no file. `POST api/save` sends it in
  `X-Freemapper-Revision`. Under the folder lock the server reads the file again and compares; when the revisions
  differ, or the page names none although the file exists, nothing is written and the answer is 409 from
  `freemapper_web::stale` with the current revision and the sentence `page-save-stale` in the page's language. The
  page shows it in the suite banner, reads the project again and keeps the area it shows; saving once more replaces
  the file knowingly. A first save, before the file exists, needs no revision. A successful save answers the new
  revision.
- **The exception of core ADR 0030.** The area page is no editor and does not write at once with an undo stack. It
  keeps its save button, the replace dialog of the run panel and `.omap.bak`: saving an area can drop fetched data
  (another UTM zone or provider), and creating the map can replace a map with the mapper's work in it, which no undo
  of the page brings back. What it takes from the frame is the look, `api.js`, `texts.js`, the banner and the revision
  check on its one writing route.
- **Messages.** The page keeps its own message line under the save button (`#message`, `role="status"`) with its
  colours for a save that worked and an error, because `status.js` writes text only and the page has no status bar;
  replacing it would drop the colour. The new conflict of a stale save goes to the suite banner (`#banner`), as in the
  other two pages.
- **Rules and size.** A test runs `freemapper_web::check_page_rules` over `web/` without `web/vendor/` plus the suite
  assets, within 300 KiB. Leaflet counts separately, also within 300 KiB.

## Consequences

- The page has a dark mode and looks like the other two pages of the suite; a fix in the frame reaches it with the
  next core tag.
- Two open pages of the same folder, or a page and a run in a terminal, can no longer overwrite each other's
  `freemapper.toml` unnoticed. A script that posts to `api/save` on an existing project must send the revision from
  `api/project`.
- The page's words live in the catalog and are checked like the program's: same ids in both languages, every id the
  page uses exists, every `page-*` message is used, no selectors.
- Numbers the page fills into its texts (size, tile count, download) are formatted for the page's language, so a
  German page writes "1.234" where it wrote "1234".
