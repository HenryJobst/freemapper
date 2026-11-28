# 0205. A GeoPackage reader without SQLite

Date: 2026-11-27
Status: Accepted

## Context

The land cover of Lantmäteriet (and later its buildings) is a GeoPackage, an SQLite file with WKB geometries. freemapper
is one executable without a C library; a SQLite binding would bring one in and a second licence to check.

## Decision

`freemapper_sources::gpkg` reads the file format itself, in safe Rust: the header, the pages, table b-trees (interior
and leaf), varints, record serial types and overflow chains, REAL columns that SQLite stores as integers, a rowid
alias column, and the GeoPackage geometry blob (big or little endian, optional envelope, empty geometries) with
ISO and EWKB WKB including Z and M. It reads tables only, no index, no SQL. Depth, page budget and WKB nesting are
bounded, so a damaged or hostile file is an error and not a hang or a panic. It was checked against a real land cover
file (168 areas) and is tested with synthetic files.

## Consequences

- A file with a feature the reader does not know (an encoding other than UTF-8, a table without rows in a b-tree it
  cannot walk) is refused with a message; nothing is guessed.
- The whole file is in memory; a paged reader is future work if a municipality needs it.

Source: `crates/freemapper-sources/src/gpkg.rs`.
