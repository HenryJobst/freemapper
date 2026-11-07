# M2 + M3a: LGB Fetch, Cache, Orthophoto and Hillshade Templates – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper run <dir>` downloads the LGB orthophoto (DOP20) and DTM (DGM1) tiles for the map
area, caches them, and writes orthophoto, hillshade and slope templates that appear georeferenced behind
the map in OOM.

**Architecture:** A tile grid (`TileId`, `BoundingBox`) selects the 1 km tiles covering map area + buffer.
A `Fetcher` trait (HTTP in production, in-memory in tests) feeds a shared on-disk `Cache` keyed by tile
and publication date, using the LGB currency CSVs. A small raster type with world-file georeferencing
mosaics and clips the tiles. Derived templates are written as **JPEG/PNG + world file**, which OOM reads
for `TemplateImage` (world file first, CRS from the template's `<crs_spec>`), so no GeoTIFF writer is needed.

**Tech Stack:** Rust 2024; new crates `ureq` (HTTP), `zip` (tile archives), `image` (JPEG/PNG),
`tiff` (DGM1 GeoTIFF f32 LZW), `dirs` (cache location).

**Spec:** `docs/superpowers/specs/2026-11-07-freemapper-design.md` (§4.1, §4.2, §4.4, §4.7, §6 M2/M3).
Scope cut: this plan covers M2 except place/OSM-id lookup (`--place`, `--osm`) and M3 except contours and
nDSM. Those follow in plan M2b/M3b.

## Global Constraints

- Source code entirely English; docs bilingual (README DE + EN); terms from `docs/glossary.md`
  (`TileId`, `BoundingBox`, `Cache`, `Product`, `Dtm`, `Hillshade`, `Slope`, `Orthophoto`, `currency`,
  `fetch`, `derive`).
- Pure-Rust dependencies only (`ureq` uses rustls; no OpenSSL, GDAL, PROJ).
- Dependency versions at least two weeks old, pinned exactly where newer releases are younger:
  `ureq = "=3.4.2"`, `zip = { version = "=9.0.5", default-features = false, features = ["deflate"] }`,
  `image = { version = "0.25.10", default-features = false, features = ["jpeg", "png"] }`,
  `tiff = "0.11.3"`, `dirs = "7.0.0"`.
- LGB base URL `https://data.geobasis-bb.de/geobasis/daten/`; tiles `dop/rgb_jpg/dop_33EEE-NNNN.zip`
  (members `dop_33EEE-NNNN.jpg` + `.jgw`, 5000×5000 px, 0.2 m) and `dgm/tif/dgm_33EEE-NNNN.zip`
  (members `dgm_33EEE-NNNN.tif` + `.tfw`, 1000×1000 f32, 1 m, LZW, nodata `-9999`).
- Currency CSVs `https://data.geobasis-bb.de/geobasis/information/aktualitaeten/bb_dop_aktualitaet.csv`
  and `…/bb_dgm_aktualitaet.csv`: UTF-8 with BOM, `;`-separated, header
  `sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate`, dates `YYYY-MM-DD`,
  empty when absent. Live values for `33421-5821`: DOP `2026-05-01;;2026-10-09`, DGM
  `2018-11-27;2023-05-04;2026-01-26`.
- Attribution: `© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert` (year = latest publication year
  of the used tiles), in map notes and `ATTRIBUTION.txt`, next to the existing OSM line.
- Hillshade: azimuth 315°, altitude 45°. Default buffer: schoolyard 20 m, sprint 50 m (exists).
- Every OOM-facing change must still open in OOM 0.9.6 without warnings.

## Review Focus

1. **Map area across a tile edge** – the mosaic must be seamless, with no gap or doubled row at the
   1 km boundary (Task 4: `paste_two_tiles_side_by_side_is_seamless`).
2. **Area outside Brandenburg / tile missing from the currency CSV** – a clear error naming the tile,
   no half-written project (Task 3: `fetch_unknown_tile_names_it`).
3. **Download interrupted or truncated** – no corrupt archive in the cache; the next run downloads it
   again (Task 3: `truncated_download_is_not_cached`).
4. **`--offline` with an empty cache** – an error listing the missing tiles, no network access
   (Task 3: `offline_without_cache_lists_missing_tiles`).
5. **`export` before `fetch`/`derive`** – still works and references only templates that exist; an
   edited map is still protected (Task 7: `export_without_rasters_has_only_candidates`).

Also covered: DGM nodata cells become transparent instead of black (Task 5:
`nodata_neighbourhood_is_transparent`); old `freemapper.toml` files without tile records still load
(Task 6: `project_file_without_tiles_loads`).

---

### Task 1: Area fixes and bounding box

Deferred M1 minors plus the projected bounding box that the tile grid needs.

**Files:**
- Modify: `crates/freemapper/src/crs.rs` (add `BoundingBox`)
- Modify: `crates/freemapper/src/area.rs`
- Modify: `crates/freemapper/src/project.rs` (outline tags)

**Interfaces:**
- Produces: `crs::BoundingBox { min_e: f64, min_n: f64, max_e: f64, max_n: f64 }` (`Debug, Clone, Copy,
  PartialEq`) with `fn buffered(&self, m: f64) -> Self`, `fn snapped_outward(&self, step: f64) -> Self`,
  `fn intersection(&self, other: &Self) -> Option<Self>` (None if empty), `fn width(&self) -> f64`,
  `fn height(&self) -> f64`.
- Produces: `MapArea::projected_bbox(&self) -> BoundingBox` (exterior ring through `utm33_forward`).
- Produces: `AreaError::MultiPolygon(usize)` and `AreaError::OutOfRange { lat: f64, lon: f64 }`.

- [ ] **Step 1: Write failing tests** in `area.rs`:
  - `multipolygon_with_one_part_is_accepted`: a `MultiPolygon` geometry with one polygon parses to that
    polygon.
  - `multipolygon_with_several_parts_is_rejected`: two parts → `Err(AreaError::MultiPolygon(2))`, and its
    message contains `"2 parts"`.
  - `coordinates_out_of_range_are_rejected`: `[200.0, 52.0]` in a ring →
    `Err(AreaError::OutOfRange { .. })`.
  - `projected_bbox_of_fixture`: fixture `tests/fixtures/clara-zetkin-schule.geojson` → bbox inside tile
    `33421-5821`: `421000 < min_e`, `max_e < 422000`, `5821000 < min_n`, `max_n < 5822000`.
  - In `crs.rs`: `bbox_buffer_snap_intersection`: `{0.3, 0.3, 9.7, 9.7}.buffered(1.0)` →
    `{-0.7, -0.7, 10.7, 10.7}`; `.snapped_outward(1.0)` → `{-1, -1, 11, 11}`; disjoint intersection → None.
  - In `project.rs`: `outline_has_no_currency_tag`: the outline from `map_area_outline` has tags
    `freemapper:source` and `freemapper:source_id` and no `freemapper:currency` (the OSM extract date is
    unknown; the declination date was wrong there).
- [ ] **Step 2: Run** `cargo test -p freemapper` → the new tests fail.
- [ ] **Step 3: Implement.** `from_geojson` also accepts `MultiPolygon` with exactly one part, checks
  every vertex for `-90 ≤ lat ≤ 90`, `-180 ≤ lon ≤ 180`, and gives the error messages
  `"MultiPolygon with {n} parts; split the map area into one polygon"` and
  `"coordinate out of range: lat {lat}, lon {lon}"`. Remove the currency tag from `map_area_outline`.
- [ ] **Step 4: Run** `cargo test --workspace` → all pass.
- [ ] **Step 5: Commit** `fix(area): MultiPolygon, coordinate range, projected bbox; drop wrong outline currency`.

### Task 2: Tile grid and LGB product catalogue

**Files:**
- Create: `crates/freemapper/src/tiles.rs`
- Create: `crates/freemapper/src/sources/mod.rs` (`pub mod lgb;`), `crates/freemapper/src/sources/lgb.rs`
- Modify: `crates/freemapper/src/lib.rs` (`pub mod tiles; pub mod sources;`)

**Interfaces:**
- Produces: `tiles::TileId { east_km: u32, north_km: u32 }` (`Debug, Clone, Copy, PartialEq, Eq, Hash,
  PartialOrd, Ord`), `fn name(&self) -> String` (`"33421-5821"`, i.e. `33` + EEE + `-` + NNNN),
  `fn parse(s: &str) -> Option<Self>`, `fn bounds(&self) -> BoundingBox`.
- Produces: `tiles::tiles_covering(bbox: &BoundingBox) -> Vec<TileId>`: half-open cover, east from
  `floor(min_e/1000)` to `ceil(max_e/1000) - 1`, likewise north; sorted.
- Produces: `sources::lgb::Product { Orthophoto, Dtm }` (`Serialize/Deserialize`, lowercase),
  `fn tile_url(self, t: TileId) -> String`, `fn currency_csv_url(self) -> &'static str`,
  `fn raster_member(self, t: TileId) -> String` (`dop_…jpg` / `dgm_…tif`),
  `fn world_file_member(self, t: TileId) -> String` (`.jgw` / `.tfw`), `fn cache_dir_name(self) -> &'static str`
  (`"dop20rgb"` / `"dgm1"`).
- Produces: `sources::lgb::TileCurrency { created: NaiveDate, revised: Option<NaiveDate>, published: NaiveDate }`
  with `fn currency(&self) -> NaiveDate` (revised, else created);
  `parse_currency_csv(text: &str) -> Result<HashMap<TileId, TileCurrency>, CurrencyCsvError>`
  (`CurrencyCsvError { line: usize, message: String }`).
- Produces: `sources::lgb::attribution(year: i32) -> String`.

- [ ] **Step 1: Write failing tests:**
  - `tile_name_and_parse_roundtrip`: `TileId { east_km: 421, north_km: 5821 }.name() == "33421-5821"`,
    `TileId::parse("33421-5821")` gives it back, `parse("421-5821")` is None.
  - `tiles_covering_school_is_one_tile`; `tiles_covering_edge` : `{421990, 5821500, 422010, 5821600}` →
    `[421/5821, 422/5821]`; a bbox ending exactly at `422000` stays one tile.
  - `urls_match_lgb_layout`: `Product::Orthophoto.tile_url(t)` ==
    `"https://data.geobasis-bb.de/geobasis/daten/dop/rgb_jpg/dop_33421-5821.zip"`, DTM ==
    `"…/dgm/tif/dgm_33421-5821.zip"`; member names as in Global Constraints.
  - `currency_csv_parses_bom_and_empty_fields`: the three-line excerpt
    `"\u{feff}sheetnr;sheetname;creationdate;revisiondate;publicationdate;processstepdate\n33421-5821;;2018-11-27;2023-05-04;2026-01-26;\n33421-5822;;2026-05-01;;2026-10-09;2026-06-15\n"`
    → two entries; currency of the first is 2023-05-04, of the second 2026-05-01.
  - `currency_csv_reports_bad_line`: a bad date on line 3 → `Err` with `line == 3`.
  - `attribution_text`: `attribution(2026) == "© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert"`.
- [ ] **Step 2: Run** `cargo test -p freemapper tiles sources` → fail (missing modules).
- [ ] **Step 3: Implement** the signatures above. Skip lines whose `sheetnr` does not parse as a tile id
  only if they are empty; any other malformed line is an error.
- [ ] **Step 4: Run** `cargo test --workspace` → pass.
- [ ] **Step 5: Commit** `feat: LGB tile grid, product catalogue and currency CSV`.

### Task 3: Fetcher and tile cache

**Files:**
- Create: `crates/freemapper/src/fetch.rs`
- Modify: `crates/freemapper/src/lib.rs`, `Cargo.toml` (workspace deps), `crates/freemapper/Cargo.toml`
  (`ureq`, `zip`, `dirs`)

**Interfaces:**
- Consumes: `TileId`, `Product`, `TileCurrency`, `parse_currency_csv` (Task 2).
- Produces:
  - `trait Fetcher { fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>; }`
  - `struct HttpFetcher;` implementing it with `ureq`, user agent `freemapper/<CARGO_PKG_VERSION>`,
    HTTP status ≥ 400 → `FetchError::Http { url, message }`.
  - `struct Cache { root: PathBuf }` with `Cache::new(root: PathBuf)`, `Cache::default_location() ->
    Option<Cache>` (`dirs::cache_dir()/freemapper`), `fn tile_path(&self, product, tile, published:
    NaiveDate) -> PathBuf` (`<root>/lgb/<cache_dir_name>/<tile name>_<published>.zip`),
    `fn currency_csv_path(&self, product) -> PathBuf` (`<root>/lgb/<cache_dir_name>/currency.csv`).
  - `enum FetchMode { Online, Offline }`
  - `struct FetchedTile { product: Product, tile: TileId, url: String, currency: TileCurrency, zip: PathBuf }`
  - `fn fetch_tiles(fetcher: &dyn Fetcher, cache: &Cache, product: Product, tiles: &[TileId], mode:
    FetchMode, progress: &mut dyn FnMut(&str)) -> Result<Vec<FetchedTile>, FetchError>`
  - `enum FetchError { Http { url, message }, Io(PathBuf, io::Error), CorruptArchive { url, message },
    TileNotCovered(TileId), MissingOffline(Vec<TileId>), CurrencyCsv(CurrencyCsvError) }` with messages
    `"tile {name} is not covered by LGB Brandenburg data"` and
    `"offline and not cached: {names joined by \", \"}"`.

Behaviour: online, download the currency CSV once per call and store it in the cache; offline, read the
cached CSV. For each tile, look up its currency (missing → `TileNotCovered`), reuse
`tile_path(…, published)` if it exists, otherwise download to `<path>.part`, check that the bytes are a
zip containing `raster_member` and `world_file_member` (else `CorruptArchive`, delete the `.part`),
rename into place and delete older `<tile>_*.zip` of the same product. Offline, a tile without a cached
zip is collected and reported together in `MissingOffline`. Progress messages: `"downloading {url}"`,
`"cached {tile name}"`.

- [ ] **Step 1: Write failing tests** with an in-test `MapFetcher(HashMap<String, Vec<u8>>, Cell<usize>
  call count)` and zips built with `zip::ZipWriter` in memory:
  - `fetch_downloads_then_reuses_cache`: first call downloads CSV + tile (2 requests); second online
    call only re-downloads the CSV (3 requests total) and returns the same zip path.
  - `new_publication_replaces_old_tile`: CSV with publication 2026-01-26, then 2026-06-01 → only
    `33421-5821_2026-06-01.zip` remains.
  - `truncated_download_is_not_cached`: tile bytes are the first 100 bytes of a valid zip →
    `CorruptArchive`, no `.zip` and no `.part` file left in the cache.
  - `fetch_unknown_tile_names_it`: tile `33999-5999` not in the CSV → error message contains `"33999-5999"`.
  - `offline_without_cache_lists_missing_tiles`: empty cache, two tiles, cached CSV present →
    `MissingOffline` with both names; the fetcher was never called.
  - `#[ignore] live_school_tile_currency`: `HttpFetcher` gets the DOP CSV; entry for `33421-5821` exists.
- [ ] **Step 2: Run** `cargo test -p freemapper fetch` → fail.
- [ ] **Step 3: Implement** as specified.
- [ ] **Step 4: Run** `cargo test --workspace` → pass; run once
  `cargo test -p freemapper live_school_tile_currency -- --ignored` → pass.
- [ ] **Step 5: Commit** `feat: tile fetcher with currency-keyed cache`.

### Task 4: Raster type, world files, orthophoto mosaic

**Files:**
- Create: `crates/freemapper/src/raster.rs`
- Create: `crates/freemapper/src/derive/mod.rs` (`pub mod orthophoto; pub mod terrain;`, `DeriveError`),
  `crates/freemapper/src/derive/orthophoto.rs`
- Modify: `crates/freemapper/src/lib.rs`, manifests (`image`)

**Interfaces:**
- Consumes: `BoundingBox` (Task 1), `FetchedTile` (Task 3).
- Produces:
  - `raster::GridSpec { west: f64, north: f64, pixel_size: f64, width: u32, height: u32 }` with
    `fn from_world_file(text: &str, width: u32, height: u32) -> Result<Self, RasterError>` (rejects
    non-zero rotation terms and non-square pixels; world files give the *centre* of the top-left pixel,
    so `west = c - a/2`, `north = f + |e|/2`), `fn world_file(&self) -> String` (6 lines, 10 decimals,
    pixel centre), `fn bounds(&self) -> BoundingBox`, `fn covering(bbox: &BoundingBox, pixel_size: f64)
    -> Self` (bbox snapped outward to the pixel grid).
  - `raster::Raster<T> { grid: GridSpec, data: Vec<T> }` (row-major from the north-west corner) with
    `fn filled(grid: GridSpec, value: T) -> Self`, `fn get(&self, x: u32, y: u32) -> T`,
    `fn paste(&mut self, other: &Raster<T>)` (copies the overlap; grids must share pixel size and be
    aligned to within 1 % of a pixel, else panic — LGB tiles always are).
  - `derive::DeriveError { Io(PathBuf, io::Error), Zip(String), Image(String), Raster(RasterError),
    NoTiles }`.
  - `derive::orthophoto::build_orthophoto(tiles: &[FetchedTile], bbox: &BoundingBox, out_jpg: &Path)
    -> Result<(), DeriveError>`: decode each tile JPEG from its zip, paste into a white
    `Raster<[u8; 3]>` covering `bbox` at the tiles' pixel size, write JPEG quality 90 and the world
    file next to it (`orthophoto.jgw`).

- [ ] **Step 1: Write failing tests:**
  - `world_file_roundtrip_lgb`: the DOP world file `0.2/0/0/-0.2/421000.1/5821999.9` with 5000×5000 →
    `west 421000.0`, `north 5822000.0`, `pixel_size 0.2`, `bounds().max_e == 422000.0`; `world_file()`
    parses back to the same grid.
  - `rotated_world_file_is_rejected`.
  - `covering_snaps_to_pixel_grid`: bbox `{421100.05, 5821100.05, 421101.0, 5821101.0}`, 0.2 m →
    `west 421100.0`, `width 5`, `height 5`.
  - `paste_two_tiles_side_by_side_is_seamless`: two 10×10 rasters (1 m) west/east of x = 10, values 1
    and 2, pasted into a 20×10 target → columns 0–9 are 1, 10–19 are 2, no default value left.
  - `orthophoto_from_synthetic_tile`: a zip with a 50×50 red JPEG and world file
    `1/0/0/-1/421000.5/5821999.5` (west 421000, north 5822000) and the bbox
    `{421010, 5821960, 421030, 5821990}` → output JPEG 20×30 px, centre pixel red within JPEG tolerance (each channel within ±10), `orthophoto.jgw`
    parses to a grid whose bounds contain the bbox.
- [ ] **Step 2: Run** `cargo test -p freemapper raster orthophoto` → fail.
- [ ] **Step 3: Implement.** Decode with `image::load_from_memory_with_format(…, ImageFormat::Jpeg)`,
  paste only the overlap (crop before converting, to keep memory at one tile).
- [ ] **Step 4: Run** `cargo test --workspace` → pass.
- [ ] **Step 5: Commit** `feat: raster mosaic with world files and orthophoto template`.

### Task 5: DTM mosaic, hillshade and slope

**Files:**
- Create: `crates/freemapper/src/derive/terrain.rs`
- Modify: manifests (`tiff`)

**Interfaces:**
- Consumes: `Raster`, `GridSpec` (Task 4), `FetchedTile` (Task 3).
- Produces:
  - `terrain::mosaic_dtm(tiles: &[FetchedTile], bbox: &BoundingBox) -> Result<Raster<f32>, DeriveError>`
    (TIFF via `tiff::decoder::Decoder`, grid from the `.tfw`; `-9999` and unfilled cells become `NaN`).
  - `terrain::hillshade(dtm: &Raster<f32>) -> Raster<Option<u8>>`
  - `terrain::slope_deg(dtm: &Raster<f32>) -> Raster<Option<f32>>`
  - `terrain::slope_shade(slope: &Raster<Option<f32>>) -> Raster<Option<u8>>`:
    `255 - round(min(s, 30) / 30 * 255)` (flat white, ≥ 30° black).
  - `terrain::write_gray_png(r: &Raster<Option<u8>>, out_png: &Path) -> Result<(), DeriveError>`:
    grey + alpha PNG (`None` → alpha 0) plus world file `<name>.pgw`.

Gradient (Horn): for the 3×3 window `a b c / d e f / g h i` (rows north to south, edges replicated),
`dz_de = ((c + 2f + i) - (a + 2d + g)) / (8·px)`, `dz_dn = ((a + 2b + c) - (g + 2h + i)) / (8·px)`.
Surface normal `n = (-dz_de, -dz_dn, 1) / |…|`; sun vector for azimuth 315°, altitude 45°:
`l = (sin az · cos alt, cos az · cos alt, sin alt)`; hillshade `= round(255 · max(0, n·l))`;
slope `= atan(√(dz_de² + dz_dn²))` in degrees. Any `NaN` in the window → `None`.

- [ ] **Step 1: Write failing tests** on synthetic 5×5 rasters, 1 m:
  - `flat_hillshade_is_180`: constant DTM → every cell `Some(180)`; slope `Some(0.0)`.
  - `east_rising_plane`: `z = e` (1 m per m; the slope faces west, towards the sun) → hillshade
    `Some(218)`, slope 45° (±0.01).
  - `west_rising_plane`: `z = -e` (faces east, away from the sun) → hillshade `Some(37)`.
  - `north_rising_plane`: `z = n` (faces south) → hillshade `Some(37)`.
  - `nodata_neighbourhood_is_transparent`: one `NaN` in the centre → that cell and its 8 neighbours are
    `None`, the corners of the 5×5 are `Some`.
  - `slope_shade_ramp`: 0° → 255, 15° → 127, 45° → 0.
  - `dtm_from_synthetic_tile`: a zip with a 4×4 f32 LZW TIFF written by `tiff::encoder` (values 0..16,
    one `-9999`) and a `.tfw` → mosaic returns the values, the nodata cell is `NaN`.
  - `gray_png_has_alpha_and_world_file`: write a 2×1 raster `[Some(10), None]` → PNG pixel 0 is
    (10, 255 alpha), pixel 1 alpha 0; `.pgw` exists.
- [ ] **Step 2: Run** `cargo test -p freemapper terrain` → fail.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test --workspace` → pass.
- [ ] **Step 5: Commit** `feat: DTM mosaic, hillshade and slope templates`.

### Task 6: Project fetch and derive

**Files:**
- Modify: `crates/freemapper/src/project.rs`

**Interfaces:**
- Consumes: Tasks 1–5.
- Produces:
  - `project::TileRecord { product: Product, tile: String, url: String, currency: NaiveDate,
    published: NaiveDate }` (serde) and `ProjectFile.tiles: Vec<TileRecord>` with `#[serde(default)]`.
  - `ProjectFile::bbox(&self) -> Result<BoundingBox, ProjectError>` (area bbox buffered by `buffer_m`).
  - `fetch_project(dir: &Path, fetcher: &dyn Fetcher, cache: &Cache, mode: FetchMode,
    progress: &mut dyn FnMut(&str)) -> Result<ProjectFile, ProjectError>`: fetches both products for
    `tiles_covering(bbox)`, replaces `tiles` and saves. If a tile's `published` differs from the previous
    record, emits `"warning: {product} tile {name} changed upstream ({old} → {new})"`.
  - `derive_project(dir: &Path, cache: &Cache, progress: &mut dyn FnMut(&str)) -> Result<Vec<PathBuf>,
    ProjectError>`: rebuilds `FetchedTile`s from the records (`Cache::tile_path`), writes
    `templates/orthophoto.jpg`, `templates/hillshade.png`, `templates/slope.png` (+ world files),
    returns the written image paths. No records → `ProjectError::NotFetched` ("run `freemapper fetch` first").
  - `ProjectError` gains `Fetch(#[from] FetchError)`, `Derive(#[from] DeriveError)`, `NotFetched`.

- [ ] **Step 1: Write failing tests** (with the in-memory fetcher from Task 3, moved to a
  `#[cfg(test)] pub(crate) mod test_support` in `fetch.rs` so both test modules share it, plus synthetic
  zips covering tile `33421-5821`):
  - `project_file_without_tiles_loads`: an M1-style toml without `tiles` loads with `tiles == []`.
  - `fetch_records_tiles`: after `fetch_project` the toml holds two records (orthophoto, dtm) for
    `33421-5821` with url, currency and published from the CSV.
  - `fetch_warns_on_upstream_change`: second fetch with a newer publication → a progress message
    starting with `"warning: "`.
  - `derive_writes_three_templates`: after fetch, `derive_project` writes the three images and their
    world files.
  - `derive_before_fetch_fails`: `NotFetched`.
- [ ] **Step 2: Run** `cargo test -p freemapper project` → fail.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test --workspace` → pass.
- [ ] **Step 5: Commit** `feat: fetch and derive for projects`.

### Task 7: Image templates in the export

**Files:**
- Modify: `crates/freemapper/src/omap/model.rs`, `crates/freemapper/src/omap/writer.rs`,
  `crates/freemapper/src/project.rs`, `crates/freemapper/tests/omap_writer.rs`

**Interfaces:**
- Produces: `TemplateKind::Image` (written as `TemplateImage`); `TemplateRef.behind_map: bool`. The
  writer writes all `behind_map` templates first (stable order) and sets
  `first_front_template` to their count; the `<map_view>` refs follow the written order.
- `export_starter_project` adds, for each image that exists in `templates/`, in this order, behind the map:
  `orthophoto.jpg` (visible, opacity 1.0), `hillshade.png` (hidden, 0.5), `slope.png` (hidden, 0.5),
  each `georeferenced: true`, `crs_spec: Some("+init=epsg:25833")`; then `candidates.omap` in front
  (visible, 0.7) as before. With tile records, notes and `ATTRIBUTION.txt` also carry
  `attribution(latest published year)`.

- [ ] **Step 1: Write failing tests:**
  - In `tests/omap_writer.rs`: `behind_templates_come_first`: refs `[map (front), image (behind)]` →
    the image template is written first, `first_front_template="1"`, map-view ref 0 is the image.
  - `image_template_has_crs_spec`: an `Image` ref writes `type="TemplateImage"`, `georef="true"` and
    `<crs_spec>+init=epsg:25833</crs_spec>` (check how `crs_spec` is currently written and keep it).
  - In `project.rs`: `export_without_rasters_has_only_candidates`: export of a fresh project → one
    template, `first_front_template="0"`.
  - `export_after_derive_references_images`: after fetch + derive (synthetic) → 4 templates,
    `first_front_template="3"`, `relpath="templates/orthophoto.jpg"` and the LGB attribution in the notes
    and in `ATTRIBUTION.txt`.
- [ ] **Step 2: Run** `cargo test --workspace` → fail.
- [ ] **Step 3: Implement**; update existing `TemplateRef` literals with `behind_map: false`. The golden
  file `minimal.xmap` must stay byte-identical.
- [ ] **Step 4: Run** `cargo test --workspace` → pass.
- [ ] **Step 5: Commit** `feat: orthophoto, hillshade and slope as image templates behind the map`.

### Task 8: CLI `fetch`, `derive`, `run`, docs and acceptance

**Files:**
- Modify: `crates/freemapper-cli/src/main.rs`, `crates/freemapper-cli/tests/cli.rs`
- Modify: `README.md`, `README.en.md`, `docs/glossary.md`, spec §4.2/§4.4 (template file formats)

**Interfaces:**
- Consumes: `fetch_project`, `derive_project`, `export_starter_project`, `HttpFetcher`,
  `Cache::default_location`.
- CLI: `fetch <dir> [--offline]`, `derive <dir>`, `run <dir> [--offline] [--force]` (fetch + derive +
  export). Progress messages go to stderr; no cache directory → error `"no cache directory on this system"`.

- [ ] **Step 1: Write failing tests** in `cli.rs`:
  - `derive_before_fetch_explains_next_step`: `init` then `derive` → non-zero exit, stderr contains
    `"freemapper fetch"`.
  - `offline_fetch_with_empty_cache_fails_cleanly`: `init`, then `fetch --offline` with env
    `XDG_CACHE_HOME`/`HOME` pointed at an empty temp dir → non-zero exit, stderr contains `"33421-5821"`.
    (If `dirs` ignores these on macOS, add a hidden `--cache-dir <path>` option to `fetch`/`derive`/`run`
    and use it here instead.)
  - `#[ignore] live_run_school`: `init` + `run` against the network → `templates/orthophoto.jpg`,
    `hillshade.png`, `slope.png` exist and the map has 4 templates.
- [ ] **Step 2: Run** `cargo test -p freemapper-cli` → fail.
- [ ] **Step 3: Implement** the subcommands.
- [ ] **Step 4: Docs.** README DE/EN: usage `init → run`, cache location, attribution. Glossary: add
  `Weltdatei | world file | Textdatei mit Pixelgröße und Lage eines Rasterbilds (.jgw/.pgw/.tfw). | GridSpec::from_world_file`
  and `Mosaik | mosaic | Zusammengesetzte Kacheln, auf das Kartengebiet zugeschnitten. | mosaic_dtm`.
  Spec §4.2: `orthophoto.jpg`, `hillshade.png`, `slope.png` with world files instead of `.tif`, with
  the reason (OOM `TemplateImage` reads world files; no GeoTIFF writer needed).
- [ ] **Step 5: Run** `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
  then `cargo test -p freemapper-cli live_run_school -- --ignored` → all pass.
- [ ] **Step 6: Acceptance sample.** Regenerate the Clara-Zetkin-Schule sample with `run --force` and
  hand it to the user: OOM opens it without warnings, the orthophoto lies under the school outline, the
  hillshade and slope can be switched on in the template list.
- [ ] **Step 7: Commit** `feat(cli): fetch, derive, run; docs for M2/M3a`.
