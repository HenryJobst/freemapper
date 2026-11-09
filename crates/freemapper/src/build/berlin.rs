//! Berlin ALKIS structures and building parts → candidate objects (Berlin plus 1).
//!
//! The Geoportal features run through the same [`Rules`] engine as the OSM features
//! (`assets/rules/berlin-candidates.toml`). A feature becomes a tagged feature first: its string
//! and number properties become `berlin:<name>` tags, so no OSM rule can match one and no Berlin
//! rule an OSM tag. The building parts are parsed from the stored buildings in one place
//! ([`building_part_features`]).

use std::collections::HashMap;

use chrono::NaiveDate;
use geo::{BoundingRect as _, Centroid as _};
use serde_json::Value;

use crate::build::candidates::{Rules, Target, osm_objects};
use crate::crs::{Crs, LatLon, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::sources::osm::{OsmFeature, OsmGeometry};

/// Layer name of the building parts (`Durchfahrt im Gebäude` …) in `berlin:layer`.
pub const BUILDING_PARTS_LAYER: &str = "building-parts";
/// A monument area with a side longer than this is tagged `freemapper:size=large`: the mapper
/// can draw its outline instead of the 526 point.
const LARGE_MONUMENT_M: f64 = 3.0;
/// An OSM `natural=tree` this close to a cadastre tree is the same tree and is dropped.
const TREE_DEDUP_M: f64 = 2.0;
/// An OSM line candidate with at least this share of its length within [`LINE_DEDUP_M`] of an
/// ALKIS line with the same symbol is the same object and is dropped.
const LINE_DEDUP_SHARE: f64 = 0.8;
const LINE_DEDUP_M: f64 = 1.5;
/// The OSM line is measured in pieces of at most this length (ground metres).
const LINE_DEDUP_STEP_M: f64 = 0.5;
/// Fence, wall, retaining wall and hedge: the line symbols both sources give.
const LINE_DEDUP_CODES: [&str; 4] = ["515", "513.2", "518", "410.1"];
/// The tree cadastre layers.
const TREE_LAYERS: [&str; 2] = ["strassenbaeume", "anlagenbaeume"];
/// The CRS of all Geoportal Berlin layers.
const LAYER_CRS: Crs = Crs::Utm33;

/// One stored layer for [`berlin_objects`].
pub struct LayerInput<'a> {
    /// Layer file name (`bauwerke-linien`) or [`BUILDING_PARTS_LAYER`].
    pub file: &'a str,
    /// GeoJSON features in EPSG:25833.
    pub features: &'a [Value],
    /// The fetch date, which stands in for the currency (the layers carry no feature dates).
    pub currency: NaiveDate,
}

/// The building parts (`AX_Bauteil` with a `bauart`) among the normalised stored buildings.
pub fn building_part_features(buildings: &[Value]) -> Vec<Value> {
    buildings
        .iter()
        .filter(|f| {
            f["properties"]["gebnutzbez"] == "Bauteil" && f["properties"]["bauart"].is_string()
        })
        .cloned()
        .collect()
}

fn position(c: &Value) -> Option<(f64, f64)> {
    Some((c.get(0)?.as_f64()?, c.get(1)?.as_f64()?))
}

fn latlon((easting, northing): (f64, f64)) -> Option<LatLon> {
    LAYER_CRS.inverse(ProjectedPoint { easting, northing })
}

fn line_of(coordinates: &Value) -> Option<Vec<LatLon>> {
    coordinates
        .as_array()?
        .iter()
        .map(|c| position(c).and_then(latlon))
        .collect()
}

/// The exterior ring of a polygon as a closed line (holes are dropped: candidates only) and a
/// point at its centroid. A ring longer than [`LARGE_MONUMENT_M`] sets `large`.
fn polygon_parts(rings: &Value) -> Option<(Vec<LatLon>, LatLon, bool)> {
    let exterior = rings.get(0)?.as_array()?;
    let points: Vec<(f64, f64)> = exterior.iter().filter_map(position).collect();
    let polygon = geo_types::Polygon::new(geo_types::LineString::from(points.clone()), Vec::new());
    let centroid = polygon.centroid()?;
    let rect = polygon.bounding_rect()?;
    let large = rect.width().max(rect.height()) > LARGE_MONUMENT_M;
    let ring = points.iter().map(|p| latlon(*p)).collect::<Option<_>>()?;
    Some((ring, latlon((centroid.x(), centroid.y()))?, large))
}

/// The polygons of an area feature (Polygon or MultiPolygon, holes kept) in the map's `crs`; the
/// layers are in EPSG:25833. Empty for other geometries. Used by the canopy and stair builders,
/// which need the area itself and not a rule's line or point.
pub(crate) fn feature_polygons(feature: &Value, crs: Crs) -> Vec<geo_types::Polygon<f64>> {
    let to_map = |(easting, northing): (f64, f64)| -> Option<(f64, f64)> {
        if crs == LAYER_CRS {
            return Some((easting, northing));
        }
        let p = crs.forward(latlon((easting, northing))?);
        Some((p.easting, p.northing))
    };
    let ring = |coordinates: &Value| -> Option<geo_types::LineString<f64>> {
        let points: Option<Vec<(f64, f64)>> = coordinates
            .as_array()?
            .iter()
            .map(|c| position(c).and_then(to_map))
            .collect();
        Some(geo_types::LineString::from(points?))
    };
    let polygon = |rings: &Value| -> Option<geo_types::Polygon<f64>> {
        let mut rings = rings.as_array()?.iter().map(ring);
        let exterior = rings.next()??;
        Some(geo_types::Polygon::new(
            exterior,
            rings.collect::<Option<_>>()?,
        ))
    };
    let coordinates = &feature["geometry"]["coordinates"];
    match feature["geometry"]["type"].as_str() {
        Some("Polygon") => polygon(coordinates).into_iter().collect(),
        Some("MultiPolygon") => coordinates
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(polygon)
            .collect(),
        _ => Vec::new(),
    }
}

/// The lines of a line feature (LineString or MultiLineString, one per part) in the map's `crs`;
/// the layers are in EPSG:25833. Empty for other geometries. Used by the stair builder for the
/// `Freitreppe` step edges.
pub(crate) fn feature_lines(feature: &Value, crs: Crs) -> Vec<geo_types::LineString<f64>> {
    let line = |coordinates: &Value| -> Option<geo_types::LineString<f64>> {
        let points: Option<Vec<(f64, f64)>> = coordinates
            .as_array()?
            .iter()
            .map(|c| {
                let (easting, northing) = position(c)?;
                if crs == LAYER_CRS {
                    return Some((easting, northing));
                }
                let p = crs.forward(latlon((easting, northing))?);
                Some((p.easting, p.northing))
            })
            .collect();
        Some(geo_types::LineString::from(points?))
    };
    let coordinates = &feature["geometry"]["coordinates"];
    match feature["geometry"]["type"].as_str() {
        Some("LineString") => line(coordinates).into_iter().collect(),
        Some("MultiLineString") => coordinates
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(line)
            .collect(),
        _ => Vec::new(),
    }
}

/// The provenance tags of a map object or candidate built from a Berlin feature directly (not
/// through the rules), in the order [`osm_objects`] writes them: `freemapper:source`,
/// `freemapper:source_id`, `freemapper:currency`, `freemapper:alkis`.
pub(crate) fn provenance_tags(
    layer: &str,
    feature: &Value,
    currency: NaiveDate,
) -> Vec<(String, String)> {
    let own = feature_tags(layer, feature);
    let own = |key: &str| own.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
    let mut tags = vec![(
        "freemapper:source".to_string(),
        format!("geoportal-berlin/{layer}"),
    )];
    if let Some(id) = own("freemapper:source_id") {
        tags.push(("freemapper:source_id".to_string(), id));
    }
    tags.push(("freemapper:currency".to_string(), currency.to_string()));
    if let Some(alkis) = own("freemapper:alkis") {
        tags.push(("freemapper:alkis".to_string(), alkis));
    }
    tags
}

/// The Straßenbefahrung codes a candidate carries as `berlin:<code>`. There is no code list, so no
/// rule reads them; they are for the mapper (reference §14.5).
pub(crate) const SURVEY_CODES: [&str; 8] =
    ["material", "hms", "pot", "bor", "gws", "sre", "hdt", "trs"];

/// The survey date of a stored Straßenbefahrung feature (from its `gis_id`, stamped at download)
/// and whether it is guessed. A feature without one gets the dataset date, guessed.
pub(crate) fn survey_currency(feature: &Value) -> (NaiveDate, bool) {
    let properties = &feature["properties"];
    let date = properties["survey_date"]
        .as_str()
        .and_then(|d| d.parse().ok());
    let guessed = properties["survey_date_guessed"] == "yes";
    match date {
        Some(date) => (date, guessed),
        None => (crate::sources::berlin::SURVEY_DATASET_DATE, true),
    }
}

/// The tags of an object built from a Straßenbefahrung feature directly: the provenance tags
/// with the survey date as currency (and `freemapper:currency_guessed=yes` when the `gis_id`
/// gave none), the survey type as `berlin:layer` and the survey codes.
pub(crate) fn survey_tags(file: &str, feature: &Value) -> Vec<(String, String)> {
    let (currency, guessed) = survey_currency(feature);
    let mut tags = provenance_tags(file, feature, currency);
    if guessed {
        tags.push(("freemapper:currency_guessed".to_string(), "yes".to_string()));
    }
    let codes = SURVEY_CODES.map(|code| format!("berlin:{code}"));
    tags.extend(
        feature_tags(file, feature)
            .into_iter()
            .filter(|(k, _)| k == "berlin:layer" || codes.contains(k)),
    );
    tags
}

/// The tags of a Berlin feature: `berlin:layer`, each string (not empty) and number property as
/// `berlin:<name>`, and the provenance tags the objects carry on (`freemapper:source_id`, the
/// ALKIS name as `freemapper:alkis`).
fn feature_tags(layer: &str, feature: &Value) -> Vec<(String, String)> {
    let properties = feature["properties"].as_object();
    // A file with several WFS types names the type per feature.
    let layer = properties
        .and_then(|p| p.get(crate::sources::berlin::TYPE_KEY))
        .and_then(Value::as_str)
        .unwrap_or(layer);
    let mut tags = vec![("berlin:layer".to_string(), layer.to_string())];
    for (key, value) in properties.into_iter().flatten() {
        if key == crate::sources::berlin::TYPE_KEY || key == "layer" {
            continue;
        }
        let text = match value {
            // `NULL` is the WFS's word for "no value".
            Value::String(s) if !s.is_empty() && s != "NULL" => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => continue,
        };
        tags.push((format!("berlin:{key}"), text));
    }
    // A survey feature carries its own date, and a guessed one says so.
    if properties.is_some_and(|p| p.contains_key("survey_date")) {
        let (date, guessed) = survey_currency(feature);
        tags.push(("freemapper:currency".to_string(), date.to_string()));
        if guessed {
            tags.push(("freemapper:currency_guessed".to_string(), "yes".to_string()));
        }
    }
    let property = |key: &str| properties.and_then(|p| p.get(key)).and_then(Value::as_str);
    let source_id = ["uuid", "gisid", "oid", "gis_id"]
        .iter()
        .find_map(|key| property(key).filter(|v| !v.is_empty()));
    if let Some(id) = source_id {
        tags.push(("freemapper:source_id".to_string(), id.to_string()));
    }
    let mut alkis: Vec<&str> = Vec::new();
    for key in ["bezbwf", "bezart", "bezbws", "bauart"] {
        if let Some(name) = property(key).filter(|v| !v.is_empty())
            && !alkis.contains(&name)
        {
            alkis.push(name);
        }
    }
    if !alkis.is_empty() {
        tags.push(("freemapper:alkis".to_string(), alkis.join("; ")));
    }
    if TREE_LAYERS.contains(&layer)
        && let Some(tree) = tree_label(properties)
    {
        tags.push(("freemapper:baum".to_string(), tree));
    }
    tags.sort();
    tags
}

/// What a tree candidate says about the tree: `Tilia cordata, Krone 12 m` (botanical name, else
/// the German one; crown diameter in metres). `None` when the cadastre gives neither.
fn tree_label(properties: Option<&serde_json::Map<String, Value>>) -> Option<String> {
    let text = |key: &str| {
        properties?
            .get(key)?
            .as_str()
            .map(str::trim)
            .filter(|v| !v.is_empty())
    };
    let species = text("art_bot").or_else(|| text("art_dtsch"));
    let crown = properties?
        .get("kronedurch")?
        .as_f64()
        .filter(|c| *c > 0.0)
        .map(|c| format!("Krone {c} m"));
    match (species, crown) {
        (Some(species), Some(crown)) => Some(format!("{species}, {crown}")),
        (Some(only), None) => Some(only.to_string()),
        (None, crown) => crown,
    }
}

/// The tagged features of one layer, ready for the rules: a point stays a point, each part of a
/// MultiLineString is a line, each polygon is its exterior as a closed line plus a point at its
/// centroid (a rule picks one by its `draw`). Id: `berlin/<layer>/<GeoServer id>`.
pub fn tagged_features(layer: &str, features: &[Value]) -> Vec<OsmFeature> {
    let mut out = Vec::new();
    for feature in features {
        let id = format!(
            "berlin/{layer}/{}",
            feature["id"].as_str().unwrap_or_default()
        );
        let tags = feature_tags(layer, feature);
        let coordinates = &feature["geometry"]["coordinates"];
        let mut push = |tags: Vec<(String, String)>, geometry| {
            out.push(OsmFeature {
                id: id.clone(),
                tags,
                geometry,
            });
        };
        let polygon = |rings: &Value, push: &mut dyn FnMut(Vec<(String, String)>, OsmGeometry)| {
            let Some((ring, centre, large)) = polygon_parts(rings) else {
                return;
            };
            let mut point_tags = tags.clone();
            if large {
                point_tags.push(("freemapper:size".to_string(), "large".to_string()));
                point_tags.sort();
            }
            push(tags.clone(), OsmGeometry::Line(ring));
            push(point_tags, OsmGeometry::Point(centre));
        };
        match feature["geometry"]["type"].as_str() {
            Some("Point") => {
                if let Some(p) = position(coordinates).and_then(latlon) {
                    push(tags.clone(), OsmGeometry::Point(p));
                }
            }
            Some("LineString") => {
                if let Some(line) = line_of(coordinates) {
                    push(tags.clone(), OsmGeometry::Line(line));
                }
            }
            Some("MultiLineString") => {
                for line in coordinates.as_array().into_iter().flatten() {
                    if let Some(line) = line_of(line) {
                        push(tags.clone(), OsmGeometry::Line(line));
                    }
                }
            }
            Some("Polygon") => polygon(coordinates, &mut push),
            Some("MultiPolygon") => {
                for rings in coordinates.as_array().into_iter().flatten() {
                    polygon(rings, &mut push);
                }
            }
            _ => {}
        }
    }
    out
}

/// The candidate objects of the stored Berlin layers within `buffer_m` of `map_area`
/// (EPSG:25833), each tagged `freemapper:source=geoportal-berlin/<layer>`.
pub fn berlin_objects(
    layers: &[LayerInput<'_>],
    kind: MapKind,
    map_area: &geo_types::Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let rules = Rules::berlin();
    let mut objects = Vec::new();
    for layer in layers {
        objects.extend(osm_objects(
            &tagged_features(layer.file, layer.features),
            &rules,
            Target::Candidates,
            kind,
            map_area,
            buffer_m,
            georef,
            symbol_set,
            &format!("geoportal-berlin/{}", layer.file),
            layer.currency,
        )?);
    }
    Ok(objects)
}

/// Drops the OSM fence, wall, retaining wall and hedge candidates (515, 513.2, 518, 410.1) with
/// at least [`LINE_DEDUP_SHARE`] of their length within [`LINE_DEDUP_M`] of a Berlin (ALKIS)
/// line candidate with the same symbol: the cadastre surveyed them, so ALKIS wins, as for trees.
/// The ALKIS object nearest to most of the dropped line names it in `freemapper:also`
/// (`osm/way/…`, several joined by `; `), so a difference in currency stays visible.
pub fn drop_osm_lines_near_alkis(
    osm: Vec<MapObject>,
    berlin: &mut [MapObject],
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> Vec<MapObject> {
    drop_lines_near(osm, berlin, |_| true, symbol_set, scale_denominator)
}

/// The surveyed (Straßenbefahrung) lines among the Berlin objects: `geoportal-berlin/sb-…`.
fn from_survey(object: &MapObject) -> bool {
    object
        .tags
        .iter()
        .any(|(k, v)| k == "freemapper:source" && v.starts_with("geoportal-berlin/sb-"))
}

/// ALKIS wins over the survey: the surveyed fence, wall, retaining wall and hedge lines
/// (`sb-mauer`, `sb-gelaender`) with at least [`LINE_DEDUP_SHARE`] of their length within
/// [`LINE_DEDUP_M`] of an ALKIS line with the same symbol are dropped, and the ALKIS line names
/// them in `freemapper:also` (`geoportal-berlin/sb-…/<gis_id>`). The order of the sources is
/// ALKIS, survey, OSM: run [`drop_osm_lines_near_alkis`] on the result.
pub fn drop_survey_lines_near_alkis(
    berlin: Vec<MapObject>,
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> Vec<MapObject> {
    let (survey, mut alkis): (Vec<_>, Vec<_>) = berlin.into_iter().partition(from_survey);
    let mut kept = drop_lines_near(
        survey,
        &mut alkis,
        |o| !from_survey(o),
        symbol_set,
        scale_denominator,
    );
    alkis.append(&mut kept);
    alkis
}

/// Drops the `candidates` lines (fence, wall, retaining wall, hedge symbols) that follow a
/// `winners` line with the same symbol, and names them in the winner's `freemapper:also`.
/// `winners` are the Berlin objects for which `is_winner` holds.
fn drop_lines_near(
    osm: Vec<MapObject>,
    berlin: &mut [MapObject],
    is_winner: impl Fn(&MapObject) -> bool,
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> Vec<MapObject> {
    let codes: Vec<u32> = LINE_DEDUP_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let line = |o: &MapObject| {
        matches!(o.kind, MapObjectKind::Path) && o.coords.len() >= 2 && codes.contains(&o.symbol_id)
    };
    let from_berlin = |o: &MapObject| {
        is_winner(o)
            && o.tags
                .iter()
                .any(|(k, v)| k == "freemapper:source" && v.starts_with("geoportal-berlin/"))
    };
    let xy = |c: &crate::omap::MapCoord| (f64::from(c.x), f64::from(c.y));
    // Map coordinates are 1/1000 mm on paper: one ground metre is 10^6 / scale of them.
    let metre = 1e6 / f64::from(scale_denominator);
    let limit = LINE_DEDUP_M * metre;
    let cell = |(x, y): (f64, f64)| ((x / limit).floor() as i64, (y / limit).floor() as i64);
    // The ALKIS segments by symbol, in a grid of `limit`-sized cells along each segment.
    type Segment = (usize, (f64, f64), (f64, f64));
    let mut segments: Vec<(u32, Segment)> = Vec::new();
    let mut grid: HashMap<(u32, i64, i64), Vec<usize>> = HashMap::new();
    for (index, object) in berlin.iter().enumerate() {
        if !(line(object) && from_berlin(object)) {
            continue;
        }
        for pair in object.coords.windows(2) {
            let (a, b) = (xy(&pair[0]), xy(&pair[1]));
            let id = segments.len();
            segments.push((object.symbol_id, (index, a, b)));
            let steps = ((b.0 - a.0).hypot(b.1 - a.1) / (limit / 2.0))
                .ceil()
                .max(1.0) as usize;
            let mut cells: Vec<(i64, i64)> = (0..=steps)
                .map(|i| {
                    let t = i as f64 / steps as f64;
                    cell((a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1)))
                })
                .collect();
            cells.dedup();
            for (cx, cy) in cells {
                let list = grid.entry((object.symbol_id, cx, cy)).or_default();
                if list.last() != Some(&id) {
                    list.push(id);
                }
            }
        }
    }
    if segments.is_empty() {
        return osm;
    }
    let distance = |p: (f64, f64), a: (f64, f64), b: (f64, f64)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length2 = dx * dx + dy * dy;
        let t = if length2 > 0.0 {
            (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (p.0 - a.0 - t * dx).hypot(p.1 - a.1 - t * dy)
    };
    // The ALKIS object nearest to `p` within `limit`, among the segments with `symbol`. A segment
    // within `limit` of `p` has a sample cell within two cells of `p`'s.
    let nearest = |symbol: u32, p: (f64, f64)| -> Option<usize> {
        let (cx, cy) = cell(p);
        let mut best: Option<(f64, usize)> = None;
        for i in cx - 2..=cx + 2 {
            for j in cy - 2..=cy + 2 {
                for &id in grid.get(&(symbol, i, j)).into_iter().flatten() {
                    let (_, (index, a, b)) = segments[id];
                    let d = distance(p, a, b);
                    if d <= limit && best.is_none_or(|(e, _)| d < e) {
                        best = Some((d, index));
                    }
                }
            }
        }
        best.map(|(_, index)| index)
    };
    let step = LINE_DEDUP_STEP_M * metre;
    let mut kept = Vec::with_capacity(osm.len());
    for object in osm {
        if !line(&object) {
            kept.push(object);
            continue;
        }
        // Length near each ALKIS object, from the middle of pieces at most `step` long.
        let mut near: HashMap<usize, f64> = HashMap::new();
        let mut total = 0.0;
        for pair in object.coords.windows(2) {
            let (a, b) = (xy(&pair[0]), xy(&pair[1]));
            let length = (b.0 - a.0).hypot(b.1 - a.1);
            let pieces = (length / step).ceil().max(1.0) as usize;
            let piece = length / pieces as f64;
            total += length;
            for i in 0..pieces {
                let t = (i as f64 + 0.5) / pieces as f64;
                let p = (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1));
                if let Some(index) = nearest(object.symbol_id, p) {
                    *near.entry(index).or_default() += piece;
                }
            }
        }
        let covered: f64 = near.values().sum();
        if total <= 0.0 || covered < LINE_DEDUP_SHARE * total {
            kept.push(object);
            continue;
        }
        let winner = near
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|(index, _)| index)
            .expect("some length is near");
        // `osm/way/1`, `geoportal-berlin/sb-gelaender/Li_…`: the source and the id in it.
        let tag = |key: &str| {
            object
                .tags
                .iter()
                .find(|(k, _)| k == key)
                .map_or_else(String::new, |(_, v)| v.clone())
        };
        let also = format!(
            "{}/{}",
            tag("freemapper:source"),
            tag("freemapper:source_id")
        );
        let tags = &mut berlin[winner].tags;
        match tags.iter_mut().find(|(k, _)| k == "freemapper:also") {
            Some((_, v)) if !v.split("; ").any(|x| x == also) => {
                v.push_str("; ");
                v.push_str(&also);
            }
            Some(_) => {}
            None => tags.push(("freemapper:also".to_string(), also)),
        }
    }
    kept
}

/// Drops the OSM `natural=tree` candidates within [`TREE_DEDUP_M`] of a cadastre tree (a point
/// object of a tree cadastre layer): the cadastre knows the crown, so it wins. Everything else
/// stays.
pub fn drop_osm_trees_near_cadastre(
    osm: Vec<MapObject>,
    objects: &[MapObject],
    scale_denominator: u32,
) -> Vec<MapObject> {
    let has_tag = |o: &MapObject, key: &str, test: &dyn Fn(&str) -> bool| {
        o.tags.iter().any(|(k, v)| k == key && test(v))
    };
    let is_cadastre_tree = |o: &MapObject| {
        has_tag(o, "freemapper:source", &|v| {
            v.strip_prefix("geoportal-berlin/")
                .is_some_and(|layer| TREE_LAYERS.contains(&layer))
        })
    };
    let position = |o: &MapObject| match (&o.kind, o.coords.first()) {
        (MapObjectKind::Point { .. }, Some(c)) => Some((f64::from(c.x), f64::from(c.y))),
        _ => None,
    };
    let trees: Vec<(f64, f64)> = objects
        .iter()
        .filter(|o| is_cadastre_tree(o))
        .filter_map(position)
        .collect();
    if trees.is_empty() {
        return osm;
    }
    // Map coordinates are 1/1000 mm on paper: one ground metre is 10^6 / scale of them.
    let limit = TREE_DEDUP_M * 1e6 / f64::from(scale_denominator);
    // A grid of `limit`-sized cells: a tree within `limit` lies in the same or a neighbouring cell.
    let cell = |(x, y): (f64, f64)| ((x / limit).floor() as i64, (y / limit).floor() as i64);
    let mut grid: std::collections::HashMap<(i64, i64), Vec<(f64, f64)>> =
        std::collections::HashMap::new();
    for tree in trees {
        grid.entry(cell(tree)).or_default().push(tree);
    }
    osm.into_iter()
        .filter(|o| {
            let osm_tree = has_tag(o, "freemapper:osm", &|v| v == "natural=tree");
            let near = position(o).is_some_and(|p @ (x, y)| {
                let (cx, cy) = cell(p);
                (cx - 1..=cx + 1).any(|i| {
                    (cy - 1..=cy + 1).any(|j| {
                        grid.get(&(i, j)).is_some_and(|ts| {
                            ts.iter().any(|(tx, ty)| (x - tx).hypot(y - ty) <= limit)
                        })
                    })
                })
            });
            !(osm_tree && near)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::berlin::normalise_building;
    use crate::sources::berlin::test_support::*;

    fn features(collection: &str) -> Vec<Value> {
        serde_json::from_str::<Value>(collection).unwrap()["features"]
            .as_array()
            .unwrap()
            .clone()
    }

    fn tag<'a>(feature: &'a OsmFeature, key: &str) -> Option<&'a str> {
        feature
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    fn code(tags: &[(&str, &str)], geometry: &OsmGeometry, kind: MapKind) -> Option<String> {
        let mut tags: Vec<(String, String)> = tags
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        tags.sort();
        Rules::berlin()
            .symbol_for(&tags, geometry, kind)
            .map(|m| m.code)
    }

    fn some_line() -> OsmGeometry {
        OsmGeometry::Line(vec![
            LatLon {
                lat: 52.5,
                lon: 13.4,
            },
            LatLon {
                lat: 52.5,
                lon: 13.401,
            },
        ])
    }

    fn some_ring() -> OsmGeometry {
        let p = |lat, lon| LatLon { lat, lon };
        OsmGeometry::Line(vec![
            p(52.5, 13.4),
            p(52.5, 13.401),
            p(52.501, 13.401),
            p(52.5, 13.4),
        ])
    }

    /// A 600 m box around the fixture area (Alexanderplatz) at 1:1000.
    fn setup(kind: MapKind) -> (Georeferencing, geo_types::Polygon<f64>, SymbolSet) {
        let centre = LAYER_CRS
            .inverse(ProjectedPoint {
                easting: 392_400.0,
                northing: 5_820_400.0,
            })
            .unwrap();
        let georef = Georeferencing::new(centre, 1000, "2026-11-08".parse().unwrap()).unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let ring = vec![
            (e - 600.0, n - 600.0),
            (e + 600.0, n - 600.0),
            (e + 600.0, n + 600.0),
            (e - 600.0, n + 600.0),
            (e - 600.0, n - 600.0),
        ];
        (
            georef,
            geo_types::Polygon::new(ring.into(), vec![]),
            kind.symbol_set(),
        )
    }

    fn objects(layer: &str, collection: &str, kind: MapKind) -> Vec<MapObject> {
        let (georef, area, set) = setup(kind);
        let features = features(collection);
        let input = LayerInput {
            file: layer,
            features: &features,
            currency: "2026-11-09".parse().unwrap(),
        };
        berlin_objects(&[input], kind, &area, 20.0, &georef, &set).unwrap()
    }

    /// A stored Straßenbefahrung feature: `freemapper:layer` and the survey date stamped as
    /// the download does.
    fn survey_feature(layer: &str, mut feature: Value) -> Value {
        feature["properties"][crate::sources::berlin::TYPE_KEY] = layer.into();
        crate::sources::berlin::stamp_survey_date(&mut feature);
        feature
    }

    fn stored(layer: &str, collection: &str) -> Vec<Value> {
        features(collection)
            .into_iter()
            .map(|f| survey_feature(layer, f))
            .collect()
    }

    fn survey_objects(file: &str, features: &[Value], kind: MapKind) -> Vec<MapObject> {
        let (georef, area, set) = setup(kind);
        let input = LayerInput {
            file,
            features,
            currency: "2026-11-09".parse().unwrap(),
        };
        berlin_objects(&[input], kind, &area, 20.0, &georef, &set).unwrap()
    }

    fn object_tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    fn survey_point(layer: &str, id: &str, (e, n): (f64, f64)) -> Value {
        survey_feature(
            layer,
            serde_json::json!({
                "type": "Feature", "id": format!("{layer}.{id}"),
                "geometry": {"type": "Point", "coordinates": [e, n]},
                "properties": {"gis_id": format!("Pk_2015-05-02_{id}"), "pot": 2}
            }),
        )
    }

    #[test]
    fn curbs_are_edge_candidates_on_sprint_only() {
        let curbs = stored("bd_bordstein", SB_BORDSTEIN);
        let found = survey_objects("sb-bordstein", &curbs, MapKind::Sprint);
        let set = MapKind::Sprint.symbol_set();
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found
                .iter()
                .all(|o| Some(o.symbol_id) == set.id_for_code("501.1")
                    && matches!(o.kind, MapObjectKind::Path))
        );
        assert_eq!(
            object_tag(&found[0], "freemapper:source"),
            Some("geoportal-berlin/sb-bordstein")
        );
        // The survey date is the currency; the survey codes stay for the mapper, `NULL` does not.
        assert_eq!(
            object_tag(&found[0], "freemapper:source_id"),
            Some("Li_16.04.2015_18991")
        );
        assert_eq!(
            object_tag(&found[0], "freemapper:currency"),
            Some("2015-04-16")
        );
        assert_eq!(
            object_tag(&found[1], "freemapper:currency"),
            Some("2014-07-16")
        );
        assert_eq!(object_tag(&found[0], "berlin:sre"), Some("0"));
        assert_eq!(object_tag(&found[1], "berlin:sre"), None);
        // A schoolyard map has no curb candidates.
        assert!(survey_objects("sb-bordstein", &curbs, MapKind::Schoolyard).is_empty());
        // A curb shorter than 1 mm on paper (1 m at 1:1000) is dropped.
        let mut short = curbs[0].clone();
        short["geometry"]["coordinates"] =
            serde_json::json!([[[392_500.0, 5_820_500.0], [392_500.4, 5_820_500.0]]]);
        assert!(survey_objects("sb-bordstein", &[short], MapKind::Sprint).is_empty());
    }

    #[test]
    fn railings_are_impassable_fence_candidates() {
        let mut railings = stored("bf_gelaender", SB_GELAENDER);
        railings.extend(stored("bb_verkehrsschutzgitter", SB_VERKEHRSSCHUTZGITTER));
        let mut barrier = railings[0].clone();
        barrier["properties"][crate::sources::berlin::TYPE_KEY] = "bg_leitplanke".into();
        railings.push(barrier);
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let found = survey_objects("sb-gelaender", &railings, kind);
            let set = kind.symbol_set();
            assert_eq!(found.len(), 5, "{found:?}");
            assert!(
                found
                    .iter()
                    .all(|o| Some(o.symbol_id) == set.id_for_code("518"))
            );
            assert_eq!(
                object_tag(&found[0], "freemapper:currency"),
                Some("2015-05-02")
            );
        }
    }

    #[test]
    fn walls_of_the_survey_are_515() {
        let wall = survey_feature(
            "bh_mauer",
            serde_json::json!({
                "type": "Feature", "id": "bh_mauer.1",
                "geometry": {"type": "MultiLineString", "coordinates": [[[392_500.0, 5_820_500.0], [392_520.0, 5_820_500.0]]]},
                "properties": {"gis_id": "Li_25.09.2014_230", "hms": "2"}
            }),
        );
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let found = survey_objects("sb-mauer", std::slice::from_ref(&wall), kind);
            assert_eq!(found.len(), 1);
            assert_eq!(
                Some(found[0].symbol_id),
                kind.symbol_set().id_for_code("515")
            );
            assert_eq!(object_tag(&found[0], "berlin:hms"), Some("2"));
            assert_eq!(
                object_tag(&found[0], "freemapper:currency"),
                Some("2014-09-25")
            );
        }
    }

    #[test]
    fn bollards_only_on_schoolyard() {
        let furniture: Vec<Value> = ["av_poller", "as_mast", "am_fahnenmast"]
            .iter()
            .enumerate()
            .map(|(i, layer)| {
                survey_point(layer, &i.to_string(), (392_500.0 + i as f64, 5_820_500.0))
            })
            .collect();
        let found = survey_objects("sb-moebel", &furniture, MapKind::Schoolyard);
        let set = MapKind::Schoolyard.symbol_set();
        assert_eq!(found.len(), 3, "{found:?}");
        assert!(
            found
                .iter()
                .all(|o| Some(o.symbol_id) == set.id_for_code("544"))
        );
        assert_eq!(object_tag(&found[0], "berlin:pot"), Some("2"));
        // On a sprint map they would flood the candidates.
        assert!(survey_objects("sb-moebel", &furniture, MapKind::Sprint).is_empty());
        // The other schoolyard furniture: hydrant 321, play equipment 541.
        let others = [
            survey_point("aq_hydrant", "7", (392_510.0, 5_820_500.0)),
            survey_point("aw_spielgeraet", "8", (392_520.0, 5_820_500.0)),
        ];
        let found = survey_objects("sb-moebel", &others, MapKind::Schoolyard);
        let codes: Vec<_> = ["321", "541"].iter().map(|c| set.id_for_code(c)).collect();
        assert_eq!(
            found.iter().map(|o| Some(o.symbol_id)).collect::<Vec<_>>(),
            codes
        );
        assert!(survey_objects("sb-moebel", &others, MapKind::Sprint).is_empty());
    }

    #[test]
    fn advertising_column_is_531() {
        let column = survey_point("ag_werbesaeule", "1", (392_500.0, 5_820_500.0));
        let found = survey_objects("sb-objekte", &[column], MapKind::Sprint);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            Some(found[0].symbol_id),
            MapKind::Sprint.symbol_set().id_for_code("531")
        );
        assert!(matches!(found[0].kind, MapObjectKind::Point { .. }));
        // The schoolyard set has no use for it and no layer fetches it there.
        let column = survey_point("ag_werbesaeule", "1", (392_500.0, 5_820_500.0));
        assert!(survey_objects("sb-objekte", &[column], MapKind::Schoolyard).is_empty());
    }

    #[test]
    fn shelters_pumps_and_fountains_are_street_objects() {
        let shelter = survey_feature(
            "br_fahrgastunterstand",
            serde_json::json!({
                "type": "Feature", "id": "br_fahrgastunterstand.1",
                "geometry": {"type": "MultiPolygon", "coordinates": [[[
                    [392_500.0, 5_820_500.0], [392_503.0, 5_820_500.0], [392_503.0, 5_820_502.0],
                    [392_500.0, 5_820_502.0], [392_500.0, 5_820_500.0]]]]},
                "properties": {"gis_id": "Fl_20150413_37"}
            }),
        );
        let objects = [
            shelter,
            survey_point("af_wasserpumpen_brunnen", "2", (392_510.0, 5_820_500.0)),
            survey_point(
                "bv_springbrunnen_zierbrunnen",
                "3",
                (392_520.0, 5_820_500.0),
            ),
            survey_point(
                "ac_trinkwasserbrunnen_wasserspender",
                "4",
                (392_530.0, 5_820_500.0),
            ),
        ];
        let codes = |kind: MapKind| -> Vec<Option<u32>> {
            survey_objects("sb-objekte", &objects, kind)
                .iter()
                .map(|o| Some(o.symbol_id))
                .collect()
        };
        let set = MapKind::Sprint.symbol_set();
        assert_eq!(
            codes(MapKind::Sprint),
            ["522", "311", "311"].map(|c| set.id_for_code(c)).to_vec()
        );
        let set = MapKind::Schoolyard.symbol_set();
        assert_eq!(
            codes(MapKind::Schoolyard),
            ["522", "311", "311", "313"]
                .map(|c| set.id_for_code(c))
                .to_vec()
        );
    }

    /// A survey or ALKIS line of `symbol` at `n`, `e0`..`e1` (metres from Alexanderplatz).
    fn line_object(
        kind: MapKind,
        code: &str,
        source: &str,
        id: &str,
        (e0, e1, n): (f64, f64, f64),
    ) -> MapObject {
        let (georef, _, set) = setup(kind);
        let p = |e: f64| {
            georef
                .projected_to_map(ProjectedPoint {
                    easting: 392_400.0 + e,
                    northing: 5_820_400.0 + n,
                })
                .unwrap()
        };
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Path,
            coords: vec![p(e0), p(e1)],
            tags: vec![
                ("freemapper:source".into(), source.into()),
                ("freemapper:source_id".into(), id.into()),
            ],
        }
    }

    #[test]
    fn survey_lines_near_alkis_are_dropped_and_named() {
        let kind = MapKind::Sprint;
        let set = kind.symbol_set();
        let alkis = "geoportal-berlin/bauwerke-linien";
        let survey = "geoportal-berlin/sb-gelaender";
        let mut berlin = vec![
            line_object(kind, "518", alkis, "A1", (0.0, 40.0, 0.0)),
            // A wall: another symbol, stays.
            line_object(kind, "515", alkis, "A2", (0.0, 40.0, 30.0)),
        ];
        berlin.extend([
            // Along the ALKIS fence within 1 m: the same fence.
            line_object(kind, "518", survey, "S1", (1.0, 39.0, 0.8)),
            // 5 m away: another fence.
            line_object(kind, "518", survey, "S2", (1.0, 39.0, 5.0)),
            // Along the ALKIS wall but a railing: stays (different symbol).
            line_object(kind, "518", survey, "S3", (1.0, 39.0, 30.5)),
        ]);
        let kept = drop_survey_lines_near_alkis(berlin, &set, 1000);
        // The ALKIS objects are kept, the near survey fence is gone.
        let ids: Vec<_> = kept
            .iter()
            .map(|o| object_tag(o, "freemapper:source_id").unwrap())
            .collect();
        assert_eq!(ids, ["A1", "A2", "S2", "S3"]);
        assert_eq!(
            object_tag(&kept[0], "freemapper:also"),
            Some("geoportal-berlin/sb-gelaender/S1")
        );
        assert_eq!(object_tag(&kept[1], "freemapper:also"), None);
    }

    #[test]
    fn berlin_feature_becomes_tagged_feature() {
        let point = serde_json::json!({
            "type": "Feature", "id": "strassenbaeume.00008100_0014ef85",
            "geometry": {"type": "Point", "coordinates": [392419.46, 5820118.05]},
            "properties": {"gisid": "00008100_0014ef85", "kronedurch": 4.5, "nam": "", "art": null}
        });
        let tagged = tagged_features("strassenbaeume", &[point]);
        assert_eq!(tagged.len(), 1);
        assert!(matches!(tagged[0].geometry, OsmGeometry::Point(_)));
        assert_eq!(
            tagged[0].id,
            "berlin/strassenbaeume/strassenbaeume.00008100_0014ef85"
        );
        assert_eq!(tag(&tagged[0], "berlin:layer"), Some("strassenbaeume"));
        assert_eq!(tag(&tagged[0], "berlin:gisid"), Some("00008100_0014ef85"));
        assert_eq!(tag(&tagged[0], "berlin:kronedurch"), Some("4.5"));
        // Empty strings and nulls are no tags.
        assert_eq!(tag(&tagged[0], "berlin:nam"), None);
        assert_eq!(tag(&tagged[0], "berlin:art"), None);

        // A stored `layer` (the WFS type of a file with several types) is the `berlin:layer`.
        let rail = serde_json::json!({
            "type": "Feature", "id": "bf_gelaender.1",
            "geometry": {"type": "Point", "coordinates": [392419.46, 5820118.05]},
            "properties": {"freemapper:layer": "bf_gelaender", "layer": "native", "survey_date": "2015-05-02"}
        });
        let tagged = tagged_features("sb-gelaender", &[rail]);
        let layers: Vec<&str> = tagged[0]
            .tags
            .iter()
            .filter(|(k, _)| k == "berlin:layer")
            .map(|(_, v)| v.as_str())
            .collect();
        assert_eq!(layers, ["bf_gelaender"]);
        assert_eq!(tag(&tagged[0], "berlin:survey_date"), Some("2015-05-02"));

        // Each part of a MultiLineString is a line.
        let lines = serde_json::json!({
            "type": "Feature", "id": "bauwerkelinien.1",
            "geometry": {"type": "MultiLineString", "coordinates": [
                [[392400.0, 5820400.0], [392410.0, 5820400.0]],
                [[392420.0, 5820400.0], [392430.0, 5820400.0]]]},
            "properties": {"bezbwf": "Zaun"}
        });
        let tagged = tagged_features("bauwerke-linien", &[lines]);
        assert_eq!(tagged.len(), 2);
        assert!(
            tagged
                .iter()
                .all(|f| matches!(&f.geometry, OsmGeometry::Line(l) if l.len() == 2))
        );

        // A polygon is its exterior as a closed line (the hole is dropped) and a centroid point.
        let polygon = serde_json::json!({
            "type": "Feature", "id": "bauwerkeflaechen.X",
            "geometry": {"type": "MultiPolygon", "coordinates": [[
                [[392400.0, 5820400.0], [392410.0, 5820400.0], [392410.0, 5820410.0],
                 [392400.0, 5820410.0], [392400.0, 5820400.0]],
                [[392403.0, 5820403.0], [392405.0, 5820403.0], [392405.0, 5820405.0],
                 [392403.0, 5820403.0]]]]},
            "properties": {"bezbwf": "Mauer"}
        });
        let tagged = tagged_features("bauwerke-flaechen", &[polygon]);
        assert_eq!(tagged.len(), 2);
        let OsmGeometry::Line(ring) = &tagged[0].geometry else {
            panic!("ring first")
        };
        assert_eq!(ring.len(), 5);
        assert_eq!(ring.first(), ring.last());
        let OsmGeometry::Point(centre) = &tagged[1].geometry else {
            panic!("centroid second")
        };
        let centre = LAYER_CRS.forward(*centre);
        assert!((centre.easting - 392_405.0).abs() < 0.01);
        assert!((centre.northing - 5_820_405.0).abs() < 0.01);
        // 10 m across: large.
        assert_eq!(tag(&tagged[1], "freemapper:size"), Some("large"));
        assert_eq!(tag(&tagged[0], "freemapper:size"), None);
    }

    #[test]
    fn walls_fences_hedges_map_to_issprom() {
        let line = some_line();
        let ring = some_ring();
        for (layer, key, name, expected, geometry) in [
            ("bauwerke-linien", "bezbwf", "Zaun", "518", &line),
            ("bauwerke-linien", "bezbwf", "Mauer", "515", &line),
            (
                "bauwerke-linien",
                "bezbwf",
                "Mauerkante, rechts",
                "515",
                &line,
            ),
            (
                "bauwerke-linien",
                "bezbwf",
                "Mauerkante, links",
                "515",
                &line,
            ),
            ("bauwerke-linien", "bezbwf", "Mauermitte", "515", &line),
            ("bauwerke-linien", "bezbwf", "Stützmauer", "513.2", &line),
            (
                "bauwerke-linien",
                "bezbwf",
                "Stützmauer, rechts",
                "513.2",
                &line,
            ),
            (
                "bauwerke-linien",
                "bezbwf",
                "Stützmauermitte",
                "513.2",
                &line,
            ),
            ("bauwerke-flaechen", "bezbwf", "Mauer", "515", &ring),
            ("bauwerke-flaechen", "bezbwf", "Stützmauer", "513.2", &ring),
            (
                "vegetation-linien",
                "bezbws",
                "Heckenkante, rechts",
                "410.1",
                &line,
            ),
        ] {
            let tag = format!("berlin:{key}");
            for kind in [MapKind::Sprint, MapKind::Schoolyard] {
                assert_eq!(
                    code(&[("berlin:layer", layer), (&tag, name)], geometry, kind),
                    Some(expected.to_string()),
                    "{name} on {kind:?}"
                );
            }
        }
        // Height decides nothing; other names are no walls.
        assert_eq!(
            code(
                &[
                    ("berlin:layer", "bauwerke-linien"),
                    ("berlin:bezbwf", "Brückenpfeiler")
                ],
                &line,
                MapKind::Sprint
            ),
            None
        );
    }

    #[test]
    fn freitreppe_line_is_no_rule_candidate() {
        let line = some_line();
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            // `Freitreppe` lines are step edges: `build/stairs.rs` joins a flight of them into
            // one stair; no rule draws a single edge.
            let tags = [
                ("berlin:layer", "bauwerke-linien"),
                ("berlin:bezbwf", "Freitreppe"),
            ];
            assert_eq!(code(&tags, &line, kind), None);
            // Stair areas are built in `build/stairs.rs`, not by the rules.
            let area = [
                ("berlin:layer", "bauwerke-flaechen"),
                ("berlin:bezbwf", "Freitreppe"),
            ];
            assert_eq!(code(&area, &some_ring(), kind), None);
            assert_eq!(code(&area, &line, kind), None);
        }
    }

    #[test]
    fn passage_bauteile_are_canopy_candidates() {
        let stored: Vec<Value> = features(BUILDING_PARTS)
            .iter()
            .map(normalise_building)
            .collect();
        let parts = building_part_features(&stored);
        assert_eq!(parts.len(), 5);
        let mut codes = Vec::new();
        for feature in tagged_features(BUILDING_PARTS_LAYER, &parts) {
            let kind = tag(&feature, "berlin:bauart").unwrap().to_string();
            let found =
                Rules::berlin().symbol_for(&feature.tags, &feature.geometry, MapKind::Sprint);
            codes.push((kind, found.map(|m| m.code)));
        }
        let codes: Vec<(&str, Option<&str>)> = codes
            .iter()
            .map(|(k, c)| (k.as_str(), c.as_deref()))
            .collect();
        assert!(codes.contains(&("Durchfahrt im Gebäude", Some("522"))));
        assert!(codes.contains(&("Arkade", Some("522"))));
        assert!(codes.contains(&("Durchfahrt an\u{a0}überbautem Verkehrsweg", Some("522"))));
        assert!(codes.contains(&("Geringergeschossiger Gebäudeteil", None)));
        assert!(codes.contains(&(
            "Auskragender Geschossteil / zurückspringendes Geschoss",
            None
        )));
        // Whole buildings and parts without a kind are no passages.
        let buildings: Vec<Value> = features(BUILDINGS).iter().map(normalise_building).collect();
        assert!(
            building_part_features(&buildings)
                .iter()
                .all(|f| f["properties"]["gebnutzbez"] == "Bauteil")
        );
        let whole =
            serde_json::json!({"properties": {"gebnutzbez": "Gebäude", "bauart": "Arkade"}});
        let bare = serde_json::json!({"properties": {"gebnutzbez": "Bauteil"}});
        assert!(building_part_features(&[whole, bare]).is_empty());
    }

    #[test]
    fn embankment_top_only_on_sprint() {
        let sprint = objects("relief", RELIEF, MapKind::Sprint);
        // Two Oberkante lines; Unterkante and Sonstige Begrenzungskante are skipped.
        assert_eq!(sprint.len(), 2);
        let set = MapKind::Sprint.symbol_set();
        assert!(
            sprint
                .iter()
                .all(|o| Some(o.symbol_id) == set.id_for_code("104"))
        );
        assert!(objects("relief", RELIEF, MapKind::Schoolyard).is_empty());
    }

    #[test]
    fn berlin_provenance_is_geoportal() {
        let found = objects("bauwerke-linien", BAUWERKE_LINIEN, MapKind::Sprint);
        let zaun = found
            .iter()
            .find(|o| o.tags.contains(&("freemapper:alkis".into(), "Zaun".into())))
            .expect("a fence");
        let tags: Vec<(&str, &str)> = zaun
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert!(tags.contains(&("freemapper:source", "geoportal-berlin/bauwerke-linien")));
        assert!(tags.contains(&("freemapper:currency", "2026-11-09")));
        assert!(
            tags.iter()
                .any(|(k, v)| *k == "freemapper:source_id" && v.starts_with("DEBE"))
        );
        assert!(tags.iter().all(|(k, _)| *k != "freemapper:osm"));
        assert!(tags.iter().all(|(_, v)| *v != "osm"));
    }

    #[test]
    fn osm_rules_ignore_berlin_tags() {
        let berlin = [
            ("berlin:layer".to_string(), "bauwerke-linien".to_string()),
            ("berlin:bezbwf".to_string(), "Zaun".to_string()),
            ("berlin:art".to_string(), "Mauer".to_string()),
        ];
        let line = some_line();
        let osm = Rules::embedded();
        assert!(
            osm.symbol_for(&berlin.to_vec(), &line, MapKind::Sprint)
                .is_none()
        );
        assert!(!osm.wants(&berlin, MapKind::Sprint));
        // And the other way round.
        assert_eq!(code(&[("barrier", "fence")], &line, MapKind::Sprint), None);
        assert!(!Rules::berlin().wants(&[("barrier".into(), "fence".into())], MapKind::Sprint));
    }

    #[test]
    fn crown_decides_417_or_418() {
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let set = kind.symbol_set();
            let (big, small) = (set.id_for_code("417"), set.id_for_code("418"));
            // Street trees: crowns of 4.5 and 5 m.
            let street = objects("strassenbaeume", STRASSENBAEUME, kind);
            assert_eq!(street.len(), 4);
            assert!(
                street.iter().all(|o| Some(o.symbol_id) == small),
                "{kind:?}"
            );
            // Park trees: crowns of 10, 7, 10 and 10 m; from 10 m it is a large tree.
            let park = objects("anlagenbaeume", ANLAGENBAEUME, kind);
            let ids: Vec<Option<u32>> = park.iter().map(|o| Some(o.symbol_id)).collect();
            assert_eq!(ids.iter().filter(|id| **id == big).count(), 3, "{kind:?}");
            assert_eq!(ids.iter().filter(|id| **id == small).count(), 1, "{kind:?}");
        }
        // 9.9 m is small, 10 m large; an unknown crown is the small tree.
        let point = some_point();
        let code_of = |tags: &[(&str, &str)]| code(tags, &point, MapKind::Sprint);
        let layer = ("berlin:layer", "strassenbaeume");
        assert_eq!(
            code_of(&[layer, ("berlin:kronedurch", "9.9")]).as_deref(),
            Some("418")
        );
        assert_eq!(
            code_of(&[layer, ("berlin:kronedurch", "10")]).as_deref(),
            Some("417")
        );
        assert_eq!(
            code_of(&[layer, ("berlin:kronedurch", "14.5")]).as_deref(),
            Some("417")
        );
        assert_eq!(code_of(&[layer]).as_deref(), Some("418"));
        assert_eq!(
            code_of(&[("berlin:layer", "relief"), ("berlin:kronedurch", "12")]),
            None
        );
        // The candidate says which tree it is.
        let park = objects("anlagenbaeume", ANLAGENBAEUME, MapKind::Sprint);
        let first = park
            .iter()
            .find(|o| {
                o.tags.contains(&(
                    "freemapper:baum".into(),
                    "Tilia tomentosa, Krone 10 m".into(),
                ))
            })
            .expect("species and crown as the explanation");
        assert!(first.tags.contains(&(
            "freemapper:source".into(),
            "geoportal-berlin/anlagenbaeume".into()
        )));
    }

    #[test]
    fn osm_tree_near_cadastre_tree_is_dropped() {
        let (georef, _, set) = setup(MapKind::Sprint);
        let cadastre = objects("strassenbaeume", STRASSENBAEUME, MapKind::Sprint);
        let (e, n) = (392_419.46, 5_820_118.05);
        let osm = |code: &str, osm: &str, de: f64, dn: f64| MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![
                georef
                    .projected_to_map(ProjectedPoint {
                        easting: e + de,
                        northing: n + dn,
                    })
                    .unwrap(),
            ],
            tags: vec![
                ("freemapper:source".into(), "osm".into()),
                ("freemapper:osm".into(), osm.into()),
            ],
        };
        let tree = |de, dn| osm("418", "natural=tree", de, dn);
        let kept = drop_osm_trees_near_cadastre(
            vec![
                tree(1.0, 1.0),                          // 1.4 m away: the same tree
                tree(0.0, 1.99),                         // just inside 2 m
                tree(0.0, 2.5),                          // 2.5 m: another tree
                osm("526", "tourism=artwork", 0.0, 0.5), // not a tree
            ],
            &cadastre,
            georef.scale_denominator,
        );
        let osm_tags: Vec<&str> = kept.iter().map(|o| o.tags[1].1.as_str()).collect();
        assert_eq!(osm_tags, ["natural=tree", "tourism=artwork"]);
        // Without cadastre trees every OSM tree stays.
        assert_eq!(
            drop_osm_trees_near_cadastre(vec![tree(0.0, 0.0)], &[], georef.scale_denominator).len(),
            1
        );
    }

    #[test]
    fn osm_line_along_an_alkis_line_is_dropped() {
        let kind = MapKind::Sprint;
        let (georef, _, set) = setup(kind);
        let (e, n) = (392_400.0, 5_820_400.0);
        // An ALKIS fence 50 m east from (e, n) and an ALKIS wall 20 m north of it.
        let collection = serde_json::json!({"type": "FeatureCollection", "features": [
            {"type": "Feature", "id": "bauwerkelinien.Z",
             "geometry": {"type": "LineString", "coordinates": [[e, n], [e + 50.0, n]]},
             "properties": {"uuid": "DEBE00ZAUN", "bezbwf": "Zaun"}},
            {"type": "Feature", "id": "bauwerkelinien.M",
             "geometry": {"type": "LineString", "coordinates": [[e, n + 20.0], [e + 50.0, n + 20.0]]},
             "properties": {"uuid": "DEBE00MAUER", "bezbwf": "Mauer"}}
        ]})
        .to_string();
        let mut alkis = objects("bauwerke-linien", &collection, kind);
        assert_eq!(alkis.len(), 2);
        let at = |de: f64, dn: f64| {
            georef
                .projected_to_map(ProjectedPoint {
                    easting: e + de,
                    northing: n + dn,
                })
                .unwrap()
        };
        let osm = |id: &str, code: &str, from: (f64, f64), to: (f64, f64)| MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Path,
            coords: vec![at(from.0, from.1), at(to.0, to.1)],
            tags: vec![
                ("freemapper:source".into(), "osm".into()),
                ("freemapper:source_id".into(), id.into()),
            ],
        };
        let candidates = vec![
            // 1 m beside the ALKIS fence along all of it: the same fence.
            osm("way/1", "518", (0.0, 1.0), (50.0, 1.0)),
            // Beside it for 40 of 60 m (67 %): kept.
            osm("way/2", "518", (10.0, -1.0), (70.0, -1.0)),
            // 2 m away: another fence.
            osm("way/3", "518", (0.0, 2.0), (50.0, 2.0)),
            // A wall along the ALKIS fence: another symbol, kept.
            osm("way/4", "515", (0.0, 0.5), (50.0, 0.5)),
            // A wall 0.5 m beside the ALKIS wall, 40 m of its 50 m: the same wall.
            osm("way/5", "515", (5.0, 20.5), (45.0, 20.5)),
            // 85 % beside the ALKIS fence (the last 6 m of 40 m turn away): the same fence.
            MapObject {
                coords: vec![at(10.0, 1.0), at(44.0, 1.0), at(44.0, 7.0)],
                ..osm("way/6", "518", (0.0, 0.0), (0.0, 0.0))
            },
        ];
        let tree = MapObject {
            symbol_id: set.id_for_code("418").unwrap(),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![at(1.0, 0.5)],
            tags: vec![("freemapper:source_id".into(), "node/7".into())],
        };
        let mut all = candidates;
        all.push(tree);
        let kept = drop_osm_lines_near_alkis(all, &mut alkis, &set, georef.scale_denominator);
        let id = |o: &MapObject| {
            o.tags
                .iter()
                .find(|(k, _)| k == "freemapper:source_id")
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        let kept: Vec<String> = kept.iter().map(id).collect();
        assert_eq!(kept, ["way/2", "way/3", "way/4", "node/7"]);
        // The ALKIS object that wins names the OSM ways it replaces.
        let also = |o: &MapObject| {
            o.tags
                .iter()
                .find(|(k, _)| k == "freemapper:also")
                .map(|(_, v)| v.clone())
        };
        assert_eq!(also(&alkis[0]).as_deref(), Some("osm/way/1; osm/way/6"));
        assert_eq!(also(&alkis[1]).as_deref(), Some("osm/way/5"));
        // Without ALKIS lines every OSM line stays.
        let lone = vec![osm("way/1", "518", (0.0, 1.0), (50.0, 1.0))];
        assert_eq!(
            drop_osm_lines_near_alkis(lone, &mut [], &set, georef.scale_denominator).len(),
            1
        );
    }

    fn some_point() -> OsmGeometry {
        OsmGeometry::Point(LatLon {
            lat: 52.5,
            lon: 13.4,
        })
    }
}
