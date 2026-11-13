//! A layer of features as freemapper stores it in a project: one GeoJSON FeatureCollection per
//! file. The extra layers of the adapters, the full ALKIS and the tree cadastres are stored and
//! read back this way.

use serde_json::Value;

/// `features` as the GeoJSON FeatureCollection stored for a layer (an empty one for no feature).
pub fn layer_to_geojson(features: &[Value]) -> String {
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// The features of a stored layer file.
pub fn parse_layer(text: &str) -> Result<Vec<Value>, serde_json::Error> {
    let collection: Value = serde_json::from_str(text)?;
    Ok(collection["features"]
        .as_array()
        .cloned()
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_layer_comes_back_as_it_was_stored() {
        let features = vec![json!({"type": "Feature", "id": 1}), json!({"id": 2})];
        assert_eq!(parse_layer(&layer_to_geojson(&features)).unwrap(), features);
        assert_eq!(
            layer_to_geojson(&[]),
            r#"{"features":[],"type":"FeatureCollection"}"#
        );
        // A file without features is an empty layer; a file that is no JSON is an error.
        assert!(parse_layer("{}").unwrap().is_empty());
        assert!(parse_layer("{").is_err());
    }
}
