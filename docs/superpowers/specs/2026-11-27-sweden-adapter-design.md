# Sweden adapter, first stage: terrain and forest

Status: draft for approval, 2026-11-27. Research and measurements: `docs/regions/sweden-reference.md` (PR #201).
Branch `sweden-dtm-auth` already holds the three readers this design builds on: Deflate blocks in `tiff_range`,
Basic authentication in `HttpFetcher`, and `freemapper_sources::gpkg`.

## Intent

A person with a private Geotorget account can make a freemapper map of a place in Sweden. The first stage gives
what the user's own, already approved orders deliver (terrain model, laser data, land cover) plus OpenStreetMap:
contours, hillshade, slope, local relief, nDSM and vegetation, the land cover as base fill, paths and buildings
from OSM. Buildings from the register (Byggnad) and the orthophoto come in a later stage; their application is
pending and their format is unmeasured.

## Goals

- A new built-in region adapter `sweden` (code adapter, not a manifest) for the whole of Sweden.
- The whole fetch works with the readers and sources freemapper already has or got on this branch.
- Credentials from `FREEMAPPER_LM_USER` and `FREEMAPPER_LM_PASSWORD` only. A missing one is an error that names
  both variables and the three products to order in Geotorget.
- Everything stays testable offline with synthetic fixtures; a live check is a manual step with the owner's
  account.

## Non-goals and anti-goals

- No Byggnad, no orthophoto, no Ytmodell in this stage. Anything of them is added only after the owner's
  application is granted, and then it is a separate stage.
- No Lantmäteriet data in the repository, in tests, in examples or in the documentation. Fixtures are synthetic.
  The owner's application states this; breaking it is a failure even when the code works.
- No paid product. No credential ever reaches `freemapper.toml`, a cache file name, a progress line, an error
  text, `ATTRIBUTION.txt` or a log.
- No Swedish user interface or manual translation.

## Constraints

- A map of Sweden is drawn in EPSG:25833: SWEREF99 TM (EPSG:3006) has the same coordinates, so no
  transformation is needed for either the tiles or the map. The adapter states its CRS as 25833.
- `UrlPolicy`: the adapter asks `api.lantmateriet.se` (STAC, open) and `dl1.lantmateriet.se` (downloads, Basic) and
  OSM hosts, nothing else. Requests carry the fixed User-Agent only.
- Download limits are unpublished (a cap per period, HTTP 429, and a cap on concurrent STAC downloads per user).
  Fetch tile by tile, never more than a handful at a time, and treat 429 as the existing retry policy does.
- A Laserdata tile is a COPC file of about 80 MB (2.5 km); the terrain model tile is 10 km, 300 MB, read by range.

## What exists today that this relies on

- `RasterBackend` (terrain, `RasterKind::Tiles { grid_km: 10 }` fits the 10 km DTM), `PointCloudBackend`,
  `AlkisSource` (land use features) and `RegionAdapter` (`crates/freemapper-adapter-api`).
- `las` reads the real COPC tile (LAS 1.4, format 6) with the classes the vegetation filter already handles.
- `gpkg` reads the real Marktäcke file (168 areas, 384 lines, EPSG:3006).

## Approaches considered

1. **Terrain and forest first (chosen by the owner, 2026-11-27).** Adapter with DTM, point cloud, land cover, OSM;
   orthophoto and register buildings added when granted. Runnable with the data the owner has.
2. Complete adapter with orthophoto and Byggnad. Cannot be finished or tested until both applications are granted.
3. Only the building blocks, no adapter. Low risk, but no Swedish map at all.

## Design

1. **Credentials.** `freemapper` reads the two variables once at start (CLI and MCP) and builds the fetcher with
   `with_basic_auth("dl1.lantmateriet.se", …)`. The password is held in a type with no `Debug` and no `Display`.
   Reading them is the only place that touches the environment.
2. **STAC lookup** (`freemapper_sources::stac` if none exists, else the existing one): items of `stac-hojd` for a
   bounding box and a product, giving the tile id and its asset URL. The adapter never builds a URL from a tile name.
3. **Terrain.** A `RasterBackend` named `lantmateriet` with `Tiles { grid_km: 10 }`: finds the DTM tiles under the
   area, reads the blocks it needs with `tiff_range`, and stores what `derive` reads like the German tile backends.
   The surface model and the orthophoto are `None` in this stage.
4. **Point cloud.** A `PointCloudBackend` for Laserdata Skog. The tile grid is 2.5 km, but `PointCloudInfo::grid_km`
   is a whole number of km. Either the field becomes metres (touches every German adapter and the project file
   record) or the backend names its tiles itself. The plan decides after reading the callers; the lower-impact
   choice wins. Whole COPC files are downloaded, with the estimate listing them (80 MB each).
5. **Land cover.** A `Marktäcke` source (per municipality ZIP with a GeoPackage, `mark` and `markkantlinje`) behind
   `AlkisSource` as land use features. A table maps `objekttyp` to the existing `nutzart` classes (forest,
   open land, water, built-up, square); a class without a clear counterpart stays a candidate, not a map object. The
   plan lists the 8 classes seen in Kommun 0183 and the ones the product documentation adds.
6. **Buildings.** From OSM only in this stage; `no_buildings_warning` stays unset, because OSM buildings are the
   normal source here.
7. **A map without an orthophoto.** `derive` today stops with an error when the orthophoto input fails
   (`project.rs`, `inputs.orthophoto?`). This stage needs it to continue: no orthophoto template, a warning, and
   everything else as usual. This is a rule for every region, so it gets an ADR, and German regions keep
   failing as before because their backend always supplies an input.
8. **Boundary and registration.** `boundary.geojson` cut once from Lantmäteriet's municipality/county/country
   boundaries (a derived vector of an open product, simplified, in EPSG:25833), stored in the adapter crate like the
   Czech one. The region is registered built-in.
9. **Credit and notes.** Credit line “© Lantmäteriet (CC BY 4.0)” per product used. `permits.md` for a Swedish map
   says only what the sources support: protected areas of Naturvårdsverket are out of this stage, so the notes state
   that none were checked.
10. **Docs.** ADR for the GeoPackage reader, ADR for the map without orthophoto, ADR for the Sweden adapter; manual
    (DE/EN) and README table row; glossary terms; changelog DE/EN; `sweden-reference.md` gets the measured
    results of this stage.

## Left to the builder

- Naming, module layout and the exact `objekttyp` to `nutzart` table.
- Cache layout of the tiles and the retry numbers for 429.
- Whether the STAC reader is new or an existing one is extended.
- How the estimate words the 80 MB point cloud tiles.

## Open risks

- The download cap per period may stop a large map; the first live run shows it, and the error must say so.
- The point cloud tile size makes the first map slow and large (80 MB per 2.5 km square).
- `Tiles { grid_km }` and `PointCloudInfo::grid_km` are whole kilometres; a change to either touches the German
  adapters and needs the golden check.
