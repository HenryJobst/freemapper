//! The laser points of LVermGeo Rheinland-Pfalz (dl-de/by-2-0): one LAZ file per km tile in
//! EPSG:25832 (ADR 0105). The metadata of a tile gives the day of its scan; the Metalink of the
//! state lists every file with its size and SHA-256, and a download is kept only when both
//! match. The classes follow the AdV scheme: ground (2) and everything above it (20).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::NaiveDate;
use freemapper_adapter_api::cache::{CacheDir, STALE_PART_AGE};
use freemapper_adapter_api::env::AdapterEnv;
use freemapper_adapter_api::error::AdapterError;
use freemapper_adapter_api::estimate::{EstimateArea, EstimateEnv};
use freemapper_adapter_api::pointcloud::{
    PointCloudBackend, PointCloudInfo, PointCloudRecord, PointCloudTile,
};
use freemapper_adapter_api::records::{AdapterId, EstimateItem, Part, Store};

use crate::crs::BoundingBox;
use crate::fetch::{FetchError, FetchMode};
use crate::las_files::{
    cached_las, download_las, fallback, las_path, not_published,
    parse_utm32_tile_name as parse_tile_name, utm32_tile_name as tile_name,
};
use crate::note::{Arg, note};
use crate::tiles::{TileId, tiles_covering_grid};

const BASE: &str = "https://geobasis-rlp.de/data/las/current";
/// Every file of the state with its size and SHA-256 (about 6 MB for 21 000 tiles, 2026).
pub const META4_URL: &str = "https://geobasis-rlp.de/data/las/current/meta4/las_las_07.meta4";
/// The folder of the point clouds below the folder of LVermGeo RP in the cache.
const DIR: &str = "las";
/// A tile has 231 MB in the median and up to 1.2 GB (2026); the estimate counts this much per
/// tile.
pub const MB_PER_TILE: f64 = 230.0;

/// `lpolpg_32_448_5538_1_rp`: laser points of objects and ground, the stem of the tile files.
fn stem(tile: TileId) -> String {
    format!("lpolpg_32_{}_{}_1_rp", tile.east_km, tile.north_km)
}

/// URL of the point cloud of `tile`.
pub fn tile_url(tile: TileId) -> String {
    format!("{BASE}/las/{}.laz", stem(tile))
}

/// URL of the metadata of `tile`.
pub fn metadata_url(tile: TileId) -> String {
    format!("{BASE}/metadata/{}_meta.xml", stem(tile))
}

/// The point-cloud backend of Rhineland-Palatinate.
pub fn backend() -> &'static dyn PointCloudBackend {
    static RP: LazyLock<RpLasBackend> = LazyLock::new(RpLasBackend::new);
    &*RP
}

/// The laser points of LVermGeo RP, kept in the cache as LAZ.
pub struct RpLasBackend {
    info: PointCloudInfo,
}

impl RpLasBackend {
    pub fn new() -> Self {
        Self {
            info: PointCloudInfo {
                id: AdapterId::built_in("lvermgeorp"),
                cache_dir: "lvermgeo_rp".to_string(),
                source: "lvermgeorp-las".to_string(),
                epsg: 25832,
                grid_km: 1,
                left_out: Vec::new(),
            },
        }
    }

    fn dir(&self, cache: &CacheDir) -> PathBuf {
        cache.provider_dir(&self.info.cache_dir).join(DIR)
    }
}

impl Default for RpLasBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// One file of the Metalink.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Meta4File {
    bytes: u64,
    sha256: String,
}

/// The text of the first `<tag>…</tag>` in `text`, without any attributes of the tag.
fn element<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = text.find(&format!("<{tag}"))?;
    let after = &text[open + tag.len() + 1..];
    let body = &after[after.find('>')? + 1..];
    Some(body[..body.find(&format!("</{tag}>"))?].trim())
}

/// The files of the Metalink by tile. The tile comes from the URL, which has to be a tile file
/// of [`BASE`]; the `name` of a file differs from it in the Metalink of the state. Entries with
/// another URL, no size or a hash that is no SHA-256 are left out.
fn parse_meta4(bytes: &[u8]) -> Result<HashMap<TileId, Meta4File>, FetchError> {
    let corrupt = |message: &str| FetchError::CorruptArchive {
        url: META4_URL.to_string(),
        message: message.to_string(),
    };
    let text = std::str::from_utf8(bytes).map_err(|_| corrupt("not UTF-8"))?;
    if !text.contains("<metalink") {
        return Err(corrupt("no Metalink"));
    }
    let mut files = HashMap::new();
    for entry in text.split("</file>") {
        let Some(start) = entry.find("<file") else {
            continue;
        };
        let entry = &entry[start..];
        let tile = element(entry, "url").and_then(|url| {
            let name = url.strip_prefix(&format!("{BASE}/las/"))?;
            let tile = tile_of_file(name)?;
            (tile_url(tile) == url).then_some(tile)
        });
        let bytes = element(entry, "size").and_then(|s| s.parse::<u64>().ok());
        let sha256 = entry
            .contains(r#"<hash type="sha-256">"#)
            .then(|| element(entry, "hash"))
            .flatten()
            .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()));
        if let (Some(tile), Some(bytes), Some(sha256)) = (tile, bytes, sha256) {
            files.insert(
                tile,
                Meta4File {
                    bytes,
                    sha256: sha256.to_ascii_lowercase(),
                },
            );
        }
    }
    Ok(files)
}

/// The tile a file name like `lpolpg_32_448_5538_1_rp.laz` stands for.
fn tile_of_file(name: &str) -> Option<TileId> {
    let stem = name.strip_suffix(".laz")?;
    if !crate::lvermgeo_sh::is_plain(stem) {
        return None;
    }
    let ["lpolpg", "32", east, north, "1", "rp"] = stem.split('_').collect::<Vec<_>>()[..] else {
        return None;
    };
    Some(TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    })
}

/// The day the metadata of a tile names as its creation, the day of the scan.
fn creation_date(bytes: &[u8]) -> Option<NaiveDate> {
    let text = std::str::from_utf8(bytes).ok()?;
    let marker = text.find(r#"codeListValue="creation""#)?;
    let before = &text[..marker];
    let date = before.rfind("<gco:Date>")?;
    let inside = &before[date + "<gco:Date>".len()..];
    // The date has to belong to the same `CI_Date` as the marker.
    if inside.contains("</gmd:CI_Date>") {
        return None;
    }
    inside.get(..10)?.parse().ok()
}

/// The Metalink, asked for once per fetch and only when a tile has to be downloaded.
enum Index {
    NotAsked,
    Loaded(HashMap<TileId, Meta4File>),
    Failed,
}

impl PointCloudBackend for RpLasBackend {
    fn info(&self) -> &PointCloudInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        bbox: &BoundingBox,
    ) -> Result<Vec<PointCloudTile>, AdapterError> {
        let dir = self.dir(env.cache);
        let offline = env.mode == FetchMode::Offline;
        let mut index = Index::NotAsked;
        let mut fetched = Vec::new();
        for tile in tiles_covering_grid(bbox, self.info.grid_km) {
            let name = tile_name(tile);
            let url = tile_url(tile);
            let cached = cached_las(&dir, &name);
            let found = |(published, path): (NaiveDate, PathBuf)| PointCloudTile {
                record: PointCloudRecord {
                    tile: name.clone(),
                    url: url.clone(),
                    published: Some(published),
                },
                path,
            };
            if offline {
                match cached.into_iter().next() {
                    Some(newest) => fetched.push(found(newest)),
                    None => (env.progress)(note!("warning-point-cloud-no-cached", tile = &name)),
                }
                continue;
            }
            let published = match env.fetcher.get(&metadata_url(tile)).and_then(|bytes| {
                creation_date(&bytes).ok_or_else(|| FetchError::CorruptArchive {
                    url: metadata_url(tile),
                    message: "no creation date".to_string(),
                })
            }) {
                Ok(published) => published,
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Err(FetchError::NotFound(_)) => {
                    not_published(env, &name, cached, &mut fetched, found);
                    continue;
                }
                Err(e) => {
                    fallback(env, &name, cached, e, &mut fetched, found);
                    continue;
                }
            };
            let path = las_path(&dir, &name, published);
            if path.exists() {
                (env.progress)(note!("progress-point-cloud-cached", tile = &name));
                fetched.push(found((published, path)));
                continue;
            }
            if let Index::NotAsked = index {
                (env.progress)(note!("progress-downloading", url = META4_URL));
                index = match env
                    .fetcher
                    .get(META4_URL)
                    .and_then(|bytes| parse_meta4(&bytes))
                {
                    Ok(files) => Index::Loaded(files),
                    Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                    Err(e) => {
                        (env.progress)(note!(
                            "warning-point-clouds-unavailable",
                            reason = Arg::error(e)
                        ));
                        Index::Failed
                    }
                };
            }
            let files = match &index {
                Index::Loaded(files) => files,
                _ => {
                    match cached.into_iter().next() {
                        Some(newest) => fetched.push(found(newest)),
                        None => {
                            (env.progress)(note!("warning-point-cloud-no-cached", tile = &name))
                        }
                    }
                    continue;
                }
            };
            let Some(file) = files.get(&tile) else {
                not_published(env, &name, cached, &mut fetched, found);
                continue;
            };
            (env.progress)(note!("progress-downloading", url = &url));
            match download_las(env, &url, Some(file.bytes), Some(&file.sha256), &path) {
                Err(FetchError::Cancelled) => return Err(FetchError::Cancelled.into()),
                Ok(()) => {
                    for (_, older) in cached.iter().filter(|(_, p)| *p != path) {
                        let _ = std::fs::remove_file(older);
                    }
                    fetched.push(found((published, path)));
                }
                Err(e) => fallback(env, &name, cached, e, &mut fetched, found),
            }
        }
        freemapper_fs::remove_stale_cache_parts(&dir, STALE_PART_AGE);
        Ok(fetched)
    }

    fn cached(&self, cache: &CacheDir, record: &PointCloudRecord) -> Option<PathBuf> {
        let tile = parse_tile_name(&record.tile)?;
        let path = las_path(&self.dir(cache), &tile_name(tile), record.published?);
        path.exists().then_some(path)
    }

    fn estimate(&self, area: &EstimateArea, env: &EstimateEnv<'_>) -> Vec<EstimateItem> {
        let dir = self.dir(env.cache);
        tiles_covering_grid(&area.map_bbox, self.info.grid_km)
            .into_iter()
            .map(|tile| EstimateItem {
                what: format!("LVermGeo RP point cloud {}", tile_name(tile)),
                megabytes: MB_PER_TILE,
                cached: !cached_las(&dir, &tile_name(tile)).is_empty(),
                part: Part::PointCloud,
                store: Store::Cache,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use freemapper_adapter_api::registry::Registry;

    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const TILE: TileId = TileId {
        east_km: 448,
        north_km: 5538,
    };
    /// A box inside the tile, in Mainz.
    const BBOX: BoundingBox = BoundingBox {
        min_e: 448_300.0,
        min_n: 5_538_300.0,
        max_e: 448_400.0,
        max_n: 5_538_400.0,
    };

    fn registry() -> Registry {
        let mut registry = Registry::new();
        for adapter in crate::states::adapters() {
            registry.add(adapter).unwrap();
        }
        registry
    }

    /// A small LAZ point cloud of `points` points, in EPSG:25832 like the tiles.
    fn laz(points: u32) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.laz");
        let mut builder = las::Builder::from((1, 2));
        builder.point_format.is_compressed = true;
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(0.0),
            y: transform(5_000_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for i in 0..points {
            writer
                .write_point(las::Point {
                    x: 448_350.0,
                    y: 5_538_350.0 + f64::from(i) * 0.2,
                    z: 90.0,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        std::fs::read(path).unwrap()
    }

    fn sha256(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    use sha2::{Digest as _, Sha256};

    /// A Metalink in the form of the state's, with `name`s that differ from the files.
    fn meta4(files: &[(TileId, usize, &str)]) -> Vec<u8> {
        let mut text = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metalink xmlns=\"urn:ietf:params:xml:ns:metalink\">\n    <generator>RLP-Met4Generator</generator>\n    <published>2026-11-04 20:02:20+00:00</published>",
        );
        for (tile, size, hash) in files {
            text += &format!(
                "<file name=\"LAS_{e}_{n}_las12.laz\">\n        <size>{size}</size>\n        <hash type=\"sha-256\">{hash}</hash>\n        <url>{url}</url>\n\n    </file>",
                e = tile.east_km,
                n = tile.north_km,
                url = tile_url(*tile)
            );
        }
        text += "\n</metalink>\n";
        text.into_bytes()
    }

    /// Metadata in the form of the state's, with other dates around the creation.
    fn metadata(created: &str) -> Vec<u8> {
        format!(
            r#"<gmd:MD_Metadata><gmd:dateStamp><gco:Date>2026-01-15</gco:Date></gmd:dateStamp>
            <gmd:CI_Date><gmd:date><gco:Date>{created}</gco:Date></gmd:date><gmd:dateType>
            <gmd:CI_DateTypeCode codeList="x" codeListValue="creation">creation</gmd:CI_DateTypeCode>
            </gmd:dateType></gmd:CI_Date>
            <gmd:CI_Date><gmd:date><gco:Date>2019-03-25</gco:Date></gmd:date><gmd:dateType>
            <gmd:CI_DateTypeCode codeList="x" codeListValue="publication">publication</gmd:CI_DateTypeCode>
            </gmd:dateType></gmd:CI_Date></gmd:MD_Metadata>"#
        )
        .into_bytes()
    }

    fn serve(fetcher: &MapFetcher, laz: Vec<u8>, created: &str) {
        fetcher.insert(META4_URL, meta4(&[(TILE, laz.len(), &sha256(&laz))]));
        fetcher.insert(&metadata_url(TILE), metadata(created));
        fetcher.insert(&tile_url(TILE), laz);
    }

    fn fetch(
        fetcher: &MapFetcher,
        cache: &CacheDir,
        mode: FetchMode,
    ) -> (Vec<PointCloudTile>, Vec<String>) {
        let registry = registry();
        let mut notes = Vec::new();
        let mut env = AdapterEnv {
            fetcher,
            regions: &registry,
            cache,
            mode,
            today: "2026-11-12".parse().unwrap(),
            skip_surface: false,
            progress: &mut |n| notes.push(n.to_string()),
        };
        let tiles = RpLasBackend::new().fetch(&mut env, &BBOX).unwrap();
        (tiles, notes)
    }

    fn folder(cache: &CacheDir) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(cache.root().join("lvermgeo_rp/las"))
            .into_iter()
            .flatten()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn downloads(fetcher: &MapFetcher, suffix: &str) -> usize {
        fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.ends_with(suffix))
            .count()
    }

    #[test]
    fn file_names_are_read_strictly() {
        assert_eq!(tile_of_file("lpolpg_32_448_5538_1_rp.laz"), Some(TILE));
        for bad in [
            "lpolpg_32_448_5538_1_rp.las",
            "lpolpg_33_448_5538_1_rp.laz",
            "lpolpg_32_448_5538_2_rp.laz",
            "../lpolpg_32_448_5538_1_rp.laz",
            "lpo_32_448_5538_1_rp.laz",
            "lpolpg_32_x_5538_1_rp.laz",
        ] {
            assert_eq!(tile_of_file(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_metalink_gives_size_and_hash_by_the_url_and_skips_what_it_cannot_read() {
        let other = TileId {
            east_km: 449,
            north_km: 5538,
        };
        let hash = "AB".repeat(32);
        let mut text =
            String::from_utf8(meta4(&[(TILE, 10, &hash), (other, 20, "short")])).unwrap();
        text = text.replace(
            "</metalink>",
            "<file name=\"x.laz\"><size>5</size><hash type=\"md5\">00</hash><url>https://example.org/lpolpg_32_450_5538_1_rp.laz</url></file></metalink>",
        );
        let files = parse_meta4(text.as_bytes()).unwrap();
        assert_eq!(
            files,
            HashMap::from([(
                TILE,
                Meta4File {
                    bytes: 10,
                    sha256: "ab".repeat(32)
                }
            )])
        );
        assert!(parse_meta4(b"<html>").is_err());
    }

    #[test]
    fn the_creation_date_comes_from_its_own_date_entry() {
        assert_eq!(
            creation_date(&metadata("2025-03-22")),
            "2025-03-22".parse().ok()
        );
        assert_eq!(creation_date(b"<gco:Date>2025-03-22</gco:Date>"), None);
        let elsewhere = br#"<gco:Date>2025-03-22</gco:Date></gmd:CI_Date><gmd:CI_Date>
            <gmd:CI_DateTypeCode codeListValue="creation">"#;
        assert_eq!(creation_date(elsewhere), None);
    }

    #[test]
    fn a_tile_is_downloaded_checked_and_recorded_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, laz(50), "2025-03-22");
        let (tiles, _) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1);
        assert_eq!(
            tiles[0].record,
            PointCloudRecord {
                tile: "32448-5538".to_string(),
                url: format!("{BASE}/las/lpolpg_32_448_5538_1_rp.laz"),
                published: Some("2025-03-22".parse().unwrap()),
            }
        );
        assert_eq!(folder(&cache), ["32448-5538_2025-03-22.laz"]);
        assert_eq!(
            RpLasBackend::new().cached(&cache, &tiles[0].record),
            Some(tiles[0].path.clone())
        );
        // The second fetch asks only the metadata: no Metalink, no tile.
        let (again, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(again, tiles);
        assert_eq!(downloads(&fetcher, ".laz"), 1);
        assert_eq!(downloads(&fetcher, ".meta4"), 1);
        assert!(
            notes.iter().any(|n| n == "cached point cloud 32448-5538"),
            "{notes:?}"
        );
        // A newer scan replaces the cached file.
        serve(&fetcher, laz(60), "2029-04-01");
        let (newer, _) = fetch(&fetcher, &cache, FetchMode::Online);
        assert_eq!(newer[0].record.published, "2029-04-01".parse().ok());
        assert_eq!(folder(&cache), ["32448-5538_2029-04-01.laz"]);
    }

    #[test]
    fn a_tile_with_another_hash_or_size_than_the_metalink_says_is_not_kept() {
        for (size_off, hash) in [(0, "00".repeat(32)), (1, String::new())] {
            let dir = tempfile::tempdir().unwrap();
            let cache = CacheDir::new(dir.path().to_path_buf());
            let fetcher = MapFetcher::default();
            let file = laz(50);
            let hash = if hash.is_empty() { sha256(&file) } else { hash };
            fetcher.insert(META4_URL, meta4(&[(TILE, file.len() + size_off, &hash)]));
            fetcher.insert(&metadata_url(TILE), metadata("2025-03-22"));
            fetcher.insert(&tile_url(TILE), file);
            let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online);
            assert!(tiles.is_empty());
            let why = if size_off == 0 {
                "SHA-256"
            } else {
                "the index says"
            };
            assert!(
                notes.iter().any(
                    |n| n.starts_with("warning: point cloud 32448-5538 unavailable")
                        && n.contains(why)
                ),
                "{notes:?}"
            );
            assert!(folder(&cache).is_empty());
        }
    }

    #[test]
    fn a_tile_without_metadata_is_not_published_and_offline_takes_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let (tiles, notes) = fetch(&fetcher, &cache, FetchMode::Online);
        assert!(tiles.is_empty());
        assert!(
            notes
                .iter()
                .any(|n| n == "no laser point cloud published for tile 32448-5538"),
            "{notes:?}"
        );
        assert_eq!(downloads(&fetcher, ".meta4"), 0);
        let (offline, notes) = fetch(&MapFetcher::default(), &cache, FetchMode::Offline);
        assert!(offline.is_empty());
        assert!(
            notes.iter().any(|n| n.contains("no cached point cloud")),
            "{notes:?}"
        );
        serve(&fetcher, laz(10), "2025-03-22");
        fetch(&fetcher, &cache, FetchMode::Online);
        let silent = MapFetcher::default();
        let (offline, _) = fetch(&silent, &cache, FetchMode::Offline);
        assert_eq!(offline.len(), 1);
        assert_eq!(silent.calls.get(), 0, "offline asks nothing");
    }

    #[test]
    fn without_metadata_or_metalink_the_cached_tile_serves_and_a_cancel_stops() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, laz(10), "2025-03-22");
        fetch(&fetcher, &cache, FetchMode::Online);
        // The metadata service is down.
        let down = MapFetcher::default();
        down.failing.borrow_mut().push(metadata_url(TILE));
        let (tiles, notes) = fetch(&down, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1, "the cached tile");
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with("warning: point cloud 32448-5538 unavailable")),
            "{notes:?}"
        );
        // A newer scan is named, but the Metalink is down.
        let newer = MapFetcher::default();
        newer.insert(&metadata_url(TILE), metadata("2029-04-01"));
        newer.failing.borrow_mut().push(META4_URL.to_string());
        let (tiles, notes) = fetch(&newer, &cache, FetchMode::Online);
        assert_eq!(tiles.len(), 1, "the cached tile");
        assert_eq!(tiles[0].record.published, "2025-03-22".parse().ok());
        assert!(
            notes
                .iter()
                .any(|n| n.starts_with("warning: the laser point clouds are unavailable")),
            "{notes:?}"
        );
        struct Stopped;
        impl crate::fetch::Fetcher for Stopped {
            fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
                Err(FetchError::Cancelled)
            }
        }
        let registry = registry();
        let stopped = Stopped;
        let mut env = AdapterEnv {
            fetcher: &stopped,
            regions: &registry,
            cache: &cache,
            mode: FetchMode::Online,
            today: "2026-11-12".parse().unwrap(),
            skip_surface: false,
            progress: &mut |_| {},
        };
        assert!(RpLasBackend::new().fetch(&mut env, &BBOX).is_err());
    }

    #[test]
    fn the_estimate_counts_every_tile_of_the_map_box() {
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let area = EstimateArea {
            bbox_25833: BBOX,
            map_bbox: BoundingBox {
                max_e: 449_100.0,
                ..BBOX
            },
            today: "2026-11-12".parse().unwrap(),
        };
        let items = RpLasBackend::new().estimate(&area, &EstimateEnv { cache: &cache });
        let what: Vec<(&str, f64)> = items
            .iter()
            .map(|i| (i.what.as_str(), i.megabytes))
            .collect();
        assert_eq!(
            what,
            [
                ("LVermGeo RP point cloud 32448-5538", MB_PER_TILE),
                ("LVermGeo RP point cloud 32449-5538", MB_PER_TILE),
            ]
        );
    }

    #[test]
    #[ignore = "network"]
    fn live_metalink_and_metadata_name_the_mainz_tile() {
        use crate::fetch::{Fetcher as _, HttpFetcher};
        let fetcher = HttpFetcher::new();
        let files = parse_meta4(&fetcher.get(META4_URL).unwrap()).unwrap();
        assert!(files.len() > 20_000, "{}", files.len());
        assert!(files.contains_key(&TILE));
        let created = creation_date(&fetcher.get(&metadata_url(TILE)).unwrap());
        assert!(created.is_some());
    }
}
