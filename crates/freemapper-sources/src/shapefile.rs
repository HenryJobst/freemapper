//! The polygons of an ESRI Shapefile: the shapes of the `.shp` with the text fields of the
//! `.dbf`, cut to a box. Read are the polygon types (with or without heights and measures, of
//! which only x and y are kept); the index file `.shx` is not needed.

use std::collections::BTreeMap;

use crate::crs::BoundingBox;

/// A closed ring of `[x, y]`.
pub type Ring = Vec<[f64; 2]>;
/// A polygon: its outer ring first, then its holes.
pub type Polygon = Vec<Ring>;

/// One record: the DBF fields by their name as written (values trimmed, empty ones left out)
/// and its polygons, each a list of closed rings of `[x, y]`, the outer ring first.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeRecord {
    pub fields: BTreeMap<String, String>,
    pub polygons: Vec<Polygon>,
}

/// The polygon records whose box (in the record header) meets `bbox`. The i-th shape belongs
/// to the i-th row of the table; a row flagged deleted is left out with its shape, and so is a
/// null shape.
pub fn polygons_in(shp: &[u8], dbf: &[u8], bbox: &BoundingBox) -> Result<Vec<ShapeRecord>, String> {
    let table = Table::open(dbf)?;
    if shp.len() < FILE_HEADER || be_i32(shp, 0) != Some(FILE_CODE) {
        return Err("not a Shapefile (no file code 9994)".to_string());
    }
    polygon_type(le_i32(shp, 32).unwrap_or(-1))?;
    let mut records = Vec::new();
    let (mut at, mut index) = (FILE_HEADER, 0_usize);
    while at < shp.len() {
        let number = index + 1;
        let cut = || format!("record {number}: the file ends inside it");
        // The length counts 16-bit words of the content, after the 8 bytes of the header.
        let words = be_i32(shp, at + 4).ok_or_else(cut)?;
        let length = usize::try_from(words)
            .ok()
            .and_then(|w| w.checked_mul(2))
            .ok_or_else(|| format!("record {number}: a length of {words} words"))?;
        let content = shp
            .get(at + RECORD_HEADER..)
            .and_then(|rest| rest.get(..length))
            .ok_or_else(cut)?;
        if index >= table.rows {
            return Err(format!(
                "the table has {} rows for more shapes than that",
                table.rows
            ));
        }
        if let Some(polygons) = shape(content, bbox).map_err(|e| format!("record {number}: {e}"))?
            && !table.deleted(index)
        {
            records.push(ShapeRecord {
                fields: table.fields(index),
                polygons,
            });
        }
        at += RECORD_HEADER + length;
        index += 1;
    }
    Ok(records)
}

/// The file code at the start of a `.shp`.
const FILE_CODE: i32 = 9994;
/// The bytes of the file header and of a record header.
const FILE_HEADER: usize = 100;
const RECORD_HEADER: usize = 8;
/// The bytes of a polygon record before its part indices: type, box, part and point count.
const POLYGON_HEADER: usize = 44;

fn be_i32(bytes: &[u8], at: usize) -> Option<i32> {
    let four = bytes.get(at..at.checked_add(4)?)?;
    Some(i32::from_be_bytes(four.try_into().ok()?))
}

fn le_i32(bytes: &[u8], at: usize) -> Option<i32> {
    let four = bytes.get(at..at.checked_add(4)?)?;
    Some(i32::from_le_bytes(four.try_into().ok()?))
}

fn le_f64(bytes: &[u8], at: usize) -> Option<f64> {
    let eight = bytes.get(at..at.checked_add(8)?)?;
    Some(f64::from_le_bytes(eight.try_into().ok()?))
}

/// Whether shapes of `shape_type` are read: `Ok(false)` for the null shape, an error for a
/// type that is no polygon.
fn polygon_type(shape_type: i32) -> Result<bool, String> {
    match shape_type {
        0 => Ok(false),
        // Polygon, PolygonZ, PolygonM: the x and y of all three start the same way.
        5 | 15 | 25 => Ok(true),
        other => Err(format!("shape type {other} is not a polygon")),
    }
}

/// The polygons of one record content; `None` for a null shape and for one whose box does not
/// meet `bbox` (its points are not read then).
fn shape(content: &[u8], bbox: &BoundingBox) -> Result<Option<Vec<Polygon>>, String> {
    let cut = || "its content is shorter than its counts say".to_string();
    if !polygon_type(le_i32(content, 0).ok_or_else(cut)?)? {
        return Ok(None);
    }
    let corner = |i: usize| le_f64(content, 4 + 8 * i).ok_or_else(cut);
    let (min_x, min_y, max_x, max_y) = (corner(0)?, corner(1)?, corner(2)?, corner(3)?);
    if max_x < bbox.min_e || min_x > bbox.max_e || max_y < bbox.min_n || min_y > bbox.max_n {
        return Ok(None);
    }
    let count = |at: usize| {
        le_i32(content, at)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(cut)
    };
    let (parts, points) = (count(36)?, count(40)?);
    let first_point = parts
        .checked_mul(4)
        .and_then(|n| n.checked_add(POLYGON_HEADER))
        .ok_or_else(cut)?;
    let end = points
        .checked_mul(16)
        .and_then(|n| n.checked_add(first_point))
        .ok_or_else(cut)?;
    if end > content.len() {
        return Err(cut());
    }
    let mut starts = Vec::with_capacity(parts + 1);
    for part in 0..parts {
        let start = count(POLYGON_HEADER + 4 * part)?;
        if start > points || starts.last().is_some_and(|last| *last > start) {
            return Err(format!("part {part} starts at point {start} of {points}"));
        }
        starts.push(start);
    }
    starts.push(points);
    let mut rings = Vec::with_capacity(parts);
    for pair in starts.windows(2) {
        let mut ring = Vec::with_capacity(pair[1] - pair[0] + 1);
        for point in pair[0]..pair[1] {
            let at = first_point + 16 * point;
            ring.push([
                le_f64(content, at).ok_or_else(cut)?,
                le_f64(content, at + 8).ok_or_else(cut)?,
            ]);
        }
        if ring.first() != ring.last() {
            ring.push(ring[0]);
        }
        if ring.len() >= 4 {
            rings.push(ring);
        }
    }
    Ok(Some(polygons_of(rings)))
}

/// Twice the signed area of a closed ring: negative for a clockwise ring.
fn doubled_area(ring: &[[f64; 2]]) -> f64 {
    ring.windows(2)
        .map(|pair| pair[0][0] * pair[1][1] - pair[1][0] * pair[0][1])
        .sum()
}

/// Whether `point` lies inside the closed `ring` (even-odd).
fn holds(ring: &[[f64; 2]], point: [f64; 2]) -> bool {
    let mut inside = false;
    for pair in ring.windows(2) {
        let ([x1, y1], [x2, y2]) = (pair[0], pair[1]);
        if (y1 > point[1]) != (y2 > point[1])
            && point[0] < (x2 - x1) * (point[1] - y1) / (y2 - y1) + x1
        {
            inside = !inside;
        }
    }
    inside
}

/// The rings of a record as polygons: a clockwise ring is an outer ring; a counter-clockwise
/// ring is a hole of the smallest outer ring that holds its first point, and an outer ring
/// itself when none does.
fn polygons_of(rings: Vec<Ring>) -> Vec<Polygon> {
    let (outer, holes): (Vec<_>, Vec<_>) = rings
        .into_iter()
        .partition(|ring| doubled_area(ring) <= 0.0);
    let mut polygons: Vec<Polygon> = outer.into_iter().map(|ring| vec![ring]).collect();
    let outer_count = polygons.len();
    for hole in holes {
        let home = (0..outer_count)
            .filter(|&i| holds(&polygons[i][0], hole[0]))
            .min_by(|&a, &b| {
                doubled_area(&polygons[a][0])
                    .abs()
                    .total_cmp(&doubled_area(&polygons[b][0]).abs())
            });
        match home {
            Some(i) => polygons[i].push(hole),
            None => polygons.push(vec![hole]),
        }
    }
    polygons
}

/// The table of a `.dbf`: where its rows are and what its columns are called.
struct Table<'a> {
    bytes: &'a [u8],
    rows: usize,
    first_row: usize,
    row_length: usize,
    /// Name, offset in the row, length.
    columns: Vec<(String, usize, usize)>,
}

impl<'a> Table<'a> {
    fn open(bytes: &'a [u8]) -> Result<Self, String> {
        let short = || "not a dBASE table (too short)".to_string();
        let u16_at = |at: usize| {
            bytes
                .get(at..at + 2)
                .map(|two| usize::from(u16::from_le_bytes([two[0], two[1]])))
                .ok_or_else(short)
        };
        let rows = bytes
            .get(4..8)
            .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
            .ok_or_else(short)?;
        let rows = usize::try_from(rows).map_err(|_| short())?;
        let (first_row, row_length) = (u16_at(8)?, u16_at(10)?);
        if first_row < 33 || row_length == 0 || first_row > bytes.len() {
            return Err("not a dBASE table (its header does not fit)".to_string());
        }
        let needed = rows
            .checked_mul(row_length)
            .and_then(|n| n.checked_add(first_row));
        if needed.is_none_or(|n| n > bytes.len()) {
            return Err(format!(
                "the table says {rows} rows of {row_length} bytes and is shorter than that"
            ));
        }
        let mut columns = Vec::new();
        // The first byte of a row is its deletion flag.
        let mut offset = 1;
        for descriptor in bytes[32..first_row].as_chunks::<32>().0 {
            if descriptor[0] == 0x0D {
                break;
            }
            let name: Vec<u8> = descriptor[..11]
                .iter()
                .copied()
                .take_while(|b| *b != 0)
                .collect();
            let length = usize::from(descriptor[16]);
            columns.push((
                String::from_utf8_lossy(&name).trim().to_string(),
                offset,
                length,
            ));
            offset += length;
        }
        if offset > row_length {
            return Err("the columns of the table are longer than its rows".to_string());
        }
        Ok(Self {
            bytes,
            rows,
            first_row,
            row_length,
            columns,
        })
    }

    fn row(&self, index: usize) -> &[u8] {
        let start = self.first_row + index * self.row_length;
        &self.bytes[start..start + self.row_length]
    }

    fn deleted(&self, index: usize) -> bool {
        self.row(index)[0] == b'*'
    }

    fn fields(&self, index: usize) -> BTreeMap<String, String> {
        let row = self.row(index);
        self.columns
            .iter()
            .filter_map(|(name, offset, length)| {
                let text = decode(&row[*offset..offset + length]);
                let text = text.trim_matches(|c: char| c == ' ' || c == '\0');
                (!text.is_empty()).then(|| (name.clone(), text.to_string()))
            })
            .collect()
    }
}

/// The text of a field: UTF-8 when it is, else Latin-1, the code page of the older tables that
/// come without a `.cpg` (NRW's water protection areas). One byte is one character there, so
/// an umlaut survives that UTF-8 would turn into a replacement character.
fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => bytes.iter().map(|&b| char::from(b)).collect(),
    }
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    use super::Ring;

    /// The fields and the rings of one record of [`shape_files`].
    pub type Entry<'a> = (&'a [(&'a str, &'a str)], &'a [Ring]);

    /// A `.shp` and a `.dbf` with one record per entry: its fields and its rings as given. An
    /// entry without rings is a null shape. The columns are those of all entries, in the order
    /// they first appear, as wide as their longest value.
    pub fn shape_files(records: &[Entry<'_>]) -> (Vec<u8>, Vec<u8>) {
        let mut shp = vec![0_u8; 100];
        shp[..4].copy_from_slice(&9994_i32.to_be_bytes());
        shp[28..32].copy_from_slice(&1000_i32.to_le_bytes());
        shp[32..36].copy_from_slice(&5_i32.to_le_bytes());
        for (number, (_, rings)) in records.iter().enumerate() {
            let mut content = Vec::new();
            if rings.is_empty() {
                content.extend_from_slice(&0_i32.to_le_bytes());
            } else {
                let points: Vec<[f64; 2]> = rings.iter().flatten().copied().collect();
                let fold = |pick: fn(f64, f64) -> f64, axis: usize, start: f64| {
                    points.iter().fold(start, |acc, p| pick(acc, p[axis]))
                };
                content.extend_from_slice(&5_i32.to_le_bytes());
                for corner in [
                    fold(f64::min, 0, f64::INFINITY),
                    fold(f64::min, 1, f64::INFINITY),
                    fold(f64::max, 0, f64::NEG_INFINITY),
                    fold(f64::max, 1, f64::NEG_INFINITY),
                ] {
                    content.extend_from_slice(&corner.to_le_bytes());
                }
                content.extend_from_slice(&(rings.len() as i32).to_le_bytes());
                content.extend_from_slice(&(points.len() as i32).to_le_bytes());
                let mut start = 0_i32;
                for ring in *rings {
                    content.extend_from_slice(&start.to_le_bytes());
                    start += ring.len() as i32;
                }
                for point in &points {
                    content.extend_from_slice(&point[0].to_le_bytes());
                    content.extend_from_slice(&point[1].to_le_bytes());
                }
            }
            shp.extend_from_slice(&(number as i32 + 1).to_be_bytes());
            shp.extend_from_slice(&((content.len() / 2) as i32).to_be_bytes());
            shp.extend_from_slice(&content);
        }
        let words = (shp.len() / 2) as i32;
        shp[24..28].copy_from_slice(&words.to_be_bytes());

        let mut columns: Vec<(&str, usize)> = Vec::new();
        for (fields, _) in records {
            for (name, value) in *fields {
                match columns.iter_mut().find(|(known, _)| known == name) {
                    Some((_, width)) => *width = (*width).max(value.len()),
                    None => columns.push((name, value.len().max(1))),
                }
            }
        }
        let header = 32 + 32 * columns.len() + 1;
        let row = 1 + columns.iter().map(|(_, width)| width).sum::<usize>();
        let mut dbf = vec![0_u8; 32];
        dbf[0] = 3;
        dbf[4..8].copy_from_slice(&(records.len() as u32).to_le_bytes());
        dbf[8..10].copy_from_slice(&(header as u16).to_le_bytes());
        dbf[10..12].copy_from_slice(&(row as u16).to_le_bytes());
        for (name, width) in &columns {
            let mut descriptor = [0_u8; 32];
            descriptor[..name.len()].copy_from_slice(name.as_bytes());
            descriptor[11] = b'C';
            descriptor[16] = *width as u8;
            dbf.extend_from_slice(&descriptor);
        }
        dbf.push(0x0D);
        for (fields, _) in records {
            dbf.push(b' ');
            for (name, width) in &columns {
                let value = fields
                    .iter()
                    .find(|(known, _)| known == name)
                    .map_or("", |(_, value)| *value);
                dbf.extend_from_slice(value.as_bytes());
                dbf.extend(std::iter::repeat_n(b' ', width - value.len()));
            }
        }
        dbf.push(0x1A);
        (shp, dbf)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_latin_1_field_keeps_its_umlauts() {
        assert_eq!(decode(b"D\xfcsterm\xfchle"), "Düstermühle");
        assert_eq!(decode("Düstermühle".as_bytes()), "Düstermühle");
    }

    use super::test_support::shape_files;
    use super::*;

    const ALL: BoundingBox = BoundingBox {
        min_e: -1e9,
        min_n: -1e9,
        max_e: 1e9,
        max_n: 1e9,
    };

    /// A square with the corners (`x`, `y`) and (`x + side`, `y + side`), clockwise.
    fn square(x: f64, y: f64, side: f64) -> Vec<[f64; 2]> {
        vec![
            [x, y],
            [x, y + side],
            [x + side, y + side],
            [x + side, y],
            [x, y],
        ]
    }

    /// The same square counter-clockwise: a hole.
    fn hole(x: f64, y: f64, side: f64) -> Vec<[f64; 2]> {
        let mut ring = square(x, y, side);
        ring.reverse();
        ring
    }

    #[test]
    fn a_polygon_with_a_hole_keeps_its_rings() {
        let rings = [square(0.0, 0.0, 10.0), hole(4.0, 4.0, 2.0)];
        let (shp, dbf) = shape_files(&[(&[("oid", "A"), ("nutzart", "Wald")], &rings)]);
        let records = polygons_in(&shp, &dbf, &ALL).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].polygons.len(), 1);
        assert_eq!(records[0].polygons[0].len(), 2);
        assert_eq!(records[0].polygons[0][0].len(), 5);
        assert_eq!(records[0].polygons[0][0], square(0.0, 0.0, 10.0));
        assert_eq!(records[0].fields["nutzart"], "Wald");
        assert_eq!(records[0].fields["oid"], "A");
    }

    #[test]
    fn two_outer_rings_are_two_polygons() {
        let rings = [square(0.0, 0.0, 10.0), square(20.0, 0.0, 10.0)];
        let (shp, dbf) = shape_files(&[(&[("oid", "A")], &rings)]);
        let records = polygons_in(&shp, &dbf, &ALL).unwrap();
        assert_eq!(records[0].polygons.len(), 2);
        assert_eq!(records[0].polygons[1][0], square(20.0, 0.0, 10.0));
    }

    #[test]
    fn a_hole_belongs_to_the_ring_that_holds_it() {
        let rings = [
            square(0.0, 0.0, 10.0),
            square(20.0, 0.0, 10.0),
            hole(24.0, 4.0, 2.0),
        ];
        let (shp, dbf) = shape_files(&[(&[("oid", "A")], &rings)]);
        let records = polygons_in(&shp, &dbf, &ALL).unwrap();
        assert_eq!(records[0].polygons[0].len(), 1);
        assert_eq!(records[0].polygons[1].len(), 2);
    }

    #[test]
    fn records_outside_the_box_are_left_out() {
        let (a, b) = ([square(0.0, 0.0, 10.0)], [square(100.0, 100.0, 10.0)]);
        let (shp, dbf) = shape_files(&[(&[("oid", "A")], &a), (&[("oid", "B")], &b)]);
        let bbox = BoundingBox {
            min_e: 105.0,
            min_n: 105.0,
            max_e: 120.0,
            max_n: 120.0,
        };
        let records = polygons_in(&shp, &dbf, &bbox).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].fields["oid"], "B");
    }

    #[test]
    fn umlauts_and_padding_are_read() {
        let rings = [square(0.0, 0.0, 10.0)];
        let first: &[(&str, &str)] = &[
            ("nutzart", "Sport-, Freizeit- und Erholungsfläche"),
            ("name", ""),
        ];
        // The second row makes the columns wider than the values of the first.
        let second: &[(&str, &str)] = &[
            (
                "nutzart",
                "Fläche besonderer funktionaler Prägung, die sehr lang ist",
            ),
            ("name", "Stadtpark"),
        ];
        let (shp, dbf) = shape_files(&[(first, &rings), (second, &rings)]);
        let records = polygons_in(&shp, &dbf, &ALL).unwrap();
        assert_eq!(
            records[0].fields["nutzart"],
            "Sport-, Freizeit- und Erholungsfläche"
        );
        assert!(!records[0].fields.contains_key("name"));
        assert_eq!(records[1].fields["name"], "Stadtpark");
    }

    #[test]
    fn a_deleted_row_and_a_null_shape_are_left_out() {
        let rings = [square(0.0, 0.0, 10.0)];
        let none: &[Vec<[f64; 2]>] = &[];
        let (shp, mut dbf) = shape_files(&[
            (&[("oid", "A")], &rings),
            (&[("oid", "B")], none),
            (&[("oid", "C")], &rings),
        ]);
        // Flag the first row deleted: the first byte of the first record.
        let header = usize::from(u16::from_le_bytes([dbf[8], dbf[9]]));
        dbf[header] = b'*';
        let records = polygons_in(&shp, &dbf, &ALL).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].fields["oid"], "C");
    }

    #[test]
    fn a_file_cut_short_is_an_error() {
        let rings = [square(0.0, 0.0, 10.0)];
        let (shp, dbf) = shape_files(&[(&[("oid", "A")], &rings), (&[("oid", "B")], &rings)]);
        let cut = &shp[..shp.len() - 20];
        let error = polygons_in(cut, &dbf, &ALL).unwrap_err();
        assert!(error.contains("record 2"), "{error}");
        // The table is cut short as well: one row for two shapes.
        let (_, one_row) = shape_files(&[(&[("oid", "A")], &rings)]);
        let error = polygons_in(&shp, &one_row, &ALL).unwrap_err();
        assert!(error.contains("1 rows for"), "{error}");
        // Neither file is a Shapefile at all.
        assert!(polygons_in(b"<html>", &dbf, &ALL).is_err());
        assert!(polygons_in(&shp, b"<html>", &ALL).is_err());
    }

    #[test]
    fn a_count_that_reaches_past_the_record_is_an_error() {
        let rings = [square(0.0, 0.0, 10.0)];
        let (mut shp, dbf) = shape_files(&[(&[("oid", "A")], &rings)]);
        // The number of points of the first record (after header, type, box and part count).
        let points = 100 + 8 + 4 + 32 + 4;
        shp[points..points + 4].copy_from_slice(&1_000_000_i32.to_le_bytes());
        let error = polygons_in(&shp, &dbf, &ALL).unwrap_err();
        assert!(error.contains("record 1"), "{error}");
    }

    #[test]
    fn another_shape_type_is_named() {
        let rings = [square(0.0, 0.0, 10.0)];
        let (mut shp, dbf) = shape_files(&[(&[("oid", "A")], &rings)]);
        shp[32..36].copy_from_slice(&3_i32.to_le_bytes());
        shp[108..112].copy_from_slice(&3_i32.to_le_bytes());
        let error = polygons_in(&shp, &dbf, &ALL).unwrap_err();
        assert!(error.contains("shape type 3"), "{error}");
    }

    #[test]
    fn heights_after_the_points_are_passed_over() {
        let rings = [square(0.0, 0.0, 10.0)];
        let (shp, dbf) = shape_files(&[(&[("oid", "A")], &rings)]);
        // The same record as a PolygonZ: the z range and one z per point follow the points.
        let mut z = shp.clone();
        z[32..36].copy_from_slice(&15_i32.to_le_bytes());
        z[108..112].copy_from_slice(&15_i32.to_le_bytes());
        z.extend_from_slice(&[0_u8; 16 + 5 * 8]);
        let words = i32::try_from((z.len() - 108) / 2).unwrap();
        z[104..108].copy_from_slice(&words.to_be_bytes());
        let records = polygons_in(&z, &dbf, &ALL).unwrap();
        assert_eq!(records[0].polygons[0][0], square(0.0, 0.0, 10.0));
    }
}
