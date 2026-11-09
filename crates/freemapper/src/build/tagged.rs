//! Tagged features for the candidate rules: GeoJSON features (a point, lines or polygons in a
//! projected CRS) as [`OsmFeature`]s. The tags are the caller's (`berlin:*` for Geoportal Berlin,
//! `alkis:*` for the state cadastres), so an OSM rule can never match them.

use geo::{BoundingRect as _, Centroid as _};
use serde_json::Value;

use crate::crs::{Crs, LatLon, ProjectedPoint};
use crate::sources::osm::{OsmFeature, OsmGeometry};

/// A monument area with a side longer than this is tagged `freemapper:size=large`: the mapper
/// can draw its outline instead of the 526 point.
const LARGE_MONUMENT_M: f64 = 3.0;

pub(crate) fn position(c: &Value) -> Option<(f64, f64)> {
    Some((c.get(0)?.as_f64()?, c.get(1)?.as_f64()?))
}

pub(crate) fn latlon(crs: Crs, (easting, northing): (f64, f64)) -> Option<LatLon> {
    crs.inverse(ProjectedPoint { easting, northing })
}

fn line_of(crs: Crs, coordinates: &Value) -> Option<Vec<LatLon>> {
    coordinates
        .as_array()?
        .iter()
        .map(|c| position(c).and_then(|p| latlon(crs, p)))
        .collect()
}

/// The exterior ring of a polygon as a closed line (holes are dropped: candidates only) and a
/// point at its centroid. A ring longer than [`LARGE_MONUMENT_M`] sets `large`.
fn polygon_parts(crs: Crs, rings: &Value) -> Option<(Vec<LatLon>, LatLon, bool)> {
    let exterior = rings.get(0)?.as_array()?;
    let points: Vec<(f64, f64)> = exterior.iter().filter_map(position).collect();
    let polygon = geo_types::Polygon::new(geo_types::LineString::from(points.clone()), Vec::new());
    let centroid = polygon.centroid()?;
    let rect = polygon.bounding_rect()?;
    let large = rect.width().max(rect.height()) > LARGE_MONUMENT_M;
    let ring = points
        .iter()
        .map(|p| latlon(crs, *p))
        .collect::<Option<_>>()?;
    Some((ring, latlon(crs, (centroid.x(), centroid.y()))?, large))
}

/// The tagged features of one layer, ready for the rules: a point stays a point, each part of a
/// MultiLineString is a line, each polygon is its exterior as a closed line plus a point at its
/// centroid (a rule picks one by its `draw`). `crs` is the CRS of the coordinates. Id:
/// `<id_prefix>/<feature id>`; `tags` gives the tags of a feature.
pub(crate) fn tagged_features(
    id_prefix: &str,
    features: &[Value],
    crs: Crs,
    tags: impl Fn(&Value) -> Vec<(String, String)>,
) -> Vec<OsmFeature> {
    let mut out = Vec::new();
    for feature in features {
        let id = format!("{id_prefix}/{}", feature["id"].as_str().unwrap_or_default());
        let tags = tags(feature);
        let coordinates = &feature["geometry"]["coordinates"];
        let mut push = |tags: Vec<(String, String)>, geometry| {
            out.push(OsmFeature {
                id: id.clone(),
                tags,
                geometry,
            });
        };
        let polygon = |rings: &Value, push: &mut dyn FnMut(Vec<(String, String)>, OsmGeometry)| {
            let Some((ring, centre, large)) = polygon_parts(crs, rings) else {
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
                if let Some(p) = position(coordinates).and_then(|p| latlon(crs, p)) {
                    push(tags.clone(), OsmGeometry::Point(p));
                }
            }
            Some("LineString") => {
                if let Some(line) = line_of(crs, coordinates) {
                    push(tags.clone(), OsmGeometry::Line(line));
                }
            }
            Some("MultiLineString") => {
                for line in coordinates.as_array().into_iter().flatten() {
                    if let Some(line) = line_of(crs, line) {
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
