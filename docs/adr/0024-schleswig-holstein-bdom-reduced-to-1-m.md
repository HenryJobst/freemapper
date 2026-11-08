# 0024. Schleswig-Holstein: nDSM from the bDOM reduced to 1 m

Date: 2026-11-08
Status: Accepted

## Context

The only open surface model in Schleswig-Holstein is the bDOM, 0.2 m and image-based. It costs about 105 MB and
90 s of download per km². The DGM1 is 1 m.

## Decision

The nDSM is bDOM minus DGM1. Each bDOM tile is streamed to a temporary file, decoded once and reduced to 1 m by
the median of each 5 x 5 block, ignoring NaN. Only the 1 m tile is cached. The estimate states the download, a
failed surface warns instead of failing, and freemapper warns when the bDOM is older than the orthophoto.

## Consequences

- The cache stays small, at the cost of the 0.2 m detail.
- Cost if wrong: slow first fetches in Schleswig-Holstein.

Source: spec §4.1 (Schleswig-Holstein); plan `2026-11-08-schleswig-holstein.md` (Rulings, nDSM); SH rulings;
commit b6be3c8.
