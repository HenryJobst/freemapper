# 0088. OSM base tiles through the local server

Date: 2026-11-10
Status: Accepted

Amends [0009](0009-area-selection-in-a-local-browser-page.md).

## Context

GitHub issue 11 (Ubuntu, Firefox 157): after `freemapper select` the page showed an "access blocked" 403 page from
OpenStreetMap. The page loaded the base map straight from `tile.openstreetmap.org`. The local server sends
`Referrer-Policy: no-referrer`, so that the secret path stays out of other servers' logs. The tile usage policy
(operations.osmfoundation.org/policies/tiles/) says "Browsers are expected to send a valid `Referer` header" and asks
for "a clear, unique User-Agent string that names your app". The wiki page on blocked tiles names a missing `Referer`
as the cause of the 403 tile and says the same about the `User-Agent`. Without a `Referer`, the browser's own
`User-Agent` does not help.

Setting the `Referrer-Policy` to `strict-origin-when-cross-origin` would send only `http://127.0.0.1:<port>/`, which
hides the token but is no identification of freemapper and might still be blocked as a local address.

## Decision

The local server serves the OSM layer itself: `GET /<token>/tiles/osm/{z}/{x}/{y}.png`.

- The server fetches `https://tile.openstreetmap.org/{z}/{x}/{y}.png` with the fixed header
  `User-Agent: freemapper/<version> (+https://github.com/HenryJobst/freemapper)`. No address, name or other personal
  data goes into any request. The same string is already used for Nominatim.
- The address is checked before anything leaves the machine: zoom 0 to 19, column and row digits only and below
  2^zoom, `.png` suffix. The upstream URL is built from the parsed numbers, never from the request text. Anything else
  is 400.
- No redirects are followed. Only `image/png` with status 200 passes, at most 1 MiB. Any other answer is 502 with the
  upstream status in the message. Timeout 20 s.
- At most 2 requests at once, the number the policy asks for.
- Tiles stay in memory for the `max-age` of the answer (7 days without one, as the policy asks), at most 1024 tiles or
  32 MB, oldest first. The browser gets `Cache-Control: private, max-age=<rest>`. Nothing is written to disk, and
  nothing is fetched ahead of what the user views.
- The route sits behind the token, the Host check and the Origin check like every other route. The middleware keeps
  `no-store` for all other responses.
- `tile.openstreetmap.org` leaves the CSP `img-src`, `app.js` loads `tiles/osm/…`. The attribution text stays.
- The aerial WMS layers of the states stay direct. They are state services without a `Referer` rule, and they worked
  before.

## Consequences

- The base map needs internet access on the machine that runs `freemapper select`, as before for the browser.
- OSM sees one client per program run instead of one per browser; the `User-Agent` names freemapper.
- If OSM blocks freemapper's `User-Agent`, the page shows a gray base map and the 502 message in the network log.
- Cost if wrong: a gray base map until the next release; the drawing and the aerial layers still work.

Source: GitHub issue 11; OSM tile usage policy and wiki "Blocked tiles", read 2026-11-10; ADR 0009 (the server rules kept here).
