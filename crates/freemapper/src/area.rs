//! Map area: the polygon a map is generated for.

use crate::crs::{BoundingBox, Crs, LatLon};

#[derive(Debug, thiserror::Error)]
pub enum AreaError {
    #[error("invalid GeoJSON: {0}")]
    GeoJson(String),
    #[error("GeoJSON contains no polygon")]
    NoPolygon,
    #[error("MultiPolygon with {0} parts; split the map area into one polygon")]
    MultiPolygon(usize),
    #[error("coordinate out of range: lat {lat}, lon {lon}")]
    OutOfRange { lat: f64, lon: f64 },
    /// What is wrong with a place or an OSM object, as a message of the catalog.
    #[error("{0}")]
    Place(crate::note::Note),
    #[error("not a usable map area: {0}")]
    NotAnArea(String),
}

/// Most corners of a map area outline that is checked for crossing edges.
pub const MAX_CORNERS: usize = 5000;
/// Smallest area / perimeter² of a usable map area.
const MIN_ROUNDNESS: f64 = 1e-4;

/// Metres per degree of latitude (and of longitude on the equator), close enough for a filter box.
const METRES_PER_DEGREE: f64 = 111_320.0;

/// Axis-parallel box in geographic coordinates (degrees).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoBox {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

impl GeoBox {
    pub fn contains(&self, p: LatLon) -> bool {
        (self.min_lat..=self.max_lat).contains(&p.lat)
            && (self.min_lon..=self.max_lon).contains(&p.lon)
    }
}

/// Map area as a polygon in geographic coordinates (x = longitude, y = latitude).
#[derive(Debug, Clone, PartialEq)]
pub struct MapArea {
    pub polygon: geo_types::Polygon<f64>,
    /// Identifier of the source object, e.g. `way/992423119`, if known.
    pub source_id: Option<String>,
}

impl MapArea {
    /// Reads the first polygon of a GeoJSON feature collection, feature or geometry. The
    /// source id is taken from the feature property `osm_id`.
    pub fn from_geojson(s: &str) -> Result<Self, AreaError> {
        let geojson: geojson::GeoJson = s
            .parse()
            .map_err(|e: geojson::Error| AreaError::GeoJson(e.to_string()))?;
        let features: Vec<(Option<geojson::Geometry>, Option<String>)> = match geojson {
            geojson::GeoJson::FeatureCollection(fc) => fc
                .features
                .into_iter()
                .map(|f| (f.geometry.clone(), osm_id(&f)))
                .collect(),
            geojson::GeoJson::Feature(f) => vec![(f.geometry.clone(), osm_id(&f))],
            geojson::GeoJson::Geometry(g) => vec![(Some(g), None)],
        };
        let (polygon, source_id) = features
            .into_iter()
            .find_map(|(geometry, source_id)| {
                let polygon = match geo_types::Geometry::<f64>::try_from(geometry?).ok()? {
                    geo_types::Geometry::Polygon(p) => Ok(p),
                    geo_types::Geometry::MultiPolygon(mp) if mp.0.len() == 1 => {
                        Ok(mp.0.into_iter().next().expect("one part"))
                    }
                    geo_types::Geometry::MultiPolygon(mp) => {
                        Err(AreaError::MultiPolygon(mp.0.len()))
                    }
                    _ => return None,
                };
                Some(polygon.map(|p| (p, source_id)))
            })
            .ok_or(AreaError::NoPolygon)??;
        if polygon.exterior().0.is_empty() {
            return Err(AreaError::NotAnArea(
                "the polygon has no corners".to_string(),
            ));
        }
        for c in polygon
            .exterior()
            .coords()
            .chain(polygon.interiors().iter().flat_map(|r| r.coords()))
        {
            if !(-90.0..=90.0).contains(&c.y) || !(-180.0..=180.0).contains(&c.x) {
                return Err(AreaError::OutOfRange { lat: c.y, lon: c.x });
            }
        }
        Ok(Self { polygon, source_id })
    }

    /// Checks that a drawn polygon encloses an area: at least three corners, no crossing edges,
    /// at most [`MAX_CORNERS`] corners (the crossing check takes quadratic time).
    pub fn validate(&self) -> Result<(), AreaError> {
        use geo::{Area as _, Euclidean, Length as _, Validation as _};
        let corners = self.polygon.exterior().0.len().saturating_sub(1);
        if corners > MAX_CORNERS {
            return Err(AreaError::NotAnArea(format!(
                "{corners} corners, at most {MAX_CORNERS}; simplify the outline"
            )));
        }
        if corners == 0 {
            return Err(AreaError::NotAnArea(
                "the polygon has no corners".to_string(),
            ));
        }
        let polygon = self.projected_polygon();
        // Corners on one line enclose (almost) nothing: area against perimeter², 1/(4π) for a
        // circle, 1/16 for a square, still 0.0025 for a 10 m × 1 km corridor.
        let perimeter = Euclidean.length(polygon.exterior());
        if polygon.unsigned_area() <= MIN_ROUNDNESS * perimeter * perimeter {
            return Err(AreaError::NotAnArea(
                "it has no area: draw at least three corners, with edges that do not cross"
                    .to_string(),
            ));
        }
        polygon
            .check_validation()
            .map_err(|e| AreaError::NotAnArea(format!("{e}; edges must not cross")))
    }

    /// The polygon in EPSG:25833.
    pub fn projected_polygon(&self) -> geo_types::Polygon<f64> {
        self.projected_polygon_in(Crs::Utm33)
    }

    /// The polygon in `crs`.
    pub fn projected_polygon_in(&self, crs: Crs) -> geo_types::Polygon<f64> {
        use geo::MapCoords as _;
        self.polygon.map_coords(|c| {
            let p = crs.forward(LatLon { lat: c.y, lon: c.x });
            geo_types::coord! { x: p.easting, y: p.northing }
        })
    }

    /// Geographic bounds of the polygon grown by `buffer_m` metres on every side.
    pub fn geo_box(&self, buffer_m: f64) -> GeoBox {
        use geo::BoundingRect as _;
        let rect = self
            .polygon
            .bounding_rect()
            .expect("map area polygon is not empty");
        let mid_lat = (rect.min().y + rect.max().y) / 2.0;
        let dlat = buffer_m / METRES_PER_DEGREE;
        let dlon = buffer_m / (METRES_PER_DEGREE * mid_lat.to_radians().cos());
        GeoBox {
            min_lat: rect.min().y - dlat,
            min_lon: rect.min().x - dlon,
            max_lat: rect.max().y + dlat,
            max_lon: rect.max().x + dlon,
        }
    }

    /// Bounding box of the exterior ring in EPSG:25833.
    pub fn projected_bbox(&self) -> BoundingBox {
        self.projected_bbox_in(Crs::Utm33)
    }

    /// Bounds of the polygon in `crs`.
    pub fn projected_bbox_in(&self, crs: Crs) -> BoundingBox {
        let mut points = self
            .polygon
            .exterior()
            .coords()
            .map(|c| crs.forward(LatLon { lat: c.y, lon: c.x }));
        let first = points.next().expect("map area polygon is not empty");
        points.fold(
            BoundingBox {
                min_e: first.easting,
                min_n: first.northing,
                max_e: first.easting,
                max_n: first.northing,
            },
            |b, p| BoundingBox {
                min_e: b.min_e.min(p.easting),
                min_n: b.min_n.min(p.northing),
                max_e: b.max_e.max(p.easting),
                max_n: b.max_n.max(p.northing),
            },
        )
    }

    /// Area centroid of the polygon.
    pub fn centroid(&self) -> LatLon {
        use geo::Centroid;
        let c = self
            .polygon
            .centroid()
            .expect("map area polygon is not empty");
        LatLon {
            lat: c.y(),
            lon: c.x(),
        }
    }

    /// The polygon as GeoJSON geometry.
    pub fn to_geojson_value(&self) -> serde_json::Value {
        let geometry = geojson::Geometry::from(&self.polygon);
        serde_json::to_value(geometry).expect("GeoJSON geometry serializes")
    }
}

fn osm_id(feature: &geojson::Feature) -> Option<String> {
    feature.property("osm_id")?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/clara-zetkin-schule.geojson");

    #[test]
    fn validate_refuses_too_many_corners_quickly() {
        let corners: Vec<String> = (0..=MAX_CORNERS + 1)
            .map(|i| {
                let a = i as f64 / (MAX_CORNERS + 1) as f64 * std::f64::consts::TAU;
                format!("[{},{}]", 13.84 + 0.002 * a.cos(), 52.54 + 0.001 * a.sin())
            })
            .collect();
        let text = format!(
            r#"{{"type":"Polygon","coordinates":[[{},{}]]}}"#,
            corners.join(","),
            corners[0]
        );
        let start = std::time::Instant::now();
        let err = MapArea::from_geojson(&text)
            .unwrap()
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("corners"), "{err}");
        assert!(start.elapsed() < std::time::Duration::from_secs(1));
    }

    #[test]
    fn crossing_edges_are_named() {
        let bowtie = r#"{"type":"Polygon","coordinates":[[[13.84,52.53],[13.85,52.54],[13.85,52.53],[13.84,52.54],[13.84,52.53]]]}"#;
        let err = MapArea::from_geojson(bowtie)
            .unwrap()
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("cross"), "{err}");
    }

    #[test]
    fn validate_rejects_degenerate_and_crossing() {
        assert!(MapArea::from_geojson(FIXTURE).unwrap().validate().is_ok());
        for coordinates in [
            "[[13.84,52.53],[13.85,52.54],[13.84,52.53]]",
            "[[13.84,52.53],[13.85,52.54],[13.86,52.55],[13.84,52.53]]",
            "[[13.84,52.53],[13.85,52.54],[13.85,52.53],[13.84,52.54],[13.84,52.53]]",
        ] {
            let text = format!(r#"{{"type":"Polygon","coordinates":[{coordinates}]}}"#);
            let err = MapArea::from_geojson(&text)
                .unwrap()
                .validate()
                .unwrap_err();
            assert!(
                matches!(err, AreaError::NotAnArea(_)),
                "{coordinates}: {err:?}"
            );
        }
    }

    fn geometry(json: &str) -> Result<MapArea, AreaError> {
        MapArea::from_geojson(json)
    }

    #[test]
    fn multipolygon_with_one_part_is_accepted() {
        let area = geometry(
            r#"{"type":"MultiPolygon","coordinates":[[[[13.0,52.0],[13.1,52.0],[13.1,52.1],[13.0,52.0]]]]}"#,
        )
        .unwrap();
        assert_eq!(area.polygon.exterior().0.len(), 4);
    }

    #[test]
    fn multipolygon_with_several_parts_is_rejected() {
        let err = geometry(
            r#"{"type":"MultiPolygon","coordinates":[
                [[[13.0,52.0],[13.1,52.0],[13.1,52.1],[13.0,52.0]]],
                [[[14.0,52.0],[14.1,52.0],[14.1,52.1],[14.0,52.0]]]]}"#,
        )
        .unwrap_err();
        assert!(matches!(err, AreaError::MultiPolygon(2)), "{err:?}");
        assert!(err.to_string().contains("2 parts"), "{err}");
    }

    #[test]
    fn coordinates_out_of_range_are_rejected() {
        let err = geometry(
            r#"{"type":"Polygon","coordinates":[[[200.0,52.0],[13.1,52.0],[13.1,52.1],[200.0,52.0]]]}"#,
        )
        .unwrap_err();
        assert!(matches!(err, AreaError::OutOfRange { .. }), "{err:?}");
    }

    #[test]
    fn projected_bbox_of_fixture() {
        let bbox = MapArea::from_geojson(FIXTURE).unwrap().projected_bbox();
        assert!(421_000.0 < bbox.min_e && bbox.max_e < 422_000.0, "{bbox:?}");
        assert!(
            5_821_000.0 < bbox.min_n && bbox.max_n < 5_822_000.0,
            "{bbox:?}"
        );
    }

    #[test]
    fn projected_polygon_of_fixture() {
        use geo::BoundingRect as _;
        let area = MapArea::from_geojson(FIXTURE).unwrap();
        let polygon = area.projected_polygon();
        assert_eq!(polygon.exterior().0.len(), area.polygon.exterior().0.len());
        let rect = polygon.bounding_rect().unwrap();
        let bbox = area.projected_bbox();
        assert_eq!((rect.min().x, rect.min().y), (bbox.min_e, bbox.min_n));
        assert_eq!((rect.max().x, rect.max().y), (bbox.max_e, bbox.max_n));
    }

    #[test]
    fn geo_box_grows_by_buffer() {
        let area = MapArea::from_geojson(FIXTURE).unwrap();
        let tight = area.geo_box(0.0);
        let grown = area.geo_box(100.0);
        // 100 m are about 0.0009° of latitude and 0.0015° of longitude at 52.5° N.
        assert!(
            ((tight.min_lat - grown.min_lat) - 0.000_898).abs() < 0.000_01,
            "{grown:?}"
        );
        assert!(
            ((grown.max_lon - tight.max_lon) - 0.001_477).abs() < 0.000_02,
            "{grown:?}"
        );
        assert!(tight.min_lat < 52.5402 && tight.max_lat > 52.5402);
    }

    #[test]
    fn centroid_of_fixture() {
        let area = MapArea::from_geojson(FIXTURE).unwrap();
        let c = area.centroid();
        assert!((c.lat - 52.54008).abs() < 0.0005, "{c:?}");
        assert!((c.lon - 13.84419).abs() < 0.0005, "{c:?}");
        assert_eq!(area.source_id.as_deref(), Some("way/992423119"));
    }

    #[test]
    fn bare_polygon_geometry_is_accepted() {
        let geometry = r#"{"type":"Polygon","coordinates":[[[13.0,52.0],[13.1,52.0],[13.1,52.1],[13.0,52.0]]]}"#;
        let area = MapArea::from_geojson(geometry).unwrap();
        assert_eq!(area.polygon.exterior().0.len(), 4);
        assert_eq!(area.source_id, None);
    }

    #[test]
    fn point_is_rejected() {
        let point = r#"{"type":"Point","coordinates":[13.0,52.0]}"#;
        assert!(matches!(
            MapArea::from_geojson(point),
            Err(AreaError::NoPolygon)
        ));
    }

    #[test]
    fn an_empty_polygon_is_an_error_not_a_panic() {
        for text in [
            r#"{"type":"Polygon","coordinates":[]}"#,
            r#"{"type":"Polygon","coordinates":[[]]}"#,
            r#"{"type":"MultiPolygon","coordinates":[[]]}"#,
        ] {
            let err = MapArea::from_geojson(text).unwrap_err();
            assert!(
                matches!(
                    err,
                    AreaError::NotAnArea(_) | AreaError::NoPolygon | AreaError::GeoJson(_)
                ),
                "{text}: {err:?}"
            );
        }
    }
}
