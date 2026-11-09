# 0072. Building passages from OSM are 522 candidates

Date: 2026-11-10
Status: Accepted

## Context

Passages through buildings and arcades decide routes in an old town. The OSM census of the Lübeck Altstadt (2026-11-10)
found 82 ways with `tunnel=building_passage`, plus 104 ways in the network "Lübecker Höfe und Gänge". The Viktoriapark
has 35. Until now an OSM passage was a generic 505.1 candidate, and only when its surface was not paved: the rule for
unknown ways excludes `paving_stones`, `asphalt` and the like, so the commonest passage never appeared. A passage on a
line is also the wrong shape. The competitor sees a covered strip through the building, not a path line.

What the norm says (ISSprOM 2019-2):

- 2.3: "A maximum of 2 running levels shall be mapped [...]. Only passages (e.g. underpasses, lighted tunnels, bridges)
  between these two levels, which are important for the competitors should be represented on the map."
- 4.5, 522 Canopy: "A canopy is a building construction (with a roof), normally supported by pillars, poles or walls,
  such as passages, gangways, courts, bus stops, gas stations or garages. Small passable parts of buildings which cannot
  easily be crossed by competitors, shall not be represented on the map." Minimum width 0.5 mm, minimum area 0.25 mm².
- 4.5, 512.2 Underpass or tunnel (L): "a passage running underneath the ground or a building". The line marks the end of
  the passage and "is only used when there is no other barrier line defining the extent of the lower level area".
- 4.5, 512.3 Area passable at two levels (A): the upper part of a two-level structure, drawn as a white striped pattern
  over the ground symbols.

The bundled OOM symbol set has no 512.3 (it has 501.2 and 501.12 for paving in multi-level structures). A passage
through a building has its walls drawn already (the 521 outline), so 512.2 would add a line the norm asks us to omit.
ADRs 0047 and 0061 already map the ALKIS passages and arcades to 522 candidates with the building left whole. The
norm's own list of canopy examples starts with "passages".

## Decision

- `tunnel=building_passage` on any `highway=*`, and `covered=yes|arcade|colonnade` on a footway, path, pedestrian way,
  cycleway, service way, living street, steps or track, become **522 candidates** on sprint maps. Not read: ways with
  `tunnel=yes|culvert` (a real tunnel, 512.2 territory), `indoor=yes` and `area=yes`. The surface plays no part.
- A way is a line and 522 an area, so the rule engine gets a new field, `corridor_m` (with `draw = "line"`). The way
  becomes a strip along it, ends cut straight, so it stops where the passage ends and does not bulge into the street.
  The strip is 3 m wide, or as wide as the `width` tag says, held between 1.5 m and 12 m. The lower limit keeps the
  strip above the norm's 1 m footprint minimum for 520 and 522 at 1:4000 and 1:5000 (0.25 mm).
- The rules sit before the unpaved-way rules, so a covered gravel way is a passage, no longer a 505.1 candidate.
- The building stays whole, as in ADR 0061. The mapper decides on site whether the passage can be used in the race, and
  whether 512.3 or a note in the bulletin is better; the norm asks for the second level only where it matters.
- Tags as for every OSM candidate: `freemapper:source=osm`, `freemapper:osm=highway=...; tunnel=building_passage`.

## Consequences

- Where ALKIS or the Berlin building parts also hold the passage, there are two 522 candidates on top of each other.
  The mapper deletes one; the OSM one carries the way's name in `freemapper:source_id`.
- Existing maps get the new candidates only with `run --force`.
- Not done: 512.2 end lines, 512.3 hatching. Both need the second level to be decided, which a data source cannot do.
