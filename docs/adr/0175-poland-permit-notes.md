# ADR 0175: Permit notes in Poland

## Status

Accepted, 22 November 2026. Amends a paragraph of [ADR 0171](0171-poland-cadastre-egib.md).

## Context

ADR 0171 left the permit notes refused outside Germany, because their sources (VG250, BfN) and their
law are German. Poland has open services for the same questions: GDOŚ serves the nature protection
layers, and the national boundary register (PRG) the forest districts of the State Forests.

## Decision

- **Two adapter sources.** `crates/adapters/pl/src/permits.rs` holds `gdos` (11 layers, one request
  per layer with paging, longitude-first GeoJSON via `srsName=EPSG:4326`) and `prg` (forest districts
  as GML 3.2, the directorate name looked up and stored in each district). `Polska::permit_sources()`
  returns both. The adapter revision is 4; `sdi.gdos.gov.pl` is the one new host.
- **New kinds.** `ProtectedObjectKind` gets eleven Polish kinds. `NaturalMonumentPl` is separate
  because the text of `NaturalMonument` carries the German "Naturdenkmal". `needs_nature_authority`
  is true for all but `SpecialRules`.
- **A `national` flag.** `Inputs.national` and `RawInputs.national` say whether VG250 and BfN apply.
  The `sources.json` record gets `national`, default true and skipped when true, so German output is
  unchanged. For Poland only `osm` is a source; the admin, BfN, OSM-protected and roads sections
  and the BfN merge note are not rendered, and `regional_hints()` names no Kreis, Gemeinde,
  Grundbuchamt or Ordnungsamt.
- **Non-German regions** get notes when some adapter with permit sources covers the area; otherwise
  the refusal stays. Offline, only `osm` is reported missing for Poland.
- **Hints.** `permits-hint-protected-areas-pl` and `permits-hint-forest-owner-pl`, plus
  `permits-disclaimer-regional` and `permits-osm-enclosing-gap-short`. All 21 languages carry them;
  English, German and Polish by hand, 18 by machine from the English.
- **Fixtures** are real attributes with geometry replaced by the bounding rectangle of the feature.

## Not decided here

- **Gmina, powiat, voivodeship** as a districts section: it assumes Bezirke. Done in
  [ADR 0181](0181-poland-administrative-units-in-the-permit-notes.md) as a section of its own.
- **Who decides for small objects.** Natural monuments and ecological sites are decided at municipal
  level in Poland; the hint does not name the gmina because that was not verified.
- **Private forest** is not in the PRG layer; the hint and the source `gaps` text say so.
- **The Polish legal wording** is a draft and needs a Polish reader before release. The 18 machine
  translations are unreviewed. The district-to-directorate code rule was checked on Warsaw only.
