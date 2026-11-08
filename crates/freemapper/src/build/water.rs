//! ALKIS water parcels → 301.1 uncrossable water with 301.6 bank lines (spec §4.5).
//!
//! A river is split into many parcels, so the parts are dissolved into one shape first: the
//! fill and the bank line are separate symbols, and the bank line runs only along the shore,
//! never along a parcel boundary or the clip edge (the map edge).

use geo::orient::Direction;
use geo::{Area as _, BooleanOps as _, BoundingRect as _, Intersects as _, Orient as _};
use geo_types::{Coord, LineString, MultiPolygon, Polygon};

use crate::build::out_of_bounds::{LandClip, mean_width, nutzart_label};
use crate::crs::{BoundingBox, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::sources::alkis::LandUse;

/// ISSprOM 301.1 uncrossable body of water, fill only.
const FILL_CODE: &str = "301.1";
/// ISSprOM 301.6 bank line of uncrossable water.
const BANK_CODE: &str = "301.6";
/// ISSprOM 301 uncrossable body of water, the combined symbol the OSM rules use for areas.
const OSM_AREA_CODE: &str = "301";
/// Land uses that are open water, with any `bez`. ALKIS has no depth: all of it is 301, never 302.
pub const WATER_LAND_USES: [&str; 4] =
    ["Fließgewässer", "Stehendes Gewässer", "Hafenbecken", "Meer"];
/// Parts narrower than this mean width (2·area/perimeter) are candidates: the ISSprOM 305 limit
/// for a crossable watercourse. It replaces `bez`, which BB, SN and ST do not give.
pub const NARROW_WATER_M: f64 = 2.0;
/// A segment with both ends this close to the same edge of the clip rectangle lies on it.
const EDGE_TOLERANCE_M: f64 = 0.01;
/// A parcel part counts as a source of a dissolved piece above this overlap; neighbours that
/// only touch the piece share no area.
const MIN_OVERLAP_M2: f64 = 0.01;

pub fn is_water(land: &LandUse) -> bool {
    WATER_LAND_USES.contains(&land.nutzart.as_str())
}

/// Water objects: map objects where ALKIS is reliable, candidates for the narrow parts.
#[derive(Debug)]
pub struct WaterObjects {
    /// 301.1 areas (islands as holes) and their 301.6 bank lines.
    pub map: Vec<MapObject>,
    /// The same for parts narrower than [`NARROW_WATER_M`]: ditches, or slivers of a river.
    pub candidates: Vec<MapObject>,
    /// The 301.1 areas of `map` as drawn (clipped, slivers and sliver holes dropped), in the
    /// map's CRS; OSM water areas inside it are duplicates.
    pub union: MultiPolygon<f64>,
}

/// A water part with the parcel it belongs to.
type Part<'a> = (&'a LandUse, Polygon<f64>);

/// 301.1 + 301.6 objects for the water parcels of `land_use`, clipped to `clip` (map area plus
/// buffer, the map's CRS).
pub fn water_objects(
    land_use: &[LandUse],
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<WaterObjects, GeorefError> {
    let symbols = Symbols {
        fill: symbol_set
            .id_for_code(FILL_CODE)
            .expect("ISSprOM defines 301.1"),
        bank: symbol_set
            .id_for_code(BANK_CODE)
            .expect("ISSprOM defines 301.6"),
    };
    let land_clip = LandClip::new(clip, georef);
    let (mut wide, mut narrow): (Vec<Part>, Vec<Part>) = (Vec::new(), Vec::new());
    for land in land_use.iter().filter(|l| is_water(l)) {
        for part in &land.footprint.0 {
            // One winding order for all parts: the union's fill rule follows the first ring.
            let part = part.orient(Direction::Default);
            // Also skips NaN areas and degenerate rings (a point, a line, a bow tie).
            let area = part.unsigned_area();
            if area.is_nan() || area <= 0.0 {
                continue;
            }
            if mean_width(&part) >= NARROW_WATER_M {
                wide.push((land, part));
            } else {
                narrow.push((land, part));
            }
        }
    }
    let clipper = Clipper {
        clip: &land_clip,
        bbox: clip,
        georef,
        symbols,
    };
    // Wide water is no sliver down to 2 m on the ground, even where 1 mm on paper is more
    // (4 m at 1:4000): a mill stream or a small pond stays on the map. A candidate is narrow by
    // definition, so only the area limit drops it.
    let (map, drawn) = clipper.objects(&dissolve(&wide), &wide, NARROW_WATER_M)?;
    let (candidates, _) = clipper.objects(&dissolve(&narrow), &narrow, 0.0)?;
    Ok(WaterObjects {
        map,
        candidates,
        union: MultiPolygon(drawn),
    })
}

/// The union of `parts`: one shape per connected body of water.
fn dissolve(parts: &[Part]) -> MultiPolygon<f64> {
    geo::unary_union(parts.iter().map(|(_, part)| part))
}

fn rings(polygon: &Polygon<f64>) -> impl Iterator<Item = &LineString<f64>> {
    std::iter::once(polygon.exterior()).chain(polygon.interiors())
}

#[derive(Clone, Copy)]
struct Symbols {
    fill: u32,
    bank: u32,
}

struct Clipper<'a> {
    clip: &'a LandClip,
    bbox: &'a BoundingBox,
    georef: &'a Georeferencing,
    symbols: Symbols,
}

impl Clipper<'_> {
    /// Fill and bank line objects for the pieces of `shape` inside the clip window, tagged with
    /// the parcels of `sources` they come from, and the pieces drawn.
    ///
    /// Slivers are dropped (`LandClip::is_sliver`): pieces with the width limit capped at
    /// `width_cap_m`, and holes with the paper limit alone. The hole test uses the mean width
    /// 2·area/perimeter, which is the radius of a round island, so islands up to about 2 mm
    /// across on paper (8 m at 1:4000) are filled. Such an island inside uncrossable water is
    /// unreachable anyway; the test is meant for gaps between parcels that do not quite meet.
    fn objects(
        &self,
        shape: &MultiPolygon<f64>,
        sources: &[Part],
        width_cap_m: f64,
    ) -> Result<(Vec<MapObject>, Vec<Polygon<f64>>), GeorefError> {
        let mut objects = Vec::new();
        let mut drawn = Vec::new();
        if shape.0.is_empty() {
            return Ok((objects, drawn));
        }
        for piece in shape.intersection(&self.clip.polygon).0 {
            let islands = piece
                .interiors()
                .iter()
                .filter(|hole| {
                    !self
                        .clip
                        .is_sliver(&Polygon::new((*hole).clone(), vec![]), f64::INFINITY)
                })
                .cloned()
                .collect();
            let piece = Polygon::new(piece.exterior().clone(), islands);
            if self.clip.is_sliver(&piece, width_cap_m) {
                continue;
            }
            let tags = provenance(
                sources
                    .iter()
                    .filter(|(_, part)| overlaps(part, &piece))
                    .map(|(land, _)| *land),
            );
            let mut fill = Vec::new();
            let mut banks = Vec::new();
            for ring in rings(&piece) {
                fill.extend(closed_ring(&self.to_map(&ring.0)?));
                for (line, closed) in bank_lines(ring, self.bbox) {
                    let points = self.to_map(&line)?;
                    banks.push(if closed {
                        closed_ring(&points)
                    } else {
                        points.iter().map(|&(x, y)| MapCoord::new(x, y)).collect()
                    });
                }
            }
            objects.push(self.path(self.symbols.fill, fill, &tags));
            for coords in banks {
                objects.push(self.path(self.symbols.bank, coords, &tags));
            }
            drawn.push(piece);
        }
        Ok((objects, drawn))
    }

    fn to_map(&self, coords: &[Coord<f64>]) -> Result<Vec<(i32, i32)>, GeorefError> {
        coords
            .iter()
            .map(|c| {
                self.georef
                    .projected_to_map(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .map(|m| (m.x, m.y))
            })
            .collect()
    }

    fn path(&self, symbol_id: u32, coords: Vec<MapCoord>, tags: &[(String, String)]) -> MapObject {
        MapObject {
            symbol_id,
            kind: MapObjectKind::Path,
            coords,
            tags: tags.to_vec(),
        }
    }
}

/// `part` and `piece` share area, not only a boundary.
fn overlaps(part: &Polygon<f64>, piece: &Polygon<f64>) -> bool {
    let near = match (part.bounding_rect(), piece.bounding_rect()) {
        (Some(a), Some(b)) => a.intersects(&b),
        _ => false,
    };
    near && part.intersection(piece).unsigned_area() > MIN_OVERLAP_M2
}

/// The provenance tags of a piece dissolved from `lands`: the sorted oids, the distinct
/// `nutzart / bez` and the newest update.
fn provenance<'a>(lands: impl Iterator<Item = &'a LandUse>) -> Vec<(String, String)> {
    let mut oids = Vec::new();
    let mut nutzarten = Vec::new();
    let mut currency = None;
    for land in lands {
        oids.push(land.oid.clone());
        nutzarten.push(nutzart_label(land));
        currency = currency.max(land.updated);
    }
    for list in [&mut oids, &mut nutzarten] {
        list.sort();
        list.dedup();
    }
    let mut tags = vec![
        ("freemapper:source".to_string(), "alkis-nutzung".to_string()),
        ("freemapper:source_id".to_string(), oids.join(";")),
        ("freemapper:nutzart".to_string(), nutzarten.join(";")),
    ];
    if let Some(updated) = currency {
        tags.push(("freemapper:currency".to_string(), updated.to_string()));
    }
    tags
}

/// The bank lines of a closed `ring`, without its segments on the clip rectangle `bbox`: the
/// ring itself if no segment lies there (`true`, closed), else the open runs between them.
fn bank_lines(ring: &LineString<f64>, bbox: &BoundingBox) -> Vec<(Vec<Coord<f64>>, bool)> {
    let points = &ring.0;
    if points.len() < 3 {
        return Vec::new();
    }
    let near = |a: f64, b: f64, edge: f64| {
        (a - edge).abs() <= EDGE_TOLERANCE_M && (b - edge).abs() <= EDGE_TOLERANCE_M
    };
    let on_edge: Vec<bool> = points
        .windows(2)
        .map(|s| {
            let (p, q) = (s[0], s[1]);
            near(p.x, q.x, bbox.min_e)
                || near(p.x, q.x, bbox.max_e)
                || near(p.y, q.y, bbox.min_n)
                || near(p.y, q.y, bbox.max_n)
        })
        .collect();
    let Some(first_cut) = on_edge.iter().position(|&cut| cut) else {
        return vec![(points.clone(), true)];
    };
    // Walk once around the ring, starting after a cut segment, so no run wraps around.
    let segments = on_edge.len();
    let mut lines = Vec::new();
    let mut run: Vec<Coord<f64>> = Vec::new();
    for k in 1..=segments {
        let i = (first_cut + k) % segments;
        if on_edge[i] {
            if run.len() >= 2 {
                lines.push((std::mem::take(&mut run), false));
            }
            run.clear();
        } else {
            if run.is_empty() {
                run.push(points[i]);
            }
            run.push(points[i + 1]);
        }
    }
    lines
}

/// An OSM 301 area with at least this share of its area (inside the clip window) lying in ALKIS
/// water is a duplicate of it.
pub const OSM_WATER_COVERED: f64 = 0.5;

/// `objects` without the OSM 301 areas that ALKIS water already covers.
///
/// `union` is the drawn ALKIS water ([`WaterObjects::union`]), clipped to `clip`; the OSM area is
/// clipped to the same window first, so a lake reaching beyond it counts by its visible part.
/// Everything else (other symbols, areas that are not from OSM, areas outside the window) stays.
pub fn drop_covered_water(
    objects: Vec<MapObject>,
    union: &MultiPolygon<f64>,
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    let Some(water_id) = symbol_set.id_for_code(OSM_AREA_CODE) else {
        return objects;
    };
    if union.0.is_empty() {
        return objects;
    }
    let window = LandClip::new(clip, georef).polygon;
    objects
        .into_iter()
        .filter(|object| {
            let from_osm = object
                .tags
                .iter()
                .any(|(k, v)| k == "freemapper:source" && v == "osm");
            if object.symbol_id != water_id || !from_osm || object.coords.len() < 4 {
                return true;
            }
            let ring: Vec<Coord<f64>> = object
                .coords
                .iter()
                .map(|&c| {
                    let p = georef.map_to_projected(c);
                    Coord {
                        x: p.easting,
                        y: p.northing,
                    }
                })
                .collect();
            let area = Polygon::new(LineString(ring), vec![]);
            let visible = area.intersection(&window);
            let visible_area = visible.unsigned_area();
            if visible_area <= 0.0 {
                return true;
            }
            visible.intersection(union).unsigned_area() / visible_area < OSM_WATER_COVERED
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::{Crs, LatLon, ProjectedPoint};
    use crate::omap::MapObjectKind;
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
    use crate::project::MapKind;
    use geo::Centroid as _;
    use geo_types::{LineString, Polygon};

    struct Fixture {
        georef: Georeferencing,
        clip: BoundingBox,
        centre: (f64, f64),
    }

    /// 1:4000 in Strausberg Vorstadt; clip 300 × 300 m around the reference point.
    fn fixture() -> Fixture {
        let date = "2026-11-08".parse().unwrap();
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

    fn land(oid: &str, nutzart: &str, bez: Option<&str>, parts: Vec<Polygon<f64>>) -> LandUse {
        LandUse {
            oid: oid.to_string(),
            nutzart: nutzart.to_string(),
            bez: bez.map(str::to_string),
            updated: "2025-09-30".parse().ok(),
            footprint: MultiPolygon(parts),
        }
    }

    /// A rectangle `w` × `h` m with its south-west corner `(de, dn)` from the fixture's centre.
    fn rect(f: &Fixture, de: f64, dn: f64, w: f64, h: f64) -> Polygon<f64> {
        let (e, n) = f.centre;
        Polygon::new(ring(e + de, n + dn, w, h), vec![])
    }

    fn water(f: &Fixture, land_use: &[LandUse]) -> (WaterObjects, SymbolSet) {
        let set = MapKind::Sprint.symbol_set();
        let objects = water_objects(land_use, &f.clip, &f.georef, &set).unwrap();
        (objects, set)
    }

    fn with_code<'a>(objects: &'a [MapObject], set: &SymbolSet, code: &str) -> Vec<&'a MapObject> {
        let id = set.id_for_code(code).unwrap();
        objects.iter().filter(|o| o.symbol_id == id).collect()
    }

    /// Number of rings (sub-paths ended by the hole flag) of a path object.
    fn rings(object: &MapObject) -> usize {
        object
            .coords
            .iter()
            .filter(|c| c.flags & FLAG_HOLE != 0)
            .count()
    }

    fn is_closed(object: &MapObject) -> bool {
        object
            .coords
            .last()
            .is_some_and(|c| c.flags == FLAG_HOLE | FLAG_CLOSE)
    }

    fn projected(f: &Fixture, object: &MapObject) -> Vec<ProjectedPoint> {
        object
            .coords
            .iter()
            .map(|c| f.georef.map_to_projected(*c))
            .collect()
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn water_parcels_become_uncrossable_water() {
        let f = fixture();
        // All four land uses, with Hamburg's `Fluß` and without `bez` (BB, SN, ST).
        for (nutzart, bez) in [
            ("Fließgewässer", Some("Fluß")),
            ("Fließgewässer", None),
            ("Stehendes Gewässer", Some("Teich")),
            ("Hafenbecken", None),
            ("Meer", None),
        ] {
            let pond = land(
                "DEBBAL01",
                nutzart,
                bez,
                vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
            );
            let (objects, set) = water(&f, &[pond]);
            assert!(objects.candidates.is_empty(), "{nutzart}");
            let fills = with_code(&objects.map, &set, "301.1");
            let banks = with_code(&objects.map, &set, "301.6");
            assert_eq!(
                (fills.len(), banks.len(), objects.map.len()),
                (1, 1, 2),
                "{nutzart}"
            );
            assert_eq!(fills[0].kind, MapObjectKind::Path);
            assert_eq!(rings(fills[0]), 1);
            assert!(is_closed(banks[0]), "a pond's bank line is closed");
            assert_eq!(banks[0].coords.len(), 5);
            assert!((objects.union.unsigned_area() - 1200.0).abs() < 1e-6);
        }
        // The school set (125 %) has both symbols too.
        let set = MapKind::Schoolyard.symbol_set();
        let pond = land(
            "DEBBAL01",
            "Meer",
            None,
            vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
        );
        let objects = water_objects(&[pond], &f.clip, &f.georef, &set).unwrap();
        assert_eq!(objects.map.len(), 2);
    }

    #[test]
    fn adjacent_parcels_are_dissolved() {
        let f = fixture();
        // One river in three parcels; the middle one overlaps its neighbour by 1 cm.
        let land_use = [
            land(
                "DEBBAL03",
                "Fließgewässer",
                None,
                vec![rect(&f, -90.0, -10.0, 60.0, 20.0)],
            ),
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -30.01, -10.0, 60.01, 20.0)],
            ),
            land(
                "DEBBAL02",
                "Fließgewässer",
                None,
                vec![rect(&f, 30.0, -10.0, 60.0, 20.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!((fills.len(), banks.len()), (1, 1));
        assert_eq!(rings(fills[0]), 1);
        // No bank line point inside the river at the parcel boundaries.
        let (e, n) = f.centre;
        for p in projected(&f, banks[0]) {
            let inside = (p.northing - n).abs() < 9.9;
            let at_boundary = [-30.0, 30.0]
                .iter()
                .any(|de| (p.easting - (e + de)).abs() < 0.1);
            assert!(!(inside && at_boundary), "{p:?}");
        }
        assert_eq!(objects.union.0.len(), 1);
    }

    #[test]
    fn no_bank_line_on_the_clip_edge() {
        let f = fixture();
        // A 20 m river right across the clip window, and a lake reaching over its north-east corner.
        let land_use = [
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -1000.0, -10.0, 2000.0, 20.0)],
            ),
            land(
                "DEBBAL02",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, 100.0, 100.0, 200.0, 200.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 2);
        // River: the north and the south bank; lake: one line from the north to the east edge.
        assert_eq!(banks.len(), 3);
        for bank in &banks {
            assert!(!is_closed(bank), "a cut-off ring is open");
            let points = projected(&f, bank);
            for pair in points.windows(2) {
                let on_edge =
                    |a: f64, b: f64, edge: f64| (a - edge).abs() < 0.05 && (b - edge).abs() < 0.05;
                let (p, q) = (pair[0], pair[1]);
                assert!(
                    !(on_edge(p.easting, q.easting, f.clip.min_e)
                        || on_edge(p.easting, q.easting, f.clip.max_e)
                        || on_edge(p.northing, q.northing, f.clip.min_n)
                        || on_edge(p.northing, q.northing, f.clip.max_n)),
                    "segment {p:?} – {q:?} lies on the clip edge"
                );
            }
        }
        let lake = banks
            .iter()
            .find(|b| b.coords.len() == 3)
            .expect("the lake's bank line has three points");
        let corner = projected(&f, lake)[1];
        assert!(
            (corner.easting - (f.centre.0 + 100.0)).abs() < 0.01,
            "{corner:?}"
        );
        assert!(
            (corner.northing - (f.centre.1 + 100.0)).abs() < 0.01,
            "{corner:?}"
        );
    }

    #[test]
    fn islands_get_bank_lines() {
        let f = fixture();
        let (e, n) = f.centre;
        let lake = Polygon::new(
            ring(e - 50.0, n - 40.0, 100.0, 80.0),
            vec![ring(e - 10.0, n - 10.0, 20.0, 20.0)],
        );
        let (objects, set) = water(
            &f,
            &[land("DEBBAL01", "Stehendes Gewässer", None, vec![lake])],
        );
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 1);
        assert_eq!(rings(fills[0]), 2, "the island is a hole");
        assert_eq!(fills[0].coords.len(), 10);
        assert_eq!(banks.len(), 2, "shore and island");
        assert!(banks.iter().all(|b| is_closed(b)));
    }

    #[test]
    fn three_metre_stream_is_drawn() {
        let f = fixture();
        // 3 m is wide water, but under 1 mm (4 m) on paper at 1:4000. The 0.5 m left of a lake
        // inside the clip window is a sliver.
        let land_use = [
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -30.0, 20.0, 60.0, 3.0)],
            ),
            land(
                "DEBBAL02",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, 149.5, -20.0, 100.0, 40.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        assert!(objects.candidates.is_empty());
        let fills = with_code(&objects.map, &set, "301.1");
        assert_eq!(fills.len(), 1);
        assert_eq!(tag(fills[0], "freemapper:source_id"), Some("DEBBAL01"));
        assert_eq!(with_code(&objects.map, &set, "301.6").len(), 1);
        // The union is what is drawn: the stream, not the lake's sliver.
        assert!(
            (objects.union.unsigned_area() - 180.0).abs() < 1e-6,
            "{}",
            objects.union.unsigned_area()
        );
    }

    #[test]
    fn sliver_gaps_inside_water_are_dropped() {
        let f = fixture();
        // Two parcels of a lake around a third that leaves a 5 cm gap: a hole that is no island.
        let land_use = [
            land(
                "DEBBAL01",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -50.0, -40.0, 100.0, 30.0)],
            ),
            land(
                "DEBBAL02",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -50.0, 10.0, 100.0, 30.0)],
            ),
            land(
                "DEBBAL03",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -50.0, -10.0, 30.0, 20.0)],
            ),
            land(
                "DEBBAL04",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -19.95, -10.0, 69.95, 20.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 1);
        assert_eq!(rings(fills[0]), 1);
        assert_eq!(banks.len(), 1);
    }

    #[test]
    fn narrow_water_is_a_candidate() {
        let f = fixture();
        // A 1.5 m ditch in two parcels (no `bez`, as in BB) beside a pond.
        let land_use = [
            land(
                "DEBBAL01",
                "Fließgewässer",
                None,
                vec![rect(&f, -80.0, 50.0, 40.0, 1.5)],
            ),
            land(
                "DEBBAL02",
                "Fließgewässer",
                None,
                vec![rect(&f, -40.0, 50.0, 40.0, 1.5)],
            ),
            land(
                "DEBBAL03",
                "Stehendes Gewässer",
                None,
                vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
            ),
        ];
        let (objects, set) = water(&f, &land_use);
        assert_eq!(objects.map.len(), 2, "the pond");
        assert_eq!(with_code(&objects.candidates, &set, "301.1").len(), 1);
        assert_eq!(with_code(&objects.candidates, &set, "301.6").len(), 1);
        assert_eq!(objects.candidates.len(), 2);
        assert_eq!(
            tag(&objects.candidates[0], "freemapper:source_id"),
            Some("DEBBAL01;DEBBAL02")
        );
        assert!(
            (objects.union.unsigned_area() - 1200.0).abs() < 1e-6,
            "the ditch is not part of the union"
        );
    }

    #[test]
    fn other_land_use_is_not_water() {
        let f = fixture();
        let land_use: Vec<LandUse> = [
            ("Wald", None),
            ("Wohnbaufläche", None),
            ("Straßenverkehr", None),
            ("Moor", None),
            ("Sumpf", None),
            (
                "Unland/Vegetationslose Fläche",
                Some("Gewässerbegleitfläche"),
            ),
        ]
        .into_iter()
        .map(|(nutzart, bez)| {
            land(
                "DEBBAL01",
                nutzart,
                bez,
                vec![rect(&f, -20.0, -15.0, 40.0, 30.0)],
            )
        })
        .collect();
        let (objects, _) = water(&f, &land_use);
        assert!(objects.map.is_empty() && objects.candidates.is_empty());
        assert!(objects.union.0.is_empty());
        assert!(land_use.iter().all(|l| !is_water(l)));
    }

    #[test]
    fn water_tags_carry_provenance() {
        let f = fixture();
        let mut river = land(
            "DEHHAL02",
            "Fließgewässer",
            Some("Fluß"),
            vec![rect(&f, -60.0, -10.0, 60.0, 20.0)],
        );
        river.updated = "2024-01-02".parse().ok();
        let mut stretch = land(
            "DEHHAL01",
            "Fließgewässer",
            None,
            vec![rect(&f, 0.0, -10.0, 60.0, 20.0)],
        );
        stretch.updated = "2025-09-30".parse().ok();
        let mut far = land(
            "DEHHAL03",
            "Hafenbecken",
            None,
            vec![rect(&f, -60.0, 60.0, 30.0, 30.0)],
        );
        far.updated = None;
        let (objects, set) = water(&f, &[river, stretch, far]);
        let expected = [
            ("freemapper:source", "alkis-nutzung"),
            ("freemapper:source_id", "DEHHAL01;DEHHAL02"),
            ("freemapper:nutzart", "Fließgewässer;Fließgewässer / Fluß"),
            ("freemapper:currency", "2025-09-30"),
        ];
        for code in ["301.1", "301.6"] {
            let river = with_code(&objects.map, &set, code)
                .into_iter()
                .find(|o| tag(o, "freemapper:source_id") != Some("DEHHAL03"))
                .unwrap();
            let tags: Vec<(&str, &str)> = river
                .tags
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            assert_eq!(tags, expected, "{code}");
        }
        let harbour = objects
            .map
            .iter()
            .find(|o| tag(o, "freemapper:source_id") == Some("DEHHAL03"))
            .unwrap();
        assert_eq!(tag(harbour, "freemapper:nutzart"), Some("Hafenbecken"));
        assert_eq!(tag(harbour, "freemapper:currency"), None);
    }

    #[test]
    fn invalid_parcels_do_not_crash() {
        let f = fixture();
        let (e, n) = f.centre;
        let bowtie = Polygon::new(
            LineString::from(vec![
                (e, n),
                (e + 30.0, n + 30.0),
                (e + 30.0, n),
                (e, n + 30.0),
                (e, n),
            ]),
            vec![],
        );
        let point = Polygon::new(LineString::from(vec![(e, n), (e, n), (e, n)]), vec![]);
        let flat = Polygon::new(
            LineString::from(vec![(e, n), (e + 10.0, n), (e + 20.0, n), (e, n)]),
            vec![],
        );
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        let land_use = [land(
            "DEBBAL01",
            "Stehendes Gewässer",
            None,
            vec![bowtie, point, flat, empty],
        )];
        let (objects, _) = water(&f, &land_use);
        assert!(objects.map.is_empty() && objects.candidates.is_empty());
        assert!(objects.union.0.is_empty());
    }

    #[test]
    fn ni_fixture_pond_is_drawn() {
        use crate::sources::alkis::parse_land_use;
        use crate::sources::alkis_wfs::{parse_gml, test_support::NI_LAND_USE};
        let features = parse_gml(NI_LAND_USE).unwrap();
        let collection =
            serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string();
        let land_use = parse_land_use(&collection).unwrap();
        let pond = land_use
            .iter()
            .find(|l| l.nutzart == "Stehendes Gewässer")
            .expect("the fixture has a pond");
        let centre = pond.footprint.centroid().unwrap();
        let bounds = pond.footprint.bounding_rect().unwrap();
        let georef = Georeferencing::new_in(
            Crs::Utm32,
            Crs::Utm32
                .inverse(ProjectedPoint {
                    easting: centre.x(),
                    northing: centre.y(),
                })
                .unwrap(),
            4000,
            "2026-11-08".parse().unwrap(),
        )
        .unwrap();
        let clip = BoundingBox {
            min_e: bounds.min().x - 100.0,
            min_n: bounds.min().y - 100.0,
            max_e: bounds.max().x + 100.0,
            max_n: bounds.max().y + 100.0,
        };
        let set = MapKind::Sprint.symbol_set();
        let objects = water_objects(&land_use, &clip, &georef, &set).unwrap();
        let fills = with_code(&objects.map, &set, "301.1");
        let banks = with_code(&objects.map, &set, "301.6");
        assert_eq!(fills.len(), 1);
        assert_eq!(
            tag(fills[0], "freemapper:nutzart"),
            Some("Stehendes Gewässer / Teich")
        );
        assert_eq!(
            tag(fills[0], "freemapper:source_id"),
            Some(pond.oid.as_str())
        );
        assert_eq!(tag(fills[0], "freemapper:currency"), Some("2011-09-16"));
        // The pond has an island.
        assert_eq!(rings(fills[0]), 2);
        assert_eq!(banks.len(), 2);
        assert!(banks.iter().all(|b| is_closed(b)));
        assert!(objects.candidates.is_empty());
    }

    /// A rectangle of OSM water (301) with its south-west corner at `(e, n)`.
    fn osm_water(g: &Georeferencing, set: &SymbolSet, e: f64, n: f64, w: f64, h: f64) -> MapObject {
        let pts: Vec<(i32, i32)> = ring(e, n, w, h)
            .coords()
            .map(|c| {
                let m = g
                    .projected_to_map(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .unwrap();
                (m.x, m.y)
            })
            .collect();
        MapObject {
            symbol_id: set.id_for_code("301").unwrap(),
            kind: MapObjectKind::Path,
            coords: closed_ring(&pts),
            tags: vec![("freemapper:source".into(), "osm".into())],
        }
    }

    #[test]
    fn osm_water_inside_alkis_water_is_dropped() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        // ALKIS lake: 100 x 100 m, x and y from -50 to 50 m around the centre.
        let union = MultiPolygon(vec![Polygon::new(
            ring(e - 50.0, n - 50.0, 100.0, 100.0),
            vec![],
        )]);
        let inside = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        // Three fifths in: a duplicate.
        let half = osm_water(&f.georef, &set, e + 38.0, n - 10.0, 20.0, 20.0);
        // Only a quarter in.
        let quarter = osm_water(&f.georef, &set, e + 40.0, n + 40.0, 20.0, 20.0);
        let far = osm_water(&f.georef, &set, e - 120.0, n - 120.0, 20.0, 20.0);
        let mut bridge = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        bridge.symbol_id = set.id_for_code("501.5").unwrap();
        let mut alkis = osm_water(&f.georef, &set, e - 10.0, n - 10.0, 20.0, 20.0);
        alkis.tags = vec![("freemapper:source".into(), "alkis-nutzung".into())];
        let kept = drop_covered_water(
            vec![
                inside,
                half,
                quarter.clone(),
                far.clone(),
                bridge.clone(),
                alkis.clone(),
            ],
            &union,
            &f.clip,
            &f.georef,
            &set,
        );
        assert_eq!(kept, vec![quarter, far, bridge, alkis]);
    }

    #[test]
    fn osm_water_is_measured_inside_the_clip_window() {
        let f = fixture();
        let set = MapKind::Sprint.symbol_set();
        let (e, n) = f.centre;
        // The OSM lake lies mostly outside the clip (x from 100 to 300 m); the window ends at
        // 150 m, where the drawn ALKIS water (clipped to it) covers the whole visible part.
        let union = MultiPolygon(vec![Polygon::new(
            ring(e + 100.0, n - 50.0, 50.0, 100.0),
            vec![],
        )]);
        let lake = osm_water(&f.georef, &set, e + 100.0, n - 20.0, 200.0, 40.0);
        let kept = drop_covered_water(vec![lake], &union, &f.clip, &f.georef, &set);
        assert!(kept.is_empty());
    }
}
