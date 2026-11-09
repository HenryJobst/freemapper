//! Private land (ALKIS land use) → 520 out-of-bounds candidates (spec §4.5).

use geo::{Area as _, BooleanOps as _, Length as _};
use geo_types::Polygon;

use crate::crs::{BoundingBox, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::sources::alkis::LandUse;

/// ISSprOM 520 area that shall not be entered.
const OUT_OF_BOUNDS_CODE: &str = "520";
/// Land uses that are private ground: residents, businesses, the railway (and allotment gardens,
/// see [`is_private`]).
const PRIVATE_LAND_USES: [&str; 4] = [
    "Wohnbaufläche",
    "Industrie- und Gewerbefläche",
    "Fläche gemischter Nutzung",
    "Bahnverkehr",
];
/// On the schoolyard map, parts at least this much inside the map area are the school's land.
const OWN_LAND_SHARE: f64 = 0.5;
/// Pieces smaller than (1 mm)² or narrower than 1 mm on paper are slivers along parcel
/// boundaries and at the clip edge.
const MIN_SIDE_MM: f64 = 1.0;

/// The `bez` of the sports and recreation land use that is private ground: allotment gardens.
const ALLOTMENT_BEZ: &str = "Kleingarten";
/// The `nutzart` an allotment garden is a `bez` of.
const RECREATION_NUTZART: &str = "Sport-, Freizeit- und Erholungsfläche";

pub fn is_private(land: &LandUse) -> bool {
    PRIVATE_LAND_USES.contains(&land.nutzart.as_str())
        || (land.nutzart == RECREATION_NUTZART && land.bez.as_deref() == Some(ALLOTMENT_BEZ))
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
    let clip = LandClip::new(clip, georef);
    let mut objects = Vec::new();
    let area_size = map_area.unsigned_area();
    // Schoolyard: everything off the school grounds is out of bounds (spec §4.5); sprint: only
    // private land, streets and woods are part of the course.
    let candidate = |l: &LandUse| kind == MapKind::Schoolyard || is_private(l);
    for land in land_use.iter().filter(|l| candidate(l)) {
        for part in &land.footprint.0 {
            let part_area = part.unsigned_area();
            if part_area <= 0.0 {
                continue;
            }
            // The share of the smaller of both shapes: a school parcel larger than the OSM
            // outline still counts as the school's own land.
            let overlap = part.intersection(map_area).unsigned_area();
            if kind == MapKind::Schoolyard && overlap / part_area.min(area_size) >= OWN_LAND_SHARE {
                continue;
            }
            objects.extend(clipped_part_objects(land, part, &clip, symbol_id, georef)?);
        }
    }
    Ok(objects)
}

/// The clip window for land use objects (map area plus buffer) and the sliver limits.
pub(crate) struct LandClip {
    pub(crate) polygon: Polygon<f64>,
    min_area: f64,
    min_width: f64,
}

impl LandClip {
    pub(crate) fn new(clip: &BoundingBox, georef: &Georeferencing) -> Self {
        Self::with_min_side(clip, georef, MIN_SIDE_MM)
    }

    /// The clip window with slivers below `min_side_mm` on paper instead of [`MIN_SIDE_MM`].
    pub(crate) fn with_min_side(
        clip: &BoundingBox,
        georef: &Georeferencing,
        min_side_mm: f64,
    ) -> Self {
        let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0;
        Self {
            polygon: geo_types::Rect::new(
                geo_types::coord! { x: clip.min_e, y: clip.min_n },
                geo_types::coord! { x: clip.max_e, y: clip.max_n },
            )
            .to_polygon(),
            min_area: (min_side_mm * metres_per_mm).powi(2),
            min_width: min_side_mm * metres_per_mm,
        }
    }

    /// `polygon` is a sliver: smaller than (1 mm)², or narrower than 1 mm on paper or than
    /// `width_cap_m` on the ground, whichever is less (`f64::INFINITY`: the paper limit alone,
    /// `0.0`: no width limit).
    pub(crate) fn is_sliver(&self, polygon: &Polygon<f64>, width_cap_m: f64) -> bool {
        polygon.unsigned_area() < self.min_area
            || mean_width(polygon) < self.min_width.min(width_cap_m)
    }
}

/// Mean width of a polygon: 2·area/perimeter, holes included.
pub(crate) fn mean_width(polygon: &Polygon<f64>) -> f64 {
    let perimeter: f64 = std::iter::once(polygon.exterior())
        .chain(polygon.interiors())
        .map(|r| geo::Euclidean.length(r))
        .sum();
    2.0 * polygon.unsigned_area() / perimeter.max(f64::MIN_POSITIVE)
}

/// The land use's class as tagged: `nutzart / bez`, or `nutzart` alone.
pub(crate) fn nutzart_label(land: &LandUse) -> String {
    match &land.bez {
        Some(bez) => format!("{} / {bez}", land.nutzart),
        None => land.nutzart.clone(),
    }
}

/// Area objects with `symbol_id` for the pieces of `part` (a part of `land`) inside the clip
/// window, slivers dropped, tagged with the land use's provenance.
pub(crate) fn clipped_part_objects(
    land: &LandUse,
    part: &Polygon<f64>,
    clip: &LandClip,
    symbol_id: u32,
    georef: &Georeferencing,
) -> Result<Vec<MapObject>, GeorefError> {
    clipped_pieces(part, clip, f64::INFINITY)
        .iter()
        .map(|piece| area_object(piece, symbol_id, land_tags(land), georef))
        .collect()
}

/// The pieces of `part` inside the clip window, slivers dropped ([`LandClip::is_sliver`] with
/// `width_cap_m`).
pub(crate) fn clipped_pieces(
    part: &Polygon<f64>,
    clip: &LandClip,
    width_cap_m: f64,
) -> Vec<Polygon<f64>> {
    part.intersection(&clip.polygon)
        .0
        .into_iter()
        .filter(|piece| !clip.is_sliver(piece, width_cap_m))
        .collect()
}

/// The provenance tags of an object from the land use `land`.
pub(crate) fn land_tags(land: &LandUse) -> Vec<(String, String)> {
    let mut tags = vec![
        ("freemapper:source".to_string(), "alkis-nutzung".to_string()),
        ("freemapper:source_id".to_string(), land.oid.clone()),
        ("freemapper:nutzart".to_string(), nutzart_label(land)),
    ];
    if let Some(updated) = land.updated {
        tags.push(("freemapper:currency".to_string(), updated.to_string()));
    }
    tags
}

/// The area object with `symbol_id` and `tags` for `polygon` (map CRS), holes kept.
pub(crate) fn area_object(
    polygon: &Polygon<f64>,
    symbol_id: u32,
    tags: Vec<(String, String)>,
    georef: &Georeferencing,
) -> Result<MapObject, GeorefError> {
    let mut coords = Vec::new();
    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
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
    Ok(MapObject {
        symbol_id,
        kind: MapObjectKind::Path,
        coords,
        tags,
    })
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
    fn kleingarten_is_out_of_bounds_candidate() {
        let f = fixture();
        let (e, n) = f.centre;
        let garden = |bez: Option<&str>| LandUse {
            bez: bez.map(str::to_string),
            ..land(
                "Sport-, Freizeit- und Erholungsfläche",
                vec![Polygon::new(ring(e + 55.0, n - 20.0, 12.0, 40.0), vec![])],
            )
        };
        assert!(is_private(&garden(Some("Kleingarten"))));
        // The rest of that land use stays open: parks, sports grounds, playgrounds.
        for bez in [
            None,
            Some("Grünanlage"),
            Some("Sportanlage"),
            Some("Spielplatz, Bolzplatz"),
        ] {
            assert!(!is_private(&garden(bez)), "{bez:?}");
        }
        // In every state, on both map kinds, as a 520 candidate that says what it is.
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let (found, set) = objects(&f, &[garden(Some("Kleingarten"))], kind);
            assert_eq!(found.len(), 1, "{kind:?}");
            assert_eq!(Some(found[0].symbol_id), set.id_for_code("520"));
            assert!(found[0].tags.contains(&(
                "freemapper:nutzart".to_string(),
                "Sport-, Freizeit- und Erholungsfläche / Kleingarten".to_string()
            )));
        }
        assert!(
            objects(&f, &[garden(Some("Grünanlage"))], MapKind::Sprint)
                .0
                .is_empty()
        );
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
                p.easting <= f.clip.max_e + 0.01
                    && p.easting >= f.clip.min_e - 0.01
                    && p.northing >= f.clip.min_n - 0.01
                    && p.northing <= f.clip.max_n + 0.01,
                "{p:?}"
            );
        }
    }

    #[test]
    fn large_school_parcel_is_own_land() {
        let f = fixture();
        let (e, n) = f.centre;
        // The school's parcel is four times the map area drawn from OSM.
        let school = land(
            "Fläche gemischter Nutzung",
            vec![Polygon::new(
                ring(e - 100.0, n - 100.0, 200.0, 200.0),
                vec![],
            )],
        );
        assert!(objects(&f, &[school], MapKind::Schoolyard).0.is_empty());
    }

    #[test]
    fn schoolyard_marks_all_other_land() {
        let f = fixture();
        let (e, n) = f.centre;
        let street = land(
            "Straßenverkehr",
            vec![Polygon::new(ring(e + 52.0, n - 30.0, 10.0, 60.0), vec![])],
        );
        let wood = land(
            "Wald",
            vec![Polygon::new(ring(e - 65.0, n - 30.0, 12.0, 60.0), vec![])],
        );
        let land_use = [street, wood];
        assert_eq!(
            objects(&f, &land_use, MapKind::Schoolyard).0.len(),
            2,
            "off the school grounds"
        );
        assert!(
            objects(&f, &land_use, MapKind::Sprint).0.is_empty(),
            "public on a sprint map"
        );
    }

    #[test]
    fn narrow_strips_are_dropped() {
        let f = fixture();
        let (e, n) = f.centre;
        // 8 m² but only 0.8 m wide: below 1 mm on paper at 1:1000.
        let strip = land(
            "Wohnbaufläche",
            vec![Polygon::new(ring(e + 55.0, n, 0.8, 10.0), vec![])],
        );
        assert!(objects(&f, &[strip], MapKind::Schoolyard).0.is_empty());
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
