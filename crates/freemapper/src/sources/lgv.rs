//! LGV Hamburg open data (dl-de/by-2-0, "Freie und Hansestadt Hamburg, Landesbetrieb
//! Geoinformation und Vermessung (LGV)"), all in EPSG:25832: ALKIS from the OGC API with its
//! ASCII spellings normalised to the LGB ones.

use serde_json::Value;

/// The ALKIS-vereinfacht OGC API collections (LGB-like JSON, `next` links, EPSG:25832).
pub const BUILDINGS_ENDPOINT: &str =
    "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/GebaeudeBauwerk/items";
pub const LAND_USE_ENDPOINT: &str =
    "https://api.hamburg.de/datasets/v1/alkis_vereinfacht/collections/NutzungFlurstueck/items";

/// The LGB spellings of `gebnutzbez` the build rules know.
const BUILDING_KINDS: [&str; 2] = ["Gebäude", "Bauteil"];
/// The LGB spellings of `nutzart` (the AdV class names).
const LAND_USE_CLASSES: [&str; 26] = [
    "Wohnbaufläche",
    "Industrie- und Gewerbefläche",
    "Halde",
    "Bergbaubetrieb",
    "Tagebau, Grube, Steinbruch",
    "Fläche gemischter Nutzung",
    "Fläche besonderer funktionaler Prägung",
    "Sport-, Freizeit- und Erholungsfläche",
    "Friedhof",
    "Straßenverkehr",
    "Weg",
    "Platz",
    "Bahnverkehr",
    "Flugverkehr",
    "Schiffsverkehr",
    "Landwirtschaft",
    "Wald",
    "Gehölz",
    "Heide",
    "Moor",
    "Sumpf",
    "Unland/Vegetationslose Fläche",
    "Fließgewässer",
    "Hafenbecken",
    "Stehendes Gewässer",
    "Meer",
];

/// A Hamburg building feature with the LGB spellings (`Gebaeude` → `Gebäude`).
pub fn normalise_building(feature: Value) -> Value {
    normalise(feature, "gebnutzbez", &BUILDING_KINDS)
}

/// A Hamburg land use feature with the LGB `nutzart` spellings (`Wohnbauflaeche` →
/// `Wohnbaufläche`); unknown classes are kept.
pub fn normalise_land_use(feature: Value) -> Value {
    normalise(feature, "nutzart", &LAND_USE_CLASSES)
}

/// `feature` with `property` replaced by the entry of `names` it spells in ASCII.
fn normalise(mut feature: Value, property: &str, names: &[&str]) -> Value {
    let value = &mut feature["properties"][property];
    if let Some(text) = value.as_str() {
        let key = spelling_key(text);
        if let Some(name) = names.iter().find(|name| spelling_key(name) == key) {
            *value = Value::from(*name);
        }
    }
    feature
}

/// A key equal for "Industrie- und Gewerbefläche" and "Industrie Und Gewerbeflaeche": lower
/// case, umlauts and ß written out, punctuation dropped, single spaces.
fn spelling_key(text: &str) -> String {
    let mut key = String::new();
    for c in text.to_lowercase().chars() {
        match c {
            'ä' => key.push_str("ae"),
            'ö' => key.push_str("oe"),
            'ü' => key.push_str("ue"),
            'ß' => key.push_str("ss"),
            '-' | ',' | '/' => key.push(' '),
            c => key.push(c),
        }
    }
    key.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const BUILDINGS: &str = include_str!("../../../../tests/fixtures/hamburg-buildings.json");
    pub const LAND_USE: &str = include_str!("../../../../tests/fixtures/hamburg-land-use.json");
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::sources::alkis::{parse_buildings, parse_land_use};

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
    fn hamburg_spellings_are_normalised() {
        let land = parse_land_use(&normalised(LAND_USE, normalise_land_use)).unwrap();
        let classes: Vec<(&str, Option<&str>)> = land
            .iter()
            .map(|l| (l.nutzart.as_str(), l.bez.as_deref()))
            .collect();
        assert_eq!(
            classes,
            [
                (
                    "Fläche besonderer funktionaler Prägung",
                    Some("Bildung und Wissenschaft")
                ),
                ("Straßenverkehr", Some("Begleitfläche Straßenverkehr")),
                ("Wohnbaufläche", None),
                ("Platz", Some("Parkplatz")),
                (
                    "Industrie- und Gewerbefläche",
                    Some("Handel und Dienstleistung")
                ),
            ]
        );
        let unknown = serde_json::json!({"type":"Feature","geometry":null,"properties":{"oid":"DEHH1","nutzart":"Kaum Bekannt"}});
        assert_eq!(
            normalise_land_use(unknown)["properties"]["nutzart"],
            "Kaum Bekannt"
        );
        for (hamburg, lgb) in [
            (
                "Sport Freizeit Und Erholungsflaeche",
                "Sport-, Freizeit- und Erholungsfläche",
            ),
            ("Fliessgewaesser", "Fließgewässer"),
            (
                "Unland Vegetationslose Flaeche",
                "Unland/Vegetationslose Fläche",
            ),
            ("Flaeche Gemischter Nutzung", "Fläche gemischter Nutzung"),
            ("Gehoelz", "Gehölz"),
        ] {
            let f = serde_json::json!({"type":"Feature","geometry":null,"properties":{"oid":"DEHH1","nutzart":hamburg}});
            assert_eq!(
                normalise_land_use(f)["properties"]["nutzart"],
                lgb,
                "{hamburg}"
            );
        }
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
