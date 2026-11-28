// SPDX-License-Identifier: AGPL-3.0-or-later

//! Buildings from OpenStreetMap, for a region whose adapter has no register of them. They are
//! written in the shape of the cadastre buildings (`gebnutzbez`, `funktion`, `rellage`), so the
//! build, the vegetation mask and the export read them like register buildings.

use serde_json::{Value, json};

use crate::crs::{Crs, LatLon};
use crate::sources::osm::{OsmFeature, OsmGeometry, OsmPolygon};
use freemapper_sources::ogc_api::feature_collection;

/// The `gebnutzbez` of a real building, as the build reads it.
const BUILDING_KIND: &str = "Gebäude";
/// The `rellage` of a building under the ground, as the build reads it.
const UNDERGROUND: &str = "Unter der Erdoberfläche";

/// Whether a feature with `tags` is a building to draw: any `building` value but `no`.
pub fn is_building(tags: &[(String, String)]) -> bool {
    tags.iter().any(|(k, v)| k == "building" && v != "no")
}

fn tag<'a>(tags: &'a [(String, String)], key: &str) -> Option<&'a str> {
    tags.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// A building below the ground: `location=underground` or a negative `layer`.
fn is_underground(tags: &[(String, String)]) -> bool {
    tag(tags, "location") == Some("underground")
        || tag(tags, "layer")
            .and_then(|l| l.trim().parse::<i32>().ok())
            .is_some_and(|layer| layer < 0)
}

/// The building polygons of `features` as a GeoJSON FeatureCollection in the CRS `crs`: closed
/// ways and multipolygon relations with a `building` tag, one MultiPolygon each. Open ways, nodes
/// and `building=no` are left out.
pub fn buildings_geojson(features: &[OsmFeature], crs: Crs) -> String {
    let point = |p: &LatLon| {
        let q = crs.forward(*p);
        json!([q.easting, q.northing])
    };
    let ring = |points: &[LatLon]| Value::Array(points.iter().map(point).collect());
    let polygon = |outer: &[LatLon], holes: &[Vec<LatLon>]| {
        let mut rings = vec![ring(outer)];
        rings.extend(holes.iter().map(|h| ring(h)));
        Value::Array(rings)
    };
    let mut out = Vec::new();
    for feature in features.iter().filter(|f| is_building(&f.tags)) {
        let polygons: Vec<Value> = match &feature.geometry {
            OsmGeometry::Line(points) if points.len() >= 4 && points.first() == points.last() => {
                vec![polygon(points, &[])]
            }
            OsmGeometry::Area(areas) => areas
                .iter()
                .map(|OsmPolygon { outer, holes }| polygon(outer, holes))
                .collect(),
            _ => continue,
        };
        let mut properties = json!({
            "oid": feature.id,
            "gebnutzbez": BUILDING_KIND,
            "funktion": tag(&feature.tags, "building").unwrap_or("yes"),
        });
        if is_underground(&feature.tags) {
            properties["rellage"] = json!(UNDERGROUND);
        }
        out.push(json!({
            "type": "Feature",
            "id": feature.id,
            "geometry": { "type": "MultiPolygon", "coordinates": polygons },
            "properties": properties,
        }));
    }
    feature_collection(out)
}

#[cfg(test)]
mod tests {
    use freemapper_sources::ogc_api::parse_buildings;

    use super::*;

    fn tags(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn at(lat: f64, lon: f64) -> LatLon {
        LatLon { lat, lon }
    }

    /// A closed way of a small square at 59.29 N, 18.20 E.
    fn square(id: &str, pairs: &[(&str, &str)]) -> OsmFeature {
        let ring = vec![
            at(59.2900, 18.2000),
            at(59.2900, 18.2002),
            at(59.2902, 18.2002),
            at(59.2902, 18.2000),
            at(59.2900, 18.2000),
        ];
        OsmFeature {
            id: id.to_string(),
            tags: tags(pairs),
            geometry: OsmGeometry::Line(ring),
        }
    }

    fn parsed(features: &[OsmFeature]) -> Vec<freemapper_sources::ogc_api::Building> {
        parse_buildings(&buildings_geojson(features, Crs::Utm33))
            .unwrap()
            .buildings
    }

    #[test]
    fn any_building_value_but_no_is_a_building() {
        assert!(is_building(&tags(&[("building", "yes")])));
        assert!(is_building(&tags(&[("building", "house"), ("name", "x")])));
        assert!(!is_building(&tags(&[("building", "no")])));
        assert!(!is_building(&tags(&[("highway", "path")])));
        assert!(!is_building(&[]));
    }

    #[test]
    fn a_closed_way_becomes_a_building_in_the_map_crs() {
        let buildings = parsed(&[square("way/7", &[("building", "house")])]);
        assert_eq!(buildings.len(), 1);
        let building = &buildings[0];
        assert_eq!(building.oid, "way/7");
        assert_eq!(building.kind, "Gebäude");
        assert_eq!(building.function, "house");
        assert!(!building.underground);
        let corner = Crs::Utm33.forward(at(59.2900, 18.2000));
        let first = building.footprint.0[0].exterior().0[0];
        assert!((first.x - corner.easting).abs() < 1e-6, "{first:?}");
        assert!((first.y - corner.northing).abs() < 1e-6, "{first:?}");
        assert_eq!(building.footprint.0[0].exterior().0.len(), 5);
    }

    #[test]
    fn what_is_no_building_is_left_out() {
        let open = OsmFeature {
            id: "way/8".into(),
            tags: tags(&[("building", "yes")]),
            geometry: OsmGeometry::Line(vec![at(59.0, 18.0), at(59.1, 18.1), at(59.2, 18.2)]),
        };
        let node = OsmFeature {
            id: "node/9".into(),
            tags: tags(&[("building", "yes")]),
            geometry: OsmGeometry::Point(at(59.0, 18.0)),
        };
        let features = [
            square("way/1", &[("building", "no")]),
            square("way/2", &[("highway", "service")]),
            open,
            node,
            square("way/3", &[("building", "yes")]),
        ];
        let buildings = parsed(&features);
        assert_eq!(
            buildings.iter().map(|b| b.oid.as_str()).collect::<Vec<_>>(),
            ["way/3"]
        );
    }

    #[test]
    fn a_relation_keeps_its_holes() {
        let outer = vec![
            at(59.2900, 18.2000),
            at(59.2900, 18.2010),
            at(59.2910, 18.2010),
            at(59.2910, 18.2000),
            at(59.2900, 18.2000),
        ];
        let hole = vec![
            at(59.2903, 18.2003),
            at(59.2903, 18.2006),
            at(59.2906, 18.2006),
            at(59.2906, 18.2003),
            at(59.2903, 18.2003),
        ];
        let relation = OsmFeature {
            id: "relation/5".into(),
            tags: tags(&[("building", "yes"), ("type", "multipolygon")]),
            geometry: OsmGeometry::Area(vec![OsmPolygon {
                outer,
                holes: vec![hole],
            }]),
        };
        let buildings = parsed(&[relation]);
        assert_eq!(buildings.len(), 1);
        assert_eq!(buildings[0].footprint.0.len(), 1);
        assert_eq!(buildings[0].footprint.0[0].interiors().len(), 1);
    }

    #[test]
    fn a_building_under_the_ground_is_marked_so() {
        let features = [
            square("way/1", &[("building", "yes"), ("location", "underground")]),
            square("way/2", &[("building", "yes"), ("layer", "-1")]),
            square("way/3", &[("building", "yes"), ("layer", "1")]),
            square("way/4", &[("building", "yes"), ("layer", "x")]),
            square("way/5", &[("building", "yes"), ("layer", "0")]),
        ];
        let underground: Vec<bool> = parsed(&features).iter().map(|b| b.underground).collect();
        assert_eq!(underground, [true, true, false, false, false]);
    }

    /// Writes the extract of the Swedish end-to-end test: a house and a path in its map area.
    /// Run once with `--ignored`; the file is committed.
    #[test]
    #[ignore = "writes tests/golden/adapters/sweden-building.osm.pbf"]
    fn write_the_swedish_fixture() {
        let at = |e: f64, n: f64| {
            let p = crate::crs::utm33_inverse(crate::crs::ProjectedPoint {
                easting: e,
                northing: n,
            })
            .unwrap();
            (p.lat, p.lon)
        };
        let corners = [
            (1, at(385_070.0, 6_177_600.0)),
            (2, at(385_090.0, 6_177_600.0)),
            (3, at(385_090.0, 6_177_615.0)),
            (4, at(385_070.0, 6_177_615.0)),
            (5, at(385_040.0, 6_177_560.0)),
            (6, at(385_170.0, 6_177_580.0)),
        ];
        let nodes: Vec<_> = corners
            .iter()
            .map(|&(id, (lat, lon))| (id, lat, lon, &[][..]))
            .collect();
        let bytes = crate::fetch::test_support::osm_pbf_with(
            &nodes,
            &[
                (100, &[1, 2, 3, 4, 1][..], &[("building", "house")][..]),
                (200, &[5, 6][..], &[("highway", "path")][..]),
            ],
            &[],
        );
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/golden/adapters/sweden-building.osm.pbf");
        std::fs::write(path, bytes).unwrap();
    }
}
