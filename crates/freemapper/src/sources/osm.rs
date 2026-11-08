//! OpenStreetMap data (ODbL) from the Geofabrik extracts: one per state, and one per government
//! district (Regierungsbezirk) in North Rhine-Westphalia.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::NaiveDate;
use geo_types::MultiPolygon;

use crate::area::GeoBox;
use crate::crs::{BoundingBox, LatLon};
use crate::fetch::{Cache, FetchError, FetchMode, Fetcher};
use crate::sources::region::{Region, parse_boundary, regions_for, within_margin};

/// Read buffer for the extract.
const READ_BUFFER_BYTES: usize = 4 * 1024 * 1024;

/// OSM tags as key/value pairs, sorted by key.
pub type Tags = Vec<(String, String)>;

/// Decides which tagged elements to read; called from several threads.
pub type TagFilter<'a> = &'a (dyn Fn(&[(String, String)]) -> bool + Sync);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("OSM data: {message}")]
pub struct OsmError {
    pub message: String,
}

fn osm_error(message: impl std::fmt::Display) -> OsmError {
    OsmError {
        message: message.to_string(),
    }
}

/// Geometry of an OSM feature; a closed way is a line whose last point repeats the first.
#[derive(Debug, Clone, PartialEq)]
pub enum OsmGeometry {
    Point(LatLon),
    Line(Vec<LatLon>),
}

/// A tagged node or way.
#[derive(Debug, Clone, PartialEq)]
pub struct OsmFeature {
    /// `node/<id>` or `way/<id>`.
    pub id: String,
    pub tags: Tags,
    pub geometry: OsmGeometry,
}

fn sorted_tags<'a>(tags: impl Iterator<Item = (&'a str, &'a str)>) -> Tags {
    let mut tags: Tags = tags.map(|(k, v)| (k.to_string(), v.to_string())).collect();
    tags.sort();
    tags
}

/// The tagged nodes inside `area` and the tagged ways with a node inside it, for which `wanted`
/// holds. Ways keep all their nodes, also those outside `area`. Nodes come first, each kind
/// sorted by id.
pub fn read_extract(
    path: &Path,
    area: &GeoBox,
    wanted: TagFilter<'_>,
) -> Result<Vec<OsmFeature>, OsmError> {
    use osmpbf::Element;
    let node = |id: i64, lat: f64, lon: f64, tags: Tags| -> Option<(i64, OsmFeature)> {
        let p = LatLon { lat, lon };
        (!tags.is_empty() && area.contains(p) && wanted(&tags)).then(|| {
            (
                id,
                OsmFeature {
                    id: format!("node/{id}"),
                    tags,
                    geometry: OsmGeometry::Point(p),
                },
            )
        })
    };
    let mut nodes = osmpbf::ElementReader::from_path(path)
        .map_err(in_file(path))?
        .par_map_reduce(
            |element| match element {
                Element::Node(n) => node(n.id(), n.lat(), n.lon(), sorted_tags(n.tags()))
                    .into_iter()
                    .collect(),
                Element::DenseNode(n) => node(n.id(), n.lat(), n.lon(), sorted_tags(n.tags()))
                    .into_iter()
                    .collect(),
                _ => Vec::new(),
            },
            Vec::new,
            |mut a, b| {
                a.extend(b);
                a
            },
        )
        .map_err(in_file(path))?;
    nodes.sort_by_key(|(id, _)| *id);

    let mut ways: Vec<(i64, Tags, Vec<i64>)> = Vec::new();
    let mut coords = std::collections::HashMap::new();
    // Buffered: the indexed reader seeks and reads many small blobs.
    let file = std::fs::File::open(path).map_err(in_file(path))?;
    osmpbf::IndexedReader::new(std::io::BufReader::with_capacity(READ_BUFFER_BYTES, file))
        .map_err(in_file(path))?
        .read_ways_and_deps(
            |way| {
                let tags = sorted_tags(way.tags());
                !tags.is_empty() && wanted(&tags)
            },
            |element| match element {
                Element::Way(w) => ways.push((w.id(), sorted_tags(w.tags()), w.refs().collect())),
                Element::Node(n) => {
                    coords.insert(
                        n.id(),
                        LatLon {
                            lat: n.lat(),
                            lon: n.lon(),
                        },
                    );
                }
                Element::DenseNode(n) => {
                    coords.insert(
                        n.id(),
                        LatLon {
                            lat: n.lat(),
                            lon: n.lon(),
                        },
                    );
                }
                Element::Relation(_) => {}
            },
        )
        .map_err(in_file(path))?;
    ways.sort_by_key(|(id, _, _)| *id);

    let mut features: Vec<OsmFeature> = nodes.into_iter().map(|(_, f)| f).collect();
    for (id, tags, refs) in ways {
        let points: Option<Vec<LatLon>> = refs.iter().map(|r| coords.get(r).copied()).collect();
        if let Some(points) = points.filter(|p| touches(p, area)) {
            features.push(OsmFeature {
                id: format!("way/{id}"),
                tags,
                geometry: OsmGeometry::Line(points),
            });
        }
    }
    Ok(features)
}

/// Error converter naming the extract file.
fn in_file<E: std::fmt::Display>(path: &Path) -> impl Fn(E) -> OsmError + '_ {
    move |e| osm_error(format!("{}: {e}", path.display()))
}

/// Whether a way with these points has a vertex inside `area` or a segment crossing it.
fn touches(points: &[LatLon], area: &GeoBox) -> bool {
    use geo::Intersects as _;
    let rect = geo_types::Rect::new(
        geo_types::coord! { x: area.min_lon, y: area.min_lat },
        geo_types::coord! { x: area.max_lon, y: area.max_lat },
    );
    points.iter().any(|p| area.contains(*p))
        || points.windows(2).any(|w| {
            geo_types::Line::new(
                geo_types::coord! { x: w[0].lon, y: w[0].lat },
                geo_types::coord! { x: w[1].lon, y: w[1].lat },
            )
            .intersects(&rect)
        })
}

/// The features as a GeoJSON FeatureCollection (tags as properties).
pub fn features_to_geojson(features: &[OsmFeature]) -> String {
    let features: Vec<serde_json::Value> = features
        .iter()
        .map(|f| {
            let geometry = match &f.geometry {
                OsmGeometry::Point(p) => {
                    serde_json::json!({ "type": "Point", "coordinates": [p.lon, p.lat] })
                }
                OsmGeometry::Line(points) => serde_json::json!({
                    "type": "LineString",
                    "coordinates": points.iter().map(|p| [p.lon, p.lat]).collect::<Vec<_>>()
                }),
            };
            let properties: serde_json::Map<String, serde_json::Value> = f
                .tags
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            serde_json::json!({ "type": "Feature", "id": f.id, "properties": properties, "geometry": geometry })
        })
        .collect();
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// Reads features written by [`features_to_geojson`].
pub fn features_from_geojson(text: &str) -> Result<Vec<OsmFeature>, OsmError> {
    let collection: serde_json::Value = serde_json::from_str(text).map_err(osm_error)?;
    let features = collection["features"]
        .as_array()
        .ok_or_else(|| osm_error("no features"))?;
    let point = |c: &serde_json::Value| -> Option<LatLon> {
        Some(LatLon {
            lon: c.get(0)?.as_f64()?,
            lat: c.get(1)?.as_f64()?,
        })
    };
    features
        .iter()
        .map(|f| {
            let id = f["id"]
                .as_str()
                .ok_or_else(|| osm_error("feature without id"))?;
            let bad = || osm_error(format!("{id}: invalid geometry"));
            let coordinates = &f["geometry"]["coordinates"];
            let geometry = match f["geometry"]["type"].as_str() {
                Some("Point") => OsmGeometry::Point(point(coordinates).ok_or_else(bad)?),
                Some("LineString") => OsmGeometry::Line(
                    coordinates
                        .as_array()
                        .ok_or_else(bad)?
                        .iter()
                        .map(|c| point(c).ok_or_else(bad))
                        .collect::<Result<_, _>>()?,
                ),
                _ => return Err(bad()),
            };
            let mut tags: Tags = f["properties"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                .collect();
            tags.sort();
            Ok(OsmFeature {
                id: id.to_string(),
                tags,
                geometry,
            })
        })
        .collect()
}

/// The Brandenburg extract; it holds Berlin too.
pub const EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/brandenburg-latest.osm.pbf";
pub const STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/brandenburg-updates/state.txt";
const SAXONY_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/sachsen-latest.osm.pbf";
const SAXONY_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/sachsen-updates/state.txt";
const SAXONY_ANHALT_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/sachsen-anhalt-latest.osm.pbf";
const SAXONY_ANHALT_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/sachsen-anhalt-updates/state.txt";
const HAMBURG_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/hamburg-latest.osm.pbf";
const HAMBURG_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/hamburg-updates/state.txt";
const HESSEN_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/hessen-latest.osm.pbf";
const HESSEN_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/hessen-updates/state.txt";
const SCHLESWIG_HOLSTEIN_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf";
const SCHLESWIG_HOLSTEIN_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/schleswig-holstein-updates/state.txt";
const MECKLENBURG_VORPOMMERN_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern-latest.osm.pbf";
const MECKLENBURG_VORPOMMERN_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern-updates/state.txt";
const NIEDERSACHSEN_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/niedersachsen-latest.osm.pbf";
const NIEDERSACHSEN_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/niedersachsen-updates/state.txt";
const RHEINLAND_PFALZ_EXTRACT_URL: &str =
    "https://download.geofabrik.de/europe/germany/rheinland-pfalz-latest.osm.pbf";
const RHEINLAND_PFALZ_STATE_URL: &str =
    "https://download.geofabrik.de/europe/germany/rheinland-pfalz-updates/state.txt";
/// A cached extract this much older than Geofabrik's state is still used.
pub(crate) const MAX_EXTRACT_AGE_DAYS: i64 = 7;
const EXTRACT_SUFFIX: &str = ".osm.pbf";

/// A government district (Regierungsbezirk) of North Rhine-Westphalia. Geofabrik serves each as
/// its own extract (137-225 MB), so a map reads one of them instead of the 917 MB state extract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernmentDistrict {
    Arnsberg,
    Detmold,
    Duesseldorf,
    Koeln,
    Muenster,
}

/// The Geofabrik extract of a government district.
struct DistrictExtract {
    url: &'static str,
    state_url: &'static str,
    /// Start of the cache file name, `nrw-<slug>_`.
    prefix: &'static str,
    /// Rough download size in MB.
    megabytes: f64,
    label: &'static str,
}

/// The extract of the district with Geofabrik's `slug`: URL, state URL and cache prefix all come
/// from the slug.
macro_rules! district_extract {
    ($slug:literal, $megabytes:literal, $label:literal) => {
        DistrictExtract {
            url: concat!(
                "https://download.geofabrik.de/europe/germany/nordrhein-westfalen/",
                $slug,
                "-regbez-latest.osm.pbf"
            ),
            state_url: concat!(
                "https://download.geofabrik.de/europe/germany/nordrhein-westfalen/",
                $slug,
                "-regbez-updates/state.txt"
            ),
            prefix: concat!("nrw-", $slug, "_"),
            megabytes: $megabytes,
            label: $label,
        }
    };
}

impl GovernmentDistrict {
    /// The district's extract: slug, size (Geofabrik, 2026-11-08) and label.
    fn extract(self) -> DistrictExtract {
        match self {
            Self::Arnsberg => {
                district_extract!("arnsberg", 195.2, "North Rhine-Westphalia, Arnsberg")
            }
            Self::Detmold => {
                district_extract!("detmold", 137.3, "North Rhine-Westphalia, Detmold")
            }
            Self::Duesseldorf => {
                district_extract!("duesseldorf", 217.2, "North Rhine-Westphalia, Düsseldorf")
            }
            Self::Koeln => district_extract!("koeln", 225.2, "North Rhine-Westphalia, Köln"),
            Self::Muenster => {
                district_extract!("muenster", 140.9, "North Rhine-Westphalia, Münster")
            }
        }
    }

    /// All districts, in the order their extracts are read.
    pub const ALL: [Self; 5] = [
        Self::Arnsberg,
        Self::Detmold,
        Self::Duesseldorf,
        Self::Koeln,
        Self::Muenster,
    ];

    /// The district's extract boundary: Geofabrik's `<district>-regbez.poly` in EPSG:25833,
    /// simplified 25 m (ODbL). Neighbouring boundaries overlap by a few hundred metres, as the
    /// extracts do.
    fn boundary(self) -> &'static MultiPolygon<f64> {
        static BOUNDARIES: [OnceLock<MultiPolygon<f64>>; 5] = [
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
        ];
        let (index, text) = match self {
            Self::Arnsberg => (
                0,
                include_str!("../../../../assets/regions/osm/nrw-arnsberg.geojson"),
            ),
            Self::Detmold => (
                1,
                include_str!("../../../../assets/regions/osm/nrw-detmold.geojson"),
            ),
            Self::Duesseldorf => (
                2,
                include_str!("../../../../assets/regions/osm/nrw-duesseldorf.geojson"),
            ),
            Self::Koeln => (
                3,
                include_str!("../../../../assets/regions/osm/nrw-koeln.geojson"),
            ),
            Self::Muenster => (
                4,
                include_str!("../../../../assets/regions/osm/nrw-muenster.geojson"),
            ),
        };
        BOUNDARIES[index].get_or_init(|| parse_boundary(text))
    }
}

/// A Geofabrik extract freemapper reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractSource {
    /// Brandenburg with Berlin.
    Brandenburg,
    Saxony,
    SaxonyAnhalt,
    Hamburg,
    Hessen,
    SchleswigHolstein,
    MecklenburgVorpommern,
    /// Niedersachsen, with Bremen inside it.
    Niedersachsen,
    /// Rheinland-Pfalz (without Wiesbaden and Mainz-Kastel, which are Hesse).
    RheinlandPfalz,
    /// A North Rhine-Westphalia government district.
    NordrheinWestfalen(GovernmentDistrict),
}

/// The extracts a map of `bbox` (EPSG:25833) reads, each once: the extract of every region the
/// area touches; in North Rhine-Westphalia every government district extract whose boundary lies
/// within the region margin of the area (two or more on a district border). Warnings go to
/// `progress`.
pub fn extracts_for(bbox: &BoundingBox, progress: &mut dyn FnMut(&str)) -> Vec<ExtractSource> {
    let mut sources: Vec<ExtractSource> = Vec::new();
    for region in regions_for(bbox) {
        let found: Vec<ExtractSource> = match region {
            Region::Brandenburg | Region::Berlin => vec![ExtractSource::Brandenburg],
            Region::Saxony => vec![ExtractSource::Saxony],
            Region::SaxonyAnhalt => vec![ExtractSource::SaxonyAnhalt],
            Region::Hamburg => vec![ExtractSource::Hamburg],
            Region::Hessen => vec![ExtractSource::Hessen],
            Region::SchleswigHolstein => vec![ExtractSource::SchleswigHolstein],
            Region::MecklenburgVorpommern => vec![ExtractSource::MecklenburgVorpommern],
            Region::Niedersachsen => vec![ExtractSource::Niedersachsen],
            Region::RheinlandPfalz => vec![ExtractSource::RheinlandPfalz],
            Region::NordrheinWestfalen => nrw_districts_for(bbox, progress)
                .into_iter()
                .map(ExtractSource::NordrheinWestfalen)
                .collect(),
        };
        for source in found {
            if !sources.contains(&source) {
                sources.push(source);
            }
        }
    }
    sources
}

/// The districts whose extract boundary lies within the margin of `bbox`. The boundaries cover
/// the state with its margin; should an area at the state border still miss them all, the
/// nearest district's extract is read.
fn nrw_districts_for(
    bbox: &BoundingBox,
    progress: &mut dyn FnMut(&str),
) -> Vec<GovernmentDistrict> {
    let near: Vec<GovernmentDistrict> = GovernmentDistrict::ALL
        .into_iter()
        .filter(|district| within_margin(district.boundary(), bbox))
        .collect();
    if !near.is_empty() {
        return near;
    }
    let nearest = nearest_district(bbox);
    if let Some(district) = nearest {
        progress(&format!(
            "warning: no North Rhine-Westphalia district extract within 100 m of the area; \
             reading the nearest, {}",
            ExtractSource::NordrheinWestfalen(district).label()
        ));
    }
    nearest.into_iter().collect()
}

/// The district whose extract boundary is nearest to `bbox`.
fn nearest_district(bbox: &BoundingBox) -> Option<GovernmentDistrict> {
    use geo::{Distance as _, Euclidean};
    let rect = geo_types::Rect::new(
        geo_types::coord! { x: bbox.min_e, y: bbox.min_n },
        geo_types::coord! { x: bbox.max_e, y: bbox.max_n },
    )
    .to_polygon();
    GovernmentDistrict::ALL
        .into_iter()
        .map(|district| (district, Euclidean.distance(district.boundary(), &rect)))
        .filter(|(_, distance)| distance.is_finite())
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(district, _)| district)
}

impl ExtractSource {
    pub fn url(self) -> &'static str {
        match self {
            Self::Brandenburg => EXTRACT_URL,
            Self::Saxony => SAXONY_EXTRACT_URL,
            Self::SaxonyAnhalt => SAXONY_ANHALT_EXTRACT_URL,
            Self::Hamburg => HAMBURG_EXTRACT_URL,
            Self::Hessen => HESSEN_EXTRACT_URL,
            Self::SchleswigHolstein => SCHLESWIG_HOLSTEIN_EXTRACT_URL,
            Self::MecklenburgVorpommern => MECKLENBURG_VORPOMMERN_EXTRACT_URL,
            Self::Niedersachsen => NIEDERSACHSEN_EXTRACT_URL,
            Self::RheinlandPfalz => RHEINLAND_PFALZ_EXTRACT_URL,
            Self::NordrheinWestfalen(district) => district.extract().url,
        }
    }

    pub fn state_url(self) -> &'static str {
        match self {
            Self::Brandenburg => STATE_URL,
            Self::Saxony => SAXONY_STATE_URL,
            Self::SaxonyAnhalt => SAXONY_ANHALT_STATE_URL,
            Self::Hamburg => HAMBURG_STATE_URL,
            Self::Hessen => HESSEN_STATE_URL,
            Self::SchleswigHolstein => SCHLESWIG_HOLSTEIN_STATE_URL,
            Self::MecklenburgVorpommern => MECKLENBURG_VORPOMMERN_STATE_URL,
            Self::Niedersachsen => NIEDERSACHSEN_STATE_URL,
            Self::RheinlandPfalz => RHEINLAND_PFALZ_STATE_URL,
            Self::NordrheinWestfalen(district) => district.extract().state_url,
        }
    }

    /// Start of the cache file name, `<prefix><date>.osm.pbf`.
    fn prefix(self) -> &'static str {
        match self {
            Self::Brandenburg => "brandenburg_",
            Self::Saxony => "sachsen_",
            Self::SaxonyAnhalt => "sachsen-anhalt_",
            Self::Hamburg => "hamburg_",
            Self::Hessen => "hessen_",
            Self::SchleswigHolstein => "schleswig-holstein_",
            Self::MecklenburgVorpommern => "mecklenburg-vorpommern_",
            Self::Niedersachsen => "niedersachsen_",
            Self::RheinlandPfalz => "rheinland-pfalz_",
            Self::NordrheinWestfalen(district) => district.extract().prefix,
        }
    }

    /// Rough download size in MB.
    pub fn megabytes(self) -> f64 {
        match self {
            Self::Brandenburg => 300.0,
            Self::Saxony => 270.0,
            Self::SaxonyAnhalt => 177.0,
            Self::Hamburg => 54.0,
            Self::Hessen => 348.0,
            Self::SchleswigHolstein => 159.0,
            Self::MecklenburgVorpommern => 128.8,
            Self::Niedersachsen => 510.0,
            Self::RheinlandPfalz => 275.0,
            Self::NordrheinWestfalen(district) => district.extract().megabytes,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Brandenburg => "Brandenburg",
            Self::Saxony => "Saxony",
            Self::SaxonyAnhalt => "Saxony-Anhalt",
            Self::Hamburg => "Hamburg",
            Self::Hessen => "Hesse",
            Self::SchleswigHolstein => "Schleswig-Holstein",
            Self::MecklenburgVorpommern => "Mecklenburg-Vorpommern",
            Self::Niedersachsen => "Lower Saxony",
            Self::RheinlandPfalz => "Rhineland-Palatinate",
            Self::NordrheinWestfalen(district) => district.extract().label,
        }
    }
}
/// Largest OSM extract downloaded: the largest state extract is about 350 MB.
const MAX_EXTRACT_BYTES: u64 = 2 * 1024 * 1024 * 1024;
/// Partial downloads older than this are left over from interrupted runs.
const STALE_PART_AGE: std::time::Duration = std::time::Duration::from_secs(3600);

/// A cached extract and the date of its data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extract {
    pub path: PathBuf,
    /// Date of Geofabrik's state when downloaded; names the cache file.
    pub date: NaiveDate,
    /// Date of the data: the replication timestamp in the file header, else `date`.
    pub data_date: NaiveDate,
}

/// Date of the `timestamp=` line in a Geofabrik `state.txt`.
pub fn parse_state_date(text: &str) -> Option<NaiveDate> {
    let value = text
        .lines()
        .find_map(|line| line.strip_prefix("timestamp="))?;
    value.get(..10)?.parse().ok()
}

/// Date of the newest cached OSM extract (Geofabrik's state when downloaded).
pub fn cached_extract_date(cache: &Cache, source: ExtractSource) -> Option<NaiveDate> {
    cached_extracts(cache, source).first().map(|e| e.date)
}

/// Cached extracts, newest first.
fn cached_extracts(cache: &Cache, source: ExtractSource) -> Vec<Extract> {
    let mut extracts: Vec<Extract> = std::fs::read_dir(cache.osm_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let date = name
                .strip_prefix(source.prefix())?
                .strip_suffix(EXTRACT_SUFFIX)?
                .parse()
                .ok()?;
            let path = entry.path();
            Some(Extract {
                data_date: check_pbf(&path, source.url())
                    .ok()
                    .flatten()
                    .unwrap_or(date),
                path,
                date,
            })
        })
        .collect();
    extracts.sort_by_key(|e| std::cmp::Reverse(e.date));
    extracts
}

/// Makes the extract of `source` available in `cache`. Online, the state is checked on every
/// call and the extract (a few hundred MB) downloaded only when the cached one is more than
/// [`MAX_EXTRACT_AGE_DAYS`] older; offline, the newest cached extract is used.
pub fn fetch_extract(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    source: ExtractSource,
    mode: FetchMode,
    progress: &mut dyn FnMut(&str),
) -> Result<Extract, FetchError> {
    crate::fsutil::remove_stale_parts(&cache.osm_dir(), STALE_PART_AGE);
    let newest = cached_extracts(cache, source).into_iter().next();
    if mode == FetchMode::Offline {
        return newest.ok_or(FetchError::OsmExtractMissing);
    }
    match refresh_extract(fetcher, cache, source, newest.clone(), progress) {
        Ok(extract) => Ok(extract),
        // Geofabrik unreachable, a portal page or a broken download: a cached extract is better
        // than no candidates.
        Err(e) => match newest {
            Some(cached) => {
                progress(&format!(
                    "warning: Geofabrik unavailable ({e}); using the cached OSM extract from {}",
                    cached.data_date
                ));
                Ok(cached)
            }
            None => Err(e),
        },
    }
}

/// The online part of [`fetch_extract`]: checks the state and downloads a newer extract.
fn refresh_extract(
    fetcher: &dyn Fetcher,
    cache: &Cache,
    source: ExtractSource,
    newest: Option<Extract>,
    progress: &mut dyn FnMut(&str),
) -> Result<Extract, FetchError> {
    let state = fetcher.get(source.state_url())?;
    let date =
        parse_state_date(&String::from_utf8_lossy(&state)).ok_or_else(|| FetchError::Http {
            url: source.state_url().to_string(),
            message: "no timestamp in the state file".to_string(),
        })?;
    if let Some(cached) = newest.filter(|e| (date - e.date).num_days() <= MAX_EXTRACT_AGE_DAYS) {
        progress(&format!(
            "cached OSM extract {} from {}",
            source.label(),
            cached.date
        ));
        return Ok(cached);
    }
    progress(&format!(
        "downloading {} (about {} MB)",
        source.url(),
        source.megabytes()
    ));
    let path = cache
        .osm_dir()
        .join(format!("{}{date}{EXTRACT_SUFFIX}", source.prefix()));
    let part = path.with_extension("pbf.part");
    let result = fetcher
        .download_to(source.url(), &part, MAX_EXTRACT_BYTES, progress)
        .and_then(|()| check_pbf(&part, source.url()));
    let header_date = match result {
        Ok(header_date) => header_date,
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            return Err(e);
        }
    };
    std::fs::rename(&part, &path).map_err(|e| FetchError::Io(path.clone(), e))?;
    for older in cached_extracts(cache, source)
        .into_iter()
        .filter(|e| e.path != path)
    {
        let _ = std::fs::remove_file(older.path);
    }
    Ok(Extract {
        path,
        date,
        data_date: header_date.unwrap_or(date),
    })
}

/// Checks that `path`, downloaded from `url`, starts with an OSM PBF header block and returns the
/// date of its replication timestamp, if it has one.
fn check_pbf(path: &Path, url: &str) -> Result<Option<NaiveDate>, FetchError> {
    let corrupt = |message: String| FetchError::CorruptExtract {
        url: url.to_string(),
        message,
    };
    let mut blobs = osmpbf::BlobReader::from_path(path).map_err(|e| corrupt(e.to_string()))?;
    let first = blobs
        .next()
        .ok_or_else(|| corrupt("empty file".to_string()))?
        .map_err(|e| corrupt(e.to_string()))?;
    match first.decode().map_err(|e| corrupt(e.to_string()))? {
        osmpbf::BlobDecode::OsmHeader(header) => Ok(header
            .osmosis_replication_timestamp()
            .and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0))
            .map(|time| time.date_naive())),
        _ => Err(corrupt("no OSM header".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    pub(crate) const FIXTURE_PBF: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/strausberg-school.osm.pbf"
    );

    fn state(date: &str) -> Vec<u8> {
        format!(
            "# original OSM minutely replication sequence number 7361559\ntimestamp={date}T21\\:21\\:21Z\nsequenceNumber=4956\n"
        )
        .into_bytes()
    }

    fn serve(fetcher: &MapFetcher, date: &str) {
        fetcher.insert(STATE_URL, state(date));
        fetcher.insert(EXTRACT_URL, std::fs::read(FIXTURE_PBF).unwrap());
    }

    fn online(fetcher: &MapFetcher, cache: &Cache) -> Result<Extract, FetchError> {
        fetch_extract(
            fetcher,
            cache,
            ExtractSource::Brandenburg,
            FetchMode::Online,
            &mut |_| {},
        )
    }

    #[test]
    fn saxony_extract_is_cached_beside_brandenburg() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-11-06");
        let saxony = ExtractSource::Saxony;
        assert_eq!(
            saxony.url(),
            "https://download.geofabrik.de/europe/germany/sachsen-latest.osm.pbf"
        );
        assert_eq!(
            saxony.state_url(),
            "https://download.geofabrik.de/europe/germany/sachsen-updates/state.txt"
        );
        fetcher.insert(saxony.state_url(), state("2026-11-06"));
        fetcher.insert(saxony.url(), std::fs::read(FIXTURE_PBF).unwrap());
        online(&fetcher, &cache).unwrap();
        let mut messages = Vec::new();
        fetch_extract(&fetcher, &cache, saxony, FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert_eq!(
            extracts(&cache),
            [
                "brandenburg_2026-11-06.osm.pbf",
                "sachsen_2026-11-06.osm.pbf"
            ]
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("sachsen-latest.osm.pbf (about 270 MB)")),
            "{messages:?}"
        );
        assert_eq!(
            cached_extract_date(&cache, saxony),
            "2026-11-06".parse().ok()
        );
        // Berlin (Mitte) reads the Brandenburg extract.
        let mitte = BoundingBox {
            min_e: 390_500.0,
            min_n: 5_819_500.0,
            max_e: 391_500.0,
            max_n: 5_820_500.0,
        };
        assert_eq!(
            extracts_for(&mitte, &mut |_| {}),
            [ExtractSource::Brandenburg]
        );
    }

    #[test]
    fn hamburg_extract_urls() {
        let hamburg = ExtractSource::Hamburg;
        assert_eq!(
            hamburg.url(),
            "https://download.geofabrik.de/europe/germany/hamburg-latest.osm.pbf"
        );
        assert_eq!(
            hamburg.state_url(),
            "https://download.geofabrik.de/europe/germany/hamburg-updates/state.txt"
        );
        assert_eq!(hamburg.label(), "Hamburg");
    }

    #[test]
    fn hessen_extract_urls() {
        let hessen = ExtractSource::Hessen;
        assert_eq!(
            hessen.url(),
            "https://download.geofabrik.de/europe/germany/hessen-latest.osm.pbf"
        );
        assert_eq!(
            hessen.state_url(),
            "https://download.geofabrik.de/europe/germany/hessen-updates/state.txt"
        );
        assert_eq!(hessen.label(), "Hesse");
        assert_eq!(hessen.megabytes(), 348.0);
    }

    #[test]
    fn schleswig_holstein_extract_urls() {
        let sh = ExtractSource::SchleswigHolstein;
        assert_eq!(
            sh.url(),
            "https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf"
        );
        assert_eq!(
            sh.state_url(),
            "https://download.geofabrik.de/europe/germany/schleswig-holstein-updates/state.txt"
        );
        assert_eq!(sh.label(), "Schleswig-Holstein");
        assert_eq!(sh.megabytes(), 159.0);
    }

    #[test]
    fn mv_extract_urls() {
        let mv = ExtractSource::MecklenburgVorpommern;
        assert_eq!(
            mv.url(),
            "https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern-latest.osm.pbf"
        );
        assert_eq!(
            mv.state_url(),
            "https://download.geofabrik.de/europe/germany/mecklenburg-vorpommern-updates/state.txt"
        );
        assert_eq!(mv.label(), "Mecklenburg-Vorpommern");
        assert_eq!(mv.megabytes(), 128.8);
    }

    #[test]
    fn ni_extract_urls() {
        let ni = ExtractSource::Niedersachsen;
        assert_eq!(
            ni.url(),
            "https://download.geofabrik.de/europe/germany/niedersachsen-latest.osm.pbf"
        );
        assert_eq!(
            ni.state_url(),
            "https://download.geofabrik.de/europe/germany/niedersachsen-updates/state.txt"
        );
        assert_eq!(ni.label(), "Lower Saxony");
        assert_eq!(ni.megabytes(), 510.0);
        // Hannover: the one state extract.
        let p = crate::crs::utm33_forward(crate::crs::LatLon {
            lat: 52.3749,
            lon: 9.7428,
        });
        let hannover = BoundingBox {
            min_e: p.easting - 200.0,
            min_n: p.northing - 200.0,
            max_e: p.easting + 200.0,
            max_n: p.northing + 200.0,
        };
        assert_eq!(extracts_for(&hannover, &mut |_| {}), [ni]);
    }

    #[test]
    fn rp_extract_urls() {
        let rp = ExtractSource::RheinlandPfalz;
        assert_eq!(
            rp.url(),
            "https://download.geofabrik.de/europe/germany/rheinland-pfalz-latest.osm.pbf"
        );
        assert_eq!(
            rp.state_url(),
            "https://download.geofabrik.de/europe/germany/rheinland-pfalz-updates/state.txt"
        );
        assert_eq!(rp.label(), "Rhineland-Palatinate");
        assert_eq!(rp.megabytes(), 275.0);
        // Koblenz: the one state extract.
        let p = crate::crs::utm33_forward(crate::crs::LatLon {
            lat: 50.35964,
            lon: 7.59917,
        });
        let koblenz = BoundingBox {
            min_e: p.easting - 200.0,
            min_n: p.northing - 200.0,
            max_e: p.easting + 200.0,
            max_n: p.northing + 200.0,
        };
        assert_eq!(extracts_for(&koblenz, &mut |_| {}), [rp]);
    }

    #[test]
    fn nrw_extracts_by_regierungsbezirk() {
        // Hildegard-von-Bingen-Gymnasium Koeln (25833 E -66680 / N 5671681): Koeln only.
        let school = BoundingBox {
            min_e: -66_880.0,
            min_n: 5_671_481.0,
            max_e: -66_480.0,
            max_n: 5_671_881.0,
        };
        let koeln = ExtractSource::NordrheinWestfalen(GovernmentDistrict::Koeln);
        assert_eq!(extracts_for(&school, &mut |_| {}), [koeln]);
        assert_eq!(
            koeln.url(),
            "https://download.geofabrik.de/europe/germany/nordrhein-westfalen/koeln-regbez-latest.osm.pbf"
        );
        assert_eq!(
            koeln.state_url(),
            "https://download.geofabrik.de/europe/germany/nordrhein-westfalen/koeln-regbez-updates/state.txt"
        );
        assert_eq!(koeln.label(), "North Rhine-Westphalia, Köln");
        assert_eq!(koeln.megabytes(), 225.2);
        for (district, slug) in [
            (GovernmentDistrict::Arnsberg, "arnsberg"),
            (GovernmentDistrict::Detmold, "detmold"),
            (GovernmentDistrict::Duesseldorf, "duesseldorf"),
            (GovernmentDistrict::Koeln, "koeln"),
            (GovernmentDistrict::Muenster, "muenster"),
        ] {
            let source = ExtractSource::NordrheinWestfalen(district);
            let base = format!(
                "https://download.geofabrik.de/europe/germany/nordrhein-westfalen/{slug}-regbez"
            );
            assert_eq!(source.url(), format!("{base}-latest.osm.pbf"));
            assert_eq!(source.state_url(), format!("{base}-updates/state.txt"));
            assert_eq!(source.prefix(), format!("nrw-{slug}_"));
        }
        // Places well inside the other four districts read only their own extract.
        // City centres of Arnsberg, Bielefeld, Duesseldorf and Muenster (25833).
        for (e, n, district) in [
            (17_721.0, 5_716_725.0, GovernmentDistrict::Arnsberg),
            (56_406.0, 5_783_147.0, GovernmentDistrict::Detmold),
            (-73_850.0, 5_707_053.0, GovernmentDistrict::Duesseldorf),
            (-6_319.0, 5_782_536.0, GovernmentDistrict::Muenster),
        ] {
            let bbox = BoundingBox {
                min_e: e - 200.0,
                min_n: n - 200.0,
                max_e: e + 200.0,
                max_n: n + 200.0,
            };
            assert_eq!(
                extracts_for(&bbox, &mut |_| {}),
                [ExtractSource::NordrheinWestfalen(district)],
                "{e} {n}"
            );
        }
        // The extract is cached as nrw-koeln_<date>.osm.pbf.
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(koeln.state_url(), state("2026-11-06"));
        fetcher.insert(koeln.url(), std::fs::read(FIXTURE_PBF).unwrap());
        let mut messages = Vec::new();
        fetch_extract(&fetcher, &cache, koeln, FetchMode::Online, &mut |m| {
            messages.push(m.to_string())
        })
        .unwrap();
        assert_eq!(extracts(&cache), ["nrw-koeln_2026-11-06.osm.pbf"]);
        assert!(
            messages
                .iter()
                .any(|m| m.contains("koeln-regbez-latest.osm.pbf (about 225.2 MB)")),
            "{messages:?}"
        );
        assert_eq!(
            cached_extract_date(&cache, koeln),
            "2026-11-06".parse().ok()
        );
        // A neighbouring district's cache file is not taken for Koeln's.
        let detmold = ExtractSource::NordrheinWestfalen(GovernmentDistrict::Detmold);
        assert_eq!(cached_extract_date(&cache, detmold), None);
    }

    #[test]
    fn nearest_district_backs_up_the_margin() {
        // 7 km into Belgium west of Aachen: no boundary within the margin, Koeln is nearest.
        let belgium = BoundingBox {
            min_e: -135_721.0,
            min_n: 5_662_943.0,
            max_e: -135_321.0,
            max_n: 5_663_343.0,
        };
        assert!(
            GovernmentDistrict::ALL
                .iter()
                .all(|d| !within_margin(d.boundary(), &belgium))
        );
        assert_eq!(nearest_district(&belgium), Some(GovernmentDistrict::Koeln));
    }

    #[test]
    fn state_border_sliver_falls_back_to_the_nearest_district() {
        // In the Eifel on the Rhineland-Palatinate border, 25833 E -83909 / N 5619611: the region
        // margin reaches a sliver where the state boundary pokes about 30 m past Geofabrik's Koeln
        // poly, but no district boundary is within the margin. The nearest district is read, with
        // a warning.
        let sliver = BoundingBox {
            min_e: -83_909.0,
            min_n: 5_619_611.0,
            max_e: -83_909.0,
            max_n: 5_619_611.0,
        };
        // Rhineland-Palatinate lies on the other side of that border, so it is read too.
        assert_eq!(
            regions_for(&sliver),
            [Region::NordrheinWestfalen, Region::RheinlandPfalz]
        );
        assert!(
            GovernmentDistrict::ALL
                .iter()
                .all(|d| !within_margin(d.boundary(), &sliver))
        );
        let mut messages = Vec::new();
        assert_eq!(
            extracts_for(&sliver, &mut |m| messages.push(m.to_string())),
            [
                ExtractSource::NordrheinWestfalen(GovernmentDistrict::Koeln),
                ExtractSource::RheinlandPfalz
            ]
        );
        assert_eq!(
            messages,
            [
                "warning: no North Rhine-Westphalia district extract within 100 m of the area; \
              reading the nearest, North Rhine-Westphalia, Köln"
            ]
        );
        // Inside a district no warning is given.
        let mut quiet = Vec::new();
        let school = BoundingBox {
            min_e: -66_680.0,
            min_n: 5_671_681.0,
            max_e: -66_680.0,
            max_n: 5_671_681.0,
        };
        extracts_for(&school, &mut |m| quiet.push(m.to_string()));
        assert!(quiet.is_empty(), "{quiet:?}");
    }

    #[test]
    fn corrupt_district_extract_names_its_url() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        let koeln = ExtractSource::NordrheinWestfalen(GovernmentDistrict::Koeln);
        fetcher.insert(koeln.state_url(), state("2026-11-06"));
        fetcher.insert(koeln.url(), b"<html>portal</html>".to_vec());
        let err =
            fetch_extract(&fetcher, &cache, koeln, FetchMode::Online, &mut |_| {}).unwrap_err();
        let text = err.to_string();
        assert!(
            text.contains("koeln-regbez-latest.osm.pbf is not a complete OSM extract"),
            "{text}"
        );
        assert!(!text.contains("brandenburg"), "{text}");
        assert!(extracts(&cache).is_empty(), "{:?}", extracts(&cache));
    }

    fn extracts(cache: &Cache) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(cache.osm_dir())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn school_box() -> GeoBox {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/clara-zetkin-schule.geojson"
        ))
        .unwrap();
        crate::area::MapArea::from_geojson(&text)
            .unwrap()
            .geo_box(20.0)
    }

    fn has(tags: &[(String, String)], key: &str, value: &str) -> bool {
        tags.iter().any(|(k, v)| k == key && v == value)
    }

    fn candidates_wanted(tags: &[(String, String)]) -> bool {
        has(tags, "emergency", "fire_hydrant")
            || has(tags, "amenity", "bench")
            || has(tags, "leisure", "pitch")
    }

    fn read_school(wanted: TagFilter<'_>) -> Vec<OsmFeature> {
        read_extract(Path::new(FIXTURE_PBF), &school_box(), wanted).unwrap()
    }

    #[test]
    fn fixture_has_school_hydrant() {
        let features = read_school(&candidates_wanted);
        let hydrant = features
            .iter()
            .find(|f| f.id == "node/9669755236")
            .expect("hydrant");
        let OsmGeometry::Point(p) = &hydrant.geometry else {
            panic!("{hydrant:?}")
        };
        assert!(
            (p.lat - 52.5396417).abs() < 1e-6 && (p.lon - 13.845286).abs() < 1e-6,
            "{p:?}"
        );
    }

    #[test]
    fn fixture_benches_as_nodes_and_ways() {
        // The bench ways stand in the wood north-west of the school: use the whole fixture box.
        let fixture_box = GeoBox {
            min_lat: 52.5378,
            min_lon: 13.8405,
            max_lat: 52.5425,
            max_lon: 13.8470,
        };
        let features =
            read_extract(Path::new(FIXTURE_PBF), &fixture_box, &candidates_wanted).unwrap();
        let bench = |id: &str| features.iter().find(|f| f.id == id);
        assert!(matches!(
            bench("node/9105117049").map(|f| &f.geometry),
            Some(OsmGeometry::Point(_))
        ));
        let way = bench("way/1274507364").expect("bench way");
        assert!(
            matches!(&way.geometry, OsmGeometry::Line(points) if points.len() == 2),
            "{way:?}"
        );
        let pitch = bench("way/1028507308").expect("pitch");
        let OsmGeometry::Line(ring) = &pitch.geometry else {
            panic!("{pitch:?}")
        };
        assert_eq!(ring.len(), 5);
        assert_eq!(ring.first(), ring.last(), "closed way");
    }

    #[test]
    fn way_crossing_the_box_is_complete() {
        let area = school_box();
        let outside = |p: &crate::crs::LatLon| {
            p.lat < area.min_lat
                || p.lat > area.max_lat
                || p.lon < area.min_lon
                || p.lon > area.max_lon
        };
        let roads =
            read_school(&|tags: &[(String, String)]| tags.iter().any(|(k, _)| k == "highway"));
        let crossing = roads.iter().find(|f| match &f.geometry {
            OsmGeometry::Line(points) => points.iter().any(outside) && !points.iter().all(outside),
            OsmGeometry::Point(_) => false,
        });
        assert!(
            crossing.is_some(),
            "a road leaving the box keeps its outer nodes"
        );
    }

    #[test]
    fn untagged_or_unwanted_elements_are_left_out() {
        let features = read_school(&candidates_wanted);
        assert!(
            features.iter().all(|f| candidates_wanted(&f.tags)),
            "{features:?}"
        );
        assert!(
            !features.iter().any(|f| f.id == "node/12293219862"),
            "lamp not wanted"
        );
    }

    #[test]
    fn geojson_roundtrip() {
        let features = read_school(&candidates_wanted);
        let back = features_from_geojson(&features_to_geojson(&features)).unwrap();
        assert_eq!(back, features);
    }

    #[test]
    #[ignore = "network; downloads about 300 MB unless cached"]
    fn live_osm_extract() {
        let cache = Cache::default_location().unwrap();
        let fetcher = crate::fetch::HttpFetcher::new();
        let extract = fetch_extract(
            &fetcher,
            &cache,
            ExtractSource::Brandenburg,
            FetchMode::Online,
            &mut |m| eprintln!("{m}"),
        )
        .unwrap();
        let features = read_extract(&extract.path, &school_box(), &candidates_wanted).unwrap();
        assert!(
            features.iter().any(|f| f.id == "way/1028507308"),
            "school pitch"
        );
    }

    #[test]
    fn state_date_parses() {
        let text = String::from_utf8(state("2026-11-06")).unwrap();
        assert_eq!(parse_state_date(&text), "2026-11-06".parse().ok());
        assert_eq!(parse_state_date("sequenceNumber=1"), None);
    }

    #[test]
    fn extract_downloaded_then_reused() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-11-06");
        let first = online(&fetcher, &cache).unwrap();
        assert_eq!(first.date.to_string(), "2026-11-06");
        assert_eq!(extracts(&cache), ["brandenburg_2026-11-06.osm.pbf"]);
        assert_eq!(fetcher.calls.get(), 2);
        let again = online(&fetcher, &cache).unwrap();
        assert_eq!(again, first);
        assert_eq!(fetcher.calls.get(), 3, "only the state is fetched again");
    }

    #[test]
    fn extract_within_seven_days_is_reused() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-11-01");
        online(&fetcher, &cache).unwrap();
        fetcher.insert(STATE_URL, state("2026-11-08"));
        let extract = online(&fetcher, &cache).unwrap();
        assert_eq!(extract.date.to_string(), "2026-11-01");
        assert_eq!(extracts(&cache), ["brandenburg_2026-11-01.osm.pbf"]);
    }

    #[test]
    fn older_extract_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-10-01");
        online(&fetcher, &cache).unwrap();
        fetcher.insert(STATE_URL, state("2026-11-06"));
        let extract = online(&fetcher, &cache).unwrap();
        assert_eq!(extract.date.to_string(), "2026-11-06");
        assert_eq!(extracts(&cache), ["brandenburg_2026-11-06.osm.pbf"]);
    }

    #[test]
    fn unreachable_state_uses_cached_extract() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-11-06");
        let cached = online(&fetcher, &cache).unwrap();
        fetcher.failing.borrow_mut().push(STATE_URL.to_string());
        let mut messages = Vec::new();
        let extract = fetch_extract(
            &fetcher,
            &cache,
            ExtractSource::Brandenburg,
            FetchMode::Online,
            &mut |m| messages.push(m.to_string()),
        )
        .unwrap();
        assert_eq!(extract, cached);
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("warning: Geofabrik unavailable")),
            "{messages:?}"
        );
        // Without a cached extract the error stands.
        let empty = Cache::new(dir.path().join("empty"));
        assert!(
            fetch_extract(
                &fetcher,
                &empty,
                ExtractSource::Brandenburg,
                FetchMode::Online,
                &mut |_| {}
            )
            .is_err()
        );
    }

    #[test]
    fn state_without_timestamp_or_failed_download_use_cached_extract() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-10-01");
        let cached = online(&fetcher, &cache).unwrap();
        // A captive portal answers the state request.
        fetcher.insert(STATE_URL, b"<html>login</html>".to_vec());
        assert_eq!(online(&fetcher, &cache).unwrap(), cached);
        // A newer extract exists, but its download fails.
        fetcher.insert(STATE_URL, state("2026-11-06"));
        fetcher.failing.borrow_mut().push(EXTRACT_URL.to_string());
        let mut messages = Vec::new();
        let extract = fetch_extract(
            &fetcher,
            &cache,
            ExtractSource::Brandenburg,
            FetchMode::Online,
            &mut |m| messages.push(m.to_string()),
        )
        .unwrap();
        assert_eq!(extract, cached);
        assert!(
            messages.iter().any(|m| m.starts_with("warning: ")),
            "{messages:?}"
        );
    }

    #[test]
    fn offline_uses_newest_extract() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-11-06");
        let online_extract = online(&fetcher, &cache).unwrap();
        let calls = fetcher.calls.get();
        let offline = fetch_extract(
            &fetcher,
            &cache,
            ExtractSource::Brandenburg,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(offline, online_extract);
        assert_eq!(fetcher.calls.get(), calls);
    }

    #[test]
    fn offline_without_extract_fails() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let err = fetch_extract(
            &MapFetcher::default(),
            &cache,
            ExtractSource::Brandenburg,
            FetchMode::Offline,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(matches!(err, FetchError::OsmExtractMissing), "{err:?}");
        assert!(err.to_string().contains("freemapper fetch"), "{err}");
    }

    #[test]
    fn way_segment_crossing_the_box_counts() {
        let area = GeoBox {
            min_lat: 52.0,
            min_lon: 13.0,
            max_lat: 52.1,
            max_lon: 13.1,
        };
        let p = |lat, lon| LatLon { lat, lon };
        // A straight fence passing through the box, both vertices outside it.
        assert!(touches(&[p(52.05, 12.9), p(52.05, 13.2)], &area));
        assert!(touches(&[p(52.05, 13.05), p(52.5, 13.5)], &area));
        assert!(!touches(&[p(52.2, 12.9), p(52.2, 13.2)], &area));
    }

    #[test]
    fn extract_date_comes_from_the_header() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        // Geofabrik's state is a few days older than the file's own header (2026-11-06).
        serve(&fetcher, "2026-11-03");
        let extract = online(&fetcher, &cache).unwrap();
        assert_eq!(
            extract.date.to_string(),
            "2026-11-03",
            "cache key from the state"
        );
        assert_eq!(
            extract.data_date.to_string(),
            "2026-11-06",
            "currency from the header"
        );
        let cached = online(&fetcher, &cache).unwrap();
        assert_eq!(cached.data_date, extract.data_date);
    }

    #[test]
    fn stale_partial_extract_is_removed() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        std::fs::create_dir_all(cache.osm_dir()).unwrap();
        let stale = cache.osm_dir().join("brandenburg_2026-10-01.osm.pbf.part");
        std::fs::write(&stale, b"partial").unwrap();
        let file = std::fs::File::options().write(true).open(&stale).unwrap();
        file.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(7200))
            .unwrap();
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-11-06");
        online(&fetcher, &cache).unwrap();
        assert!(!stale.exists());
    }

    #[test]
    fn download_reports_its_size() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        serve(&fetcher, "2026-11-06");
        let mut messages = Vec::new();
        fetch_extract(
            &fetcher,
            &cache,
            ExtractSource::Brandenburg,
            FetchMode::Online,
            &mut |m| messages.push(m.to_string()),
        )
        .unwrap();
        assert!(
            messages
                .iter()
                .any(|m| m.starts_with("downloaded ") && m.ends_with(" MB")),
            "{messages:?}"
        );
    }

    #[test]
    fn corrupt_extract_names_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let broken = dir.path().join("brandenburg_2026-11-06.osm.pbf");
        std::fs::write(&broken, b"not a pbf").unwrap();
        let err = read_extract(&broken, &school_box(), &candidates_wanted).unwrap_err();
        assert!(
            err.to_string().contains("brandenburg_2026-11-06.osm.pbf"),
            "{err}"
        );
    }

    #[test]
    fn failed_download_leaves_no_extract() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().to_path_buf());
        let fetcher = MapFetcher::default();
        fetcher.insert(STATE_URL, state("2026-11-06"));
        fetcher.insert(EXTRACT_URL, b"<html>not a pbf</html>".to_vec());
        let err = online(&fetcher, &cache).unwrap_err();
        assert!(matches!(err, FetchError::CorruptExtract { .. }), "{err:?}");
        assert!(extracts(&cache).is_empty(), "{:?}", extracts(&cache));
    }
}
