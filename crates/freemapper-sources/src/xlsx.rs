//! Reading the cells of an xlsx workbook's first sheet (a zip of XML parts), for the metadata
//! lists that come as spreadsheets.

use std::io::Read as _;

use crate::xml::resolve_entity;

/// Largest workbook part inflated (the DGM/DOM sheet is 8 MB).
const MAX_PART_BYTES: u64 = 64 << 20;
/// Most columns per worksheet row (the metadata lists have six or nine).
const MAX_COLUMNS: usize = 64;
/// Most rows per worksheet (the HVBG lists have about 31 000).
const MAX_ROWS: usize = 200_000;
/// Most cells per worksheet, the empty ones left of a cell included (the lists have 300 000).
const MAX_CELLS: usize = 4_000_000;

/// An Excel serial date (days since 1899-12-30) as `YYYY-MM-DD`.
pub fn excel_date(serial: &str) -> Option<String> {
    let days = serial
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|d| (1.0..2_958_466.0).contains(d))?;
    let date = chrono::NaiveDate::from_ymd_opt(1899, 12, 30)?
        .checked_add_days(chrono::Days::new(days.floor() as u64))?;
    Some(date.format("%Y-%m-%d").to_string())
}

/// The cells of the first worksheet of an xlsx workbook, row by row; a cell's column comes from
/// its reference (`B2`), so skipped cells are empty strings. A sheet of more than 200 000 rows,
/// 64 columns or 4 million cells is refused.
pub fn xlsx_rows(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("not an xlsx workbook: {e}"))?;
    let shared = match workbook_part(&mut archive, "xl/sharedStrings.xml")? {
        Some(xml) => shared_strings(&xml)?,
        None => Vec::new(),
    };
    let sheet = first_sheet(&mut archive)?;
    let xml = workbook_part(&mut archive, &sheet)?
        .ok_or_else(|| format!("the workbook has no {sheet}"))?;
    sheet_rows(&xml, &shared)
}

type Workbook<'a> = zip::ZipArchive<std::io::Cursor<&'a [u8]>>;

/// The text of the workbook part `name`, `None` when there is none.
fn workbook_part(archive: &mut Workbook, name: &str) -> Result<Option<String>, String> {
    let Ok(part) = archive.by_name(name) else {
        return Ok(None);
    };
    let mut text = String::new();
    part.take(MAX_PART_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|e| format!("{name}: {e}"))?;
    if text.len() as u64 > MAX_PART_BYTES {
        return Err(format!("{name}: too large"));
    }
    Ok(Some(text))
}

/// The value of the attribute `name` (local name, any prefix) of an XML start tag.
fn attribute(e: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    e.attributes().flatten().find_map(|a| {
        let key: &str = a.key.as_ref();
        (key.rsplit(':').next() == Some(name)).then(|| a.value.to_string())
    })
}

fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or_default()
}

/// The part of the first sheet in workbook order (`xl/workbook.xml` and its relationships);
/// `xl/worksheets/sheet1.xml` when the workbook does not tell.
fn first_sheet(archive: &mut Workbook) -> Result<String, String> {
    use quick_xml::events::Event;
    const FALLBACK: &str = "xl/worksheets/sheet1.xml";
    let (Some(workbook), Some(rels)) = (
        workbook_part(archive, "xl/workbook.xml")?,
        workbook_part(archive, "xl/_rels/workbook.xml.rels")?,
    ) else {
        return Ok(FALLBACK.to_string());
    };
    let first_start =
        |xml: &str, element: &str, matches: &dyn Fn(&quick_xml::events::BytesStart) -> bool| {
            let mut reader = quick_xml::Reader::from_str(xml);
            loop {
                match reader.read_event() {
                    Ok(Event::Start(e) | Event::Empty(e))
                        if local_name(e.name().as_ref()) == element && matches(&e) =>
                    {
                        return Some(e.into_owned());
                    }
                    Ok(Event::Eof) | Err(_) => return None,
                    _ => {}
                }
            }
        };
    let Some(id) = first_start(&workbook, "sheet", &|_| true).and_then(|e| attribute(&e, "id"))
    else {
        return Ok(FALLBACK.to_string());
    };
    let target = first_start(&rels, "Relationship", &|e| {
        attribute(e, "Id").as_deref() == Some(id.as_str())
    })
    .and_then(|e| attribute(&e, "Target"));
    Ok(match target {
        Some(t) if !t.contains("..") => match t.strip_prefix('/') {
            Some(absolute) => absolute.to_string(),
            None => format!("xl/{t}"),
        },
        _ => FALLBACK.to_string(),
    })
}

/// The shared strings (`<si>`), rich text runs joined, phonetic runs left out.
fn shared_strings(xml: &str) -> Result<Vec<String>, String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let (mut strings, mut text) = (Vec::new(), String::new());
    let (mut in_t, mut in_phonetic) = (false, false);
    loop {
        match reader
            .read_event()
            .map_err(|e| format!("sharedStrings.xml: {e}"))?
        {
            Event::Start(e) => match local_name(e.name().as_ref()) {
                "si" => text.clear(),
                "t" => in_t = true,
                "rPh" => in_phonetic = true,
                _ => {}
            },
            Event::End(e) => match local_name(e.name().as_ref()) {
                "si" => strings.push(std::mem::take(&mut text)),
                "t" => in_t = false,
                "rPh" => in_phonetic = false,
                _ => {}
            },
            Event::Empty(e) if local_name(e.name().as_ref()) == "si" => strings.push(String::new()),
            Event::Text(t) if in_t && !in_phonetic => text.push_str(t.as_ref()),
            Event::CData(t) if in_t && !in_phonetic => text.push_str(&t.into_inner()),
            Event::GeneralRef(r) if in_t && !in_phonetic => text.extend(resolve_entity(r.as_ref())),
            Event::Eof => return Ok(strings),
            _ => {}
        }
    }
}

/// The zero-based column of a cell reference (`B2` → 1, `AA7` → 26).
fn column_of(reference: &str) -> Option<usize> {
    let letters: Vec<u8> = reference
        .bytes()
        .take_while(u8::is_ascii_alphabetic)
        .collect();
    if letters.is_empty() || letters.len() > 3 {
        return None;
    }
    let number = letters.iter().fold(0usize, |n, &b| {
        n * 26 + usize::from(b.to_ascii_uppercase() - b'A') + 1
    });
    Some(number - 1)
}

/// The rows of a worksheet: shared strings (`t="s"`) resolved, inline strings (`t="inlineStr"`)
/// joined, numbers and formula results as written.
fn sheet_rows(xml: &str, shared: &[String]) -> Result<Vec<Vec<String>>, String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut rows = Vec::new();
    let mut row: Vec<String> = Vec::new();
    // The open cell: its column and type, and the text read so far.
    let mut cell: Option<(usize, Option<String>)> = None;
    let mut text = String::new();
    let (mut in_value, mut in_phonetic) = (false, false);
    let mut cells = 0;
    let mut push = |rows: &mut Vec<Vec<String>>, row: Vec<String>| {
        cells += row.len();
        if rows.len() >= MAX_ROWS {
            return Err(format!("the worksheet has more than {MAX_ROWS} rows"));
        }
        if cells > MAX_CELLS {
            return Err(format!("the worksheet has more than {MAX_CELLS} cells"));
        }
        rows.push(row);
        Ok(())
    };
    loop {
        let event = reader.read_event().map_err(|e| format!("worksheet: {e}"))?;
        match event {
            Event::Start(e) => match local_name(e.name().as_ref()) {
                "row" => row.clear(),
                "c" => {
                    let column = attribute(&e, "r")
                        .and_then(|r| column_of(&r))
                        .unwrap_or(row.len());
                    if column >= MAX_COLUMNS {
                        return Err(format!("a cell beyond column {MAX_COLUMNS}"));
                    }
                    cell = Some((column, attribute(&e, "t")));
                    text.clear();
                }
                "v" | "t" => in_value = true,
                "rPh" => in_phonetic = true,
                _ => {}
            },
            Event::Empty(e) if local_name(e.name().as_ref()) == "row" => {
                push(&mut rows, Vec::new())?;
            }
            Event::End(e) => match local_name(e.name().as_ref()) {
                "row" => push(&mut rows, std::mem::take(&mut row))?,
                "c" => {
                    let Some((column, kind)) = cell.take() else {
                        continue;
                    };
                    let value = match kind.as_deref() {
                        Some("s") => {
                            let index: usize = text
                                .trim()
                                .parse()
                                .map_err(|_| format!("bad shared string index {text}"))?;
                            shared
                                .get(index)
                                .cloned()
                                .ok_or_else(|| format!("no shared string {index}"))?
                        }
                        _ => std::mem::take(&mut text),
                    };
                    if row.len() <= column {
                        row.resize(column + 1, String::new());
                    }
                    row[column] = value;
                }
                "v" | "t" => in_value = false,
                "rPh" => in_phonetic = false,
                _ => {}
            },
            Event::Text(t) if in_value && !in_phonetic => text.push_str(t.as_ref()),
            Event::CData(t) if in_value && !in_phonetic => text.push_str(&t.into_inner()),
            Event::GeneralRef(r) if in_value && !in_phonetic => {
                text.extend(resolve_entity(r.as_ref()));
            }
            Event::Eof => return Ok(rows),
            _ => {}
        }
    }
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    use std::io::Write as _;

    pub fn escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// An xlsx workbook like Excel writes: the first sheet in workbook order (`sheet_rows`, the
    /// `<row>` elements) is `worksheets/sheet2.xml`, a decoy second sheet is `sheet1.xml`;
    /// `shared` holds the `<si>` elements.
    pub fn workbook(sheet_rows: &str, shared: &str) -> Vec<u8> {
        const MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
        const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
        let sheet = |rows: &str| {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="{MAIN}" xmlns:r="{REL}"><dimension ref="A1:F9"/><sheetData>{rows}</sheetData></worksheet>"#
            )
        };
        let parts = [
            (
                "xl/workbook.xml".to_string(),
                format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="{MAIN}" xmlns:r="{REL}"><sheets><sheet name="2026-08-04_Metadaten" sheetId="2" r:id="rId1"/><sheet name="Tabelle1" sheetId="1" r:id="rId2"/></sheets></workbook>"#
                ),
            ),
            (
                "xl/_rels/workbook.xml.rels".to_string(),
                format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId2" Type="{REL}/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId1" Type="{REL}/worksheet" Target="/xl/worksheets/sheet2.xml"/><Relationship Id="rId6" Type="{REL}/sharedStrings" Target="sharedStrings.xml"/></Relationships>"#
                ),
            ),
            (
                "xl/sharedStrings.xml".to_string(),
                format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><sst xmlns="{MAIN}">{shared}</sst>"#
                ),
            ),
            ("xl/worksheets/sheet2.xml".to_string(), sheet(sheet_rows)),
            (
                "xl/worksheets/sheet1.xml".to_string(),
                sheet(r#"<row r="1"><c r="A1" t="inlineStr"><is><t>decoy</t></is></c></row>"#),
            ),
        ];
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, xml) in parts {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(xml.as_bytes()).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::workbook;
    use super::*;

    fn strings(row: &[&str]) -> Vec<String> {
        row.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn xlsx_rows_read_shared_strings_and_serials() {
        let shared = "<si><t>Aktualitä: 5.08.2026</t></si><si><t>Kachelname</t></si>\
            <si><t>Bildflugdatum</t></si><si><t>Gemeinde</t></si><si><t>Kreis</t></si>\
            <si><r><t>Frankfurt </t></r><r><rPr><b/></rPr><t>am Main</t></r><rPh><t>ふりがな</t></rPh></si>\
            <si><t>Groß-Gerau &amp; Co</t></si>";
        let rows = r#"<row r="1"><c r="A1" s="3" t="s"><v>0</v></c><c r="B1" s="3"/></row>
            <row r="2"><c r="A2" t="s"><v>1</v></c><c r="B2" t="s"><v>2</v></c><c r="D2" t="s"><v>3</v></c><c r="E2" t="s"><v>4</v></c></row>
            <row r="3"><c r="A3"><v>4765552</v></c><c r="B3" s="1"><v>44985</v></c><c r="D3" t="s"><v>5</v></c><c r="E3" t="inlineStr"><is><t>Kreisfreie Stadt </t><r><t>Frankfurt am Main</t></r></is></c><c r="F3" t="str"><f>A1</f><v>Darmstadt</v></c></row>
            <row r="5"><c r="C5" t="s"><v>6</v></c></row>"#;
        let read = xlsx_rows(&workbook(rows, shared)).unwrap();
        assert_eq!(
            read,
            [
                strings(&["Aktualitä: 5.08.2026"]),
                strings(&["Kachelname", "Bildflugdatum", "", "Gemeinde", "Kreis"]),
                strings(&[
                    "4765552",
                    "44985",
                    "",
                    "Frankfurt am Main",
                    "Kreisfreie Stadt Frankfurt am Main",
                    "Darmstadt"
                ]),
                strings(&["", "", "Groß-Gerau & Co"]),
            ]
        );
        assert_eq!(excel_date("44985").as_deref(), Some("2023-02-28"));
        assert_eq!(excel_date("45888").as_deref(), Some("2025-08-19"));
        assert_eq!(excel_date("46315.75").as_deref(), Some("2026-10-20"));
        assert_eq!(excel_date("Bildflugdatum"), None);
        assert!(xlsx_rows(b"PK not a workbook").is_err());
    }

    #[test]
    fn oversized_sheets_are_refused() {
        let far = workbook(r#"<row r="1"><c r="BM1"><v>1</v></c></row>"#, "");
        let err = xlsx_rows(&far).unwrap_err();
        assert!(err.contains("column"), "{err}");
        let long = workbook(&"<row/>".repeat(200_001), "");
        let err = xlsx_rows(&long).unwrap_err();
        assert!(err.contains("rows"), "{err}");
        // 64 columns are allowed, but not 70 000 rows of them.
        let wide = workbook(&r#"<row><c r="BL1"><v>1</v></c></row>"#.repeat(70_000), "");
        let err = xlsx_rows(&wide).unwrap_err();
        assert!(err.contains("cells"), "{err}");
    }
}
