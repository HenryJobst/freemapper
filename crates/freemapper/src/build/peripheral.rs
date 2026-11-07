//! Peripheral information: magnetic north lines and the info block below the map content.

use crate::crs::{BoundingBox, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};

/// ISSprOM 601 magnetic north line.
const NORTH_LINE_CODE: &str = "601";
const TITLE_CODE: &str = "991";
const TEXT_CODE: &str = "992";
const SMALL_TEXT_CODE: &str = "993";
const BAR_LINE_CODE: &str = "994";
/// Length of the bar scale on paper.
const BAR_LENGTH_MM: f64 = 50.0;
/// Text alignments as in OOM.
const ALIGN_LEFT: u8 = 0;
const ALIGN_CENTRE: u8 = 1;
const ALIGN_BASELINE: u8 = 0;

/// Axis-parallel rectangle on paper in µm (map coordinates, y pointing south).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaperRect {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl PaperRect {
    /// The paper rectangle enclosing `bbox` (EPSG:25833), which is rotated on the map by the
    /// grivation.
    pub fn of_bbox(bbox: &BoundingBox, georef: &Georeferencing) -> Result<Self, GeorefError> {
        let corners = [
            (bbox.min_e, bbox.min_n),
            (bbox.max_e, bbox.min_n),
            (bbox.max_e, bbox.max_n),
            (bbox.min_e, bbox.max_n),
        ]
        .map(|(easting, northing)| georef.projected_to_map(ProjectedPoint { easting, northing }));
        let mut rect: Option<Self> = None;
        for corner in corners {
            let c = corner?;
            let r = rect.get_or_insert(Self {
                min_x: c.x,
                min_y: c.y,
                max_x: c.x,
                max_y: c.y,
            });
            r.min_x = r.min_x.min(c.x);
            r.min_y = r.min_y.min(c.y);
            r.max_x = r.max_x.max(c.x);
            r.max_y = r.max_y.max(c.y);
        }
        Ok(rect.expect("four corners"))
    }
}

/// Texts of the info block.
#[derive(Debug, Clone, PartialEq)]
pub struct PeripheralInfo {
    pub title: String,
    pub scale_denominator: u32,
    pub contour_interval_m: u32,
    /// Attribution lines, the freemapper credit line last.
    pub attribution: Vec<String>,
}

fn sheet_tags() -> Vec<(String, String)> {
    vec![("freemapper:source".to_string(), "sheet".to_string())]
}

fn symbol(symbol_set: &SymbolSet, code: &str) -> u32 {
    symbol_set
        .id_for_code(code)
        .unwrap_or_else(|| panic!("symbol set defines {code}"))
}

fn line(symbol_id: u32, from: (i32, i32), to: (i32, i32)) -> MapObject {
    MapObject {
        symbol_id,
        kind: MapObjectKind::Path,
        coords: vec![MapCoord::new(from.0, from.1), MapCoord::new(to.0, to.1)],
        tags: sheet_tags(),
    }
}

fn text(symbol_id: u32, at: (i32, i32), text: String, h_align: u8) -> MapObject {
    MapObject {
        symbol_id,
        kind: MapObjectKind::Text {
            text,
            h_align,
            v_align: ALIGN_BASELINE,
        },
        coords: vec![MapCoord::new(at.0, at.1)],
        tags: sheet_tags(),
    }
}

fn mm(v: f64) -> i32 {
    (v * 1000.0).round() as i32
}

/// Vertical magnetic north lines `spacing_mm` apart across `content`, one through x = 0.
pub fn north_lines(content: &PaperRect, spacing_mm: f64, symbol_set: &SymbolSet) -> Vec<MapObject> {
    let id = symbol(symbol_set, NORTH_LINE_CODE);
    let spacing = spacing_mm * 1000.0;
    let first = (f64::from(content.min_x) / spacing).ceil() as i64;
    let last = (f64::from(content.max_x) / spacing).floor() as i64;
    (first..=last)
        .map(|k| {
            let x = (k as f64 * spacing).round() as i32;
            line(id, (x, content.min_y), (x, content.max_y))
        })
        .collect()
}

/// Ground distance for a bar scale label: whole metres without decimals.
fn metres_label(metres: f64) -> String {
    if metres.fract() == 0.0 {
        format!("{metres:.0}")
    } else {
        format!("{metres}")
    }
}

/// Title, scale and contour interval, bar scale and attribution below `content`.
pub fn info_block(
    content: &PaperRect,
    info: &PeripheralInfo,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    let (title, text_id, small) = (
        symbol(symbol_set, TITLE_CODE),
        symbol(symbol_set, TEXT_CODE),
        symbol(symbol_set, SMALL_TEXT_CODE),
    );
    let bar = symbol(symbol_set, BAR_LINE_CODE);
    let x0 = content.min_x;
    let mut y = content.max_y + mm(12.0);
    let mut objects = vec![text(title, (x0, y), info.title.clone(), ALIGN_LEFT)];

    y += mm(6.0);
    objects.push(text(
        text_id,
        (x0, y),
        format!(
            "Maßstab 1:{} · Äquidistanz {} m",
            info.scale_denominator, info.contour_interval_m
        ),
        ALIGN_LEFT,
    ));

    y += mm(7.0);
    let bar_end = x0 + mm(BAR_LENGTH_MM);
    objects.push(line(bar, (x0, y), (bar_end, y)));
    let bar_metres = BAR_LENGTH_MM * f64::from(info.scale_denominator) / 1000.0;
    for (fraction, unit) in [(0.0, ""), (0.5, ""), (1.0, " m")] {
        let x = x0 + mm(BAR_LENGTH_MM * fraction);
        objects.push(line(bar, (x, y), (x, y - mm(1.5))));
        let label = format!("{}{unit}", metres_label(bar_metres * fraction));
        objects.push(text(small, (x, y - mm(2.5)), label, ALIGN_CENTRE));
    }

    y += mm(6.0);
    for line_text in &info.attribution {
        objects.push(text(small, (x0, y), line_text.clone(), ALIGN_LEFT));
        y += mm(2.5);
    }
    objects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;

    fn rect_mm(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> PaperRect {
        let um = |mm: f64| (mm * 1000.0) as i32;
        PaperRect {
            min_x: um(min_x),
            min_y: um(min_y),
            max_x: um(max_x),
            max_y: um(max_y),
        }
    }

    fn school_set() -> SymbolSet {
        SymbolSet::school_2019().with_sheet_symbols()
    }

    fn info(scale_denominator: u32, contour_interval_m: u32) -> PeripheralInfo {
        PeripheralInfo {
            title: "Clara-Zetkin-Schule".to_string(),
            scale_denominator,
            contour_interval_m,
            attribution: vec!["© OSM".to_string(), "Created with freemapper".to_string()],
        }
    }

    fn texts(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .filter_map(|o| match &o.kind {
                MapObjectKind::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn north_lines_cover_content() {
        let content = rect_mm(-100.0, -80.0, 120.0, 60.0);
        let set = school_set();
        let lines = north_lines(&content, 37.5, &set);
        let xs: Vec<i32> = lines.iter().map(|l| l.coords[0].x).collect();
        assert_eq!(xs, [-75_000, -37_500, 0, 37_500, 75_000, 112_500]);
        for line in &lines {
            assert_eq!(Some(line.symbol_id), set.id_for_code("601"));
            assert_eq!(line.kind, MapObjectKind::Path);
            assert_eq!(line.coords.len(), 2);
            assert_eq!(
                (line.coords[0].y, line.coords[1].y),
                (content.min_y, content.max_y)
            );
            assert_eq!(line.coords[0].x, line.coords[1].x);
        }
    }

    #[test]
    fn north_line_spacing_sprint() {
        let content = rect_mm(-10.0, -10.0, 40.0, 10.0);
        let xs: Vec<i32> = north_lines(&content, 30.0, &school_set())
            .iter()
            .map(|l| l.coords[0].x)
            .collect();
        assert_eq!(xs, [0, 30_000]);
    }

    #[test]
    fn info_block_sits_below_content() {
        for content in [
            rect_mm(-5.0, -5.0, 5.0, 5.0),
            rect_mm(-400.0, -300.0, 400.0, 300.0),
        ] {
            for o in info_block(&content, &info(1000, 1), &school_set()) {
                assert!(o.coords.iter().all(|c| c.y > content.max_y), "{o:?}");
            }
        }
    }

    #[test]
    fn info_block_texts_schoolyard() {
        let set = school_set();
        let objects = info_block(&rect_mm(-50.0, -50.0, 50.0, 50.0), &info(1000, 1), &set);
        assert_eq!(
            texts(&objects),
            [
                "Clara-Zetkin-Schule",
                "Maßstab 1:1000 · Äquidistanz 1 m",
                "0",
                "25",
                "50 m",
                "© OSM",
                "Created with freemapper"
            ]
        );
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("991"));
        assert_eq!(objects[0].coords[0].x, -50_000);
    }

    #[test]
    fn bar_scale_sprint() {
        let set = school_set();
        let objects = info_block(&rect_mm(-50.0, -50.0, 50.0, 50.0), &info(4000, 2), &set);
        let all = texts(&objects);
        assert_eq!(
            &all[1..5],
            ["Maßstab 1:4000 · Äquidistanz 2 m", "0", "100", "200 m"]
        );
        let bar = set.id_for_code("994");
        let longest = objects
            .iter()
            .filter(|o| Some(o.symbol_id) == bar)
            .map(|o| (o.coords[1].x - o.coords[0].x).abs())
            .max();
        assert_eq!(longest, Some(50_000));
    }

    #[test]
    fn paper_rect_of_bbox() {
        let date = "2026-11-07".parse().unwrap();
        let georef = Georeferencing::new(
            LatLon {
                lat: 52.5401,
                lon: 13.8442,
            },
            1000,
            date,
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let bbox = BoundingBox {
            min_e: e - 100.0,
            min_n: n - 50.0,
            max_e: e + 100.0,
            max_n: n + 50.0,
        };
        let rect = PaperRect::of_bbox(&bbox, &georef).unwrap();
        assert!(
            rect.min_x < 0 && rect.max_x > 0 && rect.min_y < 0 && rect.max_y > 0,
            "{rect:?}"
        );
        // The rotated box is wider than 200 mm on paper (factor ≈ 1, grivation 6°).
        assert!(rect.max_x - rect.min_x > 200_000, "{rect:?}");
    }
}
