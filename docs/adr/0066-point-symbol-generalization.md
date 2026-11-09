# 0066. Point symbols keep the norm's gap; dense tree groups become areas

Date: 2026-11-09
Status: Accepted

## Context

The point candidates of OSM, ALKIS, NAS, the street survey, the registers and the tree cadastres pile up. At
Viktoriapark 2 078 points gave 722 tree pairs closer than their symbols allow. The training PDF draws every
candidate as a map object, so the overlaps print. ISSprOM 2019-2 (January 2022) gives the values below.

## Decision

- User decision of 2026-11-09: every point symbol the build adds, candidate or map object, keeps the norm's
  gap. `generalize_points` (`build/points.rs`) runs in `starter_documents` after the whole dedup chain and before
  `drawn_registers`, so OOM, OCAD and the PDF get the same result on both map kinds.
- Gap: §3.3 gives 0.15 mm between lines of the same colour and names no gap for points. Ruling:
  `POINT_GAP_MM = 0.15` between the outer edges of any two point symbols. Sizes (`POINT_SIZES_MM`) are the larger
  of the norm's outer measure and the bundled drawing: 311 0.90, 312 1.20, 313 1.45, 417 1.40 (white mask after
  the errata of 2022), 418 0.80, 526 1.00, 531 1.27 mm. §3.1 enlarges everything on the schoolyard by 1.25, except
  the native school symbols 321 (1.30), 542 (1.72), 544 (1.10) and 545 (1.40). At 1:4000 two 418 need 3.8 m
  between centres, 417 and 418 5.0 m, two 417 6.2 m.
- 520 (§4.5: "No feature shall be represented in this area, except … very large trees"). A 520 map object
  removes every point in it. A 520 candidate is a land-use guess, so the candidate points under it stay in
  `candidates.omap` and the OCAD candidates, tagged `freemapper:hidden=520`, and only the training PDF leaves
  them out. A 520 candidate never touches a map point. The mapper adds very large trees back.
- Tree groups: 3 or more trees (`GROUP_MIN_TREES`) chained by single linkage. A row, whose centres lie in a
  corridor narrower than 2 symbol sizes or which is at least 3 times as long as wide, is only thinned, so
  avenues stay rows of tree symbols. Any other group: the crown discs (`freemapper:crown_m`, else 12 m for 417
  and 5 m for 418), united and closed by 0.5 mm. At least half of it on a drawn 501.x → 501.3 (§4.5); else a
  crown cover of at least 50 % → 405, because the 402 screen itself leaves half the area white; else 402
  (§4.4). The norm gives no canopy share, so 50 % is a ruling.
- The area is cut at the clip window, 520, 521 and water, then each piece must reach the minimum: 402 and 501.3
  6.25 mm² and 2.2 mm wide, 405 1 mm² and 0.3 mm. Width means a disc of that width fits (an opening), because
  2·area/perimeter halves the width of compact shapes. A piece below the minimum is left out and its trees go
  back to the thinning. On 402 the 417 trees stay points ("may be added").
- Area tags: `freemapper:source=generalization`, `freemapper:source_id` = the merged trees (sorted),
  `freemapper:currency` = the oldest, `freemapper:generalized=<n> trees, cover <p> %`.
- Thinning is greedy. Map points come first and no candidate drops one. Then: size, rank (ADR 0063), measured
  crown before the default, crown, source and id, x, y. Rank always comes before the crown, because "crown when
  both are measured, else rank" has no total order (a measured OSM, a default ALKIS and a measured register tree
  can beat each other in a circle). The kept point names the dropped ones in `freemapper:generalized`; a map
  point names only the map points it drops.
- Out of scope: gaps to lines and areas, displacement, boulder groups (207/208 for a later 204 source).

## Consequences

- Berlin map objects stay unchanged; the Viktoriapark candidates and the PDF change. There, 80 of 84 tree
  clusters are rows and stay points, and 82 points in residential 520 candidates are hidden in the PDF.
- On the schoolyard the 520 covers everything off the grounds, so the PDF shows no point there.
- Cost if wrong: a thinned point the mapper wanted, an area over distinct trees, or a hidden point under a wrong
  520 that the PDF lacks. A cluster that is half row, half grove is judged as a whole.

Source: user decisions of 2026-11-09; ISSprOM 2019-2 §3.1, §3.3, §4.4, §4.5; plan
`docs/superpowers/plans/2026-11-09-states-plus-1.md` (Task 5b); branch `states-plus-1`, Task 5b, its review and
fix round (rulings on 520, rows, rank and crown).
