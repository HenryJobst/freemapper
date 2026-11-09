# 0066. Point symbols keep the norm's gap; dense tree groups become areas

Date: 2026-11-09
Status: Accepted, the tree groups amended on 2026-11-10 (see the amendment below)

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
  and 5 m for 418), united and closed by 0.5 mm. At least half of it on a drawn 501.x → 501.3 (§4.5). Else
  only on open land: at least half of it on 401 to 404 among the map objects, the candidates and the green
  strips of the paved build (`OPEN_LAND_SHARE`, measured before the cuts; the generalization's own areas never
  count). There a crown cover of at least 50 % → 405, because the 402 screen itself leaves half the area
  white; else 402 (§4.4). Off paving and open land the trees stay points and are thinned: a 405 is white and
  invisible on the white base, and a 402 asserts open land the map does not know (ruling of 2026-11-09). The
  norm gives no canopy share, so 50 % is a ruling.
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
- Few parks have open land on the map today (OSM grass pitches, the Berlin green strips). Viktoriapark (4 group
  areas, 84 trees) and Sanssouci (2 areas, 58 trees) therefore have no group area: those trees are thinned
  points, which the PDF shows.
- On the schoolyard the 520 covers everything off the grounds, so the PDF shows no point there.
- Cost if wrong: a thinned point the mapper wanted, an area over distinct trees, fewer areas where open land
  is missing from the map, or a hidden point under a wrong 520 that the PDF lacks. A cluster that is half row, half grove is judged as a whole.

Source: user decisions of 2026-11-09; ISSprOM 2019-2 §3.1, §3.3, §4.4, §4.5; plan
`docs/superpowers/plans/2026-11-09-states-plus-1.md` (Task 5b); branch `states-plus-1`, Task 5b, its review and
fix round (rulings on 520, rows, rank and crown).

## Amendment of 2026-11-10: tree groups on the white base

The user overruled the open-land-only ruling. In the Sanssouci training PDF most park trees stayed a field of
green dots 4 to 7 m apart (1 540 visible tree points, 338 pairs closer than 5 m), because a white 405 on the
white base shows nothing and the map knew no open land there. User rule: closely spaced single trees become
forest or another area symbol that the map shows.

- Group: 3 or more trees chained by single linkage when they break the point gap, when their crowns come
  within `GROUP_CLOSE_MM` (0.5 mm) of each other, or when they stand closer than `GROUP_LINK_MM` (2.5 mm,
  10 m at 1:4000). 2.5 mm is the side of the smallest 402 (6.25 mm², §4.4). The norm names no spacing, so the
  value is a ruling. With touching crowns alone, Potsdam's park trees (no crown in the data, 5 m by default)
  mostly stayed points.
- Region: each tree claims a disc of 1.25 mm radius or its crown, whichever is larger; the union is closed by
  0.5 mm. The cover is the share of that region under the crowns.
- Symbol: on paving (at least half) 501.3, as before, but there the trees claim only their crowns and a piece
  must take in at least 9 trees (`PAVED_MIN_TREES`); a group on paving at least 3 times as long as wide is a
  street and stays points. Elsewhere 405 at a cover of 50 % or more, else 402, on open land and off it. A 405
  with less than half on open land gets a 416 distinct vegetation boundary along its outline (§4.4: "a
  distinct forest edge"). The 416 is left out within 0.1 mm of a cut (building, water, 520, paving, clip
  window) and below its isolated minimum of 2.65 mm (five dots). Such a forest needs 1.77 mm² and 1.5 mm width,
  the smallest area a dotted line may enclose (§3.3: 1.5 mm across). A 402 needs no boundary, its yellow
  screen shows.
- I rejected 406 to 410: they claim slow running, and park lawns under trees run well. 402 on the white base
  claims open land under the trees. In a park that is usually right, and the mapper checks it anyway.
- Rows: the cluster test stays (corridor under 2 symbol sizes, or 3 times as long as wide), but the long
  form now also needs to be narrower than 2 mean crowns, so a park strip several crowns deep is a group. The
  opening of the region is at least 1.2 mean claim diameters (`ROW_OPENING`), so a row joined to a block
  falls out of the area and its trees stay points.
- Off paving, 402 and 405 are cut at the drawn paving too: a road through a stand is no forest.
- The training PDF draws point patterns of area symbols now (402's white dots, 501.3), with dots whose centre
  lies inside the area; before, a 402 printed as plain 401 yellow.

Measured on the examples (candidates and map together, offline from the stored data):

| | before | after |
|---|---:|---:|
| Sanssouci tree points (visible) | 1 654 (1 540) | 880 (766) |
| Sanssouci tree pairs closer than 5 m | 338 | 130 |
| Sanssouci group areas | 0 | 51 × 402 |
| Viktoriapark tree points (visible) | 1 660 (1 578) | 949 (867) |
| Viktoriapark group areas | 0 | 44 × 405 (51 × 416), 9 × 402 |

Cost if wrong: a 402 over ground that is no lawn, a 405 where only a few large trees stand (the default crowns
are small, the measured ones in Berlin often large), and many dotted 416 lines in a park full of tree stands.
The remaining points are rows and trees more than 10 m from the next one.
