# 0129. Permit notes for Hamburg, water protection and forest functions

Date: 2026-11-14
Status: Accepted

Research: [`nachrecherche-2026-11-14.md`](../research/wald-und-sprint-behoerden/nachrecherche-2026-11-14.md).
Builds on [0120](0120-permit-sources-of-region-adapters.md), [0124](0124-forest-administrations-in-the-permit-notes.md)
and [0127](0127-permit-notes-follow-the-map-area.md).

## Context

Hamburg was the second city state without its Bezirke in the notes; ADR 0120 noted that another city state may need
its own hint texts. A live check on 2026-11-14 found six WFS on `geodienste.hamburg.de` (a host of Hamburg's
manifest) that answer GeoJSON (`application/geo+json`) under dl-de/by-2-0: Bezirke, the Grünplan's list of public
green spaces, the monument list, the forest of the city (Revierförstereien and Bannwald), the Erholungswald and the
water protection areas with their zone. Hamburg splits its offices differently from Berlin: the Bezirksamt has a
Fachamt Management des öffentlichen Raumes, and the monuments belong to the central Denkmalschutzamt.

Water protection zones and forest functions (Erholungswald, Bannwald, Naturwald, closed forest) matter for both
forest and sprint maps: zones I and II usually rule out an event. Several other states publish them too.

## Decision

- **Two more kinds of finding**, for every region: `WaterProtection { name, zone, citation }` (section
  `WaterProtection`) and `ForestFunction { kind, name }` with the kinds recreation, protected, natural and closed
  (section `ForestFunctions`). `inner_zone` reads the zone text (I and II inner; IIIA, III B, 3 and IV outer).
- **Hints**: one per water protection area and zone, saying whether it is an inner zone or, without a zone in the
  data, both cases; one per forest function with the names of its areas.
- **Offices of a city state** come from the source of its districts: `PermitSourceInfo::offices`
  (`DistrictOffices::Berlin` or `Hamburg`). The hints carry `offices = hamburg` for Hamburg and nothing for Berlin,
  so Berlin's stored notes stay as they are. Hamburg's monuments get one hint for the Denkmalschutzamt, not one per
  Bezirk.
- **Hamburg's six sources** keep only the properties a finding reads. The monument list describes buildings with
  their architects, which never reach `data/permits/`.
- **An empty answer** of Hamburg's services is a FeatureCollection without `features`; the generic reader reads it
  as empty.

## Consequences

- Sprint maps in Hamburg get the Bezirksamt, the green space and the monuments; forest maps the Revierförsterei,
  the Bannwald and the Erholungswald; both the water protection zones.
- Water protection and forest functions of other states (Berlin, Brandenburg, NRW, Saarland, Hesse,
  Baden-Württemberg, Bremen) can now be added as sources without changing the engine.
- The hint texts for Hamburg's offices are a ruling from the offices' public structure; the notes stay hints.
