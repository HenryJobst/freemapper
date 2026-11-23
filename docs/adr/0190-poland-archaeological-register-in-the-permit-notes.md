# ADR 0190: The register of archaeological monuments of the NID in the Polish permit notes

## Status

Accepted, 23 November 2026. Extends [ADR 0184](0184-poland-monument-register-in-the-permit-notes.md),
which left archaeological sites and the Pomniki historii under "Not decided here".

## Context

The register of immovable monuments (ADR 0184) holds no archaeological sites. Checked on 23 November
2026, all anonymous, with the fixed `User-Agent` of the program:

- The NID runs the register of archaeological monuments (book C) as its own WMS,
  `https://usluga.zabytek.gov.pl/INSPIRE_AMD/service.svc/get`, layer `Archaeological_Monuments`. It is the
  same software as the other register: `GetFeatureInfo` with the map box as one pixel returns every site that
  meets the box, no geometry. Dataset 210 of dane.gov.pl names this WMS and gives CC BY 4.0. Biskupin box: 8
  sites in 5 KB, 0.4 s. Kraków Stare Miasto: 64 sites in 38 KB, 0.5 s. All of Poland as one box: 2.8 MB, 2.8 s.
- The Archaeological Record of Poland (AZP) is a WFS (`WFS_AZP`) of the sheet grid only. The sites of the
  record that are not entered in the register are not open.
- The Pomniki historii are a WFS with polygons (`INSPIRE_WFS_MHD`, 387 features, 2 MB for the country, bbox in
  EPSG:2180 with the northing first works, CC BY 4.0). See the reference. Not built here.

## Decision

- **New source `nid-archeologia`** in the Polish adapter, same host (so no new entry in the hosts list;
  revision 8), one request per map area. The parser of ADR 0184 serves both registers; it lowers the names of
  the attributes, because this register writes them in camel case. Entries are merged by id. An answer of 5000
  entries or more, an exception report or a non-XML answer is a failure, so the section is left out.
- **Position.** None, as with the other register: each site gets the rectangle of the map box, and the texts say
  "in the rectangle around the map area".
- **Section** `PermitSection::ArchaeologyRegister`, finding `FindingKind::RegisterArchaeology { name }`, field
  `register_archaeology` of the regional notes with the shape of `register_monuments`. Names are grouped with
  their counts, at most ten. They are mostly "place, st. n", so a long list stays short only where sites share a
  name.
- **Hint** `permits-hint-archaeology-register-pl` when at least one site is found. It says the Act on the
  Protection of Monuments (ustawa o ochronie zabytków) regulates searching for hidden or abandoned movable
  monuments with electronic or technical devices (metal detectors) and makes archaeological research and other
  work at a registered monument subject to the permit of the voivodeship conservator (art. 36 ust. 1). It names
  no walk, run or control, and says to ask the conservator's office. It gives no number of a point, because the
  numbering of the research permit was not confirmed from a primary text, and it does not say whether the
  notification route for searches added in 2023 (Dz. U. 2023 poz. 1904) replaces the permit; both are open.
- **Credit** the same as ADR 0184, "© Narodowy Instytut Dziedzictwa (CC BY 4.0)".
- **Texts** in all 21 catalogs; English, German and Polish by hand, the other 18 machine-made and unreviewed.

## Not decided here

- **Pomniki historii.** A source of its own with positions would be a GeoJSON reader, a conversion of the map box
  to EPSG:2180, and a section that can say "in the map area". Biskupin, Ostrów Lednicki and Krzemionki are
  Pomniki historii of class `archaeological` and are also in this register.
- **Sites only in the AZP record.** No open source.
- **The Polish legal wording** needs a Polish reader before release; so does the point of the Act for the
  research permit.

## Addendum, 23 November 2026: the legal wording checked

The decision left two points open; both are settled from the texts on the Sejm ELI API (`api.sejm.gov.pl/eli`):

- Archaeological research is art. 36 ust. 1 pkt 5; searching with electronic or technical devices is pkt 12.
- The amendment Dz. U. 2023 poz. 1904 repeals pkt 12 and adds art. 36b (an adult, with the consent of the owner
  and the holder of the property, after a notification through the app of the ministry), art. 36c (ban on such
  searches among others on registered monuments and within 10 m of them) and art. 36d (the search register).
  It is **not yet in force**: its entry into force was moved to 1 May 2025 (Dz. U. 2024 poz. 647) and then to
  1 January 2027 (Dz. U. 2025 poz. 537; Dz. U. 2025 poz. 1673, art. 4, adapts it to that date). On 23 November 2026
  the permit of pkt 12 therefore still applies. An earlier assumption that the permit had ended on 1 May 2025 is wrong.

The hint `permits-hint-archaeology-register-pl` now names pkt 5, pkt 12 and pkt 1 (conservation, restoration and
building work at a registered monument), says that the permit for searches falls away on 1 January 2027, and names
what replaces it (art. 36b, 36c). It does not say "regulates". The date can move again; the reference file says to
check it before each release. The texts changed in all 21 catalogs, by hand in English, German and Polish and
machine-made and unreviewed in the other 18 as before. The adapter revision stays 8.
