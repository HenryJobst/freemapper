//! Permit sources that are shapefiles in a zip for a whole state (ADR 0131), and the reading
//! they share with the forest sources of North Rhine-Westphalia (ADR 0124): the zip is
//! downloaded (50 MB at most), its `.shp` and `.dbf` read, the records whose box meets the map
//! box kept and turned into GeoJSON features as longitude, latitude with only the fields their
//! reading needs. A table without a `.cpg` in Latin-1 keeps its umlauts (the shapefile reader
//! falls back to it).
//!
//! The engine hands these sources the shared download cache (ADR 0132): a zip is kept there,
//! named by the date the server gives for it, and downloaded again only when the server has a
//! newer one. A server that cannot be asked leaves the cached copy in use.

use std::io::Read as _;
use std::path::Path;

use freemapper_adapter_api::permits::{
    PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
};
use freemapper_sources::shapefile::{ShapeRecord, polygons_in};
use geo_types::Rect;
use serde_json::Value;

use crate::crs::{BoundingBox, Crs, LatLon, ProjectedPoint};
use crate::fetch::{CacheDir, FetchError, Fetcher, get_limited_with_retries, write_cache_file};
use crate::wfs_permits::{Reading, finding, stored};

/// The largest zip read; the largest one used is NRW's water protection areas (15 MB).
const MAX_ZIP: u64 = 50 * 1024 * 1024;

/// The largest file read out of the zip; the largest is the table of NRW's Staatswald (18 MB).
const MAX_ENTRY: u64 = 100 * 1024 * 1024;

/// How far around the map box records are kept, in metres: a record whose box only nearly
/// meets it costs nothing, one cut off would be missing.
const MARGIN_M: f64 = 50.0;

/// The records of the zip at `url` (one shapefile in `crs`) whose box meets the map box `bbox`
/// (WGS 84, x = longitude). A download that is no such zip is an error with the URL.
pub(crate) fn shape_records(
    fetcher: &dyn Fetcher,
    url: &str,
    crs: Crs,
    bbox: &Rect<f64>,
    part: &Path,
) -> Result<Vec<ShapeRecord>, FetchError> {
    let cut = map_box(bbox, crs);
    get_limited_with_retries(fetcher, url, part, MAX_ZIP, |bytes| {
        records_in(bytes, &cut).map_err(|message| FetchError::Http {
            url: url.to_string(),
            message,
        })
    })
}

/// [`shape_records`] with the zip kept in `cache`.
pub(crate) fn cached_shape_records(
    fetcher: &dyn Fetcher,
    url: &str,
    crs: Crs,
    bbox: &Rect<f64>,
    part: &Path,
    cache: &CacheDir,
) -> Result<Vec<ShapeRecord>, FetchError> {
    let bytes = current_zip(fetcher, url, part, &cache.provider_dir(CACHE_FOLDER))?;
    records_in(&bytes, &map_box(bbox, crs)).map_err(|message| FetchError::Http {
        url: url.to_string(),
        message,
    })
}

/// The cache folder of these files.
const CACHE_FOLDER: &str = "permits";

/// How long a copy the server gives no date for is used before it is fetched again.
const UNDATED_FOR: std::time::Duration = std::time::Duration::from_secs(7 * 24 * 3600);

/// The bytes of the zip at `url`, from the copy in `dir` when the server has no newer one.
/// The copies are named `<host>-<file>-<YYYYMMDD>.zip` by the server's date, or `…-undated`
/// when it gives none; a new download replaces the older copies.
fn current_zip(
    fetcher: &dyn Fetcher,
    url: &str,
    part: &Path,
    dir: &Path,
) -> Result<Vec<u8>, FetchError> {
    let stem = cache_stem(url);
    let copies = cached_copies(dir, &stem);
    let read = |path: &Path| std::fs::read(path).map_err(|e| FetchError::Io(path.to_path_buf(), e));
    let name = match fetcher.last_modified(url) {
        Ok(Some(changed)) => format!("{stem}-{}.zip", changed.format("%Y%m%d")),
        Ok(None) => {
            let undated = dir.join(format!("{stem}-undated.zip"));
            if crate::clock::age_of(&undated).is_some_and(|age| age < UNDATED_FOR) {
                return read(&undated);
            }
            format!("{stem}-undated.zip")
        }
        // The server cannot be asked for the date: a copy, however old, is better than none;
        // without one the download is tried all the same, as some servers answer no HEAD.
        Err(_) => match copies.last() {
            Some(path) => return read(path),
            None => format!("{stem}-undated.zip"),
        },
    };
    let path = dir.join(&name);
    if path.exists() {
        return read(&path);
    }
    let fresh = get_limited_with_retries(fetcher, url, part, MAX_ZIP, |bytes| {
        zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .map(|_| bytes.to_vec())
            .map_err(|e| FetchError::Http {
                url: url.to_string(),
                message: format!("not a zip: {e}"),
            })
    });
    match fresh {
        Ok(bytes) => {
            write_cache_file(&path, &bytes)?;
            for older in copies.iter().filter(|c| **c != path) {
                let _ = std::fs::remove_file(older);
            }
            Ok(bytes)
        }
        Err(e) => copies.last().map_or(Err(e), |path| read(path)),
    }
}

/// The name of the copies of `url`: its host and file name, with only letters, digits, `-`
/// and `_` (anything else becomes `_`), so that nothing of the URL becomes a path.
fn cache_stem(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = rest.split('/').next().unwrap_or_default();
    let file = rest.rsplit('/').next().unwrap_or_default();
    let file = file.strip_suffix(".zip").unwrap_or(file);
    format!("{host}-{file}")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// The cached copies of `stem` in `dir`, oldest first (by the date in their name; an undated
/// one last).
fn cached_copies(dir: &Path, stem: &str) -> Vec<std::path::PathBuf> {
    let prefix = format!("{stem}-");
    let mut copies: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".zip"))
        })
        .collect();
    copies.sort();
    copies
}

/// The map box (WGS 84, x = longitude) in `crs`, with [`MARGIN_M`] around it.
fn map_box(bbox: &Rect<f64>, crs: Crs) -> BoundingBox {
    let corners = [
        (bbox.min().y, bbox.min().x),
        (bbox.min().y, bbox.max().x),
        (bbox.max().y, bbox.min().x),
        (bbox.max().y, bbox.max().x),
    ]
    .map(|(lat, lon)| crs.forward(LatLon { lat, lon }));
    let fold = |pick: fn(f64, f64) -> f64, start: f64, axis: fn(&ProjectedPoint) -> f64| {
        corners.iter().map(axis).fold(start, pick)
    };
    BoundingBox {
        min_e: fold(f64::min, f64::INFINITY, |p| p.easting) - MARGIN_M,
        min_n: fold(f64::min, f64::INFINITY, |p| p.northing) - MARGIN_M,
        max_e: fold(f64::max, f64::NEG_INFINITY, |p| p.easting) + MARGIN_M,
        max_n: fold(f64::max, f64::NEG_INFINITY, |p| p.northing) + MARGIN_M,
    }
}

/// The records of the one `.shp` and `.dbf` in the zip `bytes` whose box meets `cut`.
fn records_in(bytes: &[u8], cut: &BoundingBox) -> Result<Vec<ShapeRecord>, String> {
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("not a zip: {e}"))?;
    let mut entry = |extension: &str| -> Result<Vec<u8>, String> {
        let index = (0..zip.len())
            .find(|&i| {
                zip.by_index_raw(i).is_ok_and(|f| {
                    f.name()
                        .is_ok_and(|n| n.to_ascii_lowercase().ends_with(extension))
                })
            })
            .ok_or_else(|| format!("the zip has no {extension} file"))?;
        let mut content = Vec::new();
        // A file larger than any of the states' is cut off and then refused below, so a
        // packed bomb never fills the memory.
        zip.by_index(index)
            .map_err(|e| e.to_string())?
            .take(MAX_ENTRY + 1)
            .read_to_end(&mut content)
            .map_err(|e| e.to_string())?;
        if content.len() as u64 > MAX_ENTRY {
            return Err(format!(
                "the {extension} file is larger than {MAX_ENTRY} bytes"
            ));
        }
        Ok(content)
    };
    let shp = entry(".shp")?;
    let dbf = entry(".dbf")?;
    polygons_in(&shp, &dbf, cut)
}

/// A record as a GeoJSON feature with the fields of `kept` and its polygons in `crs` as
/// longitude, latitude; `None` when a position cannot be converted.
pub(crate) fn record_feature(record: &ShapeRecord, kept: &[&str], crs: Crs) -> Option<Value> {
    let properties: serde_json::Map<String, Value> = kept
        .iter()
        .filter_map(|key| {
            let value = record.fields.get(*key)?;
            Some((key.to_string(), Value::String(value.clone())))
        })
        .collect();
    let mut polygons = Vec::with_capacity(record.polygons.len());
    for polygon in &record.polygons {
        let mut rings = Vec::with_capacity(polygon.len());
        for ring in polygon {
            let mut positions = Vec::with_capacity(ring.len());
            for &[easting, northing] in ring {
                let p = crs.inverse(ProjectedPoint { easting, northing })?;
                positions.push(serde_json::json!([p.lon, p.lat]));
            }
            rings.push(Value::Array(positions));
        }
        polygons.push(Value::Array(rings));
    }
    Some(serde_json::json!({
        "type": "Feature",
        "geometry": { "type": "MultiPolygon", "coordinates": polygons },
        "properties": properties,
    }))
}

/// A shapefile of a state read by one [`Reading`], as the table sources of WFS are.
pub struct ShapeTableSource {
    info: PermitSourceInfo,
    url: &'static str,
    crs: Crs,
    reading: Reading,
}

/// A source of the zip at `url` (one shapefile in `crs`), its one layer `layer`.
#[allow(clippy::too_many_arguments)]
pub fn source(
    key: &'static str,
    label: Wording,
    what: Wording,
    credit: &'static str,
    layer: &'static [&'static str],
    sections: &'static [PermitSection],
    url: &'static str,
    crs: Crs,
    reading: Reading,
) -> Box<dyn PermitSource> {
    debug_assert_eq!(layer.len(), 1);
    Box::new(ShapeTableSource {
        info: PermitSourceInfo {
            key,
            label,
            what,
            credit,
            note: None,
            sampled: false,
            layers: layer,
            sections,
            offices: None,
        },
        url,
        crs,
        reading,
    })
}

impl PermitSource for ShapeTableSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let records = shape_records(fetcher, self.url, self.crs, bbox, part)?;
        Ok(vec![self.features(&records)])
    }

    fn fetch_cached(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
        cache: &CacheDir,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let records = cached_shape_records(fetcher, self.url, self.crs, bbox, part, cache)?;
        Ok(vec![self.features(&records)])
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        if layer != self.info.layers[0] {
            return Vec::new();
        }
        features
            .iter()
            .filter_map(|f| finding(self.reading, f))
            .collect()
    }
}

impl ShapeTableSource {
    /// The records as stored features.
    fn features(&self, records: &[ShapeRecord]) -> Vec<Value> {
        let kept = self.reading.kept();
        records
            .iter()
            .filter_map(|r| record_feature(r, &kept, self.crs))
            .filter_map(|f| stored(&f, self.reading))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;
    use freemapper_adapter_api::permits::FindingKind;
    use freemapper_sources::shapefile::test_support::{Entry, shape_files};
    use geo_types::Coord;
    use std::io::Write as _;

    /// A zip of the files of `records`.
    fn zip_of(records: &[Entry<'_>]) -> Vec<u8> {
        let (shp, dbf) = shape_files(records);
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, bytes) in [("wsg.shp", shp), ("wsg.dbf", dbf)] {
            zip.start_file(name, options).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    /// A square of 200 m at the south-west corner `(e, n)`.
    fn square(e: f64, n: f64) -> Vec<[f64; 2]> {
        vec![
            [e, n],
            [e, n + 200.0],
            [e + 200.0, n + 200.0],
            [e + 200.0, n],
            [e, n],
        ]
    }

    /// The box around 1 km of `crs` at `(e, n)` in WGS 84.
    fn around(crs: Crs, e: f64, n: f64) -> Rect<f64> {
        let corner = |e, n| {
            let p = crs
                .inverse(ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap();
            Coord { x: p.lon, y: p.lat }
        };
        Rect::new(corner(e - 500.0, n - 500.0), corner(e + 500.0, n + 500.0))
    }

    fn fetch(source: &dyn PermitSource, url: &str, zip: Vec<u8>, bbox: &Rect<f64>) -> Vec<Value> {
        let fetcher = MapFetcher::default();
        fetcher.insert(url, zip);
        let part = tempfile::tempdir().unwrap();
        let mut layers = source
            .fetch(&fetcher, bbox, &part.path().join("x.part"))
            .unwrap();
        assert_eq!(layers.len(), 1);
        layers.remove(0)
    }

    /// Brandenburg's zones keep their name, zone and ordinance, nothing else; a zone far from
    /// the map is not kept.
    #[test]
    fn brandenburg_water_zones_from_the_shapefile() {
        let near = [
            ("KLARNAME", "Beispielhausen"),
            ("SCHUTZZONE", "Zone II"),
            ("FUNDSTELLE", "GVBl.II/08, [Nr. 1], S.1"),
            ("DURCH", "Ministerium"),
            ("LANDKREIS", "Musterkreis"),
        ];
        let far = [("KLARNAME", "Fernab"), ("SCHUTZZONE", "Zone III")];
        let zip = zip_of(&[
            (&near, &[square(390_000.0, 5_800_000.0)]),
            (&far, &[square(450_000.0, 5_900_000.0)]),
        ]);
        let source = crate::lfb_permits::sources()
            .into_iter()
            .find(|s| s.info().key == "wasserschutzgebiete")
            .unwrap();
        let features = fetch(
            source.as_ref(),
            "https://data.geobasis-bb.de/geofachdaten/Wasser/Gewaesserbewirtschaftung/wsg.zip",
            zip,
            &around(Crs::Utm33, 390_100.0, 5_800_100.0),
        );
        assert_eq!(features.len(), 1, "{features:?}");
        let keys: Vec<&String> = features[0]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .collect();
        assert_eq!(keys, ["FUNDSTELLE", "KLARNAME", "SCHUTZZONE"]);
        assert_eq!(
            source.findings("wasserschutzgebiete", &features)[0].kind,
            FindingKind::WaterProtection {
                name: Some("Beispielhausen".into()),
                zone: Some("Zone II".into()),
                citation: Some("GVBl.II/08, [Nr. 1], S.1".into()),
                authority: None,
            }
        );
    }

    /// NRW's table is Latin-1 without a `.cpg`; a planned zone says so.
    #[test]
    fn nrw_water_zones_keep_umlauts_and_mark_planned_ones() {
        let set = [
            ("name", "D\u{fc}sterm\u{fc}hle"),
            ("wsg_zone", "1"),
            ("areastatus", "F"),
        ];
        let planned = [
            ("name", "Genkeltalsperre"),
            ("wsg_zone", "3B"),
            ("areastatus", "P"),
        ];
        let mut zip = zip_of(&[
            (&set, &[square(365_000.0, 5_617_000.0)]),
            (&planned, &[square(365_300.0, 5_617_000.0)]),
        ]);
        // The test tables are UTF-8; write the umlauts as the state does, one Latin-1 byte.
        let utf8 = "D\u{fc}sterm\u{fc}hle".as_bytes().to_vec();
        let latin1: Vec<u8> = "D\u{fc}sterm\u{fc}hle".chars().map(|c| c as u8).collect();
        zip = rezip_replacing(&zip, &utf8, &latin1);
        let source = crate::wuh_permits::sources()
            .into_iter()
            .find(|s| s.info().key == "wasserschutzgebiete")
            .unwrap();
        let features = fetch(
            source.as_ref(),
            "https://www.opengeodata.nrw.de/produkte/umwelt_klima/wasser/trinkwasser/wasserschutzgebiete/Wasserschutzgebiete-NRW_EPSG25832_Shape.zip",
            zip,
            &around(Crs::Utm32, 365_200.0, 5_617_100.0),
        );
        let kinds: Vec<FindingKind> = source
            .findings("wasserschutzgebiete", &features)
            .into_iter()
            .map(|f| f.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                FindingKind::WaterProtection {
                    name: Some("D\u{fc}sterm\u{fc}hle".into()),
                    zone: Some("Schutzzone 1".into()),
                    citation: None,
                    authority: None,
                },
                FindingKind::WaterProtection {
                    name: Some("Genkeltalsperre".into()),
                    zone: Some("Schutzzone 3B (geplant)".into()),
                    citation: None,
                    authority: None,
                },
            ]
        );
    }

    /// The zip with `from` replaced by `to` in its `.dbf` (padded with spaces to the width).
    fn rezip_replacing(zip: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
        let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let name = file.name().unwrap().to_string();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if name.ends_with(".dbf") {
                let at = bytes.windows(from.len()).position(|w| w == from).unwrap();
                let mut padded = to.to_vec();
                padded.resize(from.len(), b' ');
                bytes[at..at + from.len()].copy_from_slice(&padded);
            }
            out.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            out.write_all(&bytes).unwrap();
        }
        out.finish().unwrap().into_inner()
    }

    /// The cache (ADR 0132): the first fetch downloads and keeps the zip by the server's
    /// date; the same date asks only for the date; a newer one downloads and drops the older
    /// copy; a server that cannot be asked leaves the copy in use.
    #[test]
    fn statewide_zips_are_kept_in_the_cache() {
        let url =
            "https://data.geobasis-bb.de/geofachdaten/Wasser/Gewaesserbewirtschaftung/wsg.zip";
        let zone = [("KLARNAME", "Beispielhausen"), ("SCHUTZZONE", "Zone III")];
        let zip = zip_of(&[(&zone, &[square(390_000.0, 5_800_000.0)])]);
        let source = crate::lfb_permits::sources()
            .into_iter()
            .find(|s| s.info().key == "wasserschutzgebiete")
            .unwrap();
        let root = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(root.path().to_path_buf());
        let part = root.path().join("x.part");
        let bbox = around(Crs::Utm33, 390_100.0, 5_800_100.0);
        let day = |d| chrono::NaiveDate::from_ymd_opt(2026, 10, d).unwrap();
        let copies = || {
            let mut names: Vec<String> = std::fs::read_dir(cache.provider_dir("permits"))
                .unwrap()
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        };
        let fetcher = MapFetcher::default();
        fetcher.insert(url, zip.clone());
        fetcher.set_modified(url, day(5));
        let first = source.fetch_cached(&fetcher, &bbox, &part, &cache).unwrap();
        assert_eq!(first[0].len(), 1);
        assert_eq!(copies(), ["data_geobasis-bb_de-wsg-20261005.zip"]);
        let asked = fetcher.calls.get();

        // The same date: only the date is asked.
        let again = source.fetch_cached(&fetcher, &bbox, &part, &cache).unwrap();
        assert_eq!(again, first);
        assert_eq!(fetcher.calls.get(), asked + 1);

        // A newer file: downloaded, the older copy goes.
        fetcher.set_modified(url, day(12));
        source.fetch_cached(&fetcher, &bbox, &part, &cache).unwrap();
        assert_eq!(copies(), ["data_geobasis-bb_de-wsg-20261012.zip"]);

        // The server down: the copy is used.
        let down = MapFetcher::default();
        down.failing.borrow_mut().push(url.to_string());
        let cached = source.fetch_cached(&down, &bbox, &part, &cache).unwrap();
        assert_eq!(cached, first);

        // Without a copy, a server that is down is an error.
        let empty = tempfile::tempdir().unwrap();
        let nothing = CacheDir::new(empty.path().to_path_buf());
        assert!(source.fetch_cached(&down, &bbox, &part, &nothing).is_err());
    }

    /// A server that gives no date: the undated copy serves for a week.
    #[test]
    fn an_undated_zip_is_kept_for_a_week() {
        let url =
            "https://data.geobasis-bb.de/geofachdaten/Wasser/Gewaesserbewirtschaftung/wsg.zip";
        let zone = [("KLARNAME", "Beispielhausen"), ("SCHUTZZONE", "Zone III")];
        let fetcher = MapFetcher::default();
        fetcher.insert(url, zip_of(&[(&zone, &[square(390_000.0, 5_800_000.0)])]));
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("permits");
        let part = root.path().join("x.part");
        current_zip(&fetcher, url, &part, &dir).unwrap();
        let asked = fetcher.calls.get();
        current_zip(&fetcher, url, &part, &dir).unwrap();
        assert_eq!(fetcher.calls.get(), asked + 1, "only the date is asked");
        assert!(dir.join("data_geobasis-bb_de-wsg-undated.zip").exists());
    }

    #[test]
    fn a_cache_name_is_made_of_plain_characters() {
        assert_eq!(
            cache_stem(
                "https://www.opengeodata.nrw.de/a/b/Wasserschutzgebiete-NRW_EPSG25832_Shape.zip"
            ),
            "www_opengeodata_nrw_de-Wasserschutzgebiete-NRW_EPSG25832_Shape"
        );
        assert_eq!(cache_stem("https://x.de/../../etc/passwd"), "x_de-passwd");
    }

    #[test]
    fn a_download_that_is_no_zip_is_an_error() {
        let fetcher = MapFetcher::default();
        let url =
            "https://data.geobasis-bb.de/geofachdaten/Wasser/Gewaesserbewirtschaftung/wsg.zip";
        fetcher.insert(url, b"<html>moved</html>".to_vec());
        let part = tempfile::tempdir().unwrap();
        let err = shape_records(
            &fetcher,
            url,
            Crs::Utm33,
            &around(Crs::Utm33, 390_100.0, 5_800_100.0),
            &part.path().join("x.part"),
        )
        .unwrap_err();
        assert!(
            matches!(&err, FetchError::Http { url: u, .. } if u == url),
            "{err:?}"
        );
    }
}
