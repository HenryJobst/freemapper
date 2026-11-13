//! Symbol sets: symbols copied verbatim from OOM symbol set files, colours in the CMYK of IOF
//! "Printing and Colour Definitions" Rev. 4 (see `assets/symbol-sets/README.md`).

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
    /// Codes of another norm and the code of this set that draws them, `None` for a code this
    /// set leaves out (`isom-aliases.toml`); asked before `codes`.
    aliases: Vec<(String, Option<String>)>,
}

const SCHOOL_2019_ADDITIONS: &str =
    include_str!("../../../../assets/symbol-sets/school-2019-additions.xml");
const SHEET_ADDITIONS: &str = include_str!("../../../../assets/symbol-sets/sheet-additions.xml");
const WALKABLE_AREA_ADDITIONS: &str =
    include_str!("../../../../assets/symbol-sets/walkable-area-additions.xml");
/// The code of the walkable area (belaufbare Fläche), the base fill of the sprint map.
pub const WALKABLE_AREA_CODE: &str = "990";
/// The colour of the walkable area: the CMYK and spot colour mix of "Lower brown 30%" (501.5,
/// IOF Rev. 4: 6/23/33/0), at a priority behind every other colour, so that it is painted first
/// and everything else lies on it. `{priority}` is the next free priority.
const WALKABLE_AREA_COLOUR: &str = r#"<color priority="{priority}" name="Lower brown 30% for walkable area" c="0.06" m="0.23" y="0.33" k="0" opacity="1"><spotcolors knockout="true"><component factor="0.3" spotcolor="8"/></spotcolors><cmyk method="custom"/><rgb method="cmyk" r="0.94" g="0.77" b="0.67"/></color>"#;
/// IOF school symbol set 2019: all ISSprOM 2019 symbols are enlarged to 125 %.
pub const SCHOOL_SYMBOL_ENLARGEMENT: f64 = 1.25;

const ISSPROM_2019_4000: &str =
    include_str!("../../../../assets/symbol-sets/ISSprOM 2019_4000.omap");
const ISOM_2017_2_15000: &str =
    include_str!("../../../../assets/symbol-sets/ISOM 2017-2_15000.omap");
const ISOM_ALIASES: &str = include_str!("../../../../assets/symbol-sets/isom-aliases.toml");

/// `isom-aliases.toml`.
#[derive(serde::Deserialize)]
struct AliasTable {
    drop: Vec<String>,
    alias: std::collections::BTreeMap<String, String>,
}

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
            aliases: Vec::new(),
        })
    }

    /// ISSprOM 2019 at 1:4000 as shipped with OOM, with the colours of IOF Rev. 4.
    pub fn issprom_2019() -> Self {
        Self::from_omap_xml(ISSPROM_2019_4000).expect("embedded ISSprOM 2019 symbol set is valid")
    }

    /// ISOM 2017-2 as shipped with OOM for 1:15000, with the colours of IOF Rev. 4, its symbols
    /// enlarged by `enlargement` (1.5 for 1:10000, 2.0 for 1:7500). It answers the ISSprOM codes
    /// the build modules use through `isom-aliases.toml`.
    pub fn isom_2017_2(enlargement: f64) -> Self {
        let isom = Self::from_omap_xml(ISOM_2017_2_15000)
            .expect("embedded ISOM 2017-2 symbol set is valid");
        let symbols = enlarge_symbols(&isom.symbols_xml, enlargement)
            .expect("embedded ISOM 2017-2 symbols are valid XML");
        let scale = (f64::from(isom.scale_denominator) / enlargement).round();
        let map = ISOM_2017_2_15000
            .replacen(&isom.symbols_xml, &symbols, 1)
            .replacen(
                r#"<georeferencing scale="15000""#,
                &format!(r#"<georeferencing scale="{scale}""#),
                1,
            );
        let table: AliasTable =
            toml::from_str(ISOM_ALIASES).expect("embedded isom-aliases.toml is valid");
        let aliases = table
            .drop
            .into_iter()
            .map(|code| (code, None))
            .chain(table.alias.into_iter().map(|(from, to)| (from, Some(to))))
            .collect();
        Self {
            aliases,
            ..Self::from_omap_xml(&map).expect("generated ISOM 2017-2 symbol set is valid")
        }
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

        let map = ISSPROM_2019_4000
            .replacen(&issprom.symbols_xml, &symbols, 1)
            .replacen(
                r#"<georeferencing scale="4000""#,
                r#"<georeferencing scale="1000""#,
                1,
            );
        Self::from_omap_xml(&map)
            .expect("generated school symbol set is valid")
            .with_additions(SCHOOL_2019_ADDITIONS)
    }

    /// This set plus the freemapper sheet symbols for the peripheral information (991–994).
    /// They are added unscaled: their sizes are absolute on paper.
    pub fn with_sheet_symbols(&self) -> Self {
        self.with_additions(SHEET_ADDITIONS)
    }

    /// This set plus the walkable area (belaufbare Fläche, [`WALKABLE_AREA_CODE`]): a colour
    /// with the mix of "Lower brown 30%" appended behind all others, and the area symbol in it.
    /// Existing colour priorities and symbol ids stay unchanged.
    pub fn with_walkable_area(&self) -> Self {
        let count: usize = first_attribute_after(&self.colors_xml, "<colors ", "count")
            .and_then(|c| c.parse().ok())
            .expect("colours fragment has a count");
        let colour = WALKABLE_AREA_COLOUR.replace("{priority}", &count.to_string());
        let closing = self
            .colors_xml
            .rfind("</colors>")
            .expect("colours fragment is closed");
        let colors_xml = format!("{}{colour}\n</colors>", &self.colors_xml[..closing]).replacen(
            &format!(r#"<colors count="{count}""#),
            &format!(r#"<colors count="{}""#, count + 1),
            1,
        );
        let symbol = WALKABLE_AREA_ADDITIONS
            .replace(r#"inner_color="32""#, &format!(r#"inner_color="{count}""#));
        Self {
            colors_xml,
            ..self.clone()
        }
        .with_additions(&symbol)
    }

    /// Appends symbols given one per line with `id="@"`; they get consecutive ids after the
    /// highest existing id, so existing ids stay unchanged.
    pub fn with_additions(&self, additions_xml: &str) -> Self {
        let mut next_id = self.max_id() + 1;
        let additions = additions_xml
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let symbol = line.replacen(r#"id="@""#, &format!(r#"id="{next_id}""#), 1);
                next_id += 1;
                symbol
            })
            .collect::<Vec<_>>();
        let count: usize = first_attribute_after(&self.symbols_xml, "<symbols ", "count")
            .and_then(|c| c.parse().ok())
            .expect("symbols fragment has a count");
        let closing = self
            .symbols_xml
            .rfind("</symbols>")
            .expect("symbols fragment is closed");
        let symbols = format!(
            "{}{}\n</symbols>",
            &self.symbols_xml[..closing],
            additions.join("\n")
        )
        .replacen(
            &format!(r#"<symbols count="{count}""#),
            &format!(r#"<symbols count="{}""#, count + additions.len()),
            1,
        );
        let map = format!(
            r#"<map><georeferencing scale="{}"/>{}<barrier>{symbols}<view>{}</view></barrier></map>"#,
            self.scale_denominator, self.colors_xml, self.grid_xml
        );
        Self {
            aliases: self.aliases.clone(),
            ..Self::from_omap_xml(&map).expect("symbol set with additions is valid")
        }
    }

    /// Id of the first top-level symbol with the given code, or with the code an alias names
    /// for it; `None` for a code the set has not or leaves out.
    pub fn id_for_code(&self, code: &str) -> Option<u32> {
        let code = match self.aliases.iter().find(|(from, _)| from == code) {
            Some((_, to)) => to.as_deref()?,
            None => code,
        };
        self.codes
            .iter()
            .find(|(c, _)| c.0 == code)
            .map(|(_, id)| *id)
    }

    /// Whether an alias of this set leaves the code out (a code the norm has no symbol for).
    pub fn leaves_out(&self, code: &str) -> bool {
        self.aliases
            .iter()
            .any(|(from, to)| from == code && to.is_none())
    }

    /// OOM type of the top-level symbol `id` (1 point, 2 line, 4 area, 8 text, 16 combined).
    pub fn symbol_type(&self, id: u32) -> Option<u32> {
        let marker = format!(r#"" id="{id}""#);
        self.symbols_xml
            .match_indices(r#"<symbol type=""#)
            .find_map(|(at, open)| {
                let rest = &self.symbols_xml[at + open.len()..];
                let digits = rest.find('"')?;
                rest[digits..]
                    .starts_with(&marker)
                    .then(|| rest[..digits].parse().ok())?
            })
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
    fn the_isom_set_has_the_path_classes_and_answers_issprom_codes() {
        let set = SymbolSet::isom_2017_2(1.0);
        assert_eq!(set.name, "ISOM 2017-2");
        assert_eq!(set.scale_denominator, 15000);
        for code in [
            "101", "102", "502", "503", "504", "505", "506", "507", "508", "520", "521", "532",
        ] {
            assert!(set.id_for_code(code).is_some(), "{code}");
        }
        assert_eq!(set.id_for_code("505.1"), set.id_for_code("505"));
        assert_eq!(set.id_for_code("505.2"), set.id_for_code("504"));
        assert_eq!(set.id_for_code("532.7"), set.id_for_code("532"));
        // 501.1 is a step in ISSprOM and the motorway in ISOM: the alias wins.
        assert_eq!(set.id_for_code("501.1"), None);
        assert!(set.leaves_out("501.1"));
        assert_eq!(set.id_for_code("990"), None);
        assert!(!set.leaves_out("505"));
        // The sheet symbols keep the aliases.
        let sheet = set.with_sheet_symbols();
        assert!(sheet.id_for_code("991").is_some());
        assert_eq!(sheet.id_for_code("505.1"), set.id_for_code("505"));
    }

    #[test]
    fn the_isom_set_at_1_to_10000_and_7500_is_enlarged() {
        let base = SymbolSet::isom_2017_2(1.0);
        let width = |set: &SymbolSet| -> f64 {
            first_attr(&set.symbols_xml, r#" code="505""#, "line_width")
                .parse()
                .unwrap()
        };
        for (enlargement, scale) in [(1.5, 10000), (2.0, 7500)] {
            let set = SymbolSet::isom_2017_2(enlargement);
            assert_eq!(set.scale_denominator, scale);
            assert_eq!(width(&set), (width(&base) * enlargement).round());
        }
    }

    /// An alias keeps the kind of geometry: a line code answers with a line symbol, an area code
    /// with an area or a combined symbol.
    #[test]
    fn isom_aliases_keep_the_symbol_type() {
        let issprom = SymbolSet::issprom_2019();
        let isom = SymbolSet::isom_2017_2(1.0);
        let table: AliasTable = toml::from_str(ISOM_ALIASES).unwrap();
        for (from, to) in &table.alias {
            let before = issprom
                .symbol_type(
                    issprom
                        .id_for_code(from)
                        .unwrap_or_else(|| panic!("{from}")),
                )
                .unwrap();
            let after = isom
                .symbol_type(isom.id_for_code(to).unwrap_or_else(|| panic!("{to}")))
                .unwrap();
            assert!(
                before == after || after == 16,
                "{from} (type {before}) -> {to} (type {after})"
            );
        }
        for code in &table.drop {
            assert!(isom.leaves_out(code));
        }
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

    /// (top-level symbol id, element, attribute, value) for every size attribute in `xml`.
    fn size_attributes(xml: &str) -> Vec<(String, String, &'static str, f64)> {
        use quick_xml::events::Event;
        const SIZES: [(&str, &str); 3] = [
            ("line_symbol", "line_width"),
            ("point_symbol", "inner_radius"),
            ("point_symbol", "outer_width"),
        ];
        let mut reader = quick_xml::Reader::from_str(xml);
        let (mut depth, mut symbol_id) = (0, String::new());
        let mut sizes = Vec::new();
        loop {
            let (e, is_start) = match reader.read_event().unwrap() {
                Event::Start(e) => (e, true),
                Event::Empty(e) => (e, false),
                Event::End(_) => {
                    depth -= 1;
                    continue;
                }
                Event::Eof => break,
                _ => continue,
            };
            let tag = e.name().as_ref().to_string();
            if tag == "symbol" && depth == 1 {
                symbol_id = attribute(&e, "id").unwrap().unwrap();
            }
            for (element, name) in SIZES {
                if tag == element
                    && let Some(value) = attribute(&e, name).unwrap()
                {
                    sizes.push((symbol_id.clone(), tag.clone(), name, value.parse().unwrap()));
                }
            }
            if is_start {
                depth += 1;
            }
        }
        sizes
    }

    #[test]
    fn school_set_enlarges_every_issprom_size() {
        let issprom = size_attributes(&SymbolSet::issprom_2019().symbols_xml);
        let school = size_attributes(&SymbolSet::school_2019().symbols_xml);
        assert!(issprom.len() > 300, "{} sizes", issprom.len());
        // The school additions come after all ISSprOM symbols.
        assert!(school.len() > issprom.len());
        // 501 edge (108) and the combined paved areas (104, 113) are redefined by the norm.
        let redefined = ["104", "108", "113"];
        for (base, enlarged) in issprom.iter().zip(&school) {
            assert_eq!(
                (&base.0, &base.1, base.2),
                (&enlarged.0, &enlarged.1, enlarged.2)
            );
            if redefined.contains(&base.0.as_str()) && base.2 == "line_width" {
                continue;
            }
            assert_eq!(enlarged.3, (base.3 * 1.25).round(), "{base:?}");
        }
    }

    #[test]
    fn sheet_additions_extend_both_sets() {
        let school = SymbolSet::school_2019().with_sheet_symbols();
        assert_eq!(school.symbol_count(), 195);
        assert_eq!(school.id_for_code("321"), Some(185));
        for (code, id) in [("991", 191), ("992", 192), ("993", 193), ("994", 194)] {
            assert_eq!(school.id_for_code(code), Some(id), "code {code}");
        }
        assert!(
            school
                .symbols_xml
                .starts_with(r#"<symbols count="195" id="ISSprOM 2019">"#)
        );
        assert_eq!(school.scale_denominator, 1000);

        let sprint = SymbolSet::issprom_2019().with_sheet_symbols();
        assert_eq!(sprint.symbol_count(), 189);
        assert_eq!(sprint.id_for_code("991"), Some(185));
        assert_eq!(sprint.scale_denominator, 4000);
        assert_eq!(
            first_attr(&sprint.symbols_xml, r#"code="994""#, "line_width"),
            "250"
        );
        let title = r#"code="991""#;
        assert_eq!(first_attr(&sprint.symbols_xml, title, "size"), "5000");
        assert_eq!(first_attr(&sprint.symbols_xml, title, "bold"), "true");
    }

    #[test]
    fn walkable_area_is_the_last_colour_and_its_own_symbol() {
        let plain = SymbolSet::issprom_2019().with_sheet_symbols();
        let set = plain.with_walkable_area();
        // A colour behind all 32 of the norm set, the priorities stay contiguous.
        assert!(set.colors_xml.starts_with(r#"<colors count="33">"#));
        let table = colour_table(&set.colors_xml);
        assert_eq!(table.len(), 33);
        let base = &table[32];
        assert_eq!(base.name, "Lower brown 30% for walkable area");
        assert!(
            set.colors_xml
                .contains(r#"<color priority="32" name="Lower brown 30% for walkable area""#)
        );
        // The mix of the paved area's lower brown (501.5), IOF Rev. 4.
        let lower = table.iter().find(|c| c.name == "Lower brown 30%").unwrap();
        assert_eq!(base.cmyk, lower.cmyk);
        assert_eq!(base.rgb, lower.rgb);
        assert_eq!(base.components, lower.components);
        // Its symbol comes after the sheet symbols, which keep their ids.
        assert_eq!(set.id_for_code(WALKABLE_AREA_CODE), Some(189));
        assert_eq!(set.id_for_code("991"), plain.id_for_code("991"));
        assert_eq!(set.symbol_type(189), Some(4));
        let from = set.symbols_xml.find(r#" code="990""#).unwrap();
        assert_eq!(
            first_attr(&set.symbols_xml[from..], "<area_symbol ", "inner_color"),
            "32"
        );
        assert!(
            set.symbols_xml
                .contains(r#"name="Walkable area (base fill)""#)
        );
        // Every other colour is as before.
        assert!(set.colors_xml.starts_with(
            &plain.colors_xml[..plain.colors_xml.len() - "</colors>".len()].replacen(
                "count=\"32\"",
                "count=\"33\"",
                1
            )
        ));
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

    /// One `<color>` of a colour table: name, CMYK, RGB, both methods and the spot colour
    /// components (factor, priority).
    struct Colour {
        name: String,
        cmyk: [f64; 4],
        rgb: [f64; 3],
        cmyk_method: String,
        rgb_method: String,
        components: Vec<(f64, usize)>,
    }

    fn colour_table(colors_xml: &str) -> Vec<Colour> {
        let num = |xml: &str, anchor: &str, name: &str| -> f64 {
            first_attr(xml, anchor, name).parse().unwrap()
        };
        // Each piece keeps a blank before its first attribute, which `first_attr` needs.
        colors_xml
            .split("<color ")
            .skip(1)
            .map(|c| format!(" {c}"))
            .map(|c| Colour {
                name: first_attr(&c, "", "name").to_string(),
                cmyk: ["c", "m", "y", "k"].map(|n| num(&c, "", n)),
                rgb: ["r", "g", "b"].map(|n| num(&c, "<rgb ", n)),
                cmyk_method: first_attr(&c, "<cmyk ", "method").to_string(),
                rgb_method: first_attr(&c, "<rgb ", "method").to_string(),
                components: c
                    .split("<component")
                    .skip(1)
                    .map(|k| (num(k, "", "factor"), num(k, "", "spotcolor") as usize))
                    .collect(),
            })
            .collect()
    }

    fn close(a: &[f64], b: &[f64]) -> bool {
        a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.0006)
    }

    #[test]
    fn colours_follow_iof_rev4() {
        let table = colour_table(&SymbolSet::issprom_2019().colors_xml);
        let cmyk = |name: &str| {
            let c = table.iter().find(|c| c.name == name).unwrap();
            c.cmyk.map(|v| (v * 1000.0).round() / 10.0)
        };
        // IOF "Printing and Colour Definitions", Revision 4 (September 2024), chapter 6.
        for (name, expected) in [
            ("Purple for course overprint", [35.0, 85.0, 0.0, 0.0]),
            ("Purple 50%", [18.0, 43.0, 0.0, 0.0]),
            ("Black 100%", [0.0, 0.0, 0.0, 100.0]),
            ("Green 100%", [80.0, 0.0, 100.0, 0.0]),
            ("Green 60%", [48.0, 0.0, 60.0, 0.0]),
            ("Green 30%", [24.0, 0.0, 30.0, 0.0]),
            ("Blue 100%", [100.0, 0.0, 0.0, 0.0]),
            ("Blue 70%", [70.0, 0.0, 0.0, 0.0]),
            ("Brown 100%", [25.0, 75.0, 100.0, 0.0]),
            ("Lower brown 50%", [10.0, 35.0, 50.0, 0.0]),
            ("Lower brown 30%", [6.0, 23.0, 33.0, 0.0]),
            ("Green 50%/Yellow 100%", [38.0, 27.0, 100.0, 0.0]),
            ("Yellow 100%", [0.0, 27.0, 79.0, 0.0]),
            ("Yellow 50%", [0.0, 14.0, 40.0, 0.0]),
        ] {
            assert_eq!(cmyk(name), expected, "{name}");
        }
    }

    #[test]
    fn buildings_are_black_50() {
        // ISSprOM 2019-2 (02.01.2022): 521 Building, change of colour 60 % to 50 %.
        for set in [SymbolSet::issprom_2019(), SymbolSet::school_2019()] {
            let table = colour_table(&set.colors_xml);
            let black_50 = table.iter().position(|c| c.name == "Black 50%").unwrap();
            for code in ["521.1", "521.3"] {
                let from = set.symbols_xml.find(&format!(r#" code="{code}""#)).unwrap();
                let fill = first_attr(&set.symbols_xml[from..], "<area_symbol ", "inner_color");
                assert_eq!(fill, black_50.to_string(), "{code}");
            }
        }
    }

    #[test]
    fn isom_colours_follow_iof_rev4() {
        let table = colour_table(&SymbolSet::isom_2017_2(1.0).colors_xml);
        let cmyk = |name: &str| {
            let c = table.iter().find(|c| c.name == name).unwrap();
            c.cmyk.map(|v| (v * 1000.0).round() / 10.0)
        };
        for (name, expected) in [
            ("Green 100%", [80.0, 0.0, 100.0, 0.0]),
            ("Green 60%", [48.0, 0.0, 60.0, 0.0]),
            ("Brown 100%", [25.0, 75.0, 100.0, 0.0]),
            ("Upper brown 50%", [10.0, 35.0, 50.0, 0.0]),
            ("Lower brown 50%", [10.0, 35.0, 50.0, 0.0]),
            ("Yellow 100%/Green 50%", [38.0, 27.0, 100.0, 0.0]),
            ("Yellow", [0.0, 27.0, 79.0, 0.0]),
            ("Yellow 50%", [0.0, 14.0, 40.0, 0.0]),
            ("Blue 100%", [100.0, 0.0, 0.0, 0.0]),
            ("Black 65%", [0.0, 0.0, 0.0, 65.0]),
        ] {
            assert_eq!(cmyk(name), expected, "{name}");
        }
    }

    #[test]
    fn colour_table_is_what_oom_computes() {
        for set in [SymbolSet::issprom_2019(), SymbolSet::isom_2017_2(1.0)] {
            colour_table_matches_oom(&set.colors_xml);
        }
    }

    fn colour_table_matches_oom(colors_xml: &str) {
        let table = colour_table(colors_xml);
        for c in &table {
            // OOM recomputes these on load (MapColor::cmykFromSpotColors, rgbFromSpotColors).
            let mut cmyk = [0.0; 4];
            let mut rgb = [1.0; 3];
            for &(factor, spot) in &c.components {
                let spot = &table[spot];
                for (v, spot) in cmyk.iter_mut().zip(spot.cmyk) {
                    *v += factor * spot * (1.0 - *v);
                }
                for (v, spot) in rgb.iter_mut().zip(spot.rgb) {
                    *v -= factor * (1.0 - spot) * *v;
                }
            }
            match c.cmyk_method.as_str() {
                "spotcolor" => assert!(close(&c.cmyk, &cmyk), "{} {:?}", c.name, cmyk),
                method => assert_eq!(method, "custom", "{}", c.name),
            }
            let [cy, m, y, k] = c.cmyk;
            let from_cmyk = [
                (1.0 - cy) * (1.0 - k),
                (1.0 - m) * (1.0 - k),
                (1.0 - y) * (1.0 - k),
            ];
            match c.rgb_method.as_str() {
                "spotcolor" => assert!(close(&c.rgb, &rgb), "{} {:?}", c.name, rgb),
                "cmyk" => assert!(close(&c.rgb, &from_cmyk), "{} {:?}", c.name, from_cmyk),
                other => panic!("{}: rgb method {other}", c.name),
            }
        }
    }
}
