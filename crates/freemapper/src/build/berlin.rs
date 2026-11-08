//! Berlin ALKIS structures and building parts → candidate objects (Berlin plus 1).
//!
//! The Geoportal features run through the same [`Rules`] engine as the OSM features
//! (`assets/rules/berlin-candidates.toml`). A feature becomes a tagged feature first: its string
//! and number properties become `berlin:<name>` tags, so no OSM rule can match one and no Berlin
//! rule an OSM tag. The building parts are parsed from the stored buildings in one place
//! ([`building_part_features`]).

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

/// The tags of a Berlin feature: `berlin:layer`, each string (not empty) and number property as
/// `berlin:<name>`, and the provenance tags the objects carry on (`freemapper:source_id`, the
/// ALKIS name as `freemapper:alkis`).
fn feature_tags(layer: &str, feature: &Value) -> Vec<(String, String)> {
    let mut tags = vec![("berlin:layer".to_string(), layer.to_string())];
    let properties = feature["properties"].as_object();
    for (key, value) in properties.into_iter().flatten() {
        let text = match value {
            Value::String(s) if !s.is_empty() => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => continue,
        };
        tags.push((format!("berlin:{key}"), text));
    }
    let property = |key: &str| properties.and_then(|p| p.get(key)).and_then(Value::as_str);
    let source_id = ["uuid", "gisid", "oid"]
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
    fn freitreppe_line_is_a_stair_candidate() {
        let line = some_line();
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let tags = [
                ("berlin:layer", "bauwerke-linien"),
                ("berlin:bezbwf", "Freitreppe"),
            ];
            assert_eq!(code(&tags, &line, kind), Some("532.7".to_string()));
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

    fn some_point() -> OsmGeometry {
        OsmGeometry::Point(LatLon {
            lat: 52.5,
            lon: 13.4,
        })
    }
}
