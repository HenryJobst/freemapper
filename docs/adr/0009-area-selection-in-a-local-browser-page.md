# 0009. Area selection in a local browser page

Date: 2026-11-06
Status: Accepted

## Context

Users must draw or pick the map area. A native desktop GUI is a non-goal of the MVP. The same area choice must be
open to agents through MCP.

## Decision

`freemapper select` starts a local web server in the binary and opens a Leaflet page with OSM tiles and the
orthophoto. The user draws a rectangle or polygon or picks an OSM polygon, sees the tile grid and the download
estimate, and saves the area to `freemapper.toml`. Rulings from M8 on 2026-11-07:

- The server listens on 127.0.0.1 only, with a random 128-bit token in every path, a Host check against DNS
  rebinding, a CSP, `X-Frame-Options: DENY` and `no-store`.
- The MCP tool `open_area_selector` opens the browser by default and returns the URL too.
- There is no lock between a CLI `run` and a page in another terminal.

## Consequences

- Saving an existing project changes only its map area.
- Cost if wrong: a browser window opens unasked on an agent's desktop, and a save during a parallel CLI run can
  be overwritten.

Source: spec §3, §4.6; plan-mode notes "Gebietsauswahl"; plan `2026-11-07-m8-area-selector.md`; M8 rulings.
