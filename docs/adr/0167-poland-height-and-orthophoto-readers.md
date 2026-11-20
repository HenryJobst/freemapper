# 0167. Poland: readers and warp for the national height model and orthophoto

Date: 2026-11-20

Status: Accepted, supplements ADR 0164

## Context

Stage 2 of Poland (ADR 0164) brings terrain and orthophoto. GUGiK delivers per sheet, in
PL-1992 (EPSG:2180): heights as ESRI ASCII grids (NMT 1 m, NMPT 0.5 m), orthophotos as tiled
TIFFs with JPEG tiles. A project lies in a UTM zone (32, 33 or 34). The index of the sheets is a
WFS of GUGiK, and it was down while this was built; what follows needs no service.

Probed on real files of the Słubice sheet `N-33-126-C-c-4-3` (NMT 2020, orthophoto 2025):

- The ASCII grid has `xllcenter`, `yllcenter`, `cellsize 1.00` and `nodata_value -9999`;
  2268 by 2445 cells, 89 % with a height.
- The orthophoto is 9074 by 9781 pixels of 0.25 m, 256 pixel tiles, YCbCr, EPSG:2180, the tie
  point at the north-west corner. The `tiff` crate stops on every tile ("Premature End of
  image"): the tag `JPEGTables` holds the table segments three times, each between its own start
  and end marker.

## Decision

- **Warp backwards.** `freemapper-sources::warp` projects every pixel centre of the project grid
  to PL-1992 and reads the source there (bilinear for heights and colours, nearest on request).
  This needs `Pl1992::forward` only; `Pl1992::inverse` stays out of freemapper-core.
- **Sheets are laid one after the other** into one result and fill only what is still empty: the
  first sheet wins, a pixel without a height falls to the next sheet. A sheet's nodata corner
  cannot leave a hole where the next sheet covers it. In a bilinear read a pixel without a height
  drops out and the others are weighed again, so a hole does not pull its neighbours down; the
  last half pixel of a sheet is kept by reading its edge pixel twice.
- **Heights** are read by `freemapper-sources::asc` (nodata becomes NaN, corner and centre
  headers both). **Colours** are read by `freemapper-sources::orto`: the tiles are taken out of
  the file, one set of table segments is put in front of each, and `image` decodes them. Only the
  tiles a window touches are decoded.
- **The 0.05 m orthophoto is never read:** a sheet with a pixel finer than 0.2 m is refused
  (`OrtoError::TooFine`); such a sheet is 1.4 GB.
- **Which sheet:** `freemapper-adapters-pl::sheets::best_per_cell` keeps one sheet per sheet
  number, ranked by source (LiDAR over photogrammetry over unknown), then pixel (finer first),
  then year (newer first), then sheet number and address, so the answer does not depend on the
  order of the index.

## Consequences

- The plan task "raster warp in freemapper-core" and "Pl1992::inverse" fall away: no core
  release is needed.
- `PolskaRasters` will be a per-area provider (`RasterKind::PerArea`): `fetch` downloads the
  sheets, warps them into the project grid and stores them in the project folder, so `derive`
  does not change.
- The index client, the download cap and the estimate follow when the index is reachable again.
- GUGiK's servers are unreliable (about every third connection fails after 10 s, the index
  database was down for hours). The client needs retries with a wait and a clear message.

## Addendum: the layer of the Polish adapter

`PolskaRasters` warps the GUGiK sheets itself, so `freemapper-adapters-pl` depends on
`freemapper-sources`, as `freemapper-adapters-de` does. The layer table of `scripts/check.sh
architecture` (ADR 0162) lists the edge. The index service is reached through a `SheetCatalog`, so the
adapter holds no network code of its own.
