# 0202. Paper frames on the area page and as a template map

Date: 2026-11-27
Status: Accepted

## Context

A mapper who chooses the area of a forest map does not know which scale gives a sheet that fits the terrain, and
finds out by trying (issue #343). They also draw a rectangle of the paper size on a helper layer in OpenOrienteering
Mapper by hand. The base map stays independent of the paper: the sheets are cut later, with freemapper-print, when
the courses are made. The user chose on 2026-11-27 to set frames on the area page, store them and write them as a
template map (spec `docs/superpowers/specs/2026-11-27-paper-frames-design.md`).

## Decision

- A paper frame is the full sheet of A6, A5, A4, A3 or A2 (ISO 216), landscape or portrait, at the project's scale:
  A4 portrait at 1:15000 covers 3150 × 4455 m, A6 landscape at 1:4000 592 × 420 m. No margin, no layout.
  `paper.rs` computes the ground size once; the page gets the table from the server.
- The area page shows a frame the mapper drags and keeps ("Keep frame"). Kept frames are stored in
  `freemapper.toml` as `paper_frames` (format, orientation, centre in EPSG:4326). They do not enter the
  fingerprint of the derived data, so a change needs `export` only.
- `export` writes `templates/paper-frame.omap` when frames are stored: one closed line per frame in a helper line
  symbol of its own, in the map's georeferencing, axis-aligned in map coordinates. Without frames it removes the
  file. A newly written map lists it after the candidate layers, hidden, at 70 % opacity (ADR 0156). A kept map is
  not edited; the run says in one line how to open the template in OOM.
- Each frame carries `freemapper:source=paper-frame`, `freemapper:paper` and `freemapper:orientation`, plain words
  reserved in `freemapper-contract` (core ADR 0046, core v0.33.0). freemapper takes them from
  `freemapper_contract::tags`, and a test checks a written template with `check_paper_frames`. `paper_frames` in
  `freemapper.toml` stays freemapper's own storage, not a contract key.
- MCP `list_layers` names the layer `paper-frame` when the file exists. PDFs, the check print, the training map and
  the OCAD output leave the frames out; OCAD export says so in a warning.

Rejected: frames on the page only, stored nowhere (no helper layer in OOM, which the issue asked for); a frame as a
line in the base map (it pollutes the map of sure objects and has to be removed before printing).

## Consequences

- A mapper sees on the page what a sheet covers at the scale, and finds the frames in OOM as a layer to switch on.
- freemapper-print can read the frames and offer a layout per frame; the contract fixture `paper-frames` covers it.
- A kept map does not list the template until `--force` or until the mapper opens it in OOM. A kept map that lists
  a template whose frames were all removed shows OOM's missing-template mark.
- Cost if wrong: if mappers want frames in the base map or in OCAD, the template has to be copied there; the tags
  stay. If the frame should be the printable map area rather than the whole sheet, the size table changes and
  print must agree, because the tags then mean something else.

Source: spec `docs/superpowers/specs/2026-11-27-paper-frames-design.md`; plan
`docs/superpowers/plans/2026-11-27-paper-frames.md`; issue #343.
