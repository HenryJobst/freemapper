# 0030. North Rhine-Westphalia: tile names and dates only from the meta list

Date: 2026-11-08
Status: Accepted

## Context

The NRW DGM1 and DOM1 are 1 km tiles in single files. The year of the laser survey is part of each file name
(`dgm1_32_354_5642_1_nw_2022.tif`) and differs per tile, so a name cannot be computed from the coordinates. The
directory listing of the file server has the names, but it is 3 to 4 MB and slow (about 3 s), and for some products it
holds stale duplicates: the DOP and bDOM50 listings name 756 and 1,196 tiles twice. Each product also has a small
`*_meta.zip` (about 110 KB, 0.2 to 0.35 s) with one CSV row per tile, giving the exact file name and the survey date.

## Decision

freemapper reads `dgm1_meta.zip` and `dom1_meta.zip` and takes every tile name and date from them. It never computes a
name and never reads the directory listing. If the list holds a tile twice, the newer date wins. The list is stored
with its release date; a server answer older than the stored list is ignored with a warning, and an unreadable answer
falls back to the stored list. The list is capped at 8 MB and the rows are checked: rows without a usable name are
skipped and counted in a warning. A tile that the list does not name inside NRW is reported as missing, and the
templates stay empty there. Offline, the stored list is used, or the run stops with a message.

## Consequences

- One small request names every tile and gives the date shown in the project entry.
- The DGM1 and DOM1 of a tile come from the same flight, so the nDSM mixes no years.
- If the CSV loses a column the reader stops and names it.
- Cost if wrong: if the list is behind the files, a newer tile is not used until the list is updated.

Source: NRW reference §2.1, §2.2, §7; NRW plan `2026-11-08-nordrhein-westfalen.md` (Rulings) and commits 679083a,
10d4c56 on branch `nordrhein-westfalen`.
