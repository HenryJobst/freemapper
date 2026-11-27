// SPDX-License-Identifier: AGPL-3.0-or-later

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
use crate::build::buildings::{building_objects, object_footprints};
use crate::build::candidates::{Rules, Target, osm_objects};
use crate::build::canopy::{CanopyInput, canopy_objects};
use crate::build::cliffs::{
    Ground, Level, generalize_cliffs, hide_landforms_on_rock, reveal_generalized,
};
use crate::build::content::{ContentSettings, route};
use crate::build::contours::{Contours, contour_objects};
use crate::build::dedup::{
    drop_areas_by_rank, drop_cadastre_lines_by_rank, drop_landscape, drop_osm_lines_near_cadastre,
    drop_points_near_registers, drop_trees_by_rank,
};
use crate::build::firm_paths::cut_firm_paths;
use crate::build::landforms::{Landforms, landform_objects};
use crate::build::light::{LightReport, light_level};
use crate::build::line_cuts::{
    CONTOUR_CODES, MIN_PIECE_MM, NORTH_LINE_CODES, contour_cut_outs, cut_lines_at_ways,
};
use crate::build::open_land::{
    OpenLandObjects, drop_open_land_under_alkis, open_land_objects, osm_forest_polygons,
};
use crate::build::out_of_bounds::out_of_bounds_objects;
use crate::build::passages::hide_unreachable as hide_unreachable_passages;
use crate::build::paved::{
    PavedObjects, StoredLayer, StreetSurvey, paved_area_objects, unpaved_streets,
};
use crate::build::paved_borders::{border_input, paved_borders as paved_borders_of};
use crate::build::peripheral::{PaperRect, north_lines};
use crate::build::points::{GeneralizedPoints, generalize_points, way_footprints};
use crate::build::stairs::{
    CadastreStairs, NAS_STAIR_LAYER, SetStairs, StairInput, drop_osm_stairs_on_cadastre,
    holds_stairs, stair_objects,
};
use crate::build::state::{NasInput, TreeInput, nas_objects, state_objects, tree_objects};
use crate::build::ticks::orient_ticks;
use crate::build::training::TrainingInput;
use crate::build::vegetation::{Vegetation, cut_vegetation_at_out_of_bounds, vegetation_objects};
use crate::build::water::{
    drop_covered_water, drop_fountains_in_osm_basins, drop_points_in_water, water_objects,
};
use crate::crs::{BoundingBox, LatLon};
use crate::derive::contours::MIN_LENGTH_MM as CONTOUR_MIN_LENGTH_MM;
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
    /// The layers of the sets of extra layers (ADR 0149), then of the full ALKIS (NAS) and of the
    /// tree cadastres, in table order.
    pub sets: Vec<crate::build::sets::SetLayers>,
    pub nas: Vec<Layer>,
    pub trees: Vec<Layer>,
    pub land_use: Vec<LandUse>,
    /// The OSM features and the date of their extract.
    pub osm: Option<(Vec<OsmFeature>, NaiveDate)>,
    /// The terrain model for the stair direction, in the map's CRS.
    pub stair_terrain: Option<Raster<f32>>,
    /// The terrain model the ticks of earth banks follow.
    pub terrain: Option<Raster<f32>>,
    /// Where the orthophoto shows rock, per metre (forest maps; `derive/rock_seen.rs`).
    pub rock_seen: Option<Raster<f32>>,
    pub contours: Option<Contours>,
    pub landforms: Option<Landforms>,
    pub vegetation: Option<Vegetation>,
}

/// Line symbol used for the map area outline in the candidates template map.
pub const OUTLINE_SYMBOL_CODE: &str = "501.1";
/// The outline of a symbol set without 501.1 (the forest map): the sheet symbol 994, which no
/// content group claims.
const OUTLINE_FALLBACK_CODE: &str = "994";
/// The distance of the north lines on paper at symbol size 100 %. ISOM 2017-2 (symbol 601) says
/// 20 mm, which is 300 m on the ground at 1:15 000 (ISOM 2000 had 30 mm); ISSprOM 2019 and the
/// school symbol set keep 30 mm.
fn north_line_spacing(kind: MapKind) -> SymbolMm {
    match kind {
        MapKind::Schoolyard | MapKind::Sprint => SymbolMm(30.0),
        MapKind::Forest => SymbolMm(20.0),
    }
}

/// The symbol of the map area outline in `symbol_set`: 501.1, else the sheet symbol 994.
pub(crate) fn outline_symbol_id(symbol_set: &SymbolSet) -> Option<u32> {
    symbol_set
        .id_for_code(OUTLINE_SYMBOL_CODE)
        .or_else(|| symbol_set.id_for_code(OUTLINE_FALLBACK_CODE))
}

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
        symbol_id: outline_symbol_id(symbol_set)
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
    // The layers of the sets whose stairs rank with the cadastre.
    fn stair_layers(set: &crate::build::sets::SetLayers) -> impl Iterator<Item = &Layer> {
        set.layers
            .iter()
            .filter(move |l| holds_stairs(set.drawing, &l.name))
    }
    let cadastre: Vec<CadastreStairs<'_>> = stored
        .sets
        .iter()
        .filter_map(|set| {
            let layers: Vec<(&str, &[Value])> = stair_layers(set)
                .map(|l| (l.name.as_str(), l.features.as_slice()))
                .collect();
            (!layers.is_empty()).then_some(CadastreStairs {
                drawing: set.drawing,
                crs: set.crs,
                layers,
            })
        })
        .collect();
    // The NAS stair areas and bottom edges are in one layer.
    let nas = layer(&stored.nas, NAS_STAIR_LAYER);
    // Canopies are cut at the 521 footprints; flat entrance steps face them.
    let mut canopies = Vec::new();
    for set in &stored.sets {
        for layer in set
            .layers
            .iter()
            .filter(|l| set.drawing.canopy_areas.iter().any(|a| a.layer == l.name))
        {
            canopies.extend(canopy_objects(
                &[CanopyInput::Set {
                    drawing: set.drawing,
                    layer: &layer.name,
                    features: &layer.features,
                    crs: set.crs,
                }],
                footprints,
                map_area,
                *buffer_m,
                georef,
                symbol_set,
                layer.fetched,
            )?);
        }
    }
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
    // The stair areas of the sets that bring their rules, in their CRS.
    let set_stairs: Vec<SetStairs<'_>> = stored
        .sets
        .iter()
        .flat_map(|set| {
            set.stair_layers().map(move |layer| SetStairs {
                source: set.source(&layer.name),
                features: &layer.features,
                crs: set.crs,
                currency: layer.fetched,
            })
        })
        .collect();
    if cadastre.is_empty() && nas.is_none() && set_stairs.is_empty() {
        return Ok(Structures {
            canopies,
            stairs: Vec::new(),
        });
    }
    fn features(layer: Option<&Layer>) -> &[Value] {
        layer.map_or(&[], |l| &l.features)
    }
    // Both layers come from one fetch; the later date stands for the currency.
    let fetched = stored
        .sets
        .iter()
        .flat_map(stair_layers)
        .chain(nas)
        .map(|l| l.fetched)
        .chain(set_stairs.iter().map(|s| s.currency))
        .max()
        .expect("one of the layers is stored");
    let stairs = stair_objects(
        &StairInput {
            cadastre,
            nas: features(nas),
            sets: &set_stairs,
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
    // The sets of extra layers through their rules (ADR 0149), then the building parts of the
    // buildings (passages) for the sets that name a layer for them.
    let mut cadastre = Vec::new();
    for set in &stored.sets {
        cadastre.extend(crate::build::sets::set_objects(
            set, *map_kind, map_area, *buffer_m, georef, symbol_set,
        )?);
    }
    if let Some((buildings, fetched)) = buildings {
        cadastre.extend(crate::build::sets::building_part_objects(
            &buildings.features,
            fetched,
            *map_kind,
            map_area,
            *buffer_m,
            georef,
            symbol_set,
        )?);
    }
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
    let mut survey = StreetSurvey::default();
    for set in &stored.sets {
        let layers: Vec<StoredLayer<'_>> = set
            .layers
            .iter()
            .map(|l| (l.name.as_str(), l.features.as_slice(), l.fetched))
            .collect();
        survey.extend(StreetSurvey::of_set(
            set.drawing,
            set.crs,
            &layers,
            georef.crs(),
        ));
    }
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
/// the sources, resolves the duplicates between them (`resolve`) and puts the lists of the map
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
        rock_seen,
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
        rock_seen,
    };
    // The duplicates between the sources go and the points are generalized.
    let Resolved {
        points: generalized,
        canopies,
        stairs,
        out_of_bounds,
        paved,
        open_land,
    } = resolve(
        Drawn {
            osm_candidates,
            osm_map,
            cadastre,
            canopies: structures.canopies,
            stairs: structures.stairs,
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
        .chain(stairs)
        .chain(contour_objects)
        .chain(paved.candidates)
        .chain(open_land.candidates)
        .chain(water.candidates)
        .chain(out_of_bounds)
        .collect();
    let content = PaperRect::of_bbox(clip, georef)?;
    // North lines first: within their black they lie under walls and fences.
    let spacing =
        north_line_spacing(map_kind).on_paper(SymbolScale::new(map_kind, georef.scale_denominator));
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
    pub rock_seen: Option<Raster<f32>>,
}

/// What the sources drew, before the duplicates between them are resolved (`resolve`).
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
    /// The 532.x stair candidates of the cadastres and the survey ([`stair_objects`]).
    pub stairs: Vec<MapObject>,
    /// The areas that shall not be entered (520) of the land use.
    pub out_of_bounds: Vec<MapObject>,
    /// The paved areas and the open land of the land use; they are only looked at.
    pub paved: PavedObjects,
    pub open_land: OpenLandObjects,
}

/// What `resolve` leaves of the [`Drawn`] objects.
struct Resolved {
    /// The candidates of OSM (first) and of the cadastres, the group areas and the OSM map
    /// objects, after the point generalization.
    pub points: GeneralizedPoints,
    pub canopies: Vec<MapObject>,
    pub stairs: Vec<MapObject>,
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
        mut stairs,
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
    // An OSM stair on a stair of the cadastre is that stair, on the map and among the candidates
    // (`build/stairs.rs`).
    let scale = SymbolScale::new(*map_kind, georef.scale_denominator);
    let osm_candidates =
        drop_osm_stairs_on_cadastre(osm_candidates, &mut stairs, symbol_set, scale);
    let osm_map = drop_osm_stairs_on_cadastre(osm_map, &mut stairs, symbol_set, scale);
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
    let cadastre = crate::build::canopy::drop_roofs_under_canopies(cadastre, &mut canopies);
    // An object of a landscape model (the Basis-DLM) goes where the cadastre or OpenStreetMap,
    // on the map or among the candidates, has the same object (ADR 0135).
    let (cadastre, osm_candidates, osm_map) = drop_landscape(
        cadastre,
        osm_candidates,
        osm_map,
        symbol_set,
        georef.scale_denominator,
    );
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
    let osm_candidates =
        drop_open_land_under_alkis(osm_candidates, &open_land.map, georef, symbol_set);
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
        stairs,
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
    rock_seen: Arc<Option<Raster<f32>>>,
    /// The objects of the map and of the candidates, as the build put them.
    map: Vec<MapObject>,
    candidates: Vec<MapObject>,
    /// Where the point candidates stand among `candidates` ([`Built::point_candidates`]).
    points: Range<usize>,
    paved_borders: PavedBorders,
}

/// The objects of the map and of the candidates, placed by content settings.
pub struct Placed {
    /// The objects of the map, cleaned at the light level (`build/light.rs`).
    pub map: Vec<MapObject>,
    /// The candidates, and after them what the light level took off the map.
    pub candidates: Vec<MapObject>,
    /// The objects the training level starts from: the map before the light level and the
    /// candidates the training map draws.
    pub training: TrainingInput,
    /// What the light level did.
    pub light: LightReport,
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
            rock_seen,
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
            rock_seen: Arc::new(rock_seen),
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
        cut_vegetation_at_out_of_bounds(&mut map, &mut candidates, georef, symbol_set)?;
        // The cliffs on a forest map keep the norm's gaps, rock pillars become 206
        // (`build/cliffs.rs`); after the routing, which decides which cliffs are on the map.
        let rock = generalize_cliffs(
            &mut map,
            &mut candidates,
            Ground {
                terrain: self.terrain.as_ref().as_ref(),
                rock_seen: self.rock_seen.as_ref().as_ref(),
            },
            self.map_kind,
            georef,
            symbol_set,
            &Level::map(self.map_kind),
        );
        // The contours stop below the cliffs drawn and at the pillars; a stub shorter than a
        // contour may be goes. The knolls and depressions of the terrain on the rock are hidden.
        if !rock.stops.is_empty() {
            let min_contour_m = CONTOUR_MIN_LENGTH_MM
                .metres(SymbolScale::new(self.map_kind, georef.scale_denominator));
            for list in [&mut candidates, &mut map] {
                cut_lines_at_ways(
                    list,
                    &CONTOUR_CODES,
                    &rock.stops,
                    min_contour_m,
                    georef,
                    symbol_set,
                );
            }
        }
        hide_landforms_on_rock(&mut map, &mut candidates, &rock.mask, georef);
        // Otherwise the contours run through all symbols, paving and paths too, and are cut out
        // only at the small landforms and the stairways drawn (ISSprOM 2019-2, 101;
        // `build/line_cuts.rs`); after the rock, which hides some of the landforms.
        let min_piece_m = MIN_PIECE_MM.metres(georef.scale_denominator);
        let cut_outs = contour_cut_outs(&map, &candidates, self.map_kind, georef, symbol_set);
        for list in [&mut candidates, &mut map] {
            cut_lines_at_ways(
                list,
                &CONTOUR_CODES,
                &cut_outs,
                min_piece_m,
                georef,
                symbol_set,
            );
        }
        // The north lines stop at paving and paths (ISSprOM 2019-2 §4.6).
        let ways = way_footprints(&map, &candidates, &map, self.map_kind, georef, symbol_set);
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
        // The last step on the lines: the ticks of fences follow what lies beside them, those of
        // earth banks down the slope (`build/ticks.rs`).
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
        // The training map starts from the map as it is now (spec 2026-11-14, Order), before the
        // light level; the cliffs taken off the map stay out of it.
        let training = TrainingInput::of(&map, &candidates);
        reveal_generalized(&mut candidates);
        // Last, the light level: what is doubled or hidden on the map goes to the candidates,
        // and nothing moves (`build/light.rs`, ADR 0157). After the ticks are turned, so one
        // orientation serves the map and the training map; the light level turns no line.
        let (taken, light) = light_level(&mut map, self.map_kind, georef, symbol_set);
        candidates.extend(taken);
        Ok(Placed {
            map,
            candidates,
            training,
            light,
        })
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
    use crate::derive::landforms::{Landform, LandformKind};
    use crate::omap::MapObjectKind;
    use crate::omap::coord::MapCoord;
    use crate::sources::osm::OsmGeometry;

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
    fn forest_north_lines_are_20_mm_at_1_to_15000_and_300_m_on_the_ground() {
        let at = |kind, scale| {
            let s = SymbolScale::new(kind, scale);
            let spacing = north_line_spacing(kind);
            (spacing.on_paper(s).0, spacing.metres(s))
        };
        let (mm, m) = at(MapKind::Forest, 15_000);
        assert!(
            (mm - 20.0).abs() < 1e-9 && (m - 300.0).abs() < 1e-6,
            "{mm} {m}"
        );
        let (mm, m) = at(MapKind::Forest, 10_000);
        assert!(
            (mm - 30.0).abs() < 1e-9 && (m - 300.0).abs() < 1e-6,
            "{mm} {m}"
        );
        let (mm, m) = at(MapKind::Sprint, 4_000);
        assert!(
            (mm - 30.0).abs() < 1e-9 && (m - 120.0).abs() < 1e-6,
            "{mm} {m}"
        );
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
            rock_seen: None,
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
            stairs: Vec::new(),
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
        // The contour runs across the paving (ISSprOM 2019-2, 101).
        assert_eq!(count(&placed.candidates, f.id("101")), 1);
        // The paved area got its border.
        assert_eq!(count(&placed.map, f.id("501.5")), 1);
        assert!(count(&placed.map, f.id("501.1")) >= 1);
        assert_eq!((built.map.clone(), built.candidates.clone()), before);
    }

    /// `map` and `candidates` of a fixture of `kind` built and placed with the safe profile, the
    /// groups `places` sent where they say.
    fn placed_with(
        f: &Fixture,
        map: Vec<MapObject>,
        candidates: Vec<MapObject>,
        places: &[(Group, Place)],
    ) -> Placed {
        let mut settings = ContentSettings::with_profile(Profile::Safe);
        for &(group, place) in places {
            settings.set(group, Some(place));
        }
        Built::new(surroundings(f), map, candidates, &settings)
            .place(&settings)
            .unwrap()
    }

    fn point(f: &Fixture, code: &str, at: (f64, f64)) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![f.at(at)],
            tags: Vec::new(),
        }
    }

    fn of_symbol(objects: &[MapObject], id: u32) -> Vec<&MapObject> {
        objects.iter().filter(|o| o.symbol_id == id).collect()
    }

    #[test]
    fn the_light_level_moves_a_hidden_bank_to_the_candidates_and_the_training_map_starts_before_it()
    {
        use crate::build::candidate_layers::{CandidateLayer, split};
        use crate::build::points::HIDDEN_KEY;
        let f = fixture(MapKind::Forest);
        let way = path(&f, "505", &[(-50.0, 0.0), (100.0, 0.0)]);
        let bank = path(&f, "104", &[(0.0, 0.0), (60.0, 0.0)]);
        let first = square(&f, "401", (-150.0, -150.0), 20.0);
        let placed = placed_with(
            &f,
            vec![way.clone(), bank.clone()],
            vec![first],
            &[(Group::Paths, Place::Map), (Group::Slopes, Place::Map)],
        );
        let bank_id = f.id("104");
        assert!(of_symbol(&placed.map, bank_id).is_empty());
        let taken = of_symbol(&placed.candidates, bank_id);
        assert_eq!(taken.len(), 1);
        assert_eq!(taken[0].coords, bank.coords);
        assert!(taken[0].tags.iter().all(|(k, _)| k != HIDDEN_KEY));
        assert!(
            taken[0]
                .tags
                .iter()
                .any(|(k, v)| k == "freemapper:generalized" && v == "hidden by path")
        );
        assert_eq!(placed.light.gave_way, 1);
        // In the terrain layer, and the training map starts from the map before it.
        let layers = split(&placed.candidates, &f.set);
        let terrain = &layers
            .iter()
            .find(|(layer, _)| *layer == CandidateLayer::Terrain)
            .unwrap()
            .1;
        assert!(terrain.contains(taken[0]));
        assert_eq!(of_symbol(&placed.training.map, bank_id).len(), 1);
        assert!(of_symbol(&placed.training.candidates, bank_id).is_empty());
    }

    #[test]
    fn a_contour_runs_across_a_path_and_a_paved_area() {
        let f = fixture(MapKind::Sprint);
        let contour = path(&f, "101", &[(-40.0, 20.0), (80.0, 20.0)]);
        // A paved square and a footpath across the contour, on the map and among the candidates.
        let placed = placed_with(
            &f,
            vec![
                square(&f, "501.5", (0.0, 0.0), 40.0),
                path(&f, "506", &[(60.0, -20.0), (60.0, 60.0)]),
            ],
            vec![
                contour.clone(),
                path(&f, "505.1", &[(-20.0, -20.0), (-20.0, 60.0)]),
            ],
            &[
                (Group::Contours, Place::Candidates),
                (Group::Paths, Place::Map),
            ],
        );
        assert_eq!(of_symbol(&placed.candidates, f.id("101")), [&contour]);
        // On the map as well.
        let placed = placed_with(
            &f,
            vec![
                square(&f, "501.5", (0.0, 0.0), 40.0),
                path(&f, "506", &[(60.0, -20.0), (60.0, 60.0)]),
            ],
            vec![contour.clone()],
            &[(Group::Contours, Place::Map), (Group::Paths, Place::Map)],
        );
        assert_eq!(of_symbol(&placed.map, f.id("101")), [&contour]);
    }

    #[test]
    fn a_contour_stops_at_a_stair() {
        let f = fixture(MapKind::Sprint);
        // A stair 4 m wide on paper's 1 mm (532.9) across a contour on the map, and one across a
        // contour among the candidates.
        let placed = placed_with(
            &f,
            vec![
                path(&f, "532.9", &[(0.0, 10.0), (0.0, 30.0)]),
                path(&f, "101", &[(-40.0, 20.0), (40.0, 20.0)]),
            ],
            vec![
                path(&f, "532.6", &[(100.0, 10.0), (100.0, 30.0)]),
                path(&f, "102", &[(60.0, 20.0), (140.0, 20.0)]),
            ],
            &[
                (Group::Stairs, Place::Map),
                (Group::Contours, Place::Candidates),
            ],
        );
        // Both contours are cut, one piece on each side of the stair.
        let contours: Vec<&MapObject> = placed
            .candidates
            .iter()
            .filter(|o| [f.id("101"), f.id("102")].contains(&o.symbol_id))
            .collect();
        assert_eq!(contours.len(), 4, "{contours:?}");
        // The gap is the stair's width: about 4 m for the one, 1.6 m for the other.
        let ends = |id: u32| -> Vec<f64> {
            let mut xs: Vec<f64> = of_symbol(&placed.candidates, id)
                .iter()
                .flat_map(|o| [o.coords[0], *o.coords.last().unwrap()])
                .map(|c| f64::from(c.x) / f.metre())
                .collect();
            xs.sort_by(f64::total_cmp);
            xs
        };
        let gap = |xs: &[f64]| xs[2] - xs[1];
        assert!(
            (gap(&ends(f.id("101"))) - 4.0).abs() < 0.1,
            "{:?}",
            ends(f.id("101"))
        );
        assert!(
            (gap(&ends(f.id("102"))) - 1.6).abs() < 0.1,
            "{:?}",
            ends(f.id("102"))
        );
    }

    #[test]
    fn a_contour_stops_at_a_small_knoll() {
        let f = fixture(MapKind::Sprint);
        // A knoll on the contour among the candidates; a depression and a pit beside the one on
        // the map, an earth wall across it.
        let placed = placed_with(
            &f,
            vec![
                path(&f, "101", &[(-40.0, 20.0), (40.0, 20.0)]),
                point(&f, "111", (0.0, 40.0)),
                path(&f, "105", &[(20.0, 10.0), (20.0, 30.0)]),
            ],
            vec![
                point(&f, "109", (0.0, 0.0)),
                path(&f, "101", &[(-40.0, 0.0), (40.0, 0.0)]),
            ],
            &[
                (Group::Contours, Place::Map),
                (Group::Landforms, Place::Candidates),
                (Group::Slopes, Place::Map),
            ],
        );
        let contours = of_symbol(&placed.map, f.id("101"));
        // The one through the knoll is cut there, the other one at the earth wall; the depression
        // 20 m away does not touch it.
        assert_eq!(contours.len(), 4, "{contours:?}");
        let on = |n: f64| {
            contours
                .iter()
                .filter(|o| o.coords[0].y == f.at((0.0, n)).y)
                .count()
        };
        assert_eq!((on(0.0), on(20.0)), (2, 2));
    }

    #[test]
    fn a_contour_crosses_an_earth_wall_on_a_forest_map() {
        // ISOM 2017-2: contours cross the earth wall (105), and it is no small earth wall.
        let f = fixture(MapKind::Forest);
        let placed = placed_with(
            &f,
            vec![
                path(&f, "101", &[(-200.0, 0.0), (200.0, 0.0)]),
                path(&f, "105", &[(0.0, -50.0), (0.0, 50.0)]),
            ],
            Vec::new(),
            &[(Group::Contours, Place::Map), (Group::Slopes, Place::Map)],
        );
        assert_eq!(of_symbol(&placed.map, f.id("101")).len(), 1);
    }

    #[test]
    fn a_north_line_still_stops_at_a_path() {
        let f = fixture(MapKind::Sprint);
        let placed = placed_with(
            &f,
            vec![
                path(&f, "601", &[(0.0, -40.0), (0.0, 80.0)]),
                square(&f, "501.5", (-20.0, 0.0), 40.0),
            ],
            vec![path(&f, "506", &[(-40.0, 60.0), (40.0, 60.0)])],
            &[(Group::Paths, Place::Candidates)],
        );
        // Cut at the paving and at the path: three pieces.
        assert_eq!(count(&placed.map, f.id("601")), 3);
    }

    #[test]
    fn the_cut_below_a_cliff_stays() {
        let f = fixture(MapKind::Forest);
        // A cliff across a contour; without a terrain model it stays as drawn.
        let placed = placed_with(
            &f,
            vec![
                path(&f, "201", &[(-60.0, 0.0), (60.0, 0.0)]),
                path(&f, "101", &[(0.0, -100.0), (0.0, 100.0)]),
            ],
            Vec::new(),
            &[(Group::Contours, Place::Map), (Group::Slopes, Place::Map)],
        );
        assert_eq!(count(&placed.map, f.id("201")), 1);
        assert_eq!(count(&placed.map, f.id("101")), 2);
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

    /// The projected point `de` m east and `dn` m north of the reference point.
    fn projected(f: &Fixture, (de, dn): (f64, f64)) -> (f64, f64) {
        let reference = f.georef.ref_projected;
        (reference.easting + de, reference.northing + dn)
    }

    /// The geographic position of that point (iterating the forward projection).
    fn lat_lon(f: &Fixture, at: (f64, f64)) -> LatLon {
        let (easting, northing) = projected(f, at);
        let mut p = LatLon {
            lat: 52.4876,
            lon: 13.3801,
        };
        for _ in 0..6 {
            let q = crate::crs::utm33_forward(p);
            p.lat += (northing - q.northing) / 111_200.0;
            p.lon += (easting - q.easting) / (111_200.0 * p.lat.to_radians().cos());
        }
        p
    }

    /// The ring of a square of `side` m with its south-west corner at `from`.
    fn ring(f: &Fixture, from: (f64, f64), side: f64) -> Polygon<f64> {
        let (w, s) = projected(f, from);
        let ring = vec![
            (w, s),
            (w + side, s),
            (w + side, s + side),
            (w, s + side),
            (w, s),
        ];
        Polygon::new(ring.into(), vec![])
    }

    /// A square parcel of the land use.
    fn parcel(f: &Fixture, oid: &str, nutzart: &str, from: (f64, f64), side: f64) -> LandUse {
        LandUse {
            oid: oid.to_string(),
            nutzart: nutzart.to_string(),
            bez: None,
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(vec![ring(f, from, side)]),
        }
    }

    fn osm_feature(id: &str, tags: &[(&str, &str)], geometry: OsmGeometry) -> OsmFeature {
        OsmFeature {
            id: id.to_string(),
            tags: tags
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            geometry,
        }
    }

    fn osm_line(f: &Fixture, id: &str, tags: &[(&str, &str)], points: &[(f64, f64)]) -> OsmFeature {
        let points = points.iter().map(|&p| lat_lon(f, p)).collect();
        osm_feature(id, tags, OsmGeometry::Line(points))
    }

    /// The stored OSM candidates, as of an extract of 2026-11-01.
    fn osm_extract(features: Vec<OsmFeature>) -> Option<(Vec<OsmFeature>, NaiveDate)> {
        Some((features, "2026-11-01".parse().unwrap()))
    }

    /// The objects of `source_id` among `objects`.
    fn of_source<'a>(objects: &'a [MapObject], source_id: &str) -> Vec<&'a MapObject> {
        objects
            .iter()
            .filter(|o| source_ids(std::slice::from_ref(o)) == [source_id])
            .collect()
    }

    fn symbols_of(objects: &[MapObject], source_id: &str) -> Vec<u32> {
        of_source(objects, source_id)
            .iter()
            .map(|o| o.symbol_id)
            .collect()
    }

    /// Builds the sprint fixture from `stored` and places it with the profile `safe`.
    fn placed_safe(f: &Fixture, stored: Stored) -> Placed {
        let safe = ContentSettings::with_profile(Profile::Safe);
        build(frame(f), stored, &safe)
            .unwrap()
            .place(&safe)
            .unwrap()
    }

    #[test]
    fn a_firm_gravel_footway_is_cut_at_the_street_of_the_land_use() {
        let f = fixture(MapKind::Sprint);
        // A firm-gravel footway (ADR 0079: 501.6, a map object) from the open into a street
        // parcel 40 to 10 m west of the reference point: east from 80 m to 25 m west of it.
        let east_end = |land_use: Vec<LandUse>| {
            let footway = osm_line(
                &f,
                "way/5",
                &[("highway", "footway"), ("surface", "compacted")],
                &[(-80.0, 5.0), (-25.0, 5.0)],
            );
            let placed = placed_safe(
                &f,
                Stored {
                    land_use,
                    osm: osm_extract(vec![footway]),
                    ..Stored::default()
                },
            );
            let pieces = of_source(&placed.map, "way/5");
            assert_eq!(pieces.len(), 1, "one piece, in the open");
            assert_eq!(pieces[0].symbol_id, f.id("501.6"));
            pieces[0].coords.last().unwrap().x
        };
        let open = east_end(Vec::new());
        let street = parcel(
            &f,
            "DEBBAL0100000030",
            "Straßenverkehr",
            (-40.0, -10.0),
            30.0,
        );
        let cut = east_end(vec![street]);
        // The street starts 15 m before the way's end (3.75 mm on paper at 1:4000): the cut
        // piece ends clearly before the whole one.
        assert!(open - cut > 2_000, "{open} {cut}");
    }

    #[test]
    fn an_unpaved_street_keeps_its_parcel_a_candidate() {
        let f = fixture(MapKind::Sprint);
        let street = || {
            parcel(
                &f,
                "DEBBAL0100000030",
                "Straßenverkehr",
                (-40.0, -10.0),
                30.0,
            )
        };
        let paved = f.id("501.5");
        let placed = placed_safe(
            &f,
            Stored {
                land_use: vec![street()],
                ..Stored::default()
            },
        );
        assert_eq!(symbols_of(&placed.map, "DEBBAL0100000030"), [paved]);
        // A sand road across the parcel: not paved, the parcel is left to check.
        let road = osm_line(
            &f,
            "way/3",
            &[("highway", "residential"), ("surface", "sand")],
            &[(-50.0, 5.0), (0.0, 5.0)],
        );
        let placed = placed_safe(
            &f,
            Stored {
                land_use: vec![street()],
                osm: osm_extract(vec![road]),
                ..Stored::default()
            },
        );
        assert!(symbols_of(&placed.map, "DEBBAL0100000030").is_empty());
        assert_eq!(symbols_of(&placed.candidates, "DEBBAL0100000030"), [paved]);
    }

    #[test]
    fn fountain_jets_in_a_basin_are_left_out_of_the_candidates() {
        let f = fixture(MapKind::Sprint);
        // A lake of the land use 40 to 10 m west of the reference point, and an OSM pond 20 to
        // 30 m east of it, which stays a candidate.
        let lake = parcel(
            &f,
            "DEBBAL0100000040",
            "Stehendes Gewässer",
            (-40.0, -10.0),
            30.0,
        );
        let pond = osm_line(
            &f,
            "way/11",
            &[("natural", "water")],
            &[
                (20.0, 0.0),
                (30.0, 0.0),
                (30.0, 10.0),
                (20.0, 10.0),
                (20.0, 0.0),
            ],
        );
        let node = |id: &str, tag: (&str, &str), at: (f64, f64)| {
            osm_feature(id, &[tag], OsmGeometry::Point(lat_lon(&f, at)))
        };
        let placed = placed_safe(
            &f,
            Stored {
                land_use: vec![lake],
                osm: osm_extract(vec![
                    pond,
                    node("node/20", ("amenity", "fountain"), (-25.0, 5.0)),
                    node("node/21", ("amenity", "fountain"), (25.0, 5.0)),
                    node("node/22", ("man_made", "water_well"), (26.0, 6.0)),
                    node("node/23", ("amenity", "fountain"), (5.0, 0.0)),
                ]),
                ..Stored::default()
            },
        );
        assert_eq!(symbols_of(&placed.candidates, "way/11"), [f.id("301")]);
        // In the lake, in the pond, a well in the pond: no fountain. In the open: one.
        for jet in ["node/20", "node/21", "node/22"] {
            assert!(symbols_of(&placed.candidates, jet).is_empty(), "{jet}");
            assert!(symbols_of(&placed.map, jet).is_empty(), "{jet}");
        }
        assert_eq!(symbols_of(&placed.candidates, "node/23"), [f.id("311")]);
    }

    /// A fountain of the landscape model, as the layer `dlm-bauwerk` stores it.
    fn model_fountain(f: &Fixture, oid: &str, at: (f64, f64)) -> Value {
        let (easting, northing) = projected(f, at);
        serde_json::json!({
            "type": "Feature",
            "id": oid,
            "geometry": {"type": "Point", "coordinates": [easting, northing]},
            "properties": {
                "bauwerksfunktion": "1780",
                "beginnt": "2025-07-03",
                "nas_type": "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                "oid": oid,
            },
        })
    }

    #[test]
    fn a_fountain_of_the_landscape_model_goes_beside_the_osm_fountain() {
        let f = fixture(MapKind::Sprint);
        // An OSM fountain 5 m east of the reference point; the model has it 8 m further east,
        // and another fountain 40 m north that no other source has.
        let fountain = osm_feature(
            "node/23",
            &[("amenity", "fountain")],
            OsmGeometry::Point(lat_lon(&f, (5.0, 0.0))),
        );
        let placed = placed_safe(
            &f,
            Stored {
                nas: vec![Layer {
                    name: "dlm-bauwerk".to_string(),
                    features: vec![
                        model_fountain(&f, "DEBYBDLMNO000GU9", (13.0, 0.0)),
                        model_fountain(&f, "DEBYBDLMNO000GV0", (5.0, 40.0)),
                    ],
                    fetched: "2026-11-14".parse().unwrap(),
                }],
                osm: osm_extract(vec![fountain]),
                ..Stored::default()
            },
        );
        let spring = f.id("311");
        assert!(symbols_of(&placed.candidates, "DEBYBDLMNO000GU9").is_empty());
        assert_eq!(symbols_of(&placed.candidates, "DEBYBDLMNO000GV0"), [spring]);
        let kept = of_source(&placed.candidates, "node/23");
        assert_eq!(kept.len(), 1);
        assert!(
            kept[0].tags.contains(&(
                "freemapper:also".to_string(),
                "atkis-dlm/bauwerk/DEBYBDLMNO000GU9".to_string()
            )),
            "{:?}",
            kept[0].tags
        );
    }

    #[test]
    fn landforms_leave_built_land_buildings_and_roads() {
        let f = fixture(MapKind::Sprint);
        // A school 5 to 15 m east and 30 to 40 m north of the reference point, houses west of
        // it, a street south-east of it.
        let school = Building {
            oid: "DEBBAL0100000001".to_string(),
            kind: "Gebäude".to_string(),
            function: "Allgemein bildende Schule".to_string(),
            underground: false,
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(vec![ring(&f, (5.0, 30.0), 10.0)]),
        };
        let point = |kind, at: (f64, f64), height_m| Landform {
            kind,
            points: vec![projected(&f, at)],
            rotation_deg: 0.0,
            height_m,
        };
        let cliff = |a: (f64, f64), b: (f64, f64), height_m| Landform {
            kind: LandformKind::Cliff,
            points: vec![projected(&f, a), projected(&f, b)],
            rotation_deg: 0.0,
            height_m,
        };
        let forms = vec![
            // Among the houses: a heap or a garden mound.
            point(LandformKind::Knoll, (-30.0, 0.0), 1.1),
            point(LandformKind::Knoll, (25.0, 0.0), 1.2),
            // 5 m east of the school: a light well.
            point(LandformKind::Depression, (20.0, 35.0), 1.3),
            point(LandformKind::Depression, (32.0, 20.0), 1.4),
            // Along the north edge of the street: a retaining wall, offered as an impassable
            // wall (515).
            cliff((20.0, -29.0), (40.0, -29.0), 3.5),
            cliff((-20.0, 20.0), (-2.0, 20.0), 3.6),
        ];
        let placed = placed_safe(
            &f,
            Stored {
                buildings: Some(Buildings {
                    buildings: vec![school],
                    features: Vec::new(),
                    fetched: "2026-11-01".parse().ok(),
                }),
                land_use: vec![
                    parcel(
                        &f,
                        "DEBBAL0100000041",
                        "Wohnbaufläche",
                        (-40.0, -10.0),
                        20.0,
                    ),
                    parcel(
                        &f,
                        "DEBBAL0100000042",
                        "Straßenverkehr",
                        (20.0, -50.0),
                        20.0,
                    ),
                ],
                landforms: Some(Landforms {
                    forms,
                    source: "lgb-dgm1".to_string(),
                    currency: None,
                }),
                ..Stored::default()
            },
        );
        // (height, symbol) of every landform candidate.
        let mut found: Vec<(&str, u32)> = placed
            .candidates
            .iter()
            .filter_map(|o| {
                let height = o.tags.iter().find(|(k, _)| k == "freemapper:height")?;
                Some((height.1.as_str(), o.symbol_id))
            })
            .collect();
        found.sort();
        assert_eq!(
            found,
            [
                ("1.2", f.id("109")),
                ("1.4", f.id("111")),
                ("3.5", f.id("515")),
                ("3.6", f.id("201")),
            ]
        );
    }

    /// A knoll on bare rock is a boulder (204) from 1 m, a large boulder (205) from 2 m; a lower one,
    /// and a knoll off the rock, stay knolls (ADR 0199).
    #[test]
    fn a_knoll_on_bare_rock_is_a_boulder() {
        let f = fixture(MapKind::Forest);
        let rock = osm_feature(
            "way/1",
            &[("natural", "bare_rock")],
            OsmGeometry::Area(vec![crate::sources::osm::OsmPolygon {
                outer: [
                    (0.0, 0.0),
                    (40.0, 0.0),
                    (40.0, 40.0),
                    (0.0, 40.0),
                    (0.0, 0.0),
                ]
                .iter()
                .map(|&p| lat_lon(&f, p))
                .collect(),
                holes: Vec::new(),
            }]),
        );
        let form = |kind, at: (f64, f64), height_m| Landform {
            kind,
            points: vec![projected(&f, at)],
            rotation_deg: 0.0,
            height_m,
        };
        let forms = vec![
            form(LandformKind::Knoll, (5.0, 5.0), 0.8),
            form(LandformKind::Knoll, (15.0, 5.0), 1.0),
            form(LandformKind::Knoll, (25.0, 5.0), 1.4),
            form(LandformKind::Knoll, (35.0, 5.0), 2.0),
            form(LandformKind::Knoll, (5.0, 15.0), 2.5),
            Landform {
                rotation_deg: 40.0,
                ..form(LandformKind::ElongatedKnoll, (15.0, 15.0), 3.5)
            },
            Landform {
                rotation_deg: 40.0,
                ..form(LandformKind::ElongatedKnoll, (80.0, 20.0), 1.5)
            },
            form(LandformKind::Knoll, (80.0, 10.0), 3.0),
            form(LandformKind::Depression, (30.0, 30.0), 3.0),
        ];
        let placed = placed_safe(
            &f,
            Stored {
                osm: osm_extract(vec![rock]),
                landforms: Some(Landforms {
                    forms,
                    source: "lgb-dgm1".to_string(),
                    currency: None,
                }),
                ..Stored::default()
            },
        );
        let mut found: Vec<(&str, u32)> = placed
            .candidates
            .iter()
            .filter_map(|o| {
                let height = o.tags.iter().find(|(k, _)| k == "freemapper:height")?;
                Some((height.1.as_str(), o.symbol_id))
            })
            .collect();
        found.sort();
        let expected = vec![
            ("0.8", f.id("109")),
            ("1.0", f.id("204")),
            ("1.4", f.id("204")),
            ("1.5", f.id("110")),
            ("2.0", f.id("205")),
            ("2.5", f.id("205")),
            ("3.0", f.id("109")),
            ("3.0", f.id("111")),
            ("3.5", f.id("205")),
        ];
        assert_eq!(found, expected);
        // The elongated knoll off the rock is turned, the block on it is not.
        let rotation = |height: &str| {
            placed.candidates.iter().find_map(|o| {
                let tagged = o
                    .tags
                    .contains(&("freemapper:height".to_string(), height.to_string()));
                match o.kind {
                    MapObjectKind::Point { rotation_rad } if tagged => Some(rotation_rad),
                    _ => None,
                }
            })
        };
        assert_ne!(rotation("1.5"), Some(0.0));
        assert_eq!(rotation("3.5"), Some(0.0));
    }

    /// Boulders closer than their symbols allow (0.55 mm centre to centre, 8.25 m at 1:15 000) are
    /// a boulder cluster (207) from three in a chain on; two stay single (ADR 0199).
    #[test]
    fn boulders_too_close_for_their_symbols_are_a_cluster() {
        let f = fixture(MapKind::Forest);
        let rock = osm_feature(
            "way/1",
            &[("natural", "bare_rock")],
            OsmGeometry::Area(vec![crate::sources::osm::OsmPolygon {
                outer: [
                    (0.0, 0.0),
                    (60.0, 0.0),
                    (60.0, 60.0),
                    (0.0, 60.0),
                    (0.0, 0.0),
                ]
                .iter()
                .map(|&p| lat_lon(&f, p))
                .collect(),
                holes: Vec::new(),
            }]),
        );
        let boulder = |at: (f64, f64), height_m| Landform {
            kind: LandformKind::Knoll,
            points: vec![projected(&f, at)],
            rotation_deg: 0.0,
            height_m,
        };
        let forms = vec![
            // Three 4 m apart: one cluster as high as the highest.
            boulder((5.0, 5.0), 1.2),
            boulder((9.0, 5.0), 2.5),
            boulder((13.0, 5.0), 1.4),
            // Two 4 m apart: both stay.
            boulder((40.0, 5.0), 1.5),
            boulder((44.0, 5.0), 1.6),
            // A chain of four, 7 m apart: the ends are 21 m apart and still one cluster.
            boulder((5.0, 25.0), 3.1),
            boulder((19.0, 25.0), 3.3),
            boulder((12.0, 25.0), 3.2),
            boulder((26.0, 25.0), 3.4),
            // Alone, far from the others.
            boulder((5.0, 50.0), 1.9),
        ];
        let placed = placed_safe(
            &f,
            Stored {
                osm: osm_extract(vec![rock]),
                landforms: Some(Landforms {
                    forms,
                    source: "lgb-dgm1".to_string(),
                    currency: None,
                }),
                ..Stored::default()
            },
        );
        let mut found: Vec<(&str, u32)> = placed
            .candidates
            .iter()
            .filter_map(|o| {
                let height = o.tags.iter().find(|(k, _)| k == "freemapper:height")?;
                Some((height.1.as_str(), o.symbol_id))
            })
            .collect();
        found.sort();
        let expected = vec![
            ("1.5", f.id("204")),
            ("1.6", f.id("204")),
            ("1.9", f.id("204")),
            ("2.5", f.id("207")),
            ("3.4", f.id("207")),
        ];
        assert_eq!(found, expected);
        let counts: Vec<&str> = placed
            .candidates
            .iter()
            .filter_map(|o| {
                o.tags
                    .iter()
                    .find(|(k, _)| k == "freemapper:boulders")
                    .map(|(_, v)| v.as_str())
            })
            .collect();
        assert_eq!(counts.len(), 2);
        assert!(counts.contains(&"3") && counts.contains(&"4"), "{counts:?}");
        // A cluster stands in the middle of its boulders.
        let centre = |height: &str| {
            let object = placed
                .candidates
                .iter()
                .find(|o| {
                    o.tags
                        .contains(&("freemapper:height".to_string(), height.to_string()))
                        && o.symbol_id == f.id("207")
                })
                .unwrap();
            object.coords[0]
        };
        for (height, at) in [("2.5", (9.0, 5.0)), ("3.4", (15.5, 25.0))] {
            let (e, n) = projected(&f, at);
            let expected = f
                .georef
                .projected_to_map(crate::crs::ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap();
            let got = centre(height);
            assert!(
                (got.x - expected.x).abs() <= 2 && (got.y - expected.y).abs() <= 2,
                "{height}: {got:?} against {expected:?}"
            );
        }
    }

    /// Exactly the width of two symbols apart counts as too close: three boulders 0.55 mm on
    /// paper from each other make a cluster.
    #[test]
    fn boulders_exactly_one_gap_apart_are_a_cluster() {
        let mut f = fixture(MapKind::Forest);
        f.georef.grid_scale_factor = 1.0;
        let gap = 0.55 * f.georef.metres_per_map_mm();
        assert_eq!(gap * 4.0, (gap * 4.0).round(), "an exact gap: {gap}");
        let (east, north) = projected(&f, (5.0, 5.0));
        let (east, north) = (east.floor(), north.floor());
        let rock = osm_feature(
            "way/1",
            &[("natural", "bare_rock")],
            OsmGeometry::Area(vec![crate::sources::osm::OsmPolygon {
                outer: [
                    (0.0, 0.0),
                    (60.0, 0.0),
                    (60.0, 60.0),
                    (0.0, 60.0),
                    (0.0, 0.0),
                ]
                .iter()
                .map(|&p| lat_lon(&f, p))
                .collect(),
                holes: Vec::new(),
            }]),
        );
        let forms = (0..3)
            .map(|k| Landform {
                kind: LandformKind::Knoll,
                points: vec![(east + gap * f64::from(k), north)],
                rotation_deg: 0.0,
                height_m: 2.5,
            })
            .collect();
        let placed = placed_safe(
            &f,
            Stored {
                osm: osm_extract(vec![rock]),
                landforms: Some(Landforms {
                    forms,
                    source: "lgb-dgm1".to_string(),
                    currency: None,
                }),
                ..Stored::default()
            },
        );
        let found: Vec<u32> = placed
            .candidates
            .iter()
            .filter(|o| o.tags.iter().any(|(k, _)| k == "freemapper:height"))
            .map(|o| o.symbol_id)
            .collect();
        assert_eq!(found, [f.id("207")]);
    }
}
