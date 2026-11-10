# 0080. The PDF is the map without candidates, the scale bar is optional

Date: 2026-11-10
Status: Accepted, amends ADR 0002

## Context

ADR 0002 made `--pdf` the quick training map: every candidate drawn as a map object, with a scale bar in the title
block. Most people print a map of the sure objects only, and German orienteering maps usually carry no scale bar. On
2026-11-10 the user asked for the PDF without candidates by default, with candidates on request, and for the scale bar
to be off unless asked.

## Decision

- `--pdf` (CLI `export` and `run`, MCP `pdf`) writes `<name>.pdf` with exactly the objects of the OOM map (base fill,
  buildings, areas, water, north lines) and the sheet: title, scale number, contour interval, data credits, freemapper
  credit line.
- `--pdf-candidates` (MCP `pdf_candidates`) writes `<name>-candidates.pdf`: the map plus every candidate, as the old
  `--pdf` did, still without the map area outline, points hidden under a 520 candidate and unreachable passages
  (`freemapper:hidden`). The two switches are independent; together they write both files side by side. File names
  follow the map file, and both go through the rooted no-follow writer (ADR 0060).
- `--pdf-scale-bar` (MCP `pdf_scale_bar`) draws the scale bar on whichever PDFs are written. The scale number in the
  title block stays. The OOM and OCAD maps keep the bar,
  only the PDF leaves it out.
- The choice is not stored in `freemapper.toml`: the project file records no output choices, and `--ocad` and `--pdf`
  never were. The area selection page has no output controls, so it changes nothing.

## Consequences

- Default change: `--pdf` no longer draws candidates and no scale bar. The old output is `--pdf-candidates
  --pdf-scale-bar`. The changelog says so.
- Every example keeps `<name>.pdf` and `preview.jpg`. Only two examples, Cochem and Blankenese, also keep
  `<name>-candidates.pdf` and `preview-candidates.jpg`; they are the pair on the comparison page
  (`examples/vergleich.md`, `examples/compare.en.md`), chosen by the pixel difference between the two renders.
- Cost if wrong: users who want the candidates by default pass one more switch.

## Amendment 2026-11-10: the bar stays a sheet object

The first version tagged the bar objects `freemapper:source=scale-bar`. freemapper-print and freemapper-courseplan
recognise sheet objects (frame text, title, scale) only by `freemapper:source=sheet`, so a v0.9.0 map had its bar
printed as map content, which shifted the page centre and widened the courseplan extent.

- `freemapper:source=sheet` is the stable contract for every sheet object, the bar included.
- The bar objects (line, ticks and labels) carry a second tag, `freemapper:sheet=scale-bar`. The PDF export filters on
  that tag. The OOM and OCAD maps keep the bar as before.
- No object carries the source `scale-bar` any more. Maps written by v0.9.0 (the examples among them) still have it;
  print and courseplan accept both values, and freemapper does not rewrite old files.
