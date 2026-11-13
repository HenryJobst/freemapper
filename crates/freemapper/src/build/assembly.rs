//! The map assembly (ADR 0115): the objects of all sources, put together to the map and the
//! candidates in a fixed order of steps.
//!
//! The assembly reads no file. It gets what the project file says about the map ([`Frame`])
//! and the stored data, read ([`Stored`]), and builds every object once ([`build`]): it draws
//! the sources, resolves the duplicates between them, generalizes the points and puts the
//! lists of the map and of the candidates in their order. The result ([`Built`]) is placed by
//! the content settings as often as asked ([`Built::place`], ADR 0100): `route` sends each
//! object to the map, to the candidates or leaves it out, and the steps that look at both
//! lists follow. The project (`project/export.rs`) reads the folder, calls the assembly and
//! makes the documents from what it returns.

use std::ops::Range;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use chrono::NaiveDate;
use geo_types::{MultiPolygon, Polygon};
use serde_json::Value;

use crate::area::MapArea;
use crate::build::barrier_gaps::move_points_off_barriers;
use crate::build::barriers::resolve_barrier_overlaps;
use crate::build::berlin::{
    BUILDING_PARTS_LAYER, LayerInput, berlin_objects, building_part_features,
    drop_shelters_under_canopies,
};
use crate::build::buildings::{building_objects, object_footprints};
use crate::build::candidates::{Rules, Target, osm_objects};
use crate::build::canopy::{CANOPY_LAYER, CanopyInput, canopy_objects};
use crate::build::content::{ContentSettings, route};
use crate::build::contours::{Contours, contour_objects};
use crate::build::dedup::{
    drop_areas_by_rank, drop_cadastre_lines_by_rank, drop_osm_lines_near_cadastre,
    drop_points_near_registers, drop_trees_by_rank,
};
use crate::build::firm_paths::cut_firm_paths;
use crate::build::landforms::{Landforms, landform_objects};
use crate::build::line_cuts::{CONTOUR_CODES, MIN_PIECE_MM, NORTH_LINE_CODES, cut_lines_at_ways};
use crate::build::open_land::{
    OpenLandObjects, drop_open_land_under_alkis, open_land_objects, osm_forest_polygons,
};
use crate::build::out_of_bounds::out_of_bounds_objects;
use crate::build::passages::hide_unreachable as hide_unreachable_passages;
use crate::build::paved::{
    GREEN_LAYER, PavedObjects, SURFACE_LAYER, StreetSurvey, paved_area_objects, unpaved_streets,
};
use crate::build::paved_borders::{border_input, paved_borders as paved_borders_of};
use crate::build::peripheral::{PaperRect, north_lines};
use crate::build::points::{GeneralizedPoints, generalize_points, way_footprints};
use crate::build::stairs::{
    NAS_STAIR_LAYER, STEP_EDGE_LAYER, SURVEY_STAIR_LAYER, StairInput, stair_objects,
};
use crate::build::state::{NasInput, TreeInput, nas_objects, state_objects, tree_objects};
use crate::build::ticks::orient_ticks;
use crate::build::vegetation::{Vegetation, cut_vegetation_at_out_of_bounds, vegetation_objects};
use crate::build::water::{
    drop_covered_water, drop_fountains_in_osm_basins, drop_points_in_water, water_objects,
};
use crate::crs::{BoundingBox, LatLon};
use crate::georef::{GeorefError, Georeferencing};
use crate::measure::{SymbolMm, SymbolScale};
use crate::omap::{MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::raster::Raster;
use crate::sources::alkis::{Building, LandUse};
use crate::sources::osm::OsmFeature;

/// What the project file says about the map.
pub struct Frame {
    pub map_kind: MapKind,
    pub georef: Georeferencing,
    pub symbol_set: SymbolSet,
    /// The map area and the id of the OSM object it was taken from, if known.
    pub area: MapArea,
    pub area_source_id: Option<String>,
    /// Objects are drawn up to this far around the map area (ground metres).
    pub buffer_m: f64,
    /// The map area plus buffer, in the map's CRS.
    pub clip: BoundingBox,
}

/// The stored ALKIS buildings: parsed, and the raw features (the canopies, the passages and the
/// Berlin building parts are picked from these).
pub struct Buildings {
    pub buildings: Vec<Building>,
    pub features: Vec<Value>,
    /// The date of their fetch, if the project records one; the canopies, passages and building
    /// parts are drawn only then.
    pub fetched: Option<NaiveDate>,
}

/// One stored layer of extra data: its file name (for a tree cadastre the dataset), its features
/// and the date of its fetch, which stands in for the currency of a feature without a date.
pub struct Layer {
    pub name: String,
    pub features: Vec<Value>,
    pub fetched: NaiveDate,
}

/// The stored data of a project, read: what [`build`] draws from. A source that was not fetched
/// or does not cover the map area is empty.
#[derive(Default)]
pub struct Stored {
    pub buildings: Option<Buildings>,
    /// The extra layers of Berlin, the full ALKIS (NAS) and the tree cadastres, in table order.
    pub berlin: Vec<Layer>,
    pub nas: Vec<Layer>,
    pub trees: Vec<Layer>,
    pub land_use: Vec<LandUse>,
    /// The OSM features and the date of their extract.
    pub osm: Option<(Vec<OsmFeature>, NaiveDate)>,
    /// The terrain model for the stair direction, in the map's CRS.
    pub stair_terrain: Option<Raster<f32>>,
    /// The terrain model the ticks of earth banks follow.
    pub terrain: Option<Raster<f32>>,
    pub contours: Option<Contours>,
    pub landforms: Option<Landforms>,
    pub vegetation: Option<Vegetation>,
}

/// Line symbol used for the map area outline in the candidates template map.
pub const OUTLINE_SYMBOL_CODE: &str = "501.1";
/// The outline of a symbol set without 501.1 (the forest map): the sheet symbol 994, which no
/// content group claims.
const OUTLINE_FALLBACK_CODE: &str = "994";
/// The distance of the north lines on paper at symbol size 100 %.
const NORTH_LINE_SPACING_MM: SymbolMm = SymbolMm(30.0);

/// The exterior ring of the map area as a closed path object.
pub fn map_area_outline(
    area: &MapArea,
    source_id: Option<&str>,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<MapObject, GeorefError> {
    let points = area
        .polygon
        .exterior()
        .points()
        .map(|p| {
            let c = georef.geographic_to_map(LatLon {
                lat: p.y(),
                lon: p.x(),
            })?;
            Ok((c.x, c.y))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut tags = vec![("freemapper:source".to_string(), "osm".to_string())];
    if let Some(id) = source_id {
        tags.push(("freemapper:source_id".to_string(), id.to_string()));
    }
    Ok(MapObject {
        symbol_id: symbol_set
            .id_for_code(OUTLINE_SYMBOL_CODE)
            .or_else(|| symbol_set.id_for_code(OUTLINE_FALLBACK_CODE))
            .expect("symbol set defines 501.1 or the sheet symbols"),
        kind: MapObjectKind::Path,
        coords: closed_ring(&points),
        tags,
    })
}

/// The layer `name` of `layers`, if stored.
fn layer<'a>(layers: &'a [Layer], name: &str) -> Option<&'a Layer> {
    layers.iter().find(|l| l.name == name)
}

/// The 522 canopy map objects and the 532.x stair candidates.
struct Structures {
    canopies: Vec<MapObject>,
    stairs: Vec<MapObject>,
}

/// The canopies of the Berlin `bauwerke-flaechen` layer and of the ALKIS buildings of any
/// state, and the stairs of the Berlin layers (stair areas, step edges, the survey) and of the
/// full ALKIS.
fn structures(
    stored: &Stored,
    footprints: &[Polygon<f64>],
    map_area: &Polygon<f64>,
    frame: &Frame,
) -> Result<Structures, GeorefError> {
    let Frame {
        map_kind,
        georef,
        symbol_set,
        buffer_m,
        ..
    } = frame;
    // The stair areas are in the same layer as the canopies; the step edges are lines.
    let areas = layer(&stored.berlin, CANOPY_LAYER);
    let edges = layer(&stored.berlin, STEP_EDGE_LAYER);
    let survey = layer(&stored.berlin, SURVEY_STAIR_LAYER);
    // The NAS stair areas and bottom edges are in one layer.
    let nas = layer(&stored.nas, NAS_STAIR_LAYER);
    // Canopies are cut at the 521 footprints; flat entrance steps face them.
    let mut canopies = match areas {
        Some(layer) => canopy_objects(
            &[CanopyInput::Berlin(&layer.features)],
            footprints,
            map_area,
            *buffer_m,
            georef,
            symbol_set,
            layer.fetched,
        )?,
        None => Vec::new(),
    };
    // The canopies among the ALKIS buildings of any state (`Überdachung`, `Carport`).
    if let Some((buildings, fetched)) = dated(&stored.buildings) {
        canopies.extend(canopy_objects(
            &[CanopyInput::Buildings(&buildings.features)],
            footprints,
            map_area,
            *buffer_m,
            georef,
            symbol_set,
            fetched,
        )?);
    }
    if areas.is_none() && edges.is_none() && survey.is_none() && nas.is_none() {
        return Ok(Structures {
            canopies,
            stairs: Vec::new(),
        });
    }
    fn features(layer: Option<&Layer>) -> &[Value] {
        layer.map_or(&[], |l| &l.features)
    }
    // Both layers come from one fetch; the later date stands for the currency.
    let fetched = [areas, edges, survey, nas]
        .into_iter()
        .flatten()
        .map(|l| l.fetched)
        .max()
        .expect("one of the layers is stored");
    let stairs = stair_objects(
        &StairInput {
            features: features(areas),
            step_edges: features(edges),
            survey: features(survey),
            nas: features(nas),
            terrain: stored.stair_terrain.as_ref(),
            buildings: footprints,
        },
        *map_kind,
        map_area,
        *buffer_m,
        georef,
        symbol_set,
        fetched,
    )?;
    Ok(Structures { canopies, stairs })
}

/// The buildings with the date of their fetch, when the project records one.
fn dated(buildings: &Option<Buildings>) -> Option<(&Buildings, NaiveDate)> {
    let buildings = buildings.as_ref()?;
    Some((buildings, buildings.fetched?))
}

/// The candidates of the cadastres, the registers and the survey: Berlin first, then the state
/// passages, the full ALKIS (NAS) and the tree cadastres.
fn cadastre_candidates(
    stored: &Stored,
    map_area: &Polygon<f64>,
    frame: &Frame,
) -> Result<Vec<MapObject>, GeorefError> {
    let Frame {
        map_kind,
        georef,
        symbol_set,
        buffer_m,
        ..
    } = frame;
    let buildings = dated(&stored.buildings);
    // The Berlin layers and the building parts of the buildings (passages), through the Berlin
    // rules. The building parts are in the buildings file (`bauart` is set by the Berlin source
    // only).
    let parts = buildings.map(|(b, fetched)| (building_part_features(&b.features), fetched));
    let mut inputs: Vec<LayerInput<'_>> = stored
        .berlin
        .iter()
        .map(|l| LayerInput {
            file: &l.name,
            features: &l.features,
            currency: l.fetched,
        })
        .collect();
    if let Some((features, fetched)) = &parts {
        inputs.push(LayerInput {
            file: BUILDING_PARTS_LAYER,
            features,
            currency: *fetched,
        });
    }
    let mut cadastre = berlin_objects(&inputs, *map_kind, map_area, *buffer_m, georef, symbol_set)?;
    // 522 candidates for the building passages of the ALKIS buildings of any state. Berlin
    // building parts (`bauart`) go through the Berlin rules instead.
    if let Some((buildings, fetched)) = buildings {
        cadastre.extend(state_objects(
            &buildings.features,
            *map_kind,
            map_area,
            *buffer_m,
            georef,
            symbol_set,
            fetched,
        )?);
    }
    // The full ALKIS: walls, fences, hedges, monuments, bridges, embankment tops and ditches.
    let nas: Vec<NasInput<'_>> = stored
        .nas
        .iter()
        .map(|l| NasInput {
            file: &l.name,
            features: &l.features,
            fetched: l.fetched,
        })
        .collect();
    cadastre.extend(nas_objects(
        &nas, *map_kind, map_area, *buffer_m, georef, symbol_set,
    )?);
    // A forest map draws no register trees (spec 2026-11-13).
    if *map_kind != MapKind::Forest {
        let trees: Vec<TreeInput<'_>> = stored
            .trees
            .iter()
            .map(|l| TreeInput {
                dataset: &l.name,
                features: &l.features,
                fetched: l.fetched,
            })
            .collect();
        cadastre.extend(tree_objects(
            &trees, *map_kind, map_area, *buffer_m, georef, symbol_set,
        )?);
    }
    Ok(cadastre)
}

/// 501.5 paved areas from the land use; sprint maps only (spec §4.5). In Berlin the
/// Straßenbefahrung cuts its green strips out and adds its paving as candidates.
fn paved(stored: &Stored, frame: &Frame) -> Result<PavedObjects, GeorefError> {
    let Frame {
        map_kind,
        georef,
        symbol_set,
        clip,
        ..
    } = frame;
    if *map_kind != MapKind::Sprint {
        return Ok(PavedObjects::default());
    }
    let streets = stored
        .osm
        .as_ref()
        .map(|(features, _)| unpaved_streets(features, georef.crs()));
    let features =
        |name: &str| layer(&stored.berlin, name).map_or(&[][..], |l| l.features.as_slice());
    let survey =
        StreetSurvey::from_features(features(GREEN_LAYER), features(SURFACE_LAYER), georef.crs());
    paved_area_objects(
        &stored.land_use,
        streets.as_deref().unwrap_or_default(),
        &survey,
        clip,
        georef,
        symbol_set,
    )
}

/// Objects for the OSM features whose rule has `target`.
fn osm(
    target: Target,
    stored: &Stored,
    map_area: &Polygon<f64>,
    frame: &Frame,
) -> Result<Vec<MapObject>, GeorefError> {
    let Some((features, extract)) = &stored.osm else {
        return Ok(Vec::new());
    };
    osm_objects(
        features,
        &Rules::embedded(),
        target,
        frame.map_kind,
        map_area,
        frame.buffer_m,
        &frame.georef,
        &frame.symbol_set,
        "osm",
        *extract,
    )
}

/// Builds every object of the map `frame` describes from the `stored` data ([`Built`]): draws
/// the sources, resolves the duplicates between them ([`resolve`]) and puts the lists of the map
/// and of the candidates in their order. `saved` are the content settings of the project.
pub fn build(frame: Frame, stored: Stored, saved: &ContentSettings) -> Result<Built, GeorefError> {
    let map_area = frame.area.projected_polygon_in(frame.georef.crs());
    // Buildings first: they are the map's own objects, candidates only a template.
    let footprints = stored
        .buildings
        .as_ref()
        .map_or_else(Vec::new, |b| object_footprints(&b.buildings));
    let building_objects = match &stored.buildings {
        Some(stored) => building_objects(
            &stored.buildings,
            &map_area,
            frame.buffer_m,
            &frame.georef,
            &frame.symbol_set,
        )?,
        None => Vec::new(),
    };
    let structures = structures(&stored, &footprints, &map_area, &frame)?;
    let paved = paved(&stored, &frame)?;
    let water = water_objects(
        &stored.land_use,
        &frame.clip,
        &frame.georef,
        &frame.symbol_set,
    )?;
    let cadastre = cadastre_candidates(&stored, &map_area, &frame)?;
    // Parks, meadows and woods of the land use: 401, 403 and 405 (`build/open_land.rs`).
    let osm_forest = match (&stored.osm, frame.map_kind) {
        (Some((features, _)), MapKind::Forest) => osm_forest_polygons(features, frame.georef.crs()),
        _ => Vec::new(),
    };
    let open_land = open_land_objects(
        &stored.land_use,
        &map_area,
        &frame.clip,
        frame.map_kind,
        &frame.georef,
        &frame.symbol_set,
        &osm_forest,
    )?;
    // 520 candidates for the private land around the map area.
    let out_of_bounds = out_of_bounds_objects(
        &stored.land_use,
        &map_area,
        &frame.clip,
        frame.map_kind,
        &frame.georef,
        &frame.symbol_set,
    )?;
    let osm_candidates = osm(Target::Candidates, &stored, &map_area, &frame)?;
    let osm_map = osm(Target::Map, &stored, &map_area, &frame)?;
    let outline = map_area_outline(
        &frame.area,
        frame.area_source_id.as_deref(),
        &frame.georef,
        &frame.symbol_set,
    )?;
    let Stored {
        land_use,
        terrain,
        contours,
        landforms,
        vegetation,
        ..
    } = stored;
    let Frame {
        map_kind,
        georef,
        symbol_set,
        buffer_m,
        clip,
        ..
    } = frame;
    let surroundings = Surroundings {
        map_kind,
        georef,
        symbol_set,
        footprints,
        water: water.union,
        clip,
        terrain,
    };
    // The duplicates between the sources go and the points are generalized.
    let Resolved {
        points: generalized,
        canopies,
        out_of_bounds,
        paved,
        open_land,
    } = resolve(
        Drawn {
            osm_candidates,
            osm_map,
            cadastre,
            canopies: structures.canopies,
            out_of_bounds,
            paved,
            open_land,
        },
        &surroundings,
    )?;
    let Surroundings {
        georef,
        symbol_set,
        footprints,
        water: water_union,
        clip,
        ..
    } = &surroundings;
    let mut point_candidates = generalized.candidates;
    point_candidates.extend(generalized.areas);
    // The outline first, then the points: the project names a register by them.
    let points = 1..1 + point_candidates.len();
    let contour_objects = match &contours {
        Some(contours) => contour_objects(
            contours,
            footprints,
            water_union,
            &map_area,
            buffer_m,
            georef,
            symbol_set,
        )?,
        None => Vec::new(),
    };
    let mut candidates: Vec<MapObject> = std::iter::once(outline)
        .chain(point_candidates)
        .chain(structures.stairs)
        .chain(contour_objects)
        .chain(paved.candidates)
        .chain(open_land.candidates)
        .chain(water.candidates)
        .chain(out_of_bounds)
        .collect();
    let content = PaperRect::of_bbox(clip, georef)?;
    // North lines first: within their black they lie under walls and fences.
    let spacing =
        NORTH_LINE_SPACING_MM.on_paper(SymbolScale::new(map_kind, georef.scale_denominator));
    let mut map_objects = north_lines(&content, spacing, symbol_set);
    map_objects.extend(building_objects);
    // Canopies after the buildings; their Black 20 % lies under the building fill anyway.
    map_objects.extend(canopies);
    // Parks under everything but the walkable area: colour priority puts yellow low.
    map_objects.extend(open_land.map);
    map_objects.extend(paved.map);
    map_objects.extend(water.map);
    map_objects.extend(generalized.map);
    // New object builders go here (landform candidates, vegetation): they see every object built
    // so far (`map_objects`, `candidates`), `water_union`, `footprints`, `map_area` and `georef`,
    // and what they add is placed by the content settings like the rest.
    // Knolls, depressions, banks and cliffs from the terrain, where nothing drawn shows them yet.
    // Before the routing, so that the content settings move the banks and cliffs (group `slopes`)
    // like the contours; with no source rank, `balanced` leaves them candidates.
    if let Some(landforms) = &landforms {
        let drawn: Vec<&MapObject> = map_objects.iter().chain(&candidates).collect();
        let landforms = landform_objects(
            landforms,
            footprints,
            &land_use,
            water_union,
            &drawn,
            &map_area,
            buffer_m,
            georef,
            symbol_set,
        )?;
        candidates.extend(landforms);
    }
    // The vegetation from the point clouds, where the user chose it (group `vegetation`).
    if let Some(vegetation) = &vegetation {
        let vegetation = vegetation_objects(
            vegetation,
            footprints,
            water_union,
            &map_objects,
            &map_area,
            buffer_m,
            georef,
            symbol_set,
        )?;
        candidates.extend(vegetation);
    }
    let mut built = Built::new(surroundings, map_objects, candidates, saved);
    built.points = points;
    Ok(built)
}

/// What the steps of the assembly look at besides the objects: the kind and the frame of the
/// map, the buildings, the drawn water, the clip window and the terrain.
struct Surroundings {
    pub map_kind: MapKind,
    pub georef: Georeferencing,
    pub symbol_set: SymbolSet,
    pub footprints: Vec<Polygon<f64>>,
    pub water: MultiPolygon<f64>,
    pub clip: BoundingBox,
    pub terrain: Option<Raster<f32>>,
}

/// What the sources drew, before the duplicates between them are resolved ([`resolve`]).
struct Drawn {
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
struct Resolved {
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
fn resolve(drawn: Drawn, surroundings: &Surroundings) -> Result<Resolved, GeorefError> {
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
    /// Where the point candidates stand among `candidates` ([`Built::point_candidates`]).
    points: Range<usize>,
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
    fn new(
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
            points: 0..0,
            paved_borders,
        }
    }

    /// The candidates of OSM, the cadastres and the registers after the point generalization,
    /// with the group areas made from them: the project names a register by them.
    pub fn point_candidates(&self) -> &[MapObject] {
        &self.candidates[self.points.clone()]
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
        let min_piece_m = MIN_PIECE_MM.metres(georef.scale_denominator);
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
        move_points_off_barriers(
            &mut [&mut map, &mut candidates],
            self.map_kind,
            georef.scale_denominator,
            symbol_set,
        );
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

    /// The frame of the fixture: a map area of about 200 m around the reference point, taken
    /// from `way/1`, with a buffer of 50 m.
    fn frame(f: &Fixture) -> Frame {
        let (lat, lon) = (52.4876, 13.3801);
        let ring: Vec<(f64, f64)> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .iter()
            .map(|(x, y)| (lon + x * 0.0015, lat + y * 0.001))
            .collect();
        Frame {
            map_kind: f.kind,
            georef: f.georef.clone(),
            symbol_set: f.set.clone(),
            area: MapArea {
                polygon: Polygon::new(ring.into(), vec![]),
                source_id: None,
            },
            area_source_id: Some("way/1".to_string()),
            buffer_m: 50.0,
            clip: f.clip,
        }
    }

    #[test]
    fn nothing_stored_gives_the_outline_and_the_north_lines() {
        let f = fixture(MapKind::Sprint);
        let safe = ContentSettings::with_profile(Profile::Safe);
        let built = build(frame(&f), Stored::default(), &safe).unwrap();
        assert!(built.point_candidates().is_empty());
        let placed = built.place(&safe).unwrap();
        // The candidates: the outline of the map area, from the OSM object the frame names.
        assert_eq!(placed.candidates.len(), 1);
        assert_eq!(placed.candidates[0].symbol_id, f.id(OUTLINE_SYMBOL_CODE));
        assert_eq!(source_ids(&placed.candidates), ["way/1"]);
        // The map: north lines and nothing else.
        assert!(!placed.map.is_empty());
        assert_eq!(count(&placed.map, f.id("601")), placed.map.len());
    }

    #[test]
    fn the_land_use_is_drawn_after_the_north_lines_without_a_project_folder() {
        let f = fixture(MapKind::Sprint);
        let (e, n) = (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        );
        let wood: Vec<(f64, f64)> = vec![
            (e - 30.0, n - 30.0),
            (e + 30.0, n - 30.0),
            (e + 30.0, n + 30.0),
            (e - 30.0, n + 30.0),
            (e - 30.0, n - 30.0),
        ];
        let stored = Stored {
            land_use: vec![LandUse {
                oid: "DEBE00YY0000001".to_string(),
                nutzart: "Wald".to_string(),
                bez: None,
                updated: "2025-09-30".parse().ok(),
                footprint: MultiPolygon(vec![Polygon::new(wood.into(), vec![])]),
            }],
            ..Stored::default()
        };
        let safe = ContentSettings::with_profile(Profile::Safe);
        let built = build(frame(&f), stored, &safe).unwrap();
        // North lines first, the wood of the land use behind them.
        let (north, forest) = (f.id("601"), f.id("405"));
        assert_eq!(built.map.first().map(|o| o.symbol_id), Some(north));
        let first_forest = built.map.iter().position(|o| o.symbol_id == forest);
        let last_north = built.map.iter().rposition(|o| o.symbol_id == north);
        assert!(first_forest > last_north, "{first_forest:?} {last_north:?}");
        assert!(first_forest.is_some());
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
