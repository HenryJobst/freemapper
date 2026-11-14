//! The document model of the permit notes: which administrative units and protected areas the
//! map area lies in, with their shares, which OSM objects tell who runs a part of it, and the
//! hints whom this usually concerns.
//!
//! The notes are text: no entry carries a geometry. The geometries stay in the stored raw
//! files of the sources. Shares are area(map ∩ unit) / area(map), computed in the project's
//! UTM zone.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike as _, NaiveDate};
use freemapper_sources::bfn::{BFN_CREDIT, ProtectedArea};
use freemapper_sources::vg250::{AdminUnit, BKG_CREDIT};

/// The levels and kinds the entries name, for the writers of the notes.
pub use freemapper_sources::bfn::ProtectedKind;
pub use freemapper_sources::vg250::AdminLevel;
use geo_types::{LineString, MultiPolygon, Polygon};
use serde::{Deserialize, Serialize};

use crate::area::MapArea;
use crate::crs::{Crs, LatLon};
use crate::permits::osm::{OsmFinding, OsmFindingKind};
use crate::permits::regional::{HintFacts, RegionalInput, RegionalNotes, assemble_regional};
use crate::project::MapKind;

/// The version of the JSON shape of [`PermitNotes`].
pub const SCHEMA: u32 = 1;

/// Below this share of the map area a unit or protected area only touches the map at its edge.
const EDGE_SHARE: f64 = 0.005;

/// An OSM protected area is left out when its overlap with a BfN area is at least this part of
/// the larger of the two: the two footprints mostly match.
const COVERED_BY_BFN: f64 = 0.5;

/// Overlaps smaller than this are noise of the floating point arithmetic, not a touch.
const MIN_OVERLAP_M2: f64 = 0.01;

/// The Fluent ids of the hints, in the order the notes give them.
pub mod hint_key {
    pub const CITY_STATE: &str = "permits-hint-city-state";
    /// Args `kreis` and `kreisfrei` (`yes`/`no`); both missing when no Kreis is known.
    pub const NATURE_AUTHORITY: &str = "permits-hint-nature-authority";
    pub const NATIONAL_PARK: &str = "permits-hint-national-park";
    pub const NSG_PATHS: &str = "permits-hint-nsg-paths";
    pub const NATURA2000: &str = "permits-hint-natura2000";
    /// Args `gemeinde` and, where there is one, `amt` (kind and name, "Amt Märkische Schweiz").
    pub const ORDER_OFFICE: &str = "permits-hint-order-office";
    /// Arg `operator`.
    pub const PARK_OPERATOR: &str = "permits-hint-park-operator";
    /// Arg `operator`; schoolyard maps only.
    pub const SCHOOL_OPERATOR: &str = "permits-hint-school-operator";
    /// Arg `district`: the Ordnungsamt and the Straßen- und Grünflächenamt of its office. In
    /// place of [`CITY_STATE`] and [`ORDER_OFFICE`] when a regional source read the districts.
    pub const DISTRICT_OFFICE: &str = "permits-hint-district-office";
    /// Arg `district`: a monument on the map lies in it.
    pub const MONUMENT_AUTHORITY: &str = "permits-hint-monument-authority";
    /// Arg `district`: a Naturdenkmal or a geschützter Landschaftsbestandteil on the map lies
    /// in it.
    pub const NATURE_OBJECT_AUTHORITY: &str = "permits-hint-nature-object-authority";
    /// Args `owner` and `percent` (the share of the map area, one decimal, a point).
    pub const OWNER_SHARE: &str = "permits-hint-owner-share";
    /// Arg `rest` (`yes`) when a regional source named public owners: the text speaks of the
    /// rest of the map area.
    pub const OWNERS_UNKNOWN: &str = "permits-hint-owners-unknown";
    /// Args `office`, `administration` and, where the source names them, `units` (joined with
    /// ", "): a forest office that is more than an edge of the map. Forest maps only.
    pub const FOREST_OFFICE: &str = "permits-hint-forest-office";
    /// Arg `office`: `known` when a [`FOREST_OFFICE`] hint is there, else `unknown`. Forest maps
    /// only.
    pub const FOREST_OWNER: &str = "permits-hint-forest-owner";
    /// Args `owner`, `percent` (as for [`OWNER_SHARE`]) and, where known, `office`.
    pub const STATE_FOREST: &str = "permits-hint-state-forest";
}

/// The credit line of the OSM data, as on the map.
const OSM_CREDIT: &str = crate::project::OSM_ATTRIBUTION;

/// What one source gave.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceInput<T> {
    Read { items: Vec<T>, fetched: NaiveDate },
    Unreachable { reason: String },
    NotRead,
}

impl<T> SourceInput<T> {
    pub(crate) fn items(&self) -> &[T] {
        match self {
            SourceInput::Read { items, .. } => items,
            _ => &[],
        }
    }
}

/// What the three national sources and the regional ones gave for the map area.
#[derive(Debug, Clone, PartialEq)]
pub struct Inputs {
    pub admin: SourceInput<AdminUnit>,
    pub protected: SourceInput<ProtectedArea>,
    pub osm: SourceInput<OsmFinding>,
    /// The sources of the region adapters whose ground the map area touches, adapter by
    /// adapter in the order of the registry (ADR 0120).
    pub regional: Vec<RegionalInput>,
}

/// The permit notes of one map area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermitNotes {
    /// [`SCHEMA`].
    pub schema: u32,
    pub created: NaiveDate,
    pub map_kind: MapKind,
    /// The map area in square metres, in the project's UTM zone.
    pub area_m2: f64,
    /// Land, Kreis, Verwaltungsgemeinschaft, Gemeinde; within a level by share, descending.
    pub admin: Vec<Share<AdminEntry>>,
    /// BfN areas in the order of the spec; within a kind by share, descending.
    pub protected: Vec<Share<ProtectedEntry>>,
    /// OSM protected areas no BfN area covers, by share, descending.
    pub osm_protected: Vec<Share<OsmEntry>>,
    /// Operators and owners of public-kind OSM objects on the map, by name.
    pub operators: Vec<OperatorEntry>,
    pub hints: Vec<Hint>,
    /// `vg250`, `bfn`, `osm`, in this order.
    pub sources: Vec<SourceStatus>,
    /// What the region adapters brought, one entry per adapter (ADR 0120). Missing in notes
    /// written before it existed and in notes of a map area no adapter brings sources for.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regional: Vec<RegionalNotes>,
}

/// An entry with its share of the map area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Share<T> {
    pub item: T,
    /// The share of the map area in percent, rounded to one decimal.
    pub percent: f64,
    /// The entry covers less than 0.5 % of the map area.
    pub at_edge: bool,
}

/// An administrative unit of VG250.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminEntry {
    pub level: AdminLevel,
    /// `gen`.
    pub name: String,
    /// `bez`, for example "Kreisfreie Stadt" or "Amt".
    pub kind: String,
    pub ags: String,
}

/// A protected area of the BfN.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtectedEntry {
    pub kind: ProtectedKind,
    pub name: String,
    pub legal_date: Option<NaiveDate>,
    pub legal_title: Option<String>,
    pub legal_link: Option<String>,
    /// The Natura 2000 site page of an FFH or bird area.
    #[serde(default)]
    pub site_link: Option<String>,
    pub year: Option<i32>,
}

/// A protected area of OSM that the BfN does not list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmEntry {
    pub osm_id: String,
    /// The matching `key=value`, for example `boundary=protected_area`.
    pub object: String,
    pub name: Option<String>,
    pub operator: Option<String>,
    pub protect_class: Option<String>,
    pub protection_title: Option<String>,
}

/// How a name is given on an OSM object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorRole {
    Operator,
    Owner,
}

/// One operator or owner with the kinds of object it runs or owns on the map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorEntry {
    pub name: String,
    /// Sorted and without duplicates.
    pub roles: Vec<OperatorRole>,
    /// The `key=value` of the objects, sorted and without duplicates.
    pub objects: Vec<String>,
}

/// A hint, rendered from the Fluent id `key` with `args`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hint {
    pub key: String,
    pub args: BTreeMap<String, String>,
}

impl Hint {
    fn new(key: &str, args: &[(&str, &str)]) -> Self {
        Self {
            key: key.to_string(),
            args: args
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

/// What became of one source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceStatus {
    /// `vg250`, `bfn` or `osm`.
    pub id: String,
    /// `read`, `unreachable` or `not-read`.
    pub state: String,
    pub fetched: Option<NaiveDate>,
    pub reason: Option<String>,
    /// The credit line the licence asks for; only for a source that was read.
    pub credit: Option<String>,
}

/// The permit notes of `area` from what the sources gave. Shares are computed in `crs`, the
/// project's UTM zone; a source that was not read gives no entries and only its status.
pub fn assemble(
    area: &MapArea,
    crs: Crs,
    kind: MapKind,
    inputs: &Inputs,
    today: NaiveDate,
) -> PermitNotes {
    let plane = Plane::new(area, crs);
    let bfn: Vec<MultiPolygon<f64>> = inputs
        .protected
        .items()
        .iter()
        .map(|p| plane.project(&p.geometry))
        .collect();
    let admin = without_own_vwg(admin_shares(&plane, inputs.admin.items()));
    let protected = protected_shares(&plane, inputs.protected.items(), &bfn);
    let osm: Vec<(&OsmFinding, MultiPolygon<f64>, f64)> = inputs
        .osm
        .items()
        .iter()
        .filter_map(|f| {
            let geometry = plane.project(&f.geometry);
            let share = plane.share(&geometry)?;
            Some((f, geometry, share))
        })
        .collect();
    let osm_protected = osm_protected(&osm, &bfn);
    let operated: Vec<&OsmFinding> = osm
        .iter()
        .map(|(f, _, _)| *f)
        .filter(|f| f.kind == OsmFindingKind::Operated)
        .collect();
    let (regional, facts) = assemble_regional(&plane, &inputs.regional);
    let hints = hints(kind, &admin, &protected, &operated, &facts);
    PermitNotes {
        schema: SCHEMA,
        created: today,
        map_kind: kind,
        area_m2: plane.area_m2,
        admin,
        protected,
        osm_protected,
        operators: operators(&operated),
        hints,
        sources: vec![
            status("vg250", &inputs.admin, BKG_CREDIT),
            status("bfn", &inputs.protected, BFN_CREDIT),
            status("osm", &inputs.osm, OSM_CREDIT),
        ],
        regional,
    }
}

/// The map area in the project's UTM zone, against which shares are measured.
pub(crate) struct Plane {
    crs: Crs,
    map: MultiPolygon<f64>,
    area_m2: f64,
}

impl Plane {
    fn new(area: &MapArea, crs: Crs) -> Self {
        use geo::Area as _;
        let mut plane = Self {
            crs,
            map: MultiPolygon::new(vec![]),
            area_m2: 0.0,
        };
        plane.map = plane.project(&MultiPolygon::new(vec![area.polygon.clone()]));
        plane.area_m2 = plane.map.unsigned_area();
        plane
    }

    /// A position (WGS 84, x = longitude) in the plane; `None` when it is no finite position.
    pub(crate) fn project_point(&self, c: geo_types::Coord<f64>) -> Option<geo_types::Coord<f64>> {
        if !((-180.0..=180.0).contains(&c.x) && (-90.0..=90.0).contains(&c.y)) {
            return None;
        }
        let p = self.crs.forward(LatLon { lat: c.y, lon: c.x });
        (p.easting.is_finite() && p.northing.is_finite())
            .then_some(geo_types::coord! { x: p.easting, y: p.northing })
    }

    /// Whether a projected position lies inside the map area (on its border counts).
    pub(crate) fn contains(&self, c: geo_types::Coord<f64>) -> bool {
        use geo::Intersects as _;
        self.map.intersects(&c)
    }

    /// Whether the part of the projected `geometry` on the map overlaps `district` (both
    /// projected) by more than the noise of the arithmetic.
    pub(crate) fn share_within(
        &self,
        geometry: &MultiPolygon<f64>,
        district: &MultiPolygon<f64>,
    ) -> bool {
        use geo::BooleanOps as _;
        if overlap(geometry, district) < MIN_OVERLAP_M2 {
            return false;
        }
        let on_map = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.map.intersection(geometry)
        }));
        on_map.is_ok_and(|on_map| overlap(&on_map, district) >= MIN_OVERLAP_M2)
    }

    /// `geometry` (WGS 84, x = longitude) in the plane, without degenerate rings: fewer than
    /// four corners, or a corner that is not a finite position.
    pub(crate) fn project(&self, geometry: &MultiPolygon<f64>) -> MultiPolygon<f64> {
        let ring = |ring: &LineString<f64>| -> Option<LineString<f64>> {
            if ring.0.len() < 4
                || !ring
                    .0
                    .iter()
                    .all(|c| (-180.0..=180.0).contains(&c.x) && (-90.0..=90.0).contains(&c.y))
            {
                return None;
            }
            let projected: LineString<f64> = ring
                .0
                .iter()
                .map(|c| {
                    let p = self.crs.forward(LatLon { lat: c.y, lon: c.x });
                    geo_types::coord! { x: p.easting, y: p.northing }
                })
                .collect();
            projected
                .0
                .iter()
                .all(|c| c.x.is_finite() && c.y.is_finite())
                .then_some(projected)
        };
        MultiPolygon::new(
            geometry
                .0
                .iter()
                .filter_map(|p| {
                    let exterior = ring(p.exterior())?;
                    Some(Polygon::new(
                        exterior,
                        p.interiors().iter().filter_map(ring).collect(),
                    ))
                })
                .collect(),
        )
    }

    /// The share of the map area a projected geometry covers, from 0 to 1; `None` when it is
    /// not on the map.
    pub(crate) fn share(&self, projected: &MultiPolygon<f64>) -> Option<f64> {
        let overlap = overlap(&self.map, projected);
        (self.area_m2 > 0.0 && overlap >= MIN_OVERLAP_M2).then(|| (overlap / self.area_m2).min(1.0))
    }
}

/// The area two projected geometries share. The boolean operations of `geo` 0.33 (i_overlay)
/// do not panic on the inputs `Plane::project` lets through; should one still panic, the two
/// count as apart instead of ending the run.
fn overlap(a: &MultiPolygon<f64>, b: &MultiPolygon<f64>) -> f64 {
    use geo::{Area as _, BooleanOps as _, BoundingRect as _, Intersects as _};
    match (a.bounding_rect(), b.bounding_rect()) {
        (Some(ra), Some(rb)) if ra.intersects(&rb) => {}
        _ => return 0.0,
    }
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        a.intersection(b).unsigned_area()
    }))
    .unwrap_or(0.0)
}

impl<T> Share<T> {
    pub(crate) fn new(item: T, share: f64) -> Self {
        Self {
            item,
            percent: (share * 1000.0).round() / 10.0,
            at_edge: share < EDGE_SHARE,
        }
    }
}

fn level_rank(level: AdminLevel) -> usize {
    AdminLevel::ALL
        .iter()
        .position(|l| *l == level)
        .unwrap_or(usize::MAX)
}

fn kind_rank(kind: ProtectedKind) -> usize {
    ProtectedKind::ALL
        .iter()
        .position(|k| *k == kind)
        .unwrap_or(usize::MAX)
}

/// The units on the map, top down; within a level by share, descending.
fn admin_shares(plane: &Plane, units: &[AdminUnit]) -> Vec<Share<AdminEntry>> {
    let mut measured: Vec<(f64, AdminEntry)> = units
        .iter()
        .filter_map(|u| {
            let share = plane.share(&plane.project(&u.geometry))?;
            Some((
                share,
                AdminEntry {
                    level: u.level,
                    name: u.name.clone(),
                    kind: u.kind.clone(),
                    ags: u.ags.clone(),
                },
            ))
        })
        .collect();
    measured.sort_by(|(a, x), (b, y)| {
        level_rank(x.level)
            .cmp(&level_rank(y.level))
            .then(b.total_cmp(a))
            .then_with(|| x.name.cmp(&y.name))
    });
    measured
        .into_iter()
        .map(|(share, entry)| Share::new(entry, share))
        .collect()
}

/// `units` without the `vg250_vwg` entries that are no Verwaltungsgemeinschaft: VG250 lists an
/// amtsfreie or verbandsfreie Gemeinde (and a kreisfreie Stadt, as "Amtsfreie Gemeinde") there
/// too. Only the kind tells them apart: an Amt or a Verbandsgemeinde often carries the name of
/// one of its Gemeinden.
fn without_own_vwg(units: Vec<Share<AdminEntry>>) -> Vec<Share<AdminEntry>> {
    units
        .into_iter()
        .filter(|s| s.item.level != AdminLevel::Verwaltungsgemeinschaft || !is_free(&s.item))
        .collect()
}

/// A `vg250_vwg` entry that is a Gemeinde of its own ("Amtsfreie Gemeinde", "Verbandsfreie
/// Gemeinde").
fn is_free(unit: &AdminEntry) -> bool {
    unit.kind.to_lowercase().contains("frei")
}

/// The BfN areas on the map in the order of the spec, within a kind by share, descending; the
/// zones of a Biosphärenreservat right after their reserve.
fn protected_shares(
    plane: &Plane,
    areas: &[ProtectedArea],
    projected: &[MultiPolygon<f64>],
) -> Vec<Share<ProtectedEntry>> {
    let mut measured: Vec<(f64, ProtectedEntry)> = areas
        .iter()
        .zip(projected)
        .filter_map(|(p, geometry)| {
            let share = plane.share(geometry)?;
            Some((
                share,
                ProtectedEntry {
                    kind: p.kind,
                    name: p.name.clone(),
                    legal_date: p.legal_date,
                    legal_title: p.legal_title.clone(),
                    legal_link: p.legal_link.clone(),
                    site_link: p.site_link.clone(),
                    year: p.year,
                },
            ))
        })
        .collect();
    measured.sort_by(|(a, x), (b, y)| {
        kind_rank(x.kind)
            .cmp(&kind_rank(y.kind))
            .then(b.total_cmp(a))
            .then_with(|| x.name.cmp(&y.name))
    });
    let (mut zones, rest): (Vec<_>, Vec<_>) = measured
        .into_iter()
        .partition(|(_, e)| e.kind == ProtectedKind::BiosphaerenreservatZone);
    let zone_rank = kind_rank(ProtectedKind::BiosphaerenreservatZone);
    let mut ordered = Vec::new();
    for (share, entry) in rest {
        // Zones whose reserve is not on the map keep the place of their kind.
        if kind_rank(entry.kind) > zone_rank {
            ordered.append(&mut zones);
        }
        let reserve =
            (entry.kind == ProtectedKind::Biosphaerenreservat).then(|| format!("{} (", entry.name));
        ordered.push((share, entry));
        if let Some(prefix) = reserve {
            let (mine, others): (Vec<_>, Vec<_>) = std::mem::take(&mut zones)
                .into_iter()
                .partition(|(_, z)| z.name.starts_with(&prefix));
            ordered.extend(mine);
            zones = others;
        }
    }
    ordered.append(&mut zones);
    ordered
        .into_iter()
        .map(|(share, entry)| Share::new(entry, share))
        .collect()
}

/// The OSM protected areas on the map that the BfN does not list: one is the same as a BfN area
/// when their overlap is at least half of the larger of the two. A small OSM area inside a large
/// LSG or Naturpark stays. By share, descending.
fn osm_protected(
    on_map: &[(&OsmFinding, MultiPolygon<f64>, f64)],
    bfn: &[MultiPolygon<f64>],
) -> Vec<Share<OsmEntry>> {
    use geo::Area as _;
    let bfn: Vec<(&MultiPolygon<f64>, f64)> = bfn.iter().map(|g| (g, g.unsigned_area())).collect();
    let mut kept: Vec<(f64, OsmEntry)> = on_map
        .iter()
        .filter(|(f, _, _)| f.kind == OsmFindingKind::ProtectedArea)
        .filter(|(_, geometry, _)| {
            let own = geometry.unsigned_area();
            !bfn.iter().any(|(b, area)| {
                let larger = own.max(*area);
                larger > 0.0 && overlap(geometry, b) / larger >= COVERED_BY_BFN
            })
        })
        .map(|(f, _, share)| {
            (
                *share,
                OsmEntry {
                    osm_id: f.osm_id.clone(),
                    object: f.object.clone(),
                    name: f.name.clone(),
                    operator: f.operator.clone(),
                    protect_class: f.protect_class.clone(),
                    protection_title: f.protection_title.clone(),
                },
            )
        })
        .collect();
    kept.sort_by(|(a, x), (b, y)| b.total_cmp(a).then_with(|| x.osm_id.cmp(&y.osm_id)));
    kept.into_iter()
        .map(|(share, entry)| Share::new(entry, share))
        .collect()
}

/// The operators and owners of the public-kind objects on the map, by name.
fn operators(operated: &[&OsmFinding]) -> Vec<OperatorEntry> {
    let mut by_name: BTreeMap<&str, (BTreeSet<OperatorRole>, BTreeSet<&str>)> = BTreeMap::new();
    for f in operated {
        for (role, name) in [
            (OperatorRole::Operator, &f.operator),
            (OperatorRole::Owner, &f.owner),
        ] {
            if let Some(name) = name {
                let (roles, objects) = by_name.entry(name).or_default();
                roles.insert(role);
                objects.insert(&f.object);
            }
        }
    }
    by_name
        .into_iter()
        .map(|(name, (roles, objects))| OperatorEntry {
            name: name.to_string(),
            roles: roles.into_iter().collect(),
            objects: objects.into_iter().map(str::to_string).collect(),
        })
        .collect()
}

/// Berlin (`11`) and Hamburg (`02`): the Land is the Gemeinde.
fn is_city_state(land: &AdminEntry) -> bool {
    land.level == AdminLevel::Land && (land.ags.starts_with("11") || land.ags.starts_with("02"))
}

/// The kinds the nature authority looks after.
fn needs_nature_authority(kind: ProtectedKind) -> bool {
    use ProtectedKind::*;
    matches!(
        kind,
        Naturschutzgebiet
            | Landschaftsschutzgebiet
            | Ffh
            | Vogelschutzgebiet
            | Biosphaerenreservat
            | BiosphaerenreservatZone
            | NationalesNaturmonument
    )
}

/// The names that run the objects of `objects` on the map: `operator`, else `owner`.
fn runners<'a>(operated: &[&'a OsmFinding], objects: &[&str]) -> BTreeSet<&'a str> {
    operated
        .iter()
        .filter(|f| objects.contains(&f.object.as_str()))
        .filter_map(|f| f.operator.as_deref().or(f.owner.as_deref()))
        .collect()
}

/// The hints, in the order of [`hint_key`], each key and args once.
fn hints(
    kind: MapKind,
    admin: &[Share<AdminEntry>],
    protected: &[Share<ProtectedEntry>],
    operated: &[&OsmFinding],
    regional: &[HintFacts],
) -> Vec<Hint> {
    // Within a level the first unit has the largest share.
    let largest = |level: AdminLevel| admin.iter().map(|s| &s.item).find(|u| u.level == level);
    let any_protected =
        |test: &dyn Fn(ProtectedKind) -> bool| protected.iter().any(|s| test(s.item.kind));
    let mut hints = Vec::new();
    // The districts a regional source read: their offices take the place of the general
    // hint of the city state.
    let districts: Vec<&str> = regional
        .iter()
        .flat_map(|f| f.districts.iter().map(String::as_str))
        .collect();

    if admin.iter().any(|s| is_city_state(&s.item)) && districts.is_empty() {
        hints.push(Hint::new(hint_key::CITY_STATE, &[]));
    }
    if any_protected(&needs_nature_authority) {
        let land = largest(AdminLevel::Land).filter(|l| is_city_state(l));
        let authority = match (land, largest(AdminLevel::Kreis)) {
            (Some(land), _) => Some((land.name.as_str(), true)),
            (None, Some(kreis)) => Some((kreis.name.as_str(), kreis.kind == "Kreisfreie Stadt")),
            (None, None) => None,
        };
        hints.push(match authority {
            Some((name, kreisfrei)) => Hint::new(
                hint_key::NATURE_AUTHORITY,
                &[
                    ("kreis", name),
                    ("kreisfrei", if kreisfrei { "yes" } else { "no" }),
                ],
            ),
            None => Hint::new(hint_key::NATURE_AUTHORITY, &[]),
        });
    }
    if any_protected(&|k| k == ProtectedKind::Nationalpark) {
        hints.push(Hint::new(hint_key::NATIONAL_PARK, &[]));
    }
    if any_protected(&|k| k == ProtectedKind::Naturschutzgebiet) {
        hints.push(Hint::new(hint_key::NSG_PATHS, &[]));
    }
    if any_protected(&|k| matches!(k, ProtectedKind::Ffh | ProtectedKind::Vogelschutzgebiet)) {
        hints.push(Hint::new(hint_key::NATURA2000, &[]));
    }
    // A Revier covers towns as well: the forest hints are for forest maps alone.
    if kind == MapKind::Forest {
        let offices: Vec<_> = regional.iter().flat_map(|f| &f.forest_offices).collect();
        for office in &offices {
            let units = office.units.join(", ");
            let mut args = vec![
                ("office", office.office.as_str()),
                ("administration", office.administration.as_str()),
            ];
            if !units.is_empty() {
                args.push(("units", units.as_str()));
            }
            hints.push(Hint::new(hint_key::FOREST_OFFICE, &args));
        }
        let office = if offices.is_empty() {
            "unknown"
        } else {
            "known"
        };
        hints.push(Hint::new(hint_key::FOREST_OWNER, &[("office", office)]));
    }
    let in_city_state = largest(AdminLevel::Land).is_some_and(is_city_state);
    if let Some(gemeinde) =
        largest(AdminLevel::Gemeinde).filter(|_| districts.is_empty() || !in_city_state)
    {
        // `without_own_vwg` has left out the amtsfreie Gemeinden; an Amt may carry the name
        // of the Gemeinde.
        let amt = largest(AdminLevel::Verwaltungsgemeinschaft)
            .filter(|v| !is_free(v))
            .map(|v| format!("{} {}", v.kind, v.name).trim().to_string());
        let mut args = vec![("gemeinde", gemeinde.name.as_str())];
        if let Some(amt) = &amt {
            args.push(("amt", amt.as_str()));
        }
        hints.push(Hint::new(hint_key::ORDER_OFFICE, &args));
    }
    for district in &districts {
        hints.push(Hint::new(
            hint_key::DISTRICT_OFFICE,
            &[("district", district)],
        ));
    }
    for district in regional.iter().flat_map(|f| &f.monument_districts) {
        hints.push(Hint::new(
            hint_key::MONUMENT_AUTHORITY,
            &[("district", district)],
        ));
    }
    for district in regional.iter().flat_map(|f| &f.nature_districts) {
        hints.push(Hint::new(
            hint_key::NATURE_OBJECT_AUTHORITY,
            &[("district", district)],
        ));
    }
    for operator in runners(operated, &["leisure=park", "leisure=garden"]) {
        hints.push(Hint::new(
            hint_key::PARK_OPERATOR,
            &[("operator", operator)],
        ));
    }
    if kind == MapKind::Schoolyard {
        let schools = [
            "amenity=school",
            "amenity=kindergarten",
            "amenity=college",
            "amenity=university",
        ];
        for operator in runners(operated, &schools) {
            hints.push(Hint::new(
                hint_key::SCHOOL_OPERATOR,
                &[("operator", operator)],
            ));
        }
    }
    let owners: Vec<&(Option<String>, f64)> = regional
        .iter()
        .flat_map(|f| f.owners.iter().flatten())
        .collect();
    for (owner, percent) in &owners {
        let percent = format!("{percent:.1}");
        hints.push(match owner {
            Some(owner) => Hint::new(
                hint_key::OWNER_SHARE,
                &[("owner", owner), ("percent", &percent)],
            ),
            None => Hint::new(
                hint_key::OWNER_SHARE,
                &[("unnamed", "yes"), ("percent", &percent)],
            ),
        });
    }
    let state_forest: Vec<_> = regional.iter().flat_map(|f| &f.state_forest).collect();
    for (forest, percent) in &state_forest {
        let percent = format!("{percent:.1}");
        let mut args = vec![
            ("owner", forest.owner.as_str()),
            ("percent", percent.as_str()),
        ];
        if let Some(office) = &forest.office {
            args.push(("office", office.as_str()));
        }
        hints.push(Hint::new(hint_key::STATE_FOREST, &args));
    }
    hints.push(if owners.is_empty() && state_forest.is_empty() {
        Hint::new(hint_key::OWNERS_UNKNOWN, &[])
    } else {
        Hint::new(hint_key::OWNERS_UNKNOWN, &[("rest", "yes")])
    });

    let mut once: Vec<Hint> = Vec::with_capacity(hints.len());
    for hint in hints {
        if !once.contains(&hint) {
            once.push(hint);
        }
    }
    once
}

/// The status of one source; the credit, with the year of the fetch, only when it was read.
pub(crate) fn status<T>(id: &str, input: &SourceInput<T>, credit: &str) -> SourceStatus {
    let (state, fetched, reason) = match input {
        SourceInput::Read { fetched, .. } => ("read", Some(*fetched), None),
        SourceInput::Unreachable { reason } => ("unreachable", None, Some(reason.clone())),
        SourceInput::NotRead => ("not-read", None, None),
    };
    SourceStatus {
        id: id.to_string(),
        state: state.to_string(),
        fetched,
        reason,
        credit: fetched.map(|f| credit.replace("{year}", &f.year().to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permits::regional::MAX_MONUMENT_LINKS;
    use geo_types::{Coord, LineString, Polygon};

    const FETCHED: NaiveDate = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();

    /// A box in degrees near Berlin, x = longitude.
    fn square(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> MultiPolygon<f64> {
        let c = |x, y| Coord { x, y };
        MultiPolygon::new(vec![Polygon::new(
            LineString::new(vec![
                c(min_lon, min_lat),
                c(max_lon, min_lat),
                c(max_lon, max_lat),
                c(min_lon, max_lat),
                c(min_lon, min_lat),
            ]),
            vec![],
        )])
    }

    /// The map area: 13.0–13.1 E, 52.0–52.1 N.
    fn map_area() -> MapArea {
        MapArea {
            polygon: square(13.0, 52.0, 13.1, 52.1).0.remove(0),
            source_id: None,
        }
    }

    fn unit(
        level: AdminLevel,
        name: &str,
        kind: &str,
        ags: &str,
        g: MultiPolygon<f64>,
    ) -> AdminUnit {
        AdminUnit {
            level,
            name: name.into(),
            kind: kind.into(),
            ags: ags.into(),
            geometry: g,
        }
    }

    fn protected(kind: ProtectedKind, name: &str, g: MultiPolygon<f64>) -> ProtectedArea {
        ProtectedArea {
            kind,
            name: name.into(),
            legal_date: None,
            legal_title: None,
            legal_link: None,
            site_link: None,
            year: Some(2024),
            geometry: g,
        }
    }

    fn finding(kind: OsmFindingKind, id: &str, object: &str, g: MultiPolygon<f64>) -> OsmFinding {
        OsmFinding {
            kind,
            osm_id: id.into(),
            object: object.into(),
            name: Some(format!("Name {id}")),
            operator: None,
            owner: None,
            protect_class: None,
            protection_title: None,
            geometry: g,
        }
    }

    fn read<T>(items: Vec<T>) -> SourceInput<T> {
        SourceInput::Read {
            items,
            fetched: FETCHED,
        }
    }

    fn inputs(
        admin: Vec<AdminUnit>,
        protected: Vec<ProtectedArea>,
        osm: Vec<OsmFinding>,
    ) -> Inputs {
        Inputs {
            admin: read(admin),
            protected: read(protected),
            osm: read(osm),
            regional: Vec::new(),
        }
    }

    fn notes(kind: MapKind, inputs: &Inputs) -> PermitNotes {
        assemble(&map_area(), Crs::Utm33, kind, inputs, FETCHED)
    }

    fn hint<'a>(notes: &'a PermitNotes, key: &str) -> Vec<&'a Hint> {
        notes.hints.iter().filter(|h| h.key == key).collect()
    }

    fn args(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        Hint::new("", pairs).args
    }

    fn brandenburg() -> AdminUnit {
        unit(
            AdminLevel::Land,
            "Brandenburg",
            "Land",
            "12",
            square(12.0, 51.0, 15.0, 54.0),
        )
    }

    #[test]
    fn shares_of_an_area_across_two_kreise() {
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    unit(
                        AdminLevel::Kreis,
                        "Kreis Ost",
                        "Landkreis",
                        "12002",
                        square(13.07, 51.9, 13.3, 52.2),
                    ),
                    unit(
                        AdminLevel::Kreis,
                        "Kreis West",
                        "Landkreis",
                        "12001",
                        square(12.8, 51.9, 13.07, 52.2),
                    ),
                    brandenburg(),
                    // Beside the map area: not listed.
                    unit(
                        AdminLevel::Kreis,
                        "Kreis Fern",
                        "Landkreis",
                        "12003",
                        square(13.5, 51.9, 13.6, 52.2),
                    ),
                ],
                vec![],
                vec![],
            ),
        );
        let listed: Vec<(AdminLevel, &str)> = found
            .admin
            .iter()
            .map(|s| (s.item.level, s.item.name.as_str()))
            .collect();
        assert_eq!(
            listed,
            [
                (AdminLevel::Land, "Brandenburg"),
                (AdminLevel::Kreis, "Kreis West"),
                (AdminLevel::Kreis, "Kreis Ost"),
            ]
        );
        assert!((found.admin[0].percent - 100.0).abs() <= 0.1);
        assert!(
            (found.admin[1].percent - 70.0).abs() <= 0.5,
            "{:?}",
            found.admin[1]
        );
        assert!(
            (found.admin[2].percent - 30.0).abs() <= 0.5,
            "{:?}",
            found.admin[2]
        );
        let kreise: f64 = found.admin[1..].iter().map(|s| s.percent).sum();
        assert!((kreise - 100.0).abs() <= 1.0, "{kreise}");
        assert!(found.admin.iter().all(|s| !s.at_edge));
        // Rounded to one decimal.
        for s in &found.admin {
            assert_eq!((s.percent * 10.0).round() / 10.0, s.percent);
        }
        // About 11.1 km × 6.9 km.
        assert!((7.0e7..8.0e7).contains(&found.area_m2), "{}", found.area_m2);
    }

    #[test]
    fn a_unit_touching_the_edge_is_marked() {
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    unit(
                        AdminLevel::Gemeinde,
                        "Randdorf",
                        "Gemeinde",
                        "12002002",
                        square(13.0997, 51.9, 13.3, 52.2),
                    ),
                    unit(
                        AdminLevel::Gemeinde,
                        "Hauptstadt",
                        "Stadt",
                        "12001001",
                        square(12.8, 51.9, 13.0997, 52.2),
                    ),
                ],
                vec![protected(
                    ProtectedKind::Landschaftsschutzgebiet,
                    "Randlandschaft",
                    // As wide as the map area: a long edge in degrees bows away in UTM.
                    square(13.0, 52.0997, 13.1, 53.0),
                )],
                vec![],
            ),
        );
        let gemeinden: Vec<(&str, bool)> = found
            .admin
            .iter()
            .map(|s| (s.item.name.as_str(), s.at_edge))
            .collect();
        assert_eq!(gemeinden, [("Hauptstadt", false), ("Randdorf", true)]);
        assert!(found.admin[1].percent < 0.5);
        assert_eq!(found.protected.len(), 1);
        assert!(found.protected[0].at_edge);
        // An area at the edge still asks for the nature authority.
        assert_eq!(hint(&found, hint_key::NATURE_AUTHORITY).len(), 1);
    }

    #[test]
    fn protected_areas_follow_the_order_of_the_spec() {
        use ProtectedKind::*;
        // Shares by width: a box from 13.0 to 13.0 + w covers w × 1000 % of the map area.
        let part = |w: f64| square(12.9, 51.9, 13.0 + w, 52.2);
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![],
                vec![
                    protected(Naturpark, "Naturpark", part(0.1)),
                    protected(Landschaftsschutzgebiet, "LSG", part(0.1)),
                    protected(BiosphaerenreservatZone, "Reserve B (Kernzone)", part(0.01)),
                    protected(Naturschutzgebiet, "NSG klein", part(0.02)),
                    protected(
                        BiosphaerenreservatZone,
                        "Reserve A (Pflegezone)",
                        part(0.05),
                    ),
                    protected(Biosphaerenreservat, "Reserve B", part(0.03)),
                    protected(Vogelschutzgebiet, "SPA", part(0.1)),
                    protected(Ffh, "FFH", part(0.1)),
                    protected(Naturschutzgebiet, "NSG groß", part(0.06)),
                    protected(Biosphaerenreservat, "Reserve A", part(0.08)),
                    protected(NationalesNaturmonument, "NNM", part(0.1)),
                    protected(Nationalpark, "NLP", part(0.1)),
                    // Beside the map area: not listed.
                    protected(
                        Naturschutzgebiet,
                        "NSG fern",
                        square(14.0, 52.0, 14.1, 52.1),
                    ),
                ],
                vec![],
            ),
        );
        let names: Vec<&str> = found
            .protected
            .iter()
            .map(|s| s.item.name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "NLP",
                "NNM",
                "NSG groß",
                "NSG klein",
                "FFH",
                "SPA",
                "Reserve A",
                "Reserve A (Pflegezone)",
                "Reserve B",
                "Reserve B (Kernzone)",
                "LSG",
                "Naturpark",
            ]
        );
        assert!((found.protected[2].percent - 60.0).abs() <= 0.5);
        assert_eq!(found.protected[0].item.year, Some(2024));
        assert_eq!(hint(&found, hint_key::NATIONAL_PARK).len(), 1);
        assert_eq!(hint(&found, hint_key::NSG_PATHS).len(), 1);
        assert_eq!(hint(&found, hint_key::NATURA2000).len(), 1);
    }

    fn osm_protected(id: &str, g: MultiPolygon<f64>) -> OsmFinding {
        let mut f = finding(
            OsmFindingKind::ProtectedArea,
            id,
            "boundary=protected_area",
            g,
        );
        f.protection_title = Some("Geschützter Landschaftsbestandteil".into());
        f
    }

    #[test]
    fn an_osm_area_the_bfn_covers_is_dropped() {
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![],
                vec![
                    protected(
                        ProtectedKind::Naturschutzgebiet,
                        "NSG",
                        square(13.0, 52.0, 13.05, 52.1),
                    ),
                    protected(
                        ProtectedKind::Landschaftsschutzgebiet,
                        "LSG groß",
                        square(13.055, 51.0, 14.0, 53.0),
                    ),
                ],
                vec![
                    // The NSG again, slightly shifted: the two cover each other mostly.
                    osm_protected("way/1", square(13.001, 52.0, 13.051, 52.1)),
                ],
            ),
        );
        assert!(found.osm_protected.is_empty(), "{:?}", found.osm_protected);
    }

    /// The BfN merges large areas (LSG, Naturpark) that hold small ones it does not list, such
    /// as a Geschützter Landschaftsbestandteil. Those stay, and so does a large OSM area around
    /// a small BfN one: only footprints that mostly match are the same area.
    #[test]
    fn a_small_osm_area_inside_a_large_lsg_stays() {
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![],
                vec![
                    protected(
                        ProtectedKind::Landschaftsschutzgebiet,
                        "LSG groß",
                        square(13.055, 51.0, 14.0, 53.0),
                    ),
                    protected(
                        ProtectedKind::Naturschutzgebiet,
                        "NSG klein",
                        square(13.01, 52.01, 13.02, 52.02),
                    ),
                ],
                vec![
                    // A small area inside the large LSG.
                    osm_protected("way/2", square(13.06, 52.05, 13.07, 52.06)),
                    // A large area around the small NSG.
                    osm_protected("way/5", square(13.0, 52.0, 13.05, 52.05)),
                ],
            ),
        );
        let ids: Vec<&str> = found
            .osm_protected
            .iter()
            .map(|s| s.item.osm_id.as_str())
            .collect();
        assert_eq!(ids, ["way/5", "way/2"], "{:?}", found.osm_protected);
    }

    #[test]
    fn an_osm_area_beside_it_stays() {
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![],
                vec![protected(
                    ProtectedKind::Naturschutzgebiet,
                    "NSG",
                    square(13.0, 52.0, 13.03, 52.1),
                )],
                vec![
                    // Overlaps the NSG with 0.01 of its 0.04 width: a third of the smaller.
                    osm_protected("way/3", square(13.02, 52.0, 13.06, 52.1)),
                    // Not on the map: not listed.
                    osm_protected("way/4", square(13.5, 52.0, 13.6, 52.1)),
                ],
            ),
        );
        assert_eq!(found.osm_protected.len(), 1);
        let kept = &found.osm_protected[0];
        assert_eq!(kept.item.osm_id, "way/3");
        assert_eq!(kept.item.object, "boundary=protected_area");
        assert_eq!(
            kept.item.protection_title.as_deref(),
            Some("Geschützter Landschaftsbestandteil")
        );
        assert!((kept.percent - 40.0).abs() <= 0.5, "{kept:?}");
    }

    #[test]
    fn a_kreisfreie_stadt_is_its_own_nature_authority() {
        let lsg = || {
            vec![protected(
                ProtectedKind::Landschaftsschutzgebiet,
                "LSG",
                square(12.0, 51.0, 14.0, 53.0),
            )]
        };
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    brandenburg(),
                    unit(
                        AdminLevel::Kreis,
                        "Potsdam",
                        "Kreisfreie Stadt",
                        "12054",
                        square(12.9, 51.9, 13.2, 52.2),
                    ),
                ],
                lsg(),
                vec![],
            ),
        );
        let found = hint(&found, hint_key::NATURE_AUTHORITY);
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].args,
            args(&[("kreis", "Potsdam"), ("kreisfrei", "yes")])
        );

        // A Landkreis with the larger share, a kreisfreie Stadt with the smaller one.
        let landkreis = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    brandenburg(),
                    unit(
                        AdminLevel::Kreis,
                        "Potsdam",
                        "Kreisfreie Stadt",
                        "12054",
                        square(12.9, 51.9, 13.02, 52.2),
                    ),
                    unit(
                        AdminLevel::Kreis,
                        "Potsdam-Mittelmark",
                        "Landkreis",
                        "12069",
                        square(13.02, 51.9, 13.2, 52.2),
                    ),
                ],
                lsg(),
                vec![],
            ),
        );
        let found = hint(&landkreis, hint_key::NATURE_AUTHORITY);
        assert_eq!(
            found[0].args,
            args(&[("kreis", "Potsdam-Mittelmark"), ("kreisfrei", "no")])
        );
        assert!(hint(&landkreis, hint_key::CITY_STATE).is_empty());

        // No protected area of a kind the nature authority looks after: no hint.
        let naturpark = notes(
            MapKind::Sprint,
            &inputs(
                vec![brandenburg()],
                vec![protected(
                    ProtectedKind::Naturpark,
                    "Naturpark",
                    square(12.0, 51.0, 14.0, 53.0),
                )],
                vec![],
            ),
        );
        assert!(hint(&naturpark, hint_key::NATURE_AUTHORITY).is_empty());
    }

    #[test]
    fn berlin_gets_the_city_state_hint() {
        let berlin = || square(12.9, 51.9, 13.2, 52.2);
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    unit(
                        AdminLevel::Gemeinde,
                        "Berlin",
                        "Stadt",
                        "11000000",
                        berlin(),
                    ),
                    unit(
                        AdminLevel::Kreis,
                        "Berlin",
                        "Kreisfreie Stadt",
                        "11000",
                        berlin(),
                    ),
                    unit(AdminLevel::Land, "Berlin", "Land", "11", berlin()),
                ],
                vec![protected(
                    ProtectedKind::Naturschutzgebiet,
                    "NSG",
                    square(13.0, 52.0, 13.05, 52.05),
                )],
                vec![],
            ),
        );
        let keys: Vec<&str> = found.hints.iter().map(|h| h.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                hint_key::CITY_STATE,
                hint_key::NATURE_AUTHORITY,
                hint_key::NSG_PATHS,
                hint_key::ORDER_OFFICE,
                hint_key::OWNERS_UNKNOWN,
            ]
        );
        assert_eq!(
            hint(&found, hint_key::NATURE_AUTHORITY)[0].args,
            args(&[("kreis", "Berlin"), ("kreisfrei", "yes")])
        );
        let levels: Vec<AdminLevel> = found.admin.iter().map(|s| s.item.level).collect();
        assert_eq!(
            levels,
            [AdminLevel::Land, AdminLevel::Kreis, AdminLevel::Gemeinde]
        );
    }

    #[test]
    fn an_amt_is_named_in_the_order_office_hint() {
        let all = || square(12.9, 51.9, 13.2, 52.2);
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    unit(AdminLevel::Gemeinde, "Buckow", "Stadt", "12064061", all()),
                    unit(
                        AdminLevel::Verwaltungsgemeinschaft,
                        "Märkische Schweiz",
                        "Amt",
                        "12064",
                        all(),
                    ),
                    brandenburg(),
                ],
                vec![],
                vec![],
            ),
        );
        let office = hint(&found, hint_key::ORDER_OFFICE);
        assert_eq!(office.len(), 1);
        assert_eq!(
            office[0].args,
            args(&[("amt", "Amt Märkische Schweiz"), ("gemeinde", "Buckow")])
        );

        // An amtsfreie Gemeinde is its own Verwaltungsgemeinschaft in VG250: no Amt is named.
        let amtsfrei = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    unit(
                        AdminLevel::Gemeinde,
                        "Strausberg",
                        "Stadt",
                        "12064472",
                        all(),
                    ),
                    unit(
                        AdminLevel::Verwaltungsgemeinschaft,
                        "Strausberg",
                        "Amtsfreie Gemeinde",
                        "12064472",
                        all(),
                    ),
                ],
                vec![],
                vec![],
            ),
        );
        assert_eq!(
            hint(&amtsfrei, hint_key::ORDER_OFFICE)[0].args,
            args(&[("gemeinde", "Strausberg")])
        );
    }

    /// VG250 lists an amtsfreie Gemeinde and a kreisfreie Stadt in `vg250_vwg` too, under the
    /// name of the Gemeinde (live for Sanssouci: "Amtsfreie Gemeinde Potsdam"). That is no
    /// Verwaltungsgemeinschaft and is left out of the list.
    #[test]
    fn an_amtsfreie_gemeinde_is_no_verwaltungsgemeinschaft() {
        let all = || square(12.9, 51.9, 13.2, 52.2);
        let levels = |found: &PermitNotes| -> Vec<(AdminLevel, String)> {
            found
                .admin
                .iter()
                .map(|s| (s.item.level, s.item.name.clone()))
                .collect()
        };
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    brandenburg(),
                    unit(
                        AdminLevel::Kreis,
                        "Potsdam",
                        "Kreisfreie Stadt",
                        "12054",
                        all(),
                    ),
                    unit(
                        AdminLevel::Verwaltungsgemeinschaft,
                        "Potsdam",
                        "Amtsfreie Gemeinde",
                        "120540000",
                        all(),
                    ),
                    unit(AdminLevel::Gemeinde, "Potsdam", "Stadt", "12054000", all()),
                ],
                vec![],
                vec![],
            ),
        );
        assert_eq!(
            levels(&found),
            [
                (AdminLevel::Land, "Brandenburg".to_string()),
                (AdminLevel::Kreis, "Potsdam".to_string()),
                (AdminLevel::Gemeinde, "Potsdam".to_string()),
            ]
        );
        // A kind with "frei" alone is enough. The name of a Gemeinde on the map is not: the
        // entry of kind "Gemeinde" stays (fix round 1; see an_amt_named_after_its_gemeinde_stays).
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    unit(
                        AdminLevel::Verwaltungsgemeinschaft,
                        "Irgendwo",
                        "Verbandsfreie Gemeinde",
                        "1",
                        all(),
                    ),
                    unit(
                        AdminLevel::Verwaltungsgemeinschaft,
                        "Buckow",
                        "Gemeinde",
                        "2",
                        all(),
                    ),
                    unit(AdminLevel::Gemeinde, "Buckow", "Stadt", "3", all()),
                ],
                vec![],
                vec![],
            ),
        );
        assert_eq!(
            levels(&found),
            [
                (AdminLevel::Verwaltungsgemeinschaft, "Buckow".to_string()),
                (AdminLevel::Gemeinde, "Buckow".to_string())
            ]
        );
        // A real Amt stays.
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    unit(
                        AdminLevel::Verwaltungsgemeinschaft,
                        "Märkische Schweiz",
                        "Amt",
                        "12064",
                        all(),
                    ),
                    unit(AdminLevel::Gemeinde, "Buckow", "Stadt", "3", all()),
                ],
                vec![],
                vec![],
            ),
        );
        assert_eq!(found.admin.len(), 2);
    }

    /// An Amt can carry the name of one of its Gemeinden (live VG250 near Brück: the Amt Brück
    /// holds the Stadt Brück). It is a real Verwaltungsgemeinschaft and stays, and the order
    /// office is that of the Amt.
    #[test]
    fn an_amt_named_after_its_gemeinde_stays() {
        let all = || square(12.9, 51.9, 13.2, 52.2);
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![
                    brandenburg(),
                    unit(
                        AdminLevel::Verwaltungsgemeinschaft,
                        "Brück",
                        "Amt",
                        "120695",
                        all(),
                    ),
                    unit(AdminLevel::Gemeinde, "Brück", "Stadt", "12069052", all()),
                ],
                vec![],
                vec![],
            ),
        );
        let levels: Vec<(AdminLevel, &str, &str)> = found
            .admin
            .iter()
            .map(|s| (s.item.level, s.item.kind.as_str(), s.item.name.as_str()))
            .collect();
        assert_eq!(
            levels,
            [
                (AdminLevel::Land, "Land", "Brandenburg"),
                (AdminLevel::Verwaltungsgemeinschaft, "Amt", "Brück"),
                (AdminLevel::Gemeinde, "Stadt", "Brück"),
            ]
        );
        assert_eq!(
            hint(&found, hint_key::ORDER_OFFICE)[0].args,
            args(&[("amt", "Amt Brück"), ("gemeinde", "Brück")])
        );
    }

    #[test]
    fn unreachable_sources_are_recorded() {
        let found = notes(
            MapKind::Schoolyard,
            &Inputs {
                admin: SourceInput::Unreachable {
                    reason: "HTTP 503".into(),
                },
                protected: SourceInput::NotRead,
                osm: read(vec![]),
                regional: Vec::new(),
            },
        );
        assert!(found.admin.is_empty() && found.protected.is_empty());
        let keys: Vec<&str> = found.hints.iter().map(|h| h.key.as_str()).collect();
        assert_eq!(keys, [hint_key::OWNERS_UNKNOWN]);
        assert_eq!(
            found.sources,
            [
                SourceStatus {
                    id: "vg250".into(),
                    state: "unreachable".into(),
                    fetched: None,
                    reason: Some("HTTP 503".into()),
                    credit: None,
                },
                SourceStatus {
                    id: "bfn".into(),
                    state: "not-read".into(),
                    fetched: None,
                    reason: None,
                    credit: None,
                },
                SourceStatus {
                    id: "osm".into(),
                    state: "read".into(),
                    fetched: Some(FETCHED),
                    reason: None,
                    credit: Some("© OpenStreetMap contributors, ODbL".into()),
                },
            ]
        );

        let read_all = notes(MapKind::Sprint, &inputs(vec![], vec![], vec![]));
        let credits: Vec<&str> = read_all
            .sources
            .iter()
            .map(|s| s.credit.as_deref().unwrap())
            .collect();
        assert!(
            credits[0].starts_with("© BKG (2026) dl-de/by-2-0"),
            "{}",
            credits[0]
        );
        assert!(credits[1].starts_with("© Bundesamt für Naturschutz (BfN) (2026)"));
        assert!(credits[1].ends_with("Nicht für Planungszwecke geeignet."));
    }

    #[test]
    fn operators_are_grouped_by_name() {
        let on_map = || square(13.01, 52.01, 13.02, 52.02);
        let with = |id: &str, object: &str, operator: Option<&str>, owner: Option<&str>| {
            let mut f = finding(OsmFindingKind::Operated, id, object, on_map());
            f.operator = operator.map(str::to_string);
            f.owner = owner.map(str::to_string);
            f
        };
        let osm = vec![
            with("way/1", "leisure=park", Some("Grünflächenamt"), None),
            with(
                "way/2",
                "leisure=pitch",
                Some("Grünflächenamt"),
                Some("Stadt X"),
            ),
            with("way/3", "leisure=park", Some("Grünflächenamt"), None),
            with("way/4", "amenity=school", None, Some("Stadt X")),
            with(
                "way/5",
                "landuse=cemetery",
                Some("Kirchengemeinde"),
                Some("Kirchengemeinde"),
            ),
            // Not on the map.
            {
                let mut f = with("way/6", "leisure=park", Some("Fernpark gGmbH"), None);
                f.geometry = square(13.5, 52.0, 13.6, 52.1);
                f
            },
        ];
        let sprint = notes(MapKind::Sprint, &inputs(vec![], vec![], osm.clone()));
        assert_eq!(
            sprint.operators,
            [
                OperatorEntry {
                    name: "Grünflächenamt".into(),
                    roles: vec![OperatorRole::Operator],
                    objects: vec!["leisure=park".into(), "leisure=pitch".into()],
                },
                OperatorEntry {
                    name: "Kirchengemeinde".into(),
                    roles: vec![OperatorRole::Operator, OperatorRole::Owner],
                    objects: vec!["landuse=cemetery".into()],
                },
                OperatorEntry {
                    name: "Stadt X".into(),
                    roles: vec![OperatorRole::Owner],
                    objects: vec!["amenity=school".into(), "leisure=pitch".into()],
                },
            ]
        );
        let park = hint(&sprint, hint_key::PARK_OPERATOR);
        assert_eq!(park.len(), 1);
        assert_eq!(park[0].args, args(&[("operator", "Grünflächenamt")]));
        assert!(hint(&sprint, hint_key::SCHOOL_OPERATOR).is_empty());

        let schoolyard = notes(MapKind::Schoolyard, &inputs(vec![], vec![], osm));
        let school = hint(&schoolyard, hint_key::SCHOOL_OPERATOR);
        assert_eq!(school.len(), 1);
        assert_eq!(school[0].args, args(&[("operator", "Stadt X")]));
        // Protected OSM areas are not operators.
        assert!(sprint.osm_protected.is_empty());
    }

    #[test]
    fn degenerate_geometries_do_not_break_the_notes() {
        let c = |x, y| Coord { x, y };
        let broken = MultiPolygon::new(vec![
            // Two corners only.
            Polygon::new(LineString::new(vec![c(13.0, 52.0), c(13.1, 52.1)]), vec![]),
            // Not a number.
            Polygon::new(
                LineString::new(vec![
                    c(13.0, 52.0),
                    c(f64::NAN, 52.0),
                    c(13.1, 52.1),
                    c(13.0, 52.0),
                ]),
                vec![],
            ),
            // All corners in one place. (A line in degrees is not one in UTM.)
            Polygon::new(
                LineString::new(vec![
                    c(13.05, 52.05),
                    c(13.05, 52.05),
                    c(13.05, 52.05),
                    c(13.05, 52.05),
                ]),
                vec![],
            ),
        ]);
        let found = notes(
            MapKind::Sprint,
            &inputs(
                vec![unit(
                    AdminLevel::Gemeinde,
                    "Kaputt",
                    "Gemeinde",
                    "1",
                    broken.clone(),
                )],
                vec![protected(
                    ProtectedKind::Naturschutzgebiet,
                    "Kaputt",
                    broken,
                )],
                vec![],
            ),
        );
        assert!(found.admin.is_empty() && found.protected.is_empty());
    }

    #[test]
    fn the_site_link_stays_apart_from_the_legal_link() {
        let mut ffh = protected(ProtectedKind::Ffh, "FFH", square(13.0, 52.0, 13.05, 52.1));
        ffh.site_link = Some("https://natura2000.example/DE1".into());
        let found = notes(MapKind::Sprint, &inputs(vec![], vec![ffh], vec![]));
        let entry = &found.protected[0].item;
        assert_eq!(entry.legal_link, None);
        assert_eq!(
            entry.site_link.as_deref(),
            Some("https://natura2000.example/DE1")
        );
    }

    #[test]
    fn notes_round_trip_through_json() {
        let mut nsg = protected(
            ProtectedKind::Naturschutzgebiet,
            "NSG",
            square(13.0, 52.0, 13.05, 52.1),
        );
        nsg.legal_date = NaiveDate::from_ymd_opt(1966, 8, 1);
        nsg.legal_title = Some("Verordnung".into());
        nsg.legal_link = Some("https://example.org/vo".into());
        nsg.site_link = Some("https://natura2000.example/DE1".into());
        let mut park = finding(
            OsmFindingKind::Operated,
            "way/9",
            "leisure=park",
            square(13.01, 52.01, 13.02, 52.02),
        );
        park.operator = Some("Grünflächenamt".into());
        let found = notes(
            MapKind::Schoolyard,
            &Inputs {
                admin: read(vec![
                    brandenburg(),
                    unit(
                        AdminLevel::Gemeinde,
                        "Buckow",
                        "Stadt",
                        "12064061",
                        square(12.9, 51.9, 13.2, 52.2),
                    ),
                ]),
                protected: read(vec![nsg]),
                osm: SourceInput::Unreachable {
                    reason: "no extract".into(),
                },
                regional: Vec::new(),
            },
        );
        assert_eq!(found.schema, SCHEMA);
        assert_eq!(found.created, FETCHED);
        let json = serde_json::to_string_pretty(&found).unwrap();
        let back: PermitNotes = serde_json::from_str(&json).unwrap();
        assert_eq!(back, found);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["schema"], 1);
        assert_eq!(value["map_kind"], "schoolyard");
        assert_eq!(value["protected"][0]["item"]["kind"], "naturschutzgebiet");
        assert_eq!(value["protected"][0]["item"]["legal_date"], "1966-08-01");
        assert_eq!(value["admin"][0]["item"]["level"], "land");
        assert_eq!(value["sources"][2]["state"], "unreachable");
        // Nothing of a geometry is written.
        assert!(!json.contains("coordinates") && !json.contains("geometry"));
    }

    // The regional part (ADR 0120).

    use crate::permits::regional::{RegionalSource, Words};
    use freemapper_adapter_api::permits::{
        FindingKind, PermitFinding, PermitSection, ProtectedObjectKind,
    };

    fn regional_source(key: &str, section: PermitSection) -> RegionalSource {
        RegionalSource {
            key: key.into(),
            label: Words {
                en: "Portal".into(),
                de: "Portal".into(),
            },
            what: Words {
                en: key.into(),
                de: key.into(),
            },
            credit: "dl-de/zero-2-0".into(),
            note: None,
            sections: vec![section],
        }
    }

    fn regional(key: &str, section: PermitSection, findings: Vec<PermitFinding>) -> RegionalInput {
        RegionalInput {
            adapter: "city".into(),
            source: regional_source(key, section),
            findings: read(findings),
        }
    }

    fn area(kind: FindingKind, g: MultiPolygon<f64>) -> PermitFinding {
        PermitFinding {
            kind,
            geometry: geo_types::Geometry::MultiPolygon(g),
        }
    }

    fn point(kind: FindingKind, lon: f64, lat: f64) -> PermitFinding {
        PermitFinding {
            kind,
            geometry: geo_types::Geometry::MultiPoint(geo_types::MultiPoint(vec![
                geo_types::Point::new(lon, lat),
            ])),
        }
    }

    fn district(name: &str, g: MultiPolygon<f64>) -> PermitFinding {
        area(FindingKind::District { name: name.into() }, g)
    }

    /// A city state over the whole map: Land, Kreis and Gemeinde with the AGS of Berlin.
    fn city_state() -> Vec<AdminUnit> {
        let all = || square(12.9, 51.9, 13.2, 52.2);
        vec![
            unit(AdminLevel::Land, "Berlin", "Land", "11", all()),
            unit(
                AdminLevel::Kreis,
                "Berlin",
                "Kreisfreie Stadt",
                "11000",
                all(),
            ),
            unit(AdminLevel::Gemeinde, "Berlin", "Stadt", "11000000", all()),
        ]
    }

    fn with_regional(admin: Vec<AdminUnit>, regional: Vec<RegionalInput>) -> Inputs {
        Inputs {
            regional,
            ..inputs(admin, vec![], vec![])
        }
    }

    fn keys(notes: &PermitNotes) -> Vec<&str> {
        notes.hints.iter().map(|h| h.key.as_str()).collect()
    }

    fn near(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1.0
    }

    #[test]
    fn bezirk_shares_and_the_bezirksamt_hints() {
        let monument = FindingKind::Monument {
            kind: "Baudenkmal".into(),
            link: None,
        };
        let tree = FindingKind::ProtectedObject {
            kind: ProtectedObjectKind::NaturalMonument,
            name: Some("Platane".into()),
            citation: None,
        };
        let sources = |districts: SourceInput<PermitFinding>| {
            vec![
                RegionalInput {
                    findings: districts,
                    ..regional("bezirke", PermitSection::Districts, vec![])
                },
                regional(
                    "denkmale",
                    PermitSection::Monuments,
                    vec![area(monument.clone(), square(13.01, 52.01, 13.02, 52.02))],
                ),
                regional(
                    "schutz",
                    PermitSection::ProtectedObjects,
                    vec![point(tree.clone(), 13.08, 52.05)],
                ),
            ]
        };
        let districts = read(vec![
            district("West", square(12.9, 51.9, 13.07, 52.2)),
            district("Ost", square(13.07, 51.9, 13.2, 52.2)),
            district("Fern", square(13.5, 51.9, 13.6, 52.2)),
        ]);
        let found = notes(
            MapKind::Sprint,
            &with_regional(city_state(), sources(districts)),
        );
        let part = &found.regional[0];
        assert_eq!(part.adapter, "city");
        let shares: Vec<(&str, f64)> = part
            .districts
            .as_ref()
            .unwrap()
            .iter()
            .map(|s| (s.item.name.as_str(), s.percent))
            .collect();
        assert_eq!(shares.len(), 2, "{shares:?}");
        assert!(
            shares[0].0 == "West" && near(shares[0].1, 70.0),
            "{shares:?}"
        );
        assert!(
            shares[1].0 == "Ost" && near(shares[1].1, 30.0),
            "{shares:?}"
        );
        assert_eq!(
            keys(&found),
            [
                hint_key::DISTRICT_OFFICE,
                hint_key::DISTRICT_OFFICE,
                hint_key::MONUMENT_AUTHORITY,
                hint_key::NATURE_OBJECT_AUTHORITY,
                hint_key::OWNERS_UNKNOWN,
            ],
            "the Bezirke take the place of the city-state and Gemeinde hints"
        );
        let district_args: Vec<&str> = found
            .hints
            .iter()
            .filter_map(|h| h.args.get("district").map(String::as_str))
            .collect();
        assert_eq!(district_args, ["West", "Ost", "West", "Ost"]);

        // Without the Bezirke the hints of step 1 come back, and the section drops.
        let down = SourceInput::Unreachable {
            reason: "HTTP 503".into(),
        };
        let found = notes(MapKind::Sprint, &with_regional(city_state(), sources(down)));
        assert_eq!(found.regional[0].districts, None);
        assert_eq!(
            keys(&found),
            [
                hint_key::CITY_STATE,
                hint_key::ORDER_OFFICE,
                hint_key::OWNERS_UNKNOWN,
            ]
        );
        assert!(found.regional[0].monuments.is_some());
        assert_eq!(found.regional[0].sources[0].status.state, "unreachable");
        assert_eq!(found.regional[0].sources[1].status.state, "read");
        assert_eq!(
            found.regional[0].sources[1].status.credit.as_deref(),
            Some("dl-de/zero-2-0")
        );

        // Hamburg, or any city state without a regional source, keeps the general hint.
        let found = notes(MapKind::Sprint, &with_regional(city_state(), vec![]));
        assert!(found.regional.is_empty());
        assert_eq!(keys(&found)[0], hint_key::CITY_STATE);
    }

    /// The parts of a park are one entry with the sum of their shares (review finding 1).
    #[test]
    fn the_parts_of_a_park_are_summed() {
        let park = || FindingKind::GreenSpace {
            name: "Viktoriapark".into(),
            district: Some("West".into()),
            dedication: Some("gewidmet".into()),
        };
        let parts = vec![
            // 10 % and 9 % of the map.
            area(park(), square(13.0, 52.0, 13.01, 52.1)),
            area(park(), square(13.01, 52.0, 13.019, 52.1)),
        ];
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                vec![],
                vec![regional("gruen", PermitSection::GreenSpaces, parts)],
            ),
        );
        let entries = &found.regional[0].green_spaces.as_ref().unwrap().entries;
        let shares: Vec<(&str, f64)> = entries
            .iter()
            .map(|s| (s.item.name.as_str(), s.percent))
            .collect();
        assert_eq!(shares.len(), 1, "{shares:?}");
        assert!(
            shares[0].0 == "Viktoriapark" && (shares[0].1 - 19.0).abs() < 0.3,
            "{shares:?}"
        );
    }

    fn forest_unit(office: &str, unit: &str, g: MultiPolygon<f64>) -> PermitFinding {
        area(
            FindingKind::ForestUnit {
                administration: "Landesforst".into(),
                office: office.into(),
                unit: Some(unit.into()),
            },
            g,
        )
    }

    fn state_forest(office: &str, g: MultiPolygon<f64>) -> PermitFinding {
        area(
            FindingKind::StateForest {
                owner: "Bundesland Nordrhein-Westfalen".into(),
                office: Some(office.into()),
            },
            g,
        )
    }

    /// Forest units are summed per office and unit, by share; a forest map gets the hint of
    /// each office that is more than an edge, with its units, and the hint on the owner's
    /// consent (ADR 0124).
    #[test]
    fn forest_units_are_summed_and_name_their_office_on_a_forest_map() {
        let units = || {
            regional(
                "reviere",
                PermitSection::ForestUnits,
                vec![
                    // 30 % and 20 % of the map in one Revier, 49.8 % in another of the same
                    // office, 0.2 % of another office at the east edge.
                    forest_unit(
                        "Oberförsterei A",
                        "Revier Nord",
                        square(12.9, 51.9, 13.03, 52.2),
                    ),
                    forest_unit(
                        "Oberförsterei A",
                        "Revier Nord",
                        square(13.03, 51.9, 13.05, 52.2),
                    ),
                    forest_unit(
                        "Oberförsterei A",
                        "Revier Süd",
                        square(13.05, 51.9, 13.0998, 52.2),
                    ),
                    forest_unit(
                        "Oberförsterei B",
                        "Revier Rand",
                        square(13.0998, 51.9, 13.2, 52.2),
                    ),
                ],
            )
        };
        let found = notes(MapKind::Forest, &with_regional(vec![], vec![units()]));
        let shares: Vec<(&str, Option<&str>, f64, bool)> = found.regional[0]
            .forest_units
            .as_ref()
            .unwrap()
            .iter()
            .map(|s| {
                (
                    s.item.office.as_str(),
                    s.item.unit.as_deref(),
                    s.percent,
                    s.at_edge,
                )
            })
            .collect();
        assert_eq!(shares.len(), 3, "{shares:?}");
        assert!(
            shares[0].0 == "Oberförsterei A"
                && shares[0].1 == Some("Revier Nord")
                && near(shares[0].2, 50.0),
            "{shares:?}"
        );
        assert!(
            shares[1].1 == Some("Revier Süd") && near(shares[1].2, 49.8),
            "{shares:?}"
        );
        assert!(
            shares[2].0 == "Oberförsterei B" && shares[2].3,
            "{shares:?}"
        );
        assert_eq!(
            found.regional[0].forest_units.as_ref().unwrap()[0]
                .item
                .administration,
            "Landesforst"
        );

        let offices: Vec<&Hint> = found
            .hints
            .iter()
            .filter(|h| h.key == hint_key::FOREST_OFFICE)
            .collect();
        assert_eq!(offices.len(), 1, "{:?}", found.hints);
        assert_eq!(offices[0].args["office"], "Oberförsterei A");
        assert_eq!(offices[0].args["administration"], "Landesforst");
        assert_eq!(offices[0].args["units"], "Revier Nord, Revier Süd");
        let owner = found
            .hints
            .iter()
            .find(|h| h.key == hint_key::FOREST_OWNER)
            .expect("the owner's consent");
        assert_eq!(owner.args.get("office").map(String::as_str), Some("known"));

        // A sprint map keeps the section and has no forest hint: a Revier also covers towns.
        let found = notes(MapKind::Sprint, &with_regional(vec![], vec![units()]));
        assert!(found.regional[0].forest_units.is_some());
        assert!(
            !keys(&found)
                .iter()
                .any(|k| [hint_key::FOREST_OFFICE, hint_key::FOREST_OWNER].contains(k)),
            "{:?}",
            found.hints
        );
    }

    /// A forest map without forest units, in a state without such a source or with the source
    /// down, gets the general hint on the owner's consent that says the office is not named.
    #[test]
    fn a_forest_map_without_forest_units_gets_the_general_forest_hint() {
        for regional_inputs in [
            vec![],
            vec![RegionalInput {
                findings: SourceInput::Unreachable {
                    reason: "HTTP 503".into(),
                },
                ..regional("reviere", PermitSection::ForestUnits, vec![])
            }],
        ] {
            let found = notes(MapKind::Forest, &with_regional(vec![], regional_inputs));
            assert!(!keys(&found).contains(&hint_key::FOREST_OFFICE));
            let owner = found
                .hints
                .iter()
                .find(|h| h.key == hint_key::FOREST_OWNER)
                .expect("the owner's consent");
            assert_eq!(
                owner.args.get("office").map(String::as_str),
                Some("unknown")
            );
        }
        let found = notes(MapKind::Schoolyard, &with_regional(vec![], vec![]));
        assert!(!keys(&found).contains(&hint_key::FOREST_OWNER));
    }

    /// State forest is summed per owner and office and named in a hint on every map kind; the
    /// owners hint then speaks of the rest of the map.
    #[test]
    fn state_forest_is_summed_and_named() {
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                vec![],
                vec![regional(
                    "staatswald",
                    PermitSection::StateForest,
                    vec![
                        // 10 % and 9 %.
                        state_forest("Regionalforstamt X", square(13.0, 52.0, 13.01, 52.1)),
                        state_forest("Regionalforstamt X", square(13.01, 52.0, 13.019, 52.1)),
                    ],
                )],
            ),
        );
        let entries = found.regional[0].state_forest.as_ref().unwrap();
        assert_eq!(entries.len(), 1, "{entries:?}");
        assert_eq!(entries[0].item.owner, "Bundesland Nordrhein-Westfalen");
        assert_eq!(
            entries[0].item.office.as_deref(),
            Some("Regionalforstamt X")
        );
        assert!((entries[0].percent - 19.0).abs() < 0.3, "{entries:?}");
        let hint = found
            .hints
            .iter()
            .find(|h| h.key == hint_key::STATE_FOREST)
            .expect("the state forest hint");
        assert_eq!(hint.args["owner"], "Bundesland Nordrhein-Westfalen");
        assert_eq!(hint.args["office"], "Regionalforstamt X");
        let unknown = found
            .hints
            .iter()
            .find(|h| h.key == hint_key::OWNERS_UNKNOWN)
            .unwrap();
        assert_eq!(unknown.args.get("rest").map(String::as_str), Some("yes"));
    }

    /// A Bezirk that comes as two features is one entry with the sum of both (review finding 9).
    #[test]
    fn a_bezirk_in_two_features_is_summed() {
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                city_state(),
                vec![regional(
                    "bezirke",
                    PermitSection::Districts,
                    vec![
                        // 30 % and 20 % of the map.
                        district("West", square(12.9, 51.9, 13.03, 52.2)),
                        district("West", square(13.03, 51.9, 13.05, 52.2)),
                        district("Ost", square(13.05, 51.9, 13.2, 52.2)),
                    ],
                )],
            ),
        );
        let shares: Vec<(&str, f64)> = found.regional[0]
            .districts
            .as_ref()
            .unwrap()
            .iter()
            .map(|s| (s.item.name.as_str(), s.percent))
            .collect();
        assert_eq!(shares.len(), 2, "{shares:?}");
        assert!(
            shares[0].0 == "West" && (shares[0].1 - 50.0).abs() < 0.5,
            "{shares:?}"
        );
        assert!(
            shares[1].0 == "Ost" && (shares[1].1 - 50.0).abs() < 0.5,
            "{shares:?}"
        );
    }

    /// The authority hints name the Bezirke that are more than an edge, as the Bezirksamt hint
    /// does; an object that lies only in edge Bezirke names those (review finding 6).
    #[test]
    fn authority_hints_skip_edge_bezirke_unless_the_object_lies_only_there() {
        let monument = FindingKind::Monument {
            kind: "Baudenkmal".into(),
            link: None,
        };
        let pond = FindingKind::ProtectedObject {
            kind: ProtectedObjectKind::ProtectedLandscapePart,
            name: Some("Pfuhl".into()),
            citation: None,
        };
        // West 70 %, Ost about 30 %, Rand 0.2 % of the map at its east side.
        let bezirke = || {
            regional(
                "bezirke",
                PermitSection::Districts,
                vec![
                    district("West", square(12.9, 51.9, 13.07, 52.2)),
                    district("Ost", square(13.07, 51.9, 13.0998, 52.2)),
                    district("Rand", square(13.0998, 51.9, 13.2, 52.2)),
                ],
            )
        };
        let hints = |objects: Vec<PermitFinding>, objects_too: Vec<PermitFinding>| {
            let found = notes(
                MapKind::Sprint,
                &with_regional(
                    city_state(),
                    vec![
                        bezirke(),
                        regional("denkmale", PermitSection::Monuments, objects),
                        regional("schutz", PermitSection::ProtectedObjects, objects_too),
                    ],
                ),
            );
            let of = |key: &str| -> Vec<String> {
                hint(&found, key)
                    .iter()
                    .map(|h| h.args["district"].clone())
                    .collect()
            };
            (
                of(hint_key::DISTRICT_OFFICE),
                of(hint_key::MONUMENT_AUTHORITY),
                of(hint_key::NATURE_OBJECT_AUTHORITY),
            )
        };
        // Both objects cross from Ost into Rand: only Ost is named.
        let (offices, monuments, nature) = hints(
            vec![area(monument.clone(), square(13.09, 52.05, 13.1, 52.06))],
            vec![area(pond.clone(), square(13.09, 52.07, 13.1, 52.08))],
        );
        assert_eq!(
            offices,
            ["West", "Ost"],
            "an edge Bezirk has no office hint"
        );
        assert_eq!(monuments, ["Ost"]);
        assert_eq!(nature, ["Ost"]);
        // Both objects lie only in Rand: Rand is named, the edge Bezirk it is.
        let (offices, monuments, nature) = hints(
            vec![area(monument, square(13.0999, 52.05, 13.1, 52.06))],
            vec![area(pond, square(13.0999, 52.07, 13.1, 52.08))],
        );
        assert_eq!(offices, ["West", "Ost"]);
        assert_eq!(monuments, ["Rand"]);
        assert_eq!(nature, ["Rand"]);
    }

    #[test]
    fn owner_shares_are_one_line_per_owner() {
        let owner = |name: &str| FindingKind::OwnerShare {
            owner: Some(name.into()),
        };
        let parcels = vec![
            area(owner("Land Berlin"), square(13.0, 52.0, 13.02, 52.1)),
            area(owner("Land Berlin"), square(13.02, 52.0, 13.05, 52.1)),
            area(
                owner("Bundesrepublik Deutschland"),
                square(13.09, 52.0, 13.1, 52.1),
            ),
            // Off the map.
            area(owner("Land Berlin"), square(13.5, 52.0, 13.6, 52.1)),
        ];
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                vec![],
                vec![regional("liegenschaften", PermitSection::Owners, parcels)],
            ),
        );
        let owners: Vec<(&str, f64)> = found.regional[0]
            .owners
            .as_ref()
            .unwrap()
            .iter()
            .map(|s| (s.item.owner.as_deref().unwrap(), s.percent))
            .collect();
        assert_eq!(owners.len(), 2, "{owners:?}");
        assert!(
            owners[0].0 == "Land Berlin" && near(owners[0].1, 50.0),
            "{owners:?}"
        );
        assert!(
            owners[1].0 == "Bundesrepublik Deutschland" && near(owners[1].1, 10.0),
            "{owners:?}"
        );
        let shares = hint(&found, hint_key::OWNER_SHARE);
        assert_eq!(shares.len(), 2);
        assert_eq!(shares[0].args["owner"], "Land Berlin");
        assert_eq!(shares[0].args["percent"], format!("{:.1}", owners[0].1));
        assert_eq!(
            hint(&found, hint_key::OWNERS_UNKNOWN)[0].args,
            args(&[("rest", "yes")])
        );
        // The notes name no parcel.
        let json = serde_json::to_string(&found).unwrap();
        assert!(!json.contains("coordinates"), "{json}");
    }

    /// The owners a source does not name are summed into one entry without a name, and their
    /// hint carries no name either (review finding 8).
    #[test]
    fn unnamed_owners_are_one_entry_without_a_name() {
        let parcels = vec![
            area(
                FindingKind::OwnerShare {
                    owner: Some("Land Berlin".into()),
                },
                square(13.0, 52.0, 13.05, 52.1),
            ),
            area(
                FindingKind::OwnerShare { owner: None },
                square(13.05, 52.0, 13.07, 52.1),
            ),
            area(
                FindingKind::OwnerShare { owner: None },
                square(13.07, 52.0, 13.08, 52.1),
            ),
        ];
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                vec![],
                vec![regional("liegenschaften", PermitSection::Owners, parcels)],
            ),
        );
        let owners: Vec<(Option<&str>, f64)> = found.regional[0]
            .owners
            .as_ref()
            .unwrap()
            .iter()
            .map(|s| (s.item.owner.as_deref(), s.percent))
            .collect();
        assert_eq!(owners.len(), 2, "{owners:?}");
        assert!(
            owners[0].0 == Some("Land Berlin") && near(owners[0].1, 50.0),
            "{owners:?}"
        );
        assert!(
            owners[1].0.is_none() && near(owners[1].1, 30.0),
            "{owners:?}"
        );
        let shares = hint(&found, hint_key::OWNER_SHARE);
        assert_eq!(shares.len(), 2);
        assert_eq!(shares[0].args["owner"], "Land Berlin");
        assert_eq!(
            shares[1].args,
            args(&[
                ("unnamed", "yes"),
                ("percent", &format!("{:.1}", owners[1].1))
            ])
        );
    }

    #[test]
    fn monuments_are_counted_and_links_capped_at_20() {
        let monument = |kind: &str, link: Option<String>| FindingKind::Monument {
            kind: kind.into(),
            link,
        };
        let on_map = || square(13.01, 52.01, 13.02, 52.02);
        let mut found_list: Vec<PermitFinding> = (0..25)
            .map(|i| {
                area(
                    monument("Baudenkmal", Some(format!("https://list.example/{i:02}"))),
                    on_map(),
                )
            })
            .collect();
        found_list.push(area(
            monument("Gartendenkmal", Some("https://list.example/garden".into())),
            on_map(),
        ));
        found_list.push(area(monument("Ensemble", None), on_map()));
        found_list.push(point(monument("Ensemble", None), 13.05, 52.05));
        // Off the map.
        found_list.push(area(
            monument("Baudenkmal", Some("https://list.example/far".into())),
            square(13.5, 52.0, 13.6, 52.1),
        ));
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                vec![],
                vec![regional("denkmale", PermitSection::Monuments, found_list)],
            ),
        );
        let monuments = found.regional[0].monuments.as_ref().unwrap();
        assert_eq!(monuments.total, 28);
        let groups: Vec<(&str, usize, usize)> = monuments
            .groups
            .iter()
            .map(|g| (g.kind.as_str(), g.count, g.links.len()))
            .collect();
        assert_eq!(
            groups,
            [
                ("Baudenkmal", 25, 19),
                ("Ensemble", 2, 0),
                ("Gartendenkmal", 1, 1)
            ]
        );
        let links: usize = monuments.groups.iter().map(|g| g.links.len()).sum();
        assert_eq!(links, MAX_MONUMENT_LINKS);
        assert_eq!(monuments.links_left, 6);
        assert!(
            !monuments.groups[0]
                .links
                .contains(&"https://list.example/far".to_string())
        );
        // No district is known, so the hint of the monument authority names none.
        assert!(hint(&found, hint_key::MONUMENT_AUTHORITY).is_empty());
    }

    #[test]
    fn a_naturdenkmal_point_inside_counts_and_outside_not() {
        let tree = |name: &str| FindingKind::ProtectedObject {
            kind: ProtectedObjectKind::NaturalMonument,
            name: Some(name.into()),
            citation: Some("VO vom 20.05.2021".into()),
        };
        let objects = vec![
            point(tree("Platane"), 13.05, 52.05),
            point(tree("Platane"), 13.06, 52.05),
            point(tree("Eiche"), 13.5, 52.05),
            area(
                FindingKind::ProtectedObject {
                    kind: ProtectedObjectKind::ProtectedLandscapePart,
                    name: Some("Pfuhl".into()),
                    citation: None,
                },
                square(13.09, 52.0, 13.11, 52.1),
            ),
        ];
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                vec![],
                vec![
                    regional(
                        "bezirke",
                        PermitSection::Districts,
                        vec![district("Mitte", square(12.9, 51.9, 13.2, 52.2))],
                    ),
                    regional("schutz", PermitSection::ProtectedObjects, objects),
                ],
            ),
        );
        let objects = found.regional[0].protected_objects.as_ref().unwrap();
        assert_eq!(objects.len(), 2, "{objects:?}");
        assert_eq!(objects[0].kind, ProtectedObjectKind::NaturalMonument);
        assert_eq!(objects[0].name.as_deref(), Some("Platane"));
        assert_eq!((objects[0].count, objects[0].percent), (2, None));
        assert_eq!(objects[1].kind, ProtectedObjectKind::ProtectedLandscapePart);
        assert!(near(objects[1].percent.unwrap(), 10.0), "{objects:?}");
        assert_eq!(
            hint(&found, hint_key::NATURE_OBJECT_AUTHORITY)[0].args,
            args(&[("district", "Mitte")])
        );

        // Only a point outside: nothing on the map, and no hint.
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                vec![],
                vec![
                    regional(
                        "bezirke",
                        PermitSection::Districts,
                        vec![district("Mitte", square(12.9, 51.9, 13.2, 52.2))],
                    ),
                    regional(
                        "schutz",
                        PermitSection::ProtectedObjects,
                        vec![point(tree("Eiche"), 13.5, 52.05)],
                    ),
                ],
            ),
        );
        assert_eq!(found.regional[0].protected_objects, Some(vec![]));
        assert!(hint(&found, hint_key::NATURE_OBJECT_AUTHORITY).is_empty());
    }

    /// `permits.json` as step 1 wrote it, without the regional part, still reads; notes without
    /// a regional part are written as before.
    #[test]
    fn an_old_permits_json_still_reads() {
        let old = r#"{
            "schema": 1,
            "created": "2026-11-13",
            "map_kind": "sprint",
            "area_m2": 1032000.0,
            "admin": [{"item": {"level": "land", "name": "Brandenburg", "kind": "Land", "ags": "12"}, "percent": 100.0, "at_edge": false}],
            "protected": [],
            "osm_protected": [],
            "operators": [],
            "hints": [{"key": "permits-hint-owners-unknown", "args": {}}],
            "sources": [{"id": "vg250", "state": "read", "fetched": "2026-11-13", "reason": null, "credit": "© BKG"}]
        }"#;
        let notes: PermitNotes = serde_json::from_str(old).unwrap();
        assert!(notes.regional.is_empty());
        assert_eq!(notes.schema, SCHEMA);
        let written = serde_json::to_value(&notes).unwrap();
        assert!(written.get("regional").is_none());
        assert_eq!(
            written,
            serde_json::from_str::<serde_json::Value>(old).unwrap()
        );
    }

    #[test]
    fn regional_notes_round_trip_through_json() {
        let found = notes(
            MapKind::Sprint,
            &with_regional(
                city_state(),
                vec![regional(
                    "bezirke",
                    PermitSection::Districts,
                    vec![district("Mitte", square(12.9, 51.9, 13.2, 52.2))],
                )],
            ),
        );
        let json = serde_json::to_string_pretty(&found).unwrap();
        let back: PermitNotes = serde_json::from_str(&json).unwrap();
        assert_eq!(back, found);
        assert_eq!(found.schema, 1);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            value["regional"][0]["districts"][0]["item"]["name"],
            "Mitte"
        );
        // A section no source fills is not written.
        assert!(value["regional"][0].get("monuments").is_none());
        assert!(!json.contains("coordinates"));
    }
}
