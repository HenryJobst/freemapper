# Map preview Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every map export writes `<name>-preview.webp`; the area page shows it and recomputes it within about a second when the content settings on the page change.

**Architecture:** A raster backend (`tiny-skia`) draws the same `draw::Primitive`s as the PDF; WebP lossless via `image`. `starter_documents` splits into a cached build phase and a cheap placement phase that applies `ContentSettings`. `freemapper select` gets three `api/map-image` routes with an in-memory cache of the build phase; the page shows the image and asks for a live one after content changes.

**Tech Stack:** Rust 2024 (pinned toolchain), `tiny-skia`, `image` (`webp` feature), axum 0.8, plain JS page (`web/app.js`), Fluent catalogs.

**Spec:** `docs/superpowers/specs/2026-11-12-map-preview-design.md` (read it whole; its "Settled after the builder check" section overrides earlier sections where they differ).

## Global Constraints

- `.omap`, candidates map, OCAD files and PDFs stay byte-identical: `scripts/golden.sh test` passes with the existing `expected.sha256` files. No `Golden-Bless:` trailer in this plan.
- The preview and `data/map-preview.json` are written only by `export_map_preview_from`, never by `export_starter_project_from` (G4 calls the latter and must not see a new file).
- No C libraries, no new system dependencies; new crates only `tiny-skia` and the `webp` feature of `image`.
- `freemapper.toml` keeps its shape (contract test); nothing new in it.
- House rules and checks: `/Users/henryprivat/Documents/gitrepos/freemapper/.superpowers/sdd/2026-11-11-self-update/common-context-b.md` (ignore its worktree path). Checks before each commit: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo clippy -p freemapper-cli --all-targets --features test-stub -- -D warnings`; `cargo test --workspace --locked`; Tasks 1 and 2 also `scripts/golden.sh test`.
- Disk is tight: build only in the worktree, `CARGO_INCREMENTAL=0` for release builds.
- No personal data in requests, fixtures or files.

## Review Focus

1. A project whose map is kept (exists, no `force`): no preview written, and an old preview is not left claiming to show the current settings (the sidecar still names the settings of the export that wrote it).
2. Rapid content changes on the page: only the newest live image is shown; no request storm (debounce) and no stale image after a newer one.
3. Area saved but not run again: live preview refused with 409 and the page says a new run is needed; the stored image stays visible.
4. A run going on while the page asks for a live preview: 409, and no read of half-written `data/`.
5. Paper orientation and clip: the image matches the map PDF's map area (same rectangle, same rotation), not a north-up bbox.

---

### Task 1: Raster backend and the preview file

**Files:**
- Modify: `Cargo.toml` (workspace deps: `tiny-skia`; `image` features `+ "webp"`), `crates/freemapper/Cargo.toml`, `Cargo.lock`
- Create: `crates/freemapper/src/render/raster.rs`; Modify: `crates/freemapper/src/render/mod.rs`
- Modify: `crates/freemapper/src/project/export.rs`, `crates/freemapper/src/project.rs` (the run's map step), `crates/freemapper-cli/src/main.rs` (the `export` command)

**Interfaces:**
- Produces:
  - `render::raster::rasterize(primitives: &[Primitive], clip: PaperBox, colours: &[RenderColour], px_per_mm: f64) -> image::RgbaImage` — white background, painted by colour priority exactly as `render::pdf` orders them, even-odd fills, strokes with width/cap/join, `Shape::Text` skipped. Paper mm with the same axis convention as the PDF; the image's top-left is the clip's top-left as the PDF page shows it.
  - `render::raster::preview_px_per_mm(clip: &PaperBox) -> f64` = `min(8.0, 2048.0 / long_side_mm)`.
  - `project::export::map_preview_webp(documents: &StarterDocuments, dir: &Path) -> Result<MapImage, ProjectError>` with `pub struct MapImage { pub webp: Vec<u8>, pub width: u32, pub height: u32 }` — draws the base part and the map part (never the sheet part, never candidates), clipped to `PaperRect::of_bbox(project.bbox())` as `write_pdf_file` does.
  - `project::export::export_map_preview_from(dir: &Path, documents: &StarterDocuments) -> Result<PathBuf, ProjectError>` — writes `<name>-preview.webp` (path from `map_preview_path(dir)`) and `data/map-preview.json` `{ "fingerprint": "<sha256 hex>", "content": <ContentSettings the documents were placed with> }` via `write_inside`.
  - `project::export::map_preview_path(dir: &Path) -> PathBuf` (`<stem of map_path>-preview.webp`).
  - `project::export::project_fingerprint(project: &ProjectFile) -> Result<String, ProjectError>` — SHA-256 hex of the TOML the library writes for `project` with `content = None`.
  - `project::export::read_map_preview_record(dir: &Path) -> Option<MapPreviewRecord>` with `pub struct MapPreviewRecord { pub fingerprint: String, pub content: ContentSettings }`.
  - `StarterDocuments` must carry the `ContentSettings` it was placed with (a field), so `export_map_preview_from` can record it.
- Call sites: the library run's `MapOutcome::Written` branch (build the documents once, then `export_starter_project_from` and `export_map_preview_from`); the CLI `export` command's "wrote" branch. Not in the "kept" branches.

- [ ] Step 1: failing unit tests in `raster.rs`: a filled 10×10 mm square in red (priority 1) on a 20×20 mm clip at 4 px/mm → pixel (20,20) red, pixel (70,70) white; an even-odd ring with a hole → hole pixel white; two overlapping fills → the lower priority number is on top; a 1 mm stroke → pixel on the line coloured, 2 mm away white; `preview_px_per_mm` for 300×200 mm = 6.826…, for 100×50 mm = 8.0.
- [ ] Step 2: failing tests in `project/export.rs` (or `project.rs` test module, where export tests live): after building a test project's documents, `export_map_preview_from` writes a WebP that `image` decodes with width/height from `preview_px_per_mm` (±1 px) and a sidecar whose `content` equals the documents' settings; `project_fingerprint` is equal for two projects that differ only in `[content]` and different when the area differs; the library run with an existing map and no `force` writes no preview.
- [ ] Step 3: implement; reuse an existing CMYK→sRGB conversion (freemapper-core's `freemapper_render::svg::srgb` if freemapper already depends on freemapper-render, else the formula from it, with a comment naming the source).
- [ ] Step 4: all checks, `scripts/golden.sh test` green unchanged.
- [ ] Step 5: commit `feat(export): a WebP preview of the map next to the .omap`.

### Task 2: Build phase and placement phase

**Files:**
- Modify: `crates/freemapper/src/project/export.rs` (and `build/paved_borders.rs` only if the precompute needs it)

**Interfaces:**
- Consumes: Task 1's `StarterDocuments` content field.
- Produces:
  - `pub struct BuiltProject` (Clone; fields private) — everything `starter_documents` computes before `route`, plus whatever the steps after `route` need.
  - `pub fn build_project(dir: &Path) -> Result<BuiltProject, ProjectError>`.
  - `pub fn place(built: &BuiltProject, content: &ContentSettings) -> Result<StarterDocuments, ProjectError>`.
  - `pub fn starter_documents(dir: &Path)` keeps its signature and result: build, then place with `content_settings(&project)`.
  - `BuiltProject::fingerprint(&self) -> &str` (the Task 1 fingerprint of the project it was built from).

- [ ] Step 1: a test that `place(&build_project(dir)?, &settings)` equals `starter_documents(dir)` for the project's own settings, compared through the written `.omap` bytes, on a project with buildings, paved areas and a fence; and that two `place` calls with `safe` and `full` give different map part object counts.
- [ ] Step 2: split. Leave a commented spot before `route` in the build phase where new object builders go (freemapper-29's landform candidates go there).
- [ ] Step 3: move every step after `route` whose result does not depend on the content settings into the build phase, starting with the paved borders (3.1 s of 3.7 s on Viktoriapark). Prove independence in a comment (what `route` can change, what the step reads); if a step depends on the settings, keep it and say so in the report.
- [ ] Step 4: `scripts/golden.sh test` green with the unchanged `expected.sha256` (byte-identical outputs) and all checks.
- [ ] Step 5: measure with a release build (`CARGO_INCREMENTAL=0 cargo build --release -p freemapper-cli`) on a copy of `examples/berlin-viktoriapark`: time of `build_project`, of `place`, of `map_preview_webp`. A temporary `eprintln!` is fine for measuring; do not commit it. Put the numbers in the report.
- [ ] Step 6: commit `refactor(export): build once, place by content settings`.

### Task 3: Server routes

**Files:**
- Create: `crates/freemapper-cli/src/select_preview.rs` (or inside `select.rs` if smaller); Modify: `crates/freemapper-cli/src/select.rs` (routes, `AppState`), `crates/freemapper-cli/i18n/{de,en}/freemapper.ftl`, `crates/freemapper-cli/tests/select.rs`

**Interfaces:**
- Consumes: `build_project`, `place`, `map_preview_webp`, `map_preview_path`, `read_map_preview_record`, `project_fingerprint`.
- Produces (HTTP, under the page token, behind the existing guards):
  - `GET api/map-image` → 200 `image/webp` (stored file), `Cache-Control: no-store`; 404 when there is none.
  - `GET api/map-image/state` → `{ "stored": bool, "fresh": bool, "content": <recorded settings or null> }` — `fresh` = the record's fingerprint equals the saved project's. The page needs it to decide between stored image and live preview. (Fold into `api/project` instead if that is simpler; say which in the report.)
  - `POST api/map-image` `{ "content"?: ContentSettings }` → 200 `{ "id": u64, "width", "height", "content" }`; 409 run going on / no data yet / data not for the saved area (each its own catalog sentence in the page language); 422 invalid content via `freemapper_web::refused`.
  - `GET api/map-image/live/{id}` → 200 `image/webp` for the newest id, 404 otherwise.
- Cache: one `BuiltProject` in `AppState`, keyed by the fingerprint plus the newest modification time under `data/` and `templates/`; built on demand in `spawn_blocking`; one live computation at a time (`tokio::sync::Mutex`).

- [ ] Step 1: integration tests in `tests/select.rs` without network: copy a small project with stored data (reuse an existing fixture or build one with the test stub the other select tests use; say which in the report) → `GET api/map-image` 404 before export; after `freemapper export` 200 with `RIFF….WEBP` magic; `POST api/map-image` with `{"content":{"profile":"full"}}` → 200 and `GET …/live/<id>` 200 webp, an older id 404; after saving a different area → 409 with the German sentence for `Accept-Language: de`; invalid content → 422.
- [ ] Step 2: implement; texts as `cli-*` or `page-*` ids per the catalog rules the other select refusals use.
- [ ] Step 3: all checks; commit `feat(select): map image routes with a cached build`.

### Task 4: The page and the docs

**Files:**
- Modify: `crates/freemapper-cli/web/app.js` (and `run.js` if the run panel lives there), `web/index.html`, `web/app.css`, `crates/freemapper-cli/src/select_page.rs` (tests), catalogs
- Create: `docs/adr/<next>-map-preview.md`; Modify: `docs/adr/README.md`, `docs/manual/{de,en}.md`, `AGENTS.md`, `scripts/browser-smoke.md`, `CHANGELOG.md`, `CHANGELOG.en.md`

**Interfaces:**
- Consumes: Task 3's routes.

- [ ] Step 1: the section "Kartenvorschau"/"Map preview" below the run status, as the spec's Page section says: stored image after a run and on load; live image 400 ms after the last content change (sequence number, only the newest answer wins, old image dimmed while computing); the state line; "Open full size" link; controls locked while a run goes on as the other settings are.
- [ ] Step 2: all new texts in both catalogs, `every_page_message_is_used` green; page rules test (300 KiB budget) and CSP test unchanged and green.
- [ ] Step 3: screenshots DE/EN light/dark of the run panel with a preview: `p4-de-light.png`, `p4-de-dark.png`, `p4-en-light.png`, `p4-en-dark.png` in `/private/tmp/claude-502/-Users-henryprivat-Documents-gitrepos-freemapper/626dafd0-2cfb-4a8b-9864-ebf6d40c8bb0/scratchpad/` (headless browser; do not fake; say so if none runs).
- [ ] Step 4: docs: ADR (preview file, raster backend and licences, the split, routes, CSP unchanged, text not drawn), README index of ADRs, manual DE+EN (area page chapter, files in the folder), AGENTS.md "What lands in the project folder", browser smoke step, changelog DE+EN. unslop and humanizer on the prose; German says „…“ and Bundesland.
- [ ] Step 5: all checks; commits `feat(select): the area page shows the map preview` and `docs: map preview`.
