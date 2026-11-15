# 0141. Bavaria: the DOM20 read by range

Date: 2026-11-15
Status: Accepted

Design: [`docs/superpowers/specs/2026-11-15-bayern-dom20-range-design.md`](../superpowers/specs/2026-11-15-bayern-dom20-range-design.md).
It changes how [0123](0123-bavaria-the-cadastre-from-files.md) loads the surface model.

## Context

Bavaria has no surface model at 1 m. freemapper loaded the DOM20 (20 cm, from the aerial pictures) in tiles of
1 km, 32 to 56 MB each, and kept a copy reduced to 1 m. A map area touches two to four tiles, so a first run
loaded 100 to 200 MB for the nDSM template, much of it outside the area.

The file server answers ranges (`206`). A tile is a classic TIFF of 5000 × 5000 Float32, LZW with the floating
point predictor, in 400 blocks of 256 px (51.2 m), stored in row order; its IFD and all its arrays lie in the
last 4 KB of the file. A block holds 133 KB on average.

## Decision

- freemapper reads a DOM20 tile by range: the header, the IFD, its arrays, then only the blocks under the 1 m
  cells of the map area. Blocks that follow each other in the file are one range. A generic reader of tiled
  Float32 TIFFs (`freemapper_sources::tiff_range`) does this; it tells a failed request from a file it does not
  read (a BigTIFF, strips, another sample type or compression) and names what the file has.
- The cached copy stays the tile at 1 m and grows: each cell is the highest of its 25 pixels, and a cell on a
  block border takes the higher of what each read gives. `f32::max` passes over NaN, so the order of reads does
  not change a cell. The copy lists the blocks read in its ImageDescription, so one rename writes the cells and
  the list: a crash, or two projects reading the same tile at once, cannot leave a list that names blocks the
  copy lacks (the last writer wins, and what it lost is read again). A copy without the list holds the whole
  tile, as every copy cached before; a copy that cannot be read starts anew.
- A newer tile on the server starts a new copy with a new list; the old copy goes.
- Offline, a copy that lacks blocks of the area is "offline and not cached". A server that ignores ranges, or a
  tile laid out otherwise (another block size, another format), gets the whole tile downloaded as before, with a
  progress line that says why. A tile gone from the server keeps its copy; the cells it lacks stay empty, with
  the warning a missing tile part has.
- The download estimate counts the blocks still to read under the map box, at 45/400 MB each.

## Consequences

- The Schmausenbuck read 92 MB instead of about 212 MB (667 of 1600 blocks), and its surface model is the same
  cell for cell as from the whole tiles (1 616 528 cells). A second run online reads no block; offline it runs
  from the cache.
- A small map in one tile reads a few MB: 300 m need about 49 of 400 blocks.
- A cached copy holds NaN outside the blocks read. The mosaic takes only the cells of the area, which are read.

## Left out

- The same for other states with large tiled GeoTIFFs: the reader is generic, the backends are not changed.
- A cache of the 20 cm pixels: the copy keeps 1 m, as before.
