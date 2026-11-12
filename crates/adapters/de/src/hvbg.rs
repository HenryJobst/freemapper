//! HVBG Hesse open data (dl-de/zero-2-0, "Hessische Verwaltung für Bodenmanagement und
//! Geoinformation"), all in EPSG:25832: the DGM1 and DOM1 1 km tiles read by byte ranges out of
//! the Downloadcenter's municipal zips, found through the DGM/DOM metadata list.

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::AdapterId;
use freemapper_sources::raster_fetch::{
    fetch_area_heights, fetch_wms_orthophoto, warn_beyond_state,
};
use freemapper_sources::wms::OrthophotoSource;

use crate::note::{Note, note};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{CacheDir, FetchError, FetchMode, Fetcher, get_with_retries, write_cache_file};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::{decode_float_tiff, write_float_raster};
use crate::sources::height_model::HeightModel;
use crate::sources::region::{HESSEN, tile_centre_in};
use crate::sources::wms::WmsService;
use crate::sources::xlsx::{excel_date, xlsx_rows};
use crate::sources::zip_range::{
    Member, cached_member, find_member, parse_directory, read_directory_bytes, refused,
};
use crate::tiles::{TileId, tiles_covering};

/// The Downloadcenter's REST API (undocumented): the product tree and the files per district.
pub const REST_URL: &str =
    "https://gds.hessen.de/INTERSHOP/rest/WFS/HLBG-Geodaten-Site/-/downloadcenter";
/// The orthophoto WMS. `he_dop20_rgb` is only served below scale 1:9000, which is where the
/// 0.2 m requests are; MaxWidth/MaxHeight is 3000 (an oversize request is an XML exception).
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows"),
    layer: Cow::Borrowed("he_dop20_rgb"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed(""),
    max_px: 3000,
};
/// The queryable info layer with the flight date per tile.
const WMS_INFO_LAYER: &str = "he_dop_rgb_info";

/// The host of the files and documents the REST answers link by path.
const HOST: &str = "https://gds.hessen.de";
/// Most listing pages read per district (20 files each; Marburg-Biedenkopf needs two).
const MAX_PAGES: u64 = 20;

/// A height model of the Downloadcenter (1 km Float32 GeoTIFF tiles in one zip per municipality).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain model from the statewide laser scan.
    Dgm1,
    /// DOM1, the surface model from the same scan.
    Dom1,
}

impl Model {
    /// The product's path in the Downloadcenter tree.
    pub fn product(self) -> &'static str {
        match self {
            Self::Dgm1 => "3D-Daten/Digitales Geländemodell (DGM1)",
            Self::Dom1 => "3D-Daten/Digitales Oberflächenmodell (DOM1)",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (HVBG DGM1)",
            Self::Dom1 => "surface model (HVBG DOM1)",
        }
    }

    /// The product code in lower case: the start of the tile members
    /// (`dgm1_32_476_5552_1_he.tif`), the end of the municipal file names
    /// (`Frankfurt_am_Main-DGM1`) and the cache folder.
    pub fn code(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Dom1 => "dom1",
        }
    }
}

/// A municipality and its district, as the metadata list names them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub municipality: String,
    pub district: String,
}

impl Place {
    /// A file name for the cached directory of the place's zip.
    fn cache_key(&self) -> String {
        let key: String = format!(
            "{}__{}",
            name_key(&self.district),
            name_key(&self.municipality)
        )
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
        if key.len() <= MAX_KEY {
            return key;
        }
        // Far longer than any real name: shortened, with a hash of the whole key to keep
        // different names apart (FNV-1a, so the file names stay the same across releases).
        let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
        format!("{}_{hash:016x}", &key[..MAX_KEY - 17])
    }
}

/// Longest cache key (bytes) used as is; the longest real name is 35 characters.
const MAX_KEY: usize = 100;

/// `text` for a message: control characters become spaces, at most `max` characters.
fn clean(text: &str, max: usize) -> String {
    let text: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max - 1).collect();
    format!("{cut}…")
}

/// `<municipality> (<district>)` for messages, cleaned and capped (the names come from the
/// server).
fn place_label(municipality: &str, district: &str) -> String {
    format!("{} ({})", clean(municipality, 60), clean(district, 40))
}

/// What the metadata list tells about one tile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TileRecord {
    /// Every municipality whose zip holds the tile, in the list's order.
    pub places: Vec<Place>,
    /// The flight date, `YYYY-MM-DD`.
    pub date: Option<String>,
}

/// The DGM/DOM metadata list: tile → municipalities, district and flight date.
#[derive(Debug, Clone, Default)]
pub struct TileIndex {
    tiles: HashMap<TileId, TileRecord>,
    /// The tiles (`EEENNNN`) none of whose rows has a readable `Bildflugdatum`, sorted.
    pub undated: Vec<String>,
}

impl TileIndex {
    /// The index from the rows of the list's first sheet; the header row is the one with
    /// `Kachelname` (7 digits, `EEENNNN`), `Bildflugdatum` (Excel serial), `Gemeinde` and `Kreis`.
    pub fn from_rows(rows: &[Vec<String>]) -> Result<Self, String> {
        let (at, header) = rows
            .iter()
            .enumerate()
            .find(|(_, row)| row.iter().any(|c| c.trim() == "Kachelname"))
            .ok_or("the metadata list has no header row with Kachelname")?;
        let column = |name: &str| {
            header
                .iter()
                .position(|c| c.trim() == name)
                .ok_or_else(|| format!("the metadata list has no column {name}"))
        };
        let (tile_at, date_at, municipality_at, district_at) = (
            column("Kachelname")?,
            column("Bildflugdatum")?,
            column("Gemeinde")?,
            column("Kreis")?,
        );
        let mut tiles: HashMap<TileId, TileRecord> = HashMap::new();
        for row in &rows[at + 1..] {
            let cell = |i: usize| row.get(i).map_or("", |c| c.trim());
            let Some(tile) = tile_of(cell(tile_at)) else {
                continue;
            };
            let record = tiles.entry(tile).or_default();
            let place = Place {
                municipality: cell(municipality_at).to_string(),
                district: cell(district_at).to_string(),
            };
            if !place.municipality.is_empty() && !record.places.contains(&place) {
                record.places.push(place);
            }
            // An unreadable date only costs the tile its date (the record's informational flight
            // date), never the whole list.
            if let Some(date) = metadata_date(cell(date_at)) {
                record.date.get_or_insert(date);
            }
        }
        if tiles.is_empty() {
            return Err("the metadata list names no tile".to_string());
        }
        let mut undated: Vec<String> = tiles
            .iter()
            .filter(|(_, r)| r.date.is_none())
            .map(|(t, _)| format!("{:03}{:04}", t.east_km, t.north_km))
            .collect();
        undated.sort();
        // No date at all points to a changed format of the list, not to a gap.
        if undated.len() == tiles.len() {
            return Err("the metadata list has no readable Bildflugdatum".to_string());
        }
        Ok(Self { tiles, undated })
    }

    pub fn get(&self, tile: TileId) -> Option<&TileRecord> {
        self.tiles.get(&tile)
    }
}

/// A `Bildflugdatum` as `YYYY-MM-DD`: an Excel serial, or text `DD.MM.YYYY`.
fn metadata_date(cell: &str) -> Option<String> {
    excel_date(cell).or_else(|| {
        chrono::NaiveDate::parse_from_str(cell, "%d.%m.%Y")
            .ok()
            .map(|d| d.format("%Y-%m-%d").to_string())
    })
}

/// The tile of a `Kachelname` (`4765552` → 476/5552).
fn tile_of(name: &str) -> Option<TileId> {
    let number = name.parse::<f64>().ok()?;
    if number.fract() != 0.0 || !(1e6..1e7).contains(&number) {
        return None;
    }
    let number = number as u32;
    Some(TileId {
        east_km: number / 10_000,
        north_km: number % 10_000,
    })
}

/// The words of a name: lower case, umlauts and ß written out, `_ - ( ) , .` as spaces.
fn words(name: &str) -> Vec<String> {
    let mut spelled = String::new();
    for c in name.to_lowercase().chars() {
        match c {
            'ä' => spelled.push_str("ae"),
            'ö' => spelled.push_str("oe"),
            'ü' => spelled.push_str("ue"),
            'ß' => spelled.push_str("ss"),
            '_' | '-' | '(' | ')' | ',' | '.' => spelled.push(' '),
            c => spelled.push(c),
        }
    }
    spelled.split_whitespace().map(str::to_string).collect()
}

/// A key equal for the spellings of one name in the metadata list and in the Downloadcenter
/// ("Main-Taunus" and `Main-Taunus-Kreis`, "Landeshauptstadt Wiesbaden" and `Kreisfreie Stadt
/// Wiesbaden`): [`words`] without "kreisfreie stadt", "landeshauptstadt", "landkreis" and
/// "kreis" (also glued on: `Hochtaunuskreis` → `hochtaunus`).
pub(crate) fn name_key(name: &str) -> String {
    let words = words(name);
    let mut kept: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        match words[i].as_str() {
            "kreisfreie" if words.get(i + 1).is_some_and(|w| w == "stadt") => i += 2,
            "landeshauptstadt" | "landkreis" | "kreis" => i += 1,
            word => {
                // `Hochtaunuskreis`, `Odenwaldkreis`: the district word glued on.
                kept.push(
                    word.strip_suffix("kreis")
                        .filter(|w| !w.is_empty())
                        .unwrap_or(word),
                );
                i += 1;
            }
        }
    }
    kept.join(" ")
}

/// Whether `name` is a city district ("Kreisfreie Stadt …", "Landeshauptstadt …").
fn is_city(name: &str) -> bool {
    let words = words(name);
    words.iter().any(|w| w == "landeshauptstadt")
        || words
            .windows(2)
            .any(|w| w[0] == "kreisfreie" && w[1] == "stadt")
}

/// The municipality key of a municipal file name (`Neustadt_(Hessen)-DGM1.zip` →
/// `neustadt hessen`).
fn file_key(name: &str, model: Model) -> String {
    let stem = name
        .len()
        .checked_sub(4)
        .filter(|&at| name.is_char_boundary(at) && name[at..].eq_ignore_ascii_case(".zip"))
        .map_or(name, |at| &name[..at]);
    let key = name_key(stem);
    match key.rsplit_once(' ') {
        Some((municipality, code)) if code == model.code() => municipality.to_string(),
        _ => key,
    }
}

/// The candidate that names `wanted`: the one with the same [`name_key`] (spaces ignored; of
/// several, the one that is a city district when `wanted` is one, so "Kassel" is the Landkreis),
/// else the only one whose key words contain `wanted`'s or are contained in them (so "Gemarkung
/// Michelbuch (gemeindefrei)" finds `Michelbuch`, but "Nidda" never `Niddatal`).
fn best_match(candidates: &[String], wanted: &str) -> Option<usize> {
    let target = name_key(wanted);
    if target.is_empty() {
        return None;
    }
    let keys: Vec<String> = candidates.iter().map(|c| name_key(c)).collect();
    let compact = |key: &str| key.replace(' ', "");
    let only = |found: Vec<usize>| (found.len() == 1).then(|| found[0]);
    let equal: Vec<usize> = (0..keys.len())
        .filter(|&i| compact(&keys[i]) == compact(&target))
        .collect();
    match equal.len() {
        0 => only(
            (0..keys.len())
                .filter(|&i| {
                    !keys[i].is_empty()
                        && (has_words(&keys[i], &target) || has_words(&target, &keys[i]))
                })
                .collect(),
        ),
        1 => Some(equal[0]),
        _ => only(
            equal
                .into_iter()
                .filter(|&i| is_city(&candidates[i]) == is_city(wanted))
                .collect(),
        ),
    }
}

/// Whether the words of `key` contain those of `part` in a row.
fn has_words(key: &str, part: &str) -> bool {
    let (key, part): (Vec<&str>, Vec<&str>) = (key.split(' ').collect(), part.split(' ').collect());
    key.windows(part.len()).any(|w| w == part.as_slice())
}

/// `text` percent-encoded: unreserved characters kept, `/` and existing `%XX` too in a path,
/// spaces as `+` in a query value.
fn percent_encode(text: &str, path: bool) -> String {
    let bytes = text.as_bytes();
    let mut out = String::new();
    for (i, &b) in bytes.iter().enumerate() {
        let escaped = b == b'%'
            && bytes
                .get(i + 1..i + 3)
                .is_some_and(|h| h.iter().all(u8::is_ascii_hexdigit));
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) || (path && (b == b'/' || escaped)) {
            out.push(char::from(b));
        } else if b == b' ' && !path {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The product tree of `model` (its district nodes and description).
pub fn tree_url(model: Model) -> String {
    format!(
        "{REST_URL}?path={}&navigation=all",
        percent_encode(model.product(), false)
    )
}

/// Page `page` (from 1) of the file listing of `district` under `product`.
pub fn listing_url(product: &str, district: &str, page: u64) -> String {
    let url = format!(
        "{REST_URL}?path={}",
        percent_encode(&format!("{product}/{district}"), false)
    );
    if page > 1 {
        format!("{url}&page={page}")
    } else {
        url
    }
}

/// The URL of a server path from a REST answer (`/downloadcenter/…/Wiesbaden - DGM1.zip`);
/// `None` for anything but a path on the Downloadcenter's host.
pub fn file_url(path: &str) -> Option<String> {
    let path = path.strip_prefix(HOST).unwrap_or(path);
    (path.starts_with('/') && !path.starts_with("//"))
        .then(|| format!("{HOST}{}", percent_encode(path, true)))
}

fn answer(bytes: &[u8]) -> Result<Value, String> {
    let answer: Value =
        serde_json::from_slice(bytes).map_err(|e| format!("not a Downloadcenter answer: {e}"))?;
    if !answer["searchresult"].is_object() {
        return Err("not a Downloadcenter answer: no searchresult".to_string());
    }
    Ok(answer)
}

/// A linked document: its URL and a safe file name.
type Link = (String, String);

/// The district nodes of a product tree answer and the metadata list linked in the product
/// description.
fn parse_tree(bytes: &[u8]) -> Result<(Vec<String>, Option<Link>), String> {
    let answer = answer(bytes)?;
    let product = &answer["searchresult"]["product"];
    if product["id"].is_null() {
        return Err("the product tree answer has no product id".to_string());
    }
    let nodes = answer["navigation"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    let below = |node: &&Value| node["parentId"] == product["id"];
    let districts: Vec<String> = nodes
        .iter()
        .filter(below)
        .filter_map(|n| n["name"].as_str().map(str::to_string))
        .collect();
    if districts.is_empty() {
        return Err("the product tree lists no district".to_string());
    }
    let description = product["description"].as_str().unwrap_or_default();
    Ok((districts, metadata_link(description)))
}

/// The metadata list (`…/Metadaten_DGM-DOM_<Monat>-<Jahr>.xlsx`) linked in a product
/// description: its URL and a file name for its release.
fn metadata_link(description: &str) -> Option<Link> {
    let href = description
        .split("href=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .find(|href| {
            let file = href.rsplit('/').next().unwrap_or_default().to_lowercase();
            file.starts_with("metadaten") && file.ends_with(".xlsx")
        })?;
    let name: String = href
        .rsplit('/')
        .next()?
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    Some((file_url(href)?, name))
}

/// The files of a district listing: name and URL.
pub type Files = Vec<(String, String)>;

/// One listing page: its files (name, URL), the number of files in all (if told), per page,
/// and on this page (any kind).
fn parse_listing(bytes: &[u8]) -> Result<(Files, Option<u64>, u64, u64), String> {
    let answer = answer(bytes)?;
    let result = &answer["searchresult"];
    let downloads = result["downloads"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    let files = downloads
        .iter()
        .filter(|d| d["downloadType"].as_str().is_none_or(|t| t == "File"))
        .map(|d| {
            let name = d["name"].as_str();
            let url = d["downloadLink"]["uri"].as_str().and_then(file_url);
            match (name, url) {
                (Some(name), Some(url)) => Ok((name.to_string(), url)),
                _ => Err(format!("a download without name or link: {d}")),
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    let paging = &result["paging"];
    Ok((
        files,
        paging["total"].as_u64(),
        paging["pageSize"].as_u64().filter(|&n| n > 0).unwrap_or(20),
        downloads.len() as u64,
    ))
}

/// The files (name, URL) of `district` under `product`, page by page. The URLs carry a date
/// and expire after a day or two, so they are never cached.
pub fn downloadcenter_files(
    fetcher: &dyn Fetcher,
    product: &str,
    district: &str,
) -> Result<Files, FetchError> {
    let mut files = Vec::new();
    // The page count from `paging.total`; without it, pages are read until a short one.
    let mut pages = None;
    let mut previous: Option<Files> = None;
    for page in 1..=MAX_PAGES {
        let url = listing_url(product, district, page);
        let (found, total, size, count) = get_with_retries(fetcher, &url, |b| {
            parse_listing(b).map_err(|m| refused(&url, m))
        })?;
        if !found.is_empty() && previous.as_ref() == Some(&found) {
            return Err(refused(
                &url,
                format!("page {page} repeats page {}; the paging changed", page - 1),
            ));
        }
        if page == 1 {
            pages = total.map(|t| t.div_ceil(size).max(1));
        }
        files.extend(found.iter().cloned());
        previous = Some(found);
        let more = match pages {
            Some(pages) => page < pages,
            None => count >= size,
        };
        if !more {
            return Ok(files);
        }
    }
    Err(refused(
        &listing_url(product, district, 1),
        format!("the listing has more than {MAX_PAGES} pages"),
    ))
}

fn offline_hint(what: &str) -> FetchError {
    refused(
        REST_URL,
        format!("offline and not cached: {what}; run `freemapper fetch` online"),
    )
}

/// A municipal zip: where it is (the cached directory offline) and its members.
struct Zip {
    url: String,
    members: Vec<Member>,
}

/// One [`fetch_model`] run: the metadata list, and each listing and zip directory read once.
struct Session<'a> {
    fetcher: &'a dyn Fetcher,
    mode: FetchMode,
    model: Model,
    dir: PathBuf,
    index: Rc<TileIndex>,
    /// Problems with the metadata list (see [`HeightModel::warnings`]).
    warnings: Vec<Note>,
    /// The district nodes of the product tree (online only).
    districts: Vec<String>,
    listings: HashMap<String, Rc<Files>>,
    /// The zip of each place looked up, `None` for a place without a district or file.
    zips: HashMap<String, Option<Rc<Zip>>>,
    /// The places whose zip could not be read in this run, with the error.
    failed: HashMap<String, FetchError>,
    /// The places without a district or file, as `<municipality> (<district>)`.
    unresolved: Vec<String>,
    /// The cache keys of those places.
    unresolved_keys: Vec<String>,
}

/// The metadata list and the DGM1 product tree of the first [`fetch_model_with`] run, kept for
/// the next one (the DOM1 run after the DGM1 run) so each is read and parsed once.
#[derive(Default)]
pub struct MetadataMemo {
    loaded: Option<(Rc<TileIndex>, Vec<Note>)>,
    /// The DGM1 tree's districts and the metadata link of its description.
    tree: Option<(Vec<String>, Option<Link>)>,
}

/// The kept metadata lists in `dir`, newest first.
fn kept_lists(dir: &Path) -> Vec<PathBuf> {
    let mut lists: Vec<(Option<std::time::SystemTime>, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".xlsx") && e.path().is_file())
        .map(|e| (e.metadata().and_then(|m| m.modified()).ok(), e.path()))
        .collect();
    lists.sort_by_key(|a| std::cmp::Reverse(a.0));
    lists.into_iter().map(|(_, path)| path).collect()
}

/// The metadata list (`Metadaten_DGM-DOM_<Monat>-<Jahr>.xlsx`) kept in `dir` under its release
/// name: reused while the DGM1 product description links the same release (`link`), the older
/// ones removed once a newer one parsed. A new list that cannot be downloaded or does not parse
/// falls back to the newest kept one with a warning. Offline the newest kept one.
fn metadata_list(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    dir: &Path,
    link: Option<Link>,
) -> Result<(TileIndex, Vec<Note>), FetchError> {
    let read_index = |bytes: &[u8]| TileIndex::from_rows(&xlsx_rows(bytes)?);
    if mode == FetchMode::Offline {
        let newest = kept_lists(dir)
            .into_iter()
            .next()
            .ok_or_else(|| offline_hint("HVBG DGM/DOM metadata list"))?;
        let bytes = std::fs::read(&newest).map_err(|e| FetchError::Io(newest.clone(), e))?;
        let index = read_index(&bytes).map_err(|m| refused(&newest.display().to_string(), m))?;
        return Ok((index, Vec::new()));
    }
    // Both models take the list from the DGM1 description, as the plan says.
    let (url, name) = link.ok_or_else(|| {
        refused(
            &tree_url(Model::Dgm1),
            "the product description links no metadata list (Metadaten_….xlsx)",
        )
    })?;
    let file = dir.join(&name);
    if let Some(index) = std::fs::read(&file).ok().and_then(|b| read_index(&b).ok()) {
        return Ok((index, Vec::new()));
    }
    // A list that cannot be downloaded or does not parse never replaces a good one; the
    // newest kept one (as offline mode would use it) stands in.
    let fall_back = |message: String| {
        for older in kept_lists(dir).into_iter().filter(|p| *p != file) {
            let Some(index) = std::fs::read(&older).ok().and_then(|b| read_index(&b).ok()) else {
                continue;
            };
            let old_name = older.file_name().unwrap_or_default().to_string_lossy();
            let warning = note!(
                "warning-hvbg-list-unreadable-older",
                name = &name,
                detail = &message,
                older = old_name.to_string()
            );
            return Some((index, vec![warning]));
        }
        None
    };
    let bytes = match get_with_retries(fetcher, &url, |b| Ok(b.to_vec())) {
        Ok(bytes) => bytes,
        Err(e) => return fall_back(e.to_string()).ok_or(e),
    };
    let index = match read_index(&bytes) {
        Ok(index) => index,
        Err(message) => {
            return fall_back(message.clone()).ok_or_else(|| refused(&url, message));
        }
    };
    // Kept only once it parsed, so a broken answer never replaces a good copy.
    write_cache_file(&file, &bytes)?;
    // Only the older lists go; whatever else lies there (another run's partial file) stays.
    for old in kept_lists(dir).into_iter().filter(|p| *p != file) {
        let _ = std::fs::remove_file(old);
    }
    Ok((index, Vec::new()))
}

/// `e` once more, for a place that failed earlier in the run: the same kind with the same URL
/// or path (only the kinds a zip read cannot produce keep just their text).
fn again(e: &FetchError) -> FetchError {
    match e {
        FetchError::Http { url, message } => FetchError::Http {
            url: url.clone(),
            message: message.clone(),
        },
        FetchError::Io(path, error) => FetchError::Io(
            path.clone(),
            std::io::Error::new(error.kind(), error.to_string()),
        ),
        FetchError::CorruptArchive { url, message } => FetchError::CorruptArchive {
            url: url.clone(),
            message: message.clone(),
        },
        FetchError::CorruptExtract { url, message } => FetchError::CorruptExtract {
            url: url.clone(),
            message: message.clone(),
        },
        FetchError::NotFound(url) => FetchError::NotFound(url.clone()),
        FetchError::Refused {
            url,
            status,
            detail,
        } => FetchError::Refused {
            url: url.clone(),
            status: *status,
            detail: detail.clone(),
        },
        FetchError::TooLarge { url, limit_mb } => FetchError::TooLarge {
            url: url.clone(),
            limit_mb: *limit_mb,
        },
        FetchError::NotStored { label, path } => FetchError::NotStored {
            label: label.clone(),
            path: path.clone(),
        },
        FetchError::RangeIgnored { url, range, status } => FetchError::RangeIgnored {
            url: url.clone(),
            range: range.clone(),
            status: *status,
        },
        FetchError::TilesNotCovered(tiles) => FetchError::TilesNotCovered(tiles.clone()),
        FetchError::MissingOffline(product, tiles) => {
            FetchError::MissingOffline(*product, tiles.clone())
        }
        FetchError::OsmExtractMissing => FetchError::OsmExtractMissing,
        FetchError::Cancelled => FetchError::Cancelled,
        FetchError::CurrencyCsv(_) | FetchError::Alkis(_) => refused(REST_URL, e.to_string()),
    }
}

impl<'a> Session<'a> {
    /// The metadata list and, online, the product tree's districts; what `memo` holds is not
    /// read again.
    fn open(
        fetcher: &'a dyn Fetcher,
        cache: &CacheDir,
        mode: FetchMode,
        model: Model,
        memo: &mut MetadataMemo,
    ) -> Result<Self, FetchError> {
        let read_tree = |model: Model| {
            let url = tree_url(model);
            get_with_retries(fetcher, &url, |b| {
                parse_tree(b).map_err(|m| refused(&url, m))
            })
        };
        let mut districts = Vec::new();
        if mode == FetchMode::Online && (memo.loaded.is_none() || model == Model::Dgm1) {
            if memo.tree.is_none() {
                memo.tree = Some(read_tree(Model::Dgm1)?);
            }
            if model == Model::Dgm1 {
                districts = memo.tree.as_ref().map(|t| t.0.clone()).unwrap_or_default();
            }
        }
        if mode == FetchMode::Online && model != Model::Dgm1 {
            districts = read_tree(model)?.0;
        }
        if memo.loaded.is_none() {
            let link = memo.tree.as_ref().and_then(|t| t.1.clone());
            let dir = cache.provider_dir("hvbg").join("metadata");
            let (index, mut warnings) = metadata_list(fetcher, mode, &dir, link)?;
            if !index.undated.is_empty() {
                let shown: Vec<&str> = index.undated.iter().take(5).map(String::as_str).collect();
                let more = index.undated.len() - shown.len();
                warnings.push(if more > 0 {
                    note!(
                        "warning-hvbg-undated-more",
                        tiles = shown.join(", "),
                        more = more
                    )
                } else {
                    note!("warning-hvbg-undated", tiles = shown.join(", "))
                });
            }
            memo.loaded = Some((Rc::new(index), warnings));
        }
        let (index, warnings) = memo.loaded.clone().expect("loaded above");
        Ok(Self {
            fetcher,
            mode,
            model,
            dir: cache.provider_dir("hvbg").join(model.code()),
            index,
            warnings,
            districts,
            listings: HashMap::new(),
            zips: HashMap::new(),
            failed: HashMap::new(),
            unresolved: Vec::new(),
            unresolved_keys: Vec::new(),
        })
    }

    /// The flight date of `tile`.
    fn date(&self, tile: TileId) -> Option<String> {
        self.index.get(tile).and_then(|r| r.date.clone())
    }

    /// The GeoTIFF of `tile` from the first zip of its municipalities that holds it (zips whose
    /// directory is already read first, those that failed last); `None` when every zip was read
    /// (places without a zip skipped) and none holds it. A zip that could not be read is an error
    /// unless another one has the tile, so a server failure never turns into a silently empty
    /// tile.
    fn tile(&mut self, tile: TileId) -> Result<Option<Vec<u8>>, FetchError> {
        let Some(record) = self.index.get(tile) else {
            return Ok(None);
        };
        let mut places = record.places.clone();
        places.sort_by_key(|p| {
            let key = p.cache_key();
            (
                self.failed.contains_key(&key),
                !self.zips.contains_key(&key),
            )
        });
        let prefix = format!(
            "{}_32_{:03}_{:04}_",
            self.model.code(),
            tile.east_km,
            tile.north_km
        );
        let mut failure = None;
        for place in &places {
            let zip = match self.zip(place) {
                Ok(Some(zip)) => zip,
                // No zip of this place exists: other places may still hold the tile.
                Ok(None) => continue,
                Err(e) => {
                    failure.get_or_insert(e);
                    continue;
                }
            };
            let Some(member) = find_member(&zip.members, |file| {
                file.starts_with(&prefix) && file.ends_with(".tif")
            }) else {
                continue;
            };
            // Named after the tile, never after the member (whose name comes from the server).
            let file = self
                .dir
                .join(format!("32_{:03}_{:04}.tif", tile.east_km, tile.north_km));
            let label = self.model.label();
            return cached_member(self.fetcher, self.mode, &zip.url, member, &file, || {
                offline_hint(&format!("{label} {}", member.name))
            })
            .map(Some);
        }
        failure.map_or(Ok(None), Err)
    }

    /// The zip of `place`, its directory read once per run and kept for offline runs; `None`
    /// for a place without a district or file in the Downloadcenter (noted in
    /// [`Self::unresolved`]). A place that failed is not tried again in the run.
    fn zip(&mut self, place: &Place) -> Result<Option<Rc<Zip>>, FetchError> {
        let key = place.cache_key();
        if let Some(zip) = self.zips.get(&key) {
            return Ok(zip.clone());
        }
        if let Some(e) = self.failed.get(&key) {
            return Err(again(e));
        }
        let zip = self.read_zip(place, &key).inspect_err(|e| {
            self.failed.insert(key.clone(), again(e));
        })?;
        let zip = zip.map(Rc::new);
        if zip.is_none() {
            let name = place_label(&place.municipality, &place.district);
            if !self.unresolved.contains(&name) {
                self.unresolved.push(name);
            }
            self.unresolved_keys.push(key.clone());
        }
        self.zips.insert(key, zip.clone());
        Ok(zip)
    }

    fn read_zip(&mut self, place: &Place, key: &str) -> Result<Option<Zip>, FetchError> {
        // The member count (u16, little-endian), then the central directory; and a marker that
        // the place had no zip in the last online run, so offline runs know it too (a cached
        // directory wins over the marker).
        let file = self.dir.join("zips").join(format!("{key}.bin"));
        let no_zip = self.dir.join("zips").join(format!("{key}.none"));
        let remove = |path: &Path| match std::fs::remove_file(path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(FetchError::Io(path.to_path_buf(), e))
            }
            _ => Ok(()),
        };
        Ok(match self.mode {
            FetchMode::Online => {
                let Some(url) = self.zip_url(place)? else {
                    // A cached directory is kept: the listing may lack the file only for a
                    // while, and offline runs prefer the directory over the marker.
                    write_cache_file(&no_zip, b"")?;
                    return Ok(None);
                };
                let (directory, count) = read_directory_bytes(self.fetcher, &url)?;
                let members = parse_directory(&directory, count).map_err(|m| refused(&url, m))?;
                // Kept only once it parsed, so a broken answer never replaces a good copy.
                write_cache_file(&file, &[&count.to_le_bytes()[..], &directory].concat())?;
                remove(&no_zip)?;
                Some(Zip { url, members })
            }
            FetchMode::Offline => {
                if !file.exists() && no_zip.exists() {
                    return Ok(None);
                }
                let hint =
                    || offline_hint(&format!("{} of {}", self.model.label(), place.municipality));
                let cached = std::fs::read(&file).map_err(|_| hint())?;
                let (count, directory) = cached.split_at_checked(2).ok_or_else(hint)?;
                let url = file.display().to_string();
                let members = parse_directory(directory, u16::from_le_bytes([count[0], count[1]]))
                    .map_err(|m| refused(&url, m))?;
                Some(Zip { url, members })
            }
        })
    }

    /// The current URL of the zip of `place`: its district in the product tree, then its file
    /// in the district's listing; `None` when either has no unique match.
    fn zip_url(&mut self, place: &Place) -> Result<Option<String>, FetchError> {
        let product = self.model.product();
        let Some(district) = best_match(&self.districts, &place.district) else {
            return Ok(None);
        };
        let district = self.districts[district].clone();
        let files = match self.listings.get(&district) {
            Some(files) => Rc::clone(files),
            None => {
                let files = Rc::new(downloadcenter_files(self.fetcher, product, &district)?);
                self.listings.insert(district.clone(), Rc::clone(&files));
                files
            }
        };
        let keys: Vec<String> = files
            .iter()
            .map(|(name, _)| file_key(name, self.model))
            .collect();
        Ok(best_match(&keys, &place.municipality).map(|at| files[at].1.clone()))
    }
}

/// The flight date (`DD.MM.YYYY`) of the orthophoto at the centre of `bbox` (EPSG:25832), by a
/// GetFeatureInfo on the info layer; `None` outside the coverage or when the answer has none.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    let (e, n) = (
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    let url = format!(
        "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo&LAYERS={WMS_INFO_LAYER}\
         &QUERY_LAYERS={WMS_INFO_LAYER}&STYLES=&CRS=EPSG:{}&BBOX={},{},{},{}\
         &WIDTH=100&HEIGHT=100&I=50&J=50&INFO_FORMAT=text/plain",
        WMS.url,
        WMS.crs.epsg(),
        crate::sources::wms::mm(e - 100.0),
        crate::sources::wms::mm(n - 100.0),
        crate::sources::wms::mm(e + 100.0),
        crate::sources::wms::mm(n + 100.0),
    );
    let bytes = get_with_retries(fetcher, &url, |b| Ok(b.to_vec())).ok()?;
    parse_flight_date(&String::from_utf8_lossy(&bytes))
}

/// `bildflug = 'DD-MM-YYYY'` of a GetFeatureInfo text answer, as `DD.MM.YYYY`.
fn parse_flight_date(text: &str) -> Option<String> {
    let rest = text.split("bildflug").nth(1)?;
    let value = rest.split('\'').nth(1)?;
    let parts: Vec<&str> = value.split('-').collect();
    let [d, m, y] = parts[..] else { return None };
    let digits = |s: &str, n| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    (digits(d, 2) && digits(m, 2) && digits(y, 4)).then(|| format!("{d}.{m}.{y}"))
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), read out of the Downloadcenter's
/// municipal zips and cached, mosaicked into a Float32 GeoTIFF `out` with its world file.
///
/// The metadata list (its URL from the product description) names each tile's municipalities;
/// their files are looked up in the district listings right before reading, since the file URLs
/// expire. Tiles outside Hesse stay empty; returned are the missing tiles inside Hesse and the
/// flight date of the centre tile. No tile at all is an error.
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    fetch_model_with(
        fetcher,
        cache,
        mode,
        model,
        bbox,
        root,
        out,
        &mut MetadataMemo::default(),
    )
}

/// [`fetch_model`] sharing the metadata list and the DGM1 product tree with other runs through
/// `memo`.
#[allow(clippy::too_many_arguments)]
pub fn fetch_model_with(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
    memo: &mut MetadataMemo,
) -> Result<HeightModel, FetchError> {
    let mut session = Session::open(fetcher, cache, mode, model, memo)?;
    let grid = GridSpec::covering(bbox, 1.0);
    let mut raster = Raster::filled(grid, f32::NAN);
    let (mut found, mut missing) = (0, Vec::new());
    for tile in tiles_covering(bbox) {
        let Some(bytes) = session.tile(tile)? else {
            // A tile the list names is Hesse's even where its centre lies across the border.
            if tile_centre_in(tile, HESSEN) || session.index.get(tile).is_some() {
                missing.push(tile);
            }
            continue;
        };
        found += 1;
        let name = format!(
            "{} tile 32_{:03}_{:04}",
            model.label(),
            tile.east_km,
            tile.north_km
        );
        let part =
            decode_float_tiff(&bytes).map_err(|m| refused(REST_URL, format!("{name}: {m}")))?;
        raster
            .paste_with(&part.grid, |x, y| part.get(x, y))
            .map_err(|e| refused(REST_URL, format!("{name}: {e}")))?;
    }
    if found == 0 {
        return Err(refused(
            REST_URL,
            format!(
                "no tile of the {} covers the map area; the Downloadcenter may have changed",
                model.label()
            ),
        ));
    }
    write_float_raster(&raster, root, out, REST_URL)?;
    let centre = TileId {
        east_km: ((bbox.min_e + bbox.max_e) / 2000.0).floor() as u32,
        north_km: ((bbox.min_n + bbox.max_n) / 2000.0).floor() as u32,
    };
    let unresolved_empty = missing
        .iter()
        .copied()
        .filter(|&tile| {
            session.index.get(tile).is_some_and(|r| {
                r.places
                    .iter()
                    .any(|p| session.unresolved_keys.contains(&p.cache_key()))
            })
        })
        .collect();
    Ok(HeightModel {
        missing,
        date: session.date(centre),
        unresolved: session.unresolved,
        unresolved_empty,
        warnings: session.warnings,
    })
}

/// The provider id of the adapters.
const PROVIDER: &str = "hvbg";

/// The rasters of HVBG (Hesse): requested for the area and stored in the project folder.
pub struct HvbgBackend {
    info: ProviderInfo,
}

impl HvbgBackend {
    pub fn new() -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some("hvbg".to_string()),
                terrain_source: "hvbg-dgm1".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

impl Default for HvbgBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for HvbgBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        fetch_rasters(env, project, bbox)
    }
}

/// The HVBG rasters of `bbox` (EPSG:25832): the orthophoto from the WMS, DGM1 and DOM1 read out
/// of the Downloadcenter's municipal zips. The DOM1 is only a template: an unavailable one costs
/// a warning.
fn fetch_rasters(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
) -> Result<(), AdapterError> {
    let (fetcher, mode) = (env.fetcher, env.mode);
    let cache = env.cache;
    let dir = &project.dir().to_path_buf();
    project.records().tiles.clear();
    project.records().ndsm_tiles.clear();
    warn_beyond_state(env, project, PROVIDER, "HVBG");
    let flight = || flight_date(fetcher, bbox);
    fetch_wms_orthophoto(
        env,
        project,
        bbox,
        (&OrthophotoSource::Wms(WMS), "HVBG", &flight, None),
    )?;
    // What both models report alike is said once, after both: the places without a
    // Downloadcenter file (with the tiles that stayed empty for want of another file) and
    // the metadata list's problems.
    let mut unresolved: Vec<String> = Vec::new();
    let mut unresolved_empty: Vec<TileId> = Vec::new();
    let mut list_warnings: Vec<Note> = Vec::new();
    let mut memo = MetadataMemo::default();
    fetch_area_heights(
        env,
        project,
        bbox,
        "Hesse",
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, _| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            fetch_model_with(fetcher, cache, mode, model, bbox, dir, out, &mut memo)
        },
        &mut |_, found, _, _| {
            for place in &found.unresolved {
                if !unresolved.contains(place) {
                    unresolved.push(place.clone());
                }
            }
            for tile in &found.unresolved_empty {
                if !unresolved_empty.contains(tile) {
                    unresolved_empty.push(*tile);
                }
            }
            for warning in &found.warnings {
                if !list_warnings.contains(warning) {
                    list_warnings.push(warning.clone());
                }
            }
        },
    )?;
    for warning in &list_warnings {
        (env.progress)(warning.clone());
    }
    if !unresolved.is_empty() {
        let places: Vec<String> = unresolved.iter().map(|p| format!("\"{p}\"")).collect();
        (env.progress)(if unresolved_empty.is_empty() {
            note!("warning-downloadcenter-no-file", places = places.join(", "))
        } else {
            note!(
                "warning-downloadcenter-no-file-empty",
                places = places.join(", "),
                count = unresolved_empty.len(),
                names = crate::sources::lgv::names(&unresolved_empty)
            )
        });
    }
    project.save()?;
    Ok(())
}

#[cfg(any(test, feature = "testing"))]
pub mod test_support {
    use crate::sources::xlsx::test_support::{escape, workbook};
    use crate::tiles::TileId;

    /// The metadata list the product descriptions of the fixtures link.
    pub const METADATA_URL: &str = "https://gds.hessen.de/INTERSHOP/static/WFS/HLBG-Geodaten-Site/-/HLBG-Geodaten/de_DE/Downloadcenter/Daten/3D-Daten/Metadaten_DGM-DOM_August-2026.xlsx";
    pub const DGM1_TREE: &str = include_str!("../../../../tests/fixtures/hessen-dc-dgm1-tree.json");
    pub const DGM1_FRANKFURT: &str =
        include_str!("../../../../tests/fixtures/hessen-dc-dgm1-frankfurt.json");
    pub const DGM1_WIESBADEN: &str =
        include_str!("../../../../tests/fixtures/hessen-dc-dgm1-wiesbaden.json");
    pub const DOP_MARBURG: [&str; 2] = [
        include_str!("../../../../tests/fixtures/hessen-dc-dop-marburg-p1.json"),
        include_str!("../../../../tests/fixtures/hessen-dc-dop-marburg-p2.json"),
    ];

    /// The DGM/DOM metadata list with one row per (tile, flight date as Excel serial,
    /// municipality, district), laid out like the August 2026 release.
    pub fn metadata_xlsx(rows: &[(TileId, u32, &str, &str)]) -> Vec<u8> {
        let mut strings: Vec<String> = [
            "Aktualitä: 5.08.2026",
            "Kachelname",
            "Bildflugdatum",
            "Sensor",
            "Gemeinde",
            "Kreis",
            "Regierungsbezirk",
            "Riegl VQ-780i",
            "Darmstadt",
        ]
        .map(str::to_string)
        .to_vec();
        let mut index = |text: &str| {
            let at = strings.iter().position(|s| s == text).unwrap_or_else(|| {
                strings.push(text.to_string());
                strings.len() - 1
            });
            format!(r#"t="s"><v>{at}</v>"#)
        };
        let mut sheet = String::from(
            r#"<row r="1"><c r="A1" s="3" t="s"><v>0</v></c><c r="B1" s="3"/></row><row r="2">"#,
        );
        for (i, column) in ["A", "B", "C", "D", "E", "F"].iter().enumerate() {
            sheet.push_str(&format!(r#"<c r="{column}2" t="s"><v>{}</v></c>"#, i + 1));
        }
        sheet.push_str("</row>");
        for (n, (tile, serial, municipality, district)) in rows.iter().enumerate() {
            let r = n + 3;
            let (sensor, m, d, rb) = (
                index("Riegl VQ-780i"),
                index(municipality),
                index(district),
                index("Darmstadt"),
            );
            sheet.push_str(&format!(
                r#"<row r="{r}"><c r="A{r}"><v>{:03}{:04}</v></c><c r="B{r}" s="1"><v>{serial}</v></c><c r="C{r}" {sensor}</c><c r="D{r}" {m}</c><c r="E{r}" {d}</c><c r="F{r}" {rb}</c></row>"#,
                tile.east_km, tile.north_km
            ));
        }
        let shared: String = strings
            .iter()
            .map(|s| format!("<si><t>{}</t></si>", escape(s)))
            .collect();
        workbook(&sheet, &shared)
    }

    /// A product tree answer listing `districts`, whose description links [`METADATA_URL`] when
    /// `link`.
    pub fn tree_answer(districts: &[&str], link: bool) -> Vec<u8> {
        let navigation: Vec<serde_json::Value> = districts
            .iter()
            .map(|d| serde_json::json!({"name": d, "id": format!("DC{d}"), "level": 3, "parentId": "DC0100057"}))
            .collect();
        let path = METADATA_URL.strip_prefix("https://gds.hessen.de").unwrap();
        serde_json::json!({
            "navigation": navigation,
            "searchresult": {
                "product": {
                    "name": "Digitales Geländemodell (DGM1)",
                    "id": "DC0100057",
                    "description": if link {
                        format!(r#"<ul><li><a href="{path}" target="_blank">metadatenliste</a></li></ul>"#)
                    } else {
                        "<b>Digitales Oberflächenmodell (DOM1)</b>".to_string()
                    },
                },
                "downloads": [],
                "paging": {"total": 0, "pageSize": 20, "page": 1},
            }
        })
        .to_string()
        .into_bytes()
    }

    /// A one-page district listing of `files` (name, `downloadLink.uri`).
    pub fn listing_answer(files: &[(&str, &str)]) -> Vec<u8> {
        listing_page(files, Some(files.len()))
    }

    /// A listing page of `files` whose paging tells `total` files (or nothing).
    pub fn listing_page(files: &[(&str, &str)], total: Option<usize>) -> Vec<u8> {
        let downloads: Vec<serde_json::Value> = files
            .iter()
            .map(|(name, uri)| serde_json::json!({"name": name, "downloadType": "File", "downloadLink": {"uri": uri}}))
            .collect();
        serde_json::json!({
            "searchresult": {
                "packages": [],
                "product": {"name": "district"},
                "downloads": downloads,
                "paging": match total {
                    Some(total) => serde_json::json!({"total": total, "pageSize": 20, "page": 1}),
                    None => serde_json::json!({"pageSize": 20}),
                },
            }
        })
        .to_string()
        .into_bytes()
    }

    /// A 1 km tile of 1 m pixels at `tile`, every value `value` plus `slope` per metre east.
    pub fn tile_tiff(tile: TileId, value: f32, slope: f32) -> Vec<u8> {
        let grid = crate::raster::GridSpec {
            west: f64::from(tile.east_km) * 1000.0,
            north: f64::from(tile.north_km + 1) * 1000.0,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        let values: Vec<f32> = (0..1000 * 1000)
            .map(|i| value + (i % 1000) as f32 * slope)
            .collect();
        crate::fetch::test_support::geotiff(&grid, &values)
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::Cache;
    use crate::fetch::test_support::MapFetcher;
    use crate::sources::zip_range::test_support::archive;

    fn tile(east_km: u32, north_km: u32) -> TileId {
        TileId { east_km, north_km }
    }

    fn strings(row: &[&str]) -> Vec<String> {
        row.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn tile_index_gives_municipality_and_date() {
        let xlsx = metadata_xlsx(&[
            (
                tile(476, 5552),
                44985,
                "Frankfurt am Main",
                "Kreisfreie Stadt Frankfurt am Main",
            ),
            (
                tile(445, 5555),
                44629,
                "Wiesbaden",
                "Landeshauptstadt Wiesbaden",
            ),
            (tile(445, 5555), 44629, "Niedernhausen", "Rheingau-Taunus"),
            (tile(445, 5555), 44629, "Taunusstein", "Rheingau-Taunus"),
        ]);
        let index = TileIndex::from_rows(&xlsx_rows(&xlsx).unwrap()).unwrap();
        let school = index.get(tile(476, 5552)).unwrap();
        assert_eq!(
            school.places,
            [Place {
                municipality: "Frankfurt am Main".to_string(),
                district: "Kreisfreie Stadt Frankfurt am Main".to_string(),
            }]
        );
        assert_eq!(school.date.as_deref(), Some("2023-02-28"));
        let shared: Vec<&str> = index
            .get(tile(445, 5555))
            .unwrap()
            .places
            .iter()
            .map(|p| p.municipality.as_str())
            .collect();
        assert_eq!(shared, ["Wiesbaden", "Niedernhausen", "Taunusstein"]);
        assert!(index.get(tile(477, 5552)).is_none());
        let headless = vec![strings(&["4765552", "44985"])];
        assert!(TileIndex::from_rows(&headless).is_err());
    }

    #[test]
    fn district_and_municipality_names_match() {
        assert_eq!(name_key("Main-Taunus-Kreis"), "main taunus");
        assert_eq!(name_key("Main-Taunus"), "main taunus");
        assert_eq!(name_key("Kreisfreie Stadt Wiesbaden"), "wiesbaden");
        assert_eq!(name_key("Landeshauptstadt Wiesbaden"), "wiesbaden");
        assert_eq!(name_key("Landkreis Groß-Gerau"), "gross gerau");
        assert_eq!(name_key("Odenwaldkreis"), "odenwald");
        assert_eq!(name_key("Hochtaunuskreis"), "hochtaunus");
        assert_eq!(
            file_key("Neustadt_(Hessen)-DGM1.zip", Model::Dgm1),
            "neustadt hessen"
        );
        assert_eq!(name_key("Neustadt (Hessen)"), "neustadt hessen");
        assert_eq!(file_key("Wiesbaden - DGM1.zip", Model::Dgm1), "wiesbaden");
        assert_eq!(
            file_key("Frankfurt_am_Main-DOM1", Model::Dom1),
            "frankfurt am main"
        );
        let tree: serde_json::Value = serde_json::from_str(DGM1_TREE).unwrap();
        let districts: Vec<String> = tree["navigation"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["name"].as_str().unwrap().to_string())
            .collect();
        let district = |xlsx: &str| best_match(&districts, xlsx).map(|i| districts[i].as_str());
        assert_eq!(district("Main-Taunus"), Some("Main-Taunus-Kreis"));
        assert_eq!(
            district("Landeshauptstadt Wiesbaden"),
            Some("Kreisfreie Stadt Wiesbaden")
        );
        // The district and the city of Kassel both reduce to "kassel".
        assert_eq!(district("Kassel"), Some("Landkreis Kassel"));
        assert_eq!(
            district("Kreisfreie Stadt Kassel"),
            Some("Kreisfreie Stadt Kassel")
        );
        assert_eq!(district("Hochtaunus"), Some("Hochtaunuskreis"));
        assert_eq!(district("Vogelsberg"), None);
        let files: Vec<String> = [
            "Bad_Homburg_v.d._Höhe-DGM1",
            "Neustadt_(Hessen)-DGM1",
            "Wiesbaden - DGM1",
        ]
        .iter()
        .map(|f| file_key(f, Model::Dgm1))
        .collect();
        assert_eq!(best_match(&files, "Bad Homburg v. d. Höhe"), Some(0));
        assert_eq!(best_match(&files, "Neustadt (Hessen)"), Some(1));
        assert_eq!(best_match(&files, "Wiesbaden"), Some(2));
        assert_eq!(best_match(&files, "Taunusstein"), None);
        // Containment only on whole words: Nidda is not Niddatal.
        let wetterau: Vec<String> = ["Niddatal - DGM1", "Gemarkung Michelbuch - DGM1"]
            .iter()
            .map(|f| file_key(f, Model::Dgm1))
            .collect();
        assert_eq!(best_match(&wetterau, "Nidda"), None);
        assert_eq!(best_match(&wetterau, "Niddatal"), Some(0));
        assert_eq!(
            best_match(&wetterau, "Gemarkung Michelbuch (gemeindefrei)"),
            Some(1)
        );
        let both: Vec<String> = vec!["niddatal".to_string(), "nidda".to_string()];
        assert_eq!(best_match(&both, "Nidda"), Some(1));
    }

    #[test]
    fn listing_pages_are_followed() {
        let product = "Luftbildinformationen/Digitale Orthophotos DOP20";
        let district = "Landkreis Marburg-Biedenkopf";
        assert_eq!(
            listing_url(product, district, 1),
            "https://gds.hessen.de/INTERSHOP/rest/WFS/HLBG-Geodaten-Site/-/downloadcenter?path=Luftbildinformationen%2FDigitale+Orthophotos+DOP20%2FLandkreis+Marburg-Biedenkopf"
        );
        let fetcher = MapFetcher::default();
        for (page, body) in [(1, DOP_MARBURG[0]), (2, DOP_MARBURG[1])] {
            fetcher.insert(
                &listing_url(product, district, page),
                body.as_bytes().to_vec(),
            );
        }
        let files = downloadcenter_files(&fetcher, product, district).unwrap();
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            [
                "Amöneburg - DOP20",
                "Kirchhain-DOP20",
                "Neustadt_(Hessen)-DOP20",
                "Weimar (Lahn) - DOP20",
                "Wetter (Hessen) - DOP20",
                "Wohratal - DOP20"
            ]
        );
        assert_eq!(
            files[0].1,
            "https://gds.hessen.de/downloadcenter/20261108/Luftbildinformationen/Digitale%20Orthophotos%20DOP20/Landkreis%20Marburg-Biedenkopf/Am%C3%B6neburg%20-%20DOP20.zip"
        );
        assert_eq!(fetcher.calls.get(), 2);
        // The package of all files is not a municipality.
        let frankfurt = "Kreisfreie Stadt Frankfurt am Main";
        fetcher.insert(
            &listing_url(Model::Dgm1.product(), frankfurt, 1),
            DGM1_FRANKFURT.as_bytes().to_vec(),
        );
        let files = downloadcenter_files(&fetcher, Model::Dgm1.product(), frankfurt).unwrap();
        assert_eq!(
            files,
            [(
                "Frankfurt_am_Main-DGM1".to_string(),
                "https://gds.hessen.de/downloadcenter/20261108/3D-Daten/Digitales%20Gel%C3%A4ndemodell%20%28DGM1%29/Kreisfreie%20Stadt%20Frankfurt%20am%20Main/Frankfurt_am_Main-DGM1.zip".to_string()
            )]
        );
        assert_eq!(file_url("//evil.example/x.zip"), None);
        assert_eq!(
            file_url("/a%20b/c d.zip").as_deref(),
            Some("https://gds.hessen.de/a%20b/c%20d.zip")
        );
    }

    #[test]
    fn lzw_predictor_float_tiff_decodes() {
        use tiff::encoder::colortype::Gray32;
        use tiff::encoder::{Compression, Predictor, TiffEncoder};
        use tiff::tags::Tag;
        // Like the 2026 HVBG build: Float32, LZW with horizontal differencing on the 32-bit
        // words, tiepoint and pixel scale but no GeoKeys, −9999 for NoData. The `tiff` encoder
        // refuses the predictor for floats, so the bit patterns are written as 32-bit integers
        // and the sample format is set to IEEE float afterwards.
        let (width, height) = (7, 3);
        let heights: Vec<f32> = (0..width * height)
            .map(|i| {
                if i == 20 {
                    -9999.0
                } else {
                    97.6 + i as f32 * 1.25
                }
            })
            .collect();
        let words: Vec<u32> = heights.iter().map(|h| h.to_bits()).collect();
        let mut tiff = std::io::Cursor::new(Vec::new());
        {
            let mut encoder = TiffEncoder::new(&mut tiff)
                .unwrap()
                .with_compression(Compression::Lzw)
                .with_predictor(Predictor::Horizontal);
            let mut image = encoder.new_image::<Gray32>(width, height).unwrap();
            image
                .encoder()
                .write_tag(Tag::ModelPixelScaleTag, &[1.0, 1.0, 0.0][..])
                .unwrap();
            image
                .encoder()
                .write_tag(
                    Tag::ModelTiepointTag,
                    &[0.0, 0.0, 0.0, 476_000.0, 5_553_000.0, 0.0][..],
                )
                .unwrap();
            image.write_data(&words).unwrap();
        }
        let mut bytes = tiff.into_inner();
        // The first IFD (little endian): its entries, 12 bytes each.
        let at = |b: &[u8], i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        let ifd = at(&bytes, 4) as usize;
        let count = usize::from(u16::from_le_bytes([bytes[ifd], bytes[ifd + 1]]));
        let mut tags = HashMap::new();
        for entry in (0..count).map(|i| ifd + 2 + 12 * i) {
            let tag = u16::from_le_bytes([bytes[entry], bytes[entry + 1]]);
            tags.insert(tag, entry);
        }
        let short = |b: &[u8], tag: u16| u16::from_le_bytes([b[tags[&tag] + 8], b[tags[&tag] + 9]]);
        assert_eq!(
            (short(&bytes, 259), short(&bytes, 317)),
            (5, 2),
            "LZW, predictor 2"
        );
        assert!(!tags.contains_key(&34735), "no GeoKeyDirectory");
        assert!(
            (at(&bytes, tags[&273] + 8) as usize) < ifd,
            "the IFD follows the strip"
        );
        let sample_format = tags[&339] + 8;
        bytes[sample_format..sample_format + 2].copy_from_slice(&3u16.to_le_bytes());
        let raster = crate::sources::float_tiff::decode_float_tiff(&bytes).unwrap();
        assert_eq!(
            raster.grid,
            crate::raster::GridSpec {
                west: 476_000.0,
                north: 5_553_000.0,
                pixel_size: 1.0,
                width: 7,
                height: 3
            }
        );
        assert_eq!(&raster.data[..20], &heights[..20]);
        assert!(raster.data[20].is_nan());
    }

    /// Frankfurt's DGM1 zip with `members` (name, tile, value), its listing and the tree.
    fn serve_frankfurt(fetcher: &MapFetcher, members: &[(&str, TileId, f32)]) -> String {
        fetcher.insert(&tree_url(Model::Dgm1), DGM1_TREE.as_bytes().to_vec());
        fetcher.insert(
            &listing_url(
                Model::Dgm1.product(),
                "Kreisfreie Stadt Frankfurt am Main",
                1,
            ),
            DGM1_FRANKFURT.as_bytes().to_vec(),
        );
        let url = file_url("/downloadcenter/20261108/3D-Daten/Digitales Geländemodell (DGM1)/Kreisfreie Stadt Frankfurt am Main/Frankfurt_am_Main-DGM1.zip").unwrap();
        let mut parts: Vec<(&str, Vec<u8>, bool)> = members
            .iter()
            .map(|(name, t, v)| (*name, tile_tiff(*t, *v, 0.0), true))
            .collect();
        parts.push((
            "DGM1_32_476_5552_1_he.tfw",
            b"1\n0\n0\n-1\n476000.5\n5552999.5\n".to_vec(),
            true,
        ));
        fetcher.insert(&url, archive(&parts));
        url
    }

    fn frankfurt_rows(tiles: &[(TileId, u32)]) -> Vec<u8> {
        let rows: Vec<(TileId, u32, &str, &str)> = tiles
            .iter()
            .map(|(t, serial)| {
                (
                    *t,
                    *serial,
                    "Frankfurt am Main",
                    "Kreisfreie Stadt Frankfurt am Main",
                )
            })
            .collect();
        metadata_xlsx(&rows)
    }

    /// Across the tile edge E 477 000 near the Lessing-Gymnasium; the centre lies in 476/5552.
    const SCHOOL: BoundingBox = BoundingBox {
        min_e: 476_800.0,
        min_n: 5_552_500.0,
        max_e: 477_100.0,
        max_n: 5_552_600.0,
    };

    #[test]
    fn hvbg_tiles_mosaic_into_the_area() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        let zip = serve_frankfurt(
            &fetcher,
            &[
                ("DGM1_32_476_5552_1_he.tif", tile(476, 5552), 110.0),
                ("DGM1_32_477_5552_1_he.tif", tile(477, 5552), 120.0),
            ],
        );
        fetcher.insert(
            METADATA_URL,
            frankfurt_rows(&[(tile(476, 5552), 44985), (tile(477, 5552), 45000)]),
        );
        let out = dir.path().join("dgm1.tif");
        let fetched = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(
            fetched,
            HeightModel {
                missing: Vec::new(),
                date: Some("2023-02-28".to_string()),
                unresolved: Vec::new(),
                unresolved_empty: Vec::new(),
                warnings: Vec::new(),
            }
        );
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (300, 100));
        assert_eq!(raster.get(10, 50), 110.0);
        assert_eq!(raster.get(290, 50), 120.0);
        // The seam at E 477 000: pixel 199 is the last of 476/5552, 200 the first of 477/5552.
        assert_eq!((raster.get(199, 50), raster.get(200, 50)), (110.0, 120.0));
        // Cached tiles whose CRC matches are not read again; the directory is.
        let zip_reads = || fetcher.log.borrow().iter().filter(|u| **u == zip).count();
        let before = zip_reads();
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(zip_reads() - before, 2);
        // Offline the cached list, directory and tiles are used.
        let offline = MapFetcher::default();
        let again = fetch_model(
            &offline,
            &cache,
            FetchMode::Offline,
            Model::Dgm1,
            &SCHOOL,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(again, fetched);
        assert_eq!(offline.calls.get(), 0);
        let err = fetch_model(
            &offline,
            &cache,
            FetchMode::Offline,
            Model::Dom1,
            &SCHOOL,
            out.parent().unwrap(),
            &out,
        )
        .unwrap_err();
        assert!(err.to_string().contains("online"), "{err}");
    }

    #[test]
    fn missing_tiles_inside_hesse_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_frankfurt(
            &fetcher,
            &[("DGM1_32_476_5552_1_he.tif", tile(476, 5552), 110.0)],
        );
        // 477/5552 is listed but not in the zip; 476/5553 is not listed at all.
        fetcher.insert(
            METADATA_URL,
            frankfurt_rows(&[(tile(476, 5552), 44985), (tile(477, 5552), 44985)]),
        );
        let bbox = BoundingBox {
            max_n: 5_553_100.0,
            ..SCHOOL
        };
        let fetched = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert_eq!(
            fetched.missing,
            [tile(476, 5553), tile(477, 5552), tile(477, 5553)]
        );
        // No tile at all: the zips or the list changed.
        let none = MapFetcher::default();
        serve_frankfurt(&none, &[]);
        none.insert(METADATA_URL, frankfurt_rows(&[(tile(476, 5552), 44985)]));
        let err = fetch_model(
            &none,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no tile"), "{err}");
    }

    const WI: (&str, &str) = ("Wiesbaden", "Landeshauptstadt Wiesbaden");
    const TS: (&str, &str) = ("Taunusstein", "Rheingau-Taunus");

    /// The DGM1 tree, the Wiesbaden and Rheingau-Taunus listings, Wiesbaden's zip (the older
    /// build, members in lower case, value 1) and Taunusstein's (value 2) with the given tiles,
    /// and the metadata list of `rows`; returns both zip URLs.
    fn serve_taunus(
        fetcher: &MapFetcher,
        wiesbaden_tiles: &[TileId],
        taunusstein_tiles: &[TileId],
        rows: &[(TileId, (&str, &str))],
    ) -> (String, String) {
        fetcher.insert(&tree_url(Model::Dgm1), DGM1_TREE.as_bytes().to_vec());
        let product = Model::Dgm1.product();
        fetcher.insert(
            &listing_url(product, "Kreisfreie Stadt Wiesbaden", 1),
            DGM1_WIESBADEN.as_bytes().to_vec(),
        );
        let taunusstein_uri = "/downloadcenter/20261108/3D-Daten/Digitales Geländemodell (DGM1)/Rheingau-Taunus-Kreis/Taunusstein_DGM1.zip";
        fetcher.insert(
            &listing_url(product, "Rheingau-Taunus-Kreis", 1),
            listing_answer(&[
                (
                    "Niedernhausen - DGM1",
                    "/downloadcenter/20261108/x/Niedernhausen - DGM1.zip",
                ),
                ("Taunusstein_DGM1", taunusstein_uri),
            ]),
        );
        let zip = |prefix: &str, tiles: &[TileId], value: f32| {
            let members: Vec<(String, Vec<u8>, bool)> = tiles
                .iter()
                .map(|t| {
                    (
                        format!("{prefix}_32_{}_{}_1_he.tif", t.east_km, t.north_km),
                        tile_tiff(*t, value, 0.0),
                        true,
                    )
                })
                .collect();
            let members: Vec<(&str, Vec<u8>, bool)> = members
                .iter()
                .map(|(n, b, d)| (n.as_str(), b.clone(), *d))
                .collect();
            archive(&members)
        };
        let wiesbaden = file_url("/downloadcenter/20261108/3D-Daten/Digitales Geländemodell (DGM1)/Kreisfreie Stadt Wiesbaden/Wiesbaden - DGM1.zip").unwrap();
        fetcher.insert(&wiesbaden, zip("dgm1", wiesbaden_tiles, 1.0));
        let taunusstein = file_url(taunusstein_uri).unwrap();
        fetcher.insert(&taunusstein, zip("DGM1", taunusstein_tiles, 2.0));
        let rows: Vec<(TileId, u32, &str, &str)> =
            rows.iter().map(|&(t, (m, d))| (t, 44629, m, d)).collect();
        fetcher.insert(METADATA_URL, metadata_xlsx(&rows));
        (wiesbaden, taunusstein)
    }

    #[test]
    fn tile_in_two_zips_is_read_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        let (wiesbaden, taunusstein) = serve_taunus(
            &fetcher,
            &[tile(445, 5555)],
            &[
                tile(445, 5554),
                tile(445, 5555),
                tile(446, 5554),
                tile(446, 5555),
            ],
            &[
                (tile(445, 5554), WI),
                (tile(445, 5554), TS),
                (tile(445, 5555), WI),
                (tile(445, 5555), TS),
                (tile(446, 5554), TS),
                (tile(446, 5555), TS),
            ],
        );
        let bbox = BoundingBox {
            min_e: 445_500.0,
            min_n: 5_554_500.0,
            max_e: 446_500.0,
            max_n: 5_555_500.0,
        };
        let out = dir.path().join("dgm1.tif");
        let fetched = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert!(fetched.missing.is_empty());
        let reads = |url: &str| fetcher.log.borrow().iter().filter(|u| *u == url).count();
        // Each directory once (two ranges); 445/5555 from Wiesbaden only, 445/5554 is not in
        // Wiesbaden's zip and comes from Taunusstein's.
        assert_eq!(reads(&wiesbaden), 2 + 2);
        assert_eq!(reads(&taunusstein), 2 + 3 * 2);
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!(raster.get(250, 250), 1.0, "445/5555 from Wiesbaden");
        assert_eq!(raster.get(250, 750), 2.0, "445/5554 from Taunusstein");
        assert_eq!(raster.get(750, 250), 2.0);
    }

    #[test]
    fn flight_date_from_get_feature_info() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!("{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetFeatureInfo", WMS.url),
            b"name = 'dop_476500_5552000'  land = 'he'  aufloesung = '20'  farbe = 'rgb'  \
              bezug = 'BDOM'  flugjahr = '2025'  bildflug = '19-08-2025'  aenderung = '12-10-2026'\n"
                .to_vec(),
        );
        assert_eq!(
            flight_date(&fetcher, &SCHOOL).as_deref(),
            Some("19.08.2025")
        );
        let url = fetcher.log.borrow()[0].clone();
        for part in [
            "LAYERS=he_dop_rgb_info",
            "QUERY_LAYERS=he_dop_rgb_info",
            "CRS=EPSG:25832",
            "WIDTH=100&HEIGHT=100&I=50&J=50",
            "INFO_FORMAT=text/plain",
        ] {
            assert!(url.contains(part), "{part} in {url}");
        }
        let centre_box = format!(
            "BBOX={},{},{},{}",
            (SCHOOL.min_e + SCHOOL.max_e) / 2.0 - 100.0,
            (SCHOOL.min_n + SCHOOL.max_n) / 2.0 - 100.0,
            (SCHOOL.min_e + SCHOOL.max_e) / 2.0 + 100.0,
            (SCHOOL.min_n + SCHOOL.max_n) / 2.0 + 100.0,
        );
        assert!(url.contains(&centre_box), "{centre_box} in {url}");

        let none = MapFetcher::default();
        none.insert_prefix(&WMS.url, b"Search returned no results.\n".to_vec());
        assert_eq!(flight_date(&none, &SCHOOL), None);
        let xml = MapFetcher::default();
        xml.insert_prefix(&WMS.url, b"<ServiceExceptionReport/>".to_vec());
        assert_eq!(flight_date(&xml, &SCHOOL), None);
        assert_eq!(flight_date(&MapFetcher::default(), &SCHOOL), None);
    }

    #[test]
    fn unreadable_zip_is_not_a_missing_tile() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        // 445/5554 is listed for both, but only Wiesbaden's zip (which fails) could hold it.
        let (wiesbaden, _) = serve_taunus(
            &fetcher,
            &[tile(445, 5554)],
            &[tile(446, 5554)],
            &[
                (tile(445, 5554), WI),
                (tile(445, 5554), TS),
                (tile(446, 5554), TS),
            ],
        );
        fetcher.failing.borrow_mut().push(wiesbaden);
        let bbox = BoundingBox {
            min_e: 445_500.0,
            min_n: 5_554_500.0,
            max_e: 446_500.0,
            max_n: 5_554_600.0,
        };
        let err = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("503"), "{err}");
    }

    #[test]
    fn failed_place_is_tried_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        let (wiesbaden, _) = serve_taunus(
            &fetcher,
            &[],
            &[tile(445, 5554)],
            &[
                (tile(445, 5554), WI),
                (tile(445, 5554), TS),
                (tile(445, 5555), WI),
                (tile(445, 5555), TS),
            ],
        );
        fetcher.failing.borrow_mut().push(wiesbaden.clone());
        let bbox = BoundingBox {
            min_e: 445_500.0,
            min_n: 5_554_500.0,
            max_e: 445_600.0,
            max_n: 5_555_500.0,
        };
        let err = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("503"), "{err}");
        let reads = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| **u == wiesbaden)
            .count();
        assert_eq!(
            reads,
            crate::fetch::ATTEMPTS as usize,
            "one directory read, retried"
        );
    }

    #[test]
    fn listing_without_total_is_followed_to_a_short_page() {
        let product = Model::Dgm1.product();
        let district = "Wetteraukreis";
        let names: Vec<String> = (0..22).map(|i| format!("Gemeinde {i} - DGM1")).collect();
        let files: Vec<(&str, String)> = names
            .iter()
            .map(|n| (n.as_str(), format!("/downloadcenter/20261108/w/{n}.zip")))
            .collect();
        let page = |range: std::ops::Range<usize>| {
            let entries: Vec<(&str, &str)> =
                files[range].iter().map(|(n, u)| (*n, u.as_str())).collect();
            listing_page(&entries, None)
        };
        let fetcher = MapFetcher::default();
        fetcher.insert(&listing_url(product, district, 1), page(0..20));
        fetcher.insert(&listing_url(product, district, 2), page(20..22));
        let found = downloadcenter_files(&fetcher, product, district).unwrap();
        assert_eq!(found.len(), 22);
        assert_eq!(fetcher.calls.get(), 2);
        // A short first page is the whole listing.
        let short = MapFetcher::default();
        short.insert(&listing_url(product, district, 1), page(0..3));
        assert_eq!(
            downloadcenter_files(&short, product, district)
                .unwrap()
                .len(),
            3
        );
        assert_eq!(short.calls.get(), 1);
        // A server that ignores the page answers page 1 again: an error, not 20 copies.
        let repeating = MapFetcher::default();
        repeating.insert(&listing_url(product, district, 1), page(0..20));
        repeating.insert(&listing_url(product, district, 2), page(0..20));
        let err = downloadcenter_files(&repeating, product, district).unwrap_err();
        assert!(err.to_string().contains("page 2"), "{err}");
    }

    #[test]
    fn damaged_member_names_the_tile() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        let zip = serve_frankfurt(&fetcher, &[]);
        fetcher.insert(
            &zip,
            archive(&[("DGM1_32_476_5552_1_he.tif", b"not a tiff".to_vec(), true)]),
        );
        fetcher.insert(METADATA_URL, frankfurt_rows(&[(tile(476, 5552), 44985)]));
        let err = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("32_476_5552"), "{err}");
    }

    #[test]
    fn unreadable_flight_date_only_empties_that_tiles_date() {
        let rows = |date: &str| {
            vec![
                strings(&["Kachelname", "Bildflugdatum", "Gemeinde", "Kreis"]),
                strings(&["4765552", date, "Frankfurt am Main", "Frankfurt"]),
                strings(&["4775552", "28.02.2023", "Frankfurt am Main", "Frankfurt"]),
            ]
        };
        let index = TileIndex::from_rows(&rows("28.02.2023")).unwrap();
        assert_eq!(
            index.get(tile(476, 5552)).unwrap().date.as_deref(),
            Some("2023-02-28")
        );
        assert!(index.undated.is_empty());
        for bad in ["unbekannt", "", "2026-03-01"] {
            let index = TileIndex::from_rows(&rows(bad)).unwrap();
            assert_eq!(index.get(tile(476, 5552)).unwrap().date, None);
            assert_eq!(
                index.get(tile(477, 5552)).unwrap().date.as_deref(),
                Some("2023-02-28")
            );
            assert_eq!(index.undated, ["4765552"]);
        }
        // No readable date at all points to a changed format: refused.
        let all_bad = vec![
            strings(&["Kachelname", "Bildflugdatum", "Gemeinde", "Kreis"]),
            strings(&["4765552", "", "Frankfurt am Main", "Frankfurt"]),
        ];
        let err = TileIndex::from_rows(&all_bad).unwrap_err();
        assert!(err.contains("Bildflugdatum"), "{err}");
    }

    #[test]
    fn undated_centre_tile_gives_no_record_date_and_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_frankfurt(
            &fetcher,
            &[
                ("DGM1_32_476_5552_1_he.tif", tile(476, 5552), 110.0),
                ("DGM1_32_477_5552_1_he.tif", tile(477, 5552), 120.0),
            ],
        );
        // Serial 0 is no date.
        fetcher.insert(
            METADATA_URL,
            frankfurt_rows(&[(tile(476, 5552), 0), (tile(477, 5552), 45000)]),
        );
        let fetched = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert_eq!(fetched.date, None);
        assert_eq!(fetched.warnings.len(), 1, "{:?}", fetched.warnings);
        assert!(fetched.warnings[0].contains("4765552"), "{fetched:?}");
    }

    #[test]
    fn unparsable_new_list_falls_back_to_the_kept_older_one() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_frankfurt(
            &fetcher,
            &[("DGM1_32_476_5552_1_he.tif", tile(476, 5552), 110.0)],
        );
        let metadata = cache.provider_dir("hvbg").join("metadata");
        std::fs::create_dir_all(&metadata).unwrap();
        let older = metadata.join("Metadaten_DGM-DOM_Juli-2026.xlsx");
        std::fs::write(&older, frankfurt_rows(&[(tile(476, 5552), 44985)])).unwrap();
        fetcher.insert(METADATA_URL, b"<html>maintenance</html>".to_vec());
        let out = dir.path().join("o.tif");
        let fetched = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(fetched.date.as_deref(), Some("2023-02-28"));
        assert_eq!(fetched.warnings.len(), 1, "{:?}", fetched.warnings);
        let warning = &fetched.warnings[0];
        assert!(
            warning.contains("Metadaten_DGM-DOM_August-2026.xlsx")
                && warning.contains("Metadaten_DGM-DOM_Juli-2026.xlsx"),
            "{warning}"
        );
        assert!(older.exists());
        assert!(!metadata.join("Metadaten_DGM-DOM_August-2026.xlsx").exists());
        // Without an older list the broken one is an error.
        std::fs::remove_file(&older).unwrap();
        let err = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            out.parent().unwrap(),
            &out,
        )
        .unwrap_err();
        assert!(err.to_string().contains("xlsx"), "{err}");
    }

    #[test]
    fn metadata_list_download_failure_uses_the_kept_list() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_frankfurt(
            &fetcher,
            &[("DGM1_32_476_5552_1_he.tif", tile(476, 5552), 110.0)],
        );
        let metadata = cache.provider_dir("hvbg").join("metadata");
        std::fs::create_dir_all(&metadata).unwrap();
        let older = metadata.join("Metadaten_DGM-DOM_Juli-2026.xlsx");
        std::fs::write(&older, frankfurt_rows(&[(tile(476, 5552), 44985)])).unwrap();
        // The new list is not served at all: its download fails.
        let out = dir.path().join("o.tif");
        let run = || {
            fetch_model(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                out.parent().unwrap(),
                &out,
            )
        };
        let fetched = run().unwrap();
        assert_eq!(fetched.date.as_deref(), Some("2023-02-28"));
        assert_eq!(fetched.warnings.len(), 1, "{:?}", fetched.warnings);
        let warning = &fetched.warnings[0];
        assert!(
            warning.contains("Metadaten_DGM-DOM_August-2026.xlsx")
                && warning.contains("Metadaten_DGM-DOM_Juli-2026.xlsx"),
            "{warning}"
        );
        assert!(older.exists());
        // Without a kept list the download error stays.
        std::fs::remove_file(&older).unwrap();
        assert!(run().is_err());
    }

    #[test]
    fn metadata_clean_up_only_removes_older_lists() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_frankfurt(
            &fetcher,
            &[("DGM1_32_476_5552_1_he.tif", tile(476, 5552), 110.0)],
        );
        fetcher.insert(METADATA_URL, frankfurt_rows(&[(tile(476, 5552), 44985)]));
        let metadata = cache.provider_dir("hvbg").join("metadata");
        std::fs::create_dir_all(metadata.join("subdir")).unwrap();
        std::fs::write(metadata.join("Metadaten_DGM-DOM_Juli-2026.xlsx"), b"old").unwrap();
        std::fs::write(metadata.join("other.part"), b"partial").unwrap();
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert!(!metadata.join("Metadaten_DGM-DOM_Juli-2026.xlsx").exists());
        assert!(metadata.join("Metadaten_DGM-DOM_August-2026.xlsx").exists());
        assert!(metadata.join("other.part").exists());
        assert!(metadata.join("subdir").is_dir());
    }

    #[test]
    fn shared_memo_reads_tree_and_list_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_frankfurt(
            &fetcher,
            &[("DGM1_32_476_5552_1_he.tif", tile(476, 5552), 110.0)],
        );
        fetcher.insert(METADATA_URL, frankfurt_rows(&[(tile(476, 5552), 44985)]));
        let mut memo = MetadataMemo::default();
        for _ in 0..2 {
            fetch_model_with(
                &fetcher,
                &cache,
                FetchMode::Online,
                Model::Dgm1,
                &SCHOOL,
                dir.path(),
                &dir.path().join("o.tif"),
                &mut memo,
            )
            .unwrap();
        }
        let reads = |url: &str| fetcher.log.borrow().iter().filter(|u| *u == url).count();
        assert_eq!(reads(&tree_url(Model::Dgm1)), 1);
        assert_eq!(reads(METADATA_URL), 1);
    }

    #[test]
    fn long_place_names_give_bounded_distinct_cache_keys() {
        let place = |name: &str| Place {
            municipality: name.to_string(),
            district: "Rheingau-Taunus".to_string(),
        };
        assert_eq!(place("Wiesbaden").cache_key(), "rheingau_taunus__wiesbaden");
        let (a, b) = (
            place(&"x".repeat(400)),
            place(&format!("{}y", "x".repeat(400))),
        );
        assert!(a.cache_key().len() <= 120, "{}", a.cache_key().len());
        assert_ne!(a.cache_key(), b.cache_key());
        assert_eq!(a.cache_key(), a.cache_key());
    }

    #[test]
    fn place_names_in_messages_are_cleaned_and_capped() {
        let long = format!("Evil\nName\u{7}{}", "z".repeat(300));
        let shown = place_label(&long, "Kreis\r");
        assert!(shown.chars().count() <= 100, "{shown}");
        assert!(!shown.chars().any(char::is_control), "{shown:?}");
        assert!(shown.starts_with("Evil Name"), "{shown}");
        assert_eq!(
            place_label("Grebenhain", "Vogelsberg"),
            "Grebenhain (Vogelsberg)"
        );
    }

    #[test]
    fn western_tiles_with_negative_zone_33_eastings_are_inside_hesse() {
        use crate::crs::{Crs, ProjectedPoint};
        // Lorch am Rhein: 417/5550 lies in Hesse at about −11 km in EPSG:25833, 416/5550 across
        // the Rhine in Rhineland-Palatinate.
        let centre = Crs::Utm33
            .reproject(
                Crs::Utm32,
                ProjectedPoint {
                    easting: 417_500.0,
                    northing: 5_550_500.0,
                },
            )
            .unwrap();
        assert!(centre.easting < 0.0, "{centre:?}");
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        fetcher.insert(&tree_url(Model::Dgm1), DGM1_TREE.as_bytes().to_vec());
        let uri = "/downloadcenter/20261108/3D-Daten/Digitales Geländemodell (DGM1)/Rheingau-Taunus-Kreis/Lorch - DGM1.zip";
        fetcher.insert(
            &listing_url(Model::Dgm1.product(), "Rheingau-Taunus-Kreis", 1),
            listing_answer(&[("Lorch - DGM1", uri)]),
        );
        fetcher.insert(
            &file_url(uri).unwrap(),
            archive(&[(
                "DGM1_32_418_5550_1_he.tif",
                tile_tiff(tile(418, 5550), 300.0, 0.0),
                true,
            )]),
        );
        fetcher.insert(
            METADATA_URL,
            metadata_xlsx(&[(tile(418, 5550), 44629, "Lorch", "Rheingau-Taunus")]),
        );
        let bbox = BoundingBox {
            min_e: 416_500.0,
            min_n: 5_550_400.0,
            max_e: 418_500.0,
            max_n: 5_550_500.0,
        };
        let fetched = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert_eq!(fetched.missing, [tile(417, 5550)]);
        // A tile the list names but no zip holds is missing even with its centre outside Hesse.
        let cache = Cache::new(dir.path().join("cache2"));
        fetcher.insert(
            METADATA_URL,
            metadata_xlsx(&[
                (tile(416, 5550), 44629, "Lorch", "Rheingau-Taunus"),
                (tile(418, 5550), 44629, "Lorch", "Rheingau-Taunus"),
            ]),
        );
        let fetched = fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            dir.path(),
            &dir.path().join("o2.tif"),
        )
        .unwrap();
        assert_eq!(fetched.missing, [tile(416, 5550), tile(417, 5550)]);
    }

    #[test]
    fn tree_without_product_id_is_refused() {
        let tree = serde_json::json!({
            "navigation": [{"name": "Hochtaunuskreis", "parentId": "DC0100057"}],
            "searchresult": {"product": {"name": "Digitales Geländemodell (DGM1)"}},
        });
        let err = parse_tree(tree.to_string().as_bytes()).unwrap_err();
        assert!(err.contains("product id"), "{err}");
    }

    /// A gemeindefrei area of Rheingau-Taunus with no file, and a place in a district the
    /// (trimmed) tree does not have.
    const NO_FILE: (&str, &str) = ("Gutsbezirk Reinhardswald", "Rheingau-Taunus");
    const NO_DISTRICT: (&str, &str) = ("Grebenhain", "Vogelsberg");

    fn fetch_taunus(
        fetcher: &MapFetcher,
        cache: &Cache,
        mode: FetchMode,
        out: &Path,
    ) -> Result<HeightModel, FetchError> {
        let bbox = BoundingBox {
            min_e: 445_500.0,
            min_n: 5_554_500.0,
            max_e: 446_500.0,
            max_n: 5_554_600.0,
        };
        fetch_model(
            fetcher,
            cache,
            mode,
            Model::Dgm1,
            &bbox,
            out.parent().unwrap(),
            out,
        )
    }

    #[test]
    fn gemeindefrei_place_is_skipped_not_fatal() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_taunus(
            &fetcher,
            &[],
            &[tile(445, 5554), tile(446, 5554)],
            &[
                (tile(445, 5554), NO_FILE),
                (tile(445, 5554), NO_DISTRICT),
                (tile(445, 5554), TS),
                (tile(446, 5554), NO_FILE),
                (tile(446, 5554), TS),
            ],
        );
        let out = dir.path().join("o.tif");
        let fetched = fetch_taunus(&fetcher, &cache, FetchMode::Online, &out).unwrap();
        assert_eq!(fetched.missing, []);
        assert_eq!(fetched.unresolved_empty, []);
        assert_eq!(
            fetched.unresolved,
            [
                "Gutsbezirk Reinhardswald (Rheingau-Taunus)",
                "Grebenhain (Vogelsberg)"
            ]
        );
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!(raster.get(250, 50), 2.0);
        // Offline the same: the unresolvable places are remembered, not "not cached".
        let offline =
            fetch_taunus(&MapFetcher::default(), &cache, FetchMode::Offline, &out).unwrap();
        assert_eq!(offline, fetched);
    }

    #[test]
    fn only_unresolvable_places_make_the_tile_missing() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_taunus(
            &fetcher,
            &[],
            &[tile(446, 5554)],
            &[
                (tile(445, 5554), NO_FILE),
                (tile(445, 5554), NO_DISTRICT),
                (tile(446, 5554), TS),
            ],
        );
        let fetched = fetch_taunus(
            &fetcher,
            &cache,
            FetchMode::Online,
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert_eq!(fetched.missing, [tile(445, 5554)]);
        assert_eq!(fetched.unresolved_empty, [tile(445, 5554)]);
        assert_eq!(
            fetched.unresolved,
            [
                "Gutsbezirk Reinhardswald (Rheingau-Taunus)",
                "Grebenhain (Vogelsberg)"
            ]
        );
    }

    #[test]
    fn replayed_errors_keep_their_url() {
        for e in [
            FetchError::Http {
                url: "https://gds.hessen.de/a.zip".to_string(),
                message: "503".to_string(),
            },
            FetchError::Io(
                std::path::PathBuf::from("/cache/hvbg/dgm1/zips/x.bin"),
                std::io::Error::other("disk full"),
            ),
            FetchError::CorruptArchive {
                url: "https://gds.hessen.de/a.zip".to_string(),
                message: "short".to_string(),
            },
            FetchError::RangeIgnored {
                url: "https://gds.hessen.de/a.zip".to_string(),
                range: "bytes=0-9".to_string(),
                status: 200,
            },
            FetchError::NotFound("https://gds.hessen.de/a.zip".to_string()),
            FetchError::Refused {
                url: "https://gds.hessen.de/a.zip".to_string(),
                status: 403,
                detail: Some("NoApplicableCode: forbidden".to_string()),
            },
        ] {
            assert_eq!(again(&e).to_string(), e.to_string());
        }
    }

    #[test]
    fn transient_no_match_keeps_the_cached_directory() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve_taunus(
            &fetcher,
            &[tile(446, 5554)],
            &[tile(445, 5554)],
            &[(tile(445, 5554), TS), (tile(446, 5554), WI)],
        );
        let out = dir.path().join("o.tif");
        let first = fetch_taunus(&fetcher, &cache, FetchMode::Online, &out).unwrap();
        assert_eq!((first.missing.len(), first.unresolved.len()), (0, 0));
        let taunusstein = Place {
            municipality: TS.0.to_string(),
            district: TS.1.to_string(),
        };
        let directory = cache
            .provider_dir("hvbg")
            .join("dgm1/zips")
            .join(format!("{}.bin", taunusstein.cache_key()));
        assert!(directory.exists());
        // The listing loses Taunusstein for a while: unresolved now, but the cached directory
        // stays for offline runs.
        fetcher.insert(
            &listing_url(Model::Dgm1.product(), "Rheingau-Taunus-Kreis", 1),
            listing_answer(&[(
                "Niedernhausen - DGM1",
                "/downloadcenter/20261108/x/Niedernhausen - DGM1.zip",
            )]),
        );
        let second = fetch_taunus(&fetcher, &cache, FetchMode::Online, &out).unwrap();
        assert_eq!(second.missing, [tile(445, 5554)]);
        assert_eq!(second.unresolved, ["Taunusstein (Rheingau-Taunus)"]);
        assert!(directory.exists());
        let offline =
            fetch_taunus(&MapFetcher::default(), &cache, FetchMode::Offline, &out).unwrap();
        assert_eq!(offline, first);
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!(raster.get(250, 50), 2.0, "445/5554 from the cache");
    }
}
