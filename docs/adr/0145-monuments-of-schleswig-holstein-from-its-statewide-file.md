# 0145. Monuments of Schleswig-Holstein from its statewide file

Date: 2026-11-15
Status: Accepted

Research: [`denkmale-2026-11-15.md`](../research/wald-und-sprint-behoerden/denkmale-2026-11-15.md). Builds on
[0132](0132-statewide-permit-files-in-the-cache.md) (statewide files in the cache) and
[0144](0144-monuments-from-more-states.md) (monuments, gaps).

## Context

The Landesamt für Denkmalpflege Schleswig-Holstein has no map service for its list of monuments. The open data
portal publishes it as one GeoJSON file for the whole state, a new one each week (about 52 MB, EPSG:25832,
MultiPolygons, `LayerName` such as Baudenkmal or Gründenkmal), under CC BY 4.0 with the attribution "Landesamt für
Denkmalpflege Schleswig-Holstein". `…/collection/geodaten-denkmalliste-sh/aktuell.geojson` redirects on the same
host to the file of the week, which carries `Last-Modified`. The archaeological monuments are not in it (their
office allows publication only with written consent), and Lübeck keeps its own list.

## Decision

- A source `denkmale` (`ldsh_permits`) keeps the file in the shared cache under `permits`, as the statewide zips of
  ADR 0132 are: asked by its date, downloaded only when newer, the cached copy used when the server cannot be asked.
  The cache of ADR 0132 takes any file now (`shape_permits::current_file`, with its extension and a check).
- The file is read with the geometries left raw (`serde_json` `raw_value`); only a feature whose box meets the map box
  is parsed, turned into WGS 84 and kept with its kind.
- The source names its gaps (no archaeological monuments, no Lübeck), so the notes say so and the hint on the lower
  monument authority comes without a finding too.
- The host `opendata.schleswig-holstein.de` joins Schleswig-Holstein's manifest.

## Consequences

- The first permit notes in Schleswig-Holstein download about 52 MB, the later ones only when a new week's file is out.
- Eleven states now give monuments; Lower Saxony, Bavaria, Mecklenburg-Vorpommern and Saxony-Anhalt wait.
