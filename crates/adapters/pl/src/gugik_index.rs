// SPDX-License-Identifier: AGPL-3.0-or-later

//! The sheet index of GUGiK: which sheets of the terrain model (NMT), the surface model (NMPT) and
//! the orthophoto touch a map area. [`GugikIndex`] is the [`SheetCatalog`] of [`crate::rasters::PolskaRasters`]
//! (`crate::rasters`).
//!
//! The index is one WFS 2.0.0 per product with one layer per flight year
//! (`gugik:SkorowidzNMT2022`). A request names every layer of the product, so one answer lists
//! every sheet of every year. The layers are not fixed: a year is added every year, and the
//! orthophoto layers skip years (1957, 1958, 1961, …). The client reads the layer names from
//! `DescribeFeatureType` once and asks only for layers that exist, because a name the server does
//! not know fails the whole request.
//!
//! The reference system is EPSG:2180 with the axis order of the URN: **north, then east**. The
//! box of a request and every coordinate of an answer (`gml:lowerCorner`, `gml:upperCorner`)
//! are written `N E`.
//!
//! What the client keeps, and why:
//!
//! - Heights only as ESRI ASCII grids (`.asc`), the one format the reader of
//!   `freemapper-sources` reads. A sheet that is only published as an archive (`.zip`, `.xyz.zip`)
//!   is left out silently: it is normal, not a fault of the index, and the choice then falls to
//!   another year of the same sheet.
//! - Orthophotos only as GeoTIFF (`.tif`) **and only in colour `RGB`**. The index also lists
//!   colour-infrared sheets (`CIR`, false colour: vegetation is red) and black and white ones. A
//!   CIR orthophoto must never be taken for the orthophoto of a map, so these are filtered out
//!   here, before the choice of a sheet. The same sheet number can have a 0.05 m RGB, a 0.25 m
//!   CIR and a 0.25 m RGB sheet in one year.
//! - Orthophotos only of [`MIN_ORTO_YEAR`] and later; the older flights are black and white or
//!   coarse, and the request stays short.
//! - A download address only when it is `https` on [`DOWNLOAD_HOST`]. The addresses come from a
//!   server, and a sheet whose address leads elsewhere is skipped with a warning.
//! - A sheet that lacks its number, flight year, pixel size, extent or address is skipped with a
//!   warning; the other sheets of the answer stay.
//!
//! Every request goes through the fetcher the caller hands in, which the adapter holds to its
//! hosts ([`crate::polska`]), so the client never asks an address that its policy does not name.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::fetch::{
    Fetcher, get_with_retries, is_ogc_report, ogc_exception, one_line,
};
use freemapper_adapter_api::note;
use freemapper_adapter_api::note::{Arg, Note};
use freemapper_adapter_api::policy::host_of;
use freemapper_georef::crs::BoundingBox;
use quick_xml::events::{BytesStart, Event};

use crate::rasters::SheetCatalog;
use crate::sheets::{Product, Sheet, SheetSource};

/// The host of the index services.
pub const INDEX_HOST: &str = "mapy.geoportal.gov.pl";
/// The only host a sheet may be downloaded from.
pub const DOWNLOAD_HOST: &str = "opendata.geoportal.gov.pl";
/// The services of the index, below the host.
const SERVICE_BASE: &str = "https://mapy.geoportal.gov.pl/wss/service/PZGIK";
/// The oldest flight year of orthophotos the index is asked for.
pub const MIN_ORTO_YEAR: u16 = 2000;
/// How many skipped sheets are named one by one; the rest is counted.
const WARNED_INDIVIDUALLY: usize = 5;

/// How the index of a product is reached.
struct Service {
    /// The service folder below [`SERVICE_BASE`].
    folder: &'static str,
    /// The start of the layer names; the flight year follows.
    prefix: &'static str,
    /// The oldest year asked for.
    min_year: u16,
}

fn service(product: Product) -> Service {
    match product {
        Product::Terrain => Service {
            folder: "NumerycznyModelTerenuEVRF2007",
            prefix: "SkorowidzNMT",
            min_year: 0,
        },
        Product::Surface => Service {
            folder: "NumerycznyModelPokryciaTerenuEVRF2007",
            prefix: "SkorowidzNMPT",
            min_year: 0,
        },
        Product::Orthophoto => Service {
            folder: "ORTO",
            prefix: "SkorowidzOrtofomapy",
            min_year: MIN_ORTO_YEAR,
        },
    }
}

/// The address of the layer list of `product`.
fn describe_url(product: Product) -> String {
    format!(
        "{SERVICE_BASE}/{}/WFS/Skorowidze?SERVICE=WFS&VERSION=2.0.0&REQUEST=DescribeFeatureType",
        service(product).folder
    )
}

/// The address of the request for every sheet of `layers` that touches `area` (EPSG:2180). The
/// box is written north, east (the axis order of the URN), rounded outward to whole metres.
fn sheets_url(product: Product, layers: &[String], area: &BoundingBox) -> String {
    let names: Vec<String> = layers.iter().map(|l| format!("gugik:{l}")).collect();
    format!(
        "{SERVICE_BASE}/{}/WFS/Skorowidze?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
         &TYPENAMES={}&BBOX={},{},{},{},urn:ogc:def:crs:EPSG::2180",
        service(product).folder,
        names.join(","),
        area.min_n.floor(),
        area.min_e.floor(),
        area.max_n.ceil(),
        area.max_e.ceil(),
    )
}

/// The sheet index of GUGiK as a [`SheetCatalog`].
#[derive(Default)]
pub struct GugikIndex {
    /// The layers of each product, read once.
    layers: Mutex<HashMap<&'static str, Vec<String>>>,
}

impl GugikIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// The names of the year layers of `product`, oldest first, from the service.
    fn layers(&self, fetcher: &dyn Fetcher, product: Product) -> Result<Vec<String>, FetchError> {
        let key = service(product).folder;
        let known = self.layers.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(layers) = known.get(key) {
            return Ok(layers.clone());
        }
        drop(known);
        let url = describe_url(product);
        let layers = get_with_retries(fetcher, &url, |body| read_layers(product, body, &url))?;
        self.layers
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key, layers.clone());
        Ok(layers)
    }
}

impl SheetCatalog for GugikIndex {
    fn sheets(
        &self,
        fetcher: &dyn Fetcher,
        product: Product,
        area: &BoundingBox,
        progress: &mut dyn FnMut(Note),
    ) -> Result<Vec<Sheet>, AdapterError> {
        let layers = self.layers(fetcher, product)?;
        let url = sheets_url(product, &layers, area);
        let read = get_with_retries(fetcher, &url, |body| read_sheets(product, body, &url))?;
        warn_about(&read.skipped, progress);
        Ok(read.sheets)
    }
}

/// Tells `progress` about the sheets that were skipped: the first [`WARNED_INDIVIDUALLY`] by
/// reason, the rest as a count.
fn warn_about(skipped: &[String], progress: &mut dyn FnMut(Note)) {
    for reason in skipped.iter().take(WARNED_INDIVIDUALLY) {
        progress(note!(
            "warning-error",
            error = Arg::error(SheetSkipped(reason.clone()))
        ));
    }
    if skipped.len() > WARNED_INDIVIDUALLY {
        progress(note!(
            "warning-error",
            error = Arg::error(SheetSkipped(format!(
                "{} more sheets of the index are skipped for the same reasons",
                skipped.len() - WARNED_INDIVIDUALLY
            )))
        ));
    }
}

/// A warning about a sheet the index lists and this program does not take.
#[derive(Debug)]
struct SheetSkipped(String);

impl fmt::Display for SheetSkipped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GUGiK sheet index: {}", self.0)
    }
}

impl std::error::Error for SheetSkipped {}

/// What an answer of the index holds.
#[derive(Debug, Default)]
struct Read {
    sheets: Vec<Sheet>,
    /// Why each skipped sheet was skipped, for the warnings.
    skipped: Vec<String>,
}

pub(crate) fn http_error(url: &str, message: impl Into<String>) -> FetchError {
    FetchError::Http {
        url: url.to_string(),
        message: message.into(),
    }
}

/// An OGC exception report (the server sends one with status 200) as the error it is.
pub(crate) fn report_error(url: &str, body: &[u8]) -> FetchError {
    let detail = ogc_exception(body).unwrap_or_else(|| "an exception report".to_string());
    http_error(url, format!("the index service answered with {detail}"))
}

/// The local part of a qualified XML name.
pub(crate) fn local(name: &str) -> String {
    name.rsplit(':').next().unwrap_or_default().to_string()
}

/// The character an XML entity (`amp` in `&amp;`) stands for.
pub(crate) fn entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => None,
    }
}

/// The year layers of `product` in a `DescribeFeatureType` answer: the elements named
/// `<prefix><year>`, from the oldest year the product is asked for, oldest first.
fn read_layers(product: Product, body: &[u8], url: &str) -> Result<Vec<String>, FetchError> {
    if is_ogc_report(body) {
        return Err(report_error(url, body));
    }
    let Service {
        prefix, min_year, ..
    } = service(product);
    let mut reader = quick_xml::Reader::from_reader(body);
    let mut buffer = Vec::new();
    let mut years = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(e) | Event::Empty(e)) if local(e.name().as_ref()) == "element" => {
                let name = attribute(&e, "name").unwrap_or_default();
                if is_layer(&name, prefix)
                    && let Some(year) = name[prefix.len()..].parse::<u16>().ok()
                    && year >= min_year
                {
                    years.push(year);
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(http_error(url, format!("the layer list is no XML: {e}"))),
        }
        buffer.clear();
    }
    years.sort_unstable();
    years.dedup();
    if years.is_empty() {
        return Err(http_error(url, "the index service lists no layer"));
    }
    Ok(years.into_iter().map(|y| format!("{prefix}{y}")).collect())
}

/// The value of the attribute `name` (any prefix) of `element`.
pub(crate) fn attribute(element: &BytesStart<'_>, name: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|a| local(a.key.as_ref()) == name)
        .map(|a| {
            // The value as written (`&amp;` and all) when it holds an entity that is no XML's.
            a.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map_or_else(|_| a.value.to_string(), |v| v.to_string())
        })
}

/// The fields of one feature the client reads, by their local element names.
const FIELDS: [&str; 9] = [
    "godlo",
    "akt_rok",
    "char_przestrz",
    "piksel",
    "kolor",
    "zrodlo_danych",
    "url_do_pobrania",
    "lowerCorner",
    "upperCorner",
];

/// One feature while it is read.
struct Draft {
    /// The element name, `SkorowidzNMT2022`: its last four digits are the flight year of the
    /// layer.
    layer: String,
    fields: HashMap<&'static str, String>,
}

/// The sheets of an answer of the index for `product`. An exception report, an answer that is no
/// feature collection or ends before it closes, and an answer that holds fewer features than it
/// matched are errors (and are asked again); a single feature that cannot be used is a
/// [`Read::skipped`] entry.
fn read_sheets(product: Product, body: &[u8], url: &str) -> Result<Read, FetchError> {
    if is_ogc_report(body) {
        return Err(report_error(url, body));
    }
    let Service { prefix, .. } = service(product);
    let mut reader = quick_xml::Reader::from_reader(body);
    let mut buffer = Vec::new();
    let mut read = Read::default();
    let mut depth = 0usize;
    let mut root_closed = false;
    let mut draft: Option<Draft> = None;
    let mut text = String::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|e| http_error(url, format!("the answer is no XML: {e}")))?;
        match event {
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                if depth == 0 {
                    if name != "FeatureCollection" {
                        return Err(http_error(
                            url,
                            format!("the answer is no feature collection but <{name}>"),
                        ));
                    }
                    check_counts(&e, url)?;
                } else if draft.is_none() && is_layer(&name, prefix) {
                    draft = Some(Draft {
                        layer: name,
                        fields: HashMap::new(),
                    });
                }
                depth += 1;
                text.clear();
            }
            Event::Text(t) => text.push_str(t.as_ref()),
            Event::GeneralRef(r) => {
                if let Some(c) = entity(r.as_ref()) {
                    text.push(c);
                }
            }
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                let name = local(e.name().as_ref());
                if let Some(d) = draft.as_mut() {
                    if let Some(field) = FIELDS.iter().find(|f| **f == name) {
                        let value = text.trim();
                        if !value.is_empty() {
                            d.fields.entry(field).or_insert_with(|| value.to_string());
                        }
                    } else if name == d.layer
                        && let Some(d) = draft.take()
                    {
                        match sheet_of(product, &d) {
                            Verdict::Keep(sheet) => read.sheets.push(sheet),
                            Verdict::Unreadable => {}
                            Verdict::Defective(why) => read.skipped.push(why),
                        }
                    }
                }
                if depth == 0 {
                    root_closed = true;
                }
                text.clear();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root_closed {
        return Err(http_error(
            url,
            "the answer ends before its feature collection closes",
        ));
    }
    Ok(read)
}

/// Whether `name` is `<prefix><year>`.
fn is_layer(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix)
        .is_some_and(|rest| rest.len() == 4 && rest.bytes().all(|b| b.is_ascii_digit()))
}

/// A collection that holds fewer features than matched has been cut short by the server; a
/// sheet missing from it could be the best one.
fn check_counts(root: &BytesStart<'_>, url: &str) -> Result<(), FetchError> {
    let number = |key| attribute(root, key).and_then(|v| v.parse::<u64>().ok());
    match (number("numberMatched"), number("numberReturned")) {
        (Some(matched), Some(returned)) if returned < matched => Err(http_error(
            url,
            format!("the index answered {returned} of {matched} sheets"),
        )),
        _ => Ok(()),
    }
}

/// What to do with one feature.
enum Verdict {
    Keep(Sheet),
    /// A sheet in a format or colour this program does not read; expected, not reported.
    Unreadable,
    /// A sheet whose entry is unusable, with the reason.
    Defective(String),
}

/// The pixel size in metres of `text`: `1.00 m`, `25.0 m` (terrain and surface models) or `0.25`
/// (orthophotos).
fn parse_pixel(text: &str) -> Option<f64> {
    let value: f64 = text.split_whitespace().next()?.parse().ok()?;
    (value.is_finite() && value > 0.0).then_some(value)
}

/// The corner of `N E` as `(north, east)`.
fn parse_corner(text: &str) -> Option<(f64, f64)> {
    let mut numbers = text.split_whitespace().map(str::parse::<f64>);
    let north = numbers.next()?.ok()?;
    let east = numbers.next()?.ok()?;
    (numbers.next().is_none() && north.is_finite() && east.is_finite()).then_some((north, east))
}

/// How a sheet was measured. The index says `Skaning laserowy` for LiDAR, and `Zdj. lotnicze` or
/// `Zdj. cyfrowe` for photogrammetry.
fn parse_source(text: &str) -> SheetSource {
    match text.trim().to_lowercase().as_str() {
        "skaning laserowy" => SheetSource::Lidar,
        "zdj. lotnicze" | "zdj. cyfrowe" => SheetSource::Photogrammetry,
        _ => SheetSource::Unknown,
    }
}

/// Whether the path of `url` (without query and fragment) ends in one of `extensions`, in any
/// case (`.ASC`).
fn has_extension(url: &str, extensions: &[&str]) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url).to_lowercase();
    extensions.iter().any(|e| path.ends_with(e))
}

fn sheet_of(product: Product, d: &Draft) -> Verdict {
    let field = |key: &str| d.fields.get(key).map(String::as_str);
    let godlo = field("godlo").unwrap_or_default();
    let name = |why: &str| {
        Verdict::Defective(if godlo.is_empty() {
            format!("a sheet of {} has {why}; skipped", d.layer)
        } else {
            format!("sheet {godlo} of {} has {why}; skipped", d.layer)
        })
    };
    if godlo.is_empty() {
        return name("no sheet number");
    }
    // The address first: whatever the sheet is, one that leads off the host is never used.
    let Some(url) = field("url_do_pobrania") else {
        return name("no download address");
    };
    if !host_of(url).is_ok_and(|host| host == DOWNLOAD_HOST) {
        return name(&format!(
            "a download address that is not https on {DOWNLOAD_HOST} ({})",
            one_line(url.split(['?', '#']).next().unwrap_or(url), 120)
        ));
    }
    // Formats and colours that are not read: a sheet the index lists correctly.
    let readable = match product {
        Product::Terrain | Product::Surface => has_extension(url, &[".asc"]),
        Product::Orthophoto => has_extension(url, &[".tif", ".tiff"]),
    };
    if !readable {
        return Verdict::Unreadable;
    }
    if product == Product::Orthophoto {
        // Never a false-colour (CIR) or black and white sheet as the orthophoto of a map.
        match field("kolor").map(str::to_uppercase).as_deref() {
            Some("RGB") => {}
            Some(_) => return Verdict::Unreadable,
            None => return name("no colour"),
        }
    }
    let year = field("akt_rok")
        .and_then(|y| y.parse::<u16>().ok())
        .or_else(|| d.layer.get(d.layer.len().saturating_sub(4)..)?.parse().ok());
    let Some(year) = year else {
        return name("no flight year");
    };
    let pixel = match product {
        Product::Orthophoto => field("piksel"),
        _ => field("char_przestrz"),
    }
    .and_then(parse_pixel);
    let Some(resolution_m) = pixel else {
        return name("no pixel size");
    };
    let corners = field("lowerCorner")
        .and_then(parse_corner)
        .zip(field("upperCorner").and_then(parse_corner));
    let Some(((min_n, min_e), (max_n, max_e))) = corners else {
        return name("no extent");
    };
    if min_n >= max_n || min_e >= max_e {
        return name("an empty extent");
    }
    Verdict::Keep(Sheet {
        godlo: godlo.to_string(),
        year,
        resolution_m,
        source: parse_source(field("zrodlo_danych").unwrap_or_default()),
        url: url.to_string(),
        bbox_2180: BoundingBox {
            min_e,
            min_n,
            max_e,
            max_n,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_adapter_api::policy::{PolicyFetcher, UrlPolicy};
    use freemapper_adapter_api::testing::MapFetcher;

    #[test]
    fn a_layer_name_is_the_prefix_and_exactly_four_digits() {
        assert!(is_layer("SkorowidzNMT2022", "SkorowidzNMT"));
        // The prefix alone, a letter among the digits, one digit too many or too few, another
        // prefix: none is a layer of the product.
        assert!(!is_layer("SkorowidzNMT", "SkorowidzNMT"));
        assert!(!is_layer("SkorowidzNMT20x2", "SkorowidzNMT"));
        assert!(!is_layer("SkorowidzNMT20222", "SkorowidzNMT"));
        assert!(!is_layer("SkorowidzNMT202", "SkorowidzNMT"));
        assert!(!is_layer("SkorowidzNMPT2022", "SkorowidzNMT"));
        // Four characters that are not all digits do not pass on the length alone.
        assert!(!is_layer("SkorowidzNMTabcd", "SkorowidzNMT"));
    }

    #[test]
    fn at_most_five_skipped_sheets_are_named_and_the_rest_is_counted() {
        let lines = |count: usize| {
            let skipped: Vec<String> = (0..count).map(|i| format!("sheet {i}")).collect();
            let mut lines = Vec::new();
            warn_about(&skipped, &mut |note| lines.push(note.to_string()));
            lines
        };
        assert!(lines(0).is_empty());
        assert_eq!(lines(4).len(), 4);
        // Exactly five are all named, with no line that counts the rest.
        let five = lines(5);
        assert_eq!(five.len(), 5, "{five:?}");
        assert!(five.iter().all(|l| !l.contains("more sheets")), "{five:?}");
        let nine = lines(9);
        assert_eq!(nine.len(), 6, "{nine:?}");
        assert!(nine[5].contains("4 more sheets"), "{nine:?}");
    }

    /// Only the `element`s of a `DescribeFeatureType` answer are layers; a type or an attribute
    /// with a name of the same shape is not, and a year below the oldest asked for is left out.
    #[test]
    fn the_layers_are_the_elements_of_the_description_and_nothing_else() {
        let xml = r#"<schema>
            <element name="SkorowidzNMT2017" type="x"/>
            <element name="SkorowidzNMT2020" type="x"/>
            <complexType name="SkorowidzNMT2019"/>
            <attribute name="SkorowidzNMT2021"/>
            <element name="SkorowidzNMT2022"></element>
        </schema>"#;
        let layers = read_layers(Product::Terrain, xml.as_bytes(), "u").unwrap();
        assert_eq!(
            layers,
            ["SkorowidzNMT2017", "SkorowidzNMT2020", "SkorowidzNMT2022"]
        );
        // The orthophoto starts with the year `MIN_ORTO_YEAR`; the flights before it are left out.
        let orto = format!(
            r#"<schema><element name="SkorowidzOrtofomapy{}"/><element name="SkorowidzOrtofomapy{}"/></schema>"#,
            MIN_ORTO_YEAR - 1,
            MIN_ORTO_YEAR
        );
        let layers = read_layers(Product::Orthophoto, orto.as_bytes(), "u").unwrap();
        assert_eq!(layers, [format!("SkorowidzOrtofomapy{MIN_ORTO_YEAR}")]);
    }

    /// An escaped character in a field (`&amp;` in an address) is read as the character.
    #[test]
    fn an_escaped_ampersand_in_a_field_is_read_as_an_ampersand() {
        let xml = NMT_SLUBICE.replacen(
            "73426_993517_N-33-126-C-c-3-4.asc",
            "73426_993517_N-33-126-C-c-3-4.asc?a=1&amp;b=2",
            1,
        );
        let read = read(Product::Terrain, &xml);
        assert!(
            read.sheets.iter().any(|s| s.url.ends_with(".asc?a=1&b=2")),
            "{:?}",
            read.sheets
        );
    }

    /// An answer that stops before the collection closes is an error, even after a whole sheet.
    #[test]
    fn an_answer_cut_off_after_a_sheet_is_an_error() {
        let end = NMT_SLUBICE.rfind("</wfs:member>").unwrap() + "</wfs:member>".len();
        let result = read_sheets(Product::Terrain, &NMT_SLUBICE.as_bytes()[..end], "u");
        assert!(result.is_err(), "{result:?}");
    }

    /// An extent that is empty on one axis only is empty.
    #[test]
    fn an_extent_empty_on_one_axis_is_an_empty_extent() {
        for (from, to) in [
            (
                "<gml:lowerCorner>505645.150000 195671.250000</gml:lowerCorner>",
                "<gml:lowerCorner>509000.000000 195671.250000</gml:lowerCorner>",
            ),
            (
                "<gml:lowerCorner>505645.150000 195671.250000</gml:lowerCorner>",
                "<gml:lowerCorner>505645.150000 199000.000000</gml:lowerCorner>",
            ),
        ] {
            let xml = NMT_SLUBICE.replace(from, to);
            let read = read(Product::Terrain, &xml);
            assert!(
                read.skipped
                    .iter()
                    .any(|why| why.contains("an empty extent")),
                "{to}: {:?}",
                read.skipped
            );
        }
    }

    #[test]
    fn a_corner_is_exactly_two_finite_numbers() {
        assert_eq!(
            parse_corner("505645.15 195671.25"),
            Some((505645.15, 195671.25))
        );
        assert_eq!(parse_corner("1"), None);
        assert_eq!(parse_corner("1 2 3"), None);
        assert_eq!(parse_corner("1 x"), None);
        assert_eq!(parse_corner("inf 5"), None);
        assert_eq!(parse_corner("5 inf"), None);
        assert_eq!(parse_corner("NaN 5"), None);
    }

    #[test]
    fn the_five_xml_entities_are_read_and_others_are_not() {
        for (name, ch) in [
            ("amp", '&'),
            ("lt", '<'),
            ("gt", '>'),
            ("quot", '"'),
            ("apos", '\''),
        ] {
            assert_eq!(entity(name), Some(ch), "{name}");
        }
        assert_eq!(entity("nbsp"), None);
        assert_eq!(entity(""), None);
    }

    const NMT_SLUBICE: &str = include_str!("../tests/fixtures/gugik/nmt_slubice.xml");
    const NMT_KRAKOW: &str = include_str!("../tests/fixtures/gugik/nmt_krakow.xml");
    const ORTO_SLUBICE: &str = include_str!("../tests/fixtures/gugik/orto_slubice_2022.xml");
    const EXCEPTION: &str = include_str!("../tests/fixtures/gugik/exception_report.xml");
    const DESCRIBE_NMT: &str = include_str!("../tests/fixtures/gugik/describe_nmt.xml");
    const DESCRIBE_ORTO: &str = include_str!("../tests/fixtures/gugik/describe_orto.xml");

    /// The box around Słubice (N 507596, E 197747) of the fixtures, in EPSG:2180.
    fn slubice() -> BoundingBox {
        BoundingBox {
            min_e: 197740.0,
            min_n: 507590.0,
            max_e: 197750.0,
            max_n: 507600.0,
        }
    }

    fn read(product: Product, xml: &str) -> Read {
        read_sheets(product, xml.as_bytes(), "https://test").unwrap()
    }

    #[test]
    fn the_box_is_sent_north_east() {
        let layers = [
            "SkorowidzNMT2020".to_string(),
            "SkorowidzNMT2022".to_string(),
        ];
        let url = sheets_url(Product::Terrain, &layers, &slubice());
        assert_eq!(
            url,
            "https://mapy.geoportal.gov.pl/wss/service/PZGIK/NumerycznyModelTerenuEVRF2007/WFS/Skorowidze\
             ?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature\
             &TYPENAMES=gugik:SkorowidzNMT2020,gugik:SkorowidzNMT2022\
             &BBOX=507590,197740,507600,197750,urn:ogc:def:crs:EPSG::2180"
        );
        // A box between whole metres grows outward.
        let b = BoundingBox {
            min_e: 197740.4,
            min_n: 507590.4,
            max_e: 197749.6,
            max_n: 507599.6,
        };
        assert!(
            sheets_url(Product::Terrain, &layers, &b)
                .ends_with("BBOX=507590,197740,507600,197750,urn:ogc:def:crs:EPSG::2180")
        );
    }

    /// The recorded requests of one `sheets` call: the layer list, then the sheets, both on the
    /// host of the index, the box north first.
    #[test]
    fn a_call_asks_the_layers_once_and_then_the_sheets() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&describe_url(Product::Terrain), DESCRIBE_NMT.into());
        fetcher.insert_prefix(
            "https://mapy.geoportal.gov.pl/wss/service/PZGIK/NumerycznyModelTerenuEVRF2007/WFS/Skorowidze?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature",
            NMT_SLUBICE.into(),
        );
        let index = GugikIndex::new();
        for _ in 0..2 {
            let sheets = index
                .sheets(&fetcher, Product::Terrain, &slubice(), &mut |_| {})
                .unwrap();
            assert_eq!(sheets.len(), 3);
        }
        let log = fetcher.log.borrow();
        assert_eq!(log.len(), 3, "{log:?}");
        assert!(log[0].ends_with("REQUEST=DescribeFeatureType"));
        assert!(log[1].contains(
            "&TYPENAMES=gugik:SkorowidzNMT2018,gugik:SkorowidzNMT2019,gugik:SkorowidzNMT2020,\
             gugik:SkorowidzNMT2021,gugik:SkorowidzNMT2022,gugik:SkorowidzNMT2023,\
             gugik:SkorowidzNMT2024,gugik:SkorowidzNMT2025,gugik:SkorowidzNMT2026&BBOX=507590,197740,507600,197750,urn:ogc:def:crs:EPSG::2180"
        ));
        assert_eq!(log[1], log[2]);
    }

    /// The layer lists of the fixtures: every year of the terrain model, and only the years of
    /// the orthophoto from the year it is asked for.
    #[test]
    fn the_layers_come_from_the_service() {
        let nmt = read_layers(Product::Terrain, DESCRIBE_NMT.as_bytes(), "u").unwrap();
        assert_eq!(nmt.first().unwrap(), "SkorowidzNMT2018");
        assert_eq!(nmt.last().unwrap(), "SkorowidzNMT2026");
        assert_eq!(nmt.len(), 9);
        let orto = read_layers(Product::Orthophoto, DESCRIBE_ORTO.as_bytes(), "u").unwrap();
        assert_eq!(orto.first().unwrap(), "SkorowidzOrtofomapy2000");
        assert_eq!(orto.last().unwrap(), "SkorowidzOrtofomapy2026");
        assert!(
            !orto
                .iter()
                .any(|l| l.ends_with("1999") || l.ends_with("1957"))
        );
        assert!(
            read_layers(Product::Surface, DESCRIBE_NMT.as_bytes(), "u").is_err(),
            "a list without the layers of the product is an error"
        );
    }

    /// The Słubice sheet of 2020 and 2022 as the live index delivers them: the layers answer in
    /// a collection inside a collection, and the extent is written north, east.
    #[test]
    fn the_slubice_fixture_is_read_with_its_values() {
        let read = read(Product::Terrain, NMT_SLUBICE);
        assert!(read.skipped.is_empty(), "{:?}", read.skipped);
        // Four sheets listed; the 1 m 2020 one that is only an `.xyz.zip` is not read.
        let key = |s: &Sheet| (s.year, s.resolution_m, s.source);
        let got: Vec<_> = read.sheets.iter().map(key).collect();
        assert_eq!(
            got,
            [
                (2020, 1.0, SheetSource::Lidar),
                (2022, 1.0, SheetSource::Photogrammetry),
                (2022, 5.0, SheetSource::Photogrammetry),
            ]
        );
        let first = &read.sheets[0];
        assert_eq!(first.godlo, "N-33-126-C-c-3-4");
        assert_eq!(
            first.url,
            "https://opendata.geoportal.gov.pl/NumDaneWys/NMT/73426/73426_993517_N-33-126-C-c-3-4.asc"
        );
        // The corners are `N E`: the north is the larger number here (505645 against 195671).
        assert_eq!(
            first.bbox_2180,
            BoundingBox {
                min_e: 195671.25,
                min_n: 505645.15,
                max_e: 197940.55,
                max_n: 508090.93,
            }
        );
        // The place of the test lies inside its sheet.
        let b = &first.bbox_2180;
        assert!((b.min_e..=b.max_e).contains(&197747.0) && (b.min_n..=b.max_n).contains(&507596.0));
        // `.ASC` in capitals is read too.
        assert!(read.sheets[2].url.ends_with(".ASC"));
    }

    #[test]
    fn a_colour_infrared_orthophoto_is_never_taken() {
        let read = read(Product::Orthophoto, ORTO_SLUBICE);
        assert!(read.skipped.is_empty(), "{:?}", read.skipped);
        // Listed are 0.05 m RGB, 0.25 m CIR and 0.25 m RGB of one sheet number and year.
        let got: Vec<_> = read.sheets.iter().map(|s| s.resolution_m).collect();
        assert_eq!(got, [0.05, 0.25]);
        assert!(
            read.sheets.iter().all(|s| !s.url.contains("76965_1160663")),
            "the CIR sheet is among them"
        );
        let rgb = &read.sheets[1];
        assert_eq!(rgb.year, 2022);
        assert_eq!(rgb.source, SheetSource::Photogrammetry);
        assert!(rgb.url.ends_with(".tif"));
        // Every way a colour can read: only RGB passes (the CIR sheet takes the colour).
        for (colour, kept) in [("CIR", 2), ("B/W", 2), ("rgb", 3), ("RGB", 3)] {
            let changed =
                ORTO_SLUBICE.replace("<gugik:kolor>CIR<", &format!("<gugik:kolor>{colour}<"));
            assert_eq!(
                self::read(Product::Orthophoto, &changed).sheets.len(),
                kept,
                "{colour}"
            );
        }
        // No colour at all is no guess: the sheet is skipped with a warning.
        let blind = ORTO_SLUBICE.replace("<gugik:kolor>RGB</gugik:kolor>", "");
        let blind = self::read(Product::Orthophoto, &blind);
        assert!(blind.sheets.is_empty());
        assert_eq!(blind.skipped.len(), 2, "{:?}", blind.skipped);
    }

    /// Kraków: the sheet number of the old numbering has dots, laser scanning is read, and the
    /// 25 m and 5 m models that are only published as archives are not taken.
    #[test]
    fn a_sheet_of_the_old_numbering_and_the_archives_are_read() {
        let read = read(Product::Terrain, NMT_KRAKOW);
        assert!(read.skipped.is_empty(), "{:?}", read.skipped);
        assert_eq!(read.sheets.len(), 1);
        let sheet = &read.sheets[0];
        assert_eq!(sheet.godlo, "7.125.11.08");
        assert_eq!(sheet.year, 2021);
        assert_eq!(sheet.resolution_m, 1.0);
        assert_eq!(sheet.source, SheetSource::Lidar);
    }

    /// `25.0 m` is a pixel size, and `Skaning laserowy` is LiDAR, also where the file is read.
    #[test]
    fn a_25_metre_model_and_the_laser_source_are_read() {
        assert_eq!(parse_pixel("25.0 m"), Some(25.0));
        assert_eq!(parse_pixel("1.00 m"), Some(1.0));
        assert_eq!(parse_pixel("0.05"), Some(0.05));
        assert_eq!(parse_pixel("m"), None);
        assert_eq!(parse_pixel("0"), None);
        assert_eq!(parse_pixel(""), None);
        assert_eq!(parse_source("Skaning laserowy"), SheetSource::Lidar);
        assert_eq!(parse_source("Zdj. lotnicze"), SheetSource::Photogrammetry);
        assert_eq!(parse_source("Zdj. cyfrowe"), SheetSource::Photogrammetry);
        assert_eq!(parse_source("something new"), SheetSource::Unknown);
        // The 25 m sheet of Kraków, were it an `.asc`, comes out with 25 m.
        let as_asc = NMT_KRAKOW.replace("_asc.zip", ".asc");
        let read = read(Product::Terrain, &as_asc);
        let coarse = read
            .sheets
            .iter()
            .find(|s| s.resolution_m == 25.0)
            .expect("the 25 m sheet");
        assert_eq!(coarse.year, 2018);
        assert_eq!(coarse.source, SheetSource::Photogrammetry);
    }

    #[test]
    fn an_exception_report_is_an_error() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&describe_url(Product::Terrain), DESCRIBE_NMT.into());
        fetcher.insert_prefix("https://mapy.geoportal.gov.pl/", EXCEPTION.into());
        fetcher.insert(&describe_url(Product::Terrain), DESCRIBE_NMT.into());
        let result = GugikIndex::new().sheets(&fetcher, Product::Terrain, &slubice(), &mut |_| {});
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("InvalidParameterValue") && message.contains("SkorowidzNMT2099"),
            "{message}"
        );
        // The service says so with status 200, so the answer is asked again before it fails.
        assert_eq!(fetcher.log.borrow().len(), 1 + 3);
        let direct = read_sheets(Product::Terrain, EXCEPTION.as_bytes(), "u");
        assert!(direct.is_err());
        let layers = read_layers(Product::Terrain, EXCEPTION.as_bytes(), "u");
        assert!(layers.is_err());
    }

    #[test]
    fn a_broken_answer_is_an_error() {
        let cut = &NMT_SLUBICE[..NMT_SLUBICE.len() / 2];
        for (what, body) in [
            ("truncated", cut),
            ("no collection", "<html><body>busy</body></html>"),
            ("empty", ""),
        ] {
            assert!(
                read_sheets(Product::Terrain, body.as_bytes(), "u").is_err(),
                "{what}"
            );
        }
        // Fewer features than matched: the server cut the answer short.
        let short = NMT_SLUBICE.replacen("numberMatched=\"4\"", "numberMatched=\"9\"", 1);
        let error = read_sheets(Product::Terrain, short.as_bytes(), "u").unwrap_err();
        assert!(error.to_string().contains("4 of 9"), "{error}");
    }

    /// One bad entry costs that sheet, not the answer: a sheet that lacks its number, one whose
    /// address leads to another host, one on plain http, one without pixel size or extent.
    #[test]
    fn a_defective_sheet_is_skipped_and_the_others_stay() {
        let good = "https://opendata.geoportal.gov.pl/NumDaneWys/NMT/";
        assert!(NMT_SLUBICE.contains(good));
        // Break the three `.asc` sheets of the fixture one after the other, and let one stay.
        let mut xml = NMT_SLUBICE.to_string();
        xml = xml.replacen(
            "<gugik:url_do_pobrania>https://opendata.geoportal.gov.pl/NumDaneWys/NMT/73426/",
            "<gugik:url_do_pobrania>https://evil.example.com/NumDaneWys/NMT/73426/",
            1,
        );
        xml = xml.replacen(
            "<gugik:url_do_pobrania>https://opendata.geoportal.gov.pl/NumDaneWys/NMT/76153/",
            "<gugik:url_do_pobrania>http://opendata.geoportal.gov.pl/NumDaneWys/NMT/76153/",
            1,
        );
        let read = read(Product::Terrain, &xml);
        assert_eq!(read.sheets.len(), 1, "{:?}", read.sheets);
        assert_eq!(read.sheets[0].resolution_m, 5.0);
        assert_eq!(read.skipped.len(), 2, "{:?}", read.skipped);
        assert!(
            read.skipped[0].contains("evil.example.com") || read.skipped[0].contains("not https")
        );
        assert!(read.skipped.iter().all(|s| s.contains("N-33-126-C-c-3-4")));
        // A host that merely ends like the right one, one with user info, one on another port.
        for url in [
            "https://opendata.geoportal.gov.pl.evil.example/x.asc",
            "https://evilopendata.geoportal.gov.pl/x.asc",
            "https://opendata.geoportal.gov.pl@evil.example/x.asc",
            "https://opendata.geoportal.gov.pl:8443/x.asc",
            "https://127.0.0.1/x.asc",
        ] {
            let xml = NMT_SLUBICE.replacen(
                "https://opendata.geoportal.gov.pl/NumDaneWys/NMT/73426/73426_993517_N-33-126-C-c-3-4.asc",
                url,
                1,
            );
            let read = self::read(Product::Terrain, &xml);
            assert_eq!(read.skipped.len(), 1, "{url}: {:?}", read.skipped);
            assert!(read.sheets.iter().all(|s| !s.url.contains("evil")), "{url}");
        }
        // Lacking pieces, cut out of the sheet of the 2020 laser scan (the `.asc` before the
        // address that ends in 73426_993517): that sheet is skipped and the other two stay.
        let cut_from_first_asc = |tag: &str| {
            let mark = NMT_SLUBICE.find("73426_993517").unwrap();
            let open = format!("<gugik:{tag}>");
            let start = NMT_SLUBICE[..mark].rfind(&open).unwrap();
            let end = start + NMT_SLUBICE[start..].find("</").unwrap();
            let end = end + NMT_SLUBICE[end..].find('>').unwrap() + 1;
            format!("{}{}", &NMT_SLUBICE[..start], &NMT_SLUBICE[end..])
        };
        for (tag, reason) in [
            ("godlo", "no sheet number"),
            ("char_przestrz", "no pixel size"),
            ("akt_rok", "no flight year"),
        ] {
            let read = self::read(Product::Terrain, &cut_from_first_asc(tag));
            // The flight year falls back to the year of the layer, so that sheet survives.
            let (sheets, skipped) = if tag == "akt_rok" { (3, 0) } else { (2, 1) };
            assert_eq!(read.sheets.len(), sheets, "{tag}: {:?}", read.skipped);
            assert_eq!(read.skipped.len(), skipped, "{tag}");
            if let Some(why) = read.skipped.first() {
                assert!(why.contains(reason), "{why}");
            }
        }
        let xml = NMT_SLUBICE.replacen(
            "<gml:lowerCorner>505645.150000 195671.250000</gml:lowerCorner>",
            "",
            usize::MAX,
        );
        let read = self::read(Product::Terrain, &xml);
        assert!(read.sheets.is_empty());
        assert_eq!(read.skipped.len(), 3);
    }

    /// The warnings of a call reach `progress`, five by name and the rest counted.
    #[test]
    fn the_warnings_of_a_call_are_capped() {
        let fetcher = MapFetcher::default();
        fetcher.insert(&describe_url(Product::Terrain), DESCRIBE_NMT.into());
        let broken = NMT_SLUBICE.replace(
            "https://opendata.geoportal.gov.pl/",
            "https://evil.example.com/",
        );
        fetcher.insert_prefix("https://mapy.geoportal.gov.pl/", broken.into());
        let mut lines = Vec::new();
        let sheets = GugikIndex::new()
            .sheets(&fetcher, Product::Terrain, &slubice(), &mut |n| {
                lines.push(n.to_string())
            })
            .unwrap();
        assert!(sheets.is_empty());
        // Four sheets in the fixture, three of them `.asc`; the archive is passed over quietly.
        assert_eq!(lines.len(), 4, "{lines:?}");
        assert!(
            lines.iter().all(|l| l.contains("GUGiK sheet index")),
            "{lines:?}"
        );
    }

    /// Every request of the client lies on the host of the index, and the adapter's policy, which
    /// names the two hosts of GUGiK, lets exactly those through.
    #[test]
    fn every_request_stays_on_the_hosts_of_the_policy() {
        let policy = UrlPolicy::new([INDEX_HOST, DOWNLOAD_HOST]).unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert(&describe_url(Product::Orthophoto), DESCRIBE_ORTO.into());
        fetcher.insert_prefix("https://mapy.geoportal.gov.pl/", ORTO_SLUBICE.into());
        let guarded = PolicyFetcher::new(&fetcher, &policy);
        let sheets = GugikIndex::new()
            .sheets(&guarded, Product::Orthophoto, &slubice(), &mut |_| {})
            .unwrap();
        assert_eq!(sheets.len(), 2);
        let log = fetcher.log.borrow();
        assert_eq!(log.len(), 2);
        for url in log.iter() {
            assert_eq!(host_of(url).unwrap(), INDEX_HOST, "{url}");
            assert!(policy.check_url(url).is_ok());
        }
        // The sheets themselves are addressed on the download host, and nowhere else.
        assert!(
            sheets
                .iter()
                .all(|s| host_of(&s.url).unwrap() == DOWNLOAD_HOST)
        );
        // A policy without the index host refuses the very first request.
        let narrow = UrlPolicy::new([DOWNLOAD_HOST]).unwrap();
        let blocked = PolicyFetcher::new(&fetcher, &narrow);
        let result =
            GugikIndex::new().sheets(&blocked, Product::Orthophoto, &slubice(), &mut |_| {});
        assert!(result.is_err());
        assert_eq!(fetcher.log.borrow().len(), 2, "nothing was sent");
    }

    /// The index of GUGiK, asked live through the policy of the adapter with the fixed
    /// User-Agent of the program:
    /// `cargo test -p freemapper-adapters-pl live_ -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn live_index_lists_the_sheets_of_slubice() {
        use crate::polska::Polska;
        use crate::sheets::best_per_cell;
        use freemapper::fetch::HttpFetcher;
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        assert!(!freemapper::fetch::USER_AGENT.contains('@'));
        let adapter = Polska::new(&[]);
        let http = HttpFetcher::new();
        let fetcher = PolicyFetcher::new(&http, adapter.policy().unwrap());
        let index = GugikIndex::new();
        let area = slubice();
        let mut warnings = Vec::new();
        let mut ask = |product| {
            index
                .sheets(&fetcher, product, &area, &mut |n| {
                    warnings.push(n.to_string())
                })
                .unwrap()
        };
        let nmt = ask(Product::Terrain);
        let nmpt = ask(Product::Surface);
        let orto = ask(Product::Orthophoto);
        assert!(!nmt.is_empty() && !nmpt.is_empty() && !orto.is_empty());
        let best = best_per_cell(&nmt, 0.0);
        assert_eq!(best.len(), 1, "{best:?}");
        assert_eq!(best[0].godlo, "N-33-126-C-c-3-4");
        assert!(best[0].resolution_m <= 1.0);
        for sheet in nmt.iter().chain(&nmpt).chain(&orto) {
            assert_eq!(host_of(&sheet.url).unwrap(), DOWNLOAD_HOST);
        }
        let photo = best_per_cell(&orto, freemapper_sources::orto::FINEST_PIXEL_M);
        // Another numbering (`5.176.17.3`, 0.5 m) covers the place as well; the sheet of the
        // numbering of the grid is among the choices, in 0.25 m and never in 0.05 m or CIR.
        let sheet = photo
            .iter()
            .find(|s| s.godlo == "N-33-126-C-c-3-4")
            .expect("the sheet of the grid");
        assert_eq!(sheet.resolution_m, 0.25);
        assert!(sheet.url.ends_with(".tif"));
        println!("{best:?}\n{photo:?}\n{warnings:?}");
    }
}
