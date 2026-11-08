# 0045. Disk space: checked before fetching, optional parts left out, a full disk reported plainly

Date: 2026-11-08
Status: Accepted

## Context

freemapper writes to two places: the cache shared by all projects (`dirs::cache_dir()/freemapper` or `--cache-dir`),
which holds OSM extracts of up to about 650 MB, raster tiles and zips, and the project folder, which holds `data/`
(WMS and WCS rasters, mosaics) and `templates/`. The two can be on different drives. Until now a fetch on a nearly full
disk ran until a write failed with ENOSPC. The message was the system's ("No space left on device"), the transfer was
retried twice, and a partial file could stay behind in the cache or the project folder.

The user asked that freemapper respect the free space and warn when parts cannot be used for lack of it.

## Decision

**Query.** `fs4` (pinned `=0.13.1`, default features off) reads the space available to the user (`statvfs` on Linux
and macOS, `GetDiskFreeSpaceExW` on Windows). The workspace forbids `unsafe`, so calling `libc::statvfs` ourselves was
not an option. The query is the trait `space::DiskSpace`. `Cache` carries it, with `SystemDiskSpace` by default and
`Cache::with_disk_space` for tests. A folder that does not exist yet is measured at its nearest existing parent. Two
paths are on the same volume when `st_dev` matches (Unix) or the drive prefix matches (Windows).

**Parts.** Every `EstimateItem` now names its `Part` and its `Store` (cache or project folder). In the order they are
kept:

1. `MapData`: ALKIS and an OSM extract not cached yet. Required.
2. `Terrain`: the DGM, including the HVBG metadata list and the RP Metalink of the DGM1. Required, derive needs it.
3. `Orthophoto`: Required. Derive fails without it, and it is the template the mapper draws on.
4. `Surface`: nDOM, bDOM or DOM1. Optional, it only feeds the nDSM template.
5. `OsmUpdate`: a newer OSM extract where an older one is cached. Optional, the older one still works.

The surface model comes before the OSM update because a skipped surface model leaves the project without its nDSM
template, while a skipped update leaves an extract a few weeks old.

**Need.** For each download not cached yet: its estimated size on its own volume, plus a share for the files derived
from it in the project folder: 1.0 for terrain and orthophoto (mosaic or template about as large as the download), 0.5
for the surface model (the nDSM PNG is smaller than the model; the 105 MB SH bDOM tiles are reduced to 1 m), 0 for the
map data. A download streams into a `.part` file on the same volume and is then renamed, so it needs its size once.
The estimate is deliberately generous; when in doubt it is better to skip a template than to fill the disk.

**Reserve.** `RESERVE_BYTES` = 1 GB stays free on every volume. A fixed amount is simpler to explain than a percentage,
and 5 % of a large disk would block runs for no reason.

**Plan.** `space::plan` checks the required parts first, then the optional ones in the order above, each against what
the parts before it left. A later small part may fit where a larger one did not. Cache and project folder on the same
volume share one budget; on two volumes each is checked for itself. Downloads that are cached need nothing, so a run
that downloads nothing never fails the check.

**Fetch.** Online, `fetch_project` plans before the first request. A required part that does not fit is
`ProjectError::NotEnoughSpace`, before anything is downloaded. An optional part that does not fit costs one warning
naming the part, its need, the volume's folder and what is free beyond the reserve. A skipped surface model is not
requested; a stored one that covers the area stays, as do cached tiles recorded before. A skipped OSM update reads the
cached extract as offline would. Offline nothing is planned. When the space cannot be read, freemapper warns and
fetches unchecked.

**Write.** `stream_to_file` (HTTP downloads), the default `Fetcher::download_to` and `fsutil::write_part` (all atomic
writes, so cache entries and templates) remove their partial file on any failure. A disk-full error is not retried.
The `Io` errors of fetch, derive and project say "the disk is full, so freemapper could not write …" when the cause is
ENOSPC, EDQUOT or the Windows equivalents. The map file is now written atomically too, so a full disk leaves the old map
as it was. A full disk during an OSM update falls back to the cached extract with its own warning.

**Estimate output.** The MCP tool `estimate_download` and the area selection page show each volume with its free and
needed MB, the reserve, a `fits` flag per item and the messages of what does not fit. The CLI has no estimate command;
`fetch` and `run` print the warnings.

## Consequences

- On a full disk a run ends early with a message that says what to do, and leaves no partial files.
- On a tight disk the project loses its nDSM template first, and nothing else it needs.
- The need is an estimate. A run can still fill the disk (another program writes at the same time, a cached part's
  derived files were not counted); the write guard covers that case.
- Cost if wrong: a too generous share skips a surface model that would have fitted; a too small one leads to the
  disk-full message instead of a warning. Both are visible in the messages and fixed by changing a constant.

Source: user request of 2026-11-08; branch `disk-space`.
