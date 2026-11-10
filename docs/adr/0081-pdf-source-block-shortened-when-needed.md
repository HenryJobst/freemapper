# 0081. The source block of the PDF is shortened when needed and can be switched to the credit line

Date: 2026-11-10
Status: Accepted, extends ADR 0080

## Context

The PDF of a Berlin map printed nine lines of small print under the title block, one for each source with licence.
The user asked for the sources to be shortened when needed, and for the block to be switchable. The freemapper repo
credit line is not switchable (suite rule: every PDF names freemapper and its URL).

## Decision

- `--pdf-sources <auto|full|credit-only>` (CLI `export` and `run`, MCP `pdf_sources`), default `auto`.
  - `auto`: a block of up to 5 lines (the credit line included) is printed as it is. A longer one is compacted.
  - `full`: the whole list, never shortened.
  - `credit-only`: no data-source lines, only the freemapper credit line. The CLI and the MCP summary print a
    notice, because ODbL and `dl-de/by-2-0` ask for the sources when the map is passed on.
- Compaction, step by step until the block fits: (1) the `dl-de/zero-2-0` sources of one provider become one line,
  with "Daten geändert" or "teils Daten geändert" when some of them were changed; (2) long names get short forms
  ("Karte 1:5000 von Berlin (K5)" becomes "K5"); (3) neighbouring lines are joined with " · " up to the line width,
  which is the map width on paper (100 to 220 mm), measured with the Helvetica widths at 1.8 mm; (4) the
  `dl-de/zero-2-0` lines shrink to the provider names plus "Vollständige Quellen: ATTRIBUTION.txt im Projektordner".
- Kept word for word in every step: the OpenStreetMap credit, every line with another licence (`dl-de/by-2-0`, CC BY,
  the licence texts) and the credit line. Only `dl-de/zero-2-0` asks for no attribution, so only those lines shrink.
- The freemapper credit line is exempt from shortening and from all switches. Neither `--pdf-sources` nor any other
  option or project setting removes it.
- `ATTRIBUTION.txt`, the OOM map and the OCAD file keep the full list in every mode. Only the PDF changes; the PDF
  sheet is built again from the same texts (`PdfSources`, `build::source_block`).

## Consequences

- Short blocks (Sanssouci, Cochem and the other examples, 3 to 4 lines) stay as they are. Viktoriapark drops from
  nine lines to four.
- With `credit-only` the user names the sources when passing the map on; the notice says so.
- Cost if wrong: a source with a new licence format counts as unmergeable and stays word for word, so a long list
  of such lines can exceed the budget; the pointer line is still added.
