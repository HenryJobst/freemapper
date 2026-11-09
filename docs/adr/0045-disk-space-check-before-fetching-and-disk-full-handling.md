# 0045. Disk space: checked before fetching, optional parts left out, a full disk reported plainly

Date: 2026-11-08
Status: Accepted

## Context

freemapper writes to the cache (OSM extracts up to about 650 MB, tiles) and to the project folder (`data/`,
`templates/`), maybe on two drives. On a full disk a fetch ran until ENOSPC, retried, and could leave partial files.

## Decision

- **Query:** `fs4 =0.13.1` (`statvfs`, `GetDiskFreeSpaceExW`; the workspace forbids `unsafe`) behind the trait
  `space::DiskSpace`, carried by `Cache` (`with_disk_space` in tests). Same volume: equal `st_dev` (Unix) or volume
  serial number (Windows, `winapi-util`; since 2026-11-09, before the drive prefix of `canonicalize`, which failed
  on some RAM and virtual drives and put a volume mounted into a folder on its drive). A folder not created yet is measured at its nearest existing parent.
- **Parts**, kept in this order: map data (ALKIS, an OSM extract not cached), terrain, orthophoto (all three
  required: derive or the map needs them), surface model, update of a cached OSM extract (both optional).
- **Need:** each download not cached on its own volume, plus derived files in the project folder (share 1.0 for
  terrain and orthophoto, 0.5 for the surface model, 0 for map data). Parts whose stored files already cover the
  area are replaced in place and need no room for derived files; a stored per-area orthophoto counts as cached.
- **Reserve:** `RESERVE_BYTES` = 1 GB free per volume; a percentage would block large disks for no reason.
- **Plan:** required parts first, then the optional ones greedily in order. One budget when cache and project share
  a volume. A required shortfall stops the online fetch before any request (`NotEnoughSpace`); an optional one is
  left out with a warning naming need and free space. A stored surface model or cached tiles that cover the area
  stay, and freemapper says so. Offline, or when the space cannot be read (warned), nothing is checked.
- **Write:** downloads and atomic writes remove their partial file on failure; disk full is not retried and reads
  "the disk is full, so freemapper could not write <file>" (`ErrorKind::StorageFull`/`QuotaExceeded`, which the
  standard library maps from ENOSPC, EDQUOT and the Windows codes). The map file is written atomically. The MCP
  tool `estimate_download` and the selection page show free and needed space, `fits` per item, the messages, and
  whether the fetch stops.

## Consequences

- A tight disk costs the nDSM template first, never a part the map needs; a full disk leaves no partial files.
- The need is an estimate; the write guard covers what it misses (another program writing, derived files of
  cached parts). A volume mounted into an NTFS folder has its own serial number, so its own space is checked.
- Cost if wrong: a share too large skips a surface model that fitted; too small leads to the disk-full message.

Source: user request of 2026-11-08; branch `disk-space`.
