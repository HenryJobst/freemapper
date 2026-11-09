//! Candidates from the ALKIS vereinfacht buildings of every state (states plus 1): the building
//! passages. The canopies are map objects (`build/canopy.rs`).
//!
//! The stored `alkis-buildings.geojson` holds every `GebaeudeBauwerk` feature, but
//! `buildings.rs` keeps only the `Gebäude`. The rest (`Bauteil`, `Sonstiges Bauwerk oder
//! sonstige Einrichtung`) is read here. Features with a `bauart` are Berlin building parts and
//! stay on the Berlin rules.

use chrono::NaiveDate;
use serde_json::Value;

use crate::build::candidates::{Rules, Target, osm_objects};
use crate::build::tagged::tagged_features;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, SymbolSet};
use crate::project::MapKind;
use crate::sources::lgv::spelling_key;

/// `gebnutzbez` of a building part.
pub(crate) const PART: &str = "Bauteil";
/// `gebnutzbez` of a structure that is not a building.
pub(crate) const STRUCTURE: &str = "Sonstiges Bauwerk oder sonstige Einrichtung";
/// `funktion` of the passages (the spelling of `assets/rules/state-candidates.toml`).
pub(crate) const PASSAGES: [&str; 3] = [
    "Durchfahrt im Gebäude",
    "Durchfahrt an überbautem Verkehrsweg",
    "Arkade",
];
/// `rellage` of an underground feature.
const UNDERGROUND: &str = "Unter der Erdoberfläche";
/// The `freemapper:source` of these candidates and of the canopies.
pub(crate) const SOURCE: &str = "alkis";
/// `alkis:type` of the vereinfacht buildings and structures.
const TYPE: &str = "GebaeudeBauwerk";

/// The entry of `functions` that `feature` has as `funktion`, when it is a `gebnutzbez` = `kind`
/// feature above ground that is no Berlin building part. Spellings are compared by
/// [`spelling_key`]: Hamburg writes `Sonstiges Bauwerk Oder Sonstige Einrichtung` and
/// `Ueberdachung`, and the AdV texts have no-break spaces.
pub(crate) fn vereinfacht_function(
    feature: &Value,
    kind: &str,
    functions: &[&'static str],
) -> Option<&'static str> {
    let properties = &feature["properties"];
    let text = |key: &str| properties[key].as_str().map(spelling_key);
    if properties.get("bauart").is_some_and(|b| !b.is_null()) {
        return None;
    }
    if text("gebnutzbez")? != spelling_key(kind)
        || text("rellage") == Some(spelling_key(UNDERGROUND))
    {
        return None;
    }
    let function = text("funktion")?;
    functions
        .iter()
        .copied()
        .find(|f| spelling_key(f) == function)
}

/// The tags of a vereinfacht feature for the rules: `alkis:type` and each string property as
/// `alkis:<name>`, and the provenance tags the objects carry on (`freemapper:source_id` = oid,
/// `freemapper:currency` = `aktualit`, the function as `freemapper:alkis`).
fn feature_tags(feature: &Value) -> Vec<(String, String)> {
    let mut tags = vec![("alkis:type".to_string(), TYPE.to_string())];
    let properties = feature["properties"].as_object().into_iter().flatten();
    for (key, value) in properties {
        if let Some(text) = value.as_str().filter(|t| !t.is_empty()) {
            tags.push((format!("alkis:{key}"), text.to_string()));
        }
    }
    let property = |key: &str| {
        feature["properties"][key]
            .as_str()
            .filter(|v| !v.is_empty())
    };
    if let Some(oid) = property("oid") {
        tags.push(("freemapper:source_id".to_string(), oid.to_string()));
    }
    if let Some(date) = property("aktualit").and_then(|t| t.get(..10)) {
        tags.push(("freemapper:currency".to_string(), date.to_string()));
    }
    if let Some(function) = property("funktion") {
        tags.push(("freemapper:alkis".to_string(), function.to_string()));
    }
    tags.sort();
    tags
}

/// 522 candidates for the building passages among the stored vereinfacht `features` (in the
/// map's CRS) within `buffer_m` of `map_area`. `currency` is the fetch date, which stands in for
/// a feature without `aktualit`.
#[allow(clippy::too_many_arguments)]
pub fn state_objects(
    features: &[Value],
    kind: MapKind,
    map_area: &geo_types::Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    currency: NaiveDate,
) -> Result<Vec<MapObject>, GeorefError> {
    // The rules compare texts exactly: hand them the spelling of the rule file.
    let passages: Vec<Value> = features
        .iter()
        .filter_map(|feature| {
            let function = vereinfacht_function(feature, PART, &PASSAGES)?;
            let mut feature = feature.clone();
            feature["properties"]["gebnutzbez"] = PART.into();
            feature["properties"]["funktion"] = function.into();
            Some(feature)
        })
        .collect();
    osm_objects(
        &tagged_features("alkis", &passages, georef.crs(), feature_tags),
        &Rules::states(),
        Target::Candidates,
        kind,
        map_area,
        buffer_m,
        georef,
        symbol_set,
        SOURCE,
        currency,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::canopy::{CanopyInput, canopy_objects};
    use crate::crs::{Crs, ProjectedPoint};
    use crate::sources::alkis_wfs::{parse_gml, test_support as gml};

    /// The stored form of a vereinfacht fixture: GeoJSON features in the state's CRS.
    fn gml_features(text: &str) -> Vec<Value> {
        parse_gml(text).unwrap()
    }

    fn mv_features() -> Vec<Value> {
        let text = include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-buildings.json");
        let collection: Value = serde_json::from_str(text).unwrap();
        collection["features"].as_array().unwrap().clone()
    }

    /// Every coordinate pair of the features.
    fn corners(features: &[Value]) -> Vec<(f64, f64)> {
        fn walk(v: &Value, out: &mut Vec<(f64, f64)>) {
            if let Some(a) = v.as_array() {
                if a.len() >= 2 && a[0].is_number() {
                    out.push((a[0].as_f64().unwrap(), a[1].as_f64().unwrap()));
                } else {
                    a.iter().for_each(|x| walk(x, out));
                }
            }
        }
        let mut out = Vec::new();
        features
            .iter()
            .for_each(|f| walk(&f["geometry"]["coordinates"], &mut out));
        out
    }

    /// A map at 1:4000 around the features, in `crs`, and the map area.
    fn setup(
        crs: Crs,
        kind: MapKind,
        features: &[Value],
    ) -> (Georeferencing, geo_types::Polygon<f64>, SymbolSet) {
        let points = corners(features);
        let (e0, e1) = points.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| {
            (lo.min(p.0), hi.max(p.0))
        });
        let (n0, n1) = points.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| {
            (lo.min(p.1), hi.max(p.1))
        });
        let centre = crs
            .inverse(ProjectedPoint {
                easting: (e0 + e1) / 2.0,
                northing: (n0 + n1) / 2.0,
            })
            .unwrap();
        let georef =
            Georeferencing::new_in(crs, centre, 4000, "2026-11-09".parse().unwrap()).unwrap();
        let area = geo_types::Rect::new(
            geo_types::coord! { x: e0 - 10.0, y: n0 - 10.0 },
            geo_types::coord! { x: e1 + 10.0, y: n1 + 10.0 },
        )
        .to_polygon();
        (georef, area, kind.symbol_set())
    }

    fn passages(crs: Crs, kind: MapKind, features: &[Value]) -> Vec<MapObject> {
        let (georef, area, set) = setup(crs, kind, features);
        state_objects(
            features,
            kind,
            &area,
            20.0,
            &georef,
            &set,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap()
    }

    fn canopies(crs: Crs, features: &[Value]) -> Vec<MapObject> {
        let (georef, area, set) = setup(crs, MapKind::Sprint, features);
        canopy_objects(
            &[CanopyInput::Buildings(features)],
            &[],
            &area,
            20.0,
            &georef,
            &set,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap()
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The `Bauteil` features with a passage `funktion`, and those with `Auskragender
    /// Geschossteil`, counted straight from the properties.
    fn count(features: &[Value], function: &str) -> usize {
        features
            .iter()
            .filter(|f| f["properties"]["gebnutzbez"] == "Bauteil")
            .filter(|f| f["properties"]["funktion"] == function)
            .count()
    }

    #[test]
    fn passages_are_canopy_candidates_in_every_state() {
        let states: [(&str, Crs, Vec<Value>); 4] = [
            ("HE", Crs::Utm32, gml_features(gml::HE_BUILDINGS)),
            ("SH", Crs::Utm32, gml_features(gml::SH_BUILDINGS)),
            ("NW", Crs::Utm32, gml_features(gml::NRW_BUILDINGS)),
            ("MV", Crs::Utm33, mv_features()),
        ];
        for (state, crs, features) in &states {
            let expected: usize = PASSAGES.iter().map(|p| count(features, p)).sum();
            assert!(expected > 0, "{state}: the fixture has passages");
            assert!(
                count(features, "Auskragender Geschossteil") > 0 || *state == "SH",
                "{state}"
            );
            for kind in [MapKind::Sprint, MapKind::Schoolyard] {
                let set = kind.symbol_set();
                let found = passages(*crs, kind, features);
                assert_eq!(found.len(), expected, "{state} {kind:?}");
                for o in &found {
                    assert_eq!(Some(o.symbol_id), set.id_for_code("522"));
                    assert_eq!(tag(o, "freemapper:source"), Some("alkis"));
                    assert_eq!(tag(o, "alkis:type"), Some("GebaeudeBauwerk"));
                    assert!(tag(o, "freemapper:source_id").is_some_and(|id| id.starts_with("DE")));
                    let alkis = tag(o, "freemapper:alkis").unwrap();
                    assert!(PASSAGES.contains(&alkis), "{state}: {alkis}");
                }
            }
        }
    }

    /// A structure or part `w` × `h` m at (`e`, `n`) in the MV fixture's area.
    fn feature(kind: &str, function: &str, rellage: Option<&str>, bauart: Option<&str>) -> Value {
        let (e, n) = (312_350.0, 5_996_335.0);
        let mut properties = serde_json::json!({
            "oid": "DEMVAL0000000001", "aktualit": "2021-03-04T12:00:00Z",
            "gebnutzbez": kind, "funktion": function});
        if let Some(rellage) = rellage {
            properties["rellage"] = rellage.into();
        }
        if let Some(bauart) = bauart {
            properties["bauart"] = bauart.into();
        }
        serde_json::json!({"type": "Feature", "id": "DEMVAL0000000001",
            "geometry": {"type": "Polygon", "coordinates": [[
                [e, n], [e + 6.0, n], [e + 6.0, n + 6.0], [e, n + 6.0], [e, n]]]},
            "properties": properties})
    }

    #[test]
    fn underground_parts_are_neither_canopy_nor_passage() {
        let below = Some("Unter der Erdoberfläche");
        let control = [
            feature("Bauteil", "Durchfahrt im Gebäude", None, None),
            feature(STRUCTURE, "Carport", None, None),
        ];
        assert_eq!(
            passages(Crs::Utm33, MapKind::Sprint, &control[..1]).len(),
            1
        );
        assert_eq!(canopies(Crs::Utm33, &control[1..]).len(), 1);
        let underground = [
            feature("Bauteil", "Durchfahrt im Gebäude", below, None),
            feature("Bauteil", "Arkade", below, None),
            feature(STRUCTURE, "Überdachung", below, None),
            feature(STRUCTURE, "Carport", below, None),
        ];
        // The control features give an area to draw in.
        let mut all = control.to_vec();
        all.extend(underground);
        assert_eq!(passages(Crs::Utm33, MapKind::Sprint, &all).len(), 1);
        assert_eq!(canopies(Crs::Utm33, &all).len(), 1);
    }

    #[test]
    fn berlin_parts_stay_on_the_berlin_rules() {
        // A Berlin building part (`bauart`) never reaches the state path, whatever it is called.
        let parts = [
            feature(
                "Bauteil",
                "Durchfahrt im Gebäude",
                None,
                Some("Durchfahrt im Gebäude"),
            ),
            feature(STRUCTURE, "Überdachung", None, Some("Überdachung")),
        ];
        let control = feature("Bauteil", "Arkade", None, None);
        let mut all = parts.to_vec();
        all.push(control);
        let found = passages(Crs::Utm33, MapKind::Sprint, &all);
        assert_eq!(found.len(), 1);
        assert_eq!(tag(&found[0], "freemapper:alkis"), Some("Arkade"));
        assert!(canopies(Crs::Utm33, &all).is_empty());
        // The stored Berlin fixture, as the Berlin source normalises it, yields nothing.
        let text = include_str!("../../../../tests/fixtures/berlin-building-parts.json");
        let stored: Vec<Value> = crate::sources::berlin::parse_layer(text)
            .unwrap()
            .iter()
            .map(crate::sources::berlin::normalise_building)
            .collect();
        assert!(!stored.is_empty());
        assert!(passages(Crs::Utm33, MapKind::Sprint, &stored).is_empty());
        assert!(canopies(Crs::Utm33, &stored).is_empty());
    }
}
