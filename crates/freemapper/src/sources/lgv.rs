//! LGV Hamburg open data (dl-de/by-2-0, "Freie und Hansestadt Hamburg, Landesbetrieb
//! Geoinformation und Vermessung (LGV)"), all in EPSG:25832: ALKIS from the OGC API with its
//! ASCII spellings normalised to the LGB ones.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{Cache, FetchError, FetchMode, Fetcher, write_cache_file};
use crate::raster::{GridSpec, Raster};
use crate::sources::float_tiff::{decode_float_tiff, write_float_raster};
use crate::sources::region::{Region, tile_centre_in};
use crate::sources::wms::WmsService;
use crate::sources::zip_range::{
    cached_member, find_member, parse_directory, read_directory_bytes, refused,
};
use crate::tiles::{TileId, tiles_covering};

/// The orthophoto WMS: the leaf-off True-DOP time series, pinned to the open 2026 vintage (a
/// missing `TIME` would follow the newest, and 2023 is a closed-licence satellite image).
pub const WMS: WmsService = WmsService {
    url: Cow::Borrowed("https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt"),
    layer: Cow::Borrowed("dop_zeitreihe_unbelaubt"),
    crs: Crs::Utm32,
    extra: Cow::Borrowed("&TIME=2026"),
    max_px: 4000,
};
/// The flight dates of the DOP tiles, per vintage.
pub const METADATA_URL: &str = "https://www.daten-hamburg.de/opendata/fernerkundung_luftbilder/dop_unbelaubt/DOP_unbelaubt_AdV-Metadaten_Hamburg.zip";
const VINTAGE: &str = "2026";

/// The flight date (`DD.MM.YYYY`) of the orthophoto tile at the centre of `bbox`
/// (EPSG:25832); `None` when the metadata does not tell it.
pub fn flight_date(fetcher: &dyn Fetcher, bbox: &BoundingBox) -> Option<String> {
    use std::io::Read as _;
    let bytes = crate::fetch::get_with_retries(fetcher, METADATA_URL, |b| Ok(b.to_vec())).ok()?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let wanted = format!("AdV-Metadaten_DOP_{VINTAGE}_unbelaubt_hh.csv");
    let index = (0..archive.len()).find(|&i| {
        archive.by_index(i).is_ok_and(|f| {
            f.name()
                .is_ok_and(|n| n.rsplit('/').next() == Some(wanted.as_str()))
        })
    })?;
    let mut raw = Vec::new();
    archive.by_index(index).ok()?.read_to_end(&mut raw).ok()?;
    // UTF-8 with BOM in the newer vintages, Latin-1 in the older ones.
    let text = String::from_utf8(raw.clone())
        .unwrap_or_else(|_| raw.iter().map(|&b| char::from(b)).collect());
    let (e, n) = (
        ((bbox.min_e + bbox.max_e) / 2000.0).floor(),
        ((bbox.min_n + bbox.max_n) / 2000.0).floor(),
    );
    let tile = format!("dop20rgbi_32_{e:03}_{n:04}_1_hh_{VINTAGE}.tif");
    text.lines().find_map(|line| {
        let mut fields = line.split(';');
        (fields.next()? == tile)
            .then(|| fields.next().map(str::to_string))
            .flatten()
    })
}

/// A statewide height model archive (1 km Float32 GeoTIFF tiles, 2022).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// DGM1 from the 2022 laser scan.
    Dgm1,
    /// bDOM, the image-based 1 m surface model of 2022.
    Bdom,
}

impl Model {
    pub fn archive_url(self) -> &'static str {
        match self {
            Self::Dgm1 => {
                "https://archiv.transparenz.hamburg.de/hmbtgarchive/HMDK/dgm1_hh_2022-04-30_258334_snap_1.zip"
            }
            Self::Bdom => {
                "https://www.daten-hamburg.de/opendata/Digitales_Hoehenmodell_bDOM/dom1_hh_2022-11-21.zip"
            }
        }
    }

    /// The archive's file stem; the cache keeps each release apart, so a newer archive is
    /// never mixed with tiles of an older one.
    pub fn release(self) -> &'static str {
        let file = self.archive_url().rsplit('/').next().unwrap_or_default();
        file.strip_suffix(".zip").unwrap_or(file)
    }

    /// When the release was surveyed, as recorded in the project.
    pub fn date(self) -> &'static str {
        match self {
            Self::Dgm1 => "2022-03",
            Self::Bdom => "2022",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dgm1 => "terrain model (LGV DGM1 2022)",
            Self::Bdom => "surface model (LGV bDOM 2022)",
        }
    }

    /// Start of the tile file names, e.g. `dgm1_32_565_5937_1_hh_2022.tif`.
    fn file_prefix(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Bdom => "dom1",
        }
    }

    /// Where a tile of this model is cached: named after the tile, never the archive member.
    pub fn tile_cache_file(self, cache: &Cache, tile: TileId) -> PathBuf {
        cache
            .lgv_dir()
            .join(self.cache_name())
            .join(self.release())
            .join(format!(
                "{}_32_{:03}_{:04}_.tif",
                self.file_prefix(),
                tile.east_km,
                tile.north_km
            ))
    }

    fn cache_name(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Bdom => "bdom",
        }
    }
}

/// `tile 32_566_5937` or `tiles 32_566_5937, 32_567_5937`, as LGV names its 1 km tiles.
pub fn tile_names(tiles: &[TileId]) -> String {
    let noun = if tiles.len() == 1 { "tile" } else { "tiles" };
    format!("{noun} {}", names(tiles))
}

/// The names of `tiles` as the providers write them (`32_565_5937`), joined by commas.
pub(crate) fn names(tiles: &[TileId]) -> String {
    tiles
        .iter()
        .map(|t| format!("32_{:03}_{:04}", t.east_km, t.north_km))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), read out of the statewide archive and
/// cached, mosaicked into a Float32 GeoTIFF `out` with its world file. Tiles outside Hamburg
/// are missing from the archive and stay empty; returned are the missing tiles inside Hamburg.
/// No tile at all is an error (the archive was renamed or restructured).
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    root: &Path,
    out: &Path,
) -> Result<Vec<TileId>, FetchError> {
    let grid = GridSpec::covering(bbox, 1.0);
    let mut raster = Raster::filled(grid, f32::NAN);
    let dir = cache
        .lgv_dir()
        .join(model.cache_name())
        .join(model.release());
    // The member count (u16, little-endian), then the central directory.
    let directory_file = dir.join("directory.bin");
    let url = model.archive_url();
    let offline_hint = |what: &str| {
        refused(
            url,
            format!("offline and not cached: {what}; run `freemapper fetch` online"),
        )
    };
    let members = match mode {
        FetchMode::Online => {
            let (directory, count) = read_directory_bytes(fetcher, url)?;
            let members =
                parse_directory(&directory, count).map_err(|message| refused(url, message))?;
            // Kept only once it parsed, so a broken answer never replaces a good copy.
            write_cache_file(
                &directory_file,
                &[&count.to_le_bytes()[..], &directory].concat(),
            )?;
            members
        }
        FetchMode::Offline => {
            let cached = std::fs::read(&directory_file).map_err(|_| offline_hint(model.label()))?;
            let (count, directory) = cached
                .split_at_checked(2)
                .ok_or_else(|| offline_hint(model.label()))?;
            parse_directory(directory, u16::from_le_bytes([count[0], count[1]]))
                .map_err(|message| refused(url, message))?
        }
    };
    let mut found = 0;
    let mut missing = Vec::new();
    for tile in tiles_covering(bbox) {
        let prefix = format!(
            "{}_32_{:03}_{:04}_",
            model.file_prefix(),
            tile.east_km,
            tile.north_km
        );
        let Some(member) = find_member(&members, |file| {
            file.starts_with(&prefix) && file.ends_with(".tif")
        }) else {
            if tile_centre_in(tile, Region::Hamburg) {
                missing.push(tile);
            }
            continue;
        };
        found += 1;
        // Named after the tile, never after the archive's member name (which comes from the
        // server and could hold `..` or `\`).
        let file = dir.join(format!("{prefix}.tif"));
        let bytes = cached_member(fetcher, mode, url, member, &file, || {
            offline_hint(&member.name)
        })?;
        let part = decode_float_tiff(&bytes).map_err(|message| refused(url, message))?;
        raster
            .paste_with(&part.grid, |x, y| part.get(x, y))
            .map_err(|e| refused(url, e.to_string()))?;
    }
    if found == 0 {
        return Err(refused(
            url,
            format!(
                "no tile of the {} covers the map area; the archive may have been renamed",
                model.label()
            ),
        ));
    }
    write_float_raster(&raster, root, out, url)?;
    Ok(missing)
}

/// The ALKIS-vereinfacht OGC API collections (LGB-like JSON, `next` links, EPSG:25832).
pub const BUILDINGS_ENDPOINT: &str =
    "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/GebaeudeBauwerk/items";
pub const LAND_USE_ENDPOINT: &str =
    "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/NutzungFlurstueck/items";

/// The LGB spellings of `gebnutzbez` the build rules know.
pub(crate) const BUILDING_KINDS: [&str; 3] = [
    "Gebäude",
    "Bauteil",
    "Sonstiges Bauwerk oder sonstige Einrichtung",
];
/// The LGB spellings of `nutzart` (the AdV class names).
const LAND_USE_CLASSES: [&str; 26] = [
    "Wohnbaufläche",
    "Industrie- und Gewerbefläche",
    "Halde",
    "Bergbaubetrieb",
    "Tagebau, Grube, Steinbruch",
    "Fläche gemischter Nutzung",
    "Fläche besonderer funktionaler Prägung",
    "Sport-, Freizeit- und Erholungsfläche",
    "Friedhof",
    "Straßenverkehr",
    "Weg",
    "Platz",
    "Bahnverkehr",
    "Flugverkehr",
    "Schiffsverkehr",
    "Landwirtschaft",
    "Wald",
    "Gehölz",
    "Heide",
    "Moor",
    "Sumpf",
    "Unland/Vegetationslose Fläche",
    "Fließgewässer",
    "Hafenbecken",
    "Stehendes Gewässer",
    "Meer",
];

/// A Hamburg building feature with the LGB spellings (`Gebaeude` → `Gebäude`).
pub fn normalise_building(feature: Value) -> Value {
    normalise(feature, "gebnutzbez", &BUILDING_KINDS)
}

/// A Hamburg land use feature with the LGB `nutzart` spellings (`Wohnbauflaeche` →
/// `Wohnbaufläche`); unknown classes are kept.
pub fn normalise_land_use(feature: Value) -> Value {
    normalise(feature, "nutzart", &LAND_USE_CLASSES)
}

/// `feature` with `property` replaced by the entry of `names` it spells in ASCII.
fn normalise(mut feature: Value, property: &str, names: &[&str]) -> Value {
    let Some(value) = feature
        .get_mut("properties")
        .and_then(|p| p.get_mut(property))
    else {
        return feature;
    };
    if let Some(text) = value.as_str() {
        let key = spelling_key(text);
        if let Some(name) = names.iter().find(|name| spelling_key(name) == key) {
            *value = Value::from(*name);
        }
    }
    feature
}

/// A key equal for "Industrie- und Gewerbefläche" and "Industrie Und Gewerbeflaeche": lower
/// case, umlauts and ß written out, punctuation dropped, single spaces.
pub(crate) fn spelling_key(text: &str) -> String {
    let mut key = String::new();
    for c in text.to_lowercase().chars() {
        match c {
            'ä' => key.push_str("ae"),
            'ö' => key.push_str("oe"),
            'ü' => key.push_str("ue"),
            'ß' => key.push_str("ss"),
            '-' | ',' | '/' => key.push(' '),
            c => key.push(c),
        }
    }
    key.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const BUILDINGS: &str = include_str!("../../../../tests/fixtures/hamburg-buildings.json");
    pub const LAND_USE: &str = include_str!("../../../../tests/fixtures/hamburg-land-use.json");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::sources::alkis::{parse_buildings, parse_land_use};
    use crate::sources::zip_range::test_support::archive;

    fn normalised(collection: &str, f: fn(Value) -> Value) -> String {
        let mut v: Value = serde_json::from_str(collection).unwrap();
        let features: Vec<Value> = v["features"]
            .as_array()
            .unwrap()
            .iter()
            .cloned()
            .map(f)
            .collect();
        v["features"] = Value::from(features);
        v.to_string()
    }

    #[test]
    fn hamburg_spellings_are_normalised() {
        let land = parse_land_use(&normalised(LAND_USE, normalise_land_use)).unwrap();
        let classes: Vec<(&str, Option<&str>)> = land
            .iter()
            .map(|l| (l.nutzart.as_str(), l.bez.as_deref()))
            .collect();
        assert_eq!(
            classes,
            [
                (
                    "Fläche besonderer funktionaler Prägung",
                    Some("Bildung und Wissenschaft")
                ),
                ("Straßenverkehr", Some("Begleitfläche Straßenverkehr")),
                ("Wohnbaufläche", None),
                ("Platz", Some("Parkplatz")),
                (
                    "Industrie- und Gewerbefläche",
                    Some("Handel und Dienstleistung")
                ),
            ]
        );
        let unknown = serde_json::json!({"type":"Feature","geometry":null,"properties":{"oid":"DEHH1","nutzart":"Kaum Bekannt"}});
        assert_eq!(
            normalise_land_use(unknown)["properties"]["nutzart"],
            "Kaum Bekannt"
        );
        for (hamburg, lgb) in [
            (
                "Sport Freizeit Und Erholungsflaeche",
                "Sport-, Freizeit- und Erholungsfläche",
            ),
            ("Fliessgewaesser", "Fließgewässer"),
            (
                "Unland Vegetationslose Flaeche",
                "Unland/Vegetationslose Fläche",
            ),
            ("Flaeche Gemischter Nutzung", "Fläche gemischter Nutzung"),
            ("Gehoelz", "Gehölz"),
        ] {
            let f = serde_json::json!({"type":"Feature","geometry":null,"properties":{"oid":"DEHH1","nutzart":hamburg}});
            assert_eq!(
                normalise_land_use(f)["properties"]["nutzart"],
                lgb,
                "{hamburg}"
            );
        }
    }

    #[test]
    fn hamburg_buildings_keep_the_kind_rule() {
        let parsed = parse_buildings(&normalised(BUILDINGS, normalise_building)).unwrap();
        let kinds: Vec<&str> = parsed.buildings.iter().map(|b| b.kind.as_str()).collect();
        assert_eq!(kinds, ["Gebäude", "Bauteil", "Gebäude", "Gebäude"]);
        let rules: Vec<crate::build::buildings::Rule> = parsed
            .buildings
            .iter()
            .map(crate::build::buildings::classify)
            .collect();
        assert_eq!(
            rules[0],
            crate::build::buildings::Rule::Object,
            "the school is drawn"
        );
        assert_ne!(
            rules[2],
            crate::build::buildings::Rule::Object,
            "underground Tiefgarage"
        );
        assert!(parsed.buildings[1].underground && parsed.buildings[2].underground);
    }

    use crate::fetch::test_support::{MapFetcher, geotiff};
    use crate::fetch::{Cache, FetchMode};
    use crate::raster::GridSpec;
    use crate::tiles::TileId;

    fn tile(e_km: u32, n_km: u32, value: f32) -> Vec<u8> {
        let grid = GridSpec {
            west: f64::from(e_km) * 1000.0 - 1e-10,
            north: f64::from(n_km + 1) * 1000.0 + 1e-9,
            pixel_size: 1.0,
            width: 1000,
            height: 1000,
        };
        let mut values = vec![value; 1000 * 1000];
        // The state edge: NoData in the first row, both ways LGV writes it.
        values[0] = -9999.0;
        values[1] = -3.402_823_5e38;
        geotiff(&grid, &values)
    }

    #[test]
    fn tiles_mosaic_into_the_area() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let zip = archive(&[
            (
                "d/s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            ),
            (
                "d/s32_566/dgm1_32_566_5937_1_hh_2022.tif",
                tile(566, 5937, 8.0),
                true,
            ),
        ]);
        let fetcher = MapFetcher::default();
        fetcher.insert(Model::Dgm1.archive_url(), zip);
        // Across the tile edge E 566 000.
        let bbox = BoundingBox {
            min_e: 565_900.0,
            min_n: 5_937_500.0,
            max_e: 566_100.0,
            max_n: 5_937_600.0,
        };
        let out = dir.path().join("dgm1.tif");
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!((raster.grid.width, raster.grid.height), (200, 100));
        assert_eq!(raster.get(10, 50), 6.0);
        assert_eq!(raster.get(190, 50), 8.0);
        // Offline the cached members are used.
        let offline = MapFetcher::default();
        fetch_model(
            &offline,
            &cache,
            FetchMode::Offline,
            Model::Dgm1,
            &bbox,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        assert_eq!(offline.calls.get(), 0);
    }

    #[test]
    fn member_names_never_name_cache_files() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let zip = archive(&[(
            "..\\..\\escape\\dgm1_32_565_5937_1_hh_2022.tif",
            tile(565, 5937, 6.0),
            true,
        )]);
        let fetcher = MapFetcher::default();
        fetcher.insert(Model::Dgm1.archive_url(), zip);
        let bbox = BoundingBox {
            min_e: 565_000.0,
            min_n: 5_937_000.0,
            max_e: 565_010.0,
            max_n: 5_937_010.0,
        };
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            dir.path(),
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert!(
            release_dir(&cache, Model::Dgm1)
                .join("dgm1_32_565_5937_.tif")
                .exists()
        );
        assert!(!dir.path().join("escape").exists());
    }

    #[test]
    fn nodata_values_become_nan() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let zip = archive(&[(
            "s32_565/dgm1_32_565_5937_1_hh_2022.tif",
            tile(565, 5937, 6.0),
            true,
        )]);
        let fetcher = MapFetcher::default();
        fetcher.insert(Model::Dgm1.archive_url(), zip);
        // The tile's north-west corner.
        let bbox = BoundingBox {
            min_e: 565_000.0,
            min_n: 5_937_990.0,
            max_e: 565_010.0,
            max_n: 5_938_000.0,
        };
        let out = dir.path().join("dgm1.tif");
        fetch_model(
            &fetcher,
            &cache,
            FetchMode::Online,
            Model::Dgm1,
            &bbox,
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert!(raster.get(0, 0).is_nan() && raster.get(1, 0).is_nan());
        assert_eq!(raster.get(2, 0), 6.0);
    }

    fn release_dir(cache: &Cache, model: Model) -> std::path::PathBuf {
        cache
            .lgv_dir()
            .join(model.cache_name())
            .join(model.release())
    }

    /// The tile 565/5937 (Gymnasium Eppendorf) and, with `both`, its eastern neighbour.
    fn eppendorf(both: bool) -> BoundingBox {
        BoundingBox {
            min_e: 565_900.0,
            min_n: 5_937_500.0,
            max_e: if both { 566_100.0 } else { 565_990.0 },
            max_n: 5_937_600.0,
        }
    }

    fn fetch_dgm(
        fetcher: &dyn Fetcher,
        cache: &Cache,
        mode: FetchMode,
        bbox: &BoundingBox,
        out: &Path,
    ) -> Result<Vec<TileId>, FetchError> {
        fetch_model(
            fetcher,
            cache,
            mode,
            Model::Dgm1,
            bbox,
            out.parent().unwrap(),
            out,
        )
    }

    #[test]
    fn failed_directory_read_keeps_the_cached_one() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let out = dir.path().join("dgm1.tif");
        let fetcher = MapFetcher::default();
        fetcher.insert(
            Model::Dgm1.archive_url(),
            archive(&[(
                "s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            )]),
        );
        fetch_dgm(&fetcher, &cache, FetchMode::Online, &eppendorf(false), &out).unwrap();
        // A broken directory: 100 bytes of junk, then an end record pointing at them.
        let mut broken = vec![b'x'; 100];
        broken.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 1, 0, 1, 0]);
        broken.extend_from_slice(&100u32.to_le_bytes());
        broken.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        let bad = MapFetcher::default();
        bad.insert(Model::Dgm1.archive_url(), broken);
        assert!(fetch_dgm(&bad, &cache, FetchMode::Online, &eppendorf(false), &out).is_err());
        fetch_dgm(
            &MapFetcher::default(),
            &cache,
            FetchMode::Offline,
            &eppendorf(false),
            &out,
        )
        .unwrap();
    }

    #[test]
    fn renamed_tiles_are_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        fetcher.insert(
            Model::Dgm1.archive_url(),
            archive(&[("dgm1_hh_32_565_5937.tif", tile(565, 5937, 6.0), true)]),
        );
        let err = fetch_dgm(
            &fetcher,
            &cache,
            FetchMode::Online,
            &eppendorf(false),
            &dir.path().join("o.tif"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no tile"), "{err}");
    }

    #[test]
    fn missing_tiles_inside_hamburg_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        fetcher.insert(
            Model::Dgm1.archive_url(),
            archive(&[(
                "s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            )]),
        );
        let missing = fetch_dgm(
            &fetcher,
            &cache,
            FetchMode::Online,
            &eppendorf(true),
            &dir.path().join("o.tif"),
        )
        .unwrap();
        assert_eq!(
            missing,
            [TileId {
                east_km: 566,
                north_km: 5937
            }]
        );
    }

    #[test]
    fn archive_reads_are_retried() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let fetcher = MapFetcher::default();
        fetcher.insert(
            Model::Dgm1.archive_url(),
            archive(&[(
                "s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            )]),
        );
        fetcher
            .flaky
            .borrow_mut()
            .insert(Model::Dgm1.archive_url().to_string(), 2);
        fetch_dgm(
            &fetcher,
            &cache,
            FetchMode::Online,
            &eppendorf(false),
            &dir.path().join("o.tif"),
        )
        .unwrap();
    }

    #[test]
    fn tile_names_are_in_zone_32() {
        let tiles = [
            TileId {
                east_km: 566,
                north_km: 5937,
            },
            TileId {
                east_km: 567,
                north_km: 5937,
            },
        ];
        assert_eq!(tile_names(&tiles), "tiles 32_566_5937, 32_567_5937");
        assert_eq!(tile_names(&tiles[..1]), "tile 32_566_5937");
    }

    #[test]
    fn cache_is_kept_per_release() {
        assert_eq!(Model::Dgm1.release(), "dgm1_hh_2022-04-30_258334_snap_1");
        assert_eq!(Model::Bdom.release(), "dom1_hh_2022-11-21");
    }

    #[test]
    fn damaged_cached_tile_is_read_again() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let out = dir.path().join("dgm1.tif");
        let fetcher = MapFetcher::default();
        fetcher.insert(
            Model::Dgm1.archive_url(),
            archive(&[(
                "s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            )]),
        );
        fetch_dgm(&fetcher, &cache, FetchMode::Online, &eppendorf(false), &out).unwrap();
        let cached = release_dir(&cache, Model::Dgm1).join("dgm1_32_565_5937_.tif");
        std::fs::write(&cached, b"damaged").unwrap();
        let err = fetch_dgm(
            &MapFetcher::default(),
            &cache,
            FetchMode::Offline,
            &eppendorf(false),
            &out,
        )
        .unwrap_err();
        assert!(err.to_string().contains("online"), "{err}");
        fetch_dgm(&fetcher, &cache, FetchMode::Online, &eppendorf(false), &out).unwrap();
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert_eq!(raster.get(10, 50), 6.0);
    }

    #[test]
    fn feature_without_object_properties_is_kept() {
        let feature = serde_json::json!({"type": "Feature", "properties": "broken"});
        assert_eq!(normalise_land_use(feature.clone()), feature);
        let feature = serde_json::json!({"type": "Feature"});
        assert_eq!(normalise_building(feature.clone()), feature);
    }

    #[test]
    fn wms_asks_for_the_2026_vintage() {
        let grid = GridSpec::covering(
            &BoundingBox {
                min_e: 564_950.0,
                min_n: 5_937_650.0,
                max_e: 565_350.0,
                max_n: 5_937_950.0,
            },
            0.2,
        );
        let (_, url) = &crate::sources::wms::pieces(&WMS, &grid)[0];
        assert!(
            url.starts_with("https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt?"),
            "{url}"
        );
        assert!(url.contains("LAYERS=dop_zeitreihe_unbelaubt&"), "{url}");
        assert!(
            url.contains("&CRS=EPSG:25832&") && url.ends_with("&TIME=2026"),
            "{url}"
        );
    }

    #[test]
    fn flight_date_from_the_metadata_csv() {
        let csv = "\u{feff}Kachelinformation der DOP20 für die Datenabgabe;;\nLand;Hamburg ;;\nEigentuemer;LGV;;\nAktualitaet_Kachelinformation;10.08.2026;;\nVersion_Standard;4.1;;\nKachelname;Aktualitaet;Erfassungsmethode\ndop20rgbi_32_564_5937_1_hh_2026.tif;18.03.2026;0\ndop20rgbi_32_565_5937_1_hh_2026.tif;19.03.2026;0\n";
        let zip = archive(&[(
            "DOP_unbelaubt_AdV-Metadaten_Hamburg/AdV-Metadaten_DOP_2026_unbelaubt_hh.csv",
            csv.as_bytes().to_vec(),
            true,
        )]);
        let fetcher = MapFetcher::default();
        fetcher.insert(METADATA_URL, zip);
        let school = BoundingBox {
            min_e: 564_950.0,
            min_n: 5_937_650.0,
            max_e: 565_350.0,
            max_n: 5_937_950.0,
        };
        assert_eq!(
            flight_date(&fetcher, &school).as_deref(),
            Some("19.03.2026")
        );
        let elsewhere = BoundingBox {
            min_e: 500_000.0,
            min_n: 5_900_000.0,
            max_e: 500_100.0,
            max_n: 5_900_100.0,
        };
        assert_eq!(flight_date(&fetcher, &elsewhere), None);
    }
}
