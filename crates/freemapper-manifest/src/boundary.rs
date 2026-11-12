//! The boundary file of an adapter: GeoJSON with one polygon feature, in EPSG:25833 or in
//! longitude and latitude, read into the EPSG:25833 boundary the region functions ask.

use std::path::Path;

use freemapper_georef::crs::{LatLon, utm33_forward};
use geo::{MapCoords as _, Validation as _};
use geo_types::{MultiPolygon, Polygon, coord};

use crate::error::ManifestError;
use crate::model::BoundaryCrs;

/// The largest boundary file read, in bytes.
pub const MAX_BOUNDARY_BYTES: usize = 2 * 1024 * 1024;
/// The most vertices a boundary may have.
pub const MAX_VERTICES: usize = 1_000_000;

fn finding(code: &'static str, path: &Path, message: impl Into<String>) -> ManifestError {
    ManifestError {
        code,
        path: path.to_path_buf(),
        line: 0,
        key: String::new(),
        message: message.into(),
    }
}

/// Reads the boundary file `path`, whose positions are in `crs`.
///
/// Findings: `manifest.file-missing` (the file is not there or cannot be read),
/// `boundary.too-large`, `boundary.format` (not GeoJSON with one polygon feature),
/// `boundary.crs` (the positions cannot be of `crs`), `boundary.invalid` (a ring that crosses
/// itself or another defect of the geometry).
pub fn load_boundary(path: &Path, crs: BoundaryCrs) -> Result<MultiPolygon<f64>, ManifestError> {
    let unreadable = |e: std::io::Error| finding("manifest.file-missing", path, e.to_string());
    let size = std::fs::metadata(path).map_err(unreadable)?.len();
    if size > MAX_BOUNDARY_BYTES as u64 {
        return Err(too_large(path, size));
    }
    let text = std::fs::read_to_string(path).map_err(unreadable)?;
    parse_boundary(&text, crs, path)
}

fn too_large(path: &Path, bytes: u64) -> ManifestError {
    finding(
        "boundary.too-large",
        path,
        format!("the file has {bytes} bytes; a boundary file has at most {MAX_BOUNDARY_BYTES}"),
    )
}

/// The boundary in the GeoJSON `text`, whose positions are in `crs`, as a valid geometry in
/// EPSG:25833. `path` names the file in a finding ([`load_boundary`] lists the findings).
pub fn parse_boundary(
    text: &str,
    crs: BoundaryCrs,
    path: &Path,
) -> Result<MultiPolygon<f64>, ManifestError> {
    parse_with(text, crs, path, MAX_VERTICES)
}

fn parse_with(
    text: &str,
    crs: BoundaryCrs,
    path: &Path,
    max_vertices: usize,
) -> Result<MultiPolygon<f64>, ManifestError> {
    if text.len() > MAX_BOUNDARY_BYTES {
        return Err(too_large(path, text.len() as u64));
    }
    let format = |what: &str| finding("boundary.format", path, what);
    let geometry = match text.parse::<geojson::GeoJson>() {
        Ok(geojson::GeoJson::FeatureCollection(mut collection)) => {
            if collection.features.len() != 1 {
                return Err(format(&format!(
                    "a boundary file holds one feature, this one {}",
                    collection.features.len()
                )));
            }
            collection.features.remove(0).geometry
        }
        Ok(geojson::GeoJson::Feature(feature)) => feature.geometry,
        Ok(geojson::GeoJson::Geometry(_)) => {
            return Err(format(
                "a boundary file is a FeatureCollection with one feature",
            ));
        }
        Err(e) => return Err(format(&format!("not GeoJSON: {e}"))),
    };
    let Some(geometry) = geometry else {
        return Err(format("the feature has no geometry"));
    };
    let not_polygons = || format("the feature is no Polygon and no MultiPolygon");
    let boundary: MultiPolygon<f64> = match geometry.value {
        value @ geojson::GeometryValue::MultiPolygon { .. } => {
            MultiPolygon::try_from(value).map_err(|_| not_polygons())?
        }
        value @ geojson::GeometryValue::Polygon { .. } => {
            MultiPolygon(vec![Polygon::try_from(value).map_err(|_| not_polygons())?])
        }
        _ => return Err(not_polygons()),
    };
    let rings = || {
        boundary
            .0
            .iter()
            .flat_map(|polygon| std::iter::once(polygon.exterior()).chain(polygon.interiors()))
    };
    let vertices = rings().map(|ring| ring.0.len()).sum::<usize>();
    if vertices == 0 {
        return Err(format("the feature has no polygon"));
    }
    if vertices > max_vertices {
        return Err(finding(
            "boundary.too-large",
            path,
            format!("the boundary has {vertices} vertices; it may have {max_vertices}"),
        ));
    }
    let positions = || rings().flat_map(|ring| ring.0.iter().copied());
    if positions().any(|c| !c.x.is_finite() || !c.y.is_finite()) {
        return Err(format("a position is no number"));
    }
    let degrees = |c: geo_types::Coord<f64>| c.x.abs() <= 180.0 && c.y.abs() <= 90.0;
    let boundary = match crs {
        BoundaryCrs::Utm33 => {
            if positions().all(degrees) {
                return Err(finding(
                    "boundary.crs",
                    path,
                    "the positions are degrees, but the manifest says EPSG:25833; \
                     say `crs = 4326` in `[boundary]`",
                ));
            }
            boundary
        }
        BoundaryCrs::Wgs84 => {
            if !positions().all(degrees) {
                return Err(finding(
                    "boundary.crs",
                    path,
                    "the positions are no degrees, but the manifest says EPSG:4326",
                ));
            }
            boundary.map_coords(|c| {
                let point = utm33_forward(LatLon { lat: c.y, lon: c.x });
                coord! { x: point.easting, y: point.northing }
            })
        }
    };
    boundary.check_validation().map_err(|e| {
        finding(
            "boundary.invalid",
            path,
            format!("the boundary is no valid geometry: {e}"),
        )
    })?;
    Ok(boundary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_boundary_has_at_most_as_many_vertices_as_allowed() {
        let text = r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
            [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#;
        let path = Path::new("sample/boundary.geojson");
        assert!(parse_with(text, BoundaryCrs::Utm33, path, 5).is_ok());
        let refused = parse_with(text, BoundaryCrs::Utm33, path, 4).unwrap_err();
        assert_eq!(refused.code, "boundary.too-large");
    }
}
