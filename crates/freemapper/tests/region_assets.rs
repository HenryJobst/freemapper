//! The state boundaries and the OSM extract boundaries in the folders of the manifests
//! (`crates/adapters/de/manifests/<id>/`) are valid geometries: a ring that crosses itself can put a point near the
//! crossing into the wrong state or extract.

use geo::{MultiPolygon, Validation};

/// Checks the GeoJSON boundary `path`.
fn check_boundary(path: &std::path::Path) {
    let text = std::fs::read_to_string(path).unwrap();
    let collection: geojson::FeatureCollection = text.parse().unwrap();
    for feature in collection.features {
        let geometry = feature.geometry.expect("a boundary feature has a geometry");
        let shape: MultiPolygon<f64> = match geo_types::Geometry::<f64>::try_from(geometry).unwrap()
        {
            geo_types::Geometry::MultiPolygon(m) => m,
            geo_types::Geometry::Polygon(p) => MultiPolygon(vec![p]),
            other => panic!("{}: not a polygon: {other:?}", path.display()),
        };
        if let Err(e) = shape.check_validation() {
            panic!("{}: invalid boundary: {e}", path.display());
        }
    }
}

/// The folders of the manifests of the states, each with the boundary files it names.
fn manifests() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../adapters/de/manifests")
}

/// The files in the folders of the manifests whose name `keep` accepts.
fn boundary_files(keep: fn(&str) -> bool) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for folder in std::fs::read_dir(manifests()).unwrap() {
        for entry in std::fs::read_dir(folder.unwrap().path()).unwrap() {
            let path = entry.unwrap().path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(keep)
            {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn region_boundaries_are_valid() {
    let files = boundary_files(|name| name == "boundary.geojson");
    assert_eq!(files.len(), 14, "{files:?}");
    files.iter().for_each(|file| check_boundary(file));
}

#[test]
fn osm_extract_boundaries_are_valid() {
    // The government district extracts: five of North Rhine-Westphalia, four of
    // Baden-Württemberg.
    let files = boundary_files(|name| name.starts_with("osm-") && name.ends_with(".geojson"));
    assert_eq!(files.len(), 9, "{files:?}");
    files.iter().for_each(|file| check_boundary(file));
}

#[test]
fn niedersachsen_has_bremen_and_bremerhaven_as_holes() {
    let text = std::fs::read_to_string(manifests().join("niedersachsen/boundary.geojson")).unwrap();
    let collection: geojson::FeatureCollection = text.parse().unwrap();
    let holes: usize = collection
        .features
        .into_iter()
        .map(
            |f| match geo_types::Geometry::<f64>::try_from(f.geometry.unwrap()).unwrap() {
                geo_types::Geometry::MultiPolygon(m) => {
                    m.0.iter().map(|p| p.interiors().len()).sum()
                }
                geo_types::Geometry::Polygon(p) => p.interiors().len(),
                other => panic!("not a polygon: {other:?}"),
            },
        )
        .sum();
    assert!(holes >= 2, "{holes} holes");
}
