# ADR 0193: Every gmina of the map area in the Polish permit hints

## Status

Accepted, 24 November 2026. Extends [ADR 0183](0183-poland-gmina-as-a-contact-in-the-permit-notes.md),
which left open "which gmina holds an object, when a map area spans several".

## Context

ADR 0183 listed "several gminas named" as a gap. Reading the code showed that the data side was
already complete: the PRG layer `A03_Granice_gmin` returns every gmina whose polygon meets the map
box, `admin_units` sums the share of each, and `regional_hints` joined all gminas that are more than
an edge (below 0.5 % of the map area) into one `gmina` arg, by share, else by name. What was missing
was in the hint:

- The share of each gmina was not in the hint, though the notes hold it.
- The texts spoke of "the gmina" in the singular, with a list of names in brackets. For two gminas
  they said "the wójt ... of the gmina (A, B)" and "the manager decides", as if one body ruled.
- Nothing fixed, or tested, the order and the cases of 0, 1, 2 and many gminas.

Live probe, 24 November 2026, host `mapy.geoportal.gov.pl` only (the host the adapter asks anyway),
the layer `A03_Granice_gmin` with the box in EPSG:4326:

| Box (lat, lon) | Gminas returned |
|---|---|
| 52.07 to 52.09, 16.64 to 16.66 (Kościan) | `3011032` Kościan (rural), `3011011` Kościan (urban) |
| 50.00 to 50.10, 19.80 to 20.10 (west of Kraków) | `1261011` Kraków, `1219053` Wieliczka, `1206172` Zielonki, `1206072` Liszki, `1206162` Zabierzów, `1206152` Wielka Wieś |

The service gives every gmina in the box; the adapter needed no change. `COUNT=100` is far above
what a map area meets.

## Decision

- **All gminas, no cap.** A map area lies in few gminas, and a cut-off would hide a body that is
  competent for part of the area. The monument sections cap their names (ADR 0184); that rule does
  not carry over. A cap of ten with a count of the rest was considered and dropped for that reason.
- **Order.** Share descending; equal shares by the entry (name, then TERYT code). Several parts of
  an adapter that name the same gmina give it once, with the share of the first part.
- **Edge.** As before: a gmina below 0.5 % is left out when another gmina is more than an edge;
  where all lie at the edge, all stay. This is not new; it is now tested (shares 0.4 and 0.2).
- **Args.** `HintFacts.municipalities` holds name and share. The three hints (`permits-hint-gmina-*`)
  get `gmina` (names, ", "), `shares` (percent with one decimal, ",") and `count`. Older notes files
  with `gmina` only stay valid: the program derives `count` from the names and shows no shares.
- **Rendering.** `freemapper-cli` writes each name with its share (`gmina A (70 %)`) and passes
  `count` as a number, so the catalogs choose by the plural category of the language (`[one]`,
  `*[other]`). Only the phrase that names the body changed, in all 21 catalogs: with several
  gminas the offices (plural) are asked, each for the objects in its area; the wójt, burmistrz or
  prezydent miasta of each gmina manages its roads and decides on occupying the lane; the same head
  of each gmina manages municipal land. English, German and Polish by hand, the other 18 machine-made
  from the English, unreviewed, as before. No legal claim is new: the statutes of ADR 0183 name the
  body per gmina, and the plural says it for each.
- **Adapter revision 11.** The adapter code is unchanged; the revision marks the new hint wording
  and the live probe.

## Not decided here

- **Which gmina holds an object.** The geometry of a protected object could pick its gmina. The
  hint says "each for the objects in its area" and leaves the choice to the reader.
- **Powiat.** The starosta is named by the roads hint in general terms; several powiats are listed in
  the administrative units section as before and get no own hint.
- **Plural of other languages by a reader.** No native-speaker review, as in ADR 0183.

Source: this ADR; `docs/regions/poland-reference.md`.
