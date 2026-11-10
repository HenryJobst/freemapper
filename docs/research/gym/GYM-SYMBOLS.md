# Hallenkarte / gym map – symbol set proposal (v0.1, 2026-11-08)

For agreement between **freemapper** (draws the map), **freemapper-courseplan** (equipment library, courses,
barrier tape) and **freemapper-print** (print profile "Halle"). Sources and verification are in
`GYM-REFERENCE.md` next to this file.

Status of the basis: the IOF draft **ISInOM 6.6** (International Specification for Indoor Orienteering Maps,
"January 2026", DRAFT; PDF and the official OOM/OCAD symbol files were downloaded from kartarkiv.no, see
GYM-REFERENCE §1). ISInOM 6.6 has **no sports-equipment symbols**; its only gym reference is 1407 "Area with
obstacles … e.g. apparatus in a gym". The equipment and court-line symbols below are freemapper's own.

Licence: freemapper draws every symbol itself (GPL-3.0-or-later, like the school additions). The ISInOM
`.omap/.ocd` files and OCAD's `Gym_100.ocd`/`Halle_*.ocd` carry no licence statement, so nothing is copied from
them. Codes, colours (CMYK numbers) and dimensions are facts the set aligns with; OCAD's Gym list was used only
to check coverage.

---

## 1. Code scheme

**One line:** ISInOM 6.6 codes verbatim for everything ISInOM defines; freemapper's gym extensions in the
**19xx** range (1900–1989), four digits like ISInOM, OCAD number = code × 1000 with a three-place decimal.

Why 19xx:
- ISInOM numbers its own categories as 1x00 and mirrors ISOM: 1100 Construction, 1200 Floor, 1300 Water,
  1400 Furniture, 1700 Directions (≙ ISOM 700 course). It also reuses ISSprOM 3-digit codes (515, 516, 518,
  520, 521, 526, 530, 531, 301, 302, 417, 418, 512) and ISOM 7xx course codes.
- 1500 and 1600 are free today. They mirror ISOM 500 (man-made) and 600 (technical), though, so the IOF is
  the likeliest to claim them next. **1900 has no ISOM counterpart** (ISOM ends at 700), so the risk of a
  later collision is lowest there.
- No collision with ISOM/ISSprOM (3-digit), with the school set (321, 541–545) or with freemapper's sheet
  symbols 991–994 (3-digit). OCAD's Gym set uses 3xx/6xx/8xx; we do not follow that, because those numbers
  collide with ISOM.
- OCAD stores 1901.1 as 1_901_100 (allowed range 0.001–999 999.999).

Sub-ranges:

| Range | Content |
|---|---|
| 1900–1909 | mats, generic equipment areas |
| 1910–1929 | gymnastics apparatus |
| 1930–1939 | game equipment (goals, baskets, nets, posts, table tennis) |
| 1940–1949 | small set-up items (cones, hoops) |
| 1950–1959 | court lines (Spielfeldlinien) |
| 1960–1969 | hall fittings (floor sockets) |
| 1970–1979 | reserved (e.g. later per-sport coloured court lines) |
| 1980–1989 | set-up hazards (danger area, guy wires) |

Sub-codes `.1`, `.2` are **presets** (standard footprints) or variants of the main code. OOM code text is
written exactly as in this table. OCAD numbers follow freemapper's decimal rule (`1901.1` → 1901.100), except for the
ISInOM sub-codes listed in §7.

---

## 2. Scale and size rules

- **Reference scale 1:200.** All sizes below are on paper at 1:200. Allowed project scales are 1:100–1:500.
- **Factor over ISInOM: 1.5.** Paper sizes of ISInOM symbols are the ISInOM 1:700 values × 1.5.
  - ISInOM 3.1 allows enlargement ("symbols may only be enlarged, not reduced"); the base scale is 1:700.
  - With ×1.5 the outer wall (1102) is 0.84 mm, which is 17 cm on the ground at 1:200: about the width of
    a real wall.
  - Course symbols (701–711, 1720, 1771–1775) are **not** enlarged (ISInOM 1:700 sizes, ×1.0). Open
    question Q1 for courseplan.
- **Two size classes:**
  - **P = paper-fixed.** Line widths, signs (pillars, posts, toilets), texts, north lines. Same mm at
    every scale from 1:100 to 1:500 (the ISInOM rule "do not reduce").
  - **G = ground-true.** Footprints of equipment presets and the dash/gap of dashed court lines. Defined in
    metres. Paper size = metres × 1000 / S (S = scale denominator), i.e. ×5 mm/m at 1:200, ×10 at 1:100 and
    ×2 at 1:500.
    - freemapper scales the geometry of G symbols by 200/S when it writes the set for a project.
    - Line widths inside a G symbol stay P.
    - Clamp: no footprint side below 0.6 mm on paper (equipment stays legible at 1:500).
- Example: gym mat 2.00 × 1.00 m (1901.1) is 20 × 10 mm at 1:100, 10 × 5 mm at 1:200 and 4 × 2 mm at 1:500.
  Its outline stays 0.24 mm throughout.
- Equipment drawn as **areas** (1900–1902) is drawn to its true outline by the mapper and needs no scaling.
- Minimum gaps (ISInOM 3.3, ×1.5):
  - passage between uncrossable symbols: 0.60 mm;
  - door gap in an outer wall: 1.65 mm; in an inner wall: 1.35 mm;
  - gap between same-colour lines: 0.23 mm.

---

## 3. Colours (priority order, top first)

CMYK in %. "IOF R4" = IOF Map Specifications, Printing and Colour Definitions, Revision 4 (Sep 2024, valid
from 1 Dec 2024). "ISInOM" = value in the official ISInOM 6.6 `.omap` (no IOF R4 counterpart).

| # | Colour | C | M | Y | K | Source | Used by |
|---|---|---|---|---|---|---|---|
| 0 | Purple for course overprint (upper) | 35 | 85 | 0 | 0 | IOF R4 | 709, 711 (courseplan) |
| 1 | White for course overprint | 0 | 0 | 0 | 0 | IOF R4 | course symbols |
| 2 | Dark purple for barrier tape | 45 | 100 | 0 | 0 | ISInOM (1775) | 1775 |
| 3 | Orange for directions and danger | 0 | 60 | 100 | 0 | ISInOM (= IOF R4 "Orange for MTBO") | 1771, 1772, 1980 |
| 4 | White under directions | 0 | 0 | 0 | 0 | ISInOM | 1772 |
| 5 | Black 100% | 0 | 0 | 0 | 100 | IOF R4 | walls, outlines, signs, 601 |
| 6 | Brown 100% | 25 | 75 | 100 | 0 | IOF R4 | 1201, 1202, 1205 |
| 7 | Blue 100% | 100 | 0 | 0 | 0 | IOF R4 | 1302–1304 |
| 8 | Blue 70% | 70 | 0 | 0 | 0 | IOF R4 | 1104 |
| 9 | Middle red (stairs up) | 0 | 90 | 90 | 0 | ISInOM | 1206 |
| 10 | Middle blue (stairs down) | 70 | 35 | 0 | 0 | ISInOM | 1207 |
| 11 | Lower purple for course overprint | 35 | 85 | 0 | 0 | IOF R4 | course lines (courseplan) |
| 12 | Light green (table) | 20 | 0 | 25 | 0 | ISInOM | 1401, 1932 |
| 13 | Black 20% (high equipment) | 0 | 0 | 0 | 20 | IOF R4 | 1402, 1900, 19xx high |
| 14 | Blue 50% (thick mats) | 50 | 0 | 0 | 0 | IOF R4 | 1902 |
| 15 | Blue 30% (thin mats) | 30 | 0 | 0 | 0 | IOF R4 | 1901 |
| 16 | White for low equipment | 0 | 0 | 0 | 0 | freemapper | 1903 inner, 1920 |
| 17 | Black 50% for court lines | 0 | 0 | 0 | 50 | IOF R4 | 1950–1957, 1105, 1106, 1407 |
| 18 | Olive green | 38 | 27 | 100 | 0 | IOF R4 | 520 |
| 19 | Black 50% for building body | 0 | 0 | 0 | 50 | IOF R4 | 521 |
| 20 | Black 50% temporary | 0 | 0 | 0 | 50 | ISInOM (colour 902) | 1101 |
| 21 | Brown 15–25% (floor) | 3 | 11 | 20 | 0 | ISInOM | 1203 |
| 22 | White (out of bounds) | 0 | 0 | 0 | 0 | ISInOM | 1222 |

Notes:
- Equipment fills sit **above** the court lines (equipment stands on the lines), and court lines sit above
  the floor and the building body. That is why Black 50% appears three times.
- **Mats are blue** (thin 30%, thick 50%), not yellow. Yellow 50% on the beige floor (3/11/20/0) has too
  little contrast, and real gym mats are mostly blue.
  - Water areas 301/302 are **left out** of the gym set (swimming halls are out of scope), so blue is free
    for mats.
  - Open question Q4.
- Brown: IOF R4 changed brown to 25/75/100/0. The ISInOM `.omap` still uses 4/50/95/15, and freemapper's
  ISSprOM set (from OOM) 0/56/100/18. The gym set follows IOF R4.
- Course purple: IOF R4 is 35/85/0/0, the ISInOM `.omap` uses 20/100/0/0. The gym set follows IOF R4, as
  ISSprOM does (Q2 for print).

---

## 4. Symbols

Columns:
- **Code** and **OCAD**: OOM code and OCAD number.
- **Type**: P point, L line, A area, A+L combined (area with its outline line), T text.
- **Size @1:200**: in mm. P/G marks the size class from §2.
- **CP**: needed in courseplan's equipment library (E) or drawn by courseplan (C).
- **PR**: print must recognise it.

### 4.1 Construction (ISInOM 1100 + ISSprOM codes)

| Code | OCAD | English | Deutsch | Type | Colour | Size @1:200 | ISInOM | CP | PR |
|---|---|---|---|---|---|---|---|---|---|
| 521 | 521.000 | Building body (not accessible) | Gebäudekörper (nicht zugänglich) | A | Black 50% bldg | outline by 1102/1103 | 521 | | |
| 1101 | 1101.000 | Temporarily closed room / hall part | Zeitweise gesperrter Raum / Hallenteil | A | Black 50% temp. | – | 1101 | C | yes |
| 1102 | 1102.000 | Outer wall | Außenwand | L | Black | 0.84 P; door gap ≥ 1.65 | 1102 | | yes (hall frame) |
| 1103 | 1103.000 | Inner wall | Innenwand | L | Black | 0.45 P; door gap ≥ 1.35 | 1103 | | |
| 1103.1 | 1103.100 | Temporarily hidden inner wall | Zeitweise verdeckte Innenwand | L | Black | 0.45 P | 1103.1 | | |
| 1104 | 1104.000 | Self-closing door | Selbstschließende Tür | L | Blue 70% | 0.45 P | 1104 | | |
| 1105 | 1105.000 | Low wall, parapet | Niedrige Wand, Brüstung | L | Black 50% | 0.38 P | 1105 | | |
| 1106 | 1106.000 | Curtain or flexible wall (dividing curtain, ball-catch net) | Vorhang, bewegliche Wand (Trennvorhang, Ballfangnetz) | L | Black 50% | dots Ø 0.48, every 6.0 P | 1106 | C (raised/lowered per set-up) | yes |
| 515 | 515.000 | Uncrossable wall (free-standing) | Unpassierbare Mauer | L | Black | 0.60 P | 515 | | |
| 516 | 516.000 | Passable railing | Passierbares Geländer | L | Black | 0.32, ticks every 5.63 P | 516 | | |
| 518 | 518.000 | Uncrossable railing | Unpassierbares Geländer | L | Black | 0.60, ticks every 5.63 P | 518 | | |
| 1107 | 1107.000 | Small pillar (< 0.5 m) | Kleine Säule | P | Black | Ø 1.05 P | 1107 | | |
| 1108 | 1108.000 | Large pillar (> 0.5 m) | Große Säule | P | Black | Ø 2.37 P (draw true size if larger) | 1108 | | |
| 1109 | 1109.000 | Square pillar | Eckige Säule | P | Black | 0.84 × 0.84 P | 1109 | | |
| 520 | 520.000 | Area that shall not be entered (permanent) | Fläche, die nicht betreten werden darf (dauerhaft) | A+L | Olive green + 520.2 | – | 520 | | **yes** |
| 520.2 | 520.200 | its outline | Umrandung | L | Black | 0.30 P | 520.2 | | yes |
| 1111 | 1111.000 | Outline of floor above (gallery) | Umriss Geschoss darüber (Galerie) | L | Black | 0.23, dash 0.38/0.45 P | 1111 | | |

### 4.2 Floor, levels, water (ISInOM 1200/1300)

| Code | OCAD | English | Deutsch | Type | Colour | Size @1:200 | ISInOM | CP | PR |
|---|---|---|---|---|---|---|---|---|---|
| 1203 | 1203.000 | Floor (runnable), the hall floor | Boden (belaufbar), Hallenboden | A | Brown 15–25% | – | 1203 | | |
| 1201 | 1201.000 | Step | Stufe | L | Brown | 0.32 P | 1201 | | |
| 1202 | 1202.000 | Stage edge / steep stairway | Bühnenkante / steile Treppe | L | Brown | 0.45, ticks 1.05 P | 1202 | | |
| 1205 | 1205.000 | Stairs not to another floor (stands) | Treppe ohne Geschosswechsel (Tribüne) | A | Brown hatch | lines 0.36 every 0.92 P | 1205 | | |
| 1206 | 1206.000 | Stairs to the floor above | Treppe nach oben | A | Middle red | – | 1206 | | |
| 1207 | 1207.000 | Stairs to the floor below | Treppe nach unten | A | Middle blue | – | 1207 | | |
| 1222 | 1222.000 | Out-of-bounds area (white) | Nicht belaufbarer Bereich (weiß) | A | White | – | 1222 | | yes |
| 1223 | 1223.000 | Distinct floor marking | Markante Bodenmarkierung | L | Black | 0.15 P | 1223 | | |
| 1302 | 1302.000 | Toilet | Toilette | P | Blue | ISInOM × 1.5 P | 1302 | | |
| 1303 | 1303.000 | Shower | Dusche | P | Blue | ISInOM × 1.5 P | 1303 | | |
| 1304 | 1304.000 | Sink | Waschbecken | P | Blue | ISInOM × 1.5 P | 1304 | | |

### 4.3 Furniture and features (ISInOM 1400 + ISSprOM 5xx)

| Code | OCAD | English | Deutsch | Type | Colour | Size @1:200 | ISInOM | CP | PR |
|---|---|---|---|---|---|---|---|---|---|
| 1401 | 1401.000 | Table, low surface ≤ 1.10 m | Tisch, niedrige Ablage | A | Light green | outline 1404 | 1401 | E | |
| 1402 | 1402.000 | Cupboard, tall shelf (equipment cabinet) | Schrank, hohes Regal (Geräteschrank) | A | Black 20% | outline 1404 | 1402 | | |
| 1404 | 1404.000 | Furniture / equipment outline | Möbel-/Geräteumriss | L | Black | 0.24 P | 1404 | E | |
| 1407 | 1407.000 | Area with scattered obstacles (e.g. equipment room) | Bereich mit verstreuten Hindernissen (z. B. Geräteraum) | A | Black 50% dots | Ø 0.83 every 1.95 P | 1407 | E | |
| 1408 | 1408.000 | Area with dense obstacles | Bereich mit dichten Hindernissen | A | Black 50% dots | Ø 0.83 every 1.17 P | 1408 | E | |
| 530 | 530.000 | Prominent feature – ring | Besonderes Objekt – Ring | P | Black | Ø 1.44, line 0.24 P | 530 | E | |
| 531 | 531.000 | Special feature – x | Besonderes Objekt – x | P | Black | ISInOM × 1.5 P | 531 | E | |

### 4.4 Equipment – freemapper extension (19xx)

Shared drawing rules: every equipment symbol has a black outline of 0.24 mm (= 1404). The fill shows how a
runner can treat the object:
- **Black 20%:** "high, go around".
- **Blue 30% / 50%:** soft mats you may step on.
- **White:** low and hard.
- **No fill, dashed outline:** overhead or hanging; you can pass underneath.

Preset footprints marked "(typ.)" are typical catalogue sizes. They still need checking against the DIN/EN
standards, which were not readable (GYM-REFERENCE §4).

| Code | OCAD | English | Deutsch | Type | Colour / look | Size @1:200 | Ground | CP | PR |
|---|---|---|---|---|---|---|---|---|---|
| 1900 | 1900.000 | Equipment, high (generic) | Gerät, hoch (allgemein) | A+L | Black 20% + outline | true outline | – | E | |
| 1901 | 1901.000 | Mat, thin (gym mat, floor mat) | Matte, dünn (Turnmatte, Bodenturnmatte) | A+L | Blue 30% + outline | true outline | – | E | |
| 1901.1 | 1901.100 | Gym mat 2.00 × 1.00 | Turnmatte 2,00 × 1,00 | P | as 1901 | 10.0 × 5.0 G | DGUV | E | |
| 1901.2 | 1901.200 | Gym mat 2.00 × 1.25 | Turnmatte 2,00 × 1,25 | P | as 1901 | 10.0 × 6.25 G | DGUV | E | |
| 1901.3 | 1901.300 | Children's gym mat 1.50 × 1.00 | Kinderturnmatte 1,50 × 1,00 | P | as 1901 | 7.5 × 5.0 G | DGUV | E | |
| 1902 | 1902.000 | Mat, thick (soft landing, landing mat) | Matte, dick (Weichboden-, Niedersprungmatte) | A+L | Blue 50% + outline | true outline | – | E | |
| 1902.1 | 1902.100 | Soft landing mat 3.00 × 2.00 | Weichbodenmatte 3,00 × 2,00 | P | as 1902 | 15.0 × 10.0 G | (typ.) | E | |
| 1902.2 | 1902.200 | Landing mat 2.00 × 1.50 | Niedersprungmatte 2,00 × 1,50 | P | as 1902 | 10.0 × 7.5 G | (typ.) | E | |
| 1903 | 1903.000 | Mat trolley | Mattenwagen | P | Black 20% frame, Blue 30% inset 0.15 m | 10.5 × 5.5 G | 2.10 × 1.10 (typ.) | E | |
| 1904 | 1904.000 | Equipment, low (generic) | Gerät, niedrig (allgemein) | A+L | White + outline | true outline | – | E | |
| 1910 | 1910.000 | Vaulting box | Sprungkasten | P | Black 20%, inner line along the top | 8.0 × 4.0 G | 1.60 × 0.80 base (typ.) | E | |
| 1911 | 1911.000 | Small box | Kleiner Kasten | P | Black 20% | 4.0 × 2.5 G | 0.80 × 0.50 (typ.) | E | |
| 1912 | 1912.000 | Buck | Bock | P | body Black 20% 3.0 × 1.75 + 4 leg ticks | 5.0 × 3.5 G | 1.00 × 0.70 footprint (typ.) | E | |
| 1913 | 1913.000 | Horse (pommel or vaulting) | Pferd (Pauschen-/Sprungpferd) | P | body Black 20% 8.0 × 1.75, 2 pommel ticks 2.25 apart, legs | 8.0 × 5.0 G | body 1.60 × 0.35 (FIG), footprint 1.60 × 1.00 (typ.) | E | |
| 1914 | 1914.000 | Vaulting table | Sprungtisch | P | Black 20% | 6.0 × 4.75 G | 1.20 × 0.95 top (FIG) | E | |
| 1915 | 1915.000 | Parallel bars | Barren | P | 2 rails 0.24 black, 17.5 long, 2.5 apart; base Black 20% 11.5 × 5.0 | 17.5 × 5.0 G | rails 3.50, 0.42–0.52 apart (FIG); base 2.30 × 1.00 (typ.) | E | |
| 1916 | 1916.000 | High bar | Reck | L | bar 0.24 black, post dots Ø 0.6 P at the ends | user length; preset 12.0 G | 2.40 (FIG) | E | |
| 1916.1 | 1916.100 | Guy wire | Abspannseil | L | Black, 0.12 dash 0.5/0.3 P | as drawn | – | E | yes (hazard) |
| 1917 | 1917.000 | Balance beam | Schwebebalken | L | 0.50 G Black 20% with 0.12 P borders, feet ticks | preset 25.0 G | 5.00 × 0.10 (FIG) | E | |
| 1918 | 1918.000 | Rings (hanging) | Ringe (hängend) | P | 2 circles Ø 0.9, line 0.18 P, centres 2.5 G apart | – | 0.50 rope axes (DIN A.2) | E | |
| 1919 | 1919.000 | Mini trampoline | Minitrampolin | P | frame Black 20%, bed white with diagonal cross 0.12 | 6.0 × 6.0 G | 1.20 × 1.20 (typ.) | E | |
| 1920 | 1920.000 | Springboard | Sprungbrett | P | White fill, high end 0.35 black | 6.0 × 3.0 G | 1.20 × 0.60 (typ.) | E | |
| 1921 | 1921.000 | Bench | Langbank (Turnbank) | L | 1.25 G Black 20% main line, black borders 0.12 P, end caps | preset 17.5 G | width 0.25, length 3.50 (typ.; also 2.00/4.00) | E | |
| 1921.1 | 1921.100 | Bench, upside down (balancing) | Langbank, umgedreht | L | as 1921 + centre line 0.12 | as 1921 | – | E | |
| 1922 | 1922.000 | Wall bars | Sprossenwand | L | 0.24 black on the wall face, ticks 0.75 into the room every 5.0 G | – | element 0.90–1.00 wide (typ.) | E | |
| 1923 | 1923.000 | Climbing poles | Kletterstangen | L | no line; dots Ø 0.6 P every 3.0 G | – | ≥ 0.50 apart, ≥ 1.0 from the wall (DGUV) | E | |
| 1924 | 1924.000 | Climbing ropes | Klettertaue | L | no line; rings Ø 0.6, line 0.15 P every 3.0 G | – | – | E | |
| 1925 | 1925.000 | Hurdle | Hürde | L | 0.24 black, feet ticks 2.0 G at both ends | preset 5.0 G | 1.00 (typ.; athletics 1.18–1.20) | E | |
| 1930 | 1930.000 | Goal (handball, futsal) | Tor (Handball, Futsal) | P | posts 0.35 black, frame/net U-shape 0.18 black | 15.4 × 5.0 G | 3.00 inside + 2 × 0.08 posts, depth 1.00 (IHF/FIFA; depth typ.) | E | |
| 1931 | 1931.000 | Basketball backboard and basket | Basketballkorb mit Brett | P | board 0.30 black 9.0 G; ring Ø 2.25 G line 0.18; anchor at the floor point below the ring centre, board 1.875 G behind it | 9.0 × 3.0 G | board 1.80 (FIBA); ring centre 0.375 in front of the board (FIBA 2.5.4/2.5.7) | E | |
| 1932 | 1932.000 | Table tennis table | Tischtennisplatte | P | Light green + outline, net line 0.12 | 13.7 × 7.63 G | 2.74 × 1.525 (DIN 18032-1 Tab. A.1) | E | |
| 1933 | 1933.000 | Net (volleyball, badminton) | Netz (Volleyball, Badminton) | L | Black 0.12, dash 0.6/0.3 P | as drawn | top 2.43/2.24 m (FIVB), 1.55 m (BWF) | E | yes |
| 1934 | 1934.000 | Post, stand (net post, slalom pole) | Pfosten, Ständer | P | Black dot Ø 0.6 P | – | – | E | |
| 1940 | 1940.000 | Cone | Hütchen, Pylone | P | Black triangle outline 0.20, side 1.2 P | – | – | E | |
| 1941 | 1941.000 | Hoop (lying) | Reifen (liegend) | P | circle line 0.18 black | Ø 4.0 G | Ø 0.80 (typ.) | E | |

### 4.5 Court lines – freemapper extension (195x)

A neutral, thin look on purpose: court lines are background and frequent control sites, but they must not
compete with walls and equipment. One code per sport, so OOM, courseplan and print can show or hide each
sport separately. The DIN colours (badminton green, basketball black, handball/football orange, volleyball
blue; DIN 18032-1 Tab. A.3) are **not** used on the map (Q5).

| Code | OCAD | English | Deutsch | Type | Colour | Size @1:200 | CP | PR |
|---|---|---|---|---|---|---|---|---|
| 1950 | 1950.000 | Court line (generic) | Spielfeldlinie (allgemein) | L | Black 50% court | 0.12 P | E (control site) | yes (hide/show) |
| 1951 | 1951.000 | Court line, handball | Spielfeldlinie Handball | L | Black 50% court | 0.12 P | E | yes |
| 1952 | 1952.000 | Court line, basketball | Spielfeldlinie Basketball | L | Black 50% court | 0.12 P | E | yes |
| 1953 | 1953.000 | Court line, volleyball | Spielfeldlinie Volleyball | L | Black 50% court | 0.12 P | E | yes |
| 1954 | 1954.000 | Court line, badminton | Spielfeldlinie Badminton | L | Black 50% court | 0.12 P | E | yes |
| 1955 | 1955.000 | Court line, futsal | Spielfeldlinie Futsal | L | Black 50% court | 0.12 P | E | yes |
| 1956 | 1956.000 | Court line, dashed (handball 9 m line) | Spielfeldlinie gestrichelt (Handball-9-m-Linie) | L | Black 50% court | 0.12 P; dash/gap 0.75/0.75 G (0.15/0.15 m, IHF 1:5) | E | yes |
| 1957 | 1957.000 | Court spot (penalty and 10 m marks) | Spielfeldpunkt (Strafstoß-, 10-m-Marke) | P | Black 50% court | Ø 0.40 P | E | yes |

Notes:
- The `add-court` generator draws the **centre line of each painted line**.
  - The rules measure to line edges (FIBA: inner edge; IHF: rear edge of the goal line; volleyball,
    badminton and futsal: lines belong to the area they enclose).
  - The generator offsets by half the real line width: 2.5 cm, badminton 2 cm, futsal 4 cm, handball goal
    line between the posts 4 cm.
- Short marks (handball 7 m and 4 m lines, substitution marks; basketball throw-in lines; volleyball
  service-zone marks) are short line objects with the sport's code.

### 4.6 Hall fittings

| Code | OCAD | English | Deutsch | Type | Colour | Size @1:200 | CP | PR |
|---|---|---|---|---|---|---|---|---|
| 1960 | 1960.000 | Floor socket (cover) | Bodenhülse (Deckel) | P | Black, hollow square, line 0.12 | 0.60 × 0.60 P | E (control site) | |

The dividing curtain is 1106; the gallery is 1111 + 1205/1206; the equipment room is 521, 1101 or 1407.

### 4.7 Course, directions, hazards (drawn by courseplan; listed for code agreement)

The gym starter set contains these symbols, so stage 1 (one OOM copy per equipment set-up) can draw them by
hand. Courseplan owns their use. Sizes are ISInOM 1:700 values **× 1.0** (Q1).

| Code | OCAD | English | Deutsch | Type | Colour | Size | Source | CP | PR |
|---|---|---|---|---|---|---|---|---|---|
| 709 | 709.000 | Out-of-bounds area | Sperrgebiet | A | Purple, crossing lines | as ISInOM 709 | ISInOM 709 | C | **yes** |
| 709.4 | 709.400 | Out-of-bounds boundary (solid) | Sperrgebietsgrenze | L | Purple | 0.34 | ISInOM 709.4 (map file 709.400; course file 709.004, see §7) | C | yes |
| 711 | 711.000 | Out-of-bounds route (forbidden route) | Verbotener Weg | L | Purple | as ISOM 2017-2 711 | ISOM 2017-2; **not in ISInOM 6.6** | C | **yes** |
| 1775 | 1775.000 | Impassable border (barrier tape) | Absperrband (unpassierbar) | L | Dark purple | 0.70 | ISInOM 1775 | C | **yes** |
| 1771.1 | 1771.100 | One-way passage, large arrow | Einbahnweg, großer Pfeil | P | Orange | ISInOM | ISInOM 1771.1 | C | yes |
| 1771.2 | 1771.200 | One-way passage, line of arrows | Einbahnweg, Pfeillinie | L | Orange | ISInOM | ISInOM 1771.2 | C | yes |
| 1772 | 1772.000 | Compulsory crossing | Pflichtdurchgang | P | Orange + white | ISInOM | ISInOM 1772 | C | yes |
| 1980 | 1980.000 | Danger area (set-up specific) | Gefahrenbereich (aufbauabhängig) | A | Orange hatch 45°, lines 0.20 every 1.0 P | – | freemapper | C | **yes** |
| 1980.1 | 1980.100 | Danger area outline | Gefahrenbereich, Umrandung | L | Orange 0.25, dash 1.5/0.5 P | – | freemapper | C | yes |

The control circle, start, finish, control numbers and course lines (701–708, 1703, 1720) are courseplan's
own and are **not** in freemapper's gym set.

### 4.8 Sheet (peripheral information)

| Code | OCAD | English | Deutsch | Type | Size | PR |
|---|---|---|---|---|---|---|
| 601 | 601.000 | Magnetic north line | Magnetische Nordlinie | L | black 0.14 P, spacing 30 mm (6 m at 1:200) | **yes** (sheet may be rotated to fit the hall) |
| 1781 | 1781.000 | Text (room names, notes) | Text (Raumnamen, Hinweise) | T | Arial 3.3 mm P (not enlarged) | |
| 991–994 | 991.000–994.000 | Title, text, small text, bar scale | Titel, Text, Kleintext, Maßstabsleiste | T/L | unchanged freemapper sheet additions | yes |

- ISInOM 3.4 says that an indoor map need not be north-oriented. freemapper still draws north lines in the
  sheet part, as on every freemapper map.
- The peripheral text reads "Maßstab 1:200" without "Äquidistanz", because gym maps have no contours.

---

## 5. What courseplan and print need (summary)

- **Courseplan equipment library (E):**
  - mats, generic equipment and apparatus: 1900–1904, 1910–1925;
  - game equipment and set-up items: 1930–1934, 1940, 1941;
  - court lines and fittings: 1950–1957, 1960;
  - furniture and features: 1401, 1404, 1407, 1408, 530, 531.
  - Preset point symbols are rotatable, and their anchor is the footprint centre. Exception: 1931, anchored
    at the floor point below the ring.
- **Courseplan draws (C):** 709, 709.4, 711, 1775, 1771.x, 1772, 1980, 1980.1, plus per set-up 1101
  (closed hall part) and 1106 (curtain lowered).
- **Print must recognise:**
  - forbidden: 520, 520.2, 709, 709.4, 711, 1775, 1980, 1980.1, 1101, 1222;
  - hazards: 1916.1, 1933;
  - court lines: 1950–1957, so the "Halle" profile can hide or show sports;
  - layout: 1102 (hall frame for fitting the sheet), 601 and 991–994.
- Map parts that freemapper writes (code names English, labels via the glossary): `hall` (Halle),
  `courts` (Spielfelder), `equipment` (Geräte, empty in the starter), `peripheral information`. Print and
  courseplan can use the part names to toggle layers (Q6).

---

## 6. Deliberately not included

- **Water areas 301/302:** swimming halls are out of scope, and blue is used for mats.
- **Multi-floor symbols** (1110, 1208–1221, 1214–1220, 512.x, 1784/1785): a sports hall is one floor. A
  gallery gets 1111/1205/1206 only.
- **Small movable items** (medicine balls, ball trolley, litter bins; listed in OCAD's Gym set). ISInOM 2.1
  says that small movable items should not be mapped. Cones and hoops (1940/1941) are the exception, because
  courseplan places them deliberately as control sites.
- **Pre-defined desks and sofas** (1401.1xx, 1402.1xx, 1403.x): office furniture.

---

## 7. OCAD numbers

The rule is code × 1000, with the part after the dot read as a three-place decimal (freemapper `ocad_number`).
`1103.1` therefore becomes 1103.100, and `1901.1` becomes 1901.100.

The official ISInOM 6.6 `.ocd` deviates for some sub-codes. Those deviations need an exception table, like the
ISSprOM `NUMBERS` table, for `MapKind::Gym` only:

| Code (ISInOM text / OOM) | OCAD in ISInOM 6.6 `.ocd` | Plain rule would give |
|---|---|---|
| 520.1 (area part of 520) | 520.001 | 520.100 |
| 520.2 | 520.200 | 520.200 (same) |
| 1221.1 | 1221.001 | 1221.100 |
| 1402.1 / 1403.1 (with outline) | 1402.001 / 1403.001 | .100 |
| 709.4 | 709.400 in `Indoor_Symbols_-_ISInOM_6.6.ocd`, but **709.004** in `Indoor_Course_Setting_6.6.ocd` | 709.400 |
| 1771.1 / 1771.2 | 1771.100 / 1771.200 | same |
| 1103.1, 1201.1 | 1103.100, 1201.100 | same |

ISInOM's two files disagree for the 709 family (map file: 709.100–709.400; course file: 709.001–709.004).
freemapper follows the map file (plain rule). Courseplan should say which numbers it writes (Q3b).

The 19xx codes are freemapper's own and use the plain rule. When OCAD replaces symbols by number from the
official ISInOM set ("Load colours and symbols from…"), every ISInOM code above maps to the official symbol.
The 19xx symbols stay freemapper's placeholders.

---

## 8. Open questions for the sister sessions

- **Q1 (courseplan):** course symbols at ISInOM 1:700 size × 1.0 (control circle ≈ 6 mm, i.e. 1.2 m at
  1:200) or × 1.5 like the map?
- **Q2 (print):** course purple 35/85/0/0 (IOF R4) or 20/100/0/0 (ISInOM `.omap`)? Brown 25/75/100/0
  (IOF R4)?
- **Q3 (courseplan):** control-description symbols for gym equipment. ISInOM adds only 7.119, 7.132–7.134
  and 7.141–7.143. Who defines column-D symbols for box, mat, bench, court line, floor socket…?
- **Q3b (courseplan):** for the 709 family, the OCAD numbers of the ISInOM map file (709.400) or of the
  course-setting file (709.004)?
- **Q4 (both):** blue mats acceptable, given that blue means water on other O-maps?
- **Q5 (print):** optional "DIN colours" print variant for court lines, or always neutral grey?
- **Q6 (both):** do the map-part names `hall`/`courts`/`equipment` serve as the layer contract? Should
  set-up copies be one `.omap` per set-up (stage 1) with only the `equipment` part differing?
- **Q7 (courseplan):** 1980 danger area in orange (ISInOM "Directions" colour) or a purple course symbol?
- **Q8 (all):** presets marked "(typ.)" need real dimensions from DIN 7908/7909/EN 916/EN 12503 or the
  school's own equipment.
