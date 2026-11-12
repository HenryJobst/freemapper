# 0111. Saarland, the first state on the adapter scheme, and what it needed

Date: 2026-11-12
Status: Accepted

Gate E5 of [0082](0082-region-adapters-trait-and-manifests.md): before the contributor guide is written, one new
state is added on the adapter scheme, and what it needs beyond its own folder is written down. The user chose
Saarland on 2026-11-12. Research: [`docs/regions/saarland-reference.md`](../regions/saarland-reference.md); plan:
`docs/superpowers/plans/2026-11-12-saarland.md`.

## Context

The design of 0082 hoped that a new state is a manifest folder, a backend only if its heights need one, and one
line that registers it. Saarland was the test: a small state whose agency (LVGL) lists standard services.

## Decision

- Saarland is a built-in state: manifest folder `crates/adapters/de/manifests/saarland/`, raster backend `lvgl`
  (`crates/adapters/de/src/lvgl.rs`), id `saarland`, provider `lvgl`, EPSG:25832.
- Only what the catalogue of the LVGL lists under dl-de/by-2-0 is used: the free WMS for the orthophoto of 2025,
  the district archives of the open-data share for DGM1 and DOM1 of 2025, and the WFS "ALKIS Vereinfacht". The
  two INSPIRE WCS services are not asked, because their own capabilities still say that use beyond viewing needs
  a contract. The products the LVGL charges for (DGM1 of 2006, bDOM20, DOM025, older orthophotos) are left out;
  the user wants a concrete request first, and then an opt-in of its own.
- Credit: `© GeoBasis DE/LVGL-SL (<year>), dl-de/by-2-0, Daten geändert`, with the address of the licence text
  in the notes of the project.
- Heights: six archives per model, one per district, with 1 km tiles. The backend reads the directory of an
  archive and then the tiles of the area by range requests, as Hamburg and Hesse do. A tile on a district border
  is in several archives, the same in each (checked by CRC-32), so the first that holds it is taken.
- The WFS reader stores attribute names written all in upper case under the names of the simplified schema and
  reads the geometry from `SHAPE`. The manifest has a new optional key `[alkis] sorted = false` for a WFS that
  refuses `SORTBY`; its features are sorted by `oid` after paging. No released freemapper reads manifests yet,
  so the key costs no compatibility; a key added after 0.14.0 will need its `min_freemapper`.
- The golden net stays on the eleven states that were code before. Saarland is covered by unit tests with
  synthetic archives and recorded WFS answers, and by `adapters check --live`.

## Consequences

- The hope was a folder and one registration line. Saarland needed about twenty places in the crate (the
  enums `State` and `Agency`), the WFS reader, the manifest key, more than twenty test assertions that said
  "eleven", and the golden harness (`regions::LEGACY`, the filter of the generated boxes). The list is in
  [`saarland-reference.md`](../regions/saarland-reference.md). A table of states read from the manifests would
  bring it down; not done here.
- For the contributor guide: a state with standard services is a manifest. A state whose heights come as
  archives or tiles needs a backend in Rust, and a folder adapter cannot bring one.
- For the format: `params` is a table and cannot repeat a key, which the WCS of Saarland would have needed
  (`RESOLUTION=x(1)&RESOLUTION=y(1)`). A key for the resolution of a WCS is the likely next addition.
- `adapters check --live` was the acceptance test: Saarbrücken, St. Wendel and Perl passed at the first run.
- Saarland is unreleased; README and AGENTS.md say "from the next version".
