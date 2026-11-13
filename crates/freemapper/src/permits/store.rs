//! The raw inputs of the permit notes in the project folder (`data/permits/`), and the notes
//! file `permits.json`.
//!
//! Layout of [`PERMITS_DIR`]: `vg250-<level>.geojson` for each VG250 level and
//! `bfn-<kind>.geojson` for each BfN kind (the features as the service gave them, in a
//! FeatureCollection; the names are the serde names of [`AdminLevel`] and [`ProtectedKind`]),
//! `osm.geojson` (the OSM findings: the fields of [`OsmFinding`] as properties, the area as a
//! MultiPolygon), and `sources.json` (per source its state, fetch date, count or failure, the
//! box the sources were asked for and the key of the map area polygon). A source that was not
//! read has no files.
//!
//! The sources of the region adapters (ADR 0118) lie in a folder per adapter,
//! `<adapter id>/<source key>-<layer key>.geojson`, the features as the service gave them;
//! `sources.json` records their state in `regional`, which older records do not have.

use std::io;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use freemapper_sources::bfn::{ProtectedArea, ProtectedKind, parse_bfn};
use freemapper_sources::geojson_wfs::LatLonBox;
use freemapper_sources::vg250::{AdminLevel, AdminUnit, parse_vg250};
use geo_types::{Coord, LineString, MultiPolygon, Polygon};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::adapters::Registry;
use crate::area::MapArea;
use crate::permits::notes::{Inputs, PermitNotes, SourceInput, assemble};
use crate::permits::osm::OsmFinding;
use crate::permits::regional::{RegionalInput, RegionalSource};
use crate::permits::{Missing, PermitsError};
use crate::project::ProjectFile;

/// The folder of the raw inputs, relative to the project folder.
pub const PERMITS_DIR: &str = "data/permits";
/// The notes as JSON, in the project folder.
pub const PERMITS_JSON: &str = "permits.json";
/// The notes as Markdown, in the project folder.
pub const PERMITS_MD: &str = "permits.md";

/// The record of what was fetched, in [`PERMITS_DIR`].
const SOURCES_JSON: &str = "sources.json";
/// The OSM findings, in [`PERMITS_DIR`].
const OSM_GEOJSON: &str = "osm.geojson";
/// The version of the shape of `sources.json`.
const STORE_SCHEMA: u32 = 1;

/// What the sources answered, before it is parsed: the raw features per level and kind, the
/// OSM findings with their areas, or why a source gave nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct RawInputs {
    /// The box the sources were asked for (the map area's in WGS 84).
    pub bbox: LatLonBox,
    /// [`area_key`] of the map area the inputs belong to.
    pub area: String,
    /// The features of every VG250 level, in the order of [`AdminLevel::ALL`].
    pub admin: SourceInput<(AdminLevel, Vec<Value>)>,
    /// The features of every BfN kind, in the order of [`ProtectedKind::ALL`].
    pub protected: SourceInput<(ProtectedKind, Vec<Value>)>,
    pub osm: SourceInput<OsmFinding>,
    /// The sources of the region adapters, in the order of the registry.
    pub regional: Vec<RawRegional>,
}

/// What one source of a region adapter answered: the features per layer key.
#[derive(Debug, Clone, PartialEq)]
pub struct RawRegional {
    pub adapter: String,
    pub source: RegionalSource,
    pub layers: SourceInput<(String, Vec<Value>)>,
}

/// `sources.json`.
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    schema: u32,
    /// The box asked for: min lon, min lat, max lon, max lat.
    bbox: [f64; 4],
    /// [`area_key`] of the map area; missing in a record written before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    area: Option<String>,
    vg250: SourceRecord,
    bfn: SourceRecord,
    osm: SourceRecord,
    /// The sources of the region adapters; missing in a record written before they existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    regional: Vec<RegionalRecord>,
}

/// The state of one source of a region adapter.
#[derive(Debug, Serialize, Deserialize)]
struct RegionalRecord {
    adapter: String,
    source: String,
    #[serde(flatten)]
    record: SourceRecord,
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceRecord {
    state: Outcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fetched: Option<NaiveDate>,
    /// Features (VG250, BfN) or findings (OSM) stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Outcome {
    Read,
    Unreachable,
    NotRead,
}

fn source_record<T>(input: &SourceInput<T>, count: impl Fn(&[T]) -> usize) -> SourceRecord {
    match input {
        SourceInput::Read { items, fetched } => SourceRecord {
            state: Outcome::Read,
            fetched: Some(*fetched),
            count: Some(count(items)),
            reason: None,
        },
        SourceInput::Unreachable { reason } => SourceRecord {
            state: Outcome::Unreachable,
            fetched: None,
            count: None,
            reason: Some(reason.clone()),
        },
        SourceInput::NotRead => SourceRecord {
            state: Outcome::NotRead,
            fetched: None,
            count: None,
            reason: None,
        },
    }
}

/// The serde name of a level or kind (`gemeinde`, `landschaftsschutzgebiet`).
fn serde_name(key: impl Serialize) -> String {
    match serde_json::to_value(key) {
        Ok(Value::String(name)) => name,
        other => unreachable!("a unit variant is a string: {other:?}"),
    }
}

fn vg250_file(level: AdminLevel) -> String {
    format!("vg250-{}.geojson", serde_name(level))
}

fn bfn_file(kind: ProtectedKind) -> String {
    format!("bfn-{}.geojson", serde_name(kind))
}

/// An I/O error that names its file.
fn with_path((path, e): (PathBuf, io::Error)) -> io::Error {
    io::Error::new(e.kind(), format!("{}: {e}", path.display()))
}

/// The file of the layer `layer` of the regional source `source`, inside the folder of its
/// adapter.
fn regional_file(source: &str, layer: &str) -> String {
    format!("{source}-{layer}.geojson")
}

/// Stores `raw` in [`PERMITS_DIR`] of the project folder `dir`, replacing everything an
/// earlier call stored: the files of a source that was not read are removed. `sources.json`
/// goes first and comes back last, so a write cut short leaves no record of half the files.
pub fn store_inputs(dir: &Path, raw: &RawInputs) -> io::Result<()> {
    store_inputs_in(crate::adapters::registry(), dir, raw)
}

/// [`store_inputs`] with the regional sources of `registry`: their files are removed and
/// written only under the keys the adapters name.
pub fn store_inputs_in(registry: &Registry, dir: &Path, raw: &RawInputs) -> io::Result<()> {
    let folder = dir.join(PERMITS_DIR);
    freemapper_fs::check_folder_inside(dir, &folder).map_err(with_path)?;
    std::fs::create_dir_all(&folder)?;
    let remove = |name: &str| freemapper_fs::remove_inside(dir, &folder.join(name));
    remove(SOURCES_JSON);
    for level in AdminLevel::ALL {
        remove(&vg250_file(level));
    }
    for kind in ProtectedKind::ALL {
        remove(&bfn_file(kind));
    }
    remove(OSM_GEOJSON);
    for adapter in registry.adapters() {
        let own = folder.join(adapter.info().id.as_str());
        for source in adapter.permit_sources() {
            let info = source.info();
            for layer in info.layers {
                freemapper_fs::remove_inside(dir, &own.join(regional_file(info.key, layer)));
            }
        }
    }
    let write = |name: &str, bytes: &[u8]| {
        freemapper_fs::write_inside(dir, &folder.join(name), bytes).map_err(with_path)
    };
    if let SourceInput::Read { items, .. } = &raw.admin {
        for (level, features) in items {
            write(&vg250_file(*level), &collection(features))?;
        }
    }
    if let SourceInput::Read { items, .. } = &raw.protected {
        for (kind, features) in items {
            write(&bfn_file(*kind), &collection(features))?;
        }
    }
    if let SourceInput::Read { items, .. } = &raw.osm {
        let features: Vec<Value> = items.iter().map(osm_feature).collect();
        write(OSM_GEOJSON, &collection(&features))?;
    }
    let mut regional = Vec::new();
    for input in &raw.regional {
        // Only a source of the registry, and only its own layers, get a file.
        let Some(info) = regional_info(registry, &input.adapter, &input.source.key) else {
            continue;
        };
        if let SourceInput::Read { items, .. } = &input.layers {
            let own = folder.join(&input.adapter);
            freemapper_fs::check_folder_inside(dir, &own).map_err(with_path)?;
            std::fs::create_dir_all(&own)?;
            for (layer, features) in items {
                if !info.layers.contains(&layer.as_str()) {
                    continue;
                }
                let path = own.join(regional_file(info.key, layer));
                freemapper_fs::write_inside(dir, &path, &collection(features))
                    .map_err(with_path)?;
            }
        }
        regional.push(RegionalRecord {
            adapter: input.adapter.clone(),
            source: input.source.key.clone(),
            record: source_record(&input.layers, features::<String>),
        });
    }
    let record = Record {
        schema: STORE_SCHEMA,
        bbox: [
            raw.bbox.min_lon,
            raw.bbox.min_lat,
            raw.bbox.max_lon,
            raw.bbox.max_lat,
        ],
        area: Some(raw.area.clone()),
        vg250: source_record(&raw.admin, features::<AdminLevel>),
        bfn: source_record(&raw.protected, features::<ProtectedKind>),
        osm: source_record(&raw.osm, <[OsmFinding]>::len),
        regional,
    };
    let mut text = serde_json::to_string_pretty(&record).map_err(io::Error::other)?;
    text.push('\n');
    write(SOURCES_JSON, text.as_bytes())
}

/// The features of all levels or kinds.
fn features<K>(items: &[(K, Vec<Value>)]) -> usize {
    items.iter().map(|(_, f)| f.len()).sum()
}

/// `features` as a GeoJSON FeatureCollection.
fn collection(features: &[Value]) -> Vec<u8> {
    #[derive(Serialize)]
    struct Collection<'a> {
        #[serde(rename = "type")]
        kind: &'static str,
        features: &'a [Value],
    }
    serde_json::to_vec(&Collection {
        kind: "FeatureCollection",
        features,
    })
    .expect("JSON values serialize")
}

/// An OSM finding as a GeoJSON feature: its fields as properties, its area as geometry.
fn osm_feature(finding: &OsmFinding) -> Value {
    let ring =
        |ring: &LineString<f64>| -> Vec<[f64; 2]> { ring.0.iter().map(|c| [c.x, c.y]).collect() };
    let polygons: Vec<Vec<Vec<[f64; 2]>>> = finding
        .geometry
        .0
        .iter()
        .map(|p| {
            std::iter::once(p.exterior())
                .chain(p.interiors())
                .map(ring)
                .collect()
        })
        .collect();
    json!({
        "type": "Feature",
        "properties": serde_json::to_value(finding).expect("a finding serializes"),
        "geometry": { "type": "MultiPolygon", "coordinates": polygons },
    })
}

/// The finding of a feature [`osm_feature`] wrote.
fn osm_finding(feature: &Value) -> Option<OsmFinding> {
    let mut finding: OsmFinding =
        serde_json::from_value(feature.get("properties")?.clone()).ok()?;
    let polygons: Vec<Vec<Vec<[f64; 2]>>> =
        serde_json::from_value(feature.get("geometry")?.get("coordinates")?.clone()).ok()?;
    let ring = |points: Vec<[f64; 2]>| {
        LineString::new(points.into_iter().map(|[x, y]| Coord { x, y }).collect())
    };
    let polygons = polygons
        .into_iter()
        .map(|rings| {
            let mut rings = rings.into_iter().map(ring);
            let exterior = rings.next()?;
            Some(Polygon::new(exterior, rings.collect()))
        })
        .collect::<Option<Vec<_>>>()?;
    finding.geometry = MultiPolygon::new(polygons);
    Some(finding)
}

/// `sources.json`, or `None` when nothing was stored.
fn read_record(dir: &Path) -> Result<Option<Record>, PermitsError> {
    let path = dir.join(PERMITS_DIR).join(SOURCES_JSON);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(PermitsError::Io(path, e)),
    };
    let record: Record = serde_json::from_str(&text).map_err(|e| PermitsError::Stored {
        path: path.clone(),
        message: e.to_string(),
    })?;
    if record.schema != STORE_SCHEMA {
        return Err(PermitsError::Stored {
            path,
            message: format!("stored in version {} of its layout", record.schema),
        });
    }
    Ok(Some(record))
}

/// A key of the map area polygon: the SHA-256 of the bits of its coordinates, exterior ring
/// first, then the holes. Two areas have the same key only when every corner is the same.
pub fn area_key(area: &MapArea) -> String {
    use sha2::{Digest as _, Sha256};
    let mut hash = Sha256::new();
    let polygon = &area.polygon;
    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
        hash.update((ring.0.len() as u64).to_le_bytes());
        for c in &ring.0 {
            hash.update(c.x.to_bits().to_le_bytes());
            hash.update(c.y.to_bits().to_le_bytes());
        }
    }
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// What the stored inputs were fetched for.
pub(crate) struct StoredFor {
    /// The box the sources were asked for.
    pub bbox: LatLonBox,
    /// [`area_key`] of the map area; `None` in a record written before it existed.
    pub area: Option<String>,
}

/// What the stored inputs were fetched for, or `None` when nothing was stored.
pub(crate) fn stored_for(dir: &Path) -> Result<Option<StoredFor>, PermitsError> {
    Ok(read_record(dir)?.map(|r| {
        let [min_lon, min_lat, max_lon, max_lat] = r.bbox;
        StoredFor {
            bbox: LatLonBox {
                min_lat,
                min_lon,
                max_lat,
                max_lon,
            },
            area: r.area,
        }
    }))
}

/// The features of a stored FeatureCollection.
fn read_collection(path: &Path) -> Result<Vec<Value>, PermitsError> {
    let stored = |message: String| PermitsError::Stored {
        path: path.to_path_buf(),
        message,
    };
    let bytes = std::fs::read(path).map_err(|e| stored(e.to_string()))?;
    let mut value: Value = serde_json::from_slice(&bytes).map_err(|e| stored(e.to_string()))?;
    match value.get_mut("features").map(Value::take) {
        Some(Value::Array(features)) => Ok(features),
        _ => Err(stored("not a FeatureCollection".to_string())),
    }
}

/// A source as `sources.json` records it; `read` gives the items of one that was read.
fn load_source<T>(
    record: &SourceRecord,
    path: &Path,
    read: impl FnOnce() -> Result<Vec<T>, PermitsError>,
) -> Result<SourceInput<T>, PermitsError> {
    Ok(match record.state {
        Outcome::Read => SourceInput::Read {
            fetched: record.fetched.ok_or_else(|| PermitsError::Stored {
                path: path.to_path_buf(),
                message: "a source that was read has no fetch date".to_string(),
            })?,
            items: read()?,
        },
        Outcome::Unreachable => SourceInput::Unreachable {
            reason: record.reason.clone().unwrap_or_default(),
        },
        Outcome::NotRead => SourceInput::NotRead,
    })
}

/// The facts of the source `key` of the adapter `adapter` of `registry`, if it has one.
fn regional_info<'a>(
    registry: &'a Registry,
    adapter: &str,
    key: &str,
) -> Option<&'a freemapper_adapter_api::permits::PermitSourceInfo> {
    regional_source(registry, adapter, key).map(|s| s.info())
}

fn regional_source<'a>(
    registry: &'a Registry,
    adapter: &str,
    key: &str,
) -> Option<&'a dyn freemapper_adapter_api::permits::PermitSource> {
    if !freemapper_adapter_api::permits::plain_key(key) {
        return None;
    }
    registry
        .get(adapter)?
        .permit_sources()
        .iter()
        .find(|s| s.info().key == key)
        .map(AsRef::as_ref)
}

/// The inputs stored in [`PERMITS_DIR`] of the project folder `dir`, parsed.
/// [`PermitsError::NothingRead`] when nothing is stored or no source was read.
pub fn load_inputs(dir: &Path) -> Result<Inputs, PermitsError> {
    load_inputs_in(crate::adapters::registry(), dir)
}

/// [`load_inputs`] with the regional sources of `registry`, which parse their stored layers.
/// A stored source that no adapter of `registry` has any more is left out.
pub fn load_inputs_in(registry: &Registry, dir: &Path) -> Result<Inputs, PermitsError> {
    let Some(record) = read_record(dir)? else {
        return Err(nothing_read([Some(""); 3]));
    };
    let folder = dir.join(PERMITS_DIR);
    let sources = folder.join(SOURCES_JSON);
    let admin = load_source(&record.vg250, &sources, || {
        let mut units: Vec<AdminUnit> = Vec::new();
        for level in AdminLevel::ALL {
            let features = read_collection(&folder.join(vg250_file(level)))?;
            units.extend(features.iter().filter_map(|f| parse_vg250(level, f)));
        }
        Ok(units)
    })?;
    let protected = load_source(&record.bfn, &sources, || {
        let mut areas: Vec<ProtectedArea> = Vec::new();
        for kind in ProtectedKind::ALL {
            let features = read_collection(&folder.join(bfn_file(kind)))?;
            areas.extend(features.iter().filter_map(|f| parse_bfn(kind, f)));
        }
        Ok(areas)
    })?;
    let osm = load_source(&record.osm, &sources, || {
        let path = folder.join(OSM_GEOJSON);
        read_collection(&path)?
            .iter()
            .map(|f| {
                osm_finding(f).ok_or_else(|| PermitsError::Stored {
                    path: path.clone(),
                    message: "a feature is not an OSM finding".to_string(),
                })
            })
            .collect()
    })?;
    let mut regional = Vec::new();
    for stored in &record.regional {
        let Some(source) = regional_source(registry, &stored.adapter, &stored.source) else {
            continue;
        };
        let info = source.info();
        let own = folder.join(&stored.adapter);
        let findings = load_source(&stored.record, &sources, || {
            let mut findings = Vec::new();
            for layer in info.layers {
                let features = read_collection(&own.join(regional_file(info.key, layer)))?;
                findings.extend(source.findings(layer, &features));
            }
            Ok(findings)
        })?;
        regional.push(RegionalInput {
            adapter: stored.adapter.clone(),
            source: RegionalSource::from(info),
            findings,
        });
    }
    let inputs = Inputs {
        admin,
        protected,
        osm,
        regional,
    };
    something_read(&inputs)?;
    Ok(inputs)
}

/// [`PermitsError::NothingRead`] when no source of `inputs` was read.
fn something_read(inputs: &Inputs) -> Result<(), PermitsError> {
    if inputs
        .regional
        .iter()
        .any(|r| matches!(r.findings, SourceInput::Read { .. }))
    {
        return Ok(());
    }
    check_states([
        state_of(&inputs.admin),
        state_of(&inputs.protected),
        state_of(&inputs.osm),
    ])
}

/// [`PermitsError::NothingRead`] when no source of `raw` was read: then there is nothing to
/// store, and what an earlier fetch stored stays.
pub(crate) fn something_fetched(raw: &RawInputs) -> Result<(), PermitsError> {
    if raw
        .regional
        .iter()
        .any(|r| matches!(r.layers, SourceInput::Read { .. }))
    {
        return Ok(());
    }
    check_states([
        state_of(&raw.admin),
        state_of(&raw.protected),
        state_of(&raw.osm),
    ])
}

/// The states of VG250, the BfN and OSM, in this order.
fn check_states(states: [Option<&str>; 3]) -> Result<(), PermitsError> {
    if states.iter().any(Option::is_none) {
        Ok(())
    } else {
        Err(nothing_read(states))
    }
}

/// `None` for a source that was read, else its failure (empty when it was not read).
fn state_of<T>(input: &SourceInput<T>) -> Option<&str> {
    match input {
        SourceInput::Read { .. } => None,
        SourceInput::Unreachable { reason } => Some(reason),
        SourceInput::NotRead => Some(""),
    }
}

fn nothing_read([admin, protected, osm]: [Option<&str>; 3]) -> PermitsError {
    let missing = |source: &str, state: Option<&str>| Missing {
        source: source.to_string(),
        reason: state.filter(|r| !r.is_empty()).map(str::to_string),
    };
    PermitsError::NothingRead {
        missing: vec![
            missing("vg250", admin),
            missing("bfn", protected),
            missing("osm", osm),
        ],
    }
}

/// Writes the permit notes of `project` from `inputs` as [`PERMITS_JSON`] into the project
/// folder `dir`. [`PermitsError::NothingRead`] (and nothing written) when no source was read.
pub fn write_permit_notes(
    dir: &Path,
    project: &ProjectFile,
    inputs: &Inputs,
    today: NaiveDate,
) -> Result<PermitNotes, PermitsError> {
    something_read(inputs)?;
    let notes = assemble(
        &project.area()?,
        project.crs(),
        project.map_kind,
        inputs,
        today,
    );
    let mut text = serde_json::to_string_pretty(&notes).expect("the notes serialize");
    text.push('\n');
    freemapper_fs::write_inside(dir, &dir.join(PERMITS_JSON), text.as_bytes())
        .map_err(|(path, e)| PermitsError::Io(path, e))?;
    Ok(notes)
}
