//! The part of the permit notes that region adapters bring (ADR 0120): districts, green spaces,
//! public land, monuments, protected objects and the forest administration (ADR 0124), from the [`PermitSource`]s of the adapters
//! whose ground the map area touches.
//!
//! The engine knows the kinds of finding, not the region: it measures each finding on the map
//! polygon, counts and groups, and leaves a section out when one of the sources that fill it
//! was not read.

use std::collections::BTreeMap;

use freemapper_adapter_api::permits::{FindingKind, PermitFinding, PermitSourceInfo};
/// The kinds the regional part names, for the writers of the notes.
pub use freemapper_adapter_api::permits::{PermitSection, ProtectedObjectKind};
use geo_types::{Coord, Geometry, MultiPolygon};
use serde::{Deserialize, Serialize};

use crate::permits::notes::{Plane, Share, SourceInput, SourceStatus, status};

/// The most links to monuments the notes give; the others are counted.
pub const MAX_MONUMENT_LINKS: usize = 20;

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
    pub sections: Vec<PermitSection>,
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
            sections: info.sections.to_vec(),
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
    /// The sources of the adapter, in its order.
    pub sources: Vec<RegionalStatus>,
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

/// What the hints need of the regional part of one adapter.
#[derive(Debug, Default)]
pub(crate) struct HintFacts {
    /// The districts on the map that are more than an edge, by share, descending.
    pub districts: Vec<String>,
    /// The districts that hold a monument on the map: for each monument those that are more
    /// than an edge, or its edge districts when it lies only there.
    pub monument_districts: Vec<String>,
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
            _ => None,
        }
    }

    /// The share of the map an area covers, `Some(0.0)` for points on the map; `None` when
    /// the finding is not on the map.
    fn on_map(&self, plane: &Plane) -> Option<f64> {
        match self {
            Shape::Area(area) => plane.share(area),
            Shape::Points(points) => points.iter().any(|c| plane.contains(*c)).then_some(0.0),
        }
    }

    /// Whether the part of the finding on the map lies in `district`.
    fn in_district(&self, plane: &Plane, district: &MultiPolygon<f64>) -> bool {
        match self {
            Shape::Area(area) => plane.share_within(area, district),
            Shape::Points(points) => points
                .iter()
                .any(|c| plane.contains(*c) && geo::Contains::contains(district, c)),
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

fn protected_objects(on_map: &[(&Placed<'_>, f64)]) -> Vec<ProtectedObjectEntry> {
    let mut points: BTreeMap<(ProtectedObjectKind, Option<String>, Option<String>), usize> =
        BTreeMap::new();
    let mut areas: Vec<ProtectedObjectEntry> = Vec::new();
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
            Shape::Points(_) => {
                *points
                    .entry((*kind, name.clone(), citation.clone()))
                    .or_default() += 1;
            }
            Shape::Area(_) => {
                let share = Share::new((), *share);
                areas.push(ProtectedObjectEntry {
                    kind: *kind,
                    name: name.clone(),
                    citation: citation.clone(),
                    count: 1,
                    percent: Some(share.percent),
                    at_edge: share.at_edge,
                });
            }
        }
    }
    let mut all: Vec<ProtectedObjectEntry> = areas;
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
