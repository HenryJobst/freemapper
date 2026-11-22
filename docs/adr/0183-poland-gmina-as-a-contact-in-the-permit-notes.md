# ADR 0183: The gmina as a contact in the Polish permit notes

## Status

Accepted, 22 November 2026. Completes [ADR 0175](0175-poland-permit-notes.md) and
[ADR 0181](0181-poland-administrative-units-in-the-permit-notes.md), which both left open whether
the gmina is the body to ask.

## Context

The Polish notes name the gmina since ADR 0181, but no hint says for what it is the competent body.
The German notes name the Gemeinde, the Ordnungsamt and the lower authorities per finding. For
Poland the competence had not been read in the statutes. This ADR records what the official texts
say (consolidated texts from `api.sejm.gov.pl`, read on 22 November 2026) and adds hints only where
the text names the gmina or its head.

The head of a gmina is the wójt, in a town the burmistrz, in a large town the prezydent miasta.
Where the notes say "the gmina office" they mean the office of that body.

## What the statutes say

| Category | Competent body | Source | Verified |
|---|---|---|---|
| Natural monument (pomnik przyrody), ecological site (użytek ekologiczny), nature and landscape complex (zespół przyrodniczo-krajobrazowy), documentation site (stanowisko dokumentacyjne): who establishes it | The council of the gmina, by resolution | Ustawa o ochronie przyrody, art. 44 ust. 1 (Dz. U. 2026 poz. 1315) | yes |
| Same: what the resolution holds | Name, location, the supervisor (sprawujący nadzór), goals, and the bans chosen from art. 45 ust. 1 | art. 44 ust. 2 | yes |
| Same: draft resolutions | Need the agreement of the regional directorate (RDOŚ) | art. 44 ust. 3a | yes |
| Same: a permit or exception for a third party | The Act names none. The bans of art. 45 ust. 1 (11 kinds, none on entering or on events) apply only as far as the resolution chose them. The exceptions of art. 45 ust. 2 concern nature protection work, public investments, defence and rescue, each after agreement with the establishing body | art. 45 ust. 1 and 2 | yes |
| Same: who the supervisor is | The resolution names a person or body. That it is the wójt is not stated in the Act | art. 44 ust. 2 | **no** (depends on each resolution) |
| Protected landscape area | The voivodeship assembly (sejmik) designates it. Not the gmina | art. 23 ust. 2 | yes |
| Natura 2000, national and landscape parks, reserves | Not the gmina (RDOŚ, park administration); the Act articles for these were not read | art. 33 and others | **no**, not needed: the notes keep their existing hint |
| Public roads: manager of a gmina road | wójt (burmistrz, prezydent miasta) | Ustawa o drogach publicznych, art. 19 ust. 2 pkt 4 (Dz. U. 2025 poz. 889) | yes |
| Same: in a city with powiat rights | The prezydent miasta manages all public roads except motorways and expressways | art. 19 ust. 5 | yes |
| Same: occupying the road lane for purposes other than the road | Permit of the road manager, by decision | art. 40 ust. 1 | yes |
| Event that hinders traffic or uses a road in a special way | Permit, applied for at least 30 days ahead, from the traffic management body of the road | Prawo o ruchu drogowym, art. 65 and 65b ust. 1 (Dz. U. 2024 poz. 1251) | yes |
| Same: the traffic management body | starosta for powiat and gmina roads, prezydent miasta in cities with powiat rights, marshal of the voivodeship for voivodeship roads, GDDKiA for national roads. Not the wójt | art. 10 ust. 3 to 6 | yes |
| Same: forest on the route | Written consent of the forest owner belongs to the application | art. 65b ust. 4 pkt 6 | yes |
| Land of the gmina (park, square, school ground, forest of the gmina) | The wójt manages municipal property and represents the gmina | Ustawa o samorządzie gminnym, art. 30 ust. 2 pkt 3 and art. 31 (Dz. U. 2026 poz. 662) | yes |
| Whether a given piece of land is municipal | Not in the data of the notes | none | **no** |
| Event in a forest | Consent of the forest owner. Forest owned by a gmina has the gmina as owner. The starosta supervises forest management outside the State Forests | Ustawa o lasach, art. 29 ust. 4 and art. 5 ust. 1 pkt 2 (Dz. U. 2025 poz. 567) | yes (owner consent, supervision); that a given forest is the gmina's is not in the data |
| Monuments: park kulturowy | The council of the gmina creates it, after the opinion of the voivodeship conservator; the resolution holds bans and limits | Ustawa o ochronie zabytków, art. 16 ust. 1 and 2, art. 17 (Dz. U. 2024 poz. 1292) | yes |
| Monuments: municipal register | The wójt keeps the gminna ewidencja zabytków | art. 22 ust. 4 | yes |
| Monuments: building permits, work at a registered monument | The voivodeship conservator (not the gmina) | art. 36 ust. 1 pkt 1 and 2 | yes |
| Mass events | The wójt issues the permit (art. 24), but a mass sports event needs at least 1000 places for people, and general sport for recreation that is open and free in open terrain is excluded | Ustawa o bezpieczeństwie imprez masowych, art. 3 pkt 1 to 3, art. 24 (Dz. U. 2023 poz. 616) | yes; orienteering events fall outside it in practice, so no hint |
| Assemblies | Notification to the gmina. Not read, and a sports event is not an assembly of this kind | Prawo o zgromadzeniach | **no** |

## Decision

- **Three hints, Polish notes only,** in `regional_hints` (`notes.rs`), after the existing ones:
  - `permits-hint-gmina-nature-pl` when the map holds one of the four kinds the gmina council
    establishes (`NaturalMonumentPl`, `EcologicalSite`, `LandscapeComplex`, `DocumentationSite`).
    It says the council establishes them, the resolution names the supervisor and the bans, the Act
    itself names no permit for an event, and to ask the gmina office for the resolution. It names
    the gmina when the administrative units are read (the municipalities of the map, those that
    are more than an edge, else all), and reads on without a name when they are not. National
    parks, landscape parks, reserves, protected landscape areas and Natura 2000 get no such hint:
    the gmina does not decide there.
  - `permits-hint-gmina-roads-pl` when a gmina is known. It states art. 65 (permit, 30 days) and
    who issues it (starosta, mayor, marshal, GDDKiA), and that the gmina head manages gmina roads
    and decides on occupying the lane. It does not say that the wójt issues the event permit,
    because he does not.
  - `permits-hint-gmina-land-pl` when a gmina is known. It says that, if land belongs to the gmina,
    the wójt manages it and represents the gmina, and to ask for consent. It says the notes do not
    show who owns the land.
- **Not added.** No hint for monuments (the Polish notes read no monument source; a register of
  the national heritage institute would be one), none for mass events, none for assemblies, no
  road category from OSM (the Polish classes are not in the tags reliably).
- **Facts.** `HintFacts` gets `gmina_nature_forms` and `municipalities` (`regional.rs`). The
  municipalities come from the existing `admin_units` section; no new request, no new host, the
  adapter revision stays 5. The JSON of the notes holds hints with their args as before; old notes
  files stay valid.
- **Texts.** The three ids in all 21 catalogs of `freemapper-cli`: English, German and Polish by
  hand, the other 18 machine-made from the English, unreviewed, as in ADR 0175 and 0181. Each
  catalog test (same ids, same variables, every id used) covers them; a test renders each hint in
  all 21 languages.
- **Tests.** Offline: the four kinds, a national park, two gminas with one at the edge, no
  administrative units, a German map area, and the rendering of the hints.

## Not decided here

- **Whether the supervisor of a natural monument is usually the wójt.** The Act leaves it to the
  resolution. The hint therefore says "ask the gmina office for the resolution and the supervisor"
  and does not say that the gmina office decides.
- **Which gmina holds an object,** when a map area spans several: the hint lists all gminas of the
  map, not the one the object lies in. The geometry of the object could pick it; the effort was not
  worth it for a hint.
- **Roads by category.** A road class from BDOT10k or OSM would let the notes name the powiat or
  the voivodeship board for a road that is not a gmina road.
- **Monuments,** see above. The Polish legal wording is a draft and needs a Polish reader before
  release.

Source: this ADR; `docs/regions/poland-reference.md`.
