//! A freemapper map document as an OCAD 12 file (`docs/ocad/ocd12-reference.md`).
//!
//! OCAD has no grid scale factor, so every coordinate is re-projected: paper → EPSG:25833 with the
//! freemapper georeferencing, then back to OCAD paper with OCAD's own transformation.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::file::{
    OcdBuildError, OcdBuilder, OcdCoord, OcdObject, OcdObjectKind, OcdSymbol, TextAlign, X_CURVE_1,
    X_CURVE_2, Y_HOLE,
};
use super::symbols::{Numbers, Placeholder, PlaceholderKind, ascii, colours, placeholders};
use crate::crs::ProjectedPoint;
use crate::georef::Georeferencing;
use crate::omap::coord::{FLAG_CURVE_START, FLAG_HOLE};
use crate::omap::{MapCoord, MapDocument, MapObject, MapObjectKind};
use crate::project::MapKind;

/// OCAD's Orienteering edition draws within ±2 m of the paper origin (0.01 mm).
const MAX_COORD: i32 = 200_000;
/// Screen grid in the terrain (m).
const GRID_M: f64 = 100.0;
/// Text box estimate in em: width per character, ascent, descent, line distance.
const CHAR_WIDTH_EM: f64 = 0.6;
const ASCENT_EM: f64 = 0.9;
const DESCENT_EM: f64 = 0.25;
const LINE_EM: f64 = 1.2;

/// A world file: pixel size and rotation terms (A, D, B, E) and the centre of the upper-left
/// pixel (C, F), in EPSG:25833.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldFile {
    pub a: f64,
    pub d: f64,
    pub b: f64,
    pub e: f64,
    pub c: f64,
    pub f: f64,
}

impl WorldFile {
    pub fn parse(text: &str) -> Option<Self> {
        let v: Vec<f64> = text
            .split_whitespace()
            .map(|t| t.parse().ok())
            .collect::<Option<_>>()?;
        (v.len() == 6).then(|| Self {
            a: v[0],
            d: v[1],
            b: v[2],
            e: v[3],
            c: v[4],
            f: v[5],
        })
    }
}

/// A background map of the OCAD file, next to it.
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    Image {
        file_name: String,
        width_px: u32,
        height_px: u32,
        world: WorldFile,
        visible: bool,
        opacity: f32,
    },
    /// Another OCAD map with the same georeferencing (the candidates).
    Map { file_name: String, visible: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OcdReport {
    pub objects: usize,
    /// Objects without a usable geometry or symbol, or outside OCAD's drawing area.
    pub skipped: usize,
    /// Degenerate parts of written objects (line parts, hole rings) that were left out.
    pub dropped_parts: usize,
}

/// OCAD's paper ↔ EPSG:25833 transformation: origin in whole metres, angle = grivation.
#[derive(Debug, Clone, Copy)]
struct OcadGeoref {
    x0: f64,
    y0: f64,
    /// Metres per paper mm.
    s: f64,
    sin: f64,
    cos: f64,
}

impl OcadGeoref {
    fn new(georef: &Georeferencing) -> Self {
        let (sin, cos) = georef.grivation_deg().to_radians().sin_cos();
        Self {
            x0: georef.ref_projected.easting.round(),
            y0: georef.ref_projected.northing.round(),
            s: f64::from(georef.scale_denominator) / 1000.0,
            sin,
            cos,
        }
    }

    /// Paper position in mm (y up) of a projected point.
    fn paper_mm(&self, p: ProjectedPoint) -> (f64, f64) {
        let (de, dn) = (p.easting - self.x0, p.northing - self.y0);
        (
            (self.cos * de - self.sin * dn) / self.s,
            (self.sin * de + self.cos * dn) / self.s,
        )
    }

    /// OCAD coordinate (0.01 mm) of a freemapper paper coordinate; `None` outside the drawing
    /// area.
    fn coord(&self, georef: &Georeferencing, c: MapCoord) -> Option<OcdCoord> {
        let (x, y) = self.paper_mm(georef.map_to_projected(c));
        let (x, y) = ((x * 100.0).round(), (y * 100.0).round());
        let inside = |v: f64| v.abs() <= f64::from(MAX_COORD);
        (inside(x) && inside(y)).then(|| OcdCoord::new(x as i32, y as i32))
    }
}

fn text_align(h_align: u8) -> TextAlign {
    match h_align {
        1 => TextAlign::Centre,
        2 => TextAlign::Right,
        _ => TextAlign::Left,
    }
}

/// The OCAD symbols: one per used OOM symbol and, for texts, per used horizontal alignment.
struct SymbolTable {
    numbers: HashMap<(u32, TextAlign), i32>,
    symbols: Vec<OcdSymbol>,
}

fn symbol_table(
    doc: &MapDocument,
    kind: MapKind,
    placeholders: &HashMap<u32, Placeholder>,
) -> SymbolTable {
    let mut used: BTreeMap<u32, BTreeSet<u8>> = BTreeMap::new();
    for object in doc.parts.iter().flat_map(|p| &p.objects) {
        let align = match object.kind {
            MapObjectKind::Text { h_align, .. } => text_align(h_align) as u8,
            _ => TextAlign::Left as u8,
        };
        used.entry(object.symbol_id).or_default().insert(align);
    }
    let icon_colors: HashMap<i16, u8> = colours(&doc.symbol_set)
        .iter()
        .map(|c| (c.number, c.icon_index()))
        .collect();
    let mut numbers = Numbers::default();
    let mut table = SymbolTable {
        numbers: HashMap::new(),
        symbols: Vec::new(),
    };
    for (id, aligns) in used {
        let Some(placeholder) = placeholders.get(&id) else {
            continue;
        };
        for align in aligns {
            let align = match align {
                1 => TextAlign::Centre,
                2 => TextAlign::Right,
                _ => TextAlign::Left,
            };
            let number = numbers.assign(&placeholder.code, kind);
            table.numbers.insert((id, align), number);
            table.symbols.push(OcdSymbol {
                number,
                name: placeholder.name.chars().take(63).collect(),
                kind: placeholder.kind.ocd_kind(align),
                icon_color: icon_colors
                    .get(&placeholder.kind.color())
                    .copied()
                    .unwrap_or(0),
            });
        }
    }
    table
}

/// The rings (or line parts) of an OOM path: they end at points flagged `FLAG_HOLE`.
fn parts(coords: &[MapCoord]) -> Vec<&[MapCoord]> {
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, c) in coords.iter().enumerate() {
        if c.flags & FLAG_HOLE != 0 {
            parts.push(&coords[start..=i]);
            start = i + 1;
        }
    }
    if start < coords.len() {
        parts.push(&coords[start..]);
    }
    parts
}

/// OCAD coordinates of one ring or line part, with Bézier control point flags.
fn part_coords(
    ocad: &OcadGeoref,
    georef: &Georeferencing,
    part: &[MapCoord],
) -> Option<Vec<OcdCoord>> {
    let mut out: Vec<OcdCoord> = part
        .iter()
        .map(|&c| ocad.coord(georef, c))
        .collect::<Option<_>>()?;
    for (i, c) in part.iter().enumerate() {
        // A curve needs its two control points and its end point.
        if c.flags & FLAG_CURVE_START != 0 && i + 3 < out.len() {
            out[i + 1].xflags |= X_CURVE_1;
            out[i + 2].xflags |= X_CURVE_2;
        }
    }
    Some(out)
}

/// Twice the enclosed area of a ring (shoelace formula), in (0.01 mm)².
fn doubled_area(ring: &[OcdCoord]) -> i64 {
    ring.iter()
        .zip(ring.iter().cycle().skip(1))
        .map(|(a, b)| i64::from(a.x) * i64::from(b.y) - i64::from(b.x) * i64::from(a.y))
        .sum::<i64>()
        .abs()
}

fn distinct(coords: &[OcdCoord]) -> usize {
    coords
        .iter()
        .map(|c| (c.x, c.y))
        .collect::<BTreeSet<_>>()
        .len()
}

/// The OCAD objects of one freemapper object (empty when nothing usable remains) and the number
/// of degenerate parts left out.
fn objects_of(
    object: &MapObject,
    ocad: &OcadGeoref,
    georef: &Georeferencing,
    table: &SymbolTable,
    placeholder: &Placeholder,
) -> (Vec<OcdObject>, usize) {
    let mut dropped = 0;
    let objects = match (&object.kind, &placeholder.kind) {
        (MapObjectKind::Point { rotation_rad }, PlaceholderKind::Point { .. }) => {
            let Some(number) = table.numbers.get(&(object.symbol_id, TextAlign::Left)) else {
                return (Vec::new(), 0);
            };
            let Some(at) = object.coords.first().and_then(|&c| ocad.coord(georef, c)) else {
                return (Vec::new(), 0);
            };
            let angle = (rotation_rad.to_degrees() * 10.0)
                .round()
                .rem_euclid(3600.0) as i16;
            vec![OcdObject {
                symbol: *number,
                kind: OcdObjectKind::Point { angle },
                coords: vec![at],
            }]
        }
        (MapObjectKind::Path, PlaceholderKind::Area { .. }) => {
            let number = table.numbers[&(object.symbol_id, TextAlign::Left)];
            let mut coords = Vec::new();
            for (i, part) in parts(&object.coords).into_iter().enumerate() {
                let Some(mut ring) = part_coords(ocad, georef, part) else {
                    return (Vec::new(), 0);
                };
                if distinct(&ring) < 3 || doubled_area(&ring) == 0 {
                    if i == 0 {
                        return (Vec::new(), 0);
                    }
                    dropped += 1;
                    continue;
                }
                if i > 0 {
                    ring[0].yflags |= Y_HOLE;
                }
                coords.extend(ring);
            }
            vec![OcdObject {
                symbol: number,
                kind: OcdObjectKind::Area,
                coords,
            }]
        }
        (MapObjectKind::Path, PlaceholderKind::Line { .. }) => {
            let number = table.numbers[&(object.symbol_id, TextAlign::Left)];
            let all = parts(&object.coords);
            let lines: Vec<OcdObject> = all
                .iter()
                .filter_map(|part| part_coords(ocad, georef, part))
                .filter(|line| distinct(line) >= 2)
                .map(|coords| OcdObject {
                    symbol: number,
                    kind: OcdObjectKind::Line,
                    coords,
                })
                .collect();
            if !lines.is_empty() {
                dropped += all.len() - lines.len();
            }
            lines
        }
        (MapObjectKind::Text { text, h_align, .. }, PlaceholderKind::Text { size, .. }) => {
            let align = text_align(*h_align);
            let number = table.numbers[&(object.symbol_id, align)];
            let Some(anchor) = object.coords.first().and_then(|&c| ocad.coord(georef, c)) else {
                return (Vec::new(), 0);
            };
            if text.trim().is_empty() {
                return (Vec::new(), 0);
            }
            // Em size in 0.01 mm from points × 10.
            let em = f64::from(*size) / 10.0 * 25.4 / 72.0 * 100.0;
            let lines: Vec<&str> = text.lines().collect();
            let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f64
                * CHAR_WIDTH_EM
                * em;
            let left = match align {
                TextAlign::Left => f64::from(anchor.x),
                TextAlign::Centre => f64::from(anchor.x) - width / 2.0,
                TextAlign::Right => f64::from(anchor.x) - width,
            };
            let top = f64::from(anchor.y) + ASCENT_EM * em;
            let bottom = f64::from(anchor.y)
                - (lines.len().saturating_sub(1)) as f64 * LINE_EM * em
                - DESCENT_EM * em;
            let corner = |x: f64, y: f64| OcdCoord::new(x.round() as i32, y.round() as i32);
            let inside = |v: f64| v.abs() <= f64::from(MAX_COORD);
            if ![left, left + width].into_iter().all(inside)
                || ![top, bottom].into_iter().all(inside)
            {
                return (Vec::new(), 0);
            }
            vec![OcdObject {
                symbol: number,
                kind: OcdObjectKind::Text {
                    angle: 0,
                    text: text.clone(),
                },
                coords: vec![
                    anchor,
                    corner(left, bottom),
                    corner(left + width, bottom),
                    corner(left + width, top),
                    corner(left, top),
                ],
            }]
        }
        _ => Vec::new(),
    };
    (objects, dropped)
}

/// The type 8 string of a background map.
fn background_string(
    background: &Background,
    ocad: &OcadGeoref,
    grivation: f64,
    scale: f64,
) -> String {
    match background {
        Background::Image {
            file_name,
            width_px,
            height_px,
            world,
            visible,
            opacity,
        } => {
            let (w, h) = (f64::from(*width_px), f64::from(*height_px));
            let centre = ProjectedPoint {
                easting: world.c + world.a * (w - 1.0) / 2.0 + world.b * (h - 1.0) / 2.0,
                northing: world.f + world.d * (w - 1.0) / 2.0 + world.e * (h - 1.0) / 2.0,
            };
            let (x, y) = ocad.paper_mm(centre);
            let u = world.a.hypot(world.d) * 1000.0 / scale;
            let v = world.b.hypot(world.e) * 1000.0 / scale;
            let dim = (100.0 * (1.0 - f64::from(*opacity)))
                .round()
                .clamp(0.0, 100.0);
            format!(
                "{}\ts{}\tr1\tu{u:.10}\tv{v:.10}\tx{x:.4}\ty{y:.4}\ta{grivation:.8}\tb{grivation:.8}\td{dim:.0}",
                ascii(file_name),
                u8::from(*visible)
            )
        }
        Background::Map { file_name, visible } => {
            format!("{}\ts{}\tr1", ascii(file_name), u8::from(*visible))
        }
    }
}

/// The OCAD 12 file of `doc`, with `backgrounds` next to it.
/// OCAD's grid and zone code (`grid × 1000 + zone`, grid 8 = ETRS89/UTM, Germany), as in OOM's
/// `ocd_georef_fields.cpp`: zone 32N 8007, zone 33N 8008.
fn ocad_grid_code(crs: crate::crs::Crs) -> u32 {
    match crs {
        crate::crs::Crs::Utm32 => 8007,
        crate::crs::Crs::Utm33 => 8008,
    }
}

pub fn map_to_ocd(
    doc: &MapDocument,
    kind: MapKind,
    backgrounds: &[Background],
) -> Result<(Vec<u8>, OcdReport), OcdBuildError> {
    let georef = &doc.georef;
    let ocad = OcadGeoref::new(georef);
    let scale = f64::from(georef.scale_denominator);
    let placeholders = placeholders(&doc.symbol_set);
    let table = symbol_table(doc, kind, &placeholders);
    let mut builder = OcdBuilder::new();

    let grivation = georef.grivation_deg();
    builder.add_string(
        1039,
        &format!(
            "\tm{scale:.0}\tg{:.4}\tr1\tx{:.0}\ty{:.0}\ta{grivation:.8}\td{GRID_M:.6}\ti{}\tb0.00\tc0.00",
            GRID_M * 1000.0 / scale,
            ocad.x0,
            ocad.y0,
            ocad_grid_code(georef.crs())
        ),
    );
    for colour in colours(&doc.symbol_set) {
        builder.add_string(9, &colour.parameter_string());
    }
    let notes = ascii(&doc.notes)
        .replace("\r\n", "\n")
        .replace('\n', "\r\n");
    builder.add_string(1061, &notes);
    let mut report = OcdReport::default();
    let mut objects = Vec::new();
    for object in doc.parts.iter().flat_map(|p| &p.objects) {
        let (converted, dropped) = placeholders
            .get(&object.symbol_id)
            .map(|p| objects_of(object, &ocad, georef, &table, p))
            .unwrap_or_default();
        if converted.is_empty() {
            report.skipped += 1;
        }
        report.dropped_parts += dropped;
        objects.extend(converted);
    }
    // Only symbols that objects use: a placeholder without objects would only clutter OCAD's list.
    let used: BTreeSet<i32> = objects.iter().map(|o| o.symbol).collect();
    for symbol in table.symbols.iter().filter(|s| used.contains(&s.number)) {
        builder.add_symbol(symbol.clone())?;
    }
    let (mut min, mut max) = ((i32::MAX, i32::MAX), (i32::MIN, i32::MIN));
    for ocd_object in objects {
        for c in &ocd_object.coords {
            min = (min.0.min(c.x), min.1.min(c.y));
            max = (max.0.max(c.x), max.1.max(c.y));
        }
        builder.add_object(ocd_object)?;
        report.objects += 1;
    }
    let (cx, cy) = if report.objects > 0 {
        (
            f64::from(min.0 + max.0) / 200.0,
            f64::from(min.1 + max.1) / 200.0,
        )
    } else {
        (0.0, 0.0)
    };
    builder.add_string(
        1030,
        &format!("\tx{cx:.2}\ty{cy:.2}\tz1.00\tv0\tm50\tt50\tb50\tc50\th0\td0\tk0"),
    );
    for background in backgrounds {
        builder.add_string(8, &background_string(background, &ocad, grivation, scale));
    }
    Ok((builder.finish(), report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::ocd::read::{ReadFile, read};
    use crate::omap::{MapPart, SymbolSet, closed_ring};

    fn georef() -> Georeferencing {
        let date = "2026-11-07".parse().unwrap();
        Georeferencing::new(
            LatLon {
                lat: 52.5385,
                lon: 13.844,
            },
            4000,
            date,
        )
        .unwrap()
    }

    fn doc(objects: Vec<MapObject>) -> MapDocument {
        MapDocument {
            notes: "© GeoBasis-DE/LGB 2026\nfreemapper – https://github.com/HenryJobst/freemapper"
                .into(),
            georef: georef(),
            symbol_set: SymbolSet::issprom_2019().with_sheet_symbols(),
            parts: vec![MapPart {
                name: "default part".into(),
                objects,
            }],
            templates: Vec::new(),
        }
    }

    fn id(code: &str) -> u32 {
        SymbolSet::issprom_2019()
            .with_sheet_symbols()
            .id_for_code(code)
            .unwrap()
    }

    fn path(code: &str, coords: Vec<MapCoord>) -> MapObject {
        MapObject {
            symbol_id: id(code),
            kind: MapObjectKind::Path,
            coords,
            tags: Vec::new(),
        }
    }

    fn convert(objects: Vec<MapObject>) -> (ReadFile, OcdReport) {
        let (bytes, report) = map_to_ocd(&doc(objects), MapKind::Sprint, &[]).unwrap();
        (read(&bytes), report)
    }

    fn square(x: i32, y: i32, side: i32) -> Vec<(i32, i32)> {
        vec![(x, y), (x + side, y), (x + side, y + side), (x, y + side)]
    }

    /// OCAD's forward transformation (reference §6), independent of the converter.
    fn ocad_to_projected(file: &ReadFile, x: i32, y: i32) -> (f64, f64) {
        let scale_par = &file.strings.iter().find(|s| s.kind == 1039).unwrap().text;
        let field = |code: char| -> f64 {
            scale_par
                .split('\t')
                .find_map(|f| f.strip_prefix(code))
                .unwrap()
                .parse()
                .unwrap()
        };
        let (x0, y0, a, s) = (
            field('x'),
            field('y'),
            field('a').to_radians(),
            field('m') / 1000.0,
        );
        let (xm, ym) = (f64::from(x) / 100.0, f64::from(y) / 100.0);
        (
            x0 + s * (xm * a.cos() + ym * a.sin()),
            y0 + s * (-xm * a.sin() + ym * a.cos()),
        )
    }

    #[test]
    fn coordinates_reproject_exactly() {
        let points = [(0, 0), (100_000, 0), (0, 150_000), (-120_000, -90_000)];
        let objects = points
            .iter()
            .map(|&(x, y)| MapObject {
                symbol_id: id("418"),
                kind: MapObjectKind::Point { rotation_rad: 0.0 },
                coords: vec![MapCoord::new(x, y)],
                tags: Vec::new(),
            })
            .collect();
        let (file, report) = convert(objects);
        assert_eq!(report.objects, 4);
        let g = georef();
        for (o, &(x, y)) in file.objects.iter().zip(&points) {
            let want = g.map_to_projected(MapCoord::new(x, y));
            let (e, n) = ocad_to_projected(&file, o.coords[0].0, o.coords[0].1);
            // 0.01 mm at 1:4000 is 0.04 mm in the terrain; allow rounding.
            assert!(
                (e - want.easting).abs() < 0.01 && (n - want.northing).abs() < 0.01,
                "{x},{y}: {e},{n} vs {want:?}"
            );
        }
        let scale_par = &file.strings.iter().find(|s| s.kind == 1039).unwrap().text;
        assert!(
            scale_par.starts_with("\tm4000\tg25.0000\tr1\tx"),
            "{scale_par}"
        );
        assert!(scale_par.contains("\ti8008\t"), "{scale_par}");
    }

    #[test]
    fn ocad_grid_code_follows_the_zone() {
        assert_eq!(ocad_grid_code(crate::crs::Crs::Utm33), 8008);
        assert_eq!(ocad_grid_code(crate::crs::Crs::Utm32), 8007);
    }

    #[test]
    fn area_holes_are_flagged() {
        let mut coords = closed_ring(&square(0, 0, 10_000));
        coords.extend(closed_ring(&square(2_000, 2_000, 3_000)));
        let (file, _) = convert(vec![path("521", coords)]);
        let o = &file.objects[0];
        assert_eq!(o.otp, 3);
        assert_eq!(o.symbol, 521_000);
        assert_eq!(o.coords.len(), 10);
        let holes: Vec<usize> = o
            .coords
            .iter()
            .enumerate()
            .filter(|(_, c)| c.3 & Y_HOLE != 0)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(holes, [5]);
    }

    #[test]
    fn multipart_lines_are_split() {
        let mut coords = vec![MapCoord::new(0, 0), MapCoord::new(5_000, 0)];
        coords[1].flags = FLAG_HOLE;
        coords.extend([MapCoord::new(0, 2_000), MapCoord::new(5_000, 2_000)]);
        let (file, report) = convert(vec![path("505.1", coords)]);
        assert_eq!(report.objects, 2);
        assert!(
            file.objects
                .iter()
                .all(|o| o.otp == 2 && o.coords.len() == 2)
        );
        assert_eq!(file.objects[0].symbol, 505_000);
    }

    #[test]
    fn degenerate_objects_are_skipped() {
        let far = MapObject {
            symbol_id: id("418"),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![MapCoord::new(3_000_000, 0)],
            tags: Vec::new(),
        };
        let one_point_line = path("505.1", vec![MapCoord::new(0, 0), MapCoord::new(0, 0)]);
        let flat_area = path("521", closed_ring(&[(0, 0), (1_000, 0), (2_000, 0)]));
        let (file, report) = convert(vec![far, one_point_line, flat_area]);
        assert_eq!(
            report,
            OcdReport {
                objects: 0,
                skipped: 3,
                dropped_parts: 0
            }
        );
        assert!(file.objects.is_empty());
    }

    #[test]
    fn dropped_parts_are_counted() {
        let mut line = vec![MapCoord::new(0, 0), MapCoord::new(5_000, 0)];
        line[1].flags = FLAG_HOLE;
        line.extend([MapCoord::new(0, 2_000), MapCoord::new(0, 2_000)]);
        let mut area = closed_ring(&square(0, 0, 10_000));
        area.extend(closed_ring(&[
            (1_000, 1_000),
            (2_000, 1_000),
            (3_000, 1_000),
        ]));
        let (file, report) = convert(vec![path("505.1", line), path("521", area)]);
        assert_eq!(report.objects, 2);
        assert_eq!(report.dropped_parts, 2);
        assert_eq!(file.objects[1].coords.len(), 5, "only the outer ring");
    }

    #[test]
    fn text_reaching_beyond_the_drawing_area_is_skipped() {
        let text = MapObject {
            symbol_id: id("993"),
            kind: MapObjectKind::Text {
                text: "a long line of text near the edge of the sheet".into(),
                h_align: 0,
                v_align: 0,
            },
            coords: vec![MapCoord::new(1_985_000, 0)],
            tags: Vec::new(),
        };
        let (file, report) = convert(vec![text]);
        assert_eq!(report.skipped, 1);
        assert!(file.objects.is_empty());
        assert!(file.symbols.is_empty(), "no symbol without objects");
    }

    #[test]
    fn incomplete_curves_get_no_control_points() {
        let mut coords = vec![
            MapCoord::new(0, 0),
            MapCoord::new(1_000, 1_000),
            MapCoord::new(2_000, 0),
        ];
        coords[0].flags = FLAG_CURVE_START;
        let (file, _) = convert(vec![path("505.1", coords)]);
        assert!(file.objects[0].coords.iter().all(|c| c.2 == 0));
    }

    #[test]
    fn curves_mark_control_points() {
        let mut coords = vec![
            MapCoord::new(0, 0),
            MapCoord::new(1_000, 1_000),
            MapCoord::new(2_000, 1_000),
            MapCoord::new(3_000, 0),
        ];
        coords[0].flags = FLAG_CURVE_START;
        let (file, _) = convert(vec![path("505.1", coords)]);
        let flags: Vec<u8> = file.objects[0].coords.iter().map(|c| c.2).collect();
        assert_eq!(flags, [0, X_CURVE_1, X_CURVE_2, 0]);
    }

    #[test]
    fn texts_get_a_symbol_per_alignment_and_a_box() {
        let text = |h_align: u8| MapObject {
            symbol_id: id("993"),
            kind: MapObjectKind::Text {
                text: "Maßstab 1:4000".into(),
                h_align,
                v_align: 0,
            },
            coords: vec![MapCoord::new(0, 50_000)],
            tags: Vec::new(),
        };
        let (file, _) = convert(vec![text(0), text(1)]);
        let numbers: Vec<i32> = file.symbols.iter().map(|s| s.number).collect();
        assert_eq!(numbers, [993_000, 993_001]);
        let left = &file.objects[0];
        assert_eq!(left.text, "Maßstab 1:4000");
        assert_eq!(left.coords.len(), 5);
        let (anchor, bl, tr) = (left.coords[0], left.coords[1], left.coords[3]);
        assert_eq!(bl.0, anchor.0, "left-aligned box starts at the anchor");
        assert!(bl.1 < anchor.1 && tr.1 > anchor.1 && tr.0 > anchor.0);
        let centred = &file.objects[1];
        assert_eq!(centred.symbol, 993_001);
        let mid = (centred.coords[1].0 + centred.coords[2].0) / 2;
        assert!((mid - centred.coords[0].0).abs() <= 1);
    }

    #[test]
    fn template_position_from_world_file() {
        let g = georef();
        let (e, n) = (g.ref_projected.easting, g.ref_projected.northing);
        // A 1000 × 500 px north-up image, 0.2 m pixels, centred on the ref point.
        let world = WorldFile {
            a: 0.2,
            d: 0.0,
            b: 0.0,
            e: -0.2,
            c: e - 0.2 * 499.5,
            f: n + 0.2 * 249.5,
        };
        let background = Background::Image {
            file_name: "orthophoto.jpg".into(),
            width_px: 1000,
            height_px: 500,
            world,
            visible: true,
            opacity: 0.6,
        };
        let (bytes, _) = map_to_ocd(&doc(Vec::new()), MapKind::Sprint, &[background]).unwrap();
        let file = read(&bytes);
        let string = &file.strings.iter().find(|s| s.kind == 8).unwrap().text;
        let field = |code: char| -> f64 {
            string
                .split('\t')
                .skip(1)
                .find_map(|f| f.strip_prefix(code))
                .unwrap()
                .parse()
                .unwrap()
        };
        assert!(string.starts_with("orthophoto.jpg\ts1\tr1\t"), "{string}");
        // 0.2 m per pixel at 1:4000 is 0.05 mm on paper.
        assert!((field('u') - 0.05).abs() < 1e-9 && (field('v') - 0.05).abs() < 1e-9);
        // The ref point lies within 0.5 m of the whole-metre origin: within 0.125 mm on paper.
        assert!(field('x').abs() < 0.2 && field('y').abs() < 0.2, "{string}");
        assert!((field('a') - g.grivation_deg()).abs() < 1e-6);
        assert_eq!(field('d'), 40.0);
    }

    #[test]
    fn notes_and_colours_are_ascii() {
        let (file, _) = convert(Vec::new());
        let notes = &file.strings.iter().find(|s| s.kind == 1061).unwrap().text;
        assert_eq!(
            notes,
            "(c) GeoBasis-DE/LGB 2026\r\nfreemapper - https://github.com/HenryJobst/freemapper"
        );
        let colours = file.strings.iter().filter(|s| s.kind == 9).count();
        assert_eq!(colours, 32);
        assert!(file.strings.iter().all(|s| s.text.is_ascii()));
    }
}
