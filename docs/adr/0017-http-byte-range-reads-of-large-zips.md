# 0017. HTTP byte-range reads of large zips, with CRC checks and per-release caches

Date: 2026-11-07
Status: Accepted

## Context

Hamburg publishes DGM1 only as a 1.36 GB statewide zip and the bDOM as a 1.34 GB zip, with no WCS and no tile
download. Its central directory is about 106 KB, and one 1 km tile is about 2 MB. Hesse later turned out to pack
DGM1 and DOM1 in municipal zips.

## Decision

freemapper reads single members out of remote zips by HTTP range:

- It reads the central directory from the archive's end, then the member's bytes.
- A server that ignores `Range` is refused, and a clamped range is an error. An empty answer to a tail read is
  retried.
- Deflated members are checked against their CRC. Zip64 and encrypted archives are refused.
- Tiles are cached per tile and per archive release. The reader is shared by Hamburg and Hesse.

## Consequences

- A Hamburg map downloads a few MB instead of 2.7 GB.
- freemapper finds members in the central directory itself and inflates them with `miniz_oxide`.
- Cost if wrong: a portal that changes its archive layout or drops range support breaks DGM1 and the nDSM there.

Source: spec §4.1 (Hamburg, Hesse); Hamburg reference §13 items 4 and 5; plan `2026-11-07-hamburg.md`; commits
5b8e8be, 158ef4c, ea54358, df0c6a3.
