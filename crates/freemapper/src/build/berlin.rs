//! Berlin ALKIS structures and building parts → candidate objects (Berlin plus 1).
//!
//! The Geoportal features run through the same [`Rules`] engine as the OSM features
//! (`assets/rules/berlin-candidates.toml`). A feature becomes a tagged feature first: its string
//! and number properties become `berlin:<name>` tags, so no OSM rule can match one and no Berlin
//! rule an OSM tag. The building parts are parsed from the stored buildings in one place
//! ([`building_part_features`]).

use chrono::NaiveDate;
use geo::BoundingRect as _;
use serde_json::Value;

use crate::build::candidates::{Rules, Target, osm_objects};
use crate::build::dedup::add_also;
use crate::build::geometry::object_polygon;
use crate::build::tagged::{self, position};
use crate::crs::{Crs, LatLon};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::coord::FLAG_CURVE_START;
use crate::omap::{MapObject, SymbolSet};
use crate::project::MapKind;
use crate::sources::osm::{OsmFeature, OsmGeometry};

/// Layer name of the building parts (`Durchfahrt im Gebäude` …) in `berlin:layer`.
pub const BUILDING_PARTS_LAYER: &str = "building-parts";
/// The tree cadastre layers.
const TREE_LAYERS: [&str; 2] = freemapper_adapters_de::berlin::TREE_REGISTERS;
/// The stored kerb layer of the street survey.
const KERB_LAYER: &str = "sb-bordstein";
/// The CRS of all Geoportal Berlin layers.
pub(crate) const LAYER_CRS: Crs = Crs::Utm33;

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

fn latlon(p: (f64, f64)) -> Option<LatLon> {
    tagged::latlon(LAYER_CRS, p)
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
        None => (freemapper_adapters_de::berlin::SURVEY_DATASET_DATE, true),
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
        .and_then(|p| p.get(freemapper_adapters_de::berlin::TYPE_KEY))
        .and_then(Value::as_str)
        .unwrap_or(layer);
    let mut tags = vec![("berlin:layer".to_string(), layer.to_string())];
    for (key, value) in properties.into_iter().flatten() {
        if key == freemapper_adapters_de::berlin::TYPE_KEY || key == "layer" {
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
    // The crown diameter for the point generalization (`build/points.rs`).
    if TREE_LAYERS.contains(&layer)
        && let Some(crown) = properties
            .and_then(|p| p.get("kronedurch"))
            .and_then(Value::as_f64)
            .filter(|c| *c > 0.0)
    {
        tags.push(("freemapper:crown_m".to_string(), crown.to_string()));
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
    let mut tagged = tagged::tagged_features(
        &format!("berlin/{layer}"),
        features,
        LAYER_CRS,
        tagged::PointOn::Centroid,
        |feature| feature_tags(layer, feature),
    );
    // A surveyed bench is a line; a sprint map takes it as a control site, a point at its middle.
    let benches: Vec<OsmFeature> = tagged
        .iter()
        .filter_map(|f| {
            let OsmGeometry::Line(points) = &f.geometry else {
                return None;
            };
            if !f
                .tags
                .iter()
                .any(|(k, v)| k == "berlin:layer" && v == BENCH_LAYER)
            {
                return None;
            }
            let (min_lat, max_lat) = points.iter().fold((f64::MAX, f64::MIN), |(a, b), p| {
                (a.min(p.lat), b.max(p.lat))
            });
            let (min_lon, max_lon) = points.iter().fold((f64::MAX, f64::MIN), |(a, b), p| {
                (a.min(p.lon), b.max(p.lon))
            });
            // The schoolyard rule draws the line itself and skips this point.
            let mut tags = f.tags.clone();
            tags.push((MIDDLE_KEY.to_string(), "middle".to_string()));
            tags.sort();
            Some(OsmFeature {
                id: f.id.clone(),
                tags,
                geometry: OsmGeometry::Point(LatLon {
                    lat: (min_lat + max_lat) / 2.0,
                    lon: (min_lon + max_lon) / 2.0,
                }),
            })
        })
        .collect();
    tagged.extend(benches);
    tagged
}

/// The tag of the point at the middle of a bench line (`berlin:point=middle`).
const MIDDLE_KEY: &str = "berlin:point";
/// The survey type of the benches, which are lines.
const BENCH_LAYER: &str = "bj_sitzbank";

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
        let found = osm_objects(
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
        )?;
        if layer.file == KERB_LAYER {
            objects.extend(found.into_iter().filter_map(drop_kerb_notches));
        } else {
            objects.extend(found);
        }
    }
    Ok(objects)
}

/// A kerb detour that encloses less than this (1.5 mm² on paper, in (1/1000 mm)²) is no feature
/// of the street. ISSprOM 2019-2 §3.3 asks 1.0 mm² for the smallest area of a brown dot screen,
/// which paving is; the surveyed tree pits with their lead-in measure up to 1.1 mm² at 1:4000, so
/// the limit takes half a square millimetre on top.
const KERB_NOTCH_MAX_AREA: f64 = 1.5e6;
/// A detour longer than this on paper (10 mm, 40 m at 1:4000) is never a pit.
const KERB_NOTCH_MAX_PATH: f64 = 10_000.0;
/// A detour wider than this on paper (2.5 mm, 10 m at 1:4000) is no single pit.
const KERB_NOTCH_MAX_CHORD: f64 = 2_500.0;
/// A detour is a pit when it reaches out at least this share of its chord: a quarter-circle
/// corner reaches 0.21, a pit with its lead-in 0.4 and more.
const KERB_NOTCH_MIN_DEPTH: f64 = 0.3;
/// The kerb before and after a detour has to run along its chord: the cosine of the angle
/// between them is at least this (20 degrees). Without it a chord to the apex of a slot, or across
/// a step of the kerb, would count as a notch and draw a spike or a diagonal.
const KERB_NOTCH_MIN_COS: f64 = 0.94;
/// How far from the detour's ends the kerb direction is taken (0.5 mm), to skip survey jitter.
const KERB_NOTCH_REFERENCE: f64 = 500.0;
/// The kerb line of the street survey (501.1) without the tree pits cut into it: the survey
/// runs the kerb around each pit in a parking strip and the pit then shows as a hook along the
/// street edge. Each detour that returns to the line within [`KERB_NOTCH_MAX_PATH`] of way and
/// [`KERB_NOTCH_MAX_CHORD`] of chord, reaches out [`KERB_NOTCH_MIN_DEPTH`] of its chord and
/// encloses less than [`KERB_NOTCH_MAX_AREA`] is replaced by its chord. A closed kerb ring below that area, the
/// kerb around a lone pit, goes. `None` when the object goes; a line with curve points stays as it is. The close flag of a ring does not count as one.
pub(crate) fn drop_kerb_notches(mut object: MapObject) -> Option<MapObject> {
    if object
        .coords
        .iter()
        .any(|c| c.flags & FLAG_CURVE_START != 0)
    {
        return Some(object);
    }
    let points: Vec<(f64, f64)> = object
        .coords
        .iter()
        .map(|c| (f64::from(c.x), f64::from(c.y)))
        .collect();
    let n = points.len();
    if n < 3 {
        return Some(object);
    }
    // Twice the signed area of the loop `points[i..=j]` closed by its chord.
    let cross = |a: (f64, f64), b: (f64, f64)| a.0 * b.1 - b.0 * a.1;
    if points[0] == points[n - 1] {
        let area2: f64 = points.windows(2).map(|w| cross(w[0], w[1])).sum();
        if area2.abs() / 2.0 < KERB_NOTCH_MAX_AREA {
            return None;
        }
    }
    let mut kept = vec![object.coords[0]];
    let mut i = 0;
    while i < n - 1 {
        let mut best = None;
        let (mut path, mut area2) = (0.0, 0.0);
        for j in i + 1..n {
            path += (points[j].0 - points[j - 1].0).hypot(points[j].1 - points[j - 1].1);
            if path > KERB_NOTCH_MAX_PATH {
                break;
            }
            area2 += cross(points[j - 1], points[j]);
            let chord = (points[j].0 - points[i].0).hypot(points[j].1 - points[i].1);
            let closed = area2 + cross(points[j], points[i]);
            if j < i + 2
                || chord > KERB_NOTCH_MAX_CHORD
                || closed.abs() / 2.0 >= KERB_NOTCH_MAX_AREA
            {
                continue;
            }
            // The farthest the detour gets from the chord.
            let depth = points[i + 1..j]
                .iter()
                .map(|p| {
                    ((p.0 - points[i].0) * (points[j].1 - points[i].1)
                        - (p.1 - points[i].1) * (points[j].0 - points[i].0))
                        .abs()
                        / chord.max(1.0)
                })
                .fold(0.0, f64::max);
            if depth >= KERB_NOTCH_MIN_DEPTH * chord && runs_along(&points, i, j, chord) {
                best = Some(j);
            }
        }
        let next = best.unwrap_or(i + 1);
        kept.push(object.coords[next]);
        i = next;
    }
    object.coords = kept;
    Some(object)
}

/// Whether the kerb before `points[i]` and after `points[j]` runs along the chord `i`..`j`
/// ([`KERB_NOTCH_MIN_COS`]). A side that has no point [`KERB_NOTCH_REFERENCE`] away does not
/// count; a chord of no length always passes.
fn runs_along(points: &[(f64, f64)], i: usize, j: usize, chord: f64) -> bool {
    if chord < 1.0 {
        return true;
    }
    let along = |from: usize, forward: bool| -> Option<(f64, f64)> {
        let origin = points[from];
        let mut k = from;
        loop {
            k = if forward { k + 1 } else { k.checked_sub(1)? };
            let p = *points.get(k)?;
            let (dx, dy) = (p.0 - origin.0, p.1 - origin.1);
            if dx.hypot(dy) >= KERB_NOTCH_REFERENCE {
                return Some(if forward { (dx, dy) } else { (-dx, -dy) });
            }
        }
    };
    let chord_dir = (points[j].0 - points[i].0, points[j].1 - points[i].1);
    [along(i, false), along(j, true)]
        .into_iter()
        .flatten()
        .all(|(dx, dy)| {
            (dx * chord_dir.0 + dy * chord_dir.1) / (dx.hypot(dy) * chord) >= KERB_NOTCH_MIN_COS
        })
}

/// A survey bus shelter with at least this share of its roof under ALKIS canopies is one of them.
const SHELTER_UNDER_CANOPY_SHARE: f64 = 0.5;

/// ALKIS wins over the survey for roofs too: a survey bus shelter (`br_fahrgastunterstand`, a
/// 522 candidate) with at least [`SHELTER_UNDER_CANOPY_SHARE`] of its area under the ALKIS
/// `canopies` (522 map objects) is dropped, and the canopy with the largest share names it in
/// `freemapper:also`. Everything else stays.
pub fn drop_shelters_under_canopies(
    berlin: Vec<MapObject>,
    canopies: &mut [MapObject],
) -> Vec<MapObject> {
    use geo::{Area as _, BooleanOps as _, Intersects as _};
    let roofs: Vec<(usize, geo_types::Polygon<f64>, geo_types::Rect<f64>)> = canopies
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let polygon = object_polygon(o)?;
            let bounds = polygon.bounding_rect()?;
            Some((i, polygon, bounds))
        })
        .collect();
    if roofs.is_empty() {
        return berlin;
    }
    let is_shelter = |o: &MapObject| {
        o.tags
            .iter()
            .any(|(k, v)| k == "berlin:layer" && v == "br_fahrgastunterstand")
    };
    let mut kept = Vec::with_capacity(berlin.len());
    for object in berlin {
        let shelter = is_shelter(&object)
            .then(|| object_polygon(&object))
            .flatten()
            .and_then(|p| p.bounding_rect().map(|b| (p, b)));
        let Some((shelter, bounds)) = shelter else {
            kept.push(object);
            continue;
        };
        let area = shelter.unsigned_area();
        let shares: Vec<(usize, f64)> = roofs
            .iter()
            .filter(|(_, _, b)| b.intersects(&bounds))
            .map(|(i, roof, _)| (*i, shelter.intersection(roof).unsigned_area()))
            .filter(|&(_, share)| share > 0.0)
            .collect();
        // Canopies do not overlap each other (ALKIS areas), so the shares add up.
        let covered: f64 = shares.iter().map(|(_, share)| share).sum();
        if area <= 0.0 || covered < SHELTER_UNDER_CANOPY_SHARE * area {
            kept.push(object);
            continue;
        }
        let winner = shares
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|&(i, _)| i)
            .expect("some share is under a canopy");
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
        add_also(&mut canopies[winner].tags, &also);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::dedup::{
        drop_cadastre_lines_by_rank, drop_osm_lines_near_cadastre, drop_points_near_registers,
        drop_trees_by_rank,
    };
    use crate::crs::ProjectedPoint;
    use crate::omap::MapCoord;
    use crate::omap::MapObjectKind;
    use freemapper_adapters_de::berlin::normalise_building;
    use freemapper_adapters_de::berlin::test_support::*;

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
        feature["properties"][freemapper_adapters_de::berlin::TYPE_KEY] = layer.into();
        freemapper_adapters_de::berlin::stamp_survey_date(&mut feature);
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

    /// A kerb line of the survey from `(x, y)` points in 1/1000 mm on paper (1:4000: 250 per
    /// ground metre).
    fn kerb(points: &[(i32, i32)]) -> MapObject {
        let set = MapKind::Sprint.symbol_set();
        MapObject {
            symbol_id: set.id_for_code("501.1").unwrap(),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&(x, y)| MapCoord::new(x, y)).collect(),
            tags: vec![(
                "freemapper:source".into(),
                "geoportal-berlin/sb-bordstein".into(),
            )],
        }
    }

    fn xy(object: &MapObject) -> Vec<(i32, i32)> {
        object.coords.iter().map(|c| (c.x, c.y)).collect()
    }

    /// A tree pit cut into the parking strip: the kerb leaves the straight line, runs around a
    /// 2 m by 4.5 m slot and comes back (about 0.8 mm² at 1:4000, the survey's own shape).
    fn kerb_with_pit() -> MapObject {
        kerb(&[
            (0, 0),
            (10_000, 0),
            (10_200, 400),
            (10_200, 1_100),
            (10_450, 1_250),
            (10_700, 1_100),
            (10_700, 400),
            (10_900, 0),
            (20_000, 0),
        ])
    }

    #[test]
    fn tree_pit_notch_leaves_the_kerb_straight() {
        let kept = drop_kerb_notches(kerb_with_pit()).expect("a long kerb stays");
        assert_eq!(xy(&kept), [(0, 0), (10_000, 0), (10_900, 0), (20_000, 0)]);
        assert_eq!(kept.symbol_id, kerb_with_pit().symbol_id);
        assert_eq!(kept.tags, kerb_with_pit().tags);
    }

    #[test]
    fn wide_trapezoid_bump_at_a_tree_is_straightened() {
        // The surveyed kerb at a Viktoriapark street tree (Li_13.04.2015_719): it leaves the edge
        // on a slant, rounds the tree and comes back on a slant. Chord 1.7 mm, way 3.2 mm, 1.3 mm²
        // at 1:4000: a way of only 1.8 times the chord, so the first rule let it stand.
        let bump = kerb(&[
            (49_236, -57_276),
            (50_228, -57_491),
            (50_434, -58_123),
            (50_639, -58_755),
            (50_694, -58_848),
            (50_785, -58_910),
            (50_890, -58_927),
            (50_996, -58_898),
            (51_078, -58_828),
            (51_539, -58_308),
            (51_934, -57_862),
            (52_257, -57_932),
            (52_818, -58_054),
            (53_241, -58_146),
            (54_280, -58_348),
        ]);
        let kept = drop_kerb_notches(bump).unwrap();
        assert!(
            kept.coords.iter().all(|c| c.y > -58_500),
            "the bump is gone: {:?}",
            xy(&kept)
        );
        assert_eq!(xy(&kept)[..2], [(49_236, -57_276), (50_228, -57_491)]);
        assert_eq!(xy(&kept).last(), Some(&(54_280, -58_348)));
    }

    #[test]
    fn a_notch_ends_on_the_line_not_at_its_apex() {
        // Viktoriapark, street edge along the park (Li_15.04.2015_251): a slanted trapezoid slot
        // at a tree. The rule of ADR 0071's first amendment cut from the vertex before it to the
        // slot's apex and drew a spike; the slot has to go back to the line at its far end.
        let slot = kerb(&[
            (-25_211, -40_924),
            (-23_160, -41_374),
            (-22_811, -41_451),
            (-22_463, -41_527),
            (-21_480, -41_743),
            (-19_813, -42_109),
            (-19_390, -42_507),
            (-19_861, -43_003),
            (-19_919, -43_093),
            (-19_935, -43_208),
            (-19_901, -43_317),
            (-19_832, -43_396),
            (-19_733, -43_442),
            (-19_616, -43_443),
            (-19_514, -43_395),
            (-19_476, -43_359),
            (-18_873, -42_706),
            (-18_759, -42_582),
            (-18_573, -42_382),
            (-14_659, -43_242),
        ]);
        let kept = xy(&drop_kerb_notches(slot).unwrap());
        assert!(
            kept.iter().all(|&(x, y)| x > -19_000 || y > -42_200),
            "no spike left of the exit: {kept:?}"
        );
        assert!(kept.contains(&(-18_573, -42_382)), "{kept:?}");
    }

    #[test]
    fn a_step_of_the_kerb_is_not_cut_diagonally() {
        // Viktoriapark, left of the X mark: the kerb steps back by 5.7 m and takes a tab on the
        // way. There is no line to return to, so the corner stays as the survey has it.
        let step = kerb(&[
            (-40_386, -37_712),
            (-35_991, -38_676),
            (-35_463, -38_980),
            (-35_946, -39_587),
            (-35_962, -39_671),
            (-35_947, -39_764),
            (-35_898, -39_848),
            (-35_825, -39_907),
            (-35_744, -39_935),
            (-35_391, -40_003),
            (-34_949, -40_102),
            (-34_078, -40_298),
            (-33_608, -40_399),
            (-32_016, -40_745),
            (-31_517, -40_854),
        ]);
        assert_eq!(xy(&drop_kerb_notches(step.clone()).unwrap()), xy(&step));
    }

    #[test]
    fn a_closed_ring_with_the_close_flag_is_dropped_too() {
        // The survey ring as the builder writes it: the last coordinate carries FLAG_CLOSE and
        // FLAG_HOLE (flags 18). A pit ring of 1.8 mm by 0.8 mm (1.2 mm²) from Viktoriapark.
        use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
        let mut ring = kerb(&[(0, 0), (1_800, 0), (1_800, 800), (0, 800), (0, 0)]);
        ring.coords[4].flags = FLAG_CLOSE | FLAG_HOLE;
        assert!(drop_kerb_notches(ring).is_none());
    }

    #[test]
    fn a_bay_larger_than_the_minimum_area_stays() {
        // A 12 m by 6 m bay (4.5 mm²) is a feature of the street, not a pit.
        let bay = kerb(&[
            (0, 0),
            (10_000, 0),
            (10_000, 1_500),
            (13_000, 1_500),
            (13_000, 0),
            (20_000, 0),
        ]);
        assert_eq!(xy(&drop_kerb_notches(bay.clone()).unwrap()), xy(&bay));
    }

    #[test]
    fn a_curve_is_not_a_notch() {
        // A quarter circle of 3 mm radius at the corner: path and chord are nearly equal.
        let corner: Vec<(i32, i32)> = (0..=12)
            .map(|i| {
                let a = f64::from(i) / 12.0 * std::f64::consts::FRAC_PI_2;
                (
                    (3_000.0 * a.sin()) as i32,
                    (3_000.0 * (1.0 - a.cos())) as i32,
                )
            })
            .collect();
        let line = kerb(&corner);
        assert_eq!(xy(&drop_kerb_notches(line.clone()).unwrap()), xy(&line));
    }

    #[test]
    fn a_tiny_closed_kerb_ring_is_dropped() {
        // A tree pit ringed by its own kerb: 2 m by 2.5 m, 0.3 mm².
        let ring = kerb(&[(0, 0), (500, 0), (500, 625), (0, 625), (0, 0)]);
        assert!(drop_kerb_notches(ring).is_none());
        // A square of 12 m by 12 m stays.
        let square = kerb(&[(0, 0), (3_000, 0), (3_000, 3_000), (0, 3_000), (0, 0)]);
        assert!(drop_kerb_notches(square).is_some());
    }

    #[test]
    fn railings_are_impassable_fence_candidates() {
        let mut railings = stored("bf_gelaender", SB_GELAENDER);
        railings.extend(stored("bb_verkehrsschutzgitter", SB_VERKEHRSSCHUTZGITTER));
        let mut barrier = railings[0].clone();
        barrier["properties"][freemapper_adapters_de::berlin::TYPE_KEY] = "bg_leitplanke".into();
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
    fn schoolyard_furniture_is_544_and_321_and_541() {
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
        // A schoolyard map has no control sites.
        assert!(
            found
                .iter()
                .all(|o| object_tag(o, "freemapper:control").is_none())
        );
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
        // Hydrants and play equipment are no sprint candidates: the norm names hydrants as
        // not to be mapped, and most of the surveyed ones are flush covers.
        assert!(survey_objects("sb-moebel", &others, MapKind::Sprint).is_empty());
    }

    #[test]
    fn survey_furniture_gives_sprint_control_sites() {
        let furniture: Vec<Value> = ["av_poller", "as_mast", "am_fahnenmast"]
            .iter()
            .enumerate()
            .map(|(i, layer)| {
                survey_point(layer, &i.to_string(), (392_500.0 + i as f64, 5_820_500.0))
            })
            .collect();
        let found = survey_objects("sb-moebel", &furniture, MapKind::Sprint);
        let set = MapKind::Sprint.symbol_set();
        assert_eq!(found.len(), 3, "{found:?}");
        assert!(
            found
                .iter()
                .all(|o| Some(o.symbol_id) == set.id_for_code("531")
                    && matches!(o.kind, MapObjectKind::Point { .. }))
        );
        let kinds: Vec<_> = found
            .iter()
            .map(|o| object_tag(o, "freemapper:control"))
            .collect();
        assert_eq!(kinds, [Some("bollard"), Some("mast"), Some("flagpole")]);
        assert_eq!(object_tag(&found[0], "berlin:pot"), Some("2"));
        assert_eq!(
            object_tag(&found[0], "freemapper:source"),
            Some("geoportal-berlin/sb-moebel")
        );
    }

    #[test]
    fn a_surveyed_bench_is_a_line_on_schoolyard_and_a_control_site_on_sprint() {
        let bench = survey_feature(
            "bj_sitzbank",
            serde_json::json!({
                "type": "Feature", "id": "bj_sitzbank.1",
                "geometry": {"type": "MultiLineString", "coordinates": [[[392_500.0, 5_820_500.0], [392_502.0, 5_820_500.0]]]},
                "properties": {"gis_id": "Li_25.09.2014_1"}
            }),
        );
        let school = survey_objects(
            "sb-moebel",
            std::slice::from_ref(&bench),
            MapKind::Schoolyard,
        );
        assert_eq!(school.len(), 1);
        assert!(matches!(school[0].kind, MapObjectKind::Path));
        assert_eq!(
            Some(school[0].symbol_id),
            MapKind::Schoolyard.symbol_set().id_for_code("543")
        );
        let sprint = survey_objects("sb-moebel", &[bench], MapKind::Sprint);
        assert_eq!(sprint.len(), 1, "{sprint:?}");
        assert_eq!(
            Some(sprint[0].symbol_id),
            MapKind::Sprint.symbol_set().id_for_code("531")
        );
        assert_eq!(object_tag(&sprint[0], "freemapper:control"), Some("bench"));
        // The point is the middle of the bench: 392501, 5820500.
        let (georef, _, _) = setup(MapKind::Sprint);
        let at = georef.map_to_projected(sprint[0].coords[0]);
        assert!((at.easting - 392_501.0).abs() < 0.05, "{at:?}");
        assert!((at.northing - 5_820_500.0).abs() < 0.05, "{at:?}");
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
        assert_eq!(
            object_tag(&found[0], "freemapper:control"),
            Some("advertising_column")
        );
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
            ["522", "311", "311", "313"]
                .map(|c| set.id_for_code(c))
                .to_vec()
        );
        let set = MapKind::Schoolyard.symbol_set();
        assert_eq!(
            codes(MapKind::Schoolyard),
            ["522", "311", "311", "313"]
                .map(|c| set.id_for_code(c))
                .to_vec()
        );
    }

    #[test]
    fn survey_shelter_under_an_alkis_canopy_is_dropped() {
        let kind = MapKind::Sprint;
        let (georef, area, set) = setup(kind);
        let rect = |e: f64, n: f64, w: f64, h: f64| {
            serde_json::json!([[[e, n], [e + w, n], [e + w, n + h], [e, n + h], [e, n]]])
        };
        // An ALKIS canopy of 10 × 4 m.
        let canopy = serde_json::json!({
            "type": "Feature", "id": "bauwerkeflaechen.D",
            "geometry": {"type": "Polygon", "coordinates": rect(392_500.0, 5_820_500.0, 10.0, 4.0)},
            "properties": {"uuid": "DEBE00DACH", "bezbwf": "Überdachung"}
        });
        let mut canopies = crate::build::canopy::canopy_objects(
            &[crate::build::canopy::CanopyInput::Berlin(&[canopy])],
            &[],
            &area,
            20.0,
            &georef,
            &set,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap();
        assert_eq!(canopies.len(), 1);
        let shelter = |id: &str, e: f64, n: f64| {
            survey_feature(
                "br_fahrgastunterstand",
                serde_json::json!({
                    "type": "Feature", "id": format!("br_fahrgastunterstand.{id}"),
                    "geometry": {"type": "Polygon", "coordinates": rect(e, n, 4.0, 2.0)},
                    "properties": {"gis_id": format!("Fl_20150413_{id}")}
                }),
            )
        };
        let shelters = [
            // Wholly under the canopy: the same roof.
            shelter("1", 392_501.0, 5_820_501.0),
            // A quarter under it: another roof beside it.
            shelter("2", 392_509.0, 5_820_500.0),
            // Far away.
            shelter("3", 392_550.0, 5_820_500.0),
        ];
        let mut berlin = survey_objects("sb-objekte", &shelters, kind);
        // Another 522 candidate under the canopy that is no shelter stays.
        let mut passage = berlin[0].clone();
        passage.tags = vec![("berlin:layer".into(), "building-parts".into())];
        berlin.push(passage);
        assert_eq!(berlin.len(), 4);
        let kept = drop_shelters_under_canopies(berlin, &mut canopies);
        let ids: Vec<&str> = kept
            .iter()
            .map(|o| object_tag(o, "freemapper:source_id").unwrap_or("passage"))
            .collect();
        assert_eq!(ids, ["Fl_20150413_2", "Fl_20150413_3", "passage"]);
        assert_eq!(
            object_tag(&canopies[0], "freemapper:also"),
            Some("geoportal-berlin/sb-objekte/Fl_20150413_1")
        );
        // Without canopies every shelter stays.
        let berlin = survey_objects("sb-objekte", &shelters, kind);
        assert_eq!(drop_shelters_under_canopies(berlin, &mut []).len(), 3);
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
        let kept = drop_cadastre_lines_by_rank(berlin, &set, 1000);
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
        // The crown diameter is the generalization's (`build/points.rs`).
        assert!(
            first
                .tags
                .contains(&("freemapper:crown_m".into(), "10".into()))
        );
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
        let (kept, _) = drop_trees_by_rank(
            vec![
                tree(1.0, 1.0),                          // 1.4 m away: the same tree
                tree(0.0, 1.99),                         // just inside 2 m
                tree(0.0, 2.5),                          // 2.5 m: another tree
                osm("526", "tourism=artwork", 0.0, 0.5), // not a tree
            ],
            cadastre,
            &set,
            georef.scale_denominator,
        );
        let osm_tags: Vec<&str> = kept.iter().map(|o| o.tags[1].1.as_str()).collect();
        assert_eq!(osm_tags, ["natural=tree", "tourism=artwork"]);
        // Without cadastre trees every OSM tree stays.
        assert_eq!(
            drop_trees_by_rank(vec![tree(0.0, 0.0)], vec![], &set, georef.scale_denominator)
                .0
                .len(),
            1
        );
    }

    /// A point of a register (`beleuchtung`, `zierbrunnen`, `trinkwasserbrunnen`): stored with
    /// the type name, no survey date.
    fn register_point(layer: &str, id: &str, at: (f64, f64)) -> Value {
        serde_json::json!({
            "type": "Feature", "id": format!("{layer}.{id}"),
            "geometry": {"type": "Point", "coordinates": [at.0, at.1]},
            "properties": {"id": id, freemapper_adapters_de::berlin::TYPE_KEY: layer}
        })
    }

    /// An OSM point candidate `de`, `dn` metres from Alexanderplatz (392400, 5820400).
    fn osm_point(kind: MapKind, code: &str, osm: &str, id: &str, de: f64, dn: f64) -> MapObject {
        let (georef, _, set) = setup(kind);
        MapObject {
            symbol_id: set.id_for_code(code).unwrap(),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![
                georef
                    .projected_to_map(ProjectedPoint {
                        easting: 392_400.0 + de,
                        northing: 5_820_400.0 + dn,
                    })
                    .unwrap(),
            ],
            tags: vec![
                ("freemapper:source".into(), "osm".into()),
                ("freemapper:source_id".into(), id.into()),
                ("freemapper:osm".into(), osm.into()),
            ],
        }
    }

    fn ids(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .filter_map(|o| object_tag(o, "freemapper:source_id"))
            .collect()
    }

    #[test]
    fn lamps_of_the_cadastre_are_schoolyard_candidates() {
        let kind = MapKind::Schoolyard;
        let lamps = [register_point("beleuchtung", "1", (392_400.0, 5_820_400.0))];
        let set = kind.symbol_set();
        let lamp = survey_objects("beleuchtung", &lamps, kind);
        assert_eq!(lamp.len(), 1);
        assert_eq!(Some(lamp[0].symbol_id), set.id_for_code("545"));
        assert_eq!(
            object_tag(&lamp[0], "freemapper:source"),
            Some("geoportal-berlin/beleuchtung")
        );
        assert_eq!(object_tag(&lamp[0], "berlin:layer"), Some("beleuchtung"));
        assert!(object_tag(&lamp[0], "freemapper:control").is_none());
    }

    #[test]
    fn lamps_of_the_cadastre_are_sprint_control_sites() {
        let lamps = [register_point("beleuchtung", "1", (392_400.0, 5_820_400.0))];
        let lamp = survey_objects("beleuchtung", &lamps, MapKind::Sprint);
        assert_eq!(lamp.len(), 1);
        assert_eq!(
            Some(lamp[0].symbol_id),
            MapKind::Sprint.symbol_set().id_for_code("531")
        );
        assert_eq!(
            object_tag(&lamp[0], "freemapper:control"),
            Some("street_lamp")
        );
        assert_eq!(
            object_tag(&lamp[0], "freemapper:source"),
            Some("geoportal-berlin/beleuchtung")
        );
    }

    #[test]
    fn drinking_fountains_are_313_on_sprint_too() {
        let set = MapKind::Sprint.symbol_set();
        let register = [register_point(
            "trinkwasserbrunnen",
            "1",
            (392_400.0, 5_820_400.0),
        )];
        let found = survey_objects("trinkwasserbrunnen", &register, MapKind::Sprint);
        assert_eq!(found.len(), 1);
        assert_eq!(Some(found[0].symbol_id), set.id_for_code("313"));
        let survey = [survey_point(
            "ac_trinkwasserbrunnen_wasserspender",
            "1",
            (392_410.0, 5_820_400.0),
        )];
        let found = survey_objects("sb-objekte", &survey, MapKind::Sprint);
        assert_eq!(found.len(), 1);
        assert_eq!(Some(found[0].symbol_id), set.id_for_code("313"));
        assert!(object_tag(&found[0], "freemapper:control").is_none());
    }

    #[test]
    fn lamp_cadastre_replaces_osm_lamp_nearby() {
        let kind = MapKind::Schoolyard;
        let (georef, _, _) = setup(kind);
        let lamps = [register_point("beleuchtung", "7", (392_400.0, 5_820_400.0))];
        // A survey mast is the lamp only within 2 m; 2.5 m away it is a sign or signal mast.
        let masts = [
            survey_point("as_mast", "near", (392_401.5, 5_820_400.0)),
            survey_point("as_mast", "far", (392_402.5, 5_820_400.0)),
            survey_point("av_poller", "post", (392_400.5, 5_820_400.0)),
        ];
        let mut berlin = survey_objects("beleuchtung", &lamps, kind);
        berlin.extend(survey_objects("sb-moebel", &masts, kind));
        // An OSM lamp is the same lamp up to 5 m away (live data: 2 to 6.7 m off, lamps 20 to
        // 30 m apart).
        let osm = vec![
            osm_point(kind, "545", "highway=street_lamp", "node/1", 1.0, 1.0), // 1.4 m
            osm_point(kind, "545", "highway=street_lamp", "node/2", 0.0, 2.5), // 2.5 m
            osm_point(kind, "545", "highway=street_lamp", "node/4", 3.0, 3.9), // 4.9 m
            osm_point(kind, "545", "highway=street_lamp", "node/5", 0.0, 5.5), // 5.5 m
            osm_point(kind, "544", "man_made=mast", "node/3", 0.0, 0.5),       // not a lamp
        ];
        let (osm, berlin) =
            drop_points_near_registers(osm, berlin, &kind.symbol_set(), georef.scale_denominator);
        assert_eq!(ids(&osm), ["node/5", "node/3"]);
        let layers: Vec<_> = berlin
            .iter()
            .filter_map(|o| object_tag(o, "berlin:layer"))
            .collect();
        assert_eq!(layers, ["beleuchtung", "as_mast", "av_poller"]);
        let far = berlin
            .iter()
            .find(|o| object_tag(o, "freemapper:source_id").is_some_and(|i| i.contains("far")))
            .map(|o| object_tag(o, "freemapper:source_id"));
        assert!(far.is_some(), "the far mast stays");
        // The lamp names what it replaced.
        let also = object_tag(&berlin[0], "freemapper:also").unwrap();
        assert!(also.contains("osm/node/1"), "{also}");
        assert!(also.contains("geoportal-berlin/sb-moebel/"), "{also}");
        assert!(
            also.contains("osm/node/2") && also.contains("osm/node/4"),
            "{also}"
        );
        assert!(!also.contains("node/5"), "{also}");
    }

    #[test]
    fn survey_furniture_replaces_osm_furniture_nearby() {
        let kind = MapKind::Schoolyard;
        let (georef, _, _) = setup(kind);
        let (e, n) = (392_400.0, 5_820_400.0);
        let furniture = [
            survey_point("aq_hydrant", "hydrant", (e, n)),
            survey_point("bj_sitzbank", "bench", (e + 20.0, n)),
            survey_point("am_fahnenmast", "flag", (e + 40.0, n)),
            survey_point("aw_spielgeraet", "swing", (e + 60.0, n)),
            survey_point("av_poller", "post", (e + 80.0, n)),
        ];
        let berlin = survey_objects("sb-moebel", &furniture, kind);
        assert_eq!(berlin.len(), 5);
        // OSM benches and play equipment are short lines, like the survey ones.
        let short_line = |mut o: MapObject| {
            let c = o.coords[0];
            o.kind = MapObjectKind::Path;
            o.coords = vec![
                crate::omap::MapCoord::new(c.x - 750, c.y),
                crate::omap::MapCoord::new(c.x + 750, c.y),
            ];
            o
        };
        let hydrant = "emergency=fire_hydrant; fire_hydrant:type=pillar";
        let osm = vec![
            osm_point(kind, "321", hydrant, "node/1", 1.5, 0.0), // the same hydrant
            osm_point(kind, "321", "emergency=fire_hydrant", "node/2", 0.0, 2.5), // another
            short_line(osm_point(kind, "543", "amenity=bench", "node/3", 21.0, 1.0)),
            osm_point(kind, "544", "man_made=flagpole", "node/4", 40.5, 0.0),
            short_line(osm_point(
                kind,
                "541",
                "playground=swing",
                "node/5",
                60.0,
                1.5,
            )),
            // A bollard is no mast: the OSM mast beside it stays.
            osm_point(kind, "544", "man_made=mast", "node/6", 80.5, 0.0),
        ];
        let (osm, berlin) =
            drop_points_near_registers(osm, berlin, &kind.symbol_set(), georef.scale_denominator);
        assert_eq!(ids(&osm), ["node/2", "node/6"]);
        assert_eq!(berlin.len(), 5, "the survey wins");
        let also = |layer: &str| {
            berlin
                .iter()
                .find(|o| object_tag(o, "berlin:layer") == Some(layer))
                .and_then(|o| object_tag(o, "freemapper:also"))
                .map(str::to_string)
        };
        assert_eq!(also("aq_hydrant").as_deref(), Some("osm/node/1"));
        assert_eq!(also("bj_sitzbank").as_deref(), Some("osm/node/3"));
        assert_eq!(also("am_fahnenmast").as_deref(), Some("osm/node/4"));
        assert_eq!(also("aw_spielgeraet").as_deref(), Some("osm/node/5"));
        assert_eq!(also("av_poller"), None);
    }

    #[test]
    fn sprint_control_sites_of_all_sources_are_thinned_together() {
        use crate::build::control_sites::drop_ambiguous;
        let kind = MapKind::Sprint;
        let (georef, _, set) = setup(kind);
        let (e, n) = (392_400.0, 5_820_400.0);
        // A lone surveyed bollard with the OSM one 1.5 m off, a row of three surveyed bollards,
        // a register lamp with the OSM lamp 3 m off, a surveyed bench with the OSM bench 1 m off.
        let furniture = [
            survey_point("av_poller", "lone", (e, n)),
            survey_point("av_poller", "row1", (e + 100.0, n)),
            survey_point("av_poller", "row2", (e + 103.0, n)),
            survey_point("av_poller", "row3", (e + 106.0, n)),
            survey_point("bj_sitzbank", "bench", (e + 200.0, n)),
        ];
        let lamps = [register_point("beleuchtung", "7", (e + 300.0, n))];
        let mut berlin = survey_objects("sb-moebel", &furniture, kind);
        berlin.extend(survey_objects("beleuchtung", &lamps, kind));
        let osm = vec![
            osm_point(kind, "531", "barrier=bollard", "node/1", 1.5, 0.0),
            osm_point(kind, "531", "highway=street_lamp", "node/2", 303.0, 0.0),
            osm_point(kind, "531", "amenity=bench", "node/3", 201.0, 0.0),
            // Far from every survey object: stays.
            osm_point(kind, "531", "amenity=waste_basket", "node/4", 400.0, 0.0),
        ];
        let osm: Vec<MapObject> = osm
            .into_iter()
            .map(|mut o| {
                let osm = object_tag(&o, "freemapper:osm").unwrap_or_default();
                let site = match osm {
                    "barrier=bollard" => "bollard",
                    "highway=street_lamp" => "street_lamp",
                    "amenity=bench" => "bench",
                    _ => "waste_basket",
                };
                o.tags.push(("freemapper:control".into(), site.into()));
                o
            })
            .collect();
        let (osm, berlin) = drop_points_near_registers(osm, berlin, &set, georef.scale_denominator);
        assert_eq!(ids(&osm), ["node/4"]);
        let mut all = osm;
        all.extend(berlin);
        let kept = drop_ambiguous(all, georef.scale_denominator);
        let mut kinds: Vec<_> = kept
            .iter()
            .map(|o| object_tag(o, "freemapper:control").unwrap())
            .collect();
        kinds.sort_unstable();
        // The row is no site; the lone bollard, the bench, the lamp and the basket stay once.
        assert_eq!(kinds, ["bench", "bollard", "street_lamp", "waste_basket"]);
        let bollard = kept
            .iter()
            .find(|o| object_tag(o, "freemapper:control") == Some("bollard"))
            .unwrap();
        assert_eq!(
            object_tag(bollard, "freemapper:source"),
            Some("geoportal-berlin/sb-moebel")
        );
        assert_eq!(object_tag(bollard, "freemapper:also"), Some("osm/node/1"));
    }

    #[test]
    fn survey_mast_at_a_register_lamp_does_not_drop_osm_masts() {
        // The register lamp drops the survey mast first; a dropped mast wins nothing.
        let kind = MapKind::Schoolyard;
        let (georef, _, _) = setup(kind);
        let lamps = [register_point("beleuchtung", "7", (392_400.0, 5_820_400.0))];
        let masts = [survey_point("as_mast", "lamp", (392_401.0, 5_820_400.0))];
        let mut berlin = survey_objects("beleuchtung", &lamps, kind);
        berlin.extend(survey_objects("sb-moebel", &masts, kind));
        let osm = vec![osm_point(kind, "544", "man_made=mast", "node/1", 2.5, 0.0)];
        let (osm, berlin) =
            drop_points_near_registers(osm, berlin, &kind.symbol_set(), georef.scale_denominator);
        assert_eq!(ids(&osm), ["node/1"]);
        assert_eq!(berlin.len(), 1);
    }

    #[test]
    fn fountains_from_both_sources_dedup() {
        let kind = MapKind::Schoolyard;
        let (georef, _, _) = setup(kind);
        let (e, n) = (392_400.0, 5_820_400.0);
        let fountains = [register_point("bez_zierbrunnen", "ZB-1", (e, n))];
        let drinking = [register_point("trinkwasserbrunnen", "TW-1", (e + 100.0, n))];
        let objects = [
            survey_point("bv_springbrunnen_zierbrunnen", "jet", (e + 2.0, n)),
            survey_point("af_wasserpumpen_brunnen", "pump", (e + 30.0, n)),
            survey_point("ac_trinkwasserbrunnen_wasserspender", "tap", (e + 101.0, n)),
        ];
        let mut berlin = survey_objects("zierbrunnen", &fountains, kind);
        berlin.extend(survey_objects("trinkwasserbrunnen", &drinking, kind));
        berlin.extend(survey_objects("sb-objekte", &objects, kind));
        assert_eq!(berlin.len(), 5, "rules for both registers and the survey");
        let osm = vec![
            osm_point(kind, "311", "amenity=fountain", "node/10", 2.9, 0.0), // same fountain
            osm_point(kind, "311", "amenity=fountain", "node/11", 3.5, 0.0), // another one
            osm_point(kind, "313", "amenity=drinking_water", "node/12", 0.5, 0.0), // other kind
            osm_point(kind, "313", "amenity=drinking_water", "node/13", 100.0, 2.0),
        ];
        let (osm, berlin) =
            drop_points_near_registers(osm, berlin, &kind.symbol_set(), georef.scale_denominator);
        assert_eq!(ids(&osm), ["node/11", "node/12"]);
        // The survey jet at the register fountain and the survey tap at the register tap go;
        // the pump 30 m away stays.
        let layers: Vec<_> = berlin
            .iter()
            .filter_map(|o| object_tag(o, "berlin:layer"))
            .collect();
        assert_eq!(
            layers,
            [
                "bez_zierbrunnen",
                "trinkwasserbrunnen",
                "af_wasserpumpen_brunnen"
            ]
        );
        let also = object_tag(&berlin[0], "freemapper:also").unwrap();
        assert!(also.contains("osm/node/10"), "{also}");
        // Without registers nothing is dropped.
        let osm = vec![osm_point(
            kind,
            "311",
            "amenity=fountain",
            "node/1",
            0.0,
            0.0,
        )];
        let (osm, _) =
            drop_points_near_registers(osm, vec![], &kind.symbol_set(), georef.scale_denominator);
        assert_eq!(osm.len(), 1);
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
        let kept = drop_osm_lines_near_cadastre(all, &mut alkis, &set, georef.scale_denominator);
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
            drop_osm_lines_near_cadastre(lone, &mut [], &set, georef.scale_denominator).len(),
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
