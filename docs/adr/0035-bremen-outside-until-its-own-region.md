# 0035. Bremen stays outside until it gets its own region

Date: 2026-11-08
Status: Accepted

## Context

Bremen and Bremerhaven are enclaves of Lower Saxony, but a state of their own with their own survey office and data.
LGLN clips its WMS exactly at the Bremen border and its ALKIS stops there, although the raster tiles reach across. The
Geofabrik extract `niedersachsen-latest` includes Bremen; `bremen-latest` is 21.3 MB on its own.

## Decision

The Lower Saxony boundary asset keeps Bremen and Bremerhaven as holes, and a test checks them. A point in Bremen lies in
no region. An area whose centre is in Bremen is refused with "the map area lies outside the supported states". An area
whose centre lies in Lower Saxony but reaches into Bremen is fetched from LGLN and gets the beyond-coverage warning
(ADR 0034), not the message for a second state. Its OSM data come from the Lower Saxony extract, which contains Bremen.
Bremen gets its own region later, with its own provider.

## Consequences

- No Bremen data are presented as Lower Saxony data.
- Maps in Bremen are not possible yet.
- At the Bremen border the orthophoto is white and ALKIS empty on the Bremen side.
- Cost if wrong: Bremen users wait for a later adapter.

Source: Lower Saxony reference §4.3, §11, §13 item 7; plan `docs/superpowers/plans/2026-11-08-niedersachsen.md`
(Rulings); commits 8e54fcf, 489a152 and a6a3f18 on branch `niedersachsen`.
