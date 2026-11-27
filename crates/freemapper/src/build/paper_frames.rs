//! The paper frame layer (`templates/paper-frame.omap`): one closed line per paper frame of the
//! project, the full sheet at the map's scale, centred on the frame's centre.

//!
//! The rectangle is axis-aligned in map coordinates, as the sheet lies on the map when it is
//! printed, so the middle of its bounding box is the centre and its sides are the paper size
//! (times the frame's scale over the map's). The centre is projected with the CRS of the map's
//! georeferencing, which is the project's.

use crate::crs::LatLon;
use crate::georef::Georeferencing;
use crate::omap::symbol_set::PAPER_FRAME_CODE;
use crate::omap::{MapDocument, MapObject, MapObjectKind, MapPart, closed_ring};
use crate::paper::ground_size_m;
use crate::project::PaperFrame;

// Tag keys and values of the frame objects: reserved words of the suite contract (core ADR 0046),
// plain words a reader takes as they are.
use freemapper_contract::tags::{ORIENTATION_KEY, PAPER_KEY, SOURCE_KEY, SOURCE_PAPER_FRAME};
/// The name OOM shows in the tag list, so that the frames can be told apart. It means nothing
/// to a reader.
const NAME_KEY: &str = "freemapper:name";

/// The part of the template map that holds the frames.
pub const PAPER_FRAMES_PART: &str = "paper frames";

/// The template map of the paper frames: one part [`PAPER_FRAMES_PART`] with one closed line
/// per frame, in the order of `frames` (two equal frames give two lines). `scale_denominator`
/// is the scale the sheets are printed at, `georef` that of the map. `template_base` gives the
/// georeferencing and the symbol set, as for the candidate layers; the helper line symbol is
/// added to this document's own copy of the symbol set only. A frame whose sheet lies beyond
/// the coordinate range of an OOM map (some kilometres from the map) is left out: OOM cannot
/// hold it.
pub fn paper_frame_document(
    frames: &[PaperFrame],
    scale_denominator: u32,
    georef: &Georeferencing,
    template_base: &MapDocument,
) -> MapDocument {
    let symbol_set = template_base.symbol_set.with_paper_frame_line();
    let symbol_id = symbol_set
        .id_for_code(PAPER_FRAME_CODE)
        .expect("the paper frame line was just added");
    let objects = frames
        .iter()
        .filter_map(|frame| frame_object(frame, scale_denominator, georef, symbol_id))
        .collect();
    MapDocument {
        parts: vec![MapPart {
            name: PAPER_FRAMES_PART.to_string(),
            objects,
        }],
        symbol_set,
        ..template_base.clone()
    }
}

/// The closed line of `frame`, or `None` when it does not fit into OOM map coordinates.
fn frame_object(
    frame: &PaperFrame,
    scale_denominator: u32,
    georef: &Georeferencing,
    symbol_id: u32,
) -> Option<MapObject> {
    let centre = georef
        .geographic_to_map(LatLon {
            lat: frame.lat,
            lon: frame.lon,
        })
        .ok()?;
    // The ground the sheet covers, on the map in micrometres (1 m at 1:1000 is 1 mm).
    let (width_m, height_m) = ground_size_m(frame.format, frame.orientation, scale_denominator);
    let to_um = |m: f64| m * 1e6 / f64::from(georef.scale_denominator);
    let (half_w, half_h) = (to_um(width_m) / 2.0, to_um(height_m) / 2.0);
    let at = |c: i32, d: f64| {
        let v = (f64::from(c) + d).round();
        (v.abs() <= f64::from(i32::MAX)).then_some(v as i32)
    };
    let (x0, x1) = (at(centre.x, -half_w)?, at(centre.x, half_w)?);
    let (y0, y1) = (at(centre.y, -half_h)?, at(centre.y, half_h)?);
    let (paper, orientation) = (frame.format.name(), frame.orientation.name());
    Some(MapObject {
        symbol_id,
        kind: MapObjectKind::Path,
        coords: closed_ring(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)]),
        tags: vec![
            (SOURCE_KEY.to_string(), SOURCE_PAPER_FRAME.to_string()),
            (PAPER_KEY.to_string(), paper.to_string()),
            (ORIENTATION_KEY.to_string(), orientation.to_string()),
            (NAME_KEY.to_string(), format!("{paper} {orientation}")),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::{Crs, LatLon};
    use crate::omap::coord::FLAG_CLOSE;
    use crate::omap::{MapCoord, MapObject, MapObjectKind, MapPart, SymbolSet};
    use crate::paper::{Orientation, PaperFormat};

    const SCALE: u32 = 4000;

    fn centre() -> LatLon {
        LatLon {
            lat: 52.5401,
            lon: 13.8442,
        }
    }

    fn base(crs: Crs) -> MapDocument {
        let georef =
            Georeferencing::new_in(crs, centre(), SCALE, "2026-11-27".parse().unwrap()).unwrap();
        MapDocument {
            notes: String::new(),
            georef,
            symbol_set: SymbolSet::issprom_2019().with_sheet_symbols(),
            parts: vec![MapPart {
                name: "candidates".to_string(),
                objects: Vec::new(),
            }],
            templates: Vec::new(),
        }
    }

    fn frame(format: PaperFormat, orientation: Orientation, at: LatLon) -> PaperFrame {
        PaperFrame {
            format,
            orientation,
            lon: at.lon,
            lat: at.lat,
        }
    }

    fn document(frames: &[PaperFrame], base: &MapDocument) -> MapDocument {
        paper_frame_document(frames, SCALE, &base.georef, base)
    }

    fn objects(doc: &MapDocument) -> Vec<&MapObject> {
        doc.parts.iter().flat_map(|p| p.objects.iter()).collect()
    }

    /// The frame objects: those tagged as paper frames.
    fn frames(doc: &MapDocument) -> Vec<&MapObject> {
        objects(doc)
            .into_iter()
            .filter(|o| {
                o.tags
                    .iter()
                    .any(|(k, v)| k == "freemapper:source" && v == "paper-frame")
            })
            .collect()
    }

    /// (min x, min y, max x, max y) in map millimetres.
    fn bbox_mm(o: &MapObject) -> (f64, f64, f64, f64) {
        let xs = o.coords.iter().map(|c| f64::from(c.x) / 1000.0);
        let ys = o.coords.iter().map(|c| f64::from(c.y) / 1000.0);
        (
            xs.clone().fold(f64::INFINITY, f64::min),
            ys.clone().fold(f64::INFINITY, f64::min),
            xs.fold(f64::NEG_INFINITY, f64::max),
            ys.fold(f64::NEG_INFINITY, f64::max),
        )
    }

    fn tag<'a>(o: &'a MapObject, key: &str) -> Option<&'a str> {
        o.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// `p` moved by `de`, `dn` metres in the projected CRS of `crs`.
    fn moved(crs: Crs, p: LatLon, de: f64, dn: f64) -> LatLon {
        let mut q = crs.forward(p);
        q.easting += de;
        q.northing += dn;
        crs.inverse(q).unwrap()
    }

    #[test]
    fn a_frame_is_a_closed_path_of_the_paper_size_on_the_map() {
        let base = base(Crs::Utm33);
        let doc = document(
            &[frame(PaperFormat::A4, Orientation::Portrait, centre())],
            &base,
        );
        assert_eq!(doc.parts.len(), 1);
        assert_eq!(doc.parts[0].name, "paper frames");
        let found = frames(&doc);
        assert_eq!(found.len(), 1);
        let o = found[0];
        assert_eq!(o.kind, MapObjectKind::Path);
        // Four corners and the first again, closed.
        assert_eq!(o.coords.len(), 5);
        assert_eq!(
            (o.coords[0].x, o.coords[0].y),
            (o.coords[4].x, o.coords[4].y)
        );
        assert_ne!(o.coords[4].flags & FLAG_CLOSE, 0);
        let (x0, y0, x1, y1) = bbox_mm(o);
        assert!(((x1 - x0) - 210.0).abs() <= 0.01, "width {}", x1 - x0);
        assert!(((y1 - y0) - 297.0).abs() <= 0.01, "height {}", y1 - y0);
        // The frame is drawn in the paper frame symbol of this document's own symbol set.
        let line = doc.symbol_set.id_for_code(PAPER_FRAME_CODE).unwrap();
        assert_eq!(o.symbol_id, line);
        assert_eq!(doc.georef, base.georef);
    }

    #[test]
    fn landscape_lies_on_its_long_side_and_another_scale_scales_the_sheet() {
        let base = base(Crs::Utm33);
        let a6 = frame(PaperFormat::A6, Orientation::Landscape, centre());
        let doc = document(std::slice::from_ref(&a6), &base);
        let (x0, y0, x1, y1) = bbox_mm(frames(&doc)[0]);
        assert!(((x1 - x0) - 148.0).abs() <= 0.01);
        assert!(((y1 - y0) - 105.0).abs() <= 0.01);
        // A frame for 1:8000 on a 1:4000 map covers twice the ground, so twice the map size.
        let doc = paper_frame_document(&[a6], 2 * SCALE, &base.georef, &base);
        let (x0, y0, x1, y1) = bbox_mm(frames(&doc)[0]);
        assert!(((x1 - x0) - 296.0).abs() <= 0.01);
        assert!(((y1 - y0) - 210.0).abs() <= 0.01);
    }

    #[test]
    fn the_tags_are_plain_strings() {
        let base = base(Crs::Utm33);
        let doc = document(
            &[
                frame(PaperFormat::A4, Orientation::Portrait, centre()),
                frame(PaperFormat::A2, Orientation::Landscape, centre()),
            ],
            &base,
        );
        let found = frames(&doc);
        assert_eq!(found.len(), 2);
        assert_eq!(tag(found[0], "freemapper:source"), Some("paper-frame"));
        assert_eq!(tag(found[0], "freemapper:paper"), Some("A4"));
        assert_eq!(tag(found[0], "freemapper:orientation"), Some("portrait"));
        assert_eq!(tag(found[1], "freemapper:paper"), Some("A2"));
        assert_eq!(tag(found[1], "freemapper:orientation"), Some("landscape"));
        // Every object of the document is a frame: a reader that takes the tagged objects
        // finds them all and nothing else.
        assert_eq!(objects(&doc).len(), 2);
    }

    #[test]
    fn a_frame_beyond_the_area_is_kept() {
        let base = base(Crs::Utm33);
        // 3 km east and 2 km south of the map's reference point, far off any area around it.
        let far = moved(Crs::Utm33, centre(), 3000.0, -2000.0);
        let doc = document(&[frame(PaperFormat::A3, Orientation::Portrait, far)], &base);
        let found = frames(&doc);
        assert_eq!(found.len(), 1);
        let (x0, y0, x1, y1) = bbox_mm(found[0]);
        assert!(((x1 - x0) - 297.0).abs() <= 0.01);
        assert!(((y1 - y0) - 420.0).abs() <= 0.01);
        // About 750 mm east and 500 mm south of the reference point (the map is turned by
        // the grivation, a few degrees).
        let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        assert!(cx > 600.0 && cy > 300.0, "{cx} {cy}");
    }

    #[test]
    fn two_equal_frames_give_two_objects() {
        let base = base(Crs::Utm33);
        let a5 = frame(PaperFormat::A5, Orientation::Landscape, centre());
        let doc = document(&[a5.clone(), a5], &base);
        let found = frames(&doc);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], found[1]);
        // No frames, no objects; the part stays.
        let empty = document(&[], &base);
        assert!(objects(&empty).is_empty());
    }

    #[test]
    fn utm32_and_utm33_projects_place_the_same_centre_at_the_same_lat_lon() {
        let at = moved(Crs::Utm33, centre(), -400.0, 250.0);
        for crs in [Crs::Utm32, Crs::Utm33] {
            let base = base(crs);
            let doc = document(&[frame(PaperFormat::A4, Orientation::Landscape, at)], &base);
            let (x0, y0, x1, y1) = bbox_mm(frames(&doc)[0]);
            let middle = MapCoord::new(
                ((x0 + x1) / 2.0 * 1000.0).round() as i32,
                ((y0 + y1) / 2.0 * 1000.0).round() as i32,
            );
            let back = crs.inverse(base.georef.map_to_projected(middle)).unwrap();
            // Metres between `back` and `at`, in the projected CRS of the project.
            let (p, q) = (crs.forward(back), crs.forward(at));
            let off = (p.easting - q.easting).hypot(p.northing - q.northing);
            assert!(off <= 1.0, "{crs:?}: {off} m");
        }
    }

    #[test]
    fn the_helper_symbol_is_only_in_the_frame_document() {
        let base = base(Crs::Utm33);
        assert_eq!(base.symbol_set.id_for_code(PAPER_FRAME_CODE), None);
        let doc = document(
            &[frame(PaperFormat::A4, Orientation::Portrait, centre())],
            &base,
        );
        assert!(doc.symbol_set.id_for_code(PAPER_FRAME_CODE).is_some());
        // The symbols of the map keep their ids in the frame document.
        for code in ["501.1", "521", "991", "994"] {
            assert_eq!(
                doc.symbol_set.id_for_code(code),
                base.symbol_set.id_for_code(code),
                "{code}"
            );
        }
    }
}
