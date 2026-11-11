# 0099. Paved areas get their border

Date: 2026-11-11
Status: Accepted, amends ADR 0069

## Context

ISSprOM 2019-2 §4.5 says of the paved area (501): "It should be bordered (or framed) by the symbol Step or edge of
paved area (501.1)", and allows to leave the black line out "where it is logical". The 501.5 areas of the ALKIS
traffic parcels, the street survey and the land use were drawn without one, because a parcel boundary is no
visible edge. In the Biesdorfer Park review the user asked for paved areas with a border.

## Decision

- `build/paved_borders.rs` is a step of `starter_documents` after the paved areas and the build objects are final.
  Every 501.5 area gets the outline of its union as 501.1, minus what shows an edge already: a building outline,
  water, an existing 501.1 line within 0.75 m (the street survey's kerbs) and the edge of the map. A piece shorter
  than 1 mm on paper is left out. Tags: `freemapper:source=generalization`, `freemapper:generalized=border of a
  paved area (ISSprOM 501)`.
- The areas of the map and the areas among the candidates are bordered apart, each border going to its own list.
  A candidate edge along a map edge is not drawn twice.
- OSM service ways with a paved surface (`asphalt`, `paving_stones`, `sett` and the like) are paved lines like the
  footways of ADR 0098: 501.6 to 501.9 by `width`, 501.8 (3 m) without one, so a driveway or the access to a
  castle is wider than a footpath. Other streets keep no rule. This amends ADR 0098.
- A place that is no area object (the forecourt of the Biesdorfer Schloss is a hole in the yellow park, the walkable
  base shows through) gets no border: there is nothing to outline.

## Consequences

- Street and square areas show a black edge along their outline, as ISSprOM maps do; the kerbs of the street survey
  stay as they are and are not doubled. The output of all states changes; the commit blesses G4.
- Cost if wrong: a paved area next to another paved area of a different map layer shows a seam line where the union
  leaves a gap; a kerb 0.8 m from the outline is drawn twice.

Source: user decision 2026-11-11 after the Biesdorfer Park review; ISSprOM 2019-2 §4.5.
