# ADR 0184: The register of monuments of the NID in the Polish permit notes

## Status

Accepted, 22 November 2026. Fills the gap that [ADR 0183](0183-poland-gmina-as-a-contact-in-the-permit-notes.md)
left under "Not added" (no monument source for Poland).

## Context

The Polish notes read the GDOŚ (nature), the PRG (administrative units and forest districts) but no
monument source. The Narodowy Instytut Dziedzictwa (NID) runs a public WMS of the register of
immovable monuments at `usluga.zabytek.gov.pl`. It gives no feature download, only
`GetFeatureInfo`.

## What the service does

Checked live on 22 November 2026:

- `GetFeatureInfo` with the whole map box as one pixel (`WIDTH=1&HEIGHT=1&I=0&J=0`) returns every
  monument whose area meets the box, as a GML `FeatureCollection` with `Layer` elements and
  `Attribute Name=...` children (`INSPIREID`, `SITENAME`, `NATIONALMONUMENTSRECORD`, ...). EPSG:4326
  in WMS 1.3.0 has the latitude first.
- Słubice (52.34 to 52.36, 14.55 to 14.57): 17 entries, 11 distinct INSPIREIDs. Kraków (50.055 to
  50.073, 19.93 to 19.96): 2108 entries in 1.5 s, 39 INSPIREIDs among them repeated.
- An empty box answers a self-closing `FeatureCollection`. An error answers HTTP 200 with a
  `ServiceExceptionReport`.
- The answer has no geometry.

## Decision

- **New source `nid-zabytki`** in the Polish adapter (host `usluga.zabytek.gov.pl`, revision 6),
  one request per map area. Entries are merged by INSPIREID. An answer of 5000 entries or more is a
  failure (the box is too large to be complete). An exception report or a non-XML answer is a
  failure too, so a failing source is a warning and the section is left out, as with the other
  sources.
- **Position.** The register gives none, so each monument gets the rectangle of the map box as its
  geometry and counts fully. The texts therefore say "in the rectangle around the map area", never
  "in the map area".
- **Section** `PermitSection::MonumentRegister`, finding `FindingKind::RegisterMonument { name }`.
  The notes give the total and the names grouped with their counts, at most 10 names, the rest
  counted. Register names are often generic ("willa", "oficyna", "otoczenie"); grouping keeps the
  list short.
- **Hint** `permits-hint-monument-register-pl` when at least one monument is found. Source: Ustawa o
  ochronie zabytków, art. 36 ust. 1 pkt 1, 2 and 11 (Dz. U. 2024 poz. 1292): work at a registered
  monument, building work in its surroundings and other actions that could change its substance or
  appearance need the permit of the voivodeship conservator. The Act names no run. The hint says so.
- **Forest hint.** `permits-hint-forest-owner-pl` now says the starosta supervises forest that is
  not owned by the State Treasury (Ustawa o lasach, art. 5 ust. 1 pkt 2, Dz. U. 2025 poz. 567).
- **Credit** "© Narodowy Instytut Dziedzictwa (CC BY 4.0)". No prescribed wording was found; a
  person should confirm it.
- **Texts** in all 21 catalogs; English, German and Polish by hand, the other 18 machine-made and
  unreviewed.

## Not decided here

- **Archaeological sites and the Pomniki historii** are not in this layer; the source entry says so.
- **Positions.** A WMS rendering of the layer or a second query on a smaller box could narrow where
  the monuments lie. Not worth it for a hint.
- **The Polish legal wording** needs a Polish reader before release.

## Addendum, 23 November 2026: notification for building work in the surroundings

Dz. U. 2025 poz. 1673 (in force since 3 June 2026) repealed art. 36 ust. 1 pkt 2, 7, 9, 10 and 11 of the Act and
added art. 36aa: building work in the surroundings of a monument, a change of use, devices and signs on a monument
and other actions that could change its substance or appearance now need a notification to the voivodeship
conservator, who can object within 60 days (ust. 3) and then require a permit (ust. 10); the person may also apply
for a permit instead (ust. 13). Checked in the text of the act on the Sejm ELI API. The hint `permits-hint-monument-register-pl`
keeps the permit for conservation, restoration and building work at a registered monument (pkt 1) and names the
notification for the rest, with its date and the 60 days. The wording "need the permit ... building work in its
surroundings and other actions" of the decision above no longer holds. Not checked: how the conservators handle
notifications in practice. The texts changed in all 21 catalogs; English, German and Polish by hand, the other 18
machine-made and unreviewed as before. The adapter revision stays 8, because no source, host or rule changed.
