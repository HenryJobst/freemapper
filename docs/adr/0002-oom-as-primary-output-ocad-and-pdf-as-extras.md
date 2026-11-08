# 0002. OOM as the primary output, OCAD and PDF as extra outputs

Date: 2026-11-06
Status: Accepted

## Context

OCAD is Windows-only and commercial. OpenOrienteering Mapper (OOM) is free and cross-platform but cannot
generate base material from geodata. On 2026-11-07 the user added two later goals: an OCAD starter project and a
quick training map as a ready-to-print PDF.

## Decision

The main result is a georeferenced OOM starter project (`.omap`) with templates. Drawing and field checks stay
in OOM. Map objects, symbols and templates live in a format-neutral `MapDocument`, and export hands the same
documents to each writer:

- `export --ocad` writes OCAD 12 files directly, with placeholder symbols that OCAD replaces by number when the
  user loads the official symbol set.
- `export --pdf` renders the map with the OOM symbol set into a one-page PDF. Only this switch turns candidates
  into map objects.
- Both switches keep an existing OOM map without `--force`, so OOM work is never overwritten for an extra output.

## Consequences

- Image templates with world files work in OCAD too (ADR 0010).
- freemapper maintains its own OCD writer and a symbol renderer for lines, dashes, areas, points and texts.
- Cost if wrong: an OCAD user has to open the OCD by hand as a background map or fix placeholder symbols; the
  OCAD import of the candidates map was not verified in OCAD at the time.

Source: spec §1, §6 "Later"; commits 327d7d7, 005a28a; plans `2026-11-07-ocad-export.md`,
`2026-11-07-training-pdf.md`; OCAD rulings (Task 3, Task 4) and PDF rulings (Task 5).
