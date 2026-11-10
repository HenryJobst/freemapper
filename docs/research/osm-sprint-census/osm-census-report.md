# OSM census for SPRINT maps: what freemapper does not read yet

Method. Read-only. A scratch Rust tool (osmpbf 0.3.8, in `scratchpad/census`, not committed; target dir removed) counted
tagged nodes (N), ways with a node in the box (W) and relations with a member in the box (R) per key=value.
Boxes are the `[osm].area` of each example. Raw counts: `scratchpad/berlin.tsv`, `scratchpad/luebeck.tsv` (type, tag, count).
Sources: park = `brandenburg_2026-11-06` (Berlin is inside the Brandenburg extract; box 13.374..13.388 E, 52.485..52.491 N);
non-park = `schleswig-holstein_2026-11-07` (Luebeck Altstadt/Holstentor, sprint 1:4000, dense old town, so no fallback to Marburg).
Tagged elements: Viktoriapark N 2161 / W 2008 / R 86; Luebeck N 2365 / W 5512 / R 385.

## What is read today (state of the code)
- `read_extract` (sources/osm.rs) keeps only nodes/ways for which `Rules::wants(tags, map_kind)` holds, so only
  assets/rules/osm-candidates.toml matters for OSM (berlin-/state-candidates.toml use Geoportal/ALKIS tags, never OSM tags).
- Sprint rules read: natural=tree (418), barrier=hedge/wall/retaining_wall/fence (410.1/515/513.2/518), natural=cliff (201),
  leisure=pitch|track with surface (501.10 or 401), leisure=park, landuse=grass|meadow|village_green|recreation_ground (401),
  unpaved highway=track|path|footway|bridleway (505.1/505.2/506, map object when surface is unpaved/compacted/gravel...),
  unknown-surface paths (candidate), unpaved streets (candidate), water (natural=water, riverbank, reservoir, basin, pool,
  fountain area/node, well, spring, stream/ditch), man_made=bridge (501.5).
- Schoolyard only (not sprint): bench, picnic table, street_lamp, flagpole/mast, hydrant, playground, drinking_water.
- Not read at all: relations (the other agent's task), buildings (come from ALKIS), highway=steps/pedestrian/service (paved),
  access/foot/bicycle, barrier=gate/bollard/kerb, entrance, building:part, railway, power, memorials, natural=scrub/wood, landuse=forest/flowerbed.

## (a)+(b) Top unused groups per area, with ISSprOM 2019 mapping
Counts Park(Berlin) / Town(Luebeck). N=node, W=way. "Use" = value at 1:4000/5000.

| key=value (type) | Park | Town | ISSprOM symbol | Sprint value |
|---|---|---|---|---|
| tunnel=building_passage (W, mostly highway=footway/service) | 35 | 82 (+104 ways in network "Luebecker Hoefe und Gaenge") | 512.2 underpass / 512.3 area passable at two levels | very high in town: passages through buildings decide routes; today only a generic candidate when no paved surface tag |
| highway=steps (W) | 62 | 9 | 532 stairway | high; ALKIS stairs exist for Berlin only (sb-treppe) / some for SH; OSM adds steps and ramps in park slopes |
| access=private / no, foot=no on highway=footway/service/path (W) | 4+21+... ~38 / ... | ~35 private + 29 foot=no (ways) | 520 area that shall not be entered (or drop the 505 line) | high: private courtyards (Hoefe) and yards; today a private footway is drawn as 505.1 like a public one |
| barrier=gate, lift_gate, swing_gate (N) | 47 | 46 | 516 passable fence/railing (gap in 518), or 520 boundary | medium-high: marks whether a fence line is crossable; with access=private it closes a way |
| railway=rail/light_rail/tram (W) | 3 (+4 subway, 5 disused: ignore) | 0 | 509.1 railway / 509.2 tramway | high in general (Karlsruhe-like towns), low in these two boxes |
| natural=scrub 76, natural=wood 29, landuse=forest 10, landuse=flowerbed 4 (W) | 119 | 0 | 408/410 (scrub), 405 forest, 520 flowerbed | high in parks: park vegetation outside leisure=park; flowerbeds are uncrossable (520) |
| bridge=yes on highway (W) | 5 | 10 | 512.1 bridge entrance / deck 501.5 | medium (Luebeck canals; walls/bridge pairs matter) |
| historic=memorial, tourism=artwork (N) | 27+5 (23 Stolpersteine) | 110+16 (95 Stolpersteine, 7 plaques) | 526 monument (not Stolpersteine/plaques: flush) | medium: ~9 real monuments town, few park; Berlin layer already covers monuments, SH ALKIS partly |
| barrier=kerb (W) + node kerb=lowered/raised/flush | 65 W / 96 N | 7 W / 7 N | 501.1 step or edge of paved area (only kerb=raised, height >= ~0.1 m) | medium; Berlin has surveyed bd_bordstein, elsewhere OSM would give a first guess |
| highway=pedestrian (W, incl. area=yes) | 0 | 31 (+3 R) | 501 paved (already ALKIS); `area:highway=*` closed rings | low: orthophoto/ALKIS cover; only surface=unpaved matters |
| barrier=bollard (N+W) / cycle_barrier | 68+9 / 8 | 6 / 0 | none at sprint (passable); 530 only if prominent | low, even harmful (2392 in Berlin km2) |
| highway=street_lamp (N) | 176 | 322 (+104 lamp_type=gaslight) | none in ISSprOM sprint (545 is the school symbol set) | low for sprint; only a landmark where isolated |
| amenity=bicycle_parking / waste_basket / vending_machine (N) | 34 / 115 / 64 | 88 / 77 / 24 | none (525/530 only if very prominent) | low |
| entrance=yes|main|shop (N) | 26 | 211 | none; relevant only to 520 (road entrances) and building-passage detection | low for symbols, useful as a filter |
| man_made=planter / street_cabinet (N) | 24 / 24 | 0 / 7 | 530 or 520 (planters) | low |
| amenity=parking (W), leisure=playground/bleachers (W) | 77 / 7+3 | 17 / 2 | 501 vs 520 / 403 | low-medium: parking surface colour only |
| man_made=pier (W), chimney (W), mast (N) | 2 chimney | 5 pier | 501 pier deck, 525 small tower | low |
| building:part (W) | 26+21+... ~60 | 2360 | 521 (buildings come from ALKIS) | none; only noise, must never be a candidate |
| shop/restaurant/office/tourism POIs (N) | ~150 | ~600 | none | none |
| route=bus/bicycle relations (R) | 18 | 60+ | none (hiking/pilgrimage route=hiking could help path choice but no symbol) | none |
| power=substation (W), power=cable | 3 | 4 / 2 | 521 (already building) / none | none |

(The park access count is approximate: footway private 4, service private 21+destination 19, Luebeck footway private 18, service private 10+permit 8.)

## Surface / width / access handling for footway, path, service, pedestrian
- Surface: only the unpaved list counts (unpaved, dirt, earth, ground, grass, gravel, fine_gravel, compacted, sand, woodchips).
  Everything paved or unusual (cobblestone, sett, paving_stones, pebblestone) is ignored for footway; pebblestone/other values and
  missing surface give a candidate. `compacted` is a map object. Data: Berlin footway paving_stones 153, asphalt 124, compacted 43
  (=> 43 map lines 505.1), none 20 (candidates), sand/fine_gravel 8; Luebeck footway paving_stones 148, sett 50, none 34, compacted 6.
- Width: only used as `width<1.5` -> 506 (less distinct path) on unpaved footway/path/track/bridleway. footway width is tagged on only
  23/380 (Berlin) and 7/312 (Luebeck) ways, so 506 vs 505.1 is practically decided by the default. Width is never used for service or
  pedestrian and not for steps (5/62 tagged).
- Access/foot/bicycle: never read for ways (only `access=private` in a swimming-pool test). A private/no-access footway or service way
  is mapped like a public one. highway=service has no sprint rule at all except unpaved surface (candidate 505.2).
- tunnel / covered / indoor: read as exclusion for the map-object rules (passages fall through to a candidate), tunnel=building_passage
  gets no own symbol. layer is not read.
- highway=pedestrian, highway=steps, highway=service (paved): no rule. highway=footway area=yes: excluded from map rules, candidate only.

## (c) Biggest differences park vs. town
- Park-heavy: natural=scrub/wood, landuse=forest/flowerbed (119 vs 0), highway=steps (62 vs 9), barrier=kerb ways (65 vs 7),
  bollards (77 vs 6), natural=tree (535 vs 103, already used), compacted footways (43 vs 6), man_made=planter, leisure=pitch/playground.
- Town-heavy: tunnel=building_passage (35 vs 82) plus the Hoefe-und-Gaenge network, entrances (26 vs 211), Stolpersteine/memorials (27 vs 110),
  access=private/foot=no ways (about equal in absolute count, but 5x higher share of 5.5k ways in town for passages), gates (47 vs 46, equal),
  building:part (60 vs 2360), POIs (shops, restaurants, doctors), bicycle parking (34 vs 88), gaslights, hydrants (15 vs 81, mostly underground: excluded already).
- Park-only sampling would therefore overrate scrub/steps/kerb and miss passages, private courtyards, entrances and memorials.

## (d) Recommended 8 additions, ranked (value / effort)
1. Building passages: `tunnel=building_passage` on highway ways -> candidate (target candidates) 512.2 line + optional 512.3 corridor area.
   Effort: rule-only for a candidate line in osm-candidates.toml (fix the current paved-surface gap: today the generic rule only fires if surface is not paved);
   a real 512.3 area needs buffering code (small, near paved.rs/stairs.rs).
2. Access restrictions on ways: highway=footway|path|service|pedestrian with access=private|no or foot=no|private -> do not map as 505.x, add candidate
   520 (line) / closed ring `area:highway=*`, `landuse=*` + access=private -> 520 area. Effort: rule-only for the candidate and for dropping 505 (conditions `access!=private|no`
   on existing rules); a 520 map area from lines needs code.
3. highway=steps -> 532 stairway candidate (width/step_count/incline available). Effort: rule-only as candidate; as map object code to reuse build/stairs.rs
   (direction from DGM; OSM gives the way direction up: incline=up|down).
4. Gates: barrier=gate|lift_gate|swing_gate|entrance (N) -> short candidate line (point_as_short_line_mm) 516; with access=private|no -> note for 520.
   Effort: rule-only. Gap in fence lines later needs code.
5. Park vegetation and flowerbeds: natural=scrub -> 408/410 area, natural=wood|landuse=forest -> 405 (check the template for sprint) , landuse=flowerbed|man_made=planter(area) -> 520.
   Effort: rule-only (area candidates), plus ordering vs base fill (ADR 0069 open_land.rs style drop logic) if they become map objects.
6. Railways: railway=rail|light_rail|narrow_gauge|tram (not tunnel=yes, not disused) -> 509.1 / 509.2 lines, power=line|minor_line (overhead) -> 510. Effort: rule-only.
7. Memorials/artworks: historic=memorial|monument|wayside_cross|boundary_stone, tourism=artwork with artwork_type=statue|sculpture, excluding memorial=stolperstein|plaque|stele-flush -> 526.
   Effort: rule-only (`memorial!=stolperstein|plaque` is supported). Stolpersteine count 118 in the two boxes: must be excluded or they flood the candidates.
8. Kerbs and bridge decks: barrier=kerb with kerb=raised -> 501.1 (sprint), highway=* bridge=yes -> 512.1/501.5 candidate, man_made=pier -> 501.5. Effort: rule-only; kerb needs a
   height filter to avoid noise (Berlin 65 ways in 0.4 km2).

Deliberately not recommended: bollards, street lamps, waste baskets, bicycle parking, entrances as symbols, building:part, POIs, bus routes (no sprint symbol or only noise).
Also check before 1: surface=compacted is currently a hard map object (505.1) for 49 Berlin ways, which is a larger source of false positives in parks than any missing tag.
