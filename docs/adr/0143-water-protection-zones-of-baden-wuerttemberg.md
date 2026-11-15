# 0143. Water protection zones of Baden-Württemberg, with the authority in charge

Date: 2026-11-15
Status: Accepted

Builds on [0129](0129-permit-notes-for-hamburg-water-protection-and-forest-functions.md) (water protection in the
notes) and [0142](0142-water-protection-areas-of-the-bfg.md) (the BfG for the other states).

## Context

Baden-Württemberg was the last state without water protection areas in the permit notes: the BfG has no data from
there, and ADR 0130 left the LUBW's WFS out because the service names no licence. The catalogue entry of the
dataset "Wasserschutzgebietszone" (`19db48dd-576f-498c-bafa-083b200baad5`) points to the LUBW's terms of use for
data of the Umweltinformationssystem (UIS). They allow reuse on the condition of the line "Grundlage: Daten aus dem
Umweltinformationssystem (UIS) der LUBW Landesanstalt für Umwelt Baden-Württemberg"; the only restriction on
commercial use concerns the dataset of protected biotopes.

The WFS (`rips-gdi.lubw.baden-wuerttemberg.de/.../UIS_0100007500200036_AGGR/MapServer/WFSServer`, type
`Wasserschutzgebietszone_M1`, GeoJSON) gives per zone `WSG` (the name as the register writes it, often in capitals,
quotes and with a number: `\"MAHDENTAL\" 111032`), `ZONE` ("Zone IIIB", "Zone III und IIIA", "Zone I und II bzw.
IIA"), `STATUS` (festgesetzt or vorläufig angeordnet) and `DST`, the lower water authority in charge (a Landratsamt or
a Stadt). Zone I is not published; it comes joined with zone II.

## Decision

- A water protection finding carries the authority in charge where the data name it (`authority`, also in
  `permits.json`). The line of the section adds "(zuständig: …)", the hint names it after "the lower water
  authority".
- A source `lubw` (`lubw_permits`) reads the zones with name, zone, status and authority. A name of the register
  loses its quotes, backslashes and trailing number and is written as a name; a provisional zone says
  "(vorläufig angeordnet)".
- Baden-Württemberg's manifest names the host. With it every state of the adapter brings permit sources of its own,
  and the adapter has no state without them any more.

## Consequences

- The notes name water protection areas in every state.
- The hint of a BW zone says whom to ask by name. Other sources can fill `authority` when their data have it.
- Should the LUBW change its terms, the source comes out again.
