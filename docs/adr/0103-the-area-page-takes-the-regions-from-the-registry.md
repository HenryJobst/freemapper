# 0103. The area page and the texts take the regions from the registry

Date: 2026-11-12
Status: Accepted

Amends [0096](0096-the-area-page-on-the-suite-frame.md) and [0009](0009-area-selection-in-a-local-browser-page.md).
Part of stage C of [0082](0082-region-adapters-trait-and-manifests.md) (task C13).

## Context

The supported states were written out in five places: the error for an area outside them (in the engine and in both
catalogs of the command line), the MCP instructions, the page's own "outside" text, the ten aerial photo layers in
`web/app.js`, and the image hosts of the page's content security policy. A new state had to be added to each.

## Decision

- A region names its aerial photo for the map of the page: `RegionInfo.preview` with the WMS address, the layer,
  the image format, further query parameters, the credit and the short name of the agency. It is not the service
  the orthophoto is fetched from; most states have a mosaic for viewing with its own address.
- `GET api/adapters` gives the page `[{ id, names: { en, de }, preview }]` in the order of the registry. The page
  builds one WMS layer per service, names it with one text (`page-layer-aerial-of`, with the names of the regions
  that share the service and the agency) in the language `api/texts` chose, and replaces `{year}` in a credit by
  the current year.
- The content security policy lists the origins of the previews and nothing else for images. One function
  (`preview_origin`) decides for the policy and for the endpoint: only an `https` address with a plain host name
  counts (no port, no user, ASCII letters, digits, dots and hyphens). A preview without such a host is sent as
  `null`, so the page never offers a layer the policy would block.
- The page treats the answer as data it does not trust: names and credits reach Leaflet as escaped text, the
  further parameters are plain values and never an option of Leaflet, a region with an incomplete preview is listed
  without it, and a failing `api/adapters` leaves the page working with OpenStreetMap alone and a message.
- `error-project-outside-coverage`, `page-outside` and `mcp-instructions` take the list of the states as a value,
  in the language of the run or of the page. The English error of the engine joins the English names of the
  registry.

## Consequences

- For the eleven states the policy, the layer names and the three texts read as before, byte for byte where a test
  can compare them.
- A state added to the registry appears in the layer list, the policy and the texts without a change to the page.
- A manifest of stage D can name a preview. The filter of the policy and the escaping are what stands between such
  a manifest and the page; they have tests of their own.
- `scripts/browser-smoke.md` has a step for the layer list, because a typing error in a preview address shows
  only in a browser.
