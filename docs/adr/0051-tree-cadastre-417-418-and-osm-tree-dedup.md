# 0051. Berlin trees from the tree cadastre: 417 or 418 by crown, OSM trees nearby dropped

Date: 2026-11-09
Status: Accepted

## Context

Berlin's tree cadastre (`baumbestand:strassenbaeume`, `baumbestand:anlagenbaeume`) lists street and park trees
with species and crown diameter (`kronedurch`), under dl-de/zero-2-0. OSM maps some of the same trees as
`natural=tree`, which the OSM rules make 418 candidates. Grün Berlin's own park tree layer has two licences that
disagree (catalogue dl-de/by-2-0, service dl-de/zero-2-0).

## Decision

- The cadastre only; Grün Berlin's layer is left out until its licence is clear.
- User decision of 2026-11-08: a crown of 10 m or more is 417 (prominent large tree); every
  other tree, also one without a crown diameter, is 418. Both map kinds, candidates only, because the cadastre can
  be older than the trees. The rules use `berlin:kronedurch>=10`.
- Tag `freemapper:baum` = botanical name (else the German one) and `Krone <n> m`, e.g.
  `Tilia cordata, Krone 12 m`, so the mapper sees why a tree got its symbol.
- OSM `natural=tree` candidates within 2 m (`TREE_DEDUP_M`, map coordinates, grid index) of a cadastre tree are
  dropped: the cadastre knows the crown. OSM trees farther away stay.
- Attribution `Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0` when a tree layer with trees is stored.

## Consequences

- Berlin candidates show each cadastre tree once, with its crown size.
- Trees on private ground are not in the cadastre; there OSM and the nDSM stay the sources.
- Cost if wrong: two distinct trees within 2 m lose the OSM one.

Source: `docs/regions/berlin-plus-reference.md` §5; user decision 2026-11-08; branch `berlin-plus`, Task 4.
