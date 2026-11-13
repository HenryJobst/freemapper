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
  2. The duplicates: the block of `build_project` between the candidates of the registers and the point
     generalization becomes a function of the assembly.
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
- Until step 3, `build_project` still reads and draws in one function and builds a `Built` from its lists
  (`Built::new` with the `Surroundings`: kind, georeferencing, symbol set, buildings, water, clip window,
  terrain). After step 3 the assembly builds it itself and `Built::new` is no longer public.
- The source and the rank of an object stay tags (`freemapper:source`, `dedup::rank`); a module for them is
  another candidate of the review and not part of this.
