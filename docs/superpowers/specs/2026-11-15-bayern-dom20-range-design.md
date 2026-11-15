# Bavaria: the DOM20 by range (design)

Date: 2026-11-15. Third Bavarian follow-up of [ADR 0123](../../adr/0123-bavaria-the-cadastre-from-files.md), in
the order the user chose (vegetation, ATKIS, DOM20 by range). The user asked for work without interruptions;
the calls below are mine.

## Intent

Bavaria has no surface model at 1 m. freemapper loads the DOM20 (20 cm, from the aerial pictures) in tiles of
1 km, 32 to 56 MB each, and keeps only a copy reduced to 1 m (about 2.3 MB). A map area touches two to four
tiles, so a first run in Bavaria loads 100 to 200 MB for the nDSM template, most of it outside the area.

## What the server and the files allow (checked 2026-11-15)

- `download1.bayernwolke.de` answers `Accept-Ranges: bytes` and a range with `206`.
- A tile (`32655_5479_20_DOM.tif`, 53 298 114 bytes) is a classic little-endian TIFF: 5000 × 5000 Float32,
  LZW with the floating point predictor (3), in tiles of 256 × 256 px, 20 × 20 of them. The first IFD lies at
  the end of the file, with all its arrays (tile offsets and byte counts, pixel scale, tiepoint, the GDAL
  metadata with `BILDFLUG_DATUM`) in its last 4 KB.
- The tiles are stored in row order, one after the other: a row of blocks is one range. A block holds 40 to
  157 KB, 133 KB on average.
- The two Bavarian examples need 42 % (Schmausenbuck) and 50 % (Regensburg) of the blocks of their tiles. A
  school yard of 300 m in one tile needs about 49 of 400.

## Goals

- freemapper reads only the blocks of a DOM20 tile the map area needs, by range, and keeps the 1 m copy as now.
- A later area in the same tile reads only the blocks it lacks.
- The 1 m surface is the same, cell for cell, as when the whole tile is read.
- The download estimate counts the blocks still to read.

## Non-goals

- No change for the DGM1 (2.8 MB a tile) or for other states.
- No cache of the 20 cm values.

## Design

- **A reader of tiled TIFFs by range** in `freemapper-sources` (`tiff_range`): the header (16 bytes), the IFD
  (its entry count, then its entries), then one range over the arrays it needs. It accepts a classic TIFF with
  one Float32 sample, LZW and predictor 1, 2 or 3, in tiles; anything else is an error that names what it found.
  It reads given blocks, one range per run of blocks that lie one after the other, and decodes them with the
  LZW code of `float_tiff` (NoData to NaN as there).
- **The blocks of an area**: the 1 m cells of the tile under `GridSpec::covering(bbox, 1.0)`, as 20 cm pixels
  (five per cell), and the blocks those pixels lie in.
- **The 1 m copy grows**: each cell is the highest of its 25 values. A cell that straddles two blocks takes the
  highest of the part read so far; `f32::max` passes over NaN, so reading the other block later and taking the
  higher of the two gives the same value as reading both at once. The copy lists the blocks read in its
  ImageDescription (first a file `<name>.blocks` beside it; the review showed that two files can disagree after
  a crash or two runs at once). A copy without the list is a whole tile (every copy cached before this change).
- **When the server has a newer tile** (`ldbv_files::current` names the copy by the server's day), the new copy
  starts empty, as a whole tile did.
- **Offline**, a tile whose copy lacks blocks of the area is "offline and not cached", as a missing tile is.
- The flight date comes from the GDAL metadata of the IFD and is kept as before.
- **Estimate**: the blocks of the area not yet read, at 45/400 MB each (the old 45 MB per tile).
- A progress line per tile names how many blocks it reads.

## My calls

- Blocks as the unit, not groups of five aligned to whole metres: a straddling cell is merged by its highest
  value, which makes the finer unit safe.
- Runs of blocks merged into one range only when they lie one after the other in the file; no gap is read.
- The block list as a text file of numbers, not inside the GeoTIFF.
