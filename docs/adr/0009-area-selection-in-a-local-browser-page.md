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
- The OSM tiles come through the local server, not straight from the browser (ADR 0088).
- There is no lock between a CLI `run` and a page in another terminal.

## Consequences

- Saving an existing project changes only its map area.
- Cost if wrong: a browser window opens unasked on an agent's desktop, and a save during a parallel CLI run can
  be overwritten.

Source: spec §3, §4.6; plan-mode notes "Gebietsauswahl"; plan `2026-11-07-m8-area-selector.md`; M8 rulings.

Update 2026-11-11 (ADR 0090, stage 1): the server runs on `freemapper-web` of freemapper-core. The token, the `Host` and `Origin` allow-list, the security headers and `open_browser` live there, not in `select.rs`. Two guards are stricter: a POST needs the page's `Origin` header and `Content-Type: application/json` (the page already sends both), and a request with `Sec-Fetch-Site: cross-site` gets 403. Every response carries `X-Content-Type-Options: nosniff`, and a request body may hold 1 MiB (a polygon of 5000 vertices takes about 150 KB). The tile proxy (ADR 0088) and the API routes stay in freemapper.

Update 2026-11-11: the gap "no lock between the command line and the page" is closed for runs and area saves by `.freemapper-run.lock` (ADR 0090, stage 3). Other writers, such as a text editor or OOM, are still not locked.
