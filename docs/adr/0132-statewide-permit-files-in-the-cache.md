# 0132. Statewide permit files in the cache

Date: 2026-11-14
Status: Accepted

Builds on [0124](0124-forest-administrations-in-the-permit-notes.md) and
[0131](0131-permit-sources-from-shapefiles.md).

## Context

Three permit sources of North Rhine-Westphalia and one of Brandenburg download a shapefile zip for the whole state:
about 32 MB in NRW and 4 MB in Brandenburg on every online fetch, though the files change a few times a year. The
permit sources had no access to the shared download cache; `PermitSource::fetch` gets a fetcher, the box and a
part file only.

## Decision

- `PermitSource::fetch_cached` takes the cache as well; its default calls `fetch`, so the WFS sources stay as they
  are. The engine calls `fetch_cached`.
- The shapefile sources keep their zip in the cache folder `permits`, named `<host>-<file>-<YYYYMMDD>.zip` by the
  date the server gives (`Last-Modified`), or `…-undated.zip` when it gives none. A fetch asks only for the date
  and downloads when no copy of that date is there; the new copy replaces the older ones. An undated copy serves
  for seven days. When the server cannot be asked, or the download fails, the newest copy is used; without a copy
  the download is tried all the same, as some servers answer no request for the date.
- The cache name keeps only letters, digits, `-` and `_` of the URL's host and file name.

## Consequences

- After the first fetch a run in NRW asks three dates instead of downloading 32 MB.
- A copy can be older than the server's file when the server could not be reached; the notes stay hints and name the
  fetch date of the source as before.
- Several projects in one state share the copies.
