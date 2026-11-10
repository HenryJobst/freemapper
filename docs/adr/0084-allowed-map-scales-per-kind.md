# 0084. Allowed map scales per kind

Date: 2026-11-10
Status: Accepted

## Context

`init_project` accepted any scale above zero. `init --kind schoolyard --scale 2000` ran through the whole pipeline, and
freemapper-print then stopped with `MapScaleNotAllowed`; a sprint map at 1:2000 was refused the same way.
freemapper-print confirmed its tables (`iof-print.toml`): sprint 1:3000, 1:4000 and 1:5000, schoolyard 1:1000 only. The
forest scales (1:5000 to 1:20000) belong to print alone, since freemapper has no forest kind.

The symbol set of freemapper depends on the map kind only. A sprint map therefore always carries the symbols of 1:4000 at
100 %. freemapper-print expects 133 % symbols at 1:3000, so a map written at 1:3000 with the 1:4000 symbols would print
with symbols that are too small.

## Decision

- Allowed scales: schoolyard 1:1000; sprint 1:4000 (default) and 1:5000. `MapKind::allowed_scales` holds the table.
- 1:5000 is not in the sprint norm. freemapper accepts it, `init` prints a note, and freemapper-print warns; the symbols
  stay at 100 %.
- `init`, MCP `create_project` and the area selection page refuse other values with an error that lists the allowed
  ones: `scale 1:2000 is not allowed for schoolyard maps; use 1:1000`. For sprint 1:3000 the error adds the way out:
  "Draw at 1:4000, freemapper-print enlarges to 1:3000 when printing". The selection page offers the allowed scales as a
  list.
- A stored scale outside the table does not stop `ProjectFile::load`. It prints a warning
  (`warning: freemapper.toml: scale 1:2000 is none of the scales for schoolyard maps (1:1000) ...`), so older projects
  stay usable.
- Scale 0 stays `invalid scale`.

## Open points

Sprint maps at 1:3000 need a decision about who scales the symbols. The user could not be reached; the coordinator
decided for option B on 2026-11-10 and recorded it here.

- Option A: freemapper scales the symbols itself. The symbol set would depend on kind and scale (133 % at 1:3000), the
  `.omap` and OCAD files would be right for 1:3000 as written, and print would have to know not to enlarge twice.
- Option B (chosen): sprint maps are written only at 1:4000 (100 % symbols) or 1:5000. Exactly one place scales symbols,
  freemapper-print, which already does it for 1:3000.

Cost if wrong: users who want a 1:3000 `.omap` or OCAD file wait for freemapper-side symbol scaling (option A) and meanwhile
get the enlargement only in the printed PDF.

## Consequences

- A scale that print would refuse now fails at `init`, in seconds, instead of after a long run.
- Projects written before this ADR with another scale load with a warning and are refused by print, as before.
- Existing sprint projects at 1:3000 (if any) keep their 100 % symbols; print enlarges them.
