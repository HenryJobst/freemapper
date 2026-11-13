//! Parks, meadows and woods (ALKIS land use) → 401, 403 and 405 areas (ISSprOM 2019-2 §4.4,
//! ADR 0069), and the walkable area (belaufbare Fläche) under the whole sprint map.
//!
//! A park is yellow open land (401); tree groups (405, 402) and single trees lie on it, paths,
//! paving, buildings and water above it by colour priority. Only reliable land uses are map
//! objects; OSM parks and lawns are candidates where no ALKIS area of this kind lies.

use geo::{Area as _, BooleanOps as _, BoundingRect as _, Intersects as _};
use geo_types::{LineString, MultiPolygon, Polygon};

use crate::build::geometry::object_polygon;
use crate::build::out_of_bounds::{
    LandClip, area_object, clipped_pieces, is_out_of_bounds_part, land_tags, short_side,
};
use crate::build::peripheral::PaperRect;
use crate::crs::{BoundingBox, Crs, LatLon};
use crate::georef::{GeorefError, Georeferencing};
use crate::measure::{SymbolMm, SymbolScale};
use crate::omap::{MapCoord, MapObject, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::sources::alkis::LandUse;
use crate::sources::osm::{OsmFeature, OsmGeometry};
use freemapper_sources::alkis_source::spelling_key;

/// ISSprOM 401 open land.
pub const OPEN_LAND_CODE: &str = "401";
/// ISSprOM 403 rough open land.
const ROUGH_OPEN_LAND_CODE: &str = "403";
/// ISSprOM 405 forest.
const FOREST_CODE: &str = "405";
/// The tag of the walkable area, the base fill (ADR 0069).
pub const BASE_SOURCE: &str = freemapper_contract::tags::SOURCE_BASE;
/// The map part of the walkable area, written first.
pub const BASE_PART: &str = freemapper_contract::tags::BASE_PART;

/// The class of a land use on the map: its symbol and whether it is reliable enough to be a map
/// object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenLandClass {
    pub code: &'static str,
    pub map: bool,
}

/// `nutzart / bez` → symbol. A park, a green space and a meadow are lawn with trees: 401 (§4.4
/// "lawn, field, meadow, grassland"). A cemetery and other farmland are candidates: graves,
/// crops and gardens need the mapper. Woods are white forest, heath rough open land; moor and
/// swamp are candidates for 403 (the mapper adds the marsh).
pub fn open_land_class(land: &LandUse) -> Option<OpenLandClass> {
    let nutzart = spelling_key(&land.nutzart);
    let bez = land.bez.as_deref().map(spelling_key);
    let is = |name: &str| nutzart == spelling_key(name);
    let bez_is = |names: &[&str]| {
        bez.as_ref()
            .is_some_and(|b| names.iter().any(|n| *b == spelling_key(n)))
    };
    let class = |code, map| Some(OpenLandClass { code, map });
    if is("Sport-, Freizeit- und Erholungsfläche") {
        return if bez_is(&["Grünanlage", "Park", "Siedlungsgrünfläche"]) {
            class(OPEN_LAND_CODE, true)
        } else {
            None
        };
    }
    if is("Landwirtschaft") {
        return class(OPEN_LAND_CODE, bez_is(&["Grünland"]));
    }
    if is("Friedhof") {
        return class(OPEN_LAND_CODE, false);
    }
    if is("Wald") || is("Gehölz") {
        return class(FOREST_CODE, true);
    }
    if is("Heide") {
        return class(ROUGH_OPEN_LAND_CODE, true);
    }
    if is("Moor") || is("Sumpf") {
        return class(ROUGH_OPEN_LAND_CODE, false);
    }
    None
}

/// The open land of [`open_land_objects`].
#[derive(Debug, Default)]
pub struct OpenLandObjects {
    pub map: Vec<MapObject>,
    pub candidates: Vec<MapObject>,
}

/// 401, 403 and 405 areas for the land uses of [`open_land_class`], clipped to `clip` (map area
/// plus buffer, the map's CRS), slivers dropped, tagged with the land use's provenance. On the
/// schoolyard map the land off the school grounds is out of bounds (a 520 candidate) and gets
/// none. On a forest map the forest is the white paper: woods get no object, and no open land
/// is drawn where an ALKIS wood or one of `osm_forest` (the map's CRS) lies (spec 2026-11-13).
pub fn open_land_objects(
    land_use: &[LandUse],
    map_area: &Polygon<f64>,
    clip: &BoundingBox,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    osm_forest: &[Polygon<f64>],
) -> Result<OpenLandObjects, GeorefError> {
    let forest = (kind == MapKind::Forest).then(|| forest_cover(land_use, osm_forest, clip));
    let clip = LandClip::new(
        clip,
        SymbolScale::drawn_with(symbol_set, georef.scale_denominator),
    );
    let mut objects = OpenLandObjects::default();
    for land in land_use {
        let Some(class) = open_land_class(land) else {
            continue;
        };
        if forest.is_some() && class.code == FOREST_CODE {
            continue;
        }
        let Some(symbol_id) = symbol_set.id_for_code(class.code) else {
            continue;
        };
        for part in &land.footprint.0 {
            if part.unsigned_area() <= 0.0 || is_out_of_bounds_part(land, part, map_area, kind) {
                continue;
            }
            let target = if class.map {
                &mut objects.map
            } else {
                &mut objects.candidates
            };
            let open = match &forest {
                Some(forest) if !forest.0.is_empty() => {
                    MultiPolygon(vec![part.clone()]).difference(forest).0
                }
                _ => vec![part.clone()],
            };
            for open in &open {
                for piece in clipped_pieces(open, &clip, f64::INFINITY) {
                    target.push(area_object(&piece, symbol_id, land_tags(land), georef)?);
                }
            }
        }
    }
    Ok(objects)
}

/// The forest over the clip window: the ALKIS woods ([`open_land_class`] 405) and `osm_forest`.
fn forest_cover(
    land_use: &[LandUse],
    osm_forest: &[Polygon<f64>],
    clip: &BoundingBox,
) -> MultiPolygon<f64> {
    let window = geo_types::Rect::new(
        geo_types::coord! { x: clip.min_e, y: clip.min_n },
        geo_types::coord! { x: clip.max_e, y: clip.max_n },
    );
    let woods = land_use
        .iter()
        .filter(|land| open_land_class(land).is_some_and(|c| c.code == FOREST_CODE))
        .flat_map(|land| land.footprint.0.iter());
    let near: Vec<&Polygon<f64>> = woods
        .chain(osm_forest)
        .filter(|p| p.bounding_rect().is_some_and(|r| r.intersects(&window)))
        .collect();
    geo::unary_union(near)
}

/// The OSM forest (`landuse=forest`, `natural=wood`) of `features` as polygons in `crs`: closed
/// ways and multipolygons. A forest map keeps these features for it (rules with `forest = "-"`).
pub fn osm_forest_polygons(features: &[OsmFeature], crs: Crs) -> Vec<Polygon<f64>> {
    let is_forest = |f: &OsmFeature| {
        f.tags
            .iter()
            .any(|(k, v)| (k == "landuse" && v == "forest") || (k == "natural" && v == "wood"))
    };
    let ring = |points: &[LatLon]| {
        LineString::from(
            points
                .iter()
                .map(|p| {
                    let q = crs.forward(*p);
                    (q.easting, q.northing)
                })
                .collect::<Vec<_>>(),
        )
    };
    features
        .iter()
        .filter(|f| is_forest(f))
        .flat_map(|f| match &f.geometry {
            OsmGeometry::Line(points) if points.len() > 3 && points.first() == points.last() => {
                vec![Polygon::new(ring(points), vec![])]
            }
            OsmGeometry::Area(polygons) => polygons
                .iter()
                .map(|p| Polygon::new(ring(&p.outer), p.holes.iter().map(|h| ring(h)).collect()))
                .collect(),
            _ => vec![],
        })
        .collect()
}

/// A piece of open land below (`MIN_PIECE_SIDE_MM`)² or with a short side (`short_side`) under
/// it, enlarged like the symbols, is a sliver.
const MIN_PIECE_SIDE_MM: SymbolMm = SymbolMm(1.0);

/// The OSM open land candidates (401 areas among `candidates`) minus the open land map objects
/// (`alkis`, any 401, 403 or 405): where ALKIS knows the park, OSM adds nothing. A piece below
/// (1 mm)² or with a short side under 1 mm on paper goes; the rest keeps its tags. Other objects
/// stay as they are.
pub fn drop_open_land_under_alkis(
    candidates: Vec<MapObject>,
    alkis: &[MapObject],
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    let Some(open_id) = symbol_set.id_for_code(OPEN_LAND_CODE) else {
        return candidates;
    };
    let alkis: Vec<Polygon<f64>> = alkis.iter().filter_map(object_polygon).collect();
    if alkis.is_empty() {
        return candidates;
    }
    let min_side = MIN_PIECE_SIDE_MM.map_units(SymbolScale::drawn_with(
        symbol_set,
        georef.scale_denominator,
    ));
    let min_area = min_side * min_side;
    let mut kept = Vec::with_capacity(candidates.len());
    for object in candidates {
        let polygon = (object.symbol_id == open_id)
            .then(|| object_polygon(&object))
            .flatten();
        let Some((polygon, bounds)) = polygon.and_then(|p| p.bounding_rect().map(|b| (p, b)))
        else {
            kept.push(object);
            continue;
        };
        let near: Vec<&Polygon<f64>> = alkis
            .iter()
            .filter(|a| a.bounding_rect().is_some_and(|r| r.intersects(&bounds)))
            .collect();
        if near.is_empty() {
            kept.push(object);
            continue;
        }
        let rest = MultiPolygon(vec![polygon]).difference(&geo::unary_union(near));
        for piece in rest.0 {
            if piece.unsigned_area() < min_area || short_side(&piece) < min_side {
                continue;
            }
            kept.push(MapObject {
                coords: map_polygon_coords(&piece),
                ..object.clone()
            });
        }
    }
    kept
}

/// The coordinates of `polygon` (map coordinates) as an OOM area, holes kept.
fn map_polygon_coords(polygon: &Polygon<f64>) -> Vec<MapCoord> {
    std::iter::once(polygon.exterior())
        .chain(polygon.interiors())
        .flat_map(|ring| {
            let points: Vec<(i32, i32)> = ring
                .coords()
                .map(|c| (c.x.round() as i32, c.y.round() as i32))
                .collect();
            closed_ring(&points)
        })
        .collect()
}

/// The walkable area (belaufbare Fläche) of the sprint map: one 990 area over the paper
/// rectangle `content` that encloses the map area plus buffer (the window the training map shows,
/// so that no white wedge stays where the grivation turns the buffer's box), tagged
/// `freemapper:source=base`. None on other map kinds or without the symbol.
pub fn walkable_base(
    content: &PaperRect,
    kind: MapKind,
    symbol_set: &SymbolSet,
) -> Option<MapObject> {
    let symbol_id = symbol_set
        .id_for_code(crate::omap::symbol_set::WALKABLE_AREA_CODE)
        .filter(|_| kind == MapKind::Sprint)?;
    let PaperRect {
        min_x,
        min_y,
        max_x,
        max_y,
    } = *content;
    Some(MapObject {
        symbol_id,
        kind: crate::omap::MapObjectKind::Path,
        coords: closed_ring(&[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ]),
        tags: vec![(
            freemapper_contract::tags::SOURCE_KEY.to_string(),
            BASE_SOURCE.to_string(),
        )],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        georef: Georeferencing,
        area: Polygon<f64>,
        clip: BoundingBox,
        centre: (f64, f64),
    }

    /// 1:4000 at the Viktoriapark; map area 400 × 400 m, buffer 50 m.
    fn fixture() -> Fixture {
        let georef = Georeferencing::new(
            LatLon {
                lat: 52.4876,
                lon: 13.3801,
            },
            4000,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        Fixture {
            area: Polygon::new(ring(e - 200.0, n - 200.0, 400.0, 400.0), vec![]),
            clip: BoundingBox {
                min_e: e - 250.0,
                min_n: n - 250.0,
                max_e: e + 250.0,
                max_n: n + 250.0,
            },
            georef,
            centre: (e, n),
        }
    }

    fn ring(e: f64, n: f64, w: f64, h: f64) -> LineString<f64> {
        LineString::from(vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)])
    }

    fn land(nutzart: &str, bez: Option<&str>, parts: Vec<Polygon<f64>>) -> LandUse {
        LandUse {
            oid: "DEBE00YY0000001".to_string(),
            nutzart: nutzart.to_string(),
            bez: bez.map(str::to_string),
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(parts),
        }
    }

    fn square(f: &Fixture, de: f64, dn: f64, side: f64) -> Polygon<f64> {
        let (e, n) = f.centre;
        Polygon::new(ring(e + de, n + dn, side, side), vec![])
    }

    #[test]
    fn parks_and_meadows_are_open_land_map_objects() {
        let class = |nutzart: &str, bez: Option<&str>| open_land_class(&land(nutzart, bez, vec![]));
        let map = |code| Some(OpenLandClass { code, map: true });
        let candidate = |code| Some(OpenLandClass { code, map: false });
        let recreation = "Sport-, Freizeit- und Erholungsfläche";
        for bez in ["Grünanlage", "Park", "Siedlungsgrünfläche", "Gruenanlage"] {
            assert_eq!(class(recreation, Some(bez)), map("401"), "{bez}");
        }
        // Hamburg spells without umlauts.
        assert_eq!(
            class("Sport Freizeit Und Erholungsflaeche", Some("Gruenanlage")),
            map("401")
        );
        for bez in [
            None,
            Some("Sportanlage"),
            Some("Spielplatz, Bolzplatz"),
            Some("Kleingarten"),
        ] {
            assert_eq!(class(recreation, bez), None, "{bez:?}");
        }
        assert_eq!(class("Landwirtschaft", Some("Grünland")), map("401"));
        assert_eq!(class("Landwirtschaft", Some("Ackerland")), candidate("401"));
        assert_eq!(class("Landwirtschaft", None), candidate("401"));
        assert_eq!(class("Friedhof", None), candidate("401"));
        assert_eq!(class("Wald", None), map("405"));
        assert_eq!(class("Gehölz", None), map("405"));
        assert_eq!(class("Heide", None), map("403"));
        assert_eq!(class("Moor", None), candidate("403"));
        for nutzart in [
            "Wohnbaufläche",
            "Straßenverkehr",
            "Weg",
            "Platz",
            "Fließgewässer",
        ] {
            assert_eq!(class(nutzart, None), None, "{nutzart}");
        }
    }

    #[test]
    fn a_park_becomes_a_yellow_map_object_with_provenance() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set(4000);
        let park = land(
            "Sport-, Freizeit- und Erholungsfläche",
            Some("Grünanlage"),
            vec![square(&f, -100.0, -100.0, 120.0)],
        );
        let cemetery = land("Friedhof", None, vec![square(&f, 50.0, 50.0, 60.0)]);
        let homes = land("Wohnbaufläche", None, vec![square(&f, 50.0, -100.0, 60.0)]);
        let found = open_land_objects(
            &[park, cemetery, homes],
            &f.area,
            &f.clip,
            MapKind::Sprint,
            &f.georef,
            &set,
            &[],
        )
        .unwrap();
        assert_eq!(found.map.len(), 1);
        assert_eq!(Some(found.map[0].symbol_id), set.id_for_code("401"));
        assert!(found.map[0].tags.contains(&(
            "freemapper:nutzart".to_string(),
            "Sport-, Freizeit- und Erholungsfläche / Grünanlage".to_string()
        )));
        assert!(
            found.map[0]
                .tags
                .contains(&("freemapper:source".to_string(), "alkis-nutzung".to_string()))
        );
        assert_eq!(found.candidates.len(), 1);
        assert_eq!(Some(found.candidates[0].symbol_id), set.id_for_code("401"));
    }

    #[test]
    fn a_park_beyond_the_school_grounds_stays_out_of_bounds() {
        let f = fixture();
        let set = MapKind::Schoolyard.symbol_set(1000);
        // A park mostly outside the school's map area is a 520 candidate there, no 401.
        let outside = land(
            "Sport-, Freizeit- und Erholungsfläche",
            Some("Park"),
            vec![square(&f, 190.0, -50.0, 55.0)],
        );
        let found = open_land_objects(
            &[outside],
            &f.area,
            &f.clip,
            MapKind::Schoolyard,
            &f.georef,
            &set,
            &[],
        )
        .unwrap();
        assert!(found.map.is_empty() && found.candidates.is_empty());
        // On the grounds the lawn is open land.
        let inside = land(
            "Sport-, Freizeit- und Erholungsfläche",
            Some("Park"),
            vec![square(&f, -50.0, -50.0, 60.0)],
        );
        let found = open_land_objects(
            &[inside],
            &f.area,
            &f.clip,
            MapKind::Schoolyard,
            &f.georef,
            &set,
            &[],
        )
        .unwrap();
        assert_eq!(found.map.len(), 1);
    }

    #[test]
    fn forest_wins_over_an_alkis_park_on_a_forest_map() {
        let f = fixture();
        let park = land(
            "Sport-, Freizeit- und Erholungsfläche",
            Some("Park"),
            vec![square(&f, -100.0, -100.0, 120.0)],
        );
        let wood = land("Wald", None, vec![square(&f, 50.0, 50.0, 60.0)]);
        // OSM forest over the west half of the park.
        let forest = [
            square(&f, -100.0, -100.0, 60.0),
            square(&f, -100.0, -40.0, 60.0),
        ];
        for (kind, scale) in [(MapKind::Forest, 10000), (MapKind::Sprint, 4000)] {
            let georef = Georeferencing::new(
                f.georef.ref_geographic,
                scale,
                "2026-11-09".parse().unwrap(),
            )
            .unwrap();
            let set = kind.symbol_set(scale);
            let found = open_land_objects(
                &[park.clone(), wood.clone()],
                &f.area,
                &f.clip,
                kind,
                &georef,
                &set,
                &forest,
            )
            .unwrap();
            let open: Vec<_> = found
                .map
                .iter()
                .filter(|o| Some(o.symbol_id) == set.id_for_code("401"))
                .collect();
            let woods = found
                .map
                .iter()
                .filter(|o| Some(o.symbol_id) == set.id_for_code("405"))
                .count();
            let m = 1e6 / f64::from(scale);
            let area: f64 = open
                .iter()
                .map(|o| object_polygon(o).unwrap().unsigned_area() / (m * m))
                .sum();
            if kind == MapKind::Forest {
                // The east 60 m of the park stay open land; the wood is the white paper.
                assert!((area / (60.0 * 120.0) - 1.0).abs() < 0.01, "{area}");
                assert_eq!(woods, 0);
            } else {
                assert!((area / (120.0 * 120.0) - 1.0).abs() < 0.01, "{area}");
                assert_eq!(woods, 1);
            }
        }
    }

    #[test]
    fn osm_forest_polygons_are_woods_and_forests() {
        use crate::sources::osm::OsmPolygon;
        let ring = |dlat: f64| {
            vec![
                LatLon {
                    lat: 52.48 + dlat,
                    lon: 13.38,
                },
                LatLon {
                    lat: 52.48 + dlat,
                    lon: 13.381,
                },
                LatLon {
                    lat: 52.481 + dlat,
                    lon: 13.381,
                },
                LatLon {
                    lat: 52.48 + dlat,
                    lon: 13.38,
                },
            ]
        };
        let feature = |tags: &[(&str, &str)], geometry| OsmFeature {
            id: "way/1".to_string(),
            tags: tags
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            geometry,
        };
        let features = [
            feature(&[("landuse", "forest")], OsmGeometry::Line(ring(0.0))),
            feature(
                &[("natural", "wood")],
                OsmGeometry::Area(vec![OsmPolygon {
                    outer: ring(0.01),
                    holes: vec![],
                }]),
            ),
            feature(&[("landuse", "meadow")], OsmGeometry::Line(ring(0.02))),
            // An open way is no area.
            feature(
                &[("landuse", "forest")],
                OsmGeometry::Line(ring(0.03)[..3].to_vec()),
            ),
        ];
        let found = osm_forest_polygons(&features, f_crs());
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|p| p.unsigned_area() > 1000.0));
    }

    fn f_crs() -> crate::crs::Crs {
        fixture().georef.crs()
    }

    fn map_square(
        f: &Fixture,
        code: &str,
        de: f64,
        dn: f64,
        side: f64,
        set: &SymbolSet,
    ) -> MapObject {
        area_object(
            &square(f, de, dn, side),
            set.id_for_code(code).unwrap(),
            vec![("freemapper:source".to_string(), "osm".to_string())],
            &f.georef,
        )
        .unwrap()
    }

    #[test]
    fn osm_parks_stay_candidates_only_where_alkis_has_none() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set(4000);
        let alkis = vec![map_square(&f, "401", 0.0, 0.0, 100.0, &set)];
        let candidates = vec![
            // Inside the ALKIS park: gone.
            map_square(&f, "401", 10.0, 10.0, 50.0, &set),
            // Half outside: the outside half stays.
            map_square(&f, "401", 80.0, 10.0, 40.0, &set),
            // Away from it: unchanged.
            map_square(&f, "401", -150.0, -150.0, 40.0, &set),
            // Not open land: unchanged even inside.
            map_square(&f, "501.5", 10.0, 10.0, 20.0, &set),
        ];
        let kept = drop_open_land_under_alkis(candidates.clone(), &alkis, &f.georef, &set);
        assert_eq!(kept.len(), 3);
        let area = |o: &MapObject| object_polygon(o).unwrap().unsigned_area();
        let metre = 1e6 / 4000.0;
        assert!((area(&kept[0]) / (metre * metre) - 20.0 * 40.0).abs() < 2.0);
        assert_eq!(kept[0].tags, candidates[1].tags);
        assert_eq!(kept[1], candidates[2]);
        assert_eq!(kept[2], candidates[3]);
    }

    #[test]
    fn the_walkable_area_covers_the_map_window_on_sprint_maps_only() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set(4000);
        let content = PaperRect::of_bbox(&f.clip, &f.georef).unwrap();
        let base = walkable_base(&content, MapKind::Sprint, &set).unwrap();
        assert_eq!(Some(base.symbol_id), set.id_for_code("990"));
        assert_eq!(base.kind, crate::omap::MapObjectKind::Path);
        assert_eq!(
            base.tags,
            [("freemapper:source".to_string(), "base".to_string())]
        );
        // Every corner of the clip window lies on it.
        let polygon = object_polygon(&base).unwrap();
        use geo::Contains as _;
        for (e, n) in [
            (f.clip.min_e, f.clip.min_n),
            (f.clip.max_e, f.clip.max_n),
            (f.clip.min_e, f.clip.max_n),
            (f.clip.max_e, f.clip.min_n),
        ] {
            let c = f
                .georef
                .projected_to_map(crate::crs::ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap();
            let inside = geo_types::Point::new(f64::from(c.x), f64::from(c.y));
            assert!(
                polygon.contains(&inside) || polygon.exterior().intersects(&inside),
                "{e} {n}"
            );
        }
        let school = MapKind::Schoolyard.symbol_set(1000);
        assert!(walkable_base(&content, MapKind::Schoolyard, &school).is_none());
    }
}
