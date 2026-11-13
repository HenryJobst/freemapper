# The map assembly, step 3: the assembly builds from stored data it is handed

> One pull request (the user's choice of 2026-11-13), built in commits that each keep every output byte for
> byte. ADR: [`0115`](../../adr/0115-the-map-assembly-reads-no-file.md). Steps 1 and 2 are merged (pull requests
> 69 and 72).

**Goal:** `build_project` reads the project folder in one place and hands what it read to
`assembly::build`. The assembly draws the sources, resolves the duplicates, collects the lists and returns a
`Built`. No function of the assembly takes a folder.

## Where it starts

Most sources are split already: `stored_*_layers`, `stored_land_use`, `stored_osm_features` and
`read_stored_buildings` read, the `stored_*_objects` functions beside them draw, and each drawing function has
one caller, `build_project`. Four functions read and draw in one: `stored_berlin_structures` (it reads the
terrain for the stair direction), `stored_contour_objects`, `stored_landform_objects` and
`stored_vegetation_objects`. The tests reach all of them through `starter_documents`.

## Global constraints

- Every output stays byte for byte: `scripts/golden.sh test` passes without a blessed file after every commit;
  every commit is a `refactor`.
- `build_project`, `place`, `starter_documents` and `BuiltProject` keep their names and signatures.
- What a reader checks about the stored data (a fetch that does not cover the area, a file that does not parse)
  stays in the reader and keeps its error.
- The order of the drawing calls stays; where a read moves in front of a drawing call, only the error that is
  reported first can change.
- Merged with `main` often: the sessions of the forest map and the adapters change `build_project` too.

## Commits

### A. The four mixed functions are split

`project/export.rs`, `project.rs`:

- `read_stored_contours(dir, project) -> Result<Option<StoredContours>, ProjectError>` with the lines, the
  interval stored with them, the source and the currency of the terrain; `contour_objects` draws from it.
  The interval for the sheet (stored, else the project's, else the kind's) is worked out by the caller.
- `read_stored_landforms`, `read_stored_vegetation` likewise (forms or areas, source, currency); the drawing
  functions take the data instead of the folder.
- `stored_berlin_structures` takes the stair terrain as an argument; `build_project` reads it when one of the
  stair layers is stored, as before.

### B. The frame and the stored data

`build/assembly.rs`:

- `Frame { map_kind, georef, symbol_set, map_area, buffer_m, clip }`: what the project file says about the map.
- `Stored`: the buildings with their fetch date, the layers of the three families (file or dataset, features,
  fetch date), the land use, the OSM features with the date of their extract, the terrain for the stairs and for
  the ticks, the contours, landforms and vegetation of commit A.
- The drawing wrappers move from `project.rs` and `project/export.rs` into the assembly and take `&Frame` and
  the part of `Stored` they draw from. The project types of the layers stay where they are; the project turns
  them into the neutral ones of `Stored`.

### C. `assembly::build`

- `build(frame, stored, saved) -> Result<Built, _>` holds the body of `build_project` between the reads and the
  documents: draw, `resolve`, the lists in their order, north lines, landforms, vegetation, `Built::new`.
- `Built` says which of its candidates are the points of the registers (for the credit lines) and gives the
  frame back; `Built::new` and `Surroundings` are no longer public.
- `build_project` reads, calls `build` and makes the documents (notes, templates, sheet, credit lines).

### D. Tests and records

- Tests of `build` with a `Stored` in memory: an empty one gives the outline and the north lines; buildings and
  an OSM tree come out in the order of the lists.
- ADR 0115 gets "as built"; glossary and changelogs.

## Review focus

1. The order of the objects in both lists (the golden net holds it).
2. A stored file that is missing, does not cover the area or does not parse: the same error or the same silent
   skip as before, from the reader.
3. The stair terrain is read only when a stair layer is stored.
4. The forest map: no register trees, the OSM forest in the open land, the interval of the stored contours.
