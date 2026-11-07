//! OCAD 12 binary file builder (layout: `docs/ocad/ocd12-reference.md`).
//!
//! Everything is little-endian and packed; records are written field by field at their offsets.

/// Size of a symbol index block: next + 256 positions.
pub(crate) const SYMBOL_BLOCK_SIZE: usize = 4 + 256 * 4;
/// Size of an object index block: next + 256 entries of 40 bytes.
pub(crate) const OBJECT_BLOCK_SIZE: usize = 4 + 256 * 40;
/// Size of a string index block: next + 256 entries of 16 bytes.
pub(crate) const STRING_BLOCK_SIZE: usize = 4 + 256 * 16;
const HEADER_SIZE: usize = 60;
const BASE_SYMBOL_SIZE: usize = 796;
const OBJECT_HEADER_SIZE: usize = 56;
/// Icon palette index of white (5 × 5 × 5 colour cube).
pub const ICON_WHITE: u8 = 124;

/// One coordinate in 0.01 mm on OCAD paper (y up) with its flag bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OcdCoord {
    pub x: i32,
    pub y: i32,
    pub xflags: u8,
    pub yflags: u8,
}

impl OcdCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            xflags: 0,
            yflags: 0,
        }
    }
}

/// y flag: first point of a hole in an area.
pub const Y_HOLE: u8 = 2;
/// x flags: first and second Bézier control point.
pub const X_CURVE_1: u8 = 1;
pub const X_CURVE_2: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left = 0,
    Centre = 1,
    Right = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OcdSymbolKind {
    /// A filled disc of `diameter` (0.01 mm).
    Point {
        color: i16,
        diameter: i16,
    },
    /// A line of `width` (0.01 mm), optionally dashed (dash, gap).
    Line {
        color: i16,
        width: i16,
        dash: Option<(i16, i16)>,
    },
    Area {
        color: i16,
    },
    /// Arial at `size` (pt × 10).
    Text {
        color: i16,
        size: i16,
        bold: bool,
        align: TextAlign,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct OcdSymbol {
    /// Symbol number × 1000 (521000 for 521.000).
    pub number: i32,
    pub name: String,
    pub kind: OcdSymbolKind,
    /// Icon palette index of the glyph colour.
    pub icon_color: u8,
}

impl OcdSymbol {
    fn otp(&self) -> u8 {
        match self.kind {
            OcdSymbolKind::Point { .. } => 1,
            OcdSymbolKind::Line { .. } => 2,
            OcdSymbolKind::Area { .. } => 3,
            OcdSymbolKind::Text { .. } => 4,
        }
    }

    /// Reach of the rendering beyond the coordinates (0.01 mm).
    fn extent(&self) -> i32 {
        match self.kind {
            OcdSymbolKind::Point { diameter, .. } => (i32::from(diameter) + 1) / 2,
            OcdSymbolKind::Line { width, .. } => i32::from(width) / 2,
            OcdSymbolKind::Area { .. } | OcdSymbolKind::Text { .. } => 0,
        }
    }

    fn color(&self) -> i16 {
        match self.kind {
            OcdSymbolKind::Point { color, .. }
            | OcdSymbolKind::Line { color, .. }
            | OcdSymbolKind::Area { color }
            | OcdSymbolKind::Text { color, .. } => color,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OcdObjectKind {
    /// Angle in 0.1°, counter-clockwise.
    Point {
        angle: i16,
    },
    Line,
    Area,
    /// Unformatted text: coordinates are the anchor and the box corners BL, BR, TR, TL.
    Text {
        angle: i16,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct OcdObject {
    pub symbol: i32,
    pub kind: OcdObjectKind,
    pub coords: Vec<OcdCoord>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum OcdBuildError {
    #[error("symbol number {0} is used twice")]
    DuplicateSymbol(i32),
    #[error("object refers to symbol {0}, which is not in the file")]
    UnknownSymbol(i32),
    #[error("an object of symbol {symbol} has the wrong type for its symbol")]
    WrongType { symbol: i32 },
}

/// Collects strings, symbols and objects and lays out an OCAD 12 file.
#[derive(Debug, Default)]
pub struct OcdBuilder {
    strings: Vec<(i32, String)>,
    symbols: Vec<OcdSymbol>,
    objects: Vec<OcdObject>,
}

impl OcdBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// A parameter string of `kind` (e.g. 9 colour, 1039 scale); `text` without the NUL.
    pub fn add_string(&mut self, kind: i32, text: &str) {
        self.strings.push((kind, text.to_string()));
    }

    pub fn add_symbol(&mut self, symbol: OcdSymbol) -> Result<(), OcdBuildError> {
        if self.symbol(symbol.number).is_some() {
            return Err(OcdBuildError::DuplicateSymbol(symbol.number));
        }
        self.symbols.push(symbol);
        Ok(())
    }

    pub fn add_object(&mut self, object: OcdObject) -> Result<(), OcdBuildError> {
        let symbol = self
            .symbol(object.symbol)
            .ok_or(OcdBuildError::UnknownSymbol(object.symbol))?;
        if symbol.otp() != object_otp(&object.kind) {
            return Err(OcdBuildError::WrongType {
                symbol: object.symbol,
            });
        }
        self.objects.push(object);
        Ok(())
    }

    fn symbol(&self, number: i32) -> Option<&OcdSymbol> {
        self.symbols.iter().find(|s| s.number == number)
    }

    /// Lays out the file: header, the three index chains, then strings, symbols and objects.
    pub fn finish(self) -> Vec<u8> {
        let mut buf = vec![0u8; HEADER_SIZE];
        put_u16(&mut buf, 0, 0x0CAD);
        put_u16(&mut buf, 4, 12);
        let string_blocks = add_chain(&mut buf, self.strings.len(), STRING_BLOCK_SIZE);
        let symbol_blocks = add_chain(&mut buf, self.symbols.len(), SYMBOL_BLOCK_SIZE);
        let object_blocks = add_chain(&mut buf, self.objects.len(), OBJECT_BLOCK_SIZE);
        put_u32(&mut buf, 32, string_blocks[0] as u32);
        put_u32(&mut buf, 8, symbol_blocks[0] as u32);
        put_u32(&mut buf, 12, object_blocks[0] as u32);

        for (i, (kind, text)) in self.strings.iter().enumerate() {
            let mut bytes = text.as_bytes().to_vec();
            bytes.push(0);
            let pos = add_entity(&mut buf, &bytes);
            let entry = entry_offset(&string_blocks, i, 16);
            put_u32(&mut buf, entry, pos as u32);
            put_i32(&mut buf, entry + 4, bytes.len() as i32);
            put_i32(&mut buf, entry + 8, *kind);
        }
        for (i, symbol) in self.symbols.iter().enumerate() {
            let pos = add_entity(&mut buf, &symbol_record(symbol));
            put_u32(&mut buf, entry_offset(&symbol_blocks, i, 4), pos as u32);
        }
        for (i, object) in self.objects.iter().enumerate() {
            let symbol = self.symbol(object.symbol).expect("checked in add_object");
            let record = object_record(object);
            let pos = add_entity(&mut buf, &record);
            let entry = entry_offset(&object_blocks, i, 40);
            let extent = symbol.extent();
            let (min_x, min_y, max_x, max_y) = bounds(&object.coords);
            put_i32(&mut buf, entry, (min_x - extent) << 8);
            put_i32(&mut buf, entry + 4, (min_y - extent) << 8);
            put_i32(&mut buf, entry + 8, (max_x + extent) << 8);
            put_i32(&mut buf, entry + 12, (max_y + extent) << 8);
            put_i32(&mut buf, entry + 16, pos as i32);
            put_i32(&mut buf, entry + 20, record.len() as i32);
            put_i32(&mut buf, entry + 24, object.symbol);
            buf[entry + 28] = object_otp(&object.kind);
            buf[entry + 30] = 1;
            put_i16(&mut buf, entry + 32, symbol.color());
        }
        buf
    }
}

fn object_otp(kind: &OcdObjectKind) -> u8 {
    match kind {
        OcdObjectKind::Point { .. } => 1,
        OcdObjectKind::Line => 2,
        OcdObjectKind::Area => 3,
        OcdObjectKind::Text { .. } => 4,
    }
}

fn put_u16(buf: &mut [u8], at: usize, v: u16) {
    buf[at..at + 2].copy_from_slice(&v.to_le_bytes());
}

fn put_i16(buf: &mut [u8], at: usize, v: i16) {
    buf[at..at + 2].copy_from_slice(&v.to_le_bytes());
}

fn put_u32(buf: &mut [u8], at: usize, v: u32) {
    buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

fn put_i32(buf: &mut [u8], at: usize, v: i32) {
    buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// Pads to a multiple of 8, appends `bytes` and returns their position.
fn add_entity(buf: &mut Vec<u8>, bytes: &[u8]) -> usize {
    buf.resize(buf.len().next_multiple_of(8), 0);
    let pos = buf.len();
    buf.extend_from_slice(bytes);
    pos
}

/// Appends enough linked, zeroed index blocks for `entries` (at least one) and returns their
/// positions.
fn add_chain(buf: &mut Vec<u8>, entries: usize, block_size: usize) -> Vec<usize> {
    let blocks: Vec<usize> = (0..entries.div_ceil(256).max(1))
        .map(|_| add_entity(buf, &vec![0u8; block_size]))
        .collect();
    for pair in blocks.windows(2) {
        put_u32(buf, pair[0], pair[1] as u32);
    }
    blocks
}

/// File position of index entry `i` in a chain.
fn entry_offset(blocks: &[usize], i: usize, entry_size: usize) -> usize {
    blocks[i / 256] + 4 + (i % 256) * entry_size
}

fn bounds(coords: &[OcdCoord]) -> (i32, i32, i32, i32) {
    coords.iter().fold(
        (i32::MAX, i32::MAX, i32::MIN, i32::MIN),
        |(min_x, min_y, max_x, max_y), c| {
            (
                min_x.min(c.x),
                min_y.min(c.y),
                max_x.max(c.x),
                max_y.max(c.y),
            )
        },
    )
}

fn coord_bytes(c: &OcdCoord) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[0..4].copy_from_slice(&((c.x << 8) | i32::from(c.xflags)).to_le_bytes());
    bytes[4..8].copy_from_slice(&((c.y << 8) | i32::from(c.yflags)).to_le_bytes());
    bytes
}

/// The 22 × 22 icon: white with a glyph for the symbol type, bottom row first.
fn icon(symbol: &OcdSymbol) -> [u8; 484] {
    let mut icon = [ICON_WHITE; 484];
    let inside = |x: i32, y: i32| match symbol.kind {
        OcdSymbolKind::Point { .. } => (x - 11) * (x - 11) + (y - 11) * (y - 11) <= 25,
        OcdSymbolKind::Line { .. } => (3..19).contains(&x) && (10..12).contains(&y),
        OcdSymbolKind::Area { .. } => (4..18).contains(&x) && (4..18).contains(&y),
        // A capital T.
        OcdSymbolKind::Text { .. } => {
            ((5..17).contains(&x) && (15..18).contains(&y))
                || ((10..12).contains(&x) && (4..18).contains(&y))
        }
    };
    for y in 0..22 {
        for x in 0..22 {
            if inside(x, y) {
                icon[(y * 22 + x) as usize] = symbol.icon_color;
            }
        }
    }
    icon
}

fn symbol_record(symbol: &OcdSymbol) -> Vec<u8> {
    let size = match symbol.kind {
        OcdSymbolKind::Point { .. } => 824,
        OcdSymbolKind::Line { .. } => 872,
        OcdSymbolKind::Area { .. } => 832,
        OcdSymbolKind::Text { .. } => 1036,
    };
    let mut r = vec![0u8; size];
    put_i32(&mut r, 0, size as i32);
    put_i32(&mut r, 4, symbol.number);
    r[8] = symbol.otp();
    // Points and texts can be rotated.
    r[9] = u8::from(matches!(
        symbol.kind,
        OcdSymbolKind::Point { .. } | OcdSymbolKind::Text { .. }
    ));
    put_i32(&mut r, 16, symbol.extent());
    put_i16(&mut r, 26, 1);
    put_i16(&mut r, 28, symbol.color());
    for (i, unit) in symbol.name.encode_utf16().take(63).enumerate() {
        put_u16(&mut r, 56 + 2 * i, unit);
    }
    r[184..668].copy_from_slice(&icon(symbol));
    let b = BASE_SYMBOL_SIZE;
    match symbol.kind {
        OcdSymbolKind::Point { color, diameter } => {
            // One dot element of 3 units: 16-byte header and the centre point.
            put_u16(&mut r, b, 3);
            put_i16(&mut r, b + 4, 4);
            put_i16(&mut r, b + 8, color);
            put_i16(&mut r, b + 12, diameter);
            put_i16(&mut r, b + 14, 1);
        }
        OcdSymbolKind::Line { color, width, dash } => {
            put_i16(&mut r, b, color);
            put_i16(&mut r, b + 2, width);
            // Round joins and caps.
            put_i16(&mut r, b + 4, 1);
            if let Some((dash, gap)) = dash {
                put_i16(&mut r, b + 10, dash);
                put_i16(&mut r, b + 12, dash);
                put_i16(&mut r, b + 14, gap);
            }
        }
        OcdSymbolKind::Area { color } => {
            put_i16(&mut r, b + 4, color);
            r[b + 18] = 1;
        }
        OcdSymbolKind::Text {
            color,
            size,
            bold,
            align,
        } => {
            r[b] = 5;
            r[b + 1..b + 6].copy_from_slice(b"Arial");
            put_i16(&mut r, 828, color);
            put_i16(&mut r, 830, size);
            put_i16(&mut r, 832, if bold { 700 } else { 400 });
            put_i16(&mut r, 838, 100);
            put_i16(&mut r, 840, align as i16);
            put_i16(&mut r, 842, 120);
        }
    }
    r
}

fn object_record(object: &OcdObject) -> Vec<u8> {
    let text_bytes = match &object.kind {
        OcdObjectKind::Text { text, .. } => {
            let mut bytes: Vec<u8> = text
                .replace("\r\n", "\n")
                .replace('\n', "\r\n")
                .encode_utf16()
                .chain([0])
                .flat_map(u16::to_le_bytes)
                .collect();
            bytes.resize(bytes.len().next_multiple_of(64), 0);
            bytes
        }
        _ => Vec::new(),
    };
    let mut r = vec![0u8; OBJECT_HEADER_SIZE];
    put_i32(&mut r, 0, object.symbol);
    r[4] = object_otp(&object.kind);
    let angle = match object.kind {
        OcdObjectKind::Point { angle } | OcdObjectKind::Text { angle, .. } => angle,
        _ => 0,
    };
    put_i16(&mut r, 6, angle);
    put_u32(&mut r, 44, object.coords.len() as u32);
    put_u16(&mut r, 48, (text_bytes.len() / 8) as u16);
    for c in &object.coords {
        r.extend_from_slice(&coord_bytes(c));
    }
    r.extend_from_slice(&text_bytes);
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocd::read::read;

    fn dot(number: i32) -> OcdSymbol {
        OcdSymbol {
            number,
            name: "Prominent bush or small tree".into(),
            kind: OcdSymbolKind::Point {
                color: 3,
                diameter: 50,
            },
            icon_color: 10,
        }
    }

    fn line_symbol() -> OcdSymbol {
        OcdSymbol {
            number: 505000,
            name: "Unpaved footpath".into(),
            kind: OcdSymbolKind::Line {
                color: 14,
                width: 35,
                dash: Some((200, 25)),
            },
            icon_color: 0,
        }
    }

    fn area_symbol() -> OcdSymbol {
        OcdSymbol {
            number: 521000,
            name: "Building".into(),
            kind: OcdSymbolKind::Area { color: 2 },
            icon_color: 0,
        }
    }

    fn text_symbol() -> OcdSymbol {
        OcdSymbol {
            number: 991000,
            name: "Map title".into(),
            kind: OcdSymbolKind::Text {
                color: 2,
                size: 142,
                bold: true,
                align: TextAlign::Centre,
            },
            icon_color: 0,
        }
    }

    fn square(x: i32, y: i32, side: i32) -> Vec<OcdCoord> {
        [(0, 0), (side, 0), (side, side), (0, side), (0, 0)]
            .iter()
            .map(|&(dx, dy)| OcdCoord::new(x + dx, y + dy))
            .collect()
    }

    fn builder() -> OcdBuilder {
        let mut b = OcdBuilder::new();
        for s in [dot(418000), line_symbol(), area_symbol(), text_symbol()] {
            b.add_symbol(s).unwrap();
        }
        b
    }

    #[test]
    fn header_and_chains() {
        let mut b = builder();
        b.add_string(9, "Black\tn0\tc0\tm0\ty0\tk100\to0\tt100");
        let bytes = b.finish();
        assert_eq!(&bytes[0..2], &[0xAD, 0x0C]);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 12);
        assert_eq!(bytes[2], 0, "normal map");
        let file = read(&bytes);
        assert_eq!(file.version, 12);
        assert_eq!(file.symbols.len(), 4);
        assert_eq!(file.strings.len(), 1);
        assert!(file.objects.is_empty());
    }

    #[test]
    fn chains_grow_past_256_entries() {
        let mut b = builder();
        for i in 0..300 {
            b.add_object(OcdObject {
                symbol: 418000,
                kind: OcdObjectKind::Point { angle: 0 },
                coords: vec![OcdCoord::new(i * 10, 0)],
            })
            .unwrap();
            b.add_string(9, &format!("Colour {i}\tn{i}"));
        }
        let file = read(&b.finish());
        assert_eq!(file.objects.len(), 300);
        assert_eq!(file.strings.len(), 300);
        assert_eq!(file.objects[299].coords[0].0, 2990);
        assert_eq!(file.strings[299].text, "Colour 299\tn299");
    }

    #[test]
    fn symbol_record_sizes() {
        let file = read(&builder().finish());
        let sizes: Vec<(u8, i32)> = file.symbols.iter().map(|s| (s.otp, s.size)).collect();
        assert_eq!(sizes, [(1, 824), (2, 872), (3, 832), (4, 1036)]);
        let bush = &file.symbols[0];
        assert_eq!(bush.number, 418000);
        assert_eq!(bush.name, "Prominent bush or small tree");
        assert_eq!(bush.extent, 25);
        assert_eq!(bush.colors, [3]);
        // Dot element: type 4, colour 3, diameter 50, one coordinate.
        let element = &bush.bytes[800..816];
        assert_eq!(i16::from_le_bytes([element[0], element[1]]), 4);
        assert_eq!(i16::from_le_bytes([element[4], element[5]]), 3);
        assert_eq!(i16::from_le_bytes([element[8], element[9]]), 50);
        assert_eq!(i16::from_le_bytes([element[10], element[11]]), 1);
        let line = &file.symbols[1].bytes;
        assert_eq!(
            i16::from_le_bytes([line[796], line[797]]),
            14,
            "line colour"
        );
        assert_eq!(i16::from_le_bytes([line[798], line[799]]), 35, "line width");
        assert_eq!(i16::from_le_bytes([line[806], line[807]]), 200, "dash");
        assert_eq!(i16::from_le_bytes([line[810], line[811]]), 25, "gap");
        let area = &file.symbols[2].bytes;
        assert_eq!(i16::from_le_bytes([area[800], area[801]]), 2, "fill colour");
        assert_eq!(area[814], 1, "fill on");
        let text = &file.symbols[3].bytes;
        assert_eq!(&text[796..802], &[5, b'A', b'r', b'i', b'a', b'l']);
        assert_eq!(i16::from_le_bytes([text[830], text[831]]), 142, "size");
        assert_eq!(i16::from_le_bytes([text[832], text[833]]), 700, "bold");
        assert_eq!(
            i16::from_le_bytes([text[838], text[839]]),
            100,
            "word space"
        );
        assert_eq!(i16::from_le_bytes([text[840], text[841]]), 1, "centred");
        // Icon: white with a glyph.
        assert!(bush.bytes[184..668].contains(&ICON_WHITE));
        assert!(bush.bytes[184..668].contains(&10));
    }

    #[test]
    fn object_len_is_bytes() {
        let mut b = builder();
        b.add_object(OcdObject {
            symbol: 521000,
            kind: OcdObjectKind::Area,
            coords: square(0, 0, 1000),
        })
        .unwrap();
        let file = read(&b.finish());
        let o = &file.objects[0];
        assert_eq!(o.len, 56 + 8 * 5);
        assert_eq!(o.len as usize, o.record_size);
        assert_eq!((o.otp, o.index_otp, o.status), (3, 3, 1));
        assert_eq!((o.symbol, o.index_symbol, o.color), (521000, 521000, 2));
    }

    #[test]
    fn bbox_covers_points_and_extent() {
        let mut b = builder();
        b.add_object(OcdObject {
            symbol: 505000,
            kind: OcdObjectKind::Line,
            coords: vec![OcdCoord::new(-500, 300), OcdCoord::new(700, -200)],
        })
        .unwrap();
        b.add_object(OcdObject {
            symbol: 418000,
            kind: OcdObjectKind::Point { angle: 450 },
            coords: vec![OcdCoord::new(100, 100)],
        })
        .unwrap();
        let file = read(&b.finish());
        // Line extent 17 (half of 35), point extent 25.
        assert_eq!(file.objects[0].bbox, [-517, -217, 717, 317]);
        assert_eq!(file.objects[1].bbox, [75, 75, 125, 125]);
        assert_eq!(file.objects[1].angle, 450);
    }

    #[test]
    fn text_is_utf16_padded() {
        let mut b = builder();
        let coords = vec![OcdCoord::new(0, 0); 5];
        b.add_object(OcdObject {
            symbol: 991000,
            kind: OcdObjectKind::Text {
                angle: 0,
                text: "Maßstab 1:4000\nÄquidistanz".into(),
            },
            coords,
        })
        .unwrap();
        let file = read(&b.finish());
        let o = &file.objects[0];
        assert_eq!(o.text, "Maßstab 1:4000\r\nÄquidistanz");
        // 26 characters + NUL = 54 bytes, padded to 64.
        assert_eq!(o.n_text, 8);
        assert_eq!(o.len, 56 + 8 * 5 + 64);
        assert_eq!(o.otp, 4);
    }

    #[test]
    fn strings_are_nul_terminated() {
        let mut b = builder();
        b.add_string(1039, "\tm4000\tr1");
        let file = read(&b.finish());
        assert_eq!(file.strings[0].kind, 1039);
        assert_eq!(file.strings[0].text, "\tm4000\tr1");
        assert_eq!(file.strings[0].reserved, 10);
    }

    #[test]
    fn symbol_problems_are_refused() {
        let mut b = builder();
        assert_eq!(
            b.add_symbol(dot(418000)),
            Err(OcdBuildError::DuplicateSymbol(418000))
        );
        let point = |symbol| OcdObject {
            symbol,
            kind: OcdObjectKind::Point { angle: 0 },
            coords: vec![OcdCoord::new(0, 0)],
        };
        assert_eq!(
            b.add_object(point(1000)),
            Err(OcdBuildError::UnknownSymbol(1000))
        );
        assert_eq!(
            b.add_object(point(521000)),
            Err(OcdBuildError::WrongType { symbol: 521000 })
        );
    }
}
