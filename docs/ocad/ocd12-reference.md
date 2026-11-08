# OCAD 12 (.ocd) writer: implementation reference for freemapper

Scope: write an OCAD 12 file (`Version = 12`) with colours, placeholder symbols, map objects
(points, lines, areas with holes, unformatted texts), georeferencing (EPSG:25833) and raster
background maps. Everything here is meant to be implemented directly. No Rust code is included.

## 0. Sources, tags, conventions

| Tag | Source |
|---|---|
| **[S]** | OCAD 12 file format page (https://www.ocad.com/wiki/ocad/en/index.php?title=OCAD_12_File_Format, revision 15233) |
| **[O]** | OpenOrienteering Mapper master: `ocd_file_export.cpp`, `ocd_types*.h/.cpp`, `ocd_file_import.cpp`, `ocd_georef_fields.cpp`, `ocd_icon.cpp`, `template*.cpp`, `georeferencing.cpp`, tests |
| **[P]** | Purple Pen (`src/MapModel/MapModel/OcadExport.cs`, `OcadFileFormat.cs`). This is a second writer that produces OCAD 12 files OCAD can open. |
| **[F]** | Bytes observed in three files written by **OCAD 12.0.1 itself** (Purple Pen test data: `Lake Sammamish OCAD12.ocd`, `layout_objects12.ocd`, `nonrenderable12.ocd`; decoded with a script) |
| **[W]** | OCAD wiki pages "Create a New Map", "Background Map", "Map" (current OCAD, not version 12 specifically) |
| ⚠ | contradiction between sources, or an open risk |

* **Byte order:** little-endian everywhere. **Records are packed**, with no implicit padding inside a record.
  The spec says "packed record". OOM uses `#pragma pack(1)` and static-asserts the sizes. Every size
  below was verified against [F].
* **Delphi types:** Integer=i32, Cardinal=u32, SmallInt=i16, Word=u16, Byte=u8, Boolean=u8, WordBool=u16,
  Double=f64 (IEEE), `char` = UTF-16 code unit (Unicode Delphi).
* **Paper units:** 0.01 mm ("OCAD unit"). OOM and freemapper use 0.001 mm, so OCAD = µm / 10, rounded half up [O `convertPointMember`].
* **Axis:** OCAD paper y points **up**, while OOM and freemapper y points down, so **negate y** [O `convertPoint(x,y)` uses `-y`; import uses `* -10`].
* **Angles:** 0.1° units, counter-clockwise on paper. OOM rotation in radians maps 1:1: `ang = round(10·deg(rot))`, with
  no sign change [O `convertRotation`, import `convertAngle`]. OCAD writes values 0…3600 (it was seen writing 3600) [F]. OOM's
  importer normalises with `(a+3600)%3600`.
* **File position:** an absolute byte offset from the start of the file [S].

---

## 1. File layout

### 1.1 File header: 60 bytes [S][O `FileHeaderV12`, static_assert 60][F]

| Off | Type | Field | Write for a new file | Notes |
|---:|---|---|---|---|
| 0 | u16 | OCADMark | `0x0CAD` (bytes `AD 0C`) | [S][O][F] |
| 2 | u8 | FileType | `0` (normal map) | 1 = course-setting project, 8 = server [S]. OOM: `TypeMap`=0 |
| 3 | u8 | FileStatus | 0 | "not used" [S] |
| 4 | u16 | Version | `12` | |
| 6 | u8 | Subversion | 0 | |
| 7 | u8 | SubSubversion | 0 or 1 | OOM 0; Purple Pen 1 (`Subversion` i16 = 256); OCAD 12.0.1 writes 1 [F] |
| 8 | u32 | FirstSymbolIndexBlk | position of the first symbol index block | |
| 12 | u32 | ObjectIndexBlock | position of the first object index block | |
| 16 | i32 | OfflineSyncSerial | 0 | |
| 20 | i32 | CurrentFileVersion | **0** | OOM 0, PP 0. OCAD 12.0.1 writes **2** [F]. 0 is proven to open. ⚠ meaning undocumented |
| 24 | u32 | internal | 0 | |
| 28 | u32 | internal | 0 | |
| 32 | u32 | FirstStringIndexBlk | position of the first string index block | |
| 36 | u32 | FileNamePos | 0 | "temporary files only" [S]. OCAD writes 60, and a 128-byte 8-bit path lives at 60..187 [F] |
| 40 | u32 | FileNameSize | 0 | OCAD writes 128 [F] |
| 44 | u32 | internal | 0 | OCAD writes 888 [F]; OOM/PP write 0 |
| 48 | u32 | Res1 | 0 | |
| 52 | u32 | Res2 | 0 | |
| 56 | u32 | MrStartBlockPosition | 0 | multi-representation, unused |

### 1.2 Index blocks (all three chains) [S][O][P][F]

* Each block is `next: u32 (string, symbol) / i32 (object)` followed by `entries[256]`. `next = 0` ends the chain.
  When the 256 entries of a block are used up, append a new zeroed block anywhere and write its position into the
  previous block's `next` [O `OcdEntityIndex::insert`].
* An unused entry is all zeros. OOM treats `pos == 0` as empty [O `isValidEntry`]. Purple Pen treats an object entry with
  `Sym == 0` as empty [P]. Zero the whole entry.
* There are **no count fields** anywhere. Counts come only from the index.
* ⚠ In OCAD-written files the **last object index block's `next` is non-zero garbage beyond EOF**
  (e.g. 121223152 in a 442 KB file) [F]. Purple Pen guards for this (`while nextBlock > 0 && nextBlock < fileLength`) [P].
  **Writers write 0. Readers (your tests) must bounds-check.**

| Block | Size | Entry layout |
|---|---|---|
| Symbol index | 1028 = 4 + 256×4 | `u32 pos` (spec: `integer`). Position of the symbol record |
| Object index | 10244 = 4 + 256×40 | see 1.3 |
| String index | 4100 = 4 + 256×16 | `u32 Pos`, `i32 Len` (reserved bytes), `i32 RecType` (string type; <0 = deleted), `i32 ObjIndex` (0 for global strings) [S][O `ParameterStringIndexEntry`] |

### 1.3 Object index entry: 40 bytes [S][O `ObjectIndexEntryV9`][F]

| Off | Type | Field | Write |
|---:|---|---|---|
| 0 | i32 | rc.left (min x) | **TDPoly-encoded** (`value_0.01mm << 8`, flag bits 0) [O `convertPoint`][F] |
| 4 | i32 | rc.bottom (min y, y up) | 〃 |
| 8 | i32 | rc.right (max x) | 〃 |
| 12 | i32 | rc.top (max y) | 〃 |
| 16 | i32 | Pos | file position of the object record |
| 20 | i32 | **Len** | **object size in bytes** = 56 + 8·(nItem+nText+nObjectString+nDatabaseString) |
| 24 | i32 | Sym | symbol number (1000-based, see 3.2). Same as the object's `Sym` |
| 28 | u8 | ObjType | same as the object's `Otp` (1 point, 2 line, 3 area, 4 unformatted text, 5 formatted text, 6 line text, 7 rectangle) [S] |
| 29 | u8 | EncryptedMode | 0 |
| 30 | u8 | Status | **1** (normal). 0 = deleted, 2 = hidden, 3 = deleted for undo [S]. OOM's importer skips 0 and 3 [O] |
| 31 | u8 | ViewType | 0 |
| 32 | i16 | Color | colour number of the symbol's dominant colour. OOM: `guessDominantColor()`; PP: the symbol's single colour; −1 for image/layout objects [S][P] |
| 34 | i16 | Group | 0 |
| 36 | i16 | ImpLayer | 0 |
| 38 | u8 | DbDatasetHash | 0 |
| 39 | u8 | DbKeyHash | 0 |

⚠ **`Len` contradiction:** the spec says "number of coordinate pairs, size = 32 + 8·Len" [S]. That is wrong for V12.
OCAD's own files have `Len == exact object byte size` (1107/1107, 40/40 and 13/13 objects), and consecutive objects are exactly
`Len` apart [F]. Purple Pen writes `56 + 8·nItems` for V12 [P]. OOM writes the (8-padded) byte size and explains that the documented
meaning makes OCAD 9/10 report "damaged objects" [O `exportObjectCommon`]. **Write bytes.**

**Bounding box rule (exact, from [F]):** `rc` = min/max over **all** coordinates of the object (including Bézier
control points), **expanded on all four sides by the symbol's `Extent`**. This held for 1045/1045 point, line and area objects
[F]. OCAD's Optimize/Repair treats objects whose box does not contain all their points as damaged [P comment "otherwise
OCAD thinks the object is corrupt"; O issue #1134 "include all control points … in the object extent"]. OOM uses the rendered
extent plus all points [O]. For texts use the four box corners ± the text symbol's Extent (see 5.6).

### 1.4 Entity placement, order, alignment

* **What OOM writes** [O `OcdFile()` + `exportImplementation`]: the header at 0, the first **string** index block at **60**, the first
  **symbol** index block at **4164**, the first **object** index block at **5196**, and only then the entities in this order:
  1039 georef → 1030 view → 1061 notes → 9/10 colours → all symbols → all objects → 8 templates.
  (OOM records each block position *before* calling `addPadding`. The block then really starts at the recorded position, and 4 slack zero
  bytes follow it. This is self-consistent because reader and writer both use the header value.)
* **What OCAD 12 itself writes** [F]: header | file name (60..187) | string index block @188 | strings | symbol index block |
  symbols | object index block | objects | further object index blocks interleaved with their objects.
* **Purple Pen** [P] writes symbols, then the objects of each 256-group followed by that group's index block, then the strings followed by
  their index block. **Order is free.** Only the positions matter.
* **Alignment:** none is required. OCAD's own symbols, objects and strings sit at 4-byte (not 8-byte) aligned positions [F]. OOM pads every
  entity start to 8 [O `addPadding`]. All fixed record sizes are multiples of 4, so 4-alignment comes for free. Recommended:
  pad entity starts to 8 with zero bytes (harmless).
* **Position computation:** build the file in a `Vec<u8>`. Write a zero header, then append index blocks and entities, record each
  `pos = buf.len()` before appending, and patch index entries and header fields in place at the end.

### 1.5 What is required

Proven sufficient, because OCAD opens OOM's output [O]: header, the three index chains (OOM always writes ≥1 block each, possibly empty),
colour strings (9), symbols, objects, and the 1039 scale/georef string. Everything else (1024, 1026, 1030, 1061, 15, 8, 10, file name)
is optional. OCAD 12 itself additionally writes 1024 display, 1026 print and 1030 view [F].

---

## 2. Strings

### 2.1 Fixed strings inside symbol records

| Field | Real layout | Evidence |
|---|---|---|
| `Description: array[0..63] of char` | **64 UTF-16LE code units = 128 bytes, NUL-terminated, zero-filled, max 63 chars.** No length prefix | [S] (Delphi `char`=WideChar), [O `Utf16PascalString<64>`, `assignUtf16(…, N-1)`], [F] (descriptions decode at offset 56) |
| `FontName: string[31]` (also `UnnumText: string[3]`, `Res3: string[18]`) | **1 length byte + 31 bytes**, rest zero (32 bytes total) | [S] "first byte contains the number of characters … occupies x+1 bytes", [O `Utf8PascalString<31>` (UTF-8 for V11+)], [F] `05 'Arial'` |

⚠ The spec calls `String[x]` a "Widestring", yet says it occupies x+1 bytes. That is self-contradictory. A Delphi `string[N]` is an 8-bit
ShortString. OOM writes UTF-8 bytes, OCAD presumably ANSI. **Use ASCII font names** ("Arial") and the question is moot.

### 2.2 Parameter strings [S][O][F]

* Syntax: `first_field` `TAB code value` `TAB code value` …, then **NUL**. `code` is one character. If there is no first field the string
  starts with TAB (`"\tm4000\t…"`). Values must not contain TAB or NUL.
* Index `Len` = reserved bytes ≥ strlen + 1. OOM writes exactly strlen+1 [O `insert(type, data)`]. OCAD rounds up to a multiple of 16 [F]. PP
  writes `(chars+2)&~1` [P]. All of these work.
* Numbers use `.` as the decimal separator (Rust `format!` is locale-free). OCAD writes e.g. `m10000.00`, `x0.00`, `a0.000000` [F]. OOM writes
  integers where its field is an int (`m4000`, `x413000`) [O]. Both forms are read.
* ⚠ **Encoding:** OOM writes **UTF-8** for V11/V12 [O `Format::Encoding = Utf8Encoding`]. Purple Pen writes the system ANSI code page
  (cp1252) for all versions [P `GetDefaultOcadEncoding`]. The OCAD samples contain no non-ASCII bytes [F]. **Keep parameter strings ASCII**
  (colour names, notes, template paths; transliterate ä→ae, ß→ss), or test UTF-8 in OCAD 12 before relying on it. Object texts are
  UTF-16 (5.6) and are not affected.
* Types < 1000 may occur several times (lists) [S]. Their order in the index defines list order (colours, see 4).

---

## 3. Symbols

### 3.1 Base symbol header: 796 bytes, common to all types [S][O `BaseSymbolV11`][F]

| Off | Type | Field | Write |
|---:|---|---|---|
| 0 | i32 | Size | total record size in bytes, **including** trailing element data |
| 4 | i32 | SymNum | see 3.2 |
| 8 | u8 | Otp | 1 point, 2 line, 3 area, 4 text, 6 line text, 7 rectangle [S] |
| 9 | u8 | Flags | 1 = rotatable (point/area pattern/text), 4 = favourite; else 0 [S][O] |
| 10 | u8 | Selected | 0 |
| 11 | u8 | Status | 0 normal, 1 protected, 2 hidden (`AND 16` = selected) [S] |
| 12 | u8 | PreferredDrawingTool | 0 |
| 13 | u8 | CsMode | 0 |
| 14 | u8 | CsObjType | 0 |
| 15 | u8 | CsCdFlags | 0 |
| 16 | i32 | Extent | max reach of the rendering beyond the coordinates, in 0.01 mm. Point: max radius; line: ⌊width/2⌋ (+ border); area: 0; text: 0 (OOM) [O][F] |
| 20 | u32 | FilePos | "not defined in file" [S]. OOM 0; OCAD writes the record's own position (236/236) [F]. Writing own position or 0 both work |
| 24 | u8×2 | notUsed | 0 |
| 26 | i16 | nColors | number of colours used (≤14), −1 if >14 |
| 28 | i16×14 | Colors | colour numbers (`n` of string 9) used by the symbol, rest 0 [O `setupSymbolColors`] |
| 56 | u16×64 | Description | symbol name, UTF-16LE, NUL-terminated (2.1) |
| 184 | u8×484 | IconBits | 22×22 icon, see 3.7 |
| 668 | u16×64 | SymbolTreeGroup | 0 (no symbol-tree group) [O writes 0] |
| 796 | | type-specific part | |

### 3.2 Symbol number encoding

* **`SymNum: i32 = number × 1000`. OCAD reads the part after the dot as a three-place decimal:**
  "501.1" = "501.10" = "501.100" → **501100**. 501001 is a *different* number that OCAD shows as "501.001".
  * [S]: "1000 times the symbol number … 203.145 is stored as 203145".
  * OCAD wiki: "symbol number between 0.001 and 999999.999". OCAD always prints three decimals (`526.000`, `813.001`) [W].
  * **Verified on OCAD's official ISSprOM 2019-2 set** (`ISSprOM_blank.ocd` from the Purple Pen repo, OCAD-2018 format, decoded with the same
    script). IOF numbers are stored as x100/x200/x300: 501.100 "Step or edge of paved area", 501.200, 501.300, 513.100 "Passable wall",
    513.200 "Passable retained wall". OCAD's own variants use x001…x0nn: 501.001 "Paved area … (Area)", 501.010 "Paved footpath, 1.6 m",
    501.020 "Paved area, heavy traffic (Area & Line)", 505.001 "Unpaved footpath or track, 2 m", 521.003 "Building, minimum".
* ⚠ **OOM and Purple Pen contradict OCAD here.** OOM's `makeSymbolNumber` [O] computes
  `n = c1; if c2 >= 0 { n = c1*100 + c2 % 100 }; SymNum = c0*1000 + n % 1000; if SymNum == 0 { 1 }`. So **"501.10" → 501010** and
  **"501.1" → 501001**, and its importer reads 501100 back as "501.100" [O `setupBaseSymbol`]. Purple Pen writes `"a.b"` → `a*1000 + int(b)` [P].
  In OCAD's set, 501001 is an *area* and 501010 a *line* ("Paved footpath"), so OOM-style numbers would pull the wrong official symbols.
  (The x001-style numbers in the Lake Sammamish sample come from that integer convention via Purple Pen. They are not evidence of OCAD's
  semantics.)
* ⇒ **Do not derive OCAD numbers from OOM code strings.** Give every freemapper symbol an **explicit OCAD number** (table in §9).
  OCAD's own decimal rule makes "501.1" and "501.10" the same number, so any generic fallback must check uniqueness.
* Numbers must be unique in the file. OOM bumps duplicates to the next free number [O `makeUniqueSymbolNumber`]. Number 0 is invalid [O].
* An object's type must fit its symbol's type. OCAD's Optimize/Repair reports "objects with invalid symbol type" [W "Map"]. This matters when
  official symbols replace placeholders by number.

### 3.3 Point symbol: 800 bytes + elements [S][O `PointSymbolV11`][F]

| Off | Type | Field | Write |
|---:|---|---|---|
| 796 | u16 | DataSize | number of 8-byte units following = Σ over elements (2 + stnPoly) |
| 798 | i16 | Reserved | 0 |
| 800 | | elements | `Size = 800 + 8·DataSize` |

**Element `TSymElt`: 16 bytes + 8·stnPoly** [S][O `PointSymbolElementV8`][F]

| Off | Type | Field | Notes |
|---:|---|---|---|
| 0 | i16 | stType | 1 line, 2 area, 3 circle, 4 dot |
| 2 | u16 | stFlags | line: 0 flat/bevel, 1 round, 4 flat/miter [O] |
| 4 | i16 | stColor | colour number |
| 6 | i16 | stLineWidth | 0.01 mm (lines, circles) |
| 8 | i16 | stDiameter | dot: full diameter. **Circle: centre-line diameter = outer Ø − line width** ("line width included once") [S][O: `2·inner_r + width`][F: circle lw 18, Ø 62, extent 40] |
| 10 | i16 | stnPoly | number of coordinates |
| 12 | i16×2 | stRes1/2 | 0 |
| 16 | TDPoly×n | coordinates | symbol-local paper coordinates, same encoding as objects (`<<8`, y up) [O `exportCoordinates`] |

Minimal dot (as OCAD writes it [F]): `Size=824, DataSize=3`, element `{type 4, flags 0, color c, lw 0, Ø d, n 1}` + point (0,0).
`Extent = d/2`. ⚠ A circle with Ø ≤ line width is invisible in OOM's re-import (OOM issue #2471). Use a dot for a filled disc.

### 3.4 Line symbol: 872 bytes + elements [S][O `LineSymbolCommonV8`][F]

| Off | Type | Field | Minimal value |
|---:|---|---|---|
| 796 | i16 | LineColor | c |
| 798 | i16 | LineWidth | 0.01 mm |
| 800 | i16 | LineStyle | 0 bevel/flat, 1 round/round, 4 miter/flat; +2 = pointed ends (2, 3, 6) [S][O] |
| 802 | i16 | DistFromStart | 0 |
| 804 | i16 | DistToEnd | 0 |
| 806 | i16 | MainLength (a) | 0, or dash length |
| 808 | i16 | EndLength (b) | 0, or dash length (half for "half outer dashes") |
| 810 | i16 | MainGap (C) | 0 = solid. >0 = dashed (gap) |
| 812 | i16 | SecGap (D) | 0 |
| 814 | i16 | EndGap (E) | 0 |
| 816 | i16 | MinSym | 0 (OOM: 0 if "show at least one symbol", else −1) |
| 818 | i16 | nPrimSym | 0 |
| 820 | i16 | PrimSymDist | 0 |
| 822 | u16 | DblMode | 0 (off) |
| 824 | u16 | DblFlags | 0 |
| 826 | i16 | DblFillColor | 0 |
| 828 | i16 | DblLeftColor | 0 |
| 830 | i16 | DblRightColor | 0 |
| 832 | i16 | DblWidth | 0 |
| 834 | i16 | DblLeftWidth | 0 |
| 836 | i16 | DblRightWidth | 0 |
| 838 | i16 | DblLength | 0 |
| 840 | i16 | DblGap | 0 |
| 842 | i16 | DblBackgroundColor | 0 |
| 844 | i16×2 | DblRes | 0 |
| 848 | u16 | DecMode | 0 |
| 850 | i16 | DecSymbolSize | 0 |
| 852 | u8 | DecSymbolDistance | 0 (OCAD writes 1 [F]) |
| 853 | u8 | DecSymbolWidth | 0 |
| 854 | i16 | FrColor | 0 |
| 856 | i16 | FrWidth | 0 |
| 858 | i16 | FrStyle | 0 |
| 860 | u16 | PrimDSize | 0. Each `*DSize` counts 8-byte units incl. 2 per element header |
| 862 | u16 | SecDSize | 0 |
| 864 | u16 | CornerDSize | 0 |
| 866 | u16 | StartDSize | 0 |
| 868 | u16 | EndDSize | 0 |
| 870 | u8 | UseSymbolFlags | 0 (1 end, 2 start, 4 corner, 8 secondary) |
| 871 | u8 | Reserved | 0 |
| 872 | | elements: Prim, Sec, Corner, Start, End in this order | `Size = 872 + 8·ΣDSize` |

* Solid line as OCAD writes it [F]: colour, width, style 1, all lengths 0. `Extent = width/2` (101000: width 13 → extent 6) [F][O].
* Dash rule (reader view) [O import]: dashed ⇔ `MainGap ≠ 0 || SecGap ≠ 0`, dash = MainLength, gap = MainGap. OOM writes a
  simple dash as `MainLength = EndLength = dash, MainGap = gap` [O `exportLineSymbolCommon`].
* OOM writes the mid symbol as Prim and OOM's "dash symbol" as Corner. Not needed for placeholders.

### 3.5 Area symbol: 832 bytes + elements [S][O `AreaSymbolV12`][F]

| Off | Type | Field | Minimal (solid fill) |
|---:|---|---|---|
| 796 | i32 | BorderSym | 0 (else SymNum of a line symbol and BorderOn=1) |
| 800 | i16 | FillColor | c |
| 802 | i16 | HatchMode | 0 none, 1 single, 2 cross |
| 804 | i16 | HatchColor | 0 |
| 806 | i16 | HatchLineWidth | 0 |
| 808 | i16 | HatchDist | 0. **V9+: centre-to-centre spacing** (V8: gap) [O] |
| 810 | i16 | HatchAngle1 | 0. 0.1° [O `convertRotation`] |
| 812 | i16 | HatchAngle2 | 0 |
| 814 | u8 | FillOn | 1 |
| 815 | u8 | BorderOn | 0 |
| 816 | u8 | StructMode | 0 |
| 817 | u8 | StructDraw | 0 |
| 818 | i16 | StructWidth | 0 |
| 820 | i16 | StructHeight | 0 |
| 822 | i16 | StructAngle | 0 |
| 824 | u8 | StructIrregularVarX | 0 |
| 825 | u8 | StructIrregularVarY | 0 |
| 826 | i16 | StructIrregularMinDist | 0 |
| 828 | i16 | StructRes | 0 |
| 830 | u16 | DataSize | 0 (structure elements, like point symbols) |

* Solid area as OCAD writes it [F]: `Size 832, Extent 0, FillColor c, FillOn 1`, everything else 0.
* Hatch: `HatchMode 1, HatchColor, HatchLineWidth, HatchDist, HatchAngle1`. FillOn may stay 1 with a background colour. Set `Flags |= 1`
  if the pattern should rotate with the object angle [O].

### 3.6 Text symbol: 1036 bytes, no elements [S][O `TextSymbolV11`][F]

| Off | Type | Field | Write |
|---:|---|---|---|
| 796 | u8 + 31 | FontName | `5,'Arial'`, zero-filled |
| 828 | i16 | FontColor | c |
| 830 | i16 | FontSize | **10 × size in pt**. From an em size in mm: `round(10 · mm / 25.4 · 72)`. freemapper 5.0 / 2.5 / 1.8 mm → **142 / 71 / 51** [O] |
| 832 | i16 | Weight | 400 normal, 700 bold |
| 834 | u8 | Italic | 0/1 |
| 835 | u8 | Res1 | 0 |
| 836 | i16 | CharSpace | 0 (percent) |
| 838 | i16 | WordSpace | **100** (percent) [O][F] |
| 840 | i16 | Alignment | bits 0–1: 0 left, 1 centre, 2 right, 3 justified; bits 2–3: 0 bottom(=baseline), 4 middle, 8 top [S][O enum] |
| 842 | i16 | LineSpace | percent of font size. OCAD default 120 [F]; OOM derives it from font metrics [O] |
| 844 | i16 | ParaSpace | 0 (0.01 mm) |
| 846 | i16 | IndentFirst | 0 |
| 848 | i16 | IndentOther | 0 |
| 850 | i16 | nTabs | 0 |
| 852 | i32×32 | Tabs | 0 |
| 980 | u16 | LBOn | 0 |
| 982 | i16 | LBColor | 0 |
| 984 | i16 | LBWidth | 0 |
| 986 | i16 | LBDist | 0 |
| 988 | i16 | Res2 | 0 |
| 990 | u8 | FrMode | 0 (1 shadow, 2 line, 3 rectangle) |
| 991 | u8 | FrLineStyle | 0 (OCAD writes 4) [F] |
| 992 | u8 | PointSymOn | 0 |
| 993 | i32 | PointSymNumber | 0 (**unaligned** i32) |
| 997 | 19 B | Res3 (string[18]) | 0 |
| 1016 | i16×4 | FrLeft/Bottom/Right/Top | 0 |
| 1024 | i16 | FrColor | 0 |
| 1026 | i16 | FrWidth | 0 |
| 1028 | i16 | Res4 | 0 |
| 1030 | u16 | Res5 | 0 |
| 1032 | i16 | FrOfX | 0 |
| 1034 | i16 | FrOfY | 0 |

**Alignment is a symbol property.** OOM emits one OCAD text symbol per used horizontal alignment: the most frequent one keeps the
number, the others get the next free numbers (e.g. 993000 left, 993001 centre). OOM always uses vertical "bottom" and puts the anchor
at the first line's baseline [O `prepareTextSymbolExport`, `exportTextCoordinatesSingle`]. freemapper's sheet texts are baseline-aligned
with h_align 0 and 1, so it needs **two symbols for 993** (and for any other code used with both alignments).

### 3.7 Icon: 484 bytes [O `OcdIcon::operator IconV9`][F]

* 22×22 pixels, one byte per pixel, **bottom row first** (rows y = 21 … 0, x = 0 … 21) [O].
* Palette = 5×5×5 colour cube: `index = 25·R + 5·G + B` with `R,G,B = (channel + 32) / 64` ∈ 0…4 [O `getPaletteColorV9`].
  The spec says "256 colour palette" ⚠. The cube matches OCAD's files: white = **124**, black 0, blue 4, green(0,128,0) 10, yellow 120,
  grey(192) 93 [F].
* Not required for validity. An all-zero icon shows as a black square. Recommended: fill with 124 and draw a crude glyph in the symbol
  colour (dot / horizontal bar / filled square / "A"). OCAD's "Load Colors and Symbols From" brings real icons anyway.

### 3.8 Other symbol fields

* Symbol tree: `SymbolTreeGroup[]` refers to group ids of string type 15 (`name\tg<id>\t…`). Optional. Write zeros and no type-15
  string [O]. OCAD wrote `All symbols\tg-1\te1\tf1\tl1\ti0` in one sample [F].
* Never write deleted symbols. Unused index entries are 0.

---

## 4. Colours: string type 9 [S][O `stringForColor`][F]

`Name\tn<num>\tc<C>\tm<M>\ty<Y>\tk<K>\to<0|1>\tt<opacity>` and optionally repeated `\ts<separation>\tp<percent>`.

| Code | Meaning | Value |
|---|---|---|
| first | name | ASCII text |
| n | colour number. **This is what symbols and objects reference** | small int, unique |
| c m y k | CMYK percent | OOM: integers 0–100; OCAD writes `c25.0` or `c25` [F] |
| o | overprint | **write `o0`** (see ⚠) |
| t | "transparency" = **opacity percent** (100 = opaque) [O import: `opacity = t/100`][F `t100`] | 100 |
| s, p | spot-colour separation name and percentage | omit (no spot colours) |

* **Order = priority:** the order of type-9 strings in the string index is the colour table order. The **first is on top**
  [O import appends in string order; export writes OOM index 0 (top) first][F: purple overprint colours first, numbers
  non-sequential: n11, n17, n18, n14, n0 …]. `n` is only an identifier. Simplest: `n = position` (OOM does this).
* ⚠ Overprint: OOM writes `o1` for every colour without knockout, i.e. for all colours of a map without spot colours.
  CONDES then renders wrongly (OOM issue #2228). OCAD's own sets use `o0` except special cases [F]. **Write `o0`.**
* Spot colours (type 10: `name\tn\tv1\tc\tm\ty\tk\tf<lpi×10>\ta<angle>`) [O] are not needed.
* OOM writes "Registration black" as colour 0 only if a symbol uses it [O]. Not relevant here.

---

## 5. Objects

### 5.1 Object record: 56-byte header + data [S][O `ObjectV12`][F]

| Off | Type | Field | Write |
|---:|---|---|---|
| 0 | i32 | Sym | SymNum (>0). −1 = no symbol, −2 graphic, −3 image, −4 layout |
| 4 | u8 | Otp | 1 point, 2 line, 3 area, 4 unformatted text, 5 formatted text |
| 5 | u8 | _Customer | 0 |
| 6 | i16 | Ang | 0.1°, CCW. Point, unformatted text, area with structure/hatch rotation |
| 8 | i32 | Col | 0 for symbolised objects [O][F] (colour number only for graphic objects) |
| 12 | i16 | LineWidth | 0 |
| 14 | i16 | DiamFlags | 0 |
| 16 | i32 | ServerObjectId | 0 |
| 20 | i32 | Height | 0 (1/256 mm) |
| 24 | f64 | CreationDate | 0.0 (OOM). Optionally a Delphi TDateTime (days since 1899-12-30) |
| 32 | u32 | MultirepresentationId | 0 |
| 36 | f64 | ModificationDate | 0.0 (**unaligned** f64, packed) |
| 44 | u32 | nItem | number of coordinates |
| 48 | u16 | nText | text length in **8-byte units** |
| 50 | u16 | nObjectString | 0 |
| 52 | u16 | nDatabaseString | 0 |
| 54 | u8 | ObjectStringType | 0 |
| 55 | u8 | Res1 | 0 |
| 56 | TDPoly×nItem | coordinates | then text (nText×8 bytes) |

### 5.2 TDPoly coordinates [S][O][F]

`x_raw: i32 = (X << 8) | xflags`, `y_raw: i32 = (Y << 8) | yflags`, where X and Y are signed 0.01 mm with y up. In Rust,
`(v << 8) | flags` on i32 gives the right two's-complement result. Read with an arithmetic `>> 8`.

| Bit | x flags | y flags |
|---|---|---|
| 1 | first Bézier control point | corner point |
| 2 | second Bézier control point | **first point of a hole** (areas) |
| 4 | no left line to next point (double lines) | no right line (double lines) |
| 8 | area border / virtual line gap | dash point |

* **Bézier curves:** the anchor points carry no flag. The two control points between anchors carry x-bit 1 and x-bit 2. OOM marks P(i)
  "curve start" in its own format and gives P(i+1) x|=1 and P(i+2) x|=2 [O `exportCoordinates`][F: equal counts of x1/x2].
* OOM dash points become y|=8 for dashed lines without a dash symbol, else y|=1 (corner) [O].
* Range: 24-bit signed (±83.9 m paper). **OCAD Orienteering/Starter editions limit drawing to 4 m × 4 m centred on (0,0), i.e. ±2000 mm; Mapping
  Solution allows 80 m × 80 m** [W "Create a New Map"]. OOM warns outside ±2000 mm and shifts if needed [O `calculateAreaOffset`]. Keep
  every coordinate within ±2000 mm. With the sheet centred at (0,0) this is trivially true (A4 at 1:1000 or 1:4000).

### 5.3 Point objects [O `exportPointObject`][F]
`Otp 1, nItem 1` (the anchor), `Ang = round(10·deg(rotation))`. Write the angle only for rotatable symbols (`Flags & 1`). OOM's importer
makes a symbol rotatable if it finds non-zero angles. bbox = point ± Extent [F].

### 5.4 Line objects [O `exportPathObject`]
`Otp 2`. **One OCAD line object = one polyline.** OOM splits multi-part paths into separate objects. A **closed line** repeats the first point at the
end; OCAD has no closed flag for lines (OOM writes its raw coordinates, which include the closing duplicate). Lines with 1 point are "spurious"
and are deleted by Optimize [W "Map"].

### 5.5 Area objects with holes [S][O][F]
`Otp 3`. Coordinates = outer ring, then each hole ring. **The first point of every hole ring has y-bit 2 (`|= 2`).** Rings may be
closed (last == first) or open. OCAD itself writes both (210 open / 132 closed rings in one file) [F]. OOM writes closed rings
[O]. OOM's importer closes open rings [O `fillPathCoords`]. Recommended: closed rings, as OOM does. freemapper's OOM model already marks
ring ends (flag 16 "hole" on the last point of a part). Translate that to **y|=2 on the next point**, the first of the next ring. Do not set
it on the outer ring's first point. Areas with <3 points are spurious [W].

### 5.6 Text objects (unformatted, `Otp 4`) [O `exportTextCoordinatesSingle`, `exportTextData`][F][P]

* `nItem = 5`: **[0] anchor** = (anchor x, baseline of the first line), then the text box corners **[1] bottom-left, [2] bottom-right,
  [3] top-right, [4] top-left** (box from descent below the last baseline to ascent above the first, across the line width).
  All five are rotated with the text angle. [F] shows exactly this: anchor (−3819, 1659), BL (−3819, 1453), BR (−248, 1453),
  TR (−248, 2610), TL (−3819, 2610) for a left-aligned text. For centred or right alignment the anchor x is the centre or right edge
  (OOM keeps the anchor x and puts the box around the laid-out lines).
* `Ang` = text rotation in 0.1°. Horizontal alignment comes from the **symbol** (3.6). OOM's importer reads vertical alignment from the
  symbol too.
* Text bytes follow the coordinates: **UTF-16LE, no BOM, lines separated by CR LF, NUL-terminated, zero-padded**. `nText` =
  bytes / 8. OCAD and OOM pad to a **multiple of 64 bytes** (8 units) [F: "Hi There" occupies 64 bytes][O `exportTextData(…, 64, 128)`, max
  8192 bytes]. Purple Pen uses the minimum `chars/4 + 1` units [P]. Mimic OCAD with 64-byte chunks.
* OOM doubles a leading newline, because its importer strips one leading CR LF [O]. Avoid texts that start with a newline.
* Formatted text (`Otp 5`, 4 box corners BL, BR, TR, TL) is not needed.
* bbox: the four box corners ± the text symbol's Extent. With OOM's Extent 0 that is the box itself. ⚠ Without font metrics freemapper must
  estimate the box (e.g. width ≈ 0.6·em·chars, ascent 0.9·em, descent 0.25·em). Make it generous rather than tight, because a too-small box
  risks "damaged object" (OOM #1134). OCAD renders from the anchor and the symbol. That the box is only used for selection and bbox is
  plausible but unverified.

---

## 6. Georeferencing: string type 1039 (si_ScalePar) [S][O `exportGeoreferencing`, `OcdGeorefFields`][F][W]

OOM writes (field order as below, `fixed` notation):
`"\tm{m}\tg{g:.4}\tr{r}\tx{x}\ty{y}\ta{a:.8}\td{d:.6}\ti{i}\tb0.00\tc0.00"`

| Code | Meaning | Unit | freemapper value |
|---|---|---|---|
| m | map scale denominator | – | 1000 or 4000 |
| g | screen-grid distance on paper | mm | `d·1000/m` (e.g. 25 at 1:4000 with d=100) |
| r | real-world coordinates on | 0/1 | **1** (OOM sets 1 only if the CRS translated) |
| x | easting of paper point (0,0) | m | see below |
| y | northing of paper point (0,0) | m | see below |
| a | real-world angle (grivation) | degrees, CCW | `declination − convergence` (freemapper `grivation_deg()`) |
| d | grid distance in terrain | m | e.g. 100 |
| i | grid and zone code | – | **8008** |
| b, c | additional local easting/northing offset (GPS only [W]) | m | 0 |

* **x/y meaning:** "enter the coordinate values for the **center of your map**" [W], i.e. the real-world coordinates of OCAD paper (0,0)
  [O: `x,y = georef.toProjectedCoords(MapCoord{})`]. OOM rounds them to integers ("OCD easting and northing is integer") [O]. OCAD itself
  writes `x0.00` (2 decimals) [F]. Choose an **integer origin** so neither interpretation loses anything.
* **Transform** (OOM `Georeferencing::updateTransformation` with combined scale factor 1, which is how OOM imports OCD [O]). With paper
  `(X,Y)` in mm (y up), `s = m/1000` metres per mm and `a` in radians:
  `E = x + s·( X·cos a + Y·sin a)` and `N = y + s·(−X·sin a + Y·cos a)`.
  Inverse: `X = ( cos a·ΔE − sin a·ΔN)/s`, `Y = ( sin a·ΔE + cos a·ΔN)/s`.
  Sign check: a > 0 (magnetic north east of grid north) makes grid north point a° counter-clockwise from paper up. This matches
  freemapper's `map_to_projected` exactly, with y negated.
* ⚠ **OCAD has no grid/combined scale factor.** Its scale is relative to the grid [O issue #2080: "OCAD's combined scale factor is 1.0"].
  freemapper's paper geometry includes `grid_scale_factor k` (≈ 0.99968 at Strausberg, UTM 33 near 13.84° E), so a 1:1 copy of the paper
  coordinates drifts by |1−k|·distance ≈ **0.3 m per km** from the origin. Two options:
  * **A (what OOM does):** copy paper coordinates and set x/y = ref point. Accept the drift, plus ≤0.5 m if x/y are rounded.
  * **B (exact, recommended):** set `x0 = round(E_ref)`, `y0 = round(N_ref)` and compute every OCAD coordinate as the OCAD inverse of the
    object's projected position (`map_to_projected`, then the formula above). Equivalently, scale paper coordinates by `k` about the ref point
    and shift by the rounding offset. Symbols are unaffected.
* **Grid/zone code for EPSG:25833**, from `ocd_georef_fields.cpp` [O]. `toOcd(Epsg, 25833)` returns the **first** table match:
  `{ OcdGrid::Germany, 8, 25833 }, // ETRS89 / UTM zone 33N` with `OcdGrid::Germany = 8`, and `combineGridZone = grid·1000 + zone`
  gives **`i8008`**. Further rows exist for the same EPSG: `{ OcdGrid::Italy, 8, 25833 }` (16008), `{ OcdGrid::Unknown21, 4, 25833 }`
  (21004), `{ OcdGrid::Unknown36, 1, 25833 }` (36001), `{ OcdGrid::UtmEtrs89, 6, 25833 }` (63006). OOM's tests contain the analogous
  `"Kaiserdom … ETRS89/UTM zone 32N" → OcdGeorefFields{3.26, 15000, 477509, 5550982, 8007, 1}` [O `georef_ocd_mapping_t.cpp`].
  Fallback: WGS 84 / UTM 33N = `2033` (`OcdGrid::Utm=2`). ⚠ It is unverified that OCAD **12** (rather than 2018+) knows 8008. If it does not,
  paper↔grid georeferencing still works through m/x/y/a; only CRS-aware features need the code.
* Precision: OOM writes `a` with 8 decimals (OOM #1925), OCAD with 6 [F]. freemapper rounds grivation to 0.01°. Write it as is.
* Example (1:4000): `\tm4000\tg25.0000\tr1\tx413500\ty5824200\ta5.37000000\td100.000000\ti8008\tb0.00\tc0.00`

---

## 7. Background maps: string type 8 (si_BackgroundMap) [S][O `stringForTemplate`][P][W]

OOM V11/V12 format:
`"{path}\ts{0|1}\tr1\tu{u}\tv{v}\tx{x}\ty{y}\ta{deg}\tb{deg}\td{dim}"`

| Code | Meaning [S] | Value |
|---|---|---|
| first | file name | see paths below |
| s | visible in normal, spot-colour and draft mode | 1 / 0 |
| r | visible in background favourites | 1 (OOM; "r and s changed meaning in V11") |
| u, v | **pixel size in paper mm** (x, y) | `px_m · 1000 / m` (`u=v` for square pixels) |
| x, y | **paper position (mm, y up) of the image centre** | see below |
| a, b | angle ω / φ in degrees, CCW | rotation of the image x/y axes on paper. North-up image: `a = b = grivation` |
| d | dim % (0 = original, 100 = white [W]) | `round(100·(1 − opacity))` |
| t | transparent 0/1 | optional |

* **Semantics derived from OOM** [O]: OOM's image template is drawn centred on its origin
  (`drawImage(-w/2, -h/2)`). `template_to_map` = `R(rot)·diag(sx,sy)` + offset with `sx` = mm per pixel. The export writes
  `x = template_x/1000`, `y = −template_y/1000`, `u,v = scale`, `a = b = deg(rot)`. Purple Pen independently writes `x,y = centerPoint`,
  `u,v = mm-per-pixel × scale`, `a = angle` [P]. So `paper = (x,y) + R(a)·(u·px, v·qy)`, with (px, qy) measured from the image centre,
  qy pointing up.
* **From a world file** (A, D, B, E, C, F; C/F = centre of the upper-left pixel), image W×H pixels:
  * centre in world: `Ec = C + A·(W−1)/2 + B·(H−1)/2`, `Nc = F + D·(W−1)/2 + E·(H−1)/2`. Then `(x,y)` = OCAD inverse transform (§6) in mm.
  * north-up (B = D = 0): `u = A·1000/m`, `v = −E·1000/m`, `a = b = grivation` (grid north appears a° CCW on paper).
  * general: map the vectors `(A,D)` (pixel x) and `−(B,E)` (pixel up) to paper. Take their lengths for u/v and their angles for a, and b − 90°.
    ⚠ The exact meaning of ω versus φ is not documented. OOM and PP write a = b.
* Numbers: OOM uses Qt "smart" notation (6–10 significant digits, not fixed). Write fixed decimals (x/y 4, u/v 10, a/b 8). `.` as the separator.
* **Paths:** OOM writes a bare file name when the template is in the .ocd's folder, else the **absolute** path, with `/`→`\` [O
  `pathForTemplate`]. Purple Pen always writes absolute paths [P]. OCAD shows absolute paths for other folders, "however the relative path
  is also saved … the background map is still found if the map and background map files are moved … but the relative path keeps to be
  the same" [W, current OCAD]. ⚠ There is no evidence that OCAD 12 accepts a relative path with sub-folders (`templates\ortho.jpg`) in the first field.
  freemapper's absolute macOS/Linux paths are useless on Windows. **Safest:** place the image and its world file **next to the .ocd** and
  write the bare file name. Otherwise test `templates\x.jpg` in OCAD 12.
* **World files:** OCAD reads them when a raster is opened interactively. The "Background Map (Georeferenced)" dialog offers "Existing Offset
  and Angle" [W]. For a stored type-8 string OCAD uses the stored x/y/u/v/a/b, and OOM always writes them (loading the image first to get
  them) [O]. ⚠ It is unverified whether a string with only the file name makes OCAD fall back to the world file. **Always write explicit
  u/v/x/y/a/b.** Keep the `.jgw/.pgw` next to the image anyway (useful when the user re-opens it).
* Formats: JPEG/PNG/BMP/GIF/TIFF [W]. ⚠ Avoid 32-bit float GeoTIFFs (DGM). OCAD 9 loaded but did not display some GeoTIFFs (OOM #1648).
  Export 8-bit PNG/JPEG for hill shading.
* Order: OOM writes templates in reverse map order [O]. OCAD's list order of type-8 strings = background map order (unverified which end is top).

---

## 8. Other parameter strings

| Type | Purpose | Needed? | Format (source) |
|---|---|---|---|
| 1061 | map notes | optional. **Good place for the credit line** | first field = text, lines CR LF, no TAB [S][O] (V≤10: type 11) |
| 1030 | view: centre/zoom | optional | `\tx{cx}\ty{cy}\tz{zoom}\tv0\tm50\tt50\tb50\tc50\th0\td0\tk0` with cx/cy in mm, y up [O]. OCAD adds `\tp1\ti0\tl0` [F] |
| 1024 | display par | optional | `\ts0\tg-1` [F] |
| 1026 | print par | optional (sheet as print area) | `\ta{scale}\tr1\tL{l}\tB{b}\tR{r}\tT{t}` mm [S][P]. OCAD wrote `\ta10000.00\tL…\tB…\tR…\tT…\tx0.019531\ty0.019531\tg0\td0\ti100\tw100` [F]. ⚠ untested in OCAD 12 |
| 15 | symbol tree | no | [F] |
| 10 | spot colours | no | [O] |
| 12, 18, 27, 29 … | zoom history, bookmarks, layout objects, … | no | [S] |

OOM writes only 1039, 1030 (when a view exists), 1061 (if notes), 9/10 and 8. No 1024/1026/15 [O `exportSetup`, `exportExtras` is empty].

---

## 9. Symbol numbers versus OCAD's official ISSprOM 2019 / school 2019 sets

freemapper's codes come from OOM's `ISSprOM 2019_4000.omap` (OOM commit eae21b0) plus `school-2019-additions.xml` (321, 541–545)
and `sheet-additions.xml` (991–994).

Sources for this section:
* **[OS]** OCAD's official ISSprOM 2019-2 (Rev 6) set, decoded from `ISSprOM_blank.ocd` (Purple Pen repo, OCAD 2018 format). I re-checked
  every OCAD number below myself.
* Cross-checks from a separate research pass: British Orienteering's ISSprOM Rev 6.3 OCAD 12 file ("renumbered to match OCAD set"), a Swedish
  OCAD-derived set (ocad2geojson test data), British Orienteering's school set 2019 (Jan 2022), and omapwiki (the IOF symbol wiki).

**Background on numbering:** OOM's set follows the *first* 2019 edition. ISSprOM **2019-2** renumbered vegetation: 410 = "Vegetation: fight",
411 = "Uncrossable vegetation". IOF sub-numbers are 501.1/.2/.3, 505 (no subs) and 513.1/.2. OCAD 12 does **not** ship ISSprOM 2019. The
official set is OCAD 2018+, which opens V12 files.

| Code | OOM name (type) | OOM rule would write | OCAD ISSprOM 2019-2 [OS] | **Write** | School set 2019 | Conf. |
|---|---|---|---|---|---|---|
| 101 | Contour (line) | 101000 | 101.000 Contour (line) | **101000** | 101.000 | high |
| 102 | Index contour (line) | 102000 | 102.000 Index contour (line) | **102000** | 102.000 | high |
| 201 | Impassable cliff (line) | 201000 | 201.000 Uncrossable cliff (line) | **201000** | 201.000 | high |
| 301 | Uncrossable body of water (combined → area) | 301000 | 301.000 Uncrossable body of water (Area & Line) (area) | **301000** | 301.000 | high |
| 301.1 | Uncrossable body of water, fill (area) | 301100 ✗ (absent) | 301.001 Uncrossable body of water (Area) | **301001** | 301.001 | high |
| 301.6 | Uncrossable body of water, bank line (line) | 301600 ✗ (absent) | 301.002 Uncrossable body of water (Line) | **301002** | 301.002 | high |
| 321 | Hydrant (point) | 321000 | – (not in sprint set) | **321000** | 321.000 Hydrant (point) | medium |
| 401 | Open land (area) | 401000 | 401.000 Open land (area) | **401000** | 401.000 | high |
| 410.1 | Impassable veg., hedge (line) | 410001 ✗ = 410.001 "Vegetation: fight, one direction" (area) | **411.002** Uncrossable vegetation, hedge (line). 410.002 = "Vegetation: fight, hedge" | **411002** (sprint) / **410002** (school, 1st-edition numbering) | 410.002 | high / medium |
| 418 | Prominent bush (point) | 418000 | 418.000 (point) | **418000** | 418.000 | high |
| 501 | Paved area, light, with border (combined → area) | 501000 | 501.000 Paved area, upper, light traffic (Area & Line) (area) | **501000** | 501.000 | high |
| 501.1 | Step or edge of paved area (line) | 501001 ✗ = area | **501.100** Step or edge of paved area (line) | **501100** | 501.100 | high |
| 501.5 | Paved area, light, no border (area) | 501005 ✗ (absent) | 501.001 Paved area, upper, light traffic (Area) | **501001** | 501.001 | high |
| 501.10 | Paved area, heavy, with border (combined → area) | 501010 ✗ = "Paved footpath 1.6 m" (line) | 501.020 Paved area, heavy traffic (Area & Line). 501.021 = (Area) only | **501020** | 501.020 | high |
| 505.1 | Unpaved footpath 1.4 m (line) | 505001 ✗ = 2 m | 505.000 Unpaved footpath or track, 1.6 m (line), the smallest OCAD variant | **505000** | 505.000 ("1.4 m") | high |
| 505.2 | Unpaved footpath 2 m (line) | 505002 ✗ = 3 m | 505.001 … 2 m (line) | **505001** | 505.001 | high |
| 506 | Small unpaved footpath (line) | 506000 | 506.000 (line) | **506000** | 506.000 | high |
| 513.2 | Passable retained wall (line) | 513002 ✗ (absent) | **513.200** Passable retained wall (line) | **513200** | 513.200 | high |
| 515 | Impassable wall (line) | 515000 | 515.000 Uncrossable wall (line) | **515000** | 515.000 | high |
| 518 | Impassable fence (line) | 518000 | 518.000 Uncrossable fence or railing (line) | **518000** | 518.000 | high |
| 520 | Area not to be entered (area) | 520000 | 520.000 … (Area & Line) (area). 520.001 Area, 520.002 Line | **520000** | 520.000 | high |
| 521 | Building (combined → area) | 521000 | 521.000 Building (area). 521.002 outline (line) | **521000** | 521.000 | high |
| 521.1 | Building, minimum size (point) | 521001 ✗ (absent) | **521.003** Building, minimum (point) | **521003** | 521.003 | high |
| 541 | Playground equipment (line) | 541000 | – | **541000** | 541.000 (line) | medium |
| 542 | Table (point) | 542000 | – | **542000** | 542.000 (point) | medium |
| 543 | Bench, seat (line) | 543000 | – | **543000** | 543.000 (line) | medium |
| 544 | Pole, small pylon (point) | 544000 | – | **544000** | 544.000 (point) | medium |
| 545 | Lamppost (point) | 545000 | – | **545000** | 545.000 (point) | medium |
| 601 | Magnetic north line (line) | 601000 | 601.000 Magnetic north line black (line). 601.001 blue | **601000** | 601.000 | high |
| 991–994 | sheet texts / bar-scale line | 991000 … 994000 | none. OCAD's texts are 900.000 (36 pt), 901.000 (18 pt), 902.000 (8 pt) | **991000 … 994000** (+ e.g. 993001 for the 2nd alignment) | none at 99x | high (map sets) |

✗ means OOM's rule hits a different or absent OCAD symbol. All "Write" numbers keep the object type of freemapper's symbol. 501, 501.10 and 521
are combined symbols in OOM and become one OCAD **area** symbol.

* The school set is based on the 2019 **first** edition (hedge 410.002) and requires OCAD 2018+, so one export cannot match both sets for the
  hedge. Choose 410.1 → 411002 (sprint) or 410002 (school) by target. The school numbers come from British Orienteering's file, not OCAD's
  own (medium confidence).
* 991–994 do not collide with anything. Using 900/901/902 instead would let the official set restyle the sheet texts.

Practical notes for "Load Colors and Symbols From" [W "Map"]:
* "**Replace** existing colors and symbols" overwrites the whole symbol set *and* colour table. "Add" adds. A CRT (cross-reference) file
  can map numbers (`526.000 813.001`). ⚠ Whether "Replace" keeps map symbols that are absent from the loaded set (the 99x sheet symbols,
  school additions in a sprint set) is unverified. It is likely that they are dropped and their objects become "unknown symbol". Test this.
  "Map › Symbol Set Conversion" with a CRT file can keep extra symbols. "Add" plus manual cleanup is another fallback.
* Placeholder colours do not matter after "Replace". Placeholder symbols that survive reference colour **numbers** that may then mean
  something else.

---

## 10. Pitfalls (OOM handles these; a new writer trips over them)

1. **Index `Len` is bytes** (56 + 8·n), not coordinate count (§1.3). This was a real cause of "damaged objects" in OCAD 9/10 [O].
2. **bbox must cover every coordinate including Bézier handles, plus the symbol Extent.** Otherwise Optimize/Repair deletes the object [O #1134][P][F].
3. **y inversion everywhere:** object coordinates, symbol element coordinates, text box corners (BL/BR/TR/TL are *paper* corners
   after inversion), template y, view y. Angles keep their sign [O].
4. **Drawing area ±2000 mm** in the Orienteering edition [W]. Centre the sheet at (0,0) [O warns outside ±2 m].
5. **TDPoly flags live in the low 8 bits.** Never store raw 0.01 mm values unshifted, not even in the bbox. Hole flag = y bit 2 on the **first**
   point of each hole (OOM's model marks the last point of the previous ring) [O].
6. **Multi-part lines must be split.** Only areas have holes [O].
7. **Text:** UTF-16LE, CR LF, NUL, `nText` in 8-byte units, 64-byte padding. One text symbol per horizontal alignment. Anchor at the first
   baseline [O][F].
8. **Symbol `Size`** must equal header + 8·DataSize (point/area) or + 8·ΣDSize (line). Text 1036 exactly. Line 872, area 832, point 800 [O][F].
9. **Circle elements:** Ø is the centre-line diameter (outer Ø − width). Wrong values make invisible symbols [O #2471].
10. **Symbol numbers:** OCAD's sub-number is a three-place *decimal* ("501.1" = 501100). OOM's integer rule ("501.1" → 501001) picks wrong
    official symbols, so use the explicit table in §9. Numbers must be unique and never 0.
11. **Overprint `o0`**, not OOM's `o1` [O #2228].
12. **No combined scale factor in OCAD.** Re-project or accept ~0.3 m/km drift (§6) [O #2080].
13. **Last object-index `next` is garbage in OCAD files.** Bounds-check when reading them in tests. Write 0 yourself [F][P].
14. **Parameter strings:** no TAB inside values (map notes!), ASCII only, NUL terminator counted in `Len` [O][P].
15. **Delphi packing:** no padding inside records. Several unaligned fields: `PointSymNumber` @993 (text symbol), `ModificationDate` @36 (object),
    `Description` is UTF-16 at an even offset. Serialise field by field, never by `transmute` of a `repr(Rust)` struct.
16. **Object Status = 1.** Status 0/3 objects are skipped by readers [O]. `ObjType` in the index must equal `Otp` in the record.
17. **Combined symbols** (OOM 501 "with border", 521 Building = fill + outline): OCAD has no combined symbol. An area symbol can only reference one border
    *line symbol* via `BorderSym/BorderOn` (V9+) [O `exportCombinedAreaSymbol`]. For placeholders a plain filled area is enough. Do not invent extra numbered
    border symbols that might clash with the official set.
18. **Spurious objects** (1-point lines, <3-point areas, empty texts) are deleted by OCAD's Optimize [W].
19. **Template positioning** must be explicit. Absolute non-Windows paths are useless (§7).

---

## 11. Minimal file recipe (checklist)

1. `buf = vec![0; 60]` (header). Append the first **string index block** (4100 zero bytes), the first **symbol index block** (1028) and
   the first **object index block** (10244). Remember their positions (60 / 4160 / 5188 if you do not pad). Put them in header offsets 32 / 8 / 12.
   Header: `0x0CAD, 0, 0, 12, 0, 0(or 1)`, everything else 0.
2. Helper `add_entity(bytes) -> pos`: pad `buf` to a multiple of 8, push, return the position. Helper `index_insert(chain, entry)`: first free slot.
   If the block is full, append a new zeroed block, link it via `next`, and use slot 0.
3. **Strings:** 1039 (§6: m, g, r1, x0, y0, a, d, i8008, b0, c0), then one type-9 string per colour in priority order (top first, `n = index`,
   `o0`, `t100`), then optional 1061 credit/notes (ASCII, CR LF), optional 1030 view centre, then one type-8 string per raster template (§7,
   explicit u/v/x/y/a/b, image next to the .ocd). Each string is bytes + NUL. Index entry `{pos, len = bytes+1, type, 0}`.
4. **Symbols** (796-byte base: Size, SymNum = **explicit OCAD number from §9**, Otp, Extent, nColors/Colors, UTF-16 name @56, icon @184 filled with 124):
   * point: +DataSize=3, Reserved, dot element `{4, 0, c, 0, Ø, 1, 0, 0}` + (0,0). Size 824.
   * line: +76 bytes `{LineColor, LineWidth, LineStyle, …0}`. Optional dash: MainLength = EndLength = dash, MainGap = gap. Size 872, Extent w/2.
   * area: +36 bytes `{BorderSym 0, FillColor, HatchMode 0, …, FillOn 1, …, DataSize 0}`. Size 832.
   * text: +240 bytes `{FontName "Arial", FontColor, FontSize = pt·10, Weight, …, WordSpace 100, Alignment, LineSpace 120, …0}`. Size 1036.
     One symbol per used horizontal alignment.
   Symbol index entry = position.
5. **Objects** (56-byte header + TDPoly `(v<<8)|flags`, y negated, coordinates in 0.01 mm within ±2000 mm):
   * point: Otp 1, Ang, 1 coordinate. Line: Otp 2, one polyline per object. Area: Otp 3, rings with y|=2 on the first point of each hole.
   * text: Otp 4, Ang, 5 coordinates (anchor at baseline, BL, BR, TR, TL), UTF-16LE + CR LF + NUL padded to 64 bytes, `nText = bytes/8`.
   * index entry: bbox = coordinate min/max ± symbol Extent (shifted <<8), Pos, **Len = 56 + 8·(nItem+nText)**, Sym, ObjType = Otp, Status 1,
     Color = the symbol's main colour number.
6. Patch the header (first block positions), write the file. Verify by re-reading: walk all three chains, check `Size`/`Len`
   arithmetic, check that bbox ⊇ points, and open the file in OCAD (Viewer) and OOM. Then run OCAD's *Optimize/Repair*: "damaged objects = 0" is the acceptance test.
