# ADR 0191: The Pomniki historii in the Polish permit notes

## Status

Accepted, 23 November 2026. Extends [ADR 0190](0190-poland-archaeological-register-in-the-permit-notes.md), which left
the Pomniki historii under "Not decided here".

## Context

The NID runs a WFS of the monuments of history (Pomniki historii) with true shapes, unlike its two WMS registers.
Checked on 23 November 2026, anonymous, with the fixed `User-Agent`: `INSPIRE_WFS_MHD/service.svc/get`, WFS 2.0.0,
type `gmgml:Monuments_of_History`, `outputFormat=application/vnd.geo%2Bjson`, EPSG:2180 only. The whole country is
387 features of 184 ids (2 MB, 2.5 s); a monument comes in parts (Tarnowskie Góry has 55). The box
`<N min>,<E min>,<N max>,<E max>,EPSG:2180` works. Without the CRS suffix a box is read differently (Kraków gave one of
two parts, Słubice nothing); `urn:ogc:def:crs:EPSG::2180` and EPSG:4326 give an exception report with status 200.
GeoJSON coordinates come easting first. An empty answer carries `"bbox": [1.79769313486232e308, ...]`, which is more
than an f64 and fails a plain JSON parse. Dataset 168 of dane.gov.pl names CC BY 4.0.

## Decision

- **New source `nid-pomniki`**, same host (no new entry in the hosts list; revision 9), one request per map area.
  The fetch converts the box to EPSG:2180 with the existing `box_2180` and each coordinate back with
  `pl1992_inverse`; the answer is read member by member, so the giant `bbox` does not matter. An exception report,
  a non-JSON answer, a part without a usable shape or a non-collection fails the source (a count would be wrong); a
  part without `INSPIREID` is left out.
- **Position.** Each part is a finding with its shape. The engine now reads lines (`Shape::Lines`; a line counts as on
  the map when it touches the map area, a point and a line on the border included, an area needs an overlap).
- **Section** `PermitSection::HistoryRegister`, finding `FindingKind::RegisterHistory { id, name }`, field
  `register_history` with two lists of the shape of `register_monuments`: `in_area` (some part touches the map)
  and `nearby` (only in the rectangle around it). A monument counts once by `id`, takes the first name any part has, and
  is in the area when any part is. Names are grouped with counts, at most ten per list, the rest counted.
- **Hint** `permits-hint-history-register-pl` when either list is not empty: the President declares a registered
  immovable monument or a cultural park a Pomnik historii (art. 15 ust. 1); it already stands under that
  protection; work at a registered monument needs the permit of the voivodeship conservator (art. 36 ust. 1); in a
  cultural park the council's resolution decides (art. 16 ust. 2, art. 17 ust. 1); the Act names no walk or run.
  Texts in all 21 catalogs, English, German and Polish by hand, the rest machine-made and unreviewed.
- **Credit** as ADR 0184. Archaeological Pomniki (Biskupin, Ostrów Lednicki, Krzemionki) also come through the
  archaeological register; each section counts its own.

## Law as read on ISAP and in the Sejm API on 23 November 2026

Dz. U. 2024 poz. 1292 is the last consolidated text. Art. 36 ust. 1 pkt 5 (archaeological research) and pkt 12 (searching with devices) still apply: the repeal of pkt 12 (Dz. U. 2023 poz. 1904, art. 3) was moved to 1 January 2027 (Dz. U. 2025 poz. 537), and art. 36b then puts an adult's search under the owner's consent and a notification in an app. Since 3 June 2026 (Dz. U. 2025 poz.
1673) pkt 2, 7 and 9 to 11 are repealed and art. 36aa asks a notification for work in the surroundings. Hints name
art. 36 ust. 1 only for work at a registered monument (pkt 1), which stays.
