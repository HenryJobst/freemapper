# 0105. Vegetation from point clouds

Date: 2026-11-12
Status: Accepted (analysis, the sources of Brandenburg, Berlin, North Rhine-Westphalia, Rhineland-Palatinate and Saarland, fetch, derive and export, checked on two parks)

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
- `read_tile` reads classified LAS/LAZ tiles chunk by chunk and leaves out buildings, noise and water; a tile whose
  header bounds miss the area is not read. `CellCounts` counts every return in its cell as it is read, so a dense
  tile (Berlin has about 10 points per m²) never sits in memory.
- LGB's classes are only ground (2), the early echoes of a pulse with several (0, nearly all crowns), the other
  last echoes above ground (20: roofs, walls, trunks, undergrowth) and a few low objects (1, mostly cars); there
  is no building and no vegetation class. So the class filter does not catch walls and eaves, and two rules do:
  - returns on an ALKIS building or within 1 m of it are left out (`BuildingMask`, all buildings, canopies too);
  - a cell where at least 60 % of the returns above 2.5 m are single echoes is a roof or a wall and gets no
    class. In Sanssouci roofs have 70 to 100 % single echoes, crowns up to 30 %. This also catches buildings
    that the scan still has and the cadastre no longer (the 2021 scan against the 2025 ALKIS).
- The export also cuts the areas at the paved areas (501.5) of the map: parked cars give returns at the
  heights of bushes.
- North Rhine-Westphalia is the second source (`nrw_3dm`): Geobasis NRW lists every LAZ tile
  (`3dm_32_<E>_<N>_1_nw.laz`, 1 km, EPSG:25832, 36 000 tiles, median 80 MB, nine in ten below 165 MB, the
  largest 553 MB) with its size and date in an `index.json` of about 3 MB. A download must have exactly the size
  the index gives and be a complete point cloud; tiles are cached as `32<E>-<N>_<date>.laz`. Its classes follow
  the same AdV scheme as LGB's, with more of them: 2 ground, 20 last echoes above ground, 1 intermediate echoes,
  and the synthetic fill points 26 (ground), 21 (buildings) and 9 (water), the cellar points 24 and the bridges
  17. `read_tile` leaves out every synthetic point (LAS flag), class 24 and bridges.
- Rhineland-Palatinate is the third source (`lvermgeo_rp_las`): LVermGeo RP serves every tile as
  `lpolpg_32_<E>_<N>_1_rp.laz` (1 km, EPSG:25832, 21 000 tiles, median 231 MB, up to 1.2 GB) and lists them in
  one Metalink of the state (about 6 MB) with size and SHA-256; a download must match both. The `name` of a
  Metalink entry differs from its file, so the tile comes from the URL. The day of the scan comes from the
  metadata of the tile (`<stem>_meta.xml`, about 70 KB, the date of type `creation`); it names the cache file,
  so a fetch with the tile cached asks only the metadata, and the Metalink only when a tile has to be
  downloaded. A tile without metadata (404) is not published. The LAS header names no CRS; the backend sets
  25832. A tile of Mainz has only the classes 2 and 20.
- Berlin is the fourth source (`berlin_als`): the Geoportal Berlin has no file per tile, only nine zip64
  archives of 1 to 51 GB (226 GB in all, one per part of the city) with one plain LAS file per km tile
  (`3dm_33_<E>_<N>_1_be.las`, EPSG:25833, scan of 2021, LAS 1.4, point format 6, no CRS in the header). The
  backend picks the archive by the tile (three columns by east, three rows by north), reads its central
  directory by two or three byte ranges (`zip_range::read_members`, now with the zip64 records) and streams
  the member in ranges of 32 MB to a file: about 240 MB packed and 550 MB unpacked for the tile of the
  Viktoriapark. It is unpacked (deflate, or deflate64 in two archives, through the crate `deflate64`),
  checked against its CRC and size, and kept as LAZ (`las_files::compress_las`). The day the archive last
  changed names the cache file, so a new scan replaces it and a cached tile costs one HEAD request; any other
  change to the archive makes its cached tiles load again too, because the server says no more. A tile the
  provider's list no longer names serves from the cache (all four sources), and an archive that fails is asked
  once per fetch. The
  classes differ from the AdV scheme: 2 ground, 3, 4 and 5 low, medium and high vegetation, in which the
  buildings lie too, and 7 low points; the building mask and the single-echo rule take out the roofs.
- Saarland is the fifth source (`lvgl_las`): the LVGL publishes "3D-Messdaten (2025)" (dl-de/by-2-0) on the
  open-data share that holds its height models, in six zip64 archives of 12 to 26 GB by district
  (`LIDAR_laz_<district>_EPSG-25832_Entstehung-2025.zip`), one LAZ file per km tile
  (`3dm_32_<E>_<N>_1_SL_2025_050.laz`, EPSG:25832, LAS 1.2, point format 1, no CRS in the header), deflated.
  The tiles are thinned to at most one point per 0.5 m cell (about 4 points per m², 26 to 46 MB a tile). As
  for the height models (`lvgl::DISTRICTS`), a tile is looked for in the archives in a fixed order, the
  directory of an archive is read only when the ones before it lack the tile, an archive that cannot be read
  is passed over, and a tile on a district border (294 of them) is the same in every archive that holds it.
  The member is streamed by ranges, unpacked, checked against its CRC and kept as it is. The survey is in the
  names of the archives and of the cache folder (`lvgl/las-2025`), so a cached tile serves without a request;
  a new survey comes with new names and a new release of the backend. The record names the folder of the
  archives. The classes: 2 ground, 6 buildings, 9 water, 20 other non-ground (mostly vegetation), 8 and 30
  synthetic fill points.
- Bavaria is the sixth source (`ldbv_las`): tiles by a computed address, the day of the flight from the GPS time
  of the points, and two classes of its own that are left out. See
  [0128](0128-bavaria-vegetation-from-the-point-clouds.md).
- A backend names the CRS of its tiles (`PointCloudInfo::epsg`); the engine uses it only for a map in that CRS
  and passes it the map area plus buffer in that CRS.
- Point clouds are a new kind of source per state (open in Bavaria, Rhineland-Palatinate, North
  Rhine-Westphalia, Brandenburg, Thuringia, Berlin, Saarland; Saxony took its point clouds off its download
  area in November 2026: the shares of its tile index answer 503 or are empty, so Saxony has no backend). A region adapter offers them through
  `PointCloudBackend` (`freemapper-adapter-api`), next to its rasters and agreed with stage D of the adapters:
  the tiles under a box go into the shared cache as checked LAS/LAZ files, and the backend gives their records
  and download estimate (part `PointCloud`, optional like the surface model). Brandenburg is the first: LGB
  serves a zip of 135 to 165 MB per km tile, of which only the LAZ file is kept. A tile that fails falls back to
  the cache or is left out with a warning, because the vegetation is only a candidate.
- The content group `vegetation` (ADR 0100) switches it on: it is off in every profile, `full` included, and
  only a choice of its own (`candidates` or `map`) makes the fetch load the point clouds. A tile has about
  160 MB, a park needs two to four; a profile must not cost that unasked, and the classes are not tuned for parks
  yet. The objects are known by the tag `freemapper:point-cloud`, not by their symbols, which ALKIS and
  OpenStreetMap use too.
- The engine fetches the point clouds after the other optional parts and records the tiles in
  `freemapper.toml` (`point_clouds`; bookkeeping, so not part of the map fingerprint). Without the choice or a
  backend for the region nothing is asked for and the records go; a full disk or a failed fetch keeps the
  recorded tiles that are still cached. The download estimate and the disk check count the backend's items for
  a project with the choice (the area page's estimate before saving does not know it). `derive` counts the
  returns of the recorded tiles over the DTM on a 3 m grid over the map area plus buffer and writes the class
  areas to `data/vegetation.geojson` (property `code`, a MultiPolygon in the map's CRS); without records or
  choice it removes the file. A map reads only point clouds in its own CRS (see below).
- The export cuts the areas at the ALKIS buildings and the water and, where the map already has an area of the
  same symbol (an ALKIS park 401, a wood 405), keeps only the part that differs; patches below 1.5 mm × 1.5 mm
  go. Tags: `freemapper:source` and `freemapper:point-cloud` (the product, `lgb-als`), `freemapper:currency`
  (the oldest day of the tiles; until 2026-11-14 the newest, changed because an area can lie on any of the tiles,
  so nothing newer can be said of it).
- The golden net does not record the point clouds (a tile has about 160 MB), and its projects do not choose
  the group, so it is unchanged; the vegetation step is covered by tests on a synthetic LAZ tile.
- After the review (2026-11-12): the export keeps the areas within the buffer around the map area, as the
  other candidates; the fetch says once when the group is chosen but the state has no point clouds (or none in
  the map's CRS), and `derive` when the group is chosen but nothing is fetched yet. A failing currency list or
  index falls back to the cached tiles; a stop asked for ends the fetch instead of counting as a failing tile.
  Withheld points are left out, a return count of 0 is no single echo, and a chunk is capped at 64 MB whatever
  record length a header claims. The large tiles already use the two-hour body timeout of `download_to`.
- After the rebuild of the examples (2026-11-13): a vegetation area stops at the areas that shall not be entered
  (520) drawn with it (`build/vegetation.rs`, in `place` right after the routing). ISSprOM shows no vegetation
  in a 520, and the white of the forest (405, "White over green") lies above the olive green in the colour order,
  so the forest candidates punched white holes into the private gardens and courtyards of the candidates PDF. On
  the map the areas are cut at the 520 of the map, in the candidates at those of the map and of the candidates;
  what is left below the minimum area goes. A 520 that the content settings switch off cuts nothing.
- No code is copied from omapmaker; karttapullautin is not a dependency (see ADR 0104).

## Consequences

- The analysis is tested on synthetic returns. Its thresholds come from forest maps and still have to be checked
  on real park data.
- A first real run (Sanssouci, 2026-11-12): four tiles of 137 to 164 MB (2021 scan), 16 minutes to download;
  `derive` stayed below 210 MB of memory and took 10 minutes in a debug build. Lawns, paths and the vineyard
  terraces came out open, the tree stands runnable forest, the hedges of the parterres 410. Building edges and
  the reeds of a pond also came out 410.
- Tuning on a 450 m square around the Friedenskirche (same day): the Friedenskirche and its cloister were
  almost wholly 410 before the building mask, and a cleared building plot (roofs in the 2021 scan, gone in the
  2025 cadastre) was a large 410 area before the single-echo rule. With both, the exported 410 areas went from
  13 to 1 (reeds at a pond edge), 405 from 159 to 112; the crowns stayed runnable forest. The other thresholds
  (open share, green index, undergrowth) look right on this park and stay.
- The Volksgarten in Cologne (NRW, scan 2023) with the same values: four tiles of 45 to 59 MB, seven minutes
  for the whole run in a debug build. The lawns came out open (and stay out of the export, where ALKIS has the
  park), the tree stands runnable forest, the shrubs around the pond 410 and those on the railway embankment 406
  and 408; buildings stayed free. No value was changed for it.
- The Stadtpark in Mainz (Rhineland-Palatinate, scan 2023-03-15) with the same values: four tiles of 81 to
  152 MB, every hash matching. The crowns came out runnable forest, the lawns and the railway land open,
  scattered shrubs 406 and 408, 13 small areas 410; houses stayed free. Returns on the Rhine come out open in
  `derive`, and the export cuts them at the water. No value was changed for it.
- The Viktoriapark in Berlin (scan of February and March 2021) with the same values: two tiles read out of
  the archive `Mitte` (about 240 MB each, 105 MB each as LAZ), 30 minutes and at most 1 GB of memory for the
  whole run in a debug build. The trees came out runnable forest, the lawns and the sports ground open, the
  blocks of houses free. The scan was made without leaves: the returns pass through shrubs and crowns, so
  there are few green shades (72 areas 406, 11 areas 408, none 410). In Berlin a thicket is easily missed.
- The old town of Saarlouis (Saarland, scan of 2025-02-21, without leaves, thinned to 0.5 m) with the same
  values: four tiles of 35 to 37 MB out of the archive `SLS`. The crowns of the wooded park in the north-west
  of the area, of the street trees and on the banks of the Saar came out runnable forest (403 areas in `derive`), squares and
  lawns open (283), the banks 406 and 410 (66 and 41 areas), houses free. The thinning did not hide the
  crowns. No value was changed for it.
- Parks get the green shades first; the forest map kind reuses the module with its own cell size and minimum
  areas.
