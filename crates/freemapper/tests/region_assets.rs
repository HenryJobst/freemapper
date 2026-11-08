//! The state boundaries in `assets/regions/` and the OSM extract boundaries in
//! `assets/regions/osm/` are valid geometries: a ring that crosses itself can put a point near the
//! crossing into the wrong state or extract.

use geo::{MultiPolygon, Validation};

/// Checks every GeoJSON boundary in `dir` (not recursing) and returns how many there were.
fn check_boundaries(dir: &std::path::Path) -> usize {
    let mut checked = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "geojson") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let collection: geojson::FeatureCollection = text.parse().unwrap();
        for feature in collection.features {
            let geometry = feature.geometry.expect("a boundary feature has a geometry");
            let shape: MultiPolygon<f64> =
                match geo_types::Geometry::<f64>::try_from(geometry).unwrap() {
                    geo_types::Geometry::MultiPolygon(m) => m,
                    geo_types::Geometry::Polygon(p) => MultiPolygon(vec![p]),
                    other => panic!("{}: not a polygon: {other:?}", path.display()),
                };
            if let Err(e) = shape.check_validation() {
                panic!("{}: invalid boundary: {e}", path.display());
            }
        }
        checked += 1;
    }
    checked
}

fn assets() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/regions")
}

#[test]
fn region_boundaries_are_valid() {
    let checked = check_boundaries(&assets());
    assert!(checked >= 10, "{checked} assets");
}

#[test]
fn osm_extract_boundaries_are_valid() {
    // The five North Rhine-Westphalia government district extracts.
    let checked = check_boundaries(&assets().join("osm"));
    assert!(checked >= 5, "{checked} assets");
}

#[test]
fn niedersachsen_has_bremen_and_bremerhaven_as_holes() {
    let text = std::fs::read_to_string(assets().join("niedersachsen.geojson")).unwrap();
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
