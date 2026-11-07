//! A small OCD 12 reader for tests: walks the index chains and decodes what the builder writes.

use super::file::{OBJECT_BLOCK_SIZE, STRING_BLOCK_SIZE, SYMBOL_BLOCK_SIZE};

fn i16_at(b: &[u8], o: usize) -> i16 {
    i16::from_le_bytes([b[o], b[o + 1]])
}

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

fn i32_at(b: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadString {
    pub kind: i32,
    pub reserved: i32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadSymbol {
    pub pos: usize,
    pub size: i32,
    pub number: i32,
    pub otp: u8,
    pub extent: i32,
    pub colors: Vec<i16>,
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadObject {
    /// Index entry: bbox (decoded 0.01 mm), Len, Sym, ObjType, Status, Color.
    pub bbox: [i32; 4],
    pub len: i32,
    pub index_symbol: i32,
    pub index_otp: u8,
    pub status: u8,
    pub color: i16,
    /// Record.
    pub symbol: i32,
    pub otp: u8,
    pub angle: i16,
    /// (x, y, xflags, yflags) in 0.01 mm, y up.
    pub coords: Vec<(i32, i32, u8, u8)>,
    pub n_text: u16,
    pub text: String,
    pub record_size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadFile {
    pub version: u16,
    pub strings: Vec<ReadString>,
    pub symbols: Vec<ReadSymbol>,
    pub objects: Vec<ReadObject>,
}

/// Index entries of a chain starting at `first`: (block position, entry bytes) for each slot.
fn chain(b: &[u8], first: usize, block_size: usize, entry_size: usize) -> Vec<Vec<u8>> {
    let mut entries = Vec::new();
    let mut block = first;
    let mut guard = 0;
    while block != 0 && block + block_size <= b.len() && guard < 10_000 {
        for slot in 0..256 {
            let o = block + 4 + slot * entry_size;
            entries.push(b[o..o + entry_size].to_vec());
        }
        block = u32_at(b, block) as usize;
        guard += 1;
    }
    entries
}

fn decode(v: i32) -> (i32, u8) {
    (v >> 8, (v & 0xff) as u8)
}

pub fn read(b: &[u8]) -> ReadFile {
    assert_eq!(u16_at(b, 0), 0x0CAD, "OCAD mark");
    let version = u16_at(b, 4);
    let symbol_first = u32_at(b, 8) as usize;
    let object_first = u32_at(b, 12) as usize;
    let string_first = u32_at(b, 32) as usize;

    let strings = chain(b, string_first, STRING_BLOCK_SIZE, 16)
        .into_iter()
        .filter(|e| u32_at(e, 0) != 0)
        .map(|e| {
            let pos = u32_at(&e, 0) as usize;
            let reserved = i32_at(&e, 4);
            let end = b[pos..].iter().position(|&c| c == 0).unwrap() + pos;
            assert!(
                end < pos + reserved as usize,
                "NUL within the reserved length"
            );
            ReadString {
                kind: i32_at(&e, 8),
                reserved,
                text: String::from_utf8(b[pos..end].to_vec()).unwrap(),
            }
        })
        .collect();

    let symbols = chain(b, symbol_first, SYMBOL_BLOCK_SIZE, 4)
        .into_iter()
        .map(|e| u32_at(&e, 0) as usize)
        .filter(|&pos| pos != 0)
        .map(|pos| {
            let size = i32_at(b, pos);
            let n_colors = i16_at(b, pos + 26);
            let colors = (0..n_colors.max(0) as usize)
                .map(|i| i16_at(b, pos + 28 + 2 * i))
                .collect();
            let name: Vec<u16> = (0..64)
                .map(|i| u16_at(b, pos + 56 + 2 * i))
                .take_while(|&c| c != 0)
                .collect();
            ReadSymbol {
                pos,
                size,
                number: i32_at(b, pos + 4),
                otp: b[pos + 8],
                extent: i32_at(b, pos + 16),
                colors,
                name: String::from_utf16(&name).unwrap(),
                bytes: b[pos..pos + size as usize].to_vec(),
            }
        })
        .collect();

    let objects = chain(b, object_first, OBJECT_BLOCK_SIZE, 40)
        .into_iter()
        .filter(|e| u32_at(e, 16) != 0)
        .map(|e| {
            let pos = i32_at(&e, 16) as usize;
            let n_item = u32_at(b, pos + 44) as usize;
            let n_text = u16_at(b, pos + 48);
            let coords = (0..n_item)
                .map(|i| {
                    let (x, xf) = decode(i32_at(b, pos + 56 + 8 * i));
                    let (y, yf) = decode(i32_at(b, pos + 60 + 8 * i));
                    (x, y, xf, yf)
                })
                .collect();
            let text_start = pos + 56 + 8 * n_item;
            let units: Vec<u16> = (0..n_text as usize * 4)
                .map(|i| u16_at(b, text_start + 2 * i))
                .take_while(|&c| c != 0)
                .collect();
            ReadObject {
                bbox: [
                    decode(i32_at(&e, 0)).0,
                    decode(i32_at(&e, 4)).0,
                    decode(i32_at(&e, 8)).0,
                    decode(i32_at(&e, 12)).0,
                ],
                len: i32_at(&e, 20),
                index_symbol: i32_at(&e, 24),
                index_otp: e[28],
                status: e[30],
                color: i16_at(&e, 32),
                symbol: i32_at(b, pos),
                otp: b[pos + 4],
                angle: i16_at(b, pos + 6),
                coords,
                n_text,
                text: String::from_utf16(&units).unwrap(),
                record_size: 56 + 8 * (n_item + n_text as usize),
            }
        })
        .collect();

    ReadFile {
        version,
        strings,
        symbols,
        objects,
    }
}
