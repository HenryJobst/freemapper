# ADR 0178: The page PDF embeds Arimo

Date: 2026-11-22

Status: Accepted, amends a paragraph of ADR 0164

## Context

The sheet text of the page PDF (title, scale, credit line, names) used the standard fonts Helvetica
and Helvetica-Bold with WinAnsi encoding and no embedded font. WinAnsi has no ł, ą, ę, ś, ż, č, ř,
š, ž, ő and many more letters of the languages the program knows. ADR 0164 therefore printed the
Polish ones with their base letter ("Slubice" for "Słubice") and everything else as `?` (issue
#267). Czech, Slovak, Lithuanian, Latvian, Estonian and Hungarian names were affected too.

freemapper-core embeds Arimo (SIL OFL 1.1, metrically equal to Arial) as a subset in the PDFs of
freemapper-print and offers the writer as public API since v0.32.0 (core ADR 0045):
`PageFonts`, `FontObjects` and the characters without a glyph.

## Decision

- `render/pdf.rs` sets every text in Arimo regular or bold through core's `PageFonts` (Type0 font,
  Identity-H, subset embedded, `ToUnicode` table). The standard fonts and `WinAnsiEncoding` are gone.
- `render/metrics.rs` takes the text widths from core's `fonts::text_width`. Its Adobe AFM tables,
  the WinAnsi table and the fallback to base letters in the PDF are removed; Arimo is as wide as
  Helvetica for the Latin letters, so the sheet layout does not move.
- `write_pdf` returns a `Result` (`ProjectError::Pdf`): the subsets of the bundled font could not be
  written. That is a bug, not a user input case.
- The OCAD export keeps the base-letter fallback (`POLISH_BASE`): its text is single-byte.

## Consequences

- Names with letters beyond WinAnsi show correctly in the PDF and can be copied from it.
- The PDF is larger by the font subsets (some tens of kilobytes). Example PDFs in the repository stay
  as they are until each example is rebuilt.
- A character Arimo lacks (other scripts) shows as the missing-glyph box, no longer as `?`. Core
  reports such characters in `FontObjects::missing`; freemapper does not pass the warning on yet.
- The core pin moved to v0.32.0 in its own change (ADR 0166).
