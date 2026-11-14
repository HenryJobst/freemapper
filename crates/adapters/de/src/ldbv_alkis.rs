//! The cadastre of Bavaria. The Bayerische Vermessungsverwaltung has no service for it, only
//! files: the buildings come from the LoD2 tiles (CityGML, one file per 2 km), the land use
//! from the Shapefile of each Kreis. [`LdbvAlkis`] downloads the files the map area touches,
//! cuts the area out and hands on the simplified schema.

use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

use freemapper_adapter_api::alkis::{AlkisPart, AlkisSource, FetchArea};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::geometry::{point_box, rectangle};
use freemapper_sources::aaa::{STRUCTURE, building_function, roof_function};
use freemapper_sources::citygml::{CityBuilding, read_buildings};
use freemapper_sources::shapefile::{Polygon, ShapeRecord, polygons_in};
use geo::{BoundingRect, Intersects};
use geo_types::MultiPolygon;
use serde_json::{Map, Value, json};

use crate::crs::{BoundingBox, Crs, ProjectedPoint};
use crate::fetch::{
    FetchError, FetchMode, Fetcher, download_with_retries, get_limited_with_retries,
    write_cache_file,
};
use crate::ldbv::FILES;
use crate::ldbv_files::{Current, Published, cached, current, drop_older, missing_note};
use crate::note::{Note, note};
use crate::sources::region::{BAYERN, touches};

/// The largest LoD2 tile that is read. The centre of Munich has 162 MB.
pub const LOD2_MAX_BYTES: u64 = 512 * 1024 * 1024;
/// The largest land use zip of a Kreis that is read. The city of Munich has 12.6 MB.
pub const TN_MAX_BYTES: u64 = 64 * 1024 * 1024;
/// The largest member of a land use zip that is unpacked.
const TN_MEMBER_MAX_BYTES: u64 = 1024 * 1024 * 1024;
/// How near (m) the outline of a Kreis must come to the map area for its file to be read. The
/// outlines of the embedded file lie at most 181 m from the true ones.
pub const KREIS_MARGIN_M: f64 = 500.0;
/// `bez` given to a square without one: the cadastre does not record what the square is used
/// for, so it stays a candidate instead of becoming a paved map object.
pub const SQUARE_BEZ: &str = "ohne Unterart (Bayern)";

/// The cache folder of the provider, and of the two kinds of files in it.
const CACHE_DIR: &str = "ldbv";
const LOD2_DIR: &str = "lod2";
const TN_DIR: &str = "tn";

/// The object type of a building in `bldg:function` (`AX_Gebaeude`), and of the structures
/// among which the roofs are (`AX_SonstigesBauwerkOderSonstigeEinrichtung`).
const BUILDING_TYPE: &str = "31001";
const STRUCTURE_TYPE: &str = "51009";
/// What the oid of a building ends in, as the simplified services write it.
const BUILDING_SUFFIX: &str = "BL";
/// The class of the squares.
const SQUARE: &str = "Platz";

/// The outlines of the Kreise (`assets/kreise/bayern.geojson`, EPSG:25833) with their keys.
pub struct KreisIndex {
    /// Key, outline, the box around it.
    kreise: Vec<(String, MultiPolygon<f64>, BoundingBox)>,
}

impl KreisIndex {
    /// The index of a GeoJSON FeatureCollection whose features carry the key as `ags`.
    pub fn parse(geojson: &str) -> Result<Self, String> {
        let collection: geojson::FeatureCollection =
            geojson.parse().map_err(|e| format!("Kreise: {e}"))?;
        let mut kreise = Vec::with_capacity(collection.features.len());
        for feature in collection.features {
            let ags = feature
                .property("ags")
                .and_then(Value::as_str)
                .ok_or("Kreise: a feature without the key `ags`")?
                .to_string();
            let geometry = feature
                .geometry
                .ok_or_else(|| format!("Kreise: {ags} has no geometry"))?;
            let outline: MultiPolygon<f64> = match geo_types::Geometry::<f64>::try_from(geometry) {
                Ok(geo_types::Geometry::MultiPolygon(outline)) => outline,
                Ok(geo_types::Geometry::Polygon(outline)) => MultiPolygon(vec![outline]),
                _ => return Err(format!("Kreise: {ags} is no polygon")),
            };
            let rect = outline
                .bounding_rect()
                .ok_or_else(|| format!("Kreise: {ags} is empty"))?;
            let bounds = BoundingBox {
                min_e: rect.min().x,
                min_n: rect.min().y,
                max_e: rect.max().x,
                max_n: rect.max().y,
            };
            kreise.push((ags, outline, bounds));
        }
        if kreise.is_empty() {
            return Err("Kreise: no feature".to_string());
        }
        Ok(Self { kreise })
    }

    /// The keys of the Kreise within [`KREIS_MARGIN_M`] of `region_bbox` (EPSG:25833), sorted.
    pub fn near(&self, region_bbox: &BoundingBox) -> Vec<String> {
        let wide = region_bbox.buffered(KREIS_MARGIN_M);
        let area = rectangle(&wide);
        let mut keys: Vec<String> = self
            .kreise
            .iter()
            .filter(|(_, outline, bounds)| {
                bounds.min_e <= wide.max_e
                    && wide.min_e <= bounds.max_e
                    && bounds.min_n <= wide.max_n
                    && wide.min_n <= bounds.max_n
                    && outline.intersects(&area)
            })
            .map(|(ags, _, _)| ags.clone())
            .collect();
        keys.sort();
        keys
    }
}

/// The 96 Kreise of Bavaria, from `assets/kreise/bayern.geojson`, parsed once.
pub fn built_in_kreise() -> &'static KreisIndex {
    static KREISE: OnceLock<KreisIndex> = OnceLock::new();
    KREISE.get_or_init(|| {
        KreisIndex::parse(include_str!("../../../../assets/kreise/bayern.geojson"))
            .expect("the embedded Kreise are valid")
    })
}

/// The 2 km LoD2 tiles (south-west corner, even km, EPSG:25832) a box touches.
pub fn lod2_tiles(bbox: &BoundingBox) -> Vec<(u32, u32)> {
    let even = |metres: f64| ((metres / 2000.0).floor().max(0.0) as u32) * 2;
    let mut tiles = Vec::new();
    for east in (even(bbox.min_e)..=even(bbox.max_e)).step_by(2) {
        for north in (even(bbox.min_n)..=even(bbox.max_n)).step_by(2) {
            tiles.push((east, north));
        }
    }
    tiles
}

/// `690_5334`.
fn lod2_stem(tile: (u32, u32)) -> String {
    format!("{}_{}", tile.0, tile.1)
}

/// Where the agency publishes the LoD2 tile.
pub fn lod2_url(tile: (u32, u32)) -> String {
    format!("{FILES}/lod2/citygml/{}.gml", lod2_stem(tile))
}

/// Where the agency publishes the land use of the Kreis with the key `ags`.
pub fn tn_url(ags: &str) -> String {
    format!("{FILES}/tn/lkr/tn_{ags}.zip")
}

/// Whether the buildings of the LoD2 tile need no download: its features are cached, or the
/// server did not have the tile.
pub fn lod2_is_cached(cache: &crate::fetch::CacheDir, tile: (u32, u32)) -> bool {
    let (dir, stem) = (
        cache.provider_dir(CACHE_DIR).join(LOD2_DIR),
        lod2_stem(tile),
    );
    cached(&dir, &stem, "geojson").is_some() || missing_note(&dir, &stem).exists()
}

/// Whether the land use of the Kreis with the key `ags` is cached.
pub fn tn_is_cached(cache: &crate::fetch::CacheDir, ags: &str) -> bool {
    let dir = cache.provider_dir(CACHE_DIR).join(TN_DIR);
    cached(&dir, &format!("tn_{ags}"), "zip").is_some()
}

/// Whether the centre of the LoD2 tile lies in or near Bavaria, so the server must have it.
fn in_bavaria(tile: (u32, u32)) -> bool {
    let centre = ProjectedPoint {
        easting: f64::from(tile.0 + 1) * 1000.0,
        northing: f64::from(tile.1 + 1) * 1000.0,
    };
    Crs::Utm33
        .reproject(Crs::Utm32, centre)
        .is_some_and(|p| touches(BAYERN, &point_box(p.easting, p.northing)))
}

/// The GeoJSON geometry of polygons: a `Polygon` for one, a `MultiPolygon` for several.
fn geometry(polygons: &[Polygon]) -> Value {
    match polygons {
        [one] => json!({ "type": "Polygon", "coordinates": one }),
        several => json!({ "type": "MultiPolygon", "coordinates": several }),
    }
}

/// A feature of the simplified schema: `properties` that have a value, with the `oid` also as
/// its id.
fn feature(oid: &str, polygons: &[Polygon], properties: &[(&str, Option<&str>)]) -> Value {
    let mut schema = Map::new();
    schema.insert("oid".to_string(), Value::from(oid));
    for (name, value) in properties {
        if let Some(value) = value {
            schema.insert((*name).to_string(), Value::from(*value));
        }
    }
    json!({
        "type": "Feature",
        "id": oid,
        "geometry": geometry(polygons),
        "properties": schema,
    })
}

/// A LoD2 building in the simplified schema: a building (`31001_…`, or without a function)
/// with the label of its function, or one of the two roofed structures (`51009_1610`,
/// `51009_1611`). `None` for anything else LoD2 holds: bridges, walls, locks. LoD2 holds what
/// stands above the ground, so there is no `rellage`.
pub fn building(found: &CityBuilding) -> Option<Value> {
    let (object_type, code) = match found.function.as_deref() {
        None => (BUILDING_TYPE, ""),
        Some(function) => function.split_once('_').unwrap_or((function, "")),
    };
    let (kind, function) = match object_type {
        BUILDING_TYPE => ("Gebäude", building_function(code)),
        STRUCTURE_TYPE => (STRUCTURE, Some(roof_function(code)?)),
        _ => return None,
    };
    let oid = match &found.alkis_id {
        Some(id) => format!("{id}{BUILDING_SUFFIX}"),
        None => found.gml_id.clone(),
    };
    Some(feature(
        &oid,
        &found.ground,
        &[
            ("gebnutzbez", Some(kind)),
            ("funktion", function),
            ("aktualit", found.outline_date.as_deref()),
        ],
    ))
}

/// A record of the land use file in the simplified schema; `None` without an id. A square
/// without `bez` gets [`SQUARE_BEZ`].
fn land_use(record: &ShapeRecord) -> Option<Value> {
    let field = |name: &str| record.fields.get(name).map(String::as_str);
    let oid = field("oid").or_else(|| field("gml_id"))?;
    let nutzart = field("nutzart");
    let bez = field("bez").or_else(|| (nutzart == Some(SQUARE)).then_some(SQUARE_BEZ));
    Some(feature(
        oid,
        &record.polygons,
        &[
            ("aktualit", field("aktualit")),
            ("nutzart", nutzart),
            ("bez", bez),
            ("name", field("name")),
        ],
    ))
}

/// Whether a point of the feature lies in `bbox`'s extent: its box meets the box.
fn meets(feature: &Value, bbox: &BoundingBox) -> bool {
    let mut extent: Option<BoundingBox> = None;
    let mut visit = |x: f64, y: f64| {
        let b = extent.get_or_insert(point_box(x, y));
        b.min_e = b.min_e.min(x);
        b.min_n = b.min_n.min(y);
        b.max_e = b.max_e.max(x);
        b.max_n = b.max_n.max(y);
    };
    fn walk(value: &Value, visit: &mut dyn FnMut(f64, f64)) {
        let Some(list) = value.as_array() else { return };
        match (
            list.first().and_then(Value::as_f64),
            list.get(1).and_then(Value::as_f64),
        ) {
            (Some(x), Some(y)) => visit(x, y),
            _ => list.iter().for_each(|inner| walk(inner, visit)),
        }
    }
    walk(&feature["geometry"]["coordinates"], &mut visit);
    extent.is_some_and(|e| {
        e.min_e <= bbox.max_e
            && bbox.min_e <= e.max_e
            && e.min_n <= bbox.max_n
            && bbox.min_n <= e.max_n
    })
}

/// The failure of a file of the agency, with why.
fn refused(url: &str, message: String) -> FetchError {
    FetchError::Http {
        url: url.to_string(),
        message,
    }
}

/// The buildings of the LoD2 tile at `url`: streamed into a file beside the cache, read as a
/// stream, and the file removed again whatever happens.
fn fetch_lod2(
    fetcher: &dyn Fetcher,
    url: &str,
    part: &Path,
    progress: &mut dyn FnMut(Note),
) -> Result<Vec<Value>, FetchError> {
    let _ = std::fs::remove_file(part);
    progress(note!("progress-downloading", url = url));
    let read = download_with_retries(fetcher, url, part, LOD2_MAX_BYTES, progress).and_then(|()| {
        let file = std::fs::File::open(part).map_err(|e| FetchError::Io(part.to_path_buf(), e))?;
        let mut features = Vec::new();
        read_buildings(std::io::BufReader::new(file), &mut |found| {
            features.extend(building(&found));
        })
        .map_err(|message| refused(url, format!("LoD2 tile: {message}")))?;
        Ok(features)
    });
    let _ = std::fs::remove_file(part);
    read
}

/// The features of a cached LoD2 tile.
fn cached_lod2(path: &Path) -> Result<Vec<Value>, FetchError> {
    let bytes = std::fs::read(path).map_err(|e| FetchError::Io(path.to_path_buf(), e))?;
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(mut collection)) => match collection.remove("features") {
            Some(Value::Array(features)) => Ok(features),
            _ => Err(()),
        },
        _ => Err(()),
    }
    .map_err(|()| {
        refused(
            &path.display().to_string(),
            "the cached buildings cannot be read; delete the file and fetch again".to_string(),
        )
    })
}

/// `Nutzung.shp` and `Nutzung.dbf` of the land use zip of a Kreis.
fn nutzung(zip: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip))
        .map_err(|e| format!("not a zip archive: {e}"))?;
    let mut member = |suffix: &str| -> Result<Vec<u8>, String> {
        let index = (0..archive.len())
            .find(|&i| {
                archive.by_index_raw(i).is_ok_and(|file| {
                    file.name()
                        .is_ok_and(|name| name.to_lowercase().ends_with(suffix))
                })
            })
            .ok_or_else(|| format!("no {suffix} in the zip"))?;
        let mut bytes = Vec::new();
        archive
            .by_index(index)
            .map_err(|e| e.to_string())?
            .take(TN_MEMBER_MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > TN_MEMBER_MAX_BYTES {
            return Err(format!(
                "{suffix} is larger than {TN_MEMBER_MAX_BYTES} bytes"
            ));
        }
        Ok(bytes)
    };
    Ok((member("nutzung.shp")?, member("nutzung.dbf")?))
}

/// The cadastre of Bavaria from the files of the Bayerische Vermessungsverwaltung.
pub struct LdbvAlkis {
    /// Which Kreis lies where.
    pub kreise: &'static KreisIndex,
}

impl AlkisSource for LdbvAlkis {
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let bbox = area.bbox_for(Crs::Utm32);
        let dir = env.cache.provider_dir(CACHE_DIR).join(LOD2_DIR);
        let mut features = Vec::new();
        let mut seen = HashSet::new();
        for tile in lod2_tiles(&bbox) {
            let (stem, url) = (lod2_stem(tile), lod2_url(tile));
            let file = Published {
                dir: &dir,
                stem: &stem,
                ext: "geojson",
                url: &url,
            };
            let of_tile = match current(env.fetcher, env.mode, &file, env.progress)? {
                Current::Cached(path) => cached_lod2(&path)?,
                Current::Fetch { path, older } => {
                    let part = dir.join(format!("{stem}.gml.part"));
                    let found = fetch_lod2(env.fetcher, &url, &part, env.progress)?;
                    let collection = json!({ "type": "FeatureCollection", "features": found });
                    write_cache_file(&path, collection.to_string().as_bytes())?;
                    drop_older(&older);
                    match collection {
                        Value::Object(mut collection) => match collection.remove("features") {
                            Some(Value::Array(found)) => found,
                            _ => Vec::new(),
                        },
                        _ => Vec::new(),
                    }
                }
                Current::Missing => {
                    // A tile without buildings is an empty file on the server; no file at
                    // all inside the state means it was moved.
                    if env.mode == FetchMode::Online && in_bavaria(tile) {
                        (env.progress)(note!("warning-ldbv-no-buildings-file", tile = stem));
                    }
                    Vec::new()
                }
            };
            for feature in of_tile {
                let new = feature["id"]
                    .as_str()
                    .is_none_or(|oid| seen.insert(oid.to_string()));
                if new && meets(&feature, &bbox) {
                    features.push(feature);
                }
            }
        }
        Ok(AlkisPart {
            features,
            crs: Crs::Utm32,
        })
    }

    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError> {
        let bbox = area.bbox_for(Crs::Utm32);
        let dir = env.cache.provider_dir(CACHE_DIR).join(TN_DIR);
        let mut features = Vec::new();
        let mut seen = HashSet::new();
        for ags in self.kreise.near(&area.region_bbox) {
            let (stem, url) = (format!("tn_{ags}"), tn_url(&ags));
            let file = Published {
                dir: &dir,
                stem: &stem,
                ext: "zip",
                url: &url,
            };
            let (shp, dbf) = match current(env.fetcher, env.mode, &file, env.progress)? {
                Current::Cached(path) => {
                    let bytes =
                        std::fs::read(&path).map_err(|e| FetchError::Io(path.clone(), e))?;
                    nutzung(&bytes).map_err(|message| refused(&url, message))?
                }
                Current::Fetch { path, older } => {
                    (env.progress)(note!("progress-downloading", url = &url));
                    let part = dir.join(format!("{stem}.zip.part"));
                    let (bytes, files) = get_limited_with_retries(
                        env.fetcher,
                        &url,
                        &part,
                        TN_MAX_BYTES,
                        |bytes| {
                            let files = nutzung(bytes).map_err(|message| refused(&url, message))?;
                            Ok((bytes.to_vec(), files))
                        },
                    )?;
                    write_cache_file(&path, &bytes)?;
                    drop_older(&older);
                    files
                }
                Current::Missing => {
                    return Err(refused(
                        &url,
                        format!("the land use file of the Kreis {ags} is not on the server (404)"),
                    )
                    .into());
                }
            };
            let records = polygons_in(&shp, &dbf, &bbox).map_err(|message| {
                refused(&url, format!("land use of the Kreis {ags}: {message}"))
            })?;
            for feature in records.iter().filter_map(land_use) {
                let new = feature["id"]
                    .as_str()
                    .is_none_or(|oid| seen.insert(oid.to_string()));
                if new {
                    features.push(feature);
                }
            }
        }
        Ok(AlkisPart {
            features,
            crs: Crs::Utm32,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use freemapper_adapter_api::registry::Registry;
    use freemapper_sources::citygml::read_buildings;
    use freemapper_sources::shapefile::test_support::{Entry, shape_files};

    use super::*;
    use crate::crs::ProjectedPoint;
    use crate::fetch::Cache;
    use crate::fetch::test_support::MapFetcher;

    const FIXTURE: &str = include_str!("../../../../tests/fixtures/bayern-lod2.gml");
    /// The tile the fixture is cut from.
    const TILE: (u32, u32) = (726, 5434);
    /// A tile without buildings, as the server has it for forest and water.
    const EMPTY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/1.0" xmlns:gml="http://www.opengis.net/gml">
  <gml:name>LoD2_32_724_5434_2_BY</gml:name>
</core:CityModel>
"#;

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    /// Runs `run` with an environment on `fetcher` and a cache in `dir`; its result and the
    /// notes it reported.
    fn with_env<R>(
        fetcher: &dyn Fetcher,
        dir: &Path,
        mode: FetchMode,
        run: impl FnOnce(&mut AdapterEnv<'_>) -> R,
    ) -> (R, Vec<Note>) {
        let cache = Cache::new(dir.to_path_buf());
        let registry = Registry::new();
        let mut notes = Vec::new();
        let result = {
            let mut progress = |note: Note| notes.push(note);
            let mut env = AdapterEnv {
                fetcher,
                regions: &registry,
                cache: &cache,
                mode,
                today: "2026-11-14".parse().unwrap(),
                skip_surface: false,
                progress: &mut progress,
            };
            run(&mut env)
        };
        (result, notes)
    }

    fn warnings(notes: &[Note]) -> Vec<&'static str> {
        notes
            .iter()
            .filter(|note| note.is_warning())
            .map(Note::id)
            .collect()
    }

    /// The buildings of the fixture.
    fn fixture_buildings() -> Vec<CityBuilding> {
        let mut buildings = Vec::new();
        read_buildings(FIXTURE.as_bytes(), &mut |b| buildings.push(b)).unwrap();
        buildings
    }

    /// The area around the three buildings of the fixture, widened by `west` metres to the
    /// west.
    fn fixture_area(west: f64) -> FetchArea {
        let mut bbox: Option<BoundingBox> = None;
        for [x, y] in fixture_buildings()
            .iter()
            .flat_map(|b| b.ground.iter().flatten().flatten())
        {
            let p = Crs::Utm33
                .reproject(
                    Crs::Utm32,
                    ProjectedPoint {
                        easting: *x,
                        northing: *y,
                    },
                )
                .unwrap();
            let b = bbox.get_or_insert(BoundingBox {
                min_e: p.easting,
                min_n: p.northing,
                max_e: p.easting,
                max_n: p.northing,
            });
            b.min_e = b.min_e.min(p.easting);
            b.min_n = b.min_n.min(p.northing);
            b.max_e = b.max_e.max(p.easting);
            b.max_n = b.max_n.max(p.northing);
        }
        let mut bbox = bbox.unwrap().buffered(10.0);
        bbox.min_e -= west;
        FetchArea::new(bbox, Crs::Utm32)
    }

    /// Puts the tiles of `area` on the server, last changed on `changed`: `body` for the tile
    /// of the fixture, `others` for the rest.
    fn serve_lod2(
        fetcher: &MapFetcher,
        area: &FetchArea,
        (body, others): (&str, &str),
        changed: NaiveDate,
    ) {
        for tile in lod2_tiles(&area.bbox_for(Crs::Utm32)) {
            let url = lod2_url(tile);
            let text = if tile == TILE { body } else { others };
            fetcher.insert(&url, text.as_bytes().to_vec());
            fetcher.set_modified(&url, changed);
        }
    }

    /// Two Kreise side by side in EPSG:25833, the city of Regensburg and its Landkreis by
    /// their keys; they share the edge at E 290 000.
    fn two_kreise() -> &'static KreisIndex {
        let square = |ags: &str, west: i32| {
            format!(
                r#"{{"type":"Feature","properties":{{"ags":"{ags}","name":"{ags}"}},"geometry":{{"type":"MultiPolygon","coordinates":[[[[{west},5430000],[{west},5440000],[{east},5440000],[{east},5430000],[{west},5430000]]]]}}}}"#,
                east = west + 10_000
            )
        };
        let text = format!(
            r#"{{"type":"FeatureCollection","features":[{},{}]}}"#,
            square("09362", 280_000),
            square("09375", 290_000)
        );
        Box::leak(Box::new(KreisIndex::parse(&text).unwrap()))
    }

    /// An area of 200 m in the first of the [`two_kreise`], its east edge `gap` metres from
    /// the edge they share.
    fn kreis_area(gap: f64) -> FetchArea {
        FetchArea::new(
            BoundingBox {
                min_e: 290_000.0 - gap - 200.0,
                min_n: 5_434_000.0,
                max_e: 290_000.0 - gap,
                max_n: 5_434_200.0,
            },
            Crs::Utm32,
        )
    }

    /// A clockwise square of 20 m, `offset` metres inside the south-west corner of `area` in
    /// EPSG:25832.
    fn square_in(area: &FetchArea, offset: f64) -> Vec<[f64; 2]> {
        let b = area.bbox_for(Crs::Utm32);
        let (x, y) = (b.min_e + offset, b.min_n + offset);
        vec![
            [x, y],
            [x, y + 20.0],
            [x + 20.0, y + 20.0],
            [x + 20.0, y],
            [x, y],
        ]
    }

    /// The land use zip of a Kreis with `records`.
    fn tn_zip(records: &[Entry<'_>]) -> Vec<u8> {
        use std::io::Write;
        let (shp, dbf) = shape_files(records);
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes) in [
            ("Nutzung.cpg", b"UTF-8".to_vec()),
            ("Nutzung.dbf", dbf),
            ("Nutzung.shp", shp),
        ] {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    fn serve_tn(fetcher: &MapFetcher, ags: &str, bytes: Vec<u8>, changed: NaiveDate) {
        fetcher.insert(&tn_url(ags), bytes);
        fetcher.set_modified(&tn_url(ags), changed);
    }

    /// A LoD2 building with the function `function` and one square as its ground.
    fn city_building(function: Option<&str>) -> CityBuilding {
        CityBuilding {
            gml_id: "DEBY_LOD2_1".to_string(),
            alkis_id: Some("DEBYvAAAAAB0000x".to_string()),
            function: function.map(str::to_string),
            outline_date: Some("2026-01-09".to_string()),
            ground: vec![vec![vec![
                [0.0, 0.0],
                [0.0, 10.0],
                [10.0, 10.0],
                [10.0, 0.0],
                [0.0, 0.0],
            ]]],
        }
    }

    #[test]
    fn a_lod2_building_reads_as_the_simplified_schema() {
        let feature = building(&fixture_buildings()[0]).unwrap();
        assert_eq!(feature["type"], "Feature");
        assert_eq!(feature["id"], "DEBYvAAAAABA6CHpBL");
        assert_eq!(
            feature["properties"],
            serde_json::json!({
                "oid": "DEBYvAAAAABA6CHpBL",
                "gebnutzbez": "Gebäude",
                "funktion": "Gebäude für Wirtschaft oder Gewerbe",
                "aktualit": "2026-01-09",
            })
        );
        assert_eq!(feature["geometry"]["type"], "Polygon");
        let first = &feature["geometry"]["coordinates"][0][0];
        assert_eq!(first.as_array().unwrap().len(), 2, "no height");
        // The second building of the fixture has a hole, so two rings.
        let second = building(&fixture_buildings()[1]).unwrap();
        assert_eq!(
            second["geometry"]["coordinates"].as_array().unwrap().len(),
            2
        );
        // Without an ALKIS id the id of the LoD2 object stands in; several ground polygons
        // are a MultiPolygon.
        let mut plain = city_building(Some("31001_1010"));
        plain.alkis_id = None;
        plain.ground.push(plain.ground[0].clone());
        let feature = building(&plain).unwrap();
        assert_eq!(feature["properties"]["oid"], "DEBY_LOD2_1");
        assert_eq!(feature["properties"]["funktion"], "Wohnhaus");
        assert_eq!(feature["geometry"]["type"], "MultiPolygon");
    }

    #[test]
    fn a_roof_is_a_structure_and_other_structures_are_left_out() {
        let roof = building(&city_building(Some("51009_1610"))).unwrap();
        assert_eq!(
            roof["properties"]["gebnutzbez"],
            "Sonstiges Bauwerk oder sonstige Einrichtung"
        );
        assert_eq!(roof["properties"]["funktion"], "Überdachung");
        let carport = building(&city_building(Some("51009_1611"))).unwrap();
        assert_eq!(carport["properties"]["funktion"], "Carport");
        // A bridge, a wall, a historic structure, a lock: no buildings.
        for other in ["53001_1800", "51009_1700", "51007_1500", "52003_1020"] {
            assert!(building(&city_building(Some(other))).is_none(), "{other}");
        }
        // No function at all, or a code the list does not know: a building without one.
        for unknown in [None, Some("31001_0000"), Some("31001")] {
            let plain = building(&city_building(unknown)).unwrap();
            assert_eq!(plain["properties"]["gebnutzbez"], "Gebäude");
            assert!(plain["properties"].get("funktion").is_none(), "{unknown:?}");
        }
    }

    #[test]
    fn a_tiefgarage_in_lod2_stands_above_ground() {
        let garage = building(&city_building(Some("31001_2465"))).unwrap();
        assert_eq!(garage["properties"]["funktion"], "Tiefgarage");
        assert!(garage["properties"].get("rellage").is_none());
    }

    #[test]
    fn lod2_tiles_are_the_even_kilometres() {
        let bbox = BoundingBox {
            min_e: 691_300.0,
            min_n: 5_335_200.0,
            max_e: 692_100.0,
            max_n: 5_336_400.0,
        };
        assert_eq!(
            lod2_tiles(&bbox),
            [(690, 5334), (690, 5336), (692, 5334), (692, 5336)]
        );
        assert_eq!(
            lod2_url((690, 5334)),
            "https://download1.bayernwolke.de/a/lod2/citygml/690_5334.gml"
        );
        assert_eq!(
            tn_url("09162"),
            "https://download1.bayernwolke.de/a/tn/lkr/tn_09162.zip"
        );
    }

    #[test]
    fn buildings_come_from_the_tiles_and_are_cached_as_features() {
        let area = fixture_area(0.0);
        let fetcher = MapFetcher::default();
        serve_lod2(&fetcher, &area, (FIXTURE, EMPTY), day(2026, 10, 6));
        let dir = tempfile::tempdir().unwrap();
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        let (part, notes) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap()
        });
        assert_eq!(part.crs, Crs::Utm32);
        assert_eq!(part.features.len(), 3);
        assert_eq!(part.features[2]["properties"]["funktion"], "Überdachung");
        assert!(warnings(&notes).is_empty(), "{notes:?}");
        let tiles = dir.path().join("ldbv/lod2");
        assert!(tiles.join("726_5434-20261006.geojson").exists());
        let left: Vec<String> = std::fs::read_dir(&tiles)
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".gml"))
            .collect();
        assert!(left.is_empty(), "{left:?}");
        // Offline the cached features give the same, and nothing is asked.
        let silent = MapFetcher::default();
        let (again, _) = with_env(&silent, dir.path(), FetchMode::Offline, |env| {
            source.buildings(env, &area).unwrap()
        });
        assert_eq!(again.features, part.features);
        assert_eq!(silent.calls.get(), 0);
    }

    #[test]
    fn buildings_outside_the_area_are_left_out() {
        // An area around the first building of the fixture only.
        let wide = fixture_area(0.0);
        let first = &building(&fixture_buildings()[0]).unwrap()["geometry"]["coordinates"][0][0];
        let (x, y) = (first[0].as_f64().unwrap(), first[1].as_f64().unwrap());
        let p = Crs::Utm33
            .reproject(
                Crs::Utm32,
                ProjectedPoint {
                    easting: x,
                    northing: y,
                },
            )
            .unwrap();
        let area = FetchArea::new(
            BoundingBox {
                min_e: p.easting - 1.0,
                min_n: p.northing - 1.0,
                max_e: p.easting + 1.0,
                max_n: p.northing + 1.0,
            },
            Crs::Utm32,
        );
        let fetcher = MapFetcher::default();
        serve_lod2(&fetcher, &wide, (FIXTURE, EMPTY), day(2026, 10, 6));
        let dir = tempfile::tempdir().unwrap();
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        let (part, _) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap()
        });
        assert_eq!(part.features.len(), 1);
        assert_eq!(part.features[0]["id"], "DEBYvAAAAABA6CHpBL");
    }

    /// A server whose tiles can only be streamed to a file: `get` fails for them.
    struct Streamed {
        inner: MapFetcher,
        too_large: bool,
    }

    impl Fetcher for Streamed {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            Err(FetchError::Http {
                url: url.to_string(),
                message: "held in memory".to_string(),
            })
        }

        fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
            self.inner.last_modified(url)
        }

        fn download_to(
            &self,
            url: &str,
            path: &Path,
            max_bytes: u64,
            _progress: &mut dyn FnMut(Note),
        ) -> Result<(), FetchError> {
            if self.too_large {
                return Err(crate::fetch::too_large(url, max_bytes));
            }
            let bytes = self.inner.get(url)?;
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
            Ok(())
        }
    }

    #[test]
    fn the_tile_is_streamed_never_fetched_into_memory() {
        let area = fixture_area(0.0);
        let inner = MapFetcher::default();
        serve_lod2(&inner, &area, (FIXTURE, EMPTY), day(2026, 10, 6));
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        let dir = tempfile::tempdir().unwrap();
        let fetcher = Streamed {
            inner,
            too_large: false,
        };
        let (part, _) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap()
        });
        assert_eq!(part.features.len(), 3);
        // A file above the limit is refused with the name of the tile, and nothing is kept.
        let other = tempfile::tempdir().unwrap();
        let fetcher = Streamed {
            too_large: true,
            ..fetcher
        };
        let (error, _) = with_env(&fetcher, other.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap_err().to_string()
        });
        assert!(error.contains("726_5434"), "{error}");
        assert!(error.contains("512"), "{error}");
        let kept = std::fs::read_dir(other.path().join("ldbv/lod2"))
            .map(|entries| entries.count())
            .unwrap_or(0);
        assert_eq!(kept, 0);
    }

    #[test]
    fn half_a_citygml_file_is_refused_and_the_older_cache_stays() {
        let area = fixture_area(0.0);
        let dir = tempfile::tempdir().unwrap();
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        // A first run caches the tile of July.
        let fetcher = MapFetcher::default();
        serve_lod2(&fetcher, &area, (FIXTURE, EMPTY), day(2026, 7, 1));
        with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap()
        });
        let older = dir.path().join("ldbv/lod2/726_5434-20260701.geojson");
        assert!(older.exists());
        // In October the server sends half a file.
        let end = FIXTURE
            .match_indices("</cityObjectMember>")
            .nth(1)
            .unwrap()
            .0;
        serve_lod2(&fetcher, &area, (&FIXTURE[..end], EMPTY), day(2026, 10, 6));
        let (error, _) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap_err().to_string()
        });
        assert!(error.contains("726_5434"), "{error}");
        assert!(older.exists(), "the older features stay");
        let names: Vec<String> = std::fs::read_dir(dir.path().join("ldbv/lod2"))
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            !names
                .iter()
                .any(|name| name.contains("20261006") || name.contains(".part")),
            "{names:?}"
        );
        // An HTML page is refused the same way.
        serve_lod2(
            &fetcher,
            &area,
            ("<html><body>502</body></html>", EMPTY),
            day(2026, 10, 7),
        );
        let (refused, _) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).is_err()
        });
        assert!(refused);
        assert!(older.exists());
    }

    #[test]
    fn a_building_two_tiles_hold_is_kept_once() {
        // The area reaches into the tile to the west, which holds the same three buildings.
        let area = fixture_area(2500.0);
        assert!(lod2_tiles(&area.bbox_for(Crs::Utm32)).len() >= 2);
        let fetcher = MapFetcher::default();
        serve_lod2(&fetcher, &area, (FIXTURE, FIXTURE), day(2026, 10, 6));
        let dir = tempfile::tempdir().unwrap();
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        let (part, _) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap()
        });
        assert_eq!(part.features.len(), 3);
    }

    #[test]
    fn a_tile_the_server_does_not_have_warns_inside_bavaria() {
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        // Regensburg: the server has no file for the tile.
        let area = fixture_area(0.0);
        let tiles = lod2_tiles(&area.bbox_for(Crs::Utm32)).len();
        let dir = tempfile::tempdir().unwrap();
        let gone = MapFetcher::default();
        let (part, notes) = with_env(&gone, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap()
        });
        assert!(part.features.is_empty());
        assert_eq!(
            warnings(&notes),
            vec!["warning-ldbv-no-buildings-file"; tiles]
        );
        assert!(notes[0].contains("726_5434"), "{:?}", notes[0]);
        // Stuttgart: beyond the border nothing is said.
        let stuttgart = FetchArea::new(
            BoundingBox {
                min_e: 72_600.0,
                min_n: 5_418_800.0,
                max_e: 72_800.0,
                max_n: 5_419_000.0,
            },
            Crs::Utm32,
        );
        let (part, notes) = with_env(&gone, dir.path(), FetchMode::Online, |env| {
            source.buildings(env, &stuttgart).unwrap()
        });
        assert!(part.features.is_empty());
        assert!(notes.is_empty(), "{notes:?}");
        // A tile of forest: the server has an empty model, and nothing is said either.
        let other = tempfile::tempdir().unwrap();
        let forest = MapFetcher::default();
        serve_lod2(&forest, &area, (EMPTY, EMPTY), day(2022, 12, 6));
        let (part, notes) = with_env(&forest, other.path(), FetchMode::Online, |env| {
            source.buildings(env, &area).unwrap()
        });
        assert!(part.features.is_empty());
        assert!(warnings(&notes).is_empty(), "{notes:?}");
    }

    #[test]
    fn kreise_near_the_area_are_found() {
        let kreise = two_kreise();
        assert_eq!(
            kreise.near(&kreis_area(400.0).region_bbox),
            ["09362", "09375"]
        );
        assert_eq!(kreise.near(&kreis_area(600.0).region_bbox), ["09362"]);
        // Far from both.
        let far = BoundingBox {
            min_e: 100_000.0,
            min_n: 5_400_000.0,
            max_e: 100_200.0,
            max_n: 5_400_200.0,
        };
        assert!(kreise.near(&far).is_empty());
        assert!(KreisIndex::parse("{}").is_err());
        assert!(
            KreisIndex::parse(r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},"geometry":null}]}"#)
                .is_err()
        );
    }

    #[test]
    fn the_built_in_kreise_are_the_96_of_bavaria() {
        let kreise = built_in_kreise();
        // Regensburg lies in the city and next to its Landkreis.
        let regensburg = BoundingBox {
            min_e: 287_700.0,
            min_n: 5_433_900.0,
            max_e: 287_900.0,
            max_n: 5_434_100.0,
        };
        assert_eq!(kreise.near(&regensburg), ["09362"]);
        // The whole state touches all of them.
        let state = BoundingBox {
            min_e: 60_000.0,
            min_n: 5_240_000.0,
            max_e: 420_000.0,
            max_n: 5_620_000.0,
        };
        let all = kreise.near(&state);
        assert_eq!(all.len(), 96);
        assert!(
            all.iter()
                .all(|ags| ags.len() == 5 && ags.starts_with("09"))
        );
    }

    #[test]
    fn land_use_comes_from_the_files_of_the_kreise() {
        let area = kreis_area(400.0);
        let (wood, both, path) = (
            [square_in(&area, 10.0)],
            [square_in(&area, 50.0)],
            [square_in(&area, 90.0)],
        );
        let far = [square_in(&area, 5000.0)];
        let city: &[Entry<'_>] = &[
            (
                &[
                    ("oid", "DEBYvAAAAAA1TN"),
                    ("aktualit", "2026-10-15Z"),
                    ("nutzart", "Wald"),
                ],
                &wood,
            ),
            (
                &[
                    ("oid", "DEBYvAAAAAA2TN"),
                    ("nutzart", "Sport-, Freizeit- und Erholungsfläche"),
                    ("bez", "Grünanlage"),
                    ("name", "Stadtpark"),
                ],
                &both,
            ),
            (&[("oid", "DEBYvAAAAAA9TN"), ("nutzart", "Wald")], &far),
        ];
        let county: &[Entry<'_>] = &[
            (
                &[
                    ("oid", "DEBYvAAAAAA2TN"),
                    ("nutzart", "Sport-, Freizeit- und Erholungsfläche"),
                    ("bez", "Grünanlage"),
                    ("name", "Stadtpark"),
                ],
                &both,
            ),
            (&[("oid", "DEBYvAAAAAA3TN"), ("nutzart", "Weg")], &path),
        ];
        let fetcher = MapFetcher::default();
        serve_tn(&fetcher, "09362", tn_zip(city), day(2026, 10, 15));
        serve_tn(&fetcher, "09375", tn_zip(county), day(2026, 10, 16));
        let dir = tempfile::tempdir().unwrap();
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        let (part, notes) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.land_use(env, &area).unwrap()
        });
        assert!(warnings(&notes).is_empty(), "{notes:?}");
        assert_eq!(part.crs, Crs::Utm32);
        let oids: Vec<&str> = part
            .features
            .iter()
            .map(|f| f["properties"]["oid"].as_str().unwrap())
            .collect();
        assert_eq!(oids, ["DEBYvAAAAAA1TN", "DEBYvAAAAAA2TN", "DEBYvAAAAAA3TN"]);
        assert_eq!(
            part.features[0]["properties"],
            serde_json::json!({
                "oid": "DEBYvAAAAAA1TN",
                "aktualit": "2026-10-15Z",
                "nutzart": "Wald",
            })
        );
        assert_eq!(part.features[0]["id"], "DEBYvAAAAAA1TN");
        assert_eq!(part.features[0]["geometry"]["type"], "Polygon");
        assert_eq!(part.features[1]["properties"]["bez"], "Grünanlage");
        assert_eq!(part.features[1]["properties"]["name"], "Stadtpark");
        let files = dir.path().join("ldbv/tn");
        assert!(files.join("tn_09362-20261015.zip").exists());
        assert!(files.join("tn_09375-20261016.zip").exists());
        // Offline the same, with nothing asked.
        let silent = MapFetcher::default();
        let (again, _) = with_env(&silent, dir.path(), FetchMode::Offline, |env| {
            source.land_use(env, &area).unwrap()
        });
        assert_eq!(again.features, part.features);
        assert_eq!(silent.calls.get(), 0);
        // Farther from the edge only the file of the city is read.
        let inner = kreis_area(600.0);
        let only_city = MapFetcher::default();
        serve_tn(&only_city, "09362", tn_zip(city), day(2026, 10, 15));
        let other = tempfile::tempdir().unwrap();
        with_env(&only_city, other.path(), FetchMode::Online, |env| {
            source.land_use(env, &inner).unwrap()
        });
        assert_eq!(only_city.log.borrow().len(), 1);
    }

    #[test]
    fn a_square_without_a_use_stays_a_candidate() {
        let area = kreis_area(600.0);
        let rings = [square_in(&area, 10.0)];
        let records: &[Entry<'_>] = &[
            (&[("oid", "A"), ("nutzart", "Platz")], &rings),
            (
                &[("oid", "B"), ("nutzart", "Platz"), ("bez", "Marktplatz")],
                &rings,
            ),
            (&[("oid", "C"), ("nutzart", "Weg")], &rings),
        ];
        let fetcher = MapFetcher::default();
        serve_tn(&fetcher, "09362", tn_zip(records), day(2026, 10, 15));
        let dir = tempfile::tempdir().unwrap();
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        let (part, _) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
            source.land_use(env, &area).unwrap()
        });
        assert_eq!(
            part.features[0]["properties"]["bez"],
            "ohne Unterart (Bayern)"
        );
        assert_eq!(part.features[1]["properties"]["bez"], "Marktplatz");
        assert!(part.features[2]["properties"].get("bez").is_none());
    }

    #[test]
    fn a_zip_without_the_land_use_is_refused() {
        use std::io::Write;
        let area = kreis_area(600.0);
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("liesmich.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"nichts").unwrap();
        let wrong = zip.finish().unwrap().into_inner();
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        for body in [wrong, b"<html><body>502</body></html>".to_vec()] {
            let fetcher = MapFetcher::default();
            serve_tn(&fetcher, "09362", body, day(2026, 10, 15));
            let dir = tempfile::tempdir().unwrap();
            let (error, _) = with_env(&fetcher, dir.path(), FetchMode::Online, |env| {
                source.land_use(env, &area).unwrap_err().to_string()
            });
            assert!(error.contains("tn_09362.zip"), "{error}");
            let kept = std::fs::read_dir(dir.path().join("ldbv/tn"))
                .map(|entries| entries.count())
                .unwrap_or(0);
            assert_eq!(kept, 0, "nothing is cached");
        }
    }

    #[test]
    fn a_missing_kreis_file_names_the_kreis() {
        let area = kreis_area(600.0);
        let source = LdbvAlkis {
            kreise: two_kreise(),
        };
        let dir = tempfile::tempdir().unwrap();
        let (error, _) = with_env(
            &MapFetcher::default(),
            dir.path(),
            FetchMode::Online,
            |env| source.land_use(env, &area).unwrap_err().to_string(),
        );
        assert!(error.contains("09362"), "{error}");
        assert!(error.contains("tn_09362.zip"), "{error}");
    }
}
