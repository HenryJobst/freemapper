# 0108. A preview of the map: file, live image and routes

Date: 2026-11-12
Status: Accepted

## Context

To see what a content setting does (ADR 0100, 0101), the user had to run the whole pipeline and open the `.omap`.
The user asked for the generated map on the area page (ADR 0090), as an image written by the map generation, and
for the effect of a changed option right away.

## Decision

- **File and record.** Every export that writes a map also writes `<name>-preview.webp` (lossless, long side at
  most 2048 px, at most 8 px per mm) and `data/map-preview.json` (SHA-256 of the project file without `[content]`
  and title, plus the content settings used), together or not at all. A kept map gets no preview. The image shows
  base and map part as the map PDF draws them (same clip, same rotation), no candidates, no sheet, no text (the
  raster backend has no font; the map part holds no text objects).
- **Backend.** `render/raster.rs`, fed with the PDF's `draw::Primitive`s: `tiny-skia` 0.12 (BSD-3-Clause), WebP
  via `image-webp` (MIT or Apache-2.0), both pure Rust, so the builds stay static. CMYK to sRGB with the formula
  of `freemapper_render::svg::srgb` (core v0.21.1), copied with a note.
- **Split.** `starter_documents` is `build_project` (data up to `route`) plus `place` (settings and everything
  after). Viktoriapark, release: build 4 to 6 s, `place` about 0.5 s, raster and encode 0.1 to 0.5 s. The paved
  borders (3 s) depend on the routing only through the 501.5 areas and 501.1 lines, so a store keyed by that
  input (8 entries) reuses them; output stays byte-identical (golden net).
- **Routes.** `GET api/map-image` (stored, `no-store`), `GET api/map-image/state` (`stored`, `fresh`, `content`),
  `POST api/map-image` (live image for `{content}`, kept in memory, newest only) and `GET api/map-image/live/<id>`.
  Served from the page's origin, so the CSP stays (`img-src 'self'`). Refusals: 409 during a run, without a map or
  with stale data (area, kind or scale changed), 422 invalid content, 400 bad body. A live preview takes no folder
  lock; it checks before and after that no run started and nothing changed, and retries once.
- **Page.** A section "Map preview" in the run panel. It asks for a live image 400 ms after the last content
  change, keeps only the newest answer, dims the old image meanwhile and asks for nothing during a run. The record
  decides: equal settings show the stored image; a changed project shows it with "a new run is needed"; a map
  without a record (older freemapper) gets its own line. A save sends the area as it came in.

## Consequences

- A group change shows within about a second after one build of a few seconds.
- The WebP bytes may differ between systems, so the golden net does not hash it; a unit test decodes it.
- Two tabs share one live slot; a tab whose image was replaced asks again.
- Later: a `map` layer for MCP `preview_layer`; an overlay on the Leaflet map (the paper is rotated).

Source: the user's request of 2026-11-12.
