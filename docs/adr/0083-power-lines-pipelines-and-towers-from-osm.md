# 0083. Power lines, pipelines and towers from OSM

Date: 2026-11-10
Status: Accepted

## Context

freemapper read none of these from the geodata: overhead power lines, pipelines above ground, masts, towers and
chimneys. ISSprOM has symbols for all of them, and a course setter needs them (a mast or a chimney is a good control,
a pipeline a barrier). A count in the cached OSM extracts for the eleven example boxes found no `power=line`, no
overhead `power=cable` and no above-ground `man_made=pipeline` at all. It found `man_made=tower|mast|chimney|water_tower|lighthouse`
nodes in five of them (Viktoriapark 3 masts, Blankenese 1 lighthouse, Wismar 1 mast and 2 towers, Monschau 1 tower,
Quedlinburg 1 chimney), and many tower ways (Bautzen 13, Quedlinburg 8), which are buildings in the cadastre.

What the norm says (ISSprOM 2019-2, section 4.5, text of the 2022 update):

- 510 Power line, cableway or ski lift (line): "The bars indicate the exact location of the pylons. If a section of a
  power line, cableway or ski lift goes along a road or path (and does not offer significant additional navigational
  value) it should be omitted." Minimum length 3.75 mm (footprint 15 m).
- 511 Major power line (line): "Major power lines should be drawn with a double line. ... Very large carrying masts
  shall be represented in plan shape using symbol Building (521) or High tower (524). In this case, the cable lines
  can be left out (the map shows only the pylons)."
- 524 High tower (point): "A high tower or large pylon. Very large towers shall be represented in plan shape with
  the symbol Building (521)." Footprint 8.4 m.
- 525 Small tower (point): "An obvious small tower, platform or seat." Footprint 6 m.
- 528 Prominent line feature: "A prominent man-made line feature. For example, a low pipeline (gas, water, oil, heat,
  etc.) ... The definition of the symbol must be given on the map." Minimum length 2.2 mm (8.8 m).
- 529 Prominent uncrossable line feature: "An uncrossable man-made line feature. For example, a high pipeline ...".
  Minimum length 3 mm (12 m).
- There is no symbol for a storage tank. A large one is a building (521) in the cadastre.

## Decision

All rules are in `osm-candidates.toml`, sprint only, and all are candidates: OSM-only data is checked against the
orthophoto, as before (ADR 0072 to 0075).

| OSM | Symbol | Notes |
| --- | --- | --- |
| `power=line` with `voltage` of 110 kV or more (one value of a list is enough) | 511 | |
| `power=line\|minor_line`, overhead `power=cable` (`location=overhead`) | 510 | |
| the same with `location=underground\|underwater`, `tunnel=yes`, `line=busbar\|bay\|connection` | none | not on the ground, or inside a station |
| `power=tower` (node) | 524 | the norm asks for 524 at a major line; where a 510 bar already shows it, delete one |
| `power=pole` | none | the bars of 510 are the poles |
| `man_made=communications_tower\|chimney\|water_tower\|lighthouse` (node) | 524 | |
| `man_made=tower` with `tower:type=observation\|watchtower\|platform` or `height` under 20 m | 525 | else 524 |
| `man_made=mast`: `height` 20 m or more, or none | 524 | a mast without height is a radio mast |
| `man_made=mast`: `height` 10 to 20 m | 525 | |
| `man_made=mast` under 10 m | none on sprint | signs, lamps and flags; a 3 m mast gave a 525 beside a Berlin column |
| `man_made=pipeline` with `location=overhead`, or `overground\|surface` and `height` of 1.5 m or more | 529 | high, uncrossable |
| `man_made=pipeline` with `location=overground\|surface` | 528 | low |
| `man_made=pipeline` without `location`, underground, underwater, in a tunnel | none | buried is the usual case |
| `man_made=storage_tank` and similar | none | no symbol in the norm |

- **Minimum lengths.** The OSM lines had no length limit. 510 and 511 now need 3.75 mm, 528 2.2 mm and 529 3 mm on paper
  (`norm_min_line_mm`); the limit holds for the piece inside the map area too.
- **Points are nodes only.** A tower drawn as an outline is a building in the cadastre (521), and a candidate point in a
  building footprint is dropped (ADR 0076).
- **Control sites.** Towers, masts and pylons carry `freemapper:control=` with `pylon`, `mast`, `tower`, `chimney`,
  `water_tower` or `lighthouse`, next to the street furniture kinds of ADR 0074. They are thinned the same way: a second
  one of the same kind within 25 m takes both out (a pylon row is 200 m apart and stays), and they give way to other
  point symbols. The symbol stays 524 or 525, not 531, because the norm has a symbol for a tower.
- **Not judged.** A line along a road or path should be omitted (510). freemapper does not compare the line with the
  roads, so the mapper decides. 510 and 511 are drawn as given; the double line of 511 is part of the symbol.
- **The definition on the map.** 528 and 529 need a definition in the legend. It is the mapper's to write.
- **Mast and schoolyard.** The schoolyard keeps `man_made=mast` as 544 (any height) and gets no towers, lines or pipelines.
  The flagpole rule is split from the mast rule for this.
- **Point sizes.** `POINT_SIZES_MM` gets 524 (2.35 mm, the drawn extent, the norm says 2.1) and 525 (1.72 mm, the norm 1.5).
- **Number lists.** A `>=` condition now holds when one value of a `;` list does (`voltage=110000;380000`).

## ALKIS

The NAS layers freemapper fetches are `AX_SonstigesBauwerkOderSonstigeEinrichtung`, `AX_Vegetationsmerkmal`,
`AX_Strukturlinie3D`, `AX_BauwerkImVerkehrsbereich`, `AX_BauwerkImGewaesserbereich` and `AX_UntergeordnetesGewaesser`. Their
function code lists (`adv-codes.toml`) have no tower, mast, chimney or line. Towers, chimneys and radio masts of the
cadastre are `AX_Turm` and `AX_BauwerkOderAnlageFuerIndustrieUndGewerbe` (Schornstein, Funkmast); power lines are not part of
ALKIS but of the basic DLM (`AX_Leitung`, `AX_Freileitung`). None of these is fetched. The vereinfacht buildings
already hold towers and chimneys as buildings (`gebnutzbez` "Turm", "Bauwerk oder Anlage für Industrie und Gewerbe"),
so they are in the map as 521. No ALKIS rule is added and no fetch either.

Follow-up, when a new fetch is acceptable: `AX_Turm` and `AX_BauwerkOderAnlageFuerIndustrieUndGewerbe` as 524 or 525
points with the same control kinds, ranked above OSM in `build/dedup.rs` (ADR 0078, a cadastre tower within 5 m beats
an OSM tower as for memorials), and `AX_Leitung` from the DLM for 510.

## Consequences

- No example area has a power line or an above-ground pipeline, so no example shows them; the rules are covered by
  fixture tests only.
- Three examples get one new 524 candidate each: Wismar (radio mast, 1756 to 1757 candidates), Monschau (tower, 850 to
  851) and Quedlinburg (chimney, 1110 to 1111). Symbols +1 in each. Viktoriapark and Blankenese are unchanged.
- A town has few towers and no lines, so the cost for the candidates file is small. A rural map with a power line
  gets a 510 or 511 line over its whole length, pylons as 524 at the 511.
- Existing maps get the objects with `run --force`.
