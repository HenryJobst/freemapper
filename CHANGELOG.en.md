# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

All notable changes to freemapper are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- Longer messages use plainer wording.

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
