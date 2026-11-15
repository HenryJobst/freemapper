# 0146. Natural monuments and protected biotopes of the states

Date: 2026-11-15
Status: Accepted

Research: [`naturschutzobjekte-2026-11-15.md`](../research/wald-und-sprint-behoerden/naturschutzobjekte-2026-11-15.md).
Builds on [0120](0120-permit-sources-of-region-adapters.md) (regional sources, protected objects in Berlin), [0130](0130-permit-sources-as-tables.md)
(table sources) and [0144](0144-monuments-from-more-states.md) (gaps, the lower authority hint).

## Context

Naturdenkmale (§ 28 BNatSchG), geschützte Landschaftsbestandteile (§ 29) and gesetzlich geschützte Biotope (§ 30, or the
state's law) are not in the BfN's protected areas; only Berlin's were in the notes. For a run in the forest the
biotopes matter most: whatever can destroy them or harm them considerably is forbidden. The research found open
services with a clear licence in most states, of differing depth:

| State | Naturdenkmale | Landschaftsbestandteile | Biotope |
|---|---|---|---|
| Brandenburg | forest service (data of 2017) | forest service | LfU register, `bbgnat_t` marks the protected ones |
| Berlin | already in | already in | Umweltatlas, `schutz_ges` 1 certain, 2 suspected |
| Hamburg | protected areas service | none | biotope register, layers of the protected ones |
| Saarland | Geoportal | one ordinance found invalid, not marked | Geoportal |
| Saxony | areas only (trees with the Landkreise) | none | register of the LfULG |
| Saxony-Anhalt | areas and points | LAU | only on request |
| Bavaria | LfU | LfU | no service with geometry |
| Baden-Württemberg | LUBW | none | commercial use only on request |
| Hesse | no licence named | none | old mapping (1992 to 2006), CC BY 4.0 |
| North Rhine-Westphalia | Kreise | Kreise | LINFOS, without the type |

Lower Saxony (ArcGIS REST only), Rhineland-Palatinate (ODbL, its WFS refused the queries), Mecklenburg-Vorpommern
(licence contradicts itself) and Schleswig-Holstein (Kreis Schleswig-Flensburg only; biotopes as downloads with
contradicting licences) wait.

## Decision

- `ProtectedObjectKind::Biotope`, whose name is the type of biotope; it needs the lower nature authority as the others
  do.
- A table reading `Object` gives a protected object with name and act from the attributes the layer names; with
  `only` a feature counts only when an attribute holds one of the values (Brandenburg's `geschützter Biotop nach § 18`,
  Berlin's `schutz_ges` 1).
- `natur_permits` describes the services of ten states as tables. Their gaps are named where the data are known to be
  incomplete (Brandenburg's Naturdenkmale of 2017, Saxony's trees, Saxony-Anhalt's trees and biotopes, Hesse's old
  mapping).
- The areas of one kind, name and act are one entry with their summed share, so dozens of biotopes of one type are one
  line.
- Two hints: the protected biotopes on the map, with up to three types, and the lower nature authority for the
  protected objects outside the city states when no protected area named it already.
- Text values with `\"` (the LUBW's names) are read with plain quotes.
- The hosts `inspire.brandenburg.de`, `luis.sachsen.de` and `www.lfu.bayern.de` join their manifests.

## Consequences

- Forest maps in these states say which biotopes lie on the map and that whatever harms them is forbidden; sprint maps
  in towns list the Naturdenkmale with their species.
- Berlin's suspected biotopes stay out; North Rhine-Westphalia's biotopes have no type.
- The section's empty text no longer lists kinds a source may not hold.
