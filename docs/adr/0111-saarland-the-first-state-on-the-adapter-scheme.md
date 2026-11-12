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
  the district archives of the open-data share for DGM1 and DOM1 of 2025, and the WFS "ALKIS Vereinfacht" of the
  geoportal. The two INSPIRE WCS services are not asked, because their own capabilities still say that use beyond
  viewing needs a contract, although their catalogue records say dl-de/by-2-0. The products the LVGL charges for
  (bDOM20, DOM025, older orthophotos) are left out; the user wants a concrete request first, and then an opt-in
  of its own.
- Credit: `© GeoBasis DE/LVGL-SL <year>, dl-de/by-2-0, Daten geändert`.
- Heights: six archives per model, one per district, 0.6 to 1.3 GB each, with 1 km GeoTIFF tiles. The backend
  reads the directory of an archive (about 35 KB) and then the tiles of the area by range requests, as Hamburg
  and Hesse do. A tile is looked for district by district; a directory is read only when the districts before it
  do not hold the tile. Tiles outside the state are in no archive and stay empty.
- Two additions for the cadastre, which comes from an ArcGIS server behind the geoportal: the WFS reader stores
  attribute names written all in upper case under the names of the simplified schema and reads the geometry
  from `SHAPE`, and the manifest has a new optional key `[alkis] sorted = false` for a WFS that answers `SORTBY`
  with an exception.
- The golden net stays on the eleven states that were code before. Saarland has no recording there. It is
  covered by unit tests with synthetic archives and recorded WFS answers, and by `adapters check --live`.

## What Saarland needed, against the hope

Inside `crates/adapters/de`, as hoped: the manifest folder with its boundary, and the backend.

Inside `crates/adapters/de`, more than one line: the state and its agency are enums, so a new state adds an arm
or a row in about twenty places (`State::ALL`, `id`, `object_id_prefix`, `agency`, `crs`, the three tables of
`Agency`, the manifest and boundary tables, `backend_name`, the ALKIS constants, the credit, the estimate, the
backend table).

Outside `crates/adapters`:

- `freemapper-sources`: the WFS reader (upper-case names, `SHAPE`, unsorted paging).
- `freemapper-manifest`: the key `sorted`.
- Tests that count states or list them: the registry order, the list of states in messages, `adapters list`,
  the area page's adapters and its content security policy, the boundary files. More than twenty assertions in
  nine files said "eleven" or counted on it.
- The id tables of the engine's tests (`regions::ALL`, a point inside each region).
- One golden file, because the debug text of a WFS constant gained a field.
- The documents every state gets: manual, README, examples, changelog, AGENTS.md.

## Consequences

- For the contributor guide (stage F): a state with standard services is a manifest, and Saxony-Anhalt stays
  the only built-in example of one. Saarland shows the usual case better: standard on paper, with one odd
  thing per service. The guide should say plainly that a state whose heights come as archives or tiles needs
  a backend in Rust, and that a folder adapter cannot bring one.
- For the format: `params` is a table and cannot repeat a key, which the WCS of Saarland would have needed
  (`RESOLUTION=x(1)&RESOLUTION=y(1)`). It was not needed here because that WCS is not used. A key for the
  resolution of a WCS is the likely next addition.
- For the crate: the enums `State` and `Agency` are what makes a new state touch about twenty places. A table of
  states, read from the manifests, would bring that down to the registration line the design hoped for. Not
  done here.
- For the tests: an assertion that says `11` costs an edit per state. The ones touched now say `12`; the
  cheaper form is the length of `State::ALL`.
- `adapters check --live` was the acceptance test: three places (Saarbrücken, St. Wendel, Perl at the border)
  passed at the first run, with every request on the two declared hosts.
- Saarland is on the branch of the adapter work and unreleased. README and AGENTS.md say "from the next
  version".
