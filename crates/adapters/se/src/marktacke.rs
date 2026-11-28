// SPDX-License-Identifier: AGPL-3.0-or-later

//! The land cover of Lantmäteriet (Marktäcke): one GeoPackage per municipality in a zip file,
//! found through the STAC API of the vector products. The classes of the table `mark` become the
//! land use classes (`nutzart`) the map already draws; it has no buildings.

use std::collections::{BTreeMap, HashSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};

use crate::account;
use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::{AdapterError, FetchError};
use freemapper_adapter_api::fetch::FetchMode;
use freemapper_adapter_api::note::Arg;
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_sources::gpkg::{GeoPackage, Shape, Value, geometry};
use freemapper_sources::stac;
use serde_json::{Value as Json, json};

use crate::stac_terrain::{PROVIDER, degrees};

/// The table of the land cover and the columns the source reads.
const TABLE: &str = "mark";

/// The tables with areas: the land cover, and the wetland that lies on top of it. A file without
/// the second one is read all the same.
const TABLES: [&str; 2] = [TABLE, "sankmark"];

/// The EPSG code the GeoPackage states for its geometries.
const SRS: i32 = 3006;

/// The product a person orders for the download, named when the server refuses it.
const ORDER: &str = "Marktäcke";

/// The largest zip file taken, and the largest GeoPackage read out of it, in bytes: a big
/// municipality is a few hundred MB, and the file is read whole.
const MAX_ZIP_BYTES: u64 = 600 << 20;
const MAX_GPKG_BYTES: u64 = 1 << 30;

/// The classes of the land cover and the land use class each stands for.
const RULES: &str = include_str!("../rules/marktacke.toml");

/// The classes that are left out on purpose and not named: they say nothing a land use class can.
fn ignored() -> HashSet<String> {
    let table: toml::Table = RULES.parse().expect("the rules are TOML");
    table["ignored"]
        .as_array()
        .expect("the rules have a list `ignored`")
        .iter()
        .map(|name| name.as_str().expect("a class is text").to_string())
        .collect()
}

/// `objekttyp` to `nutzart`, from [`RULES`].
fn classes() -> BTreeMap<String, String> {
    let table: toml::Table = RULES.parse().expect("the rules are TOML");
    table["classes"]
        .as_table()
        .expect("the rules have a table `classes`")
        .iter()
        .map(|(name, nutzart)| {
            (
                name.clone(),
                nutzart.as_str().expect("a class is text").to_string(),
            )
        })
        .collect()
}

/// The land cover of Lantmäteriet as the land use of a map.
pub struct MarktackeSource {
    /// The STAC API of the vector products, from the manifest (`[alkis] stac`).
    stac_vektor: String,
    /// The collection of the land cover (`[alkis] collection`).
    collection: String,
}

impl MarktackeSource {
    pub fn new(stac_vektor: &str, collection: &str) -> Self {
        Self {
            stac_vektor: stac_vektor.to_string(),
            collection: collection.to_string(),
        }
    }
}

/// Whether `id` is fit to be a file name: the catalogue is not trusted with a path.
pub(crate) fn is_name(id: &str) -> bool {
    (1..=80).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}

/// The zip files to read: those of the municipalities the catalogue lists under the area, downloaded
/// when they are not cached; offline, every zip file the cache holds.
fn zips(
    env: &mut AdapterEnv<'_>,
    source: &MarktackeSource,
    area: &FetchArea,
    dir: &Path,
) -> Result<Vec<PathBuf>, AdapterError> {
    if env.mode == FetchMode::Offline {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.extension().is_some_and(|e| e == "zip"))
                    .collect()
            })
            .unwrap_or_default();
        found.sort();
        return Ok(found);
    }
    let wgs84 = degrees(&area.region_bbox).ok_or_else(|| FetchError::Http {
        url: source.stac_vektor.clone(),
        message: "the map area lies outside the coordinate system of Sweden".to_string(),
    })?;
    let items = stac::search(
        env.fetcher,
        &source.stac_vektor,
        &source.collection,
        wgs84,
        100,
    )?;
    let mut found = Vec::new();
    for item in items {
        let Some(asset) = item.assets.values().find(|a| a.href.ends_with(".zip")) else {
            continue;
        };
        if !is_name(&item.id) {
            continue;
        }
        // The file keeps the name the catalogue gives it (`marktacke_kn0183.zip`) when that is a
        // plain name, so a message about it means something to a person.
        let given = asset.href.rsplit('/').next().unwrap_or_default();
        let plain = given.strip_suffix(".zip").is_some_and(is_name);
        let path = dir.join(if plain {
            given.to_string()
        } else {
            format!("{}.zip", item.id)
        });
        if !path.exists() {
            (env.progress)(freemapper_adapter_api::note!(
                "progress-downloading",
                url = &asset.href
            ));
            std::fs::create_dir_all(dir).map_err(|e| FetchError::Io(dir.to_path_buf(), e))?;
            let part = path.with_extension("part");
            let result = env
                .fetcher
                .download_to(&asset.href, &part, MAX_ZIP_BYTES, env.progress)
                .and_then(|()| {
                    std::fs::rename(&part, &path).map_err(|e| FetchError::Io(path.clone(), e))
                });
            if let Err(error) = result {
                let _ = std::fs::remove_file(&part);
                return Err(account::refusal(&asset.href, ORDER, error).into());
            }
        }
        found.push(path);
    }
    Ok(found)
}

/// The GeoPackage inside the zip file at `zip`.
fn geopackage_of(zip: &Path) -> Result<Vec<u8>, FetchError> {
    let name = zip
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: name.clone(),
        message,
    };
    let file = std::fs::File::open(zip).map_err(|e| FetchError::Io(zip.to_path_buf(), e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| corrupt(e.to_string()))?;
    for i in 0..archive.len() {
        let member = archive.by_index(i).map_err(|e| corrupt(e.to_string()))?;
        if member
            .name()
            .is_ok_and(|name| name.to_ascii_lowercase().ends_with(".gpkg"))
        {
            let mut bytes = Vec::new();
            member
                .take(MAX_GPKG_BYTES)
                .read_to_end(&mut bytes)
                .map_err(|e| corrupt(e.to_string()))?;
            return Ok(bytes);
        }
    }
    Err(corrupt("the zip file holds no GeoPackage".to_string()))
}

/// The box around the rings of a polygon list.
fn extent(polygons: &[Vec<Vec<[f64; 2]>>]) -> Option<BoundingBox> {
    let mut points = polygons.iter().flatten().flatten();
    let first = points.next()?;
    let mut b = BoundingBox {
        min_e: first[0],
        min_n: first[1],
        max_e: first[0],
        max_n: first[1],
    };
    for [x, y] in points {
        b.min_e = b.min_e.min(*x);
        b.min_n = b.min_n.min(*y);
        b.max_e = b.max_e.max(*x);
        b.max_n = b.max_n.max(*y);
    }
    Some(b)
}

/// The land use features of the GeoPackage `bytes` that touch `clip`.
fn read(
    bytes: &[u8],
    zip_name: &str,
    clip: &BoundingBox,
    classes: &BTreeMap<String, String>,
    seen: &mut HashSet<String>,
    unknown: &mut HashSet<String>,
    out: &mut Vec<Json>,
) -> Result<(), FetchError> {
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: zip_name.to_string(),
        message,
    };
    let package = GeoPackage::open(bytes).map_err(corrupt)?;
    let tables = package.table_names().map_err(corrupt)?;
    for table in TABLES {
        // The wetland table may be missing; the land cover table may not.
        if table != TABLE && !tables.iter().any(|t| t == table) {
            continue;
        }
        read_table(&package, table, clip, classes, seen, unknown, out).map_err(corrupt)?;
    }
    Ok(())
}

/// The areas of the table `table` of `package` that touch `clip` and have a land use class.
fn read_table(
    package: &GeoPackage<'_>,
    table: &str,
    clip: &BoundingBox,
    classes: &BTreeMap<String, String>,
    seen: &mut HashSet<String>,
    unknown: &mut HashSet<String>,
    out: &mut Vec<Json>,
) -> Result<(), String> {
    let ignored = ignored();
    let columns = package.columns(table)?;
    let at = |name: &str| columns.iter().position(|c| c == name);
    let (Some(id), Some(class), Some(geom)) = (at("objektidentitet"), at("objekttyp"), at("geom"))
    else {
        return Err(format!("the table `{table}` lacks a column it needs"));
    };
    package.rows(table, &mut |_, row| {
        let (Value::Text(oid), Value::Text(class), Value::Blob(blob)) =
            (&row[id], &row[class], &row[geom])
        else {
            return Ok(());
        };
        let Some(nutzart) = classes.get(class) else {
            if !ignored.contains(class) {
                unknown.insert(class.clone());
            }
            return Ok(());
        };
        let Some(found) = geometry(blob)? else {
            return Ok(());
        };
        let Shape::Polygons(polygons) = found.shape else {
            return Ok(());
        };
        if found.srs_id != SRS || seen.contains(oid) {
            return Ok(());
        }
        let touches = extent(&polygons).is_some_and(|b| {
            b.min_e <= clip.max_e
                && b.max_e >= clip.min_e
                && b.min_n <= clip.max_n
                && b.max_n >= clip.min_n
        });
        if !touches {
            return Ok(());
        }
        seen.insert(oid.clone());
        let geometry = match polygons.as_slice() {
            [one] => json!({ "type": "Polygon", "coordinates": one }),
            many => json!({ "type": "MultiPolygon", "coordinates": many }),
        };
        out.push(json!({
            "type": "Feature",
            "id": oid,
            "geometry": geometry,
            "properties": { "oid": oid, "nutzart": nutzart },
        }));
        Ok(())
    })
}

impl AlkisSource for MarktackeSource {
    fn buildings(
        &self,
        _env: &mut AdapterEnv<'_>,
        _area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        Ok(AlkisPart {
            features: Vec::new(),
            crs: Crs::Utm33,
        })
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let dir = env.cache.provider_dir(PROVIDER).join("marktacke");
        let classes = classes();
        let clip = area.bbox_for(Crs::Utm33);
        let (mut seen, mut unknown, mut features) = (HashSet::new(), HashSet::new(), Vec::new());
        for zip in zips(env, self, area, &dir)? {
            let name = zip
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let bytes = geopackage_of(&zip)?;
            read(
                &bytes,
                &name,
                &clip,
                &classes,
                &mut seen,
                &mut unknown,
                &mut features,
            )?;
        }
        let mut unknown: Vec<String> = unknown.into_iter().collect();
        unknown.sort();
        for class in unknown {
            (env.progress)(freemapper_adapter_api::note!(
                "warning-error",
                error = Arg::error(std::io::Error::other(format!(
                    "the land cover class `{class}` has no land use class and is left out of the map"
                )))
            ));
        }
        Ok(AlkisPart {
            features,
            crs: Crs::Utm33,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use chrono::NaiveDate;
    use freemapper_adapter_api::alkis::{AlkisSource, FetchArea};
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::env::AdapterEnv;
    use freemapper_adapter_api::error::AdapterError;
    use freemapper_adapter_api::error::FetchError;
    use freemapper_adapter_api::fetch::{ByteRange, FetchMode, Fetcher};
    use freemapper_adapter_api::registry::Registry;
    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_georef::crs::{BoundingBox, Crs};
    use freemapper_sources::alkis_source::LAND_USE_CLASSES;

    use super::*;

    const STAC: &str = "https://stac.example.org/stac-vektor/v1";
    const ZIP_0183: &str = "https://dl.example.org/mark/marktacke/marktacke_kn0183.zip";
    const ZIP_0184: &str = "https://dl.example.org/mark/marktacke/marktacke_kn0184.zip";

    fn gpkg() -> Vec<u8> {
        std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/marktacke.gpkg"
        ))
        .unwrap()
    }

    fn zipped(name: &str, bytes: &[u8]) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(&mut out);
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
        zip.finish().unwrap();
        out.into_inner()
    }

    fn answer(items: &[(&str, &str)]) -> Vec<u8> {
        let features: Vec<String> = items
            .iter()
            .map(|(id, href)| {
                format!(
                    r#"{{"id":"{id}","collection":"marktacke","properties":{{}},
                    "assets":{{"data":{{"href":"{href}"}}}}}}"#
                )
            })
            .collect();
        format!(
            r#"{{"type":"FeatureCollection","features":[{}],"links":[]}}"#,
            features.join(",")
        )
        .into_bytes()
    }

    fn served(items: &[(&str, &str)]) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &format!("{STAC}/search?collections=marktacke"),
            answer(items),
        );
        fetcher.insert(ZIP_0183, zipped("marktacke_kn0183.gpkg", &gpkg()));
        fetcher.insert(ZIP_0184, zipped("marktacke_kn0184.gpkg", &gpkg()));
        fetcher
    }

    /// The area 385 000 to 385 200 by 6 177 500 to 6 177 700.
    fn area() -> FetchArea {
        FetchArea::new(
            BoundingBox {
                min_e: 385_000.0,
                min_n: 6_177_500.0,
                max_e: 385_200.0,
                max_n: 6_177_700.0,
            },
            Crs::Utm33,
        )
    }

    fn source() -> MarktackeSource {
        MarktackeSource::new(STAC, "marktacke")
    }

    fn land_use(
        fetcher: &dyn Fetcher,
        cache: &CacheDir,
        mode: FetchMode,
    ) -> (
        Result<freemapper_adapter_api::alkis::AlkisPart, AdapterError>,
        Vec<String>,
    ) {
        let regions = Registry::new();
        let mut lines = Vec::new();
        let result = {
            let mut progress = |note: freemapper_adapter_api::note::Note| {
                lines.push(note.id().to_string());
                for class in ["Okänd typ", "Öppen mark"] {
                    if note.contains(class) {
                        lines.push(format!("names {class}"));
                    }
                }
            };
            let mut env = AdapterEnv {
                fetcher,
                regions: &regions,
                cache,
                mode,
                today: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
                skip_surface: false,
                progress: &mut progress,
            };
            source().land_use(&mut env, &area())
        };
        (result, lines)
    }

    fn by_oid<'a>(
        part: &'a freemapper_adapter_api::alkis::AlkisPart,
        oid: &str,
    ) -> Option<&'a serde_json::Value> {
        part.features
            .iter()
            .find(|feature| feature["properties"]["oid"] == oid)
    }

    #[test]
    fn every_class_of_the_rules_is_a_land_use_class_of_the_map() {
        let classes = classes();
        assert!(classes.len() >= 7);
        for (name, nutzart) in &classes {
            assert!(
                LAND_USE_CLASSES.contains(&nutzart.as_str()),
                "{name}: {nutzart}"
            );
        }
    }

    #[test]
    fn the_known_classes_become_land_use_features_in_the_lgb_schema() {
        let fetcher = served(&[("0183", ZIP_0183)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let (result, _) = land_use(&fetcher, &cache, FetchMode::Online);
        let part = result.unwrap();
        assert_eq!(part.crs, Crs::Utm33);
        let forest = by_oid(&part, "id-forest").expect("the forest");
        assert_eq!(forest["properties"]["nutzart"], "Wald");
        assert_eq!(forest["geometry"]["type"], "Polygon");
        assert_eq!(
            forest["geometry"]["coordinates"][0][0],
            serde_json::json!([385_050.0, 6_177_550.0])
        );
        assert_eq!(
            by_oid(&part, "id-low-built").unwrap()["properties"]["nutzart"],
            "Wohnbaufläche"
        );
        assert_eq!(
            by_oid(&part, "id-square-across").unwrap()["properties"]["nutzart"],
            "Platz"
        );
    }

    #[test]
    fn a_polygon_far_from_the_area_is_dropped_and_one_across_its_edge_is_kept_whole() {
        let fetcher = served(&[("0183", ZIP_0183)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let part = land_use(&fetcher, &cache, FetchMode::Online).0.unwrap();
        assert!(by_oid(&part, "id-lake-far").is_none());
        let square = by_oid(&part, "id-square-across").unwrap();
        assert_eq!(
            square["geometry"]["coordinates"][0][2],
            serde_json::json!([385_300.0, 6_177_750.0])
        );
    }

    /// The ids of the features of the fixture whose polygon touches `clip`.
    fn ids_touching(clip: BoundingBox) -> Vec<String> {
        let mut out = Vec::new();
        read(
            &gpkg(),
            "x.zip",
            &clip,
            &classes(),
            &mut HashSet::new(),
            &mut HashSet::new(),
            &mut out,
        )
        .unwrap();
        out.iter()
            .map(|f| f["id"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn a_polygon_is_kept_when_the_clip_touches_it_in_both_directions() {
        // The forest polygon lies at 385 050 to 385 100 by 6 177 550 to 6 177 600.
        let at = |min_e: f64, max_e: f64, min_n: f64, max_n: f64| {
            ids_touching(BoundingBox {
                min_e,
                min_n,
                max_e,
                max_n,
            })
            .contains(&"id-forest".to_string())
        };
        assert!(at(385_000.0, 385_200.0, 6_177_500.0, 6_177_700.0));
        // Beyond one side, with the other direction overlapping: no.
        assert!(!at(385_000.0, 385_040.0, 6_177_500.0, 6_177_700.0), "west");
        assert!(!at(385_110.0, 385_200.0, 6_177_500.0, 6_177_700.0), "east");
        assert!(!at(385_000.0, 385_200.0, 6_177_500.0, 6_177_540.0), "south");
        assert!(!at(385_000.0, 385_200.0, 6_177_610.0, 6_177_700.0), "north");
        // Touching along an edge counts.
        assert!(at(385_100.0, 385_200.0, 6_177_500.0, 6_177_700.0));
        assert!(at(385_000.0, 385_050.0, 6_177_500.0, 6_177_700.0));
        assert!(at(385_000.0, 385_200.0, 6_177_600.0, 6_177_700.0));
        assert!(at(385_000.0, 385_200.0, 6_177_500.0, 6_177_550.0));
    }

    #[test]
    fn every_class_of_the_real_files_is_mapped_or_left_out_on_purpose() {
        // The `objekttyp` values of `mark` and `sankmark` in the files of three municipalities.
        let seen = [
            "Öppen mark",
            "Barr- och blandskog",
            "Åker",
            "Sjö",
            "Lövskog",
            "Låg bebyggelse",
            "Hög bebyggelse",
            "Industri- och handelsbebyggelse",
            "Hav",
            "Vattendragsyta",
            "Torg",
            "Sluten bebyggelse",
            "Anlagt vatten",
            "Ej karterat område",
            "Fruktodling",
            "Sankmark, fast",
            "Sankmark, våt",
        ];
        let (classes, ignored) = (classes(), ignored());
        for class in seen {
            assert!(
                classes.contains_key(class) || ignored.contains(class),
                "{class} is neither mapped nor ignored"
            );
        }
        assert_eq!(classes["Sankmark, våt"], "Sumpf");
        assert_eq!(classes["Anlagt vatten"], "Stehendes Gewässer");
        assert_eq!(classes["Sluten bebyggelse"], "Fläche gemischter Nutzung");
        assert!(ignored.contains("Ej karterat område"));
    }

    #[test]
    fn the_wetland_table_becomes_marsh_land_use() {
        let fetcher = served(&[("0183", ZIP_0183)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let part = land_use(&fetcher, &cache, FetchMode::Online).0.unwrap();
        for id in ["id-wet", "id-firm"] {
            let feature = by_oid(&part, id).unwrap_or_else(|| panic!("{id} is missing"));
            assert_eq!(feature["properties"]["nutzart"], "Sumpf", "{id}");
        }
    }

    #[test]
    fn only_a_plain_name_is_fit_to_be_a_file_name() {
        assert!(is_name("marktacke_kn0183"));
        assert!(is_name(&"a".repeat(80)));
        assert!(!is_name(""));
        assert!(!is_name(&"a".repeat(81)));
        assert!(!is_name("../etc"));
        assert!(!is_name("a b"));
        // A path character alone fails the name, however long the name is.
        assert!(!is_name("name/with-slash"));
    }

    #[test]
    fn a_class_without_a_land_use_counterpart_is_left_out_and_named_once() {
        let fetcher = served(&[("0183", ZIP_0183)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let (result, lines) = land_use(&fetcher, &cache, FetchMode::Online);
        let part = result.unwrap();
        assert!(by_oid(&part, "id-open").is_none());
        assert!(by_oid(&part, "id-unknown-1").is_none() && by_oid(&part, "id-unknown-2").is_none());
        // `Öppen mark` is left out on purpose and says nothing; `Okänd typ` is named once,
        // though it has two polygons.
        assert_eq!(
            lines.iter().filter(|id| *id == "warning-error").count(),
            1,
            "{lines:?}"
        );
        assert!(lines.contains(&"names Okänd typ".to_string()), "{lines:?}");
        assert!(
            !lines.contains(&"names Öppen mark".to_string()),
            "{lines:?}"
        );
    }

    #[test]
    fn two_municipalities_are_joined_without_a_polygon_twice() {
        let fetcher = served(&[("0183", ZIP_0183), ("0184", ZIP_0184)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let part = land_use(&fetcher, &cache, FetchMode::Online).0.unwrap();
        let forests = part
            .features
            .iter()
            .filter(|f| f["properties"]["oid"] == "id-forest")
            .count();
        assert_eq!(forests, 1);
    }

    #[test]
    fn the_zip_is_downloaded_once_and_then_read_from_the_cache() {
        let fetcher = served(&[("0183", ZIP_0183)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        land_use(&fetcher, &cache, FetchMode::Online).0.unwrap();
        land_use(&fetcher, &cache, FetchMode::Online).0.unwrap();
        let downloads = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.ends_with(".zip"))
            .count();
        assert_eq!(downloads, 1);
        let offline = land_use(&MapFetcher::default(), &cache, FetchMode::Offline)
            .0
            .unwrap();
        assert!(by_oid(&offline, "id-forest").is_some());
    }

    #[test]
    fn the_buildings_are_none() {
        let fetcher = served(&[("0183", ZIP_0183)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let regions = Registry::new();
        let mut progress = |_| {};
        let mut env = AdapterEnv {
            fetcher: &fetcher,
            regions: &regions,
            cache: &cache,
            mode: FetchMode::Online,
            today: NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
            skip_surface: false,
            progress: &mut progress,
        };
        let part = source().buildings(&mut env, &area()).unwrap();
        assert!(part.features.is_empty());
        assert_eq!(part.crs, Crs::Utm33);
    }

    /// Answers 401 for the zip files, as the download host does for an account without the order.
    struct Unauthorised<'a>(&'a MapFetcher);

    impl Fetcher for Unauthorised<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            if url.ends_with(".zip") {
                return Err(freemapper_adapter_api::fetch::status_answer_error(
                    url, 401, b"",
                ));
            }
            self.0.get(url)
        }

        fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
            self.0.get_range(url, range)
        }

        fn download_to(
            &self,
            url: &str,
            path: &std::path::Path,
            max_bytes: u64,
            progress: &mut dyn FnMut(freemapper_adapter_api::note::Note),
        ) -> Result<(), FetchError> {
            if url.ends_with(".zip") {
                return Err(freemapper_adapter_api::fetch::status_answer_error(
                    url, 401, b"",
                ));
            }
            self.0.download_to(url, path, max_bytes, progress)
        }
    }

    #[test]
    fn a_refused_download_names_the_order() {
        let inner = served(&[("0183", ZIP_0183)]);
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let message = land_use(&Unauthorised(&inner), &cache, FetchMode::Online)
            .0
            .unwrap_err()
            .to_string();
        assert!(message.contains("Marktäcke"), "{message}");
    }

    #[test]
    fn a_zip_without_a_geopackage_is_an_error_that_names_the_file() {
        let fetcher = served(&[("0183", ZIP_0183)]);
        fetcher.insert(ZIP_0183, zipped("readme.txt", b"no data"));
        let scratch = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(scratch.path().join("cache"));
        let message = land_use(&fetcher, &cache, FetchMode::Online)
            .0
            .unwrap_err()
            .to_string();
        assert!(message.contains("marktacke_kn0183.zip"), "{message}");
    }
}
