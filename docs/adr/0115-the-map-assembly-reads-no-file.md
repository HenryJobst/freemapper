# 0115. The map assembly reads no file

Date: 2026-11-13
Status: Accepted

Chosen by the user on 2026-11-13 from the architecture review of 2026-11-12 (candidate "map pipeline"), with the
cut and the name decided the same day.

## Context

The map and the candidates of a project come out of two functions in `project/export.rs`. `build_project` (380
lines) reads the stored data of the project folder through some 14 `stored_*` functions, each of which reads a
file and draws its objects, then resolves the duplicates between the sources in 140 lines, generalizes the
points and collects the lists. `place` (100 lines, [0108](0108-map-preview.md)) sends the
objects to the map or the candidates by the content settings ([0100](0100-content-profiles.md)) and runs the
steps that look at both lists.

The order of these steps is the knowledge of the build: a tree is deduplicated before it is generalized,
landforms see everything drawn so far, vegetation is cut after the routing. That order lives in a module whose
job is the files of a project, between file reads. A test of the order needs a project folder with stored data,
so the 300 tests of `project.rs` write files to try a rule of the build, and a new step is added by finding the
right line in a long function.

The review called the candidate "map pipeline". The glossary has "pipeline" for the pipe above ground (528,
529), and `start_pipeline` of the MCP server means fetch, derive and export together.

## Decision

- **A module of its own: the map assembly** (`build/assembly.rs`, German "Kartenaufbau"). It puts the objects
  of all sources together to the map and the candidates in a fixed order of steps.
- **The assembly reads no file.** It gets the stored data read (`Stored`) and returns the objects. The project
  reads the folder in one place, calls the assembly and makes the documents (notes, templates, sheet, parts)
  from what it returns.
- **Its interface is two calls**: build everything once (`Built`), place it by content settings as often as
  asked (`Built::place -> Placed`). The area page keeps a `Built` and places it again for every change of the
  settings, as before.
- **Three pull requests, each with every output byte for byte** (`refactor`, the golden net without a blessed
  file):
  1. Placing: `place` and the store of the paved borders move to the assembly. `BuiltProject` keeps its name and
     its public functions and holds the objects as `assembly::Built`.
  2. The duplicates: the block of `build_project` from the candidates of the registers to the point
     generalization becomes a function of the assembly, `resolve(Drawn, &Surroundings) -> Resolved`. The project
     draws the sources first (OSM, the cadastres, canopies, areas that shall not be entered, paving, open land)
     and hands them over; what `resolve` only looks at comes back as it was.
  3. Reading: each `stored_*` function is split into reading its file and drawing its objects; the readers fill
     `Stored`, the assembly draws.
- **Not chosen:** a trait for the stored data with a folder and a memory adapter. It would have some 14 methods
  that hide nothing, one per file. Moving only the two pure parts (steps 1 and 2) and stopping there would leave
  the long function and its need for a folder.

## Consequences

- A rule of the order is tested with a few objects in memory (`build/assembly.rs` has the first such tests: a
  contour cut at paving, the borders computed once per outcome of the streets group, vegetation cut after the
  routing). The tests of `project.rs` that only try the order can move there over time; those that try the
  reading stay.
- A new step of the build has one place, and its neighbours say what it may look at.
- The source and the rank of an object stay tags (`freemapper:source`, `dedup::rank`); a module for them is
  another candidate of the review and not part of this.

## As built (step 3)

- `assembly::build(Frame, Stored, &ContentSettings) -> Built` holds what stood in `build_project` between the
  reads and the documents. `Frame` is what the project file says about the map (kind, georeferencing, symbol set,
  map area with the id of its OSM object, buffer, clip window). `Stored` is the stored data, read: the buildings
  with the date of their fetch, the layers of Berlin, the full ALKIS and the tree cadastres (name, features,
  fetch date), the land use, the OSM features with the date of their extract, the terrain for the stair
  direction and for the ticks, the contours, landforms and vegetation of `derive`.
- `build_project` reads the folder in one function (`read_stored`), calls `build` and makes the documents:
  notes, templates, sheet, credit lines. For the credit lines `Built::point_candidates` gives the candidates a
  register is named by.
- What a reader checks stays in the reader with its error: a fetch of the buildings or of OSM that does not
  cover the area, a file that does not parse. The terrain for the stairs is read only when a layer gives stairs,
  the tree cadastres are not read for a forest map.
- The drawing of contours, landforms and vegetation left `project/export.rs` for `build/contours.rs`,
  `build/landforms.rs` and `build/vegetation.rs`; the thin wrappers `stored_*_objects` of `project.rs` are gone,
  the assembly calls the drawing functions of `build/` itself.
- `resolve`, `Drawn`, `Resolved`, `Surroundings` and `Built::new` are inner steps of the module and no longer
  public.
- Left in the project: `map_lacks_buildings` and `map_lacks_canopies` draw the buildings and the canopies on
  their own to say whether a written map lacks them. They read a written map, which the assembly does not, and
  stay where they are.
