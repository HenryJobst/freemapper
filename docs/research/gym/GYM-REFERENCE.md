# Hallenkarte / gym map – sources, findings, risks (2026-11-08)

The research behind `GYM-SYMBOLS.md` and `2026-11-09-gym-maps.md`. All downloads are under
`<Arbeitsverzeichnis>`
(untrusted data; nothing in them was executed). The parsing scripts are in `../scripts/` (Python, run with `-I`).

---

## 1. ISInOM status: the text was obtained

| What | Result |
|---|---|
| EuroCarto 2026 abstract (Petrovič, Gobec, Meyer, ICA Abstracts 12, 112, CC BY 4.0) | Read (`dl/eurocarto-abstract.pdf/.txt`). The IOF Map Commission published the first draft in 2025 and has tested it at local events. **Version 6.6 is in user evaluation (IOF MC 2026).** Figure 1 is the cover: "ISInOM … January 2026 … DRAFT … Version 6.6". Its only reference for the text is https://orienteering.sport/iof/mapping/. |
| IOF consultation letter, 1 Oct 2025 (copy on rufso.ru) | Read (`dl/rufso-consultation.pdf`). Draft published under "International Specification for Indoor Orienteering Maps (ISInOM)" on the IOF mapping page, with "Indoor symbols ISInOM.ocd" / "Indoor symbols ISInOM.omap", "Indoor Course Setting.ocd" and a Purple Pen `symbols.xml`. The IOF calls it the "provisional standard" for events until May 2026. Feedback was due by **31 May 2026**. |
| orienteering.sport/iof/mapping/ and omapwiki.orienteering.sport | **Not readable: HTTP 403, Cloudflare challenge** (same as the print session's attempt). |
| **kartarkiv.no/kurs/ (Norwegian map archive, section "Innekart (ISInOM)", "IOF ISInOM gjeldende normforslag")** | **Downloaded and read.** `dl/kartarkiv/`: `ISInOM-6.6.pdf` (22 pages, created 2026-01-18, modified 2026-03-16, SHA-256 f1ac7a33…be7e), `Indoor_Symbols_-_ISInOM_6.6.omap` (OOM, scale 1:700, 62 colours, 134 symbols, SHA-256 74e60221…dc11), `Indoor_Symbols_-_ISInOM_6.6.ocd`, `Indoor_Course_Setting_6.6.ocd`. Full text in `isinom-6.6.txt`; symbol lists in `dl/isinom-omap-list.txt`, `dl/isinom-ocd-symbols.txt`, `dl/isinom-course-ocd-symbols.txt`. |
| OCAD wiki "Symbol Set Overview" | Lists "ISInOM – DRAFT, latest change 2025-09" without a licence. |
| Licence of the ISInOM files | None stated in the PDF, `.omap` (`<notes>` empty) or `.ocd`. So freemapper copies nothing and draws its own symbols aligned with the codes and dimensions. |

### What ISInOM 6.6 says (relevant extract)

- **Scale:** base scale 1:700. When enlarged, all symbols are enlarged proportionally. Symbols may only be
  enlarged, never reduced (§3.1). Dimensions are given at 1:700, with a tolerance of ±5 % (§3.3).
- **Minimum gaps** (§3.3, figure values read from page 4):
  - passage between uncrossable symbols: 0.40 mm;
  - door in an outer wall or impassable fence: 1.1 mm; door in an inner wall or passable fence: 0.9 mm;
  - gap between same-colour lines: 0.15 mm;
  - minor objects to walls: 0.03 mm;
  - gap for a runnable area: 0.7 mm;
  - a dashed line has at least two dashes.
- **North:** the map need not be north-oriented (§3.4). **Format:** at most DIN A3 (§3.6).
- **Principles:**
  - Black line width shows passability (thick = barrier).
  - Closed areas are dark grey or marked with course symbols.
  - Each floor gets its own map section, and stairs connect floors (§2).
- **Categories:**
  - 1100 Construction, 1200 Floor, 1300 Water, 1400 Furniture, 1700 Directions/map layout, 700 Course print.
  - Specific control-description symbols: 7.119, 7.132–7.134, 7.141–7.143.
  - Reused ISSprOM codes: 521, 516, 518, 520/520.2, 512.x, 301, 302, 417, 418, 526, 530, 531; also 515
    in the `.omap`.
- **Gym:** **no equipment symbols.** The only mention is 1407 "Area with obstacles … e.g. desks in a classroom
  or apparatus in a gym".
- **Line widths at 1:700** (from the `.omap`): 1102 outer wall 0.56; 1103 inner wall 0.30; 1104 0.30 (blue
  70 %); 1105 0.25; 516 0.21; 518 0.40; 515 0.40; 520.2 0.20; 1201 0.21; 1202 0.30; 1223 0.10; 1404 0.16;
  1775 barrier tape 0.70 (dark purple 45/100/0/0).
- **Point symbols at 1:700:** 1107 Ø 0.70; 1108 Ø 1.58; 1109 0.56 square; 1106 dots Ø 0.32 every 4.0;
  1407 dots Ø 0.55 every 1.30 (1408: 0.78).
- **Numbering quirks:** the two ISInOM `.ocd` files disagree for 709.x (map file 709.100–709.400, course
  file 709.001–709.004). 520.1, 1221.1, 1402.1 and 1403.1 are stored as .001 (GYM-SYMBOLS §7).

## 2. OOM issue #613 and community sets

- **github.com/OpenOrienteering/mapper/issues/613** "Add Indoor-O, School-O & Park-O symbols sets": opened
  2016-03-03, still **open**, last activity 2019-10-15, 15 comments (read via `gh api`).
  - It quotes Midtbø (Scientific Journal of Orienteering): "micro-indoor orienteering … from 1:100 to 1:500",
    typical indoor 1:500–1:1000. That supports our 1:100–1:500 range.
  - It points to Stockholm Indoor Cup (OCAD sets on Google Drive), a Latvian OOM indoor template (Facebook
    download) and OCAD's free `Halle_100/200/300.ocd` (German) sets. None of them has licence information.
- **OCAD `Gym_100.ocd` / `Halle_200.ocd`** (downloaded only to list the symbols; `dl/ocad/*-symbols.txt`).
  The list is used for coverage only, and nothing is copied. It has 72 symbols:
  - walls (441–443), floor marking 461, floor anchoring 465, doors 471;
  - mats 301–307 (incl. mat triangle, mat trolley), long bench 311/312, parallel bars 313, boxes 314/315,
    pommel horse 316, vault table 317, minitramp 318, springboard 319;
  - stick, cone, hurdles 321–323; ball trolley, goal, hoop, medicine ball 324–327;
  - table tennis 330, wall bars 331, basketball hoop 332, horizontal bar 333, pole/climbing pole 335/336;
  - start/control/finish 601–603, forbidden area 607, dangerous area 608, forbidden route 609;
    north line 890.
  - Its 3xx/6xx numbers collide with ISOM, so we do not follow them.
- **Stockholm Indoor Cup "Kartnorm Indoor"** (with Indoor Grand Prix): the page was read, but its OCAD files sit on
  Google Drive and were not read.
- **Marco Bezzi 2.5D kids' symbols** (CC BY 4.0): pictorial, so not suitable.
- No Swiss, Czech or German national indoor norm was found in the time available. Swiss sCOOL is a school
  (outdoor) set.

## 3. Court geometry (verified against current rules, read 2026-11-08)

Downloads are in `dl/rules/`. "Lines part of the area" means that dimensions are measured to the outer edges.

### 3.1 Handball – IHF Rules of the Game, edition 1 July 2025 (`dl/rules/ihf-rules-2025.pdf`, Rule 1 and "Guidelines for Playing Courts and Goals" a–g)

- **Court:** 40 × 20 m. The diagonal of the full court is 44.72 m, and each half-court diagonal is 28.28 m
  (from the outside of the corner to the outer middle of the centre line).
- **Safety zone:** ≥ 1 m along the side lines and ≥ 2 m behind the goal lines.
- **Lines:** all 5 cm wide, except the goal line between the posts, which is 8 cm. Every line belongs to the
  area it encloses.
- **Goal:** 3 m wide and 2 m high (inside), posts 8 cm square. The rear side of the posts is in line with
  the rear edge of the goal line.
- **Goal area (6 m line):**
  - a 3 m line parallel to the goal line, 6 m out (rear edge of the goal line → front edge of the line);
  - two quarter circles of r 6 m, centred at the **rear inner corner of each goal post**.
  - The arcs meet the outer goal line 15 m apart (outer distance).
- **Free-throw line (9 m):** dashed and concentric with the goal-area line, 3 m further out. Dashes and gaps
  are 15 cm each, cut square or radially and measured over the outer chord.
- **Short lines:**
  - 7 m line: 1 m long, 7 m out (rear edge of the goal line → front edge);
  - goalkeeper's restraining line (4 m): 15 cm long, at 4 m;
  - substitution line: ends 4.5 m from the centre line, with a 15 cm mark inside and 15 cm outside the side
    line.
- **New in 2025:** a throw-off area of Ø 4 m in the middle of the centre line, as a circle line or a colour
  difference. It is mandatory only for IHF events and top leagues, so freemapper makes it optional
  (`--throw-off-circle`).

### 3.2 Basketball – FIBA Official Basketball Rules 2026, valid from 1 Oct 2026 (`dl/rules/documents-corporate-fiba-official-rules-2026-v1-1.pdf`, Art. 2)

- **Court:** 28 × 15 m, measured from the **inner edge** of the boundary lines. The boundary lines are not
  part of the court. Floor ≥ 32 × 19 m (a 2 m lane all round).
- **Lines:** 5 cm wide, a solid colour in high contrast to the floor.
  - 2026 change (Art. 2.5): line colours may differ, but all boundary lines share one colour and each pair
    of markings matches.
- **Centre:** the centre line runs 0.15 m beyond each sideline. Centre circle r 1.80 m to the outer edge.
  Free-throw semicircles r 1.80 m to the outer edge, centred on the midpoint of the free-throw line.
- **Free-throw line and restricted area:**
  - Free-throw line: far edge 5.80 m from the inner edge of the endline, 3.60 m long.
  - Restricted area: outer edges 2.45 m either side of the endline midpoint, so **4.90 m wide**.
- **3-point line:**
  - parallel lines with their outer edge **0.90 m** from the inner edge of the sidelines;
  - arc **r 6.75 m** to the outer edge, centred on the floor point below the basket centre, which lies
    **1.575 m** from the inner edge of the endline midpoint.
- **No-charge semicircle:**
  - r 1.30 m to the outer edge;
  - two parallel lines, each 0.375 m long, end **1.20 m** from the inner edge of the endline, directly
    below the front edge of the backboard;
  - the ring centre therefore sits 0.375 m in front of the board.
- **Throw-in lines:** 0.15 m long, outside the court, with their outer edge 8.325 m from the inner edge of
  the nearest endline.
- **DIN 18032-1 Tab. A.1** allows 26–28 × 14–15 m. In 15 × 27 m halls a 0.5 m clearance is enough
  (footnote d).

### 3.3 Volleyball – FIVB Official Volleyball Rules 2025–2028 (EN v05, `dl/rules/fivb-2025-2028-en.pdf`, rules 1.1–1.4, 2.5)

- **Court:** 18 × 9 m. The free zone is ≥ 3 m on all sides (FIVB/World: 5 m from the side lines and 6.5 m
  from the end lines). Free height ≥ 7 m.
- **Lines:** 5 cm wide, a light colour different from the floor and from other lines (FIVB competitions:
  white). Side and end lines are drawn **inside** the court dimensions.
- **Centre line:** its axis divides the court into two 9 × 9 m courts.
- **Attack line:** rear edge 3 m from the centre-line axis. In FIVB/World competitions it is extended by
  five 15 cm dashes (5 cm wide, 20 cm apart) to a total of 1.75 m.
- **Service zone:** two 15 cm marks, 20 cm behind the end line, in line with the side lines.
- **Posts:** 0.50–1.00 m outside the side lines. Net height 2.43 m (men), 2.24 m (women).

### 3.4 Badminton – BWF Laws of Badminton, Law 1 and Diagram A (`dl/rules/bwf-booklet-2021.pdf`, Diagram A image `dl/rules/bwfimg/diagA-000.jpg`; Law 1 text identical on worldbadminton.com/rules)

- **Lines:** 40 mm wide, easily distinguishable, preferably white or yellow. Lines are part of the area they
  define.
- **Court:** 13.40 × 6.10 m overall. Diagonal 14.723 m.
- **Across (in mm):** 40 | 420 | 40 | 2530 | 40 | 2530 | 40 | 420 | 40.
  - Singles sideline: outer edge 0.46 m in from the doubles outer edge, so the singles court is
    **5.18 m** wide.
- **Along each half (in mm), from the net to the back:** 1980 | 40 (short service line) | 3880 | 40 (doubles
  long service line) | 720 | 40 (back boundary).
  - Short service line: near edge **1.98 m** from the net.
  - Doubles long service line: rear edge **0.76 m** inside the outer edge of the back boundary (0.72 m gap
    + 0.04 m line). This resolves the 0.72/0.76 discrepancy in secondary sources.
- **Posts:** 1.55 m high, standing on the doubles side lines. Net top 1.524 m at the centre.
- **Caveat:** BWF's site says that new Laws take effect on **4 Jan 2027**. It was not checked whether Law 1
  changes (open question R5).

### 3.5 Futsal – FIFA Futsal Laws of the Game 2025–26, effective 5 Sep 2025 (copy hosted by AIFF, `dl/rules/futsal-lotg-2025-26.pdf`, Law 1)

- **Size:**
  - non-international: length 25–42 m, width 16–25 m;
  - international: 38–42 × 20–25 m;
  - the touchline must be longer than the end line.
- **Lines:** all 8 cm wide. Measurements are taken from the outside of the lines (lines belong to the area).
- **Centre:** centre mark r 6 cm; centre circle r 3 m.
- **Penalty area:**
  - two imaginary 6 m lines from the outside of each goalpost, at right angles to the end line;
  - at their ends, quarter circles of **r 6 m from the outside of the goalpost**, turned towards the touch
    lines;
  - the arcs are joined by a **3.16 m** line parallel to the goal line (3 m + 2 × 8 cm posts).
- **Marks:**
  - penalty mark 6 m from the midpoint between the posts, r 6 cm;
  - 10 m mark, r 6 cm, with marks 5 m either side of it, r 4 cm;
  - 5 m mark (8 × 40 cm) in the penalty area, 5 m from the 10 m mark (DFKSAF);
  - corner arcs r 25 cm; corner marks 5 m from the arc, outside the pitch, 8 × 40 cm, 5 cm gap from the end
    line.
- **Substitution zones:** 5 m long, starting 5 m from the halfway line. Each end is marked with an
  80 × 8 cm line, 40 cm on and 40 cm off the pitch.
- **Goals:** 3 m × 2 m inside; posts and crossbar 8 cm.
- In German halls futsal often shares the handball court (40 × 20). freemapper uses 40 × 20 as the default
  and accepts `--size`.

## 4. DIN 18032-1 and equipment

Source: the DIN-Taschenbuch 134/1 preview (`dl/din/din-taschenbuch-preview.pdf`, publicly hosted on
pageplace.de). It contains **DIN 18032-1:2014-11** Table 1 and Annex A (informative), Tables A.1–A.4.

- **Table 1, clear minimum sizes (width × length × height, m):**
  - Einzelhalle **15 × 27 × 5.5**.
  - Zweifachhalle **22 × 45 × 7**, if needed divisible into 3: 22.5 × 22 + 22.5 × 22, 22 × 27 + 22 × 18, or
    22 × 15 + 22 × 30.
  - Dreifachhalle **27 × 45 × 7**, divisible into 3 (thirds of 15 × 27); preferred type for spectators.
  - Eineinhalbfache Halle 18 × 36 × 7 (18 × 24 + 18 × 12).
  - Other proven sizes: 10 × 10 × 4, 12 × 12 × 5, 15 × 15 × 5.5, 15 × 20 × 5.5.
- **The brief's "double 22 × 44" differs from the 2014 text (22 × 45).**
  - 22 × 44 appears in the 2014 standard only as a threshold: "existing halls smaller than 22 m × 44 m"
    (Tab. A.1 footnote h). It is also the handball total area (44 × 22).
  - It is probably the size from the 2003 edition (not verified).
  - Ruling in the plan: the preset is 22 × 45, and a 22 × 44 hall matches the preset as well.
- **Table A.1** (sizes and additional obstacle-free clearance, long sides / end sides):
  - badminton 13.4 × 6.1, 0.3 / 0.8 m (0.65 m to a curtain, 1.30 m between two courts);
  - basketball 26–28 × 14–15, 2 / 2 m (0.5 m in 15 × 27 halls);
  - futsal ("Fußball") 30–50 × 15–25, regular size 40 × 20, 0.5 / 2 m;
  - handball 40 × 20, 1 / 2 m (existing halls < 22 × 44: 0.5 / 1 m);
  - table tennis 2.74 × 1.525 (table size), box 12 × 6 (14 × 7 in the Bundesliga);
  - volleyball 18 × 9, 3 / 3 m;
  - apparatus gymnastics 29 × 16 for practice;
  - trampoline 7.57 × 3.03.
- **Table A.3, line marking (width, colour):**
  - badminton 4 cm, green; basketball 5 cm, black; football/handball/hockey 5 cm, orange; volleyball 5 cm,
    blue.
  - Contrast K ≥ 0.4 to the floor (DIN 32975).
  - The main court is lined continuously. Lines of secondary courts are interrupted where they cross the
    main court. Secondary transverse courts may use an alternative colour, dashes, a thinner line or reduced
    lines.
  - Practice courts may be narrower, double lines or corner marks only.
  - The same table appears in DGUV "Sichere Schule – Sporthalle" (`dl/din/sish-sporthalle.pdf`) and in
    Kübler's blog.
- **Table A.2, safety areas for apparatus** (DTB):
  - Barren: 6 × 9.5 m area, 2.0 m to the side, 4 m forward, 3 m back;
  - Reck/Spannreck: 12 × 6 m area, 1.5 m side, 6 m forward and back;
  - Schaukelringe: 1.5 m side, 10.5 m forward, 7.5 m back;
  - Klettertaue: 1.5 m side, 4.5 m forward and back, 1 m between ropes;
  - Sprossenwände (swung out): 4 m, reducible to 2 m at curtains.
  - These are candidates for the 1980 danger area, which courseplan places.
- **Detailed court layouts:** Annex A's figures are **not** in the preview. The full-hall presets are
  therefore freemapper's centred, symmetric suggestions, not copies of a DIN figure.
- **Equipment** (DGUV "Sichere Schule – Sporthalle"):
  - Turnmatte 2.00 m long (children's 1.50), 1.25 or 1.00 m wide, 0.06/0.08 m thick;
  - Bodenturnmatte 3.5 cm thick, rolled;
  - Niedersprungmatte 15–20 cm thick;
  - climbing poles ≥ 1.0 m from the wall and ≥ 0.50 m apart;
  - ropes about 0.40 m above the floor;
  - basketball backboard clearance to the wall: class A 3.25, B 2.25, C 1.65 (enough for school sport),
    D 1.20, E 0.6–1.2 m.
- **Not verified** (paid standards; searches found tables of contents only):
  - Sprungkasten (DIN 7908 / EN 916; DIN 7908 is being partly replaced by EN 17461:2025-07);
  - Turnbank (DIN 7909), Weichboden, Minitrampolin, Sprungbrett, Bock.
  - These sizes are marked "(typ.)" in GYM-SYMBOLS.
- **FIG apparatus** (from memory, not re-read): pommel horse body 1.60 m, parallel bars rails 3.50 m,
  high bar 2.40 m, beam 5.00 × 0.10 m, vault table top 1.20 × 0.95 m. These need a check against the FIG
  Apparatus Norms.

## 5. Repository findings (freemapper, main @ b575867)

- **Symbol sets:**
  - `omap/symbol_set.rs` embeds the OOM file (`include_str!`). `SymbolSet::from_omap_xml` extracts colours,
    symbols, grid and the code → id table.
  - `with_additions(xml)` appends one symbol per line with `id="@"`; `enlarge_symbols` scales a set (school
    125 %); `with_sheet_symbols()` adds 991–994.
  - `MapKind::symbol_set(self)` depends on the kind only. The gym set needs the scale (G-class footprints),
    so it needs a new entry point, e.g. `ProjectFile::symbol_set()` or `MapKind::symbol_set_at(scale)`.
- **OCAD numbers** (`ocd/symbols.rs`):
  - `ocad_number(code, kind)` applies the decimal rule plus the `NUMBERS` exception table and a school
    special case (410.1).
  - `Numbers::assign` falls back to 999_000+ for non-numeric codes. A letter range ("G01") would therefore
    lose stable OCAD numbers, which is one more reason for numeric 19xx codes.
- **World files and templates:**
  - `raster.rs` `GridSpec::from_world_file` **rejects rotated world files** ("rotated rasters are not
    supported"). `project.rs` `check_template` uses it, so a fitted, rotated plan must not go through
    `check_template`.
  - OOM itself supports rotated world files: `src/templates/world_file.cpp` builds a full `QTransform` from
    all six parameters and shifts by (A+B)/2, (D+E)/2 to the pixel corner. Checked via the GitHub API.
  - **OCAD export** (`ocd/convert.rs` `background_parameter`): sets angles `a`/`b` to the grivation only
    and takes u/v from `hypot`. A rotated plan would come out unrotated in OCAD, so the plan must add the
    image rotation to `a`/`b`.
  - `omap/writer.rs` writes `<template type="TemplateImage" … georef="true">` with
    `<crs_spec>+init=epsg:…</crs_spec>`, rasters behind the map and the candidates `TemplateMap` in front.
- **Pipeline:**
  - `project.rs` `fetch_project` always fetches rasters (per provider), ALKIS buildings, ALKIS land use
    and OSM.
  - `starter_documents` builds parts "default part" and "peripheral information", plus the candidates
    template.
  - Peripheral info: `NORTH_LINE_SPACING_MM` × `symbol_enlargement()`, and
    `PeripheralInfo.contour_interval_m` (a gym map has none).
- **ALKIS:** `Building { oid, kind (gebnutzbez), function (e.g. "Sport-, Turnhalle"), footprint }`.
  `build/buildings.rs` makes 521 objects.
- **CLI:** `freemapper-cli/src/main.rs` (clap) has the subcommands `init`/`export`/`fetch`/`derive`/`run`/
  `select`/`mcp` and the enum `Kind { Schoolyard, Sprint }`.
- **MCP:** `mcp.rs` has the tools `find_place`, `create_project`, `estimate_download`, `start_pipeline`,
  `project_status`, `list_layers`, `open_area_selector` and `preview_layer`. `map_kind()` parses
  "schoolyard"/"sprint", and `INSTRUCTIONS` names both kinds.
- **Area selector:**
  - `web/index.html` `<select id="kind">` offers two options;
  - `web/app.js` sets the scale with `sprint ? 4000 : 1000` and the buffer per kind;
  - `select.rs` maps the kind strings.
- **ADRs** go up to 0042, so the next free number is 0043. The glossary is `docs/glossary.md`; the manual is
  `docs/manual/{de,en}.md`; the changelog is `CHANGELOG.md` / `CHANGELOG.en.md`.

## 6. Demo hall candidates (OSM, Overpass 2026-11-08, `dl/osm/`)

- **way/1466541664 "Mühlenfließhalle"**, Rotdornstraße 1, Strausberg (52.5410 N, 13.8193 E):
  - tags `leisure=sports_hall`, `building=yes`;
  - outline about 45.5 × 30.7 m with an annex (1782 m²);
  - a Dreifeld-sized hall, so it can show the triple preset.
- **way/414570767**, unnamed, `building=sports_centre` + `leisure=sports_hall`, 22.6 × 24.9 m outline,
  at 52.5404 N, 13.8446 E:
  - right next to the Clara-Zetkin-Schule (`tests/fixtures/clara-zetkin-schule.geojson`), in the project's
    test area;
  - most likely the school's sports hall (not confirmed on the ground).
- The ALKIS outlines (LGB OGC API) have not been fetched yet. The plan's last task does that live.

## 7. Risks

- **R1 – ISInOM is a draft.**
  - Consultation ended 31 May 2026. The final version may renumber or restyle symbols, and its sub-code
    numbering is already inconsistent (§1).
  - Mitigation: one code table (`gym-symbols` asset plus a test that lists every code); an ADR records the
    alignment to 6.6.
- **R2 – 19xx collision.** The IOF could use 19xx later; unlikely, because there is no ISOM category 9.
- **R3 – ALKIS outline.**
  - It is the outer face of the building and may include annexes, Bauteile or neighbouring school buildings.
    Its accuracy at 1:200 (0.1 m = 0.5 mm) is visible.
  - Mitigation: choose the building with the largest overlap with the map area; draw the wall with its
    outer edge on the outline; fall back to the map-area polygon with a warning.
- **R4 – Hall plans.**
  - Architects' plans are copyrighted and may be distorted (scans: shear, non-uniform scale). A similarity
    transform cannot fix that; fit-plan reports the residuals.
  - Plans stay local, and examples never contain a third-party plan without permission.
- **R5 – Rules change.** FIBA 2026 (valid from 1 Oct 2026), IHF 2025 (throw-off circle), BWF Laws from
  4 Jan 2027 (unchecked), FIFA futsal yearly. Mitigation: a rules edition constant per sport, written into
  each court's tags.
- **R6 – OCAD export of rotated templates** (convert.rs uses the grivation only, §5). Must be fixed in the
  plan, or the plan must be left out of OCAD with a note.
- **R7 – Unverified equipment sizes** (§4). They are presets only; the mapper draws true outlines with the
  area symbols.
- **R8 – Scale range vs ISInOM base 1:700.** An ISInOM event in a large sports complex may want 1:500–1:700.
  We allow 1:100–1:500 (OOM #613 quote). 1:700 would need a decision (Q in the plan).
- **R9 – Image size.** Large scans (> 16 384 px a side) strain OOM/QImage and memory. Reject them with a
  message.

## 8. Open questions (consolidated)

The questions for courseplan and print are in GYM-SYMBOLS §8: Q1 course symbol size; Q2 colours; Q3/Q3b
control descriptions and 709 numbering; Q4 blue mats; Q5 court-line colours; Q6 map parts as the layer
contract; Q7 danger-area colour; Q8 equipment sizes. Further questions for the user:

- **U1:** Allow 1:700 (ISInOM base) for halls larger than a Dreifachhalle, or keep 1:500 as the hard limit?
- **U2:** Should the demo example contain a hall plan? A plan needs the school authority's permission; the
  alternative is a self-drawn schematic plan under GPL/CC0.
- **U3:** Draw the 9 m line as a dashed symbol (1956, G dash 0.15/0.15 m) or as individual dash objects?
  The plan chooses the symbol.
