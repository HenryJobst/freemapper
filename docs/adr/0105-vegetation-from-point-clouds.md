# 0105. Vegetation from point clouds

Date: 2026-11-12
Status: Accepted (analysis, the Brandenburg source, fetch and derive); the export follows

## Context

The ALKIS says park, meadow or wood. It does not say where a thicket stops a runner, where one runs freely under
old trees, or where undergrowth slows. ISSprOM draws exactly that (405 to 410), and so does ISOM for forest maps,
which the user wants next. A surface model cannot tell it either: it shows the crowns, not the ground below them.
Classified laser point clouds can, because returns reach the ground through gaps in the crowns. The LGB nDOM that
freemapper reads for the nDSM template is a gridded surface (highest point per 0.2 m cell) without returns below
the crowns, so it does not help.

## Decision

- `derive/vegetation.rs` classifies 3 m cells from the returns of a classified point cloud, measured above the DTM:
  - open (401) when at least 90 % of the returns lie below 0.9 m (karttapullautin's open-land share);
  - otherwise a green index `((v − g) / (v + g) + 1) / 2`, with `g` the returns below 0.2 m or classified ground
    and `v` the returns in bands weighted by how much they slow a runner: 0.8 to 1.8 m × 1, 1.8 to 3 m × 0.5, above
    3 m × 0.1 (omapmaker's index and bands). From 0.4 slow running (406), from 0.6 walk (408), from 0.8 impassable
    (410);
  - below 0.4 runnable forest (405), or undergrowth when the returns between 0.2 and 0.8 m make up 35 % (407) or
    56 % (409) of those and the ground returns;
  - nothing where fewer than four returns fall.
- A cell that shares its class with at most one neighbour takes the class most of its 3 × 3 neighbourhood has, so
  single cells and thin lines go and a block of four stays. Areas are traced at the cell edges, simplified at
  1.5 m, and patches and holes smaller than 1.5 mm × 1.5 mm on paper are left out (36 m² at 1:4000, like
  karttapullautin's minimum).
- `read_tile` reads classified LAS/LAZ tiles chunk by chunk and leaves out buildings, noise and water. `CellCounts` counts
  every return in its cell as it is read, so a dense tile (Berlin has about 10 points per m²) never sits in memory.
- Point clouds are a new kind of source per state (open in Bavaria, Rhineland-Palatinate, North
  Rhine-Westphalia, Brandenburg, Saxony, Thuringia, Berlin, Saarland). A region adapter offers them through
  `PointCloudBackend` (`freemapper-adapter-api`), next to its rasters and agreed with stage D of the adapters:
  the tiles under a box go into the shared cache as checked LAS/LAZ files, and the backend gives their records
  and download estimate (part `PointCloud`, optional like the surface model). Brandenburg is the first: LGB
  serves a zip of 135 to 165 MB per km tile, of which only the LAZ file is kept. A tile that fails falls back to
  the cache or is left out with a warning, because the vegetation is only a candidate.
- The engine fetches the point clouds after the other optional parts and records the tiles in
  `freemapper.toml` (`point_clouds`; bookkeeping, so not part of the map fingerprint). Without a backend for the
  region nothing is asked for and the records go; a full disk or a failed fetch keeps the recorded tiles that are
  still cached. The download estimate and the disk check count the backend's items. `derive` counts the returns of
  the recorded tiles over the DTM on a 3 m grid over the map area plus buffer and writes the class areas to
  `data/vegetation.geojson` (property `code`, a MultiPolygon in the map's CRS); without records it removes the
  file. Only maps in UTM zone 33 read point clouds, because the backends serve EPSG:25833.
- The golden net does not record the point clouds (a tile has about 160 MB): its Brandenburg fetch gets no answer
  for the currency list and goes on with a warning, so G4 covers the vegetation step only once a store version
  holds a tile.
- The candidates in `export` follow; the plan is `docs/superpowers/plans/2026-11-12-park-vegetation.md`.
- No code is copied from omapmaker; karttapullautin is not a dependency (see ADR 0104).

## Consequences

- The analysis is tested on synthetic returns. Its thresholds come from forest maps and still have to be checked
  on real park data.
- A first real run (Sanssouci, 2026-11-12): four tiles of 137 to 164 MB (2021 scan), 16 minutes to download;
  `derive` stayed below 210 MB of memory and took 10 minutes in a debug build. Lawns, paths and the vineyard
  terraces came out open, the tree stands runnable forest, the hedges of the parterres 410. Building edges and
  the reeds of a pond also came out 410: the export cuts at the ALKIS buildings, the rest is for task 4.
- Parks get the green shades first; the forest map kind reuses the module with its own cell size and minimum
  areas.
