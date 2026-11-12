//! Tests of the building rules of the engine on Hamburg's buildings (LGV), normalised as the
//! ALKIS source of `freemapper-sources` does.

use freemapper_adapters_de::lgv::*;
use freemapper_sources::alkis_source::normalise_building;

mod tests {
    use super::test_support::*;
    use super::*;
    use crate::sources::alkis::parse_buildings;
    use serde_json::Value;

    fn normalised(collection: &str, f: fn(Value) -> Value) -> String {
        let mut v: Value = serde_json::from_str(collection).unwrap();
        let features: Vec<Value> = v["features"]
            .as_array()
            .unwrap()
            .iter()
            .cloned()
            .map(f)
            .collect();
        v["features"] = Value::from(features);
        v.to_string()
    }

    #[test]
    fn hamburg_buildings_keep_the_kind_rule() {
        let parsed = parse_buildings(&normalised(BUILDINGS, normalise_building)).unwrap();
        let kinds: Vec<&str> = parsed.buildings.iter().map(|b| b.kind.as_str()).collect();
        assert_eq!(kinds, ["Gebäude", "Bauteil", "Gebäude", "Gebäude"]);
        let rules: Vec<crate::build::buildings::Rule> = parsed
            .buildings
            .iter()
            .map(crate::build::buildings::classify)
            .collect();
        assert_eq!(
            rules[0],
            crate::build::buildings::Rule::Object,
            "the school is drawn"
        );
        assert_ne!(
            rules[2],
            crate::build::buildings::Rule::Object,
            "underground Tiefgarage"
        );
        assert!(parsed.buildings[1].underground && parsed.buildings[2].underground);
    }
}
