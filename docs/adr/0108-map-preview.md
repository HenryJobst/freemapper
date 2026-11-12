# 0108. A preview of the map: file, live image and routes

Date: 2026-11-12
Status: Accepted

## Context

To see what a content setting does (ADR 0100, 0101), the user had to run the whole pipeline, which takes minutes,
and then open the `.omap` or the PDF. The user asked to see the generated map on the area page (ADR 0090), with
an image file written by the map generation, and to see the effect of a changed option right away.

## Decision

- **The file.** Every export that writes a map also writes `<name>-preview.webp` next to `<name>.omap`: `run`,
  `export`, MCP `start_pipeline` and "Create the map" on the page. When an existing map is kept, nothing is
  written, so the preview never describes a map that is not there. The image shows the base part and the map part
  as the map PDF draws them, clipped to the same paper rectangle and turned by the grivation like the PDF. It
  shows no candidates, no templates and no sheet. Text is not drawn: the raster backend has no font, and the map
  part holds no text objects today. The long side is at most 2048 px and at most 8 px per mm of paper, on white.
- **The record.** Next to it the export writes `data/map-preview.json`: the SHA-256 of the project file without its
  `[content]` table and without the title (the title is on the sheet only), and the content settings the export
  used. `data/` is internal, so users see no new file, and `freemapper.toml` keeps its shape (contract test).
  Image and record are written together or not at all.
- **The backend.** `render/raster.rs` is a third backend beside the PDF, fed with the same `draw::Primitive`s,
  with the PDF's colour priority, even-odd fills and the PDF's miter limit. The rasteriser is `tiny-skia` 0.12
  (pure Rust, BSD-3-Clause, compatible with the GPL-3.0). WebP is the `webp` feature of the `image` crate
  (`image-webp`, pure Rust, MIT or Apache-2.0), lossless: a map has flat colours, and lossless WebP keeps the
  lines crisp and is smaller than PNG. CMYK becomes sRGB with the formula of `freemapper_render::svg::srgb` (core
  v0.21.1), copied with a note, because freemapper does not depend on that crate. The release builds stay static
  and nothing needs a system library.
- **The split.** `starter_documents` is two steps: `build_project` reads the data and builds the objects up to
  the point where `route` runs; `place` applies the `ContentSettings` and does everything after it. Builders
  that need every object so far belong to the build step. On the Viktoriapark example (release build) the build
  takes about 4 to 6 s, `place` about 0.5 s and rasterising plus encoding 0.1 to 0.5 s. The paved borders took 3 s
  of the old total. They depend on the routing only through the 501.5 areas and 501.1 lines (the groups
  `streets`), so they are kept in a store keyed by their input (at most 8 entries), never by the settings. A wrong
  guess then costs time and cannot change the output. `.omap`, candidates, OCAD and PDFs stay byte-identical; the
  golden net proves it.
- **The routes** of `freemapper select`, behind the page token and its guards, in the page language:
  - `GET api/map-image`: the stored image, `no-store`; 404 without a record or image.
  - `GET api/map-image/state`: `{ stored, fresh, content }`. `fresh` means the record's fingerprint equals the
    saved project's.
  - `POST api/map-image` with `{ "content": … }` (absent means the saved settings): builds a live image, keeps
    it in memory and answers its id and size. `GET api/map-image/live/<id>` serves it. Only the newest is kept,
    an older id is 404. Serving it from the page's own origin keeps the content security policy as it is
    (`img-src 'self'`, no `blob:`).
  - Refusals: 409 while a run goes on or another process holds the folder, 409 without a map, 409 when the
    stored data no longer belongs to the saved project (area, kind or scale changed) and asks for a new run, 422
    for invalid content, 400 for a body that is not a JSON object.
  The name `api/map-image` avoids `api/preview`, which is the area outline and tiles.
- **The cache.** The server keeps the build in memory, keyed by the record's fingerprint and by the newest
  modification time, the number and the total size of the entries under `data/` and `templates/`. A live preview
  takes no lock on the folder, so it can never make a run fail: it checks before and after the computation that
  no run started and that the project and the files did not change, drops the build and starts again once if they
  did. One live image is computed at a time.
- **The page** shows the image in a section "Map preview" below the run status, with a line that says what it
  shows: the stored image or the settings on the page. It asks for a live image 400 ms after the last change of
  the content, drops every answer but the newest, dims the old image while it computes and never asks while a run
  starts or goes on. The record is the authority: with equal settings the stored image is shown, with an area,
  kind or scale changed since the run the stored image with the sentence that a new run is needed. A save sends
  the area as it came from `api/project`, so the fingerprint does not depend on how the page writes the shape.

## Consequences

- A group change shows its effect in about a second on a large sprint area, after one build of a few seconds.
- The bytes of the WebP may differ between systems (anti-aliasing with floats), so the golden net does not hash
  it; a unit test decodes it instead.
- A map written by an older freemapper has no preview until the next run with `force`; the page says so.
- Two tabs share one live slot: a tab whose image was replaced asks again. A title change does not make the
  preview stale; every other change outside `[content]` does.
- Not in this change: a `map` layer for MCP `preview_layer`, and an overlay on the Leaflet map (the paper is
  rotated against north).

Source: the user's request of 2026-11-12 (map preview on the area page).
