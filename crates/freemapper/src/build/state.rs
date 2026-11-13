//! Candidates from the ALKIS of every state (states plus 1), through the state rules
//! (`assets/rules/state-candidates.toml`):
//!
//! - the building passages of the ALKIS vereinfacht buildings. The canopies are map objects
//!   (`build/canopy.rs`). The stored `alkis-buildings.geojson` holds every `GebaeudeBauwerk`
//!   feature, but `buildings.rs` keeps only the `Gebäude`. The rest (`Bauteil`, `Sonstiges
//!   Bauwerk oder sonstige Einrichtung`) is read here. Features with a `bauart` are Berlin
//!   building parts and stay on the Berlin rules.
//! - the walls, fences, hedges, monuments, bridges, embankment tops and ditches of the full
//!   ALKIS (NAS) layers ([`nas_objects`]). The rules match the AdV codes; the labels for the
//!   mapper come from `assets/rules/adv-codes.toml`.
//! - the trees of the city tree cadastres ([`tree_objects`], `sources/trees.rs`), matched on
//!   `tree:*` tags.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use chrono::NaiveDate;
use serde_json::Value;

use crate::build::candidates::{MIN_LINE_MM, Rules, Target, osm_objects};
use crate::build::dedup::add_also;
use crate::build::tagged::{PointOn, geom, tagged_features};
use crate::build::walls::{PAIRED, PAIRED_ALSO, PAIRED_LABEL, pair_distance_m, pair_wall_edges};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, SymbolSet};
use crate::project::MapKind;
use freemapper_sources::alkis_source::spelling_key;

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
    if let Some(geom) = geom(feature) {
        tags.push(("alkis:geom".to_string(), geom.to_string()));
    }
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
        &tagged_features(
            "alkis",
            &passages,
            georef.crs(),
            PointOn::Centroid,
            feature_tags,
        ),
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

/// One stored NAS layer for [`nas_objects`].
pub struct NasInput<'a> {
    /// The layer file (`sonstiges-bauwerk`).
    pub file: &'a str,
    /// The stored features, in the map's CRS.
    pub features: &'a [Value],
    /// The fetch date, which stands in for a feature without `beginnt`.
    pub fetched: NaiveDate,
}

/// The `freemapper:source` prefix of the NAS candidates: `alkis-nas/<layer file>`.
pub(crate) const NAS_SOURCE: &str = "alkis-nas";

/// The AdV code lists: type (`AX_…`) → code property → code → label.
type AdvCodes = BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>;

fn adv_codes() -> &'static AdvCodes {
    static CODES: OnceLock<AdvCodes> = OnceLock::new();
    CODES.get_or_init(|| {
        toml::from_str(include_str!("../../../../assets/rules/adv-codes.toml"))
            .expect("embedded AdV code lists are valid")
    })
}

/// The labels of the coded properties of a NAS feature (`Mauerkante, rechts`), `; `-joined in
/// the order of the code lists; `None` when no code has a label.
fn nas_label(nas_type: &str, properties: &serde_json::Map<String, Value>) -> Option<String> {
    let labels: Vec<&str> = adv_codes()
        .get(nas_type)?
        .iter()
        .filter_map(|(property, codes)| {
            let code = properties.get(property)?.as_str()?;
            codes.get(code).map(String::as_str)
        })
        .collect();
    (!labels.is_empty()).then(|| labels.join("; "))
}

/// The tags of a stored NAS feature for the rules: `alkis:type` (`AX_…`), `alkis:geom` and each
/// string property as `alkis:<name>` (the codes), and the provenance tags the objects carry on
/// (`freemapper:source_id` = `gml:id`, `freemapper:currency` = `beginnt`, the code labels as
/// `freemapper:alkis`).
pub(crate) fn nas_feature_tags(feature: &Value) -> Vec<(String, String)> {
    let empty = serde_json::Map::new();
    let properties = feature["properties"].as_object().unwrap_or(&empty);
    let text = |key: &str| {
        properties
            .get(key)
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
    };
    let mut tags = Vec::new();
    let nas_type = text("nas_type").unwrap_or_default();
    tags.push(("alkis:type".to_string(), nas_type.to_string()));
    if let Some(geom) = geom(feature) {
        tags.push(("alkis:geom".to_string(), geom.to_string()));
    }
    for (key, value) in properties {
        if key == "nas_type" {
            continue;
        }
        if let Some(value) = value.as_str().filter(|v| !v.is_empty()) {
            tags.push((format!("alkis:{key}"), value.to_string()));
        }
    }
    if let Some(oid) = text("oid") {
        tags.push(("freemapper:source_id".to_string(), oid.to_string()));
    }
    // A value that is no date leaves the fetch date (`osm_objects`).
    let date = text("beginnt")
        .and_then(|t| t.get(..10))
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    if let Some(date) = date {
        tags.push(("freemapper:currency".to_string(), date.to_string()));
    }
    let label = if text(PAIRED) == Some("yes") {
        Some(PAIRED_LABEL.to_string())
    } else {
        nas_label(nas_type, properties)
    };
    if let Some(label) = label {
        tags.push(("freemapper:alkis".to_string(), label));
    }
    tags.sort();
    tags
}

/// The candidates of the stored NAS `layers` (in the map's CRS) within `buffer_m` of
/// `map_area`, each tagged `freemapper:source=alkis-nas/<file>`. The point of an area (a
/// monument) lies on its surface. The two edges of a wall become one centre line
/// (`build/walls.rs`), then segments of one wall or fence are joined before the 1 mm drop
/// (`osm_objects`).
pub fn nas_objects(
    layers: &[NasInput<'_>],
    kind: MapKind,
    map_area: &geo_types::Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let rules = Rules::states();
    let distance = pair_distance_m(kind, georef.scale_denominator);
    let min_length = f64::from(georef.scale_denominator) / 1000.0 * MIN_LINE_MM;
    let mut objects = Vec::new();
    for layer in layers {
        let source = format!("{NAS_SOURCE}/{}", layer.file);
        // The edges of a wall become one centre line here, while each still says on which
        // side the wall lies; the joins below would lose that.
        let paired = pair_wall_edges(layer.features, distance, min_length);
        let features = tagged_features(
            &source,
            &paired,
            georef.crs(),
            PointOn::Surface,
            |feature| {
                let mut tags = nas_feature_tags(feature);
                let also = feature["properties"][PAIRED_ALSO]
                    .as_str()
                    .unwrap_or_default();
                for oid in also.split("; ").filter(|o| !o.is_empty()) {
                    add_also(&mut tags, &format!("{source}/{oid}"));
                }
                tags
            },
        );
        objects.extend(osm_objects(
            &features,
            &rules,
            Target::Candidates,
            kind,
            map_area,
            buffer_m,
            georef,
            symbol_set,
            &source,
            layer.fetched,
        )?);
    }
    Ok(objects)
}

/// One stored tree layer for [`tree_objects`].
pub struct TreeInput<'a> {
    /// The dataset (`potsdam`): the source is `trees/<dataset>`.
    pub dataset: &'a str,
    /// The stored features (`sources::trees`), in the map's CRS.
    pub features: &'a [Value],
    /// The fetch date, which stands in for a tree without `tree_date`.
    pub fetched: NaiveDate,
}

/// The `freemapper:source` prefix of the tree candidates: `trees/<dataset>`.
pub(crate) const TREE_SOURCE: &str = "trees";

/// The tree candidates of the stored `layers` within `buffer_m` of `map_area`.
pub fn tree_objects(
    layers: &[TreeInput<'_>],
    kind: MapKind,
    map_area: &geo_types::Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let rules = Rules::states();
    let mut objects = Vec::new();
    for layer in layers {
        let source = format!("{TREE_SOURCE}/{}", layer.dataset);
        let features = tagged_features(
            &source,
            layer.features,
            georef.crs(),
            PointOn::Centroid,
            tree_feature_tags,
        );
        objects.extend(osm_objects(
            &features,
            &rules,
            Target::Candidates,
            kind,
            map_area,
            buffer_m,
            georef,
            symbol_set,
            &source,
            layer.fetched,
        )?);
    }
    Ok(objects)
}

/// The tags of a stored tree for the rules: every `tree_<field>` property as `tree:<field>`
/// (`tree:dataset`, `tree:kind`, `tree:crown_m`, …), and the provenance tags the objects carry
/// on (`freemapper:source_id` = the tree number, `freemapper:currency` = the tree's own survey
/// date, `freemapper:baum` = species or kind, with the crown when known, `freemapper:crown_m` =
/// the crown diameter when known).
pub(crate) fn tree_feature_tags(feature: &Value) -> Vec<(String, String)> {
    let empty = serde_json::Map::new();
    let properties = feature["properties"].as_object().unwrap_or(&empty);
    let text = |key: &str| -> Option<String> {
        match properties.get(key)? {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    };
    let mut tags = Vec::new();
    for key in properties.keys() {
        if let (Some(field), Some(value)) = (key.strip_prefix("tree_"), text(key)) {
            tags.push((format!("tree:{field}"), value));
        }
    }
    if let Some(id) = text("tree_id") {
        tags.push(("freemapper:source_id".to_string(), id));
    }
    // A value that is no date leaves the fetch date (`osm_objects`).
    let date = text("tree_date").filter(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").is_ok());
    if let Some(date) = date {
        tags.push(("freemapper:currency".to_string(), date));
    }
    let crown_m = properties
        .get("tree_crown_m")
        .and_then(Value::as_f64)
        .filter(|c| *c > 0.0);
    // The crown diameter for the point generalization (`build/points.rs`).
    if let Some(crown) = crown_m {
        tags.push(("freemapper:crown_m".to_string(), crown.to_string()));
    }
    let crown = crown_m.map(|c| format!("Krone {c} m"));
    let label = match (text("tree_species").or_else(|| text("tree_kind")), crown) {
        (Some(name), Some(crown)) => Some(format!("{name}, {crown}")),
        (Some(name), None) => Some(name),
        (None, crown) => crown,
    };
    if let Some(label) = label {
        tags.push(("freemapper:baum".to_string(), label));
    }
    tags.sort();
    tags
}

#[cfg(test)]
pub(crate) mod test_support {
    use serde_json::Value;

    /// The stored form of the Mecklenburg-Vorpommern vereinfacht fixture.
    pub fn mv_features() -> Vec<Value> {
        let text = include_str!("../../../../tests/fixtures/mecklenburg-vorpommern-buildings.json");
        let collection: Value = serde_json::from_str(text).unwrap();
        collection["features"].as_array().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::canopy::{CanopyInput, canopy_objects};
    use crate::crs::{Crs, ProjectedPoint};
    use freemapper_sources::wfs::{parse_gml, test_support as gml};

    /// The stored form of a vereinfacht fixture: GeoJSON features in the state's CRS.
    fn gml_features(text: &str) -> Vec<Value> {
        parse_gml(text).unwrap()
    }

    use super::test_support::mv_features;

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
        (georef, area, kind.symbol_set(kind.default_scale()))
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
                let set = kind.symbol_set(kind.default_scale());
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
        let stored: Vec<Value> = freemapper_sources::layer_file::parse_layer(text)
            .unwrap()
            .iter()
            .map(freemapper_adapters_de::berlin::normalise_building)
            .collect();
        assert!(!stored.is_empty());
        assert!(passages(Crs::Utm33, MapKind::Sprint, &stored).is_empty());
        assert!(canopies(Crs::Utm33, &stored).is_empty());
    }

    #[test]
    fn passage_values_match_the_rule_file() {
        // `PASSAGES` is the spelling state.rs hands to the rule; the file must list the same.
        let rules = include_str!("../../../../assets/rules/state-candidates.toml");
        let line = rules
            .lines()
            .find(|l| l.contains("alkis:funktion="))
            .expect("the passage rule");
        let values = line
            .split("alkis:funktion=")
            .nth(1)
            .and_then(|v| v.split('"').next())
            .unwrap();
        assert_eq!(values.split('|').collect::<Vec<_>>(), PASSAGES);
    }

    // --- Full ALKIS (NAS) ---

    /// Sanssouci, Potsdam (EPSG:25833), where the NAS fixtures come from.
    const AT: (f64, f64) = (366_300.0, 5_807_200.0);

    /// A NAS feature as `alkis_nas` stores it: `oid`, `nas_type`, `beginnt` and the codes.
    fn nas(id: &str, nas_type: &str, codes: &[(&str, &str)], geometry: Value) -> Value {
        let mut properties = serde_json::json!({
            "oid": id, "nas_type": nas_type, "beginnt": "2013-01-29"});
        for (k, v) in codes {
            properties[*k] = (*v).into();
        }
        serde_json::json!({"type": "Feature", "id": id, "geometry": geometry,
            "properties": properties})
    }

    /// A line from (`de`, `dn`) metres off [`AT`], `length` m to the east.
    fn east(de: f64, dn: f64, length: f64) -> Value {
        let (e, n) = (AT.0 + de, AT.1 + dn);
        serde_json::json!({"type": "LineString", "coordinates": [[e, n], [e + length, n]]})
    }

    /// A `w` × `h` m square ring at (`de`, `dn`) off [`AT`]: as a polygon, or as a closed line.
    fn square(de: f64, dn: f64, w: f64, h: f64, polygon: bool) -> Value {
        let (e, n) = (AT.0 + de, AT.1 + dn);
        let ring = serde_json::json!([[e, n], [e + w, n], [e + w, n + h], [e, n + h], [e, n]]);
        if polygon {
            serde_json::json!({"type": "Polygon", "coordinates": [ring]})
        } else {
            serde_json::json!({"type": "LineString", "coordinates": ring})
        }
    }

    fn spot(de: f64, dn: f64) -> Value {
        serde_json::json!({"type": "Point", "coordinates": [AT.0 + de, AT.1 + dn]})
    }

    /// The NAS candidates of one stored layer, fetched 2026-11-09, on a 1:4000 map round them.
    fn nas_run(file: &str, features: &[Value], kind: MapKind) -> Vec<MapObject> {
        // A spot far off keeps the map area larger than one feature.
        let mut extent = features.to_vec();
        extent.push(nas("x", "AX_Turm", &[], spot(-40.0, -40.0)));
        extent.push(nas("y", "AX_Turm", &[], spot(80.0, 80.0)));
        let (georef, area, set) = setup(Crs::Utm33, kind, &extent);
        nas_objects(
            &[NasInput {
                file,
                features,
                fetched: "2026-11-09".parse().unwrap(),
            }],
            kind,
            &area,
            20.0,
            &georef,
            &set,
        )
        .unwrap()
    }

    fn code_of(kind: MapKind, object: &MapObject) -> String {
        let set = kind.symbol_set(kind.default_scale());
        let codes = [
            "104", "305", "306", "311", "410.1", "417", "418", "501.5", "513.2", "515", "518",
            "522", "526",
        ];
        codes
            .into_iter()
            .find(|code| set.id_for_code(code) == Some(object.symbol_id))
            .unwrap_or("other")
            .to_string()
    }

    fn label(nas_type: &str, property: &str, code: &str) -> String {
        let codes: toml::Table =
            toml::from_str(include_str!("../../../../assets/rules/adv-codes.toml")).unwrap();
        codes[nas_type][property][code]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn adv_codes_map_to_issprom() {
        const SB: &str = "AX_SonstigesBauwerkOderSonstigeEinrichtung";
        const VEG: &str = "AX_Vegetationsmerkmal";
        const UG: &str = "AX_UntergeordnetesGewaesser";
        #[derive(Clone, Copy)]
        enum G {
            Line,
            Area,
            Point,
        }
        // One case per row of the plan's table: layer file, type, code property, codes, more
        // codes, geometry, schoolyard and sprint symbol, drawn as a point.
        type Case<'a> = (
            &'a str,
            &'a str,
            &'a str,
            &'a [&'a str],
            &'a [(&'a str, &'a str)],
            G,
            Option<&'a str>,
            Option<&'a str>,
            bool,
        );
        let sb = "sonstiges-bauwerk";
        let cases: [Case; 16] = [
            (
                sb,
                SB,
                "bauwerksfunktion",
                &["1700", "1701", "1702", "1703"],
                &[],
                G::Line,
                Some("515"),
                Some("515"),
                false,
            ),
            (
                sb,
                SB,
                "bauwerksfunktion",
                &["1720", "1721", "1722", "1723"],
                &[],
                G::Line,
                Some("513.2"),
                Some("513.2"),
                false,
            ),
            (
                sb,
                SB,
                "bauwerksfunktion",
                &["1740"],
                &[],
                G::Line,
                Some("518"),
                Some("518"),
                false,
            ),
            // The Brandenburg monument is an area: the point lies on it.
            (
                sb,
                SB,
                "bauwerksfunktion",
                &["1750", "1760", "1761", "1762", "1763", "1770"],
                &[],
                G::Area,
                Some("526"),
                Some("526"),
                true,
            ),
            (
                sb,
                SB,
                "bauwerksfunktion",
                &["1780", "1781", "1782", "1783"],
                &[],
                G::Point,
                Some("311"),
                Some("311"),
                true,
            ),
            // Stairs and their bottom edges are `build/stairs.rs`'s, never a rule's.
            (
                sb,
                SB,
                "bauwerksfunktion",
                &["1620", "1621"],
                &[],
                G::Area,
                None,
                None,
                false,
            ),
            (
                sb,
                SB,
                "bauwerksfunktion",
                &["1630"],
                &[],
                G::Line,
                None,
                None,
                false,
            ),
            (
                "vegetationsmerkmal",
                VEG,
                "bewuchs",
                &["1100", "1101", "1102", "1103"],
                &[],
                G::Line,
                Some("410.1"),
                Some("410.1"),
                false,
            ),
            (
                "vegetationsmerkmal",
                VEG,
                "bewuchs",
                &["1011", "1012"],
                &[],
                G::Point,
                Some("418"),
                Some("418"),
                true,
            ),
            (
                "strukturlinie",
                "AX_Strukturlinie3D",
                "art",
                &["1220"],
                &[],
                G::Line,
                None,
                Some("104"),
                false,
            ),
            (
                "bauwerk-verkehr",
                "AX_BauwerkImVerkehrsbereich",
                "bauwerksfunktion",
                &[
                    "1800", "1801", "1802", "1803", "1804", "1805", "1806", "1807", "1808", "1820",
                ],
                &[],
                G::Area,
                None,
                Some("501.5"),
                false,
            ),
            (
                "bauwerk-gewaesser",
                "AX_BauwerkImGewaesserbereich",
                "bauwerksfunktion",
                &["2136"],
                &[],
                G::Line,
                Some("515"),
                Some("515"),
                false,
            ),
            (
                "untergeordnetes-gewaesser",
                UG,
                "funktion",
                &["1010", "1013"],
                &[],
                G::Line,
                Some("306"),
                Some("306"),
                false,
            ),
            (
                "untergeordnetes-gewaesser",
                UG,
                "funktion",
                &["1010", "1013"],
                &[("hydrologischesMerkmal", "2000")],
                G::Line,
                Some("306"),
                Some("306"),
                false,
            ),
            (
                "untergeordnetes-gewaesser",
                UG,
                "funktion",
                &["1030"],
                &[],
                G::Line,
                Some("305"),
                Some("305"),
                false,
            ),
            (
                "untergeordnetes-gewaesser",
                UG,
                "funktion",
                &["1030"],
                &[("hydrologischesMerkmal", "3000")],
                G::Line,
                Some("306"),
                Some("306"),
                false,
            ),
        ];
        for (file, nas_type, property, codes, more, g, schoolyard, sprint, as_point) in cases {
            for code in codes {
                let geometry = match g {
                    G::Line => east(0.0, 0.0, 20.0),
                    G::Area => square(0.0, 0.0, 6.0, 4.0, true),
                    G::Point => spot(0.0, 0.0),
                };
                let mut all = vec![(property, *code)];
                all.extend_from_slice(more);
                let feature = nas("DEBBAL0000000001", nas_type, &all, geometry);
                for (kind, expected) in
                    [(MapKind::Schoolyard, schoolyard), (MapKind::Sprint, sprint)]
                {
                    let found = nas_run(file, std::slice::from_ref(&feature), kind);
                    let case = format!("{nas_type} {code} {more:?} {kind:?}");
                    let Some(expected) = expected else {
                        assert!(found.is_empty(), "{case}: {found:?}");
                        continue;
                    };
                    assert_eq!(found.len(), 1, "{case}");
                    let o = &found[0];
                    assert_eq!(code_of(kind, o), expected, "{case}");
                    let point = matches!(o.kind, crate::omap::MapObjectKind::Point { .. });
                    assert_eq!(point, as_point, "{case}");
                    assert_eq!(
                        tag(o, "freemapper:alkis"),
                        Some(label(nas_type, property, code).as_str()),
                        "{case}"
                    );
                }
            }
        }
        // Everything else is skipped.
        for (file, nas_type, property, code) in [
            (sb, SB, "bauwerksfunktion", "1622"),
            (sb, SB, "bauwerksfunktion", "1640"),
            (sb, SB, "bauwerksfunktion", "1641"),
            (sb, SB, "bauwerksfunktion", "1642"),
            (sb, SB, "bauwerksfunktion", "1650"),
            (sb, SB, "bauwerksfunktion", "1670"),
            (sb, SB, "bauwerksfunktion", "1790"),
            (sb, SB, "bauwerksfunktion", "1791"),
            (sb, SB, "bauwerksfunktion", "9999"),
            ("strukturlinie", "AX_Strukturlinie3D", "art", "1230"),
            ("strukturlinie", "AX_Strukturlinie3D", "art", "1240"),
            (
                "bauwerk-gewaesser",
                "AX_BauwerkImGewaesserbereich",
                "bauwerksfunktion",
                "2050",
            ),
            ("untergeordnetes-gewaesser", UG, "funktion", "1011"),
        ] {
            for geometry in [east(0.0, 0.0, 20.0), square(0.0, 0.0, 6.0, 4.0, true)] {
                let feature = nas("DEBBAL0000000001", nas_type, &[(property, code)], geometry);
                for kind in [MapKind::Schoolyard, MapKind::Sprint] {
                    assert!(
                        nas_run(file, std::slice::from_ref(&feature), kind).is_empty(),
                        "{nas_type} {code}"
                    );
                }
            }
        }
        // The rules match codes, not labels: a label in the code property matches nothing.
        let by_label = nas(
            "DEBBAL0000000001",
            SB,
            &[("bauwerksfunktion", "Zaun")],
            east(0.0, 0.0, 20.0),
        );
        assert!(nas_run(sb, &[by_label], MapKind::Sprint).is_empty());
        // The vereinfacht passage row.
        let passage = feature("Bauteil", "Arkade", None, None);
        let found = passages(Crs::Utm33, MapKind::Schoolyard, &[passage]);
        assert_eq!(code_of(MapKind::Schoolyard, &found[0]), "522");
    }

    #[test]
    fn seasonal_brook_rule_comes_before_the_brook_rule() {
        // The first matching rule wins: a brook with `hydrologischesMerkmal` 2000/3000 must meet
        // its 306 rule before the plain brook rule (305).
        let file: toml::Table = toml::from_str(include_str!(
            "../../../../assets/rules/state-candidates.toml"
        ))
        .unwrap();
        let rules = file["rule"].as_array().unwrap();
        let brook = |seasonal: bool| {
            rules
                .iter()
                .position(|r| {
                    let when: Vec<&str> = r["when"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|c| c.as_str())
                        .collect();
                    when.contains(&"alkis:funktion=1030")
                        && when
                            .iter()
                            .any(|c| c.starts_with("alkis:hydrologischesMerkmal="))
                            == seasonal
                })
                .unwrap()
        };
        assert!(brook(true) < brook(false));
        assert_eq!(rules[brook(true)]["sprint"].as_str(), Some("306"));
        assert_eq!(rules[brook(false)]["sprint"].as_str(), Some("305"));
    }

    #[test]
    fn nas_canopies_and_building_parts_are_not_read() {
        use crate::sources::alkis_nas::{NAS_LAYERS, parse_nas_gml, test_support};
        assert!(NAS_LAYERS.iter().all(|l| l.type_name != "adv:AX_Bauteil"));
        let features = parse_nas_gml(test_support::SONSTIGES_BAUWERK)
            .unwrap()
            .features;
        let ids: Vec<&str> = features.iter().map(|f| f["id"].as_str().unwrap()).collect();
        assert!(
            ids.contains(&"DEBBAL540001vmqB"),
            "the canopy is in the fixture"
        );
        for kind in [MapKind::Schoolyard, MapKind::Sprint] {
            let found = nas_run("sonstiges-bauwerk", &features, kind);
            let mut found: Vec<&str> = found
                .iter()
                .map(|o| tag(o, "freemapper:source_id").unwrap())
                .collect();
            found.sort();
            found.dedup();
            assert!(!found.contains(&"DEBBAL540001vmqB"), "{kind:?}");
            assert_eq!(found.len(), 4, "{kind:?}: {found:?}");
        }
        // A NAS canopy or carport is never a candidate, in whatever shape.
        for code in ["1610", "1611"] {
            for geometry in [
                square(0.0, 0.0, 6.0, 4.0, true),
                east(0.0, 0.0, 20.0),
                spot(0.0, 0.0),
            ] {
                let canopy = nas(
                    "DEBBAL0000000002",
                    "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                    &[("bauwerksfunktion", code)],
                    geometry,
                );
                assert!(nas_run("sonstiges-bauwerk", &[canopy], MapKind::Sprint).is_empty());
            }
        }
    }

    #[test]
    fn area_rules_need_a_polygon() {
        let bridge = |polygon| {
            nas(
                "DEBBAL0000000003",
                "AX_BauwerkImVerkehrsbereich",
                &[("bauwerksfunktion", "1800")],
                square(0.0, 0.0, 12.0, 4.0, polygon),
            )
        };
        let found = nas_run("bauwerk-verkehr", &[bridge(true)], MapKind::Sprint);
        assert_eq!(found.len(), 1);
        assert_eq!(code_of(MapKind::Sprint, &found[0]), "501.5");
        assert_eq!(
            tag(&found[0], "alkis:geom"),
            None,
            "alkis:geom stays a rule tag"
        );
        // A closed line with a bridge code is no bridge deck.
        assert!(nas_run("bauwerk-verkehr", &[bridge(false)], MapKind::Sprint).is_empty());
        // Every area rule of the file names `alkis:geom=area`.
        let file: toml::Table = toml::from_str(include_str!(
            "../../../../assets/rules/state-candidates.toml"
        ))
        .unwrap();
        let rules = file["rule"].as_array().unwrap();
        let areas: Vec<_> = rules
            .iter()
            .filter(|r| r["draw"].as_str() == Some("area"))
            .collect();
        assert!(areas.len() >= 2);
        for rule in areas {
            let when = rule["when"].as_array().unwrap();
            assert!(
                when.iter().any(|c| c.as_str() == Some("alkis:geom=area")),
                "{when:?}"
            );
        }
    }

    #[test]
    fn nas_provenance_and_label() {
        use crate::sources::alkis_nas::{parse_nas_gml, test_support};
        let features = parse_nas_gml(test_support::SONSTIGES_BAUWERK)
            .unwrap()
            .features;
        let found = nas_run("sonstiges-bauwerk", &features, MapKind::Sprint);
        let wall = found
            .iter()
            .find(|o| tag(o, "freemapper:source_id") == Some("DEBBAL540001vq5u"))
            .expect("the 1701 wall edge");
        assert_eq!(
            tag(wall, "freemapper:source"),
            Some("alkis-nas/sonstiges-bauwerk")
        );
        assert_eq!(tag(wall, "freemapper:currency"), Some("2013-01-29"));
        assert_eq!(tag(wall, "freemapper:alkis"), Some("Mauerkante, rechts"));
        assert_eq!(
            tag(wall, "alkis:type"),
            Some("AX_SonstigesBauwerkOderSonstigeEinrichtung")
        );
        assert_eq!(code_of(MapKind::Sprint, wall), "515");
        // Without `beginnt` the fetch date stands in.
        let mut hedge = nas(
            "DEBBAL0000000004",
            "AX_Vegetationsmerkmal",
            &[("bewuchs", "1102")],
            east(0.0, 0.0, 20.0),
        );
        hedge["properties"]
            .as_object_mut()
            .unwrap()
            .remove("beginnt");
        let found = nas_run("vegetationsmerkmal", &[hedge], MapKind::Schoolyard);
        assert_eq!(tag(&found[0], "freemapper:currency"), Some("2026-11-09"));
        assert_eq!(
            tag(&found[0], "freemapper:source"),
            Some("alkis-nas/vegetationsmerkmal")
        );
        assert_eq!(
            tag(&found[0], "freemapper:alkis"),
            Some("Heckenkante, links")
        );
        // A monument area wider than 3 m is large; its point lies on the area.
        let monument = nas(
            "DEBBAL0000000005",
            "AX_SonstigesBauwerkOderSonstigeEinrichtung",
            &[("bauwerksfunktion", "1750")],
            square(0.0, 0.0, 5.0, 2.0, true),
        );
        let found = nas_run("sonstiges-bauwerk", &[monument], MapKind::Sprint);
        assert_eq!(tag(&found[0], "freemapper:size"), Some("large"));
    }

    #[test]
    fn nas_beginnt_that_is_no_date_gives_the_fetch_date() {
        for beginnt in [
            "2013-13-45T00:00:00Z",
            "unbekannt",
            "2013-01",
            "20130129T19:47:33Z",
        ] {
            let mut hedge = nas(
                "DEBBAL0000000007",
                "AX_Vegetationsmerkmal",
                &[("bewuchs", "1100")],
                east(0.0, 0.0, 20.0),
            );
            hedge["properties"]["beginnt"] = beginnt.into();
            let found = nas_run("vegetationsmerkmal", &[hedge], MapKind::Sprint);
            assert_eq!(
                tag(&found[0], "freemapper:currency"),
                Some("2026-11-09"),
                "{beginnt}"
            );
        }
    }

    #[test]
    fn nas_monument_point_lies_on_its_area() {
        // An L-shaped monument: its centroid is outside the area.
        let (e, n) = AT;
        let ring = serde_json::json!([
            [e, n],
            [e + 10.0, n],
            [e + 10.0, n + 1.0],
            [e + 1.0, n + 1.0],
            [e + 1.0, n + 10.0],
            [e, n + 10.0],
            [e, n]
        ]);
        let monument = nas(
            "DEBBAL0000000006",
            "AX_SonstigesBauwerkOderSonstigeEinrichtung",
            &[("bauwerksfunktion", "1750")],
            serde_json::json!({"type": "Polygon", "coordinates": [ring]}),
        );
        let (georef, area, set) =
            setup(Crs::Utm33, MapKind::Sprint, std::slice::from_ref(&monument));
        let found = nas_objects(
            &[NasInput {
                file: "sonstiges-bauwerk",
                features: &[monument],
                fetched: "2026-11-09".parse().unwrap(),
            }],
            MapKind::Sprint,
            &area,
            20.0,
            &georef,
            &set,
        )
        .unwrap();
        let p = georef.map_to_projected(found[0].coords[0]);
        let (x, y) = (p.easting - e, p.northing - n);
        assert!(
            (0.0..=10.0).contains(&x) && (0.0..=10.0).contains(&y) && (x <= 1.0 || y <= 1.0),
            "({x:.2}, {y:.2}) is off the L"
        );
    }

    #[test]
    fn nas_wall_edges_become_one_centre_line() {
        const SB: &str = "AX_SonstigesBauwerkOderSonstigeEinrichtung";
        let edge = |id: &str, code: &str, dn: f64, beginnt: &str| {
            let mut f = nas(id, SB, &[("bauwerksfunktion", code)], east(0.0, dn, 20.0));
            f["properties"]["beginnt"] = beginnt.into();
            f
        };
        // A wall 1 m thick: the right edge (wall on its left) below, the left edge above.
        let features = [
            edge("DEBBAL00000000r1", "1701", 0.0, "2015-03-01T10:00:00Z"),
            edge("DEBBAL00000000l1", "1702", 1.0, "2013-01-29T19:47:33Z"),
        ];
        let found = nas_run("sonstiges-bauwerk", &features, MapKind::Sprint);
        assert_eq!(found.len(), 1, "{found:?}");
        let wall = &found[0];
        assert_eq!(code_of(MapKind::Sprint, wall), "515");
        assert_eq!(tag(wall, "freemapper:source_id"), Some("DEBBAL00000000r1"));
        assert_eq!(
            tag(wall, "freemapper:also"),
            Some("alkis-nas/sonstiges-bauwerk/DEBBAL00000000l1")
        );
        assert_eq!(tag(wall, "freemapper:currency"), Some("2013-01-29"));
        assert_eq!(
            tag(wall, "freemapper:alkis"),
            Some("Mauer (Mauerkanten gepaart)")
        );
        // 2.5 m apart is more than 0.55 mm at 1:4000 (2.2 m): two lines.
        let features = [
            edge("DEBBAL00000000r1", "1701", 0.0, "2015-03-01T10:00:00Z"),
            edge("DEBBAL00000000l1", "1702", 2.5, "2013-01-29T19:47:33Z"),
        ];
        let found = nas_run("sonstiges-bauwerk", &features, MapKind::Sprint);
        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[test]
    fn nas_sided_lines_join_only_same_way_and_side() {
        const SB: &str = "AX_SonstigesBauwerkOderSonstigeEinrichtung";
        let piece = |id: &str, code: &str, from: (f64, f64), to: (f64, f64)| {
            let line = serde_json::json!({"type": "LineString", "coordinates": [
                [AT.0 + from.0, AT.1 + from.1], [AT.0 + to.0, AT.1 + to.1]]});
            nas(id, SB, &[("bauwerksfunktion", code)], line)
        };
        let ids = |found: &[MapObject]| -> Vec<String> {
            found
                .iter()
                .map(|o| {
                    tag(o, "freemapper:source_id")
                        .unwrap_or_default()
                        .to_string()
                })
                .collect()
        };
        // Retaining wall pieces (513.2, the ticks on one side): one runs the other way.
        let found = nas_run(
            "sonstiges-bauwerk",
            &[
                piece("a", "1721", (0.0, 0.0), (5.0, 0.0)),
                piece("b", "1721", (10.0, 0.0), (5.0, 0.0)),
            ],
            MapKind::Sprint,
        );
        assert_eq!(ids(&found), ["a", "b"]);
        // The same way round they join, whichever comes first.
        let found = nas_run(
            "sonstiges-bauwerk",
            &[
                piece("b", "1721", (5.0, 0.0), (10.0, 0.0)),
                piece("a", "1721", (0.0, 0.0), (5.0, 0.0)),
            ],
            MapKind::Sprint,
        );
        assert_eq!(ids(&found), ["b"]);
        let p = |c: &crate::omap::MapCoord| (c.x, c.y);
        let first = p(&found[0].coords[0]);
        let last = p(found[0].coords.last().unwrap());
        assert!(first.0 < last.0, "runs east as stored: {first:?} {last:?}");
        // A right and a left wall edge never join, even the same way round.
        let found = nas_run(
            "sonstiges-bauwerk",
            &[
                piece("r", "1701", (0.0, 20.0), (5.0, 20.0)),
                piece("l", "1702", (5.0, 20.0), (10.0, 20.0)),
            ],
            MapKind::Sprint,
        );
        assert_eq!(ids(&found), ["r", "l"]);
        // Fences have no side: a piece the other way round still joins.
        let found = nas_run(
            "sonstiges-bauwerk",
            &[
                piece("f", "1740", (0.0, 40.0), (5.0, 40.0)),
                piece("g", "1740", (10.0, 40.0), (5.0, 40.0)),
            ],
            MapKind::Sprint,
        );
        assert_eq!(ids(&found), ["f"]);
    }

    #[test]
    fn nas_fence_segments_are_joined() {
        // A fence cut into three 1.5 m segments (1 mm on paper at 1:4000 is 4 m) and a lone one.
        let fence = |id: &str, de: f64| {
            nas(
                id,
                "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                &[("bauwerksfunktion", "1740")],
                east(de, 0.0, 1.5),
            )
        };
        let features = [
            fence("DEBBAL00000000a1", 0.0),
            fence("DEBBAL00000000a2", 1.5),
            fence("DEBBAL00000000a3", 3.0),
            fence("DEBBAL00000000b1", 20.0),
        ];
        let found = nas_run("sonstiges-bauwerk", &features, MapKind::Sprint);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            tag(&found[0], "freemapper:source_id"),
            Some("DEBBAL00000000a1")
        );
        assert_eq!(
            tag(&found[0], "freemapper:also"),
            Some(
                "alkis-nas/sonstiges-bauwerk/DEBBAL00000000a2; \
                 alkis-nas/sonstiges-bauwerk/DEBBAL00000000a3"
            )
        );
        assert_eq!(found[0].coords.len(), 4);
    }
    /// The stored Potsdam fixture trees (the stump is not among them).
    fn potsdam_trees() -> Vec<Value> {
        use crate::sources::trees::{POTSDAM, parse_trees, test_support};
        parse_trees(test_support::POTSDAM, &POTSDAM, Crs::Utm33)
            .unwrap()
            .features
    }

    /// A stored tree `kind` at `AT` plus (`de`, `dn`), with an optional crown diameter.
    fn tree(id: &str, kind: &str, crown: Option<f64>, de: f64, dn: f64) -> Value {
        let mut properties = serde_json::json!({
            "tree_dataset": "potsdam", "tree_id": id, "tree_kind": kind, "tree_date": "2024-05-01"});
        if let Some(crown) = crown {
            properties["tree_crown_m"] = crown.into();
        }
        serde_json::json!({"type": "Feature", "id": id, "geometry": spot(de, dn),
            "properties": properties})
    }

    fn tree_run(features: &[Value], kind: MapKind) -> Vec<MapObject> {
        let mut extent = features.to_vec();
        extent.push(nas("x", "AX_Turm", &[], spot(-40.0, -40.0)));
        extent.push(nas("y", "AX_Turm", &[], spot(80.0, 80.0)));
        let (georef, area, set) = setup(Crs::Utm33, kind, &extent);
        tree_objects(
            &[TreeInput {
                dataset: "potsdam",
                features,
                fetched: "2026-11-09".parse().unwrap(),
            }],
            kind,
            &area,
            20.0,
            &georef,
            &set,
        )
        .unwrap()
    }

    #[test]
    fn potsdam_trees_are_418_candidates_with_their_own_date() {
        let trees = potsdam_trees();
        assert_eq!(trees.len(), 3);
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let found = tree_run(&trees, kind);
            assert_eq!(found.len(), 3, "{kind:?}");
            for o in &found {
                // No crown diameter in the data: every tree is the small one.
                assert_eq!(code_of(kind, o), "418");
                assert_eq!(tag(o, "freemapper:source"), Some("trees/potsdam"));
                assert_eq!(tag(o, "freemapper:currency"), Some("2009-06-10"));
                assert_eq!(tag(o, "freemapper:baum"), Some("Laubbaum"));
                assert_eq!(tag(o, "tree:dataset"), Some("potsdam"));
            }
            let mut ids: Vec<&str> = found
                .iter()
                .filter_map(|o| tag(o, "freemapper:source_id"))
                .collect();
            ids.sort_unstable();
            assert_eq!(ids, ["2005108M-305FD", "2005108M-30600", "2005108M-30603"]);
        }
    }

    #[test]
    fn crown_decides_417_and_stumps_stay_out() {
        let trees = [
            tree("a", "Laubbaum", Some(12.0), 0.0, 0.0),
            tree("b", "Laubbaum", Some(9.9), 5.0, 0.0),
            tree("c", "Nadelbaum", None, 10.0, 0.0),
            // A stump that reached the file (a dataset without the stored filter) draws nothing.
            tree("d", "Baumstumpf", Some(15.0), 15.0, 0.0),
        ];
        let found = tree_run(&trees, MapKind::Sprint);
        let codes: Vec<(String, &str)> = found
            .iter()
            .map(|o| {
                (
                    code_of(MapKind::Sprint, o),
                    tag(o, "freemapper:source_id").unwrap(),
                )
            })
            .collect();
        assert_eq!(
            codes,
            [
                ("417".into(), "a"),
                ("418".into(), "b"),
                ("418".into(), "c")
            ]
        );
        assert_eq!(
            tag(&found[0], "freemapper:baum"),
            Some("Laubbaum, Krone 12 m")
        );
        assert_eq!(tag(&found[0], "freemapper:crown_m"), Some("12"));
        assert_eq!(tag(&found[1], "freemapper:crown_m"), Some("9.9"));
        assert_eq!(tag(&found[2], "freemapper:crown_m"), None);
    }

    #[test]
    fn osm_tree_near_city_tree_is_dropped() {
        use crate::build::dedup::drop_trees_by_rank;
        let city = [tree("a", "Laubbaum", None, 0.0, 0.0)];
        let (georef, area, set) = setup(
            Crs::Utm33,
            MapKind::Sprint,
            &[city[0].clone(), nas("y", "AX_Turm", &[], spot(30.0, 30.0))],
        );
        let cadastre = tree_objects(
            &[TreeInput {
                dataset: "potsdam",
                features: &city,
                fetched: "2026-11-09".parse().unwrap(),
            }],
            MapKind::Sprint,
            &area,
            20.0,
            &georef,
            &set,
        )
        .unwrap();
        assert_eq!(cadastre.len(), 1);
        let osm = |osm: &str, de: f64, dn: f64| MapObject {
            symbol_id: set.id_for_code("418").unwrap(),
            kind: crate::omap::MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![
                georef
                    .projected_to_map(crate::crs::ProjectedPoint {
                        easting: AT.0 + de,
                        northing: AT.1 + dn,
                    })
                    .unwrap(),
            ],
            tags: vec![
                ("freemapper:source".into(), "osm".into()),
                ("freemapper:osm".into(), osm.into()),
            ],
        };
        let (kept, cadastre) = drop_trees_by_rank(
            vec![
                osm("natural=tree", 1.0, 1.0), // 1.4 m: the same tree
                osm("natural=tree", 0.0, 2.5), // another tree
            ],
            cadastre,
            &set,
            georef.scale_denominator,
        );
        assert_eq!(kept.len(), 1);
        assert_eq!(cadastre.len(), 1, "the city tree stays");
    }

    #[test]
    fn register_tree_beats_nas_tree() {
        use crate::build::dedup::drop_trees_by_rank;
        let city = [tree("a", "Laubbaum", Some(12.0), 0.0, 0.0)];
        let nas_trees = [
            nas(
                "DEBBAL0000000001",
                "AX_Vegetationsmerkmal",
                &[("bewuchs", "1012")],
                spot(1.0, 1.0),
            ),
            nas(
                "DEBBAL0000000002",
                "AX_Vegetationsmerkmal",
                &[("bewuchs", "1012")],
                spot(10.0, 0.0),
            ),
        ];
        let mut extent = city.to_vec();
        extent.extend(nas_trees.iter().cloned());
        extent.push(nas("y", "AX_Turm", &[], spot(30.0, 30.0)));
        let (georef, area, set) = setup(Crs::Utm33, MapKind::Sprint, &extent);
        let mut cadastre = tree_objects(
            &[TreeInput {
                dataset: "potsdam",
                features: &city,
                fetched: "2026-11-09".parse().unwrap(),
            }],
            MapKind::Sprint,
            &area,
            20.0,
            &georef,
            &set,
        )
        .unwrap();
        assert_eq!(cadastre.len(), 1);
        cadastre.extend(
            nas_objects(
                &[NasInput {
                    file: "vegetationsmerkmal",
                    features: &nas_trees,
                    fetched: "2026-11-09".parse().unwrap(),
                }],
                MapKind::Sprint,
                &area,
                20.0,
                &georef,
                &set,
            )
            .unwrap(),
        );
        assert_eq!(cadastre.len(), 3);
        let (_, kept) = drop_trees_by_rank(vec![], cadastre, &set, georef.scale_denominator);
        let sources: Vec<&str> = kept
            .iter()
            .map(|o| tag(o, "freemapper:source_id").unwrap())
            .collect();
        // The NAS tree 1.4 m from the city tree goes; the one 10 m away stays.
        assert_eq!(sources, ["a", "DEBBAL0000000002"]);
    }
}
