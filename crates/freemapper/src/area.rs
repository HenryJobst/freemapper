//! Map area: the polygon a map is generated for.

use crate::crs::LatLon;

#[derive(Debug, thiserror::Error)]
pub enum AreaError {
    #[error("invalid GeoJSON: {0}")]
    GeoJson(String),
    #[error("GeoJSON contains no polygon")]
    NoPolygon,
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
        features
            .into_iter()
            .find_map(|(geometry, source_id)| {
                let polygon = geo_types::Polygon::<f64>::try_from(geometry?).ok()?;
                Some(Self { polygon, source_id })
            })
            .ok_or(AreaError::NoPolygon)
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
}
