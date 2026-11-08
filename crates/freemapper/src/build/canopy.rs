//! ALKIS canopies (Berlin `Überdachung`, `Carport`) → 522 map objects (Berlin plus 1).
//!
//! They are cadastral survey data, as trustworthy as the 521 buildings, so they become map
//! objects on both map kinds (this amends the "candidate 522" of spec §4.5). The 522 fill
//! (Black 20 %) has a lower colour priority than the 521 fill (Black 50 %), but the black 522
//! outline would cross a building: so a canopy is cut at the 521 footprints, and a canopy that
//! lies inside them would only double the building and is dropped.

use chrono::NaiveDate;
use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Distance as _, Euclidean, Intersects as _,
};
use geo_types::{MultiPolygon, Polygon};
use serde_json::Value;

use crate::build::berlin::{feature_polygons, provenance_tags};
use crate::build::buildings::footprint_coords;
use crate::crs::Crs;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};

/// ISSprOM 522 canopy.
const CANOPY_SYMBOL_CODE: &str = "522";
/// The layer the canopies come from.
pub const CANOPY_LAYER: &str = "bauwerke-flaechen";
/// `bezbwf` of the canopies.
const CANOPY_KINDS: [&str; 2] = ["Überdachung", "Carport"];
/// Canopies and holes with an area below (`CANOPY_MIN_SIDE_MM`)² on paper are dropped, as in
/// `buildings.rs`.
const CANOPY_MIN_SIDE_MM: f64 = 0.5;
/// Outline simplification tolerance on paper, as in `buildings.rs`.
const SIMPLIFY_MM: f64 = 0.1;
/// A canopy of which at least this share lies in building footprints is inside a building.
const INSIDE_BUILDING_SHARE: f64 = 0.9;

/// 522 area objects for the canopies among the `bauwerke-flaechen` features within `buffer_m` of
/// `map_area`, generalized for the map scale. `buildings` are the 521 footprints and `map_area`
/// is in the map's CRS; `currency` is the fetch date of the layer.
pub fn canopy_objects(
    features: &[Value],
    buildings: &[Polygon<f64>],
    map_area: &Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    currency: NaiveDate,
) -> Result<Vec<MapObject>, GeorefError> {
    let symbol_id = symbol_set
        .id_for_code(CANOPY_SYMBOL_CODE)
        .expect("symbol set defines 522");
    let crs = Crs::from_epsg(georef.epsg).unwrap_or(Crs::Utm33);
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0;
    let min_area = (CANOPY_MIN_SIDE_MM * metres_per_mm).powi(2);
    let tolerance = SIMPLIFY_MM * metres_per_mm;

    let mut objects = Vec::new();
    for feature in features.iter().filter(|f| {
        f["properties"]["bezbwf"]
            .as_str()
            .is_some_and(|kind| CANOPY_KINDS.contains(&kind))
    }) {
        let tags = provenance_tags(CANOPY_LAYER, feature, currency);
        for canopy in feature_polygons(feature, crs) {
            if Euclidean.distance(&canopy, map_area) > buffer_m {
                continue;
            }
            // What is left beside the buildings must still reach the minimum size.
            for part in outside_buildings(canopy, buildings) {
                if part.unsigned_area() < min_area {
                    continue;
                }
                objects.push(MapObject {
                    symbol_id,
                    kind: MapObjectKind::Path,
                    coords: footprint_coords(&part, min_area, tolerance, georef)?,
                    tags: tags.clone(),
                });
            }
        }
    }
    Ok(objects)
}

/// The parts of `canopy` outside the `buildings`; none when at least [`INSIDE_BUILDING_SHARE`]
/// of it lies in them.
fn outside_buildings(canopy: Polygon<f64>, buildings: &[Polygon<f64>]) -> Vec<Polygon<f64>> {
    let Some(bounds) = canopy.bounding_rect() else {
        return Vec::new();
    };
    let near: Vec<Polygon<f64>> = buildings
        .iter()
        .filter(|b| b.bounding_rect().is_some_and(|r| r.intersects(&bounds)))
        .cloned()
        .collect();
    if near.is_empty() {
        return vec![canopy];
    }
    let near = MultiPolygon(near);
    let covered = canopy.intersection(&near).unsigned_area();
    if covered >= INSIDE_BUILDING_SHARE * canopy.unsigned_area() {
        return Vec::new();
    }
    if covered <= 0.0 {
        return vec![canopy];
    }
    canopy.difference(&near).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::ProjectedPoint;
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
    use crate::project::MapKind;

    /// Alexanderplatz, map area 200 × 200 m around the reference point, at the scale of `kind`.
    fn setup(kind: MapKind) -> (Georeferencing, Polygon<f64>, SymbolSet) {
        let centre = Crs::Utm33
            .inverse(ProjectedPoint {
                easting: 392_400.0,
                northing: 5_820_400.0,
            })
            .unwrap();
        let scale = match kind {
            MapKind::Schoolyard => 1000,
            _ => 4000,
        };
        let georef = Georeferencing::new(centre, scale, "2026-11-08".parse().unwrap()).unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        (
            georef,
            rect(e - 100.0, n - 100.0, 200.0, 200.0),
            kind.symbol_set(),
        )
    }

    fn ring(e: f64, n: f64, w: f64, h: f64) -> Vec<[f64; 2]> {
        vec![[e, n], [e + w, n], [e + w, n + h], [e, n + h], [e, n]]
    }

    fn rect(e: f64, n: f64, w: f64, h: f64) -> Polygon<f64> {
        let points: Vec<(f64, f64)> = ring(e, n, w, h).iter().map(|p| (p[0], p[1])).collect();
        Polygon::new(points.into(), vec![])
    }

    /// A `bauwerke-flaechen` feature with a MultiPolygon `w` × `h` m at (`e`, `n`) metres from
    /// the reference point.
    fn area(id: &str, kind: &str, (e, n): (f64, f64), (w, h): (f64, f64)) -> Value {
        let (e, n) = (392_400.0 + e, 5_820_400.0 + n);
        serde_json::json!({
            "type": "Feature", "id": format!("bauwerkeflaechen.{id}"),
            "geometry": {"type": "MultiPolygon", "coordinates": [[ring(e, n, w, h)]]},
            "properties": {"uuid": id, "bwf": "1610", "bezbwf": kind,
                           "bezdes": "Aus Katasterkarten digitalisiert"}
        })
    }

    fn objects(kind: MapKind, features: &[Value], buildings: &[Polygon<f64>]) -> Vec<MapObject> {
        let (georef, map_area, set) = setup(kind);
        canopy_objects(
            features,
            buildings,
            &map_area,
            20.0,
            &georef,
            &set,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn ueberdachung_and_carport_are_canopy_objects() {
        let mut carport = area("DEBE00YY0000CARP", "Carport", (10.0, 0.0), (3.0, 6.0));
        // A Polygon (not Multi) with a courtyard large enough to keep.
        let (e, n) = (392_400.0 + 30.0, 5_820_400.0);
        carport["geometry"] = serde_json::json!({"type": "Polygon", "coordinates": [
            ring(e, n, 20.0, 20.0), ring(e + 5.0, n + 5.0, 10.0, 10.0)]});
        let features = [
            area("DEBE00YY0000ROOF", "Überdachung", (0.0, 0.0), (5.0, 4.0)),
            carport,
            area("DEBE00YY0000STEP", "Treppe", (-20.0, 0.0), (5.0, 3.0)),
            area("DEBE00YY0000WALL", "Mauer", (-30.0, 0.0), (5.0, 3.0)),
            // 200 m outside the map area.
            area("DEBE00YY0000FAR", "Überdachung", (300.0, 0.0), (5.0, 4.0)),
        ];
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let set = kind.symbol_set();
            let found = objects(kind, &features, &[]);
            assert_eq!(found.len(), 2, "{kind:?}");
            assert!(
                found
                    .iter()
                    .all(|o| Some(o.symbol_id) == set.id_for_code("522")
                        && o.kind == MapObjectKind::Path)
            );
            let tags: Vec<(&str, &str)> = found[0]
                .tags
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            assert_eq!(
                tags,
                [
                    ("freemapper:source", "geoportal-berlin/bauwerke-flaechen"),
                    ("freemapper:source_id", "DEBE00YY0000ROOF"),
                    ("freemapper:currency", "2026-11-09"),
                    ("freemapper:alkis", "Überdachung"),
                ]
            );
            assert!(
                found[1]
                    .tags
                    .contains(&("freemapper:alkis".into(), "Carport".into()))
            );
            // The roof is a closed ring; the carport keeps its courtyard as a hole.
            assert_eq!(found[0].coords.len(), 5);
            assert_eq!(found[0].coords[4].flags, FLAG_HOLE | FLAG_CLOSE);
            assert_eq!(found[1].coords.len(), 10, "{kind:?}");
        }
    }

    #[test]
    fn tiny_canopy_is_dropped() {
        // 0.5 mm on paper: 2 m at 1:4000, 0.5 m at 1:1000.
        let small = [area(
            "DEBE00YY000SMALL",
            "Überdachung",
            (0.0, 0.0),
            (1.5, 1.5),
        )];
        let large = [area("DEBE00YY000LARGE", "Carport", (0.0, 0.0), (2.5, 2.5))];
        let tiny = [area(
            "DEBE00YY0000TINY",
            "Überdachung",
            (0.0, 0.0),
            (0.4, 0.4),
        )];
        assert!(objects(MapKind::Sprint, &small, &[]).is_empty());
        assert_eq!(objects(MapKind::Sprint, &large, &[]).len(), 1);
        assert_eq!(objects(MapKind::Schoolyard, &small, &[]).len(), 1);
        assert!(objects(MapKind::Schoolyard, &tiny, &[]).is_empty());
    }

    #[test]
    fn canopy_inside_a_building_is_not_doubled() {
        let (e, n) = (392_400.0, 5_820_400.0);
        let building = rect(e, n, 20.0, 10.0);
        let features = [
            // Under the roof of the building.
            area("DEBE00YY00INSIDE", "Überdachung", (2.0, 2.0), (5.0, 5.0)),
            // A canopy along the wall, half on the building: cut at the wall.
            area("DEBE00YY000HALF", "Überdachung", (5.0, 8.0), (5.0, 4.0)),
            // Touching the building only.
            area("DEBE00YY00ATTACH", "Carport", (20.0, 0.0), (3.0, 6.0)),
        ];
        let found = objects(MapKind::Schoolyard, &features, &[building]);
        let ids: Vec<&str> = found.iter().map(|o| o.tags[1].1.as_str()).collect();
        assert_eq!(ids, ["DEBE00YY000HALF", "DEBE00YY00ATTACH"]);
    }

    #[test]
    fn canopy_is_cut_at_the_building() {
        let (e, n) = (392_400.0, 5_820_400.0);
        let building = rect(e, n, 20.0, 10.0);
        // 5 × 4 m, the southern half on the building: 5 × 2 m remain north of the wall.
        let half = [area(
            "DEBE00YY000HALF",
            "Überdachung",
            (5.0, 8.0),
            (5.0, 4.0),
        )];
        let (georef, _, _) = setup(MapKind::Schoolyard);
        let found = objects(MapKind::Schoolyard, &half, std::slice::from_ref(&building));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].coords.len(), 5);
        for c in &found[0].coords {
            let p = georef.map_to_projected(*c);
            assert!(p.northing > n + 10.0 - 0.01, "{p:?} inside the building");
            assert!(p.northing < n + 12.0 + 0.01);
        }
        // What is left must still reach the minimum size: 1.5 × 2 m = 3 m² < 4 m² at 1:4000,
        // although the whole canopy (6 m²) would.
        let narrow = [area(
            "DEBE00YY0NARROW",
            "Überdachung",
            (5.0, 8.0),
            (1.5, 4.0),
        )];
        assert!(objects(MapKind::Sprint, &narrow, std::slice::from_ref(&building)).is_empty());
        assert_eq!(objects(MapKind::Sprint, &narrow, &[]).len(), 1);
    }
}
