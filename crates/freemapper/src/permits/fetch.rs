//! Fetching the inputs of the permit notes: the BfN protected areas and the VG250 units through
//! the policy of the national sources, the OSM findings from the extract already cached. What
//! was fetched is stored in the project (`data/permits/`) and read back from there, so a later
//! offline run gives the same notes.

use std::collections::HashSet;
use std::path::Path;

use freemapper_adapter_api::policy::{PolicyFetcher, UrlPolicy};
use freemapper_sources::bfn::{BFN, ProtectedKind};
use freemapper_sources::geojson_wfs::{GeoJsonWfs, LatLonBox, get_features};
use freemapper_sources::vg250::{AdminLevel, VG250};
use geo::Intersects as _;
use serde_json::Value;

use crate::area::MapArea;
use crate::clock;
use crate::fetch::{Cache, Fetcher};
use crate::note::{Arg, Note, note};
use crate::permits::notes::{Inputs, SourceInput};
use crate::permits::osm::{OsmFinding, findings, wants};
use crate::permits::store::{
    PERMITS_DIR, RawInputs, load_inputs, something_fetched, store_inputs, stored_box,
};
use crate::permits::{Missing, PermitsError};
use crate::project::ProjectFile;
use crate::sources::osm::{cached_extracts, extracts_for, read_extract};

/// The file each WFS answer streams into, in [`PERMITS_DIR`]; removed afterwards.
const PART: &str = "wfs.part";

/// The inputs of the permit notes of `project` in `dir`.
///
/// Online, VG250 and the BfN are asked for the box of the map area through
/// [`UrlPolicy::national`], and the OSM findings are read from the newest cached extract of
/// each region the map needs (never downloaded here: without one, OSM is not read). Everything
/// is stored in [`PERMITS_DIR`], replacing what an earlier fetch stored; a source that fails is
/// `Unreachable` and keeps none of its earlier data. When no source gives anything, nothing is
/// stored and the earlier data stay. Offline, the stored inputs are read.
///
/// [`PermitsError::NothingRead`] when no source gave anything.
pub fn fetch_inputs(
    project: &ProjectFile,
    dir: &Path,
    fetcher: &dyn Fetcher,
    cache: &Cache,
    offline: bool,
    progress: &mut dyn FnMut(Note),
) -> Result<Inputs, PermitsError> {
    let area = project.area()?;
    let bbox = map_box(&area);
    if offline {
        progress(note!("progress-permits-stored"));
        if let Some(stored) = stored_box(dir)?
            && !covers(&stored, &bbox)
        {
            return Err(PermitsError::NothingRead {
                missing: ["vg250", "bfn", "osm"]
                    .map(|source| Missing {
                        source: source.to_string(),
                        reason: Some("stored for another map area".to_string()),
                    })
                    .into(),
            });
        }
        return load_inputs(dir);
    }
    let today = clock::today();
    let folder = dir.join(PERMITS_DIR);
    freemapper_fs::check_folder_inside(dir, &folder).map_err(|(p, e)| PermitsError::Io(p, e))?;
    std::fs::create_dir_all(&folder).map_err(|e| PermitsError::Io(folder.clone(), e))?;
    let part = folder.join(PART);
    let policy = UrlPolicy::national();
    let national = PolicyFetcher::new(fetcher, &policy);
    let mut ask = |label: &str, service: &GeoJsonWfs, types: Vec<&'static str>| {
        progress(note!("progress-permits-fetch", source = label));
        let answer = fetch_source(&national, service, &types, &bbox, &part);
        freemapper_fs::remove_inside(dir, &part);
        match answer {
            Ok(features) => SourceInput::Read {
                items: features,
                fetched: today,
            },
            Err(e) => {
                let reason = e.to_string();
                progress(note!(
                    "warning-permits-source-unreachable",
                    source = label,
                    error = Arg::error(e)
                ));
                SourceInput::Unreachable { reason }
            }
        }
    };
    let admin = keyed(
        ask(
            "VG250",
            &VG250,
            AdminLevel::ALL.map(AdminLevel::type_name).into(),
        ),
        &AdminLevel::ALL,
    );
    let protected = keyed(
        ask(
            "BfN",
            &BFN,
            ProtectedKind::ALL.map(ProtectedKind::type_name).into(),
        ),
        &ProtectedKind::ALL,
    );
    let osm = osm_findings(project, &area, cache, progress)?;
    let raw = RawInputs {
        bbox,
        admin,
        protected,
        osm,
    };
    // When nothing was read, the data of an earlier fetch stay for `--offline`.
    something_fetched(&raw)?;
    store_inputs(dir, &raw).map_err(|e| PermitsError::Io(folder, e))?;
    load_inputs(dir)
}

/// The features of every type of `service`, one list per type in the order of `types`. Stops
/// at the first type that fails: a source that is down is not asked eight more times.
fn fetch_source(
    fetcher: &dyn Fetcher,
    service: &GeoJsonWfs,
    types: &[&'static str],
    bbox: &LatLonBox,
    part: &Path,
) -> Result<Vec<Vec<Value>>, crate::fetch::FetchError> {
    types
        .iter()
        .map(|type_name| get_features(fetcher, service, type_name, bbox, part))
        .collect()
}

/// The answers of a source paired with the levels or kinds they were asked for.
fn keyed<K: Copy>(input: SourceInput<Vec<Value>>, keys: &[K]) -> SourceInput<(K, Vec<Value>)> {
    match input {
        SourceInput::Read { items, fetched } => SourceInput::Read {
            items: keys.iter().copied().zip(items).collect(),
            fetched,
        },
        SourceInput::Unreachable { reason } => SourceInput::Unreachable { reason },
        SourceInput::NotRead => SourceInput::NotRead,
    }
}

/// The OSM findings on the map area from the newest cached extract of every region the map
/// needs, each object once. Not read when one of the extracts is not cached; `Unreachable`
/// when one cannot be read. Dated with the oldest data date of the extracts.
fn osm_findings(
    project: &ProjectFile,
    area: &MapArea,
    cache: &Cache,
    progress: &mut dyn FnMut(Note),
) -> Result<SourceInput<OsmFinding>, PermitsError> {
    let mut extracts = Vec::new();
    for source in extracts_for(&project.region_bbox()?, progress) {
        match cached_extracts(cache, &source).into_iter().next() {
            Some(extract) => extracts.push((source, extract)),
            None => {
                progress(note!("warning-permits-osm-no-extract"));
                return Ok(SourceInput::NotRead);
            }
        }
    }
    let Some(fetched) = extracts.iter().map(|(_, e)| e.data_date).min() else {
        progress(note!("warning-permits-osm-no-extract"));
        return Ok(SourceInput::NotRead);
    };
    let geo_box = area.geo_box(0.0);
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for (source, extract) in &extracts {
        progress(note!(
            "progress-permits-osm",
            region = source.label(),
            date = extract.data_date.to_string()
        ));
        let features = match read_extract(&extract.path, &geo_box, &wants) {
            Ok(features) => features,
            Err(e) => {
                let reason = e.to_string();
                progress(note!(
                    "warning-permits-source-unreachable",
                    source = "OpenStreetMap",
                    error = Arg::error(e)
                ));
                return Ok(SourceInput::Unreachable { reason });
            }
        };
        for finding in findings(features) {
            if finding.geometry.intersects(&area.polygon) && seen.insert(finding.osm_id.clone()) {
                found.push(finding);
            }
        }
    }
    Ok(SourceInput::Read {
        items: found,
        fetched,
    })
}

/// The box of the map area in WGS 84.
fn map_box(area: &MapArea) -> LatLonBox {
    let b = area.geo_box(0.0);
    LatLonBox {
        min_lat: b.min_lat,
        min_lon: b.min_lon,
        max_lat: b.max_lat,
        max_lon: b.max_lon,
    }
}

/// Whether `outer` contains `inner`.
fn covers(outer: &LatLonBox, inner: &LatLonBox) -> bool {
    outer.min_lat <= inner.min_lat
        && outer.min_lon <= inner.min_lon
        && outer.max_lat >= inner.max_lat
        && outer.max_lon >= inner.max_lon
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::{MapFetcher, PbfNode, PbfWay, osm_pbf_with};
    use crate::permits::store::{PERMITS_JSON, write_permit_notes};
    use crate::project::{MapKind, init_project};
    use chrono::NaiveDate;
    use freemapper_sources::bfn::fetch_bfn;
    use freemapper_sources::vg250::fetch_vg250;
    use tempfile::TempDir;

    const FIXTURES: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../freemapper-sources/tests/fixtures/permits/"
    );

    /// A map area inside Gemeinde Potsdam and the LSG of the trimmed fixtures:
    /// min lon, min lat, max lon, max lat.
    const AREA: [f64; 4] = [12.985, 52.383, 12.995, 52.388];

    fn date(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(format!("{FIXTURES}{name}")).unwrap()
    }

    fn empty_collection() -> Vec<u8> {
        br#"{"type":"FeatureCollection","features":[]}"#.to_vec()
    }

    fn project() -> (TempDir, ProjectFile) {
        let dir = tempfile::tempdir().unwrap();
        let [w, s, e, n] = AREA;
        let geojson = format!(
            r#"{{"type":"Polygon","coordinates":[[[{w},{s}],[{e},{s}],[{e},{n}],[{w},{n}],[{w},{s}]]]}}"#
        );
        let area = MapArea::from_geojson(&geojson).unwrap();
        let project = init_project(
            dir.path(),
            &area,
            MapKind::Sprint,
            None,
            date("2026-11-13"),
            None,
        )
        .unwrap();
        (dir, project)
    }

    /// Serves the fixtures by type name and every other kind and level empty. The URLs are the
    /// ones the readers ask, learned from a fetcher that answers everything empty.
    fn serve_sources(fetcher: &MapFetcher, project: &ProjectFile) {
        let probe = MapFetcher::default();
        probe.insert_prefix("https://", empty_collection());
        let scratch = tempfile::tempdir().unwrap();
        let part = scratch.path().join("probe.part");
        let bbox = map_box(&project.area().unwrap());
        fetch_vg250(&probe, &bbox, &part);
        fetch_bfn(&probe, &bbox, &part);
        for url in probe.log.borrow().iter() {
            let body = if url.contains("Landschaftsschutzgebiete") {
                fixture("bfn-lsg-sanssouci.geojson")
            } else if url.contains("vg250_gem") {
                fixture("vg250-gem-sanssouci.geojson")
            } else if url.contains("vg250_krs") {
                fixture("vg250-krs-sanssouci.geojson")
            } else {
                empty_collection()
            };
            fetcher.insert(url, body);
        }
    }

    /// A park run by an operator and a nature reserve, both closed ways on the map area, and a
    /// school that is only a node.
    fn extract() -> Vec<u8> {
        let nodes: &[PbfNode<'_>] = &[
            (1, 52.384, 12.986, &[]),
            (2, 52.384, 12.989, &[]),
            (3, 52.386, 12.989, &[]),
            (4, 52.386, 12.986, &[]),
            (5, 52.3845, 12.990, &[]),
            (6, 52.3845, 12.994, &[]),
            (7, 52.3875, 12.994, &[]),
            (8, 52.3875, 12.990, &[]),
            (
                9,
                52.385,
                12.988,
                &[("amenity", "school"), ("operator", "Stadt Potsdam")],
            ),
        ];
        let ways: &[PbfWay<'_>] = &[
            (
                10,
                &[1, 2, 3, 4, 1],
                &[
                    ("leisure", "park"),
                    ("name", "Park"),
                    ("operator", "Stiftung Preußische Schlösser und Gärten"),
                ],
            ),
            (
                11,
                &[5, 6, 7, 8, 5],
                &[
                    ("leisure", "nature_reserve"),
                    ("name", "Reservat"),
                    ("website", "https://example.org"),
                ],
            ),
        ];
        osm_pbf_with(nodes, ways, &[])
    }

    fn cache_with_extract() -> (TempDir, Cache) {
        let (root, cache) = empty_cache();
        std::fs::create_dir_all(cache.osm_dir()).unwrap();
        std::fs::write(
            cache.osm_dir().join("brandenburg_2026-11-06.osm.pbf"),
            extract(),
        )
        .unwrap();
        (root, cache)
    }

    fn empty_cache() -> (TempDir, Cache) {
        let root = tempfile::tempdir().unwrap();
        let cache = Cache::new(root.path().to_path_buf());
        (root, cache)
    }

    fn fetch(
        project: &ProjectFile,
        dir: &Path,
        fetcher: &dyn Fetcher,
        cache: &Cache,
        offline: bool,
    ) -> (Result<Inputs, PermitsError>, Vec<&'static str>) {
        let mut ids = Vec::new();
        let result = fetch_inputs(project, dir, fetcher, cache, offline, &mut |n: Note| {
            ids.push(n.id())
        });
        (result, ids)
    }

    fn is_read<T>(input: &SourceInput<T>) -> bool {
        matches!(input, SourceInput::Read { .. })
    }

    fn stored_files(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir.join(PERMITS_DIR))
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn fetch_then_offline_gives_the_same_notes() {
        let (dir, project) = project();
        let fetcher = MapFetcher::default();
        serve_sources(&fetcher, &project);
        let (_root, cache) = cache_with_extract();
        let (online, _) = fetch(&project, dir.path(), &fetcher, &cache, false);
        let online = online.unwrap();
        assert!(is_read(&online.admin) && is_read(&online.protected) && is_read(&online.osm));
        let today = date("2026-11-13");
        let notes = write_permit_notes(dir.path(), &project, &online, today).unwrap();
        assert!(dir.path().join(PERMITS_JSON).exists());
        assert!(
            notes
                .admin
                .iter()
                .any(|s| s.item.name == "Potsdam" && s.item.level == AdminLevel::Gemeinde),
            "{:?}",
            notes.admin
        );
        assert!(
            notes
                .protected
                .iter()
                .any(|s| s.item.name == "Potsdamer Havelseengebiet" && s.percent > 0.0),
            "{:?}",
            notes.protected
        );
        assert!(
            !stored_files(dir.path())
                .iter()
                .any(|n| n.ends_with(".part"))
        );

        let calls = fetcher.calls.get();
        let (offline, _) = fetch(&project, dir.path(), &fetcher, &cache, true);
        let offline = offline.unwrap();
        assert_eq!(fetcher.calls.get(), calls, "offline asks no server");
        assert_eq!(offline, online);
        assert_eq!(
            write_permit_notes(dir.path(), &project, &offline, today).unwrap(),
            notes
        );
    }

    #[test]
    fn offline_without_stored_data_is_nothing_read() {
        let (dir, project) = project();
        let fetcher = MapFetcher::default();
        let (_root, cache) = cache_with_extract();
        let (result, _) = fetch(&project, dir.path(), &fetcher, &cache, true);
        match result {
            Err(PermitsError::NothingRead { missing }) => {
                let sources: Vec<&str> = missing.iter().map(|m| m.source.as_str()).collect();
                assert_eq!(sources, ["vg250", "bfn", "osm"]);
                assert!(missing.iter().all(|m| m.reason.is_none()), "{missing:?}");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(fetcher.calls.get(), 0);
    }

    #[test]
    fn offline_data_of_another_map_area_is_not_used() {
        let (dir, project) = project();
        let fetcher = MapFetcher::default();
        serve_sources(&fetcher, &project);
        let (_root, cache) = cache_with_extract();
        fetch(&project, dir.path(), &fetcher, &cache, false)
            .0
            .unwrap();
        // A second project east of the first, with the stored data of the first.
        let other = tempfile::tempdir().unwrap();
        let [w, s, e, n] = AREA;
        let shift = 0.02;
        let geojson = format!(
            r#"{{"type":"Polygon","coordinates":[[[{w2},{s}],[{e2},{s}],[{e2},{n}],[{w2},{n}],[{w2},{s}]]]}}"#,
            w2 = w + shift,
            e2 = e + shift
        );
        let area = MapArea::from_geojson(&geojson).unwrap();
        let moved = init_project(
            other.path(),
            &area,
            MapKind::Sprint,
            None,
            date("2026-11-13"),
            None,
        )
        .unwrap();
        let to = other.path().join(PERMITS_DIR);
        std::fs::create_dir_all(&to).unwrap();
        for name in stored_files(dir.path()) {
            std::fs::copy(dir.path().join(PERMITS_DIR).join(&name), to.join(&name)).unwrap();
        }
        let (result, _) = fetch(&moved, other.path(), &fetcher, &cache, true);
        assert!(
            matches!(result, Err(PermitsError::NothingRead { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn a_down_bfn_leaves_vg250_and_osm() {
        let (dir, project) = project();
        let (_root, cache) = cache_with_extract();
        let up = MapFetcher::default();
        serve_sources(&up, &project);
        fetch(&project, dir.path(), &up, &cache, false).0.unwrap();
        assert!(
            stored_files(dir.path())
                .iter()
                .any(|n| n.starts_with("bfn-"))
        );

        let down = MapFetcher::default();
        serve_sources(&down, &project);
        down.refusing
            .borrow_mut()
            .push("https://geodienste.bfn.de".to_string());
        let (inputs, ids) = fetch(&project, dir.path(), &down, &cache, false);
        let inputs = inputs.unwrap();
        assert!(matches!(inputs.protected, SourceInput::Unreachable { .. }));
        assert!(is_read(&inputs.admin) && is_read(&inputs.osm));
        assert_eq!(
            ids.iter()
                .filter(|id| **id == "warning-permits-source-unreachable")
                .count(),
            1
        );
        let bfn_requests = down
            .log
            .borrow()
            .iter()
            .filter(|u| u.contains("bfn.de"))
            .count();
        assert_eq!(bfn_requests, 1, "a source that is down is asked once");
        assert!(
            !stored_files(dir.path())
                .iter()
                .any(|n| n.starts_with("bfn-")),
            "the earlier BfN data is not mixed with the new date: {:?}",
            stored_files(dir.path())
        );
        let (offline, _) = fetch(&project, dir.path(), &down, &cache, true);
        assert_eq!(offline.unwrap(), inputs);
    }

    /// The bytes of every file in the project's permit folder and of `permits.json`.
    fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut files: Vec<(String, Vec<u8>)> = stored_files(dir)
            .into_iter()
            .map(|name| {
                let bytes = std::fs::read(dir.join(PERMITS_DIR).join(&name)).unwrap();
                (name, bytes)
            })
            .collect();
        files.push((
            PERMITS_JSON.to_string(),
            std::fs::read(dir.join(PERMITS_JSON)).unwrap(),
        ));
        files
    }

    /// An online run where no source gives anything (both services down, no extract) leaves
    /// what an earlier run stored, so `--offline` still works and the notes stay as they were.
    #[test]
    fn an_online_run_that_reads_nothing_keeps_the_stored_data() {
        let (dir, project) = project();
        let up = MapFetcher::default();
        serve_sources(&up, &project);
        let (_root, cache) = cache_with_extract();
        let earlier = fetch(&project, dir.path(), &up, &cache, false).0.unwrap();
        write_permit_notes(dir.path(), &project, &earlier, date("2026-11-13")).unwrap();
        let before = snapshot(dir.path());

        let down = MapFetcher::default();
        down.refusing
            .borrow_mut()
            .extend(UrlPolicy::NATIONAL_HOSTS.map(|h| format!("https://{h}")));
        let (_empty_root, empty) = empty_cache();
        let (result, _) = fetch(&project, dir.path(), &down, &empty, false);
        match result {
            Err(PermitsError::NothingRead { missing }) => {
                let sources: Vec<&str> = missing.iter().map(|m| m.source.as_str()).collect();
                assert_eq!(sources, ["vg250", "bfn", "osm"]);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(snapshot(dir.path()), before);
        let (offline, _) = fetch(&project, dir.path(), &down, &empty, true);
        assert_eq!(offline.unwrap(), earlier);
    }

    #[test]
    fn no_cached_extract_means_osm_not_read() {
        let (dir, project) = project();
        let fetcher = MapFetcher::default();
        serve_sources(&fetcher, &project);
        let (_root, cache) = empty_cache();
        let (inputs, ids) = fetch(&project, dir.path(), &fetcher, &cache, false);
        let inputs = inputs.unwrap();
        assert_eq!(inputs.osm, SourceInput::NotRead);
        assert!(is_read(&inputs.admin) && is_read(&inputs.protected));
        assert!(ids.contains(&"warning-permits-osm-no-extract"), "{ids:?}");
        assert!(
            !fetcher.log.borrow().iter().any(|u| u.contains("geofabrik")),
            "no extract is downloaded"
        );
        assert!(!cache.osm_dir().exists());
    }

    #[test]
    fn requests_outside_the_national_hosts_are_refused() {
        let inner = MapFetcher::default();
        inner.insert_prefix("https://", empty_collection());
        let policy = UrlPolicy::national();
        let national = PolicyFetcher::new(&inner, &policy);
        assert!(national.get("https://example.org/wfs").is_err());
        assert_eq!(inner.calls.get(), 0, "a refused request does not go out");
        assert!(
            national
                .get("https://sgx.geodatenzentrum.de/wfs_vg250?x")
                .is_ok()
        );

        let (dir, project) = project();
        let (_root, cache) = cache_with_extract();
        fetch(&project, dir.path(), &inner, &cache, false)
            .0
            .unwrap();
        let log = inner.log.borrow();
        assert!(log.len() > 13, "{log:?}");
        for url in log.iter() {
            assert!(
                UrlPolicy::NATIONAL_HOSTS
                    .iter()
                    .any(|host| url.starts_with(&format!("https://{host}/"))),
                "{url}"
            );
        }
    }

    #[test]
    fn osm_findings_are_stored_for_offline_use() {
        let (dir, project) = project();
        let fetcher = MapFetcher::default();
        serve_sources(&fetcher, &project);
        let (_root, cache) = cache_with_extract();
        let online = fetch(&project, dir.path(), &fetcher, &cache, false)
            .0
            .unwrap();
        let SourceInput::Read { items, fetched } = &online.osm else {
            panic!("{:?}", online.osm);
        };
        let ids: Vec<&str> = items.iter().map(|f| f.osm_id.as_str()).collect();
        assert_eq!(ids, ["way/10", "way/11"], "the school node is no area");
        assert_eq!(*fetched, date("2026-11-06"));
        assert!(items.iter().all(|f| !f.geometry.0.is_empty()));
        let stored =
            std::fs::read_to_string(dir.path().join(PERMITS_DIR).join("osm.geojson")).unwrap();
        assert!(!stored.contains("website") && !stored.contains("example.org"));

        // The extract leaves the cache; the stored findings stay.
        std::fs::remove_dir_all(cache.osm_dir()).unwrap();
        let offline = fetch(&project, dir.path(), &fetcher, &cache, true)
            .0
            .unwrap();
        assert_eq!(offline.osm, online.osm);
    }
}
