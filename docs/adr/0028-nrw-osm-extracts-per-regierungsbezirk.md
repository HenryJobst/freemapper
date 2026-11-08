# 0028. North Rhine-Westphalia: OSM extracts per Regierungsbezirk

Date: 2026-11-08
Status: Accepted

## Context

The Geofabrik extract for NRW is 917 MB, 2.6 times the Hesse one. On the test line it took about 13 minutes to
download, and a full read takes minutes. Geofabrik also publishes one extract per Regierungsbezirk, 137 to 225 MB
each (ADR 0008).

## Decision

NRW maps read the five Regierungsbezirk extracts (Arnsberg, Detmold, Düsseldorf, Köln, Münster) instead of the
state extract. Each has a bundled boundary made from Geofabrik's `.poly`, converted to 25833 and simplified to
25 m. An area reads every district extract within 100 m, the same rule as for states, and features are merged by
OSM id. If no district lies within the margin, the nearest is read with a warning.

## Consequences

- A map on a district border reads two extracts.
- The OSM code knows sub-state extracts.
- Cost if wrong: five boundary assets to maintain instead of one, and two downloads for a map on a district
  border.

Source: NRW reference §11, §13 item 5; NRW plan `2026-11-08-nordrhein-westfalen.md` (Rulings) and commits
59345b1, 1aa58cf on branch `nordrhein-westfalen`.
