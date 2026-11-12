# 0100. Content profiles decide what goes straight onto the map

Date: 2026-11-12
Status: Accepted

## Context

The build puts every object on the map or among the candidates by the `target` of its rule
(`assets/rules/*.toml`, 18 of 81 OSM rules say `map`, the Berlin, state and NAS rules none) or by the module
that draws it. Buildings, ALKIS areas, water, parks and paths are map objects; walls, fences, hedges, banks,
stairs, trees, contours, areas that shall not be entered and street furniture are candidates, whatever their
source. The generated maps looked empty without the candidates and impressive with them (user, 2026-11-12),
although a wall of the cadastre is as reliable as a building.

## Decision

- `build/content.rs` sorts the objects of the map and of the candidates once more, after the build and before the
  lines are cut and turned (`project/export.rs`). An object belongs to a group by its symbol: walls, fences,
  hedges, slopes, stairs, trees, water-features (ponds, brooks, fountains), monuments, furniture, passages, contours,
  out-of-bounds, paths, streets (street and square areas and the kerbs of the street survey). The group goes to `map`, `candidates` or `off`.
- A profile gives the default. `safe` leaves every object where the build put it (the state before this ADR).
  `balanced`, the default, sends to the map the objects of the cadastre, the registers and the street survey (rank
  survey or higher, `build/dedup.rs`) of every group but furniture, contours and out-of-bounds, and for walls, fences, hedges, water, trees (and the tree
  groups made of them) and paths also those of OpenStreetMap, which is often the only source there (no register
  outside a few cities). Monuments, open land and the rest of OSM stay candidates. The green strips (401) of
  the street survey belong to `streets`. `full` sends everything
  to the map. A choice per group beats the profile.
- Hidden candidate points (`freemapper:hidden`) are never moved.
- The setting is the optional table `[content]` of `freemapper.toml`, flat, with unknown keys refused:
  `profile = "balanced"` and keys such as `walls = "map"`. The commands `init`, `export` and `run` take
  `--content <profile>` and `--content-set <group>=<place>`, MCP the parameters `content` and `content_set` of
  `create_project` and `start_pipeline`. The area page sets it through `api/save` (its own session). A new
  profile starts afresh; `--content-set` alone adds to the saved settings. The setting is saved in the project, so
  page, command line and MCP see the same state.

## Consequences

- The output of all states changes with the default; the commit blesses G4. `safe` keeps the former output.
- The credit lines are computed before the routing, so a register that only shows among the candidates is still
  named.
- The first cut took only ALKIS and the registers. In the Biesdorfer Park that left fences (74 of 98 from OSM, 24 from
  the street survey), the pond (an OSM area), the hedges and the street areas and kerbs (street survey) among the
  candidates; the user saw it on the first map. The ranks above come from that review and from a second
  pass on 2026-11-12: OSM trees, tree groups, OSM paths and railways and the green strips were added.
- Cost if wrong: official but wrong data (an old cadastre fence) now stands on the map unchecked; a user who wants
  the former behaviour writes `--content safe` or `profile = "safe"`.

Source: user decision 2026-11-12 (default balanced; adjustable on the command line, in `freemapper.toml`, in MCP and on the area page).
