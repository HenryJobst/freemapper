# 0118. Permit sources of region adapters

Date: 2026-11-13
Status: Accepted

Design: [`2026-11-13-permit-notes-berlin-design.md`](../superpowers/specs/2026-11-13-permit-notes-berlin-design.md).
Builds on [0117](0117-permit-notes-from-national-sources.md) (permit notes), [0114](0114-the-extra-layers-and-raster-templates-of-an-adapter.md)
(what an adapter fetches with code of its own) and [0106](0106-requests-go-to-public-addresses-and-declared-hosts.md)
(host rules).

## Context

For a map area in Berlin, step 1 of the permit notes could only say "Land Berlin" and that the Bezirk is not
determined. For a sprint map that is the gap that matters: the Bezirksamt is in charge of streets, parks and
monuments, and much of the land belongs to the Land. The Geoportal Berlin publishes all of it openly under
dl-de/zero-2-0, in five GeoServer WFS on `gdi.berlin.de`, a host Berlin's manifest already names. A live check on
2026-11-13 (24 requests) found the attributes and the axis rules: GeoJSON as `application/json`, a box in WGS 84
latitude first with the CRS at its end, coordinates back as longitude, latitude; blanks are a single space.

The engine must not name a region (ADR 0082, 0114). Berlin is a code adapter, so the hook is code too.

## Decision

- **A hook on the region adapter.** `RegionAdapter::permit_sources()` returns the services of a region that give
  findings for the permit notes; it is empty by default. Each `PermitSource` has a key, a publisher name and a
  short description in German and English, the credit and an optional note of the publisher, both verbatim, the
  keys of its layers, and the sections of the notes it fills. It fetches raw GeoJSON features, one list per layer,
  and turns stored features into typed findings.
- **Typed findings, the same for every region.** `District`, `GreenSpace` (name, district, dedication),
  `Playground`, `OwnerShare` (owner), `Monument` (kind, link) and `ProtectedObject` (Naturdenkmal, geschützter
  Landschaftsbestandteil or an area with special rules; name, citation), each with its geometry in WGS 84.
  The engine knows these kinds, not Berlin's attribute names.
- **Asked only on request and only on the adapter's ground.** `permits::fetch` asks the sources of an adapter
  when permit notes are asked for and the map polygon intersects the adapter's boundary (no margin), each through
  `adapters::asking`, so the adapter's own hosts hold and the policy of the national sources does not.
- **Stored like the national sources.** The features go to `data/permits/<adapter id>/<source>-<layer>.geojson`,
  their state into `sources.json` under `regional`. Offline, the stored layers are parsed again by the adapter.
  The layout keeps version 1: an older record without `regional` reads as before.
- **One source per service, one line per publisher.** A service that fails is "not reachable" in the table, and
  the sections it fills drop; the other services and the national sources stay. The table gives the services of
  one publisher one line, with the licence verbatim; the publisher's note follows the table.
- **Assembly on the map polygon.** Shares are area(map ∩ finding) / area(map) as in step 1. Points count as on the
  map when they lie inside the polygon. Green spaces are summed by name, district and dedication; playgrounds are
  counted. Owners get one line each: the sum of their parcels on the map. Monuments are counted per kind, with at
  most 20 links; rarer kinds keep theirs first, the rest are counted. Equal protected points are one entry with a
  count.
- **Hints that use the district.** When a source read the districts, the city-state hint of step 1 and the
  Ordnungsamt hint of the Gemeinde give way to one hint per district (Ordnungsamt and Straßen- und Grünflächenamt
  of the Bezirksamt), plus the lower monument authority for each district with a monument on the map and the
  lower nature conservation authority for each district with a Naturdenkmal or geschützter Landschaftsbestandteil.
  The owner hint names the share of each public owner and keeps the honest text for the rest. Without the
  districts the hints of step 1 come back; Hamburg keeps them.
- **Berlin brings five services.** `alkis_bezirke`, `gruenanlagen` (with the playgrounds), `liegenschaftsplan`,
  `denkmale` and `schutzgebiete` (layers `a_nd`, `e_fnd`, `g_glb`, `f_fspr`). The NSG, LSG, FFH, SPA and Naturpark
  layers of Berlin are left out: the BfN has them for every state, and each kind comes from one source. The parts of
  a park carry the park's name ("Viktoriapark, Rosengarten" counts as "Viktoriapark"). Links to monuments are kept
  only into the Denkmaldatenbank Berlin; they are text, never fetched.
- **`permits.json` stays schema 1.** The regional part is a new optional field `regional`; notes without it are
  written and read as before.

## Consequences

- Another state with open district, park or monument data can bring its sources the same way without a change to
  the engine; a manifest cannot declare them yet (no `[[info_layer]]`).
- The Bezirk hints depend on one service. When `alkis_bezirke` is down, the notes fall back to the city-state hint.
- The liegenschaftsplan names only public owners; the notes never show a parcel number or a person.
- `freemapper.toml`, `ATTRIBUTION.txt`, the map, the PDFs and the golden files do not change.
