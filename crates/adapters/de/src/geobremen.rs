//! Landesamt GeoInformation Bremen ("GeoBremen") open data (CC BY 4.0), all in EPSG:25832: the
//! orthophoto from the WMS, whose layers are named by flight year and city; the terrain model
//! (DGM1) and the surface model (DOM1) of the laser scans, as XYZ text out of one archive per
//! city, read tile by tile with range requests; buildings and land use from the WFS of the
//! full ALKIS, which the LGLN of Lower Saxony runs for the state
//! (`docs/regions/bremen-reference.md`).

use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::extras::NasService;
use freemapper_adapter_api::project::RasterProject;
use freemapper_adapter_api::rasters::{ProviderInfo, RasterBackend, RasterKind};
use freemapper_adapter_api::records::AdapterId;
use freemapper_sources::aaa::AaaAlkis;
use freemapper_sources::raster_fetch::{
    fetch_area_heights, fetch_wms_orthophoto, warn_beyond_state,
};
use freemapper_sources::wms::{DateLabel, OrthophotoSource};

use std::borrow::Cow;
use std::path::PathBuf;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{CacheDir, FetchError, FetchMode, Fetcher};
use crate::note::{Arg, note};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::write_float_raster;
use crate::sources::height_model::{HeightModel, centre_tile, names};
use crate::sources::region::{BREMEN, tile_centre_in};
use crate::sources::wms::WmsService;
use crate::sources::zip_range::{Member, find_member, refused};
use crate::tiles::{TileId, tiles_covering};
use crate::xyz_archive::{kept_directory, xyz_tile};

/// The WMS with the orthophotos of every flight since 1951, one layer per flight and city
/// (`dop10_2025_HB`, `dop10_2025_BHV`), cut at the city borders.
pub const ORTHOPHOTOS: &str = "https://geodienste.bremen.de/wms_dop_lb";

/// The layers of the newest flights when the adapter was written, and their year: asked for
/// when the capabilities of the service cannot be read.
pub const KNOWN_LAYERS: &str = "dop10_2025_HB,dop10_2025_BHV";
const KNOWN_YEAR: u16 = 2025;

/// The WFS of the full ALKIS of the state in its simple feature form, on the server of the
/// LGLN.
pub const ALKIS_URL: &str = "https://opendata.lgln.niedersachsen.de/doorman/noauth/alkishb_wfs_sf";

/// The full ALKIS of the state: [`AaaAlkis`] turns its objects into the simplified schema.
pub const ALKIS: NasService = NasService {
    name: Cow::Borrowed("GeoBremen ALKIS"),
    url: Cow::Borrowed(ALKIS_URL),
    region: AdapterId::built_in(BREMEN),
    crs: Crs::Utm32,
    // The service names no limit of its own; a larger answer is asked again in quadrants.
    max_count: 100_000,
    max_bytes: 256 << 20,
    layers: None,
    namespaces: None,
};

/// `bez` of a square without a function: a third of the squares have none, and what they are
/// used for is open, so they stay candidates.
pub const SQUARE_BEZ: &str = "ohne Unterart (Bremen)";

/// The ALKIS of Bremen: the full ALKIS as the simplified schema.
pub fn alkis() -> AaaAlkis {
    AaaAlkis {
        service: ALKIS,
        square_bez: Some(Cow::Borrowed(SQUARE_BEZ)),
        cache_dir: Cow::Borrowed(PROVIDER),
    }
}

/// The file server of the open data.
pub const FILES: &str = "https://gdi2.geo.bremen.de/inspire/download";

/// The provider id of the adapter, and the name of its folder in the cache.
const PROVIDER: &str = "geobremen";

/// The state as the messages name it.
const STATE: &str = "Bremen";

/// The agency as the messages name it.
const AGENCY: &str = "GeoBremen";

/// freemapper's name for the date of the orthophoto: the service names the year of a flight,
/// not its day.
pub const FLIGHT_LABEL: DateLabel = DateLabel {
    record: "Bildflugjahr",
    shown: "year of the flight",
};

/// The WMS asked for `layers`.
pub fn wms(layers: &str) -> WmsService {
    WmsService {
        url: Cow::Borrowed(ORTHOPHOTOS),
        layer: Cow::Owned(layers.to_string()),
        crs: Crs::Utm32,
        extra: Cow::Borrowed(""),
        max_px: 4000,
    }
}

/// The two cities of the state. Each has its own flights and its own archives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum City {
    Bremen,
    Bremerhaven,
}

impl City {
    /// The city a 1 km tile (EPSG:25832) belongs to: Bremerhaven begins 26 km north of the
    /// northernmost tile of Bremen.
    pub fn of(tile: TileId) -> City {
        if tile.north_km >= 5910 {
            City::Bremerhaven
        } else {
            City::Bremen
        }
    }

    /// The city as the layers and the archives name it.
    fn code(self) -> &'static str {
        match self {
            City::Bremen => "HB",
            City::Bremerhaven => "BHV",
        }
    }

    /// The year of the laser scan the archives of the city hold.
    pub fn scan_year(self) -> u16 {
        match self {
            City::Bremen => 2017,
            City::Bremerhaven => 2015,
        }
    }
}

/// The city (0 Bremen, 1 Bremerhaven) and the flight year of a colour orthophoto layer. A
/// layer is named `dop<resolution>_<year>_<city>`, the resolution in centimetres (`10`, or
/// `12_5` for 12.5 cm); the infrared images (`dop10_cir_2025_HB`) and the scanned aerial photos
/// (`luftbild_1951_HB`) are no such layer.
fn flight_of(layer: &str) -> Option<(usize, u16)> {
    let mut parts = layer.strip_prefix("dop")?.rsplit('_');
    let city = match parts.next()? {
        "HB" => 0,
        "BHV" => 1,
        _ => return None,
    };
    let year = parts.next().filter(|year| year.len() == 4)?.parse().ok()?;
    let resolution: Vec<&str> = parts.collect();
    let digits = |part: &&str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    (!resolution.is_empty() && resolution.iter().all(digits)).then_some((city, year))
}

/// The newest colour orthophoto of each city among the layers of the capabilities `text`: the
/// layers to ask for, and the flight years of Bremen and Bremerhaven. `None` when a city has
/// no such layer.
pub fn newest_layers(text: &str) -> Option<(String, [u16; 2])> {
    let mut newest: [Option<(u16, &str)>; 2] = [None, None];
    for name in text
        .split("<Name>")
        .skip(1)
        .filter_map(|rest| rest.split('<').next())
    {
        if let Some((city, year)) = flight_of(name)
            && newest[city].is_none_or(|(known, _)| year > known)
        {
            newest[city] = Some((year, name));
        }
    }
    let (bremen, bremerhaven) = (newest[0]?, newest[1]?);
    Some((
        format!("{},{}", bremen.1, bremerhaven.1),
        [bremen.0, bremerhaven.0],
    ))
}

/// The layers of the newest flights and their years, from the capabilities of the WMS; the
/// known ones when the service does not answer or names none.
fn flights(fetcher: &dyn Fetcher) -> (String, [u16; 2]) {
    let url = format!("{ORTHOPHOTOS}?SERVICE=WMS&REQUEST=GetCapabilities");
    crate::fetch::get_with_retries(fetcher, &url, |bytes| Ok(bytes.to_vec()))
        .ok()
        .and_then(|bytes| newest_layers(&String::from_utf8_lossy(&bytes)))
        .unwrap_or_else(|| (KNOWN_LAYERS.to_string(), [KNOWN_YEAR; 2]))
}

/// A height model of the file server: 1 km tiles of XYZ text, all tiles of a city in one
/// archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1, the terrain from the laser scan.
    Dgm1,
    /// DOM1, the surface from the same laser scan (leaf-off, 2017 in Bremen and 2015 in
    /// Bremerhaven).
    Dom1,
}

impl Model {
    /// The folder of the file server that holds the archives of the model.
    fn folder(self) -> &'static str {
        match self {
            Self::Dgm1 => "DGM",
            Self::Dom1 => "DOM",
        }
    }

    /// Start of the tile names, e.g. `dgm1_32487_5881_1_hb.xyz`.
    fn prefix(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Dom1 => "dom1",
        }
    }

    /// The archive with the tiles of `city`. Its name carries the year of the scan: a new scan
    /// gets a new address, and this one then answers "not found".
    pub fn archive_url(self, city: City) -> String {
        format!(
            "{FILES}/{}/data/Gitternetz_{}_{}_{}_ASCII_XYZ.zip",
            self.folder(),
            self.prefix().to_uppercase(),
            city.scan_year(),
            city.code()
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (GeoBremen DGM1)",
            Self::Dom1 => "surface model (GeoBremen DOM1)",
        }
    }

    /// The folder of the cache that holds the tiles and the directories of the model.
    fn cache_folder(self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(PROVIDER).join(self.prefix())
    }

    /// The start of the name of every cache file of `tile`; the checksum of the member the
    /// tile was read from follows.
    fn cache_name(self, tile: TileId) -> String {
        format!(
            "{}_32_{:03}_{:04}_",
            self.prefix(),
            tile.east_km,
            tile.north_km
        )
    }

    /// Whether the cache holds `tile` of this model, read from whichever member.
    pub fn is_cached(self, cache: &CacheDir, tile: TileId) -> bool {
        let start = self.cache_name(tile);
        std::fs::read_dir(self.cache_folder(cache)).is_ok_and(|files| {
            files.flatten().any(|file| {
                let name = file.file_name();
                let name = name.to_string_lossy();
                name.starts_with(&start) && name.ends_with(".tif")
            })
        })
    }

    /// The member with the heights of `tile` among `members`. The archives name a tile
    /// `dgm1_32487_5881_1_hb.xyz`, the terrain of Bremerhaven with two underscores after the
    /// easting, so the name is compared part by part.
    fn member_of(self, members: &[Member], tile: TileId) -> Option<&Member> {
        let east = format!("32{:03}", tile.east_km);
        let north = format!("{:04}", tile.north_km);
        find_member(members, |name| {
            let mut parts = name.split('_').filter(|part| !part.is_empty());
            name.ends_with(".xyz")
                && parts.next() == Some(self.prefix())
                && parts.next() == Some(east.as_str())
                && parts.next() == Some(north.as_str())
        })
    }
}

/// The heights of a tile from its XYZ member: a header line, then one grid point per line,
/// `E N height`. The four archives write the points in three ways: at whole metres or at the
/// centres of the cells, the easting with or without the zone number in front, from the north
/// or from the south. A point stands for the cell it lies in, so a point at whole metres for
/// the cell to its north-east: such a model lies half a metre off. The surface model of
/// Bremerhaven repeats the first column of the next tile; points outside the tile are passed
/// over. A file without a point of the tile is an error.
fn parse_xyz(bytes: &[u8], tile: TileId) -> Result<Raster<f32>, String> {
    /// The zone number in front of an easting: `32487000` is 487 km in zone 32.
    const ZONE: f64 = 32_000_000.0;
    let grid = GridSpec {
        west: f64::from(tile.east_km) * 1000.0,
        north: f64::from(tile.north_km + 1) * 1000.0,
        pixel_size: 1.0,
        width: 1000,
        height: 1000,
    };
    let mut raster = Raster::filled(grid, f32::NAN);
    let mut points = 0_u32;
    for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        let text = String::from_utf8_lossy(line);
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let mut fields = text.split_ascii_whitespace().map(str::parse::<f64>);
        let (Some(Ok(e)), Some(Ok(n)), Some(Ok(z)), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            // The header `x y z`.
            if index == 0 {
                continue;
            }
            let excerpt: String = text.chars().take(60).collect();
            return Err(format!("line {} is not `E N height`: {excerpt}", index + 1));
        };
        let e = if e >= ZONE { e - ZONE } else { e };
        let (col, row) = ((e - grid.west).floor(), (grid.north - n).ceil() - 1.0);
        if !(0.0..1000.0).contains(&col) || !(0.0..1000.0).contains(&row) {
            continue;
        }
        let height = if z < -9998.0 { f32::NAN } else { z as f32 };
        raster.set(col as u32, row as u32, height);
        points += 1;
    }
    if points == 0 {
        return Err(format!(
            "no point of the tile 32_{}_{} in the file",
            tile.east_km, tile.north_km
        ));
    }
    Ok(raster)
}

/// The members of the archive of `model` for `city`, with the directory kept in the cache.
fn directory(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    city: City,
) -> Result<Vec<Member>, FetchError> {
    let file = model
        .cache_folder(cache)
        .join(format!("directory-{}.bin", city.code().to_lowercase()));
    kept_directory(fetcher, mode, &model.archive_url(city), &file)
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832) on its 1 m grid, read out of the
/// archives of their cities and cached, and what the fetch found besides: the tiles inside the
/// state that the archive of their city lacks, and the year of the scan at the centre of the
/// area. A tile outside the state is in no archive and stays empty. No tile at all is an
/// error, and so is an archive that cannot be read.
fn mosaic(
    fetcher: &dyn Fetcher,
    cache: &CacheDir,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
) -> Result<(Raster<f32>, HeightModel), FetchError> {
    let folder = format!("{FILES}/{}/data", model.folder());
    let mut raster = Raster::filled(GridSpec::covering(bbox, 1.0), f32::NAN);
    let mut found = HeightModel::default();
    let mut archives: [Option<Vec<Member>>; 2] = [None, None];
    let mut read = 0;
    for tile in tiles_covering(bbox) {
        let city = City::of(tile);
        let members = match &mut archives[city as usize] {
            Some(members) => members,
            unread => unread.insert(directory(fetcher, cache, mode, model, city)?),
        };
        let Some(member) = model.member_of(members, tile) else {
            if tile_centre_in(tile, BREMEN) {
                found.missing.push(tile);
            }
            continue;
        };
        let file = model.cache_folder(cache).join(format!(
            "{}{:08x}.tif",
            model.cache_name(tile),
            member.crc
        ));
        let url = model.archive_url(city);
        let part = xyz_tile(fetcher, mode, (&url, member), &file, tile, parse_xyz)?;
        raster
            .paste_with(&part.grid, |x, y| part.get(x, y))
            .map_err(|e| refused(&folder, e.to_string()))?;
        read += 1;
    }
    if read == 0 {
        return Err(refused(
            &folder,
            match mode {
                FetchMode::Offline => format!(
                    "offline and not cached: {}; run `freemapper fetch` online",
                    model.label()
                ),
                FetchMode::Online => {
                    format!("no tile of the {} covers the map area", model.label())
                }
            },
        ));
    }
    found.missing.sort();
    found.date = Some(City::of(centre_tile(bbox)).scan_year().to_string());
    Ok((raster, found))
}

/// The rasters of GeoBremen: requested for the area and stored in the project folder.
pub struct GeoBremenBackend {
    info: ProviderInfo,
}

impl GeoBremenBackend {
    pub fn new() -> Self {
        Self {
            info: ProviderInfo {
                id: AdapterId::built_in(PROVIDER),
                cache_dir: Some(PROVIDER.to_string()),
                terrain_source: "geobremen-dgm1".to_string(),
                kind: RasterKind::PerArea,
            },
        }
    }
}

impl Default for GeoBremenBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterBackend for GeoBremenBackend {
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

/// A year as the messages name a date: the German texts say "vom" before it.
fn year_words(year: &str) -> Arg {
    Arg::Words {
        en: year.to_string(),
        de: format!("Jahr {year}"),
    }
}

/// The rasters of `bbox` (EPSG:25832): the orthophoto of the newest flight from the WMS, DGM1
/// and DOM1 read out of the archives of the file server. The surface model is only a
/// template: an unavailable one costs a warning, and a stored one stays while it covers the
/// area. It is as old as the laser scan, so the fetch says when the orthophoto is newer.
fn fetch_rasters(
    env: &mut AdapterEnv<'_>,
    project: &mut dyn RasterProject,
    bbox: &BoundingBox,
) -> Result<(), AdapterError> {
    let (fetcher, mode, cache) = (env.fetcher, env.mode, env.cache);
    let root = project.dir().to_path_buf();
    project.records().tiles.clear();
    project.records().ndsm_tiles.clear();
    warn_beyond_state(env, project, PROVIDER, AGENCY);
    let city = City::of(centre_tile(bbox));
    // Offline the stored image is used, and no layer is asked for.
    let (layers, years) = match mode {
        FetchMode::Online => flights(fetcher),
        FetchMode::Offline => (KNOWN_LAYERS.to_string(), [KNOWN_YEAR; 2]),
    };
    let flight = || Some(years[city as usize].to_string());
    fetch_wms_orthophoto(
        env,
        project,
        bbox,
        (
            &OrthophotoSource::Wms(wms(&layers)),
            AGENCY,
            &flight,
            Some(&FLIGHT_LABEL),
        ),
    )?;
    fetch_area_heights(
        env,
        project,
        bbox,
        STATE,
        (Model::Dgm1.label(), Model::Dom1.label()),
        &mut |surface, out, progress| {
            let model = if surface { Model::Dom1 } else { Model::Dgm1 };
            let (raster, mut found) = mosaic(fetcher, cache, mode, model, bbox)?;
            write_float_raster(
                &raster,
                &root,
                out,
                &format!("{FILES}/{}/data", model.folder()),
            )?;
            // Told here, with the state as it is written: the shared warning makes a key of
            // the name, which the catalogs do not have.
            let missing = std::mem::take(&mut found.missing);
            if !missing.is_empty() {
                progress(note!(
                    "warning-model-tiles-missing",
                    model = crate::note::model(model.label()),
                    count = missing.len(),
                    names = names(&missing),
                    state = STATE
                ));
            }
            Ok(found)
        },
        &mut |surface, found, records, progress| {
            let photo = records
                .orthophoto
                .as_ref()
                .and_then(|o| o.flight.as_deref());
            if let (true, Some(scan), Some(photo)) = (surface, found.date.as_deref(), photo)
                && scan < photo
            {
                progress(note!(
                    "warning-surface-much-older",
                    model = crate::note::model(Model::Dom1.label()),
                    surface = year_words(scan),
                    flight = year_words(photo)
                ));
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::Cache;
    use crate::fetch::test_support::{MapFetcher, solid_jpeg};
    use crate::sources::zip_range::test_support::archive;
    use freemapper_adapter_api::records::{WcsRecord, stored_bbox};
    use freemapper_sources::raster_fetch::{SURFACE_RELPATH, TERRAIN_RELPATH};
    use std::path::Path;

    fn tile(east_km: u32, north_km: u32) -> TileId {
        TileId { east_km, north_km }
    }

    /// Bremen: a strip across the edge between the tiles 487 and 488, in their ten northern
    /// rows.
    fn bremen() -> BoundingBox {
        BoundingBox {
            min_e: 487_900.0,
            min_n: 5_881_990.0,
            max_e: 488_100.0,
            max_n: 5_882_000.0,
        }
    }

    /// Bremerhaven: the same strip between the tiles 471 and 472.
    fn bremerhaven() -> BoundingBox {
        BoundingBox {
            min_e: 471_900.0,
            min_n: 5_933_990.0,
            max_e: 472_100.0,
            max_n: 5_934_000.0,
        }
    }

    /// How an archive writes the points of a tile.
    #[derive(Clone, Copy)]
    enum Spelling {
        /// Bremen: whole metres, no zone number, from the north.
        WholeMetres,
        /// The terrain of Bremerhaven: cell centres, the zone number in front, from the north.
        ZoneCentres,
        /// The surface of Bremerhaven: whole metres, the zone number in front, from the south,
        /// with the first column of the next tile at the end of every row.
        ZoneWholeMetres,
    }

    /// The northern `rows` rows of a tile in `spelling`, CRLF, after the header line. The
    /// height is `base` plus the column in centimetres.
    fn xyz(spelling: Spelling, east_km: u32, north_km: u32, rows: u32, base: f32) -> Vec<u8> {
        let mut text = String::from("x y z\r\n");
        let (west, top) = (east_km * 1000, (north_km + 1) * 1000 - 1);
        let height = |col: u32| base + col as f32 / 100.0;
        match spelling {
            Spelling::WholeMetres => {
                for row in 0..rows {
                    for col in 0..1000 {
                        text.push_str(&format!(
                            "{} {} {:.2}\r\n",
                            west + col,
                            top - row,
                            height(col)
                        ));
                    }
                }
            }
            Spelling::ZoneCentres => {
                for row in 0..rows {
                    for col in 0..1000 {
                        text.push_str(&format!(
                            "32{}.5 {}.5 {:.2}\r\n",
                            west + col,
                            top - row,
                            height(col)
                        ));
                    }
                }
            }
            Spelling::ZoneWholeMetres => {
                for row in (0..rows).rev() {
                    for col in 0..=1000 {
                        text.push_str(&format!(
                            "32{} {} {:.2}\r\n",
                            west + col,
                            top - row,
                            height(col)
                        ));
                    }
                }
            }
        }
        text.into_bytes()
    }

    /// The name of a tile as the archive of `model` and `city` writes it.
    fn member_name(model: Model, city: City, east_km: u32, north_km: u32) -> String {
        let prefix = model.prefix();
        match (model, city) {
            (Model::Dgm1, City::Bremen) => {
                format!("Gitternetz_DGM1_2017/{prefix}_32{east_km}_{north_km}_1_hb.xyz")
            }
            (Model::Dom1, City::Bremen) => {
                format!("Gitternetz_DOM1_2017/{prefix}_32{east_km}_{north_km}_1_hb.xyz")
            }
            (Model::Dgm1, City::Bremerhaven) => {
                format!("Gitternetz_DGM1_2015_BHV/{prefix}_32{east_km}__{north_km}_1_hb.xyz")
            }
            (Model::Dom1, City::Bremerhaven) => {
                format!("Gitternetz_DOM1_2015_BHV/{prefix}_32{east_km}_{north_km}_1_hb.xyz")
            }
        }
    }

    fn spelling(model: Model, city: City) -> Spelling {
        match (model, city) {
            (_, City::Bremen) => Spelling::WholeMetres,
            (Model::Dgm1, City::Bremerhaven) => Spelling::ZoneCentres,
            (Model::Dom1, City::Bremerhaven) => Spelling::ZoneWholeMetres,
        }
    }

    /// The archive of `model` for `city` with the tiles `tiles` (their ten northern rows), as
    /// the file server holds it: a folder, the tiles, and the thumbnail file of Windows.
    fn serve(fetcher: &MapFetcher, model: Model, city: City, tiles: &[(u32, u32, f32)]) {
        let mut members: Vec<(String, Vec<u8>)> = tiles
            .iter()
            .map(|(e, n, base)| {
                (
                    member_name(model, city, *e, *n),
                    xyz(spelling(model, city), *e, *n, 10, *base),
                )
            })
            .collect();
        members.push(("Gitternetz/Thumbs.db".to_string(), b"thumbs".to_vec()));
        let members: Vec<(&str, Vec<u8>, bool)> = members
            .iter()
            .map(|(name, bytes)| (name.as_str(), bytes.clone(), true))
            .collect();
        fetcher.insert(&model.archive_url(city), archive(&members));
    }

    fn read(out: &Path) -> Raster<f32> {
        crate::derive::terrain::read_float_raster(out).unwrap()
    }

    const CAPABILITIES: &str = "<Layer><Name>DOP Luftbilder Land Bremen</Name>\
        <Layer><Name>dop10_2025_HB</Name></Layer><Layer><Name>dop10_cir_2025_HB</Name></Layer>\
        <Layer><Name>dop10_2023_HB</Name></Layer><Layer><Name>dop20_2015_HB</Name></Layer>\
        <Layer><Name>luftbild_1997_HB</Name></Layer>\
        <Layer><Name>dop10_2023_BHV</Name></Layer><Layer><Name>dop10_cir_2025_BHV</Name></Layer>\
        <Layer><Name>dop20_2018_BHV</Name></Layer><Layer><Name>dop12_5_2004_BHV</Name></Layer>\
        <Layer><Name>luftbild_2000_BHV</Name></Layer></Layer>";

    /// Each city has its own flights. The newest colour image of each is asked for; infrared
    /// images and scanned photos are not, whatever their year.
    #[test]
    fn the_newest_flight_of_each_city_is_the_orthophoto() {
        assert_eq!(
            newest_layers(CAPABILITIES),
            Some(("dop10_2025_HB,dop10_2023_BHV".to_string(), [2025, 2023]))
        );
        // A resolution written with an underscore is still a flight.
        let old = "<Name>dop10_2002_HB</Name><Name>dop12_5_2004_BHV</Name>";
        assert_eq!(
            newest_layers(old),
            Some(("dop10_2002_HB,dop12_5_2004_BHV".to_string(), [2002, 2004]))
        );
        // A city without a flight: no answer, and the known layers are asked for.
        assert_eq!(newest_layers("<Name>dop10_2027_HB</Name>"), None);
        assert_eq!(newest_layers("<html>maintenance</html>"), None);
        assert_eq!(
            newest_layers("<Name>dop10_cir_2027_HB</Name><Name>luftbild_2027_BHV</Name>"),
            None
        );
        // The layers the adapter knows are layers of this kind.
        assert!(
            newest_layers(
                &KNOWN_LAYERS
                    .replace(',', "</Name><Name>")
                    .replace("dop", "<Name>dop")
            )
            .is_some_and(|(layers, years)| layers == KNOWN_LAYERS && years == [KNOWN_YEAR; 2])
        );
    }

    #[test]
    fn a_tile_lies_in_the_archive_of_its_city() {
        assert_eq!(City::of(tile(487, 5881)), City::Bremen);
        assert_eq!(City::of(tile(470, 5897)), City::Bremen);
        assert_eq!(City::of(tile(471, 5933)), City::Bremerhaven);
        assert_eq!(City::of(tile(466, 5924)), City::Bremerhaven);
        assert_eq!(
            Model::Dgm1.archive_url(City::Bremen),
            "https://gdi2.geo.bremen.de/inspire/download/DGM/data/Gitternetz_DGM1_2017_HB_ASCII_XYZ.zip"
        );
        assert_eq!(
            Model::Dgm1.archive_url(City::Bremerhaven),
            "https://gdi2.geo.bremen.de/inspire/download/DGM/data/Gitternetz_DGM1_2015_BHV_ASCII_XYZ.zip"
        );
        assert_eq!(
            Model::Dom1.archive_url(City::Bremen),
            "https://gdi2.geo.bremen.de/inspire/download/DOM/data/Gitternetz_DOM1_2017_HB_ASCII_XYZ.zip"
        );
        assert_eq!(
            Model::Dom1.archive_url(City::Bremerhaven),
            "https://gdi2.geo.bremen.de/inspire/download/DOM/data/Gitternetz_DOM1_2015_BHV_ASCII_XYZ.zip"
        );
    }

    /// The archives name their tiles alike, but the terrain of Bremerhaven has two underscores
    /// after the easting. A tile is found either way, and only by its own name.
    #[test]
    fn a_tile_is_found_whatever_its_underscores() {
        let names = [
            "Gitternetz_DGM1_2015_BHV/dgm1_32466__5924_1_hb.xyz",
            "Gitternetz_DGM1_2017/dgm1_32487_5881_1_hb.xyz",
            "Gitternetz_DOM1_2017/dom1_32487_5881_1_hb.xyz",
            "Gitternetz_DGM1_2017/Thumbs.db",
        ];
        let files: Vec<(&str, Vec<u8>, bool)> = names
            .iter()
            .map(|name| (*name, b"x y z".to_vec(), true))
            .collect();
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/a.zip", archive(&files));
        let members =
            crate::sources::zip_range::read_members(&fetcher, "https://example.org/a.zip").unwrap();
        assert_eq!(members.len(), 4);
        let name = |model: Model, tile: TileId| {
            model
                .member_of(&members, tile)
                .map(|member| member.name.as_str())
        };
        assert_eq!(
            name(Model::Dgm1, tile(466, 5924)),
            Some("Gitternetz_DGM1_2015_BHV/dgm1_32466__5924_1_hb.xyz")
        );
        assert_eq!(
            name(Model::Dgm1, tile(487, 5881)),
            Some("Gitternetz_DGM1_2017/dgm1_32487_5881_1_hb.xyz")
        );
        assert_eq!(
            name(Model::Dom1, tile(487, 5881)),
            Some("Gitternetz_DOM1_2017/dom1_32487_5881_1_hb.xyz")
        );
        assert_eq!(name(Model::Dom1, tile(466, 5924)), None);
        assert_eq!(name(Model::Dgm1, tile(487, 5882)), None);
        assert_eq!(name(Model::Dgm1, tile(48, 5881)), None);
    }

    /// The same heights in the three spellings of the archives come to the same raster: a
    /// point stands for the cell it lies in.
    #[test]
    fn the_points_of_a_tile_are_read_in_each_spelling() {
        let at = tile(471, 5933);
        let rasters: Vec<Raster<f32>> = [
            Spelling::WholeMetres,
            Spelling::ZoneCentres,
            Spelling::ZoneWholeMetres,
        ]
        .into_iter()
        .map(|spelling| parse_xyz(&xyz(spelling, 471, 5933, 3, 2.0), at).unwrap())
        .collect();
        let whole = &rasters[0];
        assert_eq!(whole.grid.west, 471_000.0);
        assert_eq!(whole.grid.north, 5_934_000.0);
        assert_eq!((whole.grid.width, whole.grid.height), (1000, 1000));
        // The north-west cell, the east end of the third row, and nothing below it.
        assert_eq!(whole.get(0, 0), 2.0);
        assert_eq!(whole.get(999, 2), 11.99);
        assert!(whole.get(0, 3).is_nan());
        for other in &rasters[1..] {
            assert!(
                other
                    .data
                    .iter()
                    .zip(&whole.data)
                    .all(|(a, b)| a == b || (a.is_nan() && b.is_nan()))
            );
        }
        // The southernmost row at whole metres is the last row of the tile.
        let south = parse_xyz(b"x y z\r\n471000 5933000 1.5\r\n", at).unwrap();
        assert_eq!(south.get(0, 999), 1.5);
        let centre = parse_xyz(b"32471999.5 5933000.5 1.5\n", at).unwrap();
        assert_eq!(centre.get(999, 999), 1.5);
    }

    #[test]
    fn a_file_without_a_point_of_the_tile_is_an_error() {
        let other = xyz(Spelling::WholeMetres, 487, 5881, 1, 1.0);
        let err = parse_xyz(&other, tile(488, 5881)).unwrap_err();
        assert_eq!(err, "no point of the tile 32_488_5881 in the file");
        assert!(parse_xyz(b"x y z\r\n", tile(488, 5881)).is_err());
        // A line that is no point, after the header.
        let err = parse_xyz(b"x y z\n487000 5881999 1.0\n<html>\n", tile(487, 5881)).unwrap_err();
        assert_eq!(err, "line 3 is not `E N height`: <html>");
    }

    /// The tiles of an area in Bremen are read out of the archive of the city, with the year
    /// of its scan; a second run reads them from the cache.
    #[test]
    fn the_terrain_is_read_from_the_archive_of_the_city() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(
            &fetcher,
            Model::Dgm1,
            City::Bremen,
            &[(487, 5881, 3.0), (488, 5881, 4.0)],
        );
        let (raster, found) =
            mosaic(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &bremen()).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (200, 10));
        // Column 900 of the western tile, column 0 of the eastern one.
        assert_eq!(raster.get(0, 5), 12.0);
        assert_eq!(raster.get(100, 5), 4.0);
        assert!(found.missing.is_empty());
        assert_eq!(found.date.as_deref(), Some("2017"));
        assert!(Model::Dgm1.is_cached(&cache, tile(487, 5881)));
        assert!(!Model::Dom1.is_cached(&cache, tile(487, 5881)));
        // A second run reads the tiles from the cache: only the directory is asked again
        // (the end of the archive, then the directory).
        let asked = fetcher.calls.get();
        let (again, _) =
            mosaic(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &bremen()).unwrap();
        assert_eq!(again, raster);
        assert_eq!(fetcher.calls.get() - asked, 2);
        // Offline nothing is asked, and the year is known all the same.
        let asked = fetcher.calls.get();
        let (offline, found) =
            mosaic(&fetcher, &cache, FetchMode::Offline, Model::Dgm1, &bremen()).unwrap();
        assert_eq!(offline, raster);
        assert_eq!(found.date.as_deref(), Some("2017"));
        assert_eq!(fetcher.calls.get(), asked);
    }

    /// Bremerhaven has archives of its own, from an older scan and in other spellings.
    #[test]
    fn bremerhaven_is_read_from_its_own_archives() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        for model in [Model::Dgm1, Model::Dom1] {
            serve(
                &fetcher,
                model,
                City::Bremerhaven,
                &[(471, 5933, 3.0), (472, 5933, 4.0)],
            );
            let (raster, found) =
                mosaic(&fetcher, &cache, FetchMode::Online, model, &bremerhaven()).unwrap();
            assert_eq!(raster.get(0, 5), 12.0, "{model:?}");
            assert_eq!(raster.get(100, 5), 4.0, "{model:?}");
            assert_eq!(found.date.as_deref(), Some("2015"));
            assert!(found.missing.is_empty());
        }
        // The archives of Bremen were not asked.
        let log = fetcher.log.borrow();
        assert!(log.iter().all(|url| url.contains("_BHV_")), "{log:?}");
    }

    /// Offline, an area whose tiles were never fetched says so.
    #[test]
    fn offline_without_the_tiles_says_to_fetch_online() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        let err = mosaic(&fetcher, &cache, FetchMode::Offline, Model::Dgm1, &bremen()).unwrap_err();
        assert!(err.to_string().contains("offline and not cached"), "{err}");
        assert_eq!(fetcher.calls.get(), 0);
    }

    /// A tile the archive does not hold is missing when it lies inside the state. The other
    /// tiles are read.
    #[test]
    fn a_tile_inside_the_state_that_the_archive_lacks_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dom1, City::Bremen, &[(487, 5881, 7.5)]);
        assert!(tile_centre_in(tile(488, 5881), BREMEN));
        let (raster, found) =
            mosaic(&fetcher, &cache, FetchMode::Online, Model::Dom1, &bremen()).unwrap();
        assert_eq!(raster.get(0, 5), 16.5);
        assert!(raster.get(150, 5).is_nan());
        assert_eq!(found.missing, [tile(488, 5881)]);
    }

    /// South of Bremen lies Lower Saxony: the archive holds no tile there, and none is
    /// missing.
    #[test]
    fn tiles_outside_the_state_stay_empty_without_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        // A strip from Stuhr in Lower Saxony north into Bremen-Huchting.
        let bbox = BoundingBox {
            min_e: 481_000.0,
            min_n: 5_869_990.0,
            max_e: 481_200.0,
            max_n: 5_877_000.0,
        };
        assert!(!tile_centre_in(tile(481, 5869), BREMEN));
        assert!(!tile_centre_in(tile(481, 5872), BREMEN));
        assert!(tile_centre_in(tile(481, 5876), BREMEN));
        serve(&fetcher, Model::Dgm1, City::Bremen, &[(481, 5876, 2.0)]);
        let (raster, found) =
            mosaic(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &bbox).unwrap();
        assert_eq!(raster.get(100, 5), 3.0);
        assert!(raster.get(100, 7_005).is_nan());
        for north_km in 5869..=5872 {
            assert!(!found.missing.contains(&tile(481, north_km)), "{north_km}");
        }
    }

    /// No tile at all is an error that names the file server, not an empty raster.
    #[test]
    fn no_tile_at_all_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        serve(&fetcher, Model::Dgm1, City::Bremen, &[(470, 5890, 1.0)]);
        let err = mosaic(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &bremen()).unwrap_err();
        let text = err.to_string();
        assert!(
            text.contains("no tile of the terrain model (GeoBremen DGM1)"),
            "{text}"
        );
        assert!(
            text.contains("https://gdi2.geo.bremen.de/inspire/download/DGM/data"),
            "{text}"
        );
    }

    /// The name of an archive carries the year of the scan. When the agency replaces it, the
    /// old address is gone: an error that names it, not a map without terrain.
    #[test]
    fn an_archive_that_is_gone_is_an_error_with_its_address() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        let err = mosaic(&fetcher, &cache, FetchMode::Online, Model::Dgm1, &bremen()).unwrap_err();
        assert!(matches!(err, FetchError::NotFound(_)), "{err:?}");
        assert!(
            err.to_string()
                .contains("Gitternetz_DGM1_2017_HB_ASCII_XYZ.zip"),
            "{err}"
        );
    }

    /// A project folder with its raster records.
    struct TestProject {
        dir: PathBuf,
        region: BoundingBox,
        tiles: Vec<freemapper_adapter_api::records::TileRecord>,
        ndsm_tiles: Vec<freemapper_adapter_api::records::NdsmRecord>,
        orthophoto: Option<freemapper_adapter_api::records::WmsRecord>,
        terrain: Option<WcsRecord>,
        surface: Option<WcsRecord>,
    }

    impl TestProject {
        fn new(dir: PathBuf, region: BoundingBox) -> Self {
            std::fs::create_dir_all(&dir).unwrap();
            Self {
                dir,
                region,
                tiles: Vec::new(),
                ndsm_tiles: Vec::new(),
                orthophoto: None,
                terrain: None,
                surface: None,
            }
        }
    }

    impl RasterProject for TestProject {
        fn records(&mut self) -> freemapper_adapter_api::project::RasterRecords<'_> {
            freemapper_adapter_api::project::RasterRecords {
                tiles: &mut self.tiles,
                ndsm_tiles: &mut self.ndsm_tiles,
                orthophoto: &mut self.orthophoto,
                terrain: &mut self.terrain,
                surface: &mut self.surface,
            }
        }

        fn save(&mut self) -> Result<(), AdapterError> {
            Ok(())
        }

        fn dir(&self) -> &Path {
            &self.dir
        }

        fn region_bbox(&self) -> BoundingBox {
            self.region
        }

        fn crs(&self) -> Crs {
            Crs::Utm32
        }
    }

    fn serve_orthophoto(fetcher: &MapFetcher, layers: &str, bbox: &BoundingBox) {
        let photo = GridSpec::covering(bbox, crate::sources::wms::ORTHOPHOTO_PIXEL_M);
        for (piece, url) in crate::sources::wms::pieces(&wms(layers), &photo) {
            fetcher.insert(&url, solid_jpeg(piece.width, piece.height, [90, 110, 70]));
        }
    }

    /// What a fetch of `bbox` stored and said.
    fn fetched(
        fetcher: &MapFetcher,
        scratch: &Path,
        bbox: &BoundingBox,
    ) -> (TestProject, Vec<String>) {
        let cache = Cache::new(scratch.join("cache"));
        let mut project = TestProject::new(scratch.join("project"), *bbox);
        let registry = freemapper_adapter_api::registry::Registry::new();
        let mut notes = Vec::new();
        let mut env = AdapterEnv {
            fetcher,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: "2026-11-13".parse().unwrap(),
            skip_surface: false,
            progress: &mut |n| notes.push(n.to_string()),
        };
        fetch_rasters(&mut env, &mut project, bbox).unwrap();
        (project, notes)
    }

    /// The backend stores the orthophoto of the newest flight with its year, and both models
    /// with the year of the scan. The surface model is the DOM1 as it is. It is eight years
    /// older than the image, and the fetch says so.
    #[test]
    fn the_rasters_of_bremen_carry_their_years() {
        let bbox = bremen();
        let scratch = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        let tiles = [(487, 5881, 3.0), (488, 5881, 4.0)];
        serve(&fetcher, Model::Dgm1, City::Bremen, &tiles);
        serve(&fetcher, Model::Dom1, City::Bremen, &[(487, 5881, 10.5)]);
        fetcher.insert_prefix(
            &format!("{ORTHOPHOTOS}?SERVICE=WMS&REQUEST=GetCapabilities"),
            CAPABILITIES.as_bytes().to_vec(),
        );
        serve_orthophoto(&fetcher, "dop10_2025_HB,dop10_2023_BHV", &bbox);
        let (project, notes) = fetched(&fetcher, scratch.path(), &bbox);
        let terrain = read(&project.dir.join(TERRAIN_RELPATH));
        let surface = read(&project.dir.join(SURFACE_RELPATH));
        assert_eq!(terrain.get(0, 5), 12.0);
        assert_eq!(terrain.get(150, 5), 4.5);
        assert_eq!(surface.get(0, 5), 19.5);
        assert!(surface.get(150, 5).is_nan());
        let record = |date: &str| {
            Some(WcsRecord {
                fetched: "2026-11-13".parse().unwrap(),
                bbox: stored_bbox(&bbox),
                date: Some(date.to_string()),
            })
        };
        assert_eq!(project.terrain, record("2017"));
        assert_eq!(project.surface, record("2017"));
        let orthophoto = project.orthophoto.as_ref().unwrap();
        assert_eq!(orthophoto.flight.as_deref(), Some("2025"));
        assert_eq!(orthophoto.flight_label.as_deref(), Some("Bildflugjahr"));
        let warnings: Vec<&String> = notes.iter().filter(|n| n.starts_with("warning")).collect();
        assert_eq!(
            warnings,
            [
                "warning: the surface model (GeoBremen DOM1) has no tile 32_488_5881 inside \
                 Bremen; the templates stay empty there",
                "warning: the surface model (GeoBremen DOM1) is from 2017, the orthophoto from \
                 2025. The nDSM may lack newer buildings and trees."
            ]
        );
    }

    /// In Bremerhaven the year of the image is that of its own flight, and the German warning
    /// reads as a sentence.
    #[test]
    fn bremerhaven_carries_the_year_of_its_own_flight() {
        let bbox = bremerhaven();
        let scratch = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        let tiles = [(471, 5933, 3.0), (472, 5933, 4.0)];
        serve(&fetcher, Model::Dgm1, City::Bremerhaven, &tiles);
        serve(&fetcher, Model::Dom1, City::Bremerhaven, &tiles);
        fetcher.insert_prefix(
            &format!("{ORTHOPHOTOS}?SERVICE=WMS&REQUEST=GetCapabilities"),
            CAPABILITIES.as_bytes().to_vec(),
        );
        serve_orthophoto(&fetcher, "dop10_2025_HB,dop10_2023_BHV", &bbox);
        let (project, _) = fetched(&fetcher, scratch.path(), &bbox);
        assert_eq!(
            project.orthophoto.as_ref().unwrap().flight.as_deref(),
            Some("2023")
        );
        assert_eq!(
            project.surface.as_ref().unwrap().date.as_deref(),
            Some("2015")
        );
        let german = note!(
            "warning-surface-much-older",
            model = crate::note::model(Model::Dom1.label()),
            surface = year_words("2015"),
            flight = year_words("2023")
        )
        .text(
            &freemapper_adapter_api::note::NOTES
                .parse_lang_arg("de")
                .unwrap(),
            &|e, _| e.to_string(),
        );
        assert_eq!(
            german,
            "Warnung: Das Oberflächenmodell (GeoBremen DOM1) ist vom Jahr 2015, das Orthophoto \
             vom Jahr 2023. Im nDSM können neuere Gebäude und Bäume fehlen."
        );
    }

    /// When the capabilities cannot be read, the layers known to the adapter are asked for,
    /// and the image carries their year.
    #[test]
    fn without_the_capabilities_the_known_flights_are_asked_for() {
        let bbox = bremen();
        let scratch = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        let tiles = [(487, 5881, 3.0), (488, 5881, 4.0)];
        serve(&fetcher, Model::Dgm1, City::Bremen, &tiles);
        serve(&fetcher, Model::Dom1, City::Bremen, &tiles);
        serve_orthophoto(&fetcher, KNOWN_LAYERS, &bbox);
        let (project, _) = fetched(&fetcher, scratch.path(), &bbox);
        assert_eq!(
            project.orthophoto.as_ref().unwrap().flight.as_deref(),
            Some("2025")
        );
    }

    #[test]
    fn the_backend_names_its_provider() {
        let backend = GeoBremenBackend::new();
        let info = backend.provider();
        assert_eq!(info.id.as_str(), "geobremen");
        assert_eq!(info.cache_dir.as_deref(), Some("geobremen"));
        assert_eq!(info.terrain_source, "geobremen-dgm1");
        assert_eq!(info.kind, RasterKind::PerArea);
    }

    #[test]
    fn the_alkis_is_the_full_alkis_of_the_state() {
        let source = alkis();
        assert_eq!(
            source.service.url,
            "https://opendata.lgln.niedersachsen.de/doorman/noauth/alkishb_wfs_sf"
        );
        assert_eq!(source.service.crs, Crs::Utm32);
        assert_eq!(source.service.region.as_str(), "bremen");
        assert_eq!(source.square_bez.as_deref(), Some("ohne Unterart (Bremen)"));
        assert_eq!(source.cache_dir, "geobremen");
    }
}
