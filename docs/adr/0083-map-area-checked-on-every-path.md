# 0083. The map area is checked on every path

Date: 2026-11-10
Status: Accepted

## Context

`MapArea::validate()` (at least three corners, no crossing edges, at most 5000 corners) ran only on the area selection
page. `init --geojson`, the MCP tool `create_project(geojson)` and a hand-edited `freemapper.toml` took polygons
unchecked. `{"type":"Polygon","coordinates":[]}` hit an `expect("map area polygon is not empty")` and panicked, in the
MCP server inside the async handler. Bow-tie polygons and outlines with more than 5000 corners passed, and the
crossing check is quadratic.

## Decision

- `MapArea::from_geojson` refuses a polygon without corners (`not a usable map area: the polygon has no corners`).
  `validate()` does the same for an area built by hand.
- `init_project` and `set_map_area` call `validate()` before they touch the disk. The CLI (`init --geojson`) and the
  MCP handler call it as well, so the error comes before any place lookup or folder is created.
- `ProjectFile::area()` reads the stored area and validates it; `bbox()`, `region_bbox()` and every later step that
  needs the polygon use it. `ProjectFile::load` itself does not fail on a bad area, so the area selection page can still
  replace it.
- The error text is the existing one (`not a usable map area: ...`) and names the reason: no corners, too many corners
  (with the limit), or crossing edges.

## Consequences

- No panic on user input for these paths.
- A project file with a crossing outline that used to run now stops with a clear error. Fix: draw the area again with
  `select`.
- Cost if wrong: a legitimate outline with more than 5000 corners needs simplifying first. That limit was there already.
