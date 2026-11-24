// SPDX-License-Identifier: AGPL-3.0-or-later

//! BDOT10k, the topographic object database of GUGiK, as the fallback where the cadastre EGiB
//! ([`crate::egib`]) gives nothing (ADR 0178). Some municipalities serve no cadastre at all
//! (the national WFS then answers with zero features), and the national service has no land
//! use class for most parcels of the big cities.
//!
//! There is no WFS of BDOT10k that cuts out an area. The WFS `PobieranieBDOT10k` lists the
//! download packages, one per powiat (`ms:BDOT10k_powiaty`, 380 of them), and takes a `BBOX`
//! (north first, like every EPSG:2180 box of GUGiK). So the client
//!
//! 1. asks the index which powiats the map area touches (a 1.5 KB answer),
//! 2. downloads the GML archive of each into the shared cache (`gugik/bdot10k/<teryt>_<date>.zip`,
//!    7 to 89 MB, measured on 22 November 2026), and
//! 3. reads from it only the object classes it maps, streaming, and keeps what lies in the box.
//!
//! The GML is GML 3.2 in EPSG:2180, a polygon as a `posList` of **E N** pairs (EGiB has `N E`).
//! One file holds one class (`OT_BUBD_A`, `OT_PTLZ_A`, ...), so a class that has no object has a
//! file of 454 bytes.
//!
//! What is mapped, and what is left out and why, is in [`building_letter`] and
//! [`land_use_of`]. Roads (`SKDR_L`, `SKJZ_L`) and tracks (`SKTR_L`) are lines without a width,
//! so they give no area to paint; OSM has them. The hosts are the two of GUGiK the adapter
//! already asks ([`crate::gugik_index::INDEX_HOST`], [`crate::gugik_index::DOWNLOAD_HOST`]).

use freemapper_adapter_api::fetch::download_with_retries;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use freemapper_adapter_api::alkis::AlkisPart;
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::fetch::{FetchMode, Fetcher, get_with_retries, is_ogc_report};
use freemapper_adapter_api::note;
use freemapper_adapter_api::note::{Arg, Note};
use freemapper_adapter_api::policy::host_of;
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_sources::zip_range::{find_member, read_member_up_to, read_members};
use geo::{Contains, InteriorPoint};
use geo_types::{Coord, LineString, Polygon};
use quick_xml::events::Event;
use serde_json::{Value, json};

use crate::egib::{self, Feature};
use crate::gugik_index::{DOWNLOAD_HOST, entity, http_error, local, report_error};
use crate::rasters::PROVIDER;

/// The WFS that lists the packages.
pub const INDEX_ENDPOINT: &str =
    "https://mapy.geoportal.gov.pl/wss/service/PZGIK/BDOT/WFS/PobieranieBDOT10k";

/// The most an archive may weigh, in megabytes of a million bytes. The largest powiat measured
/// is Warszawa with 89 MB.
pub const ARCHIVE_CAP_MB: u64 = 150;

/// Below this share of the map area covered by parcels with a land use class, EGiB counts as
/// empty for land use and BDOT10k fills in. Measured on 22 November 2026 (`docs/regions/poland-reference.md`).
pub const MIN_LAND_USE_SHARE: f64 = 0.25;

/// The most a map area may touch: a box of a few square kilometres lies in one powiat, rarely
/// in two or three. More is a box that is not a map area.
pub(crate) const MAX_PACKAGES: usize = 6;

/// The most one class file may hold unpacked. The largest measured is 136 MB (`OT_RTLW_L`, which
/// the client never reads); the buildings of Warszawa are the largest class it reads.
const MAX_ENTRY_BYTES: u64 = 1 << 30;

/// The entries of an archive at most. One powiat holds 75.
const MAX_ENTRIES: usize = 1000;

/// The classes read for [`Gap::Buildings`].
pub const BUILDING_CLASSES: [&str; 1] = ["BUBD_A"];

/// The classes read for [`Gap::LandUse`].
pub const LAND_USE_CLASSES: [&str; 10] = [
    "PTLZ_A", "PTTR_A", "PTUT_A", "PTWP_A", "PTZB_A", "PTNZ_A", "PTPL_A", "KUKO_A", "KUSC_A",
    "KUSK_A",
];

/// The class read for [`Gap::Cemeteries`].
pub const CEMETERY_CLASSES: [&str; 1] = ["KUSC_A"];

/// What a cemetery file may hold unpacked. The largest measured is 127 KB (Poznań, 82 objects).
const CEMETERY_MEMBER_BYTES: u64 = 64 << 20;

/// What EGiB left empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    Buildings,
    LandUse,
    /// Not a gap: the cemeteries that lie on parcels EGiB gives no land use (ADR 0194).
    Cemeteries,
}

impl Gap {
    fn classes(self) -> &'static [&'static str] {
        match self {
            Gap::Buildings => &BUILDING_CLASSES,
            Gap::LandUse => &LAND_USE_CLASSES,
            Gap::Cemeteries => &CEMETERY_CLASSES,
        }
    }

    /// The thing as a word of a message, in both program languages.
    fn words(self) -> Arg {
        match self {
            Gap::Buildings => Arg::Words {
                en: "buildings".into(),
                de: "Gebäude".into(),
            },
            Gap::LandUse => Arg::Words {
                en: "land use".into(),
                de: "Landnutzung".into(),
            },
            Gap::Cemeteries => Arg::Words {
                en: "cemeteries".into(),
                de: "Friedhöfe".into(),
            },
        }
    }
}

/// A download package of the index: one powiat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// The four-digit code of the powiat.
    pub teryt: String,
    pub name: String,
    /// The date of the last update as the index states it (`2024-04-04`).
    pub updated: String,
    pub url: String,
}

/// The request for the packages that touch `bbox` (EPSG:2180). The properties are named: without
/// `PROPERTYNAME` every answer carries the outline of the powiat and weighs 63 MB.
pub fn index_url(bbox: &BoundingBox) -> String {
    let crs = "urn:ogc:def:crs:EPSG::2180";
    format!(
        "{INDEX_ENDPOINT}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES=ms:BDOT10k_powiaty\
         &PROPERTYNAME=TERYT,NAZWA_POWIATU,URL_GML,Data_aktualizacji&BBOX={},{},{},{},{crs}",
        bbox.min_n.floor(),
        bbox.min_e.floor(),
        bbox.max_n.ceil(),
        bbox.max_e.ceil(),
    )
}

/// The packages in an answer of the index. A package whose code is not four digits, or whose
/// address is not the file of that code on the download host, is left out (the addresses come
/// from a server): an answer with packages but none usable is an error.
pub fn read_index(body: &[u8], url: &str) -> Result<Vec<Package>, FetchError> {
    if is_ogc_report(body) {
        return Err(report_error(url, body));
    }
    let mut reader = quick_xml::Reader::from_reader(body);
    let mut buffer = Vec::new();
    let (mut depth, mut closed) = (0_usize, false);
    let mut text = String::new();
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let (mut seen, mut packages) = (0_usize, Vec::new());
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|e| http_error(url, format!("the answer is no XML: {e}")))?;
        match event {
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                if depth == 0 && name != "FeatureCollection" {
                    return Err(http_error(
                        url,
                        format!("the answer is no feature collection but <{name}>"),
                    ));
                }
                if name == "BDOT10k_powiaty" {
                    fields.clear();
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
                match name.as_str() {
                    "TERYT" | "NAZWA_POWIATU" | "URL_GML" | "Data_aktualizacji" => {
                        fields.insert(name, text.trim().to_string());
                    }
                    "BDOT10k_powiaty" => {
                        seen += 1;
                        packages.extend(package(&fields));
                    }
                    _ => {}
                }
                if depth == 0 {
                    closed = true;
                }
                text.clear();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !closed {
        return Err(http_error(
            url,
            "the answer ends before its feature collection closes",
        ));
    }
    if seen > 0 && packages.is_empty() {
        return Err(http_error(
            url,
            "the index names packages but none with a usable code and address",
        ));
    }
    Ok(packages)
}

/// A package from the fields of one entry, `None` when its code or address is not as expected.
fn package(fields: &BTreeMap<String, String>) -> Option<Package> {
    let teryt = fields.get("TERYT")?;
    let url = fields.get("URL_GML")?;
    let code_ok = teryt.len() == 4 && teryt.bytes().all(|b| b.is_ascii_digit());
    let address_ok = host_of(url).is_ok_and(|host| host == DOWNLOAD_HOST)
        && url.starts_with("https://")
        && !url.contains("..")
        && url.starts_with("https://opendata.geoportal.gov.pl/bdot10k/")
        && url.ends_with(&format!("/{teryt}_GML.zip"));
    if !code_ok || !address_ok {
        return None;
    }
    let updated = fields.get("Data_aktualizacji").map_or("", |d| d.as_str());
    let date = updated.get(..10).unwrap_or_default();
    let date_ok = date.len() == 10
        && date.chars().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    Some(Package {
        teryt: teryt.clone(),
        name: fields.get("NAZWA_POWIATU").cloned().unwrap_or_default(),
        updated: if date_ok { date } else { "unknown" }.to_string(),
        url: url.clone(),
    })
}

/// An object of one class file: the values the mapping reads, and its polygons as
/// `(north, east)` points in EPSG:2180 (the order [`Feature`] has).
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub class: String,
    /// The `lokalnyId`.
    pub id: String,
    pub fields: BTreeMap<String, String>,
    pub polygons: Vec<Vec<Vec<(f64, f64)>>>,
}

impl Object {
    fn field(&self, name: &str) -> &str {
        self.fields.get(name).map_or("", String::as_str)
    }
}

/// The elements of an object the mapping reads (local names).
const FIELDS: [&str; 6] = [
    "lokalnyId",
    "identyfikatorEGiB",
    "kodKst",
    "kategoriaIstnienia",
    "rodzaj",
    "materialNawierzchni",
];

/// The objects of one class file that touch `bbox` (EPSG:2180), streamed. `source` names the
/// file in errors. An answer that is no GML, one with a coordinate list that is no pairs of
/// numbers and one that ends before its collection closes are errors.
pub fn read_class(
    input: impl std::io::BufRead,
    class: &str,
    bbox: &BoundingBox,
    source: &str,
) -> Result<Vec<Object>, FetchError> {
    let element = format!("OT_{class}");
    let mut reader = quick_xml::Reader::from_reader(input);
    let mut buffer = Vec::new();
    let (mut depth, mut closed) = (0_usize, false);
    let mut text = String::new();
    let mut draft: Option<Object> = None;
    let mut ring: Option<Vec<(f64, f64)>> = None;
    let mut rings: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut objects = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|e| http_error(source, format!("the file is no XML: {e}")))?;
        match event {
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                if depth == 0 {
                    if name != "FeatureCollection" {
                        return Err(http_error(
                            source,
                            format!("the file is no feature collection but <{name}>"),
                        ));
                    }
                } else if name == element {
                    draft = Some(Object {
                        class: class.to_string(),
                        id: String::new(),
                        fields: BTreeMap::new(),
                        polygons: Vec::new(),
                    });
                } else if draft.is_some() && name == "Polygon" {
                    rings.clear();
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
                    if FIELDS.contains(&name.as_str()) {
                        d.fields.insert(name.clone(), text.trim().to_string());
                    }
                    if name == "posList" {
                        ring = Some(read_ring(&text, source)?);
                    } else if name == "LinearRing" {
                        rings.extend(ring.take());
                    } else if name == "Polygon" && !rings.is_empty() {
                        d.polygons.push(std::mem::take(&mut rings));
                    } else if name == element
                        && let Some(mut done) = draft.take()
                    {
                        done.id = done.field("lokalnyId").to_string();
                        if touches(&done, bbox) {
                            objects.push(done);
                        }
                    }
                }
                if depth == 0 {
                    closed = true;
                }
                text.clear();
            }
            // A root without content (`<gml:FeatureCollection/>`) is complete.
            Event::Empty(e) if depth == 0 => {
                let name = local(e.name().as_ref());
                if name != "FeatureCollection" {
                    return Err(http_error(
                        source,
                        format!("the file is no feature collection but <{name}>"),
                    ));
                }
                closed = true;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !closed {
        return Err(http_error(
            source,
            "the file ends before its feature collection closes",
        ));
    }
    Ok(objects)
}

/// The points of a `posList`: pairs of `E N`, returned as `(north, east)`.
fn read_ring(text: &str, source: &str) -> Result<Vec<(f64, f64)>, FetchError> {
    let numbers = text
        .split_whitespace()
        .map(|n| n.parse::<f64>().ok().filter(|v| v.is_finite()))
        .collect::<Option<Vec<_>>>()
        .filter(|n| n.len() % 2 == 0)
        .ok_or_else(|| http_error(source, "a coordinate list is not pairs of numbers"))?;
    Ok(numbers
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[1], p[0]))
        .collect())
}

/// Whether the envelope of the object's points meets `bbox`, edges included.
fn touches(object: &Object, bbox: &BoundingBox) -> bool {
    let mut points = object.polygons.iter().flatten().flatten().peekable();
    if points.peek().is_none() {
        return false;
    }
    let (mut min_n, mut max_n, mut min_e, mut max_e) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &(n, e) in points {
        min_n = min_n.min(n);
        max_n = max_n.max(n);
        min_e = min_e.min(e);
        max_e = max_e.max(e);
    }
    max_e >= bbox.min_e && min_e <= bbox.max_e && max_n >= bbox.min_n && min_n <= bbox.max_n
}

/// The objects of `classes` in the archive `path` that touch `bbox`. A class the archive has no
/// file for is empty. An entry that unpacks to more than `MAX_ENTRY_BYTES` is
/// [`FetchError::TooLarge`], and a file that is no archive is an error.
pub fn read_archive(
    path: &Path,
    classes: &[&str],
    bbox: &BoundingBox,
) -> Result<Vec<Object>, FetchError> {
    read_archive_capped(path, classes, bbox, MAX_ENTRY_BYTES)
}

fn read_archive_capped(
    path: &Path,
    classes: &[&str],
    bbox: &BoundingBox,
    cap: u64,
) -> Result<Vec<Object>, FetchError> {
    let source = path.display().to_string();
    let file = File::open(path).map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .map_err(|e| http_error(&source, format!("the download is no archive: {e}")))?;
    if archive.len() > MAX_ENTRIES {
        return Err(http_error(
            &source,
            format!("the archive holds more than {MAX_ENTRIES} files"),
        ));
    }
    let names: Vec<String> = archive
        .file_names()
        .filter_map(Result::ok)
        .map(std::borrow::Cow::into_owned)
        .collect();
    let mut found = Vec::new();
    for class in classes {
        let suffix = format!("__OT_{class}.xml");
        let Some(name) = names.iter().find(|n| n.ends_with(&suffix)) else {
            continue;
        };
        let entry = archive
            .by_name(name)
            .map_err(|e| http_error(&source, format!("{name} cannot be read: {e}")))?;
        if entry.size() > cap {
            return Err(freemapper_adapter_api::fetch::too_large(&source, cap));
        }
        // The declared size is the server's word: stop reading at the cap anyway.
        let limited = BufReader::new(entry.take(cap + 1));
        found.extend(read_class(limited, class, bbox, name)?);
    }
    Ok(found)
}

/// The letter of the register's building kind (the `RODZAJ` of EGiB) that a KŚT code of
/// `kodKst` stands for. The pairs were checked against the 21 940 buildings of the powiat of
/// Słubice (22 November 2026): the count of each code equals the count of the general function
/// it names (`funkcjaOgolnaBudynku`), and the functions are the kinds of EGiB word for word
/// (101 industrial, 102 transport and communication, 103 commercial and services, 104 tanks,
/// silos and storage, 105 office, 106 hospital, 107 education, science, culture and sport, 108
/// production, services and farm buildings, 109 other non-residential, 110 residential). `None`
/// for a code that is none of them.
pub fn building_letter(kod_kst: &str) -> Option<&'static str> {
    match kod_kst.trim() {
        "101" => Some("p"),
        "102" => Some("t"),
        "103" => Some("h"),
        "104" => Some("s"),
        "105" => Some("b"),
        "106" => Some("z"),
        "107" => Some("k"),
        "108" => Some("g"),
        "109" => Some("i"),
        "110" => Some("m"),
        _ => None,
    }
}

/// The land use of an object of a land cover class, as `(nutzart, bez)` in the LGB spellings
/// the engine reads and the same ones [`egib::land_use_of_class`] gives. `None`: the object is
/// left out.
///
/// Mapped (the values are those of the powiat of Słubice, checked on 22 November 2026):
/// - `PTLZ_A` forest: `las` and `zagajnik` (grove) `Wald`, `zadrzewienie` (tree stand) `Gehölz`.
/// - `PTTR_A`: `roślinność trawiasta` `Landwirtschaft / Grünland`, `uprawa na gruntach ornych`
///   `Landwirtschaft`.
/// - `PTUT_A` permanent crops: orchard, nursery and plantation `Landwirtschaft`; allotment
///   gardens `Sport-, Freizeit- und Erholungsfläche / Kleingarten` (private, ADR 0049).
/// - `PTWP_A` water: `woda stojąca` `Stehendes Gewässer`, `woda płynąca` `Fließgewässer`.
/// - `PTZB_A` built-up land: single and multi-family housing `Wohnbaufläche`, industrial and
///   storage `Industrie- und Gewerbefläche`; commercial and "other" left out, as EGiB leaves
///   out `Bi`.
/// - `PTNZ_A`: industrial and storage land `Industrie- und Gewerbefläche`; land under
///   technical installations (solar farms, masts) left out.
/// - `PTPL_A` squares: `Platz` unless the surface is bare ground, gravel or crushed stone,
///   which is no paving.
/// - `KUKO_A`: `parking` `Platz / Parkplatz`; filling stations, stations and ports are left out.
/// - `KUSC_A`: cemeteries (`cmentarz ...`) `Friedhof`; churches and monasteries left out.
/// - `KUSK_A`: `park lub skwer` `Sport-, Freizeit- und Erholungsfläche / Grünanlage`.
///
/// Left out entirely: bushes (`PTRK_A`), sand and gravel pits (`PTWZ_A`, `PTGN_A`), wetlands
/// (`OIMK_A`, which overlaps forest and grass), waste land and roads and tracks (lines).
pub fn land_use_of(object: &Object) -> Option<(&'static str, Option<&'static str>)> {
    let kind = object.field("rodzaj");
    match (object.class.as_str(), kind) {
        ("PTLZ_A", "las" | "zagajnik") => Some(("Wald", None)),
        ("PTLZ_A", "zadrzewienie") => Some(("Gehölz", None)),
        ("PTTR_A", "roślinność trawiasta") => Some(("Landwirtschaft", Some("Grünland"))),
        ("PTTR_A", "uprawa na gruntach ornych") => Some(("Landwirtschaft", None)),
        ("PTUT_A", "sad" | "szkółka roślin" | "plantacja") => Some(("Landwirtschaft", None)),
        ("PTUT_A", "ogródki działkowe") => {
            Some(("Sport-, Freizeit- und Erholungsfläche", Some("Kleingarten")))
        }
        ("PTWP_A", "woda stojąca") => Some(("Stehendes Gewässer", None)),
        ("PTWP_A", "woda płynąca") => Some(("Fließgewässer", None)),
        ("PTZB_A", "jednorodzinna" | "wielorodzinna") => Some(("Wohnbaufläche", None)),
        ("PTZB_A" | "PTNZ_A", "przemysłowo-składowa" | "teren przemysłowo-składowy") => {
            Some(("Industrie- und Gewerbefläche", None))
        }
        ("PTPL_A", _) => (!matches!(
            object.field("materialNawierzchni"),
            "grunt naturalny" | "żwir" | "tłuczeń"
        ))
        .then_some(("Platz", None)),
        ("KUKO_A", "parking") => Some(("Platz", Some("Parkplatz"))),
        ("KUSC_A", "cmentarz komunalny" | "cmentarz wyznaniowy" | "cmentarz wojenny") => {
            Some(("Friedhof", None))
        }
        ("KUSK_A", "park lub skwer") => {
            Some(("Sport-, Freizeit- und Erholungsfläche", Some("Grünanlage")))
        }
        _ => None,
    }
}

/// Whether a building is on the map: standing (`eksploatowany`) or out of use (`nieczynny`).
/// One under construction (`w budowie`) is no building yet and a destroyed one (`zniszczony`)
/// none any more; an object without the value counts as standing.
fn is_standing(object: &Object) -> bool {
    matches!(
        object.field("kategoriaIstnienia"),
        "" | "eksploatowany" | "nieczynny"
    )
}

/// The id of an object as the engine sees it: the id of the building in EGiB where the object
/// has one (so the same building is not found twice), else the `lokalnyId` of BDOT10k.
fn oid(object: &Object) -> String {
    match object.field("identyfikatorEGiB") {
        "" => format!("BDOT10k.{}.{}", object.class, object.id),
        egib_id => egib_id.to_string(),
    }
}

fn as_feature(object: &Object) -> Feature {
    Feature {
        id: oid(object),
        kind: String::new(),
        polygons: object.polygons.clone(),
    }
}

/// The buildings of `objects` as the engine reads them, in the CRS of the map.
pub fn buildings(objects: &[Object], crs: Crs) -> AlkisPart {
    let features = objects
        .iter()
        .filter(|o| o.class == "BUBD_A" && is_standing(o) && !o.id.is_empty())
        .filter_map(|o| {
            let function =
                egib::building_function(building_letter(o.field("kodKst")).unwrap_or("i"));
            Some(json!({
                "type": "Feature",
                "id": oid(o),
                "geometry": egib::geometry(&as_feature(o), crs)?,
                "properties": {
                    "oid": oid(o),
                    "gebnutzbez": "Gebäude",
                    "funktion": function,
                },
            }))
        })
        .collect();
    AlkisPart { features, crs }
}

/// The land use of `objects` as the engine reads it, in the CRS of the map. An object whose
/// representative point lies in `covered` (the land use EGiB did give, in EPSG:2180 with
/// `x` east and `y` north) is left out: EGiB comes first.
pub fn land_use(objects: &[Object], crs: Crs, covered: &[Polygon<f64>]) -> AlkisPart {
    let features = objects
        .iter()
        .filter(|o| !o.id.is_empty())
        .filter(|o| {
            let Some(point) = polygon_of(o).and_then(|p| p.interior_point()) else {
                return true;
            };
            !covered.iter().any(|c| c.contains(&point))
        })
        .filter_map(|o| {
            let (nutzart, bez) = land_use_of(o)?;
            let mut properties = json!({"oid": oid(o), "nutzart": nutzart});
            if let Some(bez) = bez {
                properties["bez"] = json!(bez);
            }
            Some(json!({
                "type": "Feature",
                "id": oid(o),
                "geometry": egib::geometry(&as_feature(o), crs)?,
                "properties": properties,
            }))
        })
        .collect::<Vec<Value>>();
    AlkisPart { features, crs }
}

/// The first polygon of an object as a geometry with `x` east and `y` north.
fn polygon_of(object: &Object) -> Option<Polygon<f64>> {
    let rings = object.polygons.first()?;
    to_polygon(rings)
}

/// A polygon from rings of `(north, east)` points.
pub(crate) fn to_polygon(rings: &[Vec<(f64, f64)>]) -> Option<Polygon<f64>> {
    let line = |ring: &Vec<(f64, f64)>| {
        LineString::new(ring.iter().map(|&(n, e)| Coord { x: e, y: n }).collect())
    };
    let (outer, holes) = rings.split_first()?;
    (outer.len() >= 4).then(|| Polygon::new(line(outer), holes.iter().map(line).collect()))
}

/// The share of `bbox` the parcels of `features` cover that give a land use, from 0 to 1. A
/// parcel counts with the part of it that lies in the box, so a large wood outside the corner
/// of the box does not stand for the box.
pub fn land_use_share(features: &[Feature], bbox: &BoundingBox) -> f64 {
    let total = (bbox.max_e - bbox.min_e) * (bbox.max_n - bbox.min_n);
    if total <= 0.0 {
        return 0.0;
    }
    let covered: f64 = features
        .iter()
        .filter(|f| egib::parcel_land_use(&f.kind).is_some())
        .flat_map(|f| &f.polygons)
        .map(|rings| {
            let mut parts = rings.iter().map(|ring| clipped_area(ring, bbox));
            let outer = parts.next().unwrap_or(0.0);
            outer - parts.sum::<f64>()
        })
        .sum();
    (covered / total).clamp(0.0, 1.0)
}

/// Whether the parcels of `features` leave the land use of `bbox` to BDOT10k: they cover less of
/// it than [`MIN_LAND_USE_SHARE`] (the share itself is enough).
pub fn land_use_is_thin(features: &[Feature], bbox: &BoundingBox) -> bool {
    land_use_share(features, bbox) < MIN_LAND_USE_SHARE
}

/// The area of the part of a ring of `(north, east)` points that lies in `bbox`
/// (Sutherland-Hodgman against the four sides: the window is convex).
fn clipped_area(ring: &[(f64, f64)], bbox: &BoundingBox) -> f64 {
    // Points as (e, n); each side keeps what satisfies `inside` and cuts at `at`.
    let mut points: Vec<(f64, f64)> = ring.iter().map(|&(n, e)| (e, n)).collect();
    if points.first() == points.last() {
        points.pop();
    }
    type Side<'a> = (
        &'a dyn Fn((f64, f64)) -> bool,
        &'a dyn Fn((f64, f64), (f64, f64)) -> (f64, f64),
    );
    let cut_e = |value: f64| {
        move |a: (f64, f64), b: (f64, f64)| {
            let t = (value - a.0) / (b.0 - a.0);
            (value, a.1 + t * (b.1 - a.1))
        }
    };
    let cut_n = |value: f64| {
        move |a: (f64, f64), b: (f64, f64)| {
            let t = (value - a.1) / (b.1 - a.1);
            (a.0 + t * (b.0 - a.0), value)
        }
    };
    let (min_e, max_e, min_n, max_n) = (bbox.min_e, bbox.max_e, bbox.min_n, bbox.max_n);
    let (west, east, south, north) = (cut_e(min_e), cut_e(max_e), cut_n(min_n), cut_n(max_n));
    let sides: [Side<'_>; 4] = [
        (&|p| p.0 >= min_e, &west),
        (&|p| p.0 <= max_e, &east),
        (&|p| p.1 >= min_n, &south),
        (&|p| p.1 <= max_n, &north),
    ];
    for (inside, at) in sides {
        let input = std::mem::take(&mut points);
        for (i, &current) in input.iter().enumerate() {
            let previous = input[(i + input.len() - 1) % input.len()];
            match (inside(previous), inside(current)) {
                (true, true) => points.push(current),
                (true, false) => points.push(at(previous, current)),
                (false, true) => {
                    points.push(at(previous, current));
                    points.push(current);
                }
                (false, false) => {}
            }
        }
        if points.is_empty() {
            return 0.0;
        }
    }
    let twice: f64 = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let q = points[(i + 1) % points.len()];
            p.0 * q.1 - q.0 * p.1
        })
        .sum();
    twice.abs() / 2.0
}

/// What BDOT10k gave for a gap.
#[derive(Debug, Default, PartialEq)]
pub struct Filled {
    pub objects: Vec<Object>,
    /// The powiats read.
    pub powiats: Vec<String>,
}

/// The file of a package in the shared cache. The date is the `Last-Modified` of the archive
/// where the server sends one, else the date of the index, so a package the office publishes
/// again is downloaded again, and the older file of the powiat is removed.
fn cache_file(env: &AdapterEnv<'_>, package: &Package, stamp: &str) -> PathBuf {
    env.cache
        .provider_dir(PROVIDER)
        .join("bdot10k")
        .join(format!("{}_{stamp}.zip", package.teryt))
}

/// Reads what BDOT10k holds for `gap` in `bbox` (EPSG:2180): asks the index, downloads the
/// archives of the powiats the box touches unless the cache holds them, and reads the classes
/// of the gap from them. Offline nothing is asked and nothing is found: the stored data of the
/// project is read instead.
pub fn fill(
    env: &mut AdapterEnv<'_>,
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    gap: Gap,
) -> Result<Filled, AdapterError> {
    fill_capped(env, fetcher, bbox, gap, ARCHIVE_CAP_MB * 1_000_000)
}

fn fill_capped(
    env: &mut AdapterEnv<'_>,
    fetcher: &dyn Fetcher,
    bbox: &BoundingBox,
    gap: Gap,
    cap_bytes: u64,
) -> Result<Filled, AdapterError> {
    if env.mode == FetchMode::Offline {
        return Ok(Filled::default());
    }
    let url = index_url(bbox);
    let packages = get_with_retries(fetcher, &url, |body| read_index(body, &url))?;
    if packages.len() > MAX_PACKAGES {
        return Err(AdapterError::Fetch(http_error(
            &url,
            format!("the box touches {} powiats", packages.len()),
        )));
    }
    let mut filled = Filled::default();
    for package in &packages {
        let stamp = match fetcher.last_modified(&package.url) {
            Ok(Some(date)) => date.format("%Y%m%d").to_string(),
            _ => package.updated.replace('-', ""),
        };
        let path = cache_file(env, package, &stamp);
        if !path.exists() {
            download_with_retries(fetcher, &package.url, &path, cap_bytes, &mut *env.progress)?;
            remove_older(&path, &package.teryt);
        }
        let read = read_archive(&path, gap.classes(), bbox).inspect_err(|_| {
            // A broken download must not be read again from the cache.
            let _ = std::fs::remove_file(&path);
        })?;
        filled.objects.extend(read);
        filled.powiats.push(package.name.clone());
    }
    Ok(filled)
}

/// The cemeteries and church grounds of BDOT10k in `bbox` (EPSG:2180), read as one member of
/// each powiat archive by byte ranges (ADR 0017), about 30 KB for a map, not the whole archive
/// that [`fill`] downloads. For the parcels of the class `Bi` that EGiB gives no land use
/// (ADR 0194).
pub fn cemeteries(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Result<Filled, AdapterError> {
    let url = index_url(bbox);
    let packages = get_with_retries(fetcher, &url, |body| read_index(body, &url))?;
    if packages.len() > MAX_PACKAGES {
        return Err(AdapterError::Fetch(http_error(
            &url,
            format!("the box touches {} powiats", packages.len()),
        )));
    }
    let suffix = "__ot_kusc_a.xml";
    let mut filled = Filled::default();
    for package in &packages {
        let members = read_members(fetcher, &package.url)?;
        // A class without an object is a file of 454 bytes, which an archive may also leave out.
        if let Some(member) = find_member(&members, |name| name.to_lowercase().ends_with(suffix)) {
            let bytes = read_member_up_to(fetcher, &package.url, member, CEMETERY_MEMBER_BYTES)?;
            filled.objects.extend(read_class(
                BufReader::new(bytes.as_slice()),
                "KUSC_A",
                bbox,
                &package.url,
            )?);
        }
        filled.powiats.push(package.name.clone());
    }
    Ok(filled)
}

/// Removes the other files of the powiat `teryt` next to `kept`.
fn remove_older(kept: &Path, teryt: &str) {
    let Some(dir) = kept.parent() else { return };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if entry.path() != kept && name.starts_with(&format!("{teryt}_")) && name.ends_with(".zip")
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// The note that says BDOT10k fills a gap.
pub fn filled_note(gap: Gap, count: usize, powiats: &[String]) -> Note {
    note!(
        "progress-bdot-filled",
        what = gap.words(),
        count = count,
        powiat = powiats.join(", ")
    )
}

/// The note that says EGiB failed and BDOT10k stands in for it.
pub fn instead_note(gap: Gap, error: FetchError, powiats: &[String]) -> Note {
    note!(
        "warning-bdot-instead",
        what = gap.words(),
        error = Arg::error(error),
        powiat = powiats.join(", ")
    )
}

/// The note that says BDOT10k could not fill a gap.
pub fn failed_note(gap: Gap, error: AdapterError) -> Note {
    note!(
        "warning-bdot-failed",
        what = gap.words(),
        error = Arg::error(error)
    )
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::testing::MapFetcher;

    use super::*;

    const INDEX: &str = include_str!("../tests/fixtures/bdot10k/powiaty-slubice.xml");
    const BUBD: &str = include_str!("../tests/fixtures/bdot10k/BUBD_A.xml");
    const KUKO: &str = include_str!("../tests/fixtures/bdot10k/KUKO_A.xml");
    const PTLZ: &str = include_str!("../tests/fixtures/bdot10k/PTLZ_A.xml");
    const PTPL: &str = include_str!("../tests/fixtures/bdot10k/PTPL_A.xml");
    const PTTR: &str = include_str!("../tests/fixtures/bdot10k/PTTR_A.xml");
    const PTZB: &str = include_str!("../tests/fixtures/bdot10k/PTZB_A.xml");
    const URL: &str = "https://mapy.geoportal.gov.pl/wss/service/PZGIK/BDOT/WFS/PobieranieBDOT10k";
    const ARCHIVE_URL: &str =
        "https://opendata.geoportal.gov.pl/bdot10k/schemat2021/08/0805_GML.zip";

    /// The box of the fixtures at Słubice, EPSG:2180.
    fn slubice() -> BoundingBox {
        BoundingBox {
            min_e: 197_500.0,
            min_n: 507_300.0,
            max_e: 198_000.0,
            max_n: 507_900.0,
        }
    }

    fn read(class: &str, text: &str) -> Vec<Object> {
        read_class(Cursor::new(text.as_bytes()), class, &slubice(), "test").unwrap()
    }

    /// An object of `class` with the values `fields` and a square ring.
    fn object(class: &str, fields: &[(&str, &str)]) -> Object {
        Object {
            class: class.to_string(),
            id: "x".to_string(),
            fields: fields
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            polygons: vec![vec![vec![
                (507_600.0, 197_600.0),
                (507_600.0, 197_610.0),
                (507_610.0, 197_610.0),
                (507_610.0, 197_600.0),
                (507_600.0, 197_600.0),
            ]]],
        }
    }

    fn use_of(
        class: &str,
        fields: &[(&str, &str)],
    ) -> Option<(&'static str, Option<&'static str>)> {
        land_use_of(&object(class, fields))
    }

    fn zip_of(files: &[(&str, &str)]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, text) in files {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(text.as_bytes()).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// An archive of the fixtures as the office names the files.
    fn archive() -> Vec<u8> {
        let name =
            |class: &str| format!("PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_{class}.xml");
        let files = [
            ("BUBD_A", BUBD),
            ("KUKO_A", KUKO),
            ("PTLZ_A", PTLZ),
            ("PTPL_A", PTPL),
            ("PTTR_A", PTTR),
            ("PTZB_A", PTZB),
        ];
        let files: Vec<(String, &str)> = files.iter().map(|(c, t)| (name(c), *t)).collect();
        zip_of(
            &files
                .iter()
                .map(|(n, t)| (n.as_str(), *t))
                .collect::<Vec<_>>(),
        )
    }

    struct Run {
        _scratch: tempfile::TempDir,
        cache: CacheDir,
        notes: Vec<String>,
    }

    impl Run {
        fn new() -> Self {
            let scratch = tempfile::tempdir().unwrap();
            let cache = CacheDir::new(scratch.path().join("cache"));
            Self {
                _scratch: scratch,
                cache,
                notes: Vec::new(),
            }
        }

        fn fill(
            &mut self,
            fetcher: &MapFetcher,
            gap: Gap,
            mode: FetchMode,
            cap: u64,
        ) -> Result<Filled, AdapterError> {
            let registry = freemapper_adapter_api::registry::Registry::new();
            let notes = &mut self.notes;
            let mut progress = |n: Note| notes.push(n.to_string());
            let mut env = AdapterEnv {
                fetcher,
                regions: &registry,
                cache: &self.cache,
                mode,
                today: NaiveDate::from_ymd_opt(2026, 11, 22).unwrap(),
                skip_surface: false,
                progress: &mut progress,
            };
            fill_capped(&mut env, fetcher, &slubice(), gap, cap)
        }
    }

    fn served() -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(URL, INDEX.as_bytes().to_vec());
        fetcher.insert(ARCHIVE_URL, archive());
        fetcher
    }

    // The index.

    #[test]
    fn the_index_is_asked_for_the_box_north_first_and_with_the_properties_named() {
        let url = index_url(&BoundingBox {
            min_e: 197_600.4,
            min_n: 507_400.2,
            max_e: 198_100.7,
            max_n: 507_900.0,
        });
        assert!(url.starts_with(URL), "{url}");
        assert!(url.contains("TYPENAMES=ms:BDOT10k_powiaty"), "{url}");
        assert!(
            url.contains("PROPERTYNAME=TERYT,NAZWA_POWIATU,URL_GML,Data_aktualizacji"),
            "{url}"
        );
        assert!(
            url.contains("BBOX=507400,197600,507900,198101,urn:ogc:def:crs:EPSG::2180"),
            "{url}"
        );
    }

    #[test]
    fn a_package_of_the_index_is_read() {
        let packages = read_index(INDEX.as_bytes(), URL).unwrap();
        assert_eq!(
            packages,
            [Package {
                teryt: "0805".into(),
                name: "powiat słubicki".into(),
                updated: "2024-04-04".into(),
                url: ARCHIVE_URL.into(),
            }]
        );
    }

    #[test]
    fn an_index_without_a_powiat_gives_no_package() {
        let empty =
            r#"<wfs:FeatureCollection xmlns:wfs="w" numberReturned="0"></wfs:FeatureCollection>"#;
        assert_eq!(read_index(empty.as_bytes(), URL).unwrap(), []);
    }

    /// An index entry with the given fields.
    fn entry(teryt: &str, url: &str, date: &str) -> String {
        format!(
            "<wfs:FeatureCollection xmlns:ms=\"m\" xmlns:wfs=\"w\"><wfs:member><ms:BDOT10k_powiaty>\
             <ms:TERYT>{teryt}</ms:TERYT><ms:NAZWA_POWIATU>p</ms:NAZWA_POWIATU>\
             <ms:Data_aktualizacji>{date}</ms:Data_aktualizacji><ms:URL_GML>{url}</ms:URL_GML>\
             </ms:BDOT10k_powiaty></wfs:member></wfs:FeatureCollection>"
        )
    }

    #[test]
    fn a_package_with_a_foreign_address_or_a_bad_code_is_left_out() {
        for (teryt, url) in [
            (
                "0805",
                "https://example.org/bdot10k/schemat2021/08/0805_GML.zip",
            ),
            (
                "0805",
                "http://opendata.geoportal.gov.pl/bdot10k/schemat2021/08/0805_GML.zip",
            ),
            (
                "0805",
                "https://opendata.geoportal.gov.pl/other/0805_GML.zip",
            ),
            (
                "0805",
                "https://opendata.geoportal.gov.pl/bdot10k/../x/0805_GML.zip",
            ),
            (
                "0805",
                "https://opendata.geoportal.gov.pl/bdot10k/schemat2021/08/0806_GML.zip",
            ),
            (
                "08x5",
                "https://opendata.geoportal.gov.pl/bdot10k/schemat2021/08/08x5_GML.zip",
            ),
            (
                "805",
                "https://opendata.geoportal.gov.pl/bdot10k/schemat2021/08/805_GML.zip",
            ),
        ] {
            let error = read_index(entry(teryt, url, "2024-04-04 00:00:00").as_bytes(), URL)
                .expect_err(url);
            assert!(
                error.to_string().contains("none with a usable code"),
                "{url}: {error}"
            );
        }
    }

    #[test]
    fn a_usable_package_stays_beside_an_unusable_one() {
        let good = entry("0805", ARCHIVE_URL, "2024-04-04 00:00:00");
        let bad = entry("0806", "https://example.org/x", "2024-04-04 00:00:00");
        let body = good.replace(
            "</wfs:FeatureCollection>",
            &bad[bad.find("<wfs:member>").unwrap()..],
        );
        let packages = read_index(body.as_bytes(), URL).unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].teryt, "0805");
    }

    #[test]
    fn a_date_that_is_no_date_reads_as_unknown_and_a_date_with_a_time_is_cut() {
        let at = |date: &str| {
            read_index(entry("0805", ARCHIVE_URL, date).as_bytes(), URL).unwrap()[0]
                .updated
                .clone()
        };
        assert_eq!(at("2024-04-04 00:00:00"), "2024-04-04");
        assert_eq!(at("2024-04-04"), "2024-04-04");
        assert_eq!(at("04.04.2024"), "unknown");
        assert_eq!(at(""), "unknown");
    }

    #[test]
    fn an_exception_report_an_other_root_and_a_cut_answer_are_errors_of_the_index() {
        let report = r#"<ows:ExceptionReport xmlns:ows="o"><ows:Exception exceptionCode="NoApplicableCode"><ows:ExceptionText>boom</ows:ExceptionText></ows:Exception></ows:ExceptionReport>"#;
        let error = read_index(report.as_bytes(), URL).unwrap_err();
        assert!(error.to_string().contains("boom"), "{error}");
        let error = read_index(b"<html><body/></html>", URL).unwrap_err();
        assert!(
            error.to_string().contains("no feature collection"),
            "{error}"
        );
        let cut = &INDEX[..INDEX.find("</wfs:FeatureCollection>").unwrap()];
        let error = read_index(cut.as_bytes(), URL).unwrap_err();
        assert!(error.to_string().contains("ends before"), "{error}");
    }

    // The files of an archive.

    #[test]
    fn the_buildings_of_a_file_are_read_with_the_box_applied_and_the_axes_swapped() {
        // A byte order mark leads the files of the office.
        let text = format!("\u{feff}{BUBD}");
        let objects = read("BUBD_A", &text);
        // Seven objects in the file; the one 20 km away does not touch the box.
        assert_eq!(objects.len(), 6);
        let first = &objects[0];
        assert_eq!(first.field("kodKst"), "110");
        assert_eq!(
            first.field("identyfikatorEGiB"),
            "080505_4.0001.638/14.1_BUD"
        );
        assert!(!first.id.is_empty());
        // The posList reads `E N`: Słubice lies near 197 000 east and 507 000 north. The points
        // come back as (north, east).
        let (north, east) = first.polygons[0][0][0];
        assert!((507_000.0..508_000.0).contains(&north), "{north}");
        assert!((197_000.0..198_000.0).contains(&east), "{east}");
    }

    #[test]
    fn an_object_that_only_touches_the_box_is_kept_and_one_beside_it_is_not() {
        let at = |bbox: BoundingBox| {
            read_class(Cursor::new(PTPL.as_bytes()), "PTPL_A", &bbox, "t")
                .unwrap()
                .len()
        };
        // The square at 197 539.77 / 507 539.34 reaches east, so a box ending at its westmost
        // point still meets it (edges count) and one ending 1 m earlier does not.
        let object = &read("PTPL_A", PTPL)[0];
        let west = object
            .polygons
            .iter()
            .flatten()
            .flatten()
            .map(|p| p.1)
            .fold(f64::MAX, f64::min);
        let mut b = BoundingBox {
            min_e: west - 100.0,
            min_n: 507_500.0,
            max_e: west,
            max_n: 507_550.0,
        };
        assert!(at(b) >= 1);
        b.max_e = west - 1.0;
        let touching_less = at(b);
        b.max_e = west;
        assert!(at(b) > touching_less, "the edge counts");
    }

    #[test]
    fn rings_with_holes_are_kept_with_the_outer_ring_first() {
        let wide = BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: 1_000_000.0,
            max_n: 1_000_000.0,
        };
        let objects = read_class(Cursor::new(PTTR.as_bytes()), "PTTR_A", &wide, "t").unwrap();
        assert_eq!(objects.len(), 5);
        let with_holes = objects.iter().find(|o| o.polygons[0].len() == 3).unwrap();
        assert!(with_holes.polygons[0][0].len() > 4);
    }

    #[test]
    fn a_file_without_objects_is_empty() {
        let empty = r#"<?xml version="1.0"?><gml:FeatureCollection xmlns:gml="g"/>"#;
        // A self-closing root is one empty event, which counts as closed.
        let found = read_class(Cursor::new(empty.as_bytes()), "BUBD_A", &slubice(), "t");
        assert!(found.is_ok(), "{found:?}");
        let closed = r#"<gml:FeatureCollection xmlns:gml="g"></gml:FeatureCollection>"#;
        assert_eq!(read("BUBD_A", closed), []);
    }

    #[test]
    fn broken_gml_is_an_error_and_names_what_broke() {
        let cut = &BUBD[..BUBD.rfind("</gml:featureMember>").unwrap()];
        let error =
            read_class(Cursor::new(cut.as_bytes()), "BUBD_A", &slubice(), "f.xml").unwrap_err();
        assert!(error.to_string().contains("ends before"), "{error}");
        // Cut inside a tag: the XML itself is broken.
        let error = read_class(
            Cursor::new(&BUBD.as_bytes()[..BUBD.len() - 40]),
            "BUBD_A",
            &slubice(),
            "f.xml",
        )
        .unwrap_err();
        assert!(error.to_string().contains("no XML"), "{error}");
        let letters = BUBD.replacen("197974.21 507561.49", "197974.21 x", 1);
        let error = read_class(
            Cursor::new(letters.as_bytes()),
            "BUBD_A",
            &slubice(),
            "f.xml",
        )
        .unwrap_err();
        assert!(error.to_string().contains("pairs of numbers"), "{error}");
        let odd = BUBD.replacen("197974.21 507561.49", "197974.21", 1);
        let error =
            read_class(Cursor::new(odd.as_bytes()), "BUBD_A", &slubice(), "f.xml").unwrap_err();
        assert!(error.to_string().contains("pairs of numbers"), "{error}");
        let error = read_class(
            Cursor::new(b"<html/>".as_slice()),
            "BUBD_A",
            &slubice(),
            "f.xml",
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("no feature collection"),
            "{error}"
        );
        let error = read_class(
            Cursor::new(b"<gml:FeatureCollection><b></gml:FeatureCollection>".as_slice()),
            "BUBD_A",
            &slubice(),
            "f.xml",
        )
        .unwrap_err();
        assert!(error.to_string().contains("no XML"), "{error}");
    }

    // The mapping of buildings.

    #[test]
    fn each_kst_code_of_a_building_stands_for_the_letter_of_the_register() {
        for (code, letter) in [
            ("101", "p"),
            ("102", "t"),
            ("103", "h"),
            ("104", "s"),
            ("105", "b"),
            ("106", "z"),
            ("107", "k"),
            ("108", "g"),
            ("109", "i"),
            ("110", "m"),
        ] {
            assert_eq!(building_letter(code), Some(letter), "{code}");
            assert_eq!(building_letter(&format!(" {code} ")), Some(letter));
        }
        for code in ["", "100", "111", "11", "1100", "m"] {
            assert_eq!(building_letter(code), None, "{code:?}");
        }
    }

    fn building(kst: &str, existence: &str, egib: &str) -> Object {
        let mut fields = vec![("kodKst", kst), ("kategoriaIstnienia", existence)];
        if !egib.is_empty() {
            fields.push(("identyfikatorEGiB", egib));
        }
        object("BUBD_A", &fields)
    }

    fn functions(objects: &[Object]) -> Vec<String> {
        buildings(objects, Crs::Utm33)
            .features
            .iter()
            .map(|f| f["properties"]["funktion"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn a_building_takes_the_function_of_its_register_kind() {
        let objects: Vec<Object> = [
            "110", "108", "101", "102", "103", "104", "105", "106", "107", "109",
        ]
        .iter()
        .map(|kst| building(kst, "eksploatowany", "id"))
        .collect();
        assert_eq!(
            functions(&objects),
            [
                "Wohngebäude",
                "Gebäude für Wirtschaft und Gewerbe",
                "Industriegebäude",
                "Gebäude für Verkehr und Nachrichtenwesen",
                "Handels- und Dienstleistungsgebäude",
                "Behälter, Silo und Speichergebäude",
                "Bürogebäude",
                "Gebäude für Gesundheit und Pflege",
                "Gebäude für Bildung, Wissenschaft und Kultur",
                "Sonstiges Gebäude",
            ]
        );
    }

    #[test]
    fn a_building_without_a_known_code_is_an_other_building() {
        assert_eq!(
            functions(&[building("", "eksploatowany", "a"), building("999", "", "b")]),
            ["Sonstiges Gebäude", "Sonstiges Gebäude"]
        );
    }

    #[test]
    fn only_a_standing_building_is_on_the_map() {
        let objects: Vec<Object> = ["eksploatowany", "nieczynny", "w budowie", "zniszczony", ""]
            .iter()
            .map(|e| building("110", e, "id"))
            .collect();
        // Standing, out of use and unknown stay; under construction and destroyed go.
        assert_eq!(buildings(&objects, Crs::Utm33).features.len(), 3);
    }

    #[test]
    fn a_building_keeps_the_id_it_has_in_the_register_and_else_its_own() {
        let part = buildings(
            &[
                building("110", "eksploatowany", "080505_4.0001.638/14.1_BUD"),
                building("110", "eksploatowany", ""),
            ],
            Crs::Utm33,
        );
        assert_eq!(
            part.features[0]["properties"]["oid"],
            "080505_4.0001.638/14.1_BUD"
        );
        assert_eq!(part.features[1]["properties"]["oid"], "BDOT10k.BUBD_A.x");
        assert_eq!(part.features[0]["properties"]["gebnutzbez"], "Gebäude");
        assert_eq!(part.features[0]["geometry"]["type"], "MultiPolygon");
    }

    #[test]
    fn an_object_without_an_id_is_no_building() {
        let mut nameless = building("110", "eksploatowany", "");
        nameless.id.clear();
        assert!(buildings(&[nameless], Crs::Utm33).features.is_empty());
        // Another class is no building.
        let other = object("PTLZ_A", &[("rodzaj", "las")]);
        assert!(buildings(&[other], Crs::Utm33).features.is_empty());
    }

    // The mapping of land use: one test per rule.

    #[test]
    fn forest_and_grove_are_forest() {
        assert_eq!(use_of("PTLZ_A", &[("rodzaj", "las")]), Some(("Wald", None)));
        assert_eq!(
            use_of("PTLZ_A", &[("rodzaj", "zagajnik")]),
            Some(("Wald", None))
        );
    }

    #[test]
    fn a_tree_stand_is_a_copse() {
        assert_eq!(
            use_of("PTLZ_A", &[("rodzaj", "zadrzewienie")]),
            Some(("Gehölz", None))
        );
        assert_eq!(use_of("PTLZ_A", &[("rodzaj", "other")]), None);
    }

    #[test]
    fn grass_is_grassland_and_a_field_is_farmland() {
        assert_eq!(
            use_of("PTTR_A", &[("rodzaj", "roślinność trawiasta")]),
            Some(("Landwirtschaft", Some("Grünland")))
        );
        assert_eq!(
            use_of("PTTR_A", &[("rodzaj", "uprawa na gruntach ornych")]),
            Some(("Landwirtschaft", None))
        );
        assert_eq!(use_of("PTTR_A", &[("rodzaj", "other")]), None);
    }

    #[test]
    fn orchards_nurseries_and_plantations_are_farmland() {
        for kind in ["sad", "szkółka roślin", "plantacja"] {
            assert_eq!(
                use_of("PTUT_A", &[("rodzaj", kind)]),
                Some(("Landwirtschaft", None)),
                "{kind}"
            );
        }
    }

    #[test]
    fn allotment_gardens_are_private_recreation_land() {
        assert_eq!(
            use_of("PTUT_A", &[("rodzaj", "ogródki działkowe")]),
            Some(("Sport-, Freizeit- und Erholungsfläche", Some("Kleingarten")))
        );
    }

    #[test]
    fn still_and_running_water_are_the_two_water_uses() {
        assert_eq!(
            use_of("PTWP_A", &[("rodzaj", "woda stojąca")]),
            Some(("Stehendes Gewässer", None))
        );
        assert_eq!(
            use_of("PTWP_A", &[("rodzaj", "woda płynąca")]),
            Some(("Fließgewässer", None))
        );
        assert_eq!(use_of("PTWP_A", &[("rodzaj", "morze")]), None);
    }

    #[test]
    fn housing_is_residential_land_and_commercial_land_is_left_out() {
        for kind in ["jednorodzinna", "wielorodzinna"] {
            assert_eq!(
                use_of("PTZB_A", &[("rodzaj", kind)]),
                Some(("Wohnbaufläche", None)),
                "{kind}"
            );
        }
        for kind in ["handlowo-usługowa", "pozostała zabudowa"] {
            assert_eq!(use_of("PTZB_A", &[("rodzaj", kind)]), None, "{kind}");
        }
    }

    #[test]
    fn industrial_and_storage_land_is_industrial_land() {
        assert_eq!(
            use_of("PTZB_A", &[("rodzaj", "przemysłowo-składowa")]),
            Some(("Industrie- und Gewerbefläche", None))
        );
        assert_eq!(
            use_of("PTNZ_A", &[("rodzaj", "teren przemysłowo-składowy")]),
            Some(("Industrie- und Gewerbefläche", None))
        );
        // Land under technical installations (a solar farm) is no industrial area.
        assert_eq!(
            use_of(
                "PTNZ_A",
                &[(
                    "rodzaj",
                    "teren pod urządzeniami technicznymi lub budowlami"
                )]
            ),
            None
        );
    }

    #[test]
    fn a_square_is_paved_unless_its_surface_is_bare_ground() {
        for material in [
            "kostka prefabrykowana",
            "masa bitumiczna",
            "beton",
            "płyty betonowe",
            "inny",
            "",
        ] {
            assert_eq!(
                use_of("PTPL_A", &[("materialNawierzchni", material)]),
                Some(("Platz", None)),
                "{material:?}"
            );
        }
        for material in ["grunt naturalny", "żwir", "tłuczeń"] {
            assert_eq!(
                use_of("PTPL_A", &[("materialNawierzchni", material)]),
                None,
                "{material}"
            );
        }
    }

    #[test]
    fn a_car_park_is_a_car_park_and_a_filling_station_is_left_out() {
        assert_eq!(
            use_of("KUKO_A", &[("rodzaj", "parking")]),
            Some(("Platz", Some("Parkplatz")))
        );
        for kind in ["stacja paliw", "stacja kolejowa", "port wodny lub przystań"] {
            assert_eq!(use_of("KUKO_A", &[("rodzaj", kind)]), None, "{kind}");
        }
    }

    #[test]
    fn cemeteries_are_cemeteries_and_church_grounds_are_not() {
        for kind in [
            "cmentarz komunalny",
            "cmentarz wyznaniowy",
            "cmentarz wojenny",
        ] {
            assert_eq!(
                use_of("KUSC_A", &[("rodzaj", kind)]),
                Some(("Friedhof", None)),
                "{kind}"
            );
        }
        assert_eq!(
            use_of("KUSC_A", &[("rodzaj", "zespół sakralny lub klasztorny")]),
            None
        );
    }

    #[test]
    fn a_park_is_a_green_space_and_a_sports_complex_is_left_out() {
        assert_eq!(
            use_of("KUSK_A", &[("rodzaj", "park lub skwer")]),
            Some(("Sport-, Freizeit- und Erholungsfläche", Some("Grünanlage")))
        );
        assert_eq!(
            use_of("KUSK_A", &[("rodzaj", "kompleks sportowo-rekreacyjny")]),
            None
        );
    }

    #[test]
    fn a_class_the_table_does_not_know_gives_no_land_use() {
        for class in [
            "PTRK_A", "PTWZ_A", "PTGN_A", "OIMK_A", "SKDR_L", "BUBD_A", "",
        ] {
            assert_eq!(use_of(class, &[("rodzaj", "las")]), None, "{class}");
        }
    }

    #[test]
    fn the_land_use_is_written_in_the_schema_the_engine_reads() {
        let objects = read("PTTR_A", PTTR);
        let part = land_use(&objects, Crs::Utm33, &[]);
        assert_eq!(part.features.len(), 4);
        let first = &part.features[0];
        assert_eq!(first["properties"]["nutzart"], "Landwirtschaft");
        assert_eq!(first["properties"]["bez"], "Grünland");
        assert!(
            first["properties"]["oid"]
                .as_str()
                .unwrap()
                .starts_with("BDOT10k.PTTR_A.")
        );
        // A use without a refinement carries no `bez`.
        let part = land_use(&[object("PTLZ_A", &[("rodzaj", "las")])], Crs::Utm33, &[]);
        assert!(part.features[0]["properties"].get("bez").is_none());
    }

    #[test]
    fn where_egib_has_a_land_use_the_object_is_left_out() {
        let square = |e: f64, n: f64, side: f64| {
            Polygon::new(
                LineString::from(vec![
                    (e, n),
                    (e + side, n),
                    (e + side, n + side),
                    (e, n + side),
                    (e, n),
                ]),
                vec![],
            )
        };
        let wood = object("PTLZ_A", &[("rodzaj", "las")]);
        // The wood's square is 10 m at 197 600 / 507 600: its middle is at 197 605 / 507 605.
        let covering = square(197_590.0, 507_590.0, 30.0);
        let beside = square(197_700.0, 507_700.0, 30.0);
        assert!(
            land_use(std::slice::from_ref(&wood), Crs::Utm33, &[covering])
                .features
                .is_empty()
        );
        assert_eq!(
            land_use(std::slice::from_ref(&wood), Crs::Utm33, &[beside])
                .features
                .len(),
            1
        );
    }

    // The share of the area EGiB covers.

    fn parcel(kind: &str, ring: &[(f64, f64)]) -> Feature {
        Feature {
            id: "p".into(),
            kind: kind.into(),
            polygons: vec![vec![ring.iter().map(|&(e, n)| (n, e)).collect()]],
        }
    }

    fn hundred() -> BoundingBox {
        BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: 100.0,
            max_n: 100.0,
        }
    }

    /// A parcel that covers the first `east` metres of the 100 m box over its whole height.
    fn strip(kind: &str, east: f64) -> Feature {
        parcel(
            kind,
            &[
                (0.0, 0.0),
                (east, 0.0),
                (east, 100.0),
                (0.0, 100.0),
                (0.0, 0.0),
            ],
        )
    }

    #[test]
    fn the_share_counts_the_parcels_with_a_land_use_by_the_area_in_the_box() {
        assert!((land_use_share(&[strip("R", 50.0)], &hundred()) - 0.5).abs() < 1e-12);
        // A parcel without a class (the cities) counts for nothing; so does one that gives none.
        assert_eq!(land_use_share(&[strip("", 100.0)], &hundred()), 0.0);
        assert_eq!(land_use_share(&[strip("Bi", 100.0)], &hundred()), 0.0);
        assert_eq!(land_use_share(&[], &hundred()), 0.0);
    }

    #[test]
    fn a_parcel_counts_only_with_the_part_inside_the_box() {
        // Twice as wide as the box: the box is covered once, not twice.
        let big = parcel(
            "Ls",
            &[
                (-100.0, -50.0),
                (100.0, -50.0),
                (100.0, 150.0),
                (-100.0, 150.0),
                (-100.0, -50.0),
            ],
        );
        assert!((land_use_share(&[big], &hundred()) - 1.0).abs() < 1e-12);
        // One that lies beside the box counts for nothing.
        let away = parcel(
            "Ls",
            &[(200.0, 0.0), (300.0, 0.0), (300.0, 100.0), (200.0, 0.0)],
        );
        assert_eq!(land_use_share(&[away], &hundred()), 0.0);
    }

    #[test]
    fn a_hole_of_a_parcel_is_not_covered() {
        let mut holed = strip("R", 100.0);
        holed.polygons[0].push(
            [
                (25.0, 25.0),
                (75.0, 25.0),
                (75.0, 75.0),
                (25.0, 75.0),
                (25.0, 25.0),
            ]
            .iter()
            .map(|&(e, n)| (n, e))
            .collect(),
        );
        // 100 x 100 minus 50 x 50.
        assert!((land_use_share(&[holed], &hundred()) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn an_empty_box_has_no_share() {
        let flat = BoundingBox {
            min_e: 5.0,
            min_n: 0.0,
            max_e: 5.0,
            max_n: 100.0,
        };
        assert_eq!(land_use_share(&[strip("R", 100.0)], &flat), 0.0);
    }

    #[test]
    fn exactly_the_least_share_is_enough_and_a_hair_less_is_thin() {
        // 25 m of 100 m: the share is exactly the least share.
        assert!(!land_use_is_thin(&[strip("R", 25.0)], &hundred()));
        assert!(land_use_is_thin(&[strip("R", 24.9)], &hundred()));
        assert!(!land_use_is_thin(&[strip("R", 25.1)], &hundred()));
        assert!(land_use_is_thin(&[], &hundred()));
    }

    // The archive.

    fn write_archive(dir: &Path, bytes: &[u8]) -> PathBuf {
        let path = dir.join("0805.zip");
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn an_archive_gives_the_classes_asked_for_inside_the_box() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_archive(dir.path(), &archive());
        let found = read_archive(&path, &BUILDING_CLASSES, &slubice()).unwrap();
        assert_eq!(found.len(), 6);
        assert!(found.iter().all(|o| o.class == "BUBD_A"));
        let found = read_archive(&path, &LAND_USE_CLASSES, &slubice()).unwrap();
        let count = |class: &str| found.iter().filter(|o| o.class == class).count();
        // The fixtures at Słubice: the files this archive lacks are empty, not an error.
        assert_eq!(count("PTTR_A"), 4);
        assert_eq!(count("PTLZ_A"), 1);
        assert_eq!(count("PTPL_A"), 4);
        assert_eq!(count("PTZB_A"), 4);
        assert_eq!(count("KUKO_A"), 2);
        assert_eq!(count("PTWP_A"), 0);
    }

    #[test]
    fn a_file_that_is_no_archive_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_archive(dir.path(), b"<html>Unauthorized.</html>");
        let error = read_archive(&path, &BUILDING_CLASSES, &slubice()).unwrap_err();
        assert!(error.to_string().contains("no archive"), "{error}");
        let error = read_archive(
            &dir.path().join("missing.zip"),
            &BUILDING_CLASSES,
            &slubice(),
        )
        .unwrap_err();
        assert!(matches!(error, FetchError::Io(..)), "{error}");
    }

    #[test]
    fn a_class_file_over_the_cap_is_too_large_and_one_at_the_cap_is_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_archive(dir.path(), &archive());
        let size = BUBD.len() as u64;
        let at_cap = read_archive_capped(&path, &BUILDING_CLASSES, &slubice(), size).unwrap();
        assert_eq!(at_cap.len(), 6);
        let error =
            read_archive_capped(&path, &BUILDING_CLASSES, &slubice(), size - 1).unwrap_err();
        assert!(matches!(error, FetchError::TooLarge { .. }), "{error}");
    }

    #[test]
    fn a_broken_class_file_in_an_archive_is_an_error_naming_the_file() {
        let name = "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_BUBD_A.xml";
        let cut = &BUBD[..BUBD.rfind("</gml:featureMember>").unwrap()];
        let dir = tempfile::tempdir().unwrap();
        let path = write_archive(dir.path(), &zip_of(&[(name, cut)]));
        let error = read_archive(&path, &BUILDING_CLASSES, &slubice()).unwrap_err();
        assert!(error.to_string().contains("OT_BUBD_A.xml"), "{error}");
    }

    // The download and the cache.

    #[test]
    fn a_gap_is_filled_from_the_downloaded_archive_which_stays_in_the_cache() {
        let fetcher = served();
        let mut run = Run::new();
        let filled = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Online, 150_000_000)
            .unwrap();
        assert_eq!(filled.objects.len(), 6);
        assert_eq!(filled.powiats, ["powiat słubicki"]);
        let cached = run.cache.root().join("gugik/bdot10k/0805_20240404.zip");
        assert!(cached.exists(), "{cached:?}");
        let asked = fetcher.log.borrow().clone();
        assert_eq!(
            asked.iter().filter(|u| u.as_str() == ARCHIVE_URL).count(),
            1
        );
        // The second run reads the cache and asks the index only.
        let again = run
            .fill(&fetcher, Gap::LandUse, FetchMode::Online, 150_000_000)
            .unwrap();
        assert!(!again.objects.is_empty());
        let asked = fetcher.log.borrow().clone();
        assert_eq!(
            asked.iter().filter(|u| u.as_str() == ARCHIVE_URL).count(),
            1,
            "{asked:?}"
        );
    }

    #[test]
    fn the_cache_name_follows_the_last_modified_date_and_drops_the_older_file() {
        let fetcher = served();
        let mut run = Run::new();
        run.fill(&fetcher, Gap::Buildings, FetchMode::Online, 150_000_000)
            .unwrap();
        // The office publishes the same package again.
        fetcher.set_modified(ARCHIVE_URL, NaiveDate::from_ymd_opt(2026, 1, 22).unwrap());
        run.fill(&fetcher, Gap::Buildings, FetchMode::Online, 150_000_000)
            .unwrap();
        let dir = run.cache.root().join("gugik/bdot10k");
        let mut names: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["0805_20260122.zip"]);
    }

    #[test]
    fn an_offline_run_asks_nothing_and_finds_nothing() {
        let fetcher = served();
        let mut run = Run::new();
        let filled = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Offline, 150_000_000)
            .unwrap();
        assert_eq!(filled, Filled::default());
        assert!(fetcher.log.borrow().is_empty());
    }

    #[test]
    fn an_index_that_names_no_powiat_downloads_nothing() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            URL,
            br#"<wfs:FeatureCollection xmlns:wfs="w" numberReturned="0"></wfs:FeatureCollection>"#
                .to_vec(),
        );
        let mut run = Run::new();
        let filled = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Online, 150_000_000)
            .unwrap();
        assert_eq!(filled, Filled::default());
        assert_eq!(fetcher.log.borrow().len(), 1);
    }

    #[test]
    fn a_box_over_too_many_powiats_is_refused_before_a_download() {
        let one = entry("0805", ARCHIVE_URL, "2024-04-04 00:00:00");
        let member =
            &one[one.find("<wfs:member>").unwrap()..one.find("</wfs:FeatureCollection>").unwrap()];
        let many = format!(
            "<wfs:FeatureCollection xmlns:ms=\"m\" xmlns:wfs=\"w\">{}</wfs:FeatureCollection>",
            member.repeat(MAX_PACKAGES + 1)
        );
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(URL, many.into_bytes());
        let mut run = Run::new();
        let error = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Online, 150_000_000)
            .unwrap_err();
        assert!(error.to_string().contains("touches 7 powiats"), "{error}");
        assert_eq!(fetcher.log.borrow().len(), 1);
    }

    #[test]
    fn an_index_that_is_down_is_an_error() {
        let fetcher = served();
        let index = index_url(&slubice());
        fetcher.failing.borrow_mut().push(index);
        let mut run = Run::new();
        let error = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Online, 150_000_000)
            .unwrap_err();
        assert!(error.to_string().contains("503"), "{error}");
    }

    #[test]
    fn an_archive_over_the_cap_is_refused_and_nothing_stays_in_the_cache() {
        let fetcher = served();
        let mut run = Run::new();
        let size = archive().len() as u64;
        let error = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Online, size - 1)
            .unwrap_err();
        assert!(
            matches!(error, AdapterError::Fetch(FetchError::TooLarge { .. })),
            "{error}"
        );
        assert!(
            !run.cache
                .root()
                .join("gugik/bdot10k/0805_20240404.zip")
                .exists()
        );
        // One of exactly the cap comes through.
        let filled = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Online, size)
            .unwrap();
        assert_eq!(filled.objects.len(), 6);
    }

    #[test]
    fn a_download_that_is_no_archive_is_an_error_and_is_not_kept() {
        let fetcher = served();
        fetcher.insert(ARCHIVE_URL, b"<html>Unauthorized.</html>".to_vec());
        let mut run = Run::new();
        let error = run
            .fill(&fetcher, Gap::Buildings, FetchMode::Online, 150_000_000)
            .unwrap_err();
        assert!(error.to_string().contains("no archive"), "{error}");
        assert!(
            !run.cache
                .root()
                .join("gugik/bdot10k/0805_20240404.zip")
                .exists()
        );
    }

    #[test]
    fn the_hosts_of_bdot10k_are_hosts_the_adapter_already_asks() {
        for url in [INDEX_ENDPOINT, ARCHIVE_URL] {
            let host = host_of(url).unwrap();
            assert!(crate::polska::HOSTS.contains(&host.as_str()), "{host}");
        }
    }

    #[test]
    fn the_notes_tell_what_was_filled_in_and_what_failed() {
        let filled = filled_note(Gap::Buildings, 12, &["powiat słubicki".to_string()]);
        assert!(filled.to_string().contains("12"), "{filled}");
        assert!(filled.to_string().contains("buildings"), "{filled}");
        assert!(!filled.is_warning());
        let failed = failed_note(
            Gap::LandUse,
            AdapterError::Fetch(FetchError::NotFound("u".into())),
        );
        assert!(failed.is_warning());
        assert!(failed.to_string().contains("land use"), "{failed}");
        let instead = instead_note(Gap::Buildings, FetchError::NotFound("u".into()), &[]);
        assert!(instead.is_warning());
    }

    // The cemeteries (ADR 0194).

    const KUSC: &str = include_str!("../tests/fixtures/bdot10k/KUSC_A.xml");
    const KUSC_FILE: &str = "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_KUSC_A.xml";

    fn cemetery_fetcher(members: &[(&str, &str)]) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(URL, INDEX.as_bytes().to_vec());
        fetcher.insert(ARCHIVE_URL, zip_of(members));
        fetcher
    }

    #[test]
    fn the_cemetery_file_of_each_powiat_is_read_as_one_member() {
        let fetcher = cemetery_fetcher(&[
            (
                "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_BUBD_A.xml",
                "not read",
            ),
            (KUSC_FILE, KUSC),
        ]);
        let filled = cemeteries(&fetcher, &slubice()).unwrap();
        let kinds: Vec<&str> = filled.objects.iter().map(|o| o.field("rodzaj")).collect();
        assert_eq!(
            kinds,
            ["cmentarz komunalny", "zespół sakralny lub klasztorny"]
        );
        assert!(filled.objects.iter().all(|o| o.class == "KUSC_A"));
        assert_eq!(filled.powiats, ["powiat słubicki"]);
    }

    #[test]
    fn a_cemetery_beside_the_box_is_left_out() {
        let fetcher = cemetery_fetcher(&[(KUSC_FILE, KUSC)]);
        let elsewhere = BoundingBox {
            min_e: 190_000.0,
            min_n: 500_000.0,
            max_e: 190_100.0,
            max_n: 500_100.0,
        };
        let filled = cemeteries(&fetcher, &elsewhere).unwrap();
        assert!(filled.objects.is_empty());
        assert_eq!(filled.powiats, ["powiat słubicki"]);
    }

    #[test]
    fn an_archive_without_the_cemetery_file_gives_no_object_and_no_error() {
        let fetcher = cemetery_fetcher(&[(
            "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_BUBD_A.xml",
            "not read",
        )]);
        let filled = cemeteries(&fetcher, &slubice()).unwrap();
        assert!(filled.objects.is_empty());
        assert_eq!(filled.powiats, ["powiat słubicki"]);
    }

    #[test]
    fn the_cemetery_file_is_found_whatever_the_case_of_its_name() {
        let fetcher = cemetery_fetcher(&[(
            "PL.PZGiK.333.0805/BDOT10k/pl.pzgik.333.0805__ot_kusc_a.xml",
            KUSC,
        )]);
        assert_eq!(cemeteries(&fetcher, &slubice()).unwrap().objects.len(), 2);
    }

    #[test]
    fn a_member_with_a_name_that_only_contains_the_class_is_not_the_cemetery_file() {
        let fetcher = cemetery_fetcher(&[(
            "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_KUSC_A.xml.bak",
            KUSC,
        )]);
        assert!(cemeteries(&fetcher, &slubice()).unwrap().objects.is_empty());
    }

    #[test]
    fn a_broken_cemetery_file_is_an_error_and_so_is_a_down_index() {
        let fetcher = cemetery_fetcher(&[(KUSC_FILE, "<no-xml")]);
        assert!(cemeteries(&fetcher, &slubice()).is_err());
        let fetcher = cemetery_fetcher(&[(KUSC_FILE, KUSC)]);
        fetcher.failing.borrow_mut().push(index_url(&slubice()));
        let error = cemeteries(&fetcher, &slubice()).unwrap_err();
        assert!(error.to_string().contains("503"), "{error}");
    }

    #[test]
    fn the_cemeteries_of_the_most_powiats_are_read_and_of_more_are_refused() {
        let one = entry("0805", ARCHIVE_URL, "2024-04-04 00:00:00");
        let member =
            &one[one.find("<wfs:member>").unwrap()..one.find("</wfs:FeatureCollection>").unwrap()];
        let index = |count: usize| {
            format!(
                "<wfs:FeatureCollection xmlns:ms=\"m\" xmlns:wfs=\"w\">{}</wfs:FeatureCollection>",
                member.repeat(count)
            )
        };
        let fetcher = cemetery_fetcher(&[(KUSC_FILE, KUSC)]);
        fetcher.insert_prefix(URL, index(MAX_PACKAGES).into_bytes());
        assert_eq!(
            cemeteries(&fetcher, &slubice()).unwrap().powiats.len(),
            MAX_PACKAGES
        );
        fetcher.insert_prefix(URL, index(MAX_PACKAGES + 1).into_bytes());
        let error = cemeteries(&fetcher, &slubice()).unwrap_err();
        assert!(error.to_string().contains("touches 7 powiats"), "{error}");
    }

    #[test]
    fn the_cemetery_gap_has_its_class_and_its_words() {
        assert_eq!(Gap::Cemeteries.classes(), ["KUSC_A"]);
        let note = filled_note(Gap::Cemeteries, 2, &["p".to_string()]).to_string();
        assert!(note.contains("2 BDOT10k cemeteries filled in"), "{note}");
    }
}
