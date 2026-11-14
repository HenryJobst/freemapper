//! The cached point-cloud tiles of a provider: one LAS/LAZ file per tile and publication,
//! `<tile>_<published>.laz`, checked to be complete before it is kept (ADR 0105).

use std::io::Read as _;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_adapter_api::env::AdapterEnv;
use sha2::{Digest as _, Sha256};

use crate::fetch::FetchError;
use crate::note::{Arg, note};
use crate::tiles::TileId;
use freemapper_adapter_api::pointcloud::PointCloudTile;

/// A tile a list says is larger is no tile of a state (the largest of Geobasis NRW has 553 MB,
/// of Rhineland-Palatinate about 1.2 GB).
const MAX_TILE_BYTES: u64 = 2 << 30;

/// The name of a km tile in zone 32 in the project file and the cache: `32356-5644`.
pub(crate) fn utm32_tile_name(tile: TileId) -> String {
    format!("32{:03}-{:04}", tile.east_km, tile.north_km)
}

/// The tile of a [`utm32_tile_name`].
pub(crate) fn parse_utm32_tile_name(name: &str) -> Option<TileId> {
    let (east, north) = name.strip_prefix("32")?.split_once('-')?;
    (east.len() == 3 && north.len() == 4).then_some(())?;
    Some(TileId {
        east_km: east.parse().ok()?,
        north_km: north.parse().ok()?,
    })
}

/// `<tile>_<published>.laz` in `dir`; `tile` is the provider's name of the tile (`33421-5821`).
pub(crate) fn las_path(dir: &Path, tile: &str, published: NaiveDate) -> PathBuf {
    dir.join(format!("{tile}_{published}.laz"))
}

/// The cached point clouds of `tile` in `dir`, newest first.
pub(crate) fn cached_las(dir: &Path, tile: &str) -> Vec<(NaiveDate, PathBuf)> {
    let prefix = format!("{tile}_");
    let mut files: Vec<(NaiveDate, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let date = name
                .strip_prefix(&prefix)?
                .strip_suffix(".laz")?
                .parse()
                .ok()?;
            Some((date, entry.path()))
        })
        .collect();
    files.sort_by_key(|(date, _)| std::cmp::Reverse(*date));
    files
}

/// Checks that the file at `path` is a complete LAS/LAZ point cloud: its header, the LAZ chunk
/// table at the end and the last point have to be readable.
pub(crate) fn check_las_file(url: &str, path: &Path) -> Result<(), FetchError> {
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message: format!("not a complete LAS point cloud: {message}"),
    };
    let mut reader = las::Reader::from_path(path).map_err(|e| corrupt(e.to_string()))?;
    let total = reader.header().number_of_points();
    if total > 0 {
        reader.seek(total - 1).map_err(|e| corrupt(e.to_string()))?;
        reader.read_points(1).map_err(|e| corrupt(e.to_string()))?;
    }
    Ok(())
}

/// The day the points of the LAS/LAZ file at `path` were measured, for a provider that names
/// the flight of a tile nowhere else: the newest day among 16 points spread evenly over the
/// file. Only a file whose header says adjusted standard GPS time tells it (GPS week time
/// names no week), and only a day between the first laser flights and `today` counts.
pub(crate) fn flight_day(path: &Path, today: NaiveDate) -> Option<NaiveDate> {
    /// Points looked at; the first and the last are among them.
    const SAMPLES: u64 = 16;
    /// A day of a GPS time further from its epoch is no day (some 270 years).
    const MAX_DAYS: f64 = 100_000.0;
    let mut reader = las::Reader::from_path(path).ok()?;
    if !reader.header().gps_time_type().is_standard() {
        return None;
    }
    let last = reader.header().number_of_points().checked_sub(1)?;
    let epoch = NaiveDate::from_ymd_opt(1980, 1, 6)?;
    let first_flights = NaiveDate::from_ymd_opt(1995, 1, 1)?;
    (0..SAMPLES)
        .filter_map(|i| {
            reader.seek(i * last / (SAMPLES - 1)).ok()?;
            let points = reader.read_points(1).ok()?;
            let time = points.points().next()?.ok()?.gps_time?;
            // Adjusted standard GPS time is the seconds since the GPS epoch minus 10⁹.
            let days = ((time + 1e9) / 86_400.0).floor();
            (0.0..MAX_DAYS)
                .contains(&days)
                .then(|| epoch.checked_add_days(chrono::Days::new(days as u64)))?
        })
        .filter(|day| (first_flights..=today).contains(day))
        .max()
}

/// The SHA-256 of the file at `path` in hex, read in pieces.
fn file_sha256(path: &Path) -> Result<String, FetchError> {
    let io = |e| FetchError::Io(path.to_path_buf(), e);
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 20];
    loop {
        let read = file.read(&mut buffer).map_err(io)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Streams the tile at `url` to a partial file next to `path`, checks that it has the size
/// `bytes` and the SHA-256 `sha256` the list of tiles gives (each when the list gives one) and
/// is a complete point cloud, then moves it to `path`. A broken file is not left behind. A
/// provider without a list of its tiles (`bytes` is `None`) has only the last of these checks.
pub(crate) fn download_las(
    env: &mut AdapterEnv<'_>,
    url: &str,
    bytes: Option<u64>,
    sha256: Option<&str>,
    path: &Path,
) -> Result<(), FetchError> {
    let limit = bytes.unwrap_or(MAX_TILE_BYTES);
    if limit > MAX_TILE_BYTES {
        return Err(FetchError::TooLarge {
            url: url.to_string(),
            limit_mb: MAX_TILE_BYTES >> 20,
        });
    }
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message,
    };
    let part = path.with_extension("laz.part");
    let _ = std::fs::remove_file(&part);
    let result = env
        .fetcher
        .download_to(url, &part, limit, env.progress)
        .and_then(|()| {
            let Some(bytes) = bytes else {
                return Ok(());
            };
            let got = std::fs::metadata(&part)
                .map_err(|e| FetchError::Io(part.clone(), e))?
                .len();
            if got == bytes {
                Ok(())
            } else {
                Err(corrupt(format!("{got} bytes, the index says {bytes}")))
            }
        })
        .and_then(|()| match sha256 {
            Some(expected) => {
                let got = file_sha256(&part)?;
                if got.eq_ignore_ascii_case(expected) {
                    Ok(())
                } else {
                    Err(corrupt(format!("SHA-256 {got}, the index says {expected}")))
                }
            }
            None => Ok(()),
        })
        .and_then(|()| check_las_file(url, &part))
        .and_then(|()| {
            std::fs::rename(&part, path).map_err(|e| FetchError::Io(path.to_path_buf(), e))
        });
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

/// Writes the LAS point cloud at `src` compressed as LAZ to `dst`, point by point with the same
/// header; for a provider that serves plain LAS (a fifth of the size in the cache).
pub(crate) fn compress_las(url: &str, src: &Path, dst: &Path) -> Result<(), FetchError> {
    /// Points read and written at a time.
    const CHUNK: u64 = 1 << 20;
    let corrupt = |message: String| FetchError::CorruptArchive {
        url: url.to_string(),
        message: format!("not a LAS point cloud: {message}"),
    };
    let io = |e: las::Error| FetchError::Io(dst.to_path_buf(), std::io::Error::other(e));
    let mut reader = las::Reader::from_path(src).map_err(|e| corrupt(e.to_string()))?;
    let mut builder = las::Builder::from(reader.header().clone());
    builder.point_format.is_compressed = true;
    let header = builder.into_header().map_err(|e| corrupt(e.to_string()))?;
    let result = (|| {
        // `Writer::from_path` would compress by the extension, and `dst` is a partial file.
        let file = std::fs::File::create(dst).map_err(|e| FetchError::Io(dst.to_path_buf(), e))?;
        let mut writer = las::Writer::new(std::io::BufWriter::new(file), header).map_err(io)?;
        loop {
            let points = reader
                .read_points(CHUNK)
                .map_err(|e| corrupt(e.to_string()))?;
            if points.is_empty() {
                break;
            }
            for point in points.points() {
                let point = point.map_err(|e| corrupt(e.to_string()))?;
                writer.write_point(point).map_err(io)?;
            }
        }
        writer.close().map_err(io)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(dst);
    }
    result
}

/// A tile that cannot be fetched: the vegetation is only a candidate, so the newest cached file
/// serves, or the tile is left out.
pub(crate) fn fallback(
    env: &mut AdapterEnv<'_>,
    name: &str,
    cached: Vec<(NaiveDate, PathBuf)>,
    error: FetchError,
    fetched: &mut Vec<PointCloudTile>,
    found: impl Fn((NaiveDate, PathBuf)) -> PointCloudTile,
) {
    match cached.into_iter().next() {
        Some(older) => {
            (env.progress)(note!(
                "warning-point-cloud-unavailable-cached",
                tile = name,
                reason = Arg::error(error)
            ));
            fetched.push(found(older));
        }
        None => (env.progress)(note!(
            "warning-point-cloud-unavailable-skipped",
            tile = name,
            reason = Arg::error(error)
        )),
    }
}

/// A tile the provider's list does not name (any more): the newest cached file serves if there is
/// one, else the tile is not published.
pub(crate) fn not_published(
    env: &mut AdapterEnv<'_>,
    name: &str,
    cached: Vec<(NaiveDate, PathBuf)>,
    fetched: &mut Vec<PointCloudTile>,
    found: impl Fn((NaiveDate, PathBuf)) -> PointCloudTile,
) {
    match cached.into_iter().next() {
        Some(newest) => {
            (env.progress)(note!("progress-point-cloud-cached", tile = name));
            fetched.push(found(newest));
        }
        None => (env.progress)(note!("progress-point-cloud-not-published", tile = name)),
    }
}

/// Point clouds for the tests of the backends.
#[cfg(test)]
pub(crate) mod testing {
    use chrono::NaiveDate;

    /// The GPS time of noon on `day`, as a file with adjusted standard GPS time stores it.
    pub(crate) fn gps_noon(day: NaiveDate) -> f64 {
        let epoch = NaiveDate::from_ymd_opt(1980, 1, 6).unwrap();
        (day - epoch).num_days() as f64 * 86_400.0 + 43_200.0 - 1e9
    }

    /// A LAZ point cloud (LAS 1.2, point format 1) in EPSG:25832 with one point per entry of
    /// `times`, its GPS time; `standard` says whether the header calls that adjusted standard
    /// GPS time or GPS week time.
    pub(crate) fn laz_with_times(times: &[f64], standard: bool) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.laz");
        let mut builder = las::Builder::from((1, 2));
        builder.point_format = las::point::Format::new(1).unwrap();
        builder.point_format.is_compressed = true;
        builder.gps_time_type = if standard {
            las::GpsTimeType::Standard
        } else {
            las::GpsTimeType::Week
        };
        let transform = |offset| las::Transform {
            scale: 0.01,
            offset,
        };
        builder.transforms = las::Vector {
            x: transform(600_000.0),
            y: transform(5_400_000.0),
            z: transform(0.0),
        };
        let mut writer = las::Writer::from_path(&path, builder.into_header().unwrap()).unwrap();
        for (i, time) in times.iter().enumerate() {
            writer
                .write_point(las::Point {
                    x: 655_350.0,
                    y: 5_479_350.0 + i as f64 * 0.2,
                    z: 350.0,
                    gps_time: Some(*time),
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        std::fs::read(path).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use freemapper_adapter_api::cache::CacheDir;
    use freemapper_adapter_api::registry::Registry;

    use super::testing::{gps_noon, laz_with_times};
    use super::*;
    use crate::fetch::FetchMode;
    use crate::fetch::test_support::MapFetcher;

    fn day(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    fn file(bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tile.laz");
        std::fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    /// 40 points, the first half measured on 2021-12-19, the second on 2021-12-20.
    fn two_days() -> Vec<f64> {
        (0..40)
            .map(|i| gps_noon(day(if i < 20 { "2021-12-19" } else { "2021-12-20" })) + f64::from(i))
            .collect()
    }

    #[test]
    fn flight_day_is_the_newest_day_of_the_points() {
        let (_dir, path) = file(&laz_with_times(&two_days(), true));
        assert_eq!(
            flight_day(&path, day("2026-11-14")),
            Some(day("2021-12-20"))
        );
        // The newest day counts wherever it lies in the file.
        let mut reversed = two_days();
        reversed.reverse();
        let (_dir, path) = file(&laz_with_times(&reversed, true));
        assert_eq!(
            flight_day(&path, day("2026-11-14")),
            Some(day("2021-12-20"))
        );
    }

    #[test]
    fn flight_day_is_unknown_with_week_time() {
        let (_dir, path) = file(&laz_with_times(&two_days(), false));
        assert_eq!(flight_day(&path, day("2026-11-14")), None);
    }

    #[test]
    fn flight_day_is_unknown_outside_the_plausible_years() {
        // GPS time 0 of a file with adjusted standard GPS time is 2011-09-14.
        let (_dir, path) = file(&laz_with_times(&[0.0; 20], true));
        assert_eq!(
            flight_day(&path, day("2026-11-14")),
            Some(day("2011-09-14"))
        );
        assert_eq!(flight_day(&path, day("2011-01-01")), None, "after today");
        // A time before the first flights (1985).
        let early = gps_noon(day("1985-06-01"));
        let (_dir, path) = file(&laz_with_times(&[early; 20], true));
        assert_eq!(flight_day(&path, day("2026-11-14")), None);
        // No points, and no point cloud at all.
        let (_dir, path) = file(&laz_with_times(&[], true));
        assert_eq!(flight_day(&path, day("2026-11-14")), None);
        let (_dir, path) = file(b"<html>not a tile</html>");
        assert_eq!(flight_day(&path, day("2026-11-14")), None);
    }

    #[test]
    fn a_download_without_a_listed_size_is_checked_for_completeness() {
        const URL: &str = "https://example.test/tile.laz";
        let dir = tempfile::tempdir().unwrap();
        let cache = CacheDir::new(dir.path().to_path_buf());
        let registry = Registry::new();
        let fetcher = MapFetcher::default();
        let path = dir.path().join("las/tile_2023-03-10.laz");
        let mut download = |bytes: Vec<u8>| {
            fetcher.insert(URL, bytes);
            let mut env = AdapterEnv {
                fetcher: &fetcher,
                regions: &registry,
                cache: &cache,
                mode: FetchMode::Online,
                today: day("2026-11-14"),
                skip_surface: false,
                progress: &mut |_| {},
            };
            download_las(&mut env, URL, None, None, &path)
        };
        let broken = download(b"<html>not a tile</html>".to_vec());
        assert!(
            matches!(broken, Err(FetchError::CorruptArchive { .. })),
            "{broken:?}"
        );
        let left: Vec<_> = std::fs::read_dir(dir.path().join("las"))
            .map(|entries| entries.flatten().map(|e| e.file_name()).collect())
            .unwrap_or_default();
        assert!(left.is_empty(), "{left:?}");
        download(laz_with_times(&two_days(), true)).unwrap();
        assert!(path.exists());
        assert!(!path.with_extension("laz.part").exists());
    }
}
