# 0026. Lenient LZW fallback for GeoTIFF chunks

Date: 2026-11-08
Status: Accepted

## Context

The `tiff` crate 0.11.3 refuses some complete LZW chunks with "no lzw end code found". On the LVermGeo SH DGM1
tile 611/5970 (Lübeck) the upper right edge tile fails, although a plain `weezl` decoder reaches the end code and
libtiff reads it.

## Decision

`decode_float_tiff` keeps the `tiff` path. On that error with LZW it reads the chunk table itself and decodes each
strip or tile with `weezl` up to its end code, its size or the end of its data. Each chunk must give exactly its
size, and predictors 1, 2 and 3 are undone. `weezl` became a direct dependency. The fallback bounds the image
size, uses checked arithmetic and refuses byte counts past the end of the file. For strips the bound takes at
most the image height, so `RowsPerStrip = 2^32 - 1` still reads.

## Consequences

- freemapper reads these tiles like libtiff.
- The fix has its own failing-first test with a byte-for-byte cut of the real tile.
- Cost if wrong: a second TIFF decoder to keep safe against hostile headers.

Source: commits e1b7476, eecca82, 6072684; SH rulings (Task 4 post, Final).
