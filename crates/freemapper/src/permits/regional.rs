// SPDX-License-Identifier: AGPL-3.0-or-later

//! The part of the permit notes that region adapters bring (ADR 0120): districts, green spaces,
//! public land, monuments, protected objects and the forest administration (ADR 0124), from the `PermitSource`s of the adapters
//! whose ground the map area touches.
//!
//! The engine knows the kinds of finding, not the region: it measures each finding on the map
//! polygon, counts and groups, and leaves a section out when one of the sources that fill it
//! was not read.

use std::collections::BTreeMap;

/// The kinds the regional part names, for the writers of the notes.
pub use freemapper_adapter_api::permits::{
    AdminUnitLevel, DistrictOffices, ForestFunctionKind, PermitSection, ProtectedObjectKind,
};
use freemapper_adapter_api::permits::{FindingKind, PermitFinding, PermitSourceInfo};
use geo_types::{Coord, Geometry, LineString, MultiPolygon};
use serde::{Deserialize, Serialize};

use crate::permits::notes::{Plane, Share, SourceInput, SourceStatus, status};

/// The most links to monuments the notes give; the others are counted.
pub const MAX_MONUMENT_LINKS: usize = 20;

/// The most names of monuments the notes give for a register without positions (ADR 0184); the
/// other monuments are counted.
pub const MAX_REGISTER_MONUMENT_NAMES: usize = 10;

/// A text in the languages of the program, as the notes store it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Words {
    pub en: String,
    pub de: String,
}

/// The facts of a regional source the notes keep: what [`PermitSourceInfo`] says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionalSource {
    pub key: String,
    pub label: Words,
    pub what: Words,
    /// The credit line or licence, verbatim.
    pub credit: String,
    /// The publisher's note on what the data are worth, verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The source answers only at points (ADR 0139).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub sampled: bool,
    /// Where the data are not complete (ADR 0144).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gaps: Option<Words>,
    pub sections: Vec<PermitSection>,
    /// Whose offices the hints of the districts of the source name (ADR 0129).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offices: Option<DistrictOffices>,
}

impl From<&PermitSourceInfo> for RegionalSource {
    fn from(info: &PermitSourceInfo) -> Self {
        let words = |w: freemapper_adapter_api::permits::Wording| Words {
            en: w.en.to_string(),
            de: w.de.to_string(),
        };
        Self {
            key: info.key.to_string(),
            label: words(info.label),
            what: words(info.what),
            credit: info.credit.to_string(),
            note: info.note.map(str::to_string),
            sampled: info.sampled,
            gaps: info.gaps.map(words),
            sections: info.sections.to_vec(),
            offices: info.offices,
        }
    }
}

/// What one regional source gave.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalInput {
    /// The id of the adapter that brings the source.
    pub adapter: String,
    pub source: RegionalSource,
    pub findings: SourceInput<PermitFinding>,
}

/// The regional part of the notes of one adapter. A section is `None` when one of the sources
/// that fill it was not read; then the notes leave it out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionalNotes {
    pub adapter: String,
    /// The districts on the map, by share, descending.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub districts: Option<Vec<Share<DistrictEntry>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub green_spaces: Option<GreenSpaces>,
    /// One entry per public owner, by share, descending.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<Share<OwnerEntry>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monuments: Option<Monuments>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_objects: Option<Vec<ProtectedObjectEntry>>,
    /// One entry per office and unit of the forest administration, by share, descending.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forest_units: Option<Vec<Share<ForestUnitEntry>>>,
    /// One entry per owner and office of the state forest, by share, descending.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_forest: Option<Vec<Share<StateForestEntry>>>,
    /// One entry per water protection area and zone, by share, descending (ADR 0129).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_protection: Option<Vec<Share<WaterProtectionEntry>>>,
    /// One entry per function of forest and name, by share, descending.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forest_functions: Option<Vec<Share<ForestFunctionEntry>>>,
    /// The administrative units on the map, level by level top down, within a level by share,
    /// descending (ADR 0181).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_units: Option<Vec<Share<AdminUnitEntry>>>,
    /// The monuments of a register without positions, in the rectangle around the map area
    /// (ADR 0184).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub register_monuments: Option<RegisterMonuments>,
    /// The archaeological monuments of a register without positions, in the rectangle around the
    /// map area (ADR 0190).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub register_archaeology: Option<RegisterMonuments>,
    /// The monuments of history of a region with positions (ADR 0191).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub register_history: Option<RegisterHistory>,
    /// The sources of the adapter, in its order.
    pub sources: Vec<RegionalStatus>,
}

/// The monuments of history: those that touch the map area and those that lie only in the
/// rectangle around it. A monument with several parts counts once.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterHistory {
    pub in_area: RegisterMonuments,
    pub nearby: RegisterMonuments,
}

/// The monuments of a register that gives no position: how many, and the names of the first few.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterMonuments {
    pub total: usize,
    /// The names as the register writes them with how often each occurs, by count descending,
    /// then by name; at most [`MAX_REGISTER_MONUMENT_NAMES`] names.
    pub names: Vec<RegisterMonumentName>,
    /// The monuments that no name line covers: the ones beyond the cap and those without a name.
    pub left: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterMonumentName {
    pub name: String,
    pub count: usize,
}

/// An administrative unit of a region, the name as the register writes it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AdminUnitEntry {
    pub level: AdminUnitLevel,
    pub name: String,
    /// The code of the register (the Polish TERYT).
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistrictEntry {
    pub name: String,
}

/// The green spaces on the map, and how many playgrounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GreenSpaces {
    /// One entry per name, district and dedication, by share, descending.
    pub entries: Vec<Share<GreenSpaceEntry>>,
    pub playgrounds: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GreenSpaceEntry {
    /// Empty when the data give none.
    pub name: String,
    pub district: Option<String>,
    pub dedication: Option<String>,
}

/// A public owner. `owner` is `None` for the other public owners, whose names the source
/// leaves out; they share one entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnerEntry {
    pub owner: Option<String>,
}

/// The monuments on the map, counted per kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monuments {
    pub total: usize,
    /// By count, descending, then by kind.
    pub groups: Vec<MonumentGroup>,
    /// Monuments with a link whose link is not given: at most [`MAX_MONUMENT_LINKS`] are.
    pub links_left: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonumentGroup {
    pub kind: String,
    pub count: usize,
    /// Sorted; the rarer kinds get their links first.
    pub links: Vec<String>,
}

/// A unit of a forest administration, the texts as the source composed them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ForestUnitEntry {
    pub administration: String,
    pub office: String,
    pub unit: Option<String>,
}

/// Forest of the state and the office that looks after it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateForestEntry {
    pub owner: String,
    pub office: Option<String>,
}

/// A water protection area or zone.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WaterProtectionEntry {
    pub name: Option<String>,
    pub zone: Option<String>,
    pub citation: Option<String>,
    /// The authority in charge, where the data name it (ADR 0143).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
}

/// An area of forest with a function or protection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ForestFunctionEntry {
    pub kind: ForestFunctionKind,
    pub name: Option<String>,
}

/// A protected object on the map. Equal points (same kind, name and citation) are one entry
/// with their count; an area has its share.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtectedObjectEntry {
    pub kind: ProtectedObjectKind,
    pub name: Option<String>,
    pub citation: Option<String>,
    pub count: usize,
    /// The share of an area in percent, one decimal; `None` for points.
    pub percent: Option<f64>,
    pub at_edge: bool,
}

/// A regional source with its state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionalStatus {
    pub source: RegionalSource,
    pub status: SourceStatus,
}

/// A municipality (Polish gmina) of the map for the hints: the name as the register writes it and
/// the share of the map area in percent, one decimal.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Municipality {
    pub name: String,
    pub percent: f64,
}

/// What the hints need of the regional part of one adapter.
#[derive(Debug, Default)]
pub(crate) struct HintFacts {
    /// The districts on the map that are more than an edge, by share, descending.
    pub districts: Vec<String>,
    /// The districts that hold a monument on the map: for each monument those that are more
    /// than an edge, or its edge districts when it lies only there.
    pub monument_districts: Vec<String>,
    /// The monuments on the map, for the lower monument authority outside the city states
    /// (ADR 0144).
    pub monuments: usize,
    /// A source of monuments of the adapter has gaps (ADR 0144): no monument on the map does not
    /// mean none.
    pub monument_gaps: bool,
    /// Naturdenkmale, protected landscape parts or biotopes lie on the map (ADR 0146).
    pub nature_objects: bool,
    /// A kind of protected object that the council of a gmina establishes lies on the map: a
    /// natural monument, an ecological site, a nature and landscape complex or a documentation
    /// site (Polish Nature Conservation Act, art. 44; ADR 0183).
    pub gmina_nature_forms: bool,
    /// The municipalities (Polish gminas) on the map that are more than an edge, by share,
    /// descending, as the register writes them; all of them when each lies only at the edge.
    pub municipalities: Vec<Municipality>,
    /// The monuments of a register without positions in the rectangle around the map area
    /// (Poland, ADR 0184).
    pub register_monuments: usize,
    /// The archaeological monuments of a register without positions in the rectangle around the
    /// map area (Poland, ADR 0190).
    pub register_archaeology: usize,
    /// The monuments of history in the rectangle around the map area, in the area or not (Poland,
    /// ADR 0191).
    pub register_history: usize,
    /// The types of the biotopes on the map, by share, descending (ADR 0146).
    pub biotopes: Vec<String>,
    /// The districts that hold a protected object the lower nature conservation authority
    /// looks after, chosen as for the monuments.
    pub nature_districts: Vec<String>,
    /// The public owners with their share in percent (`None` as owner: the other public
    /// owners); `None` when no owner section was read.
    pub owners: Option<Vec<(Option<String>, f64)>>,
    /// The forest offices on the map, by share, descending, each with its administration and
    /// its units, for the offices that are more than an edge; an office that lies only at the
    /// edge is left out.
    pub forest_offices: Vec<ForestOffice>,
    /// The state forest on the map with its share in percent.
    pub state_forest: Vec<(StateForestEntry, f64)>,
    /// Whose offices the district hints name; from the source of the districts.
    pub offices: Option<DistrictOffices>,
    /// The water protection areas and zones on the map, edges included.
    pub water: Vec<WaterProtectionEntry>,
    /// The functions of forest on the map with the names of their areas, in the order of the
    /// kinds.
    pub forest_functions: Vec<(ForestFunctionKind, Vec<String>)>,
}

/// A forest office the hints name.
#[derive(Debug, Clone, PartialEq)]
pub struct ForestOffice {
    pub administration: String,
    pub office: String,
    /// The units of the office on the map that are more than an edge, by share.
    pub units: Vec<String>,
}

/// A finding in the plane of the map.
struct Placed<'a> {
    kind: &'a FindingKind,
    shape: Shape,
}

enum Shape {
    Area(MultiPolygon<f64>),
    Points(Vec<Coord<f64>>),
    Lines(Vec<LineString<f64>>),
}

impl Shape {
    fn of(plane: &Plane, geometry: &Geometry<f64>) -> Option<Self> {
        match geometry {
            Geometry::MultiPolygon(mp) => Some(Shape::Area(plane.project(mp))),
            Geometry::Polygon(p) => Some(Shape::Area(
                plane.project(&MultiPolygon::new(vec![p.clone()])),
            )),
            Geometry::MultiPoint(mp) => Some(Shape::Points(
                mp.0.iter()
                    .filter_map(|p| plane.project_point(p.0))
                    .collect(),
            )),
            Geometry::Point(p) => Some(Shape::Points(
                plane.project_point(p.0).into_iter().collect(),
            )),
            Geometry::LineString(l) => {
                Some(Shape::Lines(plane.project_line(l).into_iter().collect()))
            }
            Geometry::MultiLineString(ml) => Some(Shape::Lines(
                ml.0.iter().filter_map(|l| plane.project_line(l)).collect(),
            )),
            _ => None,
        }
    }

    /// The share of the map an area covers, `Some(0.0)` for points on the map; `None` when
    /// the finding is not on the map.
    fn on_map(&self, plane: &Plane) -> Option<f64> {
        match self {
            Shape::Area(area) => plane.share(area),
            Shape::Points(points) => points.iter().any(|c| plane.contains(*c)).then_some(0.0),
            Shape::Lines(lines) => lines.iter().any(|l| plane.crosses(l)).then_some(0.0),
        }
    }

    /// Whether the part of the finding on the map lies in `district`.
    fn in_district(&self, plane: &Plane, district: &MultiPolygon<f64>) -> bool {
        match self {
            Shape::Area(area) => plane.share_within(area, district),
            Shape::Points(points) => points
                .iter()
                .any(|c| plane.contains(*c) && geo::Contains::contains(district, c)),
            // No source of lines feeds the districts; a line is in none.
            Shape::Lines(_) => false,
        }
    }
}

/// The regional parts of the notes, one per adapter in the order of `inputs`, and the facts
/// of the hints.
pub(crate) fn assemble_regional(
    plane: &Plane,
    inputs: &[RegionalInput],
) -> (Vec<RegionalNotes>, Vec<HintFacts>) {
    let mut adapters: Vec<&str> = Vec::new();
    for input in inputs {
        if !adapters.contains(&input.adapter.as_str()) {
            adapters.push(&input.adapter);
        }
    }
    adapters
        .into_iter()
        .map(|adapter| {
            let mine: Vec<&RegionalInput> =
                inputs.iter().filter(|i| i.adapter == adapter).collect();
            one_adapter(plane, adapter, &mine)
        })
        .unzip()
}

fn one_adapter(
    plane: &Plane,
    adapter: &str,
    inputs: &[&RegionalInput],
) -> (RegionalNotes, HintFacts) {
    // A section is there when every source that fills it was read.
    let read = |section: PermitSection| {
        let fillers: Vec<&&RegionalInput> = inputs
            .iter()
            .filter(|i| i.source.sections.contains(&section))
            .collect();
        !fillers.is_empty()
            && fillers
                .iter()
                .all(|i| matches!(i.findings, SourceInput::Read { .. }))
    };
    let placed: Vec<Placed<'_>> = inputs
        .iter()
        .flat_map(|i| i.findings.items())
        .filter_map(|f| {
            Some(Placed {
                kind: &f.kind,
                shape: Shape::of(plane, &f.geometry)?,
            })
        })
        .collect();
    let on_map: Vec<(&Placed<'_>, f64)> = placed
        .iter()
        .filter_map(|p| Some((p, p.shape.on_map(plane)?)))
        .collect();

    let districts = read(PermitSection::Districts).then(|| districts(&on_map));
    let district_shapes: Vec<(&str, &MultiPolygon<f64>)> = on_map
        .iter()
        .filter_map(|(p, _)| match (p.kind, &p.shape) {
            (FindingKind::District { name }, Shape::Area(area)) => Some((name.as_str(), area)),
            _ => None,
        })
        .collect();
    // The districts that hold an object `test` picks. Like the district-office hint, an object
    // names only the districts holding it that are more than an edge of the map; an object
    // that lies only in edge districts names those.
    let holding = |test: &dyn Fn(&FindingKind) -> bool| -> Vec<String> {
        let entries: Vec<&Share<DistrictEntry>> = districts.iter().flatten().collect();
        let mut named = vec![false; entries.len()];
        for (object, _) in on_map.iter().filter(|(p, _)| test(p.kind)) {
            let holds: Vec<usize> = entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    district_shapes
                        .iter()
                        .filter(|(n, _)| *n == entry.item.name)
                        .any(|(_, shape)| object.shape.in_district(plane, shape))
                })
                .map(|(i, _)| i)
                .collect();
            let inner: Vec<usize> = holds
                .iter()
                .copied()
                .filter(|&i| !entries[i].at_edge)
                .collect();
            for i in if inner.is_empty() { holds } else { inner } {
                named[i] = true;
            }
        }
        // In the order of the districts: by share.
        entries
            .iter()
            .zip(named)
            .filter(|(_, named)| *named)
            .map(|(entry, _)| entry.item.name.clone())
            .collect()
    };
    let green_spaces = read(PermitSection::GreenSpaces).then(|| green_spaces(&on_map));
    let owners = read(PermitSection::Owners).then(|| owners(&on_map));
    let monuments = read(PermitSection::Monuments).then(|| monuments(&on_map));
    let protected_objects =
        read(PermitSection::ProtectedObjects).then(|| protected_objects(&on_map));
    let forest_units = read(PermitSection::ForestUnits).then(|| forest_units(&on_map));
    let state_forest = read(PermitSection::StateForest).then(|| state_forest(&on_map));
    let water_protection = read(PermitSection::WaterProtection).then(|| water_protection(&on_map));
    let forest_functions = read(PermitSection::ForestFunctions).then(|| forest_functions(&on_map));
    let admin_units = read(PermitSection::AdminUnits).then(|| admin_units(&on_map));
    let register_monuments = read(PermitSection::MonumentRegister).then(|| {
        count_register(&on_map, |kind| match kind {
            FindingKind::RegisterMonument { name } => Some(name),
            _ => None,
        })
    });
    let register_archaeology = read(PermitSection::ArchaeologyRegister).then(|| {
        count_register(&on_map, |kind| match kind {
            FindingKind::RegisterArchaeology { name } => Some(name),
            _ => None,
        })
    });
    let register_history =
        read(PermitSection::HistoryRegister).then(|| count_history(plane, &placed));

    let facts = HintFacts {
        districts: districts
            .iter()
            .flatten()
            .filter(|s| !s.at_edge)
            .map(|s| s.item.name.clone())
            .collect(),
        monument_districts: if monuments.as_ref().is_some_and(|m| m.total > 0) {
            holding(&|k| matches!(k, FindingKind::Monument { .. }))
        } else {
            Vec::new()
        },
        monuments: monuments.as_ref().map_or(0, |m| m.total),
        nature_objects: protected_objects
            .iter()
            .flatten()
            .any(|o| o.kind.needs_nature_authority()),
        gmina_nature_forms: protected_objects.iter().flatten().any(|o| {
            matches!(
                o.kind,
                ProtectedObjectKind::NaturalMonumentPl
                    | ProtectedObjectKind::EcologicalSite
                    | ProtectedObjectKind::LandscapeComplex
                    | ProtectedObjectKind::DocumentationSite
            )
        }),
        municipalities: municipalities(admin_units.iter().flatten()),
        register_monuments: register_monuments.as_ref().map_or(0, |m| m.total),
        register_archaeology: register_archaeology.as_ref().map_or(0, |m| m.total),
        register_history: register_history
            .as_ref()
            .map_or(0, |h| h.in_area.total + h.nearby.total),
        biotopes: {
            let mut biotopes: Vec<(&ProtectedObjectEntry, f64)> = protected_objects
                .iter()
                .flatten()
                .filter(|o| o.kind == ProtectedObjectKind::Biotope)
                .map(|o| (o, o.percent.unwrap_or(0.0)))
                .collect();
            biotopes.sort_by(|a, b| b.1.total_cmp(&a.1));
            let mut names: Vec<String> = Vec::new();
            for (o, _) in biotopes {
                let name = o.name.clone().unwrap_or_default();
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            names
        },
        monument_gaps: inputs.iter().any(|i| {
            i.source.sections.contains(&PermitSection::Monuments) && i.source.gaps.is_some()
        }),
        nature_districts: if protected_objects
            .iter()
            .flatten()
            .any(|o| o.kind.needs_nature_authority())
        {
            holding(
                &|k| matches!(k, FindingKind::ProtectedObject { kind, .. } if kind.needs_nature_authority()),
            )
        } else {
            Vec::new()
        },
        owners: owners.as_ref().map(|owners| {
            owners
                .iter()
                .map(|s| (s.item.owner.clone(), s.percent))
                .collect()
        }),
        forest_offices: forest_offices(forest_units.iter().flatten()),
        state_forest: state_forest
            .iter()
            .flatten()
            .map(|s| (s.item.clone(), s.percent))
            .collect(),
        offices: inputs
            .iter()
            .filter(|i| i.source.sections.contains(&PermitSection::Districts))
            .find_map(|i| i.source.offices),
        water: water_protection
            .iter()
            .flatten()
            .map(|s| s.item.clone())
            .collect(),
        forest_functions: {
            let mut kinds: BTreeMap<ForestFunctionKind, Vec<String>> = BTreeMap::new();
            for s in forest_functions.iter().flatten() {
                let names = kinds.entry(s.item.kind).or_default();
                if let Some(name) = &s.item.name
                    && !names.contains(name)
                {
                    names.push(name.clone());
                }
            }
            kinds.into_iter().collect()
        },
    };
    let notes = RegionalNotes {
        adapter: adapter.to_string(),
        districts,
        green_spaces,
        owners,
        monuments,
        protected_objects,
        forest_units,
        state_forest,
        water_protection,
        forest_functions,
        admin_units,
        register_monuments,
        register_archaeology,
        register_history,
        sources: inputs
            .iter()
            .map(|i| RegionalStatus {
                source: i.source.clone(),
                status: status(&i.source.key, &i.findings, &i.source.credit),
            })
            .collect(),
    };
    (notes, facts)
}

/// Entries with their summed share, by share descending, then by `order`.
fn by_share<K: Ord + Clone, T>(summed: BTreeMap<K, f64>, entry: impl Fn(&K) -> T) -> Vec<Share<T>> {
    let mut list: Vec<(K, f64)> = summed.into_iter().collect();
    list.sort_by(|(a, x), (b, y)| y.total_cmp(x).then_with(|| a.cmp(b)));
    list.into_iter()
        .map(|(key, share)| Share::new(entry(&key), share.min(1.0)))
        .collect()
}

fn districts(on_map: &[(&Placed<'_>, f64)]) -> Vec<Share<DistrictEntry>> {
    let mut summed: BTreeMap<String, f64> = BTreeMap::new();
    for (p, share) in on_map {
        if let FindingKind::District { name } = p.kind {
            *summed.entry(name.clone()).or_default() += share;
        }
    }
    by_share(summed, |name| DistrictEntry { name: name.clone() })
}

/// The units summed over their findings (a unit of several parts has one entry), level by level
/// top down, within a level by share descending.
fn admin_units(on_map: &[(&Placed<'_>, f64)]) -> Vec<Share<AdminUnitEntry>> {
    let mut summed: BTreeMap<AdminUnitEntry, f64> = BTreeMap::new();
    for (p, share) in on_map {
        if let FindingKind::AdminUnit { level, name, code } = p.kind {
            let key = AdminUnitEntry {
                level: *level,
                name: name.clone(),
                code: code.clone(),
            };
            *summed.entry(key).or_default() += share;
        }
    }
    let mut list: Vec<(AdminUnitEntry, f64)> = summed.into_iter().collect();
    list.sort_by(|(a, x), (b, y)| {
        a.level
            .cmp(&b.level)
            .then_with(|| y.total_cmp(x))
            .then_with(|| a.cmp(b))
    });
    list.into_iter()
        .map(|(entry, share)| Share::new(entry, share.min(1.0)))
        .collect()
}

/// The municipalities of the units: those that are more than an edge, else all of them. The
/// units come by share, descending.
fn municipalities<'a>(units: impl Iterator<Item = &'a Share<AdminUnitEntry>>) -> Vec<Municipality> {
    let all: Vec<&Share<AdminUnitEntry>> = units
        .filter(|s| s.item.level == AdminUnitLevel::Municipality)
        .collect();
    let any_more_than_edge = all.iter().any(|s| !s.at_edge);
    all.into_iter()
        .filter(|s| !any_more_than_edge || !s.at_edge)
        .map(|s| Municipality {
            name: s.item.name.clone(),
            percent: s.percent,
        })
        .collect()
}

fn green_spaces(on_map: &[(&Placed<'_>, f64)]) -> GreenSpaces {
    type Key = (String, Option<String>, Option<String>);
    let mut summed: BTreeMap<Key, f64> = BTreeMap::new();
    let mut playgrounds = 0;
    for (p, share) in on_map {
        match p.kind {
            FindingKind::GreenSpace {
                name,
                district,
                dedication,
            } => {
                *summed
                    .entry((name.clone(), district.clone(), dedication.clone()))
                    .or_default() += share;
            }
            FindingKind::Playground => playgrounds += 1,
            _ => {}
        }
    }
    GreenSpaces {
        entries: by_share(summed, |(name, district, dedication)| GreenSpaceEntry {
            name: name.clone(),
            district: district.clone(),
            dedication: dedication.clone(),
        }),
        playgrounds,
    }
}

/// The share of the map each owner holds: the sum of the shares of its parcels. The owners
/// the source does not name are summed into one entry.
fn owners(on_map: &[(&Placed<'_>, f64)]) -> Vec<Share<OwnerEntry>> {
    let mut summed: BTreeMap<Option<String>, f64> = BTreeMap::new();
    for (p, share) in on_map {
        if let FindingKind::OwnerShare { owner } = p.kind {
            *summed.entry(owner.clone()).or_default() += share;
        }
    }
    by_share(summed, |owner| OwnerEntry {
        owner: owner.clone(),
    })
}

fn forest_units(on_map: &[(&Placed<'_>, f64)]) -> Vec<Share<ForestUnitEntry>> {
    let mut summed: BTreeMap<ForestUnitEntry, f64> = BTreeMap::new();
    for (p, share) in on_map {
        if let FindingKind::ForestUnit {
            administration,
            office,
            unit,
        } = p.kind
        {
            let key = ForestUnitEntry {
                administration: administration.clone(),
                office: office.clone(),
                unit: unit.clone(),
            };
            *summed.entry(key).or_default() += share;
        }
    }
    by_share(summed, Clone::clone)
}

/// The offices of `units` (by share) with their units that are more than an edge, in the
/// order of their largest unit; an office with only edge units is left out.
fn forest_offices<'a>(
    units: impl Iterator<Item = &'a Share<ForestUnitEntry>>,
) -> Vec<ForestOffice> {
    let mut offices: Vec<ForestOffice> = Vec::new();
    for s in units.filter(|s| !s.at_edge) {
        let unit = &s.item;
        let at = match offices
            .iter()
            .position(|o| o.administration == unit.administration && o.office == unit.office)
        {
            Some(at) => at,
            None => {
                offices.push(ForestOffice {
                    administration: unit.administration.clone(),
                    office: unit.office.clone(),
                    units: Vec::new(),
                });
                offices.len() - 1
            }
        };
        if let Some(name) = &unit.unit
            && !offices[at].units.contains(name)
        {
            offices[at].units.push(name.clone());
        }
    }
    offices
}

fn state_forest(on_map: &[(&Placed<'_>, f64)]) -> Vec<Share<StateForestEntry>> {
    let mut summed: BTreeMap<StateForestEntry, f64> = BTreeMap::new();
    for (p, share) in on_map {
        if let FindingKind::StateForest { owner, office } = p.kind {
            let key = StateForestEntry {
                owner: owner.clone(),
                office: office.clone(),
            };
            *summed.entry(key).or_default() += share;
        }
    }
    by_share(summed, Clone::clone)
}

fn water_protection(on_map: &[(&Placed<'_>, f64)]) -> Vec<Share<WaterProtectionEntry>> {
    let mut summed: BTreeMap<WaterProtectionEntry, f64> = BTreeMap::new();
    for (p, share) in on_map {
        if let FindingKind::WaterProtection {
            name,
            zone,
            citation,
            authority,
        } = p.kind
        {
            let key = WaterProtectionEntry {
                name: name.clone(),
                zone: zone.clone(),
                citation: citation.clone(),
                authority: authority.clone(),
            };
            *summed.entry(key).or_default() += share;
        }
    }
    by_share(summed, Clone::clone)
}

fn forest_functions(on_map: &[(&Placed<'_>, f64)]) -> Vec<Share<ForestFunctionEntry>> {
    let mut summed: BTreeMap<ForestFunctionEntry, f64> = BTreeMap::new();
    for (p, share) in on_map {
        if let FindingKind::ForestFunction { kind, name } = p.kind {
            let key = ForestFunctionEntry {
                kind: *kind,
                name: name.clone(),
            };
            *summed.entry(key).or_default() += share;
        }
    }
    by_share(summed, Clone::clone)
}

/// The monuments of a register without positions: counted, the names counted per name. `pick`
/// gives the name of a finding of the register and `None` for any other finding.
fn count_register(
    on_map: &[(&Placed<'_>, f64)],
    pick: impl Fn(&FindingKind) -> Option<&Option<String>>,
) -> RegisterMonuments {
    count_names(
        on_map
            .iter()
            .filter_map(|(p, _)| pick(p.kind))
            .map(Option::as_deref),
    )
}

/// The monuments of history: each counts once however many parts it has, in the area when one
/// of its parts touches the map, else nearby (every finding the source gave lies in the
/// rectangle around the map). The name is the first one any part has.
fn count_history(plane: &Plane, placed: &[Placed<'_>]) -> RegisterHistory {
    let mut by_id: BTreeMap<&str, (Option<&str>, bool)> = BTreeMap::new();
    for p in placed {
        if let FindingKind::RegisterHistory { id, name } = p.kind {
            let entry = by_id.entry(id.as_str()).or_default();
            entry.0 = entry.0.or(name.as_deref());
            entry.1 |= p.shape.on_map(plane).is_some();
        }
    }
    let of = |in_area: bool| {
        count_names(
            by_id
                .values()
                .filter(|(_, on_map)| *on_map == in_area)
                .map(|(name, _)| *name),
        )
    };
    RegisterHistory {
        in_area: of(true),
        nearby: of(false),
    }
}

/// `names` of monuments counted: the total, the names with their counts (at most
/// [`MAX_REGISTER_MONUMENT_NAMES`], by count descending, then by name) and the monuments no name
/// line covers.
fn count_names<'a>(names: impl Iterator<Item = Option<&'a str>>) -> RegisterMonuments {
    let mut total = 0;
    let mut by_name: BTreeMap<&str, usize> = BTreeMap::new();
    for name in names {
        total += 1;
        if let Some(name) = name {
            *by_name.entry(name).or_default() += 1;
        }
    }
    let mut names: Vec<RegisterMonumentName> = by_name
        .into_iter()
        .map(|(name, count)| RegisterMonumentName {
            name: name.to_string(),
            count,
        })
        .collect();
    names.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    names.truncate(MAX_REGISTER_MONUMENT_NAMES);
    let listed: usize = names.iter().map(|n| n.count).sum();
    RegisterMonuments {
        total,
        names,
        left: total - listed,
    }
}

fn monuments(on_map: &[(&Placed<'_>, f64)]) -> Monuments {
    let mut by_kind: BTreeMap<&str, (usize, Vec<&str>)> = BTreeMap::new();
    for (p, _) in on_map {
        if let FindingKind::Monument { kind, link } = p.kind {
            let (count, links) = by_kind.entry(kind.as_str()).or_default();
            *count += 1;
            if let Some(link) = link {
                links.push(link);
            }
        }
    }
    let mut groups: Vec<MonumentGroup> = by_kind
        .into_iter()
        .map(|(kind, (count, mut links))| {
            links.sort_unstable();
            links.dedup();
            MonumentGroup {
                kind: kind.to_string(),
                count,
                links: links.into_iter().map(str::to_string).collect(),
            }
        })
        .collect();
    // The rarer kinds keep their links first: one Gartendenkmal says more than the thirtieth
    // Baudenkmal.
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by(|&a, &b| {
        groups[a]
            .count
            .cmp(&groups[b].count)
            .then_with(|| groups[a].kind.cmp(&groups[b].kind))
    });
    let mut room = MAX_MONUMENT_LINKS;
    let mut links_left = 0;
    for index in order {
        let links = &mut groups[index].links;
        let keep = links.len().min(room);
        links_left += links.len() - keep;
        links.truncate(keep);
        room -= keep;
    }
    groups.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.kind.cmp(&b.kind)));
    Monuments {
        total: groups.iter().map(|g| g.count).sum(),
        groups,
        links_left,
    }
}

/// A protected object by its kind, name and act.
type ObjectKey = (ProtectedObjectKind, Option<String>, Option<String>);

fn protected_objects(on_map: &[(&Placed<'_>, f64)]) -> Vec<ProtectedObjectEntry> {
    let mut points: BTreeMap<ObjectKey, usize> = BTreeMap::new();
    // Areas of one kind, name and citation are one entry with their summed share: a map in
    // the forest can hold dozens of biotopes of one type (ADR 0146).
    let mut areas: BTreeMap<ObjectKey, (usize, f64)> = BTreeMap::new();
    for (p, share) in on_map {
        let FindingKind::ProtectedObject {
            kind,
            name,
            citation,
        } = p.kind
        else {
            continue;
        };
        match p.shape {
            Shape::Points(_) | Shape::Lines(_) => {
                *points
                    .entry((*kind, name.clone(), citation.clone()))
                    .or_default() += 1;
            }
            Shape::Area(_) => {
                let entry = areas
                    .entry((*kind, name.clone(), citation.clone()))
                    .or_default();
                entry.0 += 1;
                entry.1 += share;
            }
        }
    }
    let mut all: Vec<ProtectedObjectEntry> = areas
        .into_iter()
        .map(|((kind, name, citation), (count, share))| {
            let share = Share::new((), share.min(1.0));
            ProtectedObjectEntry {
                kind,
                name,
                citation,
                count,
                percent: Some(share.percent),
                at_edge: share.at_edge,
            }
        })
        .collect();
    all.extend(
        points
            .into_iter()
            .map(|((kind, name, citation), count)| ProtectedObjectEntry {
                kind,
                name,
                citation,
                count,
                percent: None,
                at_edge: false,
            }),
    );
    all.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| {
                b.percent
                    .unwrap_or(-1.0)
                    .total_cmp(&a.percent.unwrap_or(-1.0))
            })
            .then_with(|| a.name.cmp(&b.name))
    });
    all
}
