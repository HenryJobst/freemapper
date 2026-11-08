# 0042. Rhineland-Palatinate: provider `LvermGeoRp` and the attribution line

Date: 2026-11-08
Status: Accepted

## Context

`Provider::LvermGeo` already names the survey office of Saxony-Anhalt, and the office of Rhineland-Palatinate has the
same short name. Its licence, dl-de/by-2-0, asks for the line
"©GeoBasis-DE / LVermGeoRP<Jahr des Datenbezugs>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]". The sources
space the year in three different ways. The official licence title, "Datenlizenz Deutschland – Namensnennung – Version
2.0", contains dashes.

## Decision

The provider is `Provider::LvermGeoRp` (serde `lvermgeorp`), shown as "LVermGeo RP"; its object ids start with `DERP`.
The attribution follows the open-data page, with no blank before the year. The year is the download year, and
"[Daten bearbeitet]" stays because freemapper derives map objects. The line goes into the map credits, the map notes and
`ATTRIBUTION.txt`. The licence note `Lizenz dl-de/by-2-0 (Datenlizenz Deutschland – Namensnennung – Version 2.0):
https://www.govdata.de/dl-de/by-2-0` goes into the notes and `ATTRIBUTION.txt` only. The licence title keeps its dashes,
because it is a proper name.

## Consequences

- Saxony-Anhalt projects keep `LvermGeo` and are not touched.
- The credit matches the form the licence page prints.
- Cost if wrong: none visible; a reworded line changes only text.

Source: Rhineland-Palatinate reference §1, §9, §13 item 9; plan `docs/superpowers/plans/2026-11-08-rheinland-pfalz.md`
(Rulings); rulings ledger, Task 4 (licence title); commits 78c0f3c, 6610f0f and 4dc59c4 on branch `rheinland-pfalz`.
