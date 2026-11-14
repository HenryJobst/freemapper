# 0130. Permit sources as tables of their layers

Date: 2026-11-14
Status: Accepted

Research: [`nachrecherche-2026-11-14.md`](../research/wald-und-sprint-behoerden/nachrecherche-2026-11-14.md).
Builds on [0120](0120-permit-sources-of-region-adapters.md) and [0129](0129-permit-notes-for-hamburg-water-protection-and-forest-functions.md).

## Context

After ADR 0129 the notes know water protection zones and forest functions for every region. Several states publish
them as WFS 2.0 that answer GeoJSON: Berlin (water protection), Brandenburg (Naturwald, Totalreservat, closed
areas, Erholungswald in the IWFK service of the LFB, dl-de/by-2-0 per GovData), North Rhine-Westphalia (LINFOS
Naturwaldzellen and Wildnisentwicklungsgebiete, dl-de/by-2-0 "Land NRW"), Saarland (water protection, Naturwaldzellen,
Urwald, the Reviere of the state forest and the listed buildings, CC BY 4.0 "© GDI-SL") and Hesse (water protection
of the HLNUG, CC BY 4.0). Each differs only in its URL, its type names and the attribute names. A code module per
service, as for Berlin and Hamburg, would repeat the same fetch and the same parsing.

Bremen's water protection service and the LUBW's in Baden-Württemberg give no licence in their capabilities; the
LUBW's only from a search hit. Mecklenburg-Vorpommern's forest WFS calls itself password-protected and available on
request only, yet answered without credentials; its WMS has no open licence.

## Decision

- **A table source** (`wfs_permits::TableSource`): a service, its GeoJSON output format and a table of layers, each
  with its key, its type and a `Reading` that names the attributes of the finding (water zone: name, zone,
  ordinance; forest function: kind and name; forest unit: administration, prefix and the attribute of the office;
  monument: kind and the attribute of a link into the state's list). The fetch keeps those attributes and nothing
  else.
- A zone written as "I" or "IIIA" alone becomes "Schutzzone I" for the hints; a text that says zone stays.
- The new sources hang off the states' adapters: Berlin (`wsg`), Brandenburg (`waldfunktionen`), NRW (`linfos`),
  Saarland (`forst`, `schutzgebiete`, `denkmale`) and Hesse (`wasserschutzgebiete`). `geodienste-umwelt.hessen.de`
  joins Hesse's manifest hosts; the others were there already.
- **Only with a licence the service or its catalogue states.** Bremen, Baden-Württemberg and Mecklenburg-Vorpommern
  stay out until one is stated; for MV the maintainer asks the Landesforst.

## Consequences

- A further state with a GeoJSON WFS is a table of a few lines and a fixture test.
- In Saarland the notes name a Revier only in the state forest; private and municipal forest has none in the data.
- Every source is one more request per layer; a map run with permit notes in Brandenburg asks five more layers.
