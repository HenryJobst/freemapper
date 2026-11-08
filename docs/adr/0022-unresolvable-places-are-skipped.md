# 0022. Unresolvable places are skipped and reported

Date: 2026-11-08
Status: Accepted

## Context

The Hesse Downloadcenter lists zips per municipality and district. Unincorporated areas (gemeindefreie Gebiete
such as Reinhardswald and Spessart) have no unique district or file in the listing. Treating them as errors would
stop every fetch that touches them.

## Decision

A place with no unique district or file is skipped and named in the result (`HeightModel`, `unresolved`). Its
tiles count as missing unless another zip holds them. Read failures (network, zip, CRC) stay errors, and an area
with no tile at all stays an error.

## Consequences

- Fetches in forests on unincorporated land go through with a warning.
- A systematic naming break still fails loudly.
- Cost if wrong: a naming break near the state edge shows only as warnings.

Source: spec §4.1 (Hesse); Hesse rulings (Task 4); commits b489a46, 3381d17.
