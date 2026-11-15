//! The layers of the sets of extra layers that bring their rules (ADR 0149), drawn without
//! knowing the region: each feature is tagged `<prefix>:<property>` and `<prefix>:layer=<file>`
//! and drawn through the set's rules, with the source `<set key>/<file>`. The stair layers of a
//! set are left to `build/stairs.rs`, which takes their areas through [`stair_areas`].

use freemapper_adapter_api::extras::{SetDrawing, SetRank};
use geo_types::Polygon;
use serde_json::Value;

use crate::build::assembly::Layer;
use crate::build::candidates::{Rules, Target, osm_objects};
use crate::build::content::CANDIDATE_ONLY_KEY;
use crate::build::tagged;
use crate::crs::Crs;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, SymbolSet};
use crate::project::MapKind;

/// The drawings of the sets, by the prefix of their sources, from the registry of the run: the
/// sets the engine draws through them and those it still draws by its own code (Berlin).
fn drawings() -> &'static [(&'static str, &'static SetDrawing)] {
    static DRAWINGS: std::sync::OnceLock<Vec<(&'static str, &'static SetDrawing)>> =
        std::sync::OnceLock::new();
    DRAWINGS.get_or_init(|| {
        crate::adapters::registry()
            .adapters()
            .flat_map(|adapter| adapter.extras().iter())
            .filter_map(|set| set.drawing())
            .map(|drawing| (drawing.source_prefix, drawing))
            .collect()
    })
}

/// The drawing of the set whose objects carry `source` (`<source prefix>/<layer>`); `None` for
/// the objects of every other source.
pub(crate) fn drawing_of_source(source: &str) -> Option<&'static SetDrawing> {
    let prefix = source.split('/').next()?;
    drawings()
        .iter()
        .find(|(own, _)| *own == prefix)
        .map(|(_, drawing)| *drawing)
}

/// The rank of the objects with `source` in the duplicate step, when a set draws them: the rank
/// its drawing gives their layer.
pub(crate) fn rank_of_source(source: &str) -> Option<crate::build::dedup::Rank> {
    use crate::build::dedup::Rank;
    let drawing = drawing_of_source(source)?;
    let layer = source.split_once('/').map_or("", |(_, layer)| layer);
    Some(match drawing.rank_of(layer) {
        SetRank::Survey => Rank::Survey,
        SetRank::Cadastre => Rank::Alkis,
        SetRank::Register => Rank::Register,
    })
}

/// The drawings of all sets, in the order of the registry.
pub(crate) fn all_drawings() -> impl Iterator<Item = &'static SetDrawing> {
    drawings().iter().map(|(_, drawing)| *drawing)
}

/// The point dedups of the sets, set by set in the order of the registry.
pub(crate) fn point_dedups()
-> impl Iterator<Item = &'static freemapper_adapter_api::extras::PointDedup> {
    drawings()
        .iter()
        .flat_map(|(_, drawing)| drawing.point_dedups.iter())
}

/// The layers the sets name in the credit line, with the prefix of their sources: (prefix, file).
pub(crate) fn credited_layers() -> impl Iterator<Item = (&'static str, &'static str)> {
    drawings().iter().flat_map(|(prefix, drawing)| {
        drawing
            .credited_layers
            .iter()
            .map(move |(file, _)| (*prefix, *file))
    })
}

/// The stored layers of one set that brings its rules.
pub struct SetLayers {
    pub key: &'static str,
    pub drawing: &'static SetDrawing,
    /// The CRS the stored features are in.
    pub crs: Crs,
    pub layers: Vec<Layer>,
}

impl SetLayers {
    /// The source of the objects of `layer`.
    pub fn source(&self, layer: &str) -> String {
        format!("{}/{layer}", self.drawing.source_prefix)
    }

    /// The layers whose areas are stairs.
    pub fn stair_layers(&self) -> impl Iterator<Item = &Layer> {
        self.layers
            .iter()
            .filter(|l| self.drawing.stair_layers.contains(&l.name.as_str()))
    }
}

/// The objects of the layers of `set` other than its stair layers, through the set's rules,
/// within `buffer_m` of `map_area` (map CRS). A set whose rules do not parse draws nothing; a test
/// keeps the built-in ones parsing.
pub fn set_objects(
    set: &SetLayers,
    kind: MapKind,
    map_area: &Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let Ok(rules) = Rules::from_toml(set.drawing.rules) else {
        return Ok(Vec::new());
    };
    let mut objects = Vec::new();
    for layer in &set.layers {
        if set.drawing.stair_layers.contains(&layer.name.as_str()) {
            continue;
        }
        let source = set.source(&layer.name);
        let tagged = tagged::tagged_features(
            &source,
            &layer.features,
            set.crs,
            tagged::PointOn::Centroid,
            |feature| feature_tags(set.drawing.tag_prefix, &layer.name, feature),
        );
        let mut drawn = osm_objects(
            &tagged,
            &rules,
            Target::Candidates,
            kind,
            map_area,
            buffer_m,
            georef,
            symbol_set,
            &source,
            layer.fetched,
        )?;
        if set.drawing.candidate_layers.contains(&layer.name.as_str()) {
            for object in &mut drawn {
                object
                    .tags
                    .push((CANDIDATE_ONLY_KEY.to_string(), "yes".to_string()));
            }
        }
        objects.extend(drawn);
    }
    Ok(objects)
}

/// `<prefix>:layer=<layer>` and `<prefix>:<property>` for every text and number property.
pub(crate) fn feature_tags(prefix: &str, layer: &str, feature: &Value) -> Vec<(String, String)> {
    let mut tags = vec![(format!("{prefix}:layer"), layer.to_string())];
    for (key, value) in feature["properties"].as_object().into_iter().flatten() {
        let text = match value {
            Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
            Value::Number(n) => n.to_string(),
            _ => continue,
        };
        tags.push((format!("{prefix}:{key}"), text));
    }
    tags.sort();
    tags
}

/// The polygons of an area feature in `from`, in `to`, without their holes: a stair is its outline.
pub(crate) fn stair_areas(feature: &Value, from: Crs, to: Crs) -> Vec<Polygon<f64>> {
    tagged::polygons_in(feature, from, to)
        .into_iter()
        .map(|p| Polygon::new(p.exterior().clone(), Vec::new()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_feature_is_tagged_with_the_prefix_of_its_set() {
        let feature = json!({"properties": {"inhalt": "Geländer", "objectid": 7, "leer": " ", "nichts": null}});
        assert_eq!(
            feature_tags("hamburg", "fk-strasse-linien", &feature),
            [
                ("hamburg:inhalt".to_string(), "Geländer".to_string()),
                ("hamburg:layer".to_string(), "fk-strasse-linien".to_string()),
                ("hamburg:objectid".to_string(), "7".to_string()),
            ]
        );
    }

    #[test]
    fn the_rules_of_every_built_in_set_parse_and_draw_known_symbols() {
        let registry = crate::adapters::registry();
        let mut checked = 0;
        for adapter in registry.adapters() {
            for set in adapter.extras() {
                // A set the engine still draws by its own code brings no rules yet (Berlin).
                let Some(drawing) = set.drawing().filter(|d| d.generic) else {
                    continue;
                };
                let rules = Rules::from_toml(drawing.rules)
                    .unwrap_or_else(|e| panic!("{}: {e:?}", set.info().key));
                // The sets draw on sprint and schoolyard maps.
                for kind in [MapKind::Sprint, MapKind::Schoolyard] {
                    let symbols = kind.symbol_set(4000);
                    for code in rules.codes(kind) {
                        assert!(
                            symbols.id_for_code(&code).is_some(),
                            "{}: {code} on a {kind:?} map",
                            set.info().key
                        );
                    }
                }
                checked += 1;
            }
        }
        assert!(checked >= 1);
    }

    /// The rank of a set's objects comes from the data of its drawing: Berlin's survey, registers
    /// and cadastre layers, and Hamburg's survey (ADR 0149).
    #[test]
    fn a_source_ranks_by_the_data_of_its_set() {
        use crate::build::dedup::Rank;
        assert_eq!(
            rank_of_source("geoportal-berlin/sb-gelaender"),
            Some(Rank::Survey)
        );
        assert_eq!(
            rank_of_source("geoportal-berlin/beleuchtung"),
            Some(Rank::Register)
        );
        assert_eq!(
            rank_of_source("geoportal-berlin/strassenbaeume"),
            Some(Rank::Register)
        );
        assert_eq!(
            rank_of_source("geoportal-berlin/bauwerke-linien"),
            Some(Rank::Alkis)
        );
        assert_eq!(
            rank_of_source("lgv-feinkartierung/fk-treppen"),
            Some(Rank::Survey)
        );
        assert_eq!(rank_of_source("osm"), None);
    }

    /// The registers a set names in its credit line, by the file they are stored in.
    #[test]
    fn berlin_names_the_registers_of_its_credit_line() {
        assert_eq!(
            credited_layers().collect::<Vec<_>>(),
            [
                ("geoportal-berlin", "beleuchtung"),
                ("geoportal-berlin", "zierbrunnen"),
                ("geoportal-berlin", "trinkwasserbrunnen"),
            ]
        );
    }

    /// Which of Berlin's points beat which others is data of its set, in order: the registers
    /// first, then the survey (ADR 0149).
    #[test]
    fn berlin_brings_its_point_dedups() {
        let winners: Vec<(&str, &str)> = point_dedups().map(|d| d.winner).collect();
        assert_eq!(winners.len(), 13, "{winners:?}");
        assert_eq!(
            winners[0],
            ("freemapper:source", "geoportal-berlin/beleuchtung")
        );
        assert_eq!(winners[12], ("berlin:layer", "aw_spielgeraet"));
    }

    /// Berlin's control-site kinds, the tags its candidates keep, its fountains and the bridge
    /// decks that may win a duplicate are data of its set (ADR 0149).
    #[test]
    fn berlin_brings_its_control_kinds_kept_tags_fountains_and_bridges() {
        let berlin = drawing_of_source("geoportal-berlin/sb-moebel").unwrap();
        assert!(berlin.control_kinds.contains(&("av_poller", "bollard")));
        assert!(
            berlin
                .control_kinds
                .contains(&("beleuchtung", "street_lamp"))
        );
        assert!(berlin.kept_tags.contains(&"berlin:material"));
        assert!(
            berlin
                .fountain_tags
                .contains(&("berlin:layer", "bv_springbrunnen_zierbrunnen"))
        );
        assert_eq!(
            berlin.winning_areas,
            [("berlin:layer", "bauwerke-flaechen")]
        );
    }
}
