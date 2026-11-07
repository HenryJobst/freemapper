//! Private land (ALKIS land use) → 520 out-of-bounds candidates (spec §4.5).

use geo::{Area as _, BooleanOps as _};
use geo_types::Polygon;

use crate::crs::{BoundingBox, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::sources::alkis::LandUse;

/// ISSprOM 520 area that shall not be entered.
const OUT_OF_BOUNDS_CODE: &str = "520";
/// Land uses that are private ground: residents, businesses, the railway.
const PRIVATE_LAND_USES: [&str; 4] = [
    "Wohnbaufläche",
    "Industrie- und Gewerbefläche",
    "Fläche gemischter Nutzung",
    "Bahnverkehr",
];
/// On the schoolyard map, parts at least this much inside the map area are the school's land.
const OWN_LAND_SHARE: f64 = 0.5;
/// Parts smaller than (1 mm)² on paper are slivers along parcel boundaries.
const MIN_SIDE_MM: f64 = 1.0;

pub fn is_private(land: &LandUse) -> bool {
    PRIVATE_LAND_USES.contains(&land.nutzart.as_str())
}

/// 520 candidate objects for the private land uses, clipped to `clip` (map area plus buffer, EPSG:25833).
pub fn out_of_bounds_objects(
    land_use: &[LandUse],
    map_area: &Polygon<f64>,
    clip: &BoundingBox,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let symbol_id = symbol_set
        .id_for_code(OUT_OF_BOUNDS_CODE)
        .expect("symbol set defines 520");
    let clip_polygon = geo_types::Rect::new(
        geo_types::coord! { x: clip.min_e, y: clip.min_n },
        geo_types::coord! { x: clip.max_e, y: clip.max_n },
    )
    .to_polygon();
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0;
    let min_area = (MIN_SIDE_MM * metres_per_mm).powi(2);
    let mut objects = Vec::new();
    for land in land_use.iter().filter(|l| is_private(l)) {
        for part in &land.footprint.0 {
            let part_area = part.unsigned_area();
            if part_area <= 0.0 {
                continue;
            }
            if kind == MapKind::Schoolyard
                && part.intersection(map_area).unsigned_area() / part_area >= OWN_LAND_SHARE
            {
                continue;
            }
            for piece in part.intersection(&clip_polygon).0 {
                if piece.unsigned_area() < min_area {
                    continue;
                }
                let mut coords = Vec::new();
                for ring in std::iter::once(piece.exterior()).chain(piece.interiors()) {
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
                let nutzart = match &land.bez {
                    Some(bez) => format!("{} / {bez}", land.nutzart),
                    None => land.nutzart.clone(),
                };
                let mut tags = vec![
                    ("freemapper:source".to_string(), "alkis-nutzung".to_string()),
                    ("freemapper:source_id".to_string(), land.oid.clone()),
                    ("freemapper:nutzart".to_string(), nutzart),
                ];
                if let Some(updated) = land.updated {
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
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
    use geo_types::{LineString, MultiPolygon, Polygon};

    struct Fixture {
        georef: Georeferencing,
        area: Polygon<f64>,
        clip: BoundingBox,
        centre: (f64, f64),
    }

    /// 1:1000 at the school; map area 100 × 100 m around the reference point, buffer 20 m.
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
            area: Polygon::new(ring(e - 50.0, n - 50.0, 100.0, 100.0), vec![]),
            clip: BoundingBox {
                min_e: e - 70.0,
                min_n: n - 70.0,
                max_e: e + 70.0,
                max_n: n + 70.0,
            },
            georef,
            centre: (e, n),
        }
    }

    fn ring(e: f64, n: f64, w: f64, h: f64) -> LineString<f64> {
        LineString::from(vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)])
    }

    fn land(nutzart: &str, parts: Vec<Polygon<f64>>) -> LandUse {
        LandUse {
            oid: "DEBBAL0100000011".to_string(),
            nutzart: nutzart.to_string(),
            bez: None,
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(parts),
        }
    }

    fn objects(f: &Fixture, land_use: &[LandUse], kind: MapKind) -> (Vec<MapObject>, SymbolSet) {
        let set = kind.symbol_set();
        let objects =
            out_of_bounds_objects(land_use, &f.area, &f.clip, kind, &f.georef, &set).unwrap();
        (objects, set)
    }

    #[test]
    fn private_land_uses() {
        for nutzart in [
            "Wohnbaufläche",
            "Industrie- und Gewerbefläche",
            "Fläche gemischter Nutzung",
            "Bahnverkehr",
        ] {
            assert!(is_private(&land(nutzart, vec![])), "{nutzart}");
        }
        for nutzart in [
            "Wald",
            "Straßenverkehr",
            "Fläche besonderer funktionaler Prägung",
            "Sport-, Freizeit- und Erholungsfläche",
        ] {
            assert!(!is_private(&land(nutzart, vec![])), "{nutzart}");
        }
    }

    #[test]
    fn school_parcel_is_left_out_on_schoolyard() {
        let f = fixture();
        let (e, n) = f.centre;
        // Mostly inside the map area: the school's own land.
        let inside = land(
            "Wohnbaufläche",
            vec![Polygon::new(ring(e - 40.0, n - 40.0, 60.0, 60.0), vec![])],
        );
        assert!(
            objects(&f, std::slice::from_ref(&inside), MapKind::Schoolyard)
                .0
                .is_empty()
        );
        // A neighbour reaching 10 m into the map area.
        let neighbour = land(
            "Wohnbaufläche",
            vec![Polygon::new(ring(e + 40.0, n - 20.0, 40.0, 40.0), vec![])],
        );
        assert_eq!(objects(&f, &[neighbour], MapKind::Schoolyard).0.len(), 1);
    }

    #[test]
    fn sprint_keeps_parcels_inside_the_area() {
        let f = fixture();
        let (e, n) = f.centre;
        let inside = land(
            "Wohnbaufläche",
            vec![Polygon::new(ring(e - 40.0, n - 40.0, 60.0, 60.0), vec![])],
        );
        assert_eq!(objects(&f, &[inside], MapKind::Sprint).0.len(), 1);
    }

    #[test]
    fn large_parcel_is_clipped() {
        let f = fixture();
        let (e, n) = f.centre;
        let huge = land(
            "Bahnverkehr",
            vec![Polygon::new(
                ring(e + 30.0, n - 500.0, 500.0, 1000.0),
                vec![],
            )],
        );
        let (objects, _) = objects(&f, &[huge], MapKind::Schoolyard);
        assert_eq!(objects.len(), 1);
        let back: Vec<_> = objects[0]
            .coords
            .iter()
            .map(|c| f.georef.map_to_projected(*c))
            .collect();
        for p in back {
            assert!(
                p.easting <= f.clip.max_e + 0.01 && p.northing >= f.clip.min_n - 0.01,
                "{p:?}"
            );
        }
    }

    #[test]
    fn slivers_are_dropped() {
        let f = fixture();
        let (e, n) = f.centre;
        let sliver = land(
            "Wohnbaufläche",
            vec![Polygon::new(ring(e + 55.0, n, 0.5, 0.5), vec![])],
        );
        assert!(objects(&f, &[sliver], MapKind::Schoolyard).0.is_empty());
    }

    #[test]
    fn holes_are_kept() {
        let f = fixture();
        let (e, n) = f.centre;
        let yard = Polygon::new(
            ring(e + 52.0, n - 15.0, 16.0, 30.0),
            vec![ring(e + 56.0, n - 5.0, 8.0, 10.0)],
        );
        let (objects, set) = objects(
            &f,
            &[land("Industrie- und Gewerbefläche", vec![yard])],
            MapKind::Schoolyard,
        );
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("520"));
        assert_eq!(objects[0].coords.len(), 10);
        assert_eq!(objects[0].coords[4].flags, FLAG_HOLE | FLAG_CLOSE);
        assert_eq!(objects[0].coords[9].flags, FLAG_HOLE | FLAG_CLOSE);
    }

    #[test]
    fn tags_carry_provenance() {
        let f = fixture();
        let (e, n) = f.centre;
        let mut homes = land(
            "Wohnbaufläche",
            vec![Polygon::new(ring(e + 52.0, n, 10.0, 10.0), vec![])],
        );
        homes.bez = Some("Mehrfamilienhaus".to_string());
        let (objects, _) = objects(&f, &[homes], MapKind::Schoolyard);
        let tags: Vec<(&str, &str)> = objects[0]
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            tags,
            [
                ("freemapper:source", "alkis-nutzung"),
                ("freemapper:source_id", "DEBBAL0100000011"),
                ("freemapper:nutzart", "Wohnbaufläche / Mehrfamilienhaus"),
                ("freemapper:currency", "2025-09-30")
            ]
        );
    }
}
