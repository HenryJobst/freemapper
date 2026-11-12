//! Moved to `freemapper-adapters-de` as `lgv` (ADR 0082); this module keeps the old paths working.

pub use freemapper_adapters_de::lgv::*;
// The LGB spellings and the Hamburg normaliser live in `freemapper-sources` beside the ALKIS
// sources that use them.
#[cfg(test)]
pub(crate) use freemapper_sources::alkis_source::normalise_building;
pub(crate) use freemapper_sources::alkis_source::spelling_key;

#[cfg(test)]
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
