# Bavaria: the DOM20 by range — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** freemapper reads only the blocks of a DOM20 tile that the map area needs.

**Architecture:** A range reader of tiled Float32 TIFFs in `freemapper-sources`; the LDBV backend grows its cached
1 m copy block by block and lists the blocks read beside it; the estimate counts the blocks still to read.

**Spec:** `docs/superpowers/specs/2026-11-15-bayern-dom20-range-design.md`

## Global Constraints

- The 1 m surface equals the whole-tile path cell for cell (`f32::max` over the 25 values of a cell).
- Block list file: `<name>.blocks` beside `<name>.tif` in `ldbv/dom20/`; a copy without it is whole.
- Estimate: 45/400 MB per block not yet read.
- Classic TIFF only; LZW (5); one Float32 sample; predictor 1, 2 or 3; tiled.
- Disk: `CARGO_INCREMENTAL=0`, `cargo clean` after builds.

## Review Focus

1. A second area in the same tile reads only the blocks it lacks, and cells on a block border come out right.
2. A cached copy of the old kind (no `.blocks`) is used as a whole tile, with no request.
3. A server that ignores ranges, or a file that is no tiled Float32 TIFF, gives an error naming it.
4. Offline with blocks missing fails like a missing tile.
5. The block list of a copy whose day changed does not leak into the new copy.

### Task 1: Decode one LZW block (`float_tiff.rs`)

- Produces: `pub fn decode_lzw_block(compressed: &[u8], width: u32, height: u32, predictor: u16, little: bool) -> Result<Vec<f32>, String>`; NoData to NaN.
- Test `a_block_decodes_like_the_whole_image`: a tiled LZW TIFF written by the test helper of Task 2, each block decoded alone equals the same pixels of `decode_float_tiff`.

### Task 2: The range reader (`tiff_range.rs`)

- Produces: `pub struct TiledTiff { width, height, block_width, block_height, west, north, pixel, metadata: Option<String>, .. }`, `pub fn read_layout(fetcher, url) -> Result<TiledTiff, FetchError>`, `impl TiledTiff { pub fn blocks_across(&self) -> u32; pub fn block_count(&self) -> usize }`, `pub fn read_blocks(fetcher, url, &TiledTiff, &[usize]) -> Result<Vec<(usize, Vec<f32>)>, FetchError>`; a test helper `tiled_float_tiff(grid, values, block, predictor, metadata) -> Vec<u8>` (IFD at the end, as the LDBV writes it).
- Tests: `the_layout_is_read_from_the_end_of_the_file` (sizes, grid, metadata, four requests); `blocks_in_a_row_are_one_range`; `a_strip_tiff_is_refused`; `an_ignored_range_is_an_error`.

### Task 3: The 1 m copy grows (`ldbv.rs`)

- Produces: `fn blocks_of(tile, bbox, layout) -> Vec<usize>`, `fn merge_blocks(coarse: &mut Raster<f32>, layout, blocks)`, the `.blocks` file, the DOM20 path in `fetch_model` before `mosaic_tiles`.
- Tests: `the_dom20_is_read_by_range` (only the blocks of the area are asked for; the surface equals the whole-tile path); `a_second_area_reads_only_the_blocks_it_lacks` (block border inside, cells equal the whole-tile path); `an_old_copy_is_a_whole_tile`; `offline_with_blocks_missing_is_not_cached`; `a_newer_tile_starts_a_new_block_list`.

### Task 4: Estimate and texts

- `ldbv_items`: the surface item counts the blocks not yet read (test changed: a small area costs less than 45 MB; a copy with all blocks of the area costs 0).
- Progress note `progress-surface-blocks` (en, de). ADR 0141, manual (de, en), reference, changelog.
- Live run: the Schmausenbuck with an empty DOM20 cache in scratch; compare the stored surface with the old copy.
