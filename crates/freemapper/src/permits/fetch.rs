//! Fetching the inputs of the permit notes: the BfN protected areas and the VG250 units through
//! the policy of the national sources, the OSM findings from the extract already cached. What
//! was fetched is stored in the project (`data/permits/`) and read back from there, so a later
//! offline run gives the same notes.
//!
//! The sources of a region adapter (ADR 0120) are asked only when the map area lies on the
//! adapter's ground, and through [`crate::adapters::asking`], so the adapter's own hosts hold
//! for them and the policy of the national sources does not.

use std::collections::HashSet;
use std::path::Path;

use freemapper_adapter_api::permits::plain_key;
use freemapper_adapter_api::policy::{PolicyFetcher, UrlPolicy};
use freemapper_sources::bfn::{BFN, ProtectedKind};
use freemapper_sources::geojson_wfs::{GeoJsonWfs, LatLonBox, get_features};
use freemapper_sources::vg250::{AdminLevel, VG250};
use geo::Intersects as _;
use serde_json::Value;

use crate::adapters::{RegionAdapter, Registry};
use crate::area::MapArea;
use crate::clock;
use crate::fetch::{Cache, Fetcher};
use crate::note::{Arg, Note, note};
use crate::permits::notes::{Inputs, SourceInput};
use crate::permits::osm::{OsmFinding, findings, wants};
use crate::permits::regional::RegionalSource;
use crate::permits::store::{
    PERMITS_DIR, RawInputs, RawRegional, area_key, load_inputs_in, something_fetched,
    store_inputs_in, stored_for,
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
/// stored and the earlier data stay. Offline, the stored inputs are read, but only when they
/// were fetched for exactly this map area polygon.
///
/// The sources of the region adapters of the registry whose ground the map area touches are
/// asked behind the hosts of their adapter, and stored and read back the same way.
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
    fetch_inputs_in(
        crate::adapters::registry(),
        project,
        dir,
        fetcher,
        cache,
        offline,
        progress,
    )
}

/// [`fetch_inputs`] with the region adapters of `registry`.
pub fn fetch_inputs_in(
    registry: &Registry,
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
        // Stored for another polygon (also one inside the old box): the OSM findings were
        // filtered against the old one. A record without a key falls back to the box.
        if let Some(stored) = stored_for(dir)?
            && (!covers(&stored.bbox, &bbox)
                || stored.area.is_some_and(|key| key != area_key(&area)))
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
        return load_inputs_in(registry, dir);
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
    let regional = regional_answers(registry, &area, &bbox, dir, fetcher, &part, progress);
    let raw = RawInputs {
        bbox,
        area: area_key(&area),
        admin,
        protected,
        osm,
        regional,
    };
    // When nothing was read, the data of an earlier fetch stay for `--offline`.
    something_fetched(&raw)?;
    store_inputs_in(registry, dir, &raw).map_err(|e| PermitsError::Io(folder, e))?;
    load_inputs_in(registry, dir)
}

/// Whether the map area polygon lies on the ground of `adapter` (its boundary, without the
/// margin the downloads have): only then are its permit sources asked.
fn on_ground(adapter: &dyn RegionAdapter, area: &MapArea) -> bool {
    use geo::Intersects as _;
    let projected: geo_types::LineString<f64> = area
        .polygon
        .exterior()
        .0
        .iter()
        .map(|c| {
            let p = crate::crs::Crs::Utm33.forward(crate::crs::LatLon { lat: c.y, lon: c.x });
            geo_types::coord! { x: p.easting, y: p.northing }
        })
        .collect();
    let polygon = geo_types::Polygon::new(projected, vec![]);
    let bounds = adapter.bounds();
    let rect = geo_types::Rect::new(
        geo_types::coord! { x: bounds.min_e, y: bounds.min_n },
        geo_types::coord! { x: bounds.max_e, y: bounds.max_n },
    );
    polygon.intersects(&rect) && adapter.boundary().intersects(&polygon)
}

/// The answers of the permit sources of every adapter of `registry` whose ground the map area
/// touches, each asked through [`crate::adapters::asking`]. A source that fails is
/// `Unreachable`; the others are asked all the same.
fn regional_answers(
    registry: &Registry,
    area: &MapArea,
    bbox: &LatLonBox,
    dir: &Path,
    fetcher: &dyn Fetcher,
    part: &Path,
    progress: &mut dyn FnMut(Note),
) -> Vec<RawRegional> {
    let today = clock::today();
    let rect = geo_types::Rect::new(
        geo_types::coord! { x: bbox.min_lon, y: bbox.min_lat },
        geo_types::coord! { x: bbox.max_lon, y: bbox.max_lat },
    );
    let mut answers = Vec::new();
    for adapter in registry
        .adapters()
        .filter(|a| !a.permit_sources().is_empty() && on_ground(*a, area))
    {
        for source in adapter.permit_sources() {
            let info = source.info();
            // The keys become file names.
            if !plain_key(info.key) || !info.layers.iter().all(|l| plain_key(l)) {
                continue;
            }
            // The publisher and what the source holds, in the language of the run.
            let label = || Arg::Words {
                en: format!("{}: {}", info.label.en, info.what.en),
                de: format!("{}: {}", info.label.de, info.what.de),
            };
            progress(note!("progress-permits-fetch", source = label()));
            let answer = crate::adapters::asking(adapter, fetcher, |fetcher| {
                source.fetch(fetcher, &rect, part)
            });
            freemapper_fs::remove_inside(dir, part);
            let layers = match answer {
                Ok(lists) if lists.len() == info.layers.len() => SourceInput::Read {
                    items: info
                        .layers
                        .iter()
                        .map(|l| l.to_string())
                        .zip(lists)
                        .collect(),
                    fetched: today,
                },
                Ok(lists) => SourceInput::Unreachable {
                    reason: format!(
                        "the source gave {} layers instead of {}",
                        lists.len(),
                        info.layers.len()
                    ),
                },
                Err(e) => {
                    let reason = e.to_string();
                    progress(note!(
                        "warning-permits-source-unreachable",
                        source = label(),
                        error = Arg::error(e)
                    ));
                    SourceInput::Unreachable { reason }
                }
            };
            answers.push(RawRegional {
                adapter: adapter.info().id.as_str().to_string(),
                source: RegionalSource::from(info),
                layers,
            });
        }
    }
    answers
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
    use serde_json::Value;
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

    /// The service of Brandenburg's forest offices, which answers GML (ADR 0124).
    const BRANDENBURG_FOREST: &str = "https://www.brandenburg-forst.de/";

    fn empty_gml_collection() -> Vec<u8> {
        br#"<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs/2.0"/>"#.to_vec()
    }

    /// The ids of the adapters that brought regional inputs, each once.
    fn regional_adapters(inputs: &Inputs) -> Vec<&str> {
        let mut ids: Vec<&str> = inputs.regional.iter().map(|r| r.adapter.as_str()).collect();
        ids.dedup();
        ids
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
        // The map area lies in Brandenburg, whose forest offices answer GML (ADR 0124).
        fetcher.insert_prefix(BRANDENBURG_FOREST, empty_gml_collection());
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

    /// The files of the project's permit folder, as paths relative to it: the national sources
    /// at the top, a regional adapter's in its folder (`brandenburg/…`).
    fn stored_files(dir: &Path) -> Vec<String> {
        fn walk(root: &Path, at: &Path, names: &mut Vec<String>) {
            for entry in std::fs::read_dir(at).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(root, &path, names);
                } else {
                    let relative = path.strip_prefix(root).unwrap();
                    names.push(relative.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        let root = dir.join(PERMITS_DIR);
        let mut names = Vec::new();
        walk(&root, &root, &mut names);
        names.sort();
        names
    }

    /// Copies the permit folder of `from` into the project folder `to`.
    fn copy_stored(from: &Path, to: &Path) {
        for name in stored_files(from) {
            let target = to.join(PERMITS_DIR).join(&name);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(from.join(PERMITS_DIR).join(&name), target).unwrap();
        }
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
        copy_stored(dir.path(), other.path());
        let (result, _) = fetch(&moved, other.path(), &fetcher, &cache, true);
        assert!(
            matches!(result, Err(PermitsError::NothingRead { .. })),
            "{result:?}"
        );
    }

    /// A changed area inside the stored box: the stored OSM findings were filtered against the
    /// old polygon, so the stored data do not fit (fix round 1).
    #[test]
    fn offline_data_of_a_changed_area_inside_the_box_is_not_used() {
        let (dir, project) = project();
        let fetcher = MapFetcher::default();
        serve_sources(&fetcher, &project);
        let (_root, cache) = cache_with_extract();
        fetch(&project, dir.path(), &fetcher, &cache, false)
            .0
            .unwrap();
        // A triangle inside the box of the first area.
        let other = tempfile::tempdir().unwrap();
        let [w, s, e, n] = AREA;
        let geojson = format!(
            r#"{{"type":"Polygon","coordinates":[[[{w},{s}],[{e},{s}],[{w},{n}],[{w},{s}]]]}}"#
        );
        let area = MapArea::from_geojson(&geojson).unwrap();
        let changed = init_project(
            other.path(),
            &area,
            MapKind::Sprint,
            None,
            date("2026-11-13"),
            None,
        )
        .unwrap();
        copy_stored(dir.path(), other.path());
        let (result, _) = fetch(&changed, other.path(), &fetcher, &cache, true);
        match result {
            Err(PermitsError::NothingRead { missing }) => assert!(
                missing
                    .iter()
                    .all(|m| m.reason.as_deref() == Some("stored for another map area")),
                "{missing:?}"
            ),
            other => panic!("{other:?}"),
        }
        // The same area in another folder still reads them.
        let same = tempfile::tempdir().unwrap();
        let copy = init_project(
            same.path(),
            &project.area().unwrap(),
            MapKind::Sprint,
            None,
            date("2026-11-13"),
            None,
        )
        .unwrap();
        copy_stored(dir.path(), same.path());
        let (result, _) = fetch(&copy, same.path(), &fetcher, &cache, true);
        assert!(result.is_ok(), "{result:?}");
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
        // Every request goes to a national host, or to the forest service of Brandenburg, on
        // whose ground Potsdam lies, behind the hosts of Brandenburg's adapter (ADR 0124).
        let brandenburg = registry().get("brandenburg").expect("built in");
        let mut regional = std::collections::BTreeSet::new();
        for url in log.iter() {
            let national = UrlPolicy::NATIONAL_HOSTS
                .iter()
                .any(|host| url.starts_with(&format!("https://{host}/")));
            if !national {
                assert!(url.starts_with(BRANDENBURG_FOREST), "{url}");
                assert!(
                    brandenburg.policy().unwrap().check_url(url).is_ok(),
                    "{url}"
                );
                regional.insert(url);
            }
        }
        // One query for the Reviere, tried again because an empty JSON is no GML.
        assert_eq!(regional.len(), 1, "{log:?}");
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

    // The sources of the region adapters (ADR 0120).

    use crate::adapters::registry;
    use crate::permits::regional::RegionalInput;
    use freemapper_adapter_api::permits::{
        FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, Wording,
    };

    const BERLIN_FIXTURES: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/permits-berlin/"
    );

    fn berlin() -> &'static dyn RegionAdapter {
        registry().get("berlin").expect("built in")
    }

    /// A sprint map over the box of the live probe at the Viktoriapark, Berlin
    /// (389613,5816190 to 390529,5816909 in EPSG:25833).
    fn viktoriapark() -> (TempDir, ProjectFile) {
        project_in_25833(389_613.0, 5_816_190.0, 390_529.0, 5_816_909.0)
    }

    /// A sprint map over a box in EPSG:25833: min easting, min northing, max easting, max
    /// northing.
    fn project_in_25833(min_e: f64, min_n: f64, max_e: f64, max_n: f64) -> (TempDir, ProjectFile) {
        let dir = tempfile::tempdir().unwrap();
        let corner = |e: f64, n: f64| {
            let p = crate::crs::Crs::Utm33
                .inverse(crate::crs::ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap();
            [p.lon, p.lat]
        };
        let ring = [
            corner(min_e, min_n),
            corner(max_e, min_n),
            corner(max_e, max_n),
            corner(min_e, max_n),
            corner(min_e, min_n),
        ];
        let coordinates: Vec<String> = ring.iter().map(|[x, y]| format!("[{x},{y}]")).collect();
        let geojson = format!(
            r#"{{"type":"Polygon","coordinates":[[{}]]}}"#,
            coordinates.join(",")
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

    /// The trimmed answers of the Geoportal Berlin by WFS type; the other Berlin types and the
    /// national services answer empty.
    fn serve_berlin(fetcher: &MapFetcher, project: &ProjectFile) {
        let probe = MapFetcher::default();
        probe.insert_prefix("https://", empty_collection());
        let scratch = tempfile::tempdir().unwrap();
        let part = scratch.path().join("probe.part");
        let b = map_box(&project.area().unwrap());
        let rect = geo_types::Rect::new(
            geo_types::coord! { x: b.min_lon, y: b.min_lat },
            geo_types::coord! { x: b.max_lon, y: b.max_lat },
        );
        for source in berlin().permit_sources() {
            source.fetch(&probe, &rect, &part).unwrap();
        }
        let by_type = [
            ("alkis_bezirke:bezirksgrenzen", "bezirksgrenzen.geojson"),
            ("gruenanlagen:gruenanlagen", "gruenanlagen.geojson"),
            ("gruenanlagen:spielplaetze", "spielplaetze.geojson"),
            (
                "liegenschaftsplan:liegenschaftsplan",
                "liegenschaftsplan.geojson",
            ),
            ("denkmale:denkmale", "denkmale.geojson"),
            ("schutzgebiete:a_nd", "naturdenkmale.geojson"),
            ("forstverwalt:b_forstverwalt", "forstverwalt.geojson"),
        ];
        for url in probe.log.borrow().iter() {
            let body = by_type
                .iter()
                .find(|(t, _)| url.contains(&format!("typeNames={t}&")))
                .map(|(_, file)| std::fs::read(format!("{BERLIN_FIXTURES}{file}")).unwrap())
                .unwrap_or_else(empty_collection);
            fetcher.insert(url, body);
        }
        for host in UrlPolicy::NATIONAL_HOSTS {
            fetcher.insert_prefix(&format!("https://{host}/"), empty_collection());
        }
    }

    /// The fixture file the fetcher of [`serve_berlin`] answers for a layer of Berlin's sources;
    /// the other layers answer empty.
    const BERLIN_LAYER_FILES: [(&str, &str); 7] = [
        ("bezirksgrenzen", "bezirksgrenzen.geojson"),
        ("gruenanlagen", "gruenanlagen.geojson"),
        ("spielplaetze", "spielplaetze.geojson"),
        ("liegenschaftsplan", "liegenschaftsplan.geojson"),
        ("denkmale", "denkmale.geojson"),
        ("a-nd", "naturdenkmale.geojson"),
        ("forstverwalt", "forstverwalt.geojson"),
    ];

    /// The findings of `source` as the adapter parses the fixture files themselves, without
    /// the fetch and the store in between.
    fn fixture_findings(source: &dyn PermitSource) -> Vec<PermitFinding> {
        source
            .info()
            .layers
            .iter()
            .flat_map(|layer| {
                let features: Vec<Value> = BERLIN_LAYER_FILES
                    .iter()
                    .find(|(l, _)| l == layer)
                    .map(|(_, file)| {
                        let text =
                            std::fs::read_to_string(format!("{BERLIN_FIXTURES}{file}")).unwrap();
                        let mut value: Value = serde_json::from_str(&text).unwrap();
                        match value["features"].take() {
                            Value::Array(features) => features,
                            other => panic!("{file}: {other:?}"),
                        }
                    })
                    .unwrap_or_default();
                source.findings(layer, &features)
            })
            .collect()
    }

    /// Berlin's regional inputs as the adapter parses the fixtures, read on `fetched`.
    /// Runs `f` on 2026-11-13, the day the tests below expect a fetch to be dated with.
    fn on_the_day_of_the_fixtures<R>(f: impl FnOnce() -> R) -> R {
        let at = date("2026-11-13").and_hms_opt(12, 0, 0).unwrap();
        clock::with_fixed(at, f)
    }

    fn berlin_from_fixtures(fetched: NaiveDate) -> Vec<RegionalInput> {
        berlin()
            .permit_sources()
            .iter()
            .map(|source| RegionalInput {
                adapter: "berlin".into(),
                source: RegionalSource::from(source.info()),
                findings: SourceInput::Read {
                    items: fixture_findings(source.as_ref()),
                    fetched,
                },
            })
            .collect()
    }

    fn regional_state(inputs: &Inputs) -> Vec<(&str, bool)> {
        inputs
            .regional
            .iter()
            .map(|r| (r.source.key.as_str(), is_read(&r.findings)))
            .collect()
    }

    #[test]
    fn a_map_outside_berlin_asks_no_berlin_service() {
        let (dir, project) = project();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", empty_collection());
        let (_root, cache) = cache_with_extract();
        let inputs = fetch(&project, dir.path(), &fetcher, &cache, false)
            .0
            .unwrap();
        // Potsdam lies in Brandenburg alone, whose forest offices are its one regional source.
        assert_eq!(regional_adapters(&inputs), ["brandenburg"]);
        let log = fetcher.log.borrow();
        assert!(!log.is_empty());
        assert!(log.iter().all(|u| !u.contains("berlin.de")), "{log:#?}");
        assert!(!dir.path().join(PERMITS_DIR).join("berlin").exists());
    }

    /// A map area 100 to 300 m west of Berlin's westernmost point lies off Berlin's ground and
    /// does not touch it: no Berlin service is asked, as there is no margin (review finding 4).
    #[test]
    fn a_map_just_outside_berlin_asks_no_berlin_service() {
        let west = geo::CoordsIter::coords_iter(berlin().boundary())
            .min_by(|a, b| a.x.total_cmp(&b.x))
            .unwrap();
        let (dir, project) = project_in_25833(
            west.x - 300.0,
            west.y - 100.0,
            west.x - 100.0,
            west.y + 100.0,
        );
        assert!(!on_ground(berlin(), &project.area().unwrap()));
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", empty_collection());
        let (_root, cache) = empty_cache();
        let inputs = fetch(&project, dir.path(), &fetcher, &cache, false)
            .0
            .unwrap();
        assert_eq!(regional_adapters(&inputs), ["brandenburg"]);
        let log = fetcher.log.borrow();
        assert!(!log.is_empty());
        assert!(log.iter().all(|u| !u.contains("berlin.de")), "{log:#?}");

        // 100 m further east the area reaches into Berlin, and Berlin is asked.
        let (_dir, inside) = project_in_25833(
            west.x - 200.0,
            west.y - 100.0,
            west.x + 100.0,
            west.y + 100.0,
        );
        assert!(on_ground(berlin(), &inside.area().unwrap()));
    }

    /// With the national services down and no OSM extract, Berlin's sources alone are enough:
    /// the notes are written, online and offline (review finding 10).
    #[test]
    fn berlin_sources_alone_write_the_notes() {
        let (dir, project) = viktoriapark();
        let fetcher = MapFetcher::default();
        serve_berlin(&fetcher, &project);
        fetcher
            .refusing
            .borrow_mut()
            .extend(UrlPolicy::NATIONAL_HOSTS.map(|h| format!("https://{h}")));
        let (_root, cache) = empty_cache();
        let inputs = fetch(&project, dir.path(), &fetcher, &cache, false)
            .0
            .unwrap();
        assert!(matches!(inputs.admin, SourceInput::Unreachable { .. }));
        assert!(matches!(inputs.protected, SourceInput::Unreachable { .. }));
        assert_eq!(inputs.osm, SourceInput::NotRead);
        assert!(inputs.regional.iter().all(|r| is_read(&r.findings)));
        let today = date("2026-11-13");
        let notes = write_permit_notes(dir.path(), &project, &inputs, today).unwrap();
        assert!(dir.path().join(PERMITS_JSON).exists());
        assert!(notes.regional[0].districts.is_some());
        let (offline, _) = fetch(&project, dir.path(), &fetcher, &cache, true);
        assert_eq!(offline.unwrap(), inputs);
    }

    /// A permit source that asks one address and gives nothing.
    struct Asking {
        info: PermitSourceInfo,
        url: &'static str,
    }

    impl PermitSource for Asking {
        fn info(&self) -> &PermitSourceInfo {
            &self.info
        }

        fn fetch(
            &self,
            fetcher: &dyn crate::fetch::Fetcher,
            _bbox: &geo_types::Rect<f64>,
            _part: &Path,
        ) -> Result<Vec<Vec<Value>>, crate::fetch::FetchError> {
            fetcher.get(self.url)?;
            Ok(vec![Vec::new()])
        }

        fn findings(&self, _layer: &str, _features: &[Value]) -> Vec<PermitFinding> {
            Vec::new()
        }
    }

    fn asking(key: &'static str, url: &'static str) -> Box<dyn PermitSource> {
        Box::new(Asking {
            info: PermitSourceInfo {
                key,
                label: Wording {
                    en: "Own",
                    de: "Eigen",
                },
                what: Wording { en: key, de: key },
                credit: "open",
                note: None,
                layers: &["all"],
                sections: &[PermitSection::Districts],
                offices: None,
            },
            url,
        })
    }

    #[test]
    fn berlin_requests_go_through_the_berlin_policy() {
        // Berlin's own sources are read, though their host is no national one: they are asked
        // behind Berlin's hosts.
        let (dir, project) = viktoriapark();
        let fetcher = MapFetcher::default();
        serve_berlin(&fetcher, &project);
        let (_root, cache) = empty_cache();
        let inputs = fetch(&project, dir.path(), &fetcher, &cache, false)
            .0
            .unwrap();
        assert_eq!(
            regional_state(&inputs),
            [
                ("bezirke", true),
                ("gruenanlagen", true),
                ("liegenschaftsplan", true),
                ("denkmale", true),
                ("schutzgebiete", true),
                ("forstverwalt", true),
            ]
        );
        let policy = berlin().policy().unwrap();
        let log = fetcher.log.borrow();
        let berlin_urls: Vec<&String> = log.iter().filter(|u| u.contains("berlin.de")).collect();
        assert_eq!(berlin_urls.len(), 10, "{berlin_urls:#?}");
        for url in &berlin_urls {
            assert!(
                url.starts_with("https://gdi.berlin.de/services/wfs/"),
                "{url}"
            );
            assert!(policy.check_url(url).is_ok(), "{url}");
            assert!(UrlPolicy::national().check_url(url).is_err(), "{url}");
        }
        drop(log);

        // A source of an adapter is held to that adapter's hosts: a request off them is not
        // sent, and the national hosts do not count for it.
        let (dir, project) = self::project();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix("https://", empty_collection());
        let registry = crate::adapters::test_support::square_with_permits(
            &["own.example"],
            vec![
                asking("own", "https://own.example/wfs"),
                asking("other", "https://other.example/wfs"),
                asking("national", "https://geodienste.bfn.de/ogc/wfs/schutzgebiet"),
            ],
        );
        let (_root, cache) = empty_cache();
        let inputs = fetch_inputs_in(
            &registry,
            &project,
            dir.path(),
            &fetcher,
            &cache,
            false,
            &mut |_| {},
        )
        .unwrap();
        let states: Vec<(&str, &str)> = inputs
            .regional
            .iter()
            .map(|r| {
                let state = match &r.findings {
                    SourceInput::Read { .. } => "read",
                    SourceInput::Unreachable { .. } => "refused",
                    other => panic!("{other:?}"),
                };
                (r.source.key.as_str(), state)
            })
            .collect();
        assert_eq!(
            states,
            [
                ("own", "read"),
                ("other", "refused"),
                ("national", "refused")
            ]
        );
        let log = fetcher.log.borrow();
        assert!(log.iter().any(|u| u == "https://own.example/wfs"));
        assert!(!log.iter().any(|u| u.contains("other.example")), "{log:#?}");
        assert!(
            !log.iter()
                .any(|u| u == "https://geodienste.bfn.de/ogc/wfs/schutzgebiet"),
            "{log:#?}"
        );
    }

    #[test]
    fn a_down_berlin_service_keeps_the_rest() {
        let (dir, project) = viktoriapark();
        let fetcher = MapFetcher::default();
        serve_berlin(&fetcher, &project);
        fetcher
            .refusing
            .borrow_mut()
            .push("https://gdi.berlin.de/services/wfs/denkmale".to_string());
        let (_root, cache) = empty_cache();
        let (inputs, ids) = fetch(&project, dir.path(), &fetcher, &cache, false);
        let inputs = inputs.unwrap();
        assert_eq!(
            regional_state(&inputs),
            [
                ("bezirke", true),
                ("gruenanlagen", true),
                ("liegenschaftsplan", true),
                ("denkmale", false),
                ("schutzgebiete", true),
                ("forstverwalt", true),
            ]
        );
        assert!(is_read(&inputs.admin) && is_read(&inputs.protected));
        assert_eq!(
            ids.iter()
                .filter(|id| **id == "warning-permits-source-unreachable")
                .count(),
            1,
            "{ids:?}"
        );
        let asked = fetcher
            .log
            .borrow()
            .iter()
            .filter(|u| u.contains("/wfs/denkmale"))
            .count();
        assert_eq!(asked, 1);
        let notes = write_permit_notes(dir.path(), &project, &inputs, date("2026-11-13")).unwrap();
        let part = &notes.regional[0];
        assert!(part.monuments.is_none(), "the monuments section drops");
        assert!(part.districts.is_some() && part.owners.is_some());
        assert_eq!(part.sources[3].status.state, "unreachable");
        // The Bezirk is still known, so its offices are named.
        assert!(
            notes
                .hints
                .iter()
                .any(|h| h.key == crate::permits::notes::hint_key::DISTRICT_OFFICE)
        );
        let stored = dir.path().join(PERMITS_DIR).join("berlin");
        assert!(stored.join("bezirke-bezirksgrenzen.geojson").exists());
        assert!(!stored.join("denkmale-denkmale.geojson").exists());
        let (offline, _) = fetch(&project, dir.path(), &fetcher, &cache, true);
        assert_eq!(offline.unwrap(), inputs);
    }

    /// The progress and warning lines name a source of an adapter in the language of the run
    /// (review finding 7).
    #[test]
    fn regional_sources_are_named_in_the_language_of_the_run() {
        let (dir, project) = viktoriapark();
        let fetcher = MapFetcher::default();
        serve_berlin(&fetcher, &project);
        fetcher
            .refusing
            .borrow_mut()
            .push("https://gdi.berlin.de/services/wfs/denkmale".to_string());
        let (_root, cache) = empty_cache();
        let mut notes: Vec<Note> = Vec::new();
        fetch_inputs(&project, dir.path(), &fetcher, &cache, false, &mut |n| {
            notes.push(n)
        })
        .unwrap();
        let de = crate::note::NOTES.parse_lang_arg("de").unwrap();
        let german: Vec<String> = notes
            .iter()
            .map(|n| n.text(&de, &|e, _| e.to_string()))
            .collect();
        assert!(
            german
                .iter()
                .any(|l| l.contains("Geoportal Berlin: Bezirksgrenzen (ALKIS Berlin)")),
            "{german:#?}"
        );
        assert!(
            german.iter().any(|l| l.starts_with("Warnung: ")
                && l.contains("Geoportal Berlin: Denkmale (Denkmalkarte)")),
            "{german:#?}"
        );
        assert!(
            !german.iter().any(|l| l.contains("Bezirk borders")),
            "{german:#?}"
        );
        let english: Vec<String> = notes.iter().map(ToString::to_string).collect();
        assert!(
            english
                .iter()
                .any(|l| l.contains("Geoportal Berlin: Bezirk borders (ALKIS Berlin)")),
            "{english:#?}"
        );
    }

    #[test]
    fn offline_berlin_notes_match_online() {
        on_the_day_of_the_fixtures(|| {
            let (dir, project) = viktoriapark();
            let fetcher = MapFetcher::default();
            serve_berlin(&fetcher, &project);
            let (_root, cache) = empty_cache();
            let online = fetch(&project, dir.path(), &fetcher, &cache, false)
                .0
                .unwrap();
            let today = date("2026-11-13");
            // The notes as the adapter's own parse of the served files gives them, not read back
            // from the store: the fetch and the store must not change a finding.
            let direct = Inputs {
                regional: berlin_from_fixtures(today),
                ..online.clone()
            };
            assert_eq!(online.regional, direct.regional);
            let notes = write_permit_notes(dir.path(), &project, &direct, today).unwrap();
            let part = &notes.regional[0];
            assert_eq!(part.adapter, "berlin");
            let districts: Vec<&str> = part
                .districts
                .as_ref()
                .unwrap()
                .iter()
                .map(|s| s.item.name.as_str())
                .collect();
            assert_eq!(districts[0], "Friedrichshain-Kreuzberg", "{districts:?}");
            assert!(districts.contains(&"Tempelhof-Schöneberg"), "{districts:?}");
            let owners: Vec<Option<&str>> = part
                .owners
                .as_ref()
                .unwrap()
                .iter()
                .map(|s| s.item.owner.as_deref())
                .collect();
            assert_eq!(
                owners,
                [Some("Land Berlin"), Some("Bundesrepublik Deutschland")]
            );
            let monuments = part.monuments.as_ref().unwrap();
            assert!(monuments.total >= 40, "{}", monuments.total);
            assert!(
                monuments
                    .groups
                    .iter()
                    .any(|g| g.kind == "Gartendenkmal" && g.links.len() == 1)
            );
            let objects = part.protected_objects.as_ref().unwrap();
            assert!(!objects.is_empty());
            let green = part.green_spaces.as_ref().unwrap();
            assert!(green.entries.iter().any(|s| s.item.name == "Viktoriapark"));
            assert!(green.playgrounds >= 3, "{}", green.playgrounds);
            assert_eq!(
                green.playgrounds, 4,
                "every playground of the fixture is on the map"
            );
            let keys: Vec<&str> = notes.hints.iter().map(|h| h.key.as_str()).collect();
            assert!(!keys.contains(&crate::permits::notes::hint_key::CITY_STATE));

            let calls = fetcher.calls.get();
            let offline = fetch(&project, dir.path(), &fetcher, &cache, true)
                .0
                .unwrap();
            assert_eq!(fetcher.calls.get(), calls, "offline asks no server");
            assert_eq!(offline, online);
            assert_eq!(
                write_permit_notes(dir.path(), &project, &offline, today).unwrap(),
                notes
            );

            // A record of step 1 has no regional part; it still reads, without Berlin.
            let sources = dir.path().join(PERMITS_DIR).join("sources.json");
            let mut record: Value =
                serde_json::from_str(&std::fs::read_to_string(&sources).unwrap()).unwrap();
            assert!(record["regional"].is_array());
            record.as_object_mut().unwrap().remove("regional");
            std::fs::write(&sources, serde_json::to_vec(&record).unwrap()).unwrap();
            let old = fetch(&project, dir.path(), &fetcher, &cache, true)
                .0
                .unwrap();
            assert!(old.regional.is_empty());
            assert_eq!(old.admin, online.admin);
        });
    }

    /// The findings the engine reads back from the store, online and offline, are the ones the
    /// adapter parses from the same fixture files itself (review finding 11).
    #[test]
    fn stored_berlin_layers_parse_as_the_adapter_parses_them() {
        on_the_day_of_the_fixtures(|| {
            let (dir, project) = viktoriapark();
            let fetcher = MapFetcher::default();
            serve_berlin(&fetcher, &project);
            let (_root, cache) = empty_cache();
            let inputs = fetch(&project, dir.path(), &fetcher, &cache, false)
                .0
                .unwrap();
            let offline = fetch(&project, dir.path(), &fetcher, &cache, true)
                .0
                .unwrap();
            let expected = berlin_from_fixtures(date("2026-11-13"));
            assert_eq!(inputs.regional.len(), expected.len());
            for ((online, offline), expected) in
                inputs.regional.iter().zip(&offline.regional).zip(&expected)
            {
                assert_eq!(online.source.key, expected.source.key);
                let SourceInput::Read { items, .. } = &online.findings else {
                    panic!("{:?}", online.findings);
                };
                assert!(!items.is_empty(), "{}", online.source.key);
                assert_eq!(online, expected, "{}", expected.source.key);
                assert_eq!(offline, expected, "{}", expected.source.key);
            }
            let bezirke: &RegionalInput = &inputs.regional[0];
            let SourceInput::Read { items, .. } = &bezirke.findings else {
                panic!("{:?}", bezirke.findings);
            };
            assert!(
                items
                    .iter()
                    .all(|f| matches!(f.kind, FindingKind::District { .. }))
            );
            assert_eq!(items.len(), 2);
        });
    }
}
