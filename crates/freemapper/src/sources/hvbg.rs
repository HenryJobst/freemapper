//! HVBG Hesse open data (dl-de/zero-2-0, "Hessische Verwaltung für Bodenmanagement und
//! Geoinformation"), all in EPSG:25832: the DGM1 and DOM1 1 km tiles read by byte ranges out of
//! the Downloadcenter's municipal zips, found through the DGM/DOM metadata list.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value;

use crate::crs::BoundingBox;
use crate::fetch::{Cache, FetchError, FetchMode, Fetcher, get_with_retries, write_atomically};
use crate::raster::{GridSpec, Raster};
use crate::sources::lvermgeo::{decode_float_tiff, write_float_raster};
use crate::sources::region::{Region, tile_centre_in};
use crate::sources::xlsx::{excel_date, xlsx_rows};
use crate::sources::zip_range::{
    Member, cached_member, find_member, parse_directory, read_directory_bytes, refused,
};
use crate::tiles::{TileId, tiles_covering};

/// The Downloadcenter's REST API (undocumented): the product tree and the files per district.
pub const REST_URL: &str =
    "https://gds.hessen.de/INTERSHOP/rest/WFS/HLBG-Geodaten-Site/-/downloadcenter";
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
    fn code(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Dom1 => "dom1",
        }
    }
}

/// What [`fetch_model`] found besides the raster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightModel {
    /// Tiles inside Hesse that no municipal zip holds.
    pub missing: Vec<TileId>,
    /// The flight date (`YYYY-MM-DD`) of the tile at the centre of the area.
    pub date: Option<String>,
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
        format!(
            "{}__{}",
            name_key(&self.district),
            name_key(&self.municipality)
        )
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
    }
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
            if record.date.is_none() {
                record.date = excel_date(cell(date_at));
            }
        }
        if tiles.is_empty() {
            return Err("the metadata list names no tile".to_string());
        }
        Ok(Self { tiles })
    }

    pub fn get(&self, tile: TileId) -> Option<&TileRecord> {
        self.tiles.get(&tile)
    }
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
/// "kreis".
pub(crate) fn name_key(name: &str) -> String {
    let words = words(name);
    let mut kept: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < words.len() {
        match words[i].as_str() {
            "kreisfreie" if words.get(i + 1).is_some_and(|w| w == "stadt") => i += 2,
            "landeshauptstadt" | "landkreis" | "kreis" => i += 1,
            word => {
                kept.push(word);
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
/// else the only one whose key contains `wanted`'s or is contained in it.
fn best_match(candidates: &[String], wanted: &str) -> Option<usize> {
    let compact = |name: &str| name_key(name).replace(' ', "");
    let target = compact(wanted);
    if target.is_empty() {
        return None;
    }
    let keys: Vec<String> = candidates.iter().map(|c| compact(c)).collect();
    let only = |found: Vec<usize>| (found.len() == 1).then(|| found[0]);
    let equal: Vec<usize> = (0..keys.len()).filter(|&i| keys[i] == target).collect();
    match equal.len() {
        0 => only(
            (0..keys.len())
                .filter(|&i| {
                    !keys[i].is_empty() && (keys[i].contains(&target) || target.contains(&keys[i]))
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
pub(crate) fn tree_url(model: Model) -> String {
    format!(
        "{REST_URL}?path={}&navigation=all",
        percent_encode(model.product(), false)
    )
}

/// Page `page` (from 1) of the file listing of `district` under `product`.
pub(crate) fn listing_url(product: &str, district: &str, page: u64) -> String {
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
pub(crate) fn file_url(path: &str) -> Option<String> {
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

/// The district nodes of a product tree answer and the metadata list linked in the product
/// description.
fn parse_tree(bytes: &[u8]) -> Result<(Vec<String>, Option<String>), String> {
    let answer = answer(bytes)?;
    let product = &answer["searchresult"]["product"];
    let nodes = answer["navigation"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    let below = |node: &&Value| product["id"].is_null() || node["parentId"] == product["id"];
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
/// description.
fn metadata_link(description: &str) -> Option<String> {
    description
        .split("href=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .find(|href| {
            let file = href.rsplit('/').next().unwrap_or_default().to_lowercase();
            file.starts_with("metadaten") && file.ends_with(".xlsx")
        })
        .and_then(file_url)
}

/// The files of a district listing: name and URL.
pub type Files = Vec<(String, String)>;

/// One listing page: its files (name, URL), the number of files in all and per page.
fn parse_listing(bytes: &[u8]) -> Result<(Files, u64, u64), String> {
    let answer = answer(bytes)?;
    let result = &answer["searchresult"];
    let files = result["downloads"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
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
        paging["total"].as_u64().unwrap_or(0),
        paging["pageSize"].as_u64().unwrap_or(20),
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
    let (mut page, mut pages) = (1, 1);
    while page <= pages {
        let url = listing_url(product, district, page);
        let (found, total, size) = get_with_retries(fetcher, &url, |b| {
            parse_listing(b).map_err(|m| refused(&url, m))
        })?;
        files.extend(found);
        if page == 1 {
            pages = total.div_ceil(size.max(1)).clamp(1, MAX_PAGES);
        }
        page += 1;
    }
    Ok(files)
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
    index: TileIndex,
    /// The district nodes of the product tree (online only).
    districts: Vec<String>,
    listings: HashMap<String, Rc<Files>>,
    zips: HashMap<String, Rc<Zip>>,
}

impl<'a> Session<'a> {
    /// Online the product tree and the metadata list it links (kept for offline runs); offline
    /// the kept list.
    fn open(
        fetcher: &'a dyn Fetcher,
        mode: FetchMode,
        model: Model,
        dir: PathBuf,
    ) -> Result<Self, FetchError> {
        let list_file = dir.join("metadata.xlsx");
        let read_index = |bytes: &[u8]| TileIndex::from_rows(&xlsx_rows(bytes)?);
        let (index, districts) = match mode {
            FetchMode::Online => {
                let url = tree_url(model);
                let (districts, link) = get_with_retries(fetcher, &url, |b| {
                    parse_tree(b).map_err(|m| refused(&url, m))
                })?;
                let list = link.ok_or_else(|| {
                    refused(
                        &url,
                        "the product description links no metadata list (Metadaten_….xlsx)",
                    )
                })?;
                let (index, bytes) = get_with_retries(fetcher, &list, |b| {
                    let index = read_index(b).map_err(|m| refused(&list, m))?;
                    Ok((index, b.to_vec()))
                })?;
                // Kept only once it parsed, so a broken answer never replaces a good copy.
                write_atomically(&list_file, &bytes)?;
                (index, districts)
            }
            FetchMode::Offline => {
                let bytes = std::fs::read(&list_file)
                    .map_err(|_| offline_hint(&format!("{} metadata list", model.label())))?;
                let index =
                    read_index(&bytes).map_err(|m| refused(&list_file.display().to_string(), m))?;
                (index, Vec::new())
            }
        };
        Ok(Self {
            fetcher,
            mode,
            model,
            dir,
            index,
            districts,
            listings: HashMap::new(),
            zips: HashMap::new(),
        })
    }

    /// The flight date of `tile`.
    fn date(&self, tile: TileId) -> Option<String> {
        self.index.get(tile).and_then(|r| r.date.clone())
    }

    /// The GeoTIFF of `tile` from the first zip of its municipalities that holds it (zips whose
    /// directory is already read first); `None` when none does. An error only when no zip of
    /// the tile could be read at all.
    fn tile(&mut self, tile: TileId) -> Result<Option<Vec<u8>>, FetchError> {
        let Some(record) = self.index.get(tile) else {
            return Ok(None);
        };
        let mut places = record.places.clone();
        places.sort_by_key(|p| !self.zips.contains_key(&p.cache_key()));
        let prefix = format!(
            "{}_32_{:03}_{:04}_",
            self.model.code(),
            tile.east_km,
            tile.north_km
        );
        let (mut failure, mut resolved) = (None, false);
        for place in &places {
            let zip = match self.zip(place) {
                Ok(zip) => zip,
                Err(e) => {
                    failure = Some(e);
                    continue;
                }
            };
            resolved = true;
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
        match failure {
            Some(e) if !resolved => Err(e),
            _ => Ok(None),
        }
    }

    /// The zip of `place`, its directory read once per run and kept for offline runs.
    fn zip(&mut self, place: &Place) -> Result<Rc<Zip>, FetchError> {
        let key = place.cache_key();
        if let Some(zip) = self.zips.get(&key) {
            return Ok(Rc::clone(zip));
        }
        // The member count (u16, little-endian), then the central directory.
        let file = self.dir.join("zips").join(format!("{key}.bin"));
        let zip = match self.mode {
            FetchMode::Online => {
                let url = self.zip_url(place)?;
                let (directory, count) = read_directory_bytes(self.fetcher, &url)?;
                let members = parse_directory(&directory, count).map_err(|m| refused(&url, m))?;
                // Kept only once it parsed, so a broken answer never replaces a good copy.
                write_atomically(&file, &[&count.to_le_bytes()[..], &directory].concat())?;
                Zip { url, members }
            }
            FetchMode::Offline => {
                let hint =
                    || offline_hint(&format!("{} of {}", self.model.label(), place.municipality));
                let cached = std::fs::read(&file).map_err(|_| hint())?;
                let (count, directory) = cached.split_at_checked(2).ok_or_else(hint)?;
                let url = file.display().to_string();
                let members = parse_directory(directory, u16::from_le_bytes([count[0], count[1]]))
                    .map_err(|m| refused(&url, m))?;
                Zip { url, members }
            }
        };
        let zip = Rc::new(zip);
        self.zips.insert(key, Rc::clone(&zip));
        Ok(zip)
    }

    /// The current URL of the zip of `place`: its district in the product tree, then its file
    /// in the district's listing.
    fn zip_url(&mut self, place: &Place) -> Result<String, FetchError> {
        let product = self.model.product();
        let district = best_match(&self.districts, &place.district)
            .map(|i| self.districts[i].clone())
            .ok_or_else(|| {
                refused(
                    &tree_url(self.model),
                    format!(
                        "no unique district of the Downloadcenter matches \"{}\" of the metadata list (districts: {})",
                        place.district,
                        self.districts.join(", ")
                    ),
                )
            })?;
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
        let at = best_match(&keys, &place.municipality).ok_or_else(|| {
            let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
            refused(
                &listing_url(product, &district, 1),
                format!(
                    "no unique file of {district} matches \"{}\" of the metadata list (files: {})",
                    place.municipality,
                    names.join(", ")
                ),
            )
        })?;
        Ok(files[at].1.clone())
    }
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
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<HeightModel, FetchError> {
    let mut session = Session::open(fetcher, mode, model, cache.hvbg_dir().join(model.code()))?;
    let grid = GridSpec::covering(bbox, 1.0);
    let mut raster = Raster::filled(grid, f32::NAN);
    let (mut found, mut missing) = (0, Vec::new());
    for tile in tiles_covering(bbox) {
        let Some(bytes) = session.tile(tile)? else {
            if tile_centre_in(tile, Region::Hessen) {
                missing.push(tile);
            }
            continue;
        };
        found += 1;
        let part = decode_float_tiff(&bytes).map_err(|m| refused(REST_URL, m))?;
        raster
            .paste_with(&part.grid, |x, y| part.get(x, y))
            .map_err(|e| refused(REST_URL, e.to_string()))?;
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
    write_float_raster(&raster, out, REST_URL)?;
    let centre = TileId {
        east_km: ((bbox.min_e + bbox.max_e) / 2000.0).floor() as u32,
        north_km: ((bbox.min_n + bbox.max_n) / 2000.0).floor() as u32,
    };
    Ok(HeightModel {
        missing,
        date: session.date(centre),
    })
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::sources::xlsx::test_support::{escape, workbook};
    use crate::tiles::TileId;

    /// The metadata list the product descriptions of the fixtures link.
    pub(crate) const METADATA_URL: &str = "https://gds.hessen.de/INTERSHOP/static/WFS/HLBG-Geodaten-Site/-/HLBG-Geodaten/de_DE/Downloadcenter/Daten/3D-Daten/Metadaten_DGM-DOM_August-2026.xlsx";
    pub(crate) const DGM1_TREE: &str =
        include_str!("../../../../tests/fixtures/hessen-dc-dgm1-tree.json");
    pub(crate) const DGM1_FRANKFURT: &str =
        include_str!("../../../../tests/fixtures/hessen-dc-dgm1-frankfurt.json");
    pub(crate) const DGM1_WIESBADEN: &str =
        include_str!("../../../../tests/fixtures/hessen-dc-dgm1-wiesbaden.json");
    pub(crate) const DOP_MARBURG: [&str; 2] = [
        include_str!("../../../../tests/fixtures/hessen-dc-dop-marburg-p1.json"),
        include_str!("../../../../tests/fixtures/hessen-dc-dop-marburg-p2.json"),
    ];

    /// The DGM/DOM metadata list with one row per (tile, flight date as Excel serial,
    /// municipality, district), laid out like the August 2026 release.
    pub(crate) fn metadata_xlsx(rows: &[(TileId, u32, &str, &str)]) -> Vec<u8> {
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

    /// A product tree answer listing `districts`, whose description links [`METADATA_URL`].
    pub(crate) fn tree_answer(districts: &[&str]) -> Vec<u8> {
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
                    "description": format!(r#"<ul><li><a href="{path}" target="_blank">metadatenliste</a></li></ul>"#),
                },
                "downloads": [],
                "paging": {"total": 0, "pageSize": 20, "page": 1},
            }
        })
        .to_string()
        .into_bytes()
    }

    /// A one-page district listing of `files` (name, `downloadLink.uri`).
    pub(crate) fn listing_answer(files: &[(&str, &str)]) -> Vec<u8> {
        let downloads: Vec<serde_json::Value> = files
            .iter()
            .map(|(name, uri)| serde_json::json!({"name": name, "downloadType": "File", "downloadLink": {"uri": uri}}))
            .collect();
        serde_json::json!({
            "searchresult": {
                "packages": [],
                "product": {"name": "district"},
                "downloads": downloads,
                "paging": {"total": files.len(), "pageSize": 20, "page": 1},
            }
        })
        .to_string()
        .into_bytes()
    }

    /// A 1 km tile of 1 m pixels at `tile`, every value `value` plus `slope` per metre east.
    pub(crate) fn tile_tiff(tile: TileId, value: f32, slope: f32) -> Vec<u8> {
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
        assert_eq!(name_key("Odenwaldkreis"), "odenwaldkreis");
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
        let raster = crate::sources::lvermgeo::decode_float_tiff(&bytes).unwrap();
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
            &out,
        )
        .unwrap();
        assert_eq!(
            fetched,
            HeightModel {
                missing: Vec::new(),
                date: Some("2023-02-28".to_string())
            }
        );
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (300, 100));
        assert_eq!(raster.get(10, 50), 110.0);
        assert_eq!(raster.get(290, 50), 120.0);
        // Cached tiles whose CRC matches are not read again; the directory is.
        let zip_reads = || fetcher.log.borrow().iter().filter(|u| **u == zip).count();
        let before = zip_reads();
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &SCHOOL,
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
            &dir.path().join("o.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no tile"), "{err}");
    }

    #[test]
    fn tile_in_two_zips_is_read_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
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
        // The older Wiesbaden build spells its members in lower case.
        let wiesbaden = file_url("/downloadcenter/20261108/3D-Daten/Digitales Geländemodell (DGM1)/Kreisfreie Stadt Wiesbaden/Wiesbaden - DGM1.zip").unwrap();
        fetcher.insert(
            &wiesbaden,
            archive(&[(
                "dgm1_32_445_5555_1_he.tif",
                tile_tiff(tile(445, 5555), 1.0, 0.0),
                true,
            )]),
        );
        let taunusstein = file_url(taunusstein_uri).unwrap();
        let members: Vec<(String, Vec<u8>, bool)> =
            [(445, 5554), (445, 5555), (446, 5554), (446, 5555)]
                .iter()
                .map(|&(e, n)| {
                    (
                        format!("DGM1_32_{e}_{n}_1_he.tif"),
                        tile_tiff(tile(e, n), 2.0, 0.0),
                        true,
                    )
                })
                .collect();
        let members: Vec<(&str, Vec<u8>, bool)> = members
            .iter()
            .map(|(n, b, d)| (n.as_str(), b.clone(), *d))
            .collect();
        fetcher.insert(&taunusstein, archive(&members));
        let wi = ("Wiesbaden", "Landeshauptstadt Wiesbaden");
        let ts = ("Taunusstein", "Rheingau-Taunus");
        let rows: Vec<(TileId, u32, &str, &str)> = [
            (tile(445, 5554), wi),
            (tile(445, 5554), ts),
            (tile(445, 5555), wi),
            (tile(445, 5555), ts),
            (tile(446, 5554), ts),
            (tile(446, 5555), ts),
        ]
        .iter()
        .map(|&(t, (m, d))| (t, 44629, m, d))
        .collect();
        fetcher.insert(METADATA_URL, metadata_xlsx(&rows));
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
    fn synthetic_hessen_serves_both_models() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        crate::fetch::test_support::serve_synthetic_hessen(&fetcher, &SCHOOL);
        let mut rasters = Vec::new();
        for model in [Model::Dgm1, Model::Dom1] {
            let out = dir.path().join(format!("{}.tif", model.code()));
            let fetched =
                fetch_model(&fetcher, &cache, FetchMode::Online, model, &SCHOOL, &out).unwrap();
            assert_eq!(fetched.missing, []);
            assert_eq!(fetched.date.as_deref(), Some("2023-02-28"));
            rasters.push(crate::derive::terrain::read_float_raster(&out).unwrap());
        }
        // E 476 810: 810 m into the tile.
        assert!((rasters[0].get(10, 50) - 108.1).abs() < 1e-3);
        assert!((rasters[1].get(10, 50) - rasters[0].get(10, 50) - 12.0).abs() < 1e-3);
    }
}
