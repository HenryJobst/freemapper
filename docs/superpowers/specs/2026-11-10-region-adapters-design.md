# Region adapters design

Status: proposed · Date: 2026-11-10 · Terms follow [`docs/glossary.md`](../../glossary.md). Decision record:
[ADR 0082](../../adr/0082-region-adapters-trait-and-manifests.md). Plan:
[`2026-11-10-region-adapters.md`](../plans/2026-11-10-region-adapters.md).

This document is design only. It changes no production code. The brainstorming ran without a human partner, so
every open question was settled by taking the recommended option; section 9 lists each call with its reason and
what it costs if wrong. Counts below were taken from `main` at v0.9.0 (8004542) on 2026-11-10.

Revision 2026-11-11, after a review against `main` at v0.10.0 (c393575): the counts of section 2 still hold (440
enum references in 16 files); `freemapper-georef` is done (core v0.5.0, pinned at v0.11.0); the net is recorded from
v0.10.0 with an injectable clock (D25); the guard checks commit subjects on every push (D26); the trait keeps the
cancel of ADR 0090 (section 3.3); program language exists now (section 3.12); some tools and the weekly live check
are deferred and one new state tests the design before the guide (D27). The changed sections say so where they
change.

## 1. Intent

Other developers should add their own country or state to freemapper with as little work as possible. The
maintainer keeps the German states and at most the neighbouring countries. Today a state is wired in by hand: two
enums, `Region` and `Provider`, and `match` arms on them in 14 files. Adding North Rhine-Westphalia touched about
15 files. A contributor who knows WMS and WFS but no Rust cannot do that, and a contributor who knows Rust still
has to learn the whole fetch pipeline first.

Success looks like this:

- A state with standard OGC services (WMS, WCS, WFS or OGC API Features, a Geofabrik extract) is a folder with one
  `manifest.toml` and one boundary file. No Rust, no rebuild.
- A state with a special source (tile lists, Metalink, zip ranges, a pinned certificate) is a small Rust crate that
  supplies only the odd part. The manifest still carries everything standard.
- The eleven existing states run on the same mechanism with identical output.
- `freemapper adapter check` tells a contributor, before a pull request, whether the adapter works.

### Goals

1. A `RegionAdapter` trait and a registry replace every `match` on `Region` and `Provider`.
2. A declarative manifest format (version 1) for the standard OGC services.
3. Code adapters and code backends as workspace crates, enabled by Cargo features. No dynamic or WASM plugins.
4. `freemapper adapter check`, fixtures replay and a bilingual contributor guide.
5. Behaviour-preserving migration, proved by golden tests that run offline.

### Non-goals

- Dynamic loading of code (shared libraries, WASM). Cost and security reasons are in section 9, D3.
- A manifest language with conditions, loops or expressions. If a source needs logic, it is code.
- UTM zones other than 32 and 33, and national grids (Poland 2180, Switzerland LV95, UTM 31 for the Netherlands).
  `Crs` knows two zones today. A neighbour in another zone needs its own ADR.
- New states. Baden-Württemberg, Bremen, Bayern and Saarland come after the migration, as the first adapters of the
  new scheme (stage G of the plan). Their research is not part of this work.
- A change of the map output, the rule engine or the candidate rules.

### Anti-goals

Failures even if the code compiles:

- Any example project changes by one byte when rebuilt offline.
- A manifest from the internet can make freemapper contact a host the manifest did not declare, an address inside
  the user's network, or write a file outside the project (ADR 0059, ADR 0060).
- A contributor needs to edit `project.rs` (13 840 lines) to add a state.
- The adapter API becomes a second place for logic that belongs in the readers, so every state grows its own copy.

## 2. What the code does today

### 2.1 Where `Region` and `Provider` are matched

`Region` has 11 variants, `Provider` has 10 (Brandenburg and Berlin share LGB). Counted before the first
`#[cfg(test)]` of each file; the second number includes the tests.

| File | Production refs (with tests) | What it decides |
|---|---:|---|
| `sources/region.rs` | 16 (168) | `Region::ALL`, `of_object_id`, `provider()`, `crs()`, the 11 boundary constants and `OnceLock`s, `Provider::terrain_source`, `Provider::per_area`, `provider_tiles`, `tile_touches`, `tile_centre_in`, `raster_region`, `regions_for`, `reaches_beyond_coverage` |
| `project.rs` | 58 (139) | `map_crs`, the recorded `provider`, the fetch dispatch (10 arms to `fetch_*_rasters`), the `derive` dispatch (tile products, orthophoto source), 9 `warn_beyond_state` calls, the provider label in `raster_tiles`, 10 attribution functions and `per_area_attribution_year` |
| `sources/lgb.rs` | 32 (32) | `Product::provider`, `grid_km`, file names and `provider_dir_name` per provider |
| `layers.rs` | 27 (33) | `area_tiles`, the estimate dispatch to 10 `*_items` functions, `tile_touches` per state, the Berlin items |
| `sources/alkis.rs` | 22 (28) | buildings and land use: two 11-arm `match`es on `Region` |
| `fetch.rs` | 11 (12) | currency table per provider, the GeoSN offline special case |
| `sources/osm.rs` | 10 (11) | `extracts_for`, `ExtractSource` (URL, state URL, prefix, size, label per variant), the NRW districts |
| `sources/alkis_nas.rs`, `trees.rs`, `berlin.rs` | 1 each | a `region` field in the NAS service and tree dataset tables, the Berlin test |
| `hvbg`, `lgv`, `laiv_mv`, `lvermgeo_rp`, `lvermgeo_sh`, `geobasis_nrw` | 1 to 2 each | `tile_centre_in`, `tile_touches`, `contains_point` for the own state |
| Total | about 190 (about 440) | |

Places that know the states without matching the enums:

- `crates/freemapper-cli/web/app.js` lists the supported states in German and English (lines 46 and 121) and holds
  one hard-coded aerial WMS layer with attribution per state for the selection map.
- `project.rs:134` (the `OutsideCoverage` text) and `freemapper-cli/src/mcp.rs:31` (the tool description) list the
  states again.
- `README.md`, `README.en.md`, the manual, the glossary and `examples/README*.md` list them in prose and tables.
- `assets/regions/*.geojson` (11 boundaries, 25 m simplified, EPSG:25833) and `assets/regions/osm/` (5 NRW
  districts) are `include_str!`ed from `region.rs` and `osm.rs`.
- The CLI (`main.rs`, `select.rs`, `mcp.rs`) never matches on the enums. It calls `project` and `layers`. That
  stays true after the refactor.
- The candidate rules in `assets/rules/*.toml` are not per state. `state-candidates.toml` and `adv-codes.toml` match
  the normalised ALKIS schema, so a state joins them by producing that schema (section 3.9), not by naming itself.
  Only `berlin-candidates.toml` is per region.

### 2.2 What each state supplies

All services were read from the code, not from the documentation. "Per area" means a raster mosaic cut to the map
area and stored in `data/`; "tiles" means 1 km or 2 km tiles in the shared cache with a record per tile.

| State (id) | CRS | Orthophoto | Terrain and surface | Buildings and land use | Extra layers |
|---|---|---|---|---|---|
| Brandenburg (`brandenburg`) | 25833 | DOP20 JPEG tiles with world files, 1 km, currency CSV | DGM1 tiles; bDOM tiles (image DSM) | OGC API Features collections (LGB JSON) | NAS WFS (full ALKIS, `LGB_NAS`), Potsdam tree cadastre |
| Berlin (`berlin`) | 25833 | the LGB tiles | the LGB tiles | `berlin.rs` (Geoportal Berlin WFS) | 10 Berlin layers, K5 WMS template, tree registers, street survey, building heights 2022 |
| Saxony (`saxony`) | 25833 | WMS `dop-rgb`, flight date by WMS | DGM1 and DOM1 2 km tiles via public share links, ArcGIS tile index for currency | ALKIS vereinfacht WFS (`GEOSN`) | none |
| Saxony-Anhalt (`saxonyanhalt`) | 25832 | WMS `lsa_lvermgeo_dop20_2` | WCS `Coverage1`, multipart answer, envelope clip | WFS (`LVERMGEO`) | none |
| Hamburg (`hamburg`) | 25832 | WMS time series, pinned `TIME=2026` | DGM1 and bDOM tiles read by zip range from statewide archives | OGC API Features, WFS fallback (`LGV`), ASCII spellings normalised | none |
| Hesse (`hessen`) | 25832 | WMS `he_dop20_rgb`, date by GetFeatureInfo | DGM1 and DOM1 1 km tiles, byte ranges out of municipal zips found through the Downloadcenter metadata xlsx | WFS (`HVBG`) | none |
| Schleswig-Holstein (`schleswigholstein`) | 25832 | WMS, date lookup in code | DGM1 and bDOM tiles from the Downloadportal AJAX `overview.php`, `massen.php` links, ATOM feed, bDOM reduced to 1 m | WFS (`LVERMGEO_SH`) | none |
| Mecklenburg-Vorpommern (`mecklenburgvorpommern`) | 25833 | WCS (the WMS stamps a watermark) | WCS, masked to the tile set of `overview.php`; the models mark no NoData | OGC API Features, WFS fallback (`LAIV_MV`, accepts 25833 only) | none |
| North Rhine-Westphalia (`nordrheinwestfalen`) | 25832 | WMS `nw_dop_rgb`, date by GetFeatureInfo | DGM1 and DOM1 1 km GeoTIFF tiles; names and dates from `*_meta.zip` (CSV) | WFS (`GEOBASIS_NRW`, errors as bare HTTP 400) | none |
| Lower Saxony (`niedersachsen`) | 25832 | WMS `ni_dop20`, date by STAC | WCS (plain Float32 TIFF, 2 km pieces); nDSM from DOM1 with an age warning | WFS (`LGLN`, silent cut at 10 000 features without `COUNT`) | none |
| Rhineland-Palatinate (`rheinlandpfalz`) | 25832 | WMS dated by creation date | DGM1 and DOM1 1 km tiles; tile list is the `Laserbefliegung` WFS; bulk by Metalink (11 MB) | WFS (`LVERMGEO_RP`, `ave:Nutzung`, `gml:timePosition`) | none |

The persisted ids are the lower-case variant names (`saxonyanhalt`). They appear in `freemapper.toml`
(`provider = "lvermgeo"`, `regions = ["saxonyanhalt"]`) and in cache folder names (`lvermgeosh`).

| State | OSM extract (Geofabrik, MB) | Licence and credit | Quirks the adapter must keep |
|---|---|---|---|
| Brandenburg | `brandenburg` 300, shared with Berlin | dl-de/by-2-0 | currency CSV; ALKIS is paged OGC API |
| Berlin | the Brandenburg extract | dl-de/zero-2-0 per layer, many credit lines | `berlin:layer` tag, K5 date constant, building dates from the 2022 heights layer; boundary from ALKIS |
| Saxony | `sachsen` 270 | dl-de/by-2-0, "GeoSN" | squares without type get a marker `bez`; no canopies or passages in ALKIS |
| Saxony-Anhalt | `sachsen-anhalt` 177 | dl-de/by-2-0, "© GeoBasis-DE / LVermGeo ST" | WCS answers 404 beyond its envelope; no canopies in ALKIS |
| Hamburg | `hamburg` 54 | dl-de/by-2-0, LGV credit with year | LGB-spelling normaliser; every square that touches Hamburg has a clipped tile |
| Hesse | `hessen` 348 | dl-de/zero-2-0 | negative eastings, so no 25833 tile grid; `MAX_PAGES` 20 per district; WMS limit 3000 px |
| Schleswig-Holstein | `schleswig-holstein` 159 | CC BY 4.0, "(Quelle verändert)" | TLS pin for `geodaten.schleswig-holstein.de`: D-TRUST intermediate and root, no redirects; the advertised OpenGBD WFS refuses normal queries |
| Mecklenburg-Vorpommern | `mecklenburg-vorpommern` 128.8 | CC BY 4.0, "(Quelle verändert)" | cells outside the tile set become NaN, zeros inside stay; zone 33 per-area provider |
| North Rhine-Westphalia | 5 Regierungsbezirk extracts (137 to 225 MB), each with its own boundary | dl-de/zero-2-0, no change note | tile name carries the scan year; stale duplicates in the directory listing; district border reads two extracts |
| Lower Saxony | `niedersachsen` 510, Bremen inside | CC BY 4.0, "Daten geändert" | Bremen is a hole in the boundary; MapServer answers only when the whole image is ready, so 2 km pieces |
| Rhineland-Palatinate | `rheinland-pfalz` 275, without Wiesbaden and Mainz-Kastel | dl-de/by-2-0, credit without blank before the year | the GeoPortal proxy answers errors with HTML and status 200, so only origin hosts; missing tiles checked again after 30 days |

Lines of production code per state module (up to `mod tests`, so test support is included in a few):

| Module | Lines | Module | Lines |
|---|---:|---|---:|
| `hvbg.rs` (HE) | 1 185 | `lgv.rs` (HH) | 331 |
| `lvermgeo_sh.rs` (SH) | 964 | `lgb.rs` (BB, BE products; shared with SN) | 286 |
| `lvermgeo_rp.rs` (RP) | 789 | `geosn.rs` (SN) | 157 |
| `berlin.rs` (BE) | 643 | `lvermgeo.rs` (ST) | 151 |
| `laiv_mv.rs` (MV) | 529 | `lgln.rs` (NI) | 350 |
| `geobasis_nrw.rs` (NW) | 462 | Sum | about 5 850 |

On top of that, each state has about 60 to 110 lines of `fetch_*_rasters` in `project.rs`, one attribution function
of 5 to 20 lines, one `*_items` estimate function of 25 to 50 lines, and a handful of arms elsewhere.

### 2.3 Generic readers and state-specific code

| Generic (reused by two or more states) | Lines | State-specific |
|---|---:|---|
| `alkis_wfs.rs`: GML 3.2 reader with paging, `WfsService` table of 9 rows | 478 | `hvbg`, `lvermgeo_sh`, `lvermgeo_rp`, `laiv_mv`, `geobasis_nrw`: tile lists and their quirks |
| `alkis.rs`: OGC API Features reader, merge by oid, WFS fallback | 587 | `lgv.rs`: spelling normaliser, archive layout |
| `alkis_nas.rs`: NAS WFS reader, quadrant splitting, `NasService` table (1 row) | 906 | `berlin.rs`: 10 layers, registers, K5 |
| `trees.rs`: city tree reader, `TreeDataset` table (1 row) | 366 | `lgb.rs`/`geosn.rs`: product enum, currency CSV, ArcGIS index |
| `wms.rs`, `wcs.rs`: GetMap and GetCoverage pieces, stitching | 359, 198 | per-state flight date lookups (7 variants: WFS GeoJSON, GetFeatureInfo, STAC, XML metadata zip, tile index, creation date, none) |
| `zip_range.rs`, `xlsx.rs`, `float_tiff.rs`, `height_model.rs` | 236, 305, 370, 236 | |
| `osm.rs`: extract download, state check, PBF read | 981 | `ExtractSource` and `GovernmentDistrict` tables |

The readers are generic already. What is state-specific is mostly data (URLs, layer names, CRS, limits) plus the
dispatch around it. That is why a manifest can carry a large share, and why the dispatch is the real work.

## 3. Design

### 3.1 Vocabulary

Glossary rows are added in stage A of the plan (glossary first, ADR 0007).

| German | English | Meaning |
|---|---|---|
| Regionsadapter | region adapter | Everything freemapper needs to map one region: boundary, CRS, sources, credit. A manifest, a crate, or both. Replaces the `Region` variant and the `match` arms. |
| Adaptermanifest | adapter manifest | The TOML file (format version 1) that declares an adapter's standard services. |
| Rasterbackend | raster backend | Rust code that fetches an adapter's orthophoto, terrain or surface model when the services are not standard. Named from the manifest. |
| Adapterprüfung | adapter check | `freemapper adapter check`: static checks, fixtures replay and an optional live run. |

The existing glossary row "Quelladapter / source adapter" (`SourceAdapter`) names a trait from the first design that
was never built. Stage A renames it to point at the region adapter.

A region and its provider stay two ids because Brandenburg and Berlin share LGB. In the new model an adapter has an
`id` (the region) and a `provider` (the raster source, default the same as the id). Two adapters with the same
provider share the cache folder and the record in the project file.

### 3.2 Crates

```
freemapper-core (separate repo)
  freemapper-georef     done in v0.5.0: Crs, LatLon, ProjectedPoint, BoundingBox, TileId  (from crs.rs, tiles.rs)
  freemapper-fs         link-safe writes (already used)
  freemapper-text       Fluent catalogs (program language, in use since v0.10.0)

freemapper (this repo, workspace)
  crates/freemapper-adapter-api   NEW  traits and data types, Fetcher/FetchError/FetchMode, records, AdapterError
  crates/freemapper-sources       NEW  the generic readers: wms, wcs, wfs/gml, ogc api, nas, trees, zip_range, xlsx,
                                       float_tiff, height_model, metalink, tile-list helpers
  crates/freemapper-manifest      NEW  manifest schema, parser, validator, UrlPolicy, ManifestAdapter, loader
  crates/adapters/de              NEW  the 11 states: manifests (embedded) plus code backends, modules per state
  crates/adapters/<cc>-<name>     NEW  one crate per country or state of other maintainers, behind a Cargo feature
  crates/freemapper               the pipeline (project, fetch, derive, build, render); depends on the api and the registry
  crates/freemapper-cli           adds `adapter` subcommands, assembles the registry
```

Why these cuts:

- The api crate holds no HTTP client and no TOML. A code adapter depends on it and on `freemapper-sources`, nothing
  else. It can be tested without the pipeline.
- `freemapper-sources` must not depend on `freemapper`. Today the readers use `crate::fetch::Fetcher`,
  `crate::crs`, `crate::raster`. The first three stage-B tasks move those down.
- The readers fetch, so core ADR 0007 (no network in core) keeps them out of core. Their pure parts (GML parsing,
  Metalink parsing, the zip directory parser, tile-name arithmetic) can move to core later if a second app needs
  them. Not now.
- `HttpFetcher`, `Cache` and the project types stay in `freemapper`. Adapters see them through `AdapterEnv`
  (section 3.3).
- `freemapper-georef` is the crate that ADR 0067 and core ADR 0005 planned; freemapper owns it. The move is done:
  `crs.rs` and `tiles.rs` re-export it, and the api crate depends on it directly (D4).

Adding an adapter crate is three edits: the crate under `crates/adapters/`, a feature line in
`freemapper-cli/Cargo.toml`, and one line in `freemapper-cli/src/adapters.rs`:

```rust
#[cfg(feature = "adapter-cz")] registry.add_crate(freemapper_adapter_cz::register);
```

No `inventory` or `linkme` style auto-registration: link-time magic is a portability risk on three operating
systems, and an explicit list is greppable.

### 3.3 The trait

The signatures below are the design target. Task C1 of the plan fixes the first cut; each later task may widen a
signature when a move-only refactor demands it, and records the change in the plan.

```rust
pub trait RegionAdapter: Send + Sync {
    /// Identity and static facts: id, provider id, names, CRS, object-id prefix, OSM extracts, credit, preview WMS.
    fn info(&self) -> &RegionInfo;
    /// The boundary in EPSG:25833, parsed once, valid (ADR 0025).
    fn boundary(&self) -> &MultiPolygon<f64>;
    /// Raster sources. `None`: the region has ALKIS but no rasters and is never the raster region.
    fn rasters(&self) -> Option<&dyn RasterBackend>;
    /// Buildings and land use as GeoJSON features in the LGB schema (section 3.9).
    fn alkis(&self) -> Option<&dyn AlkisSource>;
    /// Extra layers (Berlin layers, NAS, tree cadastres); empty for most.
    fn extras(&self) -> &[Box<dyn ExtraLayer>];
    /// Credit lines for the files in `project`; empty before anything of this adapter was fetched.
    fn attribution(&self, used: &UsedSources) -> Vec<CreditLine>;
    /// Download items for the estimate (what, MB, cached, part, store).
    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv) -> Vec<EstimateItem>;
}

pub trait RasterBackend: Send + Sync {
    fn provider(&self) -> &ProviderInfo;        // id, terrain_source label, kind: Tiles { grid_km } | PerArea
    /// Move of `fetch_*_rasters`: fetches into the cache or `data/` and updates the raster records.
    fn fetch(&self, env: &mut AdapterEnv<'_>, area: &RasterArea, records: &mut RasterRecords)
        -> Result<(), AdapterError>;
    /// Move of the match in `derive_project`: what `derive` reads.
    fn derive_inputs(&self, env: &AdapterEnv<'_>, records: &RasterRecords, area: &RasterArea)
        -> Result<DeriveInputs, AdapterError>;
}

pub trait AlkisSource: Send + Sync {
    fn buildings(&self, env: &mut AdapterEnv<'_>, area: &FetchArea) -> Result<AlkisPart, AdapterError>;
    fn land_use(&self, env: &mut AdapterEnv<'_>, area: &FetchArea) -> Result<AlkisPart, AdapterError>;
}
```

`AdapterEnv` is what an adapter may touch: `fetcher: &dyn Fetcher`, `cache: CacheDir` (a path with the link-safe
helpers of `freemapper-fs`, no knowledge of disk-space accounting), `mode: FetchMode`, `progress`, `today` (the date
from the injectable clock, so an adapter never reads the system clock). The fetcher may be the `CancelCheck` wrapper
of ADR 0090; an adapter passes `FetchError::Cancelled` up inside `AdapterError` unchanged, and the pipeline maps it
back to a cancel in one place, so the cancel button of the browser page keeps working. It cannot reach
the project folder directly. Writes into the project go through `ProjectSink`, which wraps `write_inside` and
`copy_inside`. So an adapter cannot break ADR 0059 or ADR 0060 by accident.

`RasterRecords` is today's set of project fields (`tiles`, `ndsm_tiles`, `orthophoto`, `terrain`, `surface`) moved
into the api crate unchanged. `DeriveInputs` is `{ orthophoto: Tiles(Vec<FetchedTile>) | Image(PathBuf), dtm:
Tiles(..) | Stored(PathBuf), dsm: Option<..> }`. The first migration keeps the two raster kinds (tile providers and
per-area providers) as they are. Unifying them would change which files `derive` reads and risks byte changes, so
it is a possible later cleanup (D9).

Rules the trait enforces:

- An adapter returns data and records. It never prints to stdout and never reads the environment.
- Free functions of `region.rs` (`regions_for`, `raster_region`, `provider_tiles`, `reaches_beyond_coverage`) stay,
  but take `&Registry`. Their results are covered by golden test G2 before and after.
- The registry order is the order the functions iterate today (`Region::ALL`), so `raster_region` and
  `regions_for` break ties as before. Built-in adapters keep their current order; others follow, sorted by id.

### 3.4 Manifest schema, format 1

A manifest lives in a folder `<dir>/<id>/manifest.toml` with the files it names (`boundary.geojson`, optional
`rules.toml`, `fixtures/`). File names in the manifest are plain names in that folder: no separators, no `..`,
checked as in ADR 0059. A manifest cannot name an output path; freemapper decides where everything lands.

| Table | Keys | Notes |
|---|---|---|
| top | `format = 1` | integer, required. Loader accepts N and N-1 (section 3.8). |
| `[adapter]` | `id`, `revision`, `min_freemapper`, `names.{en,de}`, `crs`, `provider`, `object_id_prefix`, `maintainers`, `last_verified`, `licence_of_manifest` | `id` matches `[a-z][a-z0-9]{1,31}`. `crs` is 25832 or 25833. `revision` counts data changes of this adapter. |
| `[net]` | `hosts = [..]` | Every URL in the file must use one of these hosts. DNS names only. |
| `[boundary]` | `file`, `crs` (4326 or 25833), `simplify_m` (default 25) | `adapter boundary` builds the file. Must be a valid geometry. |
| `[[osm.extract]]` | `geofabrik = "europe/germany/hessen"`, `megabytes`, `label`, `boundary` (only when there are several) | The URLs are derived on `download.geofabrik.de`, so the host is fixed. |
| `[attribution]` | `licence`, `text` with `{year}`, `year = "newest-fetch"`, `short`, `note` | `licence` is from a closed list (D12). `text` is the legal credit line. |
| `[alkis]` | `kind = "wfs" or "ogc-api-features"`, `crs`, url keys, `normalise`, `square_bez`, `page_count`, `fallback = {..}` | `normalise` names a preset of `freemapper-sources` (section 3.9). |
| `[orthophoto]` | `kind = "wms" or "wcs" or "backend"`, url, layer or coverage, `crs`, `params = {..}`, `max_px`, `date = {..}` | `params` is a table, never a raw query string. |
| `[terrain]`, `[surface]` | `kind = "wcs" or "backend"`, url, coverage, `multipart`, `envelope`, `max_m`, `date = {..}` | `surface` is optional. |
| `[date]` kinds | `wfs-geojson` (layer, field, format), `stac-newest` (collection URL), `wms-featureinfo` (layer, field), `none` | A source with no date lookup says `none` and the log says the date is unknown. |
| `[[extra.nas]]`, `[[extra.trees]]` | the fields of `NasService` and `TreeDataset` | `layers`, `max_count`, `max_bytes`, `namespaces`; tree `format`, `fields`, `envelope`. |
| `[preview]` | `wms`, `layer`, `attribution` | The aerial layer of the selection page (replaces the hard-coded list in `app.js`). |
| `[estimate]` | `orthophoto_mb_per_km2`, `terrain_mb_per_km2`, `surface_mb_per_km2` | Defaults are the constants of `layers.rs`. |
| `backend = "<name>"` | in `[orthophoto]`, `[terrain]`, `[surface]` | Names a `RasterBackend` registered by a code adapter. Unknown name: error naming the crate and feature. |

Closed lists keep the manifest from turning into a language: five `kind`s, four `date` kinds, the `normalise`
presets, the licence ids. A new kind is a freemapper release, not a manifest trick. No conditions, no includes, no
variables. A placeholder is one of `{year}` in the credit and `{e}`, `{n}`, `{zone}` in a tile pattern (later); the
expansion percent-encodes its value.

### 3.5 Worked manifests

Both are written from the current Rust constants. They are design sketches: field names are final for review, the
exact set is fixed by task D1.

**Hamburg** (WMS, OGC API with WFS fallback, a code backend for the heights). Taken from `lgv.rs`, `alkis_wfs.rs`
(`LGV`), `osm.rs` and `project.rs::lgv_attribution`.

```toml
format = 1

[adapter]
id = "hamburg"
provider = "lgv"
revision = 1
min_freemapper = "0.10.0"
crs = 25832
object_id_prefix = "DEHH"
names = { en = "Hamburg", de = "Hamburg" }
maintainers = ["github:HenryJobst"]
last_verified = "2026-11-10"

[net]
hosts = ["geodienste.hamburg.de", "api.hamburg.de", "www.daten-hamburg.de", "download.geofabrik.de"]

[boundary]
file = "boundary.geojson"
crs = 25833

[[osm.extract]]
geofabrik = "europe/germany/hamburg"
megabytes = 54
label = "Hamburg"

[attribution]
licence = "dl-de/by-2-0"
text = "Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) {year}, dl-de/by-2-0, Daten geändert"
year = "newest-fetch"
short = "LGV Hamburg"

[alkis]
kind = "ogc-api-features"
crs = 25832
buildings = "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/GebaeudeBauwerk/items"
land_use  = "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/NutzungFlurstueck/items"
normalise = "hamburg-ascii"

[alkis.fallback]
kind = "wfs"
url = "https://geodienste.hamburg.de/WFS_HH_ALKIS_vereinfacht"
buildings_type = "ave:GebaeudeBauwerk"
land_use_type = "ave:NutzungFlurstueck"
square_bez = "none"

[orthophoto]
kind = "wms"
url = "https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt"
layer = "dop_zeitreihe_unbelaubt"
crs = 25832
params = { TIME = "2026" }
max_px = 4000
date = { backend = "lgv" }      # the flight date comes from a metadata zip: code

[terrain]
backend = "lgv"                 # DGM1 from statewide archives by zip range: code
[surface]
backend = "lgv"
```

What the manifest holds and the code does not: every URL, layer, limit, the credit, the OSM extract, the fallback
order. What stays code in `crates/adapters/de/src/lgv.rs`: the flight date from the metadata zip, the DGM1 and bDOM
tile reading, the ASCII spelling preset `hamburg-ascii` (which lives in `freemapper-sources` because the build
rules rely on `lgv::spelling_key`). Estimated: the manifest is about 55 lines, the backend about 250.

**North Rhine-Westphalia** (a tile-list state). Taken from `geobasis_nrw.rs`, `alkis_wfs.rs` (`GEOBASIS_NRW`), the
`osm.rs` district table and `nrw_attribution`.

```toml
format = 1

[adapter]
id = "nordrheinwestfalen"
provider = "geobasisnrw"
revision = 1
min_freemapper = "0.10.0"
crs = 25832
object_id_prefix = "DENW"
names = { en = "North Rhine-Westphalia", de = "Nordrhein-Westfalen" }

[net]
hosts = ["www.wfs.nrw.de", "www.wms.nrw.de", "www.opengeodata.nrw.de", "download.geofabrik.de"]

[boundary]
file = "boundary.geojson"
crs = 25833

# Five extracts, each with its own boundary; an area on a district border reads two.
[[osm.extract]]
geofabrik = "europe/germany/nordrhein-westfalen/arnsberg-regbez"
megabytes = 195.2
label = "North Rhine-Westphalia, Arnsberg"
boundary = "osm-arnsberg.geojson"
# ... detmold-regbez, duesseldorf-regbez, koeln-regbez, muenster-regbez

[attribution]
licence = "dl-de/zero-2-0"
text = "Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) {year}, dl-de/zero-2-0"
year = "newest-fetch"
short = "Geobasis NRW"

[alkis]
kind = "wfs"
crs = 25832
url = "https://www.wfs.nrw.de/geobasis/wfs_nw_alkis_vereinfacht"
buildings_type = "ave:GebaeudeBauwerk"
land_use_type = "ave:NutzungFlurstueck"
square_bez = "ohne Unterart (Nordrhein-Westfalen)"

[orthophoto]
kind = "wms"
url = "https://www.wms.nrw.de/geobasis/wms_nw_dop"
layer = "nw_dop_rgb"
crs = 25832
max_px = 4000
date = { backend = "geobasis-nrw" }    # GetFeatureInfo on nw_dop_utm_info; a `wms-featureinfo` date kind covers it later

[terrain]
backend = "geobasis-nrw"               # tile list from dgm1_meta.zip (CSV), then GeoTIFF tiles
[surface]
backend = "geobasis-nrw"
```

The backend holds the part no manifest can say: read the CSV in the zip, build the tile names (letters, digits and
underscores only), pick the newest scan year per tile, cache the tiles, mosaic them. About 460 lines today, of which
the tile list and the cache are roughly 300.

**Saxony-Anhalt** is the one existing state that needs no backend at all: WMS, WCS with `multipart = true` and an
`envelope`, and the flight date by `date = { kind = "wfs-geojson", layer = "atkis_geobasis_bestand:truedop_kacheln",
field = "BFDATUM", format = "DD.MM.YYYY" }`. Lower Saxony is the second candidate (WCS plain TIFF, `date = { kind =
"stac-newest" }`). That makes 2 of 11 states fully declarative, 9 manifest plus backend, and 1 (Berlin) a code
adapter with a thin manifest. The honest reading: the manifest saves most of the work for a new state with standard
services, and saves the maintainer little for the states that exist.

### 3.6 What a manifest cannot express

| Source type | Why not | What to do |
|---|---|---|
| Tile lists from a CSV, WFS, AJAX portal or xlsx (NW, RP, SH, MV, HE) | The list format and the name rules differ per portal | `RasterBackend` using `freemapper-sources` tile-list helpers |
| Members of large zips by byte range (HH, HE) | Needs the archive layout | backend using `zip_range` |
| Metalink bulk download (RP) | A protocol, plus hash checks | backend using `metalink` |
| A pinned certificate chain (SH) | Trust is code and a bundled PEM | backend plus a `TrustAnchors` declaration in the code adapter. A manifest can never pin or relax TLS (section 3.7). |
| Berlin's ten layers, K5, registers | Per-layer logic and dedup ranks | code adapter implementing `ExtraLayer` |
| Logic over the answer (paging cut, silent limits) | A manifest has `page_count`, `max_count` and nothing more | request in a `freemapper-sources` preset or write code |
| A source outside ALKIS-like buildings (Basis-DLM, planned for ST, HE, SH, MV) | Different schema and rank | a new `[alkis] kind` in a later format revision, with the DLM rule section |
| Tile servers by coordinate pattern (`{e}_{n}.tif`) | Expressible, but needs the tile cache generalised | `kind = "tile-pattern"` after the first migration (task D8); until then a backend |

### 3.7 Registry, discovery and trust

```rust
pub struct Registry { adapters: Vec<Arc<dyn RegionAdapter>>, backends: BTreeMap<&'static str, Arc<dyn RasterBackend>> }
impl Registry {
    pub fn builtin() -> Self;                         // embedded manifests plus feature-enabled code adapters
    pub fn with_dirs(self, dirs: &[AdapterDir]) -> (Self, Vec<AdapterWarning>);
    pub fn get(&self, id: &AdapterId) -> Option<&dyn RegionAdapter>;
    pub fn regions_for(&self, bbox: &BoundingBox) -> Vec<&dyn RegionAdapter>;
}
```

Search order and precedence:

1. Built-in (compiled in). Cannot be replaced by a folder adapter: an id collision is an error, not an override.
2. The user folder `<config dir>/freemapper/adapters/<id>/` (`dirs::config_dir()`), or `FREEMAPPER_ADAPTER_DIR`.
   Loaded by default. The user put the files there.
3. The project folder `<project>/adapters/<id>/`. Not loaded by default, and never through MCP. The CLI needs
   `--adapters-from-project`, which prints each adapter id and host list first. A downloaded project must not be able
   to add a source (ADR 0059 spirit: a shared folder is untrusted).

A broken folder adapter is skipped with a warning naming the file and line; the rest load. `freemapper adapters list`
shows id, source (built-in, user, project), revision, hosts and `last_verified`.

A project records `provider` and `regions` by id. Opening a project whose adapter is missing is an error that names
the id and says where adapters are searched, not a silent re-fetch.

### 3.8 Security

Manifests are untrusted input, even from the user folder: people copy them from forums and repositories.

- Size. A manifest file is at most 64 KiB, a boundary file at most 2 MiB, at most 64 URLs per manifest, a folder at
  most 64 files. Parsing is `toml` into a `deny_unknown_fields` struct; an unknown key is an error, so typos and
  smuggled fields fail loudly.
- URLs. `https` only; no `http`, `file`, `ftp`. The host is a DNS name (no IP literal, no `localhost`, no trailing
  dot tricks, IDNA-normalised and compared in lower case), no user info, port 443 only. Every URL's host must be in
  `[net] hosts`, and `hosts` is shown by `adapters list` and `adapter check`. No wildcard hosts.
- SSRF. The declared-host rule stops a manifest from steering freemapper to a name it did not declare, but a declared
  name can still resolve to `127.0.0.1` or `10.x`. `HttpFetcher` therefore resolves through its own resolver and
  refuses loopback, link-local, private, unique-local, multicast and unspecified addresses, for built-in adapters too,
  checked at connect time against the address actually used (no check-then-connect gap). Redirects: at most 3, only
  `https`, only to a host in the same `[net] hosts`, and the address check runs again. The existing rule for the
  pinned SH host (no redirects) stays.
- TLS. A manifest cannot supply certificates, disable verification or choose roots. Pins live in code adapters and
  go through a typed `TrustAnchors` value that names one host (today `PORTAL_HOST`).
- Size of answers. Every call has a byte limit: the manifest may lower the default, never raise it above the cap of
  its call type (NAS answers 256 MB as `MAX_ANSWER_BYTES` today, JSON 32 MB, images and tiles 200 MB). A response over the
  limit is `TooLarge`, as now.
- Paths. Names that come from a server or a manifest reach a path only through a fixed table or a strict pattern
  (ADR 0059). The manifest cannot name `data/` files, cache files or output files. The adapter id is part of cache
  folder names, so it matches `[a-z][a-z0-9]{1,31}`.
- Writes. All through `ProjectSink` and `freemapper-fs` (ADR 0060).
- Credit and rules. `[attribution].text` is printed into PDFs and `ATTRIBUTION.txt`: at most 400 characters, no
  control characters, no newline. A `rules.toml` in an adapter folder may add candidate rules only (never map
  objects), uses the existing rule engine and validator, and its `freemapper:source` is `adapter/<id>`, ranked below
  OSM so it can never win a duplicate.
- The selection page. The browser loads the `[preview]` WMS tiles itself, so `app.js` gets the list from
  `/api/adapters` and the page's content security policy lists only the declared preview hosts. The page never
  receives other manifest text as HTML (`textContent` only).
- Trust prompt. Considered and dropped for the user folder (D7): a prompt per adapter is friction that people click
  through. The policy checks above carry the weight. The project folder gets the flag instead.

### 3.9 How adapters tie into the rules and the build

The build reads normalised ALKIS features: the LGB property spellings (`gebnutzbez`, `funktion`, `bez`, `nutzart`).
Each state's ALKIS reader maps its spelling to that schema. In the new model:

- `[alkis].normalise` names a preset in `freemapper-sources`: `lgb` (default, what the nine `WfsService` rows use) or
  `hamburg-ascii` (today `lgv::normalise_building` and `normalise_land_use`). New presets are code and need a
  freemapper release. A contributor whose state differs adds a preset in a pull request, or an adapter crate with
  its own `AlkisSource`.
- The build keeps `build/state.rs` writing `alkis:gebnutzbez` and `alkis:funktion` by `lgv::spelling_key`, so the
  shared `state-candidates.toml` and `adv-codes.toml` apply to every state without a per-state rule file.
- An adapter's optional `rules.toml` adds candidate rules for what is special about its data (D11).

### 3.10 Format versions

- `format = 1` is the first. A change that old freemapper cannot read bumps the integer; additive keys do not.
- freemapper reads format N and N-1 for at least two minor releases after N appears, and prints a deprecation
  warning with the manifest path for N-1.
- `min_freemapper` is a semver floor. A manifest that needs a newer freemapper is skipped with
  "adapter hamburg needs freemapper 0.10.0 or newer (this is 0.9.0)". Unknown keys inside format 1 are errors.
- `revision` is the adapter's own data version. For external adapters, the project record stores
  `adapter_revision`, and `fetch` warns when it differs ("the adapter changed since the last fetch; run `fetch
  --force` to load the data again") without deleting anything. Built-in projects do not get the field, so the example
  files stay identical.

### 3.11 Errors

`AdapterError` has variants for manifest problems (with path, line and key), policy refusals (URL, rule), service
problems (wraps `FetchError`) and missing parts (backend not compiled in). Existing user-visible messages that the
manual quotes (chapter 14) keep their wording; the migration tasks assert the strings. New text follows the house
style: lower case start in `warning:` lines, names the file, says what to do.

### 3.12 i18n

Program language (ADR 0092: `--lang`, `FREEMAPPER_LANG`, Fluent through `freemapper-text`) is in v0.10.0. The
library reports `Note`s with a message id and arguments, rendered in the caller's language; lines without a message
yet stay plain English. For adapters: manifest names and the preview attribution are `{en, de}` tables (the shape of
`LocalizedText` in `freemapper-rules`), with English as fallback; `AdapterError` carries a stable code
(`manifest.unknown-key`, `policy.host-not-declared`) that becomes the Fluent message id; adapter messages are notes
like the rest. The supported-states sentence (the error in `project.rs`, the MCP text in `mcp.rs`, the page texts in
`app.js`) is built from `names` of the registry in the active language. The goldens render notes in English, so
they do not depend on `FREEMAPPER_LANG`.

### 3.13 Persisted ids and old projects

`ProjectFile.provider` becomes an `AdapterId` newtype over `String` that serialises exactly as the lower-case enum
did. `regions` in the buildings and land-use records do the same. `Product` (the five tile products, serialised in
`tiles`) moves with its backend and keeps its names. Legacy ids are a table in `crates/adapters/de` and a test
(G5) loads every `examples/*/freemapper.toml`, saves it and compares bytes.

### 3.14 Testing

| Layer | What | Where |
|---|---|---|
| Unit | manifest parser, validator and policy: 30 invalid manifests, each with the expected error code | `freemapper-manifest/tests` |
| Equivalence | each of the 9 `WfsService` rows, 3 `WmsService` and 2 `WcsService` constants equal the service built from its manifest | `crates/adapters/de/tests` |
| Fixtures replay | a `Fetcher` that answers from `fixtures/` (`index.toml` maps URL to file, status, content type); `adapter check --fixtures` runs fetch, derive and build on the adapter's golden area | `freemapper-sources::replay` |
| Golden G1 to G5 | section 3.15 | `crates/freemapper/tests/adapters_golden.rs` |
| Live | `adapter check --live --bbox ...` and a weekly scheduled workflow that runs it for every built-in adapter and opens an issue on failure; never part of the normal CI | `.github/workflows/adapters-live.yml` |

Fixture size is the constraint. A golden area is 150 m by 150 m (a sprint-map core, about 0.02 km²). The expected
size is 0.5 to 1.5 MB per state (orthophoto JPEG at 0.2 m is the bulk: about 750 by 750 px, 150 KB). The CI check
refuses a fixtures folder over 2 MB.

### 3.15 Migration and the behaviour-preserving net

Byte-stable examples are the goal. The examples in the repo ship their outputs (`.omap`, candidates, PDF, toml) but
not their `data/` folders or the cache, and the OSM extracts are 54 to 510 MB each. So an offline rebuild of an
example is not possible from the repo as it is. The net therefore has five layers, all offline, all recorded from
v0.10.0 code before the first refactor commit (revision 2026-11-11; the first version said v0.9.0):

| Layer | Records | Proves |
|---|---|---|
| G1 registry data | JSON of every per-region fact the old enums gave: CRS, provider, terrain source label, `per_area`, object-id prefix, OSM extract URL, state URL, prefix, MB, label, boundary SHA-256, attribution text for the years 2025 and 2026 and with each project record combination | the data moved without change |
| G2 geometry | `regions_for`, `raster_region`, `provider_tiles` (1 km and 2 km), `reaches_beyond_coverage`, `tile_touches`, `tile_centre_in`, `extracts_for` for the 11 example boxes plus a grid of 200 boxes on borders (BB/BE, ST/NI, NI/Bremen, HE/RP, NW/NI, MV/SH, MV/BB) | the dispatch picks the same adapter and tiles |
| G3 request trace | the ordered list of URLs, with method and range, that `fetch_project` (online) and `estimate_download` issue per state against a canned `Fetcher`; also `warning:` lines | the same requests go out |
| G4 pipeline | per state a mini project (`freemapper.toml`, `data/`, `templates/` inputs, tile cache, a 1 MB cut-down OSM PBF) for the golden area; offline fetch, `derive`, one document build and all four exports (ADR 0091); SHA-256 of the written `freemapper.toml`, `.omap`, `.ocd`, both PDFs, `candidates.omap`, `ATTRIBUTION.txt`, `contours.geojson`, the three template images | the output bytes are the same |
| G5 persisted format | load, save and byte-compare each `examples/*/freemapper.toml`; load each with a missing adapter and check the message | old projects keep working |

Recording G3 and G4 needs the network once, run by the maintainer (or an agent) with the brief that forbids the user
e-mail address in any request (memory: no e-mail in requests). The files are then committed under
`tests/golden/adapters/`. `FREEMAPPER_BLESS=1` rewrites a golden file; a CI step on every push and pull request
fails when a commit whose subject starts with `refactor(adapters):` changes `tests/golden/adapters/**`, so a
refactor cannot bless its own change (D26). G3 and G4 run under a fixed clock (D25): the project stores fetch dates,
the credit line takes its year from them, and two states decide by date whether to ask again.

The migration is a strangler: first the net (stage A), then the crates (B), then a registry with a `LegacyAdapter`
shim that wraps the enums (C1), then one concern at a time moved behind the trait (boundary and CRS, OSM, credit,
estimate, ALKIS, rasters, extras), then the ids, and last the deletion of `Region`, `Provider` and the old
`match`es (C14). After every task `cargo test --workspace` and the golden layers are green. A concern moves for all
states in one task, so the `match` and the trait never both own a concern.

### 3.16 Performance

`tile_centre_in` and `tile_touches` run inside loops over thousands of tiles (Hesse). Today `Region::boundary` is a
`OnceLock` read. The registry keeps parsed boundaries in the adapters (`OnceLock` each) and caches each boundary's
bounding rectangle so `intersects` checks the rectangle first; that is an exact pre-filter and changes no result.
Trait calls per tile cost one virtual call; the loops stay inside the adapter. Manifest parsing: 11 embedded
manifests at start, target under 5 ms in total (boundaries parse lazily on first use, as now). G2 includes a timing
test with a generous bound (twice the v0.10.0 time) to catch a quadratic regression, not to benchmark; it is
ignored on the shared CI runners and run by hand.

### 3.17 Documentation

Bilingual, in `docs/adapters/`: `en.md` and `de.md` (the guide), `manifest-v1.en.md` and `manifest-v1.de.md` (the
reference), a worked tutorial that adds a made-up state step by step with the commands. Outline of the guide:

1. Who this is for, and what you need (an open-data licence, a boundary, service URLs).
2. Decide: manifest only, manifest plus backend, or code adapter (a flow chart in text).
3. Create the folder with `freemapper adapter new <id>`.
4. Boundary: `freemapper adapter boundary`.
5. ALKIS: pick `wfs` or `ogc-api-features`, check the spelling preset.
6. Rasters: WMS, WCS, or write a backend (with the `RasterBackend` skeleton).
7. Credit and licence: what the text must contain per licence.
8. Test: `adapter check`, record fixtures, the golden area.
9. Submit: pull request checklist, what the maintainer reviews, the `last_verified` and maintainer fields.
10. Keeping it alive: the weekly live check, what happens to an adapter without a maintainer (marked
    "unmaintained" after three failed weekly runs, still loaded).

The manual chapter on regions links to the guide; the README gets one sentence and a link. The per-region reference
documents in `docs/regions/` stay as they are.

### 3.18 Where freemapper-core is used

- `freemapper-georef` (new, freemapper-owned): `Crs`, `BoundingBox`, `TileId`, reprojection.
- `freemapper-fs`: unchanged use; `CacheDir` and `ProjectSink` wrap it.
- `freemapper-text`: later, for the adapter error codes and the names, when program language lands.
- `freemapper-rules`: not used for manifests. Its schema pattern (validate a file against an app schema) is the
  model, but its profiles are rule files for courses and checks, and a manifest has a different shape. The
  `LocalizedText` shape is copied as a convention only, not as code.
- Nothing from freemapper is copied into another repo; the contributor guide lives here.

## 4. Open points left to the builder

- The exact field set of `RegionInfo`, `UsedSources`, `EstimateArea` (task C1 fixes them from the code that moves).
- Whether `ExtraLayer` is one trait or two (Berlin layers and NAS/trees differ in their record types).
- The final names of the `adapter` subcommands' flags.
- The wording of the new warnings.

## 5. Risks

| Risk | Likelihood | Effect | Mitigation |
|---|---|---|---|
| Behaviour drifts while 13 840 lines of `project.rs` are rearranged | medium | silent change in an example | G1 to G5 before the first move; every task move-only; golden changes forbidden in refactor PRs |
| Fixtures and PBF cut-downs bloat the repository | medium | slow clones | 2 MB per state cap, one golden area, PBF cut by a script kept in the repo |
| Recorded fixtures hide a service change | high over time | green tests, live failure | weekly live workflow; fixtures are for regression, not for truth |
| Manifest language creep | medium | a second programming language | closed lists, a format revision for every new kind, review rule: "if it needs an `if`, it is code" |
| SSRF or request smuggling through URL templates | low after the policy | internal network access | declared hosts, address checks at connect time, no raw query strings, percent-encoded placeholders, tests with a stub resolver |
| Goldens change with the date (fetch dates, credit year, 30-day rechecks) | certain without a fix | red tests on 1 January, flaky G3 | injectable clock first (D25) |
| Other sessions edit the same CLI files (page, MCP) | high | merge conflicts, lost texts | agree each edit with the owning session; stage A merges early so the goldens guard their work too |
| Contributor adapters rot | high | broken states for users | maintainer field, weekly check, "unmaintained" marking, default-off features for non-reviewed crates |
| Two extension mechanisms (manifest + backend, code adapter) confuse contributors | medium | support load | the decision flow in the guide; `adapter new --kind` scaffolds the right one |
| Compile time grows with 4 new crates | low | slower CI | the readers were one crate's modules before; measure in task B4 |
| Licence of contributed boundaries and credit text is wrong | medium | legal risk | closed licence list, `adapter check` refuses a missing credit, review focus item |
| Windows differences in folder discovery and file names | low | adapter not found | `dirs::config_dir`, CI on 3 OSes, tests with `tempfile` folders |
| Persisted-id regressions break old projects | low | users must re-fetch | G5 and the legacy id table |

## 6. Review focus

For a reviewer of this design:

1. Is "manifest plus named backend" the right middle? It departs from the agreed split of "manifests for standard,
   crates for special", where a special state was a whole crate. Check section 3.4 against the sample states.
2. The crate cut (3.2): can `freemapper-sources` really stand without `freemapper`? Look at `Raster`, `GridSpec`,
   `Cache` and the `ProjectError` uses in the readers.
3. The security policy (3.8): is the declared-host rule plus connect-time address check enough, or do we also need
   a content-type or size sniff per call type?
4. G4: is a 150 m golden area enough to catch a change in ALKIS merge, border handling and contour output? The
   border cases are in G2 and G3 only.
5. Persisted ids: any place where the enum name leaks into a file besides `freemapper.toml` and the cache folder
   names? Search `Debug` formatting of `Region` and `Provider`.
6. Performance of the registry inside `provider_tiles` for Hesse.
7. Whether `adapters-from-project` should exist at all.
8. The decision to leave `Product` and the two raster kinds alone in the first pass (D9).
9. Whether two of eleven states being fully declarative justifies the manifest effort for existing states, or only
   for new ones (D2 says: new states and the ALKIS/WMS parts of old ones).

## 7. Staged plan in one view

| Stage | Result | Tasks | Size |
|---|---|---:|---|
| A | Net: clock, glossary, G1 to G5, CI guard | 6 | about 200 lines production, 1 200 lines of tests and fixtures |
| B | Crates: api, sources (georef in core is done) | 2 | moves, about 6 000 lines moved, 400 new |
| C | Trait and registry, all concerns, enums deleted | 14 | about 4 000 lines moved, 1 500 new |
| D | Manifest: schema, loader, policy, manifests for 11 states | 7 (+1 deferred) | about 2 300 new, 11 manifests |
| E | Tooling: replay in the library, `adapter check`; then one new state as the gate | 2 (+2 deferred) | about 800 new |
| F | Docs: guide, reference, manual, README | 2 | docs only |
| G | New states BW, Bremen, Bayern, Saarland (separate plans) | 4 | one plan each |

The plan file has the detail. 33 active tasks in stages A to F (revision 2026-11-11; the first version had 37);
stage G has plans of its own.

## 8. Alternatives considered

| Option | Why not |
|---|---|
| Keep the enums, add a macro that generates the arms | Still one file edit set per state; contributors still need Rust and the full pipeline |
| Only manifests, no code adapters | 9 of 11 states need code; the maintainer would keep a growing list of "kinds" |
| Only code adapters (traits, no manifests) | Contributors without Rust cannot take part; standard OGC states repeat the same 60 lines |
| WASM or dynamic plugins | Needs an ABI, a sandbox and signing; a shared library has the full rights of the program. Cargo features cost one line and are reviewed in a pull request |
| One crate per German state | 11 crates for 11 small modules; compile and release overhead without a second user. One `adapters/de` crate with modules |
| Manifest with scripting (Lua, Rhai, jq expressions) | New dependency, new attack surface, a language to document. Closed `kind` lists instead |
| Prompt to trust each user-folder adapter | Click-through habit; the policy does the work |
| Replace record types with a unified raster model first | Larger, riskier, changes `derive` input files. Later cleanup (D9) |

## 9. Decision log

Every call below was made without a human partner, taking the recommended option. "Cost" is the cost if wrong.

| # | Decision | Reason | Cost if wrong |
|---|---|---|---|
| D1 | One `RegionAdapter` trait with optional capabilities (`rasters`, `alkis`, `extras`), not one trait per concern | the registry and `regions_for` need one handle per region; capabilities as `Option` keep a partial adapter possible (ALKIS only) | adapters with all `None` compile and do nothing; `check` must reject them |
| D2 | Manifest plus named code backend inside one adapter; a pure code adapter is the exception (Berlin) | 9 of 11 states differ only in the heights source; the rest is standard data | two documented ways to extend; a backend name is a string the loader resolves at start |
| D3 | No dynamic plugins; code enters through workspace crates and Cargo features | no ABI, no sandbox, reviewed in a pull request; WASM would still need host calls for HTTP and write rules | out-of-tree authors need a fork or a pull request; a plugin story later is a new ADR |
| D4 | Move `Crs`, `BoundingBox`, `TileId` to `freemapper-georef` in core first (done in core v0.5.0) | ADR 0067 and core ADR 0005 plan it and freemapper owns it; no copy across repos | none left |
| D5 | Adapters live in `crates/adapters/*`, explicit registration list in the CLI crate | no link-time magic; greppable; feature flags choose what ships | a forgotten line means the adapter is silently absent; `adapters list` and a test catch it |
| D6 | German states in one crate `crates/adapters/de`, manifests embedded with `include_str!` | stays in tree as required; one place for the maintainer | a German-only contributor still builds all eleven |
| D7 | Three sources: built-in, user folder (default on), project folder (opt-in flag, never MCP); no trust prompt | a shared project must not add sources; prompts get clicked away | someone who sets the flag on a hostile project is exposed to the declared hosts only |
| D8 | `[net] hosts` required; https, port 443, DNS names; connect-time address check | lets a reviewer see every host; stops declared-name rebinding to internal addresses | some legitimate service on another port or an IP-only host is not supported in format 1 |
| D9 | First migration keeps the two raster kinds, `Product`, and all record types unchanged | byte stability; unifying changes which files `derive` reads | adapter authors see two raster shapes until the cleanup |
| D10 | Ids stay `[a-z][a-z0-9]{1,31}` and the legacy ids (`saxonyanhalt`, `lvermgeosh`) are kept | persisted in projects and cache folder names | ugly ids forever; a rename needs a migration |
| D11 | An adapter `rules.toml` may add candidate rules only, ranked below OSM | candidates are suggestions; map objects must stay under maintainer review | a contributor with a real map-object rule must send a pull request to the core rules |
| D12 | Closed licence list: `dl-de/zero-2-0`, `dl-de/by-2-0`, `cc-by-4.0`, `odbl`, `cc0-1.0`, `other` (needs `licence_text`, flagged by `check`) | the credit is a legal duty and `source_block` merges lines by licence | a licence outside the list waits for a release or uses `other` |
| D13 | Boundaries come from a file in the folder, EPSG:4326 or 25833, simplified to 25 m by `adapter boundary` | matches ADR 0025 and the current assets; one rule for everyone | a 25 m simplification can put a point near a border in the wrong state (known, as today) |
| D14 | OSM extracts are declared by Geofabrik path; URLs are derived | host fixed, nothing to mistype | a non-Geofabrik source needs a new `kind` |
| D15 | Golden net of five layers, recorded from v0.10.0 code (first version: v0.9.0), with the guard that refactor commits may not edit goldens | the examples do not ship their input data, so "rebuild an example offline" needs recorded inputs | recording needs the network once; stale services do not matter afterwards |
| D16 | Strangler migration with a `LegacyAdapter` shim and one concern per task, all states at once per concern | a concern never has two owners; every task is revertible | the shim exists for about 13 tasks and is removed in C14 |
| D17 | Manifest format integer, N and N-1 accepted, `deny_unknown_fields`, `min_freemapper` | typos fail; old manifests keep working for two releases | users with old manifests see warnings |
| D18 | `adapter_revision` stored only for non-built-in adapters | examples stay byte-identical | built-in adapter data changes are not flagged in projects |
| D19 | Errors keep today's wording; new errors carry stable codes | the manual quotes messages; the codes become Fluent ids later | none until program language lands |
| D20 | Registry order equals `Region::ALL` for built-ins, then ids sorted | tie-breaking in `raster_region` must not change | an external adapter's border behaviour depends on its id |
| D21 | The selection page's aerial layers and the supported-states texts come from the registry (`/api/adapters`) | removes 4 hard-coded lists | one more endpoint to secure; text built at runtime |
| D22 | Stage G states follow in separate plans | their research is separate work | none |
| D23 | The registry is process-global (`registry()`, installed once at start); every function also has an `_in(&Registry, ..)` form | the free functions of `region.rs` are called from 40 sites and keep their signatures, so each move stays small | tests with custom registries must use the `_in` forms; hidden global state |
| D24 | The golden area is 150 m by 150 m per state and fixtures are capped at 2 MB | the repository stays cloneable; a larger area adds little to the dispatch coverage | a bug that shows only on a larger area is not caught by G4 (G2 and G3 cover borders) |
| D25 | One injectable clock (`freemapper::clock`) replaces the direct `Local::now()` and `SystemTime::now()` calls of the library before the net is recorded; goldens fix it to 2026-11-11 | stored fetch dates, the credit year and the 30-day rechecks would otherwise make G3 and G4 change with the day | one more module every date must go through; a test refuses direct calls |
| D26 | The golden guard checks the subject of every commit on push and pull request, not the pull-request title | about half of the merges reach `main` as local merges, which a title check never sees | a refactor commit with another subject prefix slips through; reviews still see it |
| D27 | `tile-pattern` (D8 of the plan), `adapter record`, `adapter new`, `adapter boundary` and the weekly live workflow wait for the first outside contributor; one new state (Saarland or Bremen) tests the design after `adapter check` and before the guide | the saving is for new states; the first one shows which tools matter | the first contributor waits for a tool, or writes a manifest by hand from the guide |

## 10. The eight rulings that matter most

1. D2: manifest plus named backend (hybrid) instead of manifest versus crate.
2. D3: no dynamic plugins; Cargo features and workspace crates.
3. D8: declared hosts, https only, address check at connect time (the SSRF answer).
4. D15 and D16: five-layer golden net first; strangler migration; refactor PRs cannot edit goldens.
5. D4: `freemapper-georef` to core first, so the api crate has no copy of core code.
6. D7: project-local adapters are opt-in and never reachable through MCP.
7. D9 and D10: raster kinds, `Product` and the legacy ids stay as they are in the first pass, for byte stability.
8. D12 and D17: closed lists (licences, kinds, presets) and a versioned, `deny_unknown_fields` format.
