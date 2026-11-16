//! The candidate layers (ADR 0156): the candidates in seven template maps by the colour families
//! a mapper thinks in, so that OpenOrienteering Mapper switches each kind on and off on its own.
//!
//! The candidates stay one list in the build ([`crate::project::StarterDocuments`]); the check
//! print, the training map and the hidden-candidate rules see all of them. Only the files are cut
//! by layer when they are written, so every candidate is in exactly one layer and the layers
//! together are the candidates.

use crate::build::assembly::outline_symbol_id;
use crate::build::canopy::is_canopy;
use crate::build::content::{Classifier, Group};
use crate::omap::{MapObject, SymbolSet};

/// One candidate layer, a template map of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CandidateLayer {
    /// Slopes, landforms and contours.
    Terrain,
    /// Watercourses, springs and water areas.
    Water,
    /// Vegetation areas, trees and hedges.
    Vegetation,
    /// Paths, streets, stairs, passages and paved areas.
    Paths,
    /// Walls and fences.
    Barriers,
    /// Monuments, street furniture, buildings and canopies.
    Features,
    /// Areas that shall not be entered.
    OutOfBounds,
}

impl CandidateLayer {
    /// The layers in the order the map file lists them.
    pub const ALL: [CandidateLayer; 7] = [
        CandidateLayer::Terrain,
        CandidateLayer::Water,
        CandidateLayer::Vegetation,
        CandidateLayer::Paths,
        CandidateLayer::Barriers,
        CandidateLayer::Features,
        CandidateLayer::OutOfBounds,
    ];

    /// The name in the file name and in the layer list of the MCP server.
    pub fn name(self) -> &'static str {
        match self {
            CandidateLayer::Terrain => "terrain",
            CandidateLayer::Water => "water",
            CandidateLayer::Vegetation => "vegetation",
            CandidateLayer::Paths => "paths",
            CandidateLayer::Barriers => "barriers",
            CandidateLayer::Features => "features",
            CandidateLayer::OutOfBounds => "out-of-bounds",
        }
    }

    /// The template map of the layer in the project folder.
    pub fn relpath(self) -> &'static str {
        match self {
            CandidateLayer::Terrain => "templates/candidates-terrain.omap",
            CandidateLayer::Water => "templates/candidates-water.omap",
            CandidateLayer::Vegetation => "templates/candidates-vegetation.omap",
            CandidateLayer::Paths => "templates/candidates-paths.omap",
            CandidateLayer::Barriers => "templates/candidates-barriers.omap",
            CandidateLayer::Features => "templates/candidates-features.omap",
            CandidateLayer::OutOfBounds => "templates/candidates-out-of-bounds.omap",
        }
    }

    /// What the layer holds, for the layer list of the MCP server.
    pub fn description(self) -> &'static str {
        match self {
            CandidateLayer::Terrain => "candidates to check: banks, cliffs, landforms, contours",
            CandidateLayer::Water => "candidates to check: brooks, ditches, springs, water areas",
            CandidateLayer::Vegetation => {
                "candidates to check: vegetation areas, open land, trees, hedges"
            }
            CandidateLayer::Paths => {
                "candidates to check: paths, streets, paved areas, stairs, passages"
            }
            CandidateLayer::Barriers => "candidates to check: walls, fences",
            CandidateLayer::Features => {
                "candidates to check: towers, monuments, street furniture, buildings, canopies"
            }
            CandidateLayer::OutOfBounds => "candidates to check: areas that shall not be entered",
        }
    }

    /// The layer of an object group.
    pub fn of_group(group: Group) -> CandidateLayer {
        match group {
            Group::Slopes | Group::Landforms | Group::Contours => CandidateLayer::Terrain,
            Group::WaterFeatures => CandidateLayer::Water,
            Group::Vegetation | Group::Trees | Group::Hedges => CandidateLayer::Vegetation,
            Group::Paths | Group::Streets | Group::Stairs | Group::Passages => {
                CandidateLayer::Paths
            }
            Group::Walls | Group::Fences => CandidateLayer::Barriers,
            Group::Monuments | Group::Furniture => CandidateLayer::Features,
            Group::OutOfBounds => CandidateLayer::OutOfBounds,
        }
    }

    /// The layer of a symbol code, for a candidate of no group: landforms (1xx) and rock (2xx) to
    /// terrain, water (3xx), vegetation (4xx), and of the man-made features (5xx) the ways, bridges
    /// and stairs (501 to 509, 512, 532) to paths, the walls and fences (513 to 519) to barriers,
    /// 520 to out-of-bounds and the rest to features, as are the technical and course symbols
    /// (6xx to 9xx). `None` for a code that is no number.
    pub fn of_code(code: &str) -> Option<CandidateLayer> {
        let number: u32 = code.split('.').next()?.parse().ok()?;
        Some(match number {
            100..=299 => CandidateLayer::Terrain,
            300..=399 => CandidateLayer::Water,
            400..=499 => CandidateLayer::Vegetation,
            501..=509 | 512 | 532 => CandidateLayer::Paths,
            513..=519 => CandidateLayer::Barriers,
            520 => CandidateLayer::OutOfBounds,
            500..=999 => CandidateLayer::Features,
            _ => return None,
        })
    }
}

/// The layer `o` goes to. A canopy goes to features (a passage, the other 522, belongs to the
/// paths), and every object of a vegetation code (4xx) to vegetation whatever its group: the
/// layers follow the colour families, so the green strips of a street survey, of the group
/// streets, are vegetation. Else the layer of its group, else that of its symbol code; a symbol
/// of no code goes to features.
fn layer_of(o: &MapObject, classifier: &Classifier, symbol_set: &SymbolSet) -> CandidateLayer {
    let code = symbol_set.code_for_id(o.symbol_id);
    if code == Some("522") && is_canopy(o) {
        return CandidateLayer::Features;
    }
    let by_code = code.and_then(CandidateLayer::of_code);
    if by_code == Some(CandidateLayer::Vegetation) {
        return CandidateLayer::Vegetation;
    }
    classifier
        .group(o)
        .map(CandidateLayer::of_group)
        .or(by_code)
        .unwrap_or(CandidateLayer::Features)
}

/// Whether `o` is the outline of the map area ([`crate::build::assembly::map_area_outline`]): a
/// line of the outline symbol from OpenStreetMap. A 501.1 of the cadastre, the street survey or
/// the paved borders has another source.
fn is_outline(o: &MapObject, outline_id: Option<u32>) -> bool {
    Some(o.symbol_id) == outline_id
        && o.tags
            .iter()
            .any(|(k, v)| k == "freemapper:source" && v == "osm")
}

/// The candidates by layer, every layer of [`CandidateLayer::ALL`] in its order, an empty one too.
/// The outline of the map area comes first in every layer, so that each layer shows the area on
/// its own; every other candidate is in exactly one layer, in the order of `candidates`.
pub fn split(
    candidates: &[MapObject],
    symbol_set: &SymbolSet,
) -> Vec<(CandidateLayer, Vec<MapObject>)> {
    let classifier = Classifier::new(symbol_set);
    let outline_id = outline_symbol_id(symbol_set);
    let (outlines, others): (Vec<&MapObject>, Vec<&MapObject>) =
        candidates.iter().partition(|o| is_outline(o, outline_id));
    let mut layers: Vec<(CandidateLayer, Vec<MapObject>)> = CandidateLayer::ALL
        .iter()
        .map(|&layer| (layer, outlines.iter().map(|o| (*o).clone()).collect()))
        .collect();
    for object in others {
        let layer = layer_of(object, &classifier, symbol_set);
        if let Some((_, objects)) = layers.iter_mut().find(|(l, _)| *l == layer) {
            objects.push(object.clone());
        }
    }
    layers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::content::LANDFORM_KEY;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::omap::MapObjectKind;
    use crate::project::MapKind;

    fn object(f: &Fixture, code: &str, tags: &[(&str, &str)]) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: vec![f.at((0.0, 0.0)), f.at((10.0, 0.0))],
            tags: tags
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    fn layer(f: &Fixture, o: &MapObject) -> CandidateLayer {
        layer_of(o, &Classifier::new(&f.set), &f.set)
    }

    #[test]
    fn every_code_of_every_symbol_set_has_a_layer() {
        for (kind, scales) in [
            (MapKind::Schoolyard, &[1000][..]),
            (MapKind::Sprint, &[4000, 5000][..]),
            (MapKind::Forest, &[15000, 10000, 7500][..]),
        ] {
            for &scale in scales {
                let set = kind.symbol_set(scale);
                let mut checked = 0;
                for id in 0..=set.max_id() {
                    if let Some(code) = set.code_for_id(id) {
                        assert!(
                            CandidateLayer::of_code(code).is_some(),
                            "{kind:?} 1:{scale}: {code} has no layer"
                        );
                        checked += 1;
                    }
                }
                assert!(checked > 100, "{kind:?}: {checked} codes");
            }
        }
    }

    #[test]
    fn every_group_has_the_layer_of_the_spec() {
        use CandidateLayer::*;
        let expected = [
            (Group::Slopes, Terrain),
            (Group::Landforms, Terrain),
            (Group::Contours, Terrain),
            (Group::WaterFeatures, Water),
            (Group::Vegetation, Vegetation),
            (Group::Trees, Vegetation),
            (Group::Hedges, Vegetation),
            (Group::Paths, Paths),
            (Group::Streets, Paths),
            (Group::Stairs, Paths),
            (Group::Passages, Paths),
            (Group::Walls, Barriers),
            (Group::Fences, Barriers),
            (Group::Monuments, Features),
            (Group::Furniture, Features),
            (Group::OutOfBounds, OutOfBounds),
        ];
        assert_eq!(expected.len(), Group::ALL.len());
        for (group, layer) in expected {
            assert_eq!(CandidateLayer::of_group(group), layer, "{group:?}");
        }
    }

    #[test]
    fn files_and_names_follow_the_table() {
        let files: Vec<&str> = CandidateLayer::ALL.iter().map(|l| l.relpath()).collect();
        assert_eq!(
            files,
            [
                "templates/candidates-terrain.omap",
                "templates/candidates-water.omap",
                "templates/candidates-vegetation.omap",
                "templates/candidates-paths.omap",
                "templates/candidates-barriers.omap",
                "templates/candidates-features.omap",
                "templates/candidates-out-of-bounds.omap",
            ]
        );
        for layer in CandidateLayer::ALL {
            assert_eq!(
                layer.relpath(),
                format!("templates/candidates-{}.omap", layer.name())
            );
        }
    }

    #[test]
    fn a_candidate_of_each_group_lands_in_its_layer() {
        let f = fixture(MapKind::Sprint);
        let osm = [("freemapper:source", "osm")];
        for (code, expected) in [
            ("104", CandidateLayer::Terrain),
            ("305", CandidateLayer::Water),
            ("410.1", CandidateLayer::Vegetation),
            ("505.1", CandidateLayer::Paths),
            ("518", CandidateLayer::Barriers),
            ("531", CandidateLayer::Features),
            ("520", CandidateLayer::OutOfBounds),
        ] {
            assert_eq!(layer(&f, &object(&f, code, &osm)), expected, "{code}");
        }
        // A bank of the terrain model is a landform whatever its symbol.
        let bank = object(&f, "513.2", &[(LANDFORM_KEY, "retaining-wall")]);
        assert_eq!(layer(&f, &bank), CandidateLayer::Terrain);
        // A hidden candidate keeps its layer.
        let hidden = object(&f, "531", &[("freemapper:hidden", "out-of-bounds")]);
        assert_eq!(layer(&f, &hidden), CandidateLayer::Features);
    }

    #[test]
    fn a_candidate_of_no_group_lands_by_its_code() {
        let f = fixture(MapKind::Sprint);
        let osm = [("freemapper:source", "osm")];
        for (code, expected) in [
            ("401", CandidateLayer::Vegetation),
            ("501", CandidateLayer::Paths),
            ("521", CandidateLayer::Features),
            ("301", CandidateLayer::Water),
            ("501.1", CandidateLayer::Paths),
        ] {
            assert_eq!(layer(&f, &object(&f, code, &osm)), expected, "{code}");
        }
        let carport = object(&f, "522", &[("freemapper:alkis", "Carport")]);
        assert_eq!(layer(&f, &carport), CandidateLayer::Features);
        let roof = object(&f, "522", &[("freemapper:alkis", "Überdachung")]);
        assert_eq!(layer(&f, &roof), CandidateLayer::Features);
        let passage = object(&f, "522", &[("freemapper:osm", "tunnel=building_passage")]);
        assert_eq!(layer(&f, &passage), CandidateLayer::Paths);
    }

    #[test]
    fn a_vegetation_code_goes_to_vegetation_whatever_its_group() {
        let f = fixture(MapKind::Sprint);
        // A green strip of the Berlin street survey: open land of the group streets.
        let strip = object(
            &f,
            "401",
            &[("freemapper:source", "geoportal-berlin/sb-gruenflaeche")],
        );
        assert_eq!(Classifier::new(&f.set).group(&strip), Some(Group::Streets));
        assert_eq!(layer(&f, &strip), CandidateLayer::Vegetation);
        // A street area of the same survey stays with the paths.
        let street = object(
            &f,
            "501.5",
            &[("freemapper:source", "geoportal-berlin/sb-flaechen")],
        );
        assert_eq!(layer(&f, &street), CandidateLayer::Paths);
    }

    #[test]
    fn the_outline_is_found_by_its_symbol_and_source_wherever_it_lies() {
        let f = fixture(MapKind::Sprint);
        let outline = object(
            &f,
            "501.1",
            &[
                ("freemapper:source", "osm"),
                ("freemapper:source_id", "way/1"),
            ],
        );
        let pond = object(&f, "301", &[("freemapper:source", "osm")]);
        let border = object(&f, "501.1", &[("freemapper:source", "generalization")]);
        let layers = split(&[pond.clone(), outline.clone(), border.clone()], &f.set);
        for (layer, objects) in &layers {
            assert_eq!(objects.first(), Some(&outline), "{layer:?}");
            assert_eq!(objects.iter().filter(|o| **o == outline).count(), 1);
        }
        assert_eq!(layers[1].1, [outline.clone(), pond], "water");
        assert_eq!(
            layers[3].1,
            [outline, border],
            "paths: a border is no outline"
        );
        // The forest map draws the outline with the sheet symbol 994.
        let forest = fixture(MapKind::Forest);
        let outline = MapObject {
            symbol_id: outline_symbol_id(&forest.set).unwrap(),
            ..object(&forest, "101", &[("freemapper:source", "osm")])
        };
        let contour = object(&forest, "101", &[("freemapper:source", "lgb-dgm1")]);
        let layers = split(&[contour.clone(), outline.clone()], &forest.set);
        assert!(layers.iter().all(|(_, o)| o.first() == Some(&outline)));
        assert_eq!(layers[0].1, [outline, contour]);
    }

    #[test]
    fn the_forest_map_puts_its_isom_codes_in_layers() {
        let f = fixture(MapKind::Forest);
        let osm = [("freemapper:source", "osm")];
        for (code, expected) in [
            ("101", CandidateLayer::Terrain),
            ("201", CandidateLayer::Terrain),
            ("406", CandidateLayer::Vegetation),
            ("503", CandidateLayer::Paths),
            ("520", CandidateLayer::OutOfBounds),
        ] {
            assert_eq!(layer(&f, &object(&f, code, &osm)), expected, "{code}");
        }
    }

    #[test]
    fn the_layers_together_are_the_candidates_and_each_starts_with_the_outline() {
        let f = fixture(MapKind::Sprint);
        let outline = object(
            &f,
            "501.1",
            &[
                ("freemapper:source", "osm"),
                ("freemapper:source_id", "way/1"),
            ],
        );
        let mut candidates = vec![outline.clone()];
        for (i, code) in [
            "104", "305", "410.1", "505.1", "518", "531", "520", "401", "531",
        ]
        .iter()
        .enumerate()
        {
            let id = format!("node/{i}");
            candidates.push(object(&f, code, &[("freemapper:source_id", id.as_str())]));
        }
        let layers = split(&candidates, &f.set);
        let names: Vec<CandidateLayer> = layers.iter().map(|(l, _)| *l).collect();
        assert_eq!(names, CandidateLayer::ALL);
        for (layer, objects) in &layers {
            assert_eq!(objects.first(), Some(&outline), "{layer:?}");
        }
        let mut union: Vec<&MapObject> = layers
            .iter()
            .flat_map(|(_, objects)| objects.iter().skip(1))
            .collect();
        assert_eq!(union.len(), candidates.len() - 1);
        for object in &candidates[1..] {
            let at = union.iter().position(|o| *o == object).unwrap();
            union.remove(at);
        }
        assert!(union.is_empty());
        let features = &layers[5].1;
        assert_eq!(features.len(), 3, "the outline and two 531");
    }

    #[test]
    fn every_layer_is_there_without_candidates() {
        let f = fixture(MapKind::Schoolyard);
        let layers = split(&[], &f.set);
        assert_eq!(layers.len(), 7);
        assert!(layers.iter().all(|(_, objects)| objects.is_empty()));
    }
}
