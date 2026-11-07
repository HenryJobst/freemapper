//! ALKIS traffic areas → 501.5 paved areas of the sprint map (spec §4.5).

use geo::Intersects as _;
use geo_types::LineString;

use crate::build::out_of_bounds::{LandClip, clipped_part_objects};
use crate::crs::{BoundingBox, utm33_forward};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, SymbolSet};
use crate::sources::alkis::LandUse;
use crate::sources::osm::{OsmFeature, OsmGeometry};

/// ISSprOM 501.5 paved area, light traffic, without border: a parcel boundary is no visible edge,
/// and ALKIS does not tell the traffic volume.
const PAVED_AREA_CODE: &str = "501.5";
/// Green strips and tree rows along the carriageway; the orthophoto shows them.
const VERGE: &str = "Begleitfläche Straßenverkehr";
/// `Platz` refinements that are paved; others (e.g. `Festplatz`, often grass) are candidates.
const PAVED_SQUARES: [&str; 3] = ["Parkplatz", "Marktplatz", "Fußgängerzone"];
/// OSM streets and unpaved surfaces: an unpaved street makes its ALKIS parcel a candidate. The
/// rule file keeps these streets in the stored OSM features (test `unpaved_streets_are_stored`).
const STREETS: [&str; 8] = [
    "residential",
    "unclassified",
    "service",
    "living_street",
    "tertiary",
    "secondary",
    "primary",
    "road",
];
const UNPAVED: [&str; 10] = [
    "unpaved",
    "dirt",
    "earth",
    "ground",
    "grass",
    "gravel",
    "fine_gravel",
    "compacted",
    "sand",
    "woodchips",
];

/// The unpaved OSM streets among `features`, in EPSG:25833.
pub fn unpaved_streets(features: &[OsmFeature]) -> Vec<LineString<f64>> {
    let tag = |f: &OsmFeature, key: &str| {
        f.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    features
        .iter()
        .filter(|f| {
            tag(f, "highway").is_some_and(|h| STREETS.contains(&h.as_str()))
                && tag(f, "surface").is_some_and(|s| UNPAVED.contains(&s.as_str()))
        })
        .filter_map(|f| match &f.geometry {
            OsmGeometry::Line(points) => Some(LineString::from(
                points
                    .iter()
                    .map(|p| {
                        let q = utm33_forward(*p);
                        (q.easting, q.northing)
                    })
                    .collect::<Vec<_>>(),
            )),
            OsmGeometry::Point(_) => None,
        })
        .collect()
}

/// Paved areas: map objects where ALKIS is reliable, candidates where it is not.
#[derive(Debug, Default)]
pub struct PavedObjects {
    /// Carriageways with sidewalks and parking lanes (`Straßenverkehr`), squares and car parks (`Platz`).
    pub map: Vec<MapObject>,
    /// `Weg`: sidewalks and paved lanes in town, but unpaved tracks in the woods.
    pub candidates: Vec<MapObject>,
}

/// 501.5 objects for the traffic areas of `land_use`, clipped to `clip` (map area plus buffer,
/// EPSG:25833). A street parcel crossed by one of `unpaved_streets` (EPSG:25833) is a candidate.
pub fn paved_area_objects(
    land_use: &[LandUse],
    unpaved_streets: &[LineString<f64>],
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<PavedObjects, GeorefError> {
    let symbol_id = symbol_set
        .id_for_code(PAVED_AREA_CODE)
        .expect("ISSprOM defines 501.5");
    let clip = LandClip::new(clip, georef);
    let mut objects = PavedObjects::default();
    for land in land_use {
        let reliable = match (land.nutzart.as_str(), land.bez.as_deref()) {
            ("Straßenverkehr", Some(VERGE)) => continue,
            ("Straßenverkehr", _) => true,
            ("Platz", None) => true,
            ("Platz", Some(bez)) => PAVED_SQUARES.contains(&bez),
            ("Weg", _) => false,
            _ => continue,
        };
        for part in &land.footprint.0 {
            let unpaved = unpaved_streets.iter().any(|street| street.intersects(part));
            let target = if reliable && !unpaved {
                &mut objects.map
            } else {
                &mut objects.candidates
            };
            target.extend(clipped_part_objects(land, part, &clip, symbol_id, georef)?);
        }
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::project::MapKind;
    use geo_types::{LineString, MultiPolygon, Polygon};

    struct Fixture {
        georef: Georeferencing,
        clip: BoundingBox,
        centre: (f64, f64),
    }

    /// 1:4000 in Strausberg Vorstadt; clip 300 × 300 m around the reference point.
    fn fixture() -> Fixture {
        let date = "2026-11-07".parse().unwrap();
        let georef = Georeferencing::new(
            LatLon {
                lat: 52.5385,
                lon: 13.844,
            },
            4000,
            date,
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        Fixture {
            clip: BoundingBox {
                min_e: e - 150.0,
                min_n: n - 150.0,
                max_e: e + 150.0,
                max_n: n + 150.0,
            },
            georef,
            centre: (e, n),
        }
    }

    fn ring(e: f64, n: f64, w: f64, h: f64) -> LineString<f64> {
        LineString::from(vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)])
    }

    fn land(nutzart: &str, bez: Option<&str>, f: &Fixture) -> LandUse {
        let (e, n) = f.centre;
        LandUse {
            oid: "DEBBAL0100000022".to_string(),
            nutzart: nutzart.to_string(),
            bez: bez.map(str::to_string),
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(vec![Polygon::new(
                ring(e - 50.0, n - 6.0, 100.0, 12.0),
                vec![],
            )]),
        }
    }

    fn paved(f: &Fixture, land_use: &[LandUse]) -> (PavedObjects, SymbolSet) {
        let set = MapKind::Sprint.symbol_set();
        let objects = paved_area_objects(land_use, &[], &f.clip, &f.georef, &set).unwrap();
        (objects, set)
    }

    #[test]
    fn traffic_areas_are_paved() {
        let f = fixture();
        let land_use = [
            land("Straßenverkehr", None, &f),
            land("Platz", Some("Parkplatz"), &f),
            land("Platz", None, &f),
        ];
        let (objects, set) = paved(&f, &land_use);
        assert_eq!(objects.map.len(), 3);
        assert!(objects.candidates.is_empty());
        let symbol = set.id_for_code("501.5").unwrap();
        assert!(objects.map.iter().all(|o| o.symbol_id == symbol));
    }

    #[test]
    fn verges_are_left_out() {
        let f = fixture();
        let land_use = [
            land("Straßenverkehr", Some("Begleitfläche Straßenverkehr"), &f),
            land("Wohnbaufläche", None, &f),
            land("Bahnverkehr", None, &f),
        ];
        let (objects, _) = paved(&f, &land_use);
        assert!(objects.map.is_empty() && objects.candidates.is_empty());
    }

    #[test]
    fn ways_are_candidates() {
        let f = fixture();
        let (objects, set) = paved(&f, &[land("Weg", None, &f)]);
        assert!(objects.map.is_empty());
        assert_eq!(objects.candidates.len(), 1);
        assert_eq!(
            objects.candidates[0].symbol_id,
            set.id_for_code("501.5").unwrap()
        );
    }

    #[test]
    fn paved_area_is_clipped() {
        let f = fixture();
        let (e, n) = f.centre;
        let mut street = land("Straßenverkehr", None, &f);
        street.footprint = MultiPolygon(vec![Polygon::new(
            ring(e - 1000.0, n - 8.0, 2000.0, 16.0),
            vec![],
        )]);
        // A 1 m strip at 1:4000 is a sliver (0.25 mm on paper).
        let mut sliver = land("Platz", None, &f);
        sliver.footprint = MultiPolygon(vec![Polygon::new(ring(e, n + 40.0, 50.0, 1.0), vec![])]);
        let (objects, _) = paved(&f, &[street, sliver]);
        assert_eq!(objects.map.len(), 1);
        for c in &objects.map[0].coords {
            let p = f.georef.map_to_projected(*c);
            assert!(
                p.easting <= f.clip.max_e + 0.01
                    && p.easting >= f.clip.min_e - 0.01
                    && p.northing <= f.clip.max_n + 0.01
                    && p.northing >= f.clip.min_n - 0.01,
                "{p:?}"
            );
        }
    }

    #[test]
    fn sliver_at_the_clip_edge_is_dropped() {
        let f = fixture();
        let (e, n) = f.centre;
        // 100 m long, but only 0.5 m of it inside the clip window.
        let mut car_park = land("Platz", None, &f);
        car_park.footprint = MultiPolygon(vec![Polygon::new(
            ring(e + 149.5, n - 20.0, 100.0, 40.0),
            vec![],
        )]);
        let (objects, _) = paved(&f, &[car_park]);
        assert!(objects.map.is_empty());
    }

    #[test]
    fn unpaved_street_makes_its_parcel_a_candidate() {
        let f = fixture();
        let (e, n) = f.centre;
        let sand_road = LineString::from(vec![(e - 80.0, n), (e + 80.0, n)]);
        let set = MapKind::Sprint.symbol_set();
        let land_use = [land("Straßenverkehr", None, &f)];
        let objects =
            paved_area_objects(&land_use, &[sand_road], &f.clip, &f.georef, &set).unwrap();
        assert!(objects.map.is_empty());
        assert_eq!(objects.candidates.len(), 1);
    }

    #[test]
    fn unpaved_streets_are_stored() {
        use crate::build::candidates::Rules;
        for street in STREETS {
            for surface in UNPAVED {
                let tags = vec![
                    ("highway".to_string(), street.to_string()),
                    ("surface".to_string(), surface.to_string()),
                ];
                assert!(
                    Rules::embedded().wants(&tags, MapKind::Sprint),
                    "{street} {surface}"
                );
            }
        }
    }

    #[test]
    fn unpaved_streets_are_picked_and_projected() {
        let point = |lat, lon| LatLon { lat, lon };
        let way = |id: &str, highway: &str, surface: &str| OsmFeature {
            id: id.to_string(),
            tags: vec![
                ("highway".to_string(), highway.to_string()),
                ("surface".to_string(), surface.to_string()),
            ],
            geometry: OsmGeometry::Line(vec![point(52.538, 13.844), point(52.539, 13.845)]),
        };
        let streets = unpaved_streets(&[
            way("way/1", "residential", "sand"),
            way("way/2", "residential", "asphalt"),
            way("way/3", "path", "sand"),
        ]);
        assert_eq!(streets.len(), 1);
        let start = utm33_forward(point(52.538, 13.844));
        assert_eq!(streets[0].0[0].x, start.easting);
    }

    #[test]
    fn only_known_squares_are_paved() {
        let f = fixture();
        for (bez, map) in [
            (None, true),
            (Some("Parkplatz"), true),
            (Some("Marktplatz"), true),
            (Some("Fußgängerzone"), true),
            (Some("Festplatz"), false),
        ] {
            let (objects, _) = paved(&f, &[land("Platz", bez, &f)]);
            assert_eq!(
                (objects.map.len(), objects.candidates.len()),
                if map { (1, 0) } else { (0, 1) },
                "{bez:?}"
            );
        }
    }

    #[test]
    fn paved_tags_carry_provenance() {
        let f = fixture();
        let (objects, _) = paved(&f, &[land("Platz", Some("Parkplatz"), &f)]);
        let tags = &objects.map[0].tags;
        for (k, v) in [
            ("freemapper:source", "alkis-nutzung"),
            ("freemapper:source_id", "DEBBAL0100000022"),
            ("freemapper:nutzart", "Platz / Parkplatz"),
            ("freemapper:currency", "2025-09-30"),
        ] {
            assert!(
                tags.iter().any(|(tk, tv)| tk == k && tv == v),
                "{k}={v} in {tags:?}"
            );
        }
    }
}
