//! What goes straight onto the map and what stays a candidate (ADR 0100).
//!
//! The build puts every object on the map or among the candidates by the rule that made it
//! (`target` in `assets/rules/*.toml`) or by the module that drew it. This step lets the user
//! decide after the fact, per group of objects: a [`Profile`] sets the default, and any
//! [`Group`] can be sent to the map, to the candidates or left out. The default, `balanced`,
//! puts the objects of the official sources (the cadastre ALKIS, the registers and the street
//! survey) on the map, and those of OpenStreetMap for the groups it maps well (walls, fences,
//! hedges, water): a wall that the cadastre surveyed is as reliable as a building.

use std::collections::HashMap;

use geo::Contains as _;
use serde::{Deserialize, Serialize};

use crate::build::dedup::{Rank, rank};
use crate::build::geometry::object_polygon;
use crate::build::points::{GENERALIZATION_SOURCE, HIDDEN_KEY};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};

/// The tag of a landform from the terrain model (knoll, depression, bank, cliff, retaining wall);
/// its value is the kind. Such objects belong to the group `landforms` whatever their symbol.
pub const LANDFORM_KEY: &str = "freemapper:landform";

/// The tag of an object made from a point cloud; its value is the product (`lgb-als`).
pub const POINT_CLOUD_KEY: &str = "freemapper:point-cloud";

/// The ALKIS land use whose areas that shall not be entered `balanced` always puts on the map.
const RAILWAY_LAND_USE: &str = "Bahnverkehr";

/// A default for all groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    /// Every object stays where its rule or module put it.
    Safe,
    /// The objects of the cadastre, the registers and the street survey go to the map, and those of
    /// OpenStreetMap for walls, fences, hedges and water.
    #[default]
    Balanced,
    /// Everything goes to the map, as in the training map.
    Full,
}

impl Profile {
    pub const ALL: [Profile; 3] = [Profile::Safe, Profile::Balanced, Profile::Full];

    pub fn name(self) -> &'static str {
        match self {
            Profile::Safe => "safe",
            Profile::Balanced => "balanced",
            Profile::Full => "full",
        }
    }

    pub fn from_name(name: &str) -> Option<Profile> {
        Profile::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// Where a group of objects goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Place {
    /// Onto the map.
    Map,
    /// Into the candidates template.
    Candidates,
    /// Nowhere.
    Off,
}

impl Place {
    pub const ALL: [Place; 3] = [Place::Map, Place::Candidates, Place::Off];

    pub fn name(self) -> &'static str {
        match self {
            Place::Map => "map",
            Place::Candidates => "candidates",
            Place::Off => "off",
        }
    }

    pub fn from_name(name: &str) -> Option<Place> {
        Place::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// A group of objects that the user can send somewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    /// Walls (515) and passable retained walls (513.2).
    Walls,
    /// Fences (518).
    Fences,
    /// Hedges (410.1).
    Hedges,
    /// Earth banks (104, 104.1) and cliffs (201).
    Slopes,
    /// Stairs (532.6 to 532.9).
    Stairs,
    /// Trees (417, 418) and the groups made of them (402, 405, 501.3).
    Trees,
    /// Brooks (305, 306), fountains, springs and prominent water features (311 to 313).
    WaterFeatures,
    /// Towers, monuments, power lines and prominent line features (510, 511, 524 to 526, 528, 529).
    Monuments,
    /// Hydrants, prominent features (531) and street furniture such as lamps (541 to 545).
    Furniture,
    /// Passages and arcades (522).
    Passages,
    /// Contours (101, 102).
    Contours,
    /// Everything `derive` finds in the terrain model: knolls, elongated knolls and depressions
    /// (109 to 111), and the banks, cliffs and retaining walls known by their tag
    /// (`LANDFORM_KEY`), which ALKIS and OpenStreetMap draw with the same symbols.
    Landforms,
    /// The vegetation areas from the laser point clouds (401, 405 to 410; ADR 0105). Off in
    /// every profile: only a choice of its own fetches the point clouds.
    Vegetation,
    /// Areas that shall not be entered (520).
    OutOfBounds,
    /// Paths and tracks (501.6 to 501.9, 505, 506, 509).
    Paths,
    /// Streets, squares and their kerbs (501.5, 501.10, 501.11 and the 501.1 of the survey).
    Streets,
}

impl Group {
    pub const ALL: [Group; 16] = [
        Group::Walls,
        Group::Fences,
        Group::Hedges,
        Group::Slopes,
        Group::Stairs,
        Group::Trees,
        Group::WaterFeatures,
        Group::Monuments,
        Group::Furniture,
        Group::Passages,
        Group::Contours,
        Group::Landforms,
        Group::Vegetation,
        Group::OutOfBounds,
        Group::Paths,
        Group::Streets,
    ];

    /// The name in `freemapper.toml`, on the command line and in the program catalog.
    pub fn name(self) -> &'static str {
        match self {
            Group::Walls => "walls",
            Group::Fences => "fences",
            Group::Hedges => "hedges",
            Group::Slopes => "slopes",
            Group::Stairs => "stairs",
            Group::Trees => "trees",
            Group::WaterFeatures => "water-features",
            Group::Monuments => "monuments",
            Group::Furniture => "furniture",
            Group::Passages => "passages",
            Group::Contours => "contours",
            Group::Landforms => "landforms",
            Group::Vegetation => "vegetation",
            Group::OutOfBounds => "out-of-bounds",
            Group::Paths => "paths",
            Group::Streets => "streets",
        }
    }

    pub fn from_name(name: &str) -> Option<Group> {
        Group::ALL.into_iter().find(|g| g.name() == name)
    }

    /// The ISSprOM codes of the group.
    fn codes(self) -> &'static [&'static str] {
        match self {
            Group::Walls => &["515", "513.2"],
            Group::Fences => &["518"],
            Group::Hedges => &["410.1"],
            Group::Slopes => &["104", "104.1", "201"],
            Group::Stairs => &["532.6", "532.7", "532.8", "532.9"],
            Group::Trees => &["417", "418"],
            Group::WaterFeatures => &["301", "305", "306", "311", "312", "313"],
            Group::Monuments => &["510", "511", "524", "525", "526", "528", "529"],
            Group::Furniture => &["321", "531", "541", "542", "543", "544", "545"],
            Group::Passages => &["522"],
            Group::Contours => &["101", "102"],
            Group::Landforms => &["109", "110", "111"],
            // Known by their tag (`POINT_CLOUD_KEY`), not their codes: ALKIS and OpenStreetMap
            // draw open land and forest with the same symbols.
            Group::Vegetation => &[],
            Group::OutOfBounds => &["520"],
            Group::Paths => &[
                "501.6", "501.7", "501.8", "501.9", "505.1", "505.2", "506", "507", "509.1",
                "509.2",
            ],
            Group::Streets => &["501.5", "501.10", "501.11"],
        }
    }

    /// The group areas of tree candidates (402, 405, 501.3 and the 416 boundary) that
    /// `generalize_points` makes belong to the trees, whatever the profile says about them.
    const TREE_AREA_CODES: [&'static str; 4] = ["402", "405", "501.3", "416"];

    /// The lowest rank of a source whose objects of this group the `balanced` profile puts on the
    /// map; `None` for a group it leaves as the build has it (areas that shall not be entered,
    /// street furniture, the knolls and depressions of the terrain model until their hit rate is
    /// measured) or sends by another rule (the contours always go to the map).
    /// OpenStreetMap counts for walls, fences, hedges, water, trees, monuments (towers, pipelines,
    /// power lines), banks, passages and paths, where it is often the only source; the rest needs the cadastre, a register or the street survey.
    fn balanced_from(self) -> Option<Rank> {
        match self {
            Group::Walls
            | Group::Fences
            | Group::Hedges
            | Group::WaterFeatures
            | Group::Trees
            | Group::Monuments
            | Group::Slopes
            | Group::Passages
            | Group::Paths
            | Group::Stairs => Some(Rank::Osm),
            Group::Streets => Some(Rank::Survey),
            Group::Furniture
            | Group::OutOfBounds
            | Group::Contours
            | Group::Landforms
            | Group::Vegetation => None,
        }
    }
}

/// The choice of a user: a profile and the groups that differ from it. In `freemapper.toml` a
/// table `[content]` with `profile = "balanced"` and keys such as `walls = "map"`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContentSettings {
    pub profile: Profile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walls: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fences: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hedges: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slopes: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stairs: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trees: Option<Place>,
    #[serde(rename = "water-features", skip_serializing_if = "Option::is_none")]
    pub water_features: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monuments: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub furniture: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passages: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contours: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landforms: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vegetation: Option<Place>,
    #[serde(rename = "out-of-bounds", skip_serializing_if = "Option::is_none")]
    pub out_of_bounds: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Place>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streets: Option<Place>,
}

impl ContentSettings {
    /// The profile `profile` without any exception.
    pub fn with_profile(profile: Profile) -> Self {
        Self {
            profile,
            ..Self::default()
        }
    }

    /// What the user chose for `group`, if anything.
    pub fn choice(&self, group: Group) -> Option<Place> {
        match group {
            Group::Walls => self.walls,
            Group::Fences => self.fences,
            Group::Hedges => self.hedges,
            Group::Slopes => self.slopes,
            Group::Stairs => self.stairs,
            Group::Trees => self.trees,
            Group::WaterFeatures => self.water_features,
            Group::Monuments => self.monuments,
            Group::Furniture => self.furniture,
            Group::Passages => self.passages,
            Group::Contours => self.contours,
            Group::Landforms => self.landforms,
            Group::Vegetation => self.vegetation,
            Group::OutOfBounds => self.out_of_bounds,
            Group::Paths => self.paths,
            Group::Streets => self.streets,
        }
    }

    /// Sends `group` to `place`; `None` takes the group back to the profile.
    pub fn set(&mut self, group: Group, place: Option<Place>) {
        let slot = match group {
            Group::Walls => &mut self.walls,
            Group::Fences => &mut self.fences,
            Group::Hedges => &mut self.hedges,
            Group::Slopes => &mut self.slopes,
            Group::Stairs => &mut self.stairs,
            Group::Trees => &mut self.trees,
            Group::WaterFeatures => &mut self.water_features,
            Group::Monuments => &mut self.monuments,
            Group::Furniture => &mut self.furniture,
            Group::Passages => &mut self.passages,
            Group::Contours => &mut self.contours,
            Group::Landforms => &mut self.landforms,
            Group::Vegetation => &mut self.vegetation,
            Group::OutOfBounds => &mut self.out_of_bounds,
            Group::Paths => &mut self.paths,
            Group::Streets => &mut self.streets,
        };
        *slot = place;
    }

    /// Parses `group=place` (a command-line pair) into the settings.
    pub fn set_from_pair(&mut self, pair: &str) -> Result<(), String> {
        let (group, place) = pair
            .split_once('=')
            .ok_or_else(|| format!("`{pair}` is not group=place"))?;
        let group =
            Group::from_name(group.trim()).ok_or_else(|| format!("unknown group `{group}`"))?;
        let place =
            Place::from_name(place.trim()).ok_or_else(|| format!("unknown place `{place}`"))?;
        self.set(group, Some(place));
        Ok(())
    }

    /// The settings a command line or a tool call asks for: `profile` (a profile name) starts
    /// afresh from that profile, `sets` (`group=place`) are added to it, or to `existing` when no
    /// profile is given. `None` when neither is given.
    pub fn from_request(
        profile: Option<&str>,
        sets: &[String],
        existing: Option<&ContentSettings>,
    ) -> Result<Option<ContentSettings>, String> {
        if profile.is_none() && sets.is_empty() {
            return Ok(None);
        }
        let mut settings = match profile {
            Some(name) => ContentSettings::with_profile(
                Profile::from_name(name.trim())
                    .ok_or_else(|| format!("unknown profile `{name}`"))?,
            ),
            None => existing.cloned().unwrap_or_default(),
        };
        for pair in sets {
            settings.set_from_pair(pair)?;
        }
        Ok(Some(settings))
    }

    /// The settings in one line, e.g. `balanced, walls=map`.
    pub fn summary(&self) -> String {
        let mut parts = vec![self.profile.name().to_string()];
        for group in Group::ALL {
            if let Some(place) = self.choice(group) {
                parts.push(format!("{}={}", group.name(), place.name()));
            }
        }
        parts.join(", ")
    }

    /// Whether the user chose to have the vegetation areas: only then does a fetch load the point
    /// clouds (about 160 MB per km² in Brandenburg) and `derive` read them.
    pub fn wants_vegetation(&self) -> bool {
        matches!(self.vegetation, Some(Place::Map | Place::Candidates))
    }

    /// Where an object of `group` with the source `rank` goes: `None` leaves it where the build
    /// put it. `railway` marks an area that shall not be entered around railway tracks.
    fn place(&self, group: Group, rank: Option<Rank>, railway: bool) -> Option<Place> {
        if let Some(choice) = self.choice(group) {
            return Some(choice);
        }
        // Not fetched unless chosen; an area left from an earlier choice goes.
        if group == Group::Vegetation {
            return Some(Place::Off);
        }
        match self.profile {
            Profile::Safe => None,
            Profile::Full => Some(Place::Map),
            // The contours come from the terrain model, which has no rank: they go to the map.
            Profile::Balanced if group == Group::Contours => Some(Place::Map),
            // The ground along the tracks is closed to everyone, not a guess about a garden.
            Profile::Balanced if group == Group::OutOfBounds && railway => Some(Place::Map),
            Profile::Balanced => group
                .balanced_from()
                .is_some_and(|lowest| rank >= Some(lowest))
                .then_some(Place::Map),
        }
    }
}

/// How many objects [`route`] moved or dropped.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Routed {
    pub to_map: usize,
    pub to_candidates: usize,
    pub dropped: usize,
}

/// Sends the objects of `map` and `candidates` by `settings`: an object of a group goes where
/// its group is sent (a moved object goes to the end of its new list). An object with no group,
/// a candidate that is hidden (`freemapper:hidden`) and an object the settings leave alone stay.
pub fn route(
    settings: &ContentSettings,
    map: &mut Vec<MapObject>,
    candidates: &mut Vec<MapObject>,
    symbol_set: &SymbolSet,
) -> Routed {
    let mut group_of: HashMap<u32, Group> = HashMap::new();
    let mut area_id: HashMap<u32, ()> = HashMap::new();
    for group in Group::ALL {
        for code in group.codes() {
            if let Some(id) = symbol_set.id_for_code(code) {
                group_of.insert(id, group);
            }
        }
    }
    for code in Group::TREE_AREA_CODES {
        if let Some(id) = symbol_set.id_for_code(code) {
            area_id.insert(id, ());
        }
    }
    // The edge of a paved area (501.1) of the street survey or the cadastre belongs to the
    // streets; one from OpenStreetMap is the outline of the map area.
    let edge_id = symbol_set.id_for_code("501.1");
    let open_land_id = symbol_set.id_for_code("401");
    let out_of_bounds_id = symbol_set.id_for_code("520");
    let tag = |o: &MapObject, key: &str| -> Option<String> {
        o.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    let classify = |o: &MapObject| -> Option<Group> {
        if o.tags.iter().any(|(k, _)| k == HIDDEN_KEY) {
            return None;
        }
        if o.tags.iter().any(|(k, _)| k == LANDFORM_KEY) {
            return Some(Group::Landforms);
        }
        if o.tags.iter().any(|(k, _)| k == POINT_CLOUD_KEY) {
            return Some(Group::Vegetation);
        }
        if Some(o.symbol_id) == edge_id {
            return (rank(o) > Some(Rank::Osm)).then_some(Group::Streets);
        }
        // The green strips along the streets, as the street survey measured them.
        if Some(o.symbol_id) == open_land_id {
            return (rank(o) == Some(Rank::Survey)).then_some(Group::Streets);
        }
        group_of.get(&o.symbol_id).copied().or_else(|| {
            (area_id.contains_key(&o.symbol_id)
                && tag(o, "freemapper:source").as_deref() == Some(GENERALIZATION_SOURCE))
            .then_some(Group::Trees)
        })
    };
    let mut routed = Routed::default();
    let mut moves: Vec<(bool, MapObject)> = Vec::new();
    for (on_map, list) in [(true, &mut *map), (false, &mut *candidates)] {
        let mut kept = Vec::with_capacity(list.len());
        for object in list.drain(..) {
            let group = classify(&object);
            // A tree group is as good as the trees it was made of: OpenStreetMap at the least.
            let source_rank = rank(&object).or_else(|| {
                (group == Some(Group::Trees) && area_id.contains_key(&object.symbol_id))
                    .then_some(Rank::Osm)
            });
            let railway = group == Some(Group::OutOfBounds)
                && tag(&object, "freemapper:nutzart")
                    .is_some_and(|nutzart| nutzart.starts_with(RAILWAY_LAND_USE));
            let place = group.and_then(|g| settings.place(g, source_rank, railway));
            match place {
                Some(Place::Off) => routed.dropped += 1,
                Some(Place::Map) if !on_map => {
                    routed.to_map += 1;
                    moves.push((true, object));
                }
                Some(Place::Candidates) if on_map => {
                    routed.to_candidates += 1;
                    moves.push((false, object));
                }
                _ => kept.push(object),
            }
        }
        *list = kept;
    }
    // A 520 on the map takes every point in it with it (ISSprOM 4.5), as in `generalize_points`.
    let closed: Vec<geo_types::Polygon<f64>> = out_of_bounds_id
        .into_iter()
        .flat_map(|id| {
            moves
                .iter()
                .filter(move |(to_map, o)| *to_map && o.symbol_id == id)
                .filter_map(|(_, o)| object_polygon(o))
        })
        .collect();
    for (to_map, object) in moves {
        if to_map {
            map.push(object);
        } else {
            candidates.push(object);
        }
    }
    if !closed.is_empty() {
        let inside = |o: &MapObject| {
            matches!(o.kind, MapObjectKind::Point { .. })
                && o.coords.first().is_some_and(|c| {
                    let at = geo_types::Point::new(f64::from(c.x), f64::from(c.y));
                    closed.iter().any(|area| area.contains(&at))
                })
        };
        for list in [&mut *map, &mut *candidates] {
            let before = list.len();
            list.retain(|o| !inside(o));
            routed.dropped += before - list.len();
        }
    }
    routed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::omap::MapObjectKind;
    use crate::project::MapKind;

    fn object(f: &Fixture, code: &str, source: &str) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: vec![f.at((0.0, 0.0)), f.at((10.0, 0.0))],
            tags: vec![("freemapper:source".to_string(), source.to_string())],
        }
    }

    fn run(
        f: &Fixture,
        settings: &ContentSettings,
        map: Vec<MapObject>,
        candidates: Vec<MapObject>,
    ) -> (Vec<MapObject>, Vec<MapObject>, Routed) {
        let (mut map, mut candidates) = (map, candidates);
        let routed = route(settings, &mut map, &mut candidates, &f.set);
        (map, candidates, routed)
    }

    #[test]
    fn names_round_trip() {
        for g in Group::ALL {
            assert_eq!(Group::from_name(g.name()), Some(g));
        }
        for p in Profile::ALL {
            assert_eq!(Profile::from_name(p.name()), Some(p));
        }
        for p in Place::ALL {
            assert_eq!(Place::from_name(p.name()), Some(p));
        }
    }

    #[test]
    fn every_group_has_codes_the_symbol_set_knows() {
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let f = fixture(kind);
            for g in Group::ALL {
                let known = g.codes().iter().any(|c| f.set.id_for_code(c).is_some());
                assert!(known || g == Group::Vegetation, "{kind:?} {}", g.name());
            }
        }
    }

    #[test]
    fn safe_leaves_everything_where_it_is() {
        let f = fixture(MapKind::Sprint);
        let settings = ContentSettings::with_profile(Profile::Safe);
        let (map, cand, routed) = run(
            &f,
            &settings,
            vec![object(&f, "501.6", "osm")],
            vec![object(&f, "515", "alkis-nas/sonstiges-bauwerk")],
        );
        assert_eq!((map.len(), cand.len(), routed), (1, 1, Routed::default()));
    }

    #[test]
    fn balanced_puts_the_official_sources_on_the_map() {
        let f = fixture(MapKind::Sprint);
        let settings = ContentSettings::default();
        let (map, cand, routed) = run(
            &f,
            &settings,
            vec![],
            vec![
                // The cadastre, a register and the street survey go to the map.
                object(&f, "515", "alkis-nas/sonstiges-bauwerk"),
                object(&f, "518", "geoportal-berlin/sb-gelaender"),
                object(&f, "417", "geoportal-berlin/anlagenbaeume"),
                object(&f, "501.5", "geoportal-berlin/sb-flaechen"),
                object(&f, "501.1", "geoportal-berlin/sb-bordstein"),
                object(&f, "401", "geoportal-berlin/sb-gruenflaeche"),
                // OpenStreetMap counts for walls, fences, hedges, water, trees and paths, and
                // a tree group is as good as its trees.
                object(&f, "515", "osm"),
                object(&f, "518", "osm"),
                object(&f, "301", "osm"),
                object(&f, "418", "osm"),
                object(&f, "505.1", "osm"),
                object(&f, "405", GENERALIZATION_SOURCE),
                // Monuments, towers and pipelines too, and the contours.
                object(&f, "526", "osm"),
                object(&f, "528", "osm"),
                object(&f, "104", "osm"),
                object(&f, "522", "osm"),
                // Not for open land, the outline of the map area is no street edge, and
                // furniture is left as it is.
                object(&f, "401", "osm"),
                object(&f, "501.1", "osm"),
                object(&f, "101", "lgb-dgm1"),
                object(&f, "531", "geoportal-berlin/beleuchtung"),
            ],
        );
        assert_eq!(map.len(), 17, "{map:?}");
        assert_eq!(cand.len(), 3);
        assert_eq!(routed.to_map, 17);
    }

    #[test]
    fn balanced_puts_the_areas_along_the_tracks_on_the_map_and_clears_the_points_in_them() {
        let f = fixture(MapKind::Sprint);
        let with_nutzart = |nutzart: &str| {
            let mut o = object(&f, "520", "alkis-nutzung");
            // A closed square 100 m across around the origin.
            o.coords = [(-50.0, -50.0), (50.0, -50.0), (50.0, 50.0), (-50.0, 50.0)]
                .iter()
                .map(|&p| f.at(p))
                .collect();
            o.coords.last_mut().unwrap().flags = crate::omap::coord::FLAG_CLOSE;
            o.tags
                .push(("freemapper:nutzart".to_string(), nutzart.to_string()));
            o
        };
        let mut lamp = object(&f, "531", "geoportal-berlin/beleuchtung");
        lamp.kind = MapObjectKind::Point { rotation_rad: 0.0 };
        lamp.coords = vec![f.at((0.0, 0.0))];
        let settings = ContentSettings::default();
        let (map, cand, routed) = run(
            &f,
            &settings,
            vec![],
            vec![with_nutzart("Bahnverkehr"), lamp.clone()],
        );
        assert_eq!((map.len(), cand.len()), (1, 0), "{map:?} {cand:?}");
        assert_eq!(routed.dropped, 1, "the lamp in the area goes");
        // Other private ground stays a candidate and keeps its points.
        let (map, cand, _) = run(
            &f,
            &settings,
            vec![],
            vec![with_nutzart("Wohnbaufläche / Mehrfamilienhaus"), lamp],
        );
        assert_eq!((map.len(), cand.len()), (0, 2));
    }

    #[test]
    fn full_puts_everything_on_the_map_except_hidden_candidates() {
        let f = fixture(MapKind::Sprint);
        let mut hidden = object(&f, "417", "osm");
        hidden
            .tags
            .push((HIDDEN_KEY.to_string(), "520".to_string()));
        let settings = ContentSettings::with_profile(Profile::Full);
        let (map, cand, _) = run(
            &f,
            &settings,
            vec![],
            vec![
                object(&f, "101", "lgb-dgm1"),
                object(&f, "520", "alkis-nutzung"),
                hidden,
            ],
        );
        assert_eq!((map.len(), cand.len()), (2, 1));
    }

    #[test]
    fn a_choice_beats_the_profile_and_off_drops() {
        let f = fixture(MapKind::Sprint);
        let mut settings = ContentSettings::with_profile(Profile::Full);
        settings.set(Group::Fences, Some(Place::Candidates));
        settings.set(Group::Walls, Some(Place::Off));
        let (map, cand, routed) = run(
            &f,
            &settings,
            vec![object(&f, "518", "alkis"), object(&f, "515", "alkis")],
            vec![object(&f, "518", "osm")],
        );
        assert_eq!((map.len(), cand.len()), (0, 2));
        assert_eq!(routed.dropped, 1);
        assert_eq!(routed.to_candidates, 1);
    }

    #[test]
    fn vegetation_is_off_until_chosen_and_goes_by_its_tag() {
        let f = fixture(MapKind::Sprint);
        let vegetation = |code: &str| {
            let mut o = object(&f, code, "lgb-als");
            o.tags
                .push((POINT_CLOUD_KEY.to_string(), "lgb-als".to_string()));
            o
        };
        // ALKIS parks and forests with the same symbols are no vegetation from a point cloud.
        let alkis = || {
            vec![
                object(&f, "401", "alkis-nutzung"),
                object(&f, "405", "alkis-nutzung"),
            ]
        };
        for profile in Profile::ALL {
            let settings = ContentSettings::with_profile(profile);
            assert!(!settings.wants_vegetation());
            let (map, cand, routed) = run(
                &f,
                &settings,
                alkis(),
                vec![vegetation("410"), vegetation("401")],
            );
            assert_eq!(
                (map.len(), cand.len(), routed.dropped),
                (2, 0, 2),
                "{profile:?}"
            );
        }
        for (place, on_map) in [(Place::Candidates, 0), (Place::Map, 2)] {
            let mut settings = ContentSettings::with_profile(Profile::Safe);
            settings.set(Group::Vegetation, Some(place));
            assert!(settings.wants_vegetation());
            let (map, cand, _) = run(
                &f,
                &settings,
                alkis(),
                vec![vegetation("410"), vegetation("401")],
            );
            assert_eq!(
                (map.len(), cand.len()),
                (2 + on_map, 2 - on_map),
                "{place:?}"
            );
        }
        let mut off = ContentSettings::default();
        off.set(Group::Vegetation, Some(Place::Off));
        assert!(!off.wants_vegetation());
    }

    #[test]
    fn tree_groups_follow_the_trees() {
        let f = fixture(MapKind::Sprint);
        let group_area = object(&f, "405", GENERALIZATION_SOURCE);
        let park = object(&f, "405", "alkis-nutzung");
        let mut settings = ContentSettings::with_profile(Profile::Safe);
        settings.set(Group::Trees, Some(Place::Map));
        let (map, cand, _) = run(&f, &settings, vec![], vec![group_area, park]);
        assert_eq!(
            (map.len(), cand.len()),
            (1, 1),
            "the land use is no tree group"
        );
    }

    #[test]
    fn a_request_starts_from_the_profile_or_adds_to_the_settings() {
        assert_eq!(ContentSettings::from_request(None, &[], None), Ok(None));
        let full = ContentSettings::from_request(Some("full"), &["walls=off".to_string()], None)
            .unwrap()
            .unwrap();
        assert_eq!(
            (full.profile, full.walls),
            (Profile::Full, Some(Place::Off))
        );
        // Only a pair: the existing settings stay, the pair is added.
        let more = ContentSettings::from_request(None, &["fences=map".to_string()], Some(&full))
            .unwrap()
            .unwrap();
        assert_eq!(
            (more.profile, more.walls, more.fences),
            (Profile::Full, Some(Place::Off), Some(Place::Map))
        );
        // A profile drops the old exceptions.
        let fresh = ContentSettings::from_request(Some("safe"), &[], Some(&full))
            .unwrap()
            .unwrap();
        assert_eq!(fresh, ContentSettings::with_profile(Profile::Safe));
        assert!(ContentSettings::from_request(Some("huge"), &[], None).is_err());
        assert_eq!(more.summary(), "full, walls=off, fences=map");
    }

    #[test]
    fn settings_read_and_write_as_toml() {
        let text = "profile = \"balanced\"\nwalls = \"map\"\nout-of-bounds = \"off\"\n";
        let settings: ContentSettings = toml::from_str(text).unwrap();
        assert_eq!(settings.walls, Some(Place::Map));
        assert_eq!(settings.out_of_bounds, Some(Place::Off));
        assert_eq!(toml::to_string(&settings).unwrap(), text);
        assert!(toml::from_str::<ContentSettings>("walls = \"everywhere\"").is_err());
        assert!(toml::from_str::<ContentSettings>("wals = \"map\"").is_err());
        let mut pairs = ContentSettings::default();
        pairs.set_from_pair("fences=candidates").unwrap();
        assert_eq!(pairs.fences, Some(Place::Candidates));
        assert!(pairs.set_from_pair("fences").is_err());
    }
}
