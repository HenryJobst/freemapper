# Paper frames on the area page and as a template layer (issue #343)

Date: 2026-11-27. Status: approved by the user, 2026-11-27.

## Intent and why

A user choosing the area of a forest map does not know which scale gives a sheet that fits the terrain, and
finds out by trying (issue #343, user `surffan`). They want to see what a sheet of A4 or A3 covers at the chosen
scale while they choose the area. They also lay a rectangle of the paper size on a helper layer in OpenOrienteering
Mapper (OOM) by hand today; freemapper can write that rectangle.

The base map stays independent of the paper size. The final sheets are cut later, when the courses are made, with
freemapper-print (owner comment on the issue). The frame is a help to cut out enough area ("better too much than
too little"), not a layout.

## Goals

- On the area page: a draggable frame of a paper format at the scale of the project, to judge scale and area.
- Formats A6, A5, A4, A3, A2, each landscape or portrait (A6 and A5 for trainings, A2 for competitions).
- The user can keep frames: each one is stored with the project and written as a hidden template map, so OOM can
  switch it on and off like a candidate layer.

## Non-goals and anti-goals

- No change to the map area, the scale or any map object by a frame. The frame never clips or moves anything.
- No frame in the base map file itself (the base map holds only reliable content, ADR 0156 keeps candidates out of
  it for the same reason).
- No sheet layout in freemapper: no margins, legend, north arrow, title or cut marks. That is freemapper-print,
  which reads the frames (see below). Changes in print are a separate piece of work in its repo.
- No custom paper sizes, no scale different from the project's (printing at 1:3000 is an enlargement of
  freemapper-print). Both can follow later.
- Anti-goal: a frame that suggests a finished sheet. It is the paper outline, nothing more.

## Constraints

- Source code and technical docs in English, user texts in German and English via the existing text tables
  (`--lang`); glossary terms (`docs/glossary.md`); an ADR (next free number at implementation time, the numbers
  race with other sessions) and changelog entries in `CHANGELOG.md` and `CHANGELOG.en.md`.
- The existing map file may hold the user's work: a run never replaces it and never edits it. (Corrected after
  reading the code: `MapWrite::Keep` only refreshes the old candidates *file*, never the map's template list.)
- Texts for the manual, README and release notes go through the unslop and humanizer skills.

## Approaches considered

1. Page only, nothing stored. Cheapest; does not give the OOM helper layer the user asked for.
2. **Chosen (user, 2026-11-27):** frames set on the page, stored in `freemapper.toml`, written as
   `templates/paper-frame.omap`, a hidden template map like the candidate layers.
3. Frame as a line in the base map. Rejected: pollutes the base map and must be removed before printing.

## Design

### Geometry

The frame is a rectangle on the ground: paper size in millimetres (ISO 216: A2 420 × 594, A3 297 × 420,
A4 210 × 297, A5 148 × 210, A6 105 × 148; landscape swaps the sides) times the scale denominator. Examples: A4
portrait at 1:15000 is 3150 × 4455 m; A6 landscape at 1:4000 is 592 × 420 m. The full sheet is drawn; no margin.
The rectangle is axis-aligned in the map's projected coordinate system (north up), as the map is.
This computation lives once in the Rust crate (`freemapper`); the page gets the ground size in metres from the
server (the page table of formats comes with the project data, like `allowed_scales`), so that page and export
cannot differ.

### Area page (`crates/freemapper-cli/web/`)

- Controls next to the scale selector: format (none, A6 … A2) and orientation (landscape, portrait). Default: no
  frame shown.
- With a format chosen, a rectangle is on the map, centred on the map centre or the area, and the user drags it with
  the mouse. It follows the scale selector and the orientation at once. The metres are computed at the frame centre
  (local metric approximation of the page's web-mercator view is accepted for display; the stored frame is exact).
- Button "Keep frame": adds the frame (format, orientation, centre) to the list of kept frames. The list shows each
  with a delete button. Kept frames are drawn thinner than the dragged one. They are saved with the same save as the
  area, kind, scale and title (`select.rs` body field `paper_frames`; absent keeps, an empty list removes all).
- The frame is shown only for a project whose scale is known: on a new project the scale selector value counts.
  Once a project exists the scale is fixed (the selector is already disabled), so kept frames stay valid.

### Project file

`freemapper.toml` gets a list `paper_frames`; each entry has `format` (`A6`…`A2`), `orientation`
(`landscape`|`portrait`) and the centre in EPSG:4326 (`lon`, `lat`). The scale is the project's. Absent list means
no frames. Reading an older file works unchanged; the list is validated on load (known format, centre inside the
map area plus the frame's own half-size is not required: a frame may reach beyond the area, that is its use).

### Export

- `export` writes `templates/paper-frame.omap` only when frames are stored: one closed line per frame in a helper
  line symbol of its own, name "A4 portrait" (language of the program) as object text or tag, so OOM shows which is which.
  Template maps share the map's georeferencing, so the rectangle is computed in the map's CRS from the centre.
- The base map lists the template after the candidate layers, hidden, at 70 % opacity, as the candidate layers
  (ADR 0156). A newly written map (first export, or `--force`) names the template only when frames are stored. A kept
  map is not edited: the file is written, and the run says in one line that the user opens it in OOM
  (menu Templates, open template map). A kept map that names a template whose file was removed shows OOM's
  missing-template mark until the user drops it there.
- The layer appears in MCP `list_layers` as `paper-frame` (additive). No new MCP tool and no new CLI switch in this
  step; frames are set on the area page only.
- A change of the frames needs `export` only, no `fetch` and no `derive`: they must not enter the fingerprint of the
  derived data.
- Each frame is a closed line object tagged for readers (the suite contract, see "freemapper-print uses the
  frames"): `freemapper:source=paper-frame`, `freemapper:paper=A4`, `freemapper:orientation=portrait`. The
  centre is the middle of the object's bounding box in map coordinates; no CRS is needed to read it.
- PDFs, the check print and the training map do not show the frames. OCAD export: the template is written like
  the candidate layers if the OCAD writer takes a template map without change; otherwise it is left out in this step and
  the manual says so (builder's call, to be reported).

### freemapper-print uses the frames (user, 2026-11-27)

freemapper-print shall evaluate the frames when there are any and use them suitably. The user left the
details to the print side; this is the proposal for that session.

- Contract: the tagged objects of `templates/paper-frame.omap` above. `freemapper.toml` keeps `paper_frames` as
  freemapper's own storage and does not become a promised key. The tag names and values are reserved words of
  `freemapper-contract` (freemapper-core, `tags` module) with a fixture of a project with frames, so the gate in
  core covers print and courseplan.
- Print reads `templates/paper-frame.omap` next to the map (as it finds the sidecar). It never prints the template
  (as with the candidate layers, `tests/candidates.rs` there).
- Use in print (proposal): for each frame it offers a layout with `paper`, `orientation` and
  `map_frame.centre_map_mm` taken from the frame, at the print scale of the map. The frame is the whole sheet; the
  printed map frame is the sheet minus the passe-partout, so the layout may show a little less than the frame:
  deliberate, a frame is "better too much than too little". Whether print only proposes (listing, `init`) or
  creates the layouts, how it names them and how a course is assigned to one, is print's decision.
- print supports `a4|a3` today. Frames in A2, A5 and A6 need those papers there too, or print says that it cannot
  use that frame and names the reason.
- Order: freemapper writes the layer and the tags (this repo), core gets the reserved words and the fixture, then
  print follows. Messages: tell the print session `freemapper-print-5e` after approval of this design so it can plan
  in parallel.

### Docs and process

- Manual (`docs/manual/de.md`, `en.md`): a short section on the frames, with the A4 example numbers above.
- README: only if a feature list there names the area page's controls (re-check before push, per the README rule).
- ADR, changelog entries (DE and EN), glossary entry for "paper frame" / "Papierrahmen".
- Tests: format table and ground size (unit); `paper_frames` round trip in `freemapper.toml`, rejection of an unknown
  format; the template file with N frames and with none; a kept map left byte-for-byte unchanged while the file is written; the save
  body of the area page; `list_layers` entry. Page script: no browser test exists, so the drag/format logic is kept thin
  and the metric part stays in Rust.
- Work as one PR on GitHub, merge on green CI (all three systems only if code differing per system is touched).

## Calls made after the builder check (author's calls, correct them)

- Existing projects: the page may save frames alone on an existing project; kind, scale, title and area stay
  untouched. A frame change is picked up by the next `export` or `run` (the page's run button does it); nothing
  re-exports by itself.
- Display on the page: web-mercator stretches distances by 1/cos(lat), so the page divides the ground size by
  that factor and draws an axis-aligned rectangle. The grid convergence (a degree or two) is ignored on screen. The
  stored frame and the export are exact in the map CRS, so a small rotation against the screen rectangle is
  expected and harmless for judging scale.
- Template references: the new reference goes after the candidate layers, hidden, 70 % opacity. A kept map is not edited
  (see Export), so its template state stays as the user set it.
- Contract order: core first (reserved words `paper-frame`, `freemapper:paper`, `freemapper:orientation` as plain,
  not localised strings such as `A4` and `portrait`, plus the fixture), then the pin move in this repo, as the
  contract rule says. The localised name is object text only and carries no meaning.
- `templates/paper-frame.omap` is freemapper's own output and is rewritten on every `export`. When no frames are
  stored, `export` removes the file and the reference, so print never finds frames that no longer exist.
- All ten format/orientation combinations are offered for every map kind and scale.
- OCAD: if the template needs more than a small change in the OCAD writer, it is left out and the manual says so.

## Left to the builder

- Line colour, width and symbol number of the helper symbol, and the object text form (tag vs. text), as long as
  the layer reads clearly over the orthophoto and is not mistaken for map content.
- How the drag is done in Leaflet without a plugin (vendored Leaflet 1.x has no draggable rectangle).
- Whether the OCAD export takes the template (see Export).
- Page layout and the German and English wording of the controls.
