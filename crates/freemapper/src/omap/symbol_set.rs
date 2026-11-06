//! Symbol sets: colours and symbols copied verbatim from OOM symbol set files.

use super::OmapError;

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
