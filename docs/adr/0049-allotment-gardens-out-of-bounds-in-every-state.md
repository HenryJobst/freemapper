# 0049. Allotment gardens (Kleingarten) are out of bounds in every state

Date: 2026-11-09
Status: Accepted

## Context

The 520 out-of-bounds candidates come from private ALKIS land uses (`PRIVATE_LAND_USES`, `is_private`). Allotment
gardens are fenced private plots, but ALKIS files them under the open land use `Sport-, Freizeit- und
Erholungsfläche`, next to parks, sports grounds and playgrounds. Berlin's `bezfkt` becomes `bez = Kleingarten`
during normalisation. The Brandenburg and Mecklenburg-Vorpommern data has the same pair (Mecklenburg-Vorpommern
fixture, the data of the Sanssouci example).

## Decision

- `is_private` is also true for `nutzart = Sport-, Freizeit- und Erholungsfläche` with `bez = Kleingarten`, in every
  state and on both map kinds. The candidate carries
  `freemapper:nutzart=Sport-, Freizeit- und Erholungsfläche / Kleingarten`.
- Every other `bez` of that land use stays open.
- The schoolyard logic is unchanged: everything off the school grounds is 520 anyway.

## Consequences

- Sprint maps near allotment gardens get new 520 candidates; examples with such areas change, e.g. Brandenburg
  Sanssouci.
- A state that spells the value differently gets nothing new, which is the old behaviour.
- Cost if wrong: an allotment colony with public paths shows as one 520 area; the mapper cuts the paths out.

Source: ALKIS fixtures (Mecklenburg-Vorpommern, Berlin); plan `docs/superpowers/plans/2026-11-09-berlin-plus.md`
(Rulings); branch `berlin-plus`, Task 4.
