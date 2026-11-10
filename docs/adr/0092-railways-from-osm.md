# 0092. Railways from OSM

Date: 2026-11-11
Status: Accepted

## Context

A user's sprint map of the Biesdorfer Park in Berlin (1:4000) had no S-Bahn tracks. freemapper read no railway at all:
neither `osm-candidates.toml` nor the state and Berlin rule files nor `adv-codes.toml` had a rule for one. The OSM census
(`docs/research/osm-sprint-census/osm-census-report.md`, item 6) had recommended `railway=rail|light_rail|narrow_gauge|tram`
as 509, and ADR 0083 took the power lines of the same item but left the railways.

What the norm says (ISSprOM 2019-2, section 4.5, text of the 2022 update):

> 509.1 Railway (L). A railway is a permanent track laid with rails on which locomotives, carriages or wagons can
> travel. If it is forbidden to cross or run along the railroad, the forbidden area around the railway shall be
> represented with symbol Area that shall not be entered (520). Minimum length (isolated): two dashes (4 mm -
> footprint 16 m). Colour: black, white.
>
> 509.2 Tramway (L). A tramway is a public vehicle running regularly along certain streets, usually on rails. The track
> can be easily crossed by the competitor. Tramways are generally not represented. However, if they serve navigation
> or orientation, they can be represented. Colour: black 50%.

And in 520 "Area that shall not be entered": "An area that shall not be entered such as a private area, a flower bed, a
railway area etc. No feature shall be represented in this area, except very prominent features such as railways, large
buildings, or very large trees."

The norm has no symbol for a siding, no rule for a double track (each track is a line of its own, like any other line)
and nothing about the embankment, bridge or tunnel of a railway beyond the general rules: a railway on a bridge is
drawn as a railway, one in a tunnel is not on the map. The symbol set bundled with freemapper
(`assets/symbol-sets/ISSprOM 2019_4000.omap`, from OpenOrienteering Mapper) has `509.1` Railway as a combined symbol of
two helper lines (a black border with white fill and black dashes inside, 0.55 mm wide on paper) and `509.2` Tramway as
a line with a 50 % border. The codes are the norm's. The school symbol set of freemapper is built from the same file
but its rules never name 509, and we know of no railway symbol in the IOF school set.

## Decision

All rules are in `osm-candidates.toml`, sprint only.

| OSM | Symbol | Target | Notes |
| --- | --- | --- | --- |
| `railway=rail\|light_rail\|narrow_gauge` | 509.1 | map | S-Bahn, regional and main line, narrow gauge |
| the same with `service=siding\|yard\|spur` | 509.1 | map | a siding is a track; the norm has no other symbol |
| the same with `service=crossover` | none | | a short link between two main tracks, only clutter |
| the same with `bridge=yes` | 509.1 | map | the line is drawn over the road or the water |
| `railway=subway` with `bridge=yes\|viaduct` | 509.1 | map | an elevated U-Bahn |
| `railway=subway` otherwise, not in a tunnel | 509.1 | candidate | OSM often omits `tunnel` on a subway |
| `railway=tram` | 509.2 | candidate | the norm: "generally not represented" |
| `tunnel=yes\|building_passage\|culvert`, `location=underground`, `area=yes` | none | | not on the ground |
| `railway=disused\|abandoned\|razed\|proposed\|construction\|platform\|station` | none | | no track in use |

- Map object, not candidate. A railway is among the most reliable things in OSM and nothing a runner could mistake for
  something else. It is also prominent: the norm names it in 520 as one of the few features to draw in a forbidden
  area. If the ruling is wrong, the cost is a line to delete in OOM, for example a siding that is gone. The cost of the
  other way round is a map without the S-Bahn, which is what the user found. Trams stay candidates because the norm
  leaves the choice to the mapper.
- The line is clipped to the map area like the other OSM lines and needs 4 mm on paper (`norm_min_line_mm`, 16 m at
  1:4000), also for the piece inside the map area.
- Double track. Each OSM way is one track and one line. Two tracks 4.5 m apart (1.1 mm at 1:4000) leave 0.55 mm
  between two 0.55 mm lines, more than the 0.15 mm gap the norm asks between lines of one colour, so nothing is merged.
  Tracks closer than that would touch; none was found in the examples.
- Layers. `layer` is ignored. The railway lies in the same part as the paving and after it, so a railway bridge is
  drawn over the road below and the white of 509.1 opens a gap in the road edge, as in a norm drawing.
- Tree groups. ADR 0069 cuts 402 and 405 at paving and paths. 509.1 joins that list with its width on paper (0.55
  mm), so a forest never covers a track.
- Land use. ALKIS `Bahnverkehr` is unchanged: it is a 520 candidate (private land, `build/out_of_bounds.rs`), not
  paving, and draws no track, so the 509.1 line is not doubled. The line and the 520 candidate lie together, as the norm
  intends; the course setter decides whether the railway area is forbidden. The `Begleitfläche Bahnverkehr` verge stays
  left out.
- Walls, fences and the walkable-area base fill are unchanged. Points under the line keep the rules of ADR 0066 and
  0074; this change adds none.
- Dedup (ADR 0078). The cadastre has no railway line, so nothing ranks against it. The `man_made=bridge` areas and the
  embankment rules are unchanged.
- Schoolyard maps: no rule. A schoolyard map is a few hundred metres wide and the school set has no railway symbol.
  A track in the box is not drawn.

## OCAD

509.1 and 509.2 follow the decimal rule of the number table (`ocd/symbols.rs`): 509.100 and 509.200, like 501.1 and
513.1, whose IOF sub-numbers OCAD stores in hundreds. The numbers were not checked against `ISSprOM_blank.ocd`, which
is not in the repository; the starting point holds placeholders that OCAD replaces by number.

## Consequences

The counts per example are in the changelog. Existing maps get the railways with `run --force`. A map with many yard
tracks looks busy; the mapper deletes the sidings that do not matter.
