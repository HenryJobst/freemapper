//! ALKIS buildings → 521 building objects (spec §4.5).

use geo::{Area as _, Distance as _, Euclidean, Simplify as _};
use geo_types::Polygon;

use crate::crs::ProjectedPoint;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::sources::alkis::Building;

/// ISSprOM 521 building.
const BUILDING_SYMBOL_CODE: &str = "521";
/// `gebnutzbez` of real buildings; parts and structures (chimneys …) are not mapped.
const BUILDING_KIND: &str = "Gebäude";
/// Footprints and holes with an area below (`MIN_SIDE_MM`)² on paper are dropped.
const MIN_SIDE_MM: f64 = 0.5;
/// Outline simplification tolerance on paper.
const SIMPLIFY_MM: f64 = 0.1;

/// What a building becomes on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Object,
    Dropped(&'static str),
}

pub fn classify(b: &Building) -> Rule {
    if b.kind != BUILDING_KIND {
        Rule::Dropped("not a building")
    } else if b.underground {
        Rule::Dropped("underground")
    } else {
        Rule::Object
    }
}

/// 521 area objects for the buildings within `buffer_m` of `map_area` (both in EPSG:25833),
/// generalized for the map scale. Each footprint part becomes one object.
pub fn building_objects(
    buildings: &[Building],
    map_area: &Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let symbol_id = symbol_set
        .id_for_code(BUILDING_SYMBOL_CODE)
        .expect("symbol set defines 521");
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0;
    let min_area = (MIN_SIDE_MM * metres_per_mm).powi(2);
    let tolerance = SIMPLIFY_MM * metres_per_mm;

    let mut objects = Vec::new();
    for building in buildings.iter().filter(|b| classify(b) == Rule::Object) {
        for part in &building.footprint.0 {
            if Euclidean.distance(part, map_area) > buffer_m || part.unsigned_area() < min_area {
                continue;
            }
            let coords = footprint_coords(part, min_area, tolerance, georef)?;
            let mut tags = vec![
                ("freemapper:source".to_string(), "alkis".to_string()),
                ("freemapper:source_id".to_string(), building.oid.clone()),
            ];
            if let Some(updated) = building.updated {
                tags.push(("freemapper:currency".to_string(), updated.to_string()));
            }
            objects.push(MapObject {
                symbol_id,
                kind: MapObjectKind::Path,
                coords,
                tags,
            });
        }
    }
    Ok(objects)
}

/// The map coordinates of an area footprint: holes below `min_area` (m²) are dropped, the outline
/// is simplified by `tolerance` (m), each ring is closed. Shared with the 522 canopies.
pub(crate) fn footprint_coords(
    part: &Polygon<f64>,
    min_area: f64,
    tolerance: f64,
    georef: &Georeferencing,
) -> Result<Vec<MapCoord>, GeorefError> {
    let interiors = part
        .interiors()
        .iter()
        .filter(|hole| Polygon::new((*hole).clone(), vec![]).unsigned_area() >= min_area)
        .cloned()
        .collect();
    let part = Polygon::new(part.exterior().clone(), interiors).simplify(tolerance);
    let mut coords = Vec::new();
    for ring in std::iter::once(part.exterior()).chain(part.interiors()) {
        let points = ring
            .coords()
            .map(|c| {
                georef
                    .projected_to_map(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .map(|m| (m.x, m.y))
            })
            .collect::<Result<Vec<_>, _>>()?;
        coords.extend(closed_ring(&points));
    }
    Ok(coords)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::omap::MapObjectKind;
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
    use geo_types::{LineString, MultiPolygon, Polygon};

    struct Fixture {
        georef: Georeferencing,
        area: Polygon<f64>,
        symbols: SymbolSet,
    }

    /// 1:1000 at the school, map area 100 × 100 m around the reference point.
    fn fixture() -> Fixture {
        let date = "2026-11-07".parse().unwrap();
        let georef = Georeferencing::new(
            LatLon {
                lat: 52.5401,
                lon: 13.8442,
            },
            1000,
            date,
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        Fixture {
            area: rect(e - 50.0, n - 50.0, 100.0, 100.0),
            georef,
            symbols: SymbolSet::school_2019(),
        }
    }

    fn ring(e: f64, n: f64, w: f64, h: f64) -> LineString<f64> {
        LineString::from(vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)])
    }

    fn rect(e: f64, n: f64, w: f64, h: f64) -> Polygon<f64> {
        Polygon::new(ring(e, n, w, h), vec![])
    }

    fn building(kind: &str, footprint: Vec<Polygon<f64>>) -> Building {
        Building {
            oid: "DEBBAL0100000001".to_string(),
            kind: kind.to_string(),
            function: "Wohnhaus".to_string(),
            underground: false,
            updated: "2023-05-04".parse().ok(),
            footprint: MultiPolygon(footprint),
        }
    }

    fn objects(f: &Fixture, buildings: &[Building]) -> Vec<MapObject> {
        building_objects(buildings, &f.area, 20.0, &f.georef, &f.symbols).unwrap()
    }

    fn centre(f: &Fixture) -> (f64, f64) {
        (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        )
    }

    #[test]
    fn classify_rules() {
        let square = vec![rect(0.0, 0.0, 10.0, 10.0)];
        let mut garage = building("Gebäude", square.clone());
        garage.function = "Garage".to_string();
        assert_eq!(classify(&garage), Rule::Object);
        let mut cellar = building("Gebäude", square.clone());
        cellar.underground = true;
        assert_eq!(classify(&cellar), Rule::Dropped("underground"));
        assert_eq!(
            classify(&building("Bauteil", square.clone())),
            Rule::Dropped("not a building")
        );
        assert_eq!(
            classify(&building(
                "Bauwerk oder Anlage für Industrie und Gewerbe",
                square
            )),
            Rule::Dropped("not a building")
        );
    }

    #[test]
    fn building_inside_area_becomes_521() {
        let f = fixture();
        let (e, n) = centre(&f);
        let objects = objects(&f, &[building("Gebäude", vec![rect(e, n, 10.0, 20.0)])]);
        assert_eq!(objects.len(), 1);
        let o = &objects[0];
        assert_eq!(Some(o.symbol_id), f.symbols.id_for_code("521"));
        assert_eq!(o.kind, MapObjectKind::Path);
        assert_eq!(o.coords.len(), 5);
        assert_eq!(o.coords[4].flags, FLAG_HOLE | FLAG_CLOSE);
        let keys: Vec<(&str, &str)> = o
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            keys,
            [
                ("freemapper:source", "alkis"),
                ("freemapper:source_id", "DEBBAL0100000001"),
                ("freemapper:currency", "2023-05-04")
            ]
        );
    }

    #[test]
    fn buildings_outside_buffer_are_skipped() {
        let f = fixture();
        let (e, n) = centre(&f);
        let far = building("Gebäude", vec![rect(e + 75.0, n, 5.0, 5.0)]);
        let near = building("Gebäude", vec![rect(e + 65.0, n, 5.0, 5.0)]);
        assert_eq!(objects(&f, &[far]).len(), 0, "25 m outside");
        assert_eq!(objects(&f, &[near]).len(), 1, "15 m outside");
    }

    #[test]
    fn tiny_building_is_dropped() {
        let f = fixture();
        let (e, n) = centre(&f);
        assert_eq!(
            objects(&f, &[building("Gebäude", vec![rect(e, n, 0.4, 0.4)])]).len(),
            0
        );
        assert_eq!(
            objects(&f, &[building("Gebäude", vec![rect(e, n, 0.6, 0.6)])]).len(),
            1
        );
    }

    #[test]
    fn outline_is_simplified() {
        let f = fixture();
        let (e, n) = centre(&f);
        let notched = Polygon::new(
            LineString::from(vec![
                (e, n),
                (e + 5.0, n - 0.05),
                (e + 10.0, n),
                (e + 10.0, n + 10.0),
                (e, n + 10.0),
                (e, n),
            ]),
            vec![],
        );
        let objects = objects(&f, &[building("Gebäude", vec![notched])]);
        assert_eq!(objects[0].coords.len(), 5);
    }

    #[test]
    fn courtyard_building_keeps_its_hole() {
        let f = fixture();
        let (e, n) = centre(&f);
        let courtyard = Polygon::new(
            ring(e, n, 30.0, 30.0),
            vec![ring(e + 10.0, n + 10.0, 10.0, 10.0)],
        );
        let objects = objects(&f, &[building("Gebäude", vec![courtyard])]);
        assert_eq!(objects.len(), 1);
        let coords = &objects[0].coords;
        assert_eq!(coords.len(), 10);
        assert_eq!(coords[4].flags, FLAG_HOLE | FLAG_CLOSE);
        assert_eq!(coords[9].flags, FLAG_HOLE | FLAG_CLOSE);
    }

    #[test]
    fn tiny_holes_are_dropped() {
        let f = fixture();
        let (e, n) = centre(&f);
        let shaft = Polygon::new(
            ring(e, n, 30.0, 30.0),
            vec![ring(e + 10.0, n + 10.0, 0.3, 0.3)],
        );
        let objects = objects(&f, &[building("Gebäude", vec![shaft])]);
        assert_eq!(objects[0].coords.len(), 5);
    }

    #[test]
    fn multipolygon_parts_become_objects() {
        let f = fixture();
        let (e, n) = centre(&f);
        let parts = vec![rect(e, n, 5.0, 5.0), rect(e + 10.0, n, 5.0, 5.0)];
        let objects = objects(&f, &[building("Gebäude", parts)]);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].tags[1], objects[1].tags[1]);
    }

    #[test]
    fn dropped_buildings_give_no_objects() {
        let f = fixture();
        let (e, n) = centre(&f);
        let mut cellar = building("Gebäude", vec![rect(e, n, 10.0, 10.0)]);
        cellar.underground = true;
        assert!(objects(&f, &[cellar]).is_empty());
    }
}
