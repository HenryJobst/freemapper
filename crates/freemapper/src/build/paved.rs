//! ALKIS traffic areas → 501.5 paved areas of the sprint map (spec §4.5).
//!
//! In Berlin the Straßenbefahrung (street survey 2014–2016, Berlin plus 2) refines them: its
//! green strips (Grünstreifen) are cut out of the 501.5 map objects, and its paved surfaces
//! outside the ALKIS traffic parcels become 501.5 candidates. The survey never replaces ALKIS:
//! a quarter of the ALKIS street area was not surveyed or was rebuilt since (reference §6).
//! Its carriageways inside the street parcels become hidden 501.11 candidates
//! ([`HIDDEN_CARRIAGEWAY`]): only the training map draws them (`build/training/carriageway.rs`).

use chrono::NaiveDate;
use freemapper_adapter_api::extras::{LayerKinds, SetDrawing};
use geo::orient::{Direction, Orient as _};
use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Distance as _, Euclidean, Intersects as _,
};
use geo_types::{LineString, MultiPolygon, Polygon, Rect};
use serde_json::Value;

use crate::build::out_of_bounds::{LandClip, area_object, clipped_pieces, land_tags};
use crate::build::points::HIDDEN_KEY;
use crate::build::sets::dated_tags;
use crate::build::tagged::polygons_in;
use crate::crs::{BoundingBox, Crs};
use crate::georef::{GeorefError, Georeferencing};
use crate::measure::{SymbolMm, SymbolScale};
use crate::omap::{MapObject, SymbolSet};
use crate::sources::alkis::LandUse;
use crate::sources::osm::{OsmFeature, OsmGeometry};

/// ISSprOM 501.5 paved area, light traffic, without border: a parcel boundary is no visible edge,
/// and ALKIS does not tell the traffic volume.
const PAVED_AREA_CODE: &str = "501.5";
/// Green strips and tree rows along the carriageway; the orthophoto shows them.
const VERGE: &str = "Begleitfläche Straßenverkehr";
/// `Platz` refinements that are paved; others (e.g. `Festplatz`, often grass) are candidates.
const PAVED_SQUARES: [&str; 3] = ["Parkplatz", "Marktplatz", "Fußgängerzone"];
/// ISSprOM 2019-2, 501.11 paved area, heavy traffic, without border ("Differences in the brown
/// colour shall be used to show differences in the traffic"): the surveyed carriageway. 501.10 of
/// the bundled set is the same fill with the 501.1 border.
pub const CARRIAGEWAY_CODE: &str = "501.11";
/// The value of [`HIDDEN_KEY`] on a carriageway candidate: the map and the check print leave it
/// out, the training map draws it.
pub const HIDDEN_CARRIAGEWAY: &str = "carriageway";
/// The land uses of the street parcels a carriageway lies in (`freemapper:nutzart` is the name,
/// or the name and ` / ` and the refinement).
pub const STREET_NUTZARTEN: [&str; 2] = ["Straßenverkehr", "Platz"];
/// ISSprOM 401 open land: the candidate for a surveyed green strip.
const GREEN_CODE: &str = "401";
/// Where a green strip and a paved object overlap by less than (1 mm)², or by a strip narrower
/// than 1 mm on paper and than [`SURVEY_SLIVER_CAP_M`], nothing is cut: there the survey and the
/// parcel boundary disagree, or the green is too narrow to show. The 401 candidates are the
/// strips minus every 501.5 object and candidate that stays, so a green candidate never lies on
/// paving that was not cut.
const GREEN_MIN_SIDE_MM: SymbolMm = SymbolMm(1.0);
/// Surveyed paving outside the parcels smaller than (1 mm)² is a sliver.
const SURVEYED_PAVED_MIN_SIDE_MM: SymbolMm = SymbolMm(1.0);
/// A survey candidate, a cut green strip and a remnant of the cut are no sliver down to this
/// width on the ground, even where 1 mm on paper is more (4 m at 1:4000): a 3 m park path stays a
/// candidate, a 3.5 m sidewalk beyond a verge stays paved, and the strips where the survey and
/// the parcel boundary disagree (mostly under 1 m) go.
const SURVEY_SLIVER_CAP_M: f64 = 2.0;
/// A green strip takes part in a cut when it overlaps the cut-out by more than this.
const CUT_OVERLAP_M2: f64 = 0.01;
/// A remnant within this distance of a strip's share of the cut-out lies along that strip.
const TOUCH_M: f64 = 0.01;

/// OSM streets and unpaved surfaces: an unpaved street makes its ALKIS parcel a candidate. The
/// rule file keeps these streets in the stored OSM features (test `unpaved_streets_are_stored`).
const STREETS: [&str; 8] = [
    "residential",
    "unclassified",
    "service",
    "living_street",
    "tertiary",
    "secondary",
    "primary",
    "road",
];
const UNPAVED: [&str; 10] = [
    "unpaved",
    "dirt",
    "earth",
    "ground",
    "grass",
    "gravel",
    "fine_gravel",
    "compacted",
    "sand",
    "woodchips",
];

/// The unpaved OSM streets among `features`, in `crs`.
pub fn unpaved_streets(features: &[OsmFeature], crs: Crs) -> Vec<LineString<f64>> {
    let tag = |f: &OsmFeature, key: &str| {
        f.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    features
        .iter()
        .filter(|f| {
            tag(f, "highway").is_some_and(|h| STREETS.contains(&h.as_str()))
                && tag(f, "surface").is_some_and(|s| UNPAVED.contains(&s.as_str()))
        })
        .filter_map(|f| match &f.geometry {
            OsmGeometry::Line(points) => Some(LineString::from(
                points
                    .iter()
                    .map(|p| {
                        let q = crs.forward(*p);
                        (q.easting, q.northing)
                    })
                    .collect::<Vec<_>>(),
            )),
            OsmGeometry::Point(_) | OsmGeometry::Area(_) => None,
        })
        .collect()
}

/// Paved areas: map objects where ALKIS is reliable, candidates where it is not.
#[derive(Debug, Default)]
pub struct PavedObjects {
    /// Carriageways with sidewalks and parking lanes (`Straßenverkehr`), squares and car parks (`Platz`).
    pub map: Vec<MapObject>,
    /// `Weg`: sidewalks and paved lanes in town, but unpaved tracks in the woods.
    pub candidates: Vec<MapObject>,
}

/// One area of the Straßenbefahrung in the map's CRS, with the tags its objects carry.
#[derive(Debug, Clone)]
pub struct SurveyArea {
    /// Exterior counter-clockwise, holes clockwise (one fill rule for the Boolean operations).
    pub polygon: Polygon<f64>,
    bounds: Rect<f64>,
    /// The `gis_id`, else the WFS id.
    pub id: String,
    /// The survey date (from the `gis_id`, else the dataset date).
    pub surveyed: NaiveDate,
    /// `surveyed` is the dataset date: the `gis_id` gave none.
    pub guessed: bool,
    /// Provenance with the survey date as currency, the survey type and codes.
    pub tags: Vec<(String, String)>,
    /// The source of its layer, `<source prefix>/<layer>`.
    source: String,
    /// It is carriageway ([`SetDrawing::carriageway_tags`]).
    carriageway: bool,
}

/// The areas of the Straßenbefahrung that refine the paved areas; empty outside Berlin.
#[derive(Debug, Default)]
pub struct StreetSurvey {
    /// Green strips ([`SetDrawing::paving_cuts`]).
    pub green: Vec<SurveyArea>,
    /// Paved surfaces ([`SetDrawing::surveyed_paving`]).
    pub surfaces: Vec<SurveyArea>,
}

/// One stored layer of a set: its file name, its features and its fetch date.
pub type StoredLayer<'a> = (&'a str, &'a [Value], NaiveDate);

impl StreetSurvey {
    /// The green strips and paved surfaces the set `drawing` describes among its stored
    /// `layers` (in `from`), in `to`. Other types (tree pits) and degenerate polygons are left
    /// out.
    pub fn of_set(drawing: &SetDrawing, from: Crs, layers: &[StoredLayer<'_>], to: Crs) -> Self {
        Self {
            green: survey_areas(drawing, drawing.paving_cuts, layers, from, to),
            surfaces: survey_areas(drawing, drawing.surveyed_paving, layers, from, to),
        }
    }

    /// The street survey of Berlin's stored `green` strips and `surfaces` (EPSG:25833), in `crs`.
    #[cfg(test)]
    pub fn from_features(green: &[Value], surfaces: &[Value], crs: Crs) -> Self {
        let dataset = freemapper_adapters_de::berlin::SURVEY_DATASET_DATE;
        Self::of_set(
            crate::build::sets::drawing_of_source("geoportal-berlin/").unwrap(),
            Crs::Utm33,
            &[
                ("sb-gruenflaeche", green, dataset),
                ("sb-flaechen", surfaces, dataset),
            ],
            crs,
        )
    }
}

fn survey_areas(
    drawing: &SetDrawing,
    kinds: &[LayerKinds],
    layers: &[StoredLayer<'_>],
    from: Crs,
    to: Crs,
) -> Vec<SurveyArea> {
    let mut areas = Vec::new();
    for kind in kinds {
        for &(layer, features, fetched) in layers.iter().filter(|(l, _, _)| *l == kind.layer) {
            let source = format!("{}/{layer}", drawing.source_prefix);
            for feature in features {
                let typed = feature["properties"][kind.field]
                    .as_str()
                    .is_some_and(|t| kind.kinds.contains(&t));
                if !typed {
                    continue;
                }
                let tags = dated_tags(drawing, layer, feature, fetched);
                let value =
                    |key: &str| tags.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
                let id = value("freemapper:source_id")
                    .or_else(|| feature["id"].as_str())
                    .unwrap_or_default()
                    .to_string();
                let surveyed = value("freemapper:currency")
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(fetched);
                let guessed = value("freemapper:currency_guessed") == Some("yes");
                let carriageway = tags.iter().any(|(k, v)| {
                    drawing
                        .carriageway_tags
                        .iter()
                        .any(|(key, value)| k == key && v == value)
                });
                for polygon in polygons_in(feature, from, to) {
                    let polygon = polygon.orient(Direction::Default);
                    // Also skips NaN areas and degenerate rings (a point, a line).
                    let area = polygon.unsigned_area();
                    if area.is_nan() || area <= 0.0 {
                        continue;
                    }
                    let Some(bounds) = polygon.bounding_rect() else {
                        continue;
                    };
                    areas.push(SurveyArea {
                        polygon,
                        bounds,
                        id: id.clone(),
                        surveyed,
                        guessed,
                        tags: tags.clone(),
                        source: source.clone(),
                        carriageway,
                    });
                }
            }
        }
    }
    areas
}

/// 501.5 objects for the traffic areas of `land_use`, clipped to `clip` (map area plus buffer,
/// the map's CRS). A street parcel crossed by one of `unpaved_streets` (same CRS) is a candidate.
/// The `survey` green strips are cut out of the map objects ([`subtract_green`]) and are 401
/// candidates where no 501.5 object or candidate stays; its paved surfaces outside the parcels are 501.5 candidates
/// ([`surveyed_paved_candidates`]), its carriageways inside the street parcels hidden 501.11
/// candidates ([`carriageway_candidates`]).
pub fn paved_area_objects(
    land_use: &[LandUse],
    unpaved_streets: &[LineString<f64>],
    survey: &StreetSurvey,
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<PavedObjects, GeorefError> {
    let symbol_id = symbol_set
        .id_for_code(PAVED_AREA_CODE)
        .expect("ISSprOM defines 501.5");
    let scale = SymbolScale::drawn_with(symbol_set, georef.scale_denominator);
    let green_clip = LandClip::with_min_side(clip, scale, GREEN_MIN_SIDE_MM);
    let surface_clip = LandClip::with_min_side(clip, scale, SURVEYED_PAVED_MIN_SIDE_MM);
    let clip = LandClip::new(clip, scale);
    let green_union = geo::unary_union(
        survey
            .green
            .iter()
            .filter(|g| g.polygon.intersects(&clip.polygon))
            .map(|g| &g.polygon),
    );
    let mut objects = PavedObjects::default();
    // What the 501.5 objects and candidates cover inside the window (`paved`), and that plus the
    // strips cut out (`drawn`).
    let mut paved = Vec::new();
    let mut drawn = Vec::new();
    // The pieces of `paved` from street parcels ([`STREET_NUTZARTEN`]).
    let mut streets = Vec::new();
    for land in land_use {
        let street = STREET_NUTZARTEN.contains(&land.nutzart.as_str());
        let reliable = match (land.nutzart.as_str(), land.bez.as_deref()) {
            ("Straßenverkehr", Some(VERGE)) => continue,
            ("Straßenverkehr", _) => true,
            ("Platz", None) => true,
            ("Platz", Some(bez)) => PAVED_SQUARES.contains(&bez),
            ("Weg", _) => false,
            _ => continue,
        };
        for part in &land.footprint.0 {
            let unpaved = unpaved_streets.iter().any(|street| street.intersects(part));
            let map = reliable && !unpaved;
            let cut = if map {
                subtract_green(part, &survey.green, &green_union, &green_clip)
            } else {
                None
            };
            let Some(cut) = cut else {
                let target = if map {
                    &mut objects.map
                } else {
                    &mut objects.candidates
                };
                for piece in clipped_pieces(part, &clip, f64::INFINITY) {
                    target.push(area_object(&piece, symbol_id, land_tags(land), georef)?);
                    if street {
                        streets.push(piece.clone());
                    }
                    paved.push(piece);
                }
                continue;
            };
            drawn.extend(cut.cut_out.iter().cloned());
            let pieces = cut
                .pieces
                .iter()
                .flat_map(|p| clipped_pieces(p, &clip, SURVEY_SLIVER_CAP_M));
            for piece in pieces {
                let cut_tags = cut.tags_for(&piece);
                // A remnant of the cut keeps what is wider than the strips themselves: a
                // sidewalk beyond a verge stays. Other pieces keep the paper limit.
                if cut_tags.is_empty() && clip.is_sliver(&piece, f64::INFINITY) {
                    continue;
                }
                let mut tags = land_tags(land);
                tags.extend(cut_tags);
                objects
                    .map
                    .push(area_object(&piece, symbol_id, tags, georef)?);
                if street {
                    streets.push(piece.clone());
                }
                paved.push(piece);
            }
        }
    }
    drawn.extend(paved.iter().cloned());
    // A strip on paving that stays (not cut: too narrow an overlap, a `Weg` candidate) is no
    // 401 candidate there.
    if let Some(open_land) = symbol_set.id_for_code(GREEN_CODE) {
        objects.candidates.extend(survey_candidates(
            &survey.green,
            &paved,
            &green_clip,
            open_land,
            georef,
        )?);
    }
    objects.candidates.extend(surveyed_paved_candidates(
        &survey.surfaces,
        &drawn,
        &surface_clip,
        symbol_id,
        georef,
    )?);
    if let Some(heavy) = symbol_set.id_for_code(CARRIAGEWAY_CODE) {
        objects.candidates.extend(carriageway_candidates(
            &survey.surfaces,
            &streets,
            &surface_clip,
            heavy,
            georef,
        )?);
    }
    Ok(objects)
}

/// Hidden candidates with `symbol_id` (501.11) for the surveyed carriageways among `surfaces`,
/// cut to the 501.5 pieces of the street parcels `streets` (map CRS, inside the clip window, the
/// green strips cut out), slivers dropped: what the survey found to be carriageway where ALKIS
/// draws the whole parcel as one paved area. Each carries its area's survey tags and
/// [`HIDDEN_KEY`]` = `[`HIDDEN_CARRIAGEWAY`].
fn carriageway_candidates(
    surfaces: &[SurveyArea],
    streets: &[Polygon<f64>],
    clip: &LandClip,
    symbol_id: u32,
    georef: &Georeferencing,
) -> Result<Vec<MapObject>, GeorefError> {
    let streets: Vec<(&Polygon<f64>, Rect<f64>)> = streets
        .iter()
        .filter_map(|s| Some((s, s.bounding_rect()?)))
        .collect();
    let mut objects = Vec::new();
    let carriageways = surfaces.iter().filter(|a| a.carriageway);
    for area in carriageways {
        let near: Vec<&Polygon<f64>> = streets
            .iter()
            .filter(|(_, bounds)| bounds.intersects(&area.bounds))
            .map(|(s, _)| *s)
            .collect();
        if near.is_empty() {
            continue;
        }
        // The pieces can overlap a little where parcels do; the union keeps them once.
        let inside = area.polygon.intersection(&geo::unary_union(near));
        for piece in inside.0 {
            if clip.is_sliver(&piece, SURVEY_SLIVER_CAP_M) {
                continue;
            }
            let mut tags = area.tags.clone();
            tags.push((HIDDEN_KEY.to_string(), HIDDEN_CARRIAGEWAY.to_string()));
            objects.push(area_object(&piece, symbol_id, tags, georef)?);
        }
    }
    Ok(objects)
}

/// A paved part with the surveyed green strips cut out.
pub(crate) struct GreenCut<'a> {
    /// What is left of the part inside the clip window; slivers are dropped later.
    pub pieces: Vec<Polygon<f64>>,
    /// The cut-out overlaps of part and strips.
    pub cut_out: Vec<Polygon<f64>>,
    /// The strips that took part, each with its share of `cut_out`.
    strips: Vec<(&'a SurveyArea, MultiPolygon<f64>)>,
}

impl GreenCut<'_> {
    /// The cut tags of `piece` (one of the remnants) for the strips along its edges:
    /// `freemapper:cut` (the survey layer), `freemapper:cut_ids` (their `gis_id`s) and
    /// `freemapper:cut_currency`: the oldest survey date that is not guessed, else the oldest
    /// guessed one with `freemapper:cut_currency_guessed=yes`. Empty when no strip touches it.
    pub(crate) fn tags_for(&self, piece: &Polygon<f64>) -> Vec<(String, String)> {
        let touching: Vec<&SurveyArea> = self
            .strips
            .iter()
            .filter(|(_, share)| Euclidean.distance(piece, share) <= TOUCH_M)
            .map(|(strip, _)| *strip)
            .collect();
        let mut ids: Vec<&str> = Vec::new();
        for strip in &touching {
            if !ids.contains(&strip.id.as_str()) {
                ids.push(&strip.id);
            }
        }
        let oldest = |guessed: bool| {
            touching
                .iter()
                .filter(|s| s.guessed == guessed)
                .map(|s| s.surveyed)
                .min()
        };
        let (currency, guessed) = match (oldest(false), oldest(true)) {
            (Some(date), _) => (date, false),
            (None, Some(date)) => (date, true),
            (None, None) => return Vec::new(),
        };
        let mut tags = vec![
            ("freemapper:cut".to_string(), touching[0].source.clone()),
            ("freemapper:cut_ids".to_string(), ids.join("; ")),
            ("freemapper:cut_currency".to_string(), currency.to_string()),
        ];
        if guessed {
            tags.push((
                "freemapper:cut_currency_guessed".to_string(),
                "yes".to_string(),
            ));
        }
        tags
    }
}

/// Cuts the `green` strips (their union `green_union`, map CRS) out of the paved `part` inside
/// `clip`'s window. Only overlaps above `clip`'s sliver limits (capped at
/// [`SURVEY_SLIVER_CAP_M`], as the 401 candidates) are cut. `None` when there is none: the part
/// stays as it is.
pub(crate) fn subtract_green<'a>(
    part: &Polygon<f64>,
    green: &'a [SurveyArea],
    green_union: &MultiPolygon<f64>,
    clip: &LandClip,
) -> Option<GreenCut<'a>> {
    let bounds = part.bounding_rect()?;
    let near = MultiPolygon(
        green_union
            .0
            .iter()
            .filter(|g| g.bounding_rect().is_some_and(|r| r.intersects(&bounds)))
            .cloned()
            .collect(),
    );
    if near.0.is_empty() {
        return None;
    }
    // Cut down to the window first: a street parcel can be far larger than the map.
    let windowed = part.orient(Direction::Default).intersection(&clip.polygon);
    let cut_out = MultiPolygon(
        windowed
            .intersection(&near)
            .0
            .into_iter()
            .filter(|overlap| !clip.is_sliver(overlap, SURVEY_SLIVER_CAP_M))
            .collect(),
    );
    if cut_out.0.is_empty() {
        return None;
    }
    let cut_bounds = cut_out.bounding_rect()?;
    let strips = green
        .iter()
        .filter(|strip| strip.bounds.intersects(&cut_bounds))
        .filter_map(|strip| {
            let share = strip.polygon.intersection(&cut_out);
            (share.unsigned_area() > CUT_OVERLAP_M2).then_some((strip, share))
        })
        .collect();
    Some(GreenCut {
        pieces: windowed.difference(&cut_out).0,
        cut_out: cut_out.0,
        strips,
    })
}

/// 501.5 candidates for the surveyed `surfaces` outside the `drawn` parcels (map CRS, inside the
/// clip window): what the survey found paved where ALKIS has no traffic area, mostly park paths,
/// squares and forecourts. Both are in the map's CRS, so the difference is too.
pub(crate) fn surveyed_paved_candidates(
    surfaces: &[SurveyArea],
    drawn: &[Polygon<f64>],
    clip: &LandClip,
    symbol_id: u32,
    georef: &Georeferencing,
) -> Result<Vec<MapObject>, GeorefError> {
    survey_candidates(surfaces, drawn, clip, symbol_id, georef)
}

/// Candidates with `symbol_id` for the `areas` inside `clip`'s window, minus `taken` (map CRS),
/// slivers dropped; each carries its area's survey tags.
fn survey_candidates(
    areas: &[SurveyArea],
    taken: &[Polygon<f64>],
    clip: &LandClip,
    symbol_id: u32,
    georef: &Georeferencing,
) -> Result<Vec<MapObject>, GeorefError> {
    let Some(window) = clip.polygon.bounding_rect() else {
        return Ok(Vec::new());
    };
    let taken: Vec<(&Polygon<f64>, Rect<f64>)> = taken
        .iter()
        .filter_map(|t| Some((t, t.bounding_rect()?)))
        .collect();
    let mut objects = Vec::new();
    for area in areas.iter().filter(|a| a.bounds.intersects(&window)) {
        // The union of the parcels near this area only: the street network as one polygon
        // would make every difference cost the whole network.
        let near: Vec<&Polygon<f64>> = taken
            .iter()
            .filter(|(_, bounds)| bounds.intersects(&area.bounds))
            .map(|(t, _)| *t)
            .collect();
        let mut rest = area.polygon.intersection(&clip.polygon);
        if !near.is_empty() {
            // Parcels can overlap a little; the union keeps the even-odd rule from reopening
            // the overlap.
            rest = rest.difference(&geo::unary_union(near));
        }
        for piece in rest.0 {
            if clip.is_sliver(&piece, SURVEY_SLIVER_CAP_M) {
                continue;
            }
            objects.push(area_object(&piece, symbol_id, area.tags.clone(), georef)?);
        }
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::project::MapKind;
    use geo_types::{LineString, MultiPolygon, Polygon};
    use serde_json::Value;

    struct Fixture {
        georef: Georeferencing,
        clip: BoundingBox,
        centre: (f64, f64),
    }

    /// 1:4000 in Strausberg Vorstadt; clip 300 × 300 m around the reference point.
    fn fixture() -> Fixture {
        let date = "2026-11-07".parse().unwrap();
        let georef = Georeferencing::new(
            LatLon {
                lat: 52.5385,
                lon: 13.844,
            },
            4000,
            date,
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        Fixture {
            clip: BoundingBox {
                min_e: e - 150.0,
                min_n: n - 150.0,
                max_e: e + 150.0,
                max_n: n + 150.0,
            },
            georef,
            centre: (e, n),
        }
    }

    fn ring(e: f64, n: f64, w: f64, h: f64) -> LineString<f64> {
        LineString::from(vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)])
    }

    fn land(nutzart: &str, bez: Option<&str>, f: &Fixture) -> LandUse {
        let (e, n) = f.centre;
        LandUse {
            oid: "DEBBAL0100000022".to_string(),
            nutzart: nutzart.to_string(),
            bez: bez.map(str::to_string),
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(vec![Polygon::new(
                ring(e - 50.0, n - 6.0, 100.0, 12.0),
                vec![],
            )]),
        }
    }

    fn paved(f: &Fixture, land_use: &[LandUse]) -> (PavedObjects, SymbolSet) {
        paved_with(f, land_use, &StreetSurvey::default())
    }

    fn paved_with(
        f: &Fixture,
        land_use: &[LandUse],
        survey: &StreetSurvey,
    ) -> (PavedObjects, SymbolSet) {
        let set = MapKind::Sprint.symbol_set(4000);
        let objects = paved_area_objects(land_use, &[], survey, &f.clip, &f.georef, &set).unwrap();
        (objects, set)
    }

    /// A stored Straßenbefahrung feature (EPSG:25833) of WFS type `layer` as Task 1 writes it.
    fn survey_feature(layer: &str, gis_id: &str, date: &str, ring: &LineString<f64>) -> Value {
        let coordinates: Vec<[f64; 2]> = ring.coords().map(|c| [c.x, c.y]).collect();
        serde_json::json!({
            "type": "Feature",
            "id": format!("{layer}.1"),
            "geometry": {"type": "Polygon", "coordinates": [coordinates]},
            "properties": {
                "freemapper:layer": layer,
                "gis_id": gis_id,
                "survey_date": date,
                "material": "03",
                "shape_area": 1.0,
            },
        })
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The area object back in EPSG:25833: one ring per closed run of coordinates.
    fn polygon_of(object: &MapObject, georef: &Georeferencing) -> Polygon<f64> {
        use crate::omap::coord::FLAG_CLOSE;
        let mut rings = Vec::new();
        let mut ring = Vec::new();
        for c in &object.coords {
            let p = georef.map_to_projected(*c);
            ring.push((p.easting, p.northing));
            if c.flags & FLAG_CLOSE != 0 {
                rings.push(LineString::from(std::mem::take(&mut ring)));
            }
        }
        let mut rings = rings.into_iter();
        Polygon::new(rings.next().expect("a ring"), rings.collect())
    }

    fn area_of(object: &MapObject, georef: &Georeferencing) -> f64 {
        use geo::Area as _;
        polygon_of(object, georef).unsigned_area()
    }

    /// A set names the areas cut out of the paved areas of the cadastre and its surveyed paving
    /// (ADR 0149): Berlin's green strips and paved surfaces of the street survey.
    #[test]
    fn a_set_names_its_paving_cuts_and_surveyed_paving() {
        let berlin = crate::build::sets::drawing_of_source("geoportal-berlin/").unwrap();
        let named = |kinds: &[freemapper_adapter_api::extras::LayerKinds]| {
            kinds
                .iter()
                .map(|k| (k.layer, k.field, k.kinds))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            named(berlin.paving_cuts),
            [(
                "sb-gruenflaeche",
                "freemapper:layer",
                &["ce_gruenflaeche"][..]
            )]
        );
        assert_eq!(
            named(berlin.surveyed_paving),
            [(
                "sb-flaechen",
                "freemapper:layer",
                &[
                    "cm_fahrbahn",
                    "cl_gehweg",
                    "ch_radweg",
                    "ck_parkflaeche",
                    "ci_oeffentlicher_platz",
                    "cj_fussgaengerzone",
                    "by_gehwegueberfahrt",
                ][..]
            )]
        );
        assert_eq!(berlin.carriageway_tags, [("berlin:layer", "cm_fahrbahn")]);
    }

    #[test]
    fn traffic_areas_are_paved() {
        let f = fixture();
        let land_use = [
            land("Straßenverkehr", None, &f),
            land("Platz", Some("Parkplatz"), &f),
            land("Platz", None, &f),
        ];
        let (objects, set) = paved(&f, &land_use);
        assert_eq!(objects.map.len(), 3);
        assert!(objects.candidates.is_empty());
        let symbol = set.id_for_code("501.5").unwrap();
        assert!(objects.map.iter().all(|o| o.symbol_id == symbol));
    }

    #[test]
    fn verges_are_left_out() {
        let f = fixture();
        let land_use = [
            land("Straßenverkehr", Some("Begleitfläche Straßenverkehr"), &f),
            land("Wohnbaufläche", None, &f),
            land("Bahnverkehr", None, &f),
        ];
        let (objects, _) = paved(&f, &land_use);
        assert!(objects.map.is_empty() && objects.candidates.is_empty());
    }

    #[test]
    fn ways_are_candidates() {
        let f = fixture();
        let (objects, set) = paved(&f, &[land("Weg", None, &f)]);
        assert!(objects.map.is_empty());
        assert_eq!(objects.candidates.len(), 1);
        assert_eq!(
            objects.candidates[0].symbol_id,
            set.id_for_code("501.5").unwrap()
        );
    }

    #[test]
    fn paved_area_is_clipped() {
        let f = fixture();
        let (e, n) = f.centre;
        let mut street = land("Straßenverkehr", None, &f);
        street.footprint = MultiPolygon(vec![Polygon::new(
            ring(e - 1000.0, n - 8.0, 2000.0, 16.0),
            vec![],
        )]);
        // A 1 m strip at 1:4000 is a sliver (0.25 mm on paper).
        let mut sliver = land("Platz", None, &f);
        sliver.footprint = MultiPolygon(vec![Polygon::new(ring(e, n + 40.0, 50.0, 1.0), vec![])]);
        let (objects, _) = paved(&f, &[street, sliver]);
        assert_eq!(objects.map.len(), 1);
        for c in &objects.map[0].coords {
            let p = f.georef.map_to_projected(*c);
            assert!(
                p.easting <= f.clip.max_e + 0.01
                    && p.easting >= f.clip.min_e - 0.01
                    && p.northing <= f.clip.max_n + 0.01
                    && p.northing >= f.clip.min_n - 0.01,
                "{p:?}"
            );
        }
    }

    #[test]
    fn sliver_at_the_clip_edge_is_dropped() {
        let f = fixture();
        let (e, n) = f.centre;
        // 100 m long, but only 0.5 m of it inside the clip window.
        let mut car_park = land("Platz", None, &f);
        car_park.footprint = MultiPolygon(vec![Polygon::new(
            ring(e + 149.5, n - 20.0, 100.0, 40.0),
            vec![],
        )]);
        let (objects, _) = paved(&f, &[car_park]);
        assert!(objects.map.is_empty());
    }

    #[test]
    fn unpaved_street_makes_its_parcel_a_candidate() {
        let f = fixture();
        let (e, n) = f.centre;
        let sand_road = LineString::from(vec![(e - 80.0, n), (e + 80.0, n)]);
        let set = MapKind::Sprint.symbol_set(4000);
        let land_use = [land("Straßenverkehr", None, &f)];
        let objects = paved_area_objects(
            &land_use,
            &[sand_road],
            &StreetSurvey::default(),
            &f.clip,
            &f.georef,
            &set,
        )
        .unwrap();
        assert!(objects.map.is_empty());
        assert_eq!(objects.candidates.len(), 1);
    }

    #[test]
    fn unpaved_streets_are_stored() {
        use crate::build::candidates::Rules;
        for street in STREETS {
            for surface in UNPAVED {
                let tags = vec![
                    ("highway".to_string(), street.to_string()),
                    ("surface".to_string(), surface.to_string()),
                ];
                assert!(
                    Rules::embedded().wants(&tags, MapKind::Sprint),
                    "{street} {surface}"
                );
            }
        }
    }

    #[test]
    fn unpaved_streets_are_picked_and_projected() {
        let point = |lat, lon| LatLon { lat, lon };
        let way = |id: &str, highway: &str, surface: &str| OsmFeature {
            id: id.to_string(),
            tags: vec![
                ("highway".to_string(), highway.to_string()),
                ("surface".to_string(), surface.to_string()),
            ],
            geometry: OsmGeometry::Line(vec![point(52.538, 13.844), point(52.539, 13.845)]),
        };
        let streets = unpaved_streets(
            &[
                way("way/1", "residential", "sand"),
                way("way/2", "residential", "asphalt"),
                way("way/3", "path", "sand"),
            ],
            Crs::Utm33,
        );
        assert_eq!(streets.len(), 1);
        let start = Crs::Utm33.forward(point(52.538, 13.844));
        assert_eq!(streets[0].0[0].x, start.easting);
    }

    #[test]
    fn only_known_squares_are_paved() {
        let f = fixture();
        for (bez, map) in [
            (None, true),
            (Some("Parkplatz"), true),
            (Some("Marktplatz"), true),
            (Some("Fußgängerzone"), true),
            (Some("Festplatz"), false),
        ] {
            let (objects, _) = paved(&f, &[land("Platz", bez, &f)]);
            assert_eq!(
                (objects.map.len(), objects.candidates.len()),
                if map { (1, 0) } else { (0, 1) },
                "{bez:?}"
            );
        }
    }

    #[test]
    fn paved_tags_carry_provenance() {
        let f = fixture();
        let (objects, _) = paved(&f, &[land("Platz", Some("Parkplatz"), &f)]);
        let tags = &objects.map[0].tags;
        for (k, v) in [
            ("freemapper:source", "alkis-nutzung"),
            ("freemapper:source_id", "DEBBAL0100000022"),
            ("freemapper:nutzart", "Platz / Parkplatz"),
            ("freemapper:currency", "2025-09-30"),
        ] {
            assert!(
                tags.iter().any(|(tk, tv)| tk == k && tv == v),
                "{k}={v} in {tags:?}"
            );
        }
    }

    #[test]
    fn surveyed_green_is_cut_from_paved_area() {
        let f = fixture();
        let (e, n) = f.centre;
        // The street is 100 × 12 m (1200 m²); at 1:4000 a green must be 4 m wide to be cut.
        let green = [
            // A 6 m strip along the south edge (240 m²) and a 6 m island (120 m², a hole).
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_1",
                "2015-04-16",
                &ring(e - 30.0, n - 6.0, 40.0, 6.0),
            ),
            survey_feature(
                "ce_gruenflaeche",
                "FL_04082014_2",
                "2014-08-04",
                &ring(e + 20.0, n - 3.0, 20.0, 6.0),
            ),
            // A park lawn that reaches 0.3 m into the street: the boundaries differ, nothing
            // is cut.
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_3",
                "2015-04-16",
                &ring(e - 50.0, n + 5.7, 30.0, 8.0),
            ),
            // A 1.5 m strip is below the 2 m limit of survey pieces: not cut, and no candidate
            // either.
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_4",
                "2015-04-16",
                &ring(e - 45.0, n - 1.0, 10.0, 1.5),
            ),
        ];
        // A market square cut by one strip whose `gis_id` gave no date.
        let mut unknown = survey_feature(
            "ce_gruenflaeche",
            "Fl_unknown",
            "2014-01-01",
            &ring(e + 20.0, n - 70.0, 10.0, 10.0),
        );
        unknown["properties"]["survey_date_guessed"] = "yes".into();
        let green: Vec<Value> = green.into_iter().chain([unknown]).collect();
        let survey = StreetSurvey::from_features(&green, &[], f.georef.crs());
        let mut car_park = land("Platz", Some("Parkplatz"), &f);
        car_park.footprint = MultiPolygon(vec![Polygon::new(
            ring(e - 50.0, n + 60.0, 40.0, 20.0),
            vec![],
        )]);
        let mut market = land("Platz", Some("Marktplatz"), &f);
        market.footprint = MultiPolygon(vec![Polygon::new(ring(e, n - 80.0, 40.0, 30.0), vec![])]);
        let (objects, set) = paved_with(
            &f,
            &[land("Straßenverkehr", None, &f), car_park, market],
            &survey,
        );
        assert_eq!(objects.map.len(), 3);
        let street = &objects.map[0];
        let area = area_of(street, &f.georef);
        assert!((area - 840.0).abs() < 1.0, "{area}");
        assert_eq!(polygon_of(street, &f.georef).interiors().len(), 1);
        // The ALKIS provenance stays; the cut names the survey strips and their oldest date.
        for (k, v) in [
            ("freemapper:source", "alkis-nutzung"),
            ("freemapper:source_id", "DEBBAL0100000022"),
            ("freemapper:nutzart", "Straßenverkehr"),
            ("freemapper:currency", "2025-09-30"),
            ("freemapper:cut", "geoportal-berlin/sb-gruenflaeche"),
            ("freemapper:cut_ids", "Fl_20150416_1; FL_04082014_2"),
            ("freemapper:cut_currency", "2014-08-04"),
        ] {
            assert_eq!(tag(street, k), Some(v), "{k} in {:?}", street.tags);
        }
        // Where no green lies, the object is as before.
        let car_park = &objects.map[1];
        assert!((area_of(car_park, &f.georef) - 800.0).abs() < 1.0);
        assert_eq!(tag(car_park, "freemapper:cut"), None);
        // Only a guessed date: it is named, and marked as guessed.
        let market = &objects.map[2];
        assert_eq!(tag(market, "freemapper:cut_ids"), Some("Fl_unknown"));
        assert_eq!(tag(market, "freemapper:cut_currency"), Some("2014-01-01"));
        assert_eq!(tag(market, "freemapper:cut_currency_guessed"), Some("yes"));
        assert_eq!(tag(street, "freemapper:cut_currency_guessed"), None);
        // The surveyed greens wider than 2 m are 401 candidates with the survey date.
        let open_land = set.id_for_code("401").unwrap();
        let greens: Vec<&MapObject> = objects
            .candidates
            .iter()
            .filter(|o| o.symbol_id == open_land)
            .collect();
        assert_eq!(greens.len(), 4);
        assert_eq!(
            tag(greens[0], "freemapper:source"),
            Some("geoportal-berlin/sb-gruenflaeche")
        );
        assert_eq!(tag(greens[0], "freemapper:currency"), Some("2015-04-16"));
    }

    #[test]
    fn green_candidates_lie_only_where_paving_was_cut() {
        let f = fixture();
        let (e, n) = f.centre;
        let green = [
            // Cut out of the street: the 401 candidate is the cut part.
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_1",
                "2015-04-16",
                &ring(e - 30.0, n - 6.0, 40.0, 6.0),
            ),
            // A park lawn 0.3 m into the street: not cut, so the candidate stops at the street.
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_3",
                "2015-04-16",
                &ring(e - 50.0, n + 5.7, 30.0, 8.0),
            ),
            // A strip on a `Weg` parcel, which is a candidate and never cut: no 401 there.
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_6",
                "2015-04-16",
                &ring(e - 20.0, n - 60.0, 20.0, 6.0),
            ),
        ];
        let survey = StreetSurvey::from_features(&green, &[], f.georef.crs());
        let mut lane = land("Weg", None, &f);
        lane.footprint = MultiPolygon(vec![Polygon::new(
            ring(e - 30.0, n - 62.0, 40.0, 10.0),
            vec![],
        )]);
        let (objects, set) = paved_with(&f, &[land("Straßenverkehr", None, &f), lane], &survey);
        let open_land = set.id_for_code("401").unwrap();
        let paved = set.id_for_code("501.5").unwrap();
        let greens: Vec<&MapObject> = objects
            .candidates
            .iter()
            .filter(|o| o.symbol_id == open_land)
            .collect();
        let ids: Vec<&str> = greens
            .iter()
            .filter_map(|o| tag(o, "freemapper:source_id"))
            .collect();
        assert_eq!(ids, ["Fl_20150416_1", "Fl_20150416_3"]);
        // The cut strip keeps its 240 m²; the lawn loses its 0.3 m in the street.
        assert!((area_of(greens[0], &f.georef) - 240.0).abs() < 1.0);
        assert!((area_of(greens[1], &f.georef) - 30.0 * 7.7).abs() < 1.0);
        // No 401 candidate lies on a 501.5 object or candidate (map coordinates round to 4 mm on
        // the ground; the lawn alone used to lie 9 m² on the street).
        let pavings: Vec<Polygon<f64>> = objects
            .map
            .iter()
            .chain(&objects.candidates)
            .filter(|o| o.symbol_id == paved)
            .map(|o| polygon_of(o, &f.georef))
            .collect();
        for green in &greens {
            let green = polygon_of(green, &f.georef);
            for paving in &pavings {
                let overlap = green.intersection(paving).unsigned_area();
                assert!(overlap < 0.5, "{overlap} m² of 401 on 501.5");
            }
        }
    }

    #[test]
    fn tree_pits_are_not_cut() {
        let f = fixture();
        let (e, n) = f.centre;
        // Larger than any real tree pit, so only its type keeps it from the cut.
        let pit = survey_feature(
            "bn_baumscheibe",
            "Fl_20150416_5",
            "2015-04-16",
            &ring(e - 10.0, n - 5.0, 8.0, 8.0),
        );
        let survey = StreetSurvey::from_features(&[pit], &[], f.georef.crs());
        let (objects, _) = paved_with(&f, &[land("Straßenverkehr", None, &f)], &survey);
        assert_eq!(objects.map.len(), 1);
        assert!((area_of(&objects.map[0], &f.georef) - 1200.0).abs() < 1.0);
        assert_eq!(tag(&objects.map[0], "freemapper:cut"), None);
        assert!(objects.candidates.is_empty());
    }

    #[test]
    fn verge_splits_parcel_into_sidewalk_and_carriageway() {
        let f = fixture();
        let (e, n) = f.centre;
        // A 14 m parcel: a 3.5 m sidewalk, a 2.5 m verge along the whole parcel, 8 m of
        // carriageway with a 5 m island. At 1:4000 the sidewalk is under 1 mm on paper, but a
        // remnant of a cut keeps what is wider than 2 m on the ground, like the verge itself.
        let mut street = land("Straßenverkehr", None, &f);
        street.footprint = MultiPolygon(vec![Polygon::new(
            ring(e - 50.0, n - 7.0, 100.0, 14.0),
            vec![],
        )]);
        let mut island = survey_feature(
            "ce_gruenflaeche",
            "Fl_unknown",
            "2014-01-01",
            &ring(e + 10.0, n + 1.0, 20.0, 5.0),
        );
        island["properties"]["survey_date_guessed"] = "yes".into();
        let green = [
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_11",
                "2015-04-16",
                &ring(e - 60.0, n - 3.5, 120.0, 2.5),
            ),
            island,
            // A strip 1 m from the west end leaves a 1 × 8 m remnant there: a sliver.
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_12",
                "2015-04-16",
                &ring(e - 49.0, n - 1.0, 10.0, 9.0),
            ),
        ];
        let survey = StreetSurvey::from_features(&green, &[], f.georef.crs());
        let (objects, set) = paved_with(&f, &[street], &survey);
        assert_eq!(objects.map.len(), 2, "{:?}", objects.map);
        let (sidewalk, carriageway) = (&objects.map[0], &objects.map[1]);
        let sidewalk_area = area_of(sidewalk, &f.georef);
        assert!((sidewalk_area - 350.0).abs() < 1.0, "{sidewalk_area}");
        // 800 m² less the island (100 m²), the strip (80 m²) and its remnant (8 m²).
        let carriageway_area = area_of(carriageway, &f.georef);
        assert!((carriageway_area - 612.0).abs() < 1.0, "{carriageway_area}");
        // Each piece names the strips along its own edges; a real date beats a guessed one.
        assert_eq!(tag(sidewalk, "freemapper:cut_ids"), Some("Fl_20150416_11"));
        assert_eq!(
            tag(carriageway, "freemapper:cut_ids"),
            Some("Fl_20150416_11; Fl_unknown; Fl_20150416_12")
        );
        for piece in [sidewalk, carriageway] {
            assert_eq!(tag(piece, "freemapper:cut_currency"), Some("2015-04-16"));
            assert_eq!(tag(piece, "freemapper:cut_currency_guessed"), None);
        }
        // The verge is cut, so its 401 candidate lies on no paving.
        let open_land = set.id_for_code("401").unwrap();
        assert_eq!(
            objects
                .candidates
                .iter()
                .filter(|c| c.symbol_id == open_land)
                .count(),
            3
        );
    }

    #[test]
    fn surveyed_paving_on_a_dropped_parcel_is_candidate() {
        let f = fixture();
        let (e, n) = f.centre;
        // A 3 m `Weg` parcel is under 1 mm on paper at 1:4000: no ALKIS candidate. The
        // surveyed sidewalk on it is one.
        let mut way = land("Weg", None, &f);
        way.footprint = MultiPolygon(vec![Polygon::new(
            ring(e - 40.0, n + 30.0, 60.0, 3.0),
            vec![],
        )]);
        let sidewalk = survey_feature(
            "cl_gehweg",
            "Fl_20150413_13",
            "2015-04-13",
            &ring(e - 40.0, n + 30.0, 60.0, 3.0),
        );
        let survey = StreetSurvey::from_features(&[], &[sidewalk], f.georef.crs());
        let (objects, _) = paved_with(&f, &[way], &survey);
        assert_eq!(objects.candidates.len(), 1, "{:?}", objects.candidates);
        assert_eq!(
            tag(&objects.candidates[0], "freemapper:source_id"),
            Some("Fl_20150413_13")
        );
    }

    #[test]
    fn surveyed_paving_outside_alkis_is_candidate() {
        let f = fixture();
        let (e, n) = f.centre;
        let mut guessed = survey_feature(
            "cl_gehweg",
            "Gw_unknown",
            "2014-01-01",
            &ring(e - 40.0, n + 40.0, 50.0, 3.0),
        );
        guessed["properties"]["survey_date_guessed"] = "yes".into();
        let surfaces = [
            // A sidewalk 3 m inside the street parcel and 6 m beyond it (360 m² outside).
            survey_feature(
                "cl_gehweg",
                "Fl_20150413_7",
                "2015-04-13",
                &ring(e - 40.0, n + 3.0, 60.0, 9.0),
            ),
            // A 3 m park path away from any street (150 m²).
            guessed,
            // The carriageway lies inside the parcel: ALKIS has it already, the training map
            // draws it from its hidden candidate.
            survey_feature(
                "cm_fahrbahn",
                "Fl_20150413_8",
                "2015-04-13",
                &ring(e - 50.0, n - 5.0, 100.0, 8.0),
            ),
            // A 1 m strip past the parcel is how ALKIS and survey disagree: a sliver.
            survey_feature(
                "ck_parkflaeche",
                "Fl_20150413_9",
                "2015-04-13",
                &ring(e + 30.0, n + 5.0, 20.0, 2.0),
            ),
        ];
        let survey = StreetSurvey::from_features(&[], &surfaces, f.georef.crs());
        let (objects, set) = paved_with(&f, &[land("Straßenverkehr", None, &f)], &survey);
        assert_eq!(objects.map.len(), 1);
        assert!((area_of(&objects.map[0], &f.georef) - 1200.0).abs() < 1.0);
        let paved = set.id_for_code("501.5").unwrap();
        let (hidden, candidates): (Vec<MapObject>, Vec<MapObject>) = objects
            .candidates
            .iter()
            .cloned()
            .partition(|c| tag(c, "freemapper:hidden").is_some());
        assert_eq!(hidden.len(), 1, "{hidden:?}");
        assert_eq!(
            tag(&hidden[0], "freemapper:source_id"),
            Some("Fl_20150413_8")
        );
        assert_eq!(candidates.len(), 2, "{candidates:?}");
        assert!(candidates.iter().all(|c| c.symbol_id == paved));
        let sidewalk = &candidates[0];
        assert!((area_of(sidewalk, &f.georef) - 360.0).abs() < 1.0);
        for (k, v) in [
            ("freemapper:source", "geoportal-berlin/sb-flaechen"),
            ("freemapper:source_id", "Fl_20150413_7"),
            ("freemapper:currency", "2015-04-13"),
            ("berlin:layer", "cl_gehweg"),
            ("berlin:material", "03"),
        ] {
            assert_eq!(tag(sidewalk, k), Some(v), "{k} in {:?}", sidewalk.tags);
        }
        assert_eq!(tag(sidewalk, "freemapper:currency_guessed"), None);
        assert_eq!(tag(sidewalk, "berlin:shape_area"), None);
        let path = &candidates[1];
        assert!((area_of(path, &f.georef) - 150.0).abs() < 1.0);
        assert_eq!(tag(path, "freemapper:currency"), Some("2014-01-01"));
        assert_eq!(tag(path, "freemapper:currency_guessed"), Some("yes"));
    }

    #[test]
    fn the_survey_carriageway_is_a_hidden_candidate() {
        use crate::build::content::{ContentSettings, Profile, route};
        let f = fixture();
        let (e, n) = f.centre;
        // A 100 × 12 m street parcel and a `Weg` beside it; the carriageway runs 20 m past the
        // parcel's east end and across the `Weg`.
        let mut way = land("Weg", None, &f);
        way.footprint = MultiPolygon(vec![Polygon::new(
            ring(e - 50.0, n + 6.0, 100.0, 10.0),
            vec![],
        )]);
        let surfaces = [
            survey_feature(
                "cm_fahrbahn",
                "Fl_20150413_8",
                "2015-04-13",
                &ring(e - 40.0, n - 3.0, 110.0, 6.0),
            ),
            survey_feature(
                "cm_fahrbahn",
                "Fl_20150413_9",
                "2015-04-13",
                &ring(e - 10.0, n + 8.0, 6.0, 6.0),
            ),
            // A sidewalk inside the parcel stays as it is: ALKIS has it.
            survey_feature(
                "cl_gehweg",
                "Fl_20150413_10",
                "2015-04-13",
                &ring(e - 40.0, n + 3.0, 80.0, 3.0),
            ),
        ];
        let survey = StreetSurvey::from_features(&[], &surfaces, f.georef.crs());
        let (objects, set) = paved_with(&f, &[land("Straßenverkehr", None, &f), way], &survey);
        let heavy = set.id_for_code("501.11").unwrap();
        let carriageways: Vec<&MapObject> = objects
            .candidates
            .iter()
            .filter(|c| c.symbol_id == heavy)
            .collect();
        // Only the one in the street parcel, cut to it: 90 × 6 m.
        assert_eq!(carriageways.len(), 1, "{:?}", objects.candidates);
        let carriageway = carriageways[0];
        let area = area_of(carriageway, &f.georef);
        assert!((area - 540.0).abs() < 1.0, "{area}");
        for (k, v) in [
            ("freemapper:hidden", "carriageway"),
            ("freemapper:source", "geoportal-berlin/sb-flaechen"),
            ("freemapper:source_id", "Fl_20150413_8"),
            ("freemapper:currency", "2015-04-13"),
            ("berlin:layer", "cm_fahrbahn"),
        ] {
            assert_eq!(
                tag(carriageway, k),
                Some(v),
                "{k} in {:?}",
                carriageway.tags
            );
        }
        // The rest is as before: the parcel on the map, the `Weg` and the 20 m past the parcel
        // as 501.5 candidates, no candidate for the sidewalk.
        let light = set.id_for_code("501.5").unwrap();
        assert_eq!(objects.map.len(), 1);
        assert_eq!(objects.candidates.len(), 3, "{:?}", objects.candidates);
        assert_eq!(
            objects
                .candidates
                .iter()
                .filter(|c| c.symbol_id == light)
                .count(),
            2
        );
        // No content setting puts it on the map.
        let (mut map, mut candidates) = (objects.map.clone(), objects.candidates.clone());
        let full = ContentSettings::with_profile(Profile::Full);
        route(&full, &mut map, &mut candidates, &set);
        assert!(candidates.contains(carriageway));
        assert!(!map.iter().any(|o| o.symbol_id == heavy));
    }

    #[test]
    fn no_carriageway_without_a_street_parcel_or_outside_berlin() {
        let f = fixture();
        let (e, n) = f.centre;
        let carriageway = survey_feature(
            "cm_fahrbahn",
            "Fl_20150413_8",
            "2015-04-13",
            &ring(e - 40.0, n - 3.0, 80.0, 6.0),
        );
        let survey = StreetSurvey::from_features(&[], &[carriageway], f.georef.crs());
        let (objects, set) = paved_with(&f, &[land("Weg", None, &f)], &survey);
        let heavy = set.id_for_code("501.11").unwrap();
        assert!(!objects.candidates.iter().any(|c| c.symbol_id == heavy));
        let (objects, _) = paved(&f, &[land("Straßenverkehr", None, &f)]);
        assert!(!objects.candidates.iter().any(|c| c.symbol_id == heavy));
    }

    #[test]
    fn no_candidate_overlaps_paved_object() {
        use geo::{Area as _, BooleanOps as _};
        let f = fixture();
        let (e, n) = f.centre;
        let parcel = |nutzart: &str, bez: Option<&str>, ring: LineString<f64>| LandUse {
            footprint: MultiPolygon(vec![Polygon::new(ring, vec![])]),
            ..land(nutzart, bez, &f)
        };
        let land_use = [
            parcel("Straßenverkehr", None, ring(e - 60.0, n - 6.0, 120.0, 12.0)),
            parcel(
                "Platz",
                Some("Parkplatz"),
                ring(e - 20.0, n + 6.0, 30.0, 25.0),
            ),
            parcel("Weg", None, ring(e + 40.0, n + 6.0, 6.0, 60.0)),
        ];
        let green = [
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_10",
                "2015-04-16",
                &ring(e - 50.0, n - 6.0, 20.0, 8.0),
            ),
            // 2.5 m: cut, so its 401 candidate overlaps no map object either.
            survey_feature(
                "ce_gruenflaeche",
                "Fl_20150416_14",
                "2015-04-16",
                &ring(e + 20.0, n - 2.0, 30.0, 2.5),
            ),
        ];
        // Every surface overlaps a parcel and reaches out of it; one crosses all three.
        let surfaces: Vec<Value> = [
            ring(e - 70.0, n - 10.0, 30.0, 25.0),
            ring(e - 25.0, n + 2.0, 80.0, 40.0),
            ring(e + 5.0, n - 8.0, 10.0, 70.0),
            ring(e - 55.0, n - 4.0, 10.0, 6.0),
        ]
        .iter()
        .enumerate()
        .map(|(i, r)| {
            survey_feature(
                "ci_oeffentlicher_platz",
                &format!("Pl_20150413_{i}"),
                "2015-04-13",
                r,
            )
        })
        .collect();
        let survey = StreetSurvey::from_features(&green, &surfaces, f.georef.crs());
        let (objects, set) = paved_with(&f, &land_use, &survey);
        let paved = set.id_for_code("501.5").unwrap();
        let surveyed: Vec<&MapObject> = objects
            .candidates
            .iter()
            .filter(|c| tag(c, "freemapper:source") == Some("geoportal-berlin/sb-flaechen"))
            .collect();
        assert!(surveyed.len() >= 3, "{surveyed:?}");
        let alkis: Vec<Polygon<f64>> = objects
            .map
            .iter()
            .chain(objects.candidates.iter().filter(|c| {
                c.symbol_id == paved && tag(c, "freemapper:source") == Some("alkis-nutzung")
            }))
            .map(|o| polygon_of(o, &f.georef))
            .collect();
        assert_eq!(alkis.len(), 3);
        for candidate in &surveyed {
            let candidate = polygon_of(candidate, &f.georef);
            for object in &alkis {
                let overlap = candidate.intersection(object).unsigned_area();
                assert!(overlap < 0.05, "{overlap} m²");
            }
        }
        let open_land = set.id_for_code("401").unwrap();
        let greens: Vec<Polygon<f64>> = objects
            .candidates
            .iter()
            .filter(|c| c.symbol_id == open_land)
            .map(|c| polygon_of(c, &f.georef))
            .collect();
        assert_eq!(greens.len(), 2);
        for green in &greens {
            for object in objects.map.iter().map(|o| polygon_of(o, &f.georef)) {
                let overlap = green.intersection(&object).unsigned_area();
                assert!(overlap < 0.05, "401 on paving: {overlap} m²");
            }
        }
    }
}
