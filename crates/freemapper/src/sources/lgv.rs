//! LGV Hamburg open data (dl-de/by-2-0, "Freie und Hansestadt Hamburg, Landesbetrieb
//! Geoinformation und Vermessung (LGV)"), all in EPSG:25832: ALKIS from the OGC API with its
//! ASCII spellings normalised to the LGB ones.

use std::path::Path;

use serde_json::Value;

use crate::crs::BoundingBox;
use crate::fetch::{ByteRange, Cache, FetchError, FetchMode, Fetcher, write_atomically};
use crate::raster::{GridSpec, Raster};
use crate::sources::lvermgeo::{decode_float_tiff, write_float_raster};
use crate::tiles::tiles_covering;

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

    fn cache_name(self) -> &'static str {
        match self {
            Self::Dgm1 => "dgm1",
            Self::Bdom => "bdom",
        }
    }
}

/// One member of a zip archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    method: u16,
    compressed: u64,
    local_header: u64,
}

/// Where the central directory lies (offset, size), from the end of a zip archive.
pub fn directory_location(tail: &[u8]) -> Result<(u64, u64), String> {
    const END: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    let at = tail
        .windows(4)
        .rposition(|w| w == END)
        .ok_or("no end of central directory: not a zip archive")?;
    let record = tail
        .get(at..at + 22)
        .ok_or("truncated end of central directory")?;
    Ok((u64::from(u32_at(record, 16)), u64::from(u32_at(record, 12))))
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// The members listed in a central directory.
pub fn parse_directory(directory: &[u8]) -> Result<Vec<Member>, String> {
    const ENTRY: u32 = 0x0201_4b50;
    let mut members = Vec::new();
    let mut at = 0;
    while at + 46 <= directory.len() {
        if u32_at(directory, at) != ENTRY {
            return Err(format!("broken central directory at byte {at}"));
        }
        let name_len = usize::from(u16_at(directory, at + 28));
        let extra_len = usize::from(u16_at(directory, at + 30));
        let comment_len = usize::from(u16_at(directory, at + 32));
        let name = directory
            .get(at + 46..at + 46 + name_len)
            .ok_or("truncated central directory")?;
        members.push(Member {
            name: String::from_utf8_lossy(name).into_owned(),
            method: u16_at(directory, at + 10),
            compressed: u64::from(u32_at(directory, at + 20)),
            local_header: u64::from(u32_at(directory, at + 42)),
        });
        at += 46 + name_len + extra_len + comment_len;
    }
    Ok(members)
}

/// The members of the archive at `url`, read by two byte ranges.
pub fn read_directory(fetcher: &dyn Fetcher, url: &str) -> Result<Vec<Member>, FetchError> {
    parse_directory(&read_directory_bytes(fetcher, url)?).map_err(|message| refused(url, message))
}

/// The central directory of the archive at `url` (the end record lies in its last 64 KiB).
fn read_directory_bytes(fetcher: &dyn Fetcher, url: &str) -> Result<Vec<u8>, FetchError> {
    let tail = fetcher.get_range(url, ByteRange::Last(65_558))?;
    let (start, len) = directory_location(&tail).map_err(|message| refused(url, message))?;
    fetcher.get_range(url, ByteRange::From { start, len })
}

fn refused(url: &str, message: impl Into<String>) -> FetchError {
    FetchError::Http {
        url: url.to_string(),
        message: message.into(),
    }
}

/// The uncompressed content of `member`, read by two byte ranges.
pub fn read_member(
    fetcher: &dyn Fetcher,
    url: &str,
    member: &Member,
) -> Result<Vec<u8>, FetchError> {
    let header = fetcher.get_range(
        url,
        ByteRange::From {
            start: member.local_header,
            len: 30,
        },
    )?;
    if header.len() < 30 || u32_at(&header, 0) != 0x0403_4b50 {
        return Err(refused(url, format!("{}: no local header", member.name)));
    }
    let data_start =
        member.local_header + 30 + u64::from(u16_at(&header, 26)) + u64::from(u16_at(&header, 28));
    let data = fetcher.get_range(
        url,
        ByteRange::From {
            start: data_start,
            len: member.compressed,
        },
    )?;
    match member.method {
        0 => Ok(data),
        8 => miniz_oxide::inflate::decompress_to_vec(&data)
            .map_err(|e| refused(url, format!("{}: {e:?}", member.name))),
        method => Err(refused(
            url,
            format!("{}: unsupported compression {method}", member.name),
        )),
    }
}

/// The 1 km tiles of `model` over `bbox` (EPSG:25832), read out of the statewide archive and
/// cached, mosaicked into a Float32 GeoTIFF `out` with its world file. Tiles outside Hamburg
/// are missing from the archive and stay empty.
pub fn fetch_model(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    mode: FetchMode,
    model: Model,
    bbox: &BoundingBox,
    out: &Path,
) -> Result<(), FetchError> {
    let grid = GridSpec::covering(bbox, 1.0);
    let mut raster = Raster::filled(grid, f32::NAN);
    let dir = cache.lgv_dir().join(model.cache_name());
    let directory_file = dir.join("directory.bin");
    let url = model.archive_url();
    let directory = match mode {
        FetchMode::Online => {
            let bytes = read_directory_bytes(fetcher, url)?;
            write_atomically(&directory_file, &bytes)?;
            bytes
        }
        FetchMode::Offline => std::fs::read(&directory_file).map_err(|_| {
            refused(
                url,
                format!(
                    "offline and not cached: {}; run `freemapper fetch` online",
                    model.label()
                ),
            )
        })?,
    };
    let members = parse_directory(&directory).map_err(|message| refused(url, message))?;
    for tile in tiles_covering(bbox) {
        let prefix = format!(
            "{}_32_{:03}_{:04}_",
            model.file_prefix(),
            tile.east_km,
            tile.north_km
        );
        let Some(member) = members.iter().find(|m| {
            let file = m.name.rsplit('/').next().unwrap_or_default();
            file.starts_with(&prefix) && file.ends_with(".tif")
        }) else {
            // Outside Hamburg: no tile.
            continue;
        };
        let file = dir.join(member.name.rsplit('/').next().unwrap_or_default());
        let bytes = match std::fs::read(&file) {
            Ok(bytes) => bytes,
            Err(_) if mode == FetchMode::Online => {
                let bytes = read_member(fetcher, url, member)?;
                write_atomically(&file, &bytes)?;
                bytes
            }
            Err(_) => {
                return Err(refused(
                    url,
                    format!(
                        "offline and not cached: {}; run `freemapper fetch` online",
                        member.name
                    ),
                ));
            }
        };
        let part = decode_float_tiff(&bytes).map_err(|message| refused(url, message))?;
        raster
            .paste_with(&part.grid, |x, y| part.get(x, y))
            .map_err(|e| refused(url, e.to_string()))?;
    }
    write_float_raster(&raster, out, url)
}

/// The ALKIS-vereinfacht OGC API collections (LGB-like JSON, `next` links, EPSG:25832).
pub const BUILDINGS_ENDPOINT: &str =
    "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/GebaeudeBauwerk/items";
pub const LAND_USE_ENDPOINT: &str =
    "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/NutzungFlurstueck/items";

/// The LGB spellings of `gebnutzbez` the build rules know.
const BUILDING_KINDS: [&str; 2] = ["Gebäude", "Bauteil"];
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
    let value = &mut feature["properties"][property];
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
fn spelling_key(text: &str) -> String {
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

    /// A zip like LGV's: deflated GeoTIFF members in folders, and a stored CSV.
    fn archive(members: &[(&str, Vec<u8>, bool)]) -> Vec<u8> {
        use std::io::Write as _;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes, deflate) in members {
            let method = if *deflate {
                zip::CompressionMethod::Deflated
            } else {
                zip::CompressionMethod::Stored
            };
            zip.start_file(
                *name,
                zip::write::SimpleFileOptions::default().compression_method(method),
            )
            .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

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
    fn central_directory_lists_members() {
        let zip = archive(&[
            (
                "dgm1_hh_2022/s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            ),
            (
                "dgm1_hh_2023-02-09.csv",
                b"Kachelname;Aktualitaet".to_vec(),
                false,
            ),
        ]);
        let (offset, size) = directory_location(&zip[zip.len().saturating_sub(70_000)..]).unwrap();
        let members = parse_directory(&zip[offset as usize..(offset + size) as usize]).unwrap();
        let names: Vec<&str> = members.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "dgm1_hh_2022/s32_565/dgm1_32_565_5937_1_hh_2022.tif",
                "dgm1_hh_2023-02-09.csv"
            ]
        );
    }

    #[test]
    fn member_is_inflated() {
        let csv = b"Kachelname;Aktualitaet\n".repeat(50);
        let zip = archive(&[
            (
                "a/dgm1_32_565_5937_1_hh_2022.tif",
                tile(565, 5937, 6.0),
                true,
            ),
            ("b.csv", csv.clone(), false),
        ]);
        let fetcher = MapFetcher::default();
        fetcher.insert("https://example.org/dgm.zip", zip);
        let members = read_directory(&fetcher, "https://example.org/dgm.zip").unwrap();
        let csv_member = members.iter().find(|m| m.name == "b.csv").unwrap();
        assert_eq!(
            read_member(&fetcher, "https://example.org/dgm.zip", csv_member).unwrap(),
            csv
        );
        let tif = members.iter().find(|m| m.name.ends_with(".tif")).unwrap();
        let bytes = read_member(&fetcher, "https://example.org/dgm.zip", tif).unwrap();
        assert!(bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*"));
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
            &out,
        )
        .unwrap();
        assert_eq!(offline.calls.get(), 0);
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
            &out,
        )
        .unwrap();
        let raster = crate::derive::terrain::read_float_raster(&out).unwrap();
        assert!(raster.get(0, 0).is_nan() && raster.get(1, 0).is_nan());
        assert_eq!(raster.get(2, 0), 6.0);
    }
}
