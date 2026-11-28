// SPDX-License-Identifier: AGPL-3.0-or-later

//! A GeoPackage (OGC, an SQLite 3 file) read without SQLite: the tables of its schema, their rows
//! and the geometries of its feature tables. Only what a download of a state's land cover needs is
//! read: rowid tables in UTF-8, walked by their b-tree (interior and leaf pages, overflow chains),
//! and the standard geometry blobs with the types point, line string, polygon and their multi
//! forms. Everything is checked against the size of the file; a damaged file is an error, never
//! a panic.

/// The value of a column of a row.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// A ring or a line of `[x, y]`.
pub type Line = Vec<[f64; 2]>;
/// A polygon: its outer ring first, then its holes.
pub type Polygon = Vec<Line>;

/// What a geometry holds. A multi geometry is the list of its parts; heights and measures are
/// dropped.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Points(Vec<[f64; 2]>),
    Lines(Vec<Line>),
    Polygons(Vec<Polygon>),
}

/// A geometry of a feature table.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub srs_id: i32,
    pub shape: Shape,
}

/// The geometry in a GeoPackage geometry blob (header `GP`, optional envelope, then WKB);
/// `None` for an empty geometry.
pub fn geometry(blob: &[u8]) -> Result<Option<Geometry>, String> {
    let bad = || "a damaged geometry".to_string();
    if blob.len() < 8 || &blob[..2] != b"GP" || blob[2] != 0 {
        return Err("not a GeoPackage geometry".to_string());
    }
    let flags = blob[3];
    let little = flags & 1 == 1;
    let srs_id = {
        let raw: [u8; 4] = blob[4..8].try_into().map_err(|_| bad())?;
        if little {
            i32::from_le_bytes(raw)
        } else {
            i32::from_be_bytes(raw)
        }
    };
    // 0 none, 1 xy, 2 xyz, 3 xym, 4 xyzm: 4 doubles at most, 2 of them per dimension pair.
    let envelope = match (flags >> 1) & 7 {
        0 => 0,
        1 => 32,
        2 | 3 => 48,
        4 => 64,
        n => return Err(format!("an envelope of kind {n}")),
    };
    if flags & 0x10 != 0 {
        return Ok(None);
    }
    let mut wkb = Wkb {
        bytes: blob.get(8 + envelope..).ok_or_else(bad)?,
        at: 0,
    };
    let shape = wkb.shape(0)?;
    Ok(Some(Geometry { srs_id, shape }))
}

/// The most nested geometries (a collection in a collection) read.
const MAX_NESTING: usize = 4;

/// A well-known-binary reader over `bytes`.
struct Wkb<'b> {
    bytes: &'b [u8],
    at: usize,
}

impl Wkb<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.bytes.len());
        let end = end.ok_or("a damaged geometry")?;
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn u32(&mut self, little: bool) -> Result<u32, String> {
        let raw: [u8; 4] = self.take(4)?.try_into().map_err(|_| "a damaged geometry")?;
        Ok(if little {
            u32::from_le_bytes(raw)
        } else {
            u32::from_be_bytes(raw)
        })
    }

    /// A count of items of `size` bytes, which the rest of the data must be able to hold.
    fn count(&mut self, little: bool, size: usize) -> Result<usize, String> {
        let n = self.u32(little)? as usize;
        if n.checked_mul(size)
            .is_none_or(|b| b > self.bytes.len() - self.at)
        {
            return Err("a damaged geometry: a count the data cannot hold".to_string());
        }
        Ok(n)
    }

    /// `n` points of `dims` ordinates each; the first two are x and y.
    fn points(&mut self, little: bool, n: usize, dims: usize) -> Result<Line, String> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let mut xy = [0.0; 2];
            for i in 0..dims {
                let raw: [u8; 8] = self.take(8)?.try_into().map_err(|_| "a damaged geometry")?;
                let v = if little {
                    f64::from_le_bytes(raw)
                } else {
                    f64::from_be_bytes(raw)
                };
                if let Some(slot) = xy.get_mut(i) {
                    *slot = v;
                }
            }
            out.push(xy);
        }
        Ok(out)
    }

    /// One geometry: its byte order, its type, its content.
    fn shape(&mut self, nesting: usize) -> Result<Shape, String> {
        let little = match self.take(1)?[0] {
            0 => false,
            1 => true,
            n => return Err(format!("a byte order of {n}")),
        };
        let code = self.u32(little)?;
        // ISO adds 1000 per Z, 2000 per M; EWKB sets high bits for them.
        let (base, dims) = match (code >> 16, code / 1000) {
            (flags, _) if flags & 0xe000 != 0 => (
                code & 0xff,
                2 + usize::from(flags & 0x8000 != 0) + usize::from(flags & 0x4000 != 0),
            ),
            (_, thousands @ 0..=3) => (
                code % 1000,
                2 + match thousands {
                    0 => 0,
                    1 | 2 => 1,
                    _ => 2,
                },
            ),
            _ => return Err(format!("the geometry type {code}")),
        };
        let dims = dims.max(2);
        let size = 8 * dims;
        match base {
            1 => Ok(Shape::Points(self.points(little, 1, dims)?)),
            2 => {
                let n = self.count(little, size)?;
                Ok(Shape::Lines(vec![self.points(little, n, dims)?]))
            }
            3 => Ok(Shape::Polygons(vec![self.rings(little, dims)?])),
            4 => {
                let parts = self.parts(little, nesting)?;
                let mut points = Vec::new();
                for part in parts {
                    match part {
                        Shape::Points(p) => points.extend(p),
                        _ => return Err("a multi point of something else".to_string()),
                    }
                }
                Ok(Shape::Points(points))
            }
            5 => {
                let mut lines = Vec::new();
                for part in self.parts(little, nesting)? {
                    match part {
                        Shape::Lines(l) => lines.extend(l),
                        _ => return Err("a multi line string of something else".to_string()),
                    }
                }
                Ok(Shape::Lines(lines))
            }
            6 => {
                let mut polygons = Vec::new();
                for part in self.parts(little, nesting)? {
                    match part {
                        Shape::Polygons(p) => polygons.extend(p),
                        _ => return Err("a multi polygon of something else".to_string()),
                    }
                }
                Ok(Shape::Polygons(polygons))
            }
            other => Err(format!("the geometry type {other}")),
        }
    }

    /// The rings of a polygon.
    fn rings(&mut self, little: bool, dims: usize) -> Result<Polygon, String> {
        let rings = self.count(little, 4)?;
        let mut out = Vec::with_capacity(rings);
        for _ in 0..rings {
            // A point has at least two ordinates of 8 bytes.
            let n = self.count(little, 16)?;
            out.push(self.points(little, n, dims)?);
        }
        Ok(out)
    }

    /// The geometries of a multi geometry.
    fn parts(&mut self, little: bool, nesting: usize) -> Result<Vec<Shape>, String> {
        if nesting >= MAX_NESTING {
            return Err("geometries nested too deep".to_string());
        }
        let n = self.count(little, 9)?;
        (0..n).map(|_| self.shape(nesting + 1)).collect()
    }
}

/// What a walk over a table calls for each row: its rowid and its record bytes.
type Each<'f> = dyn FnMut(i64, &[u8]) -> Result<(), String> + 'f;

/// What [`parse_columns`] finds in a `CREATE TABLE`.
struct Columns {
    names: Vec<String>,
    rowid_alias: Option<usize>,
    real: Vec<bool>,
}

/// The first 16 bytes of every SQLite 3 file.
const MAGIC: &[u8; 16] = b"SQLite format 3\0";
/// The deepest b-tree read; real ones have fewer than ten levels, a cycle has no end.
const MAX_DEPTH: usize = 40;
/// The largest payload of one row read, overflow pages included.
const MAX_PAYLOAD: usize = 256 << 20;
/// The most columns of a row.
const MAX_COLUMNS: usize = 2048;

/// One entry of `sqlite_master`.
struct SchemaEntry {
    kind: String,
    name: String,
    root: u32,
    sql: String,
}

/// An open GeoPackage over the bytes of its file.
pub struct GeoPackage<'a> {
    bytes: &'a [u8],
    page_size: usize,
    /// The bytes of a page that hold content (the rest is reserved).
    usable: usize,
    pages: usize,
    /// The deepest b-tree level read ([`MAX_DEPTH`]; a test sets less).
    max_depth: usize,
    schema: Vec<SchemaEntry>,
}

/// The columns of a table: their names and the one that stands for the rowid, if any.
struct TableLayout {
    root: u32,
    columns: Vec<String>,
    rowid_alias: Option<usize>,
    /// The columns with REAL affinity: SQLite stores a whole number in them as an integer.
    real: Vec<bool>,
}

impl<'a> GeoPackage<'a> {
    /// The tables of the file in `bytes`.
    pub fn open(bytes: &'a [u8]) -> Result<Self, String> {
        if bytes.len() < 100 || &bytes[..16] != MAGIC {
            return Err("not an SQLite 3 file".to_string());
        }
        let page_size = match u16::from_be_bytes([bytes[16], bytes[17]]) {
            1 => 65536,
            n if (512..=32768).contains(&n) && n.is_power_of_two() => usize::from(n),
            n => return Err(format!("a page size of {n}")),
        };
        let usable = page_size - usize::from(bytes[20]);
        if usable < 480 {
            return Err("pages with too little room".to_string());
        }
        let encoding = u32::from_be_bytes([bytes[56], bytes[57], bytes[58], bytes[59]]);
        if !matches!(encoding, 0 | 1) {
            return Err(format!("text encoding {encoding}, not UTF-8"));
        }
        let pages = bytes.len() / page_size;
        if pages == 0 {
            return Err("the file ends inside its first page".to_string());
        }
        let mut gpkg = Self {
            bytes,
            page_size,
            usable,
            pages,
            max_depth: MAX_DEPTH,
            schema: Vec::new(),
        };
        let mut schema = Vec::new();
        gpkg.walk(1, &mut |_, record| {
            let row = decode_record(record)?;
            if let [
                Value::Text(kind),
                Value::Text(name),
                _,
                Value::Int(root),
                sql,
            ] = row.as_slice()
            {
                schema.push(SchemaEntry {
                    kind: kind.clone(),
                    name: name.clone(),
                    root: u32::try_from(*root).unwrap_or(0),
                    sql: match sql {
                        Value::Text(text) => text.clone(),
                        _ => String::new(),
                    },
                });
            }
            Ok(())
        })?;
        gpkg.schema = schema;
        Ok(gpkg)
    }

    /// The names of the tables, in the order of the schema.
    pub fn table_names(&self) -> Result<Vec<String>, String> {
        Ok(self
            .schema
            .iter()
            .filter(|e| e.kind == "table")
            .map(|e| e.name.clone())
            .collect())
    }

    /// The column names of `table`, in order.
    pub fn columns(&self, table: &str) -> Result<Vec<String>, String> {
        Ok(self.layout(table)?.columns)
    }

    /// Calls `each` with the rowid and the values of every row of `table`, in rowid order. A
    /// column that is the rowid (`INTEGER PRIMARY KEY`) holds it; a row shorter than the table
    /// (a column added later) is filled with `Null`.
    pub fn rows(
        &self,
        table: &str,
        each: &mut dyn FnMut(i64, Vec<Value>) -> Result<(), String>,
    ) -> Result<(), String> {
        let layout = self.layout(table)?;
        self.walk(layout.root, &mut |rowid, record| {
            let mut values = decode_record(record)?;
            values.resize(values.len().max(layout.columns.len()), Value::Null);
            for (value, _) in values
                .iter_mut()
                .zip(&layout.real)
                .filter(|(_, real)| **real)
            {
                if let Value::Int(n) = value {
                    *value = Value::Real(*n as f64);
                }
            }
            if let Some(slot) = layout.rowid_alias.and_then(|i| values.get_mut(i)) {
                *slot = Value::Int(rowid);
            }
            each(rowid, values)
        })
        .map_err(|m| format!("table {table}: {m}"))
    }

    fn layout(&self, table: &str) -> Result<TableLayout, String> {
        let entry = self
            .schema
            .iter()
            .find(|e| e.kind == "table" && e.name.eq_ignore_ascii_case(table))
            .ok_or_else(|| format!("no table {table}"))?;
        let found = parse_columns(&entry.sql).map_err(|m| format!("table {table}: {m}"))?;
        Ok(TableLayout {
            root: entry.root,
            columns: found.names,
            rowid_alias: found.rowid_alias,
            real: found.real,
        })
    }

    /// The bytes of page `number` (counted from 1).
    fn page(&self, number: u32) -> Result<&'a [u8], String> {
        let index = (number as usize)
            .checked_sub(1)
            .filter(|&i| i < self.pages)
            .ok_or_else(|| format!("page {number} of {}", self.pages))?;
        Ok(&self.bytes[index * self.page_size..(index + 1) * self.page_size])
    }

    /// Calls `each` with the rowid and the record bytes of every row of the table b-tree at
    /// `root`, in rowid order.
    fn walk(&self, root: u32, each: &mut Each<'_>) -> Result<(), String> {
        let mut budget = self.pages;
        self.walk_page(root, 0, &mut budget, each)
    }

    fn walk_page(
        &self,
        number: u32,
        depth: usize,
        budget: &mut usize,
        each: &mut Each<'_>,
    ) -> Result<(), String> {
        if depth > self.max_depth || *budget == 0 {
            return Err("a b-tree with a cycle".to_string());
        }
        *budget -= 1;
        let page = self.page(number)?;
        let at = if number == 1 { 100 } else { 0 };
        let bad = || format!("page {number} is damaged");
        let kind = *page.get(at).ok_or_else(bad)?;
        let cells = usize::from(u16_at(page, at + 3).ok_or_else(bad)?);
        match kind {
            0x05 => {
                let right = u32_at(page, at + 8).ok_or_else(bad)?;
                for i in 0..cells {
                    let pointer = usize::from(u16_at(page, at + 12 + 2 * i).ok_or_else(bad)?);
                    let child = u32_at(page, pointer).ok_or_else(bad)?;
                    self.walk_page(child, depth + 1, budget, each)?;
                }
                self.walk_page(right, depth + 1, budget, each)
            }
            0x0d => {
                for i in 0..cells {
                    let pointer = usize::from(u16_at(page, at + 8 + 2 * i).ok_or_else(bad)?);
                    let cell = page.get(pointer..).ok_or_else(bad)?;
                    let (total, n) = varint(cell).ok_or_else(bad)?;
                    let (rowid, m) = varint(&cell[n..]).ok_or_else(bad)?;
                    let total = usize::try_from(total)
                        .ok()
                        .filter(|&t| t <= MAX_PAYLOAD)
                        .ok_or_else(|| format!("a row of {total} bytes"))?;
                    let body = &cell[n + m..];
                    let record = self.payload(body, total)?;
                    each(rowid as i64, &record)?;
                }
                Ok(())
            }
            other => Err(format!(
                "page {number} has the type {other}, not a table page"
            )),
        }
    }

    /// The `total` bytes of a row whose first part is `body`, with its overflow pages.
    fn payload(&self, body: &'a [u8], total: usize) -> Result<std::borrow::Cow<'a, [u8]>, String> {
        let u = self.usable;
        let max_local = u - 35;
        let local = if total <= max_local {
            total
        } else {
            let min_local = (u - 12) * 32 / 255 - 23;
            let k = min_local + (total - min_local) % (u - 4);
            if k <= max_local { k } else { min_local }
        };
        let cut = || "a row is cut off".to_string();
        let first = body.get(..local).ok_or_else(cut)?;
        if local == total {
            return Ok(std::borrow::Cow::Borrowed(first));
        }
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(first);
        let mut next = u32_at(body, local).ok_or_else(cut)?;
        let mut hops = 0;
        while out.len() < total {
            hops += 1;
            if next == 0 || hops > self.pages {
                return Err("an overflow chain that does not end".to_string());
            }
            let page = self.page(next)?;
            next = u32_at(page, 0).ok_or_else(cut)?;
            let want = (total - out.len()).min(u - 4);
            out.extend_from_slice(page.get(4..4 + want).ok_or_else(cut)?);
        }
        Ok(std::borrow::Cow::Owned(out))
    }
}

/// The big-endian `u16` at `at`.
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

/// The big-endian `u32` at `at`.
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// The SQLite varint at the start of `bytes` (1 to 9 bytes, most significant first, the ninth
/// with all 8 bits) and its length.
fn varint(bytes: &[u8]) -> Option<(u64, usize)> {
    let mut value = 0u64;
    for (i, &byte) in bytes.iter().take(9).enumerate() {
        if i == 8 {
            return Some(((value << 8) + u64::from(byte), 9));
        }
        value = (value << 7) + u64::from(byte & 0x7f);
        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }
    }
    None
}

/// The values of a record (header of serial types, then the values).
fn decode_record(record: &[u8]) -> Result<Vec<Value>, String> {
    let bad = || "a damaged row".to_string();
    let (header_len, mut at) = varint(record).ok_or_else(bad)?;
    let header_len = usize::try_from(header_len).map_err(|_| bad())?;
    if header_len < at || header_len > record.len() {
        return Err(bad());
    }
    let mut types = Vec::new();
    while at < header_len {
        let (kind, n) = varint(&record[at..header_len]).ok_or_else(bad)?;
        at += n;
        types.push(kind);
        if types.len() > MAX_COLUMNS {
            return Err("a row with too many columns".to_string());
        }
    }
    let mut body = header_len;
    let mut values = Vec::with_capacity(types.len());
    for kind in types {
        let size = match kind {
            0 | 8 | 9 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 6,
            6 | 7 => 8,
            10 | 11 => return Err("a reserved column type".to_string()),
            n => usize::try_from((n - 12) / 2).map_err(|_| bad())?,
        };
        let raw = record
            .get(body..body.checked_add(size).ok_or_else(bad)?)
            .ok_or_else(bad)?;
        body += size;
        let signed = |raw: &[u8]| {
            let mut v = if raw[0] & 0x80 != 0 { -1i64 } else { 0 };
            for &b in raw {
                v = (v << 8) | i64::from(b);
            }
            v
        };
        values.push(match kind {
            0 => Value::Null,
            8 => Value::Int(0),
            9 => Value::Int(1),
            1..=6 => Value::Int(signed(raw)),
            7 => Value::Real(f64::from_be_bytes(raw.try_into().map_err(|_| bad())?)),
            n if n % 2 == 0 => Value::Blob(raw.to_vec()),
            _ => Value::Text(String::from_utf8(raw.to_vec()).map_err(|_| bad())?),
        });
    }
    Ok(values)
}

/// The column names of a `CREATE TABLE` statement and the index of the column that is the
/// rowid (`INTEGER PRIMARY KEY`), if there is one.
fn parse_columns(sql: &str) -> Result<Columns, String> {
    let open = sql.find('(').ok_or("no column list")?;
    let mut parts = Vec::new();
    let (mut depth, mut quote, mut part) = (0usize, None::<char>, String::new());
    let mut closed = None;
    let mut chars = sql[open + 1..].chars();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                part.push(c);
                if c == q || (q == '[' && c == ']') {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' | '`' | '[' => {
                    quote = Some(c);
                    part.push(c);
                }
                '(' => {
                    depth += 1;
                    part.push(c);
                }
                ')' if depth == 0 => {
                    closed = Some(chars.as_str());
                    break;
                }
                ')' => {
                    depth -= 1;
                    part.push(c);
                }
                ',' if depth == 0 => parts.push(std::mem::take(&mut part)),
                _ => part.push(c),
            },
        }
    }
    let closed = closed.ok_or("the column list does not end")?;
    parts.push(part);
    if closed.to_ascii_uppercase().contains("WITHOUT ROWID") {
        return Err("a WITHOUT ROWID table".to_string());
    }
    let (mut columns, mut alias, mut real) = (Vec::new(), None, Vec::new());
    for def in parts {
        let def = def.trim();
        let upper = def.to_ascii_uppercase();
        if ["PRIMARY", "UNIQUE", "CHECK", "FOREIGN", "CONSTRAINT"]
            .iter()
            .any(|k| upper.starts_with(k))
        {
            continue;
        }
        let (name, rest) = match def.chars().next() {
            Some(q @ ('"' | '\'' | '`')) => {
                let end = def[1..].find(q).ok_or("an unclosed column name")? + 1;
                (def[1..end].to_string(), &def[end + 1..])
            }
            Some('[') => {
                let end = def.find(']').ok_or("an unclosed column name")?;
                (def[1..end].to_string(), &def[end + 1..])
            }
            _ => {
                let end = def.find(char::is_whitespace).unwrap_or(def.len());
                (def[..end].to_string(), &def[end..])
            }
        };
        let rest = rest.to_ascii_uppercase();
        let mut words = rest.split_whitespace();
        if words.next() == Some("INTEGER") && rest.contains("PRIMARY KEY") && alias.is_none() {
            alias = Some(columns.len());
        }
        // Affinity by the rules of SQLite: the declared type names REAL, FLOA or DOUB.
        let declared = rest.split_whitespace().next().unwrap_or("");
        real.push(
            ["REAL", "FLOA", "DOUB"]
                .iter()
                .any(|k| declared.contains(k)),
        );
        columns.push(name);
    }
    Ok(Columns {
        names: columns,
        rowid_alias: alias,
        real,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/gpkg")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    #[test]
    fn the_schema_names_tables_and_columns() {
        let bytes = fixture("small.gpkg");
        let gpkg = GeoPackage::open(&bytes).unwrap();
        let tables = gpkg.table_names().unwrap();
        for name in ["gpkg_geometry_columns", "mark", "markkantlinje", "points"] {
            assert!(tables.iter().any(|t| t == name), "{name} in {tables:?}");
        }
        assert_eq!(
            gpkg.columns("mark").unwrap(),
            ["fid", "geom", "objekttyp", "areal", "antal", "notering"]
        );
        assert_eq!(gpkg.columns("points").unwrap(), ["id", "geom", "namn"]);
        assert!(gpkg.columns("nope").is_err());
    }

    fn all_rows(gpkg: &GeoPackage, table: &str) -> Vec<(i64, Vec<Value>)> {
        let mut rows = Vec::new();
        gpkg.rows(table, &mut |id, values| {
            rows.push((id, values));
            Ok(())
        })
        .unwrap();
        rows
    }

    #[test]
    fn rows_carry_every_kind_of_value_and_the_rowid() {
        let bytes = fixture("small.gpkg");
        let gpkg = GeoPackage::open(&bytes).unwrap();
        let rows = all_rows(&gpkg, "mark");
        assert_eq!(
            rows.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
        let first = &rows[0].1;
        assert_eq!(first[0], Value::Int(1), "the rowid column holds the rowid");
        assert!(matches!(&first[1], Value::Blob(b) if b.starts_with(b"GP")));
        assert_eq!(first[2], Value::Text("Skog".to_string()));
        assert_eq!(first[3], Value::Real(99.5));
        assert_eq!(first[4], Value::Int(7));
        assert_eq!(first[5], Value::Blob(vec![0, 1]));
        assert_eq!(rows[1].1[2], Value::Text("Öppen mark".to_string()));
        assert_eq!(rows[1].1[4], Value::Null);
        assert_eq!(rows[2].1[4], Value::Int(-3));
        assert_eq!(rows[3].1[1], Value::Null);
        assert_eq!(rows[3].1[4], Value::Int(1 << 40));
        assert_eq!(rows[3].1[3], Value::Real(0.0));
        assert!(gpkg.rows("nope", &mut |_, _| Ok(())).is_err());
    }

    #[test]
    fn a_table_of_many_pages_and_a_row_with_an_overflow_chain_are_read() {
        let bytes = fixture("deep.gpkg");
        let gpkg = GeoPackage::open(&bytes).unwrap();
        let rows = all_rows(&gpkg, "ytor");
        assert_eq!(rows.len(), 700);
        assert!(
            rows.iter()
                .enumerate()
                .all(|(i, (id, _))| *id == i as i64 + 1)
        );
        assert_eq!(rows[0].1[2], Value::Text("yta 1".to_string()));
        assert_eq!(rows[349].1[2], Value::Text("x".repeat(5000)));
        assert_eq!(rows[699].1[2], Value::Text("yta 700".to_string()));
    }

    fn geometries(file: &str, table: &str) -> Vec<Option<Geometry>> {
        let bytes = fixture(file);
        let gpkg = GeoPackage::open(&bytes).unwrap();
        all_rows(&gpkg, table)
            .into_iter()
            .filter_map(|(_, row)| match &row[1] {
                Value::Blob(blob) => Some(geometry(blob).unwrap()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn polygons_and_multipolygons_are_read_big_endian() {
        let shapes = geometries("small.gpkg", "mark");
        // Row 4 has no geometry at all, row 3 an empty one.
        assert_eq!(shapes.len(), 3);
        let Some(Geometry {
            srs_id: 3006,
            shape: Shape::Polygons(polygons),
        }) = &shapes[0]
        else {
            panic!("{:?}", shapes[0]);
        };
        assert_eq!(polygons.len(), 1);
        assert_eq!(polygons[0].len(), 2, "outer ring and hole");
        assert_eq!(polygons[0][0][2], [10.0, 10.0]);
        assert_eq!(polygons[0][1][0], [2.0, 2.0]);
        let Some(Geometry {
            shape: Shape::Polygons(parts),
            ..
        }) = &shapes[1]
        else {
            panic!("{:?}", shapes[1]);
        };
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1][0][0], [25.0, 20.0]);
        assert_eq!(shapes[2], None, "empty");
    }

    #[test]
    fn lines_and_a_little_endian_point_with_an_envelope_are_read() {
        let lines = geometries("small.gpkg", "markkantlinje");
        let Some(Geometry {
            shape: Shape::Lines(parts),
            ..
        }) = &lines[0]
        else {
            panic!("{:?}", lines[0]);
        };
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], [[0.0, 0.0], [5.0, 5.0], [9.0, 1.0]]);
        assert_eq!(
            geometries("small.gpkg", "points")[0],
            Some(Geometry {
                srs_id: 3006,
                shape: Shape::Points(vec![[7.5, 7.5]])
            })
        );
    }

    #[test]
    fn a_damaged_geometry_is_an_error() {
        let bytes = fixture("small.gpkg");
        let gpkg = GeoPackage::open(&bytes).unwrap();
        let Value::Blob(blob) = all_rows(&gpkg, "mark").remove(0).1.remove(1) else {
            panic!("no blob");
        };
        assert!(geometry(b"XX").is_err());
        assert!(geometry(&[]).is_err());
        let mut wrong_magic = blob.clone();
        wrong_magic[0] = b'X';
        assert!(geometry(&wrong_magic).is_err());
        // Cut anywhere inside the WKB.
        for len in 8..blob.len() {
            assert!(geometry(&blob[..len]).is_err(), "cut at {len}");
        }
        // A ring that claims a billion points.
        let mut huge = blob.clone();
        huge[8 + 9..8 + 13].copy_from_slice(&1_000_000_000u32.to_be_bytes());
        assert!(geometry(&huge).is_err());
        // An unknown geometry type.
        let mut unknown = blob;
        unknown[8 + 1..8 + 5].copy_from_slice(&99u32.to_be_bytes());
        assert!(geometry(&unknown).is_err());
    }

    /// A geometry blob around `wkb`, little endian, no envelope.
    fn blob(wkb: &[u8]) -> Vec<u8> {
        let mut out = b"GP\0\x01".to_vec();
        out.extend(3006i32.to_le_bytes());
        out.extend(wkb);
        out
    }

    #[test]
    fn heights_and_measures_are_dropped() {
        let point = |code: u32, ordinates: &[f64]| {
            let mut wkb = vec![1];
            wkb.extend(code.to_le_bytes());
            ordinates.iter().for_each(|v| wkb.extend(v.to_le_bytes()));
            blob(&wkb)
        };
        let xy = Some(Geometry {
            srs_id: 3006,
            shape: Shape::Points(vec![[1.0, 2.0]]),
        });
        // ISO: Z +1000, M +2000, ZM +3000. EWKB: flag bits for Z and M.
        for (code, ordinates) in [
            (1, &[1.0, 2.0][..]),
            (1001, &[1.0, 2.0, 99.0]),
            (2001, &[1.0, 2.0, 99.0]),
            (3001, &[1.0, 2.0, 99.0, 98.0]),
            (0x8000_0001, &[1.0, 2.0, 99.0]),
            (0x4000_0001, &[1.0, 2.0, 99.0]),
            (0xc000_0001, &[1.0, 2.0, 99.0, 98.0]),
        ] {
            assert_eq!(
                geometry(&point(code, ordinates)).unwrap(),
                xy,
                "type {code}"
            );
        }
    }

    #[test]
    fn a_damaged_file_is_an_error_and_never_a_panic() {
        let bytes = fixture("small.gpkg");
        let read = |bytes: &[u8]| -> Result<(), String> {
            let gpkg = GeoPackage::open(bytes)?;
            for table in gpkg.table_names()? {
                gpkg.rows(&table, &mut |_, values| {
                    for value in values {
                        if let Value::Blob(blob) = value {
                            let _ = geometry(&blob);
                        }
                    }
                    Ok(())
                })
                .ok();
            }
            Ok(())
        };
        // Cut at every length.
        for len in (0..bytes.len()).step_by(7) {
            let _ = read(&bytes[..len]);
        }
        // Every byte of the first pages replaced by each of a few values.
        for at in 0..1536 {
            for value in [0u8, 0xff, 0x80, 0x7f] {
                let mut damaged = bytes.clone();
                damaged[at] = value;
                let _ = read(&damaged);
            }
        }
        // A page that points at itself.
        let mut cycle = fixture("deep.gpkg");
        let page = 512 * 2;
        if cycle[page] == 0x05 {
            let at = page + 8;
            cycle[at..at + 4].copy_from_slice(&3u32.to_be_bytes());
        }
        let _ = read(&cycle);
    }

    #[test]
    fn quoted_column_names_and_nested_brackets_are_read() {
        let sql = concat!(
            "CREATE TABLE t (\"a b\" INTEGER, 'c' TEXT, `d` BLOB, [e f] REAL, ",
            "g NUMERIC(10,2), h TEXT DEFAULT (1+(2)), PRIMARY KEY (g, h))"
        );
        let columns = parse_columns(sql).unwrap();
        assert_eq!(columns.names, ["a b", "c", "d", "e f", "g", "h"]);
        assert!(parse_columns("CREATE TABLE t (a INTEGER, b TEXT) WITHOUT ROWID").is_err());
        // A quote keeps a comma and a bracket inside the name; the type after a quoted name counts.
        let columns = parse_columns(
            "CREATE TABLE t (\"i,d\" INTEGER PRIMARY KEY, [x,y] FLOAT, 'z' TEXT, w REAL, v INTEGER PRIMARY KEY, u TEXT PRIMARY KEY)",
        )
        .unwrap();
        assert_eq!(columns.names, ["i,d", "x,y", "z", "w", "v", "u"]);
        assert_eq!(
            columns.rowid_alias,
            Some(0),
            "the first INTEGER PRIMARY KEY"
        );
        assert_eq!(columns.real, [false, true, false, true, false, false]);
        // A column that is not an INTEGER PRIMARY KEY is no alias, whatever else it is.
        let columns =
            parse_columns("CREATE TABLE t (a TEXT PRIMARY KEY, b INTEGER, c REAL)").unwrap();
        assert_eq!(columns.rowid_alias, None);
        let columns = parse_columns("CREATE TABLE t (`q` INTEGER PRIMARY KEY)").unwrap();
        assert_eq!(
            (columns.names, columns.rowid_alias),
            (vec!["q".to_string()], Some(0))
        );
        assert!(parse_columns("CREATE TABLE t (a INTEGER, b TEXT").is_err());
    }

    /// A geometry of `code` (little endian) holding `parts`, each already encoded.
    fn multi(code: u32, parts: &[Vec<u8>]) -> Vec<u8> {
        let mut wkb = vec![1];
        wkb.extend(code.to_le_bytes());
        wkb.extend((parts.len() as u32).to_le_bytes());
        parts.iter().for_each(|part| wkb.extend(part));
        blob(&wkb)
    }

    fn point(x: f64, y: f64, rest: &[f64]) -> Vec<u8> {
        let mut wkb = vec![1];
        wkb.extend(1u32.to_le_bytes());
        wkb.extend(x.to_le_bytes());
        wkb.extend(y.to_le_bytes());
        rest.iter().for_each(|v| wkb.extend(v.to_le_bytes()));
        wkb
    }

    #[test]
    fn a_line_keeps_its_xy_whatever_heights_and_measures_it_carries() {
        // Two points of `ordinates` each; the second only reads right when the first took its
        // exact size.
        for (code, ordinates) in [
            (2u32, 2usize),
            (1002, 3),
            (2002, 3),
            (3002, 4),
            (0x8000_0002, 3),
            (0x4000_0002, 3),
            (0xc000_0002, 4),
        ] {
            let mut wkb = vec![1];
            wkb.extend(code.to_le_bytes());
            wkb.extend(2u32.to_le_bytes());
            for point in 0..2 {
                for ordinate in 0..ordinates {
                    let value = if ordinate < 2 {
                        f64::from(point * 10 + ordinate as u32 + 1)
                    } else {
                        99.0
                    };
                    wkb.extend(value.to_le_bytes());
                }
            }
            assert_eq!(
                geometry(&blob(&wkb)).unwrap().map(|g| g.shape),
                Some(Shape::Lines(vec![vec![[1.0, 2.0], [11.0, 12.0]]])),
                "type {code}"
            );
        }
    }

    #[test]
    fn a_multi_point_is_the_points_it_holds() {
        let found = geometry(&multi(4, &[point(1.0, 2.0, &[]), point(3.0, 4.0, &[])])).unwrap();
        assert_eq!(
            found.map(|g| g.shape),
            Some(Shape::Points(vec![[1.0, 2.0], [3.0, 4.0]]))
        );
        // A multi point of something else is refused.
        let mut line = vec![1];
        line.extend(2u32.to_le_bytes());
        line.extend(0u32.to_le_bytes());
        assert!(geometry(&multi(4, &[line])).is_err());
    }

    #[test]
    fn a_varint_of_nine_bytes_keeps_all_bits_of_the_last_byte() {
        let mut bytes = vec![0xff; 8];
        bytes.push(0xab);
        let expected = (((1u64 << 56) - 1) << 8) | 0xab;
        assert_eq!(varint(&bytes), Some((expected, 9)));
        assert_eq!(varint(&[0x81, 0x00]), Some((128, 2)));
        assert_eq!(varint(&[0x7f]), Some((127, 1)));
    }

    /// Points every child of the interior page `page` (512-byte pages) at page `target`.
    fn point_children_at(bytes: &mut [u8], page: usize, target: u32) {
        let at = (page - 1) * 512;
        let cells = usize::from(u16::from_be_bytes([bytes[at + 3], bytes[at + 4]]));
        bytes[at + 8..at + 12].copy_from_slice(&target.to_be_bytes());
        for i in 0..cells {
            let pointer = at
                + usize::from(u16::from_be_bytes([
                    bytes[at + 12 + 2 * i],
                    bytes[at + 13 + 2 * i],
                ]));
            bytes[pointer..pointer + 4].copy_from_slice(&target.to_be_bytes());
        }
    }

    #[test]
    fn a_tree_that_reaches_the_same_page_again_and_again_is_refused() {
        // The root (page 72) and its child (page 73) point everywhere at one page, so the walk
        // visits thousands of pages in a file of 191: no cycle, but more visits than pages.
        let mut bytes = fixture("deep.gpkg");
        point_children_at(&mut bytes, 72, 73);
        point_children_at(&mut bytes, 73, 4);
        let gpkg = GeoPackage::open(&bytes).unwrap();
        let error = gpkg.rows("ytor", &mut |_, _| Ok(())).unwrap_err();
        assert!(error.contains("cycle"), "{error}");
    }

    #[test]
    fn the_walk_reads_down_to_the_depth_it_allows_and_no_further() {
        // The tree of `ytor`: the root (page 72), interior pages (73, 147) and the leaves.
        let bytes = fixture("deep.gpkg");
        let mut gpkg = GeoPackage::open(&bytes).unwrap();
        gpkg.max_depth = 2;
        assert_eq!(all_rows(&gpkg, "ytor").len(), 700);
        gpkg.max_depth = 1;
        let error = gpkg.rows("ytor", &mut |_, _| Ok(())).unwrap_err();
        assert!(error.contains("cycle"), "{error}");
    }

    #[test]
    fn multi_polygons_nested_beyond_the_limit_are_refused() {
        // A multi polygon of one multi polygon of one ..., ending in an empty one.
        let nested = |levels: usize| {
            let mut wkb = Vec::new();
            for _ in 0..levels {
                wkb.push(1);
                wkb.extend(6u32.to_le_bytes());
                wkb.extend(1u32.to_le_bytes());
            }
            wkb.push(1);
            wkb.extend(6u32.to_le_bytes());
            wkb.extend(0u32.to_le_bytes());
            blob(&wkb)
        };
        assert!(geometry(&nested(2)).is_ok());
        let error = geometry(&nested(10)).unwrap_err();
        assert!(error.contains("nested"), "{error}");
    }

    #[test]
    fn an_overflow_chain_that_does_not_end_is_refused() {
        // Page 5 points at itself: a row of 4 611 bytes wants nine pages of chain.
        let mut bytes = fixture("deep.gpkg");
        bytes[4 * 512..4 * 512 + 4].copy_from_slice(&5u32.to_be_bytes());
        let mut gpkg = GeoPackage::open(&bytes).unwrap();
        gpkg.pages = 6;
        let mut body = vec![0u8; 39];
        body.extend(5u32.to_be_bytes());
        let error = gpkg.payload(&body, 39 + 508 * 9).unwrap_err();
        assert!(error.contains("does not end"), "{error}");
    }

    #[test]
    fn a_ring_that_claims_more_points_than_its_bytes_hold_is_refused_whole() {
        // A polygon of one ring of two points (32 bytes) with only 16 bytes after the count.
        let mut wkb = vec![1];
        wkb.extend(3u32.to_le_bytes());
        wkb.extend(1u32.to_le_bytes());
        wkb.extend(2u32.to_le_bytes());
        wkb.extend([0u8; 16]);
        assert_eq!(
            geometry(&blob(&wkb)),
            Err("a damaged geometry: a count the data cannot hold".to_string())
        );
    }

    /// `small.gpkg` with the bytes at `at` replaced.
    fn patched(at: usize, with: &[u8]) -> Vec<u8> {
        let mut bytes = fixture("small.gpkg");
        bytes[at..at + with.len()].copy_from_slice(with);
        bytes
    }

    fn open_error(bytes: &[u8]) -> String {
        match GeoPackage::open(bytes) {
            Ok(_) => "opened".to_string(),
            Err(error) => error,
        }
    }

    #[test]
    fn the_page_size_must_be_a_power_of_two_between_512_and_65536() {
        for size in [512u16, 1024, 4096, 32768] {
            let page = &size.to_be_bytes();
            // Only the header is read before the first page: a size that is not the real one
            // may still fail later, but never with a complaint about the size.
            let error = open_error(&patched(16, page));
            assert!(!error.contains("page size"), "{size}: {error}");
        }
        for size in [0u16, 2, 256, 511, 513, 1000, 4097, 40000] {
            let error = open_error(&patched(16, &size.to_be_bytes()));
            assert_eq!(error, format!("a page size of {size}"));
        }
        // 1 stands for 65536.
        assert!(!open_error(&patched(16, &1u16.to_be_bytes())).contains("page size"));
    }

    #[test]
    fn a_page_needs_480_usable_bytes() {
        // Pages of 512 bytes: 32 reserved leave exactly 480.
        let enough = open_error(&patched(20, &[32]));
        assert!(!enough.contains("too little room"), "{enough}");
        assert_eq!(
            open_error(&patched(20, &[33])),
            "pages with too little room"
        );
        assert_eq!(
            open_error(&patched(20, &[200])),
            "pages with too little room"
        );
    }

    #[test]
    fn a_header_alone_and_a_foreign_text_encoding_are_refused() {
        let mut header = vec![0u8; 100];
        header[..16].copy_from_slice(MAGIC);
        header[16..18].copy_from_slice(&512u16.to_be_bytes());
        assert_eq!(open_error(&header), "the file ends inside its first page");
        assert_eq!(open_error(&header[..99]), "not an SQLite 3 file");
        assert_eq!(
            open_error(&patched(56, &2u32.to_be_bytes())),
            "text encoding 2, not UTF-8"
        );
        assert!(!open_error(&patched(56, &1u32.to_be_bytes())).contains("encoding"));
    }

    #[test]
    fn what_is_not_sqlite_is_refused() {
        assert!(GeoPackage::open(b"PK\x03\x04 a zip").is_err());
        assert!(GeoPackage::open(&[]).is_err());
    }
}
