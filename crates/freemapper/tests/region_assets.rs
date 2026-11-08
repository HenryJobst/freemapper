//! The state boundaries in `assets/regions/` are valid geometries: a ring that crosses itself
//! can put a point near the crossing into the wrong state.

use geo::{MultiPolygon, Validation};

#[test]
fn region_boundaries_are_valid() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/regions");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
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
    assert!(checked >= 6, "{checked} assets");
}
