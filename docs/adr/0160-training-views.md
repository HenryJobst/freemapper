# 0160. Training views

Date: 2026-11-20
Status: Accepted

## Context

Trainers use maps with a class of symbols left out: Ultimate maps without paths, contour orienteering with only
brown symbols, maps without north lines (harder, a form of training). The training map (ADR 0140, 0150) is a fixed
product; it has no way to leave something out. The north lines are objects of the `.omap` (ISSprOM 601), so a
printer cannot add or drop them by itself.

## Decision

- A view is a rule on a symbol's code and colours, with three presets (`no-paths`, `brown-only`, `no-north-lines`)
  and a free list. The rule lives in freemapper-core (`freemapper-omap`, module `view`) and is used by both
  freemapper and freemapper-print.
- freemapper writes a variant file per view next to the full training map (`export --training --view`). It cuts
  the variant from the finished objects, so a view never changes the cleaning of the training map.
- freemapper-print takes the same `--view` on any `.omap`.
- A view never removes the frame (information block, title, scale) or the credit lines.

The design is `docs/superpowers/specs/2026-11-20-training-views-design.md`.

## Consequences

- One definition of the presets, two ways to use it. A change to a preset changes both.
- The training map is not renewed for a view: trees that moved off a path stay where they are when the paths go.
- freemapper-core gets a minor release; freemapper-print follows with its option.
