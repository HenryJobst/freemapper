//! The map assembly (ADR 0115): the objects of all sources, put together to the map and the
//! candidates in a fixed order of steps.
//!
//! The assembly reads no file. It gets what the sources drew ([`Drawn`]), resolves the
//! duplicates between them and generalizes the points ([`resolve`]). It gets the objects as the
//! build made them ([`Built`]) and places them by the content settings ([`Built::place`],
//! ADR 0100): `route` sends each object to the map, to the candidates or leaves it out, and the
//! steps that look at both lists follow. The project (`project/export.rs`) reads the stored
//! data, calls the assembly and makes the documents from what it returns.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use geo_types::{MultiPolygon, Polygon};

use crate::build::barrier_gaps::move_points_off_barriers;
use crate::build::barriers::resolve_barrier_overlaps;
use crate::build::berlin::drop_shelters_under_canopies;
use crate::build::content::{ContentSettings, route};
use crate::build::dedup::{
    drop_areas_by_rank, drop_cadastre_lines_by_rank, drop_osm_lines_near_cadastre,
    drop_points_near_registers, drop_trees_by_rank,
};
use crate::build::firm_paths::cut_firm_paths;
use crate::build::line_cuts::{CONTOUR_CODES, NORTH_LINE_CODES, cut_lines_at_ways};
use crate::build::open_land::{OpenLandObjects, drop_open_land_under_alkis};
use crate::build::passages::hide_unreachable as hide_unreachable_passages;
use crate::build::paved::PavedObjects;
use crate::build::paved_borders::{border_input, paved_borders as paved_borders_of};
use crate::build::points::{GeneralizedPoints, generalize_points, way_footprints};
use crate::build::ticks::orient_ticks;
use crate::build::vegetation::cut_vegetation_at_out_of_bounds;
use crate::build::water::{drop_covered_water, drop_fountains_in_osm_basins, drop_points_in_water};
use crate::crs::BoundingBox;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, SymbolSet};
use crate::project::MapKind;
use crate::raster::Raster;

/// What the steps of the assembly look at besides the objects: the kind and the frame of the
/// map, the buildings, the drawn water, the clip window and the terrain.
pub struct Surroundings {
    pub map_kind: MapKind,
    pub georef: Georeferencing,
    pub symbol_set: SymbolSet,
    pub footprints: Vec<Polygon<f64>>,
    pub water: MultiPolygon<f64>,
    pub clip: BoundingBox,
    pub terrain: Option<Raster<f32>>,
}

/// What the sources drew, before the duplicates between them are resolved ([`resolve`]).
pub struct Drawn {
    /// The OSM objects of the candidates rules and of the map rules (the firm-gravel ways).
    pub osm_candidates: Vec<MapObject>,
    pub osm_map: Vec<MapObject>,
    /// The candidates of the cadastres, the registers and the survey, in the order of their
    /// sources: Berlin first, then the state passages, the full ALKIS (NAS) and the tree
    /// cadastres.
    pub cadastre: Vec<MapObject>,
    /// The 522 canopies.
    pub canopies: Vec<MapObject>,
    /// The areas that shall not be entered (520) of the land use.
    pub out_of_bounds: Vec<MapObject>,
    /// The paved areas and the open land of the land use; they are only looked at.
    pub paved: PavedObjects,
    pub open_land: OpenLandObjects,
}

/// What [`resolve`] leaves of the [`Drawn`] objects.
pub struct Resolved {
    /// The candidates of OSM (first) and of the cadastres, the group areas and the OSM map
    /// objects, after the point generalization.
    pub points: GeneralizedPoints,
    pub canopies: Vec<MapObject>,
    pub out_of_bounds: Vec<MapObject>,
    pub paved: PavedObjects,
    pub open_land: OpenLandObjects,
}

/// Resolves the duplicates between the sources and generalizes the points.
///
/// Duplicates go by rank (`build/dedup.rs`): register > ALKIS > survey > OSM. A register knows
/// the crown: an OSM tree at a register tree is the same tree. ALKIS surveyed the fences, walls
/// and hedges: an OSM line along one with its symbol is the same.
pub fn resolve(drawn: Drawn, surroundings: &Surroundings) -> Result<Resolved, GeorefError> {
    let Surroundings {
        map_kind,
        georef,
        symbol_set,
        footprints,
        water,
        clip,
        ..
    } = surroundings;
    let Drawn {
        osm_candidates,
        osm_map,
        cadastre,
        mut canopies,
        mut out_of_bounds,
        paved,
        open_land,
    } = drawn;
    // OSM ponds inside the drawn ALKIS water would only duplicate it, and fountain points in
    // water are mostly the jets of a basin.
    let off_water = |objects: Vec<MapObject>| {
        let kept = drop_covered_water(objects, water, clip, georef, symbol_set);
        drop_points_in_water(kept, water, georef, symbol_set)
    };
    let osm_candidates = off_water(osm_candidates);
    let osm_map = off_water(osm_map);
    // A survey line along an ALKIS line goes first, then an OSM line along either.
    let cadastre = drop_cadastre_lines_by_rank(cadastre, symbol_set, georef.scale_denominator);
    let (osm_candidates, mut cadastre) = drop_trees_by_rank(
        osm_candidates,
        cadastre,
        symbol_set,
        georef.scale_denominator,
    );
    let osm_candidates = drop_osm_lines_near_cadastre(
        osm_candidates,
        &mut cadastre,
        symbol_set,
        georef.scale_denominator,
    );
    // The lighting cadastre and the fountain registers beat the survey and OSM points at the same
    // place, and the survey furniture beats OSM; a cadastre, register or survey fountain in drawn
    // water is a jet like an OSM one.
    let (osm_candidates, cadastre) = drop_points_near_registers(
        osm_candidates,
        cadastre,
        symbol_set,
        georef.scale_denominator,
    );
    let cadastre = drop_points_in_water(cadastre, water, georef, symbol_set);
    // The same in an OSM fountain basin, which the basin names in `freemapper:also` (ADR 0087).
    let mut osm_candidates = osm_candidates;
    let cadastre = drop_fountains_in_osm_basins(cadastre, &mut osm_candidates, georef, symbol_set);
    // A surveyed bus shelter under an ALKIS canopy is that canopy.
    let cadastre = drop_shelters_under_canopies(cadastre, &mut canopies);
    // Walls, fences and hedges of any symbol that overlap on paper keep the stronger line, those
    // along a building outline go, and a tree in a hedge is the hedge (`build/barriers.rs`).
    let (osm_candidates, cadastre) = resolve_barrier_overlaps(
        osm_candidates,
        cadastre,
        footprints,
        *map_kind,
        georef,
        symbol_set,
    )?;
    // Where the parks, meadows and woods of the land use are map objects, the OSM parks and
    // lawns go (`build/open_land.rs`).
    let osm_candidates = drop_open_land_under_alkis(osm_candidates, &open_land.map, symbol_set);
    // The point symbols keep the norm's gap, dense tree groups become 501.3 on paving, else 402
    // or 405 (on a white base off open land with a 416 boundary; open land: the parks, the 401
    // green strips of the paved build, the OSM pitches among the candidates), cut at paving and
    // paths, and no point stands in a 520 area (`build/points.rs`). The OSM candidates stay
    // before the others.
    let mut points = osm_candidates;
    points.extend(cadastre);
    // An area that two sources give (a passage in OSM and in ALKIS or the Berlin building parts,
    // a private strip in OSM inside an ALKIS area that shall not be entered, an OSM bridge over
    // the ALKIS one) is kept once, by rank (`build/dedup.rs`).
    let mut points =
        drop_areas_by_rank(points, &mut [&mut out_of_bounds, &mut canopies], symbol_set);
    // A passage no runner can reach is left out of the training map (`build/passages.rs`).
    hide_unreachable_passages(
        &mut points,
        &out_of_bounds,
        footprints,
        water,
        clip,
        georef,
        symbol_set,
    );
    // The firm-gravel ways (501.6 to 501.9) are map objects; where ALKIS or the survey draws the
    // paving they are the same object (ADR 0079).
    let ground: Vec<MapObject> = paved
        .candidates
        .iter()
        .chain(&open_land.map)
        .chain(&open_land.candidates)
        .cloned()
        .collect();
    let points = generalize_points(
        points,
        cut_firm_paths(osm_map, &paved, georef, symbol_set),
        &out_of_bounds,
        &paved.map,
        &ground,
        footprints,
        water,
        clip,
        *map_kind,
        georef,
        symbol_set,
    )?;
    Ok(Resolved {
        points,
        canopies,
        out_of_bounds,
        paved,
        open_land,
    })
}

/// Every object of a map before the content settings place them ([`Built::place`]): the
/// expensive part of the assembly, built once and placed again for every change of the content
/// settings.
#[derive(Clone)]
pub struct Built {
    map_kind: MapKind,
    georef: Georeferencing,
    symbol_set: SymbolSet,
    footprints: Vec<Polygon<f64>>,
    water: MultiPolygon<f64>,
    clip: BoundingBox,
    terrain: Arc<Option<Raster<f32>>>,
    /// The objects of the map and of the candidates, as the build put them.
    map: Vec<MapObject>,
    candidates: Vec<MapObject>,
    paved_borders: PavedBorders,
}

/// The objects of the map and of the candidates, placed by content settings.
pub struct Placed {
    pub map: Vec<MapObject>,
    pub candidates: Vec<MapObject>,
}

impl Built {
    /// The objects `map` and `candidates` as the build put them, in `surroundings`. `saved` are
    /// the content settings of the project: the borders of the paved areas are computed for
    /// them here, so that the first placement finds them.
    pub fn new(
        surroundings: Surroundings,
        map: Vec<MapObject>,
        candidates: Vec<MapObject>,
        saved: &ContentSettings,
    ) -> Self {
        let Surroundings {
            map_kind,
            georef,
            symbol_set,
            footprints,
            water,
            clip,
            terrain,
        } = surroundings;
        // The borders for the project's own settings, from the paved areas and kerbs `route`
        // leaves ([`PavedBorders`]): it sends each 501.x object by the object alone and keeps the
        // order, so routing these gives the ones of the routed lists.
        let paved_borders = PavedBorders::default();
        let mut input = border_input(&[&map, &candidates], &symbol_set);
        if let [map, candidates] = &mut input[..] {
            route(saved, map, candidates, &symbol_set);
        }
        paved_borders.get_or_compute(input, |input| {
            paved_borders_of(input, &footprints, &water, &clip, &georef, &symbol_set)
        });
        Self {
            map_kind,
            georef,
            symbol_set,
            footprints,
            water,
            clip,
            terrain: Arc::new(terrain),
            map,
            candidates,
            paved_borders,
        }
    }

    pub fn georef(&self) -> &Georeferencing {
        &self.georef
    }

    pub fn symbol_set(&self) -> &SymbolSet {
        &self.symbol_set
    }

    /// The objects placed by `content` (ADR 0100): `route` sends them to the map, to the
    /// candidates or leaves them out, and the steps that look at both lists follow.
    pub fn place(&self, content: &ContentSettings) -> Result<Placed, GeorefError> {
        let Self {
            georef, symbol_set, ..
        } = self;
        let mut map = self.map.clone();
        let mut candidates = self.candidates.clone();
        // What goes onto the map and what stays a candidate, by the content settings
        // (`build/content.rs`): the objects of the cadastre and the registers go to the map by
        // default.
        route(content, &mut map, &mut candidates, symbol_set);
        // Vegetation from the point clouds stops at the areas that shall not be entered drawn with
        // it (`build/vegetation.rs`); after the routing, which decides where both end up.
        cut_vegetation_at_out_of_bounds(
            &mut map,
            &mut candidates,
            georef.scale_denominator,
            georef,
            symbol_set,
        )?;
        // Contours and north lines stop at paving and paths (`build/line_cuts.rs`).
        let ways = way_footprints(&map, &candidates, &map, self.map_kind, georef, symbol_set);
        let min_piece_m = f64::from(georef.scale_denominator) / 1000.0;
        for list in [&mut candidates, &mut map] {
            cut_lines_at_ways(list, &CONTOUR_CODES, &ways, min_piece_m, georef, symbol_set);
        }
        cut_lines_at_ways(
            &mut map,
            &NORTH_LINE_CODES,
            &ways,
            min_piece_m,
            georef,
            symbol_set,
        );
        // The border of the paved areas, 501.1 (`build/paved_borders.rs`), computed once per input
        // ([`PavedBorders`]).
        let input = border_input(&[&map, &candidates], symbol_set);
        let borders = self.paved_borders.get_or_compute(input, |input| {
            paved_borders_of(
                input,
                &self.footprints,
                &self.water,
                &self.clip,
                georef,
                symbol_set,
            )
        });
        for (list, added) in [&mut map, &mut candidates].into_iter().zip(borders) {
            list.extend(added);
        }
        // The last step on the lines: the ticks of fences point to the olive green area beside them,
        // those of earth banks down the slope (`build/ticks.rs`).
        orient_ticks(
            &mut [&mut map, &mut candidates],
            self.terrain.as_ref().as_ref(),
            georef,
            symbol_set,
        );
        // Then the points keep the norm's gap to walls and fences, on the tick side with room for
        // the ticks (`build/barrier_gaps.rs`).
        move_points_off_barriers(&mut [&mut map, &mut candidates], self.map_kind, symbol_set);
        Ok(Placed { map, candidates })
    }

    /// How many times the paved borders were computed for it (and its clones).
    #[cfg(test)]
    pub(crate) fn borders_computed(&self) -> usize {
        self.paved_borders.computed()
    }
}

/// The 501.1 borders of the paved areas, by what they are computed from
/// ([`border_input`]): 3.1 of 3.7 s of the steps after `route` on Viktoriapark.
///
/// They depend on the content settings, so they cannot be built once for all: `route` moves the
/// 501.5 areas and the kerbs (501.1 of the survey and the cadastre) of the streets group between
/// the map and the candidates or drops them, which changes which areas are bordered in which list
/// and which edges are there already. No other group holds a 501.5 or a 501.1, and nothing
/// between `route` and the borders changes one (the line cuts only touch contours and north
/// lines). So the borders follow the streets group and the profile only, and a placement whose
/// paved areas and kerbs equal those of an earlier one gets that one's borders. The build
/// computes them for the project's own settings; others are computed when first asked for and
/// kept (a handful at most: the streets group has few outcomes; the least recently used goes
/// first).
#[derive(Clone, Default)]
struct PavedBorders(Arc<Mutex<BorderStore>>);

/// The border inputs and the borders they give, per list, the one used last at the end; and how
/// many were computed.
#[derive(Default)]
struct BorderStore {
    entries: Vec<BorderEntry>,
    computed: usize,
}

impl BorderStore {
    /// The borders of `input`, if kept, which then count as used last (the least recently used
    /// entry is the first to go).
    fn take_up(&mut self, input: &[Vec<MapObject>]) -> Option<Vec<Vec<MapObject>>> {
        let at = self.entries.iter().position(|(known, _)| known == input)?;
        let entry = self.entries.remove(at);
        let borders = entry.1.clone();
        self.entries.push(entry);
        Some(borders)
    }
}

/// A border input and the borders it gives, per list.
type BorderEntry = (Vec<Vec<MapObject>>, Vec<Vec<MapObject>>);

/// How many border results a built map keeps.
const KEPT_BORDERS: usize = 8;

impl PavedBorders {
    fn lock(&self) -> MutexGuard<'_, BorderStore> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn get_or_compute(
        &self,
        input: Vec<Vec<MapObject>>,
        compute: impl FnOnce(&[Vec<MapObject>]) -> Vec<Vec<MapObject>>,
    ) -> Vec<Vec<MapObject>> {
        if let Some(borders) = self.lock().take_up(&input) {
            return borders;
        }
        // Computed without the lock: another placement may store the same input meanwhile.
        let borders = compute(&input);
        let mut store = self.lock();
        store.computed += 1;
        if store.take_up(&input).is_none() {
            if store.entries.len() >= KEPT_BORDERS {
                store.entries.remove(0);
            }
            store.entries.push((input, borders.clone()));
        }
        borders
    }

    /// How many borders were computed.
    #[cfg(test)]
    fn computed(&self) -> usize {
        self.lock().computed
    }

    /// How many inputs are kept.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.lock().entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::content::{Group, POINT_CLOUD_KEY, Place, Profile};
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::omap::MapObjectKind;
    use crate::omap::coord::MapCoord;

    /// A border input that differs by `n`.
    fn input(n: i32) -> Vec<Vec<MapObject>> {
        vec![vec![MapObject {
            symbol_id: 1,
            kind: MapObjectKind::Path,
            coords: vec![MapCoord::new(n, 0), MapCoord::new(n, 1)],
            tags: Vec::new(),
        }]]
    }

    /// Asks `store` for `n` and says whether it computed the borders.
    fn computes(store: &PavedBorders, n: i32) -> bool {
        let mut computed = false;
        let borders = store.get_or_compute(input(n), |i| {
            computed = true;
            i.to_vec()
        });
        assert_eq!(borders, input(n));
        computed
    }

    #[test]
    fn the_border_store_keeps_what_was_used_last() {
        let store = PavedBorders::default();
        for n in 0..KEPT_BORDERS as i32 {
            assert!(computes(&store, n));
        }
        // The first one used again, then one more: the second goes, the first stays.
        assert!(!computes(&store, 0));
        assert!(computes(&store, 100));
        assert!(!computes(&store, 0));
        assert!(computes(&store, 1));
        assert_eq!(store.computed(), KEPT_BORDERS + 2);
    }

    #[test]
    fn the_border_store_keeps_an_input_once() {
        let store = PavedBorders::default();
        let other = store.clone();
        // Another placement stores the same input while this one computes it.
        store.get_or_compute(input(7), |i| {
            assert!(computes(&other, 7));
            i.to_vec()
        });
        assert_eq!(store.len(), 1);
        assert!(!computes(&store, 7));
    }

    fn path(f: &Fixture, code: &str, points: &[(f64, f64)]) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| f.at(p)).collect(),
            tags: Vec::new(),
        }
    }

    fn square(f: &Fixture, code: &str, from: (f64, f64), side: f64) -> MapObject {
        let mut o = path(
            f,
            code,
            &[
                from,
                (from.0 + side, from.1),
                (from.0 + side, from.1 + side),
                (from.0, from.1 + side),
            ],
        );
        o.coords.last_mut().unwrap().flags = crate::omap::coord::FLAG_CLOSE;
        o
    }

    /// The fixture without buildings, water or terrain.
    fn surroundings(f: &Fixture) -> Surroundings {
        Surroundings {
            map_kind: f.kind,
            georef: f.georef.clone(),
            symbol_set: f.set.clone(),
            footprints: Vec::new(),
            water: MultiPolygon::new(Vec::new()),
            clip: f.clip,
            terrain: None,
        }
    }

    /// A built map of the sprint fixture with these objects.
    fn built(f: &Fixture, map: Vec<MapObject>, candidates: Vec<MapObject>) -> Built {
        Built::new(
            surroundings(f),
            map,
            candidates,
            &ContentSettings::with_profile(Profile::Safe),
        )
    }

    fn source_ids(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .filter_map(|o| {
                o.tags
                    .iter()
                    .find(|(k, _)| k == "freemapper:source_id")
                    .map(|(_, v)| v.as_str())
            })
            .collect()
    }

    #[test]
    fn a_tree_of_two_sources_is_kept_once_and_the_osm_candidates_come_first() {
        let f = fixture(MapKind::Sprint);
        let garden = square(&f, "520", (200.0, 200.0), 30.0);
        let lawn = square(&f, "401", (-200.0, -200.0), 30.0);
        let drawn = Drawn {
            osm_candidates: vec![
                // Half a metre from the register tree: the same tree.
                f.tree("418", "osm", "node/1", (10.5, 10.0), None),
                f.tree("418", "osm", "node/2", (60.0, 10.0), None),
            ],
            osm_map: Vec::new(),
            cadastre: vec![f.tree(
                "418",
                "trees/strassenbaeume",
                "tree1",
                (10.0, 10.0),
                Some(6.0),
            )],
            canopies: Vec::new(),
            out_of_bounds: vec![garden.clone()],
            paved: PavedObjects::default(),
            open_land: OpenLandObjects {
                map: vec![lawn.clone()],
                candidates: Vec::new(),
            },
        };
        let resolved = resolve(drawn, &surroundings(&f)).unwrap();
        assert_eq!(source_ids(&resolved.points.candidates), ["node/2", "tree1"]);
        assert!(resolved.points.areas.is_empty());
        assert!(resolved.points.map.is_empty());
        // What is only looked at comes back as it was.
        assert_eq!(resolved.out_of_bounds, [garden]);
        assert_eq!(resolved.open_land.map, [lawn]);
        assert!(resolved.canopies.is_empty());
    }

    fn count(objects: &[MapObject], id: u32) -> usize {
        objects.iter().filter(|o| o.symbol_id == id).count()
    }

    #[test]
    fn placing_needs_no_project_folder_and_leaves_the_built_objects_alone() {
        let f = fixture(MapKind::Sprint);
        // A paved square on the map and a contour among the candidates that crosses it.
        let built = built(
            &f,
            vec![square(&f, "501.5", (0.0, 0.0), 40.0)],
            vec![path(&f, "101", &[(-40.0, 20.0), (80.0, 20.0)])],
        );
        let before = (built.map.clone(), built.candidates.clone());
        let placed = built
            .place(&ContentSettings::with_profile(Profile::Safe))
            .unwrap();
        // The contour stops at the paving: one piece on each side of it.
        assert_eq!(count(&placed.candidates, f.id("101")), 2);
        // The paved area got its border.
        assert_eq!(count(&placed.map, f.id("501.5")), 1);
        assert!(count(&placed.map, f.id("501.1")) >= 1);
        assert_eq!((built.map.clone(), built.candidates.clone()), before);
    }

    #[test]
    fn the_borders_are_computed_once_per_outcome_of_the_streets_group() {
        let f = fixture(MapKind::Sprint);
        let built = built(&f, vec![square(&f, "501.5", (0.0, 0.0), 40.0)], Vec::new());
        // The build computed them for the project's own settings.
        assert_eq!(built.borders_computed(), 1);
        let safe = ContentSettings::with_profile(Profile::Safe);
        let mut walls_off = safe.clone();
        walls_off.set(Group::Walls, Some(Place::Off));
        for settings in [&safe, &walls_off] {
            built.place(settings).unwrap();
            assert_eq!(built.borders_computed(), 1, "{settings:?}");
        }
        let mut streets_off = safe.clone();
        streets_off.set(Group::Streets, Some(Place::Off));
        let placed = built.place(&streets_off).unwrap();
        assert_eq!(built.borders_computed(), 2);
        assert_eq!(count(&placed.map, f.id("501.5")), 0);
        assert_eq!(count(&placed.map, f.id("501.1")), 0);
        // A clone shares the store.
        built.clone().place(&streets_off).unwrap();
        assert_eq!(built.borders_computed(), 2);
    }

    #[test]
    fn vegetation_is_cut_at_the_gardens_it_is_placed_with() {
        let f = fixture(MapKind::Sprint);
        // 40 m × 40 m of forest from a point cloud, its eastern half in a private garden; both
        // candidates as the build puts them.
        let mut forest = square(&f, "405", (0.0, 0.0), 40.0);
        forest
            .tags
            .push((POINT_CLOUD_KEY.to_string(), "lgb-als".to_string()));
        let garden = square(&f, "520", (20.0, -10.0), 60.0);
        let built = built(&f, Vec::new(), vec![garden, forest.clone()]);
        let mut settings = ContentSettings::with_profile(Profile::Safe);
        settings.set(Group::Vegetation, Some(Place::Candidates));
        let placed = built.place(&settings).unwrap();
        let left: Vec<&MapObject> = placed
            .candidates
            .iter()
            .filter(|o| o.symbol_id == f.id("405"))
            .collect();
        assert_eq!(left.len(), 1);
        assert_ne!(left[0].coords, forest.coords, "cut at the garden");
        // With the vegetation left out, nothing of it is placed.
        settings.set(Group::Vegetation, Some(Place::Off));
        let placed = built.place(&settings).unwrap();
        assert_eq!(count(&placed.candidates, f.id("405")), 0);
        assert_eq!(count(&placed.map, f.id("405")), 0);
    }
}
