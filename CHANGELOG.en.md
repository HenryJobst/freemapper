# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

All notable changes to freemapper are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Disk space: before each download freemapper checks the free space of the cache and the project folder, each drive on its own, and keeps 1 GB free on every drive. If not everything fits, it leaves out the surface model, the update of an older OSM extract and, in Berlin, the structures, trees and K5 and warns with the space needed and the space free. Without room for the map data, the terrain model or the orthophoto it stops before it downloads anything. When the disk fills up during a write, freemapper says so plainly, deletes the half-written file and does not retry. The download estimate (MCP tool `estimate_download`, area selection page) shows the free space and marks what does not fit.
- Berlin: more open data from Geoportal Berlin. ALKIS canopies and carports become map objects (522) on both map kinds. freemapper cuts them at buildings and leaves them out when they lie almost wholly inside a building. Candidates now include stairs (532.6 to 532.9) with the walking direction from the terrain model. A stair that does not rise there leads up to an adjoining building or monument, or runs across its short side. Stairs wider than three times 532.9 become parallel lines. The `Freitreppe` lines are step edges, and each flight of parallel edges becomes one stair across them. Further candidates are walls, retaining walls, fences, hedges, building passages and monuments, and on sprint maps also bridges and embankment tops. Street and park trees from the tree cadastre become 417 or 418 by crown size, and OSM trees within 2 m of them are dropped. So are OSM fences, walls and hedges with 80 % of their length within 1.5 m of the same ALKIS line. The Berlin map 1:5000 (K5) comes as the template `templates/k5.png`. An existing map gets the canopies only with `run --force`.

### Changed

- Allotment gardens (ALKIS land use "Sport-, Freizeit- und Erholungsfläche" with `bez` "Kleingarten") are out-of-bounds candidates (520) in every state. Parks, sports grounds and playgrounds stay open.

## [0.4.0] - 2026-11-08

### Added

- Lower Saxony (LGLN): orthophoto from the WMS, terrain and surface models (DGM1/DOM1) from the WCS in 2 km pieces, flight date and data currency from the STAC searches, buildings and land use from ALKIS. The nDSM is DOM1 − DGM1; if the DOM1 is more than two years older than the orthophoto, freemapper warns with both dates. A message says whether the trees were in leaf on the flight. Maps use UTM zone 32. Bremen is not included yet.
- Rhineland-Palatinate (LVermGeo RP): orthophoto from the WMS, with the block's date of creation as the flight date. Terrain and surface models (DGM1/DOM1) come from 1 km laser tiles whose names and dates the `Laserbefliegung` flight index gives; for tiles without a flight year freemapper reads the state's Metalink file. If the server lacks a tile, freemapper reports it as missing and asks again after 30 days. Buildings and land use come from ALKIS. Maps use UTM zone 32. At the Hesse border (Mainz, Wiesbaden) freemapper reads the ALKIS and OSM of both states.
- Water on all maps: rivers, canals, lakes, ponds, harbour basins and sea from ALKIS become uncrossable water (301.1) with a bank line (301.6), with no lines between cadastre areas or along the map edge. Water with a mean width under 2 m comes as candidates. OSM adds ponds, swimming pools, fountain basins, streams, ditches, fountains, wells, springs, drinking fountains on schoolyard maps and bridges on sprint maps as candidates. Contours stop at the shore, and fountain points in water (mostly jets in a basin) are left out. In OCAD the fill and the bank line have the numbers 301.001 and 301.002. An existing map gets the water only with `run --force`.

### Changed

- The north lines are in the map part "default part" with the other map objects, tagged `freemapper:source=north-lines`. In OOM they can be broken at small objects, and on the training map walls and fences lie on top of them. Title, scale and attribution stay in the part "peripheral information". An existing map changes only with `run --force`.
- Colours follow the IOF Printing and Colour Definitions Rev. 4 (September 2024, valid from 1 December 2024) instead of the older values in OOM's symbol set: brown is 25/75/100/0 instead of 0/56/100/18, green 80/0/100/0 instead of 76/0/91/0, brown 50% 10/35/50/0. Buildings are filled with 50 % black instead of 60 %, as in ISSprOM 2019-2. This applies to the OOM map, the OCAD file and the training PDF. Colour names and order stay the same.
- Longer messages use plainer wording.
- If a map area reaches beyond every supported state (abroad, out to sea or into a state freemapper does not support yet, such as Bremen or Thuringia), freemapper warns at fetch that the orthophoto and ALKIS can be missing there. This applies to all states.

### Fixed

- Where the water areas of two states did not quite meet at the border in mid-river (Mainz, Rhine), a bank line ran through the river. freemapper now closes gaps under 0.5 m between water areas before it draws the bank lines. This also applies to parcels of one state. Bridges and roads (0.8 m and wider) stay land, and islands keep their bank lines.

## [0.3.0] - 2026-11-08

### Added

- Hamburg (LGV): orthophoto from the WMS (2026 vintage), terrain and surface models from the state's archives, buildings and land use from ALKIS. Maps use UTM zone 32.
- Saxony-Anhalt (LVermGeo): orthophoto, terrain and surface models (DGM1/DOM1), the nDSM, buildings and land use. Maps use UTM zone 32.
- Hesse (HVBG): orthophoto, terrain and surface models from the municipal downloads, buildings and land use from ALKIS.
- Schleswig-Holstein (LVermGeo SH): orthophoto, terrain model from the INSPIRE ATOM feed (with an XYZ fallback), surface model, buildings and land use from ALKIS.
- Mecklenburg-Vorpommern (LAiV M-V): orthophoto from the WCS, terrain and surface models, buildings and land use from ALKIS.
- North Rhine-Westphalia (Geobasis NRW): orthophoto from the WMS, terrain and surface models (DGM1/DOM1) from laser tiles, buildings and land use from ALKIS, OSM from the extracts of the five Regierungsbezirke instead of the statewide one. Maps use UTM zone 32.
- Maps can now use UTM zone 32 or 33. A project takes the zone of its state, and freemapper reports and cleans up when an existing project changes zone.
- The repository contains one example project per supported state, each with the OpenOrienteering Mapper file, the OCAD file, the training PDF and a preview image.
- The release archives include the user manual with its images, at the path the READMEs link to.
- The README explains the user manual, the example projects and the area selection in the browser.
- This changelog, and architecture decision records (ADRs) in `docs/adr/`.

### Changed

- Every state has one row in the README regions table. Details moved to the user manual.
- Before a download, the size estimate counts whole tiles for states with tiled data and marks the ones already cached.
- Older projects drop their stored records when the raster provider changes, and save that change.
- freemapper pins its Rust toolchain in `rust-toolchain.toml`, so local builds and CI use the same version.

### Fixed

- Hamburg ALKIS falls back to the classic WFS when the OGC API fails.
- State boundaries are valid geometries now, so the check whether an area lies inside a state no longer trips over slivers or self-intersections.
- A server error report is recognised as such and no longer retried, including XML errors from WMS and WCS services.
- Downloads have a size limit, follow no redirects from the Schleswig-Holstein download portal, and leave no half-finished files after a cancelled run.
- Damaged or unusual LZW-compressed TIFF files decode the way libtiff decodes them.
- A failed surface model download keeps a stored model that covers the area. An unreadable flight date no longer aborts a run.
- Missing tiles and places that cannot be resolved show up as warnings with the product and the affected tile instead of ending the run.
- An orthophoto without a flight date is reused for 30 days only, then fetched again. If the flight-date query fails, freemapper keeps a matching stored orthophoto and its date.

## [0.2.0] - 2026-11-07

### Added

- Berlin (Geoportal Berlin): ALKIS buildings and land use in projects, with the matching attribution. Where no ready nDSM exists, freemapper computes it from the bDOM minus the DGM1.
- Saxony (GeoSN): orthophoto, DGM1/DOM1 tiles, the nDSM, buildings and land use from ALKIS, and a Saxony OSM extract. Each region names its own data provider in the map credits.
- Projects pick the data region from the area's centre. Brandenburg (with Berlin cut out), Berlin and Saxony are supported.
- The area selection page and the download estimate know the regions, so they show the right layers and sizes per region.
- A bilingual user manual (German and English) with pictures. It covers installation, the AI assistant, area selection, templates, field checks, the PDF, OCAD and the regions, and it explains the messages. Both READMEs link to it.
- The README says up front that freemapper writes an OCAD starter project and a training PDF.

### Changed

- The README lists regions in one equal table instead of separate Berlin and Saxony sections.
- OSM progress messages name the extract they download (Brandenburg or Saxony).

### Fixed

- Empty answers from the Saxony WFS, tiles at the border outside the raster region and refused requests are handled. A refused request is not retried.
- The Berlin credit line names Geoportal Berlin, and the ALKIS line appears only for Berlin objects.

## [0.1.0] - 2026-11-07

### Added

- First release. freemapper builds an OpenOrienteering Mapper starter project for orienteering from official geodata of Brandenburg and OpenStreetMap, as a command line tool (`init`, `fetch`, `derive`, `export`, `run`) and as an MCP server for AI assistants.
- Georeferenced map with the IOF symbol set for 1:1000 sprint maps, peripheral information (north lines, scale bar, info block) and a credit line that names freemapper and its repository.
- Orthophoto, hillshade and slope as templates behind the map, and contour lines and the nDSM derived from the terrain data.
- Buildings from ALKIS as map objects.
- OSM data as candidates: reliable OSM ways become map objects, the rest stays as candidates for you to review. Geofabrik extracts are cached, and freemapper falls back to the cached extract when Geofabrik is unreachable.
- Sprint map objects: paved areas from ALKIS traffic areas and unpaved paths from OSM.
- Out-of-bounds areas for private land, derived from ALKIS land use.
- OCAD export (`--ocad`, OCAD 12) with placeholder symbols.
- Training map as a one-page PDF (`export --pdf`, `run --pdf`, MCP option `pdf`).
- Area selection in the browser (`freemapper select`, MCP tool `open_area_selector`) with place search, rectangle, polygon, tile outlines and a download estimate.
- Place search through Nominatim (`init --place`) and initialisation from OSM (`init --osm`). The MCP server also lists layers and delivers PNG previews.
- Release archives for Linux (x86_64 and ARM64), macOS (Apple Silicon and Intel) and Windows, each with a SHA-256 checksum.

### Fixed

- Downloads have connect, response and body timeouts, keep a cached copy when the new download is invalid, and clean up old partial files.
- An existing map is kept when you run again. freemapper refreshes its templates and says so.
- freemapper protects edited maps from being overwritten on export.
- Error messages name the product and what to do next. Scale 0 and out-of-range coordinates are rejected.
- Download errors for uncovered tiles come in one report, and offline runs fall back to cached data.
