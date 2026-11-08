# 0020. One example project per state

Date: 2026-11-08
Status: Accepted

## Context

The project wanted one example per supported state: an area from which freemapper can derive a lot, such as an
old town, a castle or a park with relief, dense buildings, stairs, walls and hedges.

## Decision

`examples/` holds one sprint map (1:4000, ISSprOM 2019) per supported state, made with `freemapper run --pdf
--ocad`. Each area was chosen from five or six candidates per state by symbol and object count and a visual
comparison. An example has the OOM map, the candidates template, the OCAD files, the PDF and a preview image.
The large raster templates and `data/` are not committed; `run --ocad --pdf` restores them. A state's example is
added on main after its branch is merged.

## Consequences

- Every new region adds an example and a row in the examples tables.
- Cost if wrong: the committed examples age as the source data changes.

Source: commits ba618f3, afd8264, b1755d5, 24ae50f; Hesse rulings (Task 6); `examples/README.md`.
