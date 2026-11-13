//! Magnetic north lines across the map content and the peripheral information below it.

use crate::crs::{BoundingBox, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::measure::PaperMm;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};

/// ISSprOM 601 magnetic north line.
const NORTH_LINE_CODE: &str = "601";
const TITLE_CODE: &str = "991";
const TEXT_CODE: &str = "992";
const SMALL_TEXT_CODE: &str = "993";
/// Tag key of the role of a sheet object. Every sheet object has `freemapper:source=sheet`
/// (the contract with freemapper-print and freemapper-courseplan); the scale bar objects add
/// `freemapper:sheet=scale-bar`, which the PDF export uses to leave them out unless asked.
const SHEET_ROLE_KEY: &str = freemapper_contract::tags::ROLE_KEY;
const SCALE_BAR_ROLE: &str = freemapper_contract::tags::ROLE_SCALE_BAR;
const BAR_LINE_CODE: &str = "994";
/// Longest bar scale on paper; the bar is the longest round ground distance that fits.
const BAR_MAX_MM: PaperMm = PaperMm(60.0);
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
    pub contour_interval_m: f64,
    /// Attribution lines, the freemapper credit line last.
    pub attribution: Vec<String>,
}

fn sheet_tags() -> Vec<(String, String)> {
    source_tags(freemapper_contract::tags::SOURCE_SHEET)
}

/// Whether a sheet object belongs to the scale bar (the graphic bar with ticks and labels, not
/// the scale number in the title block).
pub fn is_scale_bar(object: &MapObject) -> bool {
    object
        .tags
        .iter()
        .any(|(k, v)| k == SHEET_ROLE_KEY && v == SCALE_BAR_ROLE)
}

fn source_tags(source: &str) -> Vec<(String, String)> {
    vec![(
        freemapper_contract::tags::SOURCE_KEY.to_string(),
        source.to_string(),
    )]
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

/// Vertical magnetic north lines `spacing` apart across `content`, one through x = 0.
/// They are map content, tagged `freemapper:source=north-lines`: the mapper breaks them at
/// small objects.
pub fn north_lines(
    content: &PaperRect,
    spacing: PaperMm,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    let id = symbol(symbol_set, NORTH_LINE_CODE);
    let spacing = spacing.map_units();
    let first = (f64::from(content.min_x) / spacing).ceil() as i64;
    let last = (f64::from(content.max_x) / spacing).floor() as i64;
    (first..=last)
        .map(|k| {
            let x = (k as f64 * spacing).round() as i32;
            MapObject {
                tags: source_tags(freemapper_contract::tags::SOURCE_NORTH_LINES),
                ..line(id, (x, content.min_y), (x, content.max_y))
            }
        })
        .collect()
}

/// Ground distance for a bar scale label: whole metres without decimals, else a decimal comma.
fn metres_label(metres: f64) -> String {
    if metres.fract() == 0.0 {
        format!("{metres:.0}")
    } else {
        format!("{metres}").replace('.', ",")
    }
}

/// Longest ground distance of 1, 2 or 5 × 10ⁿ m whose bar is at most [`BAR_MAX_MM`] on paper.
fn bar_length_m(scale_denominator: u32) -> f64 {
    let max_m = BAR_MAX_MM.metres(scale_denominator);
    let decade = 10f64.powi(max_m.log10().floor() as i32);
    [5.0, 2.0, 1.0]
        .into_iter()
        .map(|step| step * decade)
        .find(|length| *length <= max_m)
        .unwrap_or(decade)
}

/// Bar scale with ticks at 0, ½ and the end, labelled in ground metres, starting at `origin`.
fn bar_scale(origin: (i32, i32), scale_denominator: u32, bar: u32, small: u32) -> Vec<MapObject> {
    let (x0, y) = origin;
    let length_m = bar_length_m(scale_denominator);
    let PaperMm(length_mm) = PaperMm::of_metres(length_m, scale_denominator);
    let bar_end = x0 + mm(length_mm);
    let mut objects = vec![line(bar, (x0, y), (bar_end, y))];
    for fraction in [0.0, 0.5, 1.0] {
        let x = x0 + mm(length_mm * fraction);
        objects.push(line(bar, (x, y), (x, y - mm(1.5))));
        let label = metres_label(length_m * fraction);
        objects.push(text(small, (x, y - mm(2.5)), label, ALIGN_CENTRE));
    }
    objects.push(text(
        small,
        (bar_end + mm(1.5), y),
        "m".to_string(),
        ALIGN_LEFT,
    ));
    objects
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
            info.scale_denominator,
            german_number(info.contour_interval_m)
        ),
        ALIGN_LEFT,
    ));

    y += mm(7.0);
    objects.extend(
        bar_scale((x0, y), info.scale_denominator, bar, small)
            .into_iter()
            .map(|o| MapObject {
                tags: {
                    let mut tags = sheet_tags();
                    tags.push((SHEET_ROLE_KEY.to_string(), SCALE_BAR_ROLE.to_string()));
                    tags
                },
                ..o
            }),
    );

    y += mm(6.0);
    for line_text in &info.attribution {
        objects.push(text(small, (x0, y), line_text.clone(), ALIGN_LEFT));
        y += mm(2.5);
    }
    objects
}

/// `2` or `2,5`: a whole number without decimals, else with the German decimal comma.
fn german_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string().replace('.', ",")
    }
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

    fn info(scale_denominator: u32, contour_interval_m: f64) -> PeripheralInfo {
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
        let lines = north_lines(&content, PaperMm(37.5), &set);
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
            assert_eq!(line.tags, source_tags("north-lines"));
        }
    }

    #[test]
    fn north_line_spacing_sprint() {
        let content = rect_mm(-10.0, -10.0, 40.0, 10.0);
        let sprint = SymbolSet::issprom_2019().with_sheet_symbols();
        let xs: Vec<i32> = north_lines(&content, PaperMm(30.0), &sprint)
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
            for o in info_block(&content, &info(1000, 1.0), &school_set()) {
                assert!(o.coords.iter().all(|c| c.y > content.max_y), "{o:?}");
            }
        }
    }

    #[test]
    fn info_block_texts_schoolyard() {
        let set = school_set();
        let objects = info_block(&rect_mm(-50.0, -50.0, 50.0, 50.0), &info(1000, 1.0), &set);
        assert_eq!(
            texts(&objects),
            [
                "Clara-Zetkin-Schule",
                "Maßstab 1:1000 · Äquidistanz 1 m",
                "0",
                "25",
                "50",
                "m",
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
        let objects = info_block(&rect_mm(-50.0, -50.0, 50.0, 50.0), &info(4000, 2.0), &set);
        let all = texts(&objects);
        assert_eq!(
            &all[1..6],
            ["Maßstab 1:4000 · Äquidistanz 2 m", "0", "100", "200", "m"]
        );
        let flat = info_block(&rect_mm(-50.0, -50.0, 50.0, 50.0), &info(10000, 2.5), &set);
        assert_eq!(texts(&flat)[1], "Maßstab 1:10000 · Äquidistanz 2,5 m");
        let bar = set.id_for_code("994");
        let longest = objects
            .iter()
            .filter(|o| Some(o.symbol_id) == bar)
            .map(|o| (o.coords[1].x - o.coords[0].x).abs())
            .max();
        assert_eq!(longest, Some(50_000));
    }

    #[test]
    fn bar_scale_uses_round_lengths() {
        assert_eq!(bar_length_m(1000), 50.0);
        assert_eq!(bar_length_m(4000), 200.0);
        assert_eq!(bar_length_m(1500), 50.0);
        assert_eq!(bar_length_m(2500), 100.0);
        assert_eq!(metres_label(2.5), "2,5");
        assert_eq!(metres_label(25.0), "25");
    }

    #[test]
    fn bar_unit_is_its_own_text() {
        let set = school_set();
        let content = rect_mm(-50.0, -50.0, 50.0, 50.0);
        let objects = info_block(&content, &info(1000, 1.0), &set);
        let label = |t: &str| {
            objects
                .iter()
                .find(|o| matches!(&o.kind, MapObjectKind::Text { text, .. } if text == t))
                .unwrap()
        };
        let end = label("50");
        assert!(matches!(end.kind, MapObjectKind::Text { h_align: 1, .. }));
        assert_eq!(end.coords[0].x, content.min_x + 50_000);
        let unit = label("m");
        assert!(matches!(unit.kind, MapObjectKind::Text { h_align: 0, .. }));
        assert!(unit.coords[0].x > content.min_x + 50_000);
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
