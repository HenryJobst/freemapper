# Map preview on the area page: design

Status: decided on 2026-11-12. The user asked for it; under the standing rule of autonomous work every call in
this document is Claude's and is reported afterwards ("my call" marks the ones the user did not state).

## Intent

The user wants to see the generated map on the area page (`freemapper select`). After changing an option the
effect should show right away. For that, the map generation itself writes a preview image (the user suggested
WebP).

The main use is the content profile and its 14 groups (ADR 0100, 0101). Today the user has to switch a group,
run the whole pipeline (minutes) and open the `.omap` or the PDF to see what changed.

## Goals

1. Every export of the map also writes `<name>-preview.webp` next to `<name>.omap`. That covers `freemapper run`,
   `freemapper export`, MCP `start_pipeline` and "Karte erzeugen" on the page.
2. The area page shows that image in its run panel: after a run, and on load when the folder has one.
3. When the user changes the content profile or a group on the page, the page shows a new preview for the
   settings on the page, without a run and without saving. It takes about a second on a sprint area such as the
   Viktoriapark example (Berlin, 1:4000) with a release build.
4. The page always says which state the image shows: the last generated map, or the settings on the page
   (not saved yet).

## Non-goals

- A preview before the first run. Without fetched and derived data there is nothing to draw; the page says so.
- Live preview for options other than content: area, map kind and scale need a new fetch or derive. After such
  a change the page shows the last image and says that a new run is needed. PDF and OCAD options do not change
  the map.
- Candidates, templates (orthophoto, hillshade) and the sheet (title, legend, credit lines) in the image. It shows
  the map objects only, as the map PDF draws them (base fill included), clipped to the map area. My call.
- An overlay on the Leaflet map. The paper is rotated against north by the grivation; a north-up overlay would
  not fit. My call; a later version can add it.
- MCP: `list_layers`/`preview_layer` do not get a `map` layer in this change. Session freemapper-29 is changing
  that tool for the relief layer right now. A follow-up adds it.

## Anti-goals (a failure even if it works)

- A changed map. `.omap`, candidates, OCAD and PDFs must stay byte-identical. The golden net (layer G4,
  `scripts/golden.sh test`) proves it; this change needs no `Golden-Bless:`.
- A preview that looks like a different map than the export: the live image must come from the same code path
  as the export, with only the content settings swapped.
- The page freezing or showing an older image after a newer one, when the user clicks quickly through the
  groups.
- New system dependencies or C libraries. The program stays one static executable.

## Decisions

### Image

- Format: lossless WebP (my call; the user suggested WebP). A map has flat colours; lossless WebP keeps the lines
  crisp and is smaller than PNG. The `image` crate encodes it with the `webp` feature (pure Rust).
- Rasteriser: `tiny-skia` (pure Rust, BSD-3-Clause, compatible with GPL-3.0). New module
  `crates/freemapper/src/render/raster.rs`, a third backend next to `render/pdf.rs`, from the same
  `draw::Primitive`s. Even-odd fills, strokes with width, cap and join, painted by colour priority as the PDF
  does. Colours: the sRGB of the symbol set's CMYK, with the conversion the PDF and freemapper-render use (check
  `freemapper_render::svg::srgb` or the equivalent in this crate; take one, do not invent a new one).
  `Shape::Text` is skipped: the map part holds no text objects today, and the sheet is not drawn. White
  background.
- Size: the long side of the map area gets at most 2048 px, and at most 8 px per mm of paper (my call). The
  Viktoriapark sprint area then is about 2000 px wide, a schoolyard at 1:1000 about 6–7 px/mm.
- File: `<name>-preview.webp` in the project folder, next to `<name>.omap` (my call). It is written in the same
  step as the map and replaced with it. Writing it goes through `write_inside` like every other output.
- The golden net does not hash it: it is written by its own export function, and G4 calls the export functions
  one by one. Its bytes may differ between systems (anti-aliasing with floats). A unit test checks it by decoding
  it (size, a few pixel colours).

### Fast live preview

Measured on 2026-11-12 (Viktoriapark, release build, Apple M-series): `starter_documents` takes about 7.5 s.
About 4.4–6.4 s go to reading the stored data and building every object, before `route` applies the content
settings. `route` itself takes under 1 ms. After it, cutting contours and north lines at ways takes 0.44 s,
the paved borders (`border_paved_areas`) take 3.1 s, ticks and point gaps a few ms.

So:
- `starter_documents` is split in two, in `project/export.rs`:
  - a build phase that reads the data and builds the objects up to the point where `route` runs, and returns
    a value that holds all of it (`BuiltProject` or similar);
  - a placement phase that takes that value and a `ContentSettings` and does `route` and everything after it,
    returning `StarterDocuments`.
  `starter_documents(dir)` stays as it is for its callers: build, then place with the project's settings.
- Every expensive step of the placement phase whose result does not depend on the content settings is computed
  once in the build phase. The paved borders are the case to look at first: `route` only moves objects between
  map and candidates or drops them, and the borders are computed over both lists. If a step does depend on the
  settings, keep it in the placement phase and measure; the target is a placement phase of at most about 0.5 s
  on Viktoriapark in a release build, plus rasterising and encoding.
- The output must stay byte-identical. The golden net G4 is the proof; run `scripts/golden.sh test`.

### Server (`freemapper select`)

- `GET api/map-image`: the stored `<name>-preview.webp` (`image/webp`), 404 when there is none.
  `Cache-Control: no-store`.
- `POST api/map-image` with `{ "content": <ContentSettings> }` (the shape of ADR 0101; absent means the
  project's): builds the live preview and keeps it in memory, answers
  `{ "id": <n>, "width": <px>, "height": <px>, "content": <the settings it used> }`. Then
  `GET api/map-image/live/<id>` serves that image. Only the newest live image is kept; an older id is 404.
  This keeps the content security policy as it is (`img-src 'self'`, no `blob:`). My call.
- The build phase is cached in the server's memory. The cache key is the project file's revision without the
  `[content]` table plus the modification times of `data/` and `templates/` (or what the implementation finds
  reliable); after a run or a save of the area it is built again. One live preview is computed at a time
  (`spawn_blocking`, a mutex). A request that arrives while one is computed waits; the page drops answers that
  are no longer the newest.
- Refusals with a sentence in the page language (`page_lang`, like the other routes):
  - 409 when a run is going on for the folder;
  - 409 when there is no fetched data yet ("Erzeugen Sie zuerst die Karte." / "Create the map first.");
  - 409 when the stored data does not belong to the saved area any more (area, kind or scale changed since the
    last run) – ask for a new run;
  - 422 for invalid content, as `api/save` does (`freemapper_web::refused`).
- After a run ends, the page fetches `GET api/map-image` again.

### Page

- A section "Kartenvorschau" / "Map preview" in the run panel, below the start button and the run status
  (my call). It holds the image, scaled to the panel's width, and a line that says what it shows:
  - "Stand der zuletzt erzeugten Karte." / "As generated last.";
  - "Mit den Einstellungen auf dieser Seite, noch nicht gespeichert." / "With the settings on this page, not
    saved yet.";
  - "Wird berechnet …" / "Computing …" while a live preview is computed (the old image stays, dimmed);
  - "Noch keine Karte erzeugt." / "No map generated yet." when there is none;
  - a short sentence when the area, kind or scale changed since the last run.
  Exact wording: the implementer writes it under the house rules (unslop, humanizer, „…“ in German).
- A link "In voller Größe öffnen" / "Open full size" opens the image in a new tab.
- A change of the content profile or a group asks for a live preview after 400 ms without further change
  (debounce). A sequence number makes sure only the newest answer replaces the image.
- When the page's content settings equal the saved ones and the last run used them, the page shows the stored
  image instead of computing one.
- All texts are `page-*` catalog messages in DE and EN; `every_page_message_is_used` must see them.
- The page stays within its budget (page rules test, 300 KiB).

## Constraints

- House rules of the repository (AGENTS.md "Working on the freemapper code"): English code and technical docs,
  glossary terms, German says "Bundesland", ADR per decision, changelog DE+EN, unslop and humanizer on prose.
- Checks before each commit: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo clippy -p freemapper-cli --all-targets --features test-stub -- -D warnings`,
  `cargo test --workspace --locked`, and for the build split `scripts/golden.sh test`.
- No personal data in requests, fixtures or files.
- Other sessions: freemapper-29 has a local branch `park-terrain` that touches `project/export.rs`,
  `layers.rs` and MCP `preview_layer`; FableWorker works on adapter stage D. Tell both before touching
  `project/export.rs`, and keep the split of `starter_documents` as small a diff as the goals allow.
  freemapper-29 said (2026-11-12) that park-terrain adds landform candidates right before `route` (they need every
  object built so far, `water.union`, `footprints`, `area`, `georef`), a hidden relief template, and later
  vegetation areas at the same place. All of that belongs to the build phase: the split must leave a clear spot
  before `route` where such builders go, and the build phase must carry what the steps after `route` need.

## Docs

- ADR (next free number): the preview file, the raster backend, the split of `starter_documents`, the live
  preview routes and why the CSP stays.
- Manual DE+EN: area page chapter (the preview section) and the list of files in the project folder.
- AGENTS.md "What lands in the project folder": `<name>-preview.webp`.
- `scripts/browser-smoke.md`: a step for the preview (run, image shows; change a group, image changes and says
  "not saved yet").
- Changelog DE+EN.

## Settled after the builder check (all my calls)

1. **Kept map.** The preview follows the `.omap`: it is written exactly when the `.omap` is written, by the same
   export step, and never when an existing map is kept. A folder with a map from an older freemapper has no
   preview; the page says "No map generated yet" until the next run with `force`.
2. **Which data and settings the preview belongs to.** Next to the preview, the export writes
   `data/map-preview.json`: a fingerprint of the project file without its `[content]` table (the TOML the
   library would write with `content` set to `None`, hashed with SHA-256), and the content settings the export
   used. `data/` is the internal store, so this adds nothing users see, and `freemapper.toml` keeps its shape
   (contract test). The server compares that fingerprint with the saved project:
   - equal: the stored data fits; live previews are allowed. If the page's content equals the recorded settings,
     the page shows the stored image;
   - different (area, kind, scale or anything else outside `[content]` changed since the last export): 409
     "a new run is needed", and the page shows the stored image with that sentence;
   - missing: no preview yet.
   The cache key of the build phase is the same fingerprint plus the modification times of the files under
   `data/` and `templates/`.
3. **Clip and orientation.** Exactly as the map PDF: the rectangle `PaperRect::of_bbox(project.bbox())` on
   paper, in paper orientation (rotated by the grivation like the PDF). The pixel size comes from that
   rectangle.
4. **Parts.** The image draws the base part (`walkable_base`, sprint only) and the map part, never the sheet
   part. `walkable_base` belongs to the build phase. Text shapes are skipped (no font in the raster backend);
   the ADR says so.
5. **Build value and fallback.** The cached build value holds the map and candidate lists before `route`; the
   placement phase clones them. If a step after `route` depends on the settings and cannot be precomputed, it
   stays and the measured time goes into the report: the feature does not depend on hitting 0.5 s, the page
   shows "Computing …" for as long as it takes. A live preview over 2 s on Viktoriapark is reported as a concern.
6. **Routes.** To avoid confusion with the existing `POST api/preview` (area outline and tiles), the new routes
   are `GET api/map-image`, `POST api/map-image` and `GET api/map-image/live/<id>`.
7. **Dependencies.** `tiny-skia` and the `webp` feature of `image` are new; both are pure Rust with permissive
   licences (BSD-3-Clause, MIT/Apache-2.0). The repository has no licence checker; the ADR names the licences.
   The release builds stay static (musl on Linux); CI's release build shows it.
8. **Colours.** `render/pdf.rs` writes CMYK; the raster needs sRGB. Use an existing CMYK→sRGB conversion
   (freemapper-core's renderer has one for its SVG backend) rather than a new formula, and say which in the ADR.

## Left to the builder

- The names of the types and functions of the split, and how the cache key is computed.
- Whether rasterising runs in parallel (no new thread pool dependency).
- The exact layout and wording of the section within the house style of the page.
