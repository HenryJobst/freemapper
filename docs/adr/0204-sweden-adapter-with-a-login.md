# 0204. Sweden: one national adapter with the terrain model, laser data and land cover of Lantmäteriet

Date: 2026-11-27
Status: Accepted

## Context

Lantmäteriet publishes the terrain model (Markhöjdmodell, 1 m), the laser point clouds (Laserdata Skog) and the
land cover (Marktäcke) as open data under CC BY 4.0, but the downloads (`dl1.lantmateriet.se`) want HTTP Basic
authentication of a Geotorget account that has ordered each product. The catalogues (STAC, `api.lantmateriet.se`) are
open. The research is `docs/regions/sweden-reference.md` (PR #201); the design is
`docs/superpowers/specs/2026-11-27-sweden-adapter-design.md`. Sweden's grid SWEREF99 TM (EPSG:3006) has the same
numbers as EPSG:25833, so nothing is transformed.

## Decision

- One adapter `sweden`, provider `lantmateriet`, in the crate `freemapper-adapters-se`. Every address and host is its
  manifest (`manifests/sweden/manifest.toml`, ADR 0189); the code reads what no table says: the STAC reader, the terrain
  backend (range reads of the Deflate blocks of the 10 km COG, ADR 0141), the COPC point-cloud backend and the land
  cover source.
- The manifest format grew by what Sweden needs: `[terrain]` without `[orthophoto]` (with `endpoints`), `[point_cloud]
  kind = "stac-copc"`, `[alkis] kind = "stac-gpkg"` and `[auth]`.
- **The login** is `[auth]`: one host, two variables `FREEMAPPER_…`, and the products to order. The program reads the
  variables in one place (`fetch::process_env`); the `Authorization` header goes to that host only, never along a
  redirect. The password type has no `Debug` and no `Display`. A fetch in Sweden without the variables stops before the
  first request and names both variables and the three products. **Only the adapters built into the program may
  declare `[auth]`**: an adapter from a folder that did could send the password to a host it names.
- The land cover becomes the land use of the map through a table of classes (`rules/marktacke.toml`); "open land" and
  unknown classes are left out and named in a progress line. The first stage had no buildings at first (OpenStreetMap
  buildings came with ADR 0206); Byggnad and the orthophoto wait for the owner's application (no orthophoto:
  ADR 0203).
- The point-cloud grid is 2.5 km. `PointCloudInfo::grid_km` is read by no engine code, so the backend names its tiles
  from the catalogue and sets the field to 3; no API and no German adapter changes.
- The boundary is Natural Earth 1:10m (public domain), accurate to a few hundred metres; the Lantmäteriet boundary
  product needs an order of its own.
- Sweden is `Country::Se`; the permit notes, which are German, are not made there.

## Consequences

- Download limits are unpublished. The range reads of the terrain model use the retry policy; the file downloads (laser
  data, land cover) are not asked again. A 429 ends them with a sentence about the account limit and stops the laser
  tiles after the first refusal; an account without an order gets an error that names the product (English text).
- A GeoPackage of a very large municipality is read whole into memory (cap 1 GiB).
- No Lantmäteriet data is in the repository; tests use synthetic tiles, files and answers.

Source: issue #273 (manifest rule), the design above.
