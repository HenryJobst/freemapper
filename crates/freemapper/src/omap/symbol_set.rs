//! Symbol sets: colours and symbols copied verbatim from OOM symbol set files.

use super::OmapError;
use super::enlarge::enlarge_symbols;

/// Symbol number of a norm, e.g. "521" or "501.5".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolCode(pub String);

/// Colours, symbols and grid of a symbol set as raw OOM XML fragments plus a code → id table.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolSet {
    /// Value of `<symbols id="…">`, e.g. "ISSprOM 2019".
    pub name: String,
    pub scale_denominator: u32,
    pub colors_xml: String,
    pub symbols_xml: String,
    pub grid_xml: String,
    codes: Vec<(SymbolCode, u32)>,
}

const SCHOOL_2019_ADDITIONS: &str =
    include_str!("../../../../assets/symbol-sets/school-2019-additions.xml");
/// IOF school symbol set 2019: all ISSprOM 2019 symbols are enlarged to 125 %.
const SCHOOL_SYMBOL_ENLARGEMENT: f64 = 1.25;

const ISSPROM_2019_4000: &str =
    include_str!("../../../../assets/symbol-sets/ISSprOM 2019_4000.omap");

impl SymbolSet {
    /// Extracts colours, symbols, grid and the code → id table from an OOM map file.
    pub fn from_omap_xml(xml: &str) -> Result<Self, OmapError> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml);
        let mut stack: Vec<String> = Vec::new();
        let mut scale_denominator = None;
        let mut name = None;
        let (mut colors, mut symbols, mut grid) = (None, None, None);
        let (mut colors_start, mut symbols_start) = (0, 0);
        let mut codes = Vec::new();

        loop {
            let before = position(&reader);
            let event = reader
                .read_event()
                .map_err(|e| OmapError::Xml(e.to_string()))?;
            let after = position(&reader);
            let (e, is_start) = match event {
                Event::Start(e) => (e, true),
                Event::Empty(e) => (e, false),
                Event::End(e) => {
                    match e.name().as_ref() {
                        "colors" => colors = Some(xml[colors_start..after].to_string()),
                        "symbols" => symbols = Some(xml[symbols_start..after].to_string()),
                        _ => {}
                    }
                    stack.pop();
                    continue;
                }
                Event::Eof => break,
                _ => continue,
            };
            let tag = e.name().as_ref().to_string();
            let parent = stack.last().map(String::as_str);
            match (tag.as_str(), parent) {
                ("georeferencing", _) => {
                    scale_denominator = attribute(&e, "scale")?.and_then(|v| v.parse().ok());
                }
                ("colors", _) => colors_start = before,
                ("symbols", _) => {
                    symbols_start = before;
                    name = attribute(&e, "id")?;
                }
                ("symbol", Some("symbols")) => {
                    let id = attribute(&e, "id")?.and_then(|v| v.parse::<u32>().ok());
                    let code = attribute(&e, "code")?;
                    if let (Some(id), Some(code)) = (id, code) {
                        codes.push((SymbolCode(code), id));
                    }
                }
                ("grid", Some("view")) => grid = Some(xml[before..after].to_string()),
                _ => {}
            }
            if is_start {
                stack.push(tag);
            }
        }

        Ok(Self {
            name: name.ok_or(OmapError::MissingElement("symbols"))?,
            scale_denominator: scale_denominator
                .ok_or(OmapError::MissingElement("georeferencing"))?,
            colors_xml: colors.ok_or(OmapError::MissingElement("colors"))?,
            symbols_xml: symbols.ok_or(OmapError::MissingElement("symbols"))?,
            grid_xml: grid.unwrap_or_default(),
            codes,
        })
    }

    /// ISSprOM 2019 at 1:4000 as shipped with OOM.
    pub fn issprom_2019() -> Self {
        Self::from_omap_xml(ISSPROM_2019_4000).expect("embedded ISSprOM 2019 symbol set is valid")
    }

    /// IOF symbol set for school orienteering maps 2019 at 1:1000.
    ///
    /// All ISSprOM 2019 symbols enlarged to 125 %, the border of paved areas (501, 501.10) as
    /// wide as an unpaved footpath (505.1), plus the six school symbols 321 and 541–545.
    pub fn school_2019() -> Self {
        let issprom = Self::issprom_2019();
        let mut symbols = enlarge_symbols(&issprom.symbols_xml, SCHOOL_SYMBOL_ENLARGEMENT)
            .expect("embedded ISSprOM 2019 symbols are valid XML");

        // IOF school set, 501: the edge line has the width of an unpaved footpath (505.1).
        let path_width: u32 = first_attribute_after(&symbols, r#" code="505.1""#, "line_width")
            .and_then(|w| w.parse().ok())
            .expect("ISSprOM 2019 defines 505.1");
        symbols = replace_first_attribute_after(
            &symbols,
            r#"<symbol type="2" id="108""#,
            "line_width",
            &path_width.to_string(),
        );
        // The edge inside the combined paved-area symbols is drawn half below the fill, so its
        // line is twice as wide as the visible part.
        for combined in [
            r#"<symbol type="16" id="104""#,
            r#"<symbol type="16" id="113""#,
        ] {
            symbols = replace_first_attribute_after(
                &symbols,
                combined,
                "line_width",
                &(2 * path_width).to_string(),
            );
        }

        let mut next_id = issprom.max_id() + 1;
        let additions = SCHOOL_2019_ADDITIONS
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let symbol = line.replacen(r#"id="@""#, &format!(r#"id="{next_id}""#), 1);
                next_id += 1;
                symbol
            })
            .collect::<Vec<_>>();
        let closing = symbols
            .rfind("</symbols>")
            .expect("symbols fragment is closed");
        let count = issprom.symbol_count() + additions.len();
        symbols = format!(
            "{}{}\n</symbols>",
            &symbols[..closing],
            additions.join("\n")
        );
        symbols = symbols.replacen(
            &format!(r#"<symbols count="{}""#, issprom.symbol_count()),
            &format!(r#"<symbols count="{count}""#),
            1,
        );

        let map = ISSPROM_2019_4000
            .replacen(&issprom.symbols_xml, &symbols, 1)
            .replacen(
                r#"<georeferencing scale="4000""#,
                r#"<georeferencing scale="1000""#,
                1,
            );
        Self::from_omap_xml(&map).expect("generated school symbol set is valid")
    }

    /// Id of the first top-level symbol with the given code.
    pub fn id_for_code(&self, code: &str) -> Option<u32> {
        self.codes
            .iter()
            .find(|(c, _)| c.0 == code)
            .map(|(_, id)| *id)
    }

    pub fn symbol_count(&self) -> usize {
        self.codes.len()
    }

    pub fn max_id(&self) -> u32 {
        self.codes.iter().map(|(_, id)| *id).max().unwrap_or(0)
    }
}

/// Value of attribute `name` in the first tag at or after the first occurrence of `anchor`.
fn first_attribute_after<'a>(xml: &'a str, anchor: &str, name: &str) -> Option<&'a str> {
    let (start, end) = attribute_range_after(xml, anchor, name)?;
    Some(&xml[start..end])
}

fn replace_first_attribute_after(xml: &str, anchor: &str, name: &str, value: &str) -> String {
    match attribute_range_after(xml, anchor, name) {
        Some((start, end)) => format!("{}{value}{}", &xml[..start], &xml[end..]),
        None => xml.to_string(),
    }
}

fn attribute_range_after(xml: &str, anchor: &str, name: &str) -> Option<(usize, usize)> {
    let from = xml.find(anchor)?;
    let needle = format!(" {name}=\"");
    let start = from + xml[from..].find(&needle)? + needle.len();
    let end = start + xml[start..].find('"')?;
    Some((start, end))
}

/// Current byte offset of `reader` in its input.
pub(crate) fn position(reader: &quick_xml::Reader<&[u8]>) -> usize {
    usize::try_from(reader.buffer_position()).expect("XML input fits in memory")
}

/// Unescaped value of attribute `name`, if present.
pub(crate) fn attribute(
    e: &quick_xml::events::BytesStart<'_>,
    name: &str,
) -> Result<Option<String>, OmapError> {
    let attr = e
        .try_get_attribute(name)
        .map_err(|err| OmapError::Xml(err.to_string()))?;
    attr.map(|a| {
        a.normalized_value(quick_xml::XmlVersion::Explicit1_0)
            .map(|v| v.into_owned())
    })
    .transpose()
    .map_err(|err| OmapError::Xml(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_attr<'a>(xml: &'a str, anchor: &str, name: &str) -> &'a str {
        first_attribute_after(xml, anchor, name).unwrap_or_else(|| panic!("{anchor} {name}"))
    }

    #[test]
    fn school_set() {
        let issprom = SymbolSet::issprom_2019();
        let school = SymbolSet::school_2019();
        assert_eq!(school.symbol_count(), 191);
        assert_eq!(school.scale_denominator, 1000);
        assert_eq!(school.name, "ISSprOM 2019");
        for (code, id) in [
            ("321", 185),
            ("541", 186),
            ("542", 187),
            ("543", 188),
            ("544", 189),
            ("545", 190),
        ] {
            assert_eq!(school.id_for_code(code), Some(id), "code {code}");
        }
        assert!(
            school
                .symbols_xml
                .starts_with(r#"<symbols count="191" id="ISSprOM 2019">"#)
        );

        let contour = r#"<symbol type="2" id="0" code="101""#;
        let base: f64 = first_attr(&issprom.symbols_xml, contour, "line_width")
            .parse()
            .unwrap();
        let enlarged = first_attr(&school.symbols_xml, contour, "line_width");
        assert_eq!(enlarged, format!("{}", (base * 1.25).round()));

        let path = first_attr(&school.symbols_xml, r#"code="505.1""#, "line_width");
        assert_eq!(path, "438");
        let edge = first_attr(
            &school.symbols_xml,
            r#"<symbol type="2" id="108""#,
            "line_width",
        );
        assert_eq!(edge, path, "visible edge 501.1 is as wide as 505.1");
        for id in ["104", "113"] {
            // The border of the combined symbols is drawn half below the brown fill.
            let combined = format!(r#"<symbol type="16" id="{id}""#);
            assert_eq!(
                first_attr(&school.symbols_xml, &combined, "line_width"),
                "876",
                "id {id}"
            );
        }

        let mut reader = quick_xml::Reader::from_str(&school.symbols_xml);
        while !matches!(reader.read_event().unwrap(), quick_xml::events::Event::Eof) {}
    }

    #[test]
    fn issprom_codes() {
        let set = SymbolSet::issprom_2019();
        for (code, id) in [
            ("101", 0),
            ("102", 2),
            ("103", 5),
            ("501", 104),
            ("501.5", 105),
            ("501.1", 108),
            ("520", 139),
            ("521", 141),
            ("522", 145),
        ] {
            assert_eq!(set.id_for_code(code), Some(id), "code {code}");
        }
        assert_eq!(set.id_for_code("9999"), None);
        assert_eq!(set.symbol_count(), 185);
        assert_eq!(set.max_id(), 184);
        assert_eq!(set.name, "ISSprOM 2019");
        assert_eq!(set.scale_denominator, 4000);
        assert!(set.colors_xml.starts_with(r#"<colors count="32">"#));
        assert!(set.colors_xml.ends_with("</colors>"));
        assert!(
            set.symbols_xml
                .starts_with(r#"<symbols count="185" id="ISSprOM 2019">"#)
        );
        assert!(set.symbols_xml.ends_with("</symbols>"));
        assert!(set.grid_xml.starts_with("<grid "));
    }
}
